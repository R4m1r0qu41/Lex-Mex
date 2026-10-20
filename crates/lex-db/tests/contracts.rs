use lex_db::{
    contract::{BuildSpecification, DatabaseManifest, Handling, InputFile, validate_manifest},
    probe::{build_fixture, connection, verify_fixture},
};
use serde_json::{Value, json};

const FIXTURE: &[u8] = include_bytes!("../../../fixtures/database/adversarial.json");

#[test]
fn sqlite_features_and_full_fixture_parity() {
    let report = lex_db::probe::run_probe().unwrap();
    assert!(report.fts5 && report.json && report.strict);
    assert_eq!(report.verified_records, 12);
}

#[test]
fn corruption_of_optional_presence_numbers_children_and_reviews_fails() {
    for mutation in [
        "UPDATE provisions SET extras=json_remove(extras,'$.unknown_future_metadata.present_null') WHERE ordinal=0",
        "UPDATE provisions SET extras=json_set(extras,'$.unknown_future_metadata.decimal',1) WHERE ordinal=0",
        "DELETE FROM legal_operator_reviews",
        "UPDATE term_usages SET extras=json_set(extras,'$.span','changed')",
        "UPDATE reference_qualifiers SET ordinal=ordinal+10",
    ] {
        let mut c = connection().unwrap();
        build_fixture(&mut c, FIXTURE).unwrap();
        c.execute(mutation, []).unwrap();
        assert!(verify_fixture(&c).is_err(), "{mutation}");
    }
}

