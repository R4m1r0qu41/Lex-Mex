use lex_db::{
    build::{check_loaded, output_directory},
    contract::{Handling, InputFile},
    digest,
    input::{Inspection, Snapshot},
    load::populate,
    probe::connection,
};
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

fn fixture() -> (Inspection, Snapshot) {
    let f: Value = serde_json::from_slice(include_bytes!(
        "../../../fixtures/database/adversarial.json"
    ))
    .unwrap();
    let directory = "corpus/mx/db-fixture";
    let mut data = Snapshot::new();
    for (name, value) in [
        ("instrument.json", f["instruments"][0].clone()),
        ("provisions.json", f["provisions"].clone()),
        ("references.json", f["reference_edges"].clone()),
        ("terms.json", f["defined_terms"].clone()),
        ("term-usages.json", f["term_usages"].clone()),
        ("glossary-coverage.json", f["glossary_coverage"][0].clone()),
        (
            "source-manifest.json",
            json!({"fixture_source":"unchanged"}),
        ),
        ("enactment-evidence.json", json!([])),
    ] {
        data.insert(
            format!("{directory}/{name}"),
            serde_json::to_vec(&value).unwrap(),
        );
    }
    for (path, value) in [
        (
            "adapters/diputados/_closure-accepted-gaps.json",
            json!({"accepted":[{"slug":"db-fixture","reason":"synthetic fixture gap","errors":["synthetic"]}]}),
        ),
        (
            lex_db::contract::DEBT_PATH,
            json!({"rows":[{"instrument_id":f["instruments"][0]["id"],"printed_label":"9","reviewer":"Fixture","review_status":"accepted"}]}),
        ),
        (
            "corpus/mx/test-standard/standard.json",
            json!({"id":"fixture-standard","title":"Not a law"}),
        ),
    ] {
        data.insert(path.into(), serde_json::to_vec(&value).unwrap());
    }
    data.insert(
        "corpus/reference/fixture-registry/source-manifest.json".into(),
        serde_json::to_vec(
            &json!({"name":"Synthetic reference registry","status":"evidence only"}),
        )
        .unwrap(),
    );
    data.insert(
        "corpus/mx/test-standard/extracted-text.txt".into(),
        b"Excluded standard body".to_vec(),
    );
    let files = data
        .iter()
        .map(|(p, b)| InputFile {
            path: p.clone(),
            sha256: digest(b),
            bytes: u64::try_from(b.len()).unwrap(),
            handling: if p.ends_with("extracted-text.txt") {
                Handling::ExcludedStandardBody
            } else if p.contains("test-standard")
                || p.starts_with("adapters/")
                || p.starts_with("corpus/reference/")
            {
                Handling::EvidenceOnly
            } else {
                Handling::Normalized
            },
        })
        .collect();
    (
        Inspection {
            schema_version: "1".into(),
            corpus_commit: "a".repeat(40),
            files,
            federal_instruments: 1,
            standards_catalogue: 1,
            accepted_debt_rows: 0,
        },
        data,
    )
}

