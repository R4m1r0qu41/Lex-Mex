//! Deterministic source-fidelity evidence: does every committed provision
//! appear, token for token and in order, inside the text extracted from the
//! official source file?
//!
//! This module is pure (no I/O) and read-only. It never rewrites provision
//! text and never touches review state. Both sides are tokenised the same way
//! (split on Unicode whitespace and form feed; no case folding, no punctuation
//! stripping). Unicode normalisation is deliberately not applied: no
//! normalisation crate is a workspace dependency, and the committed provisions
//! and retained extractions were verified to be NFC already, so comparing as-is
//! is equivalent today. Any tolerance is a named, narrow gap class.

use std::{
    collections::{BTreeMap, HashMap},
    sync::LazyLock,
};

use regex::Regex;
use serde::{Deserialize, Serialize};

/// Version of the fidelity report schema (`schemas/fidelity-report-v1.schema.json`).
pub const FIDELITY_SCHEMA_VERSION: &str = "1.0.0";

/// Largest source span (in tokens) that may be skipped as a gap.
const MAX_GAP: usize = 160;
/// Largest provision span (in tokens) that may be skipped as an insertion.
const MAX_INSERT: usize = 40;
/// Number of following provision tokens that must match to resume a walk.
const RESYNC_WINDOW: usize = 4;
/// Candidate anchors tried at or after the running cursor.
const CURSOR_CANDIDATES: usize = 60;
/// Candidate anchors tried globally, matched on the first tokens.
const GLOBAL_CANDIDATES: usize = 40;
/// Leading provision tokens a global anchor must match.
const GLOBAL_KEY_TOKENS: usize = 6;
/// Tokens of the document opening treated as the running header.
const HEADER_TOKENS: usize = 12;
/// Gaps whose digit-normalised form repeats this often are page furniture.
const FURNITURE_MIN_REPEATS: usize = 3;
/// Reported gap and insertion text is truncated to this many characters.
const TEXT_LIMIT: usize = 300;
/// Context tokens reported before and after a divergence.
const CONTEXT_BEFORE: usize = 4;
const CONTEXT_AFTER: usize = 6;
const SOURCE_CONTEXT: usize = 8;

/// Per-provision outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvisionStatus {
    /// Every provision token matched the source contiguously.
    Exact,
    /// All tokens found in order; every skipped source span is a classified gap.
    ExactModuloClassifiedGaps,
    /// All tokens found in order, but a skipped source span is unclassified
    /// (possibly dropped text).
    UnclassifiedGap,
    /// At least one run of provision tokens is absent from the source.
    TextNotInSource,
    /// The walk could not resynchronise with the source.
    Diverged,
    /// No candidate anchor for the provision's first token exists.
    NotLocated,
    /// The provision has no tokens.
    Empty,
}

impl ProvisionStatus {
    /// Every status, in report order.
    pub const ALL: [Self; 7] = [
        Self::Exact,
        Self::ExactModuloClassifiedGaps,
        Self::UnclassifiedGap,
        Self::TextNotInSource,
        Self::Diverged,
        Self::NotLocated,
        Self::Empty,
    ];

    /// True for the two statuses that count as passing.
    #[must_use]
    pub fn passes(self) -> bool {
        matches!(self, Self::Exact | Self::ExactModuloClassifiedGaps)
    }
}

/// Narrow classes for source text that is not part of a provision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GapClass {
    /// Recurring running header, footer or page counter.
    PageFurniture,
    /// Cámara de Diputados editorial annotation naming a DOF reform.
    EditorialReformNote,
    /// A bare `(N)` amendment mark interleaved by a compiled document.
    AmendmentMark,
    /// Anything else: possible dropped text.
    Unclassified,
}

impl GapClass {
    /// Every class, in report order.
    pub const ALL: [Self; 4] = [
        Self::PageFurniture,
        Self::EditorialReformNote,
        Self::AmendmentMark,
        Self::Unclassified,
    ];
}

/// A span of source text between provision tokens.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GapRecord {
    pub class: GapClass,
    /// Offset of the first skipped token in the source document.
    pub source_token_offset: usize,
    pub token_count: usize,
    /// Skipped text, truncated to 300 characters.
    pub text: String,
}

/// A run of provision tokens with no counterpart in the source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InsertionRecord {
    pub provision_token_offset: usize,
    pub token_count: usize,
    /// Inserted text, truncated to 300 characters.
    pub text: String,
}

/// Where a walk stopped and what each side looked like there.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Divergence {
    pub provision_token_offset: usize,
    pub provision_context: String,
    pub source_context: String,
}

