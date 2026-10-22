//! Independent M3 verifier. No loader, fixture read-back, handling, or parity helpers.
use crate::{
    contract::{DatabaseManifest, validate_manifest},
    digest,
    input::GitInput,
};
use anyhow::{Context, Result, ensure};
use rusqlite::{Connection, OpenFlags};
use serde::Serialize;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};
#[path = "verify_rows.rs"]
mod rows;

#[derive(Debug, Serialize)]
pub struct VerificationReport {
    pub schema_version: String,
    pub release_id: String,
    pub corpus_commit: String,
    pub head_commit: String,
    pub stale_input_paths: Vec<String>,
    pub input_files: usize,
    pub normalized_records: usize,
    pub table_counts: BTreeMap<String, i64>,
    pub fts_token_occurrences: i64,
    pub database_sha256: String,
    pub logical_content_sha256: String,
}

fn inventory(git: &GitInput) -> Result<BTreeMap<String, String>> {
    let policy: Value =
        serde_json::from_slice(&git.read("adapters/shared/_database-input-policy.json")?)?;
    ensure!(policy["schema_version"] == "1", "unknown input policy");
    let policy_schema: Value =
        serde_json::from_slice(&git.read("schemas/database-input-policy.schema.json")?)?;
    jsonschema::validator_for(&policy_schema)?
        .validate(&policy)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let evidence: BTreeSet<_> = policy["evidence_paths"]
        .as_array()
        .context("evidence paths absent")?
        .iter()
        .map(|p| {
            p.as_str()
                .context("invalid evidence path")
                .map(str::to_owned)
        })
        .collect::<Result<_>>()?;
    let references: BTreeSet<_> = policy["reference_files"]
        .as_array()
        .context("reference paths absent")?
        .iter()
        .map(|p| {
            p.as_str()
                .context("invalid reference path")
                .map(str::to_owned)
        })
        .collect::<Result<_>>()?;
    let paths = git.paths()?;
    let standards: BTreeSet<_> = paths
        .iter()
        .filter(|p| {
            p.starts_with("corpus/mx/")
                && p.split('/').count() == 4
                && p.ends_with("/standard.json")
        })
        .map(|p| p.split('/').nth(2).unwrap_or_default().to_owned())
        .collect();
    let mut selected = BTreeMap::new();
    for path in paths {
        let kind = if path.starts_with("corpus/mx/") {
            if path.split('/').nth(3) == Some("markdown") {
                continue;
            }
            canonical_kind(&path, &standards)?
        } else if path.starts_with("corpus/") {
            ensure!(
                references.contains(&path),
                "undeclared reference file: {path}"
            );
            "evidence_only"
        } else if evidence.contains(&path)
            || ((path.starts_with("adapters/") || path.starts_with("schemas/"))
                && Path::new(&path)
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("json")))
        {
            "evidence_only"
        } else {
            continue;
        };
        selected.insert(path, kind.to_owned());
    }
    required_inputs(&selected, &evidence, &references)?;
    Ok(selected)
}

fn required_inputs(
    selected: &BTreeMap<String, String>,
    evidence: &BTreeSet<String>,
    references: &BTreeSet<String>,
) -> Result<()> {
    for (path, kind) in selected {
        if path.starts_with("corpus/mx/") && kind == "normalized" {
            let directory = path.rsplit_once('/').context("bad canonical path")?.0;
            ensure!(
                selected.contains_key(&format!("{directory}/instrument.json")),
                "orphan canonical input: {path}"
            );
        }
    }
    for path in evidence.iter().chain(references.iter()) {
        ensure!(
            selected.contains_key(path),
            "declared input missing: {path}"
        );
    }
    let directories = selected
        .keys()
        .filter(|p| p.ends_with("/instrument.json"))
        .map(|p| {
            p.rsplit_once('/')
                .map(|(d, _)| d)
                .context("bad instrument path")
        })
        .collect::<Result<Vec<_>>>()?;
    for directory in directories {
        for name in [
            "instrument.json",
            "provisions.json",
            "references.json",
            "terms.json",
            "term-usages.json",
            "glossary-coverage.json",
            "source-manifest.json",
            "validation.json",
        ] {
            ensure!(
                selected.contains_key(&format!("{directory}/{name}")),
                "missing required input: {directory}/{name}"
            );
        }
    }
    for path in [
        "adapters/shared/_known-structural-debt.json",
        "adapters/shared/_database-input-policy.json",
        "adapters/diputados/_closure-accepted-gaps.json",
    ] {
        ensure!(selected.contains_key(path), "required contract absent");
    }
    Ok(())
}

