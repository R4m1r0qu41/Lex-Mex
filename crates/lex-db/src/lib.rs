//! Read-only corpus input contracts and a small executable `SQLite` contract probe.
//! Full-corpus publication and release verification belong to later milestones.
pub mod build;
pub mod contract;
pub mod input;
pub mod load;
mod parity;
pub mod probe;
pub mod validation;

use sha2::{Digest, Sha256};

#[must_use]
pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
