//! Synthetic public-query contract tests: scope, absence, cursors and binding.
use super::*;
use crate::contract::{BuildSpecification, DatabaseManifest};

const A: &str = "urn:lex-mx:federal:statute:ley-a";
const B: &str = "urn:lex-mx:federal:statute:ley-b";
const COMMIT: &str = "1111111111111111111111111111111111111111";
const ART1: &str = "La autoridad supervisará el ámbito de aplicación. Véase el artículo 2 Bis, fracción I, y el artículo 7 de la Ley B.\r\nSegunda línea.";

struct Fixture {
    _dir: tempfile::TempDir,
    database: std::path::PathBuf,
    release: std::path::PathBuf,
    manifest: Value,
    binding_path: std::path::PathBuf,
}

fn extras(kind: &str, label: &str, number: &str) -> Value {
    json!({"schema_version":"0.1.0","provision_type":kind,"label":label,"number":number,
        "heading_context":{"title":"Título I","chapter":null},"publication_date":"2020-01-01",
        "effective_from":"2020-01-02","effective_to":null,"temporal_status":"effective",
        "temporal_basis":"deterministic_rule","temporal_confidence":0.9,"review_status":"machine_accepted",
        "transitory_effects":[]})
}

fn source(sha: &str, url: &str) -> Value {
    json!({"instrument_id":A,"publisher":"Synthetic publisher","official_url":url,"retrieved_at":"2026-01-01T00:00:00Z",
        "source_sha256":sha,"extracted_text_sha256":"c".repeat(64),"parser_version":"0.1.0","schema_version_used":"0.1.0",
        "reviewer":"private reviewer"})
}

/// (instrument, slug, kind, number, label, text)
fn provisions() -> Vec<(
    &'static str,
    String,
    &'static str,
    &'static str,
    &'static str,
    String,
)> {
    vec![
        (
            A,
            format!("{A}:article:1"),
            "article",
            "1",
            "Artículo 1",
            ART1.into(),
        ),
        (
            A,
            format!("{A}:article:2-bis"),
            "article",
            "2 Bis",
            "Artículo 2 Bis",
            "El niño y la autoridad.".into(),
        ),
        (
            A,
            format!("{A}:article:3"),
            "article",
            "3",
            "Artículo 3",
            "Primer artículo tres.".into(),
        ),
        (
            A,
            format!("{A}:article:3-2"),
            "article",
            "3o.",
            "Artículo 3o.",
            "Segundo artículo tres.".into(),
        ),
        (
            A,
            format!("{A}:transitory:primero"),
            "transitory",
            "PRIMERO",
            "PRIMERO",
            "Entra en vigor.".into(),
        ),
        (
            A,
            format!("{A}:annex:1"),
            "annex",
            "1",
            "ANEXO 1",
            "x".repeat(600),
        ),
        (
            B,
            format!("{B}:article:7"),
            "article",
            "7",
            "Artículo 7",
            "La autoridad de la Ley B.".into(),
        ),
    ]
}

fn input(c: &Connection, path: &str, bytes: &[u8]) {
    c.execute(
        "INSERT INTO input_files(path,bytes,sha256,handling) VALUES(?1,?2,?3,'normalized')",
        params![path, bytes, crate::digest(bytes)],
    )
    .unwrap();
}

