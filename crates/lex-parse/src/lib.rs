use std::{
    collections::{HashMap, HashSet},
    fs,
    ops::Range,
    path::Path,
    process::{Command, Stdio},
};

use anyhow::{Context, Result, bail};
use chrono::NaiveDate;
use lex_core::{
    Basis, DefinedTerm, LRITF_INSTRUMENT_ID, Provision, ProvisionType, ReferenceEdge,
    ReferenceForm, ReferenceQualifier, ReferenceQualifierType, ReferenceResolutionStatus,
    SCHEMA_VERSION, Severity, TemporalEvidence, TermUsage, ValidationIssue, ValidationReport,
};
use regex::Regex;

pub mod dcg;
pub mod diputados;
pub mod html;
pub mod itf;
pub mod labels;
pub mod terms;

pub use dcg::parse_dcg;
pub use diputados::{
    DiputadosDocument, DiputadosOptions, extract_dof_publication, extract_reform_evidence,
    parse_diputados,
};
pub use html::extract_html_text;
pub use itf::{ItfDocument, parse_itf_dcg};
pub use terms::{GlossaryStyle, extract_term_usages, extract_terms, find_glossary_provision};

#[derive(Debug)]
pub struct Extraction {
    pub text: String,
    pub tool_version: String,
}

/// Extract text from a PDF with Poppler. `keep_page_breaks` retains the
/// form-feed page markers; the DCG parser needs them to merge paragraphs
/// deterministically across page boundaries.
pub fn extract_pdf(
    pdf_path: &Path,
    text_path: &Path,
    keep_page_breaks: bool,
) -> Result<Extraction> {
    if let Some(parent) = text_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut args = vec!["-layout"];
    if !keep_page_breaks {
        args.push("-nopgbrk");
    }
    let status = Command::new("pdftotext")
        .args(args)
        .arg(pdf_path)
        .arg(text_path)
        .stdin(Stdio::null())
        .status()
        .context("failed to execute pdftotext; install Poppler")?;
    if !status.success() {
        bail!("pdftotext failed with status {status}");
    }
    let text = fs::read_to_string(text_path)
        .with_context(|| format!("failed to read {}", text_path.display()))?;
    if text.trim().is_empty() {
        bail!("pdftotext produced empty output");
    }
    Ok(Extraction {
        text,
        tool_version: pdftotext_version(),
    })
}

struct ReferencePatterns {
    article: Regex,
    number: Regex,
    separator: Regex,
    paragraph: Regex,
    fraction: Regex,
    subsection: Regex,
    transitory_citation: Regex,
    /// Qualifier phrase written before the article number, ending right at
    /// the `artículo(s)` header: `la fracción XI del artículo 36`.
    pre_qualifier: Regex,
    /// Fraction citation of the containing article itself:
    /// `fracciones I, II, III y IV del presente artículo`.
    same_article_fraction: Regex,
    /// Position-relative citation of a neighboring provision: `artículo
    /// anterior` / `artículo siguiente`. Singular only — the plural
    /// (`los artículos anteriores`) names an open-ended set with no single
    /// deterministic target, so it stays unlinked.
    relative_article: Regex,
    roman: Regex,
}

impl ReferencePatterns {
    fn new() -> Result<Self> {
        Ok(Self {
            article: Regex::new(r"(?i)\bartículos?\s+")?,
            transitory_citation: Regex::new(
                r"(?i)\bdisposici(?:ón|ones)\s+(PRIMERA|SEGUNDA|TERCERA|CUARTA|QUINTA|SEXTA|SÉPTIMA|OCTAVA|NOVENA|DÉCIMA(?:\s+PRIMERA)?)\s+Transitorias?",
            )?,
            pre_qualifier: Regex::new(
                r"(?ix)\b(
                    fracci(?:ón|ones)\s+[IVXLCDM]+(?:\s*(?:,|y|o)\s*[IVXLCDM]+)* |
                    (?:primer|primero|segundo|tercer|tercero|cuarto|quinto|sexto|séptimo|octavo|noveno|décimo|penúltimo|último)
                        (?:\s+(?:,|y|o)\s+(?:primer|primero|segundo|tercer|tercero|cuarto|quinto|sexto|séptimo|octavo|noveno|décimo|penúltimo|último))*
                        \s+párrafos? |
                    párrafos?\s+(?:primero|segundo|tercero|cuarto|quinto|sexto|séptimo|octavo|noveno|décimo|penúltimo|último)
                        (?:\s*(?:,|y|o)\s*(?:primero|segundo|tercero|cuarto|quinto|sexto|séptimo|octavo|noveno|décimo|penúltimo|último))* |
                    incisos?\s+[a-z](?:\s*(?:,|y|o)\s*[a-z])*
                )\s+de(?:l|\s+los)\s*$",
            )?,
            same_article_fraction: Regex::new(
                r"(?i)\bfracci(?:ón|ones)\s+([IVXLCDM]+(?:\s*(?:,|y|o)\s*[IVXLCDM]+)*)\s+de(?:l\s+presente|\s+este)\s+artículo",
            )?,
            relative_article: Regex::new(r"(?i)\bartículo\s+(anterior|siguiente)\b")?,
            roman: Regex::new(r"\b[IVXLCDM]+\b")?,
            number: Regex::new(r"(?i)\d{1,3}(?:-[A-Z])?(?:\s+(?:Bis|Ter|Quáter))?")?,
            separator: Regex::new(
                r"(?ix)^(?:
            [\s,;:/()\-]+ |
            y\b | o\b | a\b | al\b | hasta\b |
            primer(?:o)?\b | segund[oa]\b | tercer(?:o)?\b | cuart[oa]\b |
            quint[oa]\b | sext[oa]\b | séptim[oa]\b | octav[oa]\b |
            noven[oa]\b | décim[oa]\b | últim[oa]\b |
            párrafos?\b | fracci(?:ón|ones)\b | incisos?\b | apartados?\b |
            [IVXLCDM]+\b
        )*$",
            )?,
            paragraph: Regex::new(
                r"(?i)\b(?:primer|primero|segundo|tercer|tercero|cuarto|quinto|sexto|séptimo|octavo|noveno|décimo|último)(?:\s+(?:,|y|o)\s+(?:primer|primero|segundo|tercer|tercero|cuarto|quinto|sexto|séptimo|octavo|noveno|décimo|último))*\s+párrafos?\b",
            )?,
            fraction: Regex::new(
                r"(?i)\bfracci(?:ón|ones)\s+[IVXLCDM]+(?:\s*(?:,|y|o)\s*[IVXLCDM]+)*\b",
            )?,
            subsection: Regex::new(r"(?i)\bincisos?\s+[a-z](?:\s*(?:,|y|o)\s*[a-z])*\b")?,
        })
    }
}

/// The provision or instrument title whose text is being scanned for
/// references.
#[derive(Debug, Clone, Copy)]
struct ReferenceSource<'a> {
    /// Canonical identifier the edge is anchored to: a provision ID, or the
    /// instrument ID itself for citations inside the official title.
    source_id: &'a str,
    instrument_id: &'a str,
    text: &'a str,
}

