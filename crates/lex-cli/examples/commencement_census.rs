//! Read-only census of every instrument's commencement verdict.
//!
//! `cargo run --locked -p lex-cli --example commencement_census -- <repo root>`
//!
//! Prints one tab-separated line per instrument: slug, verdict (`resolved`,
//! `bounded` or `skipped`), the pattern or skip code, the date where there is
//! one, and the number of article determinations the rule would emit if every
//! article were untouched. It writes nothing and uses an empty business-day
//! calendar, so business-day clauses never resolve. Its purpose is to compare
//! the rule's reach before and after a recognizer change.

use std::fs;
use std::path::Path;

use chrono::NaiveDate;
use lex_core::{Instrument, Provision, ReviewStatus};
use lex_parse::temporal_derive::{Commencement, derive_article_temporal_determinations};

fn main() -> anyhow::Result<()> {
    let root = std::env::args().nth(1).unwrap_or_else(|| ".".to_owned());
    let corpus = Path::new(&root).join("corpus/mx");
    let today = NaiveDate::from_ymd_opt(2026, 11, 17).expect("fixed date");
    let mut slugs = fs::read_dir(&corpus)?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().join("provisions.json").exists())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    slugs.sort();
    let trigger = regex::Regex::new(r"(?i)entrar[áa]n?\s+en\s+vigor")?;
    for slug in slugs {
        let dir = corpus.join(&slug);
        let instrument: Instrument =
            serde_json::from_slice(&fs::read(dir.join("instrument.json"))?)?;
        let mut provisions: Vec<Provision> =
            serde_json::from_slice(&fs::read(dir.join("provisions.json"))?)?;
        // Measure reach, not current state: treat every provision as untouched.
        for provision in &mut provisions {
            provision.review_status = ReviewStatus::NotAnalyzed;
        }
        let outcome = derive_article_temporal_determinations(&instrument, &provisions, today);
        let (verdict, detail, date) = match &outcome.commencement {
            None => ("none", String::new(), String::new()),
            Some(Commencement::Resolved {
                effective_from,
                pattern,
                ..
            }) => (
                "resolved",
                (*pattern).to_owned(),
                effective_from.to_string(),
            ),
            Some(Commencement::Bounded {
                latest, pattern, ..
            }) => ("bounded", (*pattern).to_owned(), latest.to_string()),
            Some(Commencement::Skipped { reason, .. }) => {
                ("skipped", reason.code().to_owned(), String::new())
            }
        };
        // The single commencement transitory, whitespace-collapsed, when there
        // is exactly one; otherwise empty.
        let clauses = provisions
            .iter()
            .filter(|provision| {
                provision.provision_type == lex_core::ProvisionType::Transitory
                    && trigger.is_match(&provision.text)
            })
            .collect::<Vec<_>>();
        let clause = match clauses.as_slice() {
            [one] => one.text.split_whitespace().collect::<Vec<_>>().join(" "),
            _ => String::new(),
        };
        println!(
            "{slug}\t{verdict}\t{detail}\t{date}\t{}\t{clause}",
            outcome.determinations.len()
        );
    }
    Ok(())
}
