use std::{
    fs,
    io::Write as _,
    path::{Path, PathBuf},
    process::{Command as ProcessCommand, Stdio},
};

use anyhow::{Context, Result, bail};
use chrono::{NaiveDate, Utc};
use clap::{Parser, Subcommand, ValueEnum};
use lex_core::{
    Corpus, Instrument, InstrumentStatus, InstrumentType, LRITF_INSTRUMENT_ID, Provision,
    ProvisionType, ReviewItem, ReviewItemStatus, ReviewResolution, SCHEMA_VERSION, SourceManifest,
    TemporalAnalysisMetadata, TemporalAnalysisRequest, TemporalAnalysisResult, TemporalEvidence,
    TemporalModelBatch, TemporalReviewResolution, TemporalStatus, apply_temporal_determinations,
    preserve_temporal_review_history, resolve_temporal_review, route_temporal_analysis,
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
        #[arg(long, value_enum, default_value = "none")]
        provider: TemporalProvider,
        #[arg(long, default_value = "gpt-5.5")]
        model: String,
    },
    ImportTemporal {
        instrument: String,
        response: PathBuf,
        #[arg(long)]
        model: String,
        #[arg(long)]
        response_id: Option<String>,
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
        #[arg(long, value_enum, default_value = "none")]
        temporal_provider: TemporalProvider,
        #[arg(long, default_value = "gpt-5.5")]
        temporal_model: String,
    },
    Review {
        #[command(subcommand)]
        command: ReviewCommand,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ExportFormat {
    Json,
    Markdown,
    Obsidian,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
enum TemporalProvider {
    None,
    Codex,
}

#[derive(Debug, Subcommand)]
enum ReviewCommand {
    List {
        #[arg(long)]
        all: bool,
    },
    Resolve {
        review_id: String,
        #[arg(long, value_enum)]
        resolution: ReviewResolutionArg,
        #[arg(long)]
        reviewer: String,
        #[arg(long)]
        note: Option<String>,
        #[arg(long, value_enum)]
        temporal_status: Option<TemporalStatusArg>,
        #[arg(long)]
        effective_from: Option<NaiveDate>,
        #[arg(long)]
        effective_to: Option<NaiveDate>,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ReviewResolutionArg {
    AcceptMachineConclusion,
    SetUnknown,
    LawyerOverride,
}

impl From<ReviewResolutionArg> for ReviewResolution {
    fn from(value: ReviewResolutionArg) -> Self {
        match value {
            ReviewResolutionArg::AcceptMachineConclusion => Self::AcceptMachineConclusion,
            ReviewResolutionArg::SetUnknown => Self::SetUnknown,
            ReviewResolutionArg::LawyerOverride => Self::LawyerOverride,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum TemporalStatusArg {
    Unknown,
    PublishedNotEffective,
    Effective,
    FutureEffective,
    PartiallyEffective,
    ConditionallyEffective,
    Repealed,
    RepealedWithSurvival,
    Superseded,
    TemporarilyApplicable,
    PendingConsolidation,
}

impl From<TemporalStatusArg> for TemporalStatus {
    fn from(value: TemporalStatusArg) -> Self {
        match value {
            TemporalStatusArg::Unknown => Self::Unknown,
            TemporalStatusArg::PublishedNotEffective => Self::PublishedNotEffective,
            TemporalStatusArg::Effective => Self::Effective,
            TemporalStatusArg::FutureEffective => Self::FutureEffective,
            TemporalStatusArg::PartiallyEffective => Self::PartiallyEffective,
            TemporalStatusArg::ConditionallyEffective => Self::ConditionallyEffective,
            TemporalStatusArg::Repealed => Self::Repealed,
            TemporalStatusArg::RepealedWithSurvival => Self::RepealedWithSurvival,
            TemporalStatusArg::Superseded => Self::Superseded,
            TemporalStatusArg::TemporarilyApplicable => Self::TemporarilyApplicable,
            TemporalStatusArg::PendingConsolidation => Self::PendingConsolidation,
        }
    }
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
        Command::AnalyzeTemporal {
            instrument,
            provider,
            model,
        } => {
            require_lritf(&instrument)?;
            run_temporal_request(&root)?;
            if provider == TemporalProvider::Codex {
                run_codex_temporal(&root, &model)?;
                run_export(&root, ExportFormat::Markdown, None)?;
                if let Some(vault) = &obsidian_vault {
                    run_export(&root, ExportFormat::Obsidian, Some(vault))?;
                }
            }
        }
        Command::ImportTemporal {
            instrument,
            response,
            model,
            response_id,
        } => {
            require_lritf(&instrument)?;
            run_temporal_import(&root, &response, &model, response_id)?;
            run_export(&root, ExportFormat::Markdown, None)?;
            if let Some(vault) = &obsidian_vault {
                run_export(&root, ExportFormat::Obsidian, Some(vault))?;
            }
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
            temporal_provider,
            temporal_model,
        } => {
            require_lritf(&instrument)?;
            run_pipeline(
                &root,
                obsidian_vault.as_deref(),
                keep_work,
                temporal_provider,
                &temporal_model,
            )?;
        }
        Command::Review { command } => {
            run_review_command(&root, obsidian_vault.as_deref(), command)?;
        }
    }
    Ok(())
}

fn run_pipeline(
    root: &Path,
    obsidian_vault: Option<&Path>,
    keep_work: bool,
    temporal_provider: TemporalProvider,
    temporal_model: &str,
) -> Result<()> {
    run_fetch(root)?;
    run_extract(root)?;
    run_parse(root)?;
    run_temporal_request(root)?;
    let report = run_validate(root)?;
    if !report.valid {
        bail!("pipeline stopped: validation failed");
    }
    if temporal_provider == TemporalProvider::Codex {
        run_codex_temporal(root, temporal_model)?;
    }
    run_export(root, ExportFormat::Markdown, None)?;
    if let Some(vault) = obsidian_vault {
        run_export(root, ExportFormat::Obsidian, Some(vault))?;
    } else {
        println!(
            "skipped Obsidian publication; pass --obsidian-vault or set LEX_MEX_OBSIDIAN_VAULT"
        );
    }
    if !keep_work {
        cleanup_work(root)?;
    }
    println!(
        "pipeline complete: {} articles, {} transitories",
        report.article_count, report.transitory_count
    );
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
    let source = config(root)?;
    reform_evidence.retain(|evidence| {
        source
            .relevant_reform_transitories
            .iter()
            .any(|(date, ordinals)| {
                ordinals.iter().any(|ordinal| {
                    evidence.provision_id
                        == format!("{LRITF_INSTRUMENT_ID}:amendment:{date}:transitory:{ordinal}")
                })
            })
    });
    evidence.append(&mut reform_evidence);
    let request = TemporalAnalysisRequest {
        schema_version: SCHEMA_VERSION.to_owned(),
        prompt_version: "temporal-v1".to_owned(),
        instrument_id: corpus.instrument.id,
        publication_date: corpus.instrument.publication_date,
        latest_reform_date: corpus.instrument.latest_reform_date,
        relevant_provisions: evidence,
        required_output_schema: "schemas/temporal-model-output.schema.json".to_owned(),
    };
    write_pretty_json(&request, &paths.temporal_request)?;
    println!(
        "wrote temporal analysis request with {} evidence items",
        request.relevant_provisions.len()
    );
    Ok(())
}

fn run_codex_temporal(root: &Path, model: &str) -> Result<()> {
    let paths = Paths::new(root);
    let request: TemporalAnalysisRequest = read_json(&paths.temporal_request)?;
    let prompt = fs::read_to_string(root.join("prompts/temporal-v1.md"))?;
    let input = format!(
        "{prompt}\n\n# Evidence request\n\n{}",
        serde_json::to_string_pretty(&request)?
    );
    if let Some(parent) = paths.temporal_model_output.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut child = ProcessCommand::new("codex")
        .args([
            "exec",
            "--ephemeral",
            "--ignore-rules",
            "--sandbox",
            "read-only",
            "--model",
            model,
            "--output-schema",
        ])
        .arg(root.join("schemas/temporal-model-output.schema.json"))
        .args(["--output-last-message"])
        .arg(&paths.temporal_model_output)
        .arg("-")
        .current_dir(root)
        .stdin(Stdio::piped())
        .spawn()
        .context("failed to start Codex temporal-analysis runner")?;
    child
        .stdin
        .take()
        .context("failed to open Codex stdin")?
        .write_all(input.as_bytes())?;
    let status = child.wait()?;
    if !status.success() {
        bail!("Codex temporal-analysis runner failed with status {status}");
    }
    run_temporal_import(root, &paths.temporal_model_output, model, None)
}

fn run_temporal_import(
    root: &Path,
    response_path: &Path,
    model: &str,
    response_id: Option<String>,
) -> Result<()> {
    let paths = Paths::new(root);
    let request_bytes = fs::read(&paths.temporal_request)?;
    let request: TemporalAnalysisRequest = serde_json::from_slice(&request_bytes)?;
    let response_bytes = fs::read(response_path)?;
    let batch: TemporalModelBatch = serde_json::from_slice(&response_bytes)?;
    let instrument: Instrument = read_json(&paths.instrument)?;
    let mut routed = route_temporal_analysis(
        &request,
        batch,
        TemporalAnalysisMetadata {
            request_sha256: sha256_hex(&request_bytes),
            response_sha256: sha256_hex(&response_bytes),
            response_id,
            model: model.to_owned(),
            analyzed_at: Utc::now(),
        },
        &instrument.source_url,
    )?;
    if paths.temporal_result.exists() && paths.review_queue.exists() {
        let previous_result: TemporalAnalysisResult = read_json(&paths.temporal_result)?;
        let previous_items: Vec<ReviewItem> = read_json(&paths.review_queue)?;
        preserve_temporal_review_history(
            &mut routed.result,
            &mut routed.review_items,
            &previous_result,
            &previous_items,
        );
    }
    write_pretty_json(&routed.result, &paths.temporal_result)?;
    write_pretty_json(&routed.review_items, &paths.review_queue)?;

    let mut corpus = read_corpus(&paths)?;
    apply_temporal_determinations(&mut corpus.provisions, &routed.result.determinations);
    write_canonical(&corpus, &paths.corpus)?;
    let machine_accepted = routed
        .result
        .determinations
        .iter()
        .filter(|item| !item.review_required && item.basis == lex_core::Basis::LlmInference)
        .count();
    let lawyer_verified = routed
        .result
        .determinations
        .iter()
        .filter(|item| item.basis == lex_core::Basis::LawyerVerified)
        .count();
    let pending = routed
        .review_items
        .iter()
        .filter(|item| item.status == ReviewItemStatus::Pending)
        .count();
    println!(
        "temporal analysis: {machine_accepted} machine-accepted, {lawyer_verified} \
         lawyer-verified, {pending} pending review"
    );
    Ok(())
}

fn run_review_command(
    root: &Path,
    obsidian_vault: Option<&Path>,
    command: ReviewCommand,
) -> Result<()> {
    match command {
        ReviewCommand::List { all } => run_review_list(root, all),
        ReviewCommand::Resolve {
            review_id,
            resolution,
            reviewer,
            note,
            temporal_status,
            effective_from,
            effective_to,
        } => {
            run_review_resolve(
                root,
                &review_id,
                TemporalReviewResolution {
                    resolution: resolution.into(),
                    reviewer,
                    note,
                    temporal_status: temporal_status.map(Into::into),
                    effective_from,
                    effective_to,
                    resolved_at: Utc::now(),
                },
            )?;
            run_export(root, ExportFormat::Markdown, None)?;
            if let Some(vault) = obsidian_vault {
                run_export(root, ExportFormat::Obsidian, Some(vault))?;
            }
            Ok(())
        }
    }
}

fn run_review_list(root: &Path, all: bool) -> Result<()> {
    let paths = Paths::new(root);
    let items: Vec<ReviewItem> = read_json(&paths.review_queue)
        .with_context(|| "review queue not found; run temporal analysis first")?;
    let visible: Vec<_> = items
        .iter()
        .filter(|item| all || item.status == ReviewItemStatus::Pending)
        .collect();
    if visible.is_empty() {
        println!("no matching review items");
        return Ok(());
    }
    for item in &visible {
        println!(
            "{}\n  {}\n  status: {:?}\n  confidence: {:.2}\n  issue: {}",
            item.id,
            item.evidence.label,
            item.status,
            item.proposed_machine_conclusion.confidence,
            item.exact_issue
        );
    }
    println!("{} review items", visible.len());
    Ok(())
}

fn run_review_resolve(
    root: &Path,
    review_id: &str,
    resolution: TemporalReviewResolution,
) -> Result<()> {
    let paths = Paths::new(root);
    let mut items: Vec<ReviewItem> = read_json(&paths.review_queue)
        .with_context(|| "review queue not found; run temporal analysis first")?;
    let mut result: TemporalAnalysisResult = read_json(&paths.temporal_result)
        .with_context(|| "temporal result not found; run temporal analysis first")?;
    let item = items
        .iter_mut()
        .find(|item| item.id == review_id)
        .with_context(|| format!("review item not found: {review_id}"))?;
    resolve_temporal_review(item, &mut result.determinations, resolution)?;

    write_pretty_json(&result, &paths.temporal_result)?;
    write_pretty_json(&items, &paths.review_queue)?;
    let mut corpus = read_corpus(&paths)?;
    apply_temporal_determinations(&mut corpus.provisions, &result.determinations);
    write_canonical(&corpus, &paths.corpus)?;
    println!("resolved {review_id}");
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
            let review_items = if paths.review_queue.exists() {
                read_json(&paths.review_queue)?
            } else {
                Vec::<ReviewItem>::new()
            };
            write_obsidian(&corpus, &review_items, vault)?;
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
    temporal_model_output: PathBuf,
    temporal_result: PathBuf,
    review_queue: PathBuf,
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
            temporal_model_output: work.join("temporal-model-output.json"),
            temporal_result: corpus.join("temporal-analysis-result.json"),
            review_queue: corpus.join("review-queue.json"),
            reform_evidence: corpus.join("reform-temporal-evidence.json"),
            markdown: corpus.join("markdown"),
            corpus,
        }
    }
}
