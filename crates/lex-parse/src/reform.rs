//! Reform-decree evidence record construction.

use chrono::NaiveDate;
use lex_core::TemporalEvidence;

use crate::grammar::slug;

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
    amendment_marks: Vec<u32>,
) -> TemporalEvidence {
    let date = date.format("%Y-%m-%d");
    TemporalEvidence {
        provision_id: format!(
            "{instrument_id}:amendment:{date}:transitory:{}",
            slug(ordinal)
        ),
        label: format!("Transitorio {ordinal} — {resolution_word} DOF {date}"),
        text,
        amendment_marks,
    }
}
