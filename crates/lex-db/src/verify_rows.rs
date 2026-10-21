//! Independent reconstruction and source-bound invariants. No insertion helpers.
use crate::{contract::DatabaseManifest, digest};
use anyhow::{Context, Result, ensure};
use rusqlite::{Connection, params};
use serde_json::{Map, Value};
use std::collections::BTreeMap;
type Data = BTreeMap<String, Value>;
const GROUPS: &[(&str, &str, &[&str])] = &[
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
// Version-2 stored JSON is compact, recursively lexically sorted, with unique keys.
// Re-encoding detects duplicates before SQLite and serde can disagree about them.
fn canonical_json(text: &str) -> Result<Value> {
    fn sorted(v: Value) -> Value {
        match v {
            Value::Object(m) => Value::Object(
                m.into_iter()
                    .collect::<BTreeMap<_, _>>()
                    .into_iter()
                    .map(|(k, v)| (k, sorted(v)))
                    .collect(),
            ),
            Value::Array(a) => Value::Array(a.into_iter().map(sorted).collect()),
            v => v,
        }
    }
    let v = sorted(serde_json::from_str(text)?);
    ensure!(
        serde_json::to_string(&v)? == text,
        "noncanonical stored JSON (duplicate keys, ordering or encoding)"
    );
    Ok(v)
}
fn input<'a>(data: &'a Data, path: &str) -> Result<&'a Value> {
    data.get(path)
        .with_context(|| format!("required verifier input absent: {path}"))
}
fn entries(value: &Value) -> Vec<&Value> {
    value
        .as_array()
        .map_or_else(|| vec![value], |a| a.iter().collect())
}
fn string<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    v[key]
        .as_str()
        .with_context(|| format!("missing string {key}"))
}
fn number(v: &Value, key: &str) -> Result<usize> {
    Ok(usize::try_from(
        v[key]
            .as_u64()
            .with_context(|| format!("missing number {key}"))?,
    )?)
}
fn tally(c: &Connection, table: &str, n: usize, counts: &mut BTreeMap<String, i64>) -> Result<()> {
    let count = c.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| {
        r.get::<_, i64>(0)
    })?;
    ensure!(count == i64::try_from(n)?, "extra/missing rows: {table}");
    counts.insert(table.into(), count);
    Ok(())
}
fn children(c: &Connection, id: &str, object: &mut Map<String, Value>) -> Result<()> {
    let mut q = c.prepare(
        "SELECT ordinal,payload FROM reference_qualifiers WHERE edge_id=?1 ORDER BY ordinal",
    )?;
    let mut cursor = q.query([id])?;
    let mut array = Vec::new();
    while let Some(row) = cursor.next()? {
        ensure!(
            row.get::<_, i64>(0)? == i64::try_from(array.len())?,
            "qualifier order differs"
        );
        array.push(canonical_json(&row.get::<_, String>(1)?)?);
    }
    ensure!(
        !object.contains_key("qualifiers"),
        "qualifiers duplicated in extras"
    );
    object.insert("qualifiers".into(), Value::Array(array));
    for (table, key) in [
        (
            "outdated_reference_approvals",
            "outdated_reference_approval",
        ),
        ("legal_operator_reviews", "legal_operator_review"),
    ] {
        let mut stmt = c.prepare(&format!("SELECT payload FROM {table} WHERE edge_id=?1"))?;
        let mut cursor = stmt.query([id])?;
        if let Some(row) = cursor.next()? {
            ensure!(!object.contains_key(key), "review duplicated in extras");
            object.insert(key.into(), canonical_json(&row.get::<_, String>(0)?)?);
        }
    }
    Ok(())
}
fn normalized(c: &Connection, data: &Data, counts: &mut BTreeMap<String, i64>) -> Result<usize> {
    let mut total = 0;
    for (filename, table, columns) in GROUPS {
        let mut expected = 0;
        let schema_path = match *filename {
            "references.json" => Some("schemas/reference-edge.schema.json"),
            "terms.json" => Some("schemas/defined-term.schema.json"),
            "term-usages.json" => Some("schemas/term-usage.schema.json"),
            "glossary-coverage.json" => Some("schemas/glossary-coverage.schema.json"),
            "enactment-evidence.json" => Some("schemas/enactment-evidence.schema.json"),
            _ => None,
        };
        let validator = schema_path
            .map(|p| {
                jsonschema::validator_for(input(data, p)?)
                    .map_err(|e| anyhow::anyhow!(e.to_string()))
            })
            .transpose()?;
        for (path, value) in data
            .iter()
            .filter(|(p, _)| p.starts_with("corpus/mx/") && p.ends_with(&format!("/{filename}")))
        {
            let original = entries(value);
            if let Some(validator) = &validator {
                for v in &original {
                    validator
                        .validate(v)
                        .map_err(|e| anyhow::anyhow!("{path}: {e}"))?;
                }
            }
            let mut stmt=c.prepare(&format!("SELECT {},extras,ordinal,source_pointer FROM {table} WHERE input_path=?1 ORDER BY ordinal",columns.join(",")))?;
            let mut cursor = stmt.query([path])?;
            let mut actual = Vec::new();
            while let Some(row) = cursor.next()? {
                let n = actual.len();
                ensure!(
                    row.get::<_, i64>(columns.len() + 1)? == i64::try_from(n)?,
                    "source ordinal differs: {path}"
                );
                let pointer = if value.is_array() {
                    format!("/{n}")
                } else {
                    String::new()
                };
                ensure!(
                    row.get::<_, String>(columns.len() + 2)? == pointer,
                    "source pointer differs: {path}"
                );
                let mut object: Map<String, Value> =
                    canonical_json(&row.get::<_, String>(columns.len())?)?
                        .as_object()
                        .context("extras must be an object")?
                        .clone();
                for (index, key) in columns.iter().enumerate() {
                    ensure!(!object.contains_key(*key), "mapped column duplicated");
                    object.insert((*key).into(), Value::String(row.get(index)?));
                }
                if *table == "reference_edges" {
                    let id = object["id"].as_str().context("edge id absent")?.to_owned();
                    children(c, &id, &mut object)?;
                }
                actual.push(Value::Object(object));
            }
            ensure!(
                actual.iter().eq(original.into_iter()),
                "independent record parity failed: {path}"
            );
            expected += actual.len();
        }
        tally(c, table, expected, counts)?;
        total += expected;
    }
    Ok(total)
}

