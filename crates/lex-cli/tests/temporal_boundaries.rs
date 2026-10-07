//! End-to-end checks that the temporal trust boundaries reject bad input
//! through the real `lex-mex` binary and leave the corpus untouched.
//!
//! Each case works on a throwaway copy of the committed LRITF adapter and
//! corpus, so a rejection is proved against real data and no repository file
//! is ever written.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{Value, json};

const SLUG: &str = "lritf";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

/// A scratch root holding the LRITF adapter and corpus.
fn scratch_root() -> tempfile::TempDir {
    let temporary = tempfile::tempdir().unwrap();
    copy_dir(
        &repo_root().join("adapters"),
        &temporary.path().join("adapters"),
    );
    copy_dir(
        &repo_root().join("corpus/mx").join(SLUG),
        &temporary.path().join("corpus/mx").join(SLUG),
    );
    temporary
}

fn corpus_dir(root: &Path) -> PathBuf {
    root.join("corpus/mx").join(SLUG)
}

/// Every corpus file's bytes, keyed by relative path.
fn snapshot(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(dir: &Path, base: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(&path, base, out);
            } else {
                out.insert(
                    path.strip_prefix(base).unwrap().display().to_string(),
                    fs::read(&path).unwrap(),
                );
            }
        }
    }
    let mut out = BTreeMap::new();
    let corpus = corpus_dir(root);
    walk(&corpus, &corpus, &mut out);
    out
}

fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_lex-mex"))
        .arg("--root")
        .arg(root)
        .args(args)
        .output()
        .unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn none_boundary() -> Value {
    json!({ "boundary_type": "none", "date": null, "description": null })
}

/// A response that satisfies the v2 schema for one provision.
fn schema_valid_batch(provision_id: &str, supporting_text: &str) -> Value {
    json!({
        "determinations": [{
            "provision_id": provision_id,
            "temporal_status": "effective",
            "effective_from": null,
            "effective_to": null,
            "confidence": 0.5,
            "supporting_text": [supporting_text],
            "effects": [{
                "effect_type": "other",
                "affected_scope": "alcance",
                "application_rule": "unknown",
                "trigger": none_boundary(),
                "end_condition": none_boundary(),
                "responsible_authorities": [],
                "verification_status": "not_required"
            }]
        }]
    })
}

/// The first evidence provision's id and a verbatim snippet of its text.
fn first_evidence(root: &Path) -> (String, String) {
    let request: Value = serde_json::from_slice(
        &fs::read(corpus_dir(root).join("temporal-analysis-request.json")).unwrap(),
    )
    .unwrap();
    let first = &request["relevant_provisions"][0];
    let text = first["text"].as_str().unwrap();
    (
        first["provision_id"].as_str().unwrap().to_owned(),
        text.chars().take(20).collect(),
    )
}

fn write_response(root: &Path, value: &Value) -> PathBuf {
    let path = root.join("response.json");
    fs::write(&path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
    path
}

fn import(root: &Path, response: &Path) -> Output {
    run(
        root,
        &[
            "import-temporal",
            SLUG,
            response.to_str().unwrap(),
            "--model",
            "test-model",
        ],
    )
}

#[test]
fn import_rejects_reviewer_only_fields_and_leaves_the_corpus_untouched() {
    let scratch = scratch_root();
    let root = scratch.path();
    let (id, snippet) = first_evidence(root);
    let before = snapshot(root);

    let mut forged = schema_valid_batch(&id, &snippet);
    forged["determinations"][0]["effects"][0]["verification_status"] = json!("externally_verified");
    let output = import(root, &write_response(root, &forged));
    assert!(!output.status.success());
    assert!(stderr(&output).contains("v2 schema"), "{}", stderr(&output));

    let mut reviewer_field = schema_valid_batch(&id, &snippet);
    reviewer_field["determinations"][0]["effects"][0]["verification_source_url"] =
        json!("https://example.invalid");
    let output = import(root, &write_response(root, &reviewer_field));
    assert!(!output.status.success());
    assert!(stderr(&output).contains("v2 schema"), "{}", stderr(&output));

    assert_eq!(before, snapshot(root));
}

#[test]
fn import_rejects_a_request_stale_against_the_current_corpus() {
    let scratch = scratch_root();
    let root = scratch.path();
    let (id, snippet) = first_evidence(root);
    let response = write_response(root, &schema_valid_batch(&id, &snippet));

    // Control: the same response clears the schema and freshness gates on the
    // untouched corpus and fails later, at routing, for an unrelated reason.
    // That proves the rejection below is caused by staleness alone.
    let control = import(root, &response);
    assert!(!control.status.success());
    assert!(
        stderr(&control).contains("omitted determination"),
        "{}",
        stderr(&control)
    );

    let provisions_path = corpus_dir(root).join("provisions.json");
    let mut provisions: Value =
        serde_json::from_slice(&fs::read(&provisions_path).unwrap()).unwrap();
    let provision = provisions
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|provision| provision["id"] == id.as_str())
        .expect("evidence provision exists in the corpus");
    let text = provision["text"].as_str().unwrap().to_owned();
    provision["text"] = json!(format!("{text} Texto agregado después de la solicitud."));
    fs::write(
        &provisions_path,
        serde_json::to_vec_pretty(&provisions).unwrap(),
    )
    .unwrap();
    let before = snapshot(root);

    let output = import(root, &response);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("stale"), "{}", stderr(&output));
    assert_eq!(before, snapshot(root));
}

#[test]
fn resolving_an_archived_review_is_rejected_without_mutation() {
    let scratch = scratch_root();
    let root = scratch.path();
    let queue_path = corpus_dir(root).join("review-queue.json");
    let mut queue: Value = serde_json::from_slice(&fs::read(&queue_path).unwrap()).unwrap();
    let mut archived = queue[0].clone();
    let archived_id = format!("{}:evidence:aaaa", archived["id"].as_str().unwrap());
    archived["id"] = json!(archived_id);
    archived["status"] = json!("pending");
    archived["resolution"] = Value::Null;
    archived["resolved_by"] = Value::Null;
    archived["resolved_at"] = Value::Null;
    queue.as_array_mut().unwrap().push(archived);
    fs::write(&queue_path, serde_json::to_vec_pretty(&queue).unwrap()).unwrap();
    let before = snapshot(root);

    let output = run(
        root,
        &[
            "review",
            "--instrument",
            SLUG,
            "resolve",
            &archived_id,
            "--resolution",
            "accept-machine-conclusion",
            "--reviewer",
            "Test reviewer",
        ],
    );
    assert!(!output.status.success());
    let message = stderr(&output);
    assert!(
        message.contains("archived") || message.contains("stale"),
        "{message}"
    );
    assert_eq!(before, snapshot(root));
}
