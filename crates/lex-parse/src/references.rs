//! Citation extraction and resolution, plus the options that govern it.

use std::{
    collections::{HashMap, HashSet},
    ops::Range,
};

use anyhow::Result;
use chrono::NaiveDate;
use lex_core::{
    Basis, Provision, ProvisionType, ReferenceEdge, ReferenceForm, ReferenceQualifier,
    ReferenceQualifierType, ReferenceResolutionStatus, ResolutionRule, SCHEMA_VERSION,
};
use regex::Regex;

use crate::labels;
use lex_grammar::{latest_dof_date, slug};

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
    /// Fraction phrase directly before a comma and the `inciso` it scopes.
    pre_fraction: Regex,
    /// Fraction citation of the containing article itself:
    /// `fracciones I, II, III y IV del presente artículo`.
    same_article_fraction: Regex,
    /// Position-relative citation of a neighboring provision: `artículo
    /// anterior`, `artículo siguiente`, or `artículo que precede`. Singular
    /// only — the plural (`los artículos anteriores`) names an open-ended set
    /// with no single deterministic target, so it stays unlinked.
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
                    incisos?\s+[a-z]\)?(?:\s*(?:,|y|o)\s*[a-z]\)?)*
                )\s+de(?:l|\s+los)\s*$",
            )?,
            pre_fraction: Regex::new(
                r"(?i)\b(fracci(?:ón|ones)\s+[IVXLCDM]+(?:\s*(?:,|y|o)\s*[IVXLCDM]+)*)\s*,\s*$",
            )?,
            same_article_fraction: Regex::new(
                r"(?i)\bfracci(?:ón|ones)\s+([IVXLCDM]+(?:\s*(?:,|y|o)\s*[IVXLCDM]+)*)\s+de(?:l\s+presente|\s+este)\s+artículo",
            )?,
            relative_article: Regex::new(r"(?i)\bartículo\s+(anterior|siguiente|que\s+precede)\b")?,
            roman: Regex::new(r"\b[IVXLCDM]+\b")?,
            number: Regex::new(
                // A hyphenated qualifier (`156-Bis`) is tried before the
                // single-letter suffix (`70-A`) so the letter rule does
                // not swallow the `B` of `-Bis`; a letter may still carry
                // a following space-separated qualifier (`15-B Bis`); and a
                // qualifier may carry a compound trailing number
                // (`95 Bis 3`, `168 Bis 10`, hyphenated `5 Bis-1`). The
                // qualifier must end at a word boundary so `136 tercer
                // párrafo` is not read as `136 Ter`.
                r"(?i)(?:\d{1,3}(?:,\d{3})+|\d{1,4})(?:-(?:Bis|Ter|Qu(?:[áa]ter|inquies|intus)|Sexies|Septies|Octies|Nonies|Decies|Undecies|Duodecies|Terdecies|Quaterdecies|Quinquiesdecies|Sexiesdecies|Septiesdecies|Octiesdecies|Noviesdecies|Vicies)\b(?:[\s-]\d{1,3})?|(?:-[A-Z])?(?:\s+(?:Bis|Ter|Qu(?:[áa]ter|inquies|intus)|Sexies|Septies|Octies|Nonies|Decies|Undecies|Duodecies|Terdecies|Quaterdecies|Quinquiesdecies|Sexiesdecies|Septiesdecies|Octiesdecies|Noviesdecies|Vicies)\b(?:[\s-]\d{1,3})?)?)",
            )?,
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
                r"(?i)\b(?:(?:(?:primer|primero|segundo|tercer|tercero|cuarto|quinto|sexto|séptimo|octavo|noveno|décimo|último)(?:\s+(?:,|y|o)\s+(?:primer|primero|segundo|tercer|tercero|cuarto|quinto|sexto|séptimo|octavo|noveno|décimo|último))*|\d{1,2}[oaºª]\.?)\s+párrafos?|párrafos?\s+(?:primero|segundo|tercero|cuarto|quinto|sexto|séptimo|octavo|noveno|décimo|último)(?:\s*(?:,|y|o)\s*(?:primero|segundo|tercero|cuarto|quinto|sexto|séptimo|octavo|noveno|décimo|último))*|párrafos?\s+\d{1,2}(?:\s*(?:,|y|o)\s*\d{1,2})*)\b",
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
    /// Superseded historical laws not present in the current corpus. Their
    /// express citations remain canonical but deliberately do not become live
    /// links to a later same-title law.
    pub historical_targets: Vec<HistoricalTarget>,
    /// A regulation's parent law. A bare `de la Ley` (or the quoted `“Ley”`)
    /// that is not followed by a title names it; the edge is emitted only when
    /// its target article exists.
    pub parent_instrument: Option<String>,
}

