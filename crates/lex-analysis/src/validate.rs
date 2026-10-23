//! Deterministic corpus validation over supplied records.

use std::collections::{HashMap, HashSet};

use lex_core::{
    Basis, DefinedTerm, LRITF_INSTRUMENT_ID, Provision, ProvisionType, ReferenceEdge,
    ReferenceForm, ReferenceResolutionStatus, ResolutionRule, SCHEMA_VERSION, Severity, TermUsage,
    ValidationIssue, ValidationReport,
};
use regex::Regex;

use crate::initial::initial_temporal_status;
use lex_grammar::char_slice;
use lex_grammar::labels;

/// Partial-repeal and conditional-commencement records must be present
/// exactly where the status needs them, and a condition may only be marked met
/// with a date and evidence.
pub(crate) fn validate_repeals_and_condition(
    provision: &Provision,
    issues: &mut Vec<ValidationIssue>,
) {
    use lex_core::{ConditionStatus, TemporalStatus};
    let id = || Some(provision.id.clone());
    let status = &provision.temporal_status;
    if *status == TemporalStatus::PartiallyRepealed && provision.repeals.is_empty() {
        issues.push(error(
            "partial_repeal_without_scope",
            "partially_repealed provision has no repeals entry".to_owned(),
            id(),
        ));
    }
    if !provision.repeals.is_empty()
        && !matches!(
            status,
            TemporalStatus::PartiallyRepealed | TemporalStatus::Repealed
        )
    {
        issues.push(error(
            "repeals_on_unrepealed_provision",
            format!("repeals recorded on a {status:?} provision"),
            id(),
        ));
    }
    if provision.repeals.iter().any(|repeal| {
        repeal.ordinals.is_empty() || repeal.ordinals.iter().any(|o| o.trim().is_empty())
    }) {
        issues.push(error(
            "repeal_without_ordinals",
            "a repeals entry names no paragraph or fraction".to_owned(),
            id(),
        ));
    }
    match (&provision.commencement_condition, status) {
        (None, TemporalStatus::ConditionalPending) => issues.push(error(
            "conditional_without_condition",
            "conditional_pending provision has no commencement_condition".to_owned(),
            id(),
        )),
        (Some(condition), _) => {
            let met = condition.condition_status == ConditionStatus::Met;
            let in_force = matches!(
                status,
                TemporalStatus::Effective | TemporalStatus::FutureEffective
            );
            if *status == TemporalStatus::ConditionalPending && met {
                issues.push(error(
                    "met_condition_still_pending",
                    "a met condition must move the provision out of conditional_pending".to_owned(),
                    id(),
                ));
            } else if *status != TemporalStatus::ConditionalPending && !(met && in_force) {
                issues.push(error(
                    "condition_on_wrong_status",
                    format!("commencement_condition on a {status:?} provision"),
                    id(),
                ));
            }
            if condition.condition_text.trim().is_empty()
                || condition.source_provision_id.trim().is_empty()
            {
                issues.push(error(
                    "condition_incomplete",
                    "commencement_condition needs its source provision and wording".to_owned(),
                    id(),
                ));
            }
            let has_evidence = condition
                .evidence
                .as_deref()
                .is_some_and(|evidence| !evidence.trim().is_empty());
            if met && (condition.met_on.is_none() || !has_evidence) {
                issues.push(error(
                    "met_condition_without_evidence",
                    "a met condition needs the date it was met and its evidence".to_owned(),
                    id(),
                ));
            }
            if !met && condition.met_on.is_some() {
                issues.push(error(
                    "unmet_condition_with_date",
                    "only a met condition may carry a met_on date".to_owned(),
                    id(),
                ));
            }
        }
        (None, _) => {}
    }
}

#[derive(Debug, Clone, Default)]
pub struct CorpusExpectations {
    /// Minimum article count; `None` means the count baseline has not
    /// been frozen yet, which downgrades the count gate to a warning.
    pub min_articles: Option<usize>,
    /// Exact article count for closed instruments.
    pub articles: Option<usize>,
    /// Exact transitory count; `None` while the baseline is unfrozen.
    pub transitories: Option<usize>,
    pub annexes: usize,
    /// Require every article to carry chapter heading context.
    pub require_chapter_context: bool,
    /// Accept gaps in article numbering (derogated articles) and order
    /// suffixed articles by the label sort key instead of requiring a
    /// strict integer sequence from 1. Gaps become structured warnings;
    /// the frozen count baseline is the drift gate.
    pub allow_article_gaps: bool,
}

#[must_use]
pub fn validate_lritf(
    provisions: &[Provision],
    references: &[ReferenceEdge],
    expected_min_articles: usize,
    expected_transitories: usize,
) -> ValidationReport {
    validate_corpus(
        &CorpusView {
            instrument_id: LRITF_INSTRUMENT_ID,
            official_title: None,
            provisions,
            references,
            terms: &[],
            term_usages: &[],
            amendment_references: &[],
        },
        &CorpusExpectations {
            min_articles: Some(expected_min_articles),
            articles: None,
            transitories: Some(expected_transitories),
            annexes: 0,
            require_chapter_context: false,
            allow_article_gaps: false,
        },
        &HashSet::new(),
        &HashSet::new(),
    )
}

