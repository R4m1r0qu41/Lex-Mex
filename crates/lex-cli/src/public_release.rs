//! Read-only, allowlisted publication over a committed corpus selection.
use anyhow::{Context, Result, bail};
use clap::Subcommand;
use lex_core::{Instrument, Provision, ReferenceEdge, SourceManifest};
use lex_source::sha256_hex;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fmt::Write,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Subcommand)]
pub(crate) enum PublicReleaseCommand {
    /// Build a new immutable publication directory; never overwrites an output.
    Build {
        #[arg(long, required = true, value_delimiter = ',')]
        instrument: Vec<String>,
        /// Directory of PDFs named by their exact source SHA-256.
        #[arg(long)]
        source_store: Option<PathBuf>,
        #[arg(long)]
        output: PathBuf,
        /// Public URL prefix, with leading and trailing slash.
        #[arg(long, default_value = "/lex-mex/")]
        base_path: String,
    },
}

struct Selected {
    slug: String,
    instrument: Instrument,
    provisions: Vec<Provision>,
    references: Vec<ReferenceEdge>,
    source: SourceManifest,
    annex_sources: BTreeMap<String, SourceManifest>,
    input_hashes: BTreeMap<String, String>,
}

pub(crate) fn run(root: &Path, command: PublicReleaseCommand) -> Result<()> {
    let PublicReleaseCommand::Build {
        instrument,
        source_store,
        output,
        base_path,
    } = command;
    let slugs = super::bundle::normalized_slugs(&instrument)?;
    super::bundle::ensure_selected_corpora_are_committed(root, &slugs)?;
    let (commit, committed_at) = super::bundle::git_provenance(root)?;
    validate_base_path(&base_path)?;
    let selected = slugs
        .iter()
        .map(|slug| {
            let context = super::instrument_context(root, slug)?;
            let report = super::validate_corpus_snapshot(root, &context)?;
            if !report.valid {
                bail!("current structural validation failed for {slug}");
            }
            load_selected(root, slug)
        })
        .collect::<Result<Vec<_>>>()?;
    // A second Git check prevents reading a known dirty selection during assembly.
    super::bundle::ensure_selected_corpora_are_committed(root, &slugs)?;
    let id = build(
        &selected,
        source_store.as_deref(),
        &output,
        &base_path,
        &commit,
        &committed_at,
    )?;
    println!("public release: {id}\noutput: {}", output.display());
    Ok(())
}

fn validate_base_path(base: &str) -> Result<()> {
    if !base.starts_with('/')
        || !base.ends_with('/')
        || base.contains("..")
        || base.contains("//")
        || !base
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"/-_".contains(&b))
    {
        bail!("base path must be a safe absolute path with a trailing slash");
    }
    Ok(())
}

fn read<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    serde_json::from_slice(&fs::read(path).with_context(|| format!("read {}", path.display()))?)
        .with_context(|| format!("parse {}", path.display()))
}

fn load_selected(root: &Path, slug: &str) -> Result<Selected> {
    let dir = root.join("corpus/mx").join(slug);
    let instrument: Instrument = read(&dir.join("instrument.json"))?;
    let source: SourceManifest = read(&dir.join("source-manifest.json"))?;
    if source.instrument_id != instrument.id
        || source.source_sha256 != instrument.source_sha256
        || source.extracted_text_sha256.as_deref()
            != Some(instrument.extracted_text_sha256.as_str())
    {
        bail!("instrument and source manifest disagree for {slug}");
    }
    let names = [
        "instrument.json",
        "provisions.json",
        "references.json",
        "source-manifest.json",
        "validation.json",
    ];
    let mut input_hashes: BTreeMap<String, String> = names
        .iter()
        .map(|name| Ok(((*name).to_owned(), sha256_hex(&fs::read(dir.join(name))?))))
        .collect::<Result<_>>()?;
    let context = super::instrument_context(root, slug)?;
    let mut annex_sources = BTreeMap::new();
    if !context.config.annex_pdf_urls.is_empty() {
        let path = dir.join("annex-source-manifests.json");
        let manifests: Vec<SourceManifest> = read(&path)?;
        input_hashes.insert(
            "annex-source-manifests.json".to_owned(),
            sha256_hex(&fs::read(&path)?),
        );
        for (index, url) in context.config.annex_pdf_urls.iter().enumerate() {
            let matches = manifests
                .iter()
                .filter(|item| &item.official_url == url)
                .collect::<Vec<_>>();
            if matches.len() != 1 {
                bail!("annex source mapping is ambiguous or missing for {slug}");
            }
            annex_sources.insert((index + 1).to_string(), matches[0].clone());
        }
    }
    Ok(Selected {
        slug: slug.to_owned(),
        instrument,
        source,
        annex_sources,
        input_hashes,
        provisions: read(&dir.join("provisions.json"))?,
        references: read(&dir.join("references.json"))?,
    })
}

