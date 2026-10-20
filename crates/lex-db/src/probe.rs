use crate::digest;
use anyhow::{Context, Result, ensure};
use rusqlite::{Connection, params};
use serde::Serialize;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};

const GROUPS: &[(&str, &[&str])] = &[
    ("instruments", &["id"]),
    ("provisions", &["id", "instrument_id", "text"]),
    (
        "reference_edges",
        &[
            "id",
            "source_provision_id",
            "target_instrument_id",
            "target_provision_id",
            "resolution_status",
        ],
    ),
    (
        "defined_terms",
        &["id", "instrument_id", "defining_provision_id"],
    ),
    ("term_usages", &["id", "provision_id", "term_id"]),
    ("glossary_coverage", &["instrument_id"]),
];

#[derive(Debug, Serialize)]
pub struct ProbeReport {
    pub sqlite_version: String,
    pub compile_options: Vec<String>,
    pub verified_records: usize,
    pub logical_fixture_sha256: String,
    pub fts5: bool,
    pub json: bool,
    pub strict: bool,
}

pub fn connection() -> Result<Connection> {
    let connection = Connection::open_in_memory()?;
    connection.execute_batch(include_str!("../schema.sql"))?;
    Ok(connection)
}

fn records<'a>(fixture: &'a Value, group: &str) -> Result<&'a Vec<Value>> {
    fixture[group]
        .as_array()
        .with_context(|| format!("fixture missing array {group}"))
}

fn check_span(text: &str, start: usize, end: usize, expected: &str) -> Result<()> {
    ensure!(start < end && end <= text.chars().count(), "invalid span");
    ensure!(
        text.chars()
            .skip(start)
            .take(end - start)
            .collect::<String>()
            == expected,
        "source span mismatch"
    );
    Ok(())
}

fn validate_fixture(fixture: &Value) -> Result<()> {
    ensure!(
        fixture["schema_version"] == crate::contract::VERSION,
        "fixture version unsupported"
    );
    let keys: BTreeSet<_> = fixture
        .as_object()
        .context("fixture must be object")?
        .keys()
        .map(String::as_str)
        .collect();
    let allowed: BTreeSet<_> = GROUPS
        .iter()
        .map(|(name, _)| *name)
        .chain(["schema_version"])
        .collect();
    ensure!(keys == allowed, "unknown/missing fixture group");
    let instruments: Vec<lex_core::Instrument> =
        serde_json::from_value(fixture["instruments"].clone())?;
    let provisions: Vec<lex_core::Provision> =
        serde_json::from_value(fixture["provisions"].clone())?;
    let provisions_by_id: BTreeMap<_, _> = provisions.iter().map(|p| (p.id.as_str(), p)).collect();
    let references: Vec<lex_core::ReferenceEdge> =
        serde_json::from_value(fixture["reference_edges"].clone())?;
    for edge in references {
        let source = provisions_by_id
            .get(edge.source_provision_id.as_str())
            .map(|p| p.text.as_str())
            .or_else(|| {
                instruments
                    .iter()
                    .find(|i| i.id == edge.source_provision_id)
                    .map(|i| i.official_title.as_str())
            })
            .context("source anchor absent")?;
        check_span(source, edge.start_char, edge.end_char, &edge.source_span)?;
        if edge.resolution_status == lex_core::ReferenceResolutionStatus::Resolved {
            let target = provisions_by_id
                .get(edge.target_provision_id.as_str())
                .context("live target absent")?;
            ensure!(
                target.instrument_id == edge.target_instrument_id,
                "live target owner mismatch"
            );
        }
        for q in &edge.qualifiers {
            ensure!(
                q.start_char.is_some() == q.end_char.is_some(),
                "partial qualifier span"
            );
            if let (Some(start), Some(end)) = (q.start_char, q.end_char) {
                check_span(source, start, end, &q.text)?;
            }
        }
        if let Some(a) = &edge.outdated_reference_approval {
            ensure!(a.matches(&edge, source), "approval binding mismatch");
        }
        if let Some(a) = &edge.legal_operator_review {
            ensure!(a.matches(&edge, source), "operator review binding mismatch");
        }
    }
    for (group, _) in GROUPS {
        for row in records(fixture, group)? {
            ensure!(
                row["schema_version"] == lex_core::SCHEMA_VERSION,
                "{group}: unsupported canonical version"
            );
        }
    }
    for term in records(fixture, "defined_terms")? {
        let term: lex_core::DefinedTerm = serde_json::from_value(term.clone())?;
        let p = provisions_by_id
            .get(term.defining_provision_id.as_str())
            .context("definition provision absent")?;
        ensure!(
            p.instrument_id == term.instrument_id,
            "definition owner mismatch"
        );
        ensure!(
            term.start_char < term.end_char && term.end_char <= p.text.chars().count(),
            "definition span invalid"
        );
        ensure!(
            term.inciso.is_none() || term.fraction.is_some(),
            "inciso requires fraction"
        );
    }
    for usage in records(fixture, "term_usages")? {
        let usage: lex_core::TermUsage = serde_json::from_value(usage.clone())?;
        let p = provisions_by_id
            .get(usage.provision_id.as_str())
            .context("usage provision absent")?;
        check_span(&p.text, usage.start_char, usage.end_char, &usage.span)?;
    }
    Ok(())
}

