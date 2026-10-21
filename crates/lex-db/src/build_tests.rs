//! End-to-end release tests with synthetic law and real compiled-builder binding.
use super::*;
use serde_json::json;
use std::process::Command;

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
    let full = root.join(path);
    fs::create_dir_all(full.parent().unwrap()).unwrap();
    fs::write(full, bytes).unwrap();
}
fn json_file(root: &Path, path: &str, value: &Value) {
    save(root, path, &serde_json::to_vec(value).unwrap());
}
fn commit(root: &Path) -> String {
    git(root, &["add", "--all"]);
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
            "synthetic fixture",
        ],
    );
    git(root, &["rev-parse", "HEAD"])
}
fn source(root: &Path) {
    let f: Value = serde_json::from_slice(include_bytes!(
        "../../../fixtures/database/adversarial.json"
    ))
    .unwrap();
    let i = &f["instruments"][0];
    let directory = "corpus/mx/db-fixture";
    // Only the two synthetic articles are needed; no false temporal claims.
    let provisions = f["provisions"].as_array().unwrap()[..2].to_vec();
    let coverage = json!({"schema_version":"0.1.0","strategy_version":"glossary-extraction-4","instrument_id":i["id"],"source_sha256":i["source_sha256"],"extracted_text_sha256":i["extracted_text_sha256"],"status":"none_detected","glossaries":[],"imports":[],"term_count":0,"usage_count":0});
    let manifest = json!({"schema_version":"0.1.0","instrument_id":i["id"],"operational_source":"camara_de_diputados","formal_publication_source":"dof","publisher":"Synthetic fixture","official_url":"https://example.invalid/fixture.pdf","retrieved_at":i["retrieved_at"],"http_status":200,"source_sha256":i["source_sha256"],"extracted_text_sha256":i["extracted_text_sha256"],"parser_version":"0.1.0","schema_version_used":"0.1.0"});
    for (name, value) in [
        ("instrument.json", i.clone()),
        ("provisions.json", json!(provisions)),
        ("references.json", json!([])),
        ("terms.json", json!([])),
        ("term-usages.json", json!([])),
        ("glossary-coverage.json", coverage),
        ("source-manifest.json", manifest),
        (
            "validation.json",
            json!({"schema_version":"0.1.0","instrument_id":i["id"],"valid":true,"article_count":2,"transitory_count":0,"reference_count":0,"issues":[]}),
        ),
    ] {
        json_file(root, &format!("{directory}/{name}"), &value);
    }
    json_file(
        root,
        "adapters/fixture/db-fixture.json",
        &json!({"instrument_id":i["id"],"slug":"db-fixture","parser":"diputados","expected_articles":2,"expected_transitories":0,"expected_annexes":0,"allow_article_gaps":true}),
    );
}

fn repository() -> (tempfile::TempDir, String) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    git(root, &["init", "-q"]);
    for (path, bytes) in BUILDER_FILES {
        save(root, path, bytes);
    }
    for (path, bytes) in include!(concat!(env!("OUT_DIR"), "/builder_inputs.rs")) {
        save(root, path, bytes);
    }
    save(root, ".gitignore", b"/derived/\n");
    source(root);
    let decisions = "## Fixture\n\nSynthetic decision; no legal authority.\n";
    save(root, "docs/decisions.md", decisions.as_bytes());
    json_file(
        root,
        crate::input::POLICY_PATH,
        &json!({"schema_version":"1","evidence_paths":["docs/decisions.md","docs/fixture-review.json"],"reference_files":[]}),
    );
    json_file(
        root,
        "adapters/diputados/_closure-accepted-gaps.json",
        &json!({"accepted":[]}),
    );
    let base = commit(root);
    let review =
        json!({"corpus_commit":base,"decision_reference":"docs/decisions.md#fixture","rows":[]});
    json_file(root, "docs/fixture-review.json", &review);
    let reviewed = commit(root);
    let seed = json!({"schema_version":"2","source_corpus_commit":reviewed,"decision_source_path":"docs/decisions.md","decision_source_sha256":digest(decisions.as_bytes()),"review_source_path":"docs/fixture-review.json","review_source_sha256":digest(&serde_json::to_vec(&review).unwrap()),"decision_sections":[{"reference":"docs/decisions.md#fixture","heading":"## Fixture","sha256":digest(decisions.as_bytes())}],"rows":[]});
    json_file(root, crate::contract::DEBT_PATH, &seed);
    let pinned = commit(root);
    (temp, pinned)
}