fn canonical_kind(path: &str, standards: &BTreeSet<String>) -> Result<&'static str> {
    let p = path.split('/').collect::<Vec<_>>();
    ensure!(p.len() == 4, "unknown canonical layout: {path}");
    if standards.contains(p[2]) {
        return match p[3] {
            "standard.json" | "validation.json" => Ok("evidence_only"),
            "clauses.json" | "transitories.json" | "supplements.json" | "extracted-text.txt" => {
                Ok("excluded_standard_body")
            }
            _ => anyhow::bail!("unknown standard file: {path}"),
        };
    }
    match p[3] {
        "instrument.json"
        | "provisions.json"
        | "references.json"
        | "terms.json"
        | "term-usages.json"
        | "glossary-coverage.json"
        | "source-manifest.json"
        | "formal-source-manifest.json"
        | "annex-source-manifests.json"
        | "enactment-evidence.json" => Ok("normalized"),
        "validation.json"
        | "reform-temporal-evidence.json"
        | "amendment-references.json"
        | "temporal-analysis-request.json"
        | "temporal-analysis-result.json"
        | "review-queue.json" => Ok("evidence_only"),
        _ => anyhow::bail!("unknown federal file: {path}"),
    }
}

fn raw_inputs(
    c: &Connection,
    git: &GitInput,
    manifest: &DatabaseManifest,
) -> Result<BTreeMap<String, Value>> {
    let selected = inventory(git)?;
    ensure!(
        selected.len() == manifest.specification.inputs.len(),
        "manifest inventory length differs from Git"
    );
    ensure!(
        c.query_row("SELECT count(*) FROM input_files", [], |r| r
            .get::<_, i64>(0))?
            == i64::try_from(selected.len())?,
        "raw input count differs"
    );
    let mut json = BTreeMap::new();
    for descriptor in &manifest.specification.inputs {
        let kind = selected
            .get(&descriptor.path)
            .context("manifest contains unselected path")?;
        ensure!(
            serde_json::to_value(&descriptor.handling)? == *kind,
            "manifest handling differs"
        );
        let bytes = git.read(&descriptor.path)?;
        ensure!(
            digest(&bytes) == descriptor.sha256 && u64::try_from(bytes.len())? == descriptor.bytes,
            "Git input descriptor mismatch: {}",
            descriptor.path
        );
        let (stored, hash, handling): (Vec<u8>, String, String) = c.query_row(
            "SELECT bytes,sha256,handling FROM input_files WHERE path=?1",
            [&descriptor.path],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )?;
        ensure!(
            stored == bytes && hash == descriptor.sha256 && &handling == kind,
            "raw input differs from pinned Git: {}",
            descriptor.path
        );
        if Path::new(&descriptor.path)
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("json"))
        {
            json.insert(descriptor.path.clone(), serde_json::from_slice(&bytes)?);
        }
    }
    Ok(json)
}

type SqlDefinition = (String, String, String, Option<String>);
fn definitions(c: &Connection) -> Result<Vec<SqlDefinition>> {
    Ok(c.prepare("SELECT type,name,tbl_name,sql FROM sqlite_schema WHERE name NOT LIKE 'sqlite_%' AND name NOT IN (SELECT name FROM pragma_table_list WHERE schema='main' AND type='shadow') ORDER BY type,name")?.query_map([],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?)))?.collect::<rusqlite::Result<_>>()?)
}

fn schema(c: &Connection, git: &GitInput, builder: &GitInput) -> Result<()> {
    ensure!(
        builder.read("crates/lex-db/schema.sql")? == include_bytes!("../schema.sql"),
        "unsupported builder DDL"
    );
    for (path, bytes) in include!(concat!(env!("OUT_DIR"), "/builder_inputs.rs")) {
        if path.starts_with("schemas/") {
            ensure!(
                git.read(path)? == *bytes && builder.read(path)? == *bytes,
                "snapshot/builder/verifier schema mismatch: {path}"
            );
        }
    }
    let expected = Connection::open_in_memory()?;
    expected.execute_batch(include_str!("../schema.sql"))?;
    ensure!(
        definitions(c)? == definitions(&expected)?,
        "SQL schema differs from pinned builder"
    );
    Ok(())
}