/// Validate one instrument's canonical corpus. `official_title` anchors
/// reference edges whose source is the instrument itself (title citations).
/// `external_targets` holds canonical provision identifiers of every other
/// loaded instrument, so cross-instrument edges can be checked for existing
/// targets.
/// Borrowed view of one instrument's canonical corpus for validation.
pub struct CorpusView<'a> {
    pub instrument_id: &'a str,
    /// Anchors reference edges whose source is the instrument itself
    /// (title citations).
    pub official_title: Option<&'a str>,
    pub provisions: &'a [Provision],
    pub references: &'a [ReferenceEdge],
    pub terms: &'a [DefinedTerm],
    pub term_usages: &'a [TermUsage],
    /// The compiled document's REFERENCIAS legend; empty for instruments
    /// without margin amendment markers.
    pub amendment_references: &'a [lex_core::AmendmentReference],
}

#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn validate_corpus(
    view: &CorpusView<'_>,
    expectations: &CorpusExpectations,
    external_targets: &HashSet<String>,
    external_terms: &HashSet<String>,
) -> ValidationReport {
    let CorpusView {
        instrument_id,
        official_title,
        provisions,
        references,
        terms,
        term_usages,
        amendment_references,
    } = *view;
    let article_count = provisions
        .iter()
        .filter(|item| item.provision_type == ProvisionType::Article)
        .count();
    let transitory_count = provisions
        .iter()
        .filter(|item| item.provision_type == ProvisionType::Transitory)
        .count();
    let annex_count = provisions
        .iter()
        .filter(|item| item.provision_type == ProvisionType::Annex)
        .count();
    let mut issues = Vec::new();

    validate_counts(
        article_count,
        transitory_count,
        annex_count,
        expectations,
        &mut issues,
    );
    validate_provisions(provisions, expectations, &mut issues);
    validate_amendment_marks(provisions, amendment_references, &mut issues);
    validate_terms(provisions, terms, term_usages, external_terms, &mut issues);
    validate_references(
        instrument_id,
        official_title,
        provisions,
        references,
        external_targets,
        &mut issues,
    );

    ValidationReport {
        schema_version: SCHEMA_VERSION.to_owned(),
        instrument_id: instrument_id.to_owned(),
        valid: !issues.iter().any(|item| item.severity == Severity::Error),
        article_count,
        transitory_count,
        reference_count: references.len(),
        issues,
    }
}

/// Every provision amendment mark must resolve through the instrument's
/// legend, and legend marker numbers must be unique — a dangling mark
/// means the parser attached a number the REFERENCIAS section never
/// defined.
fn validate_amendment_marks(
    provisions: &[Provision],
    amendment_references: &[lex_core::AmendmentReference],
    issues: &mut Vec<ValidationIssue>,
) {
    let mut legend = HashSet::new();
    for reference in amendment_references {
        if !legend.insert(reference.marker) {
            issues.push(error(
                "duplicate_amendment_marker",
                format!("legend defines marker {} more than once", reference.marker),
                None,
            ));
        }
    }
    for provision in provisions {
        for mark in &provision.amendment_marks {
            if !legend.contains(mark) {
                issues.push(error(
                    "unknown_amendment_mark",
                    format!("amendment mark ({mark}) is not defined by the legend"),
                    Some(provision.id.clone()),
                ));
            }
        }
    }
}

fn validate_counts(
    article_count: usize,
    transitory_count: usize,
    annex_count: usize,
    expectations: &CorpusExpectations,
    issues: &mut Vec<ValidationIssue>,
) {
    match expectations.min_articles {
        Some(minimum) if article_count < minimum => issues.push(error(
            "article_count",
            format!("expected at least {minimum} articles, found {article_count}"),
            None,
        )),
        _ => {}
    }
    // The article-count baseline is frozen once either a minimum or an
    // exact count is set; only the total absence of both means no parse
    // has ever been reviewed and accepted for this instrument.
    if expectations.min_articles.is_none() && expectations.articles.is_none() {
        issues.push(ValidationIssue {
            severity: Severity::Warning,
            code: "counts_not_frozen".to_owned(),
            message: format!(
                "no article-count baseline frozen; parse found {article_count} articles"
            ),
            provision_id: None,
        });
    }
    if let Some(expected) = expectations.articles
        && article_count != expected
    {
        issues.push(error(
            "article_count_exact",
            format!("expected exactly {expected} articles, found {article_count}"),
            None,
        ));
    }
    match expectations.transitories {
        Some(expected) if transitory_count != expected => issues.push(error(
            "transitory_count",
            format!("expected {expected} transitories, found {transitory_count}"),
            None,
        )),
        Some(_) => {}
        None => issues.push(ValidationIssue {
            severity: Severity::Warning,
            code: "counts_not_frozen".to_owned(),
            message: format!(
                "no transitory-count baseline frozen; parse found {transitory_count} transitories"
            ),
            provision_id: None,
        }),
    }
    if annex_count != expectations.annexes {
        issues.push(error(
            "annex_count",
            format!(
                "expected {} annexes, found {annex_count}",
                expectations.annexes
            ),
            None,
        ));
    }
}

