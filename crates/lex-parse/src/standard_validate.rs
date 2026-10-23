//! NOM/NMX standard validation over typed records and retained text.

use std::{collections::HashSet, sync::LazyLock};

use lex_core::{
    SCHEMA_VERSION, Severity, StandardClause, StandardKind, StandardMetadata, StandardStatus,
    StandardSupplement, StandardSupplementLegalCharacter, StandardTextBasis, StandardTransitory,
    StandardValidationReport, ValidationIssue,
};
use regex::Regex;

use crate::collapse_whitespace;

use crate::standard::{
    LINE_LEAD, StandardModificationTarget, clause_order, closing_signature_start,
    derive_supplement_legal_character, is_dof_page_furniture, parse_standard_modification_targets,
    real_transitorios_heading, strip_page_furniture,
};

/// Validate the standards-specific identity, lifecycle, review separation,
/// clause ordering, uniqueness, hashes, and exact source spans.
#[must_use]
pub fn validate_standard(
    metadata: &StandardMetadata,
    clauses: &[StandardClause],
    transitories: &[StandardTransitory],
    supplements: &[StandardSupplement],
    source_text: &str,
) -> StandardValidationReport {
    let mut issues = Vec::new();
    // Derived once here and shared: `apply_amendment_marks` already consumed
    // the same derivation during parsing, and re-deriving it per validator
    // would invite the two to drift.
    let targets = parse_standard_modification_targets(metadata, clauses);
    validate_metadata(metadata, &mut issues);
    validate_standard_sources(metadata, &targets, &mut issues);
    validate_clauses(metadata, clauses, source_text, &mut issues);
    validate_clause_coverage(clauses, source_text, &mut issues);
    validate_no_page_furniture(clauses, transitories, supplements, &mut issues);
    validate_transitories(metadata, transitories, source_text, &mut issues);
    validate_supplements(
        metadata,
        transitories,
        supplements,
        source_text,
        &mut issues,
    );
    validate_trailing_material(metadata, clauses, source_text, &mut issues);
    StandardValidationReport {
        schema_version: SCHEMA_VERSION.to_owned(),
        standard_id: metadata.id.clone(),
        valid: !issues.iter().any(|issue| issue.severity == Severity::Error),
        clause_count: clauses.len(),
        supplement_count: supplements.len(),
        issues,
    }
}

pub(crate) fn validate_metadata(metadata: &StandardMetadata, issues: &mut Vec<ValidationIssue>) {
    let prefix = match metadata.kind {
        StandardKind::Nom => "NOM-",
        StandardKind::Nmx => "NMX-",
    };
    if !metadata.designation.starts_with(prefix) {
        issues.push(error(
            "standard_designation",
            format!("{:?} designation must start {prefix}", metadata.kind),
            None,
        ));
    }
    if let Some(published) = metadata.published_designation.as_deref() {
        if !published.starts_with(prefix) {
            issues.push(error(
                "standard_published_designation",
                format!(
                    "{:?} published designation must start {prefix}",
                    metadata.kind
                ),
                None,
            ));
        }
        if published == metadata.designation {
            issues.push(error(
                "standard_published_designation",
                "published designation is only recorded when it differs from the current \
                 designation"
                    .to_owned(),
                None,
            ));
        } else {
            issues.push(warning(
                "standard_redesignated",
                format!(
                    "retained text was published as {published}; the registry now designates it \
                     {}",
                    metadata.designation
                ),
            ));
        }
    }
    if metadata.issuing_authorities.is_empty() {
        issues.push(error(
            "standard_issuing_authority",
            "at least one issuing authority is required".to_owned(),
            None,
        ));
    }
    if metadata.regulatory_domains.is_empty() {
        issues.push(error(
            "standard_regulatory_domain",
            "at least one regulatory domain is required".to_owned(),
            None,
        ));
    }
    if metadata.source_sha256.len() != 64 || metadata.extracted_text_sha256.len() != 64 {
        issues.push(error(
            "standard_source_hash",
            "source and extracted-text SHA-256 values must have 64 hex characters".to_owned(),
            None,
        ));
    }
    if metadata.status == StandardStatus::Replaced && metadata.replaced_by.is_empty() {
        issues.push(error(
            "standard_replacement",
            "a replaced standard must identify at least one replacement".to_owned(),
            None,
        ));
    }
    if metadata.status == StandardStatus::Cancelled && metadata.cancellation_date.is_none() {
        issues.push(error(
            "standard_cancellation_date",
            "a cancelled standard must record its cancellation date".to_owned(),
            None,
        ));
    }
    if metadata
        .effective_date
        .is_some_and(|date| date < metadata.publication_date)
    {
        issues.push(error(
            "standard_effective_date",
            "effective date cannot precede publication".to_owned(),
            None,
        ));
    }
    if metadata
        .cancellation_date
        .is_some_and(|date| date < metadata.publication_date)
    {
        issues.push(error(
            "standard_cancellation_date",
            "cancellation date cannot precede publication".to_owned(),
            None,
        ));
    }
}