/// Recompute logical rows independently; `SQLite` identifies shadow/virtual tables.
fn logical(c: &Connection) -> Result<String> {
    use rusqlite::types::ValueRef;
    let names=c.prepare("SELECT name FROM pragma_table_list WHERE schema='main' AND type='table' AND name NOT LIKE 'sqlite_%' AND name<>'corpus_release' ORDER BY name")?.query_map([],|r|r.get::<_,String>(0))?.collect::<rusqlite::Result<Vec<_>>>()?;
    let mut output = BTreeMap::new();
    for name in names {
        let mut stmt = c.prepare(&format!("SELECT * FROM \"{}\"", name.replace('"', "\"\"")))?;
        let width = stmt.column_count();
        let mut cursor = stmt.query([])?;
        let mut records = Vec::new();
        while let Some(row) = cursor.next()? {
            let mut cells = Vec::new();
            for index in 0..width {
                cells.push(match row.get_ref(index)? {
                    ValueRef::Null => serde_json::json!(["null"]),
                    ValueRef::Integer(n) => serde_json::json!(["integer", n]),
                    ValueRef::Real(n) => serde_json::json!(["real", n]),
                    ValueRef::Text(s) => serde_json::json!(["text", std::str::from_utf8(s)?]),
                    ValueRef::Blob(b) => serde_json::json!(["blob", b.len(), digest(b)]),
                });
            }
            records.push(serde_json::to_string(&cells)?);
        }
        records.sort();
        output.insert(name, records);
    }
    Ok(digest(&serde_json::to_vec(&output)?))
}

/// Compare the complete positional index against a fresh index over verified rows.
/// Writes are confined to an owned in-memory database; the release stays read-only.
fn fts(c: &Connection, database: &Path) -> Result<i64> {
    let fresh = Connection::open_in_memory()?;
    fresh.execute_batch("CREATE VIRTUAL TABLE expected USING fts5(label,heading_title,text,tokenize='unicode61 remove_diacritics 2'); CREATE VIRTUAL TABLE tokens USING fts5vocab(expected,'instance');")?;
    let mut rows =
        c.prepare("SELECT row_id,label,heading_title,text FROM provisions ORDER BY row_id")?;
    let mut cursor = rows.query([])?;
    while let Some(row) = cursor.next()? {
        fresh.execute(
            "INSERT INTO expected(rowid,label,heading_title,text) VALUES (?1,?2,?3,?4)",
            rusqlite::params![
                row.get::<_, i64>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?
            ],
        )?;
    }
    fresh.execute(
        "INSERT INTO expected(expected,rank) VALUES('integrity-check',1)",
        [],
    )?;
    let scratch = tempfile::tempdir()?;
    let copy = scratch.path().join("verification.sqlite");
    fs::copy(database, &copy)?;
    let disposable = Connection::open(&copy)?;
    disposable.execute(
        "INSERT INTO provisions_fts(provisions_fts,rank) VALUES('integrity-check',1)",
        [],
    )?;
    disposable.execute_batch("CREATE VIRTUAL TABLE temp.actual_tokens USING fts5vocab(main,'provisions_fts','instance');")?;
    let mut actual = disposable.prepare(
        "SELECT term,doc,col,offset FROM temp.actual_tokens ORDER BY term,doc,col,offset",
    )?;
    let mut expected =
        fresh.prepare("SELECT term,doc,col,offset FROM tokens ORDER BY term,doc,col,offset")?;
    let mut a = actual.query([])?;
    let mut e = expected.query([])?;
    let mut count = 0;
    loop {
        match (a.next()?, e.next()?) {
            (None, None) => break,
            (Some(a), Some(e)) => {
                for i in 0..4 {
                    ensure!(
                        a.get_ref(i)? == e.get_ref(i)?,
                        "FTS positional index differs from rebuild"
                    );
                }
                count += 1;
            }
            _ => anyhow::bail!("FTS token count differs from rebuild"),
        }
    }
    Ok(count)
}

