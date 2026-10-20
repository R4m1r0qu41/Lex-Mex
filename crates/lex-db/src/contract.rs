use crate::{digest, input::GitInput};
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeSet, path::Path};

pub const VERSION: &str = "1";
pub const DEBT_PATH: &str = "adapters/shared/_known-structural-debt.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Handling {
    Normalized,
    EvidenceOnly,
    ExcludedStandardBody,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputFile {
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
    pub handling: Handling,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildSpecification {
    pub schema_version: String,
    pub corpus_commit: String,
    pub builder_commit: String,
    pub cargo_lock_sha256: String,
    pub rust_toolchain: String,
    pub sqlite_version: String,
    pub sqlite_compile_options: Vec<String>,
    pub source_commit_timestamp: i64,
    pub inclusion_policy: String,
    pub inputs: Vec<InputFile>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatabaseManifest {
    pub schema_version: String,
    pub release_id: String,
    pub specification: BuildSpecification,
    pub database_sha256: String,
    pub logical_content_sha256: String,
}

pub fn validate_manifest(value: &Value) -> Result<DatabaseManifest> {
    let schema: Value = serde_json::from_str(include_str!(
        "../../../schemas/database-manifest.schema.json"
    ))?;
    jsonschema::validator_for(&schema)?
        .validate(value)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let manifest: DatabaseManifest = serde_json::from_value(value.clone())?;
    let spec = &manifest.specification;
    ensure!(
        spec.inclusion_policy == "federal_with_standards_catalogue",
        "unknown inclusion policy"
    );
    let paths: Vec<_> = spec.inputs.iter().map(|f| f.path.as_str()).collect();
    ensure!(
        paths.windows(2).all(|w| w[0] < w[1]),
        "input paths must be unique and sorted"
    );
    for file in &spec.inputs {
        safe_path(&file.path)?;
    }
    ensure!(
        manifest.release_id == digest(&serde_json::to_vec(spec)?),
        "release identity does not match specification"
    );
    Ok(manifest)
}

pub fn safe_path(path: &str) -> Result<()> {
    ensure!(
        !path.is_empty() && !path.contains('\\'),
        "invalid relative input path"
    );
    ensure!(
        Path::new(path)
            .components()
            .all(|c| matches!(c, std::path::Component::Normal(_))),
        "non-relative input path: {path}"
    );
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DebtSeed {
    pub schema_version: String,
    pub source_corpus_commit: String,
    pub decision_source_path: String,
    pub decision_source_sha256: String,
    pub rows: Vec<DebtRow>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DebtRow {
    pub instrument_id: String,
    pub kind: String,
    pub printed_label: String,
    pub heading_text: String,
    pub containing_provision_id: String,
    pub start_char: usize,
    pub end_char: usize,
    pub source_path: String,
    pub source_blob_oid: String,
    pub source_sha256: String,
    pub review_status: String,
    pub reviewer: String,
    pub reviewed_at: String,
    pub decision_reference: String,
}

pub fn validate_seed(git: &GitInput, bytes: &[u8]) -> Result<DebtSeed> {
    let value: Value = serde_json::from_slice(bytes)?;
    let schema: Value =
        serde_json::from_str(include_str!("../../../schemas/structural-debt.schema.json"))?;
    jsonschema::validator_for(&schema)?
        .validate(&value)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let seed: DebtSeed = serde_json::from_value(value)?;
    git.require_ancestor(&seed.source_corpus_commit)?;
    ensure!(
        digest(&git.read(&seed.decision_source_path)?) == seed.decision_source_sha256,
        "debt decision source changed; review and rebind seed"
    );
    let mut identities = BTreeSet::new();
    for row in &seed.rows {
        ensure!(
            identities.insert((&row.instrument_id, &row.printed_label)),
            "duplicate debt identity"
        );
        safe_path(&row.source_path)?;
        let raw = git.read(&row.source_path)?;
        ensure!(
            digest(&raw) == row.source_sha256,
            "stale debt source: {}",
            row.source_path
        );
        ensure!(
            git.blob_oid(&row.source_path)? == row.source_blob_oid,
            "debt blob mismatch"
        );
        let provisions: Vec<lex_core::Provision> = serde_json::from_slice(&raw)?;
        let container = provisions
            .iter()
            .find(|p| p.id == row.containing_provision_id)
            .context("debt container absent")?;
        ensure!(
            container.instrument_id == row.instrument_id,
            "debt owner mismatch"
        );
        ensure!(
            row.start_char < row.end_char && row.end_char <= container.text.chars().count(),
            "invalid debt span"
        );
        let span: String = container
            .text
            .chars()
            .skip(row.start_char)
            .take(row.end_char - row.start_char)
            .collect();
        ensure!(span == row.heading_text, "stale debt heading span");
        let target = format!(
            "{}:article:{}",
            row.instrument_id,
            lex_parse::labels::canonical_slug(&row.printed_label)
        );
        if provisions.iter().any(|p| p.id == target) {
            bail!("resolved debt still in seed: {target}");
        }
    }
    Ok(seed)
}