#[test]
fn transactional_load_preserves_rows_and_separates_catalogue() {
    let (inspection, data) = fixture();
    let mut c = connection().unwrap();
    populate(&mut c, &inspection, &data, "fixture-release", "{}").unwrap();
    let counts = check_loaded(&c, &data).unwrap();
    assert_eq!(counts["instruments"], 1);
    assert_eq!(counts["standards_catalog"], 1);
    assert_eq!(counts["instrument_reproducibility"], 0);
    assert_eq!(counts["coverage"], 4);
    assert_eq!(
        c.query_row("SELECT slug FROM instruments", [], |r| r
            .get::<_, String>(0))
            .unwrap(),
        "db-fixture"
    );
    let hits: i64 = c
        .query_row(
            "SELECT count(*) FROM provisions_fts WHERE provisions_fts MATCH 'obligacion'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(hits > 0);
    c.execute(
        "UPDATE provisions SET source_pointer='/99' WHERE ordinal=0",
        [],
    )
    .unwrap();
    assert!(check_loaded(&c, &data).is_err());
}

#[test]
fn missing_target_or_duplicate_rows_roll_back_everything() {
    for duplicate in [false, true] {
        let (mut inspection, mut data) = fixture();
        let path = "corpus/mx/db-fixture/provisions.json";
        let mut value: Value = serde_json::from_slice(&data[path]).unwrap();
        if duplicate {
            let row = value[0].clone();
            value.as_array_mut().unwrap().push(row);
        } else {
            value.as_array_mut().unwrap().retain(|r| r["number"] != "2");
        }
        data.insert(path.into(), serde_json::to_vec(&value).unwrap());
        for f in &mut inspection.files {
            let b = &data[&f.path];
            f.sha256 = digest(b);
            f.bytes = u64::try_from(b.len()).unwrap();
        }
        let mut c = connection().unwrap();
        assert!(populate(&mut c, &inspection, &data, "fixture", "{}").is_err());
        assert_eq!(
            c.query_row("SELECT count(*) FROM input_files", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            0
        );
        assert_eq!(
            c.query_row("SELECT count(*) FROM corpus_release", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }
}

fn git(root: &Path, args: &[&str]) {
    assert!(
        Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .status()
            .unwrap()
            .success()
    );
}
#[test]
fn output_path_rejects_protected_unignored_and_symlinked_locations() {
    let temp = tempfile::tempdir().unwrap();
    let root = fs::canonicalize(temp.path()).unwrap();
    git(&root, &["init", "-q"]);
    fs::write(root.join(".gitignore"), "/derived/\n").unwrap();
    for path in [
        "corpus",
        "adapters/output",
        "derived/db/../corpus",
        "derived/other",
        "/private/tmp/outside-db",
        "derived/db/./test",
    ] {
        assert!(output_directory(&root, Path::new(path)).is_err(), "{path}");
    }
    let good = output_directory(&root, Path::new("derived/db/test")).unwrap();
    assert!(good.is_dir());
    fs::write(good.join("previous-good"), b"untouched").unwrap();
    assert!(
        lex_db::build::project(
            &root,
            &"0".repeat(40),
            &"0".repeat(40),
            Path::new("derived/db/test")
        )
        .is_err()
    );
    assert_eq!(fs::read(good.join("previous-good")).unwrap(), b"untouched");
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(temp.path(), root.join("derived/db/alias")).unwrap();
        assert!(output_directory(&root, Path::new("derived/db/alias/output")).is_err());
    }
}

#[test]
fn supporting_table_and_handling_corruption_is_detected() {
    for mutation in [
        "UPDATE source_manifests SET extras='{}'",
        "UPDATE source_anchors SET kind='instrument_title' WHERE kind='provision'",
        "UPDATE coverage SET state='incorrect' WHERE stage='linked'",
        "DELETE FROM standards_catalog",
        "DELETE FROM evidence_artifacts",
        "UPDATE input_files SET handling='normalized' WHERE handling='evidence_only'",
        "UPDATE known_structural_debt SET extras='{}'",
        "UPDATE coverage_accepted_gaps SET extras='{}'",
    ] {
        let (inspection, data) = fixture();
        let mut c = connection().unwrap();
        populate(&mut c, &inspection, &data, "fixture", "{}").unwrap();
        check_loaded(&c, &data).unwrap();
        assert!(
            c.execute(mutation, []).unwrap() > 0,
            "control did not mutate: {mutation}"
        );
        assert!(check_loaded(&c, &data).is_err(), "{mutation}");
    }
}

#[test]
fn invalidated_glossary_fractions_are_preserved_as_source_accounting() {
    let (inspection, data) = fixture();
    let mut c = connection().unwrap();
    populate(&mut c, &inspection, &data, "fixture", "{}").unwrap();
    check_loaded(&c, &data).unwrap();
    let raw: String = c
        .query_row("SELECT extras FROM glossary_coverage", [], |r| r.get(0))
        .unwrap();
    let value: Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(
        value["glossaries"][0]["invalidated_fractions"],
        json!(["II"])
    );
    c.execute("UPDATE glossary_coverage SET extras=json_remove(extras,'$.glossaries[0].invalidated_fractions')",[]).unwrap();
    assert!(check_loaded(&c, &data).is_err());
}

#[test]
fn reference_registry_manifest_is_evidence_not_law_source_provenance() {
    let (inspection, data) = fixture();
    let mut c = connection().unwrap();
    populate(&mut c, &inspection, &data, "fixture", "{}").unwrap();
    check_loaded(&c, &data).unwrap();
    let path = "corpus/reference/fixture-registry/source-manifest.json";
    assert_eq!(
        c.query_row(
            "SELECT count(*) FROM source_manifests WHERE input_path=?1",
            [path],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
    assert_eq!(
        c.query_row(
            "SELECT kind FROM evidence_artifacts WHERE path=?1",
            [path],
            |r| r.get::<_, String>(0)
        )
        .unwrap(),
        "evidence_only"
    );
}

#[test]
fn exact_lookup_combines_sourced_qualifications_and_exclusions() {
    let (inspection, data) = fixture();
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("queries.sqlite");
    let mut c = rusqlite::Connection::open(&path).unwrap();
    c.execute_batch(include_str!("../schema.sql")).unwrap();
    populate(&mut c, &inspection, &data, "fixture", "{}").unwrap();
    c.close().unwrap();
    let reader = lex_db::queries::Reader::open(&path).unwrap();
    match reader
        .lookup("urn:lex-mx:federal:statute:db-fixture:article:9")
        .unwrap()
    {
        lex_db::queries::Lookup::Qualified { qualifications } => {
            assert_eq!(qualifications[0]["kind"], json!("not_separately_parsed"));
        }
        other => panic!("unexpected {other:?}"),
    }
    match reader.lookup("fixture-standard:clause:1").unwrap() {
        lex_db::queries::Lookup::Qualified { qualifications } => {
            assert_eq!(qualifications[0]["kind"], json!("excluded"));
        }
        other => panic!("unexpected {other:?}"),
    }
}