fn json_bytes(value: &Value) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn stem(id: &str) -> String {
    format!("provision-{}", sha256_hex(id.as_bytes()))
}

fn public_condition(provision: &Provision) -> Value {
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

fn public_provision(provision: &Provision, source: &Value) -> Value {
    let file = stem(&provision.id);
    json!({"id": provision.id, "label": provision.label, "number": provision.number,
        "provision_type": provision.provision_type, "heading_context": provision.heading_context,
        "text": provision.text, "text_sha256": sha256_hex(provision.text.as_bytes()),
        "publication_date": provision.publication_date, "effective_from": provision.effective_from,
        "effective_to": provision.effective_to, "temporal_status": provision.temporal_status,
        "temporal_basis": provision.temporal_basis, "review_status": provision.review_status,
        "repeals": provision.repeals, "commencement_condition": public_condition(provision),
        "markdown_path": format!("{file}.md"), "text_path": format!("{file}.txt"),
        "pdf_page": Value::Null, "source": source})
}

fn public_reference(edge: &ReferenceEdge, published: &BTreeMap<String, String>) -> Value {
    json!({"id": edge.id, "source_provision_id": edge.source_provision_id,
        "source_span": edge.source_span, "start_char": edge.start_char, "end_char": edge.end_char,
        "target_instrument_id": edge.target_instrument_id, "target_provision_id": edge.target_provision_id,
        "resolution_status": edge.resolution_status, "reference_form": edge.reference_form,
        "basis": edge.basis, "target_published": published.contains_key(&edge.target_provision_id)})
}

fn source_info(
    source: &SourceManifest,
    slug: &str,
    files: &mut BTreeMap<String, Vec<u8>>,
    store: Option<&Path>,
) -> Result<Value> {
    if source.source_sha256.len() != 64
        || !source
            .source_sha256
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        bail!("invalid source digest for {slug}");
    }
    let mut pdf_path = None;
    if let Some(store) = store {
        let path = store.join(format!("{}.pdf", source.source_sha256));
        if path.exists() {
            let bytes = fs::read(&path)?;
            if !bytes.starts_with(b"%PDF-") || sha256_hex(&bytes) != source.source_sha256 {
                bail!("source PDF digest or format mismatch for {slug}");
            }
            let relative = format!("sources/{}.pdf", source.source_sha256);
            files.insert(relative.clone(), bytes);
            pdf_path = Some(relative);
        }
    }
    Ok(
        json!({"source_sha256": source.source_sha256, "extracted_text_sha256": source.extracted_text_sha256,
        "publisher": source.publisher, "official_url": source.official_url,
        "retrieved_at": source.retrieved_at, "parser_version": source.parser_version,
        "schema_version": source.schema_version_used, "pdf_path": pdf_path,
        "pdf_verified": pdf_path.is_some(), "last_successful_upstream_check": Value::Null}),
    )
}

