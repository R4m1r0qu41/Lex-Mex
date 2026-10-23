//! Initial temporal status and repeal-note classification evaluated at parse time.

use chrono::NaiveDate;
use regex::Regex;

/// The temporal status a freshly parsed provision starts with, before any
/// temporal determination is applied. A consolidated current text proves the
/// source wording, not the legal effectiveness of every provision it prints,
/// so ordinary provisions start `Unknown`. An explicit repeal note —
/// `(Se deroga)`, `Derogado` — is a narrow deterministic source fact and
/// starts `Repealed`; a parenthesised note repealing named paragraphs or
/// fractions starts `PartiallyRepealed` (see [`initial_repeals`]). A later
/// temporal determination (reapplied from persisted results for analyzed
/// instruments) overrides this initial state.
#[must_use]
pub fn initial_temporal_status(text: &str) -> lex_core::TemporalStatus {
    if !initial_repeals(text).is_empty() {
        return lex_core::TemporalStatus::PartiallyRepealed;
    }
    let head = text.trim_start().to_lowercase();
    let standalone_marker = [
        "(se deroga",
        "se deroga",
        "(derogado",
        "derogado",
        "(derogada",
        "derogada",
    ]
    .iter()
    .any(|marker| {
        head.strip_prefix(marker).is_some_and(|tail| {
            // A marker is standalone when nothing continues its line: end of
            // text, punctuation, or a line break (the Diputados layout puts the
            // marker in its own paragraph above the amendment history).
            let rest = tail.trim_start_matches([' ', '\t']);
            rest.chars()
                .next()
                .is_none_or(|character| !character.is_alphanumeric())
        })
    });
    if standalone_marker {
        lex_core::TemporalStatus::Repealed
    } else {
        lex_core::TemporalStatus::Unknown
    }
}

/// A parenthesised note at the very start of a provision that repeals named
/// paragraphs or fractions of it, such as `(Se deroga el primer párrafo)` or
/// `(Se derogan los párrafos segundo a cuarto)`. Only this unambiguous shape is
/// recognised; a note that names another instrument or article, or that
/// continues into prose, is left for review. The ordinals are the source's own
/// statements about the text before the repeal.
#[must_use]
pub fn initial_repeals(text: &str) -> Vec<lex_core::ProvisionRepeal> {
    static NOTE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
        Regex::new(
            r"(?is)^\s*\(\s*se\s+derogan?\s+(?:el\s+|los\s+|la\s+|las\s+)?(?:(?P<list_before>[^()]*?)\s+(?P<scope_after>p[áa]rrafos?|fracci[óo]n(?:es)?)|(?P<scope_before>p[áa]rrafos?|fracci[óo]n(?:es)?)\s+(?P<list_after>[^()]*?))\s*(?:,?\s*inclusive)?\s*\)\s*(?:\.|\n|$)",
        )
        .expect("static regex")
    });
    static DOF_AFTER: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
        Regex::new(r"(?i)derogad[oa]s?\s+DOF\s+(\d{2})-(\d{2})-(\d{4})").expect("static regex")
    });
    let Some(captures) = NOTE.captures(text) else {
        return Vec::new();
    };
    let (Some(scope_match), Some(list_match)) = (
        captures
            .name("scope_after")
            .or_else(|| captures.name("scope_before")),
        captures
            .name("list_before")
            .or_else(|| captures.name("list_after")),
    ) else {
        return Vec::new();
    };
    let (scope_word, list) = (scope_match.as_str(), list_match.as_str());
    let scope = if scope_word.to_lowercase().starts_with('p') {
        lex_core::RepealScope::Paragraph
    } else {
        lex_core::RepealScope::Fraction
    };
    let Some(ordinals) = repeal_ordinals(list, scope) else {
        return Vec::new();
    };
    let whole = captures.get(0).map_or(0, |matched| matched.end());
    let window: String = text[whole..].chars().take(160).collect();
    let dof_date = DOF_AFTER.captures(&window).and_then(|found| {
        NaiveDate::from_ymd_opt(
            found[3].parse().ok()?,
            found[2].parse().ok()?,
            found[1].parse().ok()?,
        )
    });
    vec![lex_core::ProvisionRepeal {
        scope,
        ordinals,
        cause: lex_core::RepealCause::Legislative,
        dof_date,
        renumbering: text.to_lowercase().contains("recorr"),
    }]
}