fn validate_provisions(
    provisions: &[Provision],
    expectations: &CorpusExpectations,
    issues: &mut Vec<ValidationIssue>,
) {
    let mut ids = HashSet::new();
    let mut expected_number = 1_u32;
    let mut expected_annex = 1_u32;
    let mut previous_article: Option<(labels::ArticleSortKey, String)> = None;
    for provision in provisions {
        if !ids.insert(&provision.id) {
            issues.push(error(
                "duplicate_id",
                "duplicate canonical identifier".to_owned(),
                Some(provision.id.clone()),
            ));
        }
        if provision.text.trim().is_empty() {
            issues.push(error(
                "empty_text",
                "provision has no body text".to_owned(),
                Some(provision.id.clone()),
            ));
        }
        if contains_page_header_contamination(&provision.text) {
            issues.push(error(
                "header_contamination",
                "page header leaked into provision text".to_owned(),
                Some(provision.id.clone()),
            ));
        }
        if provision.review_status == lex_core::ReviewStatus::NotAnalyzed {
            let expected = initial_temporal_status(&provision.text);
            if provision.temporal_status != expected {
                issues.push(error(
                    "unanalyzed_temporal_status",
                    format!(
                        "not_analyzed provision must start {expected:?}, found {:?}",
                        provision.temporal_status
                    ),
                    Some(provision.id.clone()),
                ));
            }
        }
        validate_repeals_and_condition(provision, issues);
        match provision.provision_type {
            ProvisionType::Article => {
                if expectations.allow_article_gaps {
                    validate_article_order_by_label(provision, &mut previous_article, true, issues);
                } else {
                    match plain_article_number(&provision.number) {
                        Some(number) if number == expected_number => {
                            expected_number += 1;
                            // Order the plain article against the preceding
                            // label (a suffix of a later base must not
                            // precede it) and track it for the next one.
                            if labels::match_label_at(&provision.number)
                                .is_some_and(|label| label.raw() == provision.number)
                            {
                                validate_article_order_by_label(
                                    provision,
                                    &mut previous_article,
                                    false,
                                    issues,
                                );
                            } else {
                                previous_article = None;
                            }
                        }
                        Some(number) => issues.push(error(
                            "article_order",
                            format!("expected article {expected_number}, found {number}"),
                            Some(provision.id.clone()),
                        )),
                        // A suffixed article is ordered by its sort key whether
                        // or not gaps are allowed; its base may repeat the
                        // previous plain article, never run backwards.
                        None => validate_article_order_by_label(
                            provision,
                            &mut previous_article,
                            false,
                            issues,
                        ),
                    }
                }
                if expectations.require_chapter_context
                    && provision.heading_context.chapter.is_none()
                {
                    issues.push(error(
                        "missing_chapter_context",
                        "article lacks chapter heading context".to_owned(),
                        Some(provision.id.clone()),
                    ));
                }
            }
            ProvisionType::Annex => match provision.number.parse::<u32>() {
                Ok(number) if number == expected_annex => expected_annex += 1,
                Ok(number) => issues.push(error(
                    "annex_order",
                    format!("expected annex {expected_annex}, found {number}"),
                    Some(provision.id.clone()),
                )),
                Err(_) => issues.push(error(
                    "non_numeric_annex",
                    "annex number is not numeric".to_owned(),
                    Some(provision.id.clone()),
                )),
            },
            ProvisionType::Transitory => {}
        }
    }
}

pub(crate) fn contains_page_header_contamination(text: &str) -> bool {
    text.lines().map(str::trim).any(|line| {
        line.starts_with("CÁMARA DE DIPUTADOS DEL H. CONGRESO DE LA UNIÓN")
            || line == "Secretaría de Servicios Parlamentarios"
    })
}

