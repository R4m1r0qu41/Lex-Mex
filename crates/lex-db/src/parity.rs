//! Supporting-table checks against the external snapshot at the M2 build gate.
use crate::{contract::DEBT_PATH, input::Snapshot};
use anyhow::{Context, Result, ensure};
use rusqlite::Connection;
use serde_json::{Value, json};

fn json(data: &Snapshot, path: &str) -> Result<Value> {
    Ok(serde_json::from_slice(data.get(path).with_context(
        || format!("parity input absent: {path}"),
    )?)?)
}
fn count(c: &Connection, table: &str, expected: usize) -> Result<()> {
    ensure!(
        c.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r
            .get::<_, i64>(0))?
            == i64::try_from(expected)?,
        "unexpected supporting rows: {table}"
    );
    Ok(())
}

fn source_manifests(c: &Connection, data: &Snapshot) -> Result<()> {
    let mut expected = 0;
    for path in data.keys().filter(|p| {
        p.starts_with("corpus/mx/")
            && (p.ends_with("/source-manifest.json")
                || p.ends_with("/formal-source-manifest.json")
                || p.ends_with("/annex-source-manifests.json"))
    }) {
        let value = json(data, path)?;
        let directory = path.rsplit_once('/').context("invalid path")?.0;
        let owner = json(data, &format!("{directory}/instrument.json"))?;
        let mut stmt=c.prepare("SELECT source_pointer,instrument_id,role,ordinal,extras FROM source_manifests WHERE input_path=?1 ORDER BY ordinal")?;
        let stored = stmt
            .query_map([path], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, i64>(3)?,
                    r.get::<_, String>(4)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let records = crate::load::objects(&value)?;
        ensure!(
            stored.len() == records.len(),
            "manifest count differs: {path}"
        );
        for (n, ((pointer, row), (p, id, role, ordinal, extras))) in
            records.iter().zip(stored).enumerate()
        {
            ensure!(
                pointer == &p
                    && owner["id"] == id
                    && Some(role.as_str()) == path.rsplit('/').next()
                    && ordinal == i64::try_from(n)?
                    && serde_json::from_str::<Value>(&extras)? == **row,
                "manifest parity failed: {path}"
            );
        }
        expected += records.len();
    }
    count(c, "source_manifests", expected)
}

fn debt_and_gaps(c: &Connection, data: &Snapshot) -> Result<()> {
    let seed = json(data, DEBT_PATH)?;
    let rows = seed["rows"].as_array().context("seed rows absent")?;
    for (n, row) in rows.iter().enumerate() {
        let (path,pointer,ordinal,extras):(String,String,i64,String)=c.query_row("SELECT input_path,source_pointer,ordinal,extras FROM known_structural_debt WHERE instrument_id=?1 AND printed_label=?2",rusqlite::params![row["instrument_id"].as_str(),row["printed_label"].as_str()],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?)))?;
        let mut actual: Value = serde_json::from_str(&extras)?;
        actual["instrument_id"] = row["instrument_id"].clone();
        actual["printed_label"] = row["printed_label"].clone();
        ensure!(
            path == DEBT_PATH
                && pointer == format!("/rows/{n}")
                && ordinal == i64::try_from(n)?
                && &actual == row,
            "debt parity failed"
        );
    }
    count(c, "known_structural_debt", rows.len())?;
    let path = "adapters/diputados/_closure-accepted-gaps.json";
    let gaps = json(data, path)?;
    let rows = gaps["accepted"].as_array().context("gap rows absent")?;
    for (n, row) in rows.iter().enumerate() {
        let (input,pointer,ordinal,extras):(String,String,i64,String)=c.query_row("SELECT input_path,source_pointer,ordinal,extras FROM coverage_accepted_gaps WHERE slug=?1",[row["slug"].as_str()],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?)))?;
        ensure!(
            input == path
                && pointer == format!("/accepted/{n}")
                && ordinal == i64::try_from(n)?
                && serde_json::from_str::<Value>(&extras)? == *row,
            "accepted gap parity failed"
        );
    }
    count(c, "coverage_accepted_gaps", rows.len())
}

