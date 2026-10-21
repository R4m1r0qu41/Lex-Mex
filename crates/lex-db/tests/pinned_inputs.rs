use lex_db::{
    contract::{DEBT_PATH, decision_section, validate_seed},
    digest,
    input::{GitInput, POLICY_PATH, inspect},
};
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

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
const DECISIONS: &str = "## Fixture\n\nSynthetic fixture ruling, not a legal determination.\n\n";
const REVIEW: &str = "docs/fixture-review.json";
const SOURCE: &str = "corpus/mx/db-fixture/provisions.json";

struct Fixture {
    temp: tempfile::TempDir,
    seed: Value,
    pinned: String,
    review: Value,
}
impl Fixture {
    fn root(&self) -> &Path {
        self.temp.path()
    }
    fn input(&self) -> GitInput {
        GitInput::new(self.root(), &self.pinned).unwrap()
    }
    fn check(&self, seed: &Value) -> bool {
        validate_seed(&self.input(), &serde_json::to_vec(seed).unwrap()).is_ok()
    }
}

fn write_sources(root: &Path) -> Value {
    let mut f: Value = serde_json::from_slice(include_bytes!(
        "../../../fixtures/database/adversarial.json"
    ))
    .unwrap();
    let before = f["provisions"][0]["text"].as_str().unwrap();
    f["provisions"][0]["text"] = json!(format!(
        "{before}\n\nArtículo 9.- Nueve.\n\nArtículo 10.- Diez.\n\nArtículo 11.- Once."
    ));
    save(root, SOURCE, &serde_json::to_vec(&f["provisions"]).unwrap());
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
    save(root, "docs/decisions.md", DECISIONS.as_bytes());
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
    let policy = json!({"schema_version":"1","evidence_paths":["docs/decisions.md",REVIEW],"reference_files":[]});
    save(root, POLICY_PATH, &serde_json::to_vec(&policy).unwrap());
    f
}

fn row(source: &GitInput, f: &Value, label: u32, status: &str) -> Value {
    let text = f["provisions"][0]["text"].as_str().unwrap();
    let heading = format!("Artículo {label}.-");
    let start = text[..text.find(&heading).unwrap()].chars().count();
    json!({"instrument_id":f["instruments"][0]["id"],"kind":"merged_heading","printed_label":label.to_string(),"heading_text":heading,"containing_provision_id":f["provisions"][0]["id"],"start_char":start,"end_char":start+heading.chars().count(),"source_path":SOURCE,"source_blob_oid":source.blob_oid(SOURCE).unwrap(),"source_sha256":digest(&source.read(SOURCE).unwrap()),"review_status":status,"reviewer":"Fixture reviewer","reviewed_at":"2026-10-20","decision_reference":"docs/decisions.md#fixture"})
}

fn setup() -> Fixture {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    git(root, &["init", "-q"]);
    let f = write_sources(root);
    let base = commit(root);
    assert!(
        inspect(root, &base)
            .unwrap_err()
            .to_string()
            .contains(DEBT_PATH)
    );
    let source = GitInput::new(root, &base).unwrap();
    let accepted = row(&source, &f, 9, "accepted");
    let review = json!({"corpus_commit":base,"decision_reference":"docs/decisions.md#fixture","rows":[accepted,row(&source,&f,10,"deferred"),row(&source,&f,11,"rejected")]});
    let review_bytes = serde_json::to_vec(&review).unwrap();
    save(root, REVIEW, &review_bytes);
    let reviewed = commit(root);
    let seed = json!({"schema_version":"2","source_corpus_commit":reviewed,"decision_source_path":"docs/decisions.md","decision_source_sha256":digest(DECISIONS.as_bytes()),"review_source_path":REVIEW,"review_source_sha256":digest(&review_bytes),"decision_sections":[{"reference":"docs/decisions.md#fixture","heading":"## Fixture","sha256":digest(decision_section(DECISIONS,"## Fixture").unwrap().as_bytes())}],"rows":[review["rows"][0]]});
    save(root, DEBT_PATH, &serde_json::to_vec(&seed).unwrap());
    let pinned = commit(root);
    Fixture {
        temp,
        seed,
        pinned,
        review,
    }
}

#[test]
fn pinned_inventory_ignores_dirty_text_and_checks_bindings() {
    let f = setup();
    let first = inspect(f.root(), &f.pinned).unwrap();
    assert_eq!(first.accepted_debt_rows, 1);
    assert!(
        !first
            .files
            .iter()
            .any(|file| file.path.contains("/markdown/"))
    );
    save(f.root(), SOURCE, b"corrupted dirty file");
    let second = inspect(f.root(), &f.pinned).unwrap();
    assert_eq!(
        serde_json::to_value(first).unwrap(),
        serde_json::to_value(second).unwrap()
    );
    for (field, bad) in [
        ("start_char", json!(999_999)),
        ("source_sha256", json!("0".repeat(64))),
        ("review_status", json!("proposed")),
        ("reviewer", json!("invented")),
        ("printed_label", json!("2")),
    ] {
        let mut changed = f.seed.clone();
        changed["rows"][0][field] = bad;
        assert!(!f.check(&changed), "{field}");
    }
}

