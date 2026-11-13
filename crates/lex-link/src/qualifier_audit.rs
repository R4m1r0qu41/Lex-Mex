//! Read-only evidence about cited fractions: does a fraction named by a
//! reference qualifier actually appear as a fraction marker in the target
//! provision's text? Nothing here changes a reference edge; the result is
//! evidence for a human or a later linker rule.

use lex_core::{ReferenceQualifier, ReferenceQualifierType};

/// Where a cited fraction label was found in the target provision text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FractionOutcome {
    /// A line starts with the marker (`IV.`, `IV.-`, `IV.–` plus whitespace).
    FoundAtParagraphStart,
    /// The marker appears only inside a line, after `; `, `: `, `, `, `. ` or a
    /// conjunction (`y `, `o `): the shape of a fraction merged into the
    /// preceding paragraph at a page break.
    FoundMidParagraph,
    /// No marker for the label appears anywhere in the target text.
    Absent,
}

impl FractionOutcome {
    /// Stable `snake_case` name used in reports.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FoundAtParagraphStart => "found_at_paragraph_start",
            Self::FoundMidParagraph => "found_mid_paragraph",
            Self::Absent => "absent",
        }
    }
}

/// One cited fraction label and where it was found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FractionAudit {
    /// The `text` of the qualifier the label came from.
    pub qualifier_text: String,
    /// Normalised label: Roman numeral with optional single-space suffix.
    pub label: String,
    pub outcome: FractionOutcome,
}

const SUFFIXES: [&str; 6] = ["Bis", "Ter", "Quáter", "Quater", "Quinquies", "Sexies"];

/// Audit every `fraction` qualifier of one edge against the target text.
/// Labels are returned in qualifier order then citation order, without
/// duplicates inside one qualifier.
#[must_use]
pub fn audit_fraction_qualifiers(
    qualifiers: &[ReferenceQualifier],
    target_text: &str,
) -> Vec<FractionAudit> {
    let mut audits = Vec::new();
    for qualifier in qualifiers {
        if qualifier.qualifier_type != ReferenceQualifierType::Fraction {
            continue;
        }
        for label in parse_fraction_labels(&qualifier.text) {
            let outcome = locate_fraction(target_text, &label);
            audits.push(FractionAudit {
                qualifier_text: qualifier.text.clone(),
                label,
                outcome,
            });
        }
    }
    audits
}

/// Parse fraction labels out of qualifier text such as `fracción I`,
/// `fracciones XI, XII y XV`, `fracción II Bis`, `fracciones I a V` or a bare
/// `V`.
///
/// Matching is case-sensitive: only upper-case Roman numerals count, so words
/// such as `fracción` and OCR forms such as `ll` yield nothing. A range
/// `X a Y` expands to every plain numeral from X to Y. A range whose endpoint
/// carries a suffix (`I a II Bis`) or whose endpoints are out of order is
/// ambiguous and yields the two endpoints only.
#[must_use]
pub fn parse_fraction_labels(text: &str) -> Vec<String> {
    let tokens = numeral_tokens(text);
    let mut labels: Vec<String> = Vec::new();
    let mut push = |label: String| {
        if !labels.contains(&label) {
            labels.push(label);
        }
    };
    let mut index = 0;
    while index < tokens.len() {
        let token = &tokens[index];
        let is_range = tokens
            .get(index + 1)
            .is_some_and(|next| text[token.end..next.start].trim() == "a");
        if is_range {
            let next = &tokens[index + 1];
            match (&token.suffix, &next.suffix) {
                (None, None) if token.value <= next.value => {
                    for value in token.value..=next.value {
                        push(to_roman(value));
                    }
                }
                _ => {
                    push(token.label());
                    push(next.label());
                }
            }
            index += 2;
        } else {
            push(token.label());
            index += 1;
        }
    }
    labels
}

struct NumeralToken {
    start: usize,
    end: usize,
    value: u32,
    numeral: String,
    suffix: Option<&'static str>,
}

impl NumeralToken {
    fn label(&self) -> String {
        match self.suffix {
            Some(suffix) => format!("{} {suffix}", self.numeral),
            None => self.numeral.clone(),
        }
    }
}

