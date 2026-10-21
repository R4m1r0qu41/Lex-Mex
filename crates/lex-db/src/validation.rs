//! Read-only build gates over the complete pinned snapshot.
use crate::input::Snapshot;
use anyhow::{Context, Result, ensure};
use lex_core::{Corpus, GlossaryCoverage, Severity};
use lex_parse::{CorpusExpectations, CorpusView, validate_corpus};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::{BTreeMap, HashSet};

fn read<T: DeserializeOwned>(data: &Snapshot, path: &str) -> Result<T> {
    serde_json::from_slice(data.get(path).with_context(|| format!("missing {path}"))?)
        .with_context(|| format!("invalid {path}"))
}
fn optional<T: DeserializeOwned + Default>(data: &Snapshot, path: &str) -> Result<T> {
    data.get(path)
        .map_or_else(|| Ok(T::default()), |b| Ok(serde_json::from_slice(b)?))
}

fn corpora(data: &Snapshot) -> Result<BTreeMap<String, Corpus>> {
    let mut result = BTreeMap::new();
    for path in data.keys().filter(|p| p.ends_with("/instrument.json")) {
        let directory = path.rsplit_once('/').context("invalid path")?.0;
        result.insert(
            directory.to_owned(),
            Corpus {
                instrument: read(data, path)?,
                provisions: read(data, &format!("{directory}/provisions.json"))?,
                references: read(data, &format!("{directory}/references.json"))?,
                terms: read(data, &format!("{directory}/terms.json"))?,
                term_usages: read(data, &format!("{directory}/term-usages.json"))?,
                enactment_evidence: optional(
                    data,
                    &format!("{directory}/enactment-evidence.json"),
                )?,
                amendment_references: optional(
                    data,
                    &format!("{directory}/amendment-references.json"),
                )?,
            },
        );
    }
    Ok(result)
}

fn schemas(data: &Snapshot) -> Result<Vec<String>> {
    let mut issues = Vec::new();
    for (filename, schema_path) in [
        ("references.json", "schemas/reference-edge.schema.json"),
        (
            "glossary-coverage.json",
            "schemas/glossary-coverage.schema.json",
        ),
        ("terms.json", "schemas/defined-term.schema.json"),
        ("term-usages.json", "schemas/term-usage.schema.json"),
        (
            "source-manifest.json",
            "schemas/source-manifest.schema.json",
        ),
        (
            "enactment-evidence.json",
            "schemas/enactment-evidence.schema.json",
        ),
        ("standard.json", "schemas/standard-metadata.schema.json"),
    ] {
        let schema: Value = read(data, schema_path)?;
        let validator = jsonschema::validator_for(&schema)?;
        for path in data
            .keys()
            .filter(|p| p.starts_with("corpus/mx/") && p.ends_with(&format!("/{filename}")))
        {
            let value: Value = read(data, path)?;
            let mut failures = 0;
            let mut first = None;
            for (pointer, row) in crate::load::objects(&value)? {
                if let Err(e) = validator.validate(row) {
                    failures += 1;
                    if first.is_none() {
                        first = Some(format!("{path}{pointer}: {e}"));
                    }
                }
            }
            if let Some(first) = first {
                issues.push(format!("{failures} invalid records; first: {first}"));
            }
        }
    }
    Ok(issues)
}

fn adapter(data: &Snapshot, id: &str) -> Result<Value> {
    let mut matches = Vec::new();
    for path in data.keys().filter(|p| {
        p.starts_with("adapters/")
            && std::path::Path::new(p)
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("json"))
    }) {
        let value: Value = read(data, path)?;
        if value["instrument_id"] == id {
            matches.push(value);
        }
    }
    ensure!(
        matches.len() == 1,
        "instrument must have exactly one adapter: {id}"
    );
    matches.pop().context("adapter absent")
}
fn count(config: &Value, key: &str) -> Result<Option<usize>> {
    config
        .get(key)
        .filter(|v| !v.is_null())
        .map(|v| {
            v.as_u64()
                .context("count must be integer")
                .and_then(|n| Ok(usize::try_from(n)?))
        })
        .transpose()
}

