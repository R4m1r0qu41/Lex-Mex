//! Deterministic relational projection of already validated pinned inputs.
use crate::{
    contract::{DEBT_PATH, Handling},
    input::{Inspection, Snapshot},
};
use anyhow::{Context, Result, ensure};
use rusqlite::{Connection, params, types::Value as SqlValue};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

pub(crate) const GROUPS: &[(&str, &str, &[&str])] = &[
    ("instrument.json", "instruments", &["id"]),
    (
        "provisions.json",
        "provisions",
        &["id", "instrument_id", "text"],
    ),
    (
        "references.json",
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
        "terms.json",
        "defined_terms",
        &["id", "instrument_id", "defining_provision_id"],
    ),
    (
        "term-usages.json",
        "term_usages",
        &["id", "provision_id", "term_id"],
    ),
    (
        "glossary-coverage.json",
        "glossary_coverage",
        &["instrument_id"],
    ),
    (
        "enactment-evidence.json",
        "enactment_evidence",
        &["id", "instrument_id"],
    ),
];

pub(crate) fn objects(value: &Value) -> Result<Vec<(String, &Value)>> {
    if let Some(array) = value.as_array() {
        Ok(array
            .iter()
            .enumerate()
            .map(|(n, v)| (format!("/{n}"), v))
            .collect())
    } else {
        ensure!(value.is_object(), "input must be an object or array");
        Ok(vec![(String::new(), value)])
    }
}

pub(crate) fn heading(record: &Value) -> String {
    ["libro", "title", "chapter", "section", "apartado"]
        .iter()
        .filter_map(|key| record["heading_context"][key].as_str())
        .collect::<Vec<_>>()
        .join(" / ")
}

fn insert_record(
    c: &Connection,
    table: &str,
    columns: &[&str],
    path: &str,
    ordinal: usize,
    pointer: &str,
    record: &Value,
) -> Result<()> {
    let mut extras = record.as_object().context("record must be object")?.clone();
    let mut names = columns.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    let mut values = Vec::<SqlValue>::new();
    for column in columns {
        let value = extras
            .remove(*column)
            .with_context(|| format!("missing {column}"))?;
        values.push(
            value
                .as_str()
                .with_context(|| format!("{column} must be string"))?
                .to_owned()
                .into(),
        );
    }
    names.extend([
        "input_path".into(),
        "ordinal".into(),
        "source_pointer".into(),
    ]);
    values.extend([
        path.to_owned().into(),
        i64::try_from(ordinal)?.into(),
        pointer.to_owned().into(),
    ]);
    if table == "reference_edges" {
        crate::probe::store_children(c, record, &mut extras)?;
    }
    if table == "provisions" {
        names.push("heading_title".into());
        values.push(heading(record).into());
    }
    names.push("extras".into());
    values.push(serde_json::to_string(&extras)?.into());
    let sql = format!(
        "INSERT INTO {table} ({}) VALUES ({})",
        names.join(","),
        vec!["?"; values.len()].join(",")
    );
    c.prepare_cached(&sql)?
        .execute(rusqlite::params_from_iter(values))?;
    if matches!(table, "instruments" | "provisions") {
        c.prepare_cached("INSERT INTO source_anchors VALUES (?1,?2,?3)")?
            .execute(params![
                record["id"].as_str(),
                if table == "instruments" {
                    "instrument_title"
                } else {
                    "provision"
                },
                if table == "instruments" {
                    record["id"].as_str()
                } else {
                    record["instrument_id"].as_str()
                }
            ])?;
    }
    Ok(())
}

fn source_manifests(
    c: &Connection,
    data: &Snapshot,
    owners: &BTreeMap<String, String>,
) -> Result<()> {
    for (path, bytes) in data.iter().filter(|(p, _)| {
        p.starts_with("corpus/mx/")
            && (p.ends_with("/source-manifest.json")
                || p.ends_with("/formal-source-manifest.json")
                || p.ends_with("/annex-source-manifests.json"))
    }) {
        let directory = path.rsplit_once('/').context("invalid path")?.0;
        let owner = owners.get(directory).context("manifest owner absent")?;
        let value: Value = serde_json::from_slice(bytes)?;
        for (n, (pointer, record)) in objects(&value)?.into_iter().enumerate() {
            c.prepare_cached("INSERT INTO source_manifests VALUES (?1,?2,?3,?4,?5,?6)")?
                .execute(params![
                    path,
                    pointer,
                    owner,
                    path.rsplit('/').next(),
                    i64::try_from(n)?,
                    serde_json::to_string(record)?
                ])?;
        }
    }
    Ok(())
}