fn validate_standard_sources(
    metadata: &StandardMetadata,
    targets: &[StandardModificationTarget],
    issues: &mut Vec<ValidationIssue>,
) {
    let included_modifications = metadata
        .modifications
        .iter()
        .filter(|source| source.included_in_source)
        .count();
    match metadata.text_basis {
        StandardTextBasis::AsPublished if included_modifications > 0 => {
            issues.push(error(
                "standard_text_basis",
                "an as-published source cannot include later modifications".to_owned(),
                None,
            ));
        }
        _ => {}
    }
    for (index, source) in metadata.modifications.iter().enumerate() {
        if source.publication_date <= metadata.publication_date {
            issues.push(error(
                "standard_modification_date",
                "a modification must postdate the standard publication".to_owned(),
                None,
            ));
        }
        let named = targets
            .iter()
            .filter(|target| target.modification_index == index)
            .collect::<Vec<_>>();
        if !source.included_in_source {
            let scope = if named.is_empty() {
                // Deliberately not "affects nothing": the decree's scope is
                // unknown, which is a weaker and more accurate claim.
                "; affected clauses unknown from its title".to_owned()
            } else {
                let marked = named
                    .iter()
                    .filter(|target| target.resolved)
                    .map(|target| target.clause.as_str())
                    .collect::<Vec<_>>();
                format!(
                    "; its title names {} unit(s), {} matching committed clauses ({})",
                    named.len(),
                    marked.len(),
                    if marked.is_empty() {
                        "none".to_owned()
                    } else {
                        marked.join(", ")
                    }
                )
            };
            issues.push(warning(
                "standard_unconsolidated_modification",
                format!(
                    "source text does not incorporate modification published {}{scope}",
                    source.publication_date
                ),
            ));
        }
        // Title diagnostics are raised only where the staleness they describe
        // is real. A modification the retained text already incorporates is not
        // made less current by an unrecorded title, and warning about it would
        // bury the two records where the gap actually matters.
        if !source.included_in_source {
            if source.title.is_none() {
                issues.push(warning(
                    "standard_modification_title_absent",
                    format!(
                        "modification published {} has no recorded DOF title, so the clauses it \
                         addresses cannot be located",
                        source.publication_date
                    ),
                ));
            } else if named.is_empty() {
                issues.push(warning(
                    "standard_modification_scope_unknown",
                    format!(
                        "modification published {} records a title that names no numeral, \
                         apéndice, or anexo; its scope stays at instrument level",
                        source.publication_date
                    ),
                ));
            }
        }
        for target in named.iter().filter(|target| !target.resolved) {
            issues.push(warning(
                "standard_modification_target_unresolved",
                format!(
                    "modification published {} names {} ({}), which matches no committed clause",
                    source.publication_date,
                    target.clause,
                    target.action.as_str(),
                ),
            ));
        }
    }
    if metadata
        .systematic_review
        .as_ref()
        .is_some_and(|review| review.review_date < metadata.publication_date)
    {
        issues.push(error(
            "standard_systematic_review_date",
            "systematic review cannot predate the standard publication".to_owned(),
            None,
        ));
    }
}

