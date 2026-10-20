use lex_db::{
    contract::{DEBT_PATH, validate_seed},
    digest,
    input::{GitInput, inspect},
};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
};

fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}
fn save(root: &Path, path: &str, bytes: &[u8]) {
    let p = root.join(path);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, bytes).unwrap();
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
            "fixture",
        ],
    );
    git(root, &["rev-parse", "HEAD"])
}

fn initialize(root: &Path) {
    let result = Command::new("git")
        .arg("init")
        .arg(root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap();
    assert!(result.success());
}

#[test]
fn pinned_inventory_ignores_dirty_text_and_seed_fails_closed() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    initialize(root);
    let mut f: Value = serde_json::from_slice(include_bytes!(
        "../../../fixtures/database/adversarial.json"
    ))
    .unwrap();
    let heading = "Artículo 9.-";
    let before = f["provisions"][0]["text"].as_str().unwrap().to_owned();
    f["provisions"][0]["text"] = json!(format!("{before}\n\n{heading} Otra disposición."));
    let path = "corpus/mx/db-fixture/provisions.json";
    save(root, path, &serde_json::to_vec(&f["provisions"]).unwrap());
    for (name, value) in [
        ("instrument.json", f["instruments"][0].clone()),
        ("references.json", f["reference_edges"].clone()),
        ("terms.json", f["defined_terms"].clone()),
        ("term-usages.json", f["term_usages"].clone()),
        ("glossary-coverage.json", f["glossary_coverage"][0].clone()),
        ("source-manifest.json", json!({})),
        ("validation.json", json!({})),
    ] {
        save(
            root,
            &format!("corpus/mx/db-fixture/{name}"),
            &serde_json::to_vec(&value).unwrap(),
        );
    }
    save(
        root,
        "docs/decisions.md",
        b"Synthetic fixture ruling, not a legal determination.",
    );
    save(
        root,
        "adapters/diputados/_closure-accepted-gaps.json",
        b"{\"accepted\":[]}",
    );
    save(
        root,
        "corpus/mx/db-fixture/markdown/enactment-evidence.json",
        b"not canonical JSON",
    );
    let base = commit(root);
    assert!(
        inspect(root, &base)
            .unwrap_err()
            .to_string()
            .contains(DEBT_PATH)
    );
    let source = GitInput::new(root, &base).unwrap();
    let row = json!({"instrument_id":f["instruments"][0]["id"],"kind":"merged_heading","printed_label":"9","heading_text":heading,"containing_provision_id":f["provisions"][0]["id"],"start_char":before.chars().count()+2,"end_char":before.chars().count()+2+heading.chars().count(),"source_path":path,"source_blob_oid":source.blob_oid(path).unwrap(),"source_sha256":digest(&source.read(path).unwrap()),"review_status":"accepted","reviewer":"Fixture reviewer","reviewed_at":"2026-10-20","decision_reference":"docs/decisions.md"});
    let seed = json!({"schema_version":"1","source_corpus_commit":base,"decision_source_path":"docs/decisions.md","decision_source_sha256":digest(&source.read("docs/decisions.md").unwrap()),"rows":[row]});
    save(root, DEBT_PATH, &serde_json::to_vec(&seed).unwrap());
    let pinned = commit(root);
    let first = inspect(root, &pinned).unwrap();
    assert_eq!(first.accepted_debt_rows, 1);
    assert!(
        !first
            .files
            .iter()
            .any(|file| file.path.contains("/markdown/"))
    );
    save(root, path, b"corrupted dirty file");
    let second = inspect(root, &pinned).unwrap();
    assert_eq!(
        serde_json::to_value(first).unwrap(),
        serde_json::to_value(second).unwrap()
    );
    let git = GitInput::new(root, &pinned).unwrap();
    for (field, bad) in [
        ("start_char", json!(999_999)),
        ("source_sha256", json!("0".repeat(64))),
        ("review_status", json!("proposed")),
        ("printed_label", json!("2")),
    ] {
        let mut changed = seed.clone();
        changed["rows"][0][field] = bad;
        assert!(
            validate_seed(&git, &serde_json::to_vec(&changed).unwrap()).is_err(),
            "{field}"
        );
    }
    let mut duplicate = seed.clone();
    let row = duplicate["rows"][0].clone();
    duplicate["rows"].as_array_mut().unwrap().push(row);
    assert!(validate_seed(&git, &serde_json::to_vec(&duplicate).unwrap()).is_err());
    let mut empty = seed;
    empty["rows"] = json!([]);
    assert!(
        validate_seed(&git, &serde_json::to_vec(&empty).unwrap())
            .unwrap()
            .rows
            .is_empty()
    );
}
