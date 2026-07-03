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
    Basis, HeadingContext, LRITF_INSTRUMENT_ID, Provision, ProvisionType, ReferenceEdge,
    ReferenceForm, ReferenceQualifier, ReferenceQualifierType, ReferenceResolutionStatus,
    ReviewStatus, SCHEMA_VERSION, Severity, TemporalEvidence, TemporalStatus, ValidationIssue,
    ValidationReport,
};
use regex::Regex;

pub mod dcg;
pub mod html;

pub use dcg::parse_dcg;
pub use html::extract_html_text;

const SOURCE_HEADER: &str = "LEY PARA REGULAR LAS INSTITUCIONES DE TECNOLOGÍA FINANCIERA";
const TRANSITORY_ORDINALS: &[&str] = &[
    "PRIMERA",
    "SEGUNDA",
    "TERCERA",
    "CUARTA",
    "QUINTA",
    "SEXTA",
    "SÉPTIMA",
    "OCTAVA",
    "NOVENA",
    "DÉCIMA",
    "DÉCIMA PRIMERA",
];

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

pub fn parse_lritf(raw: &str, publication_date: NaiveDate) -> Result<Vec<Provision>> {
    let article_re = Regex::new(r"^Artículo\s+(\d+(?:\s+(?:Bis|Ter|Quáter))?)\.-\s*(.*)$")?;
    let title_re = Regex::new(r"^TÍTULO\s+([IVXLCDM]+)$")?;
    let chapter_re = Regex::new(r"^CAPÍTULO\s+([IVXLCDM]+)$")?;
    let blocks = normalized_blocks(raw);

    let mut provisions = Vec::new();
    let mut current: Option<ProvisionBuilder> = None;
    let mut current_title: Option<String> = None;
    let mut current_chapter: Option<String> = None;
    let mut in_statute_transitories = false;

    for block in blocks {
        if block.starts_with("ARTÍCULOS SEGUNDO A DÉCIMO") {
            break;
        }
        if block == "DISPOSICIONES TRANSITORIAS" {
            if let Some(builder) = current.take() {
                provisions.push(builder.finish(publication_date));
            }
            in_statute_transitories = true;
            current_title = None;
            current_chapter = None;
            continue;
        }

        if !in_statute_transitories {
            if let Some(captures) = title_re.captures(&block) {
                if let Some(builder) = current.take() {
                    provisions.push(builder.finish(publication_date));
                }
                current_title = Some(format!("Título {}", &captures[1]));
                current_chapter = None;
                continue;
            }
            if let Some(captures) = chapter_re.captures(&block) {
                if let Some(builder) = current.take() {
                    provisions.push(builder.finish(publication_date));
                }
                current_chapter = Some(format!("Capítulo {}", &captures[1]));
                continue;
            }
            if let Some(captures) = article_re.captures(&block) {
                if let Some(builder) = current.take() {
                    provisions.push(builder.finish(publication_date));
                }
                let number = captures[1].to_owned();
                current = Some(ProvisionBuilder::article(
                    number,
                    captures[2].trim(),
                    current_title.clone(),
                    current_chapter.clone(),
                ));
                continue;
            }
        } else if let Some((ordinal, body)) = parse_transitory_start(&block) {
            if let Some(builder) = current.take() {
                provisions.push(builder.finish(publication_date));
            }
            current = Some(ProvisionBuilder::transitory(ordinal, body));
            continue;
        }

        if let Some(builder) = &mut current {
            builder.push_block(&block);
        }
    }

    if let Some(builder) = current {
        provisions.push(builder.finish(publication_date));
    }
    if provisions.is_empty() {
        bail!("no LRITF provisions recognized");
    }
    Ok(provisions)
}

struct ReferencePatterns {
    article: Regex,
    number: Regex,
    separator: Regex,
    paragraph: Regex,
    fraction: Regex,
    subsection: Regex,
    transitory_citation: Regex,
}