/// Gap-tolerant article ordering for códigos. Every article number must
/// parse under the shared label grammar. Base numbers must not go
/// backwards (a real regression is an error); a skipped base number
/// (derogated article or parse gap) is an `article_gap` warning. The exact
/// ordering of same-base suffixes — a código may print `70-A` before
/// `70 Bis`, another the reverse — is drafting-dependent, so a same-base
/// key that does not increase is a `suffix_order` warning, not an error.
/// The frozen count baseline is the drift gate.
fn validate_article_order_by_label(
    provision: &Provision,
    previous: &mut Option<(labels::ArticleSortKey, String)>,
    report_gaps: bool,
    issues: &mut Vec<ValidationIssue>,
) {
    let parsed =
        labels::match_label_at(&provision.number).filter(|label| label.raw() == provision.number);
    let Some(label) = parsed else {
        issues.push(error(
            "unparseable_article_number",
            format!(
                "article number {:?} does not parse as a label",
                provision.number
            ),
            Some(provision.id.clone()),
        ));
        return;
    };
    let key = label.sort_key();
    let base = key.first().and_then(|part| part.0.first().copied());
    if let Some((previous_key, previous_number)) = previous {
        let previous_base = previous_key
            .first()
            .and_then(|part| part.0.first().copied());
        match (base, previous_base) {
            (Some(base), Some(previous_base)) if base < previous_base => issues.push(error(
                "article_order",
                format!(
                    "article {} sorts before article {previous_number}",
                    provision.number
                ),
                Some(provision.id.clone()),
            )),
            // Strict ordering already errors on a missing plain number.
            (Some(base), Some(previous_base)) if !report_gaps && base > previous_base + 1 => {}
            (Some(base), Some(previous_base)) if base > previous_base + 1 => {
                issues.push(ValidationIssue {
                    severity: Severity::Warning,
                    code: "article_gap".to_owned(),
                    message: format!(
                        "articles between {previous_number} and {} are absent (derogated, or a parse gap)",
                        provision.number
                    ),
                    provision_id: Some(provision.id.clone()),
                });
            }
            // Same base: only the intra-family suffix order is in question.
            _ if key <= *previous_key => issues.push(ValidationIssue {
                severity: Severity::Warning,
                code: "suffix_order".to_owned(),
                message: format!(
                    "article {} does not sort after {previous_number} within the same base",
                    provision.number
                ),
                provision_id: Some(provision.id.clone()),
            }),
            _ => {}
        }
    }
    *previous = Some((key, provision.number.clone()));
}

static TERM_SLUG: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$").expect("static regex"));

/// A defined term's identifier is its instrument's, `:term:`, and a slug of
/// lower-case letters and digits joined by single hyphens.
fn check_term_id(term: &DefinedTerm, issues: &mut Vec<ValidationIssue>) {
    let slug = term
        .id
        .strip_prefix(&term.instrument_id)
        .and_then(|rest| rest.strip_prefix(":term:"));
    if !slug.is_some_and(|slug| TERM_SLUG.is_match(slug)) {
        issues.push(error(
            "term_id_invalid",
            format!(
                "defined-term identifier is not `<instrument>:term:` and a lower-case slug: {}",
                term.id
            ),
            Some(term.defining_provision_id.clone()),
        ));
    }
}

/// Validate defined terms and their usages: unique identifiers, existing
/// defining provisions, definition spans that contain the term, exact usage
/// spans, resolvable usage targets (own terms or another loaded
/// instrument's), and non-overlapping usages within a provision.
fn validate_terms(
    provisions: &[Provision],
    terms: &[DefinedTerm],
    term_usages: &[TermUsage],
    external_terms: &HashSet<String>,
    issues: &mut Vec<ValidationIssue>,
) {
    let provisions_by_id: HashMap<&str, &Provision> = provisions
        .iter()
        .map(|provision| (provision.id.as_str(), provision))
        .collect();
    let mut term_ids = HashSet::new();
    for term in terms {
        check_term_id(term, issues);
        if !term_ids.insert(term.id.as_str()) {
            issues.push(error(
                "duplicate_term_id",
                format!("duplicate defined-term identifier: {}", term.id),
                Some(term.defining_provision_id.clone()),
            ));
        }
        let Some(defining) = provisions_by_id.get(term.defining_provision_id.as_str()) else {
            issues.push(error(
                "term_provision_missing",
                format!(
                    "defining provision does not exist: {}",
                    term.defining_provision_id
                ),
                Some(term.defining_provision_id.clone()),
            ));
            continue;
        };
        match char_slice(&defining.text, term.start_char, term.end_char) {
            Some(slice) if slice.contains(&term.term) => {}
            Some(_) => issues.push(error(
                "term_span_mismatch",
                format!("definition span does not contain the term {:?}", term.term),
                Some(term.defining_provision_id.clone()),
            )),
            None => issues.push(error(
                "term_offsets_invalid",
                format!(
                    "definition offsets {}..{} are outside the provision text",
                    term.start_char, term.end_char
                ),
                Some(term.defining_provision_id.clone()),
            )),
        }
    }

    let mut last_end: HashMap<&str, usize> = HashMap::new();
    for usage in term_usages {
        let Some(provision) = provisions_by_id.get(usage.provision_id.as_str()) else {
            issues.push(error(
                "term_usage_provision_missing",
                format!("usage provision does not exist: {}", usage.provision_id),
                Some(usage.provision_id.clone()),
            ));
            continue;
        };
        match char_slice(&provision.text, usage.start_char, usage.end_char) {
            Some(slice) if slice == usage.span => {}
            _ => issues.push(error(
                "term_usage_span_mismatch",
                format!(
                    "usage span {:?} does not match the provision text at {}..{}",
                    usage.span, usage.start_char, usage.end_char
                ),
                Some(usage.provision_id.clone()),
            )),
        }
        if !term_ids.contains(usage.term_id.as_str())
            && !external_terms.contains(usage.term_id.as_str())
        {
            issues.push(error(
                "term_usage_target_missing",
                format!("usage resolves to an unknown term: {}", usage.term_id),
                Some(usage.provision_id.clone()),
            ));
        }
        let cursor = last_end.entry(usage.provision_id.as_str()).or_insert(0);
        if usage.start_char < *cursor {
            issues.push(error(
                "term_usage_overlap",
                format!(
                    "usages overlap at {}..{} in {}",
                    usage.start_char, usage.end_char, usage.provision_id
                ),
                Some(usage.provision_id.clone()),
            ));
        }
        *cursor = usage.end_char.max(*cursor);
    }
}