/// A superseded law cited under a title a later law now holds. Text with a DOF
/// amendment dated on or after `until` may cite the successor, so it is
/// resolved against `successor_id` instead of the historical identity.
#[derive(Debug, Clone)]
pub struct HistoricalTarget {
    pub id: String,
    pub successor_id: String,
    pub until: NaiveDate,
}

impl ReferenceOptions {
    /// The instrument a citation in `source_text` targets, and whether that
    /// target is a historical law with no live canonical text.
    fn target_for<'a>(&'a self, target: &'a str, source_text: &str) -> (&'a str, bool) {
        match self
            .historical_targets
            .iter()
            .find(|item| item.id == target)
        {
            None => (target, false),
            Some(historical)
                if latest_dof_date(source_text).is_some_and(|date| date >= historical.until) =>
            {
                (historical.successor_id.as_str(), false)
            }
            Some(_) => (target, true),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GroupTarget<'a> {
    Internal,
    External(&'a str),
    /// The regulation's parent law named by a bare `Ley`.
    Parent(&'a str),
    Skip,
}

pub fn extract_internal_references(provisions: &[Provision]) -> Result<Vec<ReferenceEdge>> {
    let options = ReferenceOptions {
        policy: InstrumentContextPolicy::WholeGroupPresence,
        transitory_citations: false,
        same_article_fractions: true,
        relative_references: true,
        historical_targets: Vec::new(),
        parent_instrument: None,
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
            options,
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
    options: &ReferenceOptions,
    known_targets: &HashSet<String>,
) -> Vec<ReferenceEdge> {
    let mut references = Vec::new();
    for captures in patterns.relative_article.captures_iter(source.text) {
        let phrase = captures.get(0).expect("relative match");
        let direction = captures.get(1).expect("direction capture");
        let context = &source.text[phrase.end()..];
        let boundary = instrument_context_boundary(context).min(
            patterns
                .article
                .find(context)
                .map_or(context.len(), |header| header.start()),
        );
        let context = &context[..boundary];
        // A comma, semicolon or colon closes the phrase naming the cited
        // article: "el artículo anterior, se estará a lo dispuesto en la Ley
        // Federal de Procedimiento..." names another law for a different
        // clause and does not govern the citation.
        let context = &context[..context.find([',', ';', ':']).unwrap_or(context.len())];
        // Only a genitive attached to the citation ("del artículo anterior de la
        // Ley X") names its governing instrument. A law mentioned later in the
        // sentence ("...artículo anterior serán sancionadas con arreglo
        // supletorio en la Ley X") belongs to another clause.
        let lead = context.trim_start().to_lowercase();
        let attached = lead.starts_with("de ") || lead.starts_with("del ");
        let target_context = match &options.policy {
            InstrumentContextPolicy::WholeGroupPresence => sentence_target(
                &context.to_lowercase(),
                MARKER_LOOKAHEAD_CHARS,
                &[
                    "de esta ley".to_owned(),
                    "de la presente ley".to_owned(),
                    "de este ordenamiento".to_owned(),
                    "del presente ordenamiento".to_owned(),
                ],
                &[],
                None,
            ),
            InstrumentContextPolicy::SentenceEarliestMarker { .. } => {
                group_target(context, MARKER_LOOKAHEAD_CHARS, &options.policy, None)
            }
        };
        let own_context = !attached
            || match target_context {
                GroupTarget::Internal => true,
                GroupTarget::External(id) => id == source.instrument_id,
                // A relative position in the parent law cannot be inferred
                // from this provision's neighbors.
                GroupTarget::Parent(_) => false,
                GroupTarget::Skip => bare_own_regulation_context(source.instrument_id, context),
            };
        if !own_context {
            // Relative positions in another instrument cannot be inferred from
            // this source provision's local neighbors.
            continue;
        }
        let target = if direction.as_str().eq_ignore_ascii_case("siguiente") {
            neighbors.next
        } else {
            neighbors.previous
        };
        let Some(target) = target else {
            continue;
        };
        let qualifiers = pre_number_qualifiers(source.text, phrase.start(), patterns);
        references.push(with_rule(
            reference_edge(
                source,
                phrase.as_str(),
                phrase.start()..phrase.end(),
                target.to_owned(),
                qualifiers,
                ReferenceForm::Relative,
                reference_resolution_status(target, known_targets, false),
            ),
            ResolutionRule::RelativePosition,
        ));
    }
    references
}

/// A regulation can refer to its own unnamed Reglamento. Do not generalize
/// this to statutes or named external regulations; a title continuation after
/// the noun is not the bare self-reference printed in REG-LGN article216.
fn bare_own_regulation_context(instrument_id: &str, context: &str) -> bool {
    if instrument_id.split(':').nth(3) != Some("regulation") {
        return false;
    }
    let lower = context.trim_start().to_lowercase();
    lower.strip_prefix("del reglamento").is_some_and(|tail| {
        let tail = tail.trim_start();
        tail.is_empty() || tail.starts_with([',', ';', ':', ')'])
    })
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
    let last_end = accepted.last().map(regex::Match::end);
    let context_end = last_end.map_or(group.len(), |end| match options.policy {
        InstrumentContextPolicy::WholeGroupPresence => group.len(),
        InstrumentContextPolicy::SentenceEarliestMarker { .. } => {
            end + instrument_context_boundary(&group[end..])
        }
    });
    let max_marker_start = last_end.map_or(usize::MAX, |end| end + MARKER_LOOKAHEAD_CHARS);
    let mut rule = ResolutionRule::OwnInstrumentDefault;
    let target_instrument_id = match group_target(
        &group[..context_end],
        max_marker_start,
        &options.policy,
        options.parent_instrument.as_deref(),
    ) {
        GroupTarget::Internal => source.instrument_id,
        GroupTarget::External(instrument_id) => {
            rule = ResolutionRule::NamedInstrument;
            instrument_id
        }
        GroupTarget::Parent(instrument_id) => {
            rule = ResolutionRule::ParentShorthand;
            instrument_id
        }
        GroupTarget::Skip => return Vec::new(),
    };
    let (target_instrument_id, historical) = options.target_for(target_instrument_id, source.text);
    let mut references = direct_reference_edges(
        source,
        target_instrument_id,
        group_start,
        group,
        &accepted,
        &pre_qualifiers,
        patterns,
        known_targets,
        historical,
    );
    references.extend(range_expansion_edges(
        source,
        target_instrument_id,
        group_start,
        group,
        &accepted,
        known_targets,
        historical,
    ));
    if rule == ResolutionRule::ParentShorthand {
        // A parent-shorthand citation is emitted only when its target
        // exists; a miss (renumbered or repealed parent article) is not an
        // edge and not a rejection.
        references.retain(|edge| edge.resolution_status == ReferenceResolutionStatus::Resolved);
    }
    let explicit_end = last_end.map_or(group.len(), |end| {
        end + instrument_context_boundary(&group[end..])
    });
    let explicit = explicit_internal_context(
        &group[..explicit_end],
        max_marker_start,
        &options.policy,
        source.instrument_id,
    );
    if rule == ResolutionRule::OwnInstrumentDefault && explicit {
        rule = ResolutionRule::OwnInstrumentMarker;
    }
    retain_internal_candidates(&mut references, source.instrument_id, explicit);
    for edge in &mut references {
        edge.resolution_rule = Some(rule);
    }
    references
}

fn with_rule(mut edge: ReferenceEdge, rule: ResolutionRule) -> ReferenceEdge {
    edge.resolution_rule = Some(rule);
    edge
}

/// Unclassified missing internal candidates are often unresolved external
/// citations. An explicit governing self marker instead records a broken
/// source pointer and must remain visible to validation.
fn retain_internal_candidates(references: &mut Vec<ReferenceEdge>, own_id: &str, explicit: bool) {
    references.retain(|edge| {
        explicit
            || edge.resolution_status != ReferenceResolutionStatus::Unresolved
            || edge.target_instrument_id != own_id
    });
}

fn explicit_internal_context(
    context: &str,
    max_marker_start: usize,
    policy: &InstrumentContextPolicy,
    own_id: &str,
) -> bool {
    match group_target(context, max_marker_start, policy, None) {
        GroupTarget::External(id) => return id == own_id,
        GroupTarget::Parent(_) | GroupTarget::Skip => return false,
        GroupTarget::Internal => {}
    }
    let lower = context.to_lowercase();
    match policy {
        InstrumentContextPolicy::WholeGroupPresence => [
            "de esta ley",
            "de la presente ley",
            "de este ordenamiento",
            "del presente ordenamiento",
        ]
        .iter()
        .any(|marker| {
            lower
                .find(marker)
                .is_some_and(|position| position <= max_marker_start)
        }),
        InstrumentContextPolicy::SentenceEarliestMarker {
            internal_markers, ..
        } => internal_markers.iter().any(|marker| {
            find_marker(&lower, marker).is_some_and(|position| position <= max_marker_start)
        }),
    }
}

fn group_target<'a>(
    context: &str,
    max_marker_start: usize,
    policy: &'a InstrumentContextPolicy,
    parent: Option<&'a str>,
) -> GroupTarget<'a> {
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
        } => sentence_target(
            &lower,
            max_marker_start,
            internal_markers,
            external_instruments,
            parent,
        ),
    }
}