impl ReferencePatterns {
    fn new() -> Result<Self> {
        Ok(Self {
            article: Regex::new(r"(?i)\bartículos?\s+")?,
            transitory_citation: Regex::new(
                r"(?i)\bdisposici(?:ón|ones)\s+(PRIMERA|SEGUNDA|TERCERA|CUARTA|QUINTA|SEXTA|SÉPTIMA|OCTAVA|NOVENA|DÉCIMA(?:\s+PRIMERA)?)\s+Transitorias?",
            )?,
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
    };
    let target_ids: HashSet<String> = provisions.iter().map(|item| item.id.clone()).collect();
    extract_references(provisions, None, &options, &target_ids)
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
    let mut references = Vec::new();
    if let Some((instrument_id, official_title)) = title_source {
        references.extend(extract_source_references(
            ReferenceSource {
                source_id: instrument_id,
                instrument_id,
                text: official_title,
            },
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
    references
}

fn extract_reference_group(
    source: ReferenceSource<'_>,
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

fn direct_reference_edges(
    source: ReferenceSource<'_>,
    target_instrument_id: &str,
    group_start: usize,
    group: &str,
    accepted: &[regex::Match<'_>],
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
            reference_edge(
                source,
                number_match.as_str(),
                (group_start + number_match.start())..(group_start + number_match.end()),
                canonical_article_id(target_instrument_id, number_match.as_str()),
                extract_qualifiers(
                    qualifier_text,
                    &patterns.paragraph,
                    &patterns.fraction,
                    &patterns.subsection,
                ),
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
    let canonical_number = number
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
        .to_lowercase();
    format!("{instrument_id}:article:{canonical_number}")
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

fn extract_qualifiers(
    value: &str,
    paragraph_re: &Regex,
    fraction_re: &Regex,
    subsection_re: &Regex,
) -> Vec<ReferenceQualifier> {
    let mut matches = Vec::new();
    matches.extend(paragraph_re.find_iter(value).map(|item| {
        (
            item.start(),
            ReferenceQualifier {
                qualifier_type: ReferenceQualifierType::Paragraph,
                text: item.as_str().to_owned(),
            },
        )
    }));
    matches.extend(fraction_re.find_iter(value).map(|item| {
        (
            item.start(),
            ReferenceQualifier {
                qualifier_type: ReferenceQualifierType::Fraction,
                text: item.as_str().to_owned(),
            },
        )
    }));
    matches.extend(subsection_re.find_iter(value).map(|item| {
        (
            item.start(),
            ReferenceQualifier {
                qualifier_type: ReferenceQualifierType::Subsection,
                text: item.as_str().to_owned(),
            },
        )
    }));
    matches.sort_by_key(|(start, _)| *start);
    matches
        .into_iter()
        .map(|(_, qualifier)| qualifier)
        .collect()
}

fn qualifier_boundary(value: &str) -> usize {
    value
        .char_indices()
        .find_map(|(index, character)| matches!(character, '.' | '\n').then_some(index))
        .unwrap_or(value.len())
}

pub fn extract_reform_transitories(raw: &str) -> Result<Vec<TemporalEvidence>> {
    let publication_re = Regex::new(
        r"Publicado en el Diario Oficial de la Federación el (\d{1,2}) de ([a-z]+) de (\d{4})",
    )?;
    let ordinal_re = Regex::new(
        r"^(Primero|Segundo|Tercero|Cuarto|Quinto|Sexto|Séptimo|Octavo|Noveno|Décimo(?:\s+(?:Primero|Segundo|Tercero|Cuarto|Quinto|Sexto))?)\.(?:-)?\s*(.*)$",
    )?;
    let mut in_reform_appendix = false;
    let mut in_transitories = false;
    let mut publication_date: Option<NaiveDate> = None;
    let mut current: Option<ReformEvidenceBuilder> = None;
    let mut evidence = Vec::new();

    for block in normalized_blocks(raw) {
        if block.contains("ARTÍCULOS TRANSITORIOS DE DECRETOS DE REFORMA") {
            in_reform_appendix = true;
            continue;
        }
        if !in_reform_appendix {
            continue;
        }
        if block.starts_with("DECRETO por el que") {
            flush_reform_evidence(&mut current, &mut evidence);
            in_transitories = false;
        }
        if let Some(captures) = publication_re.captures(&block) {
            publication_date = spanish_date(&captures[1], &captures[2], &captures[3]);
            continue;
        }
        if block.eq_ignore_ascii_case("Transitorios") || block.ends_with(" Transitorios") {
            in_transitories = true;
            continue;
        }
        if !in_transitories {
            continue;
        }
        if block.starts_with("Ciudad de México") {
            flush_reform_evidence(&mut current, &mut evidence);
            in_transitories = false;
            continue;
        }
        if let Some(captures) = ordinal_re.captures(&block) {
            flush_reform_evidence(&mut current, &mut evidence);
            let date = publication_date
                .context("found reform transitory without its Diario Oficial publication date")?;
            current = Some(ReformEvidenceBuilder {
                date,
                ordinal: captures[1].to_owned(),
                blocks: vec![captures[2].trim().to_owned()],
            });
        } else if let Some(builder) = &mut current {
            builder.blocks.push(block);
        }
    }
    flush_reform_evidence(&mut current, &mut evidence);
    Ok(evidence)
}

#[derive(Debug, Clone, Default)]
pub struct CorpusExpectations {
    pub min_articles: usize,
    /// Exact article count for closed instruments.
    pub articles: Option<usize>,
    pub transitories: usize,
    pub annexes: usize,
    /// Require every article to carry chapter heading context.
    pub require_chapter_context: bool,
}

#[must_use]
pub fn validate_lritf(
    provisions: &[Provision],
    references: &[ReferenceEdge],
    expected_min_articles: usize,
    expected_transitories: usize,
) -> ValidationReport {
    validate_corpus(
        LRITF_INSTRUMENT_ID,
        None,
        provisions,
        references,
        &CorpusExpectations {
            min_articles: expected_min_articles,
            articles: None,
            transitories: expected_transitories,
            annexes: 0,
            require_chapter_context: false,
        },
        &HashSet::new(),
    )
}

/// Validate one instrument's canonical corpus. `official_title` anchors
/// reference edges whose source is the instrument itself (title citations).
/// `external_targets` holds canonical provision identifiers of every other
/// loaded instrument, so cross-instrument edges can be checked for existing
/// targets.
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn validate_corpus(
    instrument_id: &str,
    official_title: Option<&str>,
    provisions: &[Provision],
    references: &[ReferenceEdge],
    expectations: &CorpusExpectations,
    external_targets: &HashSet<String>,
) -> ValidationReport {
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

fn validate_counts(
    article_count: usize,
    transitory_count: usize,
    annex_count: usize,
    expectations: &CorpusExpectations,
    issues: &mut Vec<ValidationIssue>,
) {
    if article_count < expectations.min_articles {
        issues.push(error(
            "article_count",
            format!(
                "expected at least {} articles, found {article_count}",
                expectations.min_articles
            ),
            None,
        ));
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
    if transitory_count != expectations.transitories {
        issues.push(error(
            "transitory_count",
            format!(
                "expected {} transitories, found {transitory_count}",
                expectations.transitories
            ),
            None,
        ));
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

fn normalized_blocks(raw: &str) -> Vec<String> {
    let page_number = Regex::new(r"^\d+\s+de\s+\d+$").expect("static regex");
    let mut blocks = Vec::new();
    let mut current = String::new();
    let mut pending_blank = false;
    let mut crossed_page_furniture = false;

    for source_line in raw.lines() {
        let line = source_line.trim();
        if line.is_empty() {
            pending_blank = true;
            continue;
        }
        if is_page_furniture(line, &page_number) {
            if !current.is_empty() {
                crossed_page_furniture = true;
            }
            pending_blank = false;
            continue;
        }
        if pending_blank && !crossed_page_furniture {
            flush_block(&mut current, &mut blocks);
        }
        pending_blank = false;
        crossed_page_furniture = false;
        if is_immediate_structural(line) {
            flush_block(&mut current, &mut blocks);
            blocks.push(collapse_whitespace(line));
            continue;
        }
        if is_provision_start(line) {
            flush_block(&mut current, &mut blocks);
            current.push_str(line);
            continue;
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(line);
    }
    flush_block(&mut current, &mut blocks);
    blocks
}

fn is_page_furniture(line: &str, page_number: &Regex) -> bool {
    line == SOURCE_HEADER
        || line.starts_with("CÁMARA DE DIPUTADOS DEL H. CONGRESO DE LA UNIÓN")
        || line == "Secretaría General"
        || line == "Secretaría de Servicios Parlamentarios"
        || line.starts_with("Última Reforma DOF ")
        || page_number.is_match(line)
}

fn is_immediate_structural(line: &str) -> bool {
    line.starts_with("ARTÍCULOS SEGUNDO A DÉCIMO")
        || line.starts_with("TÍTULO ")
        || line.starts_with("CAPÍTULO ")
        || line == "DISPOSICIONES TRANSITORIAS"
}

fn is_provision_start(line: &str) -> bool {
    line.starts_with("Artículo ")
        || TRANSITORY_ORDINALS
            .iter()
            .any(|ordinal| line.starts_with(&format!("{ordinal}.-")))
}

fn flush_block(current: &mut String, blocks: &mut Vec<String>) {
    if !current.is_empty() {
        blocks.push(collapse_whitespace(current));
        current.clear();
    }
}

fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn parse_transitory_start(block: &str) -> Option<(&str, &str)> {
    TRANSITORY_ORDINALS.iter().find_map(|ordinal| {
        block
            .strip_prefix(&format!("{ordinal}.-"))
            .map(|body| (*ordinal, body.trim()))
    })
}

fn slug(value: &str) -> String {
    value
        .to_lowercase()
        .replace('á', "a")
        .replace('é', "e")
        .replace('í', "i")
        .replace('ó', "o")
        .replace(['ú', 'ü'], "u")
        .replace(' ', "-")
}

fn spanish_date(day: &str, month: &str, year: &str) -> Option<NaiveDate> {
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

struct ReformEvidenceBuilder {
    date: NaiveDate,
    ordinal: String,
    blocks: Vec<String>,
}

impl ReformEvidenceBuilder {
    fn finish(self) -> TemporalEvidence {
        let date = self.date.format("%Y-%m-%d");
        TemporalEvidence {
            provision_id: format!(
                "{LRITF_INSTRUMENT_ID}:amendment:{date}:transitory:{}",
                slug(&self.ordinal)
            ),
            label: format!("Transitorio {} — Decreto DOF {date}", self.ordinal),
            text: self
                .blocks
                .into_iter()
                .filter(|block| !block.is_empty())
                .collect::<Vec<_>>()
                .join("\n\n"),
        }
    }
}

fn flush_reform_evidence(
    current: &mut Option<ReformEvidenceBuilder>,
    evidence: &mut Vec<TemporalEvidence>,
) {
    if let Some(builder) = current.take() {
        evidence.push(builder.finish());
    }
}

struct ProvisionBuilder {
    provision_type: ProvisionType,
    number: String,
    label: String,
    title: Option<String>,
    chapter: Option<String>,
    blocks: Vec<String>,
}

impl ProvisionBuilder {
    fn article(
        number: String,
        initial: &str,
        title: Option<String>,
        chapter: Option<String>,
    ) -> Self {
        let mut value = Self {
            label: format!("Artículo {number}"),
            number,
            provision_type: ProvisionType::Article,
            title,
            chapter,
            blocks: Vec::new(),
        };
        value.push_block(initial);
        value
    }

    fn transitory(ordinal: &str, initial: &str) -> Self {
        let mut value = Self {
            label: ordinal.to_owned(),
            number: ordinal.to_owned(),
            provision_type: ProvisionType::Transitory,
            title: None,
            chapter: None,
            blocks: Vec::new(),
        };
        value.push_block(initial);
        value
    }

    fn push_block(&mut self, value: &str) {
        let value = value.trim();
        if !value.is_empty() {
            self.blocks.push(value.to_owned());
        }
    }

    fn finish(self, publication_date: NaiveDate) -> Provision {
        let kind = match self.provision_type {
            ProvisionType::Article => "article",
            ProvisionType::Transitory => "transitory",
            ProvisionType::Annex => "annex",
        };
        let canonical_number = if self.provision_type == ProvisionType::Article {
            self.number.to_lowercase().replace(' ', "-")
        } else {
            slug(&self.number)
        };
        Provision {
            schema_version: SCHEMA_VERSION.to_owned(),
            id: format!("{LRITF_INSTRUMENT_ID}:{kind}:{canonical_number}"),
            instrument_id: LRITF_INSTRUMENT_ID.to_owned(),
            provision_type: self.provision_type,
            label: self.label,
            number: self.number,
            heading_context: HeadingContext {
                title: self.title,
                chapter: self.chapter,
                section: None,
                apartado: None,
            },
            text: self.blocks.join("\n\n"),
            publication_date,
            effective_from: None,
            effective_to: None,
            temporal_status: TemporalStatus::Unknown,
            temporal_basis: None,
            temporal_confidence: None,
            review_status: ReviewStatus::NotAnalyzed,
            transitory_effects: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use chrono::NaiveDate;
    use lex_core::{ReferenceForm, ReferenceQualifierType, ReferenceResolutionStatus};
    use pretty_assertions::assert_eq;

    use super::{
        InstrumentContextPolicy, ReferenceOptions, extract_internal_references, extract_references,
        extract_reform_transitories, parse_dcg, parse_lritf, validate_lritf,
    };

    const FIXTURE: &str = include_str!("../../../fixtures/lritf/parser-sample.txt");
    const REFERENCE_FIXTURE: &str = include_str!("../../../fixtures/lritf/reference-sample.txt");
    const DCG_FIXTURE: &str = include_str!("../../../fixtures/ifpe-dcg-2021/parser-sample.txt");
    const DCG_ANNEX_FIXTURE: &str =
        include_str!("../../../fixtures/ifpe-dcg-2021/annex-sample.txt");
    const DCG_ID: &str = "urn:lex-mx:federal:regulation:ifpe-dcg-2021";
    const LRITF_ID: &str = "urn:lex-mx:federal:statute:lritf";

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
        }
    }

    #[test]
    fn resolves_cross_instrument_and_title_references_deterministically() {
        let date = NaiveDate::from_ymd_opt(2021, 1, 28).unwrap();
        let provisions = parse_dcg(
            DCG_FIXTURE,
            DCG_ANNEX_FIXTURE,
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
        let title = "Disposiciones aplicables a las instituciones de fondos de pago electrónico \
                     a que se refieren los artículos 48, segundo párrafo; 54, primer párrafo, y \
                     56, primer y segundo párrafos de la Ley para Regular las Instituciones de \
                     Tecnología Financiera";
        let references = extract_references(
            &provisions,
            Some((DCG_ID, title)),
            &dcg_reference_options(),
            &known_targets,
        )
        .unwrap();

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
        let evidence = extract_reform_transitories(raw).unwrap();
        assert_eq!(evidence.len(), 2);
        assert_eq!(
            evidence[1].provision_id,
            "urn:lex-mx:federal:statute:lritf:amendment:2025-11-14:transitory:segundo"
        );
        assert_eq!(evidence[1].text, "La aplicación será gradual.");
    }
}