/// Fidelity evidence for one provision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvisionFidelity {
    pub provision_id: String,
    pub status: ProvisionStatus,
    /// `main` or `annex-<n>`.
    pub source_document: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_token_start: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_token_end: Option<usize>,
    pub provision_token_count: usize,
    #[serde(default)]
    pub gaps: Vec<GapRecord>,
    #[serde(default)]
    pub insertions: Vec<InsertionRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub divergence: Option<Divergence>,
}

/// Whether the retained inputs were fit to compare against.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputStatus {
    Ok,
    InputsMissing,
    SourceHashMismatch,
    ExtractionHashMismatch,
    AnnexExtractionHashMismatch,
    /// The committed corpus files could not be read (reported by the CLI).
    CorpusUnreadable,
}

/// Instrument-level facts a report needs besides the provision results.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportMeta {
    pub instrument_id: String,
    pub input_status: InputStatus,
    pub input_detail: Option<String>,
    pub source_file_present: bool,
    pub annex_extraction_hash_checked: bool,
    pub source_sha256: String,
    pub extracted_text_sha256: Option<String>,
    pub extraction_tool: Option<String>,
}

/// Per-instrument report (`.work/<slug>/fidelity-report.json`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FidelityReport {
    pub schema_version: String,
    pub instrument_id: String,
    pub input_status: InputStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_detail: Option<String>,
    pub source_file_present: bool,
    /// True only when at least one annex text was used and every used annex
    /// text was checked against `annex-source-manifests.json`.
    pub annex_extraction_hash_checked: bool,
    pub source_sha256: String,
    pub extracted_text_sha256: Option<String>,
    pub extraction_tool: Option<String>,
    pub provision_count: usize,
    pub status_counts: BTreeMap<ProvisionStatus, usize>,
    pub gap_class_counts: BTreeMap<GapClass, usize>,
    /// True only when the inputs were usable and every non-empty provision is
    /// `exact` or `exact_modulo_classified_gaps`.
    pub all_provisions_pass: bool,
    pub provisions: Vec<ProvisionFidelity>,
}

impl FidelityReport {
    /// Assemble a report, deriving every count from the provision results.
    #[must_use]
    pub fn new(meta: ReportMeta, provisions: Vec<ProvisionFidelity>) -> Self {
        let mut status_counts: BTreeMap<ProvisionStatus, usize> =
            ProvisionStatus::ALL.iter().map(|s| (*s, 0)).collect();
        let mut gap_class_counts: BTreeMap<GapClass, usize> =
            GapClass::ALL.iter().map(|c| (*c, 0)).collect();
        for provision in &provisions {
            *status_counts.entry(provision.status).or_default() += 1;
            for gap in &provision.gaps {
                *gap_class_counts.entry(gap.class).or_default() += 1;
            }
        }
        let all_provisions_pass = meta.input_status == InputStatus::Ok
            && provisions
                .iter()
                .all(|p| p.status == ProvisionStatus::Empty || p.status.passes());
        Self {
            schema_version: FIDELITY_SCHEMA_VERSION.to_owned(),
            instrument_id: meta.instrument_id,
            input_status: meta.input_status,
            input_detail: meta.input_detail,
            source_file_present: meta.source_file_present,
            annex_extraction_hash_checked: meta.annex_extraction_hash_checked,
            source_sha256: meta.source_sha256,
            extracted_text_sha256: meta.extracted_text_sha256,
            extraction_tool: meta.extraction_tool,
            provision_count: provisions.len(),
            status_counts,
            gap_class_counts,
            all_provisions_pass,
            provisions,
        }
    }

    /// Count of provisions with the given status.
    #[must_use]
    pub fn count(&self, status: ProvisionStatus) -> usize {
        self.status_counts.get(&status).copied().unwrap_or(0)
    }
}

/// Tokenise as the check does: Unicode whitespace and form feed separate
/// tokens; nothing else is altered.
#[must_use]
pub fn tokenize(text: &str) -> Vec<&str> {
    text.split(|c: char| c.is_whitespace() || c == '\u{c}')
        .filter(|token| !token.is_empty())
        .collect()
}

/// True for a bare amendment mark such as `(72)`.
#[must_use]
pub fn is_amendment_mark(token: &str) -> bool {
    token
        .strip_prefix('(')
        .and_then(|rest| rest.strip_suffix(')'))
        .is_some_and(|digits| !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
}

/// One source text, tokenised once and indexed by token.
#[derive(Debug)]
pub struct SourceDocument<'a> {
    tokens: Vec<&'a str>,
    index: HashMap<&'a str, Vec<usize>>,
}

impl<'a> SourceDocument<'a> {
    #[must_use]
    pub fn new(text: &'a str) -> Self {
        let tokens = tokenize(text);
        let mut index: HashMap<&'a str, Vec<usize>> = HashMap::new();
        for (offset, token) in tokens.iter().enumerate() {
            index.entry(token).or_default().push(offset);
        }
        Self { tokens, index }
    }