/// Independently verify retirement history without the builder's seed validator.
fn retired_debt(git: &GitInput, data: &BTreeMap<String, Value>) -> Result<()> {
    let seed = data
        .get("adapters/shared/_known-structural-debt.json")
        .context("seed absent")?;
    let schema = data
        .get("schemas/structural-debt.schema.json")
        .context("seed schema absent")?;
    jsonschema::validator_for(schema)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?
        .validate(seed)
        .map_err(|e| anyhow::anyhow!("seed schema: {e}"))?;
    let Some(retirements) = seed.get("retirements").and_then(Value::as_array) else {
        return Ok(());
    };
    let original_commit = retirement_text(seed, "source_corpus_commit")?;
    git.require_ancestor(&original_commit)?;
    let original = git.at(&original_commit)?;
    let review_path = retirement_text(seed, "review_source_path")?;
    let raw_review = original.read(&review_path)?;
    ensure!(
        digest(&raw_review) == retirement_text(seed, "review_source_sha256")?
            && git.read(&review_path)? == raw_review,
        "retirement review authority changed"
    );
    let review: Value = serde_json::from_slice(&raw_review)?;
    let accepted: Vec<_> = review["rows"]
        .as_array()
        .context("review rows absent")?
        .iter()
        .filter(|r| r["review_status"] == "accepted")
        .collect();
    let active = seed["rows"].as_array().context("active rows absent")?;
    ensure!(
        active.len() + retirements.len() == accepted.len(),
        "active/retired partition count differs"
    );
    let mut identities = BTreeSet::new();
    for row in active.iter().chain(retirements.iter().map(|r| &r["row"])) {
        let identity = (
            retirement_text(row, "instrument_id")?,
            retirement_text(row, "printed_label")?,
        );
        ensure!(identities.insert(identity), "active/retired duplicate");
        ensure!(
            accepted
                .iter()
                .filter(|expected| row
                    .as_object()
                    .is_some_and(|m| m.iter().all(|(k, v)| expected.get(k) == Some(v))))
                .count()
                == 1,
            "retirement differs from original accepted row"
        );
    }
    for retirement in retirements {
        retirement_proof(git, &original, &original_commit, retirement)?;
    }

    Ok(())
}

fn retirement_text(v: &Value, key: &str) -> Result<String> {
    Ok(v[key]
        .as_str()
        .with_context(|| format!("retirement lacks {key}"))?
        .to_owned())
}
fn retirement_proof(
    git: &GitInput,
    original: &GitInput,
    original_commit: &str,
    retirement: &Value,
) -> Result<()> {
    let row = &retirement["row"];
    let owner = retirement_text(row, "instrument_id")?;
    let target = format!(
        "{}:article:{}",
        owner,
        lex_parse::labels::canonical_slug(&retirement_text(row, "printed_label")?)
    );
    ensure!(
        target == retirement_text(retirement, "provision_id")?,
        "retired target label differs"
    );
    let path = retirement_text(row, "source_path")?;
    ensure!(
        path == retirement_text(retirement, "source_path")?,
        "retired source path differs"
    );
    let at = retirement_text(retirement, "resolved_at_commit")?;
    git.require_ancestor(&at)?;
    let resolved = git.at(&at)?;
    resolved.require_ancestor(original_commit)?;
    ensure!(at != original_commit, "resolution must follow acceptance");
    let old = original.read(&path)?;
    ensure!(
        digest(&old) == retirement_text(row, "source_sha256")?
            && original.blob_oid(&path)? == retirement_text(row, "source_blob_oid")?,
        "historical retirement locator differs"
    );
    let old: Value = serde_json::from_slice(&old)?;
    let old = old.as_array().context("historical provisions absent")?;
    ensure!(
        !old.iter().any(|p| p["id"] == target),
        "target existed before retirement"
    );
    let container = old
        .iter()
        .find(|p| p["id"] == row["containing_provision_id"])
        .context("historical container absent")?;
    let start = usize::try_from(row["start_char"].as_u64().context("start absent")?)?;
    let end = usize::try_from(row["end_char"].as_u64().context("end absent")?)?;
    let body = retirement_text(container, "text")?;
    ensure!(
        container["instrument_id"] == owner
            && start < end
            && end <= body.chars().count()
            && body
                .chars()
                .skip(start)
                .take(end - start)
                .collect::<String>()
                == retirement_text(row, "heading_text")?,
        "historical retirement span differs"
    );
    let raw = resolved.read(&path)?;
    ensure!(
        digest(&raw) == retirement_text(retirement, "source_sha256")?
            && resolved.blob_oid(&path)? == retirement_text(retirement, "source_blob_oid")?,
        "resolution source differs"
    );
    for raw in [raw, git.read(&path)?] {
        let values: Value = serde_json::from_slice(&raw)?;
        let values = values.as_array().context("resolution provisions absent")?;
        let found: Vec<_> = values.iter().filter(|v| v["id"] == target).collect();
        ensure!(found.len() == 1, "retirement target absent/duplicate");
        ensure!(
            found[0]["instrument_id"] == owner
                && found[0]["provision_type"] == "article"
                && lex_parse::labels::canonical_slug(&retirement_text(found[0], "number")?)
                    == lex_parse::labels::canonical_slug(&retirement_text(row, "printed_label")?)
                && !retirement_text(found[0], "text")?.trim().is_empty(),
            "retirement target differs"
        );
        ensure!(
            !values
                .iter()
                .any(|p| p["id"] == row["containing_provision_id"]
                    && p["text"].as_str().is_some_and(|s| row["heading_text"]
                        .as_str()
                        .is_some_and(|h| s.contains(h)))),
            "retired heading remains embedded"
        );
    }
    Ok(())
}

