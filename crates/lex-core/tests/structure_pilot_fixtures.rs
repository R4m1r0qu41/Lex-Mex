//! G2 acceptance tests for the fintech structural pilot (`pilot-v1`).
//!
//! The tests read the committed G1 fixtures directly; they never copy an
//! expectation. Only the G0 development records and the synthetic texts are
//! loaded: the eight answer-sealed holdout provisions are skipped before any
//! text is read, and the sealed gold file is never opened.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use lex_core::structure_pilot::{ClassCounts, LookupContext, Projection, is_eligible};
use serde_json::{Value, json};

const HOLDOUT: [&str; 8] = [
    "urn:lex-mx:federal:statute:lritf:article:4",
    "urn:lex-mx:federal:statute:lritf:article:52",
    "urn:lex-mx:federal:statute:lritf:article:58",
    "urn:lex-mx:federal:statute:lic:article:16",
    "urn:lex-mx:federal:statute:lic:article:22-bis",
    "urn:lex-mx:federal:statute:lic:article:42",
    "urn:lex-mx:federal:statute:lmv:article:35",
    "urn:lex-mx:federal:statute:lmv:article:64-bis-1",
];

const G1: &str = "fixtures/fintech-structural-g1-2026-11-14";

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn load(path: &str) -> Value {
    let text = std::fs::read_to_string(repo().join(path)).expect(path);
    serde_json::from_str(&text).expect(path)
}

struct Fixtures {
    context: LookupContext,
    rows: Vec<Value>,
}

fn fixtures() -> Fixtures {
    let expected = load(&format!("{G1}/expected-behaviour.json"));
    let release = format!(
        "local-{}",
        expected["baseline_commit"].as_str().expect("baseline")
    );
    let mut context = LookupContext::new(&release);
    let snapshot = load("fixtures/fintech-structural-g0-2026-11-14/canonical-snapshot.json");
    for entry in snapshot["provisions"].as_array().expect("provisions") {
        if entry["split"] != "development" {
            continue;
        }
        let record = &entry["record"];
        let id = record["id"].as_str().expect("id");
        assert!(!HOLDOUT.contains(&id), "holdout provision loaded: {id}");
        let eligible = is_eligible(
            entry["slug"].as_str().expect("slug"),
            record["provision_type"].as_str().expect("type"),
        );
        context.insert(id, record["text"].as_str().expect("text"), eligible);
    }
    let rows = expected["cases"].as_array().expect("cases").clone();
    for row in &rows {
        if let Some(text) = row["synthetic_text"].as_str() {
            let id = row["synthetic_provision_id"]
                .as_str()
                .expect("synthetic id");
            let eligible = row["scope"]
                .as_str()
                .is_some_and(|s| s.starts_with("statute_body"));
            context.insert(id, text, eligible);
            if let Some(current) = row["current_text"].as_str() {
                context.set_current_text(id, current);
            }
        }
    }
    Fixtures { context, rows }
}

fn row_provision(row: &Value) -> &str {
    row["provision_id"]
        .as_str()
        .or_else(|| row["synthetic_provision_id"].as_str())
        .or_else(|| row["request"]["provision_id"].as_str())
        .unwrap_or_default()
}

fn request_for(row: &Value, mode: &str) -> Value {
    if mode == "request" {
        return row["request"].clone();
    }
    let mut request = json!({
        "provision_id": row_provision(row),
        "locator": row["locator"],
    });
    if mode != "complete" {
        request["mode"] = json!(mode);
    }
    request
}

fn path_of(node_id: &str) -> &str {
    node_id.rsplit(':').next().unwrap_or(node_id)
}

fn span_pair(value: &Value) -> Value {
    if value.is_null() {
        return Value::Null;
    }
    json!([value["start_char"], value["end_char"]])
}

