use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use chrono::NaiveDate;
use clap::{Parser, Subcommand, ValueEnum};
use lex_core::{
    Corpus, Instrument, InstrumentStatus, InstrumentType, LRITF_INSTRUMENT_ID, Provision,
    ProvisionType, SCHEMA_VERSION, SourceManifest, TemporalAnalysisRequest, TemporalEvidence,
};
use lex_export::{write_canonical, write_markdown, write_obsidian, write_validation};
use lex_parse::{extract_pdf, extract_reform_transitories, parse_lritf, validate_lritf};
use lex_source::{
    SourceConfig, discover, fetch, load_config, sha256_hex, write_acquisition, write_manifest,
};
use regex::Regex;

#[derive(Debug, Parser)]
#[command(name = "lex-mex", version, about = "Compile Mexican legal sources")]
struct Cli {
    #[arg(long, global = true, default_value = ".")]
    root: PathBuf,
    #[arg(
        long,
        global = true,
        env = "LEX_MEX_OBSIDIAN_VAULT",
        value_name = "PATH"
    )]
    obsidian_vault: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Discover {
        source: String,
    },
    Fetch {
        instrument: String,
    },
    Extract {
        instrument: String,
    },
    Parse {
        instrument: String,
    },
    AnalyzeTemporal {
        instrument: String,
    },
    Validate {
        instrument: String,
    },
    Export {
        instrument: String,
        #[arg(long)]
        format: ExportFormat,
    },
    Pipeline {
        instrument: String,
        #[arg(long)]
        keep_work: bool,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ExportFormat {
    Json,
    Markdown,
    Obsidian,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let root = absolute_root(&cli.root)?;
    let obsidian_vault = cli
        .obsidian_vault
        .as_deref()
        .map(absolute_root)
        .transpose()?;

    match cli.command {
        Command::Discover { source } => {
            if source != "diputados" {
                bail!("unsupported source {source:?}; expected \"diputados\"");
            }
            let config = config(&root)?;
            println!("{}", serde_json::to_string_pretty(&discover(&config))?);
        }
        Command::Fetch { instrument } => {
            require_lritf(&instrument)?;
            run_fetch(&root)?;
        }
        Command::Extract { instrument } => {
            require_lritf(&instrument)?;
            run_extract(&root)?;
        }
        Command::Parse { instrument } => {
            require_lritf(&instrument)?;
            run_parse(&root)?;
        }
        Command::AnalyzeTemporal { instrument } => {
            require_lritf(&instrument)?;
            run_temporal_request(&root)?;
        }
        Command::Validate { instrument } => {
            require_lritf(&instrument)?;
            let report = run_validate(&root)?;
            if !report.valid {
                bail!("corpus validation failed; inspect corpus/mx/lritf/validation.json");
            }
        }
        Command::Export { instrument, format } => {
            require_lritf(&instrument)?;
            run_export(&root, format, obsidian_vault.as_deref())?;
        }
        Command::Pipeline {
            instrument,
            keep_work,
        } => {
            require_lritf(&instrument)?;
            run_fetch(&root)?;
            run_extract(&root)?;
            run_parse(&root)?;
            run_temporal_request(&root)?;
            let report = run_validate(&root)?;
            if !report.valid {
                bail!("pipeline stopped: validation failed");
            }
            run_export(&root, ExportFormat::Markdown, None)?;
            if let Some(vault) = &obsidian_vault {
                run_export(&root, ExportFormat::Obsidian, Some(vault))?;
            } else {
                println!(
                    "skipped Obsidian publication; pass --obsidian-vault or set \
                     LEX_MEX_OBSIDIAN_VAULT"
                );
            }
            if !keep_work {
                cleanup_work(&root)?;
            }
            println!(
                "pipeline complete: {} articles, {} transitories",
                report.article_count, report.transitory_count
            );
        }
    }
    Ok(())
}

fn run_fetch(root: &Path) -> Result<()> {
    let source = config(root)?;
    let acquisition = fetch(&source)?;
    let paths = Paths::new(root);
    write_acquisition(&acquisition, &paths.pdf, &paths.manifest)?;
    println!(
        "fetched {} bytes; sha256 {}",
        acquisition.bytes.len(),
        acquisition.manifest.source_sha256
    );
    Ok(())
}

fn run_extract(root: &Path) -> Result<()> {
    let paths = Paths::new(root);
    let extraction = extract_pdf(&paths.pdf, &paths.text)?;
    let mut manifest: SourceManifest = read_json(&paths.manifest)?;
    manifest.extracted_text_sha256 = Some(sha256_hex(extraction.text.as_bytes()));
    manifest.extraction_tool = Some(extraction.tool_version);
    write_manifest(&manifest, &paths.manifest)?;
    println!(
        "extracted {} UTF-8 bytes; sha256 {}",
        extraction.text.len(),
        manifest
            .extracted_text_sha256
            .as_deref()
            .unwrap_or("unavailable")
    );
    Ok(())
}

fn run_parse(root: &Path) -> Result<()> {
    let paths = Paths::new(root);
    let source = config(root)?;
    let manifest: SourceManifest = read_json(&paths.manifest)?;
    let raw = fs::read_to_string(&paths.text)
        .with_context(|| format!("failed to read {}", paths.text.display()))?;
    let publication_date = NaiveDate::parse_from_str(&source.publication_date, "%Y-%m-%d")?;
    let provisions = parse_lritf(&raw, publication_date)?;
    let extracted_text_sha256 = manifest
        .extracted_text_sha256
        .clone()
        .context("manifest lacks extracted text hash; run extract first")?;
    let instrument = Instrument {
        schema_version: SCHEMA_VERSION.to_owned(),
        id: LRITF_INSTRUMENT_ID.to_owned(),
        jurisdiction: "mx".to_owned(),
        level: "federal".to_owned(),
        instrument_type: InstrumentType::Statute,
        official_title: source.official_title,
        short_name: source.short_name,
        operational_source: manifest.operational_source.clone(),
        formal_publication_source: manifest.formal_publication_source.clone(),
        publication_date,
        latest_reform_date: latest_reform_date(&raw),
        retrieved_at: manifest.retrieved_at,
        source_url: manifest.official_url,
        source_sha256: manifest.source_sha256,
        extracted_text_sha256,
        parser_version: env!("CARGO_PKG_VERSION").to_owned(),
        status: InstrumentStatus::InForce,
    };
    let corpus = Corpus {
        instrument,
        provisions,
    };
    write_canonical(&corpus, &paths.corpus)?;
    let reform_evidence = extract_reform_transitories(&raw)?;
    write_pretty_json(&reform_evidence, &paths.reform_evidence)?;
    println!("parsed {} canonical provisions", corpus.provisions.len());
    println!(
        "isolated {} reform-decree transitories for temporal analysis",
        reform_evidence.len()
    );
    Ok(())
}

fn run_temporal_request(root: &Path) -> Result<()> {
    let paths = Paths::new(root);
    let corpus = read_corpus(&paths)?;
    let mut evidence: Vec<TemporalEvidence> = corpus
        .provisions
        .iter()
        .filter(|item| item.provision_type == ProvisionType::Transitory)
        .map(|item| TemporalEvidence {
            provision_id: item.id.clone(),
            label: item.label.clone(),
            text: item.text.clone(),
        })
        .collect();
    let mut reform_evidence: Vec<TemporalEvidence> = read_json(&paths.reform_evidence)?;
    evidence.append(&mut reform_evidence);
    let request = TemporalAnalysisRequest {
        schema_version: SCHEMA_VERSION.to_owned(),
        prompt_version: "temporal-v1".to_owned(),
        instrument_id: corpus.instrument.id,
        publication_date: corpus.instrument.publication_date,
        latest_reform_date: corpus.instrument.latest_reform_date,
        relevant_provisions: evidence,
        required_output_schema: "schemas/temporal-analysis.schema.json".to_owned(),
    };
    write_pretty_json(&request, &paths.temporal_request)?;
    println!(
        "wrote temporal analysis request with {} evidence items",
        request.relevant_provisions.len()
    );
    Ok(())
}

fn run_validate(root: &Path) -> Result<lex_core::ValidationReport> {
    let paths = Paths::new(root);
    let source = config(root)?;
    let provisions: Vec<Provision> = read_json(&paths.provisions)?;
    let report = validate_lritf(
        &provisions,
        source.expected_min_articles,
        source.expected_transitories,
    );
    write_validation(&report, &paths.corpus)?;
    println!(
        "validation: {}; {} articles, {} transitories, {} issues",
        if report.valid { "valid" } else { "invalid" },
        report.article_count,
        report.transitory_count,
        report.issues.len()
    );
    Ok(report)
}

fn run_export(root: &Path, format: ExportFormat, obsidian_vault: Option<&Path>) -> Result<()> {
    let paths = Paths::new(root);
    let corpus = read_corpus(&paths)?;
    match format {
        ExportFormat::Json => write_canonical(&corpus, &paths.corpus)?,
        ExportFormat::Markdown => write_markdown(&corpus, &paths.markdown)?,
        ExportFormat::Obsidian => {
            let vault = obsidian_vault.context(
                "Obsidian export requires --obsidian-vault PATH or \
                 LEX_MEX_OBSIDIAN_VAULT",
            )?;
            write_obsidian(&corpus, vault)?;
            println!("published Obsidian vault {}", vault.display());
        }
    }
    println!("exported {format:?}");
    Ok(())
}

fn read_corpus(paths: &Paths) -> Result<Corpus> {
    Ok(Corpus {
        instrument: read_json(&paths.instrument)?,
        provisions: read_json(&paths.provisions)?,
    })
}

fn config(root: &Path) -> Result<SourceConfig> {
    load_config(&root.join("adapters/diputados/lritf.json"))
}

fn latest_reform_date(raw: &str) -> Option<NaiveDate> {
    let regex = Regex::new(r"Última (?:r|R)eforma(?: publicada)? DOF (\d{2}-\d{2}-\d{4})")
        .expect("static regex");
    regex
        .captures(raw)
        .and_then(|captures| NaiveDate::parse_from_str(&captures[1], "%d-%m-%Y").ok())
}

fn require_lritf(value: &str) -> Result<()> {
    if value.eq_ignore_ascii_case("lritf") {
        Ok(())
    } else {
        bail!("the bootstrap slice supports only \"lritf\", received {value:?}")
    }
}

fn cleanup_work(root: &Path) -> Result<()> {
    let work = root.join(".work/lritf");
    if work.exists() {
        fs::remove_dir_all(&work)
            .with_context(|| format!("failed to delete temporary work {}", work.display()))?;
    }
    Ok(())
}

fn absolute_root(root: &Path) -> Result<PathBuf> {
    if root.is_absolute() {
        Ok(root.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(root))
    }
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("invalid JSON in {}", path.display()))
}

fn write_pretty_json<T: serde::Serialize>(value: &T, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(value)?)
        .with_context(|| format!("failed to write {}", path.display()))
}

struct Paths {
    pdf: PathBuf,
    text: PathBuf,
    corpus: PathBuf,
    manifest: PathBuf,
    instrument: PathBuf,
    provisions: PathBuf,
    temporal_request: PathBuf,
    reform_evidence: PathBuf,
    markdown: PathBuf,
}

impl Paths {
    fn new(root: &Path) -> Self {
        let work = root.join(".work/lritf");
        let corpus = root.join("corpus/mx/lritf");
        Self {
            pdf: work.join("LRITF.pdf"),
            text: work.join("LRITF.txt"),
            manifest: corpus.join("source-manifest.json"),
            instrument: corpus.join("instrument.json"),
            provisions: corpus.join("provisions.json"),
            temporal_request: corpus.join("temporal-analysis-request.json"),
            reform_evidence: corpus.join("reform-temporal-evidence.json"),
            markdown: corpus.join("markdown"),
            corpus,
        }
    }
}