fn validate_one(
    data: &Snapshot,
    directory: &str,
    corpus: &Corpus,
    all: &BTreeMap<String, Corpus>,
    accepted: &Value,
) -> Result<()> {
    let config = adapter(data, &corpus.instrument.id)?;
    let targets: HashSet<_> = all
        .values()
        .flat_map(|c| c.provisions.iter().map(|p| p.id.clone()))
        .collect();
    let terms: HashSet<_> = all
        .values()
        .flat_map(|c| c.terms.iter().map(|t| t.id.clone()))
        .collect();
    let report = validate_corpus(
        &CorpusView {
            instrument_id: &corpus.instrument.id,
            official_title: Some(&corpus.instrument.official_title),
            provisions: &corpus.provisions,
            references: &corpus.references,
            terms: &corpus.terms,
            term_usages: &corpus.term_usages,
            amendment_references: &corpus.amendment_references,
        },
        &CorpusExpectations {
            min_articles: count(&config, "expected_min_articles")?,
            articles: count(&config, "expected_articles")?,
            transitories: count(&config, "expected_transitories")?,
            annexes: count(&config, "expected_annexes")?.unwrap_or(0),
            require_chapter_context: config["parser"] == "ifpe-dcg",
            allow_article_gaps: config["allow_article_gaps"].as_bool().unwrap_or(false),
        },
        &targets,
        &terms,
    );
    let errors: Vec<_> = report
        .issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .map(|i| format!("{}: {}", i.code, i.message))
        .collect();
    // Committed graph may predate a failed relink. Do not fabricate failed edges
    // or resolve accepted gaps. Any present graph validation error is fatal.
    ensure!(errors.is_empty(), "{directory}: {}", errors.join("; "));
    ensure!(
        config["instrument_id"] == corpus.instrument.id,
        "adapter owner differs"
    );
    for edge in &corpus.references {
        if edge.resolution_rule == Some(lex_core::ResolutionRule::ParentShorthand) {
            ensure!(
                config["parent_instrument_id"].as_str() == Some(edge.target_instrument_id.as_str()),
                "parent rule mismatch: {}",
                edge.id
            );
        }
        if edge.resolution_status == lex_core::ReferenceResolutionStatus::Resolved {
            ensure!(
                all.values()
                    .any(|c| c.instrument.id == edge.target_instrument_id
                        && c.provisions
                            .iter()
                            .any(|p| p.id == edge.target_provision_id)),
                "resolved target owner differs: {}",
                edge.id
            );
        }
    }
    lex_core::validate_publication_metadata(corpus).map_err(anyhow::Error::msg)?;
    let coverage: GlossaryCoverage = read(data, &format!("{directory}/glossary-coverage.json"))?;
    coverage
        .validate(
            &corpus.instrument,
            &corpus.provisions,
            &corpus.terms,
            &corpus.term_usages,
        )
        .map_err(anyhow::Error::msg)?;
    let manifest: Value = read(data, &format!("{directory}/source-manifest.json"))?;
    ensure!(
        manifest["instrument_id"] == corpus.instrument.id
            && manifest["source_sha256"] == corpus.instrument.source_sha256
            && manifest["extracted_text_sha256"] == corpus.instrument.extracted_text_sha256,
        "manifest binding mismatch: {directory}"
    );
    if let Some(notice) = &corpus.instrument.source_repeal_notice {
        if notice.successor_clause.is_some() {
            ensure!(
                all.values()
                    .any(|c| notice.matches_successor(&corpus.instrument.source_sha256, c)),
                "repeal evidence missing: {directory}"
            );
        }
        if let Some(id) = &notice.successor_instrument_id {
            ensure!(
                all.values().any(|c| &c.instrument.id == id),
                "repeal successor missing"
            );
        }
    }
    stored_validation(data, directory, corpus, accepted)?;
    Ok(())
}

fn stored_validation(
    data: &Snapshot,
    directory: &str,
    corpus: &Corpus,
    accepted: &Value,
) -> Result<()> {
    let stored: Value = read(data, &format!("{directory}/validation.json"))?;
    ensure!(
        stored["instrument_id"] == corpus.instrument.id,
        "validation report owner differs"
    );
    let stored_errors: Vec<_> = stored["issues"]
        .as_array()
        .context("validation issues absent")?
        .iter()
        .filter(|i| i["severity"] == "error")
        .map(|i| {
            format!(
                "{}: {}",
                i["code"].as_str().unwrap_or(""),
                i["message"].as_str().unwrap_or("")
            )
        })
        .collect();
    if !stored_errors.is_empty() {
        let slug = directory.rsplit('/').next().context("slug absent")?;
        let gap = accepted["accepted"]
            .as_array()
            .context("gap list absent")?
            .iter()
            .find(|g| g["slug"] == slug)
            .context("unaccepted stored validation errors")?;
        let mut expected: Vec<_> = gap["errors"]
            .as_array()
            .context("gap errors absent")?
            .iter()
            .map(|v| {
                v.as_str()
                    .context("gap error not string")
                    .map(str::to_owned)
            })
            .collect::<Result<_>>()?;
        let mut actual = stored_errors;
        expected.sort();
        actual.sort();
        ensure!(
            expected == actual,
            "stored errors differ from accepted gap: {slug}"
        );
    }
    Ok(())
}

pub fn validate(data: &Snapshot) -> Result<()> {
    let mut issues = schemas(data)?;
    let all = corpora(data)?;
    let gaps: Value = read(data, "adapters/diputados/_closure-accepted-gaps.json")?;
    for (directory, corpus) in &all {
        ensure!(
            corpus.instrument.schema_version == lex_core::SCHEMA_VERSION,
            "unknown instrument version"
        );
        ensure!(
            corpus
                .provisions
                .iter()
                .all(|p| p.schema_version == lex_core::SCHEMA_VERSION
                    && p.instrument_id == corpus.instrument.id),
            "provision schema/owner mismatch"
        );
        if let Err(error) = validate_one(data, directory, corpus, &all, &gaps) {
            issues.push(format!("{directory}: {error:#}"));
        }
    }
    ensure!(
        issues.is_empty(),
        "pinned input validation failed:\n{}",
        issues.join("\n")
    );
    Ok(())
}