fn assemble_instrument(
    selected: &Selected,
    published: &BTreeMap<String, String>,
    files: &mut BTreeMap<String, Vec<u8>>,
    store: Option<&Path>,
) -> Result<Value> {
    let prefix = format!("mx/{}", selected.slug);
    let source = source_info(&selected.source, &selected.slug, files, store)?;
    let mut index = format!(
        "# {}\n\nSource: {}\n\nSource SHA-256: `{}`\n\n",
        selected.instrument.official_title,
        selected.source.official_url,
        selected.source.source_sha256
    );
    let mut annex_info = BTreeMap::new();
    for (number, manifest) in &selected.annex_sources {
        annex_info.insert(
            number.clone(),
            source_info(manifest, &selected.slug, files, store)?,
        );
    }
    let mut provisions = Vec::new();
    for provision in &selected.provisions {
        let info = if provision.provision_type == lex_core::ProvisionType::Annex
            && !annex_info.is_empty()
        {
            annex_info
                .get(&provision.number)
                .with_context(|| format!("missing source for annex {}", provision.id))?
        } else {
            &source
        };
        provisions.push(public_provision(provision, info));
    }
    for (provision, public) in selected.provisions.iter().zip(&provisions) {
        let backing = &public["source"];
        let official = backing["official_url"]
            .as_str()
            .context("missing public source URL")?;
        let digest = backing["source_sha256"]
            .as_str()
            .context("missing public source digest")?;
        let name = stem(&provision.id);
        writeln!(index, "- [{}]({name}.md)", provision.label)?;
        let mut md = format!(
            "# {}\n\nInstrument: {}\n\nStable ID: `{}`\n\nSource: {}\n\nSource SHA-256: `{}`\n\nStatus: {} (qualified repository classification, not a guarantee of legal validity).\n\n## Source text\n\n{}\n",
            provision.label,
            selected.instrument.official_title,
            provision.id,
            official,
            digest,
            serde_json::to_value(&provision.temporal_status)?,
            provision.text
        );
        let refs = selected
            .references
            .iter()
            .filter(|edge| edge.source_provision_id == provision.id)
            .collect::<Vec<_>>();
        if !refs.is_empty() {
            md.push_str("\n## Recorded express references\n\n");
            for edge in refs {
                if let Some(slug) = published.get(&edge.target_provision_id).filter(|_| {
                    edge.resolution_status == lex_core::ReferenceResolutionStatus::Resolved
                }) {
                    writeln!(
                        md,
                        "- [{}](../{}/{}.md): `{}`",
                        edge.source_span,
                        slug,
                        stem(&edge.target_provision_id),
                        edge.target_provision_id
                    )?;
                } else {
                    writeln!(
                        md,
                        "- {}: `{}` (unresolved, historical, or outside this selection)",
                        edge.source_span, edge.target_provision_id
                    )?;
                }
            }
        }
        files.insert(format!("{prefix}/{name}.md"), md.into_bytes());
        // Exact canonical body: deliberately no added newline, headings or links.
        let text_bytes = provision.text.as_bytes().to_vec();
        files.insert(format!("{prefix}/{name}.txt"), text_bytes);
    }
    files.insert(format!("{prefix}/index.md"), index.into_bytes());
    let document = json!({"schema_version": "public-instrument-v1", "slug": selected.slug,
        "id": selected.instrument.id, "official_title": selected.instrument.official_title,
        "short_name": selected.instrument.short_name, "publication_date": selected.instrument.publication_date,
        "latest_reform_date": selected.instrument.latest_reform_date, "source": source,
        "provisions": provisions, "references": selected.references.iter().map(|edge| public_reference(edge, published)).collect::<Vec<_>>(),
        "temporal_effects_included": false});
    validate_schema(
        &document,
        include_str!("../../../schemas/public-instrument-v1.schema.json"),
    )?;
    files.insert(format!("{prefix}/instrument.json"), json_bytes(&document)?);
    Ok(
        json!({"slug": selected.slug, "id": selected.instrument.id, "title": selected.instrument.official_title,
        "short_name": selected.instrument.short_name, "provision_count": selected.provisions.len(),
        "document_path": format!("{prefix}/instrument.json"), "source": source,
        "canonical_input_hashes": selected.input_hashes, "coverage": {"structured": true,
            "references": "express_with_disclosed_unresolved_targets", "historical_validity": "not_established"}}),
    )
}

fn validate_schema(value: &Value, text: &str) -> Result<()> {
    let schema: Value = serde_json::from_str(text)?;
    jsonschema::validator_for(&schema)?
        .validate(value)
        .map_err(|error| anyhow::anyhow!(error.to_string()))
}