#[test]
fn complete_release_is_reproducible_read_only_and_refuses_overwrite() {
    let (temp, pinned) = repository();
    let root = temp.path();
    let first = project(root, &pinned, &pinned, Path::new("derived/db/first")).unwrap();
    let manifest = fs::read(first.directory.join("manifest.json")).unwrap();
    let value: Value = serde_json::from_slice(&manifest).unwrap();
    validate_manifest(&value).unwrap();
    assert_eq!(
        value["database_sha256"],
        digest(&fs::read(first.directory.join("corpus.sqlite")).unwrap())
    );
    let db = Connection::open_with_flags(
        first.directory.join("corpus.sqlite"),
        OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    assert!(db.execute("DELETE FROM input_files", []).is_err());
    drop(db);
    // Dirty canonical files never become committed snapshot inputs.
    save(
        root,
        "corpus/mx/db-fixture/provisions.json",
        b"dirty invalid JSON",
    );
    let second = project(root, &pinned, &pinned, Path::new("derived/db/second")).unwrap();
    assert_eq!(
        manifest,
        fs::read(second.directory.join("manifest.json")).unwrap()
    );
    assert_eq!(first.database_sha256, second.database_sha256);
    assert!(project(root, &pinned, &pinned, Path::new("derived/db/first")).is_err());
    assert_eq!(
        manifest,
        fs::read(first.directory.join("manifest.json")).unwrap()
    );
    assert!(
        fs::read_dir(root.join("derived/db/first"))
            .unwrap()
            .all(|e| !e
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".building-"))
    );
    // Unsupported committed source contracts cannot replace a good release.
    let source_path = "corpus/mx/db-fixture/source-manifest.json";
    let mut source: Value =
        serde_json::from_slice(&fs::read(root.join(source_path)).unwrap()).unwrap();
    source["operational_source"] = json!("unregistered_fixture_source");
    json_file(root, source_path, &source);
    // Restore the intentionally dirty JSON before creating this corrupt commit.
    let canonical = git(
        root,
        &[
            "show",
            &format!("{pinned}:corpus/mx/db-fixture/provisions.json"),
        ],
    );
    save(
        root,
        "corpus/mx/db-fixture/provisions.json",
        canonical.as_bytes(),
    );
    let invalid = commit(root);
    let error = project(root, &invalid, &invalid, Path::new("derived/db/first")).unwrap_err();
    assert!(format!("{error:#}").contains("unregistered_fixture_source"));
    assert_eq!(
        manifest,
        fs::read(first.directory.join("manifest.json")).unwrap()
    );
    // An old builder claim cannot label a newer compiled implementation.
    save(root, "crates/lex-db/schema.sql", b"different schema");
    let different = commit(root);
    assert!(check_builder(&GitInput::new(root, &different).unwrap()).is_err());
}