/// How a citation group's target instrument is decided.
#[derive(Debug, Clone)]
pub enum InstrumentContextPolicy {
    /// Original LRITF behavior: a group is skipped when generic external-law
    /// context appears anywhere in the group without internal-law context.
    /// Preserved verbatim so the audited LRITF graph stays byte-identical.
    WholeGroupPresence,
    /// Sentence-scoped decision used for multi-instrument corpora: within the
    /// citation sentence, the earliest marker decides between this
    /// instrument, a configured external instrument, or an unlinked external
    /// law. No marker means the citation is internal.
    SentenceEarliestMarker {
        /// Lowercase phrases marking a citation as internal.
        internal_markers: Vec<String>,
        /// Lowercase official-name fragments mapped to instrument IDs.
        external_instruments: Vec<(String, String)>,
    },
}

#[derive(Debug, Clone)]
pub struct ReferenceOptions {
    pub policy: InstrumentContextPolicy,
    /// Extract `disposición ORDINAL Transitoria` citations as transitory
    /// reference edges. Enabled for multi-instrument extraction only.
    pub transitory_citations: bool,
    /// Extract `fracción N del presente artículo` citations as edges
    /// targeting the containing provision, so the fraction numerals can
    /// link to their own fraction blocks.
    pub same_article_fractions: bool,
    /// Extract `artículo anterior` / `artículo siguiente` citations as
    /// edges targeting the source provision's neighbor of the same
    /// provision type in document order.
    pub relative_references: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GroupTarget<'a> {
    Internal,
    External(&'a str),
    Skip,
}

pub fn extract_internal_references(provisions: &[Provision]) -> Result<Vec<ReferenceEdge>> {
    let options = ReferenceOptions {
        policy: InstrumentContextPolicy::WholeGroupPresence,
        transitory_citations: false,
        same_article_fractions: true,
        relative_references: true,
    };
    let target_ids: HashSet<String> = provisions.iter().map(|item| item.id.clone()).collect();
    extract_references(provisions, None, &options, &target_ids)
}

/// The source provision's same-type neighbors in document order, used to
/// resolve `artículo anterior` / `artículo siguiente`.
#[derive(Debug, Clone, Copy, Default)]
struct RelativeNeighbors<'a> {
    previous: Option<&'a str>,
    next: Option<&'a str>,
}

/// Compute each provision's same-type neighbors in document order. An
/// `artículo anterior` inside a transitory refers to the previous
/// transitory, not to the last numbered article, so neighbor sequences
/// never cross provision types.
fn relative_neighbors<'a>(provisions: &'a [Provision]) -> HashMap<&'a str, RelativeNeighbors<'a>> {
    let mut sequences: HashMap<&ProvisionType, Vec<&'a str>> = HashMap::new();
    for provision in provisions {
        sequences
            .entry(&provision.provision_type)
            .or_default()
            .push(provision.id.as_str());
    }
    let mut neighbors = HashMap::new();
    for sequence in sequences.values() {
        for (index, id) in sequence.iter().enumerate() {
            neighbors.insert(
                *id,
                RelativeNeighbors {
                    previous: index.checked_sub(1).map(|prev| sequence[prev]),
                    next: sequence.get(index + 1).copied(),
                },
            );
        }
    }
    neighbors
}

/// Extract reference edges from every provision and, when provided, from the
/// instrument's official title. `known_targets` holds canonical provision
/// identifiers across every loaded instrument; a target inside it resolves.
#[allow(clippy::implicit_hasher)]
pub fn extract_references(
    provisions: &[Provision],
    title_source: Option<(&str, &str)>,
    options: &ReferenceOptions,
    known_targets: &HashSet<String>,
) -> Result<Vec<ReferenceEdge>> {
    let patterns = ReferencePatterns::new()?;
    let neighbors = relative_neighbors(provisions);
    let mut references = Vec::new();
    if let Some((instrument_id, official_title)) = title_source {
        // The instrument title has no position in the provision sequence,
        // so it can never carry a relative reference.
        references.extend(extract_source_references(
            ReferenceSource {
                source_id: instrument_id,
                instrument_id,
                text: official_title,
            },
            RelativeNeighbors::default(),
            &patterns,
            options,
            known_targets,
        ));
    }
    for provision in provisions {
        references.extend(extract_source_references(
            ReferenceSource {
                source_id: &provision.id,
                instrument_id: &provision.instrument_id,
                text: &provision.text,
            },
            neighbors
                .get(provision.id.as_str())
                .copied()
                .unwrap_or_default(),
            &patterns,
            options,
            known_targets,
        ));
    }
    references.sort_by(|left, right| {
        left.source_provision_id
            .cmp(&right.source_provision_id)
            .then(left.start_char.cmp(&right.start_char))
            .then(left.end_char.cmp(&right.end_char))
            .then(left.target_provision_id.cmp(&right.target_provision_id))
    });
    Ok(references)
}

fn extract_source_references(
    source: ReferenceSource<'_>,
    neighbors: RelativeNeighbors<'_>,
    patterns: &ReferencePatterns,
    options: &ReferenceOptions,
    known_targets: &HashSet<String>,
) -> Vec<ReferenceEdge> {
    let headers: Vec<_> = patterns.article.find_iter(source.text).collect();
    let mut references = Vec::new();
    for (index, header) in headers.iter().enumerate() {
        let group_end = headers
            .get(index + 1)
            .map_or(source.text.len(), regex::Match::start);
        let group = &source.text[header.end()..group_end];
        references.extend(extract_reference_group(
            source,
            header.start(),
            header.end(),
            group,
            patterns,
            options,
            known_targets,
        ));
    }
    if options.transitory_citations {
        references.extend(extract_transitory_citations(
            source,
            patterns,
            options,
            known_targets,
        ));
    }
    if options.same_article_fractions {
        references.extend(extract_same_article_fractions(
            source,
            patterns,
            known_targets,
        ));
    }
    if options.relative_references {
        references.extend(extract_relative_references(
            source,
            neighbors,
            patterns,
            known_targets,
        ));
    }
    references
}

/// Extract `artículo anterior` / `artículo siguiente` citations as edges
/// targeting the source provision's same-type neighbor in document order.
/// A phrase with no neighbor in that direction (`artículo anterior` inside
/// the first article) produces no edge. Bare self-references (`este
/// artículo`, `el presente artículo`) are deliberately not extracted: the
/// reader is already inside the target, so a link adds nothing, and the
/// fraction-scoped form is already handled by the same-article path.
fn extract_relative_references(
    source: ReferenceSource<'_>,
    neighbors: RelativeNeighbors<'_>,
    patterns: &ReferencePatterns,
    known_targets: &HashSet<String>,
) -> Vec<ReferenceEdge> {
    let mut references = Vec::new();
    for captures in patterns.relative_article.captures_iter(source.text) {
        let phrase = captures.get(0).expect("relative match");
        let direction = captures.get(1).expect("direction capture");
        let target = if direction.as_str().eq_ignore_ascii_case("anterior") {
            neighbors.previous
        } else {
            neighbors.next
        };
        let Some(target) = target else {
            continue;
        };
        let qualifiers = pre_number_qualifiers(source.text, phrase.start(), patterns);
        references.push(reference_edge(
            source,
            phrase.as_str(),
            phrase.start()..phrase.end(),
            target.to_owned(),
            qualifiers,
            ReferenceForm::Relative,
            known_targets,
        ));
    }
    references
}