pub fn verify(
    root: &Path,
    database: &Path,
    manifest_path: &Path,
    expected_commit: &str,
) -> Result<VerificationReport> {
    let value: Value = serde_json::from_slice(&fs::read(manifest_path)?)?;
    let manifest = validate_manifest(&value)?;
    ensure!(
        manifest.specification.corpus_commit == expected_commit,
        "manifest corpus commit differs from requested commit"
    );
    ensure!(
        digest(&fs::read(database)?) == manifest.database_sha256,
        "database checksum differs from sidecar"
    );
    for suffix in ["sqlite-wal", "sqlite-shm", "sqlite-journal"] {
        ensure!(
            !database.with_extension(suffix).exists(),
            "SQLite sidecar present"
        );
    }
    let git = GitInput::new(root, expected_commit)?;
    let builder = GitInput::new(root, &manifest.specification.builder_commit)?;
    ensure!(
        digest(&builder.read("Cargo.lock")?) == manifest.specification.cargo_lock_sha256,
        "builder lock hash mismatch"
    );
    let timestamp: i64 = git
        .command(&["show", "-s", "--format=%ct", expected_commit])?
        .trim()
        .parse()?;
    ensure!(
        timestamp == manifest.specification.source_commit_timestamp,
        "source timestamp mismatch"
    );
    let c = Connection::open_with_flags(database, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    c.execute_batch("PRAGMA query_only=ON; PRAGMA foreign_keys=ON;")?;
    schema(&c, &git, &builder)?;
    ensure!(
        c.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))? == "ok",
        "database integrity failed"
    );
    ensure!(
        !c.prepare("PRAGMA foreign_key_check")?.exists([])?,
        "foreign keys failed"
    );
    let stored: (String, String) =
        c.query_row("SELECT id,specification FROM corpus_release", [], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })?;
    ensure!(
        stored.0 == manifest.release_id
            && serde_json::from_str::<Value>(&stored.1)?
                == serde_json::to_value(&manifest.specification)?,
        "stored release differs from sidecar"
    );
    let data = raw_inputs(&c, &git, &manifest)?;
    retired_debt(&git, &data)?;
    let (normalized_records, table_counts) = rows::check(&c, &data, &manifest)?;
    let logical_content_sha256 = logical(&c)?;
    ensure!(
        logical_content_sha256 == manifest.logical_content_sha256,
        "logical content checksum differs"
    );
    let fts_token_occurrences = fts(&c, database)?;
    ensure!(
        digest(&fs::read(database)?) == manifest.database_sha256,
        "release changed during verification"
    );
    let head = git.command(&["rev-parse", "HEAD"])?.trim().to_owned();
    let paths: BTreeSet<_> = manifest
        .specification
        .inputs
        .iter()
        .map(|f| f.path.as_str())
        .collect();
    let changed = git.command(&["diff", "--name-only", "-z", expected_commit, &head])?;
    let stale_input_paths = changed
        .split('\0')
        .filter(|p| {
            paths.contains(p)
                || (p.starts_with("corpus/") && p.split('/').nth(3) != Some("markdown"))
                || ((p.starts_with("adapters/") || p.starts_with("schemas/"))
                    && Path::new(p)
                        .extension()
                        .is_some_and(|e| e.eq_ignore_ascii_case("json")))
        })
        .map(str::to_owned)
        .collect();
    Ok(VerificationReport {
        schema_version: crate::contract::VERSION.into(),
        release_id: manifest.release_id,
        corpus_commit: expected_commit.into(),
        head_commit: head,
        stale_input_paths,
        input_files: manifest.specification.inputs.len(),
        normalized_records,
        table_counts,
        fts_token_occurrences,
        database_sha256: manifest.database_sha256,
        logical_content_sha256,
    })
}

/// Public read-only handle for fixed acceptance queries.
pub fn open_readonly(path: &Path) -> Result<Connection> {
    let c = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    c.execute_batch("PRAGMA query_only=ON; PRAGMA foreign_keys=ON;")?;
    Ok(c)
}