fn validate_clauses(
    metadata: &StandardMetadata,
    clauses: &[StandardClause],
    source_text: &str,
    issues: &mut Vec<ValidationIssue>,
) {
    if clauses.is_empty() {
        issues.push(error(
            "standard_clause_count",
            "standard has no numbered clauses".to_owned(),
            None,
        ));
        return;
    }
    let source_chars = source_text.chars().collect::<Vec<_>>();
    let mut ids = HashSet::new();
    let mut numbers = HashSet::new();
    let mut previous: Option<Vec<u32>> = None;
    let mut previous_top_level: Option<u32> = None;
    for clause in clauses {
        if clause.standard_id != metadata.id {
            issues.push(error(
                "standard_clause_instrument",
                "clause points to a different standard".to_owned(),
                Some(clause.id.clone()),
            ));
        }
        if !ids.insert(&clause.id) || !numbers.insert(&clause.number) {
            issues.push(error(
                "standard_clause_duplicate",
                "duplicate clause identifier or number".to_owned(),
                Some(clause.id.clone()),
            ));
        }
        let Some(order) = clause_order(&clause.number) else {
            issues.push(error(
                "standard_clause_number",
                "clause number is not a dot-separated numeric identifier".to_owned(),
                Some(clause.id.clone()),
            ));
            continue;
        };
        if previous.as_ref().is_some_and(|prior| order <= *prior) {
            issues.push(error(
                "standard_clause_order",
                "clause numbers do not increase in source order".to_owned(),
                Some(clause.id.clone()),
            ));
        }
        if order.len() == 1 {
            if previous_top_level.is_some_and(|prior| order[0] != prior + 1) {
                issues.push(error(
                    "standard_clause_gap",
                    "top-level clause numbers must be consecutive".to_owned(),
                    Some(clause.id.clone()),
                ));
            }
            previous_top_level = Some(order[0]);
        } else if previous_top_level != Some(order[0]) {
            issues.push(error(
                "standard_clause_parent",
                "nested clause appears outside its current top-level clause".to_owned(),
                Some(clause.id.clone()),
            ));
        }
        previous = Some(order);
        if clause.start_char >= clause.end_char || clause.end_char > source_chars.len() {
            issues.push(error(
                "standard_clause_span",
                "clause character span is outside the extracted text".to_owned(),
                Some(clause.id.clone()),
            ));
            continue;
        }
        let anchored = source_chars[clause.start_char..clause.end_char]
            .iter()
            .collect::<String>();
        if strip_page_furniture(&anchored) != clause.text {
            issues.push(error(
                "standard_clause_span",
                "clause text does not match its exact extracted-text span".to_owned(),
                Some(clause.id.clone()),
            ));
        }
    }
}

/// How much of the normative body (up to a genuine TRANSITORIOS heading) the
/// selected clause run actually spans.
///
/// Guards against NOM-052-SEMARNAT-2005's historical defect --
/// `numbered_body_run` selecting the índice (11 rows, 474 bytes) instead of
/// the real ~76-clause body -- recalibrated to a body-only denominator now
/// that `StandardSupplement` exists: measuring against the whole document
/// would conflate a correctly-parsed clause run with legitimate supplement
/// text after it, which the original floor predates.
///
/// Only evaluated when a genuine TRANSITORIOS heading exists. Without one,
/// trailing apéndices/anexos can legitimately dwarf the clause run (see
/// NOM-247-SSA1-2008), and that shape is `validate_trailing_material`'s to
/// report, not this check's -- a whole-document fallback here would flag the
/// right instrument for the wrong reason and duplicate that warning.
fn validate_clause_coverage(
    clauses: &[StandardClause],
    source_text: &str,
    issues: &mut Vec<ValidationIssue>,
) {
    const CLAUSE_COVERAGE_FLOOR: f64 = 0.31;
    let (Some(first), Some(last), Some((heading_start, _))) = (
        clauses.first(),
        clauses.last(),
        real_transitorios_heading(source_text),
    ) else {
        return;
    };
    let body_limit = source_text[..heading_start].chars().count();
    if body_limit <= first.start_char || last.end_char <= first.start_char {
        return;
    }
    // Source-text character offsets are bounded by a single document, far
    // below f64's exact-integer limit; the ratio needs floating arithmetic.
    #[allow(clippy::cast_precision_loss)]
    let span = (last.end_char - first.start_char) as f64;
    #[allow(clippy::cast_precision_loss)]
    let coverage = span / (body_limit - first.start_char) as f64;
    if coverage < CLAUSE_COVERAGE_FLOOR {
        issues.push(warning(
            "standard_clause_coverage",
            format!(
                "selected clauses span {:.1}% of the normative body up to TRANSITORIOS; a run \
                 this sparse resembles the índice-selected-as-body defect \
                 (NOM-052-SEMARNAT-2005, 2026-07-28)",
                coverage * 100.0
            ),
        ));
    }
}

