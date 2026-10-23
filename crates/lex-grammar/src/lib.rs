//! Lexical helpers shared across parsing, linking and analysis: article labels,
//! Spanish dates, exact character slicing and connector words. Nothing here
//! classifies legal meaning.

pub mod labels;

use chrono::NaiveDate;
use regex::Regex;

static DOF_DATE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"\bDOF\s+(\d{2})-(\d{2})-(\d{4})").expect("static regex")
});

/// Whether text carries any `DOF dd-mm-yyyy` amendment note, valid date or not.
#[must_use]
pub fn mentions_dof_date(text: &str) -> bool {
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

#[must_use]
pub fn latest_dof_date(text: &str) -> Option<NaiveDate> {
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

#[must_use]
pub fn char_slice(value: &str, start: usize, end: usize) -> Option<String> {
    if start > end || end > value.chars().count() {
        return None;
    }
    Some(value.chars().skip(start).take(end - start).collect())
}

#[must_use]
pub fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[must_use]
pub fn slug(value: &str) -> String {
    value
        .to_lowercase()
        .replace('á', "a")
        .replace('é', "e")
        .replace('í', "i")
        .replace('ó', "o")
        .replace(['ú', 'ü'], "u")
        .replace(' ', "-")
}

#[must_use]
pub fn spanish_date(day: &str, month: &str, year: &str) -> Option<NaiveDate> {
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

#[must_use]
pub fn is_connector(word: &str) -> bool {
    matches!(
        word.to_lowercase().as_str(),
        "de" | "del" | "la" | "el" | "los" | "las" | "y" | "o" | "en" | "a" | "una" | "un"
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn latest_dof_date_reads_every_date_in_a_note_list() {
        use crate::latest_dof_date;
        use chrono::NaiveDate;
        let single = "Artículo reformado DOF 10-01-2014";
        assert_eq!(
            latest_dof_date(single),
            NaiveDate::from_ymd_opt(2014, 1, 10)
        );
        let listed =
            "Cantidades actualizadas DOF 13-01-2016, 12-01-2017,\n   30-12-2024 y 28-12-2025";
        assert_eq!(
            latest_dof_date(listed),
            NaiveDate::from_ymd_opt(2025, 12, 28)
        );
        let unrelated = "Fe de erratas DOF 10-01-2025; plazo de 01-02-2030";
        assert_eq!(
            latest_dof_date(unrelated),
            NaiveDate::from_ymd_opt(2025, 1, 10)
        );
    }
}
