//! Citation extraction and defined-term usage matching over explicitly supplied
//! records, terms and context. Nothing here reads files, parses source
//! documents or depends on parser options.

pub mod references;
pub mod usages;

pub use references::{
    HistoricalTarget, InstrumentContextPolicy, ReferenceOptions, extract_internal_references,
    extract_references,
};
pub use usages::{
    extract_quoted_term_usages, extract_term_usages, extract_term_usages_with_lowercase,
};