/// First occurrence of a configured marker on lexical boundaries, so a longer
/// word or title (`Ley del Seguro Socialista`) never matches a shorter alias
/// (`ley del seguro social`).
fn find_marker(lower: &str, marker: &str) -> Option<usize> {
    lower
        .match_indices(marker)
        .find(|(position, _)| {
            let before = lower[..*position].chars().next_back();
            let after = &lower[position + marker.len()..];
            // A marker that carries its own boundary space needs no check on
            // that side; a plural (`constitucional` / `constitucionales`) is
            // still the same word.
            let before_ok = marker.starts_with(' ') || before.is_none_or(|c| !c.is_alphanumeric());
            let after_ok = marker.ends_with(' ')
                || after.chars().next().is_none_or(|c| !c.is_alphanumeric())
                || ["s", "es"].iter().any(|plural| {
                    after.strip_prefix(plural).is_some_and(|rest| {
                        rest.chars().next().is_none_or(|c| !c.is_alphanumeric())
                    })
                });
            before_ok && after_ok
        })
        .map(|(position, _)| position)
}

/// Position of the first bare parent-law marker: `de la Ley` (or `del
/// Código` when the parent is a code) not followed by a title, or the quoted
/// shorthand `de la “Ley”` / `de la "Ley"`.
fn bare_parent_marker(lower: &str, parent_id: &str) -> Option<usize> {
    let markers: &[&str] = if parent_id.contains(":code:") {
        &["del código", "del “código”", "del \"código\""]
    } else {
        &["de la ley", "de la “ley”", "de la \"ley\""]
    };
    markers
        .iter()
        .flat_map(|marker| {
            lower
                .match_indices(marker)
                .map(|(position, _)| (position, *marker))
        })
        .filter(|(position, marker)| {
            let before = lower[..*position].chars().next_back();
            if before.is_some_and(char::is_alphanumeric) {
                return false;
            }
            let quoted = marker.ends_with(['”', '"']);
            bare_word_follows(&lower[position + marker.len()..], quoted)
        })
        .map(|(position, _)| position)
        .min()
}