/// Filter-regression tripwire, not a general detector: `strip_page_furniture`
/// already removes `is_dof_page_furniture`'s four patterns before `.text` is
/// ever assigned, so this can only fire if that filter has a bug -- the same
/// relationship `contains_page_header_contamination` has to `is_page_furniture`
/// on the diputados path, which is exactly why it fires zero times there. A
/// future NOM whose page furniture takes a shape *not* in the four confirmed
/// patterns will NOT be caught here; that is a source-provenance question
/// (does this instrument's *source* bear print-view furniture at all),
/// deliberately not folded into this fix -- see the design notes.
fn validate_no_page_furniture(
    clauses: &[StandardClause],
    transitories: &[StandardTransitory],
    supplements: &[StandardSupplement],
    issues: &mut Vec<ValidationIssue>,
) {
    let spans = clauses
        .iter()
        .map(|clause| (&clause.id, clause.number.as_str(), &clause.text))
        .chain(transitories.iter().map(|transitory| {
            (
                &transitory.id,
                transitory.ordinal.as_str(),
                &transitory.text,
            )
        }))
        .chain(supplements.iter().map(|supplement| {
            (
                &supplement.id,
                supplement.heading.as_str(),
                &supplement.text,
            )
        }));
    for (id, label, text) in spans {
        if let Some(line) = text.lines().find(|line| is_dof_page_furniture(line.trim())) {
            issues.push(error(
                "standard_page_furniture",
                format!("{label} still carries DOF page furniture: {line:?}"),
                Some(id.clone()),
            ));
        }
    }
}

/// Identity, uniqueness, and exact-span checks for transitorios. No absence
/// or ordering check: an empty list is not itself an error (some retained
/// texts genuinely lack a recognizable TRANSITORIOS section), and ordinal
/// sequencing is deliberately not enforced -- this stays a lightweight
/// inspection, not a structural parse.
fn validate_transitories(
    metadata: &StandardMetadata,
    transitories: &[StandardTransitory],
    source_text: &str,
    issues: &mut Vec<ValidationIssue>,
) {
    let source_chars = source_text.chars().collect::<Vec<_>>();
    let mut ids = HashSet::new();
    let mut ordinals = HashSet::new();
    for transitory in transitories {
        if transitory.standard_id != metadata.id {
            issues.push(error(
                "standard_transitory_instrument",
                "transitory points to a different standard".to_owned(),
                Some(transitory.id.clone()),
            ));
        }
        if !ids.insert(&transitory.id) || !ordinals.insert(&transitory.ordinal) {
            issues.push(error(
                "standard_transitory_duplicate",
                "duplicate transitory identifier or ordinal".to_owned(),
                Some(transitory.id.clone()),
            ));
        }
        if transitory.start_char >= transitory.end_char || transitory.end_char > source_chars.len()
        {
            issues.push(error(
                "standard_transitory_span",
                "transitory character span is outside the extracted text".to_owned(),
                Some(transitory.id.clone()),
            ));
            continue;
        }
        let anchored = source_chars[transitory.start_char..transitory.end_char]
            .iter()
            .collect::<String>();
        if strip_page_furniture(&anchored) != transitory.text {
            issues.push(error(
                "standard_transitory_span",
                "transitory text does not match its exact extracted-text span".to_owned(),
                Some(transitory.id.clone()),
            ));
        }
    }
}

