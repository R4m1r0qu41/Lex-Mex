//! Lexical helpers shared across parsing, linking and analysis: ordinals, dates, character slicing.

use chrono::NaiveDate;
use regex::Regex;

static DOF_DATE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"\bDOF\s+(\d{2})-(\d{2})-(\d{4})").expect("static regex")
});

/// Whether text carries any `DOF dd-mm-yyyy` amendment note, valid date or not.
pub(crate) fn mentions_dof_date(text: &str) -> bool {
    DOF_DATE.is_match(text)
}

/// A `DOF` note followed by its date and any further dates in the same
/// comma- or "y"-separated list, such as `DOF 13-01-2016, 12-01-2017 y 19-01-2018`.
static DOF_DATE_RUN: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"\bDOF\s+\d{2}-\d{2}-\d{4}(?:\s*(?:,|y)\s*\d{2}-\d{2}-\d{4})*")
        .expect("static regex")
});

static LIST_DATE: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"(\d{2})-(\d{2})-(\d{4})").expect("static regex"));

pub(crate) fn latest_dof_date(text: &str) -> Option<NaiveDate> {
    DOF_DATE_RUN
        .find_iter(text)
        .flat_map(|run| LIST_DATE.captures_iter(run.as_str()))
        .filter_map(|captures| {
            NaiveDate::from_ymd_opt(
                captures[3].parse().ok()?,
                captures[2].parse().ok()?,
                captures[1].parse().ok()?,
            )
        })
        .max()
}

pub(crate) fn char_slice(value: &str, start: usize, end: usize) -> Option<String> {
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

pub(crate) fn is_connector(word: &str) -> bool {
    matches!(
        word.to_lowercase().as_str(),
        "de" | "del" | "la" | "el" | "los" | "las" | "y" | "o" | "en" | "a" | "una" | "un"
    )
}
