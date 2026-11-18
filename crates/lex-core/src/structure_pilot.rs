//! Fintech structural pilot, `pilot-v1` (G2): a derived, read-only structural
//! projection of one canonical provision text, and the lookup over it.
//!
//! The contract is the frozen G1 section of `docs/plans/fintech-structural-pilot.md`
//! (G1.1–G1.8); section references below point there. Nothing here reads or
//! writes canonical data: a projection is derived from `Provision.text` and is
//! bound to that text's hash, the projection version and the source release.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

use crate::{fraction_marker, fraction_qualifier_rank, normalize_fraction_label};

/// The projection version every binding carries (G1.2).
pub const PROJECTION_VERSION: &str = "pilot-v1";

/// Instruments whose articles are in scope (G1.1).
pub const ELIGIBLE_INSTRUMENTS: [&str; 3] = ["lritf", "lic", "lmv"];

const QUALIFIER_NAMES: [&str; 9] = [
    "Bis",
    "Ter",
    "Quater",
    "Quinquies",
    "Sexies",
    "Septies",
    "Octies",
    "Nonies",
    "Decies",
];

/// Dash and bullet characters that open an `unsupported_marker` (G1.3): the
/// hyphen and Unicode dashes, the minus sign, and common bullets.
const DASHES_AND_BULLETS: [char; 19] = [
    '-', '‐', '‑', '‒', '–', '—', '―', '−', '•', '◦', '▪', '‣', '·', '●', '○', '■', '□', '➢', '►',
];

const LETTERS: [char; 27] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'ñ', 'o', 'p', 'q', 'r',
    's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];

/// Whether a provision is in the pilot's scope (G1.1): an `article` of one of
/// [`ELIGIBLE_INSTRUMENTS`], identified by its instrument slug.
#[must_use]
pub fn is_eligible(instrument_slug: &str, provision_type: &str) -> bool {
    provision_type == "article" && ELIGIBLE_INSTRUMENTS.contains(&instrument_slug)
}