fn numeral_tokens(text: &str) -> Vec<NumeralToken> {
    // Split into whitespace/punctuation separated words, keeping byte spans.
    let mut words: Vec<(usize, usize)> = Vec::new();
    let mut start: Option<usize> = None;
    for (index, ch) in text.char_indices() {
        if ch.is_alphanumeric() {
            start.get_or_insert(index);
        } else if let Some(begin) = start.take() {
            words.push((begin, index));
        }
    }
    if let Some(begin) = start {
        words.push((begin, text.len()));
    }
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < words.len() {
        let (begin, end) = words[index];
        if let Some(value) = roman_value(&text[begin..end]) {
            let mut token = NumeralToken {
                start: begin,
                end,
                value,
                numeral: text[begin..end].to_owned(),
                suffix: None,
            };
            // A suffix must follow after whitespace only (`II Bis`).
            if let Some(&(next_begin, next_end)) = words.get(index + 1)
                && text[end..next_begin].chars().all(char::is_whitespace)
                && let Some(suffix) = SUFFIXES.iter().find(|s| **s == &text[next_begin..next_end])
            {
                token.suffix = Some(suffix);
                token.end = next_end;
                index += 1;
            }
            tokens.push(token);
        }
        index += 1;
    }
    tokens
}

/// Strictly valid upper-case Roman numeral from I to CCCXCIX (letters I V X
/// L C only; fractions never reach D or M).
fn roman_value(word: &str) -> Option<u32> {
    if word.is_empty()
        || !word
            .chars()
            .all(|c| matches!(c, 'I' | 'V' | 'X' | 'L' | 'C'))
    {
        return None;
    }
    let mut total: i64 = 0;
    let digits: Vec<i64> = word
        .chars()
        .map(|c| match c {
            'I' => 1,
            'V' => 5,
            'X' => 10,
            'L' => 50,
            _ => 100,
        })
        .collect();
    for (index, digit) in digits.iter().enumerate() {
        if digits.get(index + 1).is_some_and(|next| next > digit) {
            total -= *digit;
        } else {
            total += *digit;
        }
    }
    let value = u32::try_from(total).ok()?;
    // Reject non-canonical spellings (`IIII`, `VX`, `IC`) by round-tripping.
    (value > 0 && to_roman(value) == word).then_some(value)
}