fn complete_source(root: &Path) -> String {
    let mut f: Value = serde_json::from_slice(include_bytes!(
        "../../../fixtures/database/adversarial.json"
    ))
    .unwrap();
    f["provisions"][2]["number"] = json!("1");
    let directory = "corpus/mx/db-fixture";
    for (name, key) in [
        ("provisions.json", "provisions"),
        ("references.json", "reference_edges"),
        ("terms.json", "defined_terms"),
        ("term-usages.json", "term_usages"),
    ] {
        json_file(root, &format!("{directory}/{name}"), &f[key]);
    }
    let mut coverage = f["glossary_coverage"][0].clone();
    coverage.as_object_mut().unwrap().remove("hold");
    coverage["glossaries"][0]["provision_text_sha256"] = json!(digest(
        f["provisions"][0]["text"].as_str().unwrap().as_bytes()
    ));
    coverage["source_sha256"] = f["instruments"][0]["source_sha256"].clone();
    coverage["extracted_text_sha256"] = f["instruments"][0]["extracted_text_sha256"].clone();
    json_file(
        root,
        &format!("{directory}/glossary-coverage.json"),
        &coverage,
    );
    json_file(
        root,
        "adapters/fixture/db-fixture.json",
        &json!({"instrument_id":f["instruments"][0]["id"],"slug":"db-fixture","parser":"diputados","expected_articles":2,"expected_transitories":1,"expected_annexes":1,"allow_article_gaps":true}),
    );
    commit(root)
}

#[test]
fn independent_verifier_and_queries_cover_all_synthetic_record_groups() {
    let (temp, _) = repository();
    let root = temp.path();
    let pinned = complete_source(root);
    let built = project(root, &pinned, &pinned, Path::new("derived/db/m3")).unwrap();
    let database = built.directory.join("corpus.sqlite");
    let manifest = built.directory.join("manifest.json");
    let before = fs::read(&database).unwrap();
    let verified = crate::verify::verify(root, &database, &manifest, &pinned).unwrap();
    assert_eq!(verified.normalized_records, 12);
    assert!(verified.stale_input_paths.is_empty());
    assert!(verified.fts_token_occurrences > 0);
    assert_eq!(before, fs::read(&database).unwrap());
    let reader = crate::queries::Reader::open(&database).unwrap();
    let id = "urn:lex-mx:federal:statute:db-fixture:article:1";
    assert!(matches!(
        reader.lookup(id).unwrap(),
        crate::queries::Lookup::Found { .. }
    ));
    assert!(matches!(
        reader
            .lookup("urn:lex-mx:federal:statute:db-fixture:article:9999")
            .unwrap(),
        crate::queries::Lookup::NotEstablished { .. }
    ));
    assert_eq!(
        reader
            .article_ids("urn:lex-mx:federal:statute:db-fixture", "article", "1")
            .unwrap(),
        vec![id]
    );
    assert_eq!(reader.context_ids(id, 1).unwrap().len(), 2);
    let a = reader.search("obligacion", None, None, 100).unwrap();
    let b = reader.search("obligación", None, None, 100).unwrap();
    assert_eq!(
        a.iter().map(|h| &h.id).collect::<Vec<_>>(),
        b.iter().map(|h| &h.id).collect::<Vec<_>>()
    );
    assert!(!a.is_empty());
    assert!(
        reader
            .search("\" OR cerveza", None, None, 100)
            .unwrap()
            .is_empty()
    );
    assert!(
        reader
            .search("cerveza", Some("missing-owner"), None, 100)
            .unwrap()
            .is_empty()
    );
    assert!(reader.search(" ", None, None, 1).is_err());
    assert!(reader.search("a", None, None, 101).is_err());
    json_file(
        root,
        "adapters/fixture/new-input.json",
        &json!({"note":"new input after release"}),
    );
    commit(root);
    let stale = crate::verify::verify(root, &database, &manifest, &pinned).unwrap();
    assert_eq!(
        stale.stale_input_paths,
        vec!["adapters/fixture/new-input.json"]
    );
}