fn extract_reference_group(
    source: ReferenceSource<'_>,
    header_start: usize,
    group_start: usize,
    group: &str,
    patterns: &ReferencePatterns,
    options: &ReferenceOptions,
    known_targets: &HashSet<String>,
) -> Vec<ReferenceEdge> {
    let accepted = accepted_numbers(group, patterns);
    if accepted.is_empty() {
        return Vec::new();
    }
    let pre_qualifiers = pre_number_qualifiers(source.text, header_start, patterns);
    let context_end = accepted
        .last()
        .map_or(group.len(), |last| match options.policy {
            InstrumentContextPolicy::WholeGroupPresence => group.len(),
            InstrumentContextPolicy::SentenceEarliestMarker { .. } => {
                last.end() + qualifier_boundary(&group[last.end()..])
            }
        });
    let target_instrument_id = match group_target(&group[..context_end], &options.policy) {
        GroupTarget::Internal => source.instrument_id,
        GroupTarget::External(instrument_id) => instrument_id,
        GroupTarget::Skip => return Vec::new(),
    };
    let mut references = direct_reference_edges(
        source,
        target_instrument_id,
        group_start,
        group,
        &accepted,
        &pre_qualifiers,
        patterns,
        known_targets,
    );
    references.extend(range_expansion_edges(
        source,
        target_instrument_id,
        group_start,
        group,
        &accepted,
        known_targets,
    ));
    references
}

fn group_target<'a>(context: &str, policy: &'a InstrumentContextPolicy) -> GroupTarget<'a> {
    let lower = context.to_lowercase();
    match policy {
        InstrumentContextPolicy::WholeGroupPresence => {
            if has_external_instrument_context(&lower) && !has_internal_instrument_context(&lower) {
                GroupTarget::Skip
            } else {
                GroupTarget::Internal
            }
        }
        InstrumentContextPolicy::SentenceEarliestMarker {
            internal_markers,
            external_instruments,
        } => sentence_target(&lower, internal_markers, external_instruments),
    }
}

fn sentence_target<'a>(
    lower: &str,
    internal_markers: &[String],
    external_instruments: &'a [(String, String)],
) -> GroupTarget<'a> {
    let internal = internal_markers
        .iter()
        .filter_map(|marker| lower.find(marker.as_str()))
        .min();
    let configured = external_instruments
        .iter()
        .filter_map(|(marker, instrument_id)| {
            lower
                .find(marker.as_str())
                .map(|position| (position, instrument_id.as_str()))
        })
        .min_by_key(|(position, _)| *position);
    let generic = EXTERNAL_INSTRUMENT_MARKERS
        .iter()
        .filter_map(|marker| {
            // Match the marker at a word boundary so punctuation directly
            // after the cited law ("de la Ley,") still counts as external.
            let trimmed = marker.trim_end();
            lower.find(trimmed).and_then(|position| {
                lower[position + trimmed.len()..]
                    .chars()
                    .next()
                    .is_none_or(|following| !following.is_alphabetic())
                    .then_some((position, trimmed.len()))
            })
        })
        // A configured instrument name inside the same phrase (for example
        // "de la ley para regular…") supersedes the generic law marker.
        .filter(|(position, length)| {
            configured.is_none_or(|(configured_position, _)| {
                configured_position < *position || configured_position > position + length + 1
            })
        })
        .map(|(position, _)| position)
        .min();

    let candidates = [
        internal.map(|position| (position, GroupTarget::Internal)),
        configured.map(|(position, id)| (position, GroupTarget::External(id))),
        generic.map(|position| (position, GroupTarget::Skip)),
    ];
    candidates
        .into_iter()
        .flatten()
        .min_by_key(|(position, _)| *position)
        .map_or(GroupTarget::Internal, |(_, target)| target)
}

fn extract_transitory_citations(
    source: ReferenceSource<'_>,
    patterns: &ReferencePatterns,
    options: &ReferenceOptions,
    known_targets: &HashSet<String>,
) -> Vec<ReferenceEdge> {
    let mut references = Vec::new();
    for captures in patterns.transitory_citation.captures_iter(source.text) {
        let ordinal = captures.get(1).expect("ordinal capture");
        let citation_end = captures.get(0).expect("citation match").end();
        let context = &source.text[citation_end..];
        let context = &context[..qualifier_boundary(context)];
        let target_instrument_id = match group_target(context, &options.policy) {
            GroupTarget::Internal => source.instrument_id,
            GroupTarget::External(instrument_id) => instrument_id,
            GroupTarget::Skip => continue,
        };
        let target_provision_id = format!(
            "{target_instrument_id}:transitory:{}",
            slug(ordinal.as_str())
        );
        references.push(reference_edge(
            source,
            &source.text[ordinal.start()..citation_end],
            ordinal.start()..citation_end,
            target_provision_id,
            Vec::new(),
            ReferenceForm::Direct,
            known_targets,
        ));
    }
    references
}

fn accepted_numbers<'a>(group: &'a str, patterns: &ReferencePatterns) -> Vec<regex::Match<'a>> {
    let candidates: Vec<_> = patterns.number.find_iter(group).collect();
    let Some(first) = candidates.first() else {
        return Vec::new();
    };
    if !group[..first.start()].trim().is_empty() {
        return Vec::new();
    }
    let mut accepted = vec![*first];
    for candidate in candidates.iter().skip(1) {
        let Some(previous) = accepted.last() else {
            break;
        };
        if !patterns
            .separator
            .is_match(&group[previous.end()..candidate.start()])
        {
            break;
        }
        accepted.push(*candidate);
    }
    accepted
}

