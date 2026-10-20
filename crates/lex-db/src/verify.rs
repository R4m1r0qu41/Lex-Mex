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