/// Compare one lookup response with one expected-behaviour entry.
fn compare(expected: &Value, response: &Value) -> Vec<String> {
    let mut problems = Vec::new();
    let mut check = |what: &str, want: &Value, got: &Value| {
        if want != got {
            problems.push(format!("{what}: expected {want}, got {got}"));
        }
    };
    check("outcome", &expected["outcome"], &response["outcome"]);
    check("reason", &expected["reason"], &response["reason"]);
    check("ambiguity", &expected["ambiguity"], &response["ambiguity"]);
    if expected.get("span").is_some() {
        check("span", &expected["span"], &span_pair(&response["span"]));
    }
    if expected.get("certain_span").is_some() {
        check(
            "certain_span",
            &expected["certain_span"],
            &span_pair(&response["certain_span"]),
        );
    }
    if let Some(segments) = expected.get("unresolved_segments") {
        let got: Vec<Value> = response["unresolved_segments"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .iter()
            .map(|s| {
                let owners: Vec<Value> = s["candidate_owner_node_ids"]
                    .as_array()
                    .cloned()
                    .unwrap_or_default()
                    .iter()
                    .map(|id| json!(path_of(id.as_str().unwrap_or_default())))
                    .collect();
                json!({
                    "start_char": s["start_char"],
                    "end_char": s["end_char"],
                    "reason": s["reason"],
                    "candidate_owner_paths": owners,
                })
            })
            .collect();
        check("unresolved_segments", segments, &Value::Array(got));
    }
    if let Some(candidates) = expected.get("candidates") {
        let candidates = candidates.as_array().cloned().unwrap_or_default();
        let got_list = response["candidates"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let want: Vec<Value> = candidates
            .iter()
            .map(|c| {
                let node = c["node_id"]
                    .as_str()
                    .map(|s| json!(s.strip_prefix("path:").unwrap_or(s)));
                json!({"path": c["path"], "label_span": c["label_span"], "node": node})
            })
            .collect();
        let got: Vec<Value> = got_list
            .iter()
            .map(|c| {
                let node = c["node_id"].as_str().map(|s| json!(path_of(s)));
                json!({"path": c["path"], "label_span": span_pair(&c["label_span"]), "node": node})
            })
            .collect();
        check("candidates", &Value::Array(want), &Value::Array(got));
    }
    problems
}

#[test]
fn rows_match_expected_behaviour() {
    let fixtures = fixtures();
    let mut failures = Vec::new();
    let mut checked = 0;
    assert_eq!(fixtures.rows.len(), 123, "row count");
    for row in &fixtures.rows {
        let case = row["case_id"].as_str().expect("case");
        let expected_branch = row["decision"]["branch"].as_str().expect("branch");
        for (mode, expected) in row["expected"].as_object().expect("expected") {
            let (response, branch) = fixtures.context.lookup_traced(&request_for(row, mode));
            for problem in compare(expected, &response) {
                failures.push(format!("{case} [{mode}] {problem}"));
            }
            checked += 1;
            if branch != expected_branch {
                failures.push(format!(
                    "{case} [{mode}] branch: expected {expected_branch}, got {branch}"
                ));
            }
        }
    }
    assert_eq!(checked, 346, "row requests checked");
    assert!(
        failures.is_empty(),
        "{} problems:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn full_responses_match() {
    let fixtures = fixtures();
    let responses = load(&format!("{G1}/responses.json"));
    let mut failures = Vec::new();
    assert_eq!(responses["responses"].as_array().map(Vec::len), Some(28));
    for entry in responses["responses"].as_array().expect("responses") {
        let case = entry["case_id"].as_str().expect("case");
        let expected = &entry["response"];
        let got = fixtures.context.lookup(&expected["request"]);
        if &got != expected {
            failures.push(format!(
                "{case} [{}]:\n  expected {expected}\n  got      {got}",
                entry["mode_key"]
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn projection_example_matches() {
    let fixtures = fixtures();
    let example = load(&format!("{G1}/projection-example.json"));
    let expected = &example["projection"];
    let id = expected["provision_id"].as_str().expect("id");
    let got = fixtures
        .context
        .projection(id)
        .expect("projection")
        .to_json();
    assert_eq!(&got, expected, "\n  got {got}");
}

#[test]
fn outputs_validate_against_the_pilot_schema() {
    let fixtures = fixtures();
    let schema = load(&format!("{G1}/projection.schema.json"));
    let validator = jsonschema::validator_for(&schema).expect("schema");
    let mut failures = Vec::new();
    let mut provisions = BTreeSet::new();
    for row in &fixtures.rows {
        provisions.insert(row_provision(row).to_owned());
        for mode in row["expected"].as_object().expect("expected").keys() {
            let response = fixtures.context.lookup(&request_for(row, mode));
            if let Err(error) = validator.validate(&response) {
                failures.push(format!("{} [{mode}]: {error}", row["case_id"]));
            }
        }
    }
    for id in &provisions {
        if let Some(projection) = fixtures.context.projection(id)
            && let Err(error) = validator.validate(&projection.to_json())
        {
            failures.push(format!("projection {id}: {error}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn output_is_deterministic() {
    let first = fixtures();
    let second = fixtures();
    for row in &first.rows {
        let id = row_provision(row);
        if let (Some(a), Some(b)) = (first.context.projection(id), second.context.projection(id)) {
            assert_eq!(
                serde_json::to_string(&a.to_json()).expect("json"),
                serde_json::to_string(&b.to_json()).expect("json")
            );
        }
        for mode in row["expected"].as_object().expect("expected").keys() {
            let request = request_for(row, mode);
            assert_eq!(
                serde_json::to_string(&first.context.lookup(&request)).expect("json"),
                serde_json::to_string(&second.context.lookup(&request)).expect("json")
            );
        }
    }
}

/// Second oracle (G1.9): paragraph classes and tails over every eligible
/// non-holdout article equal the committed probe's counters. Aggregates only.
#[test]
fn corpus_counts_match_the_probe() {
    let output = std::fs::read_to_string(repo().join(format!("{G1}/measure-output.txt")))
        .expect("probe output");
    let probe: BTreeMap<&str, usize> = output
        .lines()
        .filter_map(|line| {
            let (key, value) = line.split_once(": ")?;
            Some((key, value.parse().ok()?))
        })
        .collect();
    let mut eligible = 0;
    let mut layout = 0;
    let mut counts = ClassCounts::default();
    let mut tails = 0;
    for slug in ["lritf", "lic", "lmv"] {
        let provisions = load(&format!("corpus/mx/{slug}/provisions.json"));
        for provision in provisions.as_array().expect("provisions") {
            let id = provision["id"].as_str().expect("id");
            if HOLDOUT.contains(&id) || provision["provision_type"] != "article" {
                continue;
            }
            eligible += 1;
            let projection = Projection::build(
                id,
                provision["text"].as_str().expect("text"),
                true,
                "local-test",
            );
            let Some(c) = projection.class_counts() else {
                layout += 1;
                continue;
            };
            counts.fraction += c.fraction;
            counts.inciso += c.inciso;
            counts.note += c.note;
            counts.unsupported += c.unsupported;
            counts.content += c.content;
            counts.same_line_fraction += c.same_line_fraction;
            counts.same_line_inciso += c.same_line_inciso;
            if projection.has_tail() && c.fraction > 0 {
                tails += 1;
            }
        }
    }
    let got = [
        ("eligible_articles", eligible),
        ("unexpected_layout", layout),
        ("class_F", counts.fraction),
        ("class_I", counts.inciso),
        ("class_N", counts.note),
        ("class_U", counts.unsupported),
        ("class_C", counts.content),
        ("same_line_fraction", counts.same_line_fraction),
        ("same_line_inciso", counts.same_line_inciso),
        ("tail", tails),
    ];
    let mismatches: Vec<String> = got
        .iter()
        .filter(|(key, value)| probe.get(key) != Some(value))
        .map(|(key, value)| format!("{key}: probe {:?}, projection {value}", probe.get(key)))
        .collect();
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// Sweep every eligible non-holdout article (aggregates only): each node's
/// own label lookup must resolve to exactly its printed label, or abstain;
/// it may never resolve to other text, and nothing may panic.
#[test]
fn corpus_sweep_label_lookups_are_never_wrong() {
    let mut context = LookupContext::new("local-sweep");
    let mut ids = Vec::new();
    for slug in ["lritf", "lic", "lmv"] {
        let provisions = load(&format!("corpus/mx/{slug}/provisions.json"));
        for provision in provisions.as_array().expect("provisions") {
            let id = provision["id"].as_str().expect("id");
            if HOLDOUT.contains(&id) || provision["provision_type"] != "article" {
                continue;
            }
            context.insert(id, provision["text"].as_str().expect("text"), true);
            ids.push(id.to_owned());
        }
    }
    let mut outcomes: BTreeMap<String, usize> = BTreeMap::new();
    let mut wrong = 0;
    for id in &ids {
        let projection = context.projection(id).expect("projection").to_json();
        for node in projection["nodes"].as_array().expect("nodes") {
            let path = node["path"].as_str().expect("path");
            if path == "root" || path.starts_with("tail/") {
                continue;
            }
            let label = |segment: &str| -> Option<(String, String)> {
                let mut parts = segment.split('.');
                let kind = parts.next()?;
                let value = parts.next()?;
                Some((kind.to_owned(), value.replace('_', " ")))
            };
            let mut locator = serde_json::Map::new();
            for segment in path.split('/') {
                let (kind, value) = label(segment).expect("segment");
                let key = if kind == "f" { "fraction" } else { "inciso" };
                locator.insert(key.into(), json!(value));
            }
            let response = context.lookup(&json!({
                "provision_id": id,
                "locator": locator,
                "mode": "label",
            }));
            let outcome = response["outcome"].as_str().unwrap_or_default().to_owned();
            if outcome == "resolved"
                && (response["node_id"] != node["node_id"]
                    || response["text"] != node["printed_label"])
            {
                wrong += 1;
            }
            *outcomes.entry(outcome).or_default() += 1;
        }
    }
    assert_eq!(wrong, 0, "label lookups resolved to the wrong node or text");
    assert!(
        outcomes.get("resolved").copied().unwrap_or_default() > 2000,
        "{outcomes:?}"
    );
    assert!(
        outcomes
            .keys()
            .all(|o| ["resolved", "ambiguous", "not_structured"].contains(&o.as_str())),
        "{outcomes:?}"
    );
}