fn build(
    selected: &[Selected],
    store: Option<&Path>,
    output: &Path,
    base: &str,
    commit: &str,
    committed_at: &str,
) -> Result<String> {
    if output.exists() {
        bail!("publication output already exists: {}", output.display());
    }
    let published = selected
        .iter()
        .flat_map(|item| {
            item.provisions
                .iter()
                .map(|p| (p.id.clone(), item.slug.clone()))
        })
        .collect::<BTreeMap<_, _>>();
    if published.len()
        != selected
            .iter()
            .map(|item| item.provisions.len())
            .sum::<usize>()
    {
        bail!("duplicate public provision IDs");
    }
    let mut files = BTreeMap::new();
    let instruments = selected
        .iter()
        .map(|item| assemble_instrument(item, &published, &mut files, store))
        .collect::<Result<Vec<_>>>()?;
    let mut index = "# Lex-Mex selected corpus\n\nPartial coverage. Pin this release; do not infer historical validity or source freshness.\n\n".to_owned();
    for item in selected {
        writeln!(
            index,
            "- [{}](mx/{}/index.md)",
            item.instrument.short_name, item.slug
        )?;
    }
    files.insert("index.md".to_owned(), index.into_bytes());
    files.insert(
        "coverage.json".to_owned(),
        json_bytes(&json!({"scope": "selected_partial", "instruments": instruments}))?,
    );
    let payload = json!({"schema_version": "public-release-v1", "builder_sha256": sha256_hex(include_bytes!("public_release.rs")),
        "corpus_commit": commit, "committed_at": committed_at, "base_path": base,
        "coverage": "selected_partial", "instruments": instruments,
        "files": files.iter().map(|(path, bytes)| json!({"path": path, "sha256": sha256_hex(bytes), "bytes": bytes.len()})).collect::<Vec<_>>()});
    let id = sha256_hex(&json_bytes(&payload)?);
    let mut manifest = payload;
    manifest["release_id"] = json!(id);
    validate_schema(
        &manifest,
        include_str!("../../../schemas/public-release-v1.schema.json"),
    )?;
    let parent = output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    let staging = tempfile::tempdir_in(parent)?;
    let release = staging.path().join("releases").join(&id);
    for (path, bytes) in &files {
        let target = release.join(path);
        fs::create_dir_all(target.parent().context("missing publication parent")?)?;
        fs::write(target, bytes)?;
    }
    fs::write(release.join("manifest.json"), json_bytes(&manifest)?)?;
    fs::write(
        staging.path().join("latest.json"),
        json_bytes(
            &json!({"release_id": id, "manifest_path": format!("releases/{id}/manifest.json")}),
        )?,
    )?;
    let index_url = format!("{base}releases/{id}/index.md");
    fs::write(
        staging.path().join("llms.txt"),
        format!(
            "# Lex-Mex\n\n> Selected Mexican legal corpus. Partial coverage; repository classifications are qualified.\n\nPin release `{id}`. Background text retrieval must not change the human reader selection. Read only relevant shards; do not bulk-load the corpus.\n\n## Corpus\n\n- [Selected corpus]({index_url}): per-instrument Markdown indexes and exact text links.\n- [Manifest]({base}releases/{id}/manifest.json): source snapshots, coverage, input and file hashes.\n\nNo last-successful-upstream-check ledger or historical validity guarantee is available.\n"
        ),
    )?;
    // Reserve a new destination; never replace an existing file or directory.
    fs::create_dir(output)?;
    fs::rename(staging.path(), output).context("promote staged public release")?;
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_store_rejects_wrong_bytes_before_output_is_created() {
        let temp = tempfile::tempdir().unwrap();
        let selected = fixture();
        let store = temp.path().join("sources");
        fs::create_dir(&store).unwrap();
        fs::write(
            store.join(format!("{}.pdf", selected.source.source_sha256)),
            b"%PDF-wrong",
        )
        .unwrap();
        let output = temp.path().join("out");
        assert!(
            build(
                &[selected],
                Some(&store),
                &output,
                "/lex-mex/",
                "commit",
                "date"
            )
            .is_err()
        );
        assert!(!output.exists());
    }
    #[test]
    fn release_is_reproducible_exact_and_excludes_review_artifacts() {
        let temp = tempfile::tempdir().unwrap();
        let selected = fixture();
        let out = temp.path().join("one");
        let other = temp.path().join("two");
        let id = build(&[selected], None, &out, "/lex-mex/", "commit", "date").unwrap();
        let selected = fixture();
        assert_eq!(
            id,
            build(&[selected], None, &other, "/lex-mex/", "commit", "date").unwrap()
        );
        let dir = out.join("releases").join(&id);
        let manifest: Value = read(&dir.join("manifest.json")).unwrap();
        assert!(
            !manifest["instruments"][0]["source"]["pdf_verified"]
                .as_bool()
                .unwrap()
        );
        assert!(manifest["instruments"][0]["source"]["last_successful_upstream_check"].is_null());
        let data: Value = read(&dir.join("mx/lritf/instrument.json")).unwrap();
        let txt = data["provisions"][0]["text_path"].as_str().unwrap();
        assert_eq!(
            fs::read_to_string(dir.join("mx/lritf").join(txt)).unwrap(),
            fixture().provisions[0].text
        );
        assert!(data["provisions"][0].get("transitory_effects").is_none());
        assert!(data["provisions"][0].get("reviewer").is_none());
        assert!(!dir.join("review-queue.json").exists());
        for file in manifest["files"].as_array().unwrap() {
            assert_eq!(
                sha256_hex(&fs::read(dir.join(file["path"].as_str().unwrap())).unwrap()),
                file["sha256"].as_str().unwrap()
            );
        }
        assert!(build(&[fixture()], None, &out, "/lex-mex/", "commit", "date").is_err());
    }
    #[test]
    fn source_store_includes_only_matching_pdf() {
        let temp = tempfile::tempdir().unwrap();
        let mut selected = fixture();
        let pdf = b"%PDF-1.7\nfixture";
        selected.source.source_sha256 = sha256_hex(pdf);
        selected.instrument.source_sha256 = sha256_hex(pdf);
        let store = temp.path().join("sources");
        fs::create_dir(&store).unwrap();
        fs::write(store.join(format!("{}.pdf", sha256_hex(pdf))), pdf).unwrap();
        let out = temp.path().join("out");
        let id = build(
            &[selected],
            Some(&store),
            &out,
            "/lex-mex/",
            "commit",
            "date",
        )
        .unwrap();
        assert_eq!(
            fs::read(out.join(format!("releases/{id}/sources/{}.pdf", sha256_hex(pdf)))).unwrap(),
            pdf
        );
    }
    #[test]
    fn unsafe_prefix_and_duplicate_ids_fail() {
        for base in ["relative/", "//evil/", "/../", "/lex-mex/?q="] {
            assert!(validate_base_path(base).is_err());
        }
        let temp = tempfile::tempdir().unwrap();
        assert!(
            build(
                &[fixture(), fixture()],
                None,
                &temp.path().join("out"),
                "/lex-mex/",
                "commit",
                "date"
            )
            .is_err()
        );
    }
    #[test]
    fn annex_uses_its_own_retained_source_and_markdown_identity() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../");
        let mut selected = load_selected(&root, "ifpe-dcg-2021").unwrap();
        selected
            .provisions
            .retain(|p| p.provision_type == lex_core::ProvisionType::Annex);
        selected.provisions.truncate(1);
        selected.references.clear();
        let number = selected.provisions[0].number.clone();
        let annex = selected.annex_sources.get_mut(&number).unwrap();
        let pdf = b"%PDF-1.7\nannex regression fixture";
        annex.source_sha256 = sha256_hex(pdf);
        let official = annex.official_url.to_string();
        let temp = tempfile::tempdir().unwrap();
        let store = temp.path().join("sources");
        fs::create_dir(&store).unwrap();
        fs::write(store.join(format!("{}.pdf", sha256_hex(pdf))), pdf).unwrap();
        let output = temp.path().join("out");
        let id = build(
            &[selected],
            Some(&store),
            &output,
            "/lex-mex/",
            "commit",
            "date",
        )
        .unwrap();
        let release = output.join("releases").join(id);
        let document: Value = read(&release.join("mx/ifpe-dcg-2021/instrument.json")).unwrap();
        let provision = &document["provisions"][0];
        assert_ne!(
            provision["source"]["source_sha256"],
            document["source"]["source_sha256"]
        );
        assert_eq!(provision["source"]["source_sha256"], sha256_hex(pdf));
        assert_eq!(provision["source"]["official_url"], official);
        assert_eq!(provision["source"]["pdf_verified"], true);
        let markdown = fs::read_to_string(
            release
                .join("mx/ifpe-dcg-2021")
                .join(provision["markdown_path"].as_str().unwrap()),
        )
        .unwrap();
        assert!(markdown.contains(&official));
        assert!(markdown.contains(&sha256_hex(pdf)));
    }
    fn fixture() -> Selected {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../");
        let mut selected = load_selected(&root, "lritf").unwrap();
        selected.provisions.truncate(1);
        selected.references.clear();
        selected
    }
}
