//! Schema checks for the fintech structural pilot's G1 fixtures (pilot-v1, draft 2).
//!
//! Standalone: not a workspace member, no project crate, built offline from the workspace's
//! pinned `jsonschema` 0.33. It checks the schema against its declared meta-schema, then the
//! fixture instances against the schema. It implements no G1 rule; `verify.py` checks the
//! semantic links that JSON Schema cannot express.
//!
//! Run from the repository root:
//! `cargo run --offline --locked --manifest-path fixtures/fintech-structural-g1-2026-11-14/schema-check/Cargo.toml`

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn load(dir: &Path, name: &str) -> Value {
    let path = dir.join(name);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

/// The request a row stands for under one mode key (G1.7). A row with `request` gives it raw.
fn row_request(row: &Value, mode_key: &str) -> Value {
    if let Some(raw) = row.get("request") {
        return raw.clone();
    }
    let provision = row
        .get("provision_id")
        .or_else(|| row.get("synthetic_provision_id"))
        .expect("row has a provision id")
        .clone();
    let mut request = json!({"provision_id": provision, "locator": row["locator"].clone()});
    if mode_key != "complete" {
        request["mode"] = json!(mode_key);
    }
    request
}

fn main() -> ExitCode {
    let dir: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("fixture folder").to_path_buf();
    let schema = load(&dir, "projection.schema.json");
    let mut failures: Vec<String> = Vec::new();

    if let Err(error) = jsonschema::meta::validate(&schema) {
        eprintln!("FAIL: schema does not validate against its meta-schema: {error}");
        return ExitCode::FAILURE;
    }
    let validator = match jsonschema::validator_for(&schema) {
        Ok(v) => v,
        Err(error) => {
            eprintln!("FAIL: schema does not compile: {error}");
            return ExitCode::FAILURE;
        }
    };
    let request_schema = json!({
        "$schema": schema["$schema"].clone(),
        "$defs": schema["$defs"].clone(),
        "$ref": "#/$defs/lookup_request"
    });
    let request_validator = jsonschema::validator_for(&request_schema).expect("request schema compiles");

    // Full responses must validate.
    let responses = load(&dir, "responses.json");
    let responses = responses["responses"].as_array().expect("responses array");
    for entry in responses {
        let id = format!("{}/{}", entry["case_id"].as_str().unwrap_or("?"), entry["mode_key"].as_str().unwrap_or("?"));
        for error in validator.iter_errors(&entry["response"]) {
            failures.push(format!("response {id} rejected: {error} at {}", error.instance_path));
        }
    }

    // Must-fail responses must be rejected.
    let must_fail = load(&dir, "responses-must-fail.json");
    let must_fail = must_fail["cases"].as_array().expect("cases array");
    for entry in must_fail {
        if validator.is_valid(&entry["response"]) {
            failures.push(format!("must-fail accepted: {}", entry["why"].as_str().unwrap_or("?")));
        }
    }

    // The projection example must validate.
    let projection = load(&dir, "projection-example.json");
    for error in validator.iter_errors(&projection["projection"]) {
        failures.push(format!("projection example rejected: {error} at {}", error.instance_path));
    }

    // Every row's request: valid exactly when the row is not an invalid_locator row, except
    // unknown_provision, which is well formed and fails only against the corpus.
    let rows = load(&dir, "expected-behaviour.json");
    let rows = rows["cases"].as_array().expect("cases array");
    let mut requests = 0;
    for row in rows {
        let id = row["case_id"].as_str().unwrap_or("?");
        for (mode_key, result) in row["expected"].as_object().expect("expected object") {
            let request = row_request(row, mode_key);
            let invalid = result["outcome"] == "invalid_locator" && result["reason"] != "unknown_provision";
            let valid = request_validator.is_valid(&request);
            requests += 1;
            if valid == invalid {
                failures.push(format!(
                    "row {id}/{mode_key}: request {} the request schema",
                    if valid { "unexpectedly matches" } else { "unexpectedly fails" }
                ));
            }
        }
    }

    if failures.is_empty() {
        println!(
            "PASS: schema meta-validates (draft 2020-12); {} responses valid; {} must-fail responses rejected; \
             projection example valid; {} row requests match the request schema as expected.",
            responses.len(),
            must_fail.len(),
            requests
        );
        ExitCode::SUCCESS
    } else {
        for failure in &failures {
            eprintln!("FAIL: {failure}");
        }
        ExitCode::FAILURE
    }
}