    #[must_use]
    pub fn tokens(&self) -> &[&'a str] {
        &self.tokens
    }
}

/// A provision to check.
#[derive(Debug, Clone, Copy)]
pub struct ProvisionInput<'a> {
    pub id: &'a str,
    pub text: &'a str,
}

#[derive(Debug, Clone, Copy)]
struct RawGap {
    offset: usize,
    len: usize,
    mark: bool,
}

#[derive(Debug, Clone, Copy)]
struct RawInsertion {
    offset: usize,
    len: usize,
}

#[derive(Debug)]
struct Walk {
    start: usize,
    end: usize,
    gaps: Vec<RawGap>,
    insertions: Vec<RawInsertion>,
    /// Provision token offset where the walk could not continue.
    divergence: Option<usize>,
}

impl Walk {
    fn accepted(&self) -> bool {
        self.divergence.is_none() && self.insertions.is_empty()
    }
}

/// Advance through the source and provision while tokens agree, absorbing
/// amendment marks, source gaps and provision insertions on a mismatch.
fn walk(source: &[&str], provision: &[&str], start: usize) -> Walk {
    let mut pos = start;
    let mut at = 0;
    let mut gaps = Vec::new();
    let mut insertions = Vec::new();
    while at < provision.len() {
        if source.get(pos) == Some(&provision[at]) {
            pos += 1;
            at += 1;
            continue;
        }
        if source
            .get(pos)
            .is_some_and(|token| is_amendment_mark(token))
        {
            gaps.push(RawGap {
                offset: pos,
                len: 1,
                mark: true,
            });
            pos += 1;
            continue;
        }
        let window = RESYNC_WINDOW.min(provision.len() - at);
        let wanted = &provision[at..at + window];
        if let Some(skip) =
            (1..MAX_GAP).find(|skip| source.get(pos + skip..pos + skip + window) == Some(wanted))
        {
            gaps.push(RawGap {
                offset: pos,
                len: skip,
                mark: false,
            });
            pos += skip;
            continue;
        }
        let inserted = (1..MAX_INSERT).find(|skip| {
            let window = RESYNC_WINDOW.min(provision.len().saturating_sub(at + skip));
            window > 0
                && source.get(pos..pos + window) == Some(&provision[at + skip..at + skip + window])
        });
        if let Some(skip) = inserted {
            insertions.push(RawInsertion {
                offset: at,
                len: skip,
            });
            at += skip;
            continue;
        }
        return Walk {
            start,
            end: pos,
            gaps,
            insertions,
            divergence: Some(at),
        };
    }
    Walk {
        start,
        end: pos,
        gaps,
        insertions,
        divergence: None,
    }
}

/// Locate a provision: candidate anchors are the next occurrences of its first
/// token at or after `cursor`, then occurrences of its first six tokens
/// anywhere. The first alignment with no insertion and no divergence wins;
/// otherwise the one with the furthest progress (earliest on ties).
fn align(document: &SourceDocument<'_>, provision: &[&str], cursor: usize) -> Option<Walk> {
    let positions = document.index.get(provision.first()?)?;
    let from = positions.partition_point(|&position| position < cursor);
    let near: Vec<usize> = positions
        .iter()
        .skip(from)
        .take(CURSOR_CANDIDATES)
        .copied()
        .collect();
    let mut best: Option<(usize, Walk)> = None;
    let mut consider = |candidate: usize| -> Option<Walk> {
        let attempt = walk(&document.tokens, provision, candidate);
        if attempt.accepted() {
            return Some(attempt);
        }
        let progress = attempt.divergence.unwrap_or(provision.len());
        if best
            .as_ref()
            .is_none_or(|(furthest, _)| progress > *furthest)
        {
            best = Some((progress, attempt));
        }
        None
    };
    for &candidate in &near {
        if let Some(found) = consider(candidate) {
            return Some(found);
        }
    }
    let key_len = GLOBAL_KEY_TOKENS.min(provision.len());
    let key = &provision[..key_len];
    let global = positions
        .iter()
        .filter(|&&candidate| {
            document.tokens.get(candidate..candidate + key_len) == Some(key)
                && !near.contains(&candidate)
        })
        .take(GLOBAL_CANDIDATES);
    for &candidate in global {
        if let Some(found) = consider(candidate) {
            return Some(found);
        }
    }
    best.map(|(_, attempt)| attempt)
}

static EDITORIAL_NOTE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"^(?:(?:Artículo|Párrafo|Fracción|Inciso|Numeral|Apartado|Base|Denominación|",
        r"Capítulo|Título|Sección|Reforma|Fe de erratas|Encabezado|Tabla|Cuota|Tarifa|",
        r"Nota|Subinciso|Regla|Epígrafe|Disposición|Libro)\b",
        r".{0,200}?(?:DOF|D\.O\.F\.)\s*\d{1,2}-\d{1,2}-\d{4}[\s,y\d-]*)+$"
    ))
    .expect("static editorial-note pattern is valid")
});