#[allow(clippy::too_many_lines)]
fn build_database(path: &Path) {
    let c = Connection::open(path).unwrap();
    c.execute_batch(include_str!("../schema.sql")).unwrap();
    c.execute(
        "INSERT INTO corpus_release VALUES(?1,?2)",
        params![
            "e".repeat(64),
            json!({"corpus_commit":COMMIT,"builder_commit":COMMIT}).to_string()
        ],
    )
    .unwrap();
    for (n, (id, slug)) in [(A, "ley-a"), (B, "ley-b")].into_iter().enumerate() {
        for name in ["provisions.json", "references.json"] {
            input(
                &c,
                &format!("corpus/mx/{slug}/{name}"),
                format!("{slug}{name}").as_bytes(),
            );
        }
        let instrument = json!({"official_title":format!("Ley {slug}"),"short_name":slug.to_uppercase(),"publication_date":"2020-01-01","latest_reform_date":null});
        let path = format!("corpus/mx/{slug}/instrument.json");
        input(&c, &path, instrument.to_string().as_bytes());
        c.execute("INSERT INTO instruments(id,ordinal,input_path,source_pointer,extras) VALUES(?1,?2,?3,'',?4)", params![id, i64::try_from(n).unwrap(), path, instrument.to_string()]).unwrap();
        c.execute(
            "INSERT INTO source_anchors VALUES(?1,'instrument_title',?1)",
            [id],
        )
        .unwrap();
    }
    input(
        &c,
        "corpus/mx/ley-a/source-manifest.json",
        source(&"a".repeat(64), "http://example.invalid/a.pdf")
            .to_string()
            .as_bytes(),
    );
    input(
        &c,
        "corpus/mx/ley-a/annex-source-manifests.json",
        json!([source(
            &"b".repeat(64),
            "https://example.invalid/ANEXO%201.pdf"
        )])
        .to_string()
        .as_bytes(),
    );
    for (n, (owner, id, kind, number, label, text)) in provisions().into_iter().enumerate() {
        let slug = if owner == A { "ley-a" } else { "ley-b" };
        c.execute("INSERT INTO provisions(id,instrument_id,ordinal,text,input_path,source_pointer,extras) VALUES(?1,?2,?3,?4,?5,?6,?7)",
            params![id, owner, i64::try_from(n).unwrap(), text, format!("corpus/mx/{slug}/provisions.json"), format!("/{n}"), extras(kind, label, number).to_string()]).unwrap();
        c.execute(
            "INSERT INTO source_anchors VALUES(?1,'provision',?2)",
            params![id, owner],
        )
        .unwrap();
    }
    let edges = [
        (
            format!("{A}:article:1"),
            A,
            format!("{A}:article:2-bis"),
            "resolved",
            "2 Bis",
            66,
            72,
        ),
        (
            format!("{A}:article:1"),
            B,
            format!("{B}:article:7"),
            "resolved",
            "7",
            100,
            101,
        ),
        (
            format!("{A}:article:1"),
            A,
            format!("{A}:article:99"),
            "unresolved",
            "99",
            0,
            2,
        ),
        (
            A.to_owned(),
            A,
            format!("{A}:article:1"),
            "resolved",
            "1",
            0,
            1,
        ),
        (
            format!("{B}:article:7"),
            A,
            format!("{A}:article:1"),
            "resolved",
            "1",
            0,
            1,
        ),
    ];
    for (n, (source, target_instrument, target, status, span, start, end)) in
        edges.into_iter().enumerate()
    {
        let slug = if source.starts_with(B) {
            "ley-b"
        } else {
            "ley-a"
        };
        let id = format!("{source}:reference:{n}");
        c.execute("INSERT INTO reference_edges(id,source_provision_id,target_instrument_id,target_provision_id,resolution_status,ordinal,input_path,source_pointer,extras) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![id, source, target_instrument, target, status, i64::try_from(n).unwrap(), format!("corpus/mx/{slug}/references.json"), format!("/{n}"),
                json!({"basis":"express_cross_reference","confidence":1.0,"start_char":start,"end_char":end,"reference_form":"direct","resolution_rule":"own_instrument_marker","schema_version":"0.1.0","source_span":span}).to_string()]).unwrap();
        if n == 0 {
            c.execute("INSERT INTO reference_qualifiers VALUES(?1,0,?2)", params![id, json!({"qualifier_type":"fraction","text":"fracción I","start_char":74,"end_char":84}).to_string()]).unwrap();
        }
    }
    input(&c, "adapters/shared/_known-structural-debt.json", b"[]");
    // Raw space in the adapter, percent-encoded in the retained manifest.
    input(
        &c,
        "adapters/test/ley-a.json",
        json!({"annex_pdf_urls": ["https://example.invalid/ANEXO 1.pdf"]})
            .to_string()
            .as_bytes(),
    );
    for (n, (owner, printed, container)) in [
        (A, "9 BIS", format!("{A}:article:1")),
        (B, "8 BIS", format!("{B}:article:7")),
    ]
    .into_iter()
    .enumerate()
    {
        c.execute("INSERT INTO known_structural_debt VALUES(?1,?2,'adapters/shared/_known-structural-debt.json',?3,?4,?5)",
            params![owner, printed, format!("/rows/{n}"), i64::try_from(n).unwrap(), json!({"containing_provision_id":container,"start_char":3,"end_char":9,"reviewer":"operator","decision_reference":"docs/private.md","source_blob_oid":"f".repeat(40)}).to_string()]).unwrap();
    }
    input(&c, "corpus/mx/ley-a/validation.json", b"{}");
    c.execute("INSERT INTO coverage_accepted_gaps VALUES('ley-a','corpus/mx/ley-a/validation.json','',0,?1)",
        [json!({"reason":"private rationale","errors":[format!("target does not exist: {A}:article:99"), format!("target does not exist: {A}:article:99")]}).to_string()]).unwrap();
    c.execute(
        "INSERT INTO provisions_fts(provisions_fts) VALUES('rebuild')",
        [],
    )
    .unwrap();
}

fn release_source(public: &Value) -> Value {
    let mut source = public.clone();
    source["pdf_path"] = Value::Null;
    source["pdf_verified"] = json!(false);
    source["last_successful_upstream_check"] = Value::Null;
    source
}

/// The public release document for `ley-a`, projected as the release producer does.
fn release_doc(c: &Connection) -> Value {
    let main = release_source(&public_source(
        &input_json(c, "corpus/mx/ley-a/source-manifest.json").unwrap(),
    ));
    let annex = release_source(&public_source(
        &input_json(c, "corpus/mx/ley-a/annex-source-manifests.json").unwrap()[0],
    ));
    let mut stmt = c
        .prepare("SELECT id,extras,text FROM provisions WHERE instrument_id=?1 ORDER BY ordinal")
        .unwrap();
    let rows = stmt
        .query_map([A], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })
        .unwrap()
        .collect::<rusqlite::Result<Vec<_>>>()
        .unwrap();
    let published = rows
        .iter()
        .map(|row| row.0.clone())
        .collect::<BTreeSet<_>>();
    let provisions = rows
        .iter()
        .map(|(id, extras, text)| {
            let (provision, mut fields) = provision_fields(id, A, extras, text).unwrap();
            let stem = format!("provision-{}", crate::digest(id.as_bytes()));
            fields["markdown_path"] = json!(format!("{stem}.md"));
            fields["text_path"] = json!(format!("{stem}.txt"));
            fields["pdf_page"] = Value::Null;
            fields["source"] = if provision.provision_type == lex_core::ProvisionType::Annex {
                annex.clone()
            } else {
                main.clone()
            };
            fields
        })
        .collect::<Vec<_>>();
    let references = stored_edges(
        c,
        &format!("{EDGE_SQL} WHERE a.instrument_id=?1 ORDER BY e.id"),
        A,
    )
    .unwrap()
    .iter()
    .map(|edge| {
        let mut value = edge_core(edge).unwrap();
        value["source_provision_id"] = json!(edge[1]);
        value["target_published"] = json!(published.contains(&edge[4]));
        value
    })
    .collect::<Vec<_>>();
    json!({"schema_version":"public-instrument-v1","slug":"ley-a","id":A,"official_title":"Ley ley-a",
        "short_name":"LEY-A","publication_date":"2020-01-01","latest_reform_date":null,"source":main,
        "provisions":provisions,"references":references,"temporal_effects_included":false})
}