/// A bare `Ley` is followed by punctuation, the end of the text, or a plain
/// connective, never by the words of a title (`de la Ley de Pesca`), a title
/// opening with a relative clause (`de la Ley que Regula el Uso de la
/// Fuerza…`), an anaphoric `en cita`, or a dated reading of the law
/// (`de la Ley vigente hasta el 31 de diciembre de 2001`), which names another
/// law than the regulation's current parent.
fn bare_word_follows(rest: &str, quoted: bool) -> bool {
    let Some(first) = rest.chars().next() else {
        return true;
    };
    if !first.is_whitespace() {
        // An attached comma or parenthesis may open a dated or anaphoric
        // modifier (`Ley, vigente hasta…`); other punctuation ends the phrase.
        return match rest.strip_prefix([',', '(']) {
            Some(inner) => !excluded_reading(inner.trim_start()),
            None => !first.is_alphanumeric(),
        };
    }
    let rest = rest.trim_start();
    let Some(next) = rest.chars().next() else {
        return true;
    };
    if next == '(' || next == ',' {
        return !excluded_reading(rest[1..].trim_start());
    }
    if !next.is_alphabetic() {
        return true;
    }
    if excluded_reading(rest) {
        return false;
    }
    let word = rest
        .split(|c: char| !c.is_alphabetic())
        .find(|word| !word.is_empty())
        .unwrap_or_default();
    // A quoted `“Ley”` is already an explicit shorthand; an unquoted one must
    // be followed by a plain connective or a recognised bare continuation.
    quoted
        || match word {
            "que" | "en" | "vigente" => true,
            _ => [
                "y",
                "o",
                "e",
                "conforme",
                "aplicable",
                "cuando",
                "se",
                "así",
                "como",
                "sus",
                "su",
            ]
            .contains(&word),
        }
}