/// Provision structure that a recurring gap must not contain to be page
/// furniture: an article heading marker such as `163.-` or a repeal
/// placeholder such as `(Se deroga)`.
static PROVISION_STRUCTURE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:^|\s)\d+[\p{L}\d-]*\.-(?:\s|$)|\(Se derogan?\b")
        .expect("static provision-structure pattern is valid")
});

/// Replace every run of ASCII digits with `#`.
fn normalize_digits(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_digits = false;
    for character in text.chars() {
        if character.is_ascii_digit() {
            if !in_digits {
                out.push('#');
            }
            in_digits = true;
        } else {
            in_digits = false;
            out.push(character);
        }
    }
    out
}

fn truncate_chars(text: &str) -> String {
    text.chars().take(TEXT_LIMIT).collect()
}

/// True when the tokens open with a page counter such as `3 de 120`.
fn starts_with_page_counter(tokens: &[&str]) -> bool {
    let digits = |token: &&str| !token.is_empty() && token.bytes().all(|b| b.is_ascii_digit());
    matches!(tokens, [first, de, last, ..] if digits(first) && *de == "de" && digits(last))
}

struct GapClassifier {
    frequency: HashMap<String, usize>,
    header: Vec<String>,
}

impl GapClassifier {
    fn classify(&self, tokens: &[&str], mark: bool) -> GapClass {
        let joined = tokens.join(" ");
        if mark {
            return GapClass::AmendmentMark;
        }
        if EDITORIAL_NOTE.is_match(&joined) {
            return GapClass::EditorialReformNote;
        }
        let normalized = normalize_digits(&joined);
        // Neither recurrence nor a leading running header may excuse a skipped
        // article heading or repeal placeholder: those are provision text, not
        // page furniture.
        let furniture = !PROVISION_STRUCTURE.is_match(&joined)
            && (self.opens_with_running_header(tokens)
                || self
                    .frequency
                    .get(&normalized)
                    .is_some_and(|count| *count >= FURNITURE_MIN_REPEATS));
        if furniture {
            return GapClass::PageFurniture;
        }
        GapClass::Unclassified
    }

    fn opens_with_running_header(&self, tokens: &[&str]) -> bool {
        if self.header.is_empty() || !starts_with_page_counter(tokens) {
            return false;
        }
        let normalized: Vec<String> = tokens.iter().map(|t| normalize_digits(t)).collect();
        normalized
            .windows(self.header.len())
            .any(|window| window == self.header.as_slice())
    }
}

struct Located<'a> {
    id: &'a str,
    provision: Vec<&'a str>,
    walk: Walk,
}