#[test]
fn dropped_promoted_rejected_and_duplicate_debt_rows_fail() {
    let f = setup();
    assert!(f.check(&f.seed));
    let mut dropped = f.seed.clone();
    dropped["rows"] = json!([]);
    assert!(!f.check(&dropped));
    for index in [1, 2] {
        let mut promoted = f.seed.clone();
        let mut row = f.review["rows"][index].clone();
        row["review_status"] = json!("accepted");
        promoted["rows"].as_array_mut().unwrap().push(row);
        assert!(!f.check(&promoted));
        let mut swapped = f.seed.clone();
        swapped["rows"][0] = promoted["rows"][1].clone();
        assert!(!f.check(&swapped));
    }
    let mut duplicate = f.seed.clone();
    duplicate["rows"]
        .as_array_mut()
        .unwrap()
        .push(f.seed["rows"][0].clone());
    assert!(!f.check(&duplicate));
}

#[test]
fn only_a_review_with_zero_acceptances_allows_an_empty_seed() {
    let mut f = setup();
    let mut review = f.review.clone();
    review["rows"][0]["review_status"] = json!("rejected");
    let bytes = serde_json::to_vec(&review).unwrap();
    save(f.root(), REVIEW, &bytes);
    f.pinned = commit(f.root());
    assert!(!f.check(&f.seed)); // A later changed ledger cannot silently override the pinned review.
    let mut empty = f.seed.clone();
    empty["rows"] = json!([]);
    empty["source_corpus_commit"] = json!(f.pinned);
    empty["review_source_sha256"] = json!(digest(&bytes));
    assert!(f.check(&empty));
}

#[test]
fn unrelated_decisions_do_not_stale_seed_but_a_changed_ruling_does() {
    let mut f = setup();
    save(
        f.root(),
        "docs/decisions.md",
        format!("## Unrelated\n\nOther work.\n\n{DECISIONS}## Later\n\nAnother entry.\n")
            .as_bytes(),
    );
    f.pinned = commit(f.root());
    assert!(f.check(&f.seed));
    save(
        f.root(),
        "docs/decisions.md",
        DECISIONS.replace("Synthetic", "Changed").as_bytes(),
    );
    f.pinned = commit(f.root());
    assert!(!f.check(&f.seed));
    assert!(decision_section("## Fixture\n\nA\n\n## Fixture\n\nB\n", "## Fixture").is_err());
}

#[test]
fn inventory_requires_declared_evidence_and_rejects_unknown_reference_files() {
    let mut f = setup();
    save(f.root(), "corpus/reference/new/evidence.txt", b"reference");
    f.pinned = commit(f.root());
    assert!(inspect(f.root(), &f.pinned).is_err());
    let policy = json!({"schema_version":"1","evidence_paths":["docs/decisions.md",REVIEW],"reference_files":["corpus/reference/new/evidence.txt"]});
    save(f.root(), POLICY_PATH, &serde_json::to_vec(&policy).unwrap());
    f.pinned = commit(f.root());
    let inventory = inspect(f.root(), &f.pinned).unwrap();
    assert!(inventory.files.iter().any(|file| file.path == REVIEW));
    assert!(
        inventory
            .files
            .iter()
            .any(|file| file.path == "corpus/reference/new/evidence.txt")
    );
    let missing = json!({"schema_version":"1","evidence_paths":["docs/decisions.md"],"reference_files":["corpus/reference/new/evidence.txt"]});
    save(
        f.root(),
        POLICY_PATH,
        &serde_json::to_vec(&missing).unwrap(),
    );
    f.pinned = commit(f.root());
    assert!(inspect(f.root(), &f.pinned).is_err());
}