/// Readings that name another law than the current parent: a dated reading,
/// an anaphor to an earlier-cited law, or a title opening with a relative
/// clause.
fn excluded_reading(rest: &str) -> bool {
    let mut words = rest
        .split(|c: char| !c.is_alphabetic())
        .filter(|word| !word.is_empty());
    let word = words.next().unwrap_or_default();
    let second = words.next().unwrap_or_default();
    let third = words.next().unwrap_or_default();
    let dated = |w: &str| ["a", "al", "desde", "hasta", "durante", "en"].contains(&w);
    match word {
        "que" => {
            [
                "regula",
                "establece",
                "crea",
                "declara",
                "expide",
                "instituye",
                "adiciona",
                "deroga",
                "aprueba",
                "otorga",
                "fija",
                "norma",
                "modifica",
                "reforma",
                "estaba",
                "estuvo",
                "estuviera",
                "entonces",
            ]
            .contains(&second)
                || (second == "se"
                    && [
                        "cita", "menciona", "comenta", "invoca", "refiere", "señala", "alude",
                    ]
                    .contains(&third))
        }
        "en" => {
            ["cita", "comento", "cuestión", "referencia"].contains(&second)
                || (second == "vigor" && dated(third))
        }
        "vigente" => ["hasta", "desde", "a", "en", "durante", "al"].contains(&second),
        _ => false,
    }
}

