//! Deterministic analysis over supplied canonical records: initial repeal and
//! temporal evidence classification, commencement derivation, corpus validation
//! and audited-review application. Nothing here parses source documents or
//! resolves citations; every function takes explicit records and configuration.

pub mod initial;
pub mod review;
pub mod temporal_derive;
pub mod validate;

pub use initial::{initial_repeals, initial_temporal_status};
pub use review::{apply_legal_operator_reviews, apply_outdated_reference_approvals};
pub use validate::{
    CorpusExpectations, CorpusView, validate_corpus, validate_lritf, validate_reference,
    validate_references,
};