fn validate_supplements(
    metadata: &StandardMetadata,
    transitories: &[StandardTransitory],
    supplements: &[StandardSupplement],
    source_text: &str,
    issues: &mut Vec<ValidationIssue>,
) {
    let source_chars = source_text.chars().collect::<Vec<_>>();
    let mut ids = HashSet::new();
    let mut previous_end = transitories.last().map_or(0, |item| item.end_char);
    if supplements.len() != metadata.supplement_starts.len() {
        issues.push(error(
            "standard_supplement_count",
            format!(
                "metadata configures {} supplement starts but {} supplements were provided",
                metadata.supplement_starts.len(),
                supplements.len()
            ),
            None,
        ));
    }
    for (index, supplement) in supplements.iter().enumerate() {
        let expected_sequence = index + 1;
        if supplement.standard_id != metadata.id {
            issues.push(error(
                "standard_supplement_instrument",
                "supplement points to a different standard".to_owned(),
                Some(supplement.id.clone()),
            ));
        }
        if !ids.insert(&supplement.id) || supplement.sequence != expected_sequence {
            issues.push(error(
                "standard_supplement_identity",
                "supplement identifiers and one-based sequence must be unique and source-ordered"
                    .to_owned(),
                Some(supplement.id.clone()),
            ));
        }
        if metadata
            .supplement_starts
            .get(index)
            .is_some_and(|configured| configured.kind != supplement.kind)
        {
            issues.push(error(
                "standard_supplement_kind",
                "supplement kind differs from its configured anchor".to_owned(),
                Some(supplement.id.clone()),
            ));
        }
        if supplement.start_char < previous_end
            || supplement.start_char >= supplement.end_char
            || supplement.end_char > source_chars.len()
        {
            issues.push(error(
                "standard_supplement_span",
                "supplement spans must be non-overlapping, source-ordered, and inside the extracted text"
                    .to_owned(),
                Some(supplement.id.clone()),
            ));
            continue;
        }
        let anchored = source_chars[supplement.start_char..supplement.end_char]
            .iter()
            .collect::<String>();
        if strip_page_furniture(&anchored) != supplement.text {
            issues.push(error(
                "standard_supplement_span",
                "supplement text does not match its exact extracted-text span".to_owned(),
                Some(supplement.id.clone()),
            ));
        }
        if metadata
            .supplement_starts
            .get(index)
            .is_some_and(|configured| {
                !supplement.text.starts_with(&configured.anchor)
                    || supplement.heading != collapse_whitespace(&configured.anchor)
            })
        {
            issues.push(error(
                "standard_supplement_anchor",
                "supplement does not begin at its exact configured anchor".to_owned(),
                Some(supplement.id.clone()),
            ));
        }
        validate_supplement_character(supplement, issues);
        previous_end = supplement.end_char;
    }
}

fn validate_supplement_character(
    supplement: &StandardSupplement,
    issues: &mut Vec<ValidationIssue>,
) {
    let (derived, conflict) = derive_supplement_legal_character(&supplement.text);
    if conflict {
        issues.push(error(
            "standard_supplement_character_conflict",
            "supplement contains conflicting explicit normative and non-normative signals"
                .to_owned(),
            Some(supplement.id.clone()),
        ));
    } else if supplement.legal_character != derived {
        issues.push(error(
            "standard_supplement_character",
            "supplement legal character is not the deterministic explicit-source derivation"
                .to_owned(),
            Some(supplement.id.clone()),
        ));
    } else if derived == StandardSupplementLegalCharacter::Unspecified {
        issues.push(warning(
            "standard_supplement_character_unspecified",
            format!(
                "supplement {} states no explicit normative or non-normative character",
                supplement.sequence
            ),
        ));
    }
}

pub(crate) fn error(code: &str, message: String, provision_id: Option<String>) -> ValidationIssue {
    ValidationIssue {
        severity: Severity::Error,
        code: code.to_owned(),
        message,
        provision_id,
    }
}