#[allow(clippy::too_many_arguments)]
fn direct_reference_edges(
    source: ReferenceSource<'_>,
    target_instrument_id: &str,
    group_start: usize,
    group: &str,
    accepted: &[regex::Match<'_>],
    pre_qualifiers: &[ReferenceQualifier],
    patterns: &ReferencePatterns,
    known_targets: &HashSet<String>,
) -> Vec<ReferenceEdge> {
    accepted
        .iter()
        .enumerate()
        .map(|(index, number_match)| {
            let qualifier_end = accepted
                .get(index + 1)
                .map_or(group.len(), regex::Match::start);
            let qualifier_text = &group[number_match.end()..qualifier_end];
            let qualifier_text = if index + 1 == accepted.len() {
                &qualifier_text[..qualifier_boundary(qualifier_text)]
            } else {
                qualifier_text
            };
            // A qualifier written before the header scopes over the whole
            // article list, so it attaches to every direct edge.
            let mut qualifiers = pre_qualifiers.to_vec();
            qualifiers.extend(extract_qualifiers(
                source.text,
                group_start + number_match.end(),
                qualifier_text,
                patterns,
            ));
            reference_edge(
                source,
                number_match.as_str(),
                (group_start + number_match.start())..(group_start + number_match.end()),
                canonical_article_id(target_instrument_id, number_match.as_str()),
                qualifiers,
                ReferenceForm::Direct,
                known_targets,
            )
        })
        .collect()
}

fn range_expansion_edges(
    source: ReferenceSource<'_>,
    target_instrument_id: &str,
    group_start: usize,
    group: &str,
    accepted: &[regex::Match<'_>],
    known_targets: &HashSet<String>,
) -> Vec<ReferenceEdge> {
    let mut references = Vec::new();
    for pair in accepted.windows(2) {
        let separator = group[pair[0].end()..pair[1].start()].trim().to_lowercase();
        let (Some(start), Some(end)) = (
            numeric_article_number(pair[0].as_str()),
            numeric_article_number(pair[1].as_str()),
        ) else {
            continue;
        };
        if !matches!(separator.as_str(), "a" | "al" | "hasta") || end <= start || end - start > 200
        {
            continue;
        }
        let range = (group_start + pair[0].start())..(group_start + pair[1].end());
        let source_span = &source.text[range.clone()];
        for expanded in (start + 1)..end {
            references.push(reference_edge(
                source,
                source_span,
                range.clone(),
                canonical_article_id(target_instrument_id, &expanded.to_string()),
                Vec::new(),
                ReferenceForm::RangeExpansion,
                known_targets,
            ));
        }
    }
    references
}

fn reference_edge(
    source: ReferenceSource<'_>,
    source_span: &str,
    source_range: Range<usize>,
    target_provision_id: String,
    qualifiers: Vec<ReferenceQualifier>,
    reference_form: ReferenceForm,
    known_targets: &HashSet<String>,
) -> ReferenceEdge {
    let start_char = source.text[..source_range.start].chars().count();
    let end_char = start_char + source.text[source_range].chars().count();
    let target_slug = target_provision_id.rsplit(':').next().unwrap_or("unknown");
    let form_slug = match reference_form {
        ReferenceForm::Direct => "direct",
        ReferenceForm::RangeExpansion => "range",
        ReferenceForm::Relative => "relative",
    };
    let resolution_status = if known_targets.contains(target_provision_id.as_str()) {
        ReferenceResolutionStatus::Resolved
    } else {
        ReferenceResolutionStatus::Unresolved
    };
    let target_instrument_id = target_provision_id
        .rsplit_once(":article:")
        .or_else(|| target_provision_id.rsplit_once(":transitory:"))
        .or_else(|| target_provision_id.rsplit_once(":annex:"))
        .map_or(source.instrument_id, |(instrument, _)| instrument);
    ReferenceEdge {
        schema_version: SCHEMA_VERSION.to_owned(),
        id: format!(
            "{}:reference:{start_char}-{end_char}:{target_slug}:{form_slug}",
            source.source_id
        ),
        source_provision_id: source.source_id.to_owned(),
        source_span: source_span.to_owned(),
        start_char,
        end_char,
        target_instrument_id: target_instrument_id.to_owned(),
        target_provision_id,
        qualifiers,
        basis: Basis::ExpressCrossReference,
        confidence: 1.0,
        resolution_status,
        reference_form,
    }
}

fn canonical_article_id(instrument_id: &str, number: &str) -> String {
    // Match the provision-id convention: ordinal marks are dropped so a
    // citation of "2" or "2o" both resolve to article "2".
    format!("{instrument_id}:article:{}", labels::canonical_slug(number))
}

fn numeric_article_number(number: &str) -> Option<u32> {
    number.trim().parse().ok()
}

const EXTERNAL_INSTRUMENT_MARKERS: &[&str] = &[
    "de la ley ",
    "del código ",
    "de la constitución ",
    "del reglamento ",
    "de dicha ley",
    "de esa ley",
    "de este código",
    "del presente código",
    // Anaphoric references to a previously named external instrument.
    // A código citing another law's article ends the citation "…de la
    // citada Ley"; the named law is outside the corpus, so the edge is
    // left unlinked rather than mis-resolved as internal.
    "de la citada ley",
    "de la misma ley",
    "de la referida ley",
    "de la mencionada ley",
    "de dicho código",
    "del citado código",
    "del mismo código",
    "del referido código",
    "del mencionado código",
];

fn has_internal_instrument_context(value: &str) -> bool {
    [
        "de esta ley",
        "de la presente ley",
        "de este ordenamiento",
        "del presente ordenamiento",
    ]
    .iter()
    .any(|marker| value.contains(marker))
}

fn has_external_instrument_context(value: &str) -> bool {
    EXTERNAL_INSTRUMENT_MARKERS
        .iter()
        .any(|marker| value.contains(marker))
}

/// Extract post-number qualifiers from `value`, a slice of the source text
/// beginning at byte offset `base` within `source_text`. Each qualifier is
/// anchored with its Unicode character span in the full source text.
fn extract_qualifiers(
    source_text: &str,
    base: usize,
    value: &str,
    patterns: &ReferencePatterns,
) -> Vec<ReferenceQualifier> {
    let searches = [
        (&patterns.paragraph, ReferenceQualifierType::Paragraph),
        (&patterns.fraction, ReferenceQualifierType::Fraction),
        (&patterns.subsection, ReferenceQualifierType::Subsection),
    ];
    let mut matches = Vec::new();
    for (regex, qualifier_type) in searches {
        matches.extend(regex.find_iter(value).map(|item| {
            (
                item.start(),
                anchored_qualifier(
                    source_text,
                    base + item.start(),
                    item.as_str(),
                    qualifier_type.clone(),
                ),
            )
        }));
    }
    matches.sort_by_key(|(start, _)| *start);
    matches
        .into_iter()
        .map(|(_, qualifier)| qualifier)
        .collect()
}

fn anchored_qualifier(
    source_text: &str,
    byte_start: usize,
    text: &str,
    qualifier_type: ReferenceQualifierType,
) -> ReferenceQualifier {
    let start_char = source_text[..byte_start].chars().count();
    ReferenceQualifier {
        qualifier_type,
        text: text.to_owned(),
        start_char: Some(start_char),
        end_char: Some(start_char + text.chars().count()),
    }
}

/// Capture a qualifier phrase written immediately before the `artículo(s)`
/// header, as in `las fracciones II, III, IV y V del artículo 22` or
/// `el séptimo párrafo del artículo 29`. The phrase must end exactly at the
/// header, connected by `del` / `de los`.
fn pre_number_qualifiers(
    source_text: &str,
    header_start: usize,
    patterns: &ReferencePatterns,
) -> Vec<ReferenceQualifier> {
    let before = &source_text[..header_start];
    let window_start = before
        .char_indices()
        .rev()
        .nth(119)
        .map_or(0, |(index, _)| index);
    let window = &before[window_start..];
    let Some(captures) = patterns.pre_qualifier.captures(window) else {
        return Vec::new();
    };
    let phrase = captures.get(1).expect("qualifier capture");
    let lower = phrase.as_str().to_lowercase();
    let qualifier_type = if lower.starts_with("fracci") {
        ReferenceQualifierType::Fraction
    } else if lower.starts_with("inciso") {
        ReferenceQualifierType::Subsection
    } else {
        ReferenceQualifierType::Paragraph
    };
    vec![anchored_qualifier(
        source_text,
        window_start + phrase.start(),
        phrase.as_str(),
        qualifier_type,
    )]
}

/// Extract `fracción N del presente artículo` citations as edges targeting
/// the containing provision. One edge per numeral, spanning exactly the
/// numeral, so the exporter can link it to the provision's own fraction
/// block. A numeral only produces an edge when the provision actually has
/// that fraction as a paragraph.
fn extract_same_article_fractions(
    source: ReferenceSource<'_>,
    patterns: &ReferencePatterns,
    known_targets: &HashSet<String>,
) -> Vec<ReferenceEdge> {
    // The instrument title is not a provision and has no fractions.
    if source.source_id == source.instrument_id {
        return Vec::new();
    }
    let own_fractions = fraction_labels(source.text);
    let mut references = Vec::new();
    for captures in patterns.same_article_fraction.captures_iter(source.text) {
        let numerals = captures.get(1).expect("numerals capture");
        for numeral in patterns.roman.find_iter(numerals.as_str()) {
            if !own_fractions.contains(numeral.as_str()) {
                continue;
            }
            let byte_start = numerals.start() + numeral.start();
            let byte_end = numerals.start() + numeral.end();
            references.push(reference_edge(
                source,
                numeral.as_str(),
                byte_start..byte_end,
                source.source_id.to_owned(),
                vec![anchored_qualifier(
                    source.text,
                    byte_start,
                    numeral.as_str(),
                    ReferenceQualifierType::Fraction,
                )],
                ReferenceForm::Direct,
                known_targets,
            ));
        }
    }
    references
}

/// Roman-numeral labels of the fraction paragraphs in a provision's text.
fn fraction_labels(text: &str) -> HashSet<&str> {
    let fraction_start = Regex::new(r"^([IVXLCDM]+)\.\s").expect("static regex");
    text.split("\n\n")
        .filter_map(|paragraph| {
            fraction_start
                .captures(paragraph)
                .map(|captures| captures.get(1).expect("label").as_str())
        })
        .collect()
}

fn qualifier_boundary(value: &str) -> usize {
    value
        .char_indices()
        .find_map(|(index, character)| matches!(character, '.' | '\n').then_some(index))
        .unwrap_or(value.len())
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
        Some(_) => {}
        None => issues.push(ValidationIssue {
            severity: Severity::Warning,
            code: "counts_not_frozen".to_owned(),
            message: format!(
                "no article-count baseline frozen; parse found {article_count} articles"
            ),
            provision_id: None,
        }),
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
        if provision.text.contains("CÁMARA DE DIPUTADOS")
            || provision
                .text
                .contains("Secretaría de Servicios Parlamentarios")
        {
            issues.push(error(
                "header_contamination",
                "page header leaked into provision text".to_owned(),
                Some(provision.id.clone()),
            ));
        }
        match provision.provision_type {
            ProvisionType::Article => {
                if expectations.allow_article_gaps {
                    validate_article_order_by_label(provision, &mut previous_article, issues);
                } else {
                    match provision.number.parse::<u32>() {
                        Ok(number) if number == expected_number => expected_number += 1,
                        Ok(number) => issues.push(error(
                            "article_order",
                            format!("expected article {expected_number}, found {number}"),
                            Some(provision.id.clone()),
                        )),
                        Err(_) => issues.push(ValidationIssue {
                            severity: Severity::Warning,
                            code: "non_numeric_article".to_owned(),
                            message: "article suffix requires ordering review".to_owned(),
                            provision_id: Some(provision.id.clone()),
                        }),
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

fn validate_references(
    instrument_id: &str,
    official_title: Option<&str>,
    provisions: &[Provision],
    references: &[ReferenceEdge],
    external_targets: &HashSet<String>,
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

fn validate_reference(
    instrument_id: &str,
    official_title: Option<&str>,
    reference: &ReferenceEdge,
    provisions_by_id: &HashMap<&str, &Provision>,
    external_targets: &HashSet<String>,
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

fn validate_reference_target(
    instrument_id: &str,
    reference: &ReferenceEdge,
    provisions_by_id: &HashMap<&str, &Provision>,
    external_targets: &HashSet<String>,
    issues: &mut Vec<ValidationIssue>,
) {
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
        (ReferenceResolutionStatus::Resolved, true) => return,
    };
    issues.push(error(
        code,
        message,
        Some(reference.source_provision_id.clone()),
    ));
}

fn char_slice(value: &str, start: usize, end: usize) -> Option<String> {
    if start > end || end > value.chars().count() {
        return None;
    }
    Some(value.chars().skip(start).take(end - start).collect())
}

pub(crate) fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(crate) fn slug(value: &str) -> String {
    value
        .to_lowercase()
        .replace('á', "a")
        .replace('é', "e")
        .replace('í', "i")
        .replace('ó', "o")
        .replace(['ú', 'ü'], "u")
        .replace(' ', "-")
}

pub(crate) fn spanish_date(day: &str, month: &str, year: &str) -> Option<NaiveDate> {
    let month = match month {
        "enero" => 1,
        "febrero" => 2,
        "marzo" => 3,
        "abril" => 4,
        "mayo" => 5,
        "junio" => 6,
        "julio" => 7,
        "agosto" => 8,
        "septiembre" => 9,
        "octubre" => 10,
        "noviembre" => 11,
        "diciembre" => 12,
        _ => return None,
    };
    NaiveDate::from_ymd_opt(year.parse().ok()?, month, day.parse().ok()?)
}

fn pdftotext_version() -> String {
    Command::new("pdftotext")
        .arg("-v")
        .output()
        .ok()
        .and_then(|output| {
            let text = if output.stderr.is_empty() {
                String::from_utf8(output.stdout).ok()?
            } else {
                String::from_utf8(output.stderr).ok()?
            };
            text.lines().next().map(str::to_owned)
        })
        .unwrap_or_else(|| "pdftotext (version unavailable)".to_owned())
}

fn error(code: &str, message: String, provision_id: Option<String>) -> ValidationIssue {
    ValidationIssue {
        severity: Severity::Error,
        code: code.to_owned(),
        message,
        provision_id,
    }
}

/// Build one reform-transitory `TemporalEvidence` item: the shared
/// provision-ID and label convention every compiled/consolidated
/// document's amending-act transitories use
/// (`{instrument_id}:amendment:{date}:transitory:{ordinal}`). `text` is
/// already assembled by the caller, since how source lines join into it
/// (paragraph-preserving for LRITF's block-scanned decree appendix,
/// single-space for the ITF DCG's line-scanned resolution sections)
/// differs by source shape — only the ID/label convention is shared.
pub(crate) fn reform_evidence_item(
    instrument_id: &str,
    date: NaiveDate,
    ordinal: &str,
    resolution_word: &str,
    text: String,
) -> TemporalEvidence {
    let date = date.format("%Y-%m-%d");
    TemporalEvidence {
        provision_id: format!(
            "{instrument_id}:amendment:{date}:transitory:{}",
            slug(ordinal)
        ),
        label: format!("Transitorio {ordinal} — {resolution_word} DOF {date}"),
        text,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use chrono::NaiveDate;
    use lex_core::{ReferenceForm, ReferenceQualifierType, ReferenceResolutionStatus};
    use pretty_assertions::assert_eq;

    use super::{
        DiputadosOptions, InstrumentContextPolicy, ReferenceOptions, extract_internal_references,
        extract_references, extract_reform_evidence, parse_dcg, parse_diputados, validate_lritf,
    };

    const FIXTURE: &str = include_str!("../../../fixtures/lritf/parser-sample.txt");
    const REFERENCE_FIXTURE: &str = include_str!("../../../fixtures/lritf/reference-sample.txt");
    const RELATIVE_FIXTURE: &str =
        include_str!("../../../fixtures/lritf/relative-reference-sample.txt");
    const DCG_FIXTURE: &str = include_str!("../../../fixtures/ifpe-dcg-2021/parser-sample.txt");
    const DCG_ANNEX_1_FIXTURE: &str =
        include_str!("../../../fixtures/ifpe-dcg-2021/annex-1-sample.txt");
    const DCG_ID: &str = "urn:lex-mx:federal:regulation:ifpe-dcg-2021";
    const LRITF_ID: &str = "urn:lex-mx:federal:statute:lritf";

    /// The exact options the CLI derives for the committed LRITF adapter;
    /// the committed corpus is the byte-identity fixture for them.
    fn lritf_options() -> DiputadosOptions {
        DiputadosOptions {
            instrument_id: LRITF_ID.to_owned(),
            header_lines: vec![
                "LEY PARA REGULAR LAS INSTITUCIONES DE TECNOLOGÍA FINANCIERA".to_owned(),
            ],
            stop_markers: vec!["ARTÍCULOS SEGUNDO A DÉCIMO".to_owned()],
        }
    }

    fn parse_lritf(
        raw: &str,
        date: chrono::NaiveDate,
    ) -> Result<Vec<lex_core::Provision>, anyhow::Error> {
        parse_diputados(raw, &lritf_options(), date).map(|document| document.provisions)
    }

    fn dcg_reference_options() -> ReferenceOptions {
        ReferenceOptions {
            policy: InstrumentContextPolicy::SentenceEarliestMarker {
                internal_markers: [
                    "de estas disposiciones",
                    "de las presentes disposiciones",
                    "del presente instrumento",
                    "de este instrumento",
                ]
                .iter()
                .map(|marker| (*marker).to_owned())
                .collect(),
                external_instruments: vec![(
                    "ley para regular las instituciones de tecnología financiera".to_owned(),
                    LRITF_ID.to_owned(),
                )],
            },
            transitory_citations: true,
            same_article_fractions: true,
            relative_references: true,
        }
    }

    const DCG_TITLE: &str = "Disposiciones aplicables a las instituciones de fondos de pago electrónico \
         a que se refieren los artículos 48, segundo párrafo; 54, primer párrafo, y \
         56, primer y segundo párrafos de la Ley para Regular las Instituciones de \
         Tecnología Financiera";

    fn dcg_fixture_graph() -> (
        Vec<lex_core::Provision>,
        Vec<lex_core::ReferenceEdge>,
        HashSet<String>,
    ) {
        let date = NaiveDate::from_ymd_opt(2021, 1, 28).unwrap();
        let provisions = parse_dcg(
            DCG_FIXTURE,
            &[(1, DCG_ANNEX_1_FIXTURE.to_owned())],
            DCG_ID,
            date,
            &["1".to_owned()],
        )
        .unwrap();
        let mut known_targets: HashSet<String> =
            provisions.iter().map(|item| item.id.clone()).collect();
        for target in [
            format!("{LRITF_ID}:article:48"),
            format!("{LRITF_ID}:article:54"),
            format!("{LRITF_ID}:article:56"),
            format!("{LRITF_ID}:transitory:octava"),
        ] {
            known_targets.insert(target);
        }
        let references = extract_references(
            &provisions,
            Some((DCG_ID, DCG_TITLE)),
            &dcg_reference_options(),
            &known_targets,
        )
        .unwrap();
        (provisions, references, known_targets)
    }

    #[test]
    fn resolves_cross_instrument_and_title_references_deterministically() {
        let (_provisions, references, _known_targets) = dcg_fixture_graph();

        // Title citations resolve against LRITF with their paragraph
        // qualifiers preserved.
        let title_edges: Vec<_> = references
            .iter()
            .filter(|edge| edge.source_provision_id == DCG_ID)
            .collect();
        assert_eq!(title_edges.len(), 3);
        assert_eq!(
            title_edges
                .iter()
                .map(|edge| edge.target_provision_id.as_str())
                .collect::<Vec<_>>(),
            [
                format!("{LRITF_ID}:article:48"),
                format!("{LRITF_ID}:article:54"),
                format!("{LRITF_ID}:article:56"),
            ]
        );
        assert_eq!(title_edges[0].qualifiers[0].text, "segundo párrafo");
        assert_eq!(
            title_edges[2].qualifiers[0].text,
            "primer y segundo párrafos"
        );
        assert!(
            title_edges
                .iter()
                .all(|edge| edge.resolution_status == ReferenceResolutionStatus::Resolved)
        );

        // A full-name LRITF citation inside Article 1 resolves cross-instrument.
        assert!(references.iter().any(|edge| {
            edge.source_provision_id.ends_with(":article:1")
                && edge.target_provision_id == format!("{LRITF_ID}:article:48")
                && edge.resolution_status == ReferenceResolutionStatus::Resolved
        }));

        // The short-form defined-term citation "artículo 22, fracción I de la
        // Ley," must not create any edge (regression: it previously became a
        // false internal edge).
        assert!(
            !references
                .iter()
                .any(|edge| edge.target_provision_id.ends_with(":article:22"))
        );

        // Internal citations with DCG markers stay internal.
        assert!(references.iter().any(|edge| {
            edge.source_provision_id.ends_with(":transitory:segundo")
                && edge.target_provision_id == format!("{DCG_ID}:article:15")
        }));

        // CUARTO cites LRITF's OCTAVA transitory expressly.
        let octava = references
            .iter()
            .find(|edge| edge.target_provision_id == format!("{LRITF_ID}:transitory:octava"))
            .expect("transitory citation extracted");
        assert!(octava.source_provision_id.ends_with(":transitory:cuarto"));
        assert_eq!(octava.source_span, "OCTAVA Transitoria");
        assert_eq!(
            octava.resolution_status,
            ReferenceResolutionStatus::Resolved
        );

        // Named external laws that are not configured stay unlinked.
        assert!(
            !references
                .iter()
                .any(|edge| edge.target_provision_id.contains("codigo"))
        );
    }

    #[test]
    fn captures_pre_number_qualifiers_and_same_article_fractions() {
        let (provisions, references, known_targets) = dcg_fixture_graph();

        // A qualifier written before the number — `la fracción XI del
        // artículo 36` in Anexo 1 — is captured with its anchored span.
        let annex_citation = references
            .iter()
            .find(|edge| {
                edge.source_provision_id.ends_with(":annex:1")
                    && edge.target_provision_id.ends_with(":article:36")
            })
            .expect("annex 1 cites article 36");
        assert_eq!(annex_citation.qualifiers.len(), 1);
        let qualifier = &annex_citation.qualifiers[0];
        assert_eq!(qualifier.text, "fracción XI");
        assert_eq!(qualifier.qualifier_type, ReferenceQualifierType::Fraction);
        let (start, end) = (
            qualifier.start_char.expect("anchored"),
            qualifier.end_char.expect("anchored"),
        );
        let annex_text = &provisions
            .iter()
            .find(|item| item.id.ends_with(":annex:1"))
            .expect("annex 1")
            .text;
        let span: String = annex_text.chars().skip(start).take(end - start).collect();
        assert_eq!(span, "fracción XI");

        // `las fracciones III y IV de este artículo` in Article 36 becomes
        // one self-targeting edge per numeral, each spanning the numeral.
        let same_article: Vec<_> = references
            .iter()
            .filter(|edge| {
                edge.source_provision_id.ends_with(":article:36")
                    && edge.target_provision_id == edge.source_provision_id
            })
            .collect();
        assert_eq!(
            same_article
                .iter()
                .map(|edge| edge.source_span.as_str())
                .collect::<Vec<_>>(),
            ["III", "IV"]
        );
        assert!(same_article.iter().all(|edge| {
            edge.resolution_status == ReferenceResolutionStatus::Resolved
                && edge.qualifiers.len() == 1
        }));

        // The full graph passes validation, including qualifier spans.
        let report = super::validate_corpus(
            &super::CorpusView {
                instrument_id: DCG_ID,
                official_title: Some(DCG_TITLE),
                provisions: &provisions,
                references: &references,
                terms: &[],
                term_usages: &[],
                amendment_references: &[],
            },
            &super::CorpusExpectations::default(),
            &known_targets,
            &HashSet::new(),
        );
        // The fixture is an excerpt, so count/order/unresolved-target
        // issues are expected; span integrity must hold regardless.
        assert!(
            !report.issues.iter().any(|issue| {
                matches!(
                    issue.code.as_str(),
                    "qualifier_span_mismatch"
                        | "reference_span_mismatch"
                        | "reference_offsets_invalid"
                )
            }),
            "{:?}",
            report.issues
        );
    }

    #[test]
    fn parses_articles_and_statute_transitories_without_page_furniture() {
        let date = NaiveDate::from_ymd_opt(2018, 3, 9).unwrap();
        let provisions = parse_lritf(FIXTURE, date).unwrap();

        assert_eq!(provisions.len(), 4);
        assert_eq!(
            provisions[0].id,
            "urn:lex-mx:federal:statute:lritf:article:1"
        );
        assert_eq!(
            provisions[0].heading_context.title.as_deref(),
            Some("Título I")
        );
        assert_eq!(provisions[1].text, "Texto inicial que cruza una página.");
        assert_eq!(
            provisions[2].id,
            "urn:lex-mx:federal:statute:lritf:transitory:primera"
        );
        assert!(
            !provisions
                .iter()
                .any(|item| item.text.contains("CÁMARA DE DIPUTADOS"))
        );
        assert!(
            !provisions
                .iter()
                .any(|item| item.text.contains("reforma posterior"))
        );
    }

    #[test]
    fn validates_expected_counts_and_order() {
        let date = NaiveDate::from_ymd_opt(2018, 3, 9).unwrap();
        let provisions = parse_lritf(FIXTURE, date).unwrap();
        let report = validate_lritf(&provisions, &[], 2, 2);
        assert!(report.valid, "{:?}", report.issues);
    }

    const GAPPED_CODIGO: &str = "Artículo 1o.- Uno.\n\nArtículo 2o.- Dos.\n\nArtículo 15.- Quince.\n\nArtículo 15-A.- Quince A.\n\nArtículo 17.- Diecisiete.\n\nTRANSITORIOS\n\nPRIMERO.- Entrará en vigor.\n";

    fn validate_gapped(expectations: &crate::CorpusExpectations) -> lex_core::ValidationReport {
        let date = NaiveDate::from_ymd_opt(1981, 12, 31).unwrap();
        let options = DiputadosOptions {
            instrument_id: "urn:lex-mx:federal:code:sample".to_owned(),
            header_lines: vec!["CÓDIGO DE MUESTRA".to_owned()],
            stop_markers: Vec::new(),
        };
        let document = parse_diputados(GAPPED_CODIGO, &options, date).unwrap();
        crate::validate_corpus(
            &crate::CorpusView {
                instrument_id: "urn:lex-mx:federal:code:sample",
                official_title: None,
                provisions: &document.provisions,
                references: &[],
                terms: &[],
                term_usages: &[],
                amendment_references: &[],
            },
            expectations,
            &std::collections::HashSet::new(),
            &std::collections::HashSet::new(),
        )
    }

    #[test]
    fn gap_tolerant_ordering_accepts_suffixes_and_warns_on_gaps() {
        let report = validate_gapped(&crate::CorpusExpectations {
            min_articles: Some(5),
            articles: Some(5),
            transitories: Some(1),
            annexes: 0,
            require_chapter_context: false,
            allow_article_gaps: true,
        });
        assert!(report.valid, "{:?}", report.issues);
        let gaps: Vec<&str> = report
            .issues
            .iter()
            .filter(|issue| issue.code == "article_gap")
            .filter_map(|issue| issue.provision_id.as_deref())
            .collect();
        // 2o -> 15 and 15-A -> 17 skip base numbers; 15 -> 15-A does not.
        assert_eq!(
            gaps,
            [
                "urn:lex-mx:federal:code:sample:article:15",
                "urn:lex-mx:federal:code:sample:article:17"
            ]
        );
        assert!(
            !report
                .issues
                .iter()
                .any(|issue| issue.code == "non_numeric_article"),
            "suffixed articles are first-class under the label grammar"
        );
    }

    #[test]
    fn unfrozen_count_baselines_warn_without_failing() {
        let report = validate_gapped(&crate::CorpusExpectations {
            min_articles: None,
            articles: None,
            transitories: None,
            annexes: 0,
            require_chapter_context: false,
            allow_article_gaps: true,
        });
        assert!(report.valid, "{:?}", report.issues);
        assert_eq!(
            report
                .issues
                .iter()
                .filter(|issue| issue.code == "counts_not_frozen")
                .count(),
            2
        );
    }

    #[test]
    fn strict_ordering_still_rejects_gaps_when_not_allowed() {
        let report = validate_gapped(&crate::CorpusExpectations {
            min_articles: Some(5),
            articles: None,
            transitories: Some(1),
            annexes: 0,
            require_chapter_context: false,
            allow_article_gaps: false,
        });
        assert!(!report.valid, "{:?}", report.issues);
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.code == "article_order")
        );
    }

    #[test]
    fn extracts_compound_qualified_repeated_and_ranged_internal_references() {
        let date = NaiveDate::from_ymd_opt(2018, 3, 9).unwrap();
        let provisions = parse_lritf(REFERENCE_FIXTURE, date).unwrap();
        let references = extract_internal_references(&provisions).unwrap();

        let article_one: Vec<_> = references
            .iter()
            .filter(|edge| edge.source_provision_id.ends_with(":article:1"))
            .collect();
        assert_eq!(article_one.len(), 3);
        assert_eq!(article_one[0].source_span, "2");
        assert_eq!(
            article_one[0].qualifiers[0].qualifier_type,
            ReferenceQualifierType::Fraction
        );
        assert_eq!(article_one[0].qualifiers[0].text, "fracción II");
        assert_eq!(
            article_one[1].qualifiers[0].qualifier_type,
            ReferenceQualifierType::Paragraph
        );
        assert_eq!(article_one[1].qualifiers[0].text, "segundo párrafo");

        let article_eight: Vec<_> = references
            .iter()
            .filter(|edge| edge.source_provision_id.ends_with(":article:8"))
            .collect();
        assert_eq!(article_eight.len(), 4);
        assert!(article_eight.iter().any(|edge| {
            edge.target_provision_id.ends_with(":article:6")
                && edge.reference_form == ReferenceForm::RangeExpansion
                && edge.source_span == "5 al 7"
        }));
        assert!(
            references
                .iter()
                .all(|edge| !edge.target_provision_id.ends_with(":article:89"))
        );
        assert!(
            article_eight
                .iter()
                .any(|edge| edge.target_provision_id.ends_with(":article:6-bis"))
        );

        let repeated_article_two = references
            .iter()
            .filter(|edge| {
                edge.source_provision_id.ends_with(":transitory:primera")
                    && edge.target_provision_id.ends_with(":article:2")
            })
            .count();
        assert_eq!(repeated_article_two, 2);

        let report = validate_lritf(&provisions, &references, 8, 1);
        assert!(report.valid, "{:?}", report.issues);
        assert_eq!(report.reference_count, references.len());
    }

    #[test]
    fn resolves_relative_references_by_same_type_document_order() {
        let date = NaiveDate::from_ymd_opt(2018, 3, 9).unwrap();
        let provisions = parse_lritf(RELATIVE_FIXTURE, date).unwrap();
        let references = extract_internal_references(&provisions).unwrap();
        let relatives: Vec<_> = references
            .iter()
            .filter(|edge| edge.reference_form == ReferenceForm::Relative)
            .collect();

        // Article 1 has no previous article, so its `artículo anterior`
        // produces no edge; the plural `artículos anteriores` and the word
        // `anteriormente` never match.
        assert!(
            relatives
                .iter()
                .all(|edge| !edge.source_provision_id.ends_with(":article:1"))
        );
        assert_eq!(relatives.len(), 4);

        // Article 2: `anterior` resolves backward, `siguiente` forward,
        // and the ordinal-first pre-qualifier attaches to the edge.
        let article_two: Vec<_> = relatives
            .iter()
            .filter(|edge| edge.source_provision_id.ends_with(":article:2"))
            .collect();
        assert_eq!(article_two.len(), 2);
        assert!(article_two[0].target_provision_id.ends_with(":article:1"));
        assert_eq!(article_two[0].source_span, "artículo anterior");
        assert!(article_two[0].qualifiers.is_empty());
        assert!(article_two[1].target_provision_id.ends_with(":article:3"));
        assert_eq!(article_two[1].source_span, "artículo siguiente");
        assert_eq!(article_two[1].qualifiers.len(), 1);
        assert_eq!(article_two[1].qualifiers[0].text, "primer párrafo");

        // Article 3: `del citado artículo anterior` still resolves, but the
        // intervening word keeps the pre-qualifier from attaching.
        let article_three = relatives
            .iter()
            .find(|edge| edge.source_provision_id.ends_with(":article:3"))
            .unwrap();
        assert!(article_three.target_provision_id.ends_with(":article:2"));
        assert!(article_three.qualifiers.is_empty());

        // A relative reference inside a transitory resolves against the
        // transitory sequence, never the article sequence.
        let transitory = relatives
            .iter()
            .find(|edge| edge.source_provision_id.ends_with(":transitory:segunda"))
            .unwrap();
        assert!(
            transitory
                .target_provision_id
                .ends_with(":transitory:primera")
        );

        let report = validate_lritf(&provisions, &references, 4, 2);
        assert!(report.valid, "{:?}", report.issues);
    }

    #[test]
    fn captures_noun_first_and_penultimate_pre_qualifiers() {
        let date = NaiveDate::from_ymd_opt(2018, 3, 9).unwrap();
        let provisions = parse_lritf(RELATIVE_FIXTURE, date).unwrap();
        let references = extract_internal_references(&provisions).unwrap();

        // Noun-first form: `los párrafos segundo y tercero del artículo 2`.
        let noun_first = references
            .iter()
            .find(|edge| {
                edge.source_provision_id.ends_with(":article:4")
                    && edge.target_provision_id.ends_with(":article:2")
            })
            .unwrap();
        assert_eq!(noun_first.reference_form, ReferenceForm::Direct);
        assert_eq!(noun_first.qualifiers.len(), 1);
        assert_eq!(
            noun_first.qualifiers[0].qualifier_type,
            ReferenceQualifierType::Paragraph
        );
        assert_eq!(noun_first.qualifiers[0].text, "párrafos segundo y tercero");

        // Ordinal-first `penúltimo párrafo del artículo 1`.
        let penultimate = references
            .iter()
            .find(|edge| {
                edge.source_provision_id.ends_with(":article:3")
                    && edge.target_provision_id.ends_with(":article:1")
                    && edge.reference_form == ReferenceForm::Direct
            })
            .unwrap();
        assert_eq!(penultimate.qualifiers.len(), 1);
        assert_eq!(penultimate.qualifiers[0].text, "penúltimo párrafo");
    }

    #[test]
    fn extracts_reform_decree_transitories_as_separate_evidence() {
        let raw = r"
ARTÍCULOS TRANSITORIOS DE DECRETOS DE REFORMA

DECRETO por el que se reforman diversas disposiciones.
Publicado en el Diario Oficial de la Federación el 14 de noviembre de 2025

Transitorios

Primero. El presente Decreto entrará en vigor al día siguiente.

Segundo. La aplicación será gradual.

Ciudad de México, a 1 de octubre de 2025.
";
        let evidence = extract_reform_evidence(raw, &lritf_options()).unwrap();
        assert_eq!(evidence.len(), 2);
        assert_eq!(
            evidence[1].provision_id,
            "urn:lex-mx:federal:statute:lritf:amendment:2025-11-14:transitory:segundo"
        );
        assert_eq!(evidence[1].text, "La aplicación será gradual.");
    }
}
