//! Acceptance over the real accepted database and public release; run with
//! `LEX_PUBLIC_DATABASE`, `LEX_PUBLIC_BINDING` and `LEX_PUBLIC_RELEASE` set and
//! `cargo test --release -p lex-db --test public_release -- --ignored`.
use lex_db::public::{Limits, PublicReader};
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, path::PathBuf};

fn env(name: &str) -> PathBuf {
    std::env::var_os(name).map_or_else(
        || panic!("{name} must name an accepted artifact"),
        PathBuf::from,
    )
}

fn ask(reader: &PublicReader, request: &Value) -> Value {
    let response = reader.handle(&request.to_string());
    let size = response.to_string().len();
    assert!(
        response["result"]["kind"] == "provision" || size <= 16 * 1024,
        "{size} bytes"
    );
    response
}

const PROVISION_FIELDS: [&str; 15] = [
    "id",
    "label",
    "number",
    "provision_type",
    "heading_context",
    "text",
    "text_sha256",
    "publication_date",
    "effective_from",
    "effective_to",
    "temporal_status",
    "temporal_basis",
    "review_status",
    "repeals",
    "commencement_condition",
];
const SOURCE_FIELDS: [&str; 7] = [
    "source_sha256",
    "extracted_text_sha256",
    "publisher",
    "official_url",
    "retrieved_at",
    "parser_version",
    "schema_version",
];

fn edge_view(edge: &Value) -> Value {
    json!([
        edge["id"],
        edge["source_id"],
        edge["target_provision_id"],
        edge["resolution_status"],
        edge["source_span"],
        edge["start_char"],
        edge["end_char"],
        edge["reference_form"],
        edge["basis"],
        edge["target_availability"] == "published"
    ])
}

fn release_edge_view(edge: &Value) -> Value {
    json!([
        edge["id"],
        edge["source_provision_id"],
        edge["target_provision_id"],
        edge["resolution_status"],
        edge["source_span"],
        edge["start_char"],
        edge["end_char"],
        edge["reference_form"],
        edge["basis"],
        edge["target_published"] == true && edge["resolution_status"] == "resolved"
    ])
}

fn all_pages(reader: &PublicReader, mut request: Value) -> Vec<Value> {
    let mut items = Vec::new();
    loop {
        let response = ask(reader, &request);
        assert_eq!(response["outcome"], "ok", "{response}");
        items.extend(
            response["result"]["items"]
                .as_array()
                .unwrap()
                .iter()
                .cloned(),
        );
        match response["result"]["next_cursor"].as_str() {
            Some(cursor) => request["cursor"] = json!(cursor),
            None => return items,
        }
    }
}

#[test]
#[allow(clippy::too_many_lines)]
#[ignore = "requires the accepted database, its binding and the public release"]
fn served_projection_matches_the_pinned_public_release() {
    let reader = PublicReader::open(
        &env("LEX_PUBLIC_DATABASE"),
        &env("LEX_PUBLIC_BINDING"),
        Limits::default(),
    )
    .unwrap();
    let release = env("LEX_PUBLIC_RELEASE");
    let manifest: Value =
        serde_json::from_slice(&fs::read(release.join("manifest.json")).unwrap()).unwrap();
    let mut published = BTreeSet::new();
    let mut ambiguous = 0;
    for item in manifest["instruments"].as_array().unwrap() {
        let doc: Value = serde_json::from_slice(
            &fs::read(release.join(item["document_path"].as_str().unwrap())).unwrap(),
        )
        .unwrap();
        for public in doc["provisions"].as_array().unwrap() {
            let id = public["id"].as_str().unwrap();
            published.insert(id.to_owned());
            let response = ask(&reader, &json!({"operation": "get_provision", "id": id}));
            assert_eq!(response["outcome"], "found", "{id}");
            let served = &response["result"]["provision"];
            for field in PROVISION_FIELDS {
                assert_eq!(served[field], public[field], "{id}.{field}");
            }
            for field in SOURCE_FIELDS {
                assert_eq!(
                    served["source"][field], public["source"][field],
                    "{id}.source.{field}"
                );
            }
            let resolved = ask(
                &reader,
                &json!({"operation": "resolve", "instrument": item["slug"],
                "kind": public["provision_type"], "locator": public["number"]}),
            );
            let candidates = resolved["result"]["candidates"].as_array().unwrap();
            assert!(
                candidates.iter().any(|c| c["id"] == id),
                "{id} not resolvable"
            );
            if resolved["outcome"] == "ambiguous" {
                ambiguous += 1;
            }
        }
        // Every published edge, including title-anchored ones, with the same fields.
        let mut served = BTreeSet::new();
        let anchors = std::iter::once(doc["id"].clone()).chain(
            doc["provisions"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| p["id"].clone()),
        );
        for anchor in anchors {
            for edge in all_pages(&reader, json!({"operation": "references", "id": anchor})) {
                served.insert(edge_view(&edge).to_string());
            }
        }
        let expected = doc["references"]
            .as_array()
            .unwrap()
            .iter()
            .map(|edge| release_edge_view(edge).to_string())
            .collect::<BTreeSet<_>>();
        assert_eq!(served, expected, "{}", item["slug"]);
    }
    assert_eq!(published.len(), 870);
    println!("ambiguous printed locators: {ambiguous}");
    // Broad searches stay inside the published selection across every page.
    for query in [
        "artículo",
        "autoridad",
        "Comisión Nacional Bancaria",
        "ley",
        "fracción",
    ] {
        let hits = all_pages(&reader, json!({"operation": "search", "query": query}));
        let ids = hits
            .iter()
            .map(|h| h["id"].as_str().unwrap().to_owned())
            .collect::<Vec<_>>();
        assert!(
            ids.iter().all(|id| published.contains(id)),
            "{query} leaked"
        );
        assert_eq!(
            ids.iter().collect::<BTreeSet<_>>().len(),
            ids.len(),
            "{query} pages overlap"
        );
        println!("{query}: {} hits", ids.len());
    }
    for id in [
        "urn:lex-mx:federal:code:ccf:article:100",
        "urn:lex-mx:federal:statute:laero:article:73-nonies",
    ] {
        let response = ask(&reader, &json!({"operation": "get_provision", "id": id}));
        assert_eq!(response["outcome"], "outside_published_scope", "{id}");
        let references = ask(
            &reader,
            &json!({"operation": "references", "id": id, "direction": "incoming"}),
        );
        assert_eq!(references["outcome"], "outside_published_scope", "{id}");
    }
}