/// Report substantive material following the transitorios section.
///
/// The clause parser deliberately stops at TRANSITORIOS, because what follows
/// is not clause-structured. But some of it is *normative* -- NOM-052's
/// Tablas, Listados and Anexo 1 carry the hazardous-waste classifications the
/// standard exists to establish, and NOM-010's Apéndice I carries its exposure
/// limits -- while other trailing material ("Guía de Referencia ... no es de
/// cumplimiento obligatorio") is explicitly non-binding. Lex-Mex does not yet
/// model either kind.
///
/// Without this warning a compiled standard would present a complete-looking
/// clause body while silently omitting operative content. The warning does not
/// distinguish normative from non-binding trailing material; making that call
/// requires reading it, which is the point of surfacing it to a reviewer.
///
/// Detection is structural first, byte-count last. When the transitorios
/// section ends at an APÉNDICE/ANEXO heading, that heading *is* the evidence
/// -- unmodeled material follows, and it is named unconditionally: a compact
/// single-formula apéndice below any byte threshold is exactly the silent
/// omission this warning exists for. When the section ends at the signature
/// block instead, the remainder is searched for a later annex-like heading
/// (GUÍA, LISTADO, TABLA, APÉNDICE, ANEXO -- annexes are sometimes laid out
/// after the signatures). Only a headingless remainder falls back to a size
/// heuristic, measured after stripping signature furniture (lines through the
/// last "Rúbrica"), because a long multi-signatory block is a closing
/// formality, not omitted content, and a false warning here trains reviewers
/// to dismiss the code.
fn validate_trailing_material(
    metadata: &StandardMetadata,
    clauses: &[StandardClause],
    source_text: &str,
    issues: &mut Vec<ValidationIssue>,
) {
    const TRAILING_MATERIAL_BYTES: usize = 2_000;
    static TRAILING_HEADING: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(&format!(
            r"(?mi)^{LINE_LEAD}((?:AP[ÉE]NDICE|ANEXOS?|GU[ÍI]A|LISTADOS?|TABLAS?)\b[^\r\n]*)"
        ))
        .expect("trailing-heading regex must compile")
    });
    static RUBRICA: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?i)r[úu]brica").expect("rúbrica regex must compile"));

    if !metadata.supplement_starts.is_empty() {
        return;
    }
    // A genuine TRANSITORIOS section anchors the search, same as before. Some
    // standards have none at all -- their last clause runs straight into
    // apéndices/anexos (which, per direct review of NOM-247-SSA1-2008, may be
    // normative without being signed, so nothing downstream of this anchor
    // should assume a Rúbrica exists). Fall back to the last committed
    // clause's end, converting its char offset back to the byte offset this
    // function slices `source_text` with everywhere else.
    let heading_end = if let Some((_, heading_end)) = real_transitorios_heading(source_text) {
        heading_end
    } else {
        let Some(last_clause) = clauses.last() else {
            return;
        };
        source_text
            .char_indices()
            .nth(last_clause.end_char)
            .map_or(source_text.len(), |(byte_offset, _)| byte_offset)
    };
    let section_end = closing_signature_start(source_text, heading_end, source_text.len())
        .unwrap_or(source_text.len());
    let named_heading = TRAILING_HEADING
        .captures(&source_text[heading_end..])
        .map(|captures| collapse_whitespace(&captures[1]));
    if let Some(heading) = named_heading {
        issues.push(warning(
            "standard_trailing_material",
            format!(
                "material after the transitorios section begins at \"{heading}\"; it is not \
                 represented in clauses.json and may be normative"
            ),
        ));
        return;
    }
    // Headingless remainder: measure what is left after the signature block.
    let trailing = source_text[section_end..].trim();
    let unsigned = RUBRICA
        .find_iter(trailing)
        .last()
        .map_or(trailing, |last| {
            let after = &trailing[last.end()..];
            after.split_once('\n').map_or("", |(_, rest)| rest)
        })
        .trim();
    if unsigned.len() > TRAILING_MATERIAL_BYTES {
        issues.push(warning(
            "standard_trailing_material",
            format!(
                "{} bytes of unheaded material follow the transitorios section and its signature \
                 block; this material is not represented in clauses.json and may be normative",
                unsigned.len()
            ),
        ));
    }
}

pub(crate) fn warning(code: &str, message: String) -> ValidationIssue {
    ValidationIssue {
        severity: Severity::Warning,
        code: code.to_owned(),
        message,
        provision_id: None,
    }
}