enum Pending<'a> {
    Empty(&'a str),
    NotLocated(&'a str, usize),
    Located(Located<'a>),
}

/// Check every provision of one source document, in order. Annex texts are
/// separate documents: call once per document with its own `source_document`
/// label (`main` or `annex-<n>`).
///
/// A provision that diverges still advances the running cursor to where its
/// walk stopped, and a completed alignment advances it only when it started at
/// or after the cursor.
#[must_use]
pub fn check_document(
    document: &SourceDocument<'_>,
    source_document: &str,
    provisions: &[ProvisionInput<'_>],
) -> Vec<ProvisionFidelity> {
    let mut cursor = 0;
    let mut pending: Vec<Pending<'_>> = Vec::with_capacity(provisions.len());
    for input in provisions {
        let tokens = tokenize(input.text);
        if tokens.is_empty() {
            pending.push(Pending::Empty(input.id));
            continue;
        }
        let Some(found) = align(document, &tokens, cursor) else {
            pending.push(Pending::NotLocated(input.id, tokens.len()));
            continue;
        };
        if found.divergence.is_some() {
            cursor = cursor.max(found.end);
        } else if found.start >= cursor {
            cursor = found.end;
        }
        pending.push(Pending::Located(Located {
            id: input.id,
            provision: tokens,
            walk: found,
        }));
    }

    // Recurrence is a property of the whole document, so count it over the
    // completed (non-diverged) alignments before classifying anything.
    let mut frequency: HashMap<String, usize> = HashMap::new();
    for entry in &pending {
        if let Pending::Located(located) = entry
            && located.walk.divergence.is_none()
        {
            for gap in located.walk.gaps.iter().filter(|gap| !gap.mark) {
                let text = document.tokens[gap.offset..gap.offset + gap.len].join(" ");
                *frequency.entry(normalize_digits(&text)).or_default() += 1;
            }
        }
    }
    let classifier = GapClassifier {
        frequency,
        header: document
            .tokens
            .iter()
            .take(HEADER_TOKENS)
            .map(|token| normalize_digits(token))
            .collect(),
    };

    pending
        .into_iter()
        .map(|entry| match entry {
            Pending::Empty(id) => bare(id, ProvisionStatus::Empty, source_document, 0),
            Pending::NotLocated(id, count) => {
                bare(id, ProvisionStatus::NotLocated, source_document, count)
            }
            Pending::Located(located) => finish(&located, document, source_document, &classifier),
        })
        .collect()
}

fn bare(
    id: &str,
    status: ProvisionStatus,
    source_document: &str,
    token_count: usize,
) -> ProvisionFidelity {
    ProvisionFidelity {
        provision_id: id.to_owned(),
        status,
        source_document: source_document.to_owned(),
        source_token_start: None,
        source_token_end: None,
        provision_token_count: token_count,
        gaps: Vec::new(),
        insertions: Vec::new(),
        divergence: None,
    }
}

fn finish(
    located: &Located<'_>,
    document: &SourceDocument<'_>,
    source_document: &str,
    classifier: &GapClassifier,
) -> ProvisionFidelity {
    let walk = &located.walk;
    let gaps: Vec<GapRecord> = walk
        .gaps
        .iter()
        .map(|gap| {
            let tokens = &document.tokens[gap.offset..gap.offset + gap.len];
            GapRecord {
                class: classifier.classify(tokens, gap.mark),
                source_token_offset: gap.offset,
                token_count: gap.len,
                text: truncate_chars(&tokens.join(" ")),
            }
        })
        .collect();
    let insertions: Vec<InsertionRecord> = walk
        .insertions
        .iter()
        .map(|insertion| InsertionRecord {
            provision_token_offset: insertion.offset,
            token_count: insertion.len,
            text: truncate_chars(
                &located.provision[insertion.offset..insertion.offset + insertion.len].join(" "),
            ),
        })
        .collect();
    let divergence = walk.divergence.map(|at| {
        let from = at.saturating_sub(CONTEXT_BEFORE);
        let to = (at + CONTEXT_AFTER).min(located.provision.len());
        let source_end = (walk.end + SOURCE_CONTEXT).min(document.tokens.len());
        Divergence {
            provision_token_offset: at,
            provision_context: truncate_chars(&located.provision[from..to].join(" ")),
            source_context: truncate_chars(&document.tokens[walk.end..source_end].join(" ")),
        }
    });
    let status = if divergence.is_some() {
        ProvisionStatus::Diverged
    } else if !insertions.is_empty() {
        ProvisionStatus::TextNotInSource
    } else if gaps.iter().any(|gap| gap.class == GapClass::Unclassified) {
        ProvisionStatus::UnclassifiedGap
    } else if gaps.is_empty() {
        ProvisionStatus::Exact
    } else {
        ProvisionStatus::ExactModuloClassifiedGaps
    };
    ProvisionFidelity {
        provision_id: located.id.to_owned(),
        status,
        source_document: source_document.to_owned(),
        source_token_start: Some(walk.start),
        source_token_end: Some(walk.end),
        provision_token_count: located.provision.len(),
        gaps,
        insertions,
        divergence,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(source: &str, provisions: &[&str]) -> Vec<ProvisionFidelity> {
        let document = SourceDocument::new(source);
        let inputs: Vec<ProvisionInput<'_>> = provisions
            .iter()
            .enumerate()
            .map(|(n, text)| ProvisionInput {
                id: ["p0", "p1", "p2", "p3", "p4", "p5"][n],
                text,
            })
            .collect();
        check_document(&document, "main", &inputs)
    }

    fn words(prefix: &str, count: usize) -> String {
        (0..count)
            .map(|n| format!("{prefix}{n}"))
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[test]
    fn tokenizer_splits_on_whitespace_and_form_feed_only() {
        assert_eq!(
            tokenize("A\u{c}b\u{a0}c  D,e\n\tF."),
            vec!["A", "b", "c", "D,e", "F."]
        );
        assert_eq!(tokenize(" \u{c}\n"), Vec::<&str>::new());
        // No case folding and no normalisation.
        assert_ne!(tokenize("Ley"), tokenize("ley"));
    }

    #[test]
    fn amendment_mark_shape() {
        assert!(is_amendment_mark("(72)"));
        assert!(is_amendment_mark("(1)"));
        assert!(!is_amendment_mark("()"));
        assert!(!is_amendment_mark("(72"));
        assert!(!is_amendment_mark("(72)."));
        assert!(!is_amendment_mark("(a)"));
        assert!(!is_amendment_mark("72"));
    }

    #[test]
    fn exact_provision_is_exact() {
        let source = "Artículo 1.- uno dos tres cuatro cinco seis siete";
        let result = run(
            source,
            &["Artículo 1.- uno dos tres cuatro cinco seis siete"],
        );
        assert_eq!(result[0].status, ProvisionStatus::Exact);
        assert_eq!(result[0].source_token_start, Some(0));
        assert_eq!(result[0].source_token_end, Some(9));
        assert!(result[0].gaps.is_empty() && result[0].insertions.is_empty());
    }

    #[test]
    fn whitespace_layout_differences_are_not_gaps() {
        let source = "Artículo 1.-   uno\n   dos\u{c}tres cuatro cinco";
        let result = run(source, &["Artículo 1.- uno dos tres cuatro cinco"]);
        assert_eq!(result[0].status, ProvisionStatus::Exact);
    }

    #[test]
    fn empty_provision_is_empty() {
        let result = run("a b c", &["  \n "]);
        assert_eq!(result[0].status, ProvisionStatus::Empty);
        assert_eq!(result[0].provision_token_count, 0);
    }

    #[test]
    fn unknown_first_token_is_not_located() {
        let result = run("a b c d e", &["zeta eta theta iota kappa"]);
        assert_eq!(result[0].status, ProvisionStatus::NotLocated);
        assert_eq!(result[0].provision_token_count, 5);
        assert_eq!(result[0].source_token_start, None);
    }

    #[test]
    fn amendment_marks_are_classified_gaps() {
        let source = "Artículo 5.- uno dos (72) tres cuatro cinco seis (3) siete ocho";
        let result = run(
            source,
            &["Artículo 5.- uno dos tres cuatro cinco seis siete ocho"],
        );
        assert_eq!(result[0].status, ProvisionStatus::ExactModuloClassifiedGaps);
        assert_eq!(result[0].gaps.len(), 2);
        assert!(
            result[0]
                .gaps
                .iter()
                .all(|gap| gap.class == GapClass::AmendmentMark && gap.token_count == 1)
        );
        assert_eq!(result[0].gaps[0].text, "(72)");
    }

    #[test]
    fn editorial_reform_notes_are_classified_gaps() {
        for note in [
            "Párrafo reformado DOF 09-03-2018",
            "Artículo adicionado DOF 10-01-2014. Reformado DOF 09-03-2018",
            "Fe de erratas al artículo DOF 27-09-1933",
            "Fracción reformada DOF 01-02-2003, 04-05-2006 y 07-08-2009",
        ] {
            let source =
                format!("Artículo 9.- uno dos tres cuatro {note} cinco seis siete ocho nueve");
            let result = run(
                &source,
                &["Artículo 9.- uno dos tres cuatro cinco seis siete ocho nueve"],
            );
            assert_eq!(
                result[0].status,
                ProvisionStatus::ExactModuloClassifiedGaps,
                "{note}"
            );
            assert_eq!(result[0].gaps[0].class, GapClass::EditorialReformNote);
            assert_eq!(result[0].gaps[0].text, note);
        }
    }

    #[test]
    fn dropped_sentence_is_an_unclassified_gap() {
        let source =
            "Artículo 2.- uno dos tres cuatro Esta oración falta en la copia cinco seis siete ocho";
        let result = run(
            source,
            &["Artículo 2.- uno dos tres cuatro cinco seis siete ocho"],
        );
        assert_eq!(result[0].status, ProvisionStatus::UnclassifiedGap);
        assert_eq!(result[0].gaps[0].class, GapClass::Unclassified);
        assert_eq!(result[0].gaps[0].text, "Esta oración falta en la copia");
        assert!(!FidelityReport::new(meta(), result).all_provisions_pass);
    }

    #[test]
    fn unrelated_text_resembling_a_note_prefix_is_not_a_note() {
        let source = "Artículo 2.- uno dos tres cuatro Artículo texto suelto sin fecha cinco seis siete ocho";
        let result = run(
            source,
            &["Artículo 2.- uno dos tres cuatro cinco seis siete ocho"],
        );
        assert_eq!(result[0].status, ProvisionStatus::UnclassifiedGap);
    }

    #[test]
    fn inserted_phrase_is_text_not_in_source() {
        let source = "Artículo 3.- uno dos tres cuatro cinco seis siete ocho";
        let result = run(
            source,
            &["Artículo 3.- uno dos tres cuatro texto agregado aqui cinco seis siete ocho"],
        );
        assert_eq!(result[0].status, ProvisionStatus::TextNotInSource);
        assert_eq!(result[0].insertions.len(), 1);
        assert_eq!(result[0].insertions[0].text, "texto agregado aqui");
        assert_eq!(result[0].insertions[0].provision_token_offset, 6);
        assert_eq!(result[0].insertions[0].token_count, 3);
    }

    #[test]
    fn divergence_reports_offset_and_contexts() {
        let source = format!("Artículo 4.- uno dos tres {}", words("fuente", 200));
        let provision = format!("Artículo 4.- uno dos tres {}", words("otra", 200));
        let result = run(&source, &[provision.as_str()]);
        assert_eq!(result[0].status, ProvisionStatus::Diverged);
        let divergence = result[0].divergence.as_ref().unwrap();
        assert_eq!(divergence.provision_token_offset, 5);
        assert!(
            divergence
                .provision_context
                .starts_with("4.- uno dos tres otra0")
        );
        assert!(divergence.provision_context.contains("otra0"));
        assert!(divergence.source_context.starts_with("fuente0"));
    }

    #[test]
    fn repeated_gap_text_is_page_furniture() {
        let furniture = "Ley Ejemplar CÁMARA DE DIPUTADOS Última Reforma DOF";
        let source = format!(
            "Artículo 1.- a1 b1 c1 d1 {furniture} e1 f1 g1 h1 \
             Artículo 2.- a2 b2 c2 d2 {furniture} e2 f2 g2 h2 \
             Artículo 3.- a3 b3 c3 d3 {furniture} e3 f3 g3 h3"
        );
        let provisions = [
            "Artículo 1.- a1 b1 c1 d1 e1 f1 g1 h1",
            "Artículo 2.- a2 b2 c2 d2 e2 f2 g2 h2",
            "Artículo 3.- a3 b3 c3 d3 e3 f3 g3 h3",
        ];
        let result = run(&source, &provisions);
        for provision in &result {
            assert_eq!(provision.status, ProvisionStatus::ExactModuloClassifiedGaps);
            assert_eq!(provision.gaps[0].class, GapClass::PageFurniture);
        }
        // Two occurrences are not enough to call a span furniture.
        let result = run(&source, &provisions[..2]);
        assert_eq!(result[0].status, ProvisionStatus::UnclassifiedGap);
    }

    #[test]
    fn recurring_article_headings_and_repeal_placeholders_are_not_furniture() {
        let source = "Artículo 1.- a1 b1 c1 d1 Artículo 11.- (Se deroga) e1 f1 g1 h1 \
                      Artículo 2.- a2 b2 c2 d2 Artículo 12.- (Se deroga) e2 f2 g2 h2 \
                      Artículo 3.- a3 b3 c3 d3 Artículo 13.- (Se deroga) e3 f3 g3 h3";
        let result = run(
            source,
            &[
                "Artículo 1.- a1 b1 c1 d1 e1 f1 g1 h1",
                "Artículo 2.- a2 b2 c2 d2 e2 f2 g2 h2",
                "Artículo 3.- a3 b3 c3 d3 e3 f3 g3 h3",
            ],
        );
        for provision in &result {
            assert_eq!(provision.status, ProvisionStatus::UnclassifiedGap);
            assert_eq!(provision.gaps[0].class, GapClass::Unclassified);
        }
    }

    #[test]
    fn page_counters_with_changing_digits_still_count_as_repeats() {
        let source = "Artículo 1.- a1 b1 c1 d1 Pág 1 de 90 e1 f1 g1 h1 \
                      Artículo 2.- a2 b2 c2 d2 Pág 2 de 90 e2 f2 g2 h2 \
                      Artículo 3.- a3 b3 c3 d3 Pág 3 de 90 e3 f3 g3 h3";
        let result = run(
            source,
            &[
                "Artículo 1.- a1 b1 c1 d1 e1 f1 g1 h1",
                "Artículo 2.- a2 b2 c2 d2 e2 f2 g2 h2",
                "Artículo 3.- a3 b3 c3 d3 e3 f3 g3 h3",
            ],
        );
        assert!(
            result
                .iter()
                .all(|p| p.gaps[0].class == GapClass::PageFurniture)
        );
    }

    #[test]
    fn running_header_after_page_counter_is_furniture_even_once() {
        // The document opens with its running header; a single later page
        // break repeats `N de M` plus that header.
        let header = "LEY EJEMPLAR TEXTO VIGENTE CÁMARA DE DIPUTADOS DEL H. CONGRESO DE LA UNIÓN";
        let source = format!(
            "{header} Artículo 1.- a1 b1 c1 d1 7 de 90 {header} e1 f1 g1 h1 Artículo 2.- x y z w"
        );
        let result = run(&source, &["Artículo 1.- a1 b1 c1 d1 e1 f1 g1 h1"]);
        assert_eq!(result[0].status, ProvisionStatus::ExactModuloClassifiedGaps);
        assert_eq!(result[0].gaps[0].class, GapClass::PageFurniture);
        // A header-led gap that also swallows an article heading is not furniture.
        let source =
            format!("{header} Artículo 1.- a1 b1 c1 d1 7 de 90 {header} Artículo 2.- e1 f1 g1 h1");
        let result = run(&source, &["Artículo 1.- a1 b1 c1 d1 e1 f1 g1 h1"]);
        assert_eq!(result[0].status, ProvisionStatus::UnclassifiedGap);
        // The header without a page counter is not accepted on its own.
        let source = format!("{header} Artículo 1.- a1 b1 c1 d1 {header} e1 f1 g1 h1");
        let result = run(&source, &["Artículo 1.- a1 b1 c1 d1 e1 f1 g1 h1"]);
        assert_eq!(result[0].status, ProvisionStatus::UnclassifiedGap);
    }

    #[test]
    fn anchoring_prefers_the_next_occurrence_after_the_cursor() {
        let source = "Artículo 1.- a1 b1 c1 d1 e1 Artículo 1.- a1 b1 c1 d1 e1 zz";
        let result = run(
            source,
            &["Artículo 1.- a1 b1 c1 d1 e1", "Artículo 1.- a1 b1 c1 d1 e1"],
        );
        assert_eq!(result[0].source_token_start, Some(0));
        assert_eq!(result[1].source_token_start, Some(7));
    }

    #[test]
    fn anchoring_finds_a_provision_before_the_cursor_via_global_candidates() {
        // The second provision sits before the first one in the source; the
        // first token occurs only before the cursor.
        let source = "Artículo 8.- m1 m2 m3 m4 m5 Artículo 9.- n1 n2 n3 n4 n5";
        let result = run(
            source,
            &["Artículo 9.- n1 n2 n3 n4 n5", "Artículo 8.- m1 m2 m3 m4 m5"],
        );
        assert_eq!(result[1].status, ProvisionStatus::Exact);
        assert_eq!(result[1].source_token_start, Some(0));
    }

    #[test]
    fn long_gap_is_not_skipped() {
        let source = format!(
            "Artículo 6.- uno dos tres cuatro {} cinco seis siete ocho",
            words("x", 200)
        );
        let result = run(
            &source,
            &["Artículo 6.- uno dos tres cuatro cinco seis siete ocho"],
        );
        assert_eq!(result[0].status, ProvisionStatus::Diverged);
    }

    #[test]
    fn gap_and_insertion_text_is_truncated_on_character_boundaries() {
        let long = words("áéíóú", 120);
        let source = format!("Artículo 7.- uno dos tres cuatro {long} cinco seis siete ocho");
        let result = run(
            &source,
            &["Artículo 7.- uno dos tres cuatro cinco seis siete ocho"],
        );
        assert_eq!(result[0].gaps[0].text.chars().count(), 300);
        assert_eq!(result[0].gaps[0].token_count, 120);
    }

    #[test]
    fn provision_ending_inside_a_gap_window_does_not_panic() {
        let result = run("Artículo 1.- uno", &["Artículo 1.- uno dos tres"]);
        assert_eq!(result[0].status, ProvisionStatus::Diverged);
        let result = run("Artículo 1.- uno", &["Artículo"]);
        assert_eq!(result[0].status, ProvisionStatus::Exact);
    }

    fn meta() -> ReportMeta {
        ReportMeta {
            instrument_id: "urn:test".to_owned(),
            input_status: InputStatus::Ok,
            input_detail: None,
            source_file_present: true,
            annex_extraction_hash_checked: false,
            source_sha256: "0".repeat(64),
            extracted_text_sha256: Some("1".repeat(64)),
            extraction_tool: Some("pdftotext".to_owned()),
        }
    }

    #[test]
    fn report_counts_and_pass_flag_follow_the_provisions() {
        let source = "Artículo 1.- a1 b1 c1 d1 (4) e1 Artículo 2.- a2 b2 c2 d2 e2";
        let result = run(
            source,
            &[
                "Artículo 1.- a1 b1 c1 d1 e1",
                "Artículo 2.- a2 b2 c2 d2 e2",
                "",
            ],
        );
        let report = FidelityReport::new(meta(), result);
        assert_eq!(report.provision_count, 3);
        assert_eq!(report.count(ProvisionStatus::ExactModuloClassifiedGaps), 1);
        assert_eq!(report.count(ProvisionStatus::Exact), 1);
        assert_eq!(report.count(ProvisionStatus::Empty), 1);
        assert_eq!(report.status_counts.len(), ProvisionStatus::ALL.len());
        assert_eq!(report.gap_class_counts[&GapClass::AmendmentMark], 1);
        assert!(report.all_provisions_pass);

        let mut failed = meta();
        failed.input_status = InputStatus::ExtractionHashMismatch;
        assert!(!FidelityReport::new(failed, Vec::new()).all_provisions_pass);
    }
}