fn sentence_target<'a>(
    lower: &str,
    max_marker_start: usize,
    internal_markers: &[String],
    external_instruments: &'a [(String, String)],
    parent: Option<&'a str>,
) -> GroupTarget<'a> {
    let internal = internal_markers
        .iter()
        .filter_map(|marker| find_marker(lower, marker.as_str()))
        .filter(|position| *position <= max_marker_start)
        .min();
    let configured = external_instruments
        .iter()
        .filter_map(|(marker, instrument_id)| {
            find_marker(lower, marker.as_str()).map(|position| (position, instrument_id.as_str()))
        })
        .filter(|(position, _)| *position <= max_marker_start)
        .min_by_key(|(position, _)| *position);
    let parent_position = parent.and_then(|id| bare_parent_marker(lower, id));
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
        .filter(|(position, _)| *position <= max_marker_start)
        // A bare parent-law marker is decided below, not skipped here.
        .filter(|(position, _)| parent_position != Some(*position))
        // A configured instrument name inside the same phrase (for example
        // "de la ley para regular…") supersedes the generic law marker.
        .filter(|(position, length)| {
            configured.is_none_or(|(configured_position, _)| {
                configured_position < *position || configured_position > position + length + 1
            })
        })
        .map(|(position, _)| position)
        .min();

    // The parent law governs a bare `Ley` (never one followed by a title)
    // only when the citation window has no internal marker (`de este
    // Reglamento`); a mixed citation is skipped, never guessed.
    // A configured title that opens inside the same governing phrase
    // (`de la Ley en Materia de …`) outranks its bare-parent prefix; a later,
    // separate law citation does not.
    let parent_candidate = parent
        .zip(parent_position)
        .filter(|(_, position)| *position <= max_marker_start)
        .filter(|(_, position)| {
            configured.is_none_or(|(configured_position, _)| {
                configured_position < *position || configured_position > position + 10
            })
        });
    let candidates = [
        internal.map(|position| (position, GroupTarget::Internal)),
        configured.map(|(position, id)| (position, GroupTarget::External(id))),
        generic.map(|position| (position, GroupTarget::Skip)),
        parent_candidate.map(|(id, position)| {
            (
                position,
                if internal.is_some() {
                    GroupTarget::Skip
                } else {
                    GroupTarget::Parent(id)
                },
            )
        }),
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
        let (target_instrument_id, rule) =
            match group_target(context, MARKER_LOOKAHEAD_CHARS, &options.policy, None) {
                GroupTarget::Internal => (
                    source.instrument_id,
                    if explicit_internal_context(
                        context,
                        MARKER_LOOKAHEAD_CHARS,
                        &options.policy,
                        source.instrument_id,
                    ) {
                        ResolutionRule::OwnInstrumentMarker
                    } else {
                        ResolutionRule::OwnInstrumentDefault
                    },
                ),
                GroupTarget::External(instrument_id) => {
                    (instrument_id, ResolutionRule::NamedInstrument)
                }
                GroupTarget::Parent(_) | GroupTarget::Skip => continue,
            };
        let (target_instrument_id, historical_target_unavailable) =
            options.target_for(target_instrument_id, source.text);
        let target_provision_id = format!(
            "{target_instrument_id}:transitory:{}",
            slug(ordinal.as_str())
        );
        let resolution_status = reference_resolution_status(
            &target_provision_id,
            known_targets,
            historical_target_unavailable,
        );
        if resolution_status == ReferenceResolutionStatus::Unresolved
            && target_instrument_id == source.instrument_id
            && !explicit_internal_context(
                context,
                MARKER_LOOKAHEAD_CHARS,
                &options.policy,
                source.instrument_id,
            )
        {
            continue;
        }
        references.push(with_rule(
            reference_edge(
                source,
                &source.text[ordinal.start()..citation_end],
                ordinal.start()..citation_end,
                target_provision_id,
                Vec::new(),
                ReferenceForm::Direct,
                resolution_status,
            ),
            rule,
        ));
    }
    references
}

/// A number written directly after `párrafo`/`numeral` is that qualifier's
/// number (`artículo 10, párrafo 2, inciso c)`), never another article.
fn names_qualifier_number(between: &str) -> bool {
    let lower = between.trim_end().to_lowercase();
    ["párrafo", "párrafos", "numeral", "numerales"]
        .iter()
        .any(|word| {
            lower.strip_suffix(word).is_some_and(|before| {
                before
                    .chars()
                    .next_back()
                    .is_none_or(|character| !character.is_alphabetic())
            })
        })
}