#[cfg(test)]
mod retirement_tests {
    use super::*;
    use serde_json::json;
    use std::process::Command;
    fn git(root: &Path, args: &[&str]) -> String {
        let out = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap().trim().to_owned()
    }
    fn commit(root: &Path) -> String {
        git(root, &["add", "."]);
        git(
            root,
            &[
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "-c",
                "commit.gpgsign=false",
                "commit",
                "-qm",
                "retirement fixture",
            ],
        );
        git(root, &["rev-parse", "HEAD"])
    }
    #[test]
    fn independent_retirement_check_rejects_forged_history_and_missing_current_target() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        git(root, &["init", "-q"]);
        let path = "corpus/mx/fixture/provisions.json";
        let review_path = "docs/review.json";
        fs::create_dir_all(root.join("corpus/mx/fixture")).unwrap();
        fs::create_dir_all(root.join("docs")).unwrap();
        let owner = "urn:lex-mx:federal:statute:fixture";
        let container = format!("{owner}:article:1");
        let target = format!("{owner}:article:9");
        let heading = "Artículo 9.-";
        let old =
            json!([{"id":container,"instrument_id":owner,"text":format!("{heading} Nueve.")}]);
        let raw = serde_json::to_vec(&old).unwrap();
        fs::write(root.join(path), &raw).unwrap();
        let blob = git(root, &["hash-object", "-w", path]);
        let row = json!({"instrument_id":owner,"kind":"merged_heading","printed_label":"9","heading_text":heading,"containing_provision_id":container,"start_char":0,"end_char":heading.chars().count(),"source_path":path,"source_blob_oid":blob,"source_sha256":digest(&raw),"review_status":"accepted","reviewer":"Fixture","reviewed_at":"2026-10-21","decision_reference":"docs/decisions.md#fixture"});
        let review = serde_json::to_vec(&json!({"rows":[row]})).unwrap();
        fs::write(root.join(review_path), &review).unwrap();
        let original = commit(root);
        let current = json!([{"id":container,"instrument_id":owner,"text":"Uno."},{"id":target,"instrument_id":owner,"provision_type":"article","number":"9","text":"Nueve."}]);
        let raw = serde_json::to_vec(&current).unwrap();
        fs::write(root.join(path), &raw).unwrap();
        let resolved = commit(root);
        let input = GitInput::new(root, &resolved).unwrap();
        let seed = json!({"schema_version":"3","source_corpus_commit":original,"decision_source_path":"docs/decisions.md","decision_source_sha256":"0".repeat(64),"review_source_path":review_path,"review_source_sha256":digest(&review),"decision_sections":[{"reference":"docs/decisions.md#fixture","heading":"## Fixture","sha256":"0".repeat(64)}],"rows":[],"retirements":[{"row":row,"resolved_at_commit":resolved,"provision_id":target,"source_path":path,"source_blob_oid":input.blob_oid(path).unwrap(),"source_sha256":digest(&raw)}]});
        let mut data = BTreeMap::from([
            (
                "adapters/shared/_known-structural-debt.json".into(),
                seed.clone(),
            ),
            (
                "schemas/structural-debt.schema.json".into(),
                serde_json::from_str(include_str!("../../../schemas/structural-debt.schema.json"))
                    .unwrap(),
            ),
        ]);
        assert!(retired_debt(&input, &data).is_ok());
        for field in ["provision_id", "source_sha256", "source_blob_oid"] {
            let mut bad = seed.clone();
            bad["retirements"][0][field] = json!(if field == "source_sha256" {
                "0".repeat(64)
            } else if field == "source_blob_oid" {
                "0".repeat(40)
            } else {
                "urn:absent".into()
            });
            data.insert("adapters/shared/_known-structural-debt.json".into(), bad);
            assert!(retired_debt(&input, &data).is_err(), "{field}");
        }
        let mut bad = seed.clone();
        bad["retirements"][0]["row"]["reviewer"] = json!("invented");
        data.insert("adapters/shared/_known-structural-debt.json".into(), bad);
        assert!(retired_debt(&input, &data).is_err());
        data.insert("adapters/shared/_known-structural-debt.json".into(), seed);
        fs::write(
            root.join(path),
            serde_json::to_vec(&json!([current[0]])).unwrap(),
        )
        .unwrap();
        let removed = commit(root);
        assert!(retired_debt(&GitInput::new(root, &removed).unwrap(), &data).is_err());
    }
}