fn anchors_and_headings(c: &Connection, data: &Snapshot) -> Result<()> {
    let mut expected = 0;
    for path in data
        .keys()
        .filter(|p| p.ends_with("/instrument.json") || p.ends_with("/provisions.json"))
    {
        let value = json(data, path)?;
        for (_, row) in crate::load::objects(&value)? {
            let is_title = path.ends_with("/instrument.json");
            let id = row["id"].as_str().context("anchor id absent")?;
            let (kind, owner): (String, String) = c.query_row(
                "SELECT kind,instrument_id FROM source_anchors WHERE id=?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?;
            ensure!(
                kind == if is_title {
                    "instrument_title"
                } else {
                    "provision"
                } && owner == row[if is_title { "id" } else { "instrument_id" }],
                "anchor owner/kind differs"
            );
            if !is_title {
                let heading: String = c.query_row(
                    "SELECT heading_title FROM provisions WHERE id=?1",
                    [id],
                    |r| r.get(0),
                )?;
                let expected = ["libro", "title", "chapter", "section", "apartado"]
                    .iter()
                    .filter_map(|k| row["heading_context"][k].as_str())
                    .collect::<Vec<_>>()
                    .join(" / ");
                ensure!(heading == expected, "heading search projection differs");
            }
            expected += 1;
        }
    }
    count(c, "source_anchors", expected)
}

fn input_handling(c: &Connection, data: &Snapshot) -> Result<()> {
    let standards = data
        .keys()
        .filter(|p| p.ends_with("/standard.json"))
        .filter_map(|p| p.split('/').nth(2).map(str::to_owned))
        .collect();
    let mut evidence = 0;
    for path in data.keys() {
        let handling = if path.starts_with("corpus/mx/") {
            crate::input::handling(path, &standards)?
        } else {
            crate::contract::Handling::EvidenceOnly
        };
        let expected = serde_json::to_value(&handling)?;
        let actual: String = c.query_row(
            "SELECT handling FROM input_files WHERE path=?1",
            [path],
            |r| r.get(0),
        )?;
        ensure!(expected == actual, "input handling differs: {path}");
        if handling != crate::contract::Handling::Normalized {
            let kind: String = c.query_row(
                "SELECT kind FROM evidence_artifacts WHERE path=?1",
                [path],
                |r| r.get(0),
            )?;
            ensure!(expected == kind, "evidence handling differs");
            evidence += 1;
        }
    }
    count(c, "evidence_artifacts", evidence)
}

pub(crate) fn check(c: &Connection, data: &Snapshot) -> Result<()> {
    input_handling(c, data)?;
    source_manifests(c, data)?;
    debt_and_gaps(c, data)?;
    anchors_and_headings(c, data)?;
    let mut standards = 0;
    let mut instruments = 0;
    let gaps = json(data, "adapters/diputados/_closure-accepted-gaps.json")?;
    for path in data
        .keys()
        .filter(|p| p.ends_with("/standard.json") || p.ends_with("/instrument.json"))
    {
        let row = json(data, path)?;
        let id = row["id"].as_str().context("catalogue id absent")?;
        if path.ends_with("/standard.json") {
            let (metadata, reason): (String, String) = c.query_row(
                "SELECT metadata_path,exclusion_reason FROM standards_catalog WHERE id=?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?;
            ensure!(
                metadata == *path
                    && reason
                        == "standard body excluded from provision tables and FTS by v1 policy",
                "standard catalogue differs"
            );
            standards += 1;
        } else {
            let directory = path.rsplit_once('/').context("invalid path")?.0;
            let mut stmt=c.prepare("SELECT stage,state,source_path FROM coverage WHERE instrument_id=?1 ORDER BY stage")?;
            let actual = stmt
                .query_map([id], |r| {
                    Ok(json!([
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?
                    ]))
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            let gap = gaps["accepted"]
                .as_array()
                .context("gaps absent")?
                .iter()
                .any(|g| g["slug"].as_str() == directory.rsplit('/').next());
            let expected = vec![
                json!([
                    "acquired",
                    "source_manifest_recorded",
                    format!("{directory}/source-manifest.json")
                ]),
                json!([
                    "analyzed",
                    "not_assessed",
                    format!("{directory}/provisions.json")
                ]),
                json!([
                    "linked",
                    if gap {
                        "accepted_gap"
                    } else {
                        "stored_graph_validated"
                    },
                    format!("{directory}/references.json")
                ]),
                json!([
                    "structured",
                    "canonical_records_present",
                    format!("{directory}/provisions.json")
                ]),
            ];
            ensure!(actual == expected, "stage coverage differs");
            instruments += 1;
        }
    }
    count(c, "standards_catalog", standards)?;
    count(c, "coverage", instruments * 4)?;
    count(c, "instrument_reproducibility", 0)?;
    Ok(())
}
