//! Bounded public retrieval over an accepted release, restricted to a verified
//! public selection. Responses are closed public projections, never raw rows.
use crate::contract;
use anyhow::{Context, Result, ensure};
use lex_grammar::labels::{canonical_slug, strip_accents};
use rusqlite::{Connection, OpenFlags, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
    fs,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

pub const PROJECTION_VERSION: &str = "public-query-v1";
pub const BINDING_VERSION: &str = "public-query-binding-v1";
pub const RESPONSE_VERSION: &str = "public-query-response-v1";
/// Largest request a caller may send, in UTF-8 bytes, excluding any delimiter.
pub const REQUEST_BYTES: usize = 8 * 1024;
const BINDING_SCHEMA: &str = include_str!("../../../schemas/public-query-binding-v1.schema.json");
const RESPONSE_SCHEMA: &str = include_str!("../../../schemas/public-query-response-v1.schema.json");
const RELEASE_SCHEMA: &str = include_str!("../../../schemas/public-release-v1.schema.json");
const INSTRUMENT_SCHEMA: &str = include_str!("../../../schemas/public-instrument-v1.schema.json");
const KINDS: [&str; 3] = ["article", "transitory", "annex"];
/// Provision fields served under the release pin, shared with the release projection.
pub const PROVISION_FIELDS: [&str; 15] = [
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
pub const SOURCE_FIELDS: [&str; 7] = [
    "source_sha256",
    "extracted_text_sha256",
    "publisher",
    "official_url",
    "retrieved_at",
    "parser_version",
    "schema_version",
];
const EDGE_SQL: &str = "SELECT e.id,e.source_provision_id,a.kind,e.target_instrument_id,e.target_provision_id,e.resolution_status,e.extras FROM reference_edges e JOIN source_anchors a ON a.id=e.source_provision_id";
const NOTICE: &str = "Selected partial coverage from a pinned release. Not official law or legal advice. Absence from this selection is not legal nonexistence; historical validity is not established.";
const CLASSIFICATION: &str = "Temporal and review fields are qualified repository classifications, not a guarantee of legal validity.";

/// Public response bounds; character and UTF-8 byte limits are separate.
#[derive(Debug, Clone)]
pub struct Limits {
    pub page_size: u32,
    pub excerpt_chars: usize,
    pub query_chars: usize,
    pub query_bytes: usize,
    pub locator_chars: usize,
    pub id_bytes: usize,
    pub response_bytes: usize,
    pub provision_text_bytes: usize,
    pub deadline: Duration,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            page_size: 10,
            excerpt_chars: 240,
            query_chars: 512,
            query_bytes: 2048,
            locator_chars: 80,
            id_bytes: 512,
            response_bytes: 16 * 1024,
            // Above the largest selected provision (209,814 bytes); larger
            // text gets an explicit oversized outcome, never truncation.
            provision_text_bytes: 256 * 1024,
            deadline: Duration::from_secs(2),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub schema_version: String,
    pub projection_version: String,
    pub database: DatabaseBinding,
    pub public_release: ReleaseBinding,
    pub instruments: Vec<InstrumentBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DatabaseBinding {
    pub release_id: String,
    pub database_sha256: String,
    pub logical_content_sha256: String,
    pub manifest_sha256: String,
    pub manifest_schema_version: String,
    pub corpus_commit: String,
    pub builder_commit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ReleaseBinding {
    pub release_id: String,
    pub manifest_sha256: String,
    pub corpus_commit: String,
    pub base_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct InstrumentBinding {
    pub slug: String,
    pub id: String,
    pub canonical_input_hashes: BTreeMap<String, String>,
    pub provision_count: usize,
    pub reference_count: usize,
    /// SHA-256 of `id\ttext_sha256\n` lines in canonical order.
    pub provisions_sha256: String,
    /// Annex number to its own retained source digest, when it differs.
    pub annex_sources: BTreeMap<String, String>,
}

fn file_sha256(path: &Path) -> Result<String> {
    let mut hasher = Sha256::new();
    std::io::copy(
        &mut fs::File::open(path).with_context(|| format!("open {}", path.display()))?,
        &mut hasher,
    )?;
    Ok(format!("{:x}", hasher.finalize()))
}

fn open_readonly(path: &Path) -> Result<Connection> {
    for sidecar in crate::sqlite_sidecars(path) {
        ensure!(!sidecar.exists(), "SQLite sidecar present");
    }
    let c = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    c.execute_batch("PRAGMA query_only=ON; PRAGMA foreign_keys=ON;")?;
    Ok(c)
}

fn validate(value: &Value, schema: &str) -> Result<()> {
    let schema: Value = serde_json::from_str(schema)?;
    jsonschema::validator_for(&schema)?
        .validate(value)
        .map_err(|error| anyhow::anyhow!(error.to_string()))
}

fn text_field<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value[key]
        .as_str()
        .with_context(|| format!("missing string field {key}"))
}

fn check_inputs(c: &Connection, slug: &str, hashes: &BTreeMap<String, String>) -> Result<()> {
    for required in ["instrument.json", "provisions.json", "references.json"] {
        ensure!(
            hashes.contains_key(required),
            "{slug}: binding lacks {required}"
        );
    }
    for (name, expected) in hashes {
        let actual: Option<String> = c
            .query_row(
                "SELECT sha256 FROM input_files WHERE path=?1",
                [format!("corpus/mx/{slug}/{name}")],
                |r| r.get(0),
            )
            .optional()?;
        ensure!(
            actual.as_deref() == Some(expected.as_str()),
            "{slug}/{name}: database input differs from the bound canonical input"
        );
    }
    Ok(())
}

/// Canonical `(id, text_sha256)` pairs in corpus order, hashed from stored text.
fn provision_digests(c: &Connection, instrument: &str) -> Result<Vec<(String, String)>> {
    let mut stmt =
        c.prepare("SELECT id,text FROM provisions WHERE instrument_id=?1 ORDER BY ordinal")?;
    let rows = stmt
        .query_map([instrument], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?
        .map(|row| row.map(|(id, text)| (id, crate::digest(text.as_bytes()))))
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

fn digest_lines(pairs: &[(String, String)]) -> Result<String> {
    let mut lines = String::new();
    for (id, sha) in pairs {
        writeln!(lines, "{id}\t{sha}")?;
    }
    Ok(crate::digest(lines.as_bytes()))
}

/// Every express edge whose anchor (provision or official title) is owned here.
fn reference_ids(c: &Connection, instrument: &str) -> Result<Vec<String>> {
    Ok(c.prepare("SELECT e.id FROM reference_edges e JOIN source_anchors a ON a.id=e.source_provision_id WHERE a.instrument_id=?1 ORDER BY e.id")?
        .query_map([instrument], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?)
}

fn release_file(dir: &Path, files: &BTreeMap<String, String>, path: &str) -> Result<Vec<u8>> {
    ensure!(
        !path.starts_with('/') && !path.split('/').any(|part| part == ".." || part.is_empty()),
        "unsafe release path {path}"
    );
    let expected = files
        .get(path)
        .with_context(|| format!("release manifest does not list {path}"))?;
    let bytes = fs::read(dir.join(path)).with_context(|| format!("read release file {path}"))?;
    ensure!(
        crate::digest(&bytes) == *expected,
        "release file differs from its manifest: {path}"
    );
    Ok(bytes)
}

/// Verify a public release against an accepted database and derive its binding.
/// Fails unless every selected input, provision text and edge set agrees.
pub fn bind(database_dir: &Path, release_dir: &Path) -> Result<Binding> {
    let manifest_bytes = fs::read(database_dir.join("manifest.json"))?;
    let manifest = contract::validate_manifest(&serde_json::from_slice(&manifest_bytes)?)?;
    bind_manifest(
        manifest,
        &crate::digest(&manifest_bytes),
        &database_dir.join("corpus.sqlite"),
        release_dir,
    )
}

/// `bind` after the database manifest has been read and schema-validated.
pub fn bind_manifest(
    manifest: contract::DatabaseManifest,
    manifest_sha256: &str,
    database: &Path,
    release_dir: &Path,
) -> Result<Binding> {
    ensure!(
        file_sha256(database)? == manifest.database_sha256,
        "database checksum differs from its manifest"
    );
    let c = open_readonly(database)?;
    let stored: String = c.query_row("SELECT id FROM corpus_release", [], |r| r.get(0))?;
    ensure!(
        stored == manifest.release_id,
        "stored release differs from its manifest"
    );
    let release_bytes = fs::read(release_dir.join("manifest.json"))?;
    let release: Value = serde_json::from_slice(&release_bytes)?;
    validate(&release, RELEASE_SCHEMA).context("public release manifest schema")?;
    let release_id = text_field(&release, "release_id")?.to_owned();
    let mut payload = release.clone();
    payload
        .as_object_mut()
        .context("release manifest is not an object")?
        .remove("release_id");
    let mut payload_bytes = serde_json::to_vec_pretty(&payload)?;
    payload_bytes.push(b'\n');
    ensure!(
        crate::digest(&payload_bytes) == release_id,
        "public release manifest does not match its release ID"
    );
    let files = release["files"]
        .as_array()
        .context("release files")?
        .iter()
        .map(|f| {
            Ok((
                text_field(f, "path")?.to_owned(),
                text_field(f, "sha256")?.to_owned(),
            ))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    let items = release["instruments"]
        .as_array()
        .context("release instruments")?;
    // `target_published` is defined over every provision in the selection.
    let mut published = BTreeSet::new();
    for item in items {
        let mut stmt = c.prepare("SELECT id FROM provisions WHERE instrument_id=?1")?;
        for id in stmt.query_map([text_field(item, "id")?], |r| r.get::<_, String>(0))? {
            published.insert(id?);
        }
    }
    let instruments = items
        .iter()
        .map(|item| bind_instrument(&c, release_dir, &files, &published, item))
        .collect::<Result<Vec<_>>>()?;
    ensure!(!instruments.is_empty(), "release selects no instruments");
    let binding = Binding {
        schema_version: BINDING_VERSION.into(),
        projection_version: PROJECTION_VERSION.into(),
        database: DatabaseBinding {
            release_id: manifest.release_id,
            database_sha256: manifest.database_sha256,
            logical_content_sha256: manifest.logical_content_sha256,
            manifest_sha256: manifest_sha256.to_owned(),
            manifest_schema_version: manifest.schema_version,
            corpus_commit: manifest.specification.corpus_commit,
            builder_commit: manifest.specification.builder_commit,
        },
        public_release: ReleaseBinding {
            release_id,
            manifest_sha256: crate::digest(&release_bytes),
            corpus_commit: text_field(&release, "corpus_commit")?.to_owned(),
            base_path: text_field(&release, "base_path")?.to_owned(),
        },
        instruments,
    };
    validate(&serde_json::to_value(&binding)?, BINDING_SCHEMA)?;
    Ok(binding)
}

fn same_source(release: &Value, served: &Value) -> bool {
    SOURCE_FIELDS.iter().all(|key| release[key] == served[key])
}

const COVERAGE: &str = r#"{"structured":true,"references":"express_with_disclosed_unresolved_targets","historical_validity":"not_established"}"#;

/// Bind one instrument only if every field the adapter serves under the release
/// pin equals the release's own projection: instrument and source metadata,
/// provision fields and artifact paths, and complete reference edges.
fn bind_instrument(
    c: &Connection,
    dir: &Path,
    files: &BTreeMap<String, String>,
    published: &BTreeSet<String>,
    item: &Value,
) -> Result<InstrumentBinding> {
    let slug = text_field(item, "slug")?;
    let id = text_field(item, "id")?;
    let hashes: BTreeMap<String, String> =
        serde_json::from_value(item["canonical_input_hashes"].clone())?;
    check_inputs(c, slug, &hashes)?;
    let doc: Value = serde_json::from_slice(&release_file(
        dir,
        files,
        text_field(item, "document_path")?,
    )?)?;
    validate(&doc, INSTRUMENT_SCHEMA).with_context(|| format!("{slug}: instrument schema"))?;
    let main = check_instrument(c, &doc, item, id, slug)?;
    let annexes = annex_sources_by_number(c, slug, &hashes)?;
    let provisions = doc["provisions"].as_array().context("provisions")?;
    let mut stmt =
        c.prepare("SELECT id,extras,text FROM provisions WHERE instrument_id=?1 ORDER BY ordinal")?;
    let stored = stmt
        .query_map([id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    ensure!(
        stored.len() == provisions.len() && item["provision_count"] == stored.len(),
        "{slug}: provision count differs between release and database"
    );
    let mut digests = Vec::new();
    let mut annex_sources = BTreeMap::new();
    for (public, (stored_id, extras, text)) in provisions.iter().zip(&stored) {
        let (provision, served) = provision_fields(stored_id, id, extras, text)?;
        for field in PROVISION_FIELDS {
            ensure!(
                public[field] == served[field],
                "{stored_id}: release and database differ in {field}"
            );
        }
        let stem = format!("provision-{}", crate::digest(stored_id.as_bytes()));
        ensure!(
            public["text_path"] == format!("{stem}.txt").as_str()
                && public["markdown_path"] == format!("{stem}.md").as_str(),
            "{stored_id}: release artifact paths differ"
        );
        let artifact = release_file(dir, files, &format!("mx/{slug}/{stem}.txt"))?;
        ensure!(
            artifact == text.as_bytes(),
            "{stored_id}: exact-text artifact differs"
        );
        release_file(dir, files, &format!("mx/{slug}/{stem}.md"))?;
        let expected =
            if provision.provision_type == lex_core::ProvisionType::Annex && !annexes.is_empty() {
                let annex = annexes
                    .get(&provision.number)
                    .with_context(|| format!("{stored_id}: no retained annex source"))?;
                annex_sources.insert(
                    provision.number.clone(),
                    text_field(annex, "source_sha256")?.to_owned(),
                );
                annex
            } else {
                &main
            };
        ensure!(
            same_source(&public["source"], expected),
            "{stored_id}: release source metadata differs from the database"
        );
        digests.push((stored_id.clone(), crate::digest(text.as_bytes())));
    }
    let reference_count = check_edges(c, &doc, id, slug, published)?;
    Ok(InstrumentBinding {
        slug: slug.to_owned(),
        id: id.to_owned(),
        canonical_input_hashes: hashes,
        provision_count: stored.len(),
        reference_count,
        provisions_sha256: digest_lines(&digests)?,
        annex_sources,
    })
}

/// The producer compares parsed URLs (`url::Url` on both the adapter
/// configuration and the source manifest), so this does too; an unparsable
/// URL matches nothing.
fn same_url(left: &str, right: &str) -> bool {
    matches!((url::Url::parse(left), url::Url::parse(right)), (Ok(a), Ok(b)) if a == b)
}

/// The producer's annex mapping: annex number `n` uses the retained manifest
/// for the adapter's `annex_pdf_urls[n - 1]`. The adapter is a database input
/// covered by the database checksum. URLs compare as parsed URLs, and every
/// match must be unique.
fn annex_sources_by_number(
    c: &Connection,
    slug: &str,
    hashes: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, Value>> {
    let mut stmt = c.prepare("SELECT path,bytes FROM input_files WHERE path LIKE 'adapters/%'")?;
    let adapters = stmt
        .query_map([], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, Vec<u8>>(1)?))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?
        .into_iter()
        .filter(|(path, _)| {
            path.split('/').count() == 3 && path.ends_with(&format!("/{slug}.json"))
        })
        .collect::<Vec<_>>();
    ensure!(
        adapters.len() == 1,
        "{slug}: adapter configuration is not unique"
    );
    let config: Value = serde_json::from_slice(&adapters[0].1)?;
    let urls = config["annex_pdf_urls"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    if urls.is_empty() {
        return Ok(BTreeMap::new());
    }
    ensure!(
        hashes.contains_key("annex-source-manifests.json"),
        "{slug}: annex sources are not bound"
    );
    let manifests = input_json(c, &format!("corpus/mx/{slug}/annex-source-manifests.json"))?;
    let manifests = manifests.as_array().context("annex manifests")?;
    let mut out = BTreeMap::new();
    for (index, url) in urls.iter().enumerate() {
        let wanted = url.as_str().context("annex URL")?;
        let matches = manifests
            .iter()
            .filter(|m| {
                m["official_url"]
                    .as_str()
                    .is_some_and(|got| same_url(wanted, got))
            })
            .collect::<Vec<_>>();
        ensure!(
            matches.len() == 1,
            "{slug}: annex source mapping is ambiguous or missing"
        );
        out.insert((index + 1).to_string(), public_source(matches[0]));
    }
    Ok(out)
}

/// Instrument metadata and main source must equal the DB; returns that source.
fn check_instrument(
    c: &Connection,
    doc: &Value,
    item: &Value,
    id: &str,
    slug: &str,
) -> Result<Value> {
    let extras: String = c.query_row("SELECT extras FROM instruments WHERE id=?1", [id], |r| {
        r.get(0)
    })?;
    let instrument: Value = serde_json::from_str(&extras)?;
    let main = public_source(&input_json(
        c,
        &format!("corpus/mx/{slug}/source-manifest.json"),
    )?);
    ensure!(
        doc["id"] == id
            && doc["slug"] == slug
            && [
                "official_title",
                "short_name",
                "publication_date",
                "latest_reform_date"
            ]
            .iter()
            .all(|key| doc[key] == instrument[key])
            && item["title"] == instrument["official_title"]
            && item["short_name"] == instrument["short_name"]
            && same_source(&doc["source"], &main)
            && same_source(&item["source"], &main)
            && item["coverage"] == serde_json::from_str::<Value>(COVERAGE)?,
        "{slug}: release instrument metadata differs from the database"
    );
    Ok(main)
}

/// Every release edge must equal the DB edge projection as a whole object.
fn check_edges(
    c: &Connection,
    doc: &Value,
    id: &str,
    slug: &str,
    published: &BTreeSet<String>,
) -> Result<usize> {
    let expected = stored_edges(
        c,
        &format!("{EDGE_SQL} WHERE a.instrument_id=?1 ORDER BY e.id"),
        id,
    )?
    .iter()
    .map(|edge| {
        let mut value = edge_core(edge)?;
        value["source_provision_id"] = json!(edge[1]);
        value["target_published"] = json!(published.contains(&edge[4]));
        Ok((edge[0].clone(), value))
    })
    .collect::<Result<BTreeMap<_, _>>>()?;
    // Build the release map with duplicate rejection, then compare whole maps:
    // count plus membership would let one valid edge stand in for another.
    let mut released = BTreeMap::new();
    for edge in doc["references"].as_array().context("references")? {
        let key = text_field(edge, "id")?.to_owned();
        ensure!(
            released.insert(key.clone(), edge.clone()).is_none(),
            "{slug}: release repeats reference {key}"
        );
    }
    ensure!(
        released == expected,
        "{slug}: release and database reference edges differ"
    );
    Ok(expected.len())
}

/// Served provision fields from one stored row, field-identical to the public
/// release projection of the same canonical provision.
fn provision_fields(
    id: &str,
    instrument: &str,
    extras: &str,
    text: &str,
) -> Result<(lex_core::Provision, Value)> {
    let mut value: Value = serde_json::from_str(extras)?;
    value["id"] = json!(id);
    value["instrument_id"] = json!(instrument);
    value["text"] = json!(text);
    let provision: lex_core::Provision = serde_json::from_value(value)?;
    let fields = json!({"id": id, "label": provision.label, "number": provision.number,
        "provision_type": provision.provision_type, "heading_context": provision.heading_context,
        "text": text, "text_sha256": crate::digest(text.as_bytes()),
        "publication_date": provision.publication_date, "effective_from": provision.effective_from,
        "effective_to": provision.effective_to, "temporal_status": provision.temporal_status,
        "temporal_basis": provision.temporal_basis, "review_status": provision.review_status,
        "repeals": provision.repeals, "commencement_condition": public_condition(&provision)});
    Ok((provision, fields))
}

/// Rows of `EDGE_SQL`: `[id, source, source_kind, target_instrument, target, status, extras]`.
fn stored_edges(c: &Connection, sql: &str, key: &str) -> Result<Vec<[String; 7]>> {
    Ok(c.prepare(sql)?
        .query_map([key], |r| {
            Ok([
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
                r.get(6)?,
            ])
        })?
        .collect::<rusqlite::Result<_>>()?)
}

/// Edge fields shared by the served projection and the public release.
fn edge_core(edge: &[String; 7]) -> Result<Value> {
    let extras: Value = serde_json::from_str(&edge[6])?;
    Ok(json!({"id": edge[0], "source_span": extras["source_span"],
        "start_char": extras["start_char"], "end_char": extras["end_char"],
        "target_instrument_id": edge[3], "target_provision_id": edge[4], "resolution_status": edge[5],
        "reference_form": extras["reference_form"], "basis": extras["basis"]}))
}

struct Scope {
    slug: String,
    id: String,
    title: String,
    short_name: String,
    publication_date: Value,
    latest_reform_date: Value,
    provision_count: usize,
    reference_count: usize,
    source: Value,
    annex: BTreeMap<String, Value>,
}

/// Public source fields only; preserves the recorded official URL verbatim.
fn public_source(manifest: &Value) -> Value {
    json!({"source_sha256": manifest["source_sha256"], "extracted_text_sha256": manifest["extracted_text_sha256"],
        "publisher": manifest["publisher"], "official_url": manifest["official_url"],
        "retrieved_at": manifest["retrieved_at"], "parser_version": manifest["parser_version"],
        "schema_version": manifest["schema_version_used"]})
}

fn input_json(c: &Connection, path: &str) -> Result<Value> {
    let bytes: Vec<u8> =
        c.query_row("SELECT bytes FROM input_files WHERE path=?1", [path], |r| {
            r.get(0)
        })?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn load_scope(c: &Connection, bound: &InstrumentBinding) -> Result<Scope> {
    check_inputs(c, &bound.slug, &bound.canonical_input_hashes)?;
    let stored = provision_digests(c, &bound.id)?;
    ensure!(
        stored.len() == bound.provision_count && digest_lines(&stored)? == bound.provisions_sha256,
        "{}: database provisions differ from the binding",
        bound.slug
    );
    ensure!(
        reference_ids(c, &bound.id)?.len() == bound.reference_count,
        "{}: database references differ from the binding",
        bound.slug
    );
    let extras: String = c.query_row(
        "SELECT extras FROM instruments WHERE id=?1",
        [&bound.id],
        |r| r.get(0),
    )?;
    let instrument: Value = serde_json::from_str(&extras)?;
    let main = input_json(c, &format!("corpus/mx/{}/source-manifest.json", bound.slug))?;
    let mut annex = BTreeMap::new();
    if !bound.annex_sources.is_empty() {
        // The same identity rule as the binder (annex number -> configured URL
        // -> unique retained manifest), then the bound digest must agree.
        // Selecting by digest alone failed when two annexes share content
        // (Codex audit 2026-11-17, R6).
        let by_number = annex_sources_by_number(c, &bound.slug, &bound.canonical_input_hashes)?;
        for (number, sha) in &bound.annex_sources {
            let source = by_number
                .get(number)
                .with_context(|| format!("annex {number} source is missing"))?;
            ensure!(
                source["source_sha256"] == sha.as_str(),
                "annex {number} source differs from the binding"
            );
            annex.insert(number.clone(), source.clone());
        }
    }
    Ok(Scope {
        slug: bound.slug.clone(),
        id: bound.id.clone(),
        title: text_field(&instrument, "official_title")?.to_owned(),
        short_name: text_field(&instrument, "short_name")?.to_owned(),
        publication_date: instrument["publication_date"].clone(),
        latest_reform_date: instrument["latest_reform_date"].clone(),
        provision_count: bound.provision_count,
        reference_count: bound.reference_count,
        source: public_source(&main),
        annex,
    })
}

#[derive(Debug, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
enum Request {
    Coverage {
        release_id: Option<String>,
        page_size: Option<u32>,
        cursor: Option<String>,
    },
    Resolve {
        release_id: Option<String>,
        instrument: String,
        kind: String,
        locator: String,
        as_of: Option<String>,
    },
    Search {
        release_id: Option<String>,
        query: String,
        instrument: Option<String>,
        kind: Option<String>,
        page_size: Option<u32>,
        cursor: Option<String>,
    },
    GetProvision {
        release_id: Option<String>,
        id: String,
        as_of: Option<String>,
    },
    References {
        release_id: Option<String>,
        id: String,
        direction: Option<Direction>,
        page_size: Option<u32>,
        cursor: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Direction {
    Outgoing,
    Incoming,
}

impl Request {
    fn operation(&self) -> &'static str {
        match self {
            Self::Coverage { .. } => "coverage",
            Self::Resolve { .. } => "resolve",
            Self::Search { .. } => "search",
            Self::GetProvision { .. } => "get_provision",
            Self::References { .. } => "references",
        }
    }
    fn release_id(&self) -> Option<&str> {
        match self {
            Self::Coverage { release_id, .. }
            | Self::Resolve { release_id, .. }
            | Self::Search { release_id, .. }
            | Self::GetProvision { release_id, .. }
            | Self::References { release_id, .. } => release_id.as_deref(),
        }
    }
}

enum Failure {
    Client(&'static str, String),
    /// One list item alone exceeds the response limit. The error carries a
    /// cursor positioned after that item, so traversal can continue
    /// explicitly instead of dead-ending (Codex audit 2026-11-17, R1).
    Oversized {
        message: String,
        skip_cursor: String,
    },
    Deadline,
    Service(anyhow::Error),
}

impl<E: Into<anyhow::Error>> From<E> for Failure {
    fn from(error: E) -> Self {
        let error = error.into();
        let interrupted = error.chain().any(|cause| {
            matches!(cause.downcast_ref::<rusqlite::Error>(),
                Some(rusqlite::Error::SqliteFailure(f, _)) if f.code == rusqlite::ErrorCode::OperationInterrupted)
        });
        if interrupted {
            Self::Deadline
        } else {
            Self::Service(error)
        }
    }
}

fn client(code: &'static str, message: impl Into<String>) -> Failure {
    Failure::Client(code, message.into())
}

type Outcome = std::result::Result<(&'static str, Value), Failure>;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CursorBody {
    v: u8,
    op: String,
    snapshot: String,
    request: String,
    after: Vec<String>,
}

const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

fn base64url(bytes: &[u8]) -> String {
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let n = chunk
            .iter()
            .enumerate()
            .fold(0u32, |acc, (i, b)| acc | u32::from(*b) << (16 - 8 * i));
        for i in 0..=chunk.len() {
            out.push(char::from(BASE64[(n >> (18 - 6 * i) & 63) as usize]));
        }
    }
    out
}

fn unbase64url(text: &str) -> Option<Vec<u8>> {
    let values = text
        .bytes()
        .map(|b| {
            BASE64
                .iter()
                .position(|c| *c == b)
                .and_then(|p| u32::try_from(p).ok())
        })
        .collect::<Option<Vec<_>>>()?;
    if values.len() % 4 == 1 {
        return None;
    }
    let mut out = Vec::new();
    for chunk in values.chunks(4) {
        let n = chunk
            .iter()
            .enumerate()
            .fold(0u32, |acc, (i, v)| acc | v << (18 - 6 * i));
        for i in 0..chunk.len() - 1 {
            out.push(u8::try_from(n >> (16 - 8 * i) & 255).ok()?);
        }
    }
    // Canonical encodings only: re-encoding must reproduce the input.
    (base64url(&out) == text).then_some(out)
}

/// Read-only adapter that serves only validated public responses.
pub struct PublicReader {
    connection: Connection,
    binding: Binding,
    scopes: Vec<Scope>,
    allowlist: String,
    limits: Limits,
    validator: jsonschema::Validator,
    origin: Instant,
    deadline_ms: Arc<AtomicU64>,
}

impl PublicReader {
    /// Fails closed unless the database bytes and selected inputs match the binding.
    pub fn open(database: &Path, binding: &Path, limits: Limits) -> Result<Self> {
        let value: Value = serde_json::from_slice(&fs::read(binding)?)?;
        validate(&value, BINDING_SCHEMA).context("binding does not match its schema")?;
        let binding: Binding = serde_json::from_value(value)?;
        ensure!(
            binding.projection_version == PROJECTION_VERSION,
            "binding projection version is not served by this adapter"
        );
        ensure!(
            file_sha256(database)? == binding.database.database_sha256,
            "database checksum differs from the binding"
        );
        let connection = open_readonly(database)?;
        let (id, specification): (String, String) =
            connection.query_row("SELECT id,specification FROM corpus_release", [], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })?;
        let specification: Value = serde_json::from_str(&specification)?;
        ensure!(
            id == binding.database.release_id
                && specification["corpus_commit"] == binding.database.corpus_commit.as_str()
                && specification["builder_commit"] == binding.database.builder_commit.as_str(),
            "database release differs from the binding"
        );
        let scopes = binding
            .instruments
            .iter()
            .map(|bound| load_scope(&connection, bound))
            .collect::<Result<Vec<_>>>()?;
        let allowlist = serde_json::to_string(&scopes.iter().map(|s| &s.id).collect::<Vec<_>>())?;
        let schema: Value = serde_json::from_str(RESPONSE_SCHEMA)?;
        let validator = jsonschema::validator_for(&schema)?;
        let origin = Instant::now();
        let deadline_ms = Arc::new(AtomicU64::new(u64::MAX));
        let watched = Arc::clone(&deadline_ms);
        connection.progress_handler(
            100,
            Some(move || millis(origin.elapsed()) >= watched.load(Ordering::Relaxed)),
        )?;
        Ok(Self {
            connection,
            binding,
            scopes,
            allowlist,
            limits,
            validator,
            origin,
            deadline_ms,
        })
    }

    fn past_deadline(&self) -> bool {
        millis(self.origin.elapsed()) >= self.deadline_ms.load(Ordering::Relaxed)
    }

    #[must_use]
    pub fn binding(&self) -> &Binding {
        &self.binding
    }

    /// Answer one JSON request with one closed, schema-validated response.
    #[must_use]
    pub fn handle(&self, line: &str) -> Value {
        self.deadline_ms.store(
            millis(self.origin.elapsed() + self.limits.deadline),
            Ordering::Relaxed,
        );
        let (operation, mut outcome) = self.dispatch(line);
        // Work outside SQLite (excerpts, serialization) is not interrupted by
        // the progress handler, so the whole request is checked once more
        // (Codex audit 2026-11-17, R4).
        if outcome.is_ok() && self.past_deadline() {
            outcome = Err(Failure::Deadline);
        }
        self.deadline_ms.store(u64::MAX, Ordering::Relaxed);
        let response = match outcome {
            Ok((outcome, result)) => self.envelope(operation, outcome, &result),
            Err(Failure::Client(code, message)) => {
                self.envelope(operation, "client_error", &error(code, &message))
            }
            Err(Failure::Oversized {
                message,
                skip_cursor,
            }) => {
                let mut body = error("response_too_large", &message);
                body["skip_cursor"] = json!(skip_cursor);
                self.envelope(operation, "client_error", &body)
            }
            Err(Failure::Deadline) => self.envelope(
                operation,
                "unavailable",
                &error("deadline_exceeded", "query exceeded its execution deadline"),
            ),
            Err(Failure::Service(cause)) => {
                // Operator diagnostics only; the public response stays generic.
                eprintln!("public query internal error: {cause:#}");
                self.envelope(
                    operation,
                    "unavailable",
                    &error("internal_error", "the service could not answer"),
                )
            }
        };
        if self.validator.is_valid(&response) {
            response
        } else {
            self.envelope(
                operation,
                "unavailable",
                &error("contract_violation", "response failed the public contract"),
            )
        }
    }

    /// A closed client-error response for a request rejected before parsing,
    /// such as an over-limit or non-UTF-8 transport frame.
    #[must_use]
    pub fn reject(&self, message: &str) -> Value {
        self.envelope(None, "client_error", &error("bad_request", message))
    }

    fn envelope(&self, operation: Option<&str>, outcome: &str, result: &Value) -> Value {
        let release = &self.binding.public_release;
        let database = &self.binding.database;
        json!({"schema_version": RESPONSE_VERSION, "operation": operation, "outcome": outcome,
            "snapshot": {"public_release_id": release.release_id, "public_release_corpus_commit": release.corpus_commit,
                "public_release_manifest_sha256": release.manifest_sha256, "database_release_id": database.release_id,
                "database_sha256": database.database_sha256, "database_corpus_commit": database.corpus_commit,
                "projection_version": PROJECTION_VERSION, "coverage": "selected_partial", "notice": NOTICE},
            "result": result})
    }

    fn dispatch(&self, line: &str) -> (Option<&'static str>, Outcome) {
        if line.len() > REQUEST_BYTES {
            return (
                None,
                Err(client("bad_request", "request exceeds 8192 bytes")),
            );
        }
        let request: Request = match serde_json::from_str(line) {
            Ok(request) => request,
            Err(e) => return (None, Err(client("bad_request", truncate(&e.to_string())))),
        };
        let operation = request.operation();
        if let Some(pin) = request.release_id() {
            if !is_hex64(pin) {
                return (
                    Some(operation),
                    Err(client("bad_request", "release_id must be 64 hex digits")),
                );
            }
            if pin != self.binding.public_release.release_id {
                return (
                    Some(operation),
                    Ok((
                        "release_unavailable",
                        json!({"kind": "release_unavailable", "requested_release_id": pin,
                        "message": "Only the bound current release is served; the requested release is not resolved to current text."}),
                    )),
                );
            }
        }
        let outcome = match request {
            Request::Coverage {
                page_size, cursor, ..
            } => self.coverage(page_size, cursor.as_deref()),
            Request::Resolve {
                instrument,
                kind,
                locator,
                as_of,
                ..
            } => {
                no_as_of(as_of.as_deref()).and_then(|()| self.resolve(&instrument, &kind, &locator))
            }
            Request::Search {
                query,
                instrument,
                kind,
                page_size,
                cursor,
                ..
            } => self.search(
                &query,
                instrument.as_deref(),
                kind.as_deref(),
                page_size,
                cursor.as_deref(),
            ),
            Request::GetProvision { id, as_of, .. } => {
                no_as_of(as_of.as_deref()).and_then(|()| self.get_provision(&id))
            }
            Request::References {
                id,
                direction,
                page_size,
                cursor,
                ..
            } => self.references(
                &id,
                direction.unwrap_or(Direction::Outgoing),
                page_size,
                cursor.as_deref(),
            ),
        };
        (Some(operation), outcome)
    }

    fn page_size(&self, requested: Option<u32>) -> std::result::Result<u32, Failure> {
        let size = requested.unwrap_or(self.limits.page_size);
        if (1..=self.limits.page_size).contains(&size) {
            Ok(size)
        } else {
            Err(client(
                "bad_request",
                format!("page_size must be 1..{}", self.limits.page_size),
            ))
        }
    }

    fn scope_by_name(&self, name: &str) -> std::result::Result<&Scope, Failure> {
        self.scopes
            .iter()
            .find(|s| s.slug == name || s.id == name)
            .ok_or_else(|| {
                client(
                    "unknown_instrument",
                    "instrument is not in the published selection",
                )
            })
    }

    /// Ownership is decided by identifier prefix alone, so out-of-scope
    /// database coverage is never probed or disclosed.
    fn owner(&self, id: &str) -> Option<&Scope> {
        self.scopes.iter().find(|s| {
            id == s.id
                || id
                    .strip_prefix(s.id.as_str())
                    .is_some_and(|rest| rest.starts_with(':'))
        })
    }

    fn snapshot_key(&self) -> String {
        crate::digest(
            format!(
                "{}\n{}\n{PROJECTION_VERSION}",
                self.binding.public_release.release_id, self.binding.database.release_id
            )
            .as_bytes(),
        )
    }

    fn encode_cursor(&self, op: &str, request: &str, after: Vec<String>) -> Result<String> {
        Ok(base64url(&serde_json::to_vec(&CursorBody {
            v: 1,
            op: op.into(),
            snapshot: self.snapshot_key(),
            request: request.into(),
            after,
        })?))
    }

    fn decode_cursor(
        &self,
        cursor: &str,
        op: &str,
        request: &str,
        arity: usize,
    ) -> std::result::Result<Vec<String>, Failure> {
        let body: CursorBody = (cursor.len() <= 1024)
            .then(|| unbase64url(cursor))
            .flatten()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .filter(|body: &CursorBody| body.v == 1 && body.after.len() == arity)
            .ok_or_else(|| client("invalid_cursor", "cursor is malformed"))?;
        if body.snapshot != self.snapshot_key() {
            return Err(client(
                "cursor_release_mismatch",
                "cursor belongs to a different release or projection",
            ));
        }
        if body.op != op || body.request != request {
            return Err(client(
                "cursor_request_mismatch",
                "cursor belongs to a different query, filter or page size",
            ));
        }
        Ok(body.after)
    }

    /// Fit a page into the response byte budget, shortening it deterministically.
    fn page(
        &self,
        op: &'static str,
        request: &str,
        mut rows: Vec<(Vec<String>, Value)>,
        mut more: bool,
        mut result: Value,
    ) -> Outcome {
        loop {
            let next = match rows.last() {
                Some((key, _)) if more => Some(self.encode_cursor(op, request, key.clone())?),
                _ => None,
            };
            result["items"] = json!(rows.iter().map(|(_, item)| item).collect::<Vec<_>>());
            result["next_cursor"] = json!(next);
            let size = serde_json::to_vec(&self.envelope(Some(op), "ok", &result))?.len();
            if size <= self.limits.response_bytes {
                return Ok(("ok", result));
            }
            if rows.len() <= 1 {
                let Some((key, _)) = rows.first() else {
                    return Err(client(
                        "response_too_large",
                        "the empty page exceeds the response limit",
                    ));
                };
                return Err(Failure::Oversized {
                    message:
                        "a single item exceeds the response limit; skip_cursor continues after it"
                            .to_owned(),
                    skip_cursor: self.encode_cursor(op, request, key.clone())?,
                });
            }
            rows.pop();
            more = true;
        }
    }

    fn artifacts(&self, scope: &Scope, id: &str) -> Value {
        let stem = format!(
            "releases/{}/mx/{}/provision-{}",
            self.binding.public_release.release_id,
            scope.slug,
            crate::digest(id.as_bytes())
        );
        json!({"base_path": self.binding.public_release.base_path,
            "text_path": format!("{stem}.txt"), "markdown_path": format!("{stem}.md")})
    }

    fn coverage(&self, page_size: Option<u32>, cursor: Option<&str>) -> Outcome {
        let size = self.page_size(page_size)?;
        let key = request_key("coverage", &json!({"page_size": size}));
        let after = match cursor {
            Some(c) => Some(self.decode_cursor(c, "coverage", &key, 1)?.remove(0)),
            None => None,
        };
        let mut scopes = self.scopes.iter().collect::<Vec<_>>();
        scopes.sort_by(|a, b| a.slug.cmp(&b.slug));
        let selected = scopes
            .into_iter()
            .filter(|s| after.as_ref().is_none_or(|a| s.slug.as_str() > a.as_str()))
            .collect::<Vec<_>>();
        let more = selected.len() > size as usize;
        let rows = selected
            .into_iter()
            .take(size as usize)
            .map(|s| (vec![s.slug.clone()], self.instrument_item(s)))
            .collect();
        let l = &self.limits;
        self.page("coverage", &key, rows, more, json!({"kind": "coverage", "scope": "selected_partial",
            "instrument_count": self.scopes.len(),
            "limits": {"page_size": l.page_size, "excerpt_chars": l.excerpt_chars, "query_chars": l.query_chars,
                "query_bytes": l.query_bytes, "response_bytes": l.response_bytes,
                "provision_text_bytes": l.provision_text_bytes,
                "deadline_ms": millis(l.deadline)},
            "operations": ["coverage", "resolve", "search", "get_provision", "references"]}))
    }

    fn instrument_item(&self, s: &Scope) -> Value {
        json!({"slug": s.slug, "id": s.id, "official_title": s.title, "short_name": s.short_name,
            "publication_date": s.publication_date, "latest_reform_date": s.latest_reform_date,
            "provision_count": s.provision_count, "reference_count": s.reference_count, "source": s.source,
            "index_path": format!("releases/{}/mx/{}/index.md", self.binding.public_release.release_id, s.slug),
            "coverage": {"structured": true, "references": "express_with_disclosed_unresolved_targets",
                "historical_validity": "not_established"}})
    }

    fn resolve(&self, instrument: &str, kind: &str, locator: &str) -> Outcome {
        let scope = self.scope_by_name(instrument)?;
        let kind = valid_kind(kind)?;
        let printed = locator.trim();
        if printed.chars().count() > self.limits.locator_chars
            || printed.chars().any(char::is_control)
            || !printed.chars().any(char::is_alphanumeric)
        {
            return Err(client(
                "invalid_locator",
                "locator must be a printed provision number",
            ));
        }
        let wanted = canonical_slug(printed);
        let prefix = format!("{}:{kind}:", scope.id);
        let mut stmt = self.connection.prepare("SELECT id,number,label,heading_title FROM provisions WHERE instrument_id=?1 AND provision_type=?2 ORDER BY ordinal")?;
        let mut rows = stmt.query(params![scope.id, kind])?;
        let mut candidates = Vec::new();
        while let Some(row) = rows.next()? {
            let id: String = row.get(0)?;
            let number: String = row.get(1)?;
            // Exact printed-number identity, with the canonical ID slug as the
            // only equivalence; no editorial aliases are inferred.
            if canonical_slug(&number) == wanted
                || id.strip_prefix(&prefix) == Some(wanted.as_str())
            {
                candidates.push(json!({"id": id, "number": number, "label": row.get::<_, String>(2)?,
                    "heading_title": row.get::<_, String>(3)?, "artifacts": self.artifacts(scope, &id)}));
            }
        }
        let requested = json!({"instrument": scope.slug, "kind": kind, "locator": printed});
        match candidates.len() {
            0 => self.absence(&requested, scope, kind, &wanted),
            count => {
                let total = candidates.len();
                candidates.truncate(self.limits.page_size as usize);
                Ok((
                    if count == 1 { "found" } else { "ambiguous" },
                    json!({"kind": "resolution", "requested": requested, "candidate_count": total,
                    "candidates": candidates, "truncated": total > candidates.len()}),
                ))
            }
        }
    }

    fn absence(&self, requested: &Value, scope: &Scope, kind: &str, slug: &str) -> Outcome {
        let mut qualifications = Vec::new();
        if kind == "article" {
            let mut stmt = self.connection.prepare("SELECT printed_label,extras FROM known_structural_debt WHERE instrument_id=?1 ORDER BY ordinal")?;
            let mut rows = stmt.query([&scope.id])?;
            while let Some(row) = rows.next()? {
                let printed: String = row.get(0)?;
                if canonical_slug(&printed) != slug {
                    continue;
                }
                let extras: Value = serde_json::from_str(&row.get::<_, String>(1)?)?;
                let container = extras["containing_provision_id"]
                    .as_str()
                    .filter(|id| self.owner(id).is_some());
                qualifications.push(
                    json!({"kind": "not_separately_parsed", "printed_label": printed,
                    "containing_provision_id": container,
                    "start_char": container.and(extras["start_char"].as_u64()),
                    "end_char": container.and(extras["end_char"].as_u64())}),
                );
            }
        }
        let target = format!("{}:{kind}:{slug}", scope.id);
        let mut stmt = self.connection.prepare("SELECT source_instrument_id,multiplicity FROM cited_but_missing WHERE target_provision_id=?1 AND source_instrument_id IN (SELECT value FROM json_each(?2)) ORDER BY source_instrument_id")?;
        let mut rows = stmt.query(params![target, self.allowlist])?;
        while let Some(row) = rows.next()? {
            qualifications.push(json!({"kind": "cited_not_in_corpus",
                "citing_instrument_id": row.get::<_, String>(0)?, "multiplicity": row.get::<_, i64>(1)?}));
        }
        Ok((
            "not_found",
            json!({"kind": "absence", "requested": requested, "qualifications": qualifications,
            "coverage": "not_established",
            "message": "Not found in this published selection; coverage not established. This does not assert that the provision does not exist."}),
        ))
    }

    fn outside_scope(id: &str) -> (&'static str, Value) {
        (
            "outside_published_scope",
            json!({"kind": "outside_scope", "requested_id": id,
            "message": "Identifier is outside the published selection; nothing is asserted about it."}),
        )
    }

    fn get_provision(&self, id: &str) -> Outcome {
        self.valid_id(id)?;
        let Some(scope) = self.owner(id) else {
            return Ok(Self::outside_scope(id));
        };
        let row: Option<(String, String)> = self
            .connection
            .query_row(
                "SELECT extras,text FROM provisions WHERE id=?1 AND instrument_id=?2",
                params![id, scope.id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        let Some((extras, text)) = row else {
            let rest = id
                .strip_prefix(scope.id.as_str())
                .and_then(|r| r.strip_prefix(':'))
                .and_then(|r| r.split_once(':'))
                .filter(|(kind, _)| KINDS.contains(kind));
            let requested = json!({"id": id});
            return match rest {
                Some((kind, slug)) => self.absence(&requested, scope, kind, slug),
                None => self.absence(&requested, scope, "", ""),
            };
        };
        let sha = crate::digest(text.as_bytes());
        let (provision, mut fields) = provision_fields(id, &scope.id, &extras, &text)?;
        if text.len() > self.limits.provision_text_bytes {
            return Ok((
                "oversized",
                json!({"kind": "oversized_provision", "id": id, "label": provision.label,
                "text_sha256": sha, "text_bytes": text.len(), "limit_bytes": self.limits.provision_text_bytes,
                "artifacts": self.artifacts(scope, id),
                "message": "Complete text exceeds this response limit. Retrieve the exact-text artifact and verify text_sha256; text is never truncated."}),
            ));
        }
        let source = match provision.provision_type {
            lex_core::ProvisionType::Annex => scope.annex.get(&provision.number),
            _ => None,
        }
        .unwrap_or(&scope.source);
        let extra = json!({"text_bytes": text.len(), "text_chars": text.chars().count(),
            "classification_notice": CLASSIFICATION, "source": source,
            "instrument": {"slug": scope.slug, "id": scope.id, "short_name": scope.short_name, "official_title": scope.title},
            "artifacts": self.artifacts(scope, id)});
        if let (Some(fields), Value::Object(extra)) = (fields.as_object_mut(), extra) {
            fields.extend(extra);
        }
        Ok(("found", json!({"kind": "provision", "provision": fields})))
    }

    fn valid_id(&self, id: &str) -> std::result::Result<(), Failure> {
        if id.is_empty() || id.len() > self.limits.id_bytes || id.chars().any(char::is_control) {
            Err(client("bad_request", "invalid identifier"))
        } else {
            Ok(())
        }
    }

    fn search(
        &self,
        query: &str,
        instrument: Option<&str>,
        kind: Option<&str>,
        page_size: Option<u32>,
        cursor: Option<&str>,
    ) -> Outcome {
        let literal = query.trim();
        if query.len() > self.limits.query_bytes
            || literal.chars().count() > self.limits.query_chars
        {
            return Err(client("query_too_long", "query exceeds its length limit"));
        }
        if literal.chars().any(char::is_control) {
            return Err(client("invalid_query", "query contains control characters"));
        }
        // Mirrors the tokenizer: input with no letter or digit (for example a
        // lone combining accent) would match nothing meaningful.
        if !literal.chars().any(char::is_alphanumeric) {
            return Err(client("empty_query", "query has no searchable characters"));
        }
        let scope = instrument
            .map(|name| self.scope_by_name(name))
            .transpose()?;
        let kind = kind.map(valid_kind).transpose()?;
        let size = self.page_size(page_size)?;
        let key = request_key(
            "search",
            &json!({"query": literal, "instrument": scope.map(|s| &s.id), "kind": kind, "page_size": size}),
        );
        let after = match cursor {
            Some(c) => {
                let after = self.decode_cursor(c, "search", &key, 2)?;
                let bits = after[0]
                    .parse::<u64>()
                    .map_err(|_| client("invalid_cursor", "cursor is malformed"))?;
                Some((f64::from_bits(bits), after[1].clone()))
            }
            None => None,
        };
        let allow = match scope {
            Some(s) => serde_json::to_string(&[&s.id])?,
            None => self.allowlist.clone(),
        };
        let phrase = format!("\"{}\"", literal.replace('"', "\"\""));
        // Only identifiers and scores are ranked; text is read for the page alone.
        let mut stmt = self.connection.prepare("WITH m AS MATERIALIZED (SELECT p.id AS id,bm25(provisions_fts,10,3,1) AS score FROM provisions_fts JOIN provisions p ON p.row_id=provisions_fts.rowid WHERE provisions_fts MATCH ?1 AND p.instrument_id IN (SELECT value FROM json_each(?2)) AND (?3 IS NULL OR p.provision_type=?3)) SELECT id,score FROM m WHERE ?4 IS NULL OR score>?4 OR (score=?4 AND id>?5) ORDER BY score,id LIMIT ?6")?;
        let hits = stmt
            .query_map(
                params![
                    phrase,
                    allow,
                    kind,
                    after.as_ref().map(|a| a.0),
                    after.as_ref().map(|a| a.1.as_str()),
                    size + 1
                ],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, f64>(1)?)),
            )?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let more = hits.len() > size as usize;
        let mut rows = Vec::new();
        for (id, score) in hits.into_iter().take(size as usize) {
            // Excerpts are built in Rust, beyond the SQLite progress handler.
            if self.past_deadline() {
                return Err(Failure::Deadline);
            }
            rows.push((
                vec![score.to_bits().to_string(), id.clone()],
                self.hit(&id, literal)?,
            ));
        }
        self.page(
            "search",
            &key,
            rows,
            more,
            json!({"kind": "search", "query": literal, "instrument": scope.map(|s| &s.slug), "provision_type": kind,
            "semantics": "literal_phrase_accent_insensitive"}),
        )
    }

    fn hit(&self, id: &str, literal: &str) -> Result<Value> {
        let (text, label, number, kind): (String, String, String, String) =
            self.connection.query_row(
                "SELECT text,label,number,provision_type FROM provisions WHERE id=?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )?;
        let scope = self.owner(id).context("search returned an unscoped row")?;
        let (start, end, found) = excerpt_window(&text, literal, self.limits.excerpt_chars);
        let total = text.chars().count();
        Ok(
            json!({"id": id, "instrument": {"slug": scope.slug, "short_name": scope.short_name},
            "provision_type": kind, "number": number, "label": label,
            "excerpt": {"text": text.chars().skip(start).take(end - start).collect::<String>(),
                "start_char": start, "end_char": end, "text_chars": total, "partial": end - start < total},
            "match_location": if found { "text" } else { "label_or_heading" },
            "text_sha256": crate::digest(text.as_bytes()), "artifacts": self.artifacts(scope, id)}),
        )
    }

    fn references(
        &self,
        id: &str,
        direction: Direction,
        page_size: Option<u32>,
        cursor: Option<&str>,
    ) -> Outcome {
        self.valid_id(id)?;
        let Some(scope) = self.owner(id) else {
            return Ok(Self::outside_scope(id));
        };
        let exists = id == scope.id
            || self
                .connection
                .query_row(
                    "SELECT 1 FROM provisions WHERE id=?1 AND instrument_id=?2",
                    params![id, scope.id],
                    |_| Ok(()),
                )
                .optional()?
                .is_some();
        if !exists {
            return self.get_provision(id);
        }
        let size = self.page_size(page_size)?;
        let key = request_key(
            "references",
            &json!({"id": id, "direction": direction, "page_size": size}),
        );
        let after = match cursor {
            Some(c) => Some(self.decode_cursor(c, "references", &key, 1)?.remove(0)),
            None => None,
        };
        let sql = match direction {
            Direction::Outgoing => format!(
                "{EDGE_SQL} WHERE e.source_provision_id=?1 AND ?2 IS NOT NULL AND (?3 IS NULL OR e.id>?3) ORDER BY e.id LIMIT ?4"
            ),
            // Incoming edges are listed only from published citing instruments.
            Direction::Incoming => format!(
                "{EDGE_SQL} WHERE e.target_provision_id=?1 AND a.instrument_id IN (SELECT value FROM json_each(?2)) AND (?3 IS NULL OR e.id>?3) ORDER BY e.id LIMIT ?4"
            ),
        };
        let edges = self
            .connection
            .prepare(&sql)?
            .query_map(params![id, self.allowlist, after, size + 1], |r| {
                Ok([
                    r.get::<_, String>(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                    r.get(6)?,
                ])
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let more = edges.len() > size as usize;
        let mut rows = Vec::new();
        for edge in edges.into_iter().take(size as usize) {
            rows.push((vec![edge[0].clone()], self.edge(&edge)?));
        }
        self.page("references", &key, rows, more, json!({"kind": "references", "id": id, "direction": direction,
            "scope_notice": "Only express, validated edges are listed; incoming edges come from the published selection only. No legal relationship is inferred."}))
    }

    fn edge(&self, edge: &[String; 7]) -> Result<Value> {
        let [
            id,
            source,
            source_kind,
            target_instrument,
            _,
            status,
            extras,
        ] = edge;
        let extras: Value = serde_json::from_str(extras)?;
        let qualifiers = self
            .connection
            .prepare("SELECT payload FROM reference_qualifiers WHERE edge_id=?1 ORDER BY ordinal")?
            .query_map([id], |r| r.get::<_, String>(0))?
            .map(|payload| {
                let q: Value = serde_json::from_str(&payload?)?;
                Ok(
                    json!({"qualifier_type": q["qualifier_type"], "text": q["text"],
                    "start_char": q["start_char"], "end_char": q["end_char"]}),
                )
            })
            .collect::<Result<Vec<_>>>()?;
        let published = self.scopes.iter().any(|s| s.id == *target_instrument);
        let availability = match status.as_str() {
            "resolved" if published => "published",
            "resolved" => "outside_published_scope",
            "unresolved" => "unresolved",
            _ => "historical",
        };
        let mut value = edge_core(edge)?;
        value["source_id"] = json!(source);
        value["source_kind"] = json!(source_kind);
        value["resolution_rule"] = json!(extras.get("resolution_rule"));
        value["qualifiers"] = json!(qualifiers);
        value["target_availability"] = json!(availability);
        Ok(value)
    }
}

fn millis(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

fn request_key(op: &str, params: &Value) -> String {
    crate::digest(
        json!([op, PROJECTION_VERSION, params])
            .to_string()
            .as_bytes(),
    )
}

fn is_hex64(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn truncate(message: &str) -> String {
    message.chars().take(400).collect()
}

fn error(code: &str, message: &str) -> Value {
    json!({"kind": "error", "code": code, "message": truncate(message)})
}

fn no_as_of(as_of: Option<&str>) -> std::result::Result<(), Failure> {
    match as_of {
        Some(_) => Err(client(
            "as_of_not_supported",
            "historical applicability is not established; only the bound current release is served",
        )),
        None => Ok(()),
    }
}

fn valid_kind(kind: &str) -> std::result::Result<&'static str, Failure> {
    KINDS
        .iter()
        .find(|k| **k == kind)
        .copied()
        .ok_or_else(|| client("bad_request", "kind must be article, transitory or annex"))
}

fn public_condition(provision: &lex_core::Provision) -> Value {
    provision
        .commencement_condition
        .as_ref()
        .map_or(Value::Null, |condition| {
            json!({
                "source_provision_id": condition.source_provision_id,
                "condition_text": condition.condition_text, "authority": condition.authority,
                "condition_status": condition.condition_status, "met_on": condition.met_on
            })
        })
}

/// Folded word tokens as `(start_char, end_char, folded)`; one character
/// folds to one character, so offsets stay aligned with the original text.
fn tokens(text: &str) -> Vec<(usize, usize, String)> {
    let mut out: Vec<(usize, usize, String)> = Vec::new();
    let mut current: Option<(usize, String)> = None;
    for (index, character) in text.chars().enumerate() {
        if character.is_alphanumeric() {
            let folded =
                strip_accents(&character.to_lowercase().collect::<String>()).replace('ñ', "n");
            current
                .get_or_insert_with(|| (index, String::new()))
                .1
                .push_str(&folded);
        } else if let Some((start, word)) = current.take() {
            out.push((start, index, word));
        }
    }
    if let Some((start, word)) = current {
        out.push((start, text.chars().count(), word));
    }
    out
}

/// Character window around the first token-sequence match, else the start.
fn excerpt_window(text: &str, literal: &str, width: usize) -> (usize, usize, bool) {
    let total = text.chars().count();
    let width = width.min(total);
    let words = tokens(text);
    let wanted = tokens(literal);
    let found = (!wanted.is_empty())
        .then(|| {
            words
                .windows(wanted.len())
                .find(|window| window.iter().zip(&wanted).all(|(a, b)| a.2 == b.2))
        })
        .flatten()
        .map(|window| window[0].0);
    let start = found.map_or(0, |m| m.saturating_sub(width / 4).min(total - width));
    (start, start + width, found.is_some())
}

/// Write a new binding file; an existing binding is never overwritten.
pub fn write_binding(binding: &Binding, output: &Path) -> Result<()> {
    use std::io::Write;
    let mut bytes = serde_json::to_vec_pretty(binding)?;
    bytes.push(b'\n');
    // create_new makes the no-clobber check and the creation one step, so a
    // file appearing between a check and the write is never overwritten.
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                anyhow::anyhow!("binding output already exists: {}", output.display())
            } else {
                error.into()
            }
        })?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    Ok(())
}

#[cfg(test)]
#[path = "public_tests.rs"]
mod tests;
