use crate::{digest, input::GitInput};
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

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
/// Version 1 hashes serialization in this declaration's field order.
/// Changing field order or serialization requires a manifest version change.
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
    pub review_source_path: String,
    pub review_source_sha256: String,
    pub decision_sections: Vec<DecisionSection>,
    pub rows: Vec<DebtRow>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionSection {
    pub reference: String,
    pub heading: String,
    pub sha256: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
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
    let source = git.at(&seed.source_corpus_commit)?;
    validate_decision_sections(git, &source, &seed)?;
    validate_review_set(git, &source, &seed)?;
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

/// Exact Markdown section bytes, including its heading and trailing whitespace.
/// A duplicate heading is ambiguous and fails rather than picking one.
pub fn decision_section<'a>(text: &'a str, heading: &str) -> Result<&'a str> {
    ensure!(
        heading.starts_with("## "),
        "decision heading must be level two"
    );
    let mut starts = Vec::new();
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        if line.trim_end_matches(['\r', '\n']) == heading {
            starts.push(offset);
        }
        offset += line.len();
    }
    ensure!(
        starts.len() == 1,
        "decision section absent or ambiguous: {heading}"
    );
    let start = starts[0];
    let tail = &text[start..];
    let first = tail.find('\n').map_or(tail.len(), |n| n + 1);
    let end = tail[first..]
        .find("\n## ")
        .map_or(text.len(), |n| start + first + n + 1);
    // The next section can start immediately after the heading's newline.
    let end = if tail[first..].starts_with("## ") {
        start + first
    } else {
        end
    };
    Ok(&text[start..end])
}

fn validate_decision_sections(git: &GitInput, source: &GitInput, seed: &DebtSeed) -> Result<()> {
    let original = source.read(&seed.decision_source_path)?;
    ensure!(
        digest(&original) == seed.decision_source_sha256,
        "decision snapshot digest mismatch"
    );
    let original = String::from_utf8(original)?;
    let current = String::from_utf8(git.read(&seed.decision_source_path)?)?;
    let mut references = BTreeSet::new();
    for section in &seed.decision_sections {
        ensure!(
            references.insert(section.reference.as_str()),
            "duplicate decision section reference"
        );
        ensure!(
            section.reference.split('#').next() == Some(seed.decision_source_path.as_str()),
            "decision reference path mismatch"
        );
        ensure!(
            digest(decision_section(&original, &section.heading)?.as_bytes()) == section.sha256,
            "decision section snapshot mismatch"
        );
        ensure!(
            digest(decision_section(&current, &section.heading)?.as_bytes()) == section.sha256,
            "cited decision section changed; review and rebind seed"
        );
    }
    for row in &seed.rows {
        ensure!(
            references.contains(row.decision_reference.as_str()),
            "debt row has an unbound decision reference"
        );
    }
    Ok(())
}

fn validate_review_set(git: &GitInput, source: &GitInput, seed: &DebtSeed) -> Result<()> {
    let raw = source.read(&seed.review_source_path)?;
    ensure!(
        digest(&raw) == seed.review_source_sha256,
        "review snapshot digest mismatch"
    );
    ensure!(
        digest(&git.read(&seed.review_source_path)?) == seed.review_source_sha256,
        "current review ledger changed; review and rebind seed"
    );
    let review: Value = serde_json::from_slice(&raw)?;
    source.require_ancestor(
        review["corpus_commit"]
            .as_str()
            .context("review corpus commit missing")?,
    )?;
    ensure!(
        seed.decision_sections
            .iter()
            .any(|s| Some(s.reference.as_str()) == review["decision_reference"].as_str()),
        "review decision reference is not bound"
    );
    let rows = review["rows"].as_array().context("review rows absent")?;
    let mut expected = BTreeMap::new();
    for row in rows.iter().filter(|r| r["review_status"] == "accepted") {
        let mut fields = serde_json::Map::new();
        for key in [
            "instrument_id",
            "kind",
            "printed_label",
            "heading_text",
            "containing_provision_id",
            "start_char",
            "end_char",
            "source_path",
            "source_blob_oid",
            "source_sha256",
            "review_status",
            "reviewer",
            "reviewed_at",
            "decision_reference",
        ] {
            fields.insert(
                key.into(),
                row.get(key)
                    .with_context(|| format!("accepted review row lacks {key}"))?
                    .clone(),
            );
        }
        let row: DebtRow = serde_json::from_value(Value::Object(fields))?;
        ensure!(
            expected
                .insert((row.instrument_id.clone(), row.printed_label.clone()), row)
                .is_none(),
            "duplicate accepted review identity"
        );
    }
    ensure!(
        expected.len() == seed.rows.len(),
        "seed does not contain exactly the accepted review rows"
    );
    for row in &seed.rows {
        ensure!(
            expected.get(&(row.instrument_id.clone(), row.printed_label.clone())) == Some(row),
            "seed row differs from accepted review: {} {}",
            row.instrument_id,
            row.printed_label
        );
    }
    Ok(())
}