/// Validate every supplied reference edge of one instrument against its
/// provisions and the known-target set, appending issues. Public so a
/// cross-stage test can drive it with edges produced by the linker.
pub fn validate_references<S: std::hash::BuildHasher>(
    instrument_id: &str,
    official_title: Option<&str>,
    provisions: &[Provision],
    references: &[ReferenceEdge],
    external_targets: &HashSet<String, S>,
    issues: &mut Vec<ValidationIssue>,
) {
    let provisions_by_id: HashMap<_, _> = provisions
        .iter()
        .map(|provision| (provision.id.as_str(), provision))
        .collect();
    let mut reference_ids = HashSet::new();
    for reference in references {
        if !reference_ids.insert(&reference.id) {
            issues.push(error(
                "duplicate_reference_id",
                "duplicate canonical reference identifier".to_owned(),
                Some(reference.source_provision_id.clone()),
            ));
        }
        validate_reference(
            instrument_id,
            official_title,
            reference,
            &provisions_by_id,
            external_targets,
            issues,
        );
    }
}

/// Validate one reference edge: span, target, qualifiers and resolution rule.
pub fn validate_reference<S: std::hash::BuildHasher, T: std::hash::BuildHasher>(
    instrument_id: &str,
    official_title: Option<&str>,
    reference: &ReferenceEdge,
    provisions_by_id: &HashMap<&str, &Provision, S>,
    external_targets: &HashSet<String, T>,
    issues: &mut Vec<ValidationIssue>,
) {
    let source_text = if reference.source_provision_id == instrument_id {
        official_title
    } else {
        provisions_by_id
            .get(reference.source_provision_id.as_str())
            .map(|source| source.text.as_str())
    };
    let Some(source_text) = source_text else {
        issues.push(error(
            "reference_source_missing",
            format!(
                "reference source does not exist: {}",
                reference.source_provision_id
            ),
            Some(reference.source_provision_id.clone()),
        ));
        return;
    };
    validate_reference_span(reference, source_text, issues);
    let approved = reference.resolution_status
        == ReferenceResolutionStatus::HistoricalSupersededReferenceOutdated;
    if approved != reference.outdated_reference_approval.is_some()
        || reference
            .outdated_reference_approval
            .as_ref()
            .is_some_and(|a| !a.matches(reference, source_text))
    {
        issues.push(error(
            "outdated_reference_approval_invalid",
            "outdated reference requires an exact valid source-bound approval".to_owned(),
            Some(reference.source_provision_id.clone()),
        ));
    }
    if reference
        .legal_operator_review
        .as_ref()
        .is_some_and(|review| !review.matches(reference, source_text))
    {
        issues.push(error(
            "legal_operator_review_invalid",
            "legal operator review is not bound to this exact edge and source text".to_owned(),
            Some(reference.source_provision_id.clone()),
        ));
    }
    if let Some(rule) = reference.resolution_rule
        && !resolution_rule_consistent(instrument_id, reference, rule)
    {
        issues.push(error(
            "resolution_rule_inconsistent",
            format!("resolution rule {rule:?} does not fit this edge"),
            Some(reference.source_provision_id.clone()),
        ));
    }
    validate_reference_target(
        instrument_id,
        reference,
        provisions_by_id,
        external_targets,
        issues,
    );
    if reference.basis != Basis::ExpressCrossReference {
        issues.push(error(
            "reference_basis",
            "canonical reference must use express_cross_reference basis".to_owned(),
            Some(reference.source_provision_id.clone()),
        ));
    }
    if !(0.0..=1.0).contains(&reference.confidence) {
        issues.push(error(
            "reference_confidence",
            "reference confidence must be between zero and one".to_owned(),
            Some(reference.source_provision_id.clone()),
        ));
    }
}

/// The rule must fit the edge: own-instrument, same-article and relative
/// rules stay inside the citing instrument; only a relative rule carries the
/// relative form; a same-article fraction targets its own provision.
fn resolution_rule_consistent(
    instrument_id: &str,
    reference: &ReferenceEdge,
    rule: ResolutionRule,
) -> bool {
    let own_target = reference.target_instrument_id == instrument_id;
    let relative_form = reference.reference_form == ReferenceForm::Relative;
    (rule == ResolutionRule::RelativePosition) == relative_form
        && match rule {
            ResolutionRule::OwnInstrumentDefault
            | ResolutionRule::OwnInstrumentMarker
            | ResolutionRule::RelativePosition => own_target,
            ResolutionRule::SameArticle => {
                own_target && reference.target_provision_id == reference.source_provision_id
            }
            ResolutionRule::NamedInstrument | ResolutionRule::ParentShorthand => true,
        }
}