#[test]
fn inherited_review_requires_matching_historical_hash_and_series_checked_identity() {
    let mut f = setup();
    let historical_path = "docs/fixture-historical-review.json";
    let mut historical = f.review.clone();
    historical["rows"][0]["review_status"] = json!("proposed");
    historical["rows"][0]["series_check"] = json!(true);
    let original = serde_json::to_vec(&historical).unwrap();
    save(f.root(), historical_path, &original);
    let mut policy: Value =
        serde_json::from_slice(&fs::read(f.root().join(POLICY_PATH)).unwrap()).unwrap();
    policy["evidence_paths"]
        .as_array_mut()
        .unwrap()
        .push(json!(historical_path));
    save(f.root(), POLICY_PATH, &serde_json::to_vec(&policy).unwrap());
    f.review["historical_review_sha256"] = json!(digest(&original));
    let reviewed = serde_json::to_vec(&f.review).unwrap();
    save(f.root(), REVIEW, &reviewed);
    f.pinned = commit(f.root());
    let mut seed = f.seed.clone();
    seed["review_source_sha256"] = json!(digest(&reviewed));
    // The source review predates the inventory policy, as in production.
    let policy_bytes = fs::read(f.root().join(POLICY_PATH)).unwrap();
    fs::remove_file(f.root().join(POLICY_PATH)).unwrap();
    let source_without_policy = commit(f.root());
    seed["source_corpus_commit"] = json!(source_without_policy);
    save(f.root(), POLICY_PATH, &policy_bytes);
    f.pinned = commit(f.root());
    assert!(f.check(&seed));
    // Rebinding both ledger and seed cannot admit a row outside the original set.
    historical["rows"][0]["series_check"] = json!(false);
    let invalid = serde_json::to_vec(&historical).unwrap();
    save(f.root(), historical_path, &invalid);
    f.review["historical_review_sha256"] = json!(digest(&invalid));
    let rebound = serde_json::to_vec(&f.review).unwrap();
    save(f.root(), REVIEW, &rebound);
    f.pinned = commit(f.root());
    seed["source_corpus_commit"] = json!(f.pinned);
    seed["review_source_sha256"] = json!(digest(&rebound));
    assert!(!f.check(&seed));
    // Restoring the historical bytes without its hash also fails.
    save(f.root(), historical_path, &original);
    f.pinned = commit(f.root());
    seed["source_corpus_commit"] = json!(f.pinned);
    assert!(!f.check(&seed));
}

#[test]
fn retired_debt_requires_exact_accepted_partition_and_ancestor_resolution_proof() {
    let mut f = setup();
    let old = f.seed["rows"][0].clone();
    let raw = f.input().read(SOURCE).unwrap();
    let mut provisions: Value = serde_json::from_slice(&raw).unwrap();
    let mut article = provisions[0].clone();
    article["id"] = json!(format!(
        "{}:article:9",
        old["instrument_id"].as_str().unwrap()
    ));
    article["number"] = json!("9");
    article["label"] = json!("Artículo 9");
    article["text"] = json!("Nueve.");
    let text = provisions[0]["text"]
        .as_str()
        .unwrap()
        .replace("Artículo 9.- Nueve.\n\n", "");
    provisions[0]["text"] = json!(text);
    provisions.as_array_mut().unwrap().push(article.clone());
    save(f.root(), SOURCE, &serde_json::to_vec(&provisions).unwrap());
    let resolved = commit(f.root());
    let source = GitInput::new(f.root(), &resolved).unwrap();
    f.seed["schema_version"] = json!("3");
    f.seed["rows"] = json!([]);
    f.seed["retirements"] = json!([{"row":old,"resolved_at_commit":resolved,"provision_id":article["id"],"source_path":SOURCE,"source_blob_oid":source.blob_oid(SOURCE).unwrap(),"source_sha256":digest(&source.read(SOURCE).unwrap())}]);
    f.pinned = resolved;
    assert!(f.check(&f.seed));
    for field in [
        "resolved_at_commit",
        "provision_id",
        "source_blob_oid",
        "source_sha256",
    ] {
        let mut bad = f.seed.clone();
        bad["retirements"][0][field] = json!(if field == "source_sha256" {
            "0".repeat(64)
        } else if field == "provision_id" {
            "urn:absent".into()
        } else {
            "0".repeat(40)
        });
        assert!(!f.check(&bad), "{field}");
    }
    let mut dropped = f.seed.clone();
    dropped["retirements"] = json!([]);
    assert!(!f.check(&dropped));
    let mut duplicate = f.seed.clone();
    duplicate["rows"] = json!([old]);
    assert!(!f.check(&duplicate));
    let mut promoted = f.seed.clone();
    promoted["retirements"][0]["row"] = f.review["rows"][1].clone();
    assert!(!f.check(&promoted));
    let mut invented = f.seed.clone();
    invented["retirements"][0]["row"]["reviewer"] = json!("invented");
    assert!(!f.check(&invented));
    let mut v2 = f.seed.clone();
    v2["schema_version"] = json!("2");
    assert!(!f.check(&v2));
    provisions.as_array_mut().unwrap().pop();
    save(f.root(), SOURCE, &serde_json::to_vec(&provisions).unwrap());
    f.pinned = commit(f.root());
    assert!(!f.check(&f.seed));
}