#[test]
fn independent_verifier_rejects_rehashed_mutations_including_fts() {
    let (temp, _) = repository();
    let root = temp.path();
    let pinned = complete_source(root);
    let built = project(root, &pinned, &pinned, Path::new("derived/db/m3")).unwrap();
    let original = built.directory.join("corpus.sqlite");
    let original_manifest: Value =
        serde_json::from_slice(&fs::read(built.directory.join("manifest.json")).unwrap()).unwrap();
    for sql in [
        r#"UPDATE provisions SET extras='{"label":"Artículo 999","number":"999",' || substr(extras,2) WHERE ordinal=0; INSERT INTO provisions_fts(provisions_fts) VALUES('rebuild')"#,
        r#"UPDATE reference_qualifiers SET payload='{"text":"forged",' || substr(payload,2)"#,
        "UPDATE provisions SET extras=json_remove(extras,'$.unknown_future_metadata.present_null') WHERE ordinal=0",
        "UPDATE provisions SET source_pointer='/999' WHERE ordinal=0",
        "UPDATE legal_operator_reviews SET payload=json_set(payload,'$.source_text_sha256','wrong')",
        "UPDATE reference_qualifiers SET ordinal=ordinal+10",
        "UPDATE reference_edges SET extras=json_set(extras,'$.start_char',999999) WHERE ordinal=0",
        "DELETE FROM provisions WHERE ordinal=1",
        "UPDATE input_files SET bytes=CAST('[]' AS BLOB) WHERE path LIKE '%/terms.json'",
        "INSERT INTO provisions_fts(provisions_fts) VALUES('delete-all')",
        "CREATE TABLE provisions_fts_untrusted(x TEXT)",
        "UPDATE corpus_release SET specification='{}'",
    ] {
        let copy = root.join("corrupted.sqlite");
        fs::copy(&original, &copy).unwrap();
        let c = Connection::open(&copy).unwrap();
        c.execute_batch("PRAGMA foreign_keys=OFF").unwrap();
        c.execute_batch(sql).unwrap();
        let mut m = original_manifest.clone();
        m["logical_content_sha256"] = json!(crate::logical::hash(&c).unwrap());
        if sql.contains("delete-all") {
            assert_eq!(
                m["logical_content_sha256"],
                original_manifest["logical_content_sha256"]
            );
        }
        c.close().unwrap();
        m["database_sha256"] = json!(digest(&fs::read(&copy).unwrap()));
        let sidecar = root.join("corrupted-manifest.json");
        json_file(root, "corrupted-manifest.json", &m);
        assert!(
            crate::verify::verify(root, &copy, &sidecar, &pinned).is_err(),
            "{sql}"
        );
    }
}

#[test]
fn schema_mismatch_and_wrong_builder_fail_before_output_creation() {
    let (temp, pinned) = repository();
    let root = temp.path();
    save(root, "schemas/defined-term.schema.json", b"{}");
    let changed = commit(root);
    let output = Path::new("derived/db/rejected-schema");
    let error = project(root, &changed, &pinned, output).unwrap_err();
    assert!(error.to_string().contains("snapshot schema differs"));
    assert!(!root.join(output).exists());
    save(root, "Cargo.toml", b"not the compiled builder");
    let wrong = commit(root);
    let output = Path::new("derived/db/rejected-builder");
    assert!(
        project(root, &changed, &wrong, output)
            .unwrap_err()
            .to_string()
            .contains("executing builder differs")
    );
    assert!(!root.join(output).exists());
}

#[test]
fn raw_glossary_null_is_not_erased_before_schema_validation() {
    let (temp, _) = repository();
    let root = temp.path();
    let path = "corpus/mx/db-fixture/glossary-coverage.json";
    let mut coverage: Value = serde_json::from_slice(&fs::read(root.join(path)).unwrap()).unwrap();
    coverage["hold"] = Value::Null;
    json_file(root, path, &coverage);
    let invalid = commit(root);
    let error = project(
        root,
        &invalid,
        &invalid,
        Path::new("derived/db/rejected-null"),
    )
    .unwrap_err();
    let message = format!("{error:#}");
    assert!(message.contains("glossary-coverage.json") && message.contains("null"));
    assert!(
        fs::read_dir(root.join("derived/db/rejected-null"))
            .unwrap()
            .next()
            .is_none()
    );
}