fn supporting_records(
    c: &Connection,
    data: &Snapshot,
    owners: &BTreeMap<String, String>,
) -> Result<()> {
    source_manifests(c, data, owners)?;
    let gaps_path = "adapters/diputados/_closure-accepted-gaps.json";
    let gaps: Value = serde_json::from_slice(data.get(gaps_path).context("gap input absent")?)?;
    for (n, gap) in gaps["accepted"]
        .as_array()
        .context("accepted gaps absent")?
        .iter()
        .enumerate()
    {
        c.prepare_cached("INSERT INTO coverage_accepted_gaps VALUES (?1,?2,?3,?4,?5)")?
            .execute(params![
                gap["slug"].as_str(),
                gaps_path,
                format!("/accepted/{n}"),
                i64::try_from(n)?,
                serde_json::to_string(gap)?
            ])?;
    }
    let seed: Value = serde_json::from_slice(data.get(DEBT_PATH).context("debt input absent")?)?;
    for (n, row) in seed["rows"]
        .as_array()
        .context("debt rows absent")?
        .iter()
        .enumerate()
    {
        let mut extras = row.as_object().context("debt object absent")?.clone();
        extras.remove("instrument_id");
        extras.remove("printed_label");
        c.prepare_cached("INSERT INTO known_structural_debt VALUES (?1,?2,?3,?4,?5,?6)")?
            .execute(params![
                row["instrument_id"].as_str(),
                row["printed_label"].as_str(),
                DEBT_PATH,
                format!("/rows/{n}"),
                i64::try_from(n)?,
                serde_json::to_string(&extras)?
            ])?;
    }
    for (path, bytes) in data.iter().filter(|(p, _)| p.ends_with("/standard.json")) {
        let value: Value = serde_json::from_slice(bytes)?;
        c.prepare_cached("INSERT INTO standards_catalog VALUES (?1,?2,?3)")?
            .execute(params![
                value["id"].as_str().context("standard id absent")?,
                path,
                "standard body excluded from provision tables and FTS by v1 policy"
            ])?;
    }
    for (directory, id) in owners {
        for (stage, state, file) in [
            (
                "acquired",
                "source_manifest_recorded",
                "source-manifest.json",
            ),
            ("structured", "canonical_records_present", "provisions.json"),
            (
                "linked",
                if gaps["accepted"]
                    .as_array()
                    .context("gaps absent")?
                    .iter()
                    .any(|g| g["slug"].as_str() == directory.rsplit('/').next())
                {
                    "accepted_gap"
                } else {
                    "stored_graph_validated"
                },
                "references.json",
            ),
            ("analyzed", "not_assessed", "provisions.json"),
        ] {
            // No legal-analysis completeness is inferred from pipeline success.
            c.prepare_cached("INSERT INTO coverage VALUES (?1,?2,?3,?4)")?
                .execute(params![id, stage, state, format!("{directory}/{file}")])?;
        }
    }
    Ok(())
}

/// All insertions run in one transaction. Failures leave no committed partial projection.
pub fn populate(
    c: &mut Connection,
    inspection: &Inspection,
    data: &Snapshot,
    release: &str,
    specification: &str,
) -> Result<()> {
    let tx = c.transaction()?;
    let mut owners = BTreeMap::new();
    for file in &inspection.files {
        let bytes = data.get(&file.path).context("snapshot input absent")?;
        tx.prepare_cached("INSERT INTO input_files VALUES (?1,?2,?3,?4)")?
            .execute(params![
                file.path,
                bytes,
                file.sha256,
                serde_json::to_value(&file.handling)?.as_str()
            ])?;
        if file.handling != Handling::Normalized {
            tx.prepare_cached("INSERT INTO evidence_artifacts VALUES (?1,?2)")?
                .execute(params![
                    file.path,
                    serde_json::to_value(&file.handling)?.as_str()
                ])?;
        }
        if file.path.ends_with("/instrument.json") {
            let value: Value = serde_json::from_slice(bytes)?;
            owners.insert(
                file.path
                    .rsplit_once('/')
                    .context("invalid path")?
                    .0
                    .to_owned(),
                value["id"]
                    .as_str()
                    .context("instrument id absent")?
                    .to_owned(),
            );
        }
    }
    for (filename, table, columns) in GROUPS {
        for file in inspection.files.iter().filter(|f| {
            f.handling == Handling::Normalized && f.path.ends_with(&format!("/{filename}"))
        }) {
            let value: Value = serde_json::from_slice(&data[&file.path])?;
            for (n, (pointer, row)) in objects(&value)?.into_iter().enumerate() {
                insert_record(&tx, table, columns, &file.path, n, &pointer, row)
                    .with_context(|| format!("{} {pointer}", file.path))?;
            }
        }
    }
    supporting_records(&tx, data, &owners)?;
    tx.execute(
        "INSERT INTO corpus_release VALUES (?1,?2)",
        params![release, specification],
    )?;
    tx.execute(
        "INSERT INTO provisions_fts(provisions_fts) VALUES ('rebuild')",
        [],
    )?;
    tx.execute(
        "INSERT INTO provisions_fts(provisions_fts,rank) VALUES ('integrity-check',1)",
        [],
    )?;
    ensure!(
        !tx.prepare("PRAGMA foreign_key_check")?.exists([])?,
        "foreign key check failed"
    );
    tx.commit()?;
    Ok(())
}

/// Independent row reconstruction. Uses stored columns, never the insertion map.
pub(crate) fn reconstructed(
    c: &Connection,
    table: &str,
    columns: &[&str],
    path: &str,
) -> Result<Vec<Value>> {
    let mut stmt = c.prepare(&format!(
        "SELECT {},extras,ordinal,source_pointer FROM {table} WHERE input_path=?1 ORDER BY ordinal",
        columns.join(",")
    ))?;
    let mut rows = stmt.query([path])?;
    let mut result = Vec::new();
    while let Some(row) = rows.next()? {
        let n = result.len();
        ensure!(
            row.get::<_, i64>(columns.len() + 1)? == i64::try_from(n)?,
            "noncontiguous source ordinal: {path}"
        );
        let mut extras: Map<String, Value> =
            serde_json::from_str(&row.get::<_, String>(columns.len())?)?;
        for (i, column) in columns.iter().enumerate() {
            ensure!(
                !extras.contains_key(*column),
                "mapped column duplicated in extras"
            );
            extras.insert((*column).into(), Value::String(row.get(i)?));
        }
        if table == "reference_edges" {
            crate::probe::read_children(c, &mut extras)?;
        }
        result.push(Value::Object(extras));
    }
    Ok(result)
}