fn validate_reference_span(
    reference: &ReferenceEdge,
    source_text: &str,
    issues: &mut Vec<ValidationIssue>,
) {
    for qualifier in &reference.qualifiers {
        match (qualifier.start_char, qualifier.end_char) {
            (Some(start), Some(end)) => match char_slice(source_text, start, end) {
                Some(span) if span == qualifier.text => {}
                _ => issues.push(error(
                    "qualifier_span_mismatch",
                    format!(
                        "qualifier span {:?} does not match source text at {start}..{end}",
                        qualifier.text
                    ),
                    Some(reference.source_provision_id.clone()),
                )),
            },
            (None, None) => {}
            _ => issues.push(error(
                "qualifier_offsets_incomplete",
                format!(
                    "qualifier {:?} has only one of start_char/end_char; offsets must come in \
                     pairs",
                    qualifier.text
                ),
                Some(reference.source_provision_id.clone()),
            )),
        }
    }
    match char_slice(source_text, reference.start_char, reference.end_char) {
        Some(span) if span == reference.source_span => {}
        Some(span) => issues.push(error(
            "reference_span_mismatch",
            format!(
                "reference span {:?} does not match source text {:?}",
                reference.source_span, span
            ),
            Some(reference.source_provision_id.clone()),
        )),
        None => issues.push(error(
            "reference_offsets_invalid",
            format!(
                "reference character offsets {}..{} are outside the source text",
                reference.start_char, reference.end_char
            ),
            Some(reference.source_provision_id.clone()),
        )),
    }
}

fn validate_reference_target<S: std::hash::BuildHasher, T: std::hash::BuildHasher>(
    instrument_id: &str,
    reference: &ReferenceEdge,
    provisions_by_id: &HashMap<&str, &Provision, S>,
    external_targets: &HashSet<String, T>,
    issues: &mut Vec<ValidationIssue>,
) {
    let canonical_owner = reference
        .target_provision_id
        .rsplit_once(":article:")
        .or_else(|| reference.target_provision_id.rsplit_once(":transitory:"))
        .or_else(|| reference.target_provision_id.rsplit_once(":annex:"))
        .map(|(owner, _)| owner);
    if canonical_owner != Some(reference.target_instrument_id.as_str()) {
        issues.push(error(
            "reference_target_owner_mismatch",
            "reference target identifier does not belong to its declared instrument".to_owned(),
            Some(reference.source_provision_id.clone()),
        ));
        return;
    }
    let cross_instrument = reference.target_instrument_id != instrument_id;
    let target_exists = if cross_instrument {
        external_targets.contains(reference.target_provision_id.as_str())
    } else {
        provisions_by_id.contains_key(reference.target_provision_id.as_str())
    };
    let scope = if cross_instrument {
        "cross-instrument"
    } else {
        "internal"
    };
    let (code, message) = match (&reference.resolution_status, target_exists) {
        (ReferenceResolutionStatus::Resolved, false) => (
            "resolved_reference_target_missing",
            format!(
                "resolved reference target does not exist: {}",
                reference.target_provision_id
            ),
        ),
        (ReferenceResolutionStatus::Unresolved, true) => (
            "reference_resolution_stale",
            format!(
                "reference target exists but is marked unresolved: {}",
                reference.target_provision_id
            ),
        ),
        (ReferenceResolutionStatus::Unresolved, false) => (
            "unresolved_internal_reference",
            format!(
                "{scope} reference target does not exist: {}",
                reference.target_provision_id
            ),
        ),
        (ReferenceResolutionStatus::HistoricalTargetUnavailable, false) => {
            issues.push(warning(
                "historical_reference_target_unavailable",
                format!(
                    "historical {scope} reference is preserved without a live target: {}",
                    reference.target_provision_id
                ),
                Some(reference.source_provision_id.clone()),
            ));
            return;
        }
        (ReferenceResolutionStatus::HistoricalTargetUnavailable, true) => (
            "historical_reference_target_now_available",
            format!(
                "historical reference target now exists and must be relinked: {}",
                reference.target_provision_id
            ),
        ),
        (ReferenceResolutionStatus::HistoricalSupersededReferenceOutdated, false) => {
            issues.push(warning(
                "historical_superseded_reference_outdated",
                format!(
                    "reviewed stale citation retained without a live link: {}",
                    reference.target_provision_id
                ),
                Some(reference.source_provision_id.clone()),
            ));
            return;
        }
        (ReferenceResolutionStatus::HistoricalSupersededReferenceOutdated, true) => (
            "outdated_reference_target_now_available",
            format!(
                "outdated citation target now exists and requires review: {}",
                reference.target_provision_id
            ),
        ),
        (ReferenceResolutionStatus::Resolved, true) => return,
    };
    issues.push(error(
        code,
        message,
        Some(reference.source_provision_id.clone()),
    ));
}