/// SHA-256 of the text's UTF-8 bytes, lowercase hex.
#[must_use]
pub fn text_sha256(text: &str) -> String {
    let mut out = String::with_capacity(64);
    for byte in Sha256::digest(text.as_bytes()) {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

/// The value of a Roman numeral written in canonical form (`IV`, not `IIII`).
#[must_use]
pub fn canonical_roman_value(numeral: &str) -> Option<u32> {
    fn digit(c: char) -> Option<u32> {
        Some(match c {
            'I' => 1,
            'V' => 5,
            'X' => 10,
            'L' => 50,
            'C' => 100,
            'D' => 500,
            'M' => 1000,
            _ => return None,
        })
    }
    let digits: Vec<u32> = numeral.chars().map(digit).collect::<Option<_>>()?;
    if digits.is_empty() {
        return None;
    }
    let mut total: i64 = 0;
    for (index, value) in digits.iter().enumerate() {
        let next = digits.get(index + 1).copied().unwrap_or(0);
        if next > *value {
            total -= i64::from(*value);
        } else {
            total += i64::from(*value);
        }
    }
    let total = u32::try_from(total).ok()?;
    (to_roman(total) == numeral).then_some(total)
}

fn to_roman(mut value: u32) -> String {
    let mut out = String::new();
    for (step, symbol) in [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ] {
        while value >= step {
            out.push_str(symbol);
            value -= step;
        }
    }
    out
}

/// A fraction label: base value and qualifier rank (0 for none), ordered as
/// `V` < `V Bis` < `VI` (G1.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct FractionLabel {
    value: u32,
    rank: u32,
}

impl FractionLabel {
    fn path(self) -> String {
        let mut out = format!("f.{}", to_roman(self.value));
        if let Some(name) = self
            .rank
            .checked_sub(1)
            .and_then(|index| QUALIFIER_NAMES.get(index as usize))
        {
            out.push('_');
            out.push_str(name);
        }
        out
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Class {
    Fraction {
        label: FractionLabel,
        label_len: usize,
    },
    Inciso {
        letter: usize,
    },
    Note,
    Unsupported {
        roman: Option<u32>,
        numeral_len: usize,
    },
    Content,
}

impl Class {
    fn is_marker(self) -> bool {
        matches!(self, Class::Fraction { .. } | Class::Inciso { .. })
    }
}

#[derive(Debug, Clone)]
struct Paragraph {
    start: usize,
    end: usize,
    class: Class,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NodeKind {
    Root,
    Fraction,
    Inciso,
}

impl NodeKind {
    fn as_str(self) -> &'static str {
        match self {
            NodeKind::Root => "article_root",
            NodeKind::Fraction => "fraction",
            NodeKind::Inciso => "inciso",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChildKind {
    Fraction,
    Inciso,
}

impl ChildKind {
    fn as_str(self) -> &'static str {
        match self {
            ChildKind::Fraction => "fraction",
            ChildKind::Inciso => "inciso",
        }
    }
}

#[derive(Debug, Clone)]
struct Node {
    kind: NodeKind,
    /// Marker paragraph index; `None` for the root.
    paragraph: Option<usize>,
    parent: Option<usize>,
    candidate_parents: Vec<usize>,
    tail: bool,
    path: String,
    fraction: Option<FractionLabel>,
    letter: Option<usize>,
    label_span: Option<(usize, usize)>,
    printed_label: Option<String>,
    normalised_label: Option<String>,
    ordinal: Option<usize>,
    region: (usize, usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum SegmentReason {
    TrailingAttachment,
    NoteInterruption,
    PostChildAttachment,
    UnsupportedMarker,
    SameLineMarker,
}

impl SegmentReason {
    fn as_str(self) -> &'static str {
        match self {
            SegmentReason::TrailingAttachment => "trailing_attachment",
            SegmentReason::NoteInterruption => "note_interruption",
            SegmentReason::PostChildAttachment => "post_child_attachment",
            SegmentReason::UnsupportedMarker => "unsupported_marker",
            SegmentReason::SameLineMarker => "same_line_marker",
        }
    }
}

#[derive(Debug, Clone)]
struct Segment {
    start: usize,
    end: usize,
    reason: SegmentReason,
    owners: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateLabel {
    Fraction(Option<FractionLabel>),
    Inciso(usize),
}

/// A same-line candidate (G1.3): recorded, never a node.
#[derive(Debug, Clone)]
struct SameLine {
    label: CandidateLabel,
    printed: String,
    span: (usize, usize),
    cut_end: usize,
    holder: Option<usize>,
    cuts: Vec<usize>,
    segment: usize,
}

/// An `unsupported_marker` paragraph (G1.3 class 4).
#[derive(Debug, Clone)]
struct Unsupported {
    paragraph: usize,
    roman: Option<u32>,
    numeral: String,
    owner: usize,
    segment: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScanReason {
    SequenceBreak,
    SameLineMarker,
    UnsupportedMarker,
    TailInciso,
}

impl ScanReason {
    fn as_str(self) -> &'static str {
        match self {
            ScanReason::SequenceBreak => "sequence_break",
            ScanReason::SameLineMarker => "same_line_marker",
            ScanReason::UnsupportedMarker => "unsupported_marker",
            ScanReason::TailInciso => "tail_inciso",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScanStatus {
    Complete,
    Partial,
    Ambiguous,
}

impl ScanStatus {
    fn as_str(self) -> &'static str {
        match self {
            ScanStatus::Complete => "complete",
            ScanStatus::Partial => "partial",
            ScanStatus::Ambiguous => "ambiguous",
        }
    }
}

#[derive(Debug, Clone)]
struct Scan {
    parent: usize,
    kind: ChildKind,
    reasons: Vec<ScanReason>,
}

impl Scan {
    fn status(&self) -> ScanStatus {
        if self.reasons.contains(&ScanReason::SequenceBreak) {
            ScanStatus::Ambiguous
        } else if self.reasons.is_empty() {
            ScanStatus::Complete
        } else {
            ScanStatus::Partial
        }
    }

    fn reasons_json(&self) -> Value {
        json!(self.reasons.iter().map(|r| r.as_str()).collect::<Vec<_>>())
    }
}

/// A span with its certain part and the segments that cut it (G1.5).
#[derive(Debug, Clone)]
struct CutSpan {
    /// The resolved span, or the certain part when cut; `None` when empty.
    certain: Option<(usize, usize)>,
    /// Indices of the segments that cut it, in order of `start_char`.
    segments: Vec<usize>,
}

impl CutSpan {
    fn ambiguous(&self) -> bool {
        !self.segments.is_empty()
    }
}

#[derive(Debug, Clone)]
struct Structure {
    nodes: Vec<Node>,
    has_fraction: bool,
    segments: Vec<Segment>,
    same_line: Vec<SameLine>,
    unsupported: Vec<Unsupported>,
    scans: Vec<Scan>,
    intro: Vec<CutSpan>,
    complete: Vec<CutSpan>,
}

/// Why a provision is not structured (G1.1, G1.2, G1.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotStructured {
    UnsupportedScope,
    UnexpectedLayout,
}

impl NotStructured {
    fn as_str(self) -> &'static str {
        match self {
            NotStructured::UnsupportedScope => "unsupported_scope",
            NotStructured::UnexpectedLayout => "unexpected_layout",
        }
    }
}

/// The hash-bound binding of a projection (G1.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
    pub provision_id: String,
    pub canonical_text_sha256: String,
    pub projection_version: String,
    pub source_release_id: String,
}

impl Binding {
    fn to_json(&self) -> Value {
        json!({
            "provision_id": self.provision_id,
            "canonical_text_sha256": self.canonical_text_sha256,
            "projection_version": self.projection_version,
            "source_release_id": self.source_release_id,
        })
    }
}

/// Paragraph-class and same-line counts of one projection, comparable with
/// the committed measurement probe (G1.9).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ClassCounts {
    pub fraction: usize,
    pub inciso: usize,
    pub note: usize,
    pub unsupported: usize,
    pub content: usize,
    pub same_line_fraction: usize,
    pub same_line_inciso: usize,
}

/// The derived projection of one provision.
#[derive(Debug, Clone)]
pub struct Projection {
    pub binding: Binding,
    text: Vec<char>,
    structure: Result<Structure, NotStructured>,
}

impl Projection {
    /// Build the projection of `text` (G1.2–G1.6).
    #[must_use]
    pub fn build(provision_id: &str, text: &str, eligible: bool, source_release_id: &str) -> Self {
        let binding = Binding {
            provision_id: provision_id.to_owned(),
            canonical_text_sha256: text_sha256(text),
            projection_version: PROJECTION_VERSION.to_owned(),
            source_release_id: source_release_id.to_owned(),
        };
        let chars: Vec<char> = text.chars().collect();
        let structure = if eligible {
            build_structure(text, &chars)
        } else {
            Err(NotStructured::UnsupportedScope)
        };
        Self {
            binding,
            text: chars,
            structure,
        }
    }

    /// The reason the projection is not structured, if it is not.
    #[must_use]
    pub fn not_structured_reason(&self) -> Option<NotStructured> {
        self.structure.as_ref().err().copied()
    }

    /// Whether a structured projection has a tail segment (G1.4).
    #[must_use]
    pub fn has_tail(&self) -> bool {
        self.structure.as_ref().is_ok_and(|structure| {
            structure
                .segments
                .iter()
                .any(|s| s.reason == SegmentReason::TrailingAttachment)
        })
    }

    /// Paragraph-class and same-line counts; `None` unless structured.
    #[must_use]
    pub fn class_counts(&self) -> Option<ClassCounts> {
        self.structure.as_ref().ok()?;
        let paragraphs = classify_paragraphs(&self.text);
        let mut counts = ClassCounts::default();
        for paragraph in &paragraphs {
            match paragraph.class {
                Class::Fraction { .. } => counts.fraction += 1,
                Class::Inciso { .. } => counts.inciso += 1,
                Class::Note => counts.note += 1,
                Class::Unsupported { .. } => counts.unsupported += 1,
                Class::Content => counts.content += 1,
            }
        }
        for raw in same_line_candidates(&self.text, &paragraphs) {
            match raw.label {
                CandidateLabel::Fraction(_) => counts.same_line_fraction += 1,
                CandidateLabel::Inciso(_) => counts.same_line_inciso += 1,
            }
        }
        Some(counts)
    }

    fn node_id(&self, path: &str) -> String {
        format!(
            "{}#{}:{}:{}",
            self.binding.provision_id,
            PROJECTION_VERSION,
            &self.binding.canonical_text_sha256[..12],
            path
        )
    }

    fn substring(&self, span: (usize, usize)) -> String {
        self.text[span.0..span.1].iter().collect()
    }

    /// The projection as JSON (`structural-projection`), keys sorted.
    #[must_use]
    pub fn to_json(&self) -> Value {
        let mut out = Map::new();
        out.insert("artifact".into(), json!("structural-projection"));
        out.insert("provision_id".into(), json!(self.binding.provision_id));
        out.insert(
            "canonical_text_sha256".into(),
            json!(self.binding.canonical_text_sha256),
        );
        out.insert("projection_version".into(), json!(PROJECTION_VERSION));
        out.insert(
            "source_release_id".into(),
            json!(self.binding.source_release_id),
        );
        match &self.structure {
            Err(reason) => {
                out.insert("provision_status".into(), json!("not_structured"));
                out.insert("not_structured_reason".into(), json!(reason.as_str()));
                out.insert("nodes".into(), json!([]));
                out.insert("unresolved_segments".into(), json!([]));
                out.insert("scans".into(), json!([]));
            }
            Ok(structure) => {
                out.insert("provision_status".into(), json!("structured"));
                let nodes: Vec<Value> = (0..structure.nodes.len())
                    .map(|index| self.node_json(structure, index))
                    .collect();
                out.insert("nodes".into(), Value::Array(nodes));
                let mut order: Vec<usize> = (0..structure.segments.len()).collect();
                order.sort_by_key(|index| segment_order_key(&structure.segments[*index]));
                let segments: Vec<Value> = order
                    .into_iter()
                    .map(|index| self.segment_json(structure, index))
                    .collect();
                out.insert("unresolved_segments".into(), Value::Array(segments));
                let scans: Vec<Value> = structure
                    .scans
                    .iter()
                    .map(|scan| {
                        json!({
                            "parent_node_id": self.node_id(&structure.nodes[scan.parent].path),
                            "child_kind": scan.kind.as_str(),
                            "status": scan.status().as_str(),
                            "reasons": scan.reasons_json(),
                        })
                    })
                    .collect();
                out.insert("scans".into(), Value::Array(scans));
            }
        }
        sort_keys(Value::Object(out))
    }

    fn node_json(&self, structure: &Structure, index: usize) -> Value {
        let node = &structure.nodes[index];
        let parenthood = match node.kind {
            NodeKind::Root => "root",
            _ if node.parent.is_some() => "resolved",
            _ => "ambiguous",
        };
        let status = |cut: &CutSpan| {
            if cut.ambiguous() {
                "ambiguous"
            } else {
                "resolved"
            }
        };
        json!({
            "node_id": self.node_id(&node.path),
            "kind": node.kind.as_str(),
            "path": node.path,
            "parent_node_id": node
                .parent
                .map_or(Value::Null, |p| json!(self.node_id(&structure.nodes[p].path))),
            "candidate_parent_node_ids": node
                .candidate_parents
                .iter()
                .map(|p| self.node_id(&structure.nodes[*p].path))
                .collect::<Vec<_>>(),
            "parenthood_status": parenthood,
            "ordinal_in_parent": node.ordinal,
            "label_span": span_json(node.label_span),
            "printed_label": node.printed_label,
            "normalised_label": node.normalised_label,
            "intro_span": span_json(structure.intro[index].certain),
            "intro_status": status(&structure.intro[index]),
            "complete_span": span_json(structure.complete[index].certain),
            "complete_status": status(&structure.complete[index]),
            "scope": "statute_body",
        })
    }

    fn segment_json(&self, structure: &Structure, index: usize) -> Value {
        let segment = &structure.segments[index];
        json!({
            "start_char": segment.start,
            "end_char": segment.end,
            "reason": segment.reason.as_str(),
            "candidate_owner_node_ids": segment
                .owners
                .iter()
                .map(|o| self.node_id(&structure.nodes[*o].path))
                .collect::<Vec<_>>(),
        })
    }
}

fn span_json(span: Option<(usize, usize)>) -> Value {
    span.map_or(
        Value::Null,
        |(start, end)| json!({"start_char": start, "end_char": end}),
    )
}

fn segment_order_key(segment: &Segment) -> (usize, usize, SegmentReason, Vec<usize>) {
    (
        segment.start,
        segment.end,
        segment.reason,
        segment.owners.clone(),
    )
}

/// Recursively sort object keys, so serialisation is deterministic whatever
/// `serde_json`'s map ordering (G1.8).
#[must_use]
pub fn sort_keys(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let sorted: BTreeMap<String, Value> =
                map.into_iter().map(|(k, v)| (k, sort_keys(v))).collect();
            Value::Object(sorted.into_iter().collect())
        }
        Value::Array(items) => Value::Array(items.into_iter().map(sort_keys).collect()),
        other => other,
    }
}

// ---------------------------------------------------------------------------
// Layout, paragraphs and classes (G1.2, G1.3)
// ---------------------------------------------------------------------------

fn layout_ok(text: &str) -> bool {
    if text != text.trim() {
        return false;
    }
    let mut run = 0usize;
    for c in text.chars().chain(std::iter::once('\0')) {
        if c == '\n' {
            run += 1;
        } else {
            if run != 0 && run != 2 {
                return false;
            }
            run = 0;
        }
    }
    true
}

fn split_paragraphs(chars: &[char]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut index = 0;
    while index < chars.len() {
        if chars[index] == '\n' && chars.get(index + 1) == Some(&'\n') {
            out.push((start, index));
            index += 2;
            start = index;
        } else {
            index += 1;
        }
    }
    out.push((start, chars.len()));
    out
}

fn letter_index(c: char) -> Option<usize> {
    LETTERS.iter().position(|l| *l == c)
}

fn is_roman_capital(c: char) -> bool {
    matches!(c, 'I' | 'V' | 'X' | 'L' | 'C' | 'D' | 'M')
}

fn classify(paragraph: &str, previous_base: Option<u32>) -> Class {
    if let Some(marker) = fraction_marker(paragraph) {
        let lone_ok = match marker.numeral {
            "L" => previous_base == Some(49),
            "C" => previous_base == Some(99),
            "D" | "M" => false,
            _ => true,
        };
        if let Some(value) =
            canonical_roman_value(marker.numeral).filter(|v| (1..=100).contains(v) && lone_ok)
        {
            let rank = marker
                .qualifier
                .and_then(fraction_qualifier_rank)
                .unwrap_or(0);
            return Class::Fraction {
                label: FractionLabel { value, rank },
                label_len: marker.end_char,
            };
        }
        return unsupported(paragraph);
    }
    let head: Vec<char> = paragraph.chars().take(3).collect();
    if let [letter, ')', space] = head[..]
        && space.is_whitespace()
        && let Some(letter) = letter_index(letter)
    {
        return Class::Inciso { letter };
    }
    if is_note(paragraph) {
        return Class::Note;
    }
    if is_unsupported(paragraph) {
        return unsupported(paragraph);
    }
    Class::Content
}

fn unsupported(paragraph: &str) -> Class {
    let numeral_len = paragraph
        .chars()
        .take_while(|c| is_roman_capital(*c))
        .count();
    let roman = if numeral_len > 0 {
        canonical_roman_value(&paragraph[..numeral_len])
    } else {
        None
    };
    Class::Unsupported { roman, numeral_len }
}

fn is_note(paragraph: &str) -> bool {
    if paragraph.chars().count() > 400 {
        return false;
    }
    let Some(rest) = [
        "Artículo adicionado",
        "Artículo reformado",
        "Artículo derogado",
    ]
    .iter()
    .find_map(|prefix| paragraph.strip_prefix(prefix)) else {
        return false;
    };
    if rest
        .chars()
        .next()
        .is_some_and(|c| c.is_alphanumeric() || c == '_')
    {
        return false;
    }
    let chars: Vec<char> = rest.chars().collect();
    (0..chars.len()).any(|i| {
        let d = &chars[i..];
        d.len() >= 14
            && d[..4] == ['D', 'O', 'F', ' ']
            && d[4].is_ascii_digit()
            && d[5].is_ascii_digit()
            && d[6] == '-'
            && d[7].is_ascii_digit()
            && d[8].is_ascii_digit()
            && d[9] == '-'
            && d[10..14].iter().all(char::is_ascii_digit)
    })
}

fn is_unsupported(paragraph: &str) -> bool {
    let chars: Vec<char> = paragraph.chars().collect();
    let lower_roman = |c: char| matches!(c, 'i' | 'v' | 'x' | 'l' | 'c' | 'd' | 'm');
    // Roman capitals not followed by a letter.
    let run = chars.iter().take_while(|c| is_roman_capital(**c)).count();
    if run > 0 && !chars.get(run).is_some_and(|c| c.is_alphabetic()) {
        return true;
    }
    // A capital letter (any script, accented included), a period and whitespace.
    if chars.len() >= 3 && chars[0].is_uppercase() && chars[1] == '.' && chars[2].is_whitespace() {
        return true;
    }
    // Digits followed by `.`, `)`, `°`, `º` or `o`.
    let digits = chars.iter().take_while(|c| c.is_numeric()).count();
    if digits > 0
        && chars
            .get(digits)
            .is_some_and(|c| matches!(c, '.' | ')' | '°' | 'º' | 'o'))
    {
        return true;
    }
    // Lowercase Roman followed by `.` or `)`.
    let run = chars.iter().take_while(|c| lower_roman(**c)).count();
    if run > 0 && chars.get(run).is_some_and(|c| matches!(c, '.' | ')')) {
        return true;
    }
    // Two or more letters, in any case, followed by `)`.
    let run = chars.iter().take_while(|c| c.is_alphabetic()).count();
    if run >= 2 && chars.get(run) == Some(&')') {
        return true;
    }
    // A lowercase letter (any script, accented included), a period and whitespace.
    if chars.len() >= 3 && chars[0].is_lowercase() && chars[1] == '.' && chars[2].is_whitespace() {
        return true;
    }
    // A dash or bullet followed by whitespace.
    chars.len() >= 2 && DASHES_AND_BULLETS.contains(&chars[0]) && chars[1].is_whitespace()
}

fn classify_paragraphs(chars: &[char]) -> Vec<Paragraph> {
    let mut previous_base = None;
    split_paragraphs(chars)
        .into_iter()
        .map(|(start, end)| {
            let paragraph: String = chars[start..end].iter().collect();
            let class = classify(&paragraph, previous_base);
            if let Class::Fraction { label, .. } = class {
                previous_base = Some(label.value);
            }
            Paragraph { start, end, class }
        })
        .collect()
}

struct RawCandidate {
    label: CandidateLabel,
    printed: String,
    span: (usize, usize),
    cut_end: usize,
}

fn same_line_candidates(chars: &[char], paragraphs: &[Paragraph]) -> Vec<RawCandidate> {
    let mut out = Vec::new();
    for paragraph in paragraphs {
        let text = &chars[paragraph.start..paragraph.end];
        for (index, c) in text.iter().enumerate() {
            if !matches!(c, ':' | ';' | '.') {
                continue;
            }
            let mut label_start = index + 1;
            while label_start < text.len() && text[label_start].is_whitespace() {
                label_start += 1;
            }
            if label_start == index + 1 || label_start >= text.len() {
                continue;
            }
            let absolute = paragraph.start + label_start;
            let cut_end = paragraph.start + index + 1;
            let after = &text[label_start..];
            // Fraction: a reading `fraction_marker` accepts, then whitespace and a letter.
            if is_roman_capital(after[0]) {
                let rest: String = after.iter().collect();
                if let Some(marker) = fraction_marker(&rest) {
                    let following = &after[marker.end_char..];
                    let ws = following.iter().take_while(|c| c.is_whitespace()).count();
                    if ws > 0 && following.get(ws).is_some_and(|c| c.is_alphabetic()) {
                        let rank = marker
                            .qualifier
                            .and_then(fraction_qualifier_rank)
                            .unwrap_or(0);
                        out.push(RawCandidate {
                            label: CandidateLabel::Fraction(
                                canonical_roman_value(marker.numeral)
                                    .map(|value| FractionLabel { value, rank }),
                            ),
                            printed: normalize_fraction_label(marker.label),
                            span: (absolute, absolute + marker.end_char),
                            cut_end,
                        });
                        continue;
                    }
                }
            }
            // Inciso: `x)`, then whitespace and a letter.
            if after.len() >= 4
                && after[1] == ')'
                && after[2].is_whitespace()
                && let Some(letter) = letter_index(after[0])
            {
                let ws = after[2..].iter().take_while(|c| c.is_whitespace()).count();
                if after.get(2 + ws).is_some_and(|c| c.is_alphabetic()) {
                    out.push(RawCandidate {
                        label: CandidateLabel::Inciso(letter),
                        printed: after[0].to_string(),
                        span: (absolute, absolute + 2),
                        cut_end,
                    });
                }
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Tree, ownership, spans and scans (G1.4–G1.6)
// ---------------------------------------------------------------------------

/// Certain or uncertain ownership of a paragraph (G1.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ownership {
    Certain(usize),
    Tail,
    Interruption { m: usize },
    PostChild { m: usize, fraction: usize },
}

/// The node, its parent, … up to the root (a node with ambiguous parenthood
/// reaches the root directly).
fn ancestors(nodes: &[Node], id: usize) -> Vec<usize> {
    let mut out = vec![id];
    let mut current = id;
    while let Some(parent) = nodes[current].parent {
        out.push(parent);
        current = parent;
    }
    if nodes[current].kind != NodeKind::Root {
        out.push(0);
    }
    out
}

fn region_contains(node: &Node, position: usize) -> bool {
    node.region.0 <= position && position < node.region.1
}

/// Innermost first: the smaller region, then the later node.
fn innermost_key(nodes: &[Node], id: usize) -> (usize, usize) {
    (nodes[id].region.1 - nodes[id].region.0, usize::MAX - id)
}

/// Whether `node` is strictly below `holder`, as a child or a candidate child.
fn strictly_below(nodes: &[Node], node: usize, holder: usize) -> bool {
    if node == holder {
        return false;
    }
    if holder == 0 {
        return true;
    }
    let mut current = node;
    loop {
        if nodes[current].candidate_parents.contains(&holder) {
            return true;
        }
        match nodes[current].parent {
            Some(parent) if parent == holder => return true,
            Some(parent) => current = parent,
            None => return false,
        }
    }
}

#[allow(clippy::too_many_lines)]
fn build_structure(text: &str, chars: &[char]) -> Result<Structure, NotStructured> {
    if !layout_ok(text) {
        return Err(NotStructured::UnexpectedLayout);
    }
    let paragraphs = classify_paragraphs(chars);
    let count = paragraphs.len();
    let class = |i: usize| paragraphs[i].class;
    let is_fraction = |i: usize| matches!(class(i), Class::Fraction { .. });
    let is_inciso = |i: usize| matches!(class(i), Class::Inciso { .. });
    let has_fraction = (0..count).any(is_fraction);
    let first_fraction = (0..count).find(|i| is_fraction(*i));
    let last_fraction = (0..count).rev().find(|i| is_fraction(*i));
    let end_before = |index: usize| -> usize {
        if index == 0 {
            0
        } else {
            paragraphs[index - 1].end
        }
    };

    // Closing notes: notes followed only by notes.
    let mut closing_from = count;
    while closing_from > 0 && class(closing_from - 1) == Class::Note {
        closing_from -= 1;
    }

    // The first tail paragraph (G1.4): after T's run, or after the last
    // inciso's marker paragraph in an article with no fraction.
    let tail_start = if let Some(t) = last_fraction {
        let mut j = t + 1;
        let mut last_letter: Option<usize> = None;
        for index in t + 1..count {
            match class(index) {
                Class::Inciso { letter }
                    if last_letter.map_or(index == t + 1, |last| letter > last) =>
                {
                    last_letter = Some(letter);
                    j = index + 1;
                }
                Class::Content if last_letter.is_some() => {}
                _ => break,
            }
        }
        Some(j)
    } else {
        (0..count)
            .rev()
            .find(|i| is_inciso(*i))
            .map(|last| last + 1)
    };
    // The tail exists when a non-note paragraph remains; it ends before the closing notes.
    let tail = tail_start.filter(|j| (*j..count).any(|i| class(i) != Class::Note));
    let in_tail = |i: usize| tail.is_some_and(|j| i >= j);

    // Unmodelled layouts fail closed (G1.4).
    if let Some(first) = first_fraction
        && (0..first).any(is_inciso)
    {
        return Err(NotStructured::UnexpectedLayout);
    }
    for note in 0..closing_from {
        if class(note) != Class::Note || in_tail(note) {
            continue;
        }
        for later in note + 1..count {
            match class(later) {
                Class::Fraction { .. } => break,
                Class::Inciso { .. } => return Err(NotStructured::UnexpectedLayout),
                _ => {}
            }
        }
    }

    // Nodes, in document order.
    let mut nodes = vec![Node {
        kind: NodeKind::Root,
        paragraph: None,
        parent: None,
        candidate_parents: Vec::new(),
        tail: false,
        path: "root".into(),
        fraction: None,
        letter: None,
        label_span: None,
        printed_label: None,
        normalised_label: None,
        ordinal: None,
        region: (0, chars.len()),
    }];
    let mut node_of_paragraph: Vec<Option<usize>> = vec![None; count];
    let mut child_count: Vec<usize> = vec![0];
    let mut current_fraction: Option<usize> = None;
    let mut t_node: Option<usize> = None;
    let mut fraction_occurrence: BTreeMap<FractionLabel, usize> = BTreeMap::new();
    let mut inciso_occurrence: BTreeMap<(usize, usize), usize> = BTreeMap::new();
    let mut tail_occurrence: BTreeMap<usize, usize> = BTreeMap::new();
    for (index, paragraph) in paragraphs.iter().enumerate() {
        let printed = |len: usize| -> String {
            chars[paragraph.start..paragraph.start + len]
                .iter()
                .collect()
        };
        let node = match paragraph.class {
            Class::Fraction { label, label_len } => {
                let occurrence = fraction_occurrence.entry(label).or_insert(0);
                *occurrence += 1;
                child_count[0] += 1;
                let printed_label = printed(label_len);
                Node {
                    kind: NodeKind::Fraction,
                    paragraph: Some(index),
                    parent: Some(0),
                    candidate_parents: Vec::new(),
                    tail: false,
                    path: format!("{}.{occurrence}", label.path()),
                    fraction: Some(label),
                    letter: None,
                    label_span: Some((paragraph.start, paragraph.start + label_len)),
                    normalised_label: Some(normalize_fraction_label(
                        printed_label.trim_end_matches(['.', '-', '–']),
                    )),
                    printed_label: Some(printed_label),
                    ordinal: Some(child_count[0]),
                    region: (paragraph.start, chars.len()),
                }
            }
            Class::Inciso { letter } => {
                let letter_char = LETTERS[letter];
                let common = |path: String, parent, candidates, tail, ordinal| Node {
                    kind: NodeKind::Inciso,
                    paragraph: Some(index),
                    parent,
                    candidate_parents: candidates,
                    tail,
                    path,
                    fraction: None,
                    letter: Some(letter),
                    label_span: Some((paragraph.start, paragraph.start + 2)),
                    printed_label: Some(printed(2)),
                    normalised_label: Some(letter_char.to_string()),
                    ordinal,
                    region: (paragraph.start, chars.len()),
                };
                if has_fraction && in_tail(index) {
                    let occurrence = tail_occurrence.entry(letter).or_insert(0);
                    *occurrence += 1;
                    common(
                        format!("tail/i.{letter_char}.{occurrence}"),
                        None,
                        vec![t_node.unwrap_or(0), 0],
                        true,
                        None,
                    )
                } else {
                    let parent = if has_fraction {
                        current_fraction.unwrap_or(0)
                    } else {
                        0
                    };
                    let occurrence = inciso_occurrence.entry((parent, letter)).or_insert(0);
                    *occurrence += 1;
                    child_count[parent] += 1;
                    let prefix = if parent == 0 {
                        String::new()
                    } else {
                        format!("{}/", nodes[parent].path)
                    };
                    common(
                        format!("{prefix}i.{letter_char}.{occurrence}"),
                        Some(parent),
                        Vec::new(),
                        false,
                        Some(child_count[parent]),
                    )
                }
            }
            _ => continue,
        };
        nodes.push(node);
        child_count.push(0);
        let id = nodes.len() - 1;
        if nodes[id].kind == NodeKind::Fraction {
            current_fraction = Some(id);
            if Some(index) == last_fraction {
                t_node = Some(id);
            }
        }
        node_of_paragraph[index] = Some(id);
    }

    // Regions (G1.4). Fractions precede their incisos, so a parent's region is set first.
    for id in 1..nodes.len() {
        let index = nodes[id].paragraph.unwrap_or_default();
        let next_marker = (index + 1..count).find(|i| class(*i).is_marker());
        let end = match nodes[id].kind {
            NodeKind::Fraction => (index + 1..count)
                .find(|i| is_fraction(*i))
                .map_or(chars.len(), end_before),
            NodeKind::Inciso if nodes[id].tail => next_marker.map_or(chars.len(), end_before),
            NodeKind::Inciso => {
                let parent_end = nodes[nodes[id].parent.unwrap_or_default()].region.1;
                let mut end = next_marker.map_or(parent_end, end_before).min(parent_end);
                if let Some(j) = tail
                    && j > index
                {
                    end = end.min(end_before(j));
                }
                end
            }
            NodeKind::Root => chars.len(),
        };
        nodes[id].region.1 = end;
    }

    // Ownership of every paragraph (G1.4 rules 1–6; markers and notes are certain).
    let marker_before = |i: usize| (0..i).rev().find(|k| class(*k).is_marker());
    let marker_after = |i: usize| (i + 1..count).find(|k| class(*k).is_marker());
    let mut ownership: Vec<Ownership> = Vec::with_capacity(count);
    for index in 0..count {
        let owner = if let Some(node) = node_of_paragraph[index] {
            Ownership::Certain(node)
        } else if class(index) == Class::Note {
            Ownership::Certain(0)
        } else if let Some(m_paragraph) = marker_before(index) {
            let m = node_of_paragraph[m_paragraph].unwrap_or_default();
            if in_tail(index) {
                Ownership::Tail
            } else if (m_paragraph + 1..index).any(|k| class(k) == Class::Note) {
                Ownership::Interruption { m }
            } else if nodes[m].kind == NodeKind::Fraction {
                Ownership::Certain(m)
            } else {
                match marker_after(index).map(class) {
                    Some(Class::Fraction { .. }) => Ownership::PostChild {
                        m,
                        fraction: nodes[m].parent.unwrap_or_default(),
                    },
                    // Rule 5; G1.4 leaves no other case (tail or interruption covers it).
                    _ => Ownership::Certain(m),
                }
            }
        } else {
            Ownership::Certain(0)
        };
        ownership.push(owner);
    }

    // Segments.
    let mut segments: Vec<Segment> = Vec::new();
    let mut tail_segment = None;
    let mut tail_candidates: Vec<usize> = Vec::new();
    if let Some(j) = tail {
        let last_marker = marker_before(j).and_then(|p| node_of_paragraph[p]);
        tail_candidates = last_marker.map_or_else(|| vec![0], |node| ancestors(&nodes, node));
        segments.push(Segment {
            start: paragraphs[j].start,
            end: paragraphs[closing_from - 1].end,
            reason: SegmentReason::TrailingAttachment,
            owners: tail_candidates.clone(),
        });
        tail_segment = Some(segments.len() - 1);
    }
    // Note-interruption and post-child segments: maximal runs of unlabelled
    // paragraphs under the same rule, bounded by markers and notes.
    let mut interruption_segments: Vec<(usize, usize)> = Vec::new(); // (segment, first paragraph)
    let mut post_child_segments: Vec<(usize, usize)> = Vec::new(); // (segment, m)
    let mut index = 0;
    while index < count {
        let rule = ownership[index];
        let (reason, owners) = match rule {
            Ownership::Interruption { m } => {
                (SegmentReason::NoteInterruption, ancestors(&nodes, m))
            }
            Ownership::PostChild { m, fraction } => {
                (SegmentReason::PostChildAttachment, vec![m, fraction])
            }
            _ => {
                index += 1;
                continue;
            }
        };
        let mut last = index;
        while last + 1 < count && ownership[last + 1] == rule {
            last += 1;
        }
        segments.push(Segment {
            start: paragraphs[index].start,
            end: paragraphs[last].end,
            reason,
            owners,
        });
        let segment = segments.len() - 1;
        match rule {
            Ownership::Interruption { .. } => interruption_segments.push((segment, index)),
            Ownership::PostChild { m, .. } => post_child_segments.push((segment, m)),
            _ => {}
        }
        index = last + 1;
    }

    // Unsupported markers and their owner N (G1.5 table, G1.6).
    let mut unsupported = Vec::new();
    for index in 0..count {
        let Class::Unsupported { roman, numeral_len } = class(index) else {
            continue;
        };
        let owner = match ownership[index] {
            Ownership::Certain(node) => node,
            Ownership::Tail => tail_candidates.first().copied().unwrap_or(0),
            Ownership::Interruption { m } => m,
            Ownership::PostChild { fraction, .. } => fraction,
        };
        let start = paragraphs[index].start;
        segments.push(Segment {
            start,
            end: paragraphs[index].end,
            reason: SegmentReason::UnsupportedMarker,
            owners: ancestors(&nodes, owner),
        });
        unsupported.push(Unsupported {
            paragraph: index,
            roman,
            numeral: chars[start..start + numeral_len].iter().collect(),
            owner,
            segment: segments.len() - 1,
        });
    }

    // Same-line candidates: holder and cuts (G1.5, G1.6).
    let mut same_line = Vec::new();
    for raw in same_line_candidates(chars, &paragraphs) {
        let position = raw.span.0;
        let region_node = (1..nodes.len())
            .filter(|id| region_contains(&nodes[*id], position))
            .min_by_key(|id| innermost_key(&nodes, *id))
            .unwrap_or(0);
        let holder = match raw.label {
            CandidateLabel::Fraction(_) => Some(0),
            CandidateLabel::Inciso(_) if nodes[region_node].tail => t_node,
            CandidateLabel::Inciso(_) if !has_fraction => Some(0),
            CandidateLabel::Inciso(_) => ancestors(&nodes, region_node)
                .into_iter()
                .find(|id| nodes[*id].kind == NodeKind::Fraction),
        };
        let mut cuts: Vec<usize> = holder.map_or_else(Vec::new, |holder| {
            (1..nodes.len())
                .filter(|id| {
                    region_contains(&nodes[*id], position) && strictly_below(&nodes, *id, holder)
                })
                .collect()
        });
        cuts.sort_by_key(|id| innermost_key(&nodes, *id));
        let mut owners = cuts.clone();
        owners.push(holder.unwrap_or(0));
        let paragraph_end = paragraphs
            .iter()
            .find(|p| p.start <= position && position < p.end)
            .map_or(chars.len(), |p| p.end);
        segments.push(Segment {
            start: position,
            end: paragraph_end,
            reason: SegmentReason::SameLineMarker,
            owners,
        });
        same_line.push(SameLine {
            label: raw.label,
            printed: raw.printed,
            span: raw.span,
            cut_end: raw.cut_end,
            holder,
            cuts,
            segment: segments.len() - 1,
        });
    }

    // The last paragraph each node or one of its descendants certainly owns.
    let mut last_owned: Vec<Option<usize>> = vec![None; nodes.len()];
    for (index, owner) in ownership.iter().enumerate() {
        let owner = match *owner {
            Ownership::Certain(node) => node,
            Ownership::PostChild { fraction, .. } => fraction,
            _ => continue,
        };
        for node in ancestors(&nodes, owner) {
            last_owned[node] = Some(index);
        }
    }

    // Spans (G1.5).
    let mut intro = Vec::with_capacity(nodes.len());
    let mut complete = Vec::with_capacity(nodes.len());
    // Root: intro up to the last non-note paragraph before the first child marker.
    let limit = (0..count).find(|i| class(*i).is_marker()).unwrap_or(count);
    let root_intro = (0..limit)
        .rev()
        .find(|i| class(*i) != Class::Note)
        .map_or((0, 0), |last| (0, paragraphs[last].end));
    let mut root_cuts: Vec<(usize, usize)> = Vec::new(); // (certain end, segment)
    for candidate in &same_line {
        if candidate.span.0 < root_intro.1 {
            root_cuts.push((candidate.cut_end, candidate.segment));
        }
    }
    for marker in &unsupported {
        if paragraphs[marker.paragraph].start < root_intro.1 {
            root_cuts.push((end_before(marker.paragraph), marker.segment));
        }
    }
    intro.push(cut_span(root_intro, &root_cuts, &segments));
    complete.push(CutSpan {
        certain: Some((0, chars.len())),
        segments: Vec::new(),
    });
    for id in 1..nodes.len() {
        let paragraph = &paragraphs[nodes[id].paragraph.unwrap_or_default()];
        let label_end = nodes[id].label_span.map_or(paragraph.start, |s| s.1);
        let mut intro_start = label_end;
        while intro_start < paragraph.end && chars[intro_start].is_whitespace() {
            intro_start += 1;
        }
        let intro_span = (intro_start, paragraph.end);
        let mut intro_cuts = Vec::new();
        let mut complete_cuts = Vec::new();
        for candidate in same_line.iter().filter(|c| c.cuts.contains(&id)) {
            complete_cuts.push((candidate.cut_end, candidate.segment));
            if candidate.span.0 >= intro_span.0 && candidate.span.0 < intro_span.1 {
                intro_cuts.push((candidate.cut_end, candidate.segment));
            }
        }
        intro.push(cut_span(intro_span, &intro_cuts, &segments));

        let span_end = last_owned[id].map_or(paragraph.end, |k| paragraphs[k].end);
        if let (Some(segment), Some(j)) = (tail_segment, tail)
            && tail_candidates.contains(&id)
        {
            complete_cuts.push((end_before(j), segment));
        }
        for (segment, first) in &interruption_segments {
            if segments[*segment].owners.contains(&id) {
                let mut note = *first;
                while note > 0 && class(note - 1) == Class::Note {
                    note -= 1;
                }
                complete_cuts.push((end_before(note), *segment));
            }
        }
        for (segment, m) in &post_child_segments {
            if *m == id {
                complete_cuts.push((span_end, *segment));
            }
        }
        for marker in &unsupported {
            if ancestors(&nodes, marker.owner).contains(&id) {
                complete_cuts.push((end_before(marker.paragraph), marker.segment));
            }
        }
        complete.push(cut_span(
            (paragraph.start, span_end),
            &complete_cuts,
            &segments,
        ));
    }

    // Scans (G1.6).
    let mut scans = vec![Scan {
        parent: 0,
        kind: ChildKind::Fraction,
        reasons: Vec::new(),
    }];
    if !has_fraction {
        scans.push(Scan {
            parent: 0,
            kind: ChildKind::Inciso,
            reasons: Vec::new(),
        });
    }
    for (id, node) in nodes.iter().enumerate() {
        if node.kind == NodeKind::Fraction {
            scans.push(Scan {
                parent: id,
                kind: ChildKind::Inciso,
                reasons: Vec::new(),
            });
        }
    }
    let tail_has_inciso = nodes.iter().any(|n| n.tail);
    for scan in &mut scans {
        let resolved: Vec<&Node> = nodes
            .iter()
            .filter(|n| n.parent == Some(scan.parent))
            .filter(|n| match scan.kind {
                ChildKind::Fraction => n.kind == NodeKind::Fraction,
                ChildKind::Inciso => n.kind == NodeKind::Inciso,
            })
            .collect();
        let broken = resolved.windows(2).any(|pair| match scan.kind {
            ChildKind::Fraction => pair[1].fraction <= pair[0].fraction,
            ChildKind::Inciso => pair[1].letter <= pair[0].letter,
        });
        if broken {
            scan.reasons.push(ScanReason::SequenceBreak);
        }
        if same_line.iter().any(|c| {
            c.holder == Some(scan.parent)
                && matches!(
                    (c.label, scan.kind),
                    (CandidateLabel::Fraction(_), ChildKind::Fraction)
                        | (CandidateLabel::Inciso(_), ChildKind::Inciso)
                )
        }) {
            scan.reasons.push(ScanReason::SameLineMarker);
        }
        if unsupported
            .iter()
            .any(|u| ancestors(&nodes, u.owner).contains(&scan.parent))
        {
            scan.reasons.push(ScanReason::UnsupportedMarker);
        }
        if Some(scan.parent) == t_node && scan.kind == ChildKind::Inciso && tail_has_inciso {
            scan.reasons.push(ScanReason::TailInciso);
        }
    }

    Ok(Structure {
        nodes,
        has_fraction,
        segments,
        same_line,
        unsupported,
        scans,
        intro,
        complete,
    })
}

fn cut_span(span: (usize, usize), cuts: &[(usize, usize)], segments: &[Segment]) -> CutSpan {
    if cuts.is_empty() {
        return CutSpan {
            certain: Some(span),
            segments: Vec::new(),
        };
    }
    let end = cuts
        .iter()
        .map(|(end, _)| *end)
        .min()
        .unwrap_or(span.1)
        .min(span.1);
    let mut list: Vec<usize> = cuts.iter().map(|(_, segment)| *segment).collect();
    list.sort_by_key(|index| segment_order_key(&segments[*index]));
    list.dedup();
    CutSpan {
        certain: (end > span.0).then_some((span.0, end)),
        segments: list,
    }
}

// ---------------------------------------------------------------------------
// Lookup (G1.7)
// ---------------------------------------------------------------------------

/// The deciding stage or branch of a lookup, named as in the fixtures:
/// `V`, `S`, `N`, `R`, or `L0`–`L7` for the last step.
pub type Branch = &'static str;

/// One loaded projection set and the current canonical corpus at a stated
/// release (G1.7).
///
/// The context knows only the provisions inserted into it. `unknown_provision`
/// therefore means "not loaded in this lookup context". It is never evidence that a
/// provision is absent from the repository or from the law. Callers must load
/// every provision they intend to query, text and projection together, with
/// [`LookupContext::insert`].
#[derive(Debug, Default)]
pub struct LookupContext {
    release_id: String,
    current: BTreeMap<String, String>,
    projections: BTreeMap<String, Projection>,
}

impl LookupContext {
    #[must_use]
    pub fn new(release_id: &str) -> Self {
        Self {
            release_id: release_id.to_owned(),
            ..Self::default()
        }
    }

    /// Add a current canonical provision and its projection, built from the
    /// same text at this context's release.
    pub fn insert(&mut self, provision_id: &str, text: &str, eligible: bool) {
        self.current
            .insert(provision_id.to_owned(), text.to_owned());
        self.projections.insert(
            provision_id.to_owned(),
            Projection::build(provision_id, text, eligible, &self.release_id),
        );
    }

    /// Replace the current canonical text of a provision already inserted,
    /// keeping its loaded projection, which may then be stale. It never adds
    /// a provision: for an ID not inserted it changes nothing and returns
    /// `false`.
    #[must_use]
    pub fn set_current_text(&mut self, provision_id: &str, text: &str) -> bool {
        match self.current.get_mut(provision_id) {
            Some(current) => {
                text.clone_into(current);
                true
            }
            None => false,
        }
    }

    /// The loaded projection of a provision.
    #[must_use]
    pub fn projection(&self, provision_id: &str) -> Option<&Projection> {
        self.projections.get(provision_id)
    }

    /// Run one request; the response JSON, keys sorted.
    #[must_use]
    pub fn lookup(&self, request: &Value) -> Value {
        self.lookup_traced(request).0
    }

    /// Run one request; the response and the deciding branch.
    #[must_use]
    pub fn lookup_traced(&self, request: &Value) -> (Value, Branch) {
        let (mut response, branch) = self.evaluate(request);
        response.insert("artifact".into(), json!("structural-lookup-response"));
        response.insert("request".into(), request.clone());
        response.insert("mode".into(), response_mode(request));
        (sort_keys(Value::Object(response)), branch)
    }

    fn evaluate(&self, request: &Value) -> (Map<String, Value>, Branch) {
        let mut out = Map::new();
        let parsed = match parse_request(request) {
            Ok(parsed) => parsed,
            Err(reason) => {
                set_outcome(&mut out, "invalid_locator", Some(reason));
                return (out, "V");
            }
        };
        let (Some(current_text), Some(projection)) = (
            self.current.get(&parsed.provision_id),
            self.projections.get(&parsed.provision_id),
        ) else {
            set_outcome(&mut out, "invalid_locator", Some("unknown_provision"));
            return (out, "V");
        };
        out.insert("normalised_path".into(), json!(parsed.normalised_path()));
        let current = Binding {
            provision_id: parsed.provision_id.clone(),
            canonical_text_sha256: text_sha256(current_text),
            projection_version: PROJECTION_VERSION.to_owned(),
            source_release_id: self.release_id.clone(),
        };
        let bound = &projection.binding;
        let stale = if bound.canonical_text_sha256 != current.canonical_text_sha256 {
            Some("hash_mismatch")
        } else if bound.projection_version != current.projection_version {
            Some("version_mismatch")
        } else if bound.source_release_id != current.source_release_id {
            Some("release_mismatch")
        } else {
            None
        };
        if let Some(reason) = stale {
            set_outcome(&mut out, "stale_projection", Some(reason));
            out.insert("projection_binding".into(), bound.to_json());
            out.insert("current_binding".into(), current.to_json());
            return (out, "S");
        }
        if let Value::Object(binding) = bound.to_json() {
            out.extend(binding);
        }
        let structure = match &projection.structure {
            Err(reason) => {
                set_outcome(&mut out, "not_structured", Some(reason.as_str()));
                return (out, "N");
            }
            Ok(structure) => structure,
        };
        let walk = Walk {
            projection,
            structure,
        };
        if parsed.root {
            set_scan(&mut out, None);
            let cut = if parsed.mode == Mode::Intro {
                &structure.intro[0]
            } else {
                &structure.complete[0]
            };
            walk.apply_span(0, cut, &mut out);
            return (out, "R");
        }
        walk.path(&parsed, &mut out)
    }
}

fn set_outcome(out: &mut Map<String, Value>, outcome: &str, reason: Option<&str>) {
    out.insert("outcome".into(), json!(outcome));
    if let Some(reason) = reason {
        out.insert("reason".into(), json!(reason));
    }
}

fn set_scan(out: &mut Map<String, Value>, scan: Option<&Scan>) {
    if let Some(scan) = scan {
        out.insert("scan_status".into(), json!(scan.status().as_str()));
        out.insert("scan_reasons".into(), scan.reasons_json());
    } else {
        out.insert("scan_status".into(), Value::Null);
        out.insert("scan_reasons".into(), json!([]));
    }
}

struct Walk<'a> {
    projection: &'a Projection,
    structure: &'a Structure,
}

impl Walk<'_> {
    fn node_candidate(&self, id: usize) -> (usize, Value) {
        let node = &self.structure.nodes[id];
        let span = node.label_span.unwrap_or_default();
        (
            span.0,
            json!({
                "node_id": self.projection.node_id(&node.path),
                "path": node.path,
                "label_span": {"start_char": span.0, "end_char": span.1},
            }),
        )
    }

    fn path_ambiguity(out: &mut Map<String, Value>, reason: &str, mut listed: Vec<(usize, Value)>) {
        set_outcome(out, "ambiguous", Some(reason));
        out.insert("ambiguity".into(), json!("path"));
        listed.sort_by_key(|(start, _)| *start);
        out.insert(
            "candidates".into(),
            Value::Array(listed.into_iter().map(|(_, v)| v).collect()),
        );
    }

    #[allow(clippy::too_many_lines)]
    fn path(
        &self,
        parsed: &ParsedRequest,
        out: &mut Map<String, Value>,
    ) -> (Map<String, Value>, Branch) {
        let structure = self.structure;
        let mut steps: Vec<(ChildKind, CandidateLabel)> = Vec::new();
        if let Some(fraction) = parsed.fraction {
            steps.push((
                ChildKind::Fraction,
                CandidateLabel::Fraction(Some(fraction)),
            ));
        }
        if let Some(letter) = parsed.inciso {
            if parsed.fraction.is_none() && structure.has_fraction {
                // L0: an inciso without its fraction.
                set_scan(out, None);
                let listed: Vec<(usize, Value)> = (1..structure.nodes.len())
                    .filter(|id| structure.nodes[*id].letter == Some(letter))
                    .map(|id| self.node_candidate(id))
                    .collect();
                if listed.is_empty() {
                    set_outcome(out, "not_structured", Some("incomplete_path"));
                } else {
                    Self::path_ambiguity(out, "incomplete_path", listed);
                }
                return (std::mem::take(out), "L0");
            }
            steps.push((ChildKind::Inciso, CandidateLabel::Inciso(letter)));
        }
        let mut parent = 0;
        for (kind, label) in steps {
            let scan = structure
                .scans
                .iter()
                .find(|s| s.parent == parent && s.kind == kind);
            set_scan(out, scan);
            let status = scan.map_or(ScanStatus::Complete, Scan::status);
            let matches: Vec<usize> = (1..structure.nodes.len())
                .filter(|id| {
                    let node = &structure.nodes[*id];
                    let labelled = match label {
                        CandidateLabel::Fraction(f) => {
                            node.kind == NodeKind::Fraction && node.fraction == f
                        }
                        CandidateLabel::Inciso(l) => {
                            node.kind == NodeKind::Inciso && node.letter == Some(l)
                        }
                    };
                    labelled
                        && (node.parent == Some(parent) || node.candidate_parents.contains(&parent))
                })
                .collect();
            let prefix = if parent == 0 {
                String::new()
            } else {
                format!("{}/", structure.nodes[parent].path)
            };
            let mut held: Vec<(usize, Value)> = Vec::new();
            let mut held_same_line = false;
            for candidate in &structure.same_line {
                if candidate.holder == Some(parent) && candidate_matches(candidate.label, label) {
                    held_same_line = true;
                    held.push((
                        candidate.span.0,
                        json!({
                            "node_id": Value::Null,
                            "path": format!("{prefix}(same-line {})", candidate.printed),
                            "label_span": span_json(Some(candidate.span)),
                        }),
                    ));
                }
            }
            if let CandidateLabel::Fraction(Some(fraction)) = label {
                for marker in &structure.unsupported {
                    if marker.roman == Some(fraction.value)
                        && ancestors(&structure.nodes, marker.owner).contains(&parent)
                    {
                        let start = structure.segments[marker.segment].start;
                        held.push((
                            start,
                            json!({
                                "node_id": Value::Null,
                                "path": format!("{prefix}(unsupported {})", marker.numeral),
                                "label_span": span_json(Some((start, start + marker.numeral.chars().count()))),
                            }),
                        ));
                    }
                }
            }
            let listed = || {
                let mut listed: Vec<(usize, Value)> =
                    matches.iter().map(|id| self.node_candidate(*id)).collect();
                listed.extend(held.iter().cloned());
                listed
            };
            if matches
                .iter()
                .any(|id| structure.nodes[*id].parent.is_none())
            {
                Self::path_ambiguity(out, "ambiguous_parent", listed());
                return (std::mem::take(out), "L1");
            }
            if !held.is_empty() {
                let reason = if held_same_line {
                    "same_line_marker"
                } else {
                    "unsupported_marker"
                };
                Self::path_ambiguity(out, reason, listed());
                return (std::mem::take(out), "L2");
            }
            if !matches.is_empty() && status == ScanStatus::Ambiguous {
                Self::path_ambiguity(out, "sequence_break", listed());
                return (std::mem::take(out), "L3");
            }
            if matches.len() == 1 {
                parent = matches[0];
                continue;
            }
            if !matches.is_empty() {
                // Two resolved matches imply a sequence break (G1.7).
                Self::path_ambiguity(out, "sequence_break", listed());
                return (std::mem::take(out), "L3");
            }
            let branch = match status {
                ScanStatus::Complete => {
                    let above = structure
                        .nodes
                        .iter()
                        .filter(|n| n.parent == Some(parent))
                        .all(|n| match (kind, label) {
                            (ChildKind::Fraction, CandidateLabel::Fraction(f)) => {
                                n.kind != NodeKind::Fraction || f > n.fraction
                            }
                            (ChildKind::Inciso, CandidateLabel::Inciso(l)) => {
                                n.kind != NodeKind::Inciso || Some(l) > n.letter
                            }
                            _ => true,
                        });
                    if above {
                        set_outcome(out, "not_found", Some("absent_in_complete_scan"));
                        "L5"
                    } else {
                        set_outcome(out, "not_structured", Some("sequence_gap"));
                        "L6"
                    }
                }
                ScanStatus::Partial => {
                    set_outcome(out, "not_structured", Some("partial_scan"));
                    "L7"
                }
                ScanStatus::Ambiguous => {
                    set_outcome(out, "not_structured", Some("sequence_break"));
                    "L7"
                }
            };
            return (std::mem::take(out), branch);
        }

        // Mode applied to the resolved node (stage 6).
        let node = parent;
        match parsed.mode {
            Mode::Label => {
                let span = structure.nodes[node].label_span.unwrap_or_default();
                set_outcome(out, "resolved", None);
                out.insert(
                    "node_id".into(),
                    json!(self.projection.node_id(&structure.nodes[node].path)),
                );
                out.insert("span".into(), span_json(Some(span)));
                out.insert("text".into(), json!(self.projection.substring(span)));
            }
            Mode::Intro => self.apply_span(node, &structure.intro[node], out),
            Mode::Complete => self.apply_span(node, &structure.complete[node], out),
        }
        (std::mem::take(out), "L4")
    }

    fn apply_span(&self, node: usize, cut: &CutSpan, out: &mut Map<String, Value>) {
        out.insert(
            "node_id".into(),
            json!(self.projection.node_id(&self.structure.nodes[node].path)),
        );
        if cut.ambiguous() {
            set_outcome(out, "ambiguous", Some("unresolved_boundary"));
            out.insert("ambiguity".into(), json!("span"));
            out.insert("certain_span".into(), span_json(cut.certain));
            let segments: Vec<Value> = cut
                .segments
                .iter()
                .map(|index| self.projection.segment_json(self.structure, *index))
                .collect();
            out.insert("unresolved_segments".into(), Value::Array(segments));
        } else {
            let span = cut.certain.unwrap_or_default();
            set_outcome(out, "resolved", None);
            out.insert("span".into(), span_json(Some(span)));
            out.insert("text".into(), json!(self.projection.substring(span)));
        }
    }
}

fn candidate_matches(candidate: CandidateLabel, requested: CandidateLabel) -> bool {
    match (candidate, requested) {
        (CandidateLabel::Fraction(Some(a)), CandidateLabel::Fraction(Some(b))) => a == b,
        (CandidateLabel::Inciso(a), CandidateLabel::Inciso(b)) => a == b,
        _ => false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Label,
    Intro,
    Complete,
}

#[derive(Debug)]
struct ParsedRequest {
    provision_id: String,
    root: bool,
    fraction: Option<FractionLabel>,
    inciso: Option<usize>,
    mode: Mode,
}

impl ParsedRequest {
    fn normalised_path(&self) -> String {
        if self.root {
            return "root".into();
        }
        let mut parts = Vec::new();
        if let Some(fraction) = self.fraction {
            parts.push(fraction.path());
        }
        if let Some(letter) = self.inciso {
            parts.push(format!("i.{}", LETTERS[letter]));
        }
        parts.join("/")
    }
}

fn response_mode(request: &Value) -> Value {
    match request.as_object().map(|object| object.get("mode")) {
        Some(None) => json!("complete"),
        Some(Some(Value::String(mode)))
            if ["label", "intro", "complete"].contains(&mode.as_str()) =>
        {
            json!(mode)
        }
        _ => Value::Null,
    }
}

fn parse_request(request: &Value) -> Result<ParsedRequest, &'static str> {
    let object = request.as_object().ok_or("malformed_request")?;
    if object.contains_key("node_id") {
        return Err("unsupported_request_form");
    }
    if object
        .keys()
        .any(|k| !["provision_id", "locator", "mode"].contains(&k.as_str()))
    {
        return Err("unknown_key");
    }
    if let Some(Value::Object(locator)) = object.get("locator")
        && locator
            .keys()
            .any(|k| !["fraction", "inciso", "article_root"].contains(&k.as_str()))
    {
        return Err("unknown_key");
    }
    let provision_id = object
        .get("provision_id")
        .and_then(Value::as_str)
        .ok_or("malformed_request")?;
    let locator = object
        .get("locator")
        .and_then(Value::as_object)
        .ok_or("malformed_request")?;
    if ["fraction", "inciso"]
        .iter()
        .any(|key| locator.get(*key).is_some_and(|v| !v.is_string()))
        || locator
            .get("article_root")
            .is_some_and(|v| v != &Value::Bool(true))
    {
        return Err("malformed_request");
    }
    let mode = match object.get("mode") {
        None => Mode::Complete,
        Some(Value::String(mode)) if mode == "label" => Mode::Label,
        Some(Value::String(mode)) if mode == "intro" => Mode::Intro,
        Some(Value::String(mode)) if mode == "complete" => Mode::Complete,
        Some(_) => return Err("unknown_mode"),
    };
    if locator.is_empty() {
        return Err("empty_locator");
    }
    let root = locator.contains_key("article_root");
    if root && (locator.contains_key("fraction") || locator.contains_key("inciso")) {
        return Err("conflicting_locator");
    }
    let fraction = match locator.get("fraction").and_then(Value::as_str) {
        Some(label) => Some(parse_fraction_locator(label).ok_or("invalid_fraction_label")?),
        None => None,
    };
    let inciso = match locator.get("inciso").and_then(Value::as_str) {
        Some(label) => Some(parse_inciso_locator(label).ok_or("invalid_inciso_label")?),
        None => None,
    };
    if root && mode == Mode::Label {
        return Err("label_mode_on_root");
    }
    Ok(ParsedRequest {
        provision_id: provision_id.to_owned(),
        root,
        fraction,
        inciso,
        mode,
    })
}

fn parse_fraction_locator(label: &str) -> Option<FractionLabel> {
    let (numeral, qualifier) = match label.split_once(' ') {
        Some((numeral, qualifier)) => (numeral, Some(qualifier)),
        None => (label, None),
    };
    let value = canonical_roman_value(numeral).filter(|v| (1..=100).contains(v))?;
    let rank = match qualifier {
        None => 0,
        Some(word) => fraction_qualifier_rank(word)?,
    };
    Some(FractionLabel { value, rank })
}

fn parse_inciso_locator(label: &str) -> Option<usize> {
    let body = label.strip_suffix(')').unwrap_or(label);
    let mut chars = body.chars();
    let letter = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    letter_index(letter)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_roman_values() {
        assert_eq!(canonical_roman_value("IV"), Some(4));
        assert_eq!(canonical_roman_value("XCIX"), Some(99));
        assert_eq!(canonical_roman_value("IIII"), None);
        assert_eq!(canonical_roman_value("C"), Some(100));
        assert_eq!(canonical_roman_value(""), None);
    }

    #[test]
    fn fraction_locator_syntax() {
        assert!(parse_fraction_locator("X Quáter").is_some());
        assert!(parse_fraction_locator("X QUATER").is_some());
        assert!(parse_fraction_locator("XV Bis 1").is_none());
        assert!(parse_fraction_locator("iv").is_none());
        assert!(parse_fraction_locator("IIII").is_none());
        assert!(parse_fraction_locator(" I").is_none());
        assert!(parse_fraction_locator("I  Bis").is_none());
        assert!(parse_fraction_locator("I.").is_none());
        assert!(parse_fraction_locator("CI").is_none());
    }

    #[test]
    fn inciso_locator_syntax() {
        assert_eq!(parse_inciso_locator("a"), Some(0));
        assert_eq!(parse_inciso_locator("ñ)"), Some(14));
        assert!(parse_inciso_locator("A").is_none());
        assert!(parse_inciso_locator("a b").is_none());
        assert!(parse_inciso_locator("").is_none());
    }

    #[test]
    fn layout_fails_closed() {
        assert!(layout_ok("I. Uno.\n\nII. Dos."));
        assert!(!layout_ok("I. Uno.\nII. Dos."));
        assert!(!layout_ok("I. Uno.\n\n\nII. Dos."));
        assert!(!layout_ok(" I. Uno."));
    }
}