/// The word forms of paragraph and fraction ordinals a repeal note may use.
const ORDINAL_WORDS: &[(&str, &str)] = &[
    ("primero", "1"),
    ("primer", "1"),
    ("segundo", "2"),
    ("tercero", "3"),
    ("tercer", "3"),
    ("cuarto", "4"),
    ("quinto", "5"),
    ("sexto", "6"),
    ("séptimo", "7"),
    ("octavo", "8"),
    ("noveno", "9"),
    ("décimo", "10"),
];

/// Ordinals from a list like `primer`, `segundo a cuarto`, `1, 2 y 4` or
/// `III y V`; `None` when any item is not an ordinal.
pub(crate) fn repeal_ordinals(list: &str, scope: lex_core::RepealScope) -> Option<Vec<String>> {
    let normalized = list.to_lowercase();
    let mut ordinals: Vec<String> = Vec::new();
    let mut range_from: Option<u32> = None;
    for token in normalized
        .split(|character: char| character == ',' || character.is_whitespace())
        .filter(|token| !token.is_empty())
    {
        if token == "y" || token == "e" {
            continue;
        }
        if token == "a" || token == "al" {
            range_from = ordinals.last().and_then(|last| last.parse().ok());
            range_from?;
            continue;
        }
        let value = ORDINAL_WORDS
            .iter()
            .find(|(word, _)| *word == token)
            .map(|(_, number)| (*number).to_owned())
            .or_else(|| token.parse::<u32>().ok().map(|number| number.to_string()))
            .or_else(|| {
                (scope == lex_core::RepealScope::Fraction
                    && token.chars().all(|c| matches!(c, 'i' | 'v' | 'x')))
                .then(|| token.to_uppercase())
            })?;
        if let Some(from) = range_from.take() {
            let to: u32 = value.parse().ok()?;
            if to <= from || to - from > 30 {
                return None;
            }
            ordinals.extend((from + 1..=to).map(|number| number.to_string()));
        } else {
            ordinals.push(value);
        }
    }
    (range_from.is_none() && !ordinals.is_empty()).then_some(ordinals)
}

#[cfg(test)]
mod tests {

    use lex_core::TemporalStatus;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn unanalyzed_temporal_status_is_unknown_except_for_an_explicit_repeal_note() {
        assert_eq!(
            initial_temporal_status("La presente Ley es de orden público."),
            TemporalStatus::Unknown
        );
        assert_eq!(
            initial_temporal_status(
                "[Texto invalidado.] Artículo declarado inválido por sentencia de la SCJN."
            ),
            TemporalStatus::Unknown
        );
        assert_eq!(
            initial_temporal_status("(Se deroga)."),
            TemporalStatus::Repealed
        );
        assert_eq!(
            initial_temporal_status("Se derogan los artículos 33, 33-A y 33-B de otra ley."),
            TemporalStatus::Unknown
        );
        assert_eq!(
            initial_temporal_status("Se deroga el artículo 27 de otra ley."),
            TemporalStatus::Unknown
        );
    }

    #[test]
    fn repeal_marker_alone_on_its_line_is_a_repeal_but_a_sentence_is_not() {
        // Diputados repealed articles: the marker is its own paragraph, followed
        // by the retained amendment-history line (LACP, LGOAAC, CCOM).
        for text in [
            "Se deroga\n\nArtículo derogado DOF 13-08-2009",
            "Se deroga\n\nArtículo reformado DOF 15-07-1993. Derogado DOF 10-01-2014",
            "Derogado\n\nArtículo derogado DOF 10-01-2014",
            "(Se deroga)\n\nArtículo derogado DOF 13-08-2009",
            "Se deroga  \n\nArtículo derogado DOF 13-08-2009",
        ] {
            assert_eq!(
                initial_temporal_status(text),
                TemporalStatus::Repealed,
                "{text:?}"
            );
        }
        for text in [
            "Se deroga el artículo 27 de otra ley.\n\nOtro párrafo.",
            "Se deroga la Ley de otra materia publicada en 1990.",
            "Derogado capítulo sexto de las sociedades.",
        ] {
            assert_eq!(
                initial_temporal_status(text),
                TemporalStatus::Unknown,
                "{text:?}"
            );
        }
    }
}