#[test]
fn invalid_spans_versions_duplicates_and_live_targets_fail_transactionally() {
    for (group, field, value) in [
        ("reference_edges", "target_provision_id", json!("missing")),
        ("reference_edges", "source_span", json!("wrong")),
        ("provisions", "schema_version", json!("999")),
        ("term_usages", "term_id", json!("missing")),
        ("defined_terms", "end_char", json!(99999)),
    ] {
        let mut fixture: Value = serde_json::from_slice(FIXTURE).unwrap();
        fixture[group][0][field] = value;
        let mut c = connection().unwrap();
        assert!(
            build_fixture(&mut c, &serde_json::to_vec(&fixture).unwrap()).is_err(),
            "{group}.{field}"
        );
        assert_eq!(
            c.query_row("SELECT count(*) FROM input_files", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }
    let mut fixture: Value = serde_json::from_slice(FIXTURE).unwrap();
    let duplicate = fixture["provisions"][0].clone();
    fixture["provisions"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    assert!(
        build_fixture(
            &mut connection().unwrap(),
            &serde_json::to_vec(&fixture).unwrap()
        )
        .is_err()
    );
}

#[test]
fn manifest_identity_and_closed_boundary_are_checked() {
    let spec = BuildSpecification {
        schema_version: "1".into(),
        corpus_commit: "a".repeat(40),
        builder_commit: "b".repeat(40),
        cargo_lock_sha256: "c".repeat(64),
        rust_toolchain: "fixture".into(),
        sqlite_version: "fixture".into(),
        sqlite_compile_options: vec![],
        source_commit_timestamp: 0,
        inclusion_policy: "federal_with_standards_catalogue".into(),
        inputs: vec![InputFile {
            path: "corpus/mx/test/instrument.json".into(),
            sha256: "d".repeat(64),
            bytes: 1,
            handling: Handling::Normalized,
        }],
    };
    let manifest = DatabaseManifest {
        schema_version: "1".into(),
        release_id: lex_db::digest(&serde_json::to_vec(&spec).unwrap()),
        specification: spec,
        database_sha256: "e".repeat(64),
        logical_content_sha256: "f".repeat(64),
    };
    let value = serde_json::to_value(manifest).unwrap();
    assert!(validate_manifest(&value).is_ok());
    let mut invalid = value.clone();
    invalid["release_id"] = json!("f".repeat(64));
    assert!(validate_manifest(&invalid).is_err());
    let mut invalid = value;
    invalid["unknown"] = json!(true);
    assert!(validate_manifest(&invalid).is_err());
    assert!(lex_db::contract::safe_path("../private").is_err());
    assert!(lex_db::contract::safe_path("/absolute").is_err());
}

#[test]
fn unknown_inputs_fail_and_standard_text_is_excluded() {
    use std::collections::BTreeSet;
    let standards = BTreeSet::from(["nom-fixture".to_owned()]);
    assert_eq!(
        lex_db::input::handling("corpus/mx/nom-fixture/clauses.json", &standards).unwrap(),
        Handling::ExcludedStandardBody
    );
    assert!(lex_db::input::handling("corpus/mx/test/new-unknown.json", &standards).is_err());
    assert!(lex_db::input::handling("corpus/mx/nom-fixture/new.json", &standards).is_err());
}

#[test]
fn cross_instrument_usages_and_glossary_accounting_presence_survive() {
    let mut fixture: Value = serde_json::from_slice(FIXTURE).unwrap();
    let mut owner = fixture["instruments"][0].clone();
    owner["id"] = json!("urn:lex-mx:federal:statute:other-fixture");
    let mut provision = fixture["provisions"][1].clone();
    provision["id"] = json!("urn:lex-mx:federal:statute:other-fixture:article:1");
    provision["instrument_id"] = owner["id"].clone();
    provision["text"] = json!("La cerveza.");
    let usage = json!({"schema_version":"0.1.0","id":"urn:lex-mx:federal:statute:other-fixture:usage:1","provision_id":provision["id"],"term_id":fixture["defined_terms"][0]["id"],"span":"cerveza","start_char":3,"end_char":10});
    fixture["instruments"].as_array_mut().unwrap().push(owner);
    fixture["provisions"]
        .as_array_mut()
        .unwrap()
        .push(provision);
    fixture["term_usages"].as_array_mut().unwrap().push(usage);
    fixture["glossary_coverage"][0]["glossaries"][0]["unread_fractions"] = json!([]);
    let mut c = connection().unwrap();
    build_fixture(&mut c, &serde_json::to_vec(&fixture).unwrap()).unwrap();
    verify_fixture(&c).unwrap();
    c.execute("UPDATE glossary_coverage SET extras=json_remove(extras,'$.glossaries[0].unread_fractions')",[]).unwrap();
    assert!(verify_fixture(&c).is_err());
}

#[test]
fn source_ordinals_restart_per_file_and_same_file_collisions_fail() {
    let mut c = connection().unwrap();
    build_fixture(&mut c, FIXTURE).unwrap();
    c.execute(
        "INSERT INTO input_files VALUES('second.json',?1,?2,'normalized')",
        rusqlite::params![b"[]".as_slice(), lex_db::digest(b"[]")],
    )
    .unwrap();
    for (table, columns) in [
        ("instruments", ""),
        (
            "reference_edges",
            ",source_provision_id,target_instrument_id,target_provision_id,resolution_status",
        ),
        ("defined_terms", ",instrument_id,defining_provision_id"),
        ("term_usages", ",provision_id,term_id"),
    ] {
        let sql = format!(
            "INSERT INTO {table}(id{columns},input_path,source_pointer,ordinal,extras) SELECT id||':other'{columns},'second.json','/0',0,extras FROM {table} WHERE ordinal=0 AND input_path='fixture.json'"
        );
        assert_eq!(c.execute(&sql, []).unwrap(), 1, "{table}");
        let collision = sql
            .replace("':other'", "':duplicate'")
            .replace("'second.json','/0'", "'second.json','/1'");
        assert!(
            c.execute(&collision, []).is_err(),
            "per-file ordinal must be unique in {table}"
        );
    }
    assert!(verify_fixture(&c).is_err()); // extra rows have no counterpart in fixture.json
}

#[test]
fn source_pointer_and_input_file_corruption_fail_readback() {
    for mutation in [
        "UPDATE provisions SET source_pointer='/other' WHERE ordinal=0",
        "UPDATE reference_edges SET input_path='other.json'",
    ] {
        let mut c = connection().unwrap();
        build_fixture(&mut c, FIXTURE).unwrap();
        c.execute(
            "INSERT INTO input_files VALUES('other.json',?1,?2,'normalized')",
            rusqlite::params![b"[]".as_slice(), lex_db::digest(b"[]")],
        )
        .unwrap();
        c.execute(mutation, []).unwrap();
        assert!(verify_fixture(&c).is_err());
    }
}

#[test]
fn unresolved_edges_are_stored_without_inventing_live_targets() {
    let mut f: Value = serde_json::from_slice(FIXTURE).unwrap();
    f["reference_edges"][1]["resolution_status"] = json!("unresolved");
    let mut c = connection().unwrap();
    build_fixture(&mut c, &serde_json::to_vec(&f).unwrap()).unwrap();
    verify_fixture(&c).unwrap();
    let live: Option<String> = c
        .query_row(
            "SELECT live_target FROM reference_edges WHERE resolution_status='unresolved'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(live.is_none());
}