fn span(text: &str, v: &Value, start: &str, end: &str, exact: Option<&str>) -> Result<()> {
    let a = number(v, start)?;
    let b = number(v, end)?;
    ensure!(a < b && b <= text.chars().count(), "invalid Unicode span");
    if let Some(exact) = exact {
        ensure!(
            text.chars().skip(a).take(b - a).collect::<String>() == exact,
            "source span does not match"
        );
    }
    Ok(())
}
type Anchors<'a> = BTreeMap<&'a str, (&'a str, &'a str)>;
type Records<'a> = BTreeMap<&'a str, &'a Value>;
fn anchors<'a>(
    c: &Connection,
    data: &'a Data,
    counts: &mut BTreeMap<String, i64>,
) -> Result<(Anchors<'a>, Records<'a>)> {
    let mut anchors = BTreeMap::new();
    let mut provisions = BTreeMap::new();
    let mut rowid = 0;
    for (path, value) in data
        .iter()
        .filter(|(p, _)| p.ends_with("/instrument.json") || p.ends_with("/provisions.json"))
    {
        let is_title = path.ends_with("/instrument.json");
        for v in entries(value) {
            ensure!(v["schema_version"] == "0.1.0", "unsupported source version");
            let id = string(v, "id")?;
            let owner = string(v, if is_title { "id" } else { "instrument_id" })?;
            let text = string(v, if is_title { "official_title" } else { "text" })?;
            ensure!(
                anchors.insert(id, (owner, text)).is_none(),
                "duplicate anchor identity"
            );
            let actual: (String, String) = c.query_row(
                "SELECT kind,instrument_id FROM source_anchors WHERE id=?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?;
            ensure!(
                actual
                    == (
                        if is_title {
                            "instrument_title"
                        } else {
                            "provision"
                        }
                        .into(),
                        owner.into()
                    ),
                "source anchor differs"
            );
            if is_title {
                let slug: String =
                    c.query_row("SELECT slug FROM instruments WHERE id=?1", [id], |r| {
                        r.get(0)
                    })?;
                ensure!(
                    Some(slug.as_str()) == path.split('/').nth(2),
                    "instrument slug differs"
                );
            } else {
                rowid += 1;
                provisions.insert(id, v);
                let actual: (i64, String) = c.query_row(
                    "SELECT row_id,heading_title FROM provisions WHERE id=?1",
                    [id],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )?;
                let heading = ["libro", "title", "chapter", "section", "apartado"]
                    .iter()
                    .filter_map(|k| v["heading_context"][k].as_str())
                    .collect::<Vec<_>>()
                    .join(" / ");
                ensure!(actual == (rowid, heading), "FTS row id/heading differs");
            }
        }
    }
    tally(c, "source_anchors", anchors.len(), counts)?;
    Ok((anchors, provisions))
}
fn definitions(data: &Data, provisions: &Records<'_>) -> Result<()> {
    let mut terms = BTreeMap::new();
    for value in data
        .iter()
        .filter(|(p, _)| p.ends_with("/terms.json"))
        .map(|(_, v)| v)
    {
        for v in entries(value) {
            let definition = provisions
                .get(string(v, "defining_provision_id")?)
                .context("definition anchor absent")?;
            ensure!(
                v["instrument_id"] == definition["instrument_id"],
                "definition owner differs"
            );
            span(
                string(definition, "text")?,
                v,
                "start_char",
                "end_char",
                None,
            )?;
            let entry: String = string(definition, "text")?
                .chars()
                .skip(number(v, "start_char")?)
                .take(number(v, "end_char")? - number(v, "start_char")?)
                .collect();
            ensure!(
                entry.contains(string(v, "term")?),
                "definition span does not contain term"
            );
            ensure!(
                terms.insert(string(v, "id")?, v).is_none(),
                "duplicate term identity"
            );
        }
    }
    for value in data
        .iter()
        .filter(|(p, _)| p.ends_with("/term-usages.json"))
        .map(|(_, v)| v)
    {
        for v in entries(value) {
            let p = provisions
                .get(string(v, "provision_id")?)
                .context("usage anchor absent")?;
            ensure!(
                terms.contains_key(string(v, "term_id")?),
                "usage term absent"
            );
            span(
                string(p, "text")?,
                v,
                "start_char",
                "end_char",
                Some(string(v, "span")?),
            )?;
        }
    }
    Ok(())
}
fn edges(
    c: &Connection,
    data: &Data,
    anchors: &Anchors<'_>,
    provisions: &Records<'_>,
    counts: &mut BTreeMap<String, i64>,
) -> Result<()> {
    let mut qualifiers = 0;
    let mut approvals = 0;
    let mut reviews = 0;
    for value in data
        .iter()
        .filter(|(p, _)| p.ends_with("/references.json"))
        .map(|(_, v)| v)
    {
        for v in entries(value) {
            let (_, text) = anchors
                .get(string(v, "source_provision_id")?)
                .context("edge anchor absent")?;
            span(
                text,
                v,
                "start_char",
                "end_char",
                Some(string(v, "source_span")?),
            )?;
            let live: Option<String> = c.query_row(
                "SELECT live_target FROM reference_edges WHERE id=?1",
                [string(v, "id")?],
                |r| r.get(0),
            )?;
            if v["resolution_status"] == "resolved" {
                let target = provisions
                    .get(string(v, "target_provision_id")?)
                    .context("live target absent")?;
                ensure!(
                    target["instrument_id"] == v["target_instrument_id"]
                        && live.as_deref() == v["target_provision_id"].as_str(),
                    "live target owner differs"
                );
            } else {
                ensure!(live.is_none(), "non-live edge has live target");
            }
            for q in v["qualifiers"].as_array().context("qualifiers absent")? {
                qualifiers += 1;
                if !q["start_char"].is_null() || !q["end_char"].is_null() {
                    span(text, q, "start_char", "end_char", Some(string(q, "text")?))?;
                }
            }
            for (key, count) in [
                ("outdated_reference_approval", &mut approvals),
                ("legal_operator_review", &mut reviews),
            ] {
                if let Some(review) = v.get(key) {
                    *count += 1;
                    for field in [
                        "source_provision_id",
                        "target_instrument_id",
                        "target_provision_id",
                        "source_span",
                        "start_char",
                        "end_char",
                    ] {
                        ensure!(review[field] == v[field], "review binding differs");
                    }
                    ensure!(
                        review["source_text_sha256"] == digest(text.as_bytes()),
                        "review source hash differs"
                    );
                }
            }
        }
    }
    tally(c, "reference_qualifiers", qualifiers, counts)?;
    tally(c, "outdated_reference_approvals", approvals, counts)?;
    tally(c, "legal_operator_reviews", reviews, counts)
}

fn graph(c: &Connection, data: &Data, counts: &mut BTreeMap<String, i64>) -> Result<()> {
    let (anchors, provisions) = anchors(c, data, counts)?;
    definitions(data, &provisions)?;
    edges(c, data, &anchors, &provisions, counts)
}

fn source_manifest(c: &Connection, data: &Data, path: &str) -> Result<usize> {
    if !path.starts_with("corpus/mx/")
        || !(path.ends_with("/source-manifest.json")
            || path.ends_with("/formal-source-manifest.json")
            || path.ends_with("/annex-source-manifests.json"))
    {
        return Ok(0);
    }
    let mut count = 0;
    let value = input(data, path)?;
    let directory = path.rsplit_once('/').context("invalid source path")?.0;
    let validator = jsonschema::validator_for(input(data, "schemas/source-manifest.schema.json")?)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    for v in entries(value) {
        validator
            .validate(v)
            .map_err(|e| anyhow::anyhow!("{path}: {e}"))?;
    }
    let owner = &input(data, &format!("{directory}/instrument.json"))?["id"];
    let mut stmt=c.prepare("SELECT ordinal,source_pointer,instrument_id,role,extras FROM source_manifests WHERE input_path=?1 ORDER BY ordinal")?;
    let mut cursor = stmt.query([path])?;
    for (n, v) in entries(value).iter().enumerate() {
        let row = cursor.next()?.context("source manifest row absent")?;
        ensure!(
            row.get::<_, i64>(0)? == i64::try_from(n)?
                && row.get::<_, String>(1)?
                    == if value.is_array() {
                        format!("/{n}")
                    } else {
                        String::new()
                    }
                && Value::String(row.get(2)?) == *owner
                && Some(row.get::<_, String>(3)?.as_str()) == path.rsplit('/').next()
                && canonical_json(&row.get::<_, String>(4)?)? == **v,
            "source manifest parity failed"
        );
        count += 1;
    }
    ensure!(cursor.next()?.is_none(), "extra source manifest row");
    Ok(count)
}

fn supporting(
    c: &Connection,
    data: &Data,
    manifest: &DatabaseManifest,
    counts: &mut BTreeMap<String, i64>,
) -> Result<()> {
    let mut sources = 0;
    let mut standards = 0;
    let mut evidence = 0;
    let mut coverage = 0;
    let gaps = &input(data, "adapters/diputados/_closure-accepted-gaps.json")?["accepted"];
    for file in &manifest.specification.inputs {
        let path = &file.path;
        if serde_json::to_value(&file.handling)? != "normalized" {
            let kind: String = c.query_row(
                "SELECT kind FROM evidence_artifacts WHERE path=?1",
                [path],
                |r| r.get(0),
            )?;
            ensure!(
                Value::String(kind) == serde_json::to_value(&file.handling)?,
                "evidence classification differs"
            );
            evidence += 1;
        }
        sources += source_manifest(c, data, path)?;
        if path.ends_with("/standard.json") {
            let row = input(data, path)?;
            jsonschema::validator_for(input(data, "schemas/standard-metadata.schema.json")?)
                .map_err(|e| anyhow::anyhow!(e.to_string()))?
                .validate(row)
                .map_err(|e| anyhow::anyhow!("{path}: {e}"))?;
            let (p, reason): (String, String) = c.query_row(
                "SELECT metadata_path,exclusion_reason FROM standards_catalog WHERE id=?1",
                [string(row, "id")?],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?;
            ensure!(
                p == *path
                    && reason
                        == "standard body excluded from provision tables and FTS by v1 policy",
                "standard exclusion differs"
            );
            standards += 1;
        }
        if path.ends_with("/instrument.json") {
            let directory = path.rsplit_once('/').context("invalid instrument path")?.0;
            let owner = string(input(data, path)?, "id")?;
            let gap = gaps
                .as_array()
                .context("accepted gaps absent")?
                .iter()
                .any(|g| g["slug"].as_str() == directory.rsplit('/').next());
            for (stage, state, file) in [
                (
                    "acquired",
                    "source_manifest_recorded",
                    "source-manifest.json",
                ),
                ("structured", "canonical_records_present", "provisions.json"),
                (
                    "linked",
                    if gap {
                        "accepted_gap"
                    } else {
                        "stored_graph_validated"
                    },
                    "references.json",
                ),
                ("analyzed", "not_assessed", "provisions.json"),
            ] {
                let actual: (String, String) = c.query_row(
                    "SELECT state,source_path FROM coverage WHERE instrument_id=?1 AND stage=?2",
                    params![owner, stage],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )?;
                ensure!(
                    actual == (state.into(), format!("{directory}/{file}")),
                    "coverage stage differs"
                );
                coverage += 1;
            }
        }
    }
    tally(c, "source_manifests", sources, counts)?;
    tally(c, "standards_catalog", standards, counts)?;
    tally(c, "evidence_artifacts", evidence, counts)?;
    tally(c, "coverage", coverage, counts)?;
    tally(c, "instrument_reproducibility", 0, counts)?;
    tally(c, "corpus_release", 1, counts)?;
    contracts(c, data, counts)
}

fn contracts(c: &Connection, data: &Data, counts: &mut BTreeMap<String, i64>) -> Result<()> {
    for (path, key, table, columns) in [
        (
            "adapters/shared/_known-structural-debt.json",
            "rows",
            "known_structural_debt",
            vec!["instrument_id", "printed_label"],
        ),
        (
            "adapters/diputados/_closure-accepted-gaps.json",
            "accepted",
            "coverage_accepted_gaps",
            vec!["slug"],
        ),
    ] {
        let expected = input(data, path)?[key]
            .as_array()
            .context("contract rows absent")?;
        let mut stmt=c.prepare(&format!("SELECT {},ordinal,source_pointer,extras FROM {table} WHERE input_path=?1 ORDER BY ordinal",columns.join(",")))?;
        let mut cursor = stmt.query([path])?;
        for (n, v) in expected.iter().enumerate() {
            let row = cursor.next()?.context("contract row absent")?;
            let mut object: Value = canonical_json(&row.get::<_, String>(columns.len() + 2)?)?;
            for (i, k) in columns.iter().enumerate() {
                let actual = Value::String(row.get(i)?);
                ensure!(actual == v[*k], "contract mapped identity differs");
                if table == "known_structural_debt" {
                    ensure!(object.get(*k).is_none(), "debt mapped key duplicated");
                    object[*k] = actual;
                }
            }
            ensure!(
                row.get::<_, i64>(columns.len())? == i64::try_from(n)?
                    && row.get::<_, String>(columns.len() + 1)? == format!("/{key}/{n}")
                    && object == *v,
                "source-bound contract row differs"
            );
        }
        ensure!(cursor.next()?.is_none(), "extra contract rows");
        tally(c, table, expected.len(), counts)?;
    }
    Ok(())
}

pub(super) fn check(
    c: &Connection,
    data: &Data,
    manifest: &DatabaseManifest,
) -> Result<(usize, BTreeMap<String, i64>)> {
    let mut counts = BTreeMap::new();
    let n = normalized(c, data, &mut counts)?;
    graph(c, data, &mut counts)?;
    supporting(c, data, manifest, &mut counts)?;
    tally(
        c,
        "input_files",
        manifest.specification.inputs.len(),
        &mut counts,
    )?;
    Ok((n, counts))
}

#[cfg(test)]
mod tests {
    #[test]
    fn canonical_json_rejects_duplicate_keys_at_every_depth() {
        for text in [
            r#"{"label":"forged","label":"source"}"#,
            r#"{"nested":{"x":0,"x":1}}"#,
            r#"[{"x":0,"x":1}]"#,
            r#"{"z":0,"a":1}"#,
            r#"{ "a":1}"#,
        ] {
            assert!(super::canonical_json(text).is_err(), "{text}");
        }
        let text = r#"{"a":[{"x":null,"z":1}],"z":"á"}"#;
        assert!(super::canonical_json(text).is_ok());
    }
}
