//! Immutable `SQLite` corpus projection, independent verification and read-only queries.
pub mod build;
pub mod contract;
pub mod input;
pub mod load;
pub mod logical;
mod parity;
pub mod probe;
pub mod queries;
pub mod validation;
pub mod verify;

use sha2::{Digest, Sha256};

#[must_use]
pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