/// Build a transactional M1 fixture only, with raw input and extras-only rows.
/// This deliberately does not publish a full corpus database.
pub fn build_fixture(connection: &mut Connection, bytes: &[u8]) -> Result<()> {
    let fixture: Value = serde_json::from_slice(bytes)?;
    validate_fixture(&fixture)?;
    let tx = connection.transaction()?;
    tx.execute(
        "INSERT INTO input_files VALUES ('fixture.json',?1,?2,'normalized')",
        params![bytes, digest(bytes)],
    )?;
    for (group, columns) in GROUPS {
        for (ordinal, record) in records(&fixture, group)?.iter().enumerate() {
            let mut extras = record.as_object().context("record must be object")?.clone();
            let mut values = Vec::<rusqlite::types::Value>::new();
            for column in *columns {
                let value = extras
                    .remove(*column)
                    .with_context(|| format!("{group}: missing {column}"))?;
                values.push(
                    value
                        .as_str()
                        .with_context(|| format!("{group}.{column}: expected string"))?
                        .to_owned()
                        .into(),
                );
            }
            let mut names = columns.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
            names.extend([
                "ordinal".into(),
                "input_path".into(),
                "source_pointer".into(),
            ]);
            values.extend([
                i64::try_from(ordinal)?.into(),
                "fixture.json".to_owned().into(),
                format!("/{group}/{ordinal}").into(),
            ]);
            if *group == "reference_edges" {
                store_children(&tx, record, &mut extras)?;
            }
            if *group == "provisions" {
                names.push("heading_title".into());
                let heading = ["libro", "title", "chapter", "section", "apartado"]
                    .iter()
                    .filter_map(|key| record["heading_context"][key].as_str())
                    .collect::<Vec<_>>()
                    .join(" / ");
                values.push(heading.into());
            }
            names.push("extras".into());
            values.push(serde_json::to_string(&extras)?.into());
            let sql = format!(
                "INSERT INTO {group} ({}) VALUES ({})",
                names.join(","),
                vec!["?"; values.len()].join(",")
            );
            tx.execute(&sql, rusqlite::params_from_iter(values))
                .with_context(|| format!("{group} {}", record[columns[0]]))?;
            if *group == "instruments" || *group == "provisions" {
                let owner = if *group == "instruments" {
                    record["id"].as_str()
                } else {
                    record["instrument_id"].as_str()
                };
                tx.execute(
                    "INSERT INTO source_anchors VALUES (?1,?2,?3)",
                    params![
                        record["id"].as_str(),
                        if *group == "instruments" {
                            "instrument_title"
                        } else {
                            "provision"
                        },
                        owner
                    ],
                )?;
            }
        }
    }
    tx.execute(
        "INSERT INTO provisions_fts(provisions_fts) VALUES ('rebuild')",
        [],
    )?;
    tx.commit()?;
    Ok(())
}

fn store_children(
    connection: &Connection,
    record: &Value,
    extras: &mut Map<String, Value>,
) -> Result<()> {
    let id = record["id"].as_str().context("edge id absent")?;
    let qualifiers = extras.remove("qualifiers").context("qualifiers absent")?;
    for (ordinal, q) in qualifiers
        .as_array()
        .context("qualifiers must be array")?
        .iter()
        .enumerate()
    {
        connection.execute(
            "INSERT INTO reference_qualifiers VALUES (?1,?2,?3)",
            params![id, i64::try_from(ordinal)?, serde_json::to_string(q)?],
        )?;
    }
    for (key, table) in [
        (
            "outdated_reference_approval",
            "outdated_reference_approvals",
        ),
        ("legal_operator_review", "legal_operator_reviews"),
    ] {
        if let Some(review) = extras.remove(key) {
            ensure!(
                !review.is_null(),
                "optional review must be omitted, not null"
            );
            connection.execute(
                &format!("INSERT INTO {table} VALUES (?1,?2)"),
                params![id, serde_json::to_string(&review)?],
            )?;
        }
    }
    Ok(())
}