/// Write a self-consistent release around `doc`: every artifact, file hash and
/// the release ID are recomputed, as a repacked release would be.
fn write_release(dir: &Path, c: &Connection, doc: &Value) -> Value {
    let mut files = BTreeMap::new();
    let mut save = |path: String, bytes: Vec<u8>| {
        let full = dir.join(&path);
        fs::create_dir_all(full.parent().unwrap()).unwrap();
        fs::write(&full, &bytes).unwrap();
        files.insert(path, bytes);
    };
    for provision in doc["provisions"].as_array().unwrap() {
        let text = provision["text"].as_str().unwrap();
        save(
            format!("mx/ley-a/{}", provision["text_path"].as_str().unwrap()),
            text.as_bytes().to_vec(),
        );
        save(
            format!("mx/ley-a/{}", provision["markdown_path"].as_str().unwrap()),
            format!("# {}\n\n{text}\n", provision["label"]).into_bytes(),
        );
    }
    save(
        "mx/ley-a/instrument.json".into(),
        serde_json::to_vec_pretty(doc).unwrap(),
    );
    let mut hashes = BTreeMap::new();
    for name in [
        "instrument.json",
        "provisions.json",
        "references.json",
        "source-manifest.json",
        "annex-source-manifests.json",
        "validation.json",
    ] {
        let sha: String = c
            .query_row(
                "SELECT sha256 FROM input_files WHERE path=?1",
                [format!("corpus/mx/ley-a/{name}")],
                |r| r.get(0),
            )
            .unwrap();
        hashes.insert(name, sha);
    }
    let mut manifest = json!({"schema_version":"public-release-v1","builder_sha256":"0".repeat(64),
        "corpus_commit":COMMIT,"committed_at":"2026-01-01T00:00:00Z","base_path":"/lex-mex/","coverage":"selected_partial",
        "instruments":[{"slug":"ley-a","id":A,"title":doc["official_title"],"short_name":doc["short_name"],
            "provision_count":doc["provisions"].as_array().unwrap().len(),"document_path":"mx/ley-a/instrument.json",
            "source":doc["source"],"canonical_input_hashes":hashes,"coverage":serde_json::from_str::<Value>(COVERAGE).unwrap()}],
        "files": files.iter().map(|(p, b)| json!({"path":p,"sha256":crate::digest(b),"bytes":b.len()})).collect::<Vec<_>>()});
    let mut bytes = serde_json::to_vec_pretty(&manifest).unwrap();
    bytes.push(b'\n');
    manifest["release_id"] = json!(crate::digest(&bytes));
    fs::write(
        dir.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
    manifest
}

fn db_manifest(database: &Path) -> DatabaseManifest {
    DatabaseManifest {
        schema_version: "2".into(),
        release_id: "e".repeat(64),
        specification: BuildSpecification {
            schema_version: "2".into(),
            corpus_commit: COMMIT.into(),
            builder_commit: COMMIT.into(),
            cargo_lock_sha256: "0".repeat(64),
            rust_toolchain: "test".into(),
            sqlite_version: "test".into(),
            sqlite_compile_options: vec![],
            source_commit_timestamp: 0,
            inclusion_policy: "test".into(),
            logical_digest_policy: "test".into(),
            inputs: vec![],
        },
        database_sha256: file_sha256(database).unwrap(),
        logical_content_sha256: "d".repeat(64),
    }
}

fn fixture() -> Fixture {
    fixture_with(|_| {})
}

/// As [`fixture`], with `edit` applied to the built database before the
/// release is written and bound.
fn fixture_with(edit: impl FnOnce(&Path)) -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let database = dir.path().join("corpus.sqlite");
    build_database(&database);
    edit(&database);
    let release_dir = dir.path().join("release");
    let c = open_readonly(&database).unwrap();
    let manifest = write_release(&release_dir, &c, &release_doc(&c));
    drop(c);
    let binding = bind_manifest(
        db_manifest(&database),
        &"9".repeat(64),
        &database,
        &release_dir,
    )
    .unwrap();
    let binding_path = dir.path().join("binding.json");
    write_binding(&binding, &binding_path).unwrap();
    Fixture {
        database,
        release: release_dir,
        manifest,
        binding_path,
        _dir: dir,
    }
}

fn reader(f: &Fixture, limits: Limits) -> PublicReader {
    PublicReader::open(&f.database, &f.binding_path, limits).unwrap()
}

fn ask(r: &PublicReader, request: &Value) -> Value {
    r.handle(&request.to_string())
}

fn code(response: &Value) -> &str {
    response["result"]["code"].as_str().unwrap_or("")
}

#[test]
fn binding_is_exact_and_refuses_overwrite() {
    let f = fixture();
    let binding: Binding = serde_json::from_slice(&fs::read(&f.binding_path).unwrap()).unwrap();
    assert_eq!(binding.public_release.release_id, f.manifest["release_id"]);
    assert_eq!(binding.instruments[0].provision_count, 6);
    assert_eq!(
        binding.instruments[0].reference_count, 4,
        "title-anchored edge included"
    );
    assert_eq!(binding.instruments[0].annex_sources["1"], "b".repeat(64));
    assert!(write_binding(&binding, &f.binding_path).is_err());
}