fn error(code: &str, message: String, provision_id: Option<String>) -> ValidationIssue {
    ValidationIssue {
        severity: Severity::Error,
        code: code.to_owned(),
        message,
        provision_id,
    }
}

fn warning(code: &str, message: String, provision_id: Option<String>) -> ValidationIssue {
    ValidationIssue {
        severity: Severity::Warning,
        code: code.to_owned(),
        message,
        provision_id,
    }
}

/// Numeric value of a plain article number, accepting the bare ordinal
/// marker official texts print on articles 1–9 (`1o`, `9o`). Suffixed or
/// otherwise non-numeric labels still return `None` and keep their warning.
pub(crate) fn plain_article_number(number: &str) -> Option<u32> {
    let digits = number
        .strip_suffix("/o")
        .or_else(|| number.strip_suffix(['o', 'º', '°']))
        .unwrap_or(number);
    if digits.contains(',') {
        let groups: Vec<_> = digits.split(',').collect();
        if groups[0].is_empty()
            || groups[0].len() > 3
            || groups.iter().skip(1).any(|g| g.len() != 3)
            || groups
                .iter()
                .any(|g| !g.bytes().all(|b| b.is_ascii_digit()))
        {
            return None;
        }
        return groups.concat().parse().ok();
    }
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

#[cfg(test)]
mod tests {

    use chrono::NaiveDate;
    use lex_core::{Provision, ProvisionType, TemporalStatus};
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::initial::initial_repeals;

    #[test]
    fn plain_article_number_reads_bare_ordinals_only() {
        assert_eq!(super::plain_article_number("7"), Some(7));
        assert_eq!(super::plain_article_number("9o"), Some(9));
        assert_eq!(super::plain_article_number("1°"), Some(1));
        assert_eq!(super::plain_article_number("2º"), Some(2));
        assert_eq!(super::plain_article_number("1/o"), Some(1));
        assert_eq!(super::plain_article_number("1,000"), Some(1000));
        assert_eq!(super::plain_article_number("1,00"), None);
        assert_eq!(super::plain_article_number("10"), Some(10));
        assert_eq!(super::plain_article_number("1o-A"), None);
        assert_eq!(super::plain_article_number("o"), None);
        assert_eq!(super::plain_article_number("12-Bis"), None);
    }

    fn repeal_fixture_provision(
        status: TemporalStatus,
        repeals: Vec<lex_core::ProvisionRepeal>,
        condition: Option<lex_core::CommencementCondition>,
    ) -> Provision {
        Provision {
            numbering_basis: None,
            schema_version: "0.1.0".to_owned(),
            id: "urn:lex-mx:federal:statute:muestra:article:1".to_owned(),
            instrument_id: "urn:lex-mx:federal:statute:muestra".to_owned(),
            provision_type: ProvisionType::Article,
            label: "Artículo 1".to_owned(),
            number: "1".to_owned(),
            heading_context: lex_core::HeadingContext {
                libro: None,
                title: None,
                chapter: None,
                section: None,
                apartado: None,
            },
            text: "Texto.".to_owned(),
            publication_date: Some(NaiveDate::from_ymd_opt(1990, 1, 1).unwrap()),
            publication_period: None,
            effective_from: None,
            effective_to: None,
            temporal_status: status,
            temporal_basis: None,
            temporal_confidence: None,
            review_status: lex_core::ReviewStatus::MachineAccepted,
            transitory_effects: Vec::new(),
            amendment_marks: Vec::new(),
            repeals,
            commencement_condition: condition,
        }
    }

    fn repeal_codes(provision: &Provision) -> Vec<String> {
        let mut issues = Vec::new();
        validate_repeals_and_condition(provision, &mut issues);
        issues.into_iter().map(|issue| issue.code).collect()
    }

    fn sample_repeal() -> lex_core::ProvisionRepeal {
        lex_core::ProvisionRepeal {
            scope: lex_core::RepealScope::Paragraph,
            ordinals: vec!["1".to_owned()],
            cause: lex_core::RepealCause::Legislative,
            dof_date: None,
            renumbering: false,
        }
    }

    fn sample_condition(
        status: lex_core::ConditionStatus,
        met_on: Option<NaiveDate>,
        evidence: Option<&str>,
    ) -> lex_core::CommencementCondition {
        lex_core::CommencementCondition {
            source_provision_id: "urn:lex-mx:federal:statute:muestra:transitory:tercero".to_owned(),
            condition_text: "treinta días después de que se emitan las disposiciones".to_owned(),
            authority: Some("CNBV".to_owned()),
            condition_status: status,
            met_on,
            evidence: evidence.map(str::to_owned),
        }
    }

    #[test]
    fn partial_repeal_notes_are_classified_only_in_the_unambiguous_shape() {
        let repeals = initial_repeals(
            "(Se deroga el primer párrafo). Párrafo derogado DOF 15-06-2007\n\nTexto vigente.",
        );
        assert_eq!(repeals.len(), 1);
        assert_eq!(repeals[0].scope, lex_core::RepealScope::Paragraph);
        assert_eq!(repeals[0].ordinals, ["1"]);
        assert_eq!(repeals[0].dof_date, NaiveDate::from_ymd_opt(2007, 6, 15));
        assert!(!repeals[0].renumbering);
        assert_eq!(
            initial_temporal_status(
                "(Se deroga primer párrafo). Párrafo derogado DOF 04-02-2004\n\nTexto."
            ),
            TemporalStatus::PartiallyRepealed
        );

        let listed = initial_repeals(
            "(Se derogan los párrafos 1, 2 y 4, recorriéndose los subsecuentes)\n\nTexto.",
        );
        assert!(
            listed.is_empty(),
            "prose inside the note is left for review"
        );

        let range = initial_repeals("(Se derogan los párrafos segundo a cuarto)\n\nTexto.");
        assert_eq!(range[0].ordinals, ["2", "3", "4"]);

        let fractions = initial_repeals("(Se derogan las fracciones III y V)\n\nTexto.");
        assert_eq!(fractions[0].scope, lex_core::RepealScope::Fraction);
        assert_eq!(fractions[0].ordinals, ["III", "V"]);

        for text in [
            "Se deroga el primer párrafo del artículo 5 de otra ley.",
            "(Se deroga el artículo 5 de la Ley Federal)",
            "(Se deroga)",
            "(Se deroga el texto del párrafo)",
        ] {
            assert!(initial_repeals(text).is_empty(), "{text:?}");
        }
    }

    #[test]
    fn partial_repeal_and_condition_records_are_required_where_the_status_needs_them() {
        use lex_core::ConditionStatus::{Met, Unmet, Unverified};
        let date = NaiveDate::from_ymd_opt(2024, 5, 1);

        let found = repeal_codes(&repeal_fixture_provision(
            TemporalStatus::PartiallyRepealed,
            vec![sample_repeal()],
            None,
        ));
        assert!(found.is_empty(), "{found:?}");
        assert_eq!(
            repeal_codes(&repeal_fixture_provision(
                TemporalStatus::PartiallyRepealed,
                vec![],
                None
            )),
            ["partial_repeal_without_scope"]
        );
        assert_eq!(
            repeal_codes(&repeal_fixture_provision(
                TemporalStatus::Effective,
                vec![sample_repeal()],
                None
            )),
            ["repeals_on_unrepealed_provision"]
        );

        let found = repeal_codes(&repeal_fixture_provision(
            TemporalStatus::ConditionalPending,
            vec![],
            Some(sample_condition(Unverified, None, None)),
        ));
        assert!(found.is_empty(), "{found:?}");
        let found = repeal_codes(&repeal_fixture_provision(
            TemporalStatus::ConditionalPending,
            vec![],
            Some(sample_condition(Unmet, None, None)),
        ));
        assert!(found.is_empty(), "{found:?}");
        assert_eq!(
            repeal_codes(&repeal_fixture_provision(
                TemporalStatus::ConditionalPending,
                vec![],
                None
            )),
            ["conditional_without_condition"]
        );
        let found = repeal_codes(&repeal_fixture_provision(
            TemporalStatus::Effective,
            vec![],
            Some(sample_condition(Met, date, Some("DOF 2024-05-01 acuerdo"))),
        ));
        assert!(found.is_empty(), "{found:?}");
        assert_eq!(
            repeal_codes(&repeal_fixture_provision(
                TemporalStatus::ConditionalPending,
                vec![],
                Some(sample_condition(Met, date, Some("DOF")))
            )),
            ["met_condition_still_pending"]
        );
        assert_eq!(
            repeal_codes(&repeal_fixture_provision(
                TemporalStatus::Effective,
                vec![],
                Some(sample_condition(Met, None, None))
            )),
            ["met_condition_without_evidence"]
        );
        assert_eq!(
            repeal_codes(&repeal_fixture_provision(
                TemporalStatus::Effective,
                vec![],
                Some(sample_condition(Unverified, None, None))
            )),
            ["condition_on_wrong_status"]
        );
        assert_eq!(
            repeal_codes(&repeal_fixture_provision(
                TemporalStatus::ConditionalPending,
                vec![],
                Some(sample_condition(Unverified, date, None))
            )),
            ["unmet_condition_with_date"]
        );
    }

    #[test]
    fn header_contamination_requires_standalone_page_furniture() {
        assert!(!contains_page_header_contamination(
            "La Secretaría de Servicios Parlamentarios presta apoyo a la Cámara."
        ));
        assert!(contains_page_header_contamination(
            "Texto anterior.\nSecretaría de Servicios Parlamentarios\nTexto posterior."
        ));
        assert!(contains_page_header_contamination(
            "CÁMARA DE DIPUTADOS DEL H. CONGRESO DE LA UNIÓN\nSecretaría General"
        ));
    }
}