fn to_roman(mut value: u32) -> String {
    const TABLE: [(u32, &str); 9] = [
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut out = String::new();
    for (unit, glyph) in TABLE {
        while value >= unit {
            out.push_str(glyph);
            value -= unit;
        }
    }
    out
}

/// Locate a fraction marker for `label` in `text`. Any paragraph-start marker
/// wins over a mid-paragraph one.
#[must_use]
pub fn locate_fraction(text: &str, label: &str) -> FractionOutcome {
    let mut outcome = FractionOutcome::Absent;
    for (index, _) in text.match_indices(label) {
        if !marker_follows(&text[index + label.len()..]) {
            continue;
        }
        let before = &text[..index];
        if before.is_empty() || before.ends_with('\n') {
            return FractionOutcome::FoundAtParagraphStart;
        }
        if mid_paragraph_prefix(before) {
            outcome = FractionOutcome::FoundMidParagraph;
        }
    }
    outcome
}

/// After the numeral: `.`, optional `-` or `–`, then whitespace.
fn marker_follows(after: &str) -> bool {
    let Some(rest) = after.strip_prefix('.') else {
        return false;
    };
    let rest = rest
        .strip_prefix('-')
        .or_else(|| rest.strip_prefix('–'))
        .unwrap_or(rest);
    rest.chars().next().is_some_and(char::is_whitespace)
}

fn mid_paragraph_prefix(before: &str) -> bool {
    if ["; ", ": ", ", ", ". "].iter().any(|p| before.ends_with(p)) {
        return true;
    }
    ["y ", "o "].iter().any(|conjunction| {
        before.strip_suffix(conjunction).is_some_and(|rest| {
            rest.chars()
                .next_back()
                .is_none_or(|ch| !ch.is_alphanumeric())
        })
    })
}

#[cfg(test)]
mod qualifier_audit_tests {
    use super::*;
    use FractionOutcome::{Absent, FoundAtParagraphStart, FoundMidParagraph};

    fn fraction(text: &str) -> ReferenceQualifier {
        ReferenceQualifier {
            qualifier_type: ReferenceQualifierType::Fraction,
            text: text.to_owned(),
            start_char: None,
            end_char: None,
        }
    }

    fn labels(text: &str) -> Vec<String> {
        parse_fraction_labels(text)
    }

    #[test]
    fn qualifier_labels_single_list_bis_bare() {
        assert_eq!(labels("fracción I"), ["I"]);
        assert_eq!(labels("Fracción IV"), ["IV"]);
        assert_eq!(
            labels("fracciones XI, XII, XIII, XV, XVII y XVIII"),
            ["XI", "XII", "XIII", "XV", "XVII", "XVIII"]
        );
        assert_eq!(labels("fracciones II o III"), ["II", "III"]);
        assert_eq!(labels("fracción II Bis"), ["II Bis"]);
        assert_eq!(
            labels("fracciones I Bis y III Quáter"),
            ["I Bis", "III Quáter"]
        );
        assert_eq!(labels("V"), ["V"]);
    }

    #[test]
    fn qualifier_labels_ignore_ocr_and_words() {
        assert_eq!(labels("fracción ll"), Vec::<String>::new());
        assert_eq!(labels("fracciones l y ll"), Vec::<String>::new());
        assert_eq!(labels("fracción"), Vec::<String>::new());
        assert_eq!(labels("Bis"), Vec::<String>::new());
        assert_eq!(labels("fracciones IIII y V"), ["V"]);
    }

    #[test]
    fn qualifier_labels_range_expands() {
        assert_eq!(labels("fracciones I a V"), ["I", "II", "III", "IV", "V"]);
        assert_eq!(labels("fracciones VIII a XI"), ["VIII", "IX", "X", "XI"]);
        assert_eq!(labels("fracciones I a III y VI"), ["I", "II", "III", "VI"]);
    }

    #[test]
    fn qualifier_labels_ambiguous_range_returns_endpoints() {
        assert_eq!(labels("fracciones I a II Bis"), ["I", "II Bis"]);
        assert_eq!(labels("fracciones V a II"), ["V", "II"]);
    }

    #[test]
    fn qualifier_outcome_paragraph_start() {
        let text = "Intro:\n\nI. Uno;\n\nII.- Dos;\n\nIII.– Tres;\n\nII Bis. Cuatro";
        for label in ["I", "II", "III", "II Bis"] {
            assert_eq!(
                locate_fraction(text, label),
                FoundAtParagraphStart,
                "{label}"
            );
        }
        assert_eq!(locate_fraction("IV. Texto", "IV"), FoundAtParagraphStart);
    }

    #[test]
    fn qualifier_outcome_mid_paragraph() {
        for text in [
            "a; IV. texto",
            "a: IV. texto",
            "a, IV. texto",
            "a. IV. texto",
            "III. a y IV. texto",
            "III. a o IV.- texto",
        ] {
            assert_eq!(locate_fraction(text, "IV"), FoundMidParagraph, "{text}");
        }
    }

    #[test]
    fn qualifier_outcome_paragraph_start_wins() {
        assert_eq!(
            locate_fraction("a; IV. x\n\nIV. y", "IV"),
            FoundAtParagraphStart
        );
    }

    #[test]
    fn qualifier_outcome_absent_and_no_partial_matches() {
        assert_eq!(locate_fraction("XIV. Texto", "IV"), Absent);
        assert_eq!(locate_fraction("\n\nXIV. Texto; XIV. y", "IV"), Absent);
        assert_eq!(locate_fraction("CIV. Texto", "IV"), Absent);
        assert_eq!(locate_fraction("\nCIVIL. Texto", "IV"), Absent);
        assert_eq!(locate_fraction("el IV de enero", "IV"), Absent);
        assert_eq!(locate_fraction("muy IV. texto", "IV"), Absent);
        assert_eq!(locate_fraction("I Bis. Texto", "I"), Absent);
        assert_eq!(locate_fraction("VI. Texto\n\nII. Texto", "I"), Absent);
        assert_eq!(locate_fraction("IV Texto", "IV"), Absent);
        assert_eq!(locate_fraction("IV.Texto", "IV"), Absent);
        assert_eq!(locate_fraction("iv. Texto", "IV"), Absent);
        assert_eq!(locate_fraction("", "I"), Absent);
    }

    #[test]
    fn qualifier_outcome_multibyte_and_nbsp() {
        assert_eq!(
            locate_fraction("Señalará: á é\n\nIV.\u{a0}Texto ñ", "IV"),
            FoundAtParagraphStart
        );
        assert_eq!(locate_fraction("Sañó, é; IV. ñ", "IV"), FoundMidParagraph);
    }

    #[test]
    fn qualifier_audit_uses_fraction_qualifiers_only() {
        let mut paragraph = fraction("segundo párrafo");
        paragraph.qualifier_type = ReferenceQualifierType::Paragraph;
        let audits = audit_fraction_qualifiers(
            &[paragraph, fraction("fracciones I y IV")],
            "I. Uno\n\nIV. Cuatro",
        );
        assert_eq!(audits.len(), 2);
        assert_eq!(audits[0].label, "I");
        assert_eq!(audits[1].outcome, FoundAtParagraphStart);
        assert_eq!(audits[1].qualifier_text, "fracciones I y IV");
        assert_eq!(FoundAtParagraphStart.as_str(), "found_at_paragraph_start");
    }
}