fn accepted_numbers<'a>(group: &'a str, patterns: &ReferencePatterns) -> Vec<regex::Match<'a>> {
    let candidates: Vec<_> = patterns.number.find_iter(group).collect();
    let Some(first) = candidates.first() else {
        return Vec::new();
    };
    // A flattened table can interleave a value cell between "artículo" and
    // its number (PEF Annex4: "artículo 0 74, fracción IV"). Reject this
    // malformed sequence; never repair it into an invented article74 edge.
    if first.as_str() == "0"
        && candidates.get(1).is_some_and(|next| {
            group[first.end()..next.start()]
                .chars()
                .all(char::is_whitespace)
                && next.as_str().parse::<u64>().is_ok_and(|n| n > 0)
        })
    {
        return Vec::new();
    }
    let leading = group[..first.start()].trim();
    if !leading.is_empty()
        && !leading.eq_ignore_ascii_case("constitucional")
        && !leading.eq_ignore_ascii_case("constitucionales")
    {
        return Vec::new();
    }
    let mut accepted = vec![*first];
    for candidate in candidates.iter().skip(1) {
        let Some(previous) = accepted.last() else {
            break;
        };
        let between = &group[previous.end()..candidate.start()];
        if !patterns.separator.is_match(between) || names_qualifier_number(between) {
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
    historical_target_unavailable: bool,
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
            let target_provision_id =
                canonical_article_id(target_instrument_id, number_match.as_str());
            let resolution_status = reference_resolution_status(
                &target_provision_id,
                known_targets,
                historical_target_unavailable,
            );
            reference_edge(
                source,
                number_match.as_str(),
                (group_start + number_match.start())..(group_start + number_match.end()),
                target_provision_id,
                qualifiers,
                ReferenceForm::Direct,
                resolution_status,
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
    historical_target_unavailable: bool,
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
            let target_provision_id =
                canonical_article_id(target_instrument_id, &expanded.to_string());
            let resolution_status = reference_resolution_status(
                &target_provision_id,
                known_targets,
                historical_target_unavailable,
            );
            references.push(reference_edge(
                source,
                source_span,
                range.clone(),
                target_provision_id,
                Vec::new(),
                ReferenceForm::RangeExpansion,
                resolution_status,
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
    resolution_status: ReferenceResolutionStatus,
) -> ReferenceEdge {
    let start_char = source.text[..source_range.start].chars().count();
    let end_char = start_char + source.text[source_range].chars().count();
    let target_slug = target_provision_id.rsplit(':').next().unwrap_or("unknown");
    let form_slug = match reference_form {
        ReferenceForm::Direct => "direct",
        ReferenceForm::RangeExpansion => "range",
        ReferenceForm::Relative => "relative",
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
        outdated_reference_approval: None,
        legal_operator_review: None,
        resolution_rule: None,
    }
}

fn reference_resolution_status(
    target_provision_id: &str,
    known_targets: &HashSet<String>,
    historical_target_unavailable: bool,
) -> ReferenceResolutionStatus {
    if historical_target_unavailable {
        ReferenceResolutionStatus::HistoricalTargetUnavailable
    } else if known_targets.contains(target_provision_id) {
        ReferenceResolutionStatus::Resolved
    } else {
        ReferenceResolutionStatus::Unresolved
    }
}

const EXTERNAL_INSTRUMENT_MARKERS: &[&str] = &[
    "de la ley ",
    // A regulation's quoted shorthand for its parent law ("artículo 122 de la
    // “Ley” y en la Ley Federal de Procedimiento Administrativo"): the quoted
    // word names the parent law, so the law named later must not take the
    // citation. The parent law is not linked from here.
    "de la “ley”",
    "de la \"ley\"",
    "del código ",
    "de la constitución ",
    // Adjectival Constitution reference: "el artículo 134 constitucional".
    " constitucional",
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
    // Instrument head nouns in title phrasing: a law named later in the title
    // ("Decreto por el que se abroga la Ley…") is part of that title, not the
    // cited law. Only unambiguous title openings are listed: bare "del
    // acuerdo" or "de la norma" are ordinary Spanish ("los términos del
    // acuerdo que suscriban") and must not hide an internal citation. A
    // configured title that starts at the noun still supersedes the marker.
    "del decreto por el que",
    "del decreto por el cual",
    "del decreto que",
    "del acuerdo por el que",
    "del acuerdo por el cual",
    "del acuerdo sobre",
    "del estatuto de",
    "de las reglas de carácter general",
    "de las reglas generales",
    "de los lineamientos para",
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
    let mut qualifiers = Vec::new();
    // `la fracción II, inciso a) del artículo 1`: an inciso written after a
    // fraction keeps the fraction scope as a second qualifier.
    if qualifier_type == ReferenceQualifierType::Subsection {
        let leading = &window[..phrase.start()];
        if let Some(fraction) = patterns.pre_fraction.captures(leading) {
            let fraction = fraction.get(1).expect("fraction capture");
            qualifiers.push(anchored_qualifier(
                source_text,
                window_start + fraction.start(),
                fraction.as_str(),
                ReferenceQualifierType::Fraction,
            ));
        }
    }
    qualifiers.push(anchored_qualifier(
        source_text,
        window_start + phrase.start(),
        phrase.as_str(),
        qualifier_type,
    ));
    qualifiers
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
            references.push(with_rule(
                reference_edge(
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
                    reference_resolution_status(source.source_id, known_targets, false),
                ),
                ResolutionRule::SameArticle,
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
    for (index, character) in value.char_indices() {
        if character == '\n' {
            return index;
        }
        if character != '.' {
            continue;
        }

        let before = &value[..index];
        let mut preceding = before.chars().rev();
        let ordinal_mark = preceding.next();
        let preceding_digit = preceding.next();
        let continues_numeric_ordinal =
            matches!(ordinal_mark, Some('o' | 'a' | 'O' | 'A' | 'º' | 'ª'))
                && preceding_digit.is_some_and(|character| character.is_ascii_digit());
        let continues_heading_ordinal = index == ordinal_mark.map_or(0, char::len_utf8)
            && matches!(ordinal_mark, Some('o' | 'a' | 'O' | 'A' | 'º' | 'ª'));
        if continues_numeric_ordinal || continues_heading_ordinal {
            continue;
        }
        return index;
    }
    value.len()
}

/// Find the sentence boundary used to classify a citation's governing
/// instrument. Diputados writes ordinal article numbers as `6o.` or `1a.`;
/// that dot belongs to the numeral and must not hide a following instrument
/// name such as `de la Constitución Política...`.
fn instrument_context_boundary(value: &str) -> usize {
    let end = qualifier_boundary(value);
    let head = &value[..end];
    // An independent semicolon clause ("…artículo 1; la Constitución reconoce
    // derechos") does not govern the citation. A semicolon that continues the
    // citation list (`1; 2 y 3`, `fracciones I; II`) does not close it.
    for (index, character) in head.char_indices() {
        if character == ';' && !continues_citation_list(head[index + 1..].trim_start()) {
            return index;
        }
    }
    end
}

fn continues_citation_list(rest: &str) -> bool {
    let lower = rest.to_lowercase();
    rest.chars().next().is_some_and(|c| c.is_ascii_digit())
        || ["fracci", "inciso", "párrafo", "y ", "o ", "artículo"]
            .iter()
            .any(|word| lower.starts_with(word))
        || {
            let roman = rest
                .chars()
                .take_while(|c| matches!(c, 'I' | 'V' | 'X' | 'L' | 'C' | 'D' | 'M'))
                .count();
            roman > 0
                && rest[roman..]
                    .chars()
                    .next()
                    .is_none_or(|c| !c.is_alphabetic())
        }
}

/// Maximum distance after the final citation number at which an instrument
/// marker may begin. Filtering the marker's start, rather than truncating the
/// sentence, preserves long official titles while preventing a later clause
/// from retargeting an earlier citation.
const MARKER_LOOKAHEAD_CHARS: usize = 90;

pub(crate) fn canonical_article_id(instrument_id: &str, number: &str) -> String {
    // Match the provision-id convention: ordinal marks are dropped so a
    // citation of "2" or "2o" both resolve to article "2".
    format!("{instrument_id}:article:{}", labels::canonical_slug(number))
}

pub(crate) fn numeric_article_number(number: &str) -> Option<u32> {
    number.trim().parse().ok()
}