/// Independent read-back reconstruction, compared directly with raw input values.
pub fn verify_fixture(connection: &Connection) -> Result<usize> {
    let (bytes, hash): (Vec<u8>, String) = connection.query_row(
        "SELECT bytes,sha256 FROM input_files WHERE path='fixture.json'",
        [],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    ensure!(digest(&bytes) == hash, "raw input digest mismatch");
    let fixture: Value = serde_json::from_slice(&bytes)?;
    let mut count = 0;
    for (group, columns) in GROUPS {
        let mut statement = connection.prepare(&format!("SELECT {},extras,input_path,source_pointer,ordinal FROM {group} ORDER BY input_path,ordinal", columns.join(",")))?;
        let mut rows = statement.query([])?;
        let mut actual = Vec::new();
        while let Some(row) = rows.next()? {
            ensure!(
                row.get::<_, String>(columns.len() + 1)? == "fixture.json",
                "unexpected source input path"
            );
            ensure!(
                row.get::<_, String>(columns.len() + 2)? == format!("/{group}/{}", actual.len()),
                "source JSON pointer mismatch"
            );
            ensure!(
                row.get::<_, i64>(columns.len() + 3)? == i64::try_from(actual.len())?,
                "source file ordinal mismatch"
            );
            let mut object: Map<String, Value> =
                serde_json::from_str(&row.get::<_, String>(columns.len())?)?;
            for (index, key) in columns.iter().enumerate() {
                ensure!(
                    !object.contains_key(*key),
                    "mapped key duplicated in extras"
                );
                object.insert((*key).into(), Value::String(row.get(index)?));
            }
            if *group == "reference_edges" {
                read_children(connection, &mut object)?;
            }
            actual.push(Value::Object(object));
        }
        ensure!(
            &actual == records(&fixture, group)?,
            "logical record parity mismatch: {group}"
        );
        count += actual.len();
    }
    ensure!(
        connection.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))? == "ok",
        "integrity check failed"
    );
    ensure!(
        !connection.prepare("PRAGMA foreign_key_check")?.exists([])?,
        "foreign key check failed"
    );
    Ok(count)
}

fn read_children(connection: &Connection, object: &mut Map<String, Value>) -> Result<()> {
    let id = object["id"].as_str().context("stored edge lacks id")?;
    let mut stmt = connection.prepare(
        "SELECT ordinal,payload FROM reference_qualifiers WHERE edge_id=?1 ORDER BY ordinal",
    )?;
    let stored = stmt
        .query_map([id], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let mut qualifiers = Vec::new();
    for (expected, (ordinal, payload)) in stored.into_iter().enumerate() {
        ensure!(
            ordinal == i64::try_from(expected)?,
            "qualifier ordinals not contiguous"
        );
        qualifiers.push(serde_json::from_str::<Value>(&payload)?);
    }
    let mut reviews = Vec::new();
    for (key, table) in [
        (
            "outdated_reference_approval",
            "outdated_reference_approvals",
        ),
        ("legal_operator_review", "legal_operator_reviews"),
    ] {
        let mut stmt =
            connection.prepare(&format!("SELECT payload FROM {table} WHERE edge_id=?1"))?;
        let mut rows = stmt.query([id])?;
        if let Some(row) = rows.next()? {
            reviews.push((
                key,
                serde_json::from_str::<Value>(&row.get::<_, String>(0)?)?,
            ));
        }
    }
    object.insert("qualifiers".into(), Value::Array(qualifiers));
    for (key, value) in reviews {
        object.insert(key.into(), value);
    }
    Ok(())
}

pub fn run_probe() -> Result<ProbeReport> {
    let mut connection = connection()?;
    let bytes = include_bytes!("../../../fixtures/database/adversarial.json");
    build_fixture(&mut connection, bytes)?;
    let verified_records = verify_fixture(&connection)?;
    let strict_error=connection.execute("INSERT INTO instruments(id,ordinal,input_path,source_pointer,extras) VALUES ('invalid','not-an-integer','fixture.json','/invalid','{}')",[]).err().context("STRICT must reject a text ordinal")?;
    ensure!(
        matches!(strict_error,rusqlite::Error::SqliteFailure(error,_) if error.extended_code==rusqlite::ffi::SQLITE_CONSTRAINT_DATATYPE),
        "STRICT probe failed for a different reason"
    );
    ensure!(
        connection.query_row("SELECT json_type('{\"x\":null}','$.x')", [], |r| r
            .get::<_, String>(0))?
            == "null",
        "JSON not supported"
    );
    let hits: i64 = connection.query_row(
        "SELECT count(*) FROM provisions_fts WHERE provisions_fts MATCH 'obligacion'",
        [],
        |r| r.get(0),
    )?;
    ensure!(hits > 0, "FTS5 accent fixture did not match");
    let compile_options = connection
        .prepare("PRAGMA compile_options")?
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let value: Value = serde_json::from_slice(bytes)?;
    Ok(ProbeReport {
        sqlite_version: rusqlite::version().into(),
        compile_options,
        verified_records,
        logical_fixture_sha256: digest(&serde_json::to_vec(&value)?),
        fts5: true,
        json: true,
        strict: true,
    })
}