#[test]
fn binding_fails_closed_on_mismatched_release_or_database() {
    let f = fixture();
    let txt = f.release.join(format!(
        "mx/ley-a/provision-{}.txt",
        crate::digest(format!("{A}:article:1").as_bytes())
    ));
    let original = fs::read(&txt).unwrap();
    fs::write(&txt, b"tampered").unwrap();
    let err = bind_manifest(
        db_manifest(&f.database),
        &"9".repeat(64),
        &f.database,
        &f.release,
    )
    .unwrap_err();
    assert!(err.to_string().contains("differs"), "{err}");
    fs::write(&txt, original).unwrap();
    // A relabelled manifest ID is rejected rather than trusted.
    let mut manifest = f.manifest.clone();
    manifest["corpus_commit"] = json!("2".repeat(40));
    fs::write(
        f.release.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
    assert!(
        bind_manifest(
            db_manifest(&f.database),
            &"9".repeat(64),
            &f.database,
            &f.release
        )
        .is_err()
    );
    // Any byte change to the served database fails closed at open.
    let mut binding: Value = serde_json::from_slice(&fs::read(&f.binding_path).unwrap()).unwrap();
    binding["database"]["database_sha256"] = json!("0".repeat(64));
    fs::write(&f.binding_path, binding.to_string()).unwrap();
    assert!(PublicReader::open(&f.database, &f.binding_path, Limits::default()).is_err());
    binding["database"]["database_sha256"] = json!(file_sha256(&f.database).unwrap());
    let digest = binding["instruments"][0]["provisions_sha256"].clone();
    binding["instruments"][0]["provisions_sha256"] = json!("0".repeat(64));
    fs::write(&f.binding_path, binding.to_string()).unwrap();
    assert!(PublicReader::open(&f.database, &f.binding_path, Limits::default()).is_err());
    binding["instruments"][0]["provisions_sha256"] = digest;
    fs::write(&f.binding_path, binding.to_string()).unwrap();
    assert!(PublicReader::open(&f.database, &f.binding_path, Limits::default()).is_ok());
    binding["unexpected"] = json!(true);
    fs::write(&f.binding_path, binding.to_string()).unwrap();
    assert!(PublicReader::open(&f.database, &f.binding_path, Limits::default()).is_err());
}

type Mutation = Box<dyn Fn(&mut Value)>;

fn rebind(f: &Fixture, mutate: impl Fn(&mut Value)) -> Result<Binding> {
    let c = open_readonly(&f.database).unwrap();
    let mut doc = release_doc(&c);
    mutate(&mut doc);
    let dir = tempfile::tempdir().unwrap();
    write_release(dir.path(), &c, &doc);
    drop(c);
    bind_manifest(
        db_manifest(&f.database),
        &"9".repeat(64),
        &f.database,
        dir.path(),
    )
}

#[test]
#[allow(clippy::too_many_lines)]
fn binding_rejects_rehashed_releases_with_changed_public_projection() {
    let f = fixture();
    assert!(rebind(&f, |_| {}).is_ok(), "unchanged release binds");
    let edge = |doc: &Value, source: &str| {
        doc["references"]
            .as_array()
            .unwrap()
            .iter()
            .position(|e| e["source_provision_id"] == source)
            .unwrap()
    };
    let mutations: Vec<(&str, Mutation)> = vec![
        (
            "span",
            Box::new(|d: &mut Value| {
                d["references"][0]["source_span"] = json!("D3 SYNTHETIC CHANGED SPAN");
            }),
        ),
        (
            "target",
            Box::new(|d: &mut Value| {
                d["references"][0]["target_provision_id"] = json!(format!("{A}:article:3"));
            }),
        ),
        (
            "status",
            Box::new(|d: &mut Value| d["references"][0]["resolution_status"] = json!("unresolved")),
        ),
        (
            "published flag",
            Box::new(|d: &mut Value| d["references"][0]["target_published"] = json!(false)),
        ),
        (
            "title-anchor offset",
            Box::new(move |d: &mut Value| {
                let i = edge(d, A);
                d["references"][i]["start_char"] = json!(5);
            }),
        ),
        (
            "duplicate edge in place of another",
            Box::new(|d: &mut Value| {
                d["references"][1] = d["references"][0].clone();
            }),
        ),
        (
            "dropped edge",
            Box::new(|d: &mut Value| {
                d["references"].as_array_mut().unwrap().pop();
            }),
        ),
        (
            "label",
            Box::new(|d: &mut Value| d["provisions"][0]["label"] = json!("Artículo 10")),
        ),
        (
            "temporal status",
            Box::new(|d: &mut Value| d["provisions"][0]["temporal_status"] = json!("repealed")),
        ),
        (
            "heading",
            Box::new(|d: &mut Value| {
                d["provisions"][0]["heading_context"]["title"] = json!("Título II");
            }),
        ),
        (
            "text and hash",
            Box::new(|d: &mut Value| {
                d["provisions"][1]["text"] = json!("Texto alterado.");
                d["provisions"][1]["text_sha256"] = json!(crate::digest(b"Texto alterado."));
            }),
        ),
        (
            "official source URL",
            Box::new(|d: &mut Value| {
                let url = json!("https://example.invalid/other.pdf");
                d["source"]["official_url"] = url.clone();
                for p in d["provisions"].as_array_mut().unwrap() {
                    if p["provision_type"] != "annex" {
                        p["source"]["official_url"] = url.clone();
                    }
                }
            }),
        ),
        (
            "annex source publisher",
            Box::new(|d: &mut Value| d["provisions"][5]["source"]["publisher"] = json!("Otro")),
        ),
        (
            "annex given main source",
            Box::new(|d: &mut Value| d["provisions"][5]["source"] = d["source"].clone()),
        ),
        (
            "instrument title",
            Box::new(|d: &mut Value| d["official_title"] = json!("Otra ley")),
        ),
        (
            "schema",
            Box::new(|d: &mut Value| d["provisions"][0]["operator_note"] = json!("private")),
        ),
    ];
    for (name, mutate) in mutations {
        let error = format!("{:#}", rebind(&f, mutate).expect_err(name));
        let expected = match name {
            "schema" => "instrument schema",
            "duplicate edge in place of another" => "repeats reference",
            _ => "differ",
        };
        assert!(error.contains(expected), "{name}: {error}");
    }
}

#[test]
fn annex_urls_compare_as_parsed_urls() {
    assert!(same_url(
        "https://example.invalid/ANEXO 1.pdf",
        "https://example.invalid/ANEXO%201.pdf"
    ));
    assert!(!same_url(
        "https://example.invalid/a%2Fb.pdf",
        "https://example.invalid/a/b.pdf"
    ));
    assert!(!same_url("not a url", "not a url"));
}

#[test]
fn resolves_exact_bis_transitory_and_annex_without_choosing_duplicates() {
    let f = fixture();
    let r = reader(&f, Limits::default());
    let found = ask(
        &r,
        &json!({"operation":"resolve","instrument":"ley-a","kind":"article","locator":"2 bis"}),
    );
    assert_eq!(found["outcome"], "found");
    assert_eq!(
        found["result"]["candidates"][0]["id"],
        format!("{A}:article:2-bis")
    );
    let ambiguous = ask(
        &r,
        &json!({"operation":"resolve","instrument":A,"kind":"article","locator":"3"}),
    );
    assert_eq!(ambiguous["outcome"], "ambiguous");
    assert_eq!(ambiguous["result"]["candidate_count"], 2);
    for (kind, locator) in [("transitory", "Primero"), ("annex", "1")] {
        let response = ask(
            &r,
            &json!({"operation":"resolve","instrument":"ley-a","kind":kind,"locator":locator}),
        );
        assert_eq!(response["outcome"], "found", "{response}");
    }
    let wrong_kind = ask(
        &r,
        &json!({"operation":"resolve","instrument":"ley-a","kind":"transitory","locator":"1"}),
    );
    assert_eq!(wrong_kind["outcome"], "not_found");
    let unknown = ask(
        &r,
        &json!({"operation":"resolve","instrument":"ley-b","kind":"article","locator":"7"}),
    );
    assert_eq!(code(&unknown), "unknown_instrument");
    let alias = ask(
        &r,
        &json!({"operation":"resolve","instrument":"ley-a","kind":"article","locator":"́"}),
    );
    assert_eq!(code(&alias), "invalid_locator");
}

#[test]
fn absence_distinctions_are_public_safe_and_scoped() {
    let f = fixture();
    let r = reader(&f, Limits::default());
    let debt = ask(
        &r,
        &json!({"operation":"resolve","instrument":"ley-a","kind":"article","locator":"9 Bis"}),
    );
    assert_eq!(debt["outcome"], "not_found");
    assert_eq!(
        debt["result"]["qualifications"][0]["kind"],
        "not_separately_parsed"
    );
    assert_eq!(
        debt["result"]["qualifications"][0]["containing_provision_id"],
        format!("{A}:article:1")
    );
    let cited = ask(
        &r,
        &json!({"operation":"get_provision","id":format!("{A}:article:99")}),
    );
    assert_eq!(
        cited["result"]["qualifications"],
        json!([{"kind":"cited_not_in_corpus","citing_instrument_id":A,"multiplicity":2}])
    );
    let plain = ask(
        &r,
        &json!({"operation":"get_provision","id":format!("{A}:article:42")}),
    );
    assert_eq!(plain["result"]["qualifications"], json!([]));
    assert_eq!(plain["result"]["coverage"], "not_established");
    // Out-of-scope rows exist in the database but are never disclosed.
    for id in [
        format!("{B}:article:7"),
        format!("{B}:article:8-bis"),
        "urn:lex-mx:federal:statute:ley-a-extra:article:1".to_owned(),
    ] {
        let response = ask(&r, &json!({"operation":"get_provision","id":id}));
        assert_eq!(response["outcome"], "outside_published_scope");
        assert!(!response.to_string().contains("Ley B."));
    }
    for response in [debt, cited, plain] {
        let text = response.to_string();
        for private in [
            "reviewer",
            "decision_reference",
            "source_blob_oid",
            "private",
            "input_path",
            "source_pointer",
            "adapters/",
        ] {
            assert!(!text.contains(private), "{private} leaked: {text}");
        }
    }
}

#[test]
fn provision_is_exact_with_its_own_source_and_never_truncated() {
    let f = fixture();
    let r = reader(&f, Limits::default());
    let found = ask(
        &r,
        &json!({"operation":"get_provision","id":format!("{A}:article:1")}),
    );
    assert_eq!(found["outcome"], "found");
    let p = &found["result"]["provision"];
    assert_eq!(p["text"], ART1, "CRLF and accents preserved");
    assert_eq!(p["text_sha256"], crate::digest(ART1.as_bytes()));
    assert_eq!(
        p["source"]["official_url"], "http://example.invalid/a.pdf",
        "recorded URL kept verbatim"
    );
    assert!(p["source"].get("reviewer").is_none());
    let annex = ask(
        &r,
        &json!({"operation":"get_provision","id":format!("{A}:annex:1")}),
    );
    assert_eq!(
        annex["result"]["provision"]["source"]["source_sha256"],
        "b".repeat(64)
    );
    let small = reader(
        &f,
        Limits {
            provision_text_bytes: 500,
            ..Limits::default()
        },
    );
    let oversized = ask(
        &small,
        &json!({"operation":"get_provision","id":format!("{A}:annex:1")}),
    );
    assert_eq!(oversized["outcome"], "oversized");
    assert!(oversized["result"].get("text").is_none());
    let artifact = oversized["result"]["artifacts"]["text_path"]
        .as_str()
        .unwrap();
    let local = artifact.split_once("/mx/").unwrap().1;
    assert_eq!(
        crate::digest(&fs::read(f.release.join("mx").join(local)).unwrap()),
        oversized["result"]["text_sha256"]
    );
    let as_of = ask(
        &r,
        &json!({"operation":"get_provision","id":format!("{A}:article:1"),"as_of":"2021-01-01"}),
    );
    assert_eq!(code(&as_of), "as_of_not_supported");
}

#[test]
fn search_is_scoped_literal_bounded_and_rejects_empty_normalized_input() {
    let f = fixture();
    let r = reader(&f, Limits::default());
    let hits = ask(&r, &json!({"operation":"search","query":"AUTORIDAD"}));
    assert_eq!(hits["outcome"], "ok");
    let ids = hits["result"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["id"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(ids.len(), 2, "{hits}");
    assert!(
        ids.iter().all(|id| id.starts_with(A)),
        "out-of-scope hit leaked"
    );
    let accent = ask(
        &r,
        &json!({"operation":"search","query":"ambito de aplicacion"}),
    );
    let excerpt = &accent["result"]["items"][0]["excerpt"];
    assert_eq!(accent["result"]["items"][0]["match_location"], "text");
    assert!(
        excerpt["text"]
            .as_str()
            .unwrap()
            .contains("ámbito de aplicación")
    );
    assert_eq!(excerpt["partial"], false);
    let phrase = ask(&r, &json!({"operation":"search","query":"autoridad niño"}));
    assert_eq!(
        phrase["result"]["items"],
        json!([]),
        "phrase never becomes OR"
    );
    for (query, expected) in [
        ("\u{301}", "empty_query"),
        ("  ", "empty_query"),
        ("--", "empty_query"),
        ("a\u{0}b", "invalid_query"),
    ] {
        assert_eq!(
            code(&ask(&r, &json!({"operation":"search","query":query}))),
            expected,
            "{query:?}"
        );
    }
    assert_eq!(
        code(&ask(
            &r,
            &json!({"operation":"search","query":"a".repeat(513)})
        )),
        "query_too_long"
    );
    assert_eq!(
        code(&ask(
            &r,
            &json!({"operation":"search","query":"á".repeat(1100)})
        )),
        "query_too_long"
    );
    assert_eq!(
        code(&ask(
            &r,
            &json!({"operation":"search","query":"x","instrument":"ley-b"})
        )),
        "unknown_instrument"
    );
    assert_eq!(
        code(&ask(
            &r,
            &json!({"operation":"search","query":"x","page_size":11})
        )),
        "bad_request"
    );
    let excerpt_width = reader(
        &f,
        Limits {
            excerpt_chars: 20,
            ..Limits::default()
        },
    );
    let narrow = ask(
        &excerpt_width,
        &json!({"operation":"search","query":"segunda línea"}),
    );
    let window = &narrow["result"]["items"][0]["excerpt"];
    assert_eq!(window["text"].as_str().unwrap().chars().count(), 20);
    assert_eq!(window["partial"], true);
}

#[test]
fn cursors_are_stable_bound_and_reject_malformed_input() {
    let f = fixture();
    let r = reader(&f, Limits::default());
    let request = json!({"operation":"search","query":"artículo","page_size":1});
    let first = ask(&r, &request);
    let cursor = first["result"]["next_cursor"].as_str().unwrap().to_owned();
    let mut seen = vec![first["result"]["items"][0]["id"].clone()];
    let mut next = Some(cursor.clone());
    while let Some(c) = next {
        let mut page = request.clone();
        page["cursor"] = json!(c);
        let response = ask(&r, &page);
        assert_eq!(response["outcome"], "ok", "{response}");
        seen.extend(
            response["result"]["items"]
                .as_array()
                .unwrap()
                .iter()
                .cloned(),
        );
        next = response["result"]["next_cursor"]
            .as_str()
            .map(str::to_owned);
    }
    let unique = seen
        .iter()
        .map(|v| {
            v.as_str()
                .unwrap_or_else(|| v["id"].as_str().unwrap())
                .to_owned()
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(unique.len(), seen.len(), "pages overlap");
    assert_eq!(ask(&r, &request), first, "deterministic");
    let mut other = request.clone();
    other["query"] = json!("autoridad");
    other["cursor"] = json!(cursor);
    assert_eq!(code(&ask(&r, &other)), "cursor_request_mismatch");
    let mut size = request.clone();
    size["page_size"] = json!(2);
    size["cursor"] = json!(cursor);
    assert_eq!(code(&ask(&r, &size)), "cursor_request_mismatch");
    let forged = base64url(
        &serde_json::to_vec(
            &json!({"v":1,"op":"search","snapshot":"0".repeat(64),"request":"x","after":["1","x"]}),
        )
        .unwrap(),
    );
    let mut stale = request.clone();
    stale["cursor"] = json!(forged);
    assert_eq!(code(&ask(&r, &stale)), "cursor_release_mismatch");
    for bad in [
        base64url(b"null"),
        base64url(b"7"),
        base64url(b"[]"),
        "not base64!".into(),
        "A".into(),
        format!("{cursor}A"),
        String::new(),
    ] {
        let mut page = request.clone();
        page["cursor"] = json!(bad);
        assert_eq!(code(&ask(&r, &page)), "invalid_cursor", "{bad:?}");
    }
    let mut null_cursor = request.clone();
    null_cursor["cursor"] = Value::Null;
    assert_eq!(ask(&r, &null_cursor), first, "JSON null means no cursor");
}

#[test]
fn references_include_title_edges_and_disclose_availability_only() {
    let f = fixture();
    let r = reader(&f, Limits::default());
    let out = ask(
        &r,
        &json!({"operation":"references","id":format!("{A}:article:1")}),
    );
    let items = out["result"]["items"].as_array().unwrap();
    let availability = items
        .iter()
        .map(|e| e["target_availability"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        availability,
        ["published", "outside_published_scope", "unresolved"]
    );
    assert_eq!(items[0]["qualifiers"][0]["text"], "fracción I");
    let title = ask(&r, &json!({"operation":"references","id":A}));
    assert_eq!(
        title["result"]["items"][0]["source_kind"],
        "instrument_title"
    );
    let incoming = ask(
        &r,
        &json!({"operation":"references","id":format!("{A}:article:1"),"direction":"incoming"}),
    );
    let sources = incoming["result"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["source_id"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        sources,
        [A],
        "incoming edge from unpublished ley-b withheld"
    );
    let outside = ask(
        &r,
        &json!({"operation":"references","id":format!("{B}:article:7"),"direction":"outgoing"}),
    );
    assert_eq!(outside["outcome"], "outside_published_scope");
    let page = ask(
        &r,
        &json!({"operation":"references","id":format!("{A}:article:1"),"page_size":2}),
    );
    let mut rest = json!({"operation":"references","id":format!("{A}:article:1"),"page_size":2});
    rest["cursor"] = page["result"]["next_cursor"].clone();
    assert_eq!(
        ask(&r, &rest)["result"]["items"].as_array().unwrap().len(),
        1
    );
}

#[test]
fn envelope_coverage_release_pins_and_byte_budget() {
    let f = fixture();
    let r = reader(&f, Limits::default());
    let coverage = ask(&r, &json!({"operation":"coverage"}));
    assert_eq!(coverage["result"]["items"].as_array().unwrap().len(), 1);
    assert_eq!(
        coverage["snapshot"]["public_release_id"],
        f.manifest["release_id"]
    );
    let pinned = ask(
        &r,
        &json!({"operation":"coverage","release_id":f.manifest["release_id"]}),
    );
    assert_eq!(pinned["outcome"], "ok");
    let old = ask(
        &r,
        &json!({"operation":"get_provision","id":format!("{A}:article:1"),"release_id":"0".repeat(64)}),
    );
    assert_eq!(old["outcome"], "release_unavailable");
    assert!(!old.to_string().contains("supervisará"));
    for bad in [
        "{",
        "null",
        "[]",
        r#"{"operation":"drop"}"#,
        r#"{"operation":"coverage","sql":"x"}"#,
    ] {
        let response = r.handle(bad);
        assert_eq!(response["outcome"], "client_error", "{bad}");
        assert_eq!(code(&response), "bad_request");
    }
    let tight = reader(
        &f,
        Limits {
            response_bytes: 3500,
            ..Limits::default()
        },
    );
    let shortened = ask(&tight, &json!({"operation":"search","query":"artículo"}));
    let count = shortened["result"]["items"].as_array().unwrap().len();
    assert!(
        (1..4).contains(&count) && shortened["result"]["next_cursor"].is_string(),
        "{shortened}"
    );
    assert!(shortened.to_string().len() <= 3500);
}

#[test]
fn deadline_interrupts_and_reports_unavailable() {
    let f = fixture();
    let r = reader(
        &f,
        Limits {
            deadline: Duration::ZERO,
            ..Limits::default()
        },
    );
    let response = ask(&r, &json!({"operation":"search","query":"autoridad"}));
    assert_eq!(response["outcome"], "unavailable", "{response}");
    assert_eq!(code(&response), "deadline_exceeded");
}

#[test]
fn base64url_round_trips_and_is_canonical() {
    for bytes in [&b""[..], b"f", b"fo", b"foo", b"foob", &[255, 0, 128]] {
        assert_eq!(unbase64url(&base64url(bytes)).unwrap(), bytes);
    }
    assert!(unbase64url("Zh").is_none(), "non-canonical trailing bits");
}

#[test]
fn an_oversized_item_returns_a_skip_cursor_and_traversal_continues() {
    // Codex audit 2026-11-17, R1: a row larger than the response limit used
    // to end traversal with no way past it at any page size.
    let f = fixture();
    let r = reader(
        &f,
        Limits {
            response_bytes: 1500,
            ..Limits::default()
        },
    );
    let request = json!({"operation":"search","query":"artículo","page_size":1});
    let mut response = ask(&r, &request);
    let (mut oversized, mut pages, mut cursors) = (0, 0, Vec::new());
    loop {
        pages += 1;
        assert!(pages < 50, "traversal does not terminate");
        let next = if code(&response) == "response_too_large" {
            oversized += 1;
            let skip = response["result"]["skip_cursor"].as_str();
            assert!(skip.is_some(), "no way past the oversized item: {response}");
            skip.map(str::to_owned)
        } else {
            assert_eq!(response["outcome"], "ok", "{response}");
            response["result"]["next_cursor"]
                .as_str()
                .map(str::to_owned)
        };
        let Some(cursor) = next else { break };
        assert!(!cursors.contains(&cursor), "a cursor repeated: no progress");
        cursors.push(cursor.clone());
        let mut page = request.clone();
        page["cursor"] = json!(cursor);
        response = ask(&r, &page);
    }
    assert!(oversized > 0, "the limit did not produce an oversized item");
}

const V1_SCHEMA: &str = include_str!("../../../schemas/public-query-response-v1.schema.json");
const V2_SCHEMA: &str = include_str!("../../../schemas/public-query-response-v2.schema.json");

fn accepts(schema: &str, response: &Value) -> bool {
    let schema: Value = serde_json::from_str(schema).unwrap();
    jsonschema::validator_for(&schema)
        .unwrap()
        .is_valid(response)
}

#[test]
fn the_skip_cursor_response_is_a_new_wire_version_and_v1_stays_frozen() {
    // Codex audit re-review 2026-11-17, F3: the response with a skip_cursor
    // validated against the changed schema but not against the accepted
    // closed v1, while still calling itself v1.
    assert_eq!(RESPONSE_VERSION, "public-query-response-v2");
    // The v1 file is the accepted 2026-10-25 contract, byte for byte.
    assert_eq!(
        format!("{:x}", Sha256::digest(V1_SCHEMA.as_bytes())),
        "636536a8e58f9855fa7812278944fb69c6e6192be0f4bacaf7e9f104d591f559"
    );
    // The query projection is not a response version and did not change.
    assert_eq!(PROJECTION_VERSION, "public-query-v1");

    let f = fixture();
    let r = reader(
        &f,
        Limits {
            response_bytes: 1500,
            ..Limits::default()
        },
    );
    let oversized = ask(
        &r,
        &json!({"operation":"search","query":"artículo","page_size":1}),
    );
    assert_eq!(code(&oversized), "response_too_large", "{oversized}");
    assert!(oversized["result"]["skip_cursor"].is_string());
    assert_eq!(oversized["schema_version"], "public-query-response-v2");
    assert_eq!(
        oversized["snapshot"]["projection_version"],
        "public-query-v1"
    );
    assert!(
        accepts(V2_SCHEMA, &oversized),
        "v2 rejects its own response"
    );
    assert!(
        !accepts(V1_SCHEMA, &oversized),
        "the closed v1 must not accept a v2 response"
    );

    // An ordinary response also names v2 and is not mistaken for v1.
    let coverage = ask(&r, &json!({"operation":"coverage"}));
    assert_eq!(coverage["schema_version"], "public-query-response-v2");
    assert!(accepts(V2_SCHEMA, &coverage));
    assert!(!accepts(V1_SCHEMA, &coverage));
}

#[test]
fn skip_cursor_is_structurally_confined_to_response_too_large() {
    let f = fixture();
    let r = reader(
        &f,
        Limits {
            response_bytes: 1500,
            ..Limits::default()
        },
    );
    let valid = ask(
        &r,
        &json!({"operation":"search","query":"artículo","page_size":1}),
    );
    assert!(accepts(V2_SCHEMA, &valid));
    // The same body under any other error code is invalid.
    let others = [
        "bad_request",
        "unknown_instrument",
        "invalid_locator",
        "invalid_query",
        "empty_query",
        "query_too_long",
        "invalid_cursor",
        "cursor_release_mismatch",
        "cursor_request_mismatch",
        "as_of_not_supported",
        "deadline_exceeded",
        "internal_error",
        "contract_violation",
    ];
    for other in others {
        let mut moved = valid.clone();
        moved["result"]["code"] = json!(other);
        assert!(
            !accepts(V2_SCHEMA, &moved),
            "skip_cursor accepted with {other}"
        );
        // Without the field the same code stays valid: only the pairing is rejected.
        moved["result"]
            .as_object_mut()
            .unwrap()
            .remove("skip_cursor");
        assert!(accepts(V2_SCHEMA, &moved), "{other} without a skip_cursor");
    }
    // response_too_large without a skip_cursor remains valid (an empty page
    // cannot be skipped), and an empty skip_cursor is not.
    let mut bare = valid.clone();
    bare["result"]
        .as_object_mut()
        .unwrap()
        .remove("skip_cursor");
    assert!(accepts(V2_SCHEMA, &bare));
    let mut empty = valid.clone();
    empty["result"]["skip_cursor"] = json!("");
    assert!(!accepts(V2_SCHEMA, &empty));
}

#[test]
fn work_after_sqlite_still_counts_against_the_deadline() {
    // Codex audit 2026-11-17, R4: excerpts and serialization run outside the
    // SQLite progress handler, so the whole request is checked at the end.
    let f = fixture();
    let r = reader(
        &f,
        Limits {
            deadline: Duration::ZERO,
            ..Limits::default()
        },
    );
    for request in [
        json!({"operation":"coverage"}),
        json!({"operation":"get_provision","id":format!("{A}:article:1")}),
        json!({"operation":"search","query":"autoridad"}),
    ] {
        let response = ask(&r, &request);
        assert_eq!(
            code(&response),
            "deadline_exceeded",
            "{request}: {response}"
        );
    }
}

#[test]
fn a_live_sidecar_is_found_whatever_the_database_extension() {
    // Codex audit 2026-11-17, R5: `lex.db-wal` was missed because the check
    // replaced the extension instead of appending to the file name.
    let f = fixture();
    let renamed = f.database.with_file_name("lex.db");
    fs::copy(&f.database, &renamed).unwrap();
    assert!(PublicReader::open(&renamed, &f.binding_path, Limits::default()).is_ok());
    fs::write(renamed.with_file_name("lex.db-wal"), b"").unwrap();
    let error = PublicReader::open(&renamed, &f.binding_path, Limits::default())
        .err()
        .expect("a live WAL must be refused")
        .to_string();
    assert!(error.contains("sidecar"), "{error}");
    assert_eq!(
        crate::sqlite_sidecars(Path::new("/x/corpus.sqlite"))[2],
        Path::new("/x/corpus.sqlite-journal")
    );
}

#[test]
fn a_bound_annex_opens_when_another_manifest_shares_its_content() {
    // Codex audit 2026-11-17, R6: binding selects by configured URL, opening
    // selected by digest alone, so shared content made a valid bind unopenable.
    let f = fixture_with(|database| {
        let c = Connection::open(database).unwrap();
        let path = "corpus/mx/ley-a/annex-source-manifests.json";
        let bytes = json!([
            source(&"b".repeat(64), "https://example.invalid/ANEXO%201.pdf"),
            source(&"b".repeat(64), "https://example.invalid/OTRO.pdf"),
        ])
        .to_string();
        c.execute(
            "UPDATE input_files SET bytes=?1, sha256=?2 WHERE path=?3",
            params![bytes.as_bytes(), crate::digest(bytes.as_bytes()), path],
        )
        .unwrap();
    });
    let r = reader(&f, Limits::default());
    let annex = ask(
        &r,
        &json!({"operation":"get_provision","id":format!("{A}:annex:1")}),
    );
    assert_eq!(annex["outcome"], "found", "{annex}");
    assert_eq!(
        annex["result"]["provision"]["source"]["official_url"],
        "https://example.invalid/ANEXO%201.pdf"
    );
}
