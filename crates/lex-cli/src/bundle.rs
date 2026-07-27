use std::{
    collections::{BTreeSet, HashSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};
use clap::{Subcommand, ValueEnum};
use lex_core::{
    Instrument, ReferenceEdge, ReferenceResolutionStatus, SCHEMA_VERSION, StandardMetadata,
};
use lex_source::sha256_hex;
use serde::Serialize;

const BUNDLE_SCHEMA_VERSION: &str = "0.1.0";
const CANONICAL_FILES: &[&str] = &[
    "instrument.json",
    "provisions.json",
    "references.json",
    "terms.json",
    "term-usages.json",
    "amendment-references.json",
    "source-manifest.json",
    "formal-source-manifest.json",
    "annex-source-manifests.json",
    "temporal-analysis-result.json",
    "review-queue.json",
    "reform-temporal-evidence.json",
    "validation.json",
];
const STANDARD_CANONICAL_FILES: &[&str] = &[
    "standard.json",
    "clauses.json",
    "transitories.json",
    "extracted-text.txt",
    "validation.json",
];

#[derive(Debug, Subcommand)]
pub(crate) enum BundleCommand {
    /// Copy selected committed instruments and write a provenance manifest.
    Create {
        /// Instrument slugs to include (comma-separated or repeated).
        #[arg(long, required = true, value_delimiter = ',')]
        instrument: Vec<String>,
        #[arg(long, value_enum, default_value = "canonical")]
        profile: BundleProfile,
        /// New directory to create. Existing paths are never overwritten.
        #[arg(long)]
        output: PathBuf,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub(crate) enum BundleProfile {
    Canonical,
    CanonicalMarkdown,
}

#[derive(Debug, Serialize)]
struct BundleManifest {
    schema_version: &'static str,
    corpus_schema_version: &'static str,
    profile: BundleProfile,
    corpus_commit: String,
    generated_at: String,
    instruments: Vec<BundleInstrument>,
    excluded_external_target_instrument_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
struct BundleInstrument {
    slug: String,
    id: String,
    source_sha256: String,
    extracted_text_sha256: String,
    validation_sha256: String,
    files: Vec<BundleFile>,
}

#[derive(Debug, Serialize)]
struct BundleFile {
    path: String,
    sha256: String,
    bytes: u64,
}

struct SelectedInstrument {
    source_dir: PathBuf,
    manifest: BundleInstrument,
    external_targets: BTreeSet<String>,
}

pub(crate) fn run_bundle_command(root: &Path, command: BundleCommand) -> Result<()> {
    match command {
        BundleCommand::Create {
            instrument,
            profile,
            output,
        } => {
            let (commit, commit_time) = git_provenance(root)?;
            ensure_selected_corpora_are_committed(root, &instrument)?;
            create_bundle(root, &instrument, profile, &output, &commit, &commit_time)?;
            let manifest_path = output.join("bundle-manifest.json");
            let manifest_bytes = fs::read(&manifest_path)?;
            println!(
                "{}\t{}",
                manifest_path.display(),
                sha256_hex(&manifest_bytes)
            );
            Ok(())
        }
    }
}

fn create_bundle(
    root: &Path,
    requested_slugs: &[String],
    profile: BundleProfile,
    output: &Path,
    commit: &str,
    commit_time: &str,
) -> Result<BundleManifest> {
    if output.exists() {
        bail!(
            "bundle output already exists; refusing to overwrite {}",
            output.display()
        );
    }
    let slugs = normalized_slugs(requested_slugs)?;
    let selected_ids = selected_instrument_ids(root, &slugs)?;
    let mut selected = slugs
        .iter()
        .map(|slug| collect_instrument(root, slug, profile, &selected_ids))
        .collect::<Result<Vec<_>>>()?;

    let excluded_external_target_instrument_ids = selected
        .iter()
        .flat_map(|instrument| instrument.external_targets.iter().cloned())
        .collect();

    fs::create_dir_all(output)
        .with_context(|| format!("failed to create bundle output {}", output.display()))?;
    for instrument in &selected {
        for file in &instrument.manifest.files {
            let relative = Path::new(&file.path);
            let source_relative = relative
                .strip_prefix(Path::new("instruments").join(&instrument.manifest.slug))
                .context("bundle file escaped its instrument directory")?;
            let destination = output.join(relative);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(instrument.source_dir.join(source_relative), &destination)?;
            let copied = fs::read(&destination)?;
            if sha256_hex(&copied) != file.sha256 {
                bail!(
                    "copied bundle file failed its digest check: {}",
                    destination.display()
                );
            }
        }
    }

    let manifest = BundleManifest {
        schema_version: BUNDLE_SCHEMA_VERSION,
        corpus_schema_version: SCHEMA_VERSION,
        profile,
        corpus_commit: commit.to_owned(),
        generated_at: commit_time.to_owned(),
        instruments: selected
            .drain(..)
            .map(|instrument| instrument.manifest)
            .collect(),
        excluded_external_target_instrument_ids,
    };
    let mut bytes = serde_json::to_vec_pretty(&manifest)?;
    bytes.push(b'\n');
    fs::write(output.join("bundle-manifest.json"), bytes)?;
    Ok(manifest)
}

fn normalized_slugs(requested: &[String]) -> Result<Vec<String>> {
    let mut seen = HashSet::new();
    let mut slugs = Vec::with_capacity(requested.len());
    for slug in requested {
        if !is_slug(slug) {
            bail!("instrument slug must be one path component, got {slug:?}");
        }
        if !seen.insert(slug.clone()) {
            bail!("instrument slug was selected more than once: {slug}");
        }
        slugs.push(slug.clone());
    }
    slugs.sort();
    Ok(slugs)
}

fn is_slug(slug: &str) -> bool {
    !slug.is_empty()
        && slug
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn selected_instrument_ids(root: &Path, slugs: &[String]) -> Result<HashSet<String>> {
    slugs
        .iter()
        .map(|slug| {
            let directory = root.join("corpus/mx").join(slug);
            if directory.join("instrument.json").is_file() {
                let instrument: Instrument = read_json(&directory.join("instrument.json"))?;
                Ok(instrument.id)
            } else {
                let standard: StandardMetadata = read_json(&directory.join("standard.json"))?;
                Ok(standard.id)
            }
        })
        .collect()
}

fn collect_instrument(
    root: &Path,
    slug: &str,
    profile: BundleProfile,
    selected_ids: &HashSet<String>,
) -> Result<SelectedInstrument> {
    let source_dir = root.join("corpus/mx").join(slug);
    if source_dir.join("standard.json").is_file() {
        return collect_standard(source_dir, slug, profile);
    }
    if !source_dir.join("instrument.json").is_file() {
        bail!("no committed corpus found for instrument {slug:?}");
    }
    let instrument: Instrument = read_json(&source_dir.join("instrument.json"))?;
    let validation_path = source_dir.join("validation.json");
    let validation: serde_json::Value = read_json(&validation_path)?;
    if validation.get("valid").and_then(serde_json::Value::as_bool) != Some(true) {
        bail!("instrument {slug} does not have a passing validation.json");
    }

    let mut relative_files = CANONICAL_FILES
        .iter()
        .map(PathBuf::from)
        .filter(|path| source_dir.join(path).is_file())
        .collect::<Vec<_>>();
    for required in [
        "instrument.json",
        "provisions.json",
        "references.json",
        "validation.json",
    ] {
        if !relative_files
            .iter()
            .any(|path| path == Path::new(required))
        {
            bail!("instrument {slug} is missing required bundle file {required}");
        }
    }
    if profile == BundleProfile::CanonicalMarkdown {
        collect_files(
            &source_dir.join("markdown"),
            Path::new("markdown"),
            &mut relative_files,
        )?;
    }
    relative_files.sort();

    let files = relative_files
        .iter()
        .map(|relative| {
            let bytes = fs::read(source_dir.join(relative))?;
            Ok(BundleFile {
                path: Path::new("instruments")
                    .join(slug)
                    .join(relative)
                    .to_string_lossy()
                    .into_owned(),
                sha256: sha256_hex(&bytes),
                bytes: u64::try_from(bytes.len()).context("bundle file is too large")?,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let references: Vec<ReferenceEdge> = read_json(&source_dir.join("references.json"))?;
    let external_targets = references
        .into_iter()
        .filter(|edge| edge.resolution_status == ReferenceResolutionStatus::Resolved)
        .map(|edge| edge.target_instrument_id)
        .filter(|target| !selected_ids.contains(target))
        .collect();
    let validation_sha256 = sha256_hex(&fs::read(validation_path)?);

    Ok(SelectedInstrument {
        source_dir,
        manifest: BundleInstrument {
            slug: slug.to_owned(),
            id: instrument.id,
            source_sha256: instrument.source_sha256,
            extracted_text_sha256: instrument.extracted_text_sha256,
            validation_sha256,
            files,
        },
        external_targets,
    })
}

fn collect_standard(
    source_dir: PathBuf,
    slug: &str,
    profile: BundleProfile,
) -> Result<SelectedInstrument> {
    if profile == BundleProfile::CanonicalMarkdown {
        bail!("standard {slug} has no generated Markdown profile");
    }
    let standard: StandardMetadata = read_json(&source_dir.join("standard.json"))?;
    let validation_path = source_dir.join("validation.json");
    let validation: serde_json::Value = read_json(&validation_path)?;
    if validation.get("valid").and_then(serde_json::Value::as_bool) != Some(true) {
        bail!("standard {slug} does not have a passing validation.json");
    }
    let relative_files = STANDARD_CANONICAL_FILES
        .iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    for required in &relative_files {
        if !source_dir.join(required).is_file() {
            bail!(
                "standard {slug} is missing required bundle file {}",
                required.display()
            );
        }
    }
    let files = relative_files
        .iter()
        .map(|relative| {
            let bytes = fs::read(source_dir.join(relative))?;
            Ok(BundleFile {
                path: Path::new("instruments")
                    .join(slug)
                    .join(relative)
                    .to_string_lossy()
                    .into_owned(),
                sha256: sha256_hex(&bytes),
                bytes: u64::try_from(bytes.len()).context("bundle file is too large")?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let validation_sha256 = sha256_hex(&fs::read(validation_path)?);

    Ok(SelectedInstrument {
        source_dir,
        manifest: BundleInstrument {
            slug: slug.to_owned(),
            id: standard.id,
            source_sha256: standard.source_sha256,
            extracted_text_sha256: standard.extracted_text_sha256,
            validation_sha256,
            files,
        },
        external_targets: BTreeSet::new(),
    })
}

fn collect_files(directory: &Path, relative: &Path, output: &mut Vec<PathBuf>) -> Result<()> {
    if !directory.is_dir() {
        bail!(
            "requested Markdown profile but {} is missing",
            directory.display()
        );
    }
    let mut entries = fs::read_dir(directory)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let child_relative = relative.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            collect_files(&entry.path(), &child_relative, output)?;
        } else if entry.file_type()?.is_file() {
            output.push(child_relative);
        }
    }
    Ok(())
}

fn git_provenance(root: &Path) -> Result<(String, String)> {
    let output = Command::new("git")
        .args(["show", "-s", "--format=%H%n%cI", "HEAD"])
        .current_dir(root)
        .output()
        .context("failed to read Git provenance for the corpus bundle")?;
    if !output.status.success() {
        bail!(
            "failed to read Git provenance: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let text = String::from_utf8(output.stdout)?;
    let mut lines = text.lines();
    let commit = lines.next().context("Git did not return a commit hash")?;
    let commit_time = lines.next().context("Git did not return a commit time")?;
    Ok((commit.to_owned(), commit_time.to_owned()))
}

fn ensure_selected_corpora_are_committed(root: &Path, slugs: &[String]) -> Result<()> {
    let normalized = normalized_slugs(slugs)?;
    let paths = normalized
        .iter()
        .map(|slug| format!("corpus/mx/{slug}"))
        .collect::<Vec<_>>();
    let output = Command::new("git")
        .args(["status", "--porcelain=v1", "--untracked-files=all", "--"])
        .args(&paths)
        .current_dir(root)
        .output()
        .context("failed to inspect selected corpus Git state")?;
    if !output.status.success() {
        bail!("git status failed while checking selected corpus paths");
    }
    if !output.stdout.is_empty() {
        bail!(
            "selected corpus contains changes not represented by HEAD:\n{}",
            String::from_utf8_lossy(&output.stdout).trim_end()
        );
    }
    Ok(())
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("failed to parse {}", path.display()))
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use chrono::{NaiveDate, Utc};
    use lex_core::{
        Corpus, HeadingContext, Instrument, InstrumentStatus, InstrumentType, Provision,
        ProvisionType, ReviewStatus, SCHEMA_VERSION, StandardKind, StandardMetadata,
        StandardStatus, StandardTextBasis, StandardValidationReport, TechnicalReviewStatus,
        TemporalStatus,
    };
    use lex_export::{write_canonical, write_validation};
    use url::Url;

    use super::{BundleProfile, create_bundle};

    #[test]
    fn selected_bundle_is_sorted_hashed_and_refuses_overwrite() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path();
        write_fixture(root, "second");
        write_fixture(root, "first");
        let output = root.join("out");

        let manifest = create_bundle(
            root,
            &["second".to_owned(), "first".to_owned()],
            BundleProfile::Canonical,
            &output,
            "abc123",
            "2026-07-25T12:00:00-06:00",
        )
        .unwrap();

        assert_eq!(manifest.instruments[0].slug, "first");
        assert_eq!(manifest.instruments[1].slug, "second");
        assert!(output.join("instruments/first/instrument.json").is_file());
        assert!(output.join("bundle-manifest.json").is_file());
        assert!(
            create_bundle(
                root,
                &["first".to_owned()],
                BundleProfile::Canonical,
                &output,
                "abc123",
                "2026-07-25T12:00:00-06:00",
            )
            .is_err()
        );
    }

    #[test]
    fn selected_bundle_includes_standard_specific_canonical_files() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path();
        write_standard_fixture(root, "nom-999-test-2026");
        let output = root.join("standard-out");

        let manifest = create_bundle(
            root,
            &["nom-999-test-2026".to_owned()],
            BundleProfile::Canonical,
            &output,
            "abc123",
            "2026-07-25T12:00:00-06:00",
        )
        .unwrap();

        assert_eq!(
            manifest.instruments[0].id,
            "urn:lex-mx:federal:nom:nom-999-test-2026"
        );
        assert!(
            output
                .join("instruments/nom-999-test-2026/standard.json")
                .is_file()
        );
        assert!(
            output
                .join("instruments/nom-999-test-2026/extracted-text.txt")
                .is_file()
        );
        assert!(
            output
                .join("instruments/nom-999-test-2026/transitories.json")
                .is_file()
        );
    }

    fn write_fixture(root: &Path, slug: &str) {
        let instrument_id = format!("urn:lex-mx:federal:statute:{slug}");
        let corpus = Corpus {
            instrument: Instrument {
                schema_version: SCHEMA_VERSION.to_owned(),
                id: instrument_id.clone(),
                jurisdiction: "MX".to_owned(),
                level: "federal".to_owned(),
                instrument_type: InstrumentType::Statute,
                official_title: format!("Ley {slug}"),
                short_name: slug.to_uppercase(),
                operational_source: "fixture".to_owned(),
                formal_publication_source: "fixture".to_owned(),
                publication_date: NaiveDate::from_ymd_opt(2026, 7, 25).unwrap(),
                latest_reform_date: None,
                retrieved_at: Utc::now(),
                source_url: Url::parse("https://example.test/source").unwrap(),
                source_sha256: "a".repeat(64),
                extracted_text_sha256: "b".repeat(64),
                parser_version: "fixture".to_owned(),
                status: InstrumentStatus::InForce,
                issuing_authorities: Vec::new(),
                formal_publication_url: None,
                formal_publication_code: None,
                formal_source_sha256: None,
                formal_extracted_text_sha256: None,
            },
            provisions: vec![Provision {
                schema_version: SCHEMA_VERSION.to_owned(),
                id: format!("{instrument_id}:article:1"),
                instrument_id: instrument_id.clone(),
                provision_type: ProvisionType::Article,
                label: "Artículo 1".to_owned(),
                number: "1".to_owned(),
                heading_context: HeadingContext {
                    libro: None,
                    title: None,
                    chapter: None,
                    section: None,
                    apartado: None,
                },
                text: "Fixture.".to_owned(),
                publication_date: NaiveDate::from_ymd_opt(2026, 7, 25).unwrap(),
                effective_from: None,
                effective_to: None,
                temporal_status: TemporalStatus::Unknown,
                temporal_basis: None,
                temporal_confidence: None,
                review_status: ReviewStatus::NotAnalyzed,
                transitory_effects: Vec::new(),
                amendment_marks: Vec::new(),
            }],
            references: Vec::new(),
            terms: Vec::new(),
            term_usages: Vec::new(),
            amendment_references: Vec::new(),
        };
        let directory = root.join("corpus/mx").join(slug);
        write_canonical(&corpus, &directory).unwrap();
        write_validation(
            &lex_core::ValidationReport {
                schema_version: SCHEMA_VERSION.to_owned(),
                instrument_id,
                valid: true,
                article_count: 1,
                transitory_count: 0,
                reference_count: 0,
                issues: Vec::new(),
            },
            &directory,
        )
        .unwrap();
        fs::write(directory.join("source-manifest.json"), "{}\n").unwrap();
    }

    fn write_standard_fixture(root: &Path, slug: &str) {
        let id = format!("urn:lex-mx:federal:nom:{slug}");
        let directory = root.join("corpus/mx").join(slug);
        fs::create_dir_all(&directory).unwrap();
        let metadata = StandardMetadata {
            schema_version: SCHEMA_VERSION.to_owned(),
            id: id.clone(),
            kind: StandardKind::Nom,
            designation: "NOM-999-TEST-2026".to_owned(),
            official_title: "Norma de prueba".to_owned(),
            issuing_authorities: vec!["Secretaría de Prueba".to_owned()],
            regulatory_domains: vec!["fixture".to_owned()],
            publication_date: NaiveDate::from_ymd_opt(2026, 7, 25).unwrap(),
            effective_date: None,
            cancellation_date: None,
            status: StandardStatus::Current,
            replaces: Vec::new(),
            replaced_by: Vec::new(),
            joint_prefixes: Vec::new(),
            objective: None,
            scope: None,
            conformity_assessment: None,
            text_basis: StandardTextBasis::AsPublished,
            modifications: Vec::new(),
            systematic_review: None,
            source_url: Url::parse("https://example.test/source.pdf").unwrap(),
            official_dof_url: Url::parse("https://example.test/dof").unwrap(),
            official_registry_url: Some(Url::parse("https://example.test/registry").unwrap()),
            publisher: "Fixture".to_owned(),
            retrieved_at: Utc::now(),
            source_sha256: "a".repeat(64),
            extracted_text_sha256: "b".repeat(64),
            parser_version: "0.1.0".to_owned(),
            legal_review_status: ReviewStatus::NotAnalyzed,
            technical_review_status: TechnicalReviewStatus::NotAnalyzed,
        };
        let report = StandardValidationReport {
            schema_version: SCHEMA_VERSION.to_owned(),
            standard_id: id,
            valid: true,
            clause_count: 0,
            issues: Vec::new(),
        };
        fs::write(
            directory.join("standard.json"),
            serde_json::to_vec_pretty(&metadata).unwrap(),
        )
        .unwrap();
        fs::write(directory.join("clauses.json"), "[]\n").unwrap();
        fs::write(directory.join("transitories.json"), "[]\n").unwrap();
        fs::write(directory.join("extracted-text.txt"), "fixture\n").unwrap();
        fs::write(
            directory.join("validation.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
    }
}
