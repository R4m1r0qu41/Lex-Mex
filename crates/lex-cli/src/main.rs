use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fs,
    io::Write as _,
    path::{Path, PathBuf},
    process::{Command as ProcessCommand, Stdio},
};

use anyhow::{Context, Result, bail};
use chrono::{NaiveDate, Utc};
use clap::{Parser, Subcommand, ValueEnum};
use lex_core::{
    Corpus, Instrument, InstrumentStatus, InstrumentType, ProvisionType, ReferenceResolutionStatus,
    ReviewItem, ReviewItemStatus, ReviewResolution, SCHEMA_VERSION, SourceManifest, StandardClause,
    StandardKind, StandardMetadata, StandardStatus, StandardTextBasis, TemporalAnalysisMetadata,
    TemporalAnalysisRequest, TemporalAnalysisResult, TemporalEvidence, TemporalReviewResolution,
    TemporalStatus, TransitoryEffect, apply_temporal_determinations, open_temporal_review,
    parse_temporal_model_response, preserve_temporal_review_history,
    reapply_temporal_determinations, resolve_temporal_review, route_temporal_analysis,
};
use lex_export::{
    LinkTargets, TermTargets, link_targets, term_targets, write_canonical, write_markdown,
    write_obsidian, write_validation,
};
use lex_parse::{
    Commencement, CorpusExpectations, CorpusView, DiputadosOptions, GlossaryStyle,
    InstrumentContextPolicy, ReferenceOptions, audit_pdf, derive_article_temporal_determinations,
    detect_glossary, extract_doc, extract_glossaries, extract_html_text,
    extract_internal_references, extract_pdf, extract_quoted_term_usages, extract_references,
    extract_term_usages, extract_term_usages_with_lowercase, find_admitted_furniture,
    find_glossary_provision, invalidated_fractions, parse_dcg, parse_itf_dcg, repair_article_dates,
    repair_partial_repeals, repeal_placeholders, unread_fractions, validate_corpus,
};
use lex_source::{
    SourceConfig, SourceFormat, discover, fetch, fetch_annex, fetch_formal, load_batch_manifest,
    load_config, sha256_hex, write_acquisition, write_manifest,
};
use regex::Regex;

mod bundle;
mod db;
mod public_release;
mod review_packets;
mod standards;

#[derive(Debug, Parser)]
#[command(name = "lex-mex", version, about = "Compile Mexican legal sources")]
struct Cli {
    #[arg(long, global = true, env = "LEX_MEX_ROOT", default_value = ".")]
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
    /// Inspect database contracts without building a full corpus projection.
    Db {
        #[command(subcommand)]
        command: db::DbCommand,
    },
    /// List the committed instruments available to consumer applications.
    Instruments {
        /// Emit a JSON array instead of tab-separated text.
        #[arg(long)]
        json: bool,
    },
    /// Print an absolute path that other command-line tools can consume.
    Path {
        /// Restrict the path to one committed instrument.
        instrument: Option<String>,
        #[arg(long, value_enum, default_value = "corpus")]
        kind: CorpusPathKind,
    },
    /// Search committed legal text and metadata with ripgrep.
    Search {
        pattern: String,
        /// Restrict the search to these instrument slugs (comma-separated).
        #[arg(long, value_delimiter = ',')]
        instrument: Vec<String>,
        #[arg(long, value_enum, default_value = "canonical")]
        scope: SearchScope,
        /// Additional arguments passed directly to `rg`; place them after `--`.
        #[arg(last = true, allow_hyphen_values = true)]
        rg_args: Vec<String>,
    },
    /// Create a deterministic, portable subset of the committed corpus.
    Bundle {
        #[command(subcommand)]
        command: bundle::BundleCommand,
    },
    /// Build an allowlisted public release with exact text and optional verified PDFs.
    PublicRelease {
        #[command(subcommand)]
        command: public_release::PublicReleaseCommand,
    },
    /// Compile and validate NOM/NMX records through their separate boundary.
    Standards {
        #[command(subcommand)]
        command: standards::StandardsCommand,
    },
    /// Read one PDF positionally and report recurring header/footer bands.
    /// This is audit evidence only: it never changes extracted or canonical text.
    AuditFurniture {
        /// PDF to inspect. Relative paths are resolved from --root.
        pdf: PathBuf,
        /// Emit the report as pretty JSON.
        #[arg(long)]
        json: bool,
    },
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
    /// Recompute references and defined-term usages from canonical text.
    Link {
        /// Committed instrument slug. Required unless `--all` is set.
        #[arg(
            value_name = "INSTRUMENT",
            required_unless_present = "all",
            conflicts_with = "all"
        )]
        instrument: Option<String>,
        /// Relink every committed instrument against the current corpus.
        /// Validation failures are left unchanged and reported.
        #[arg(long)]
        all: bool,
        /// Audited override for one instrument: allow this relink to change
        /// or drop persisted historical or approved edges. Every affected
        /// edge id is printed; use only for a recorded operator migration.
        #[arg(long, conflicts_with = "all")]
        migrate_protected: bool,
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
    /// Deterministically classify an instrument's articles as effective,
    /// future-effective, or repealed from plain code -- no model call --
    /// using only its own commencement clause and repeal-marker text.
    /// New determinations only touch `not_analyzed` provisions. Also repairs
    /// article start dates written by earlier versions of the rule.
    /// Skips (and reports why) any instrument whose
    /// commencement it cannot resolve unambiguously.
    DeriveTemporal {
        instrument: String,
        /// Repair only article start dates written by earlier versions of
        /// the deterministic rule: unamended articles take the original
        /// commencement, amended ones are left unset.
        /// Does not create any new temporal determinations.
        #[arg(long)]
        repair_dates_only: bool,
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
        #[arg(long, default_value = "lritf")]
        instrument: String,
        #[command(subcommand)]
        command: ReviewCommand,
    },
    /// Bulk ingestion driven by a `batches/*.json` manifest.
    Batch {
        #[command(subcommand)]
        command: BatchCommand,
    },
    /// Adapter management for batch ingestion.
    Adapter {
        #[command(subcommand)]
        command: AdapterCommand,
    },
    /// Group committed instruments into review packets and assign a
    /// reviewer, rather than gating ingestion on one-at-a-time review.
    ReviewPackets {
        #[command(subcommand)]
        command: review_packets::ReviewPacketsCommand,
    },
}

#[derive(Debug, Subcommand)]
enum BatchCommand {
    /// Run the structural pipeline (fetch through export, temporal
    /// analysis deliberately skipped) for every instrument in a batch
    /// manifest, isolating failures per instrument.
    Run {
        manifest: PathBuf,
        /// Restrict to these slugs (comma-separated).
        #[arg(long, value_delimiter = ',')]
        only: Vec<String>,
        /// After a successful parse, freeze machine-proposed count
        /// baselines (and the derived publication date) into the adapter.
        #[arg(long)]
        freeze_counts: bool,
        #[arg(long)]
        keep_work: bool,
    },
}

#[derive(Debug, Subcommand)]
enum AdapterCommand {
    /// Generate a Diputados adapter config from a batch-manifest entry.
    Scaffold { batch: PathBuf, slug: String },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ExportFormat {
    Json,
    Markdown,
    Obsidian,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CorpusPathKind {
    Corpus,
    Instrument,
    Standard,
    Provisions,
    Clauses,
    Transitories,
    Supplements,
    References,
    Markdown,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
enum SearchScope {
    /// Generated, human-readable provision Markdown.
    Markdown,
    /// Canonical JSON and sidecar records, excluding generated Markdown.
    Canonical,
    /// Both canonical records and generated Markdown.
    All,
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
    /// List review items recorded before evidence hashes were kept, and check
    /// whether the text each reviewer saw still matches the current corpus.
    /// With `--check-upstream`, also HEAD the official source and compare its
    /// headers with the retained manifest to see whether it has moved.
    Legacy {
        /// Scan every committed instrument instead of only `--instrument`.
        #[arg(long)]
        all_instruments: bool,
        #[arg(long)]
        check_upstream: bool,
    },
    /// Open a review item for a determination that was machine-accepted,
    /// so the designated legal reviewer can correct or enrich it through
    /// the audited resolution workflow.
    Open {
        provision_id: String,
        #[arg(long)]
        reason: String,
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
        #[arg(long, value_name = "PATH")]
        effects_file: Option<PathBuf>,
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
    Repealed,
    Superseded,
}

impl From<TemporalStatusArg> for TemporalStatus {
    fn from(value: TemporalStatusArg) -> Self {
        match value {
            TemporalStatusArg::Unknown => Self::Unknown,
            TemporalStatusArg::PublishedNotEffective => Self::PublishedNotEffective,
            TemporalStatusArg::Effective => Self::Effective,
            TemporalStatusArg::FutureEffective => Self::FutureEffective,
            TemporalStatusArg::Repealed => Self::Repealed,
            TemporalStatusArg::Superseded => Self::Superseded,
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
    dispatch(&root, obsidian_vault.as_deref(), cli.command)
}

/// Split from `main` so the per-subcommand match arms -- one per CLI
/// surface, growing whenever a subcommand is added -- don't make `main`
/// itself trip a function-length lint on an unrelated addition.
fn dispatch_link(
    root: &Path,
    instrument: Option<String>,
    all: bool,
    migrate_protected: bool,
) -> Result<()> {
    if all {
        let report = run_corpus_closure(root)?;
        let matches_accepted = closure_matches_accepted_gaps(root, &report)?;
        if matches_accepted {
            if !report.rejected.is_empty() {
                println!(
                    "corpus closure: the {} rejected instruments are exactly the recorded accepted gaps",
                    report.rejected.len()
                );
            }
            return Ok(());
        }
        if report.status == "ok" {
            bail!(
                "corpus closure is clean but the accepted-gap list is stale; remove the resolved entries from {CLOSURE_ACCEPTED_GAPS}"
            );
        }
        if report.status != "ok" {
            bail!(
                "corpus closure left {} instrument(s) unchanged because relink validation failed; see .work/corpus-closure.json",
                report.rejected.len()
            );
        }
        return Ok(());
    }
    let instrument = instrument.context("missing instrument slug")?;
    let context = instrument_context(root, &instrument)?;
    run_link_with(root, &context, migrate_protected)
}

fn dispatch(root: &Path, obsidian_vault: Option<&Path>, command: Command) -> Result<()> {
    match command {
        command @ (Command::Instruments { .. } | Command::Path { .. } | Command::Search { .. }) => {
            run_consumer_command(root, command)?;
        }
        Command::Bundle { command } => bundle::run_bundle_command(root, command)?,
        Command::Db { command } => db::run(root, command)?,
        Command::PublicRelease { command } => public_release::run(root, command)?,
        Command::Standards { command } => standards::run_standards_command(root, command)?,
        Command::AuditFurniture { pdf, json } => run_audit_furniture(root, &pdf, json)?,
        Command::Discover { source } => run_discover(root, &source)?,
        Command::Fetch { instrument } => {
            let context = instrument_context(root, &instrument)?;
            run_fetch(&context)?;
        }
        Command::Extract { instrument } => {
            let context = instrument_context(root, &instrument)?;
            run_extract(&context)?;
        }
        Command::Parse { instrument } => {
            let context = instrument_context(root, &instrument)?;
            run_parse(root, &context)?;
        }
        Command::Link {
            instrument,
            all,
            migrate_protected,
        } => dispatch_link(root, instrument, all, migrate_protected)?,
        Command::AnalyzeTemporal {
            instrument,
            provider,
            model,
        } => {
            let context = instrument_context(root, &instrument)?;
            run_temporal_request(&context)?;
            if provider == TemporalProvider::Codex {
                run_codex_temporal(root, &context, &model)?;
                republish_exports(root, &context, obsidian_vault)?;
            }
        }
        Command::ImportTemporal {
            instrument,
            response,
            model,
            response_id,
        } => {
            let context = instrument_context(root, &instrument)?;
            run_temporal_import(&context, &response, &model, response_id)?;
            republish_exports(root, &context, obsidian_vault)?;
        }
        Command::DeriveTemporal {
            instrument,
            repair_dates_only,
        } => run_derive_command(root, &instrument, repair_dates_only, obsidian_vault)?,
        Command::Validate { instrument } => {
            let context = instrument_context(root, &instrument)?;
            let report = run_validate(root, &context)?;
            if !report.valid {
                bail!(
                    "corpus validation failed; inspect {}",
                    context.paths.corpus.join("validation.json").display()
                );
            }
        }
        Command::Export { instrument, format } => {
            let context = instrument_context(root, &instrument)?;
            run_export(root, &context, format, obsidian_vault)?;
        }
        Command::Pipeline {
            instrument,
            keep_work,
            temporal_provider,
            temporal_model,
        } => {
            let context = instrument_context(root, &instrument)?;
            run_pipeline(
                root,
                &context,
                obsidian_vault,
                keep_work,
                temporal_provider,
                &temporal_model,
            )?;
        }
        Command::Review {
            instrument,
            command,
        } => {
            let context = instrument_context(root, &instrument)?;
            run_review_command(root, &context, obsidian_vault, command)?;
        }
        Command::Batch { command } => run_batch_command(root, obsidian_vault, command)?,
        Command::Adapter {
            command: AdapterCommand::Scaffold { batch, slug },
        } => {
            scaffold_adapter(root, &batch, &slug)?;
            preflight_adapter_uniqueness(root)?;
        }
        Command::ReviewPackets { command } => review_packets::run_review_packets(root, command)?,
    }
    Ok(())
}

fn run_audit_furniture(root: &Path, pdf: &Path, json: bool) -> Result<()> {
    let pdf = if pdf.is_absolute() {
        pdf.to_path_buf()
    } else {
        root.join(pdf)
    };
    let report =
        audit_pdf(&pdf).with_context(|| format!("failed to audit PDF {}", pdf.display()))?;
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!(
            "audited {} page(s): {} header/footer band(s), {} body-zone repeat(s)",
            report.page_count,
            report.furniture_bands.len(),
            report.body_zone_repeats.len()
        );
        for band in &report.furniture_bands {
            println!(
                "furniture y={} pages={} coverage={:.2} zone={:?} kind={:?}: {}",
                band.y,
                band.pages,
                band.coverage,
                band.zone,
                band.kind,
                band.sample_texts.join(" | ")
            );
        }
        for band in &report.body_zone_repeats {
            println!(
                "body-zone repeat y={} pages={} coverage={:.2}: {}",
                band.y,
                band.pages,
                band.coverage,
                band.sample_texts.join(" | ")
            );
        }
    }
    Ok(())
}

fn run_consumer_command(root: &Path, command: Command) -> Result<()> {
    match command {
        Command::Instruments { json } => run_instruments(root, json),
        Command::Path { instrument, kind } => run_path(root, instrument.as_deref(), kind),
        Command::Search {
            pattern,
            instrument,
            scope,
            rg_args,
        } => run_search(root, &pattern, &instrument, scope, &rg_args),
        _ => unreachable!("only consumer commands are routed to this function"),
    }
}

fn run_discover(root: &Path, source: &str) -> Result<()> {
    let configs = discover_source_configs(root, source)?;
    for config in &configs {
        println!("{}", serde_json::to_string_pretty(&discover(config))?);
    }
    Ok(())
}

#[derive(Debug, serde::Serialize)]
struct InstrumentIndexEntry {
    slug: String,
    id: String,
    short_name: String,
    official_title: String,
    instrument_type: String,
    status: String,
    publication_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    publication_period: Option<lex_core::PublicationPeriod>,
    latest_reform_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text_basis: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    unconsolidated_modification_count: Option<usize>,
    /// The designation the retained text itself was published under, when the
    /// registry has since redesignated the standard. Without it a reader of
    /// the index sees a designation appearing nowhere in the record's own
    /// source text, with nothing marking the discrepancy.
    #[serde(skip_serializing_if = "Option::is_none")]
    published_designation: Option<String>,
    /// How many clauses carry `amended_by` marks -- known-outdated text,
    /// precisely located. A count here is a staleness claim, not a currency
    /// one.
    #[serde(skip_serializing_if = "Option::is_none")]
    amendment_marked_clauses: Option<usize>,
    path: PathBuf,
}

fn committed_instrument_index(root: &Path) -> Result<Vec<InstrumentIndexEntry>> {
    let corpus_root = root.join("corpus/mx");
    if !corpus_root.is_dir() {
        bail!(
            "committed corpus directory does not exist: {}",
            corpus_root.display()
        );
    }
    let mut directories: Vec<_> = fs::read_dir(&corpus_root)?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|entry| entry.path())
        .filter(|path| {
            path.join("instrument.json").is_file() || path.join("standard.json").is_file()
        })
        .collect();
    directories.sort();

    directories
        .into_iter()
        .map(|path| {
            let slug = path
                .file_name()
                .and_then(|name| name.to_str())
                .context("corpus directory name is not valid UTF-8")?
                .to_owned();
            if path.join("instrument.json").is_file() {
                let instrument: Instrument = read_json(&path.join("instrument.json"))?;
                Ok(InstrumentIndexEntry {
                    slug,
                    id: instrument.id,
                    short_name: instrument.short_name,
                    official_title: instrument.official_title,
                    instrument_type: instrument_type_name(&instrument.instrument_type).to_owned(),
                    status: instrument_status_name(&instrument.status).to_owned(),
                    publication_date: instrument.publication_date,
                    publication_period: instrument.publication_period.clone(),
                    latest_reform_date: instrument.latest_reform_date,
                    text_basis: None,
                    unconsolidated_modification_count: None,
                    published_designation: None,
                    amendment_marked_clauses: None,
                    path,
                })
            } else {
                let standard: StandardMetadata = read_json(&path.join("standard.json"))?;
                // A committed standard always has clauses.json, but the index
                // is a listing: a directory missing it still lists (with no
                // mark count) rather than hiding every other instrument
                // behind one broken entry. A present-but-unreadable file is
                // still an error -- that is corruption, not absence.
                let clauses_path = path.join("clauses.json");
                let marked = if clauses_path.is_file() {
                    let clauses: Vec<StandardClause> = read_json(&clauses_path)?;
                    Some(
                        clauses
                            .iter()
                            .filter(|clause| !clause.amended_by.is_empty())
                            .count(),
                    )
                } else {
                    None
                };
                Ok(InstrumentIndexEntry {
                    slug,
                    id: standard.id,
                    short_name: standard.designation,
                    official_title: standard.official_title,
                    instrument_type: standard_kind_name(&standard.kind).to_owned(),
                    status: standard_status_name(&standard.status).to_owned(),
                    publication_date: Some(standard.publication_date),
                    publication_period: None,
                    latest_reform_date: standard
                        .modifications
                        .iter()
                        .map(|source| source.publication_date)
                        .max(),
                    text_basis: Some(standard_text_basis_name(&standard.text_basis).to_owned()),
                    unconsolidated_modification_count: Some(
                        standard
                            .modifications
                            .iter()
                            .filter(|source| !source.included_in_source)
                            .count(),
                    ),
                    published_designation: standard.published_designation,
                    amendment_marked_clauses: marked.filter(|&count| count > 0),
                    path,
                })
            }
        })
        .collect()
}

fn instrument_type_name(value: &InstrumentType) -> &'static str {
    match value {
        InstrumentType::Constitution => "constitution",
        InstrumentType::Code => "code",
        InstrumentType::Statute => "statute",
        InstrumentType::Regulation => "regulation",
        InstrumentType::Guideline => "guideline",
        InstrumentType::Circular => "circular",
        InstrumentType::Other => "other",
    }
}

fn instrument_status_name(value: &InstrumentStatus) -> &'static str {
    match value {
        InstrumentStatus::InForce => "in_force",
        InstrumentStatus::PartiallyEffective => "partially_effective",
        InstrumentStatus::Repealed => "repealed",
        InstrumentStatus::Unknown => "unknown",
    }
}

fn standard_kind_name(value: &StandardKind) -> &'static str {
    match value {
        StandardKind::Nom => "nom",
        StandardKind::Nmx => "nmx",
    }
}

fn standard_status_name(value: &StandardStatus) -> &'static str {
    match value {
        StandardStatus::Current => "current",
        StandardStatus::Cancelled => "cancelled",
        StandardStatus::Replaced => "replaced",
        StandardStatus::Unknown => "unknown",
    }
}

fn standard_text_basis_name(value: &StandardTextBasis) -> &'static str {
    match value {
        StandardTextBasis::AsPublished => "as_published",
        StandardTextBasis::OfficialCompilation => "official_compilation",
        StandardTextBasis::DerivedConsolidation => "derived_consolidation",
    }
}

fn run_instruments(root: &Path, json: bool) -> Result<()> {
    let instruments = committed_instrument_index(root)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&instruments)?);
    } else {
        for instrument in instruments {
            println!(
                "{}\t{}\t{}\t{}",
                instrument.slug,
                instrument.short_name,
                instrument.official_title,
                instrument.path.display()
            );
        }
    }
    Ok(())
}

fn committed_instrument_path(root: &Path, slug: &str) -> Result<PathBuf> {
    let mut components = Path::new(slug).components();
    if !matches!(components.next(), Some(std::path::Component::Normal(_)))
        || components.next().is_some()
    {
        bail!("instrument slug must be one path component, got {slug:?}");
    }
    let path = root.join("corpus/mx").join(slug);
    if !path.join("instrument.json").is_file() && !path.join("standard.json").is_file() {
        bail!("no committed corpus found for instrument {slug:?}");
    }
    Ok(path)
}

fn selected_corpus_paths(root: &Path, instruments: &[String]) -> Result<Vec<PathBuf>> {
    if instruments.is_empty() {
        let corpus_root = root.join("corpus/mx");
        if !corpus_root.is_dir() {
            bail!(
                "committed corpus directory does not exist: {}",
                corpus_root.display()
            );
        }
        Ok(vec![corpus_root])
    } else {
        instruments
            .iter()
            .map(|slug| committed_instrument_path(root, slug))
            .collect()
    }
}

fn run_path(root: &Path, instrument: Option<&str>, kind: CorpusPathKind) -> Result<()> {
    let corpus = match instrument {
        Some(slug) => committed_instrument_path(root, slug)?,
        None if matches!(kind, CorpusPathKind::Corpus) => root.join("corpus/mx"),
        None => bail!("--kind {kind:?} requires an instrument slug"),
    };
    let path = match kind {
        CorpusPathKind::Corpus => corpus,
        CorpusPathKind::Instrument => corpus.join("instrument.json"),
        CorpusPathKind::Standard => corpus.join("standard.json"),
        CorpusPathKind::Provisions => corpus.join("provisions.json"),
        CorpusPathKind::Clauses => corpus.join("clauses.json"),
        CorpusPathKind::Transitories => corpus.join("transitories.json"),
        CorpusPathKind::Supplements => corpus.join("supplements.json"),
        CorpusPathKind::References => corpus.join("references.json"),
        CorpusPathKind::Markdown => corpus.join("markdown"),
    };
    if !path.exists() {
        bail!("requested corpus path does not exist: {}", path.display());
    }
    println!("{}", path.display());
    Ok(())
}

fn run_search(
    root: &Path,
    pattern: &str,
    instruments: &[String],
    scope: SearchScope,
    rg_args: &[String],
) -> Result<()> {
    let paths = selected_corpus_paths(root, instruments)?;
    let mut command = ProcessCommand::new("rg");
    command.args(rg_args);
    match scope {
        SearchScope::Markdown => {
            command.args(["--glob", "**/markdown/**/*.md"]);
        }
        SearchScope::Canonical => {
            command.args(["--glob", "!**/markdown/**"]);
        }
        SearchScope::All => {}
    }
    command.arg("--").arg(pattern).args(paths);
    let status = command.status().context(
        "failed to execute ripgrep; install `rg` or use `lex-mex path` with another tool",
    )?;
    match status.code() {
        Some(0 | 1) => Ok(()),
        Some(code) => bail!("ripgrep failed with exit code {code}"),
        None => bail!("ripgrep terminated by signal"),
    }
}

fn republish_exports(
    root: &Path,
    context: &InstrumentContext,
    obsidian_vault: Option<&Path>,
) -> Result<()> {
    run_export(root, context, ExportFormat::Markdown, None)?;
    if let Some(vault) = obsidian_vault {
        run_export(root, context, ExportFormat::Obsidian, Some(vault))?;
    }
    Ok(())
}

fn run_batch_command(
    root: &Path,
    obsidian_vault: Option<&Path>,
    command: BatchCommand,
) -> Result<()> {
    match command {
        BatchCommand::Run {
            manifest,
            only,
            freeze_counts,
            keep_work,
        } => run_batch(
            root,
            &manifest,
            &only,
            freeze_counts,
            keep_work,
            obsidian_vault,
        ),
    }
}

/// One instrument's adapter configuration and working paths.
struct InstrumentContext {
    config: SourceConfig,
    paths: Paths,
    /// Repository root, for shared adapter tables (`_instrument-aliases.json`).
    root: PathBuf,
}

fn instrument_context(root: &Path, slug: &str) -> Result<InstrumentContext> {
    let config = find_adapter(root, slug)?;
    let paths = Paths::new(root, slug, config.source_format);
    Ok(InstrumentContext {
        config,
        paths,
        root: root.to_path_buf(),
    })
}

fn find_adapter(root: &Path, slug: &str) -> Result<SourceConfig> {
    for path in adapter_paths(root)? {
        let config = load_config(&path)?;
        if config.slug == slug {
            return Ok(config);
        }
    }
    bail!("no adapter configuration found for instrument {slug:?} under adapters/")
}

fn discover_source_configs(root: &Path, source: &str) -> Result<Vec<SourceConfig>> {
    let source_dir = root.join("adapters").join(source);
    if !source_dir.is_dir() {
        bail!("unsupported source {source:?}; expected a directory under adapters/");
    }
    let mut configs = Vec::new();
    let mut entries: Vec<_> = fs::read_dir(&source_dir)?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|entry| entry.path())
        .filter(|path| is_adapter_config(path))
        .collect();
    entries.sort();
    for path in entries {
        configs.push(load_config(&path)?);
    }
    if configs.is_empty() {
        bail!(
            "no adapter configurations found under {}",
            source_dir.display()
        );
    }
    Ok(configs)
}

fn adapter_paths(root: &Path) -> Result<Vec<PathBuf>> {
    let adapters = root.join("adapters");
    let mut paths = Vec::new();
    for entry in
        fs::read_dir(&adapters).with_context(|| format!("failed to read {}", adapters.display()))?
    {
        let directory = entry?.path();
        if !directory.is_dir() {
            continue;
        }
        for file in fs::read_dir(&directory)? {
            let path = file?.path();
            if is_adapter_config(&path) {
                paths.push(path);
            }
        }
    }
    paths.sort();
    Ok(paths)
}

/// Instrument adapter configs are `adapters/<source>/<slug>.json`. A file
/// whose name starts with `_` is shared configuration data (for example
/// `_instrument-aliases.json`, `_named-offenses.json`), not an instrument
/// adapter, and is never parsed as `SourceConfig`.
fn is_adapter_config(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension == "json")
        && !path
            .file_name()
            .is_some_and(|name| name.to_string_lossy().starts_with('_'))
}

fn run_pipeline(
    root: &Path,
    context: &InstrumentContext,
    obsidian_vault: Option<&Path>,
    keep_work: bool,
    temporal_provider: TemporalProvider,
    temporal_model: &str,
) -> Result<()> {
    run_fetch(context)?;
    run_extract(context)?;
    run_parse(root, context)?;
    run_temporal_request(context)?;
    let report = run_validate(root, context)?;
    if !report.valid {
        bail!("pipeline stopped: validation failed");
    }
    run_temporal_derive(context, false)?;
    if temporal_provider == TemporalProvider::Codex {
        run_codex_temporal(root, context, temporal_model)?;
    }
    run_export(root, context, ExportFormat::Markdown, None)?;
    if let Some(vault) = obsidian_vault {
        run_export(root, context, ExportFormat::Obsidian, Some(vault))?;
    } else {
        println!(
            "skipped Obsidian publication; pass --obsidian-vault or set LEX_MEX_OBSIDIAN_VAULT"
        );
    }
    if !keep_work {
        cleanup_work(context)?;
    }
    println!(
        "pipeline complete: {} articles, {} transitories",
        report.article_count, report.transitory_count
    );
    Ok(())
}

fn run_fetch(context: &InstrumentContext) -> Result<()> {
    let acquisition = fetch(&context.config)?;
    write_acquisition(&acquisition, &context.paths.source, &context.paths.manifest)?;
    println!(
        "fetched {} bytes; sha256 {}",
        acquisition.bytes.len(),
        acquisition.manifest.source_sha256
    );
    if let Some(formal) = fetch_formal(&context.config)? {
        write_acquisition(
            &formal,
            &context.paths.formal_source,
            &context.paths.formal_manifest,
        )?;
        println!(
            "fetched formal publication: {} bytes; sha256 {}",
            formal.bytes.len(),
            formal.manifest.source_sha256
        );
    }
    if !context.config.annex_pdf_urls.is_empty() {
        let mut manifests = Vec::with_capacity(context.config.annex_pdf_urls.len());
        for (index, url) in context.config.annex_pdf_urls.iter().enumerate() {
            let number = index + 1;
            let annex = fetch_annex(&context.config, url)?;
            fs::create_dir_all(&context.paths.work)?;
            fs::write(context.paths.annex_pdf(number), &annex.bytes)?;
            println!(
                "fetched annex {number}: {} bytes; sha256 {}",
                annex.bytes.len(),
                annex.manifest.source_sha256
            );
            manifests.push(annex.manifest);
        }
        write_pretty_json(&manifests, &context.paths.annex_manifests)?;
    }
    Ok(())
}

fn run_extract(context: &InstrumentContext) -> Result<()> {
    let paths = &context.paths;
    // Both DCG parsers rely on page-break markers for deterministic
    // paragraph merging across a page boundary; LRITF's parser never
    // reads them at all.
    let keep_page_breaks = matches!(context.config.parser.as_str(), "ifpe-dcg" | "itf-dcg");
    let extraction = match context.config.source_format {
        SourceFormat::Pdf => extract_pdf(&paths.source, &paths.text, keep_page_breaks)?,
        SourceFormat::Doc => extract_doc(&paths.source, &paths.text)?,
        SourceFormat::Html => {
            let bytes = fs::read(&paths.source).with_context(|| {
                format!("failed to read {}; run fetch first", paths.source.display())
            })?;
            let text = extract_html_text(&bytes);
            if text.trim().is_empty() {
                bail!("HTML source extraction produced empty output");
            }
            fs::write(&paths.text, &text)?;
            lex_parse::Extraction {
                text,
                tool_version: concat!("lex-parse html extractor ", env!("CARGO_PKG_VERSION"))
                    .to_owned(),
            }
        }
    };
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
    if context.config.formal_source.is_some() {
        let bytes = fs::read(&paths.formal_source).with_context(|| {
            format!(
                "failed to read {}; run fetch first",
                paths.formal_source.display()
            )
        })?;
        let text = extract_html_text(&bytes);
        if text.trim().is_empty() {
            bail!("formal source extraction produced empty output");
        }
        fs::write(&paths.formal_text, &text)?;
        let mut formal_manifest: SourceManifest = read_json(&paths.formal_manifest)?;
        formal_manifest.extracted_text_sha256 = Some(sha256_hex(text.as_bytes()));
        formal_manifest.extraction_tool =
            Some(concat!("lex-parse html extractor ", env!("CARGO_PKG_VERSION")).to_owned());
        write_manifest(&formal_manifest, &paths.formal_manifest)?;
        println!(
            "extracted formal publication: {} UTF-8 bytes; sha256 {}",
            text.len(),
            formal_manifest
                .extracted_text_sha256
                .as_deref()
                .unwrap_or("unavailable")
        );
    }
    if !context.config.annex_pdf_urls.is_empty() {
        let mut manifests: Vec<SourceManifest> = read_json(&paths.annex_manifests)
            .with_context(|| "annex manifests not found; run fetch first")?;
        for (index, manifest) in manifests.iter_mut().enumerate() {
            let number = index + 1;
            let extraction =
                extract_pdf(&paths.annex_pdf(number), &paths.annex_text(number), true)?;
            manifest.extracted_text_sha256 = Some(sha256_hex(extraction.text.as_bytes()));
            manifest.extraction_tool = Some(extraction.tool_version);
            println!(
                "extracted annex {number}: {} UTF-8 bytes; sha256 {}",
                extraction.text.len(),
                manifest
                    .extracted_text_sha256
                    .as_deref()
                    .unwrap_or("unavailable")
            );
        }
        write_pretty_json(&manifests, &paths.annex_manifests)?;
    }
    Ok(())
}

/// Everything a configured parser yields for one instrument.
struct ParsedInstrument {
    enactment_evidence: Vec<lex_core::EnactmentEvidence>,
    provisions: Vec<lex_core::Provision>,
    /// Manifest of a directly acquired formal source, when the parser
    /// depends on one.
    formal_manifest: Option<SourceManifest>,
    /// The compiled document's REFERENCIAS legend, for instruments with
    /// margin amendment markers.
    amendment_references: Vec<lex_core::AmendmentReference>,
    /// Reform-transitory evidence the parser isolated from the compiled
    /// document itself. The LRITF parser instead extracts its
    /// reform-decree appendix separately in `run_parse`.
    reform_evidence: Vec<TemporalEvidence>,
    /// Latest amending-resolution date the parser derived from the
    /// document, when the `Última reforma` scan does not apply.
    latest_reform_date: Option<NaiveDate>,
}

#[derive(Debug, serde::Serialize)]
struct FurnitureAuditDocument {
    input: String,
    report: lex_parse::furniture_audit::FurnitureReport,
    admitted_furniture: Vec<lex_parse::furniture_audit::AdmittedFurniture>,
}

#[derive(Debug, serde::Serialize)]
struct IngestionFurnitureAudit {
    schema_version: String,
    instrument_id: String,
    documents: Vec<FurnitureAuditDocument>,
}

/// Run the positional audit for every PDF consumed by this parse. A repeated
/// band is only a gate when the parser has also admitted that band's text into
/// a provision; the complete read-only evidence remains in the work directory.
fn run_furniture_audit(
    context: &InstrumentContext,
    provisions: &[lex_core::Provision],
) -> Result<()> {
    let paths = &context.paths;
    let mut inputs = Vec::new();
    if matches!(context.config.source_format, SourceFormat::Pdf) {
        inputs.push(paths.source.clone());
    }
    inputs.extend((1..=context.config.annex_pdf_urls.len()).map(|number| paths.annex_pdf(number)));
    if inputs.is_empty() {
        return Ok(());
    }

    let provision_texts = provisions
        .iter()
        .map(|provision| (provision.id.as_str(), provision.text.as_str()))
        .collect::<Vec<_>>();
    let mut documents = Vec::with_capacity(inputs.len());
    let mut potential_count = 0;
    let mut blocking_count = 0;
    for input in inputs {
        let report = audit_pdf(&input)
            .with_context(|| format!("failed to audit PDF {}", input.display()))?;
        let admitted_furniture = find_admitted_furniture(&report, provision_texts.iter().copied());
        potential_count += admitted_furniture.len();
        blocking_count += admitted_furniture
            .iter()
            .filter(|finding| finding.blocks_ingestion())
            .count();
        documents.push(FurnitureAuditDocument {
            input: input.display().to_string(),
            report,
            admitted_furniture,
        });
    }
    let audit = IngestionFurnitureAudit {
        schema_version: SCHEMA_VERSION.to_owned(),
        instrument_id: context.config.instrument_id.clone(),
        documents,
    };
    let report_path = paths.work.join("furniture-audit.json");
    write_pretty_json(&audit, &report_path)?;
    println!(
        "furniture audit: {potential_count} potential finding(s), {blocking_count} blocking; report {}",
        report_path.display()
    );
    if blocking_count > 0 {
        bail!(
            "furniture audit found {blocking_count} recurring header/footer band(s) in at least three parsed provisions; inspect {}",
            report_path.display()
        );
    }
    Ok(())
}

fn read_annex_documents(paths: &Paths, config: &SourceConfig) -> Result<Vec<(u32, String)>> {
    (1..=config.annex_pdf_urls.len())
        .map(|number| {
            let text = fs::read_to_string(paths.annex_text(number)).with_context(|| {
                format!(
                    "failed to read {}; run extract first",
                    paths.annex_text(number).display()
                )
            })?;
            Ok((u32::try_from(number).expect("annex number fits u32"), text))
        })
        .collect()
}

fn find_adapter_path(root: &Path, slug: &str) -> Result<PathBuf> {
    for path in adapter_paths(root)? {
        if load_config(&path)?.slug == slug {
            return Ok(path);
        }
    }
    bail!("no adapter configuration found for instrument {slug:?} under adapters/")
}

/// Every adapter must have a unique slug, `short_name`, and `instrument_id`;
/// with many instruments a collision would silently cross-wire corpora
/// and vault folders.
fn preflight_adapter_uniqueness(root: &Path) -> Result<()> {
    let mut slugs: HashMap<String, PathBuf> = HashMap::new();
    let mut short_names: HashMap<String, PathBuf> = HashMap::new();
    let mut instrument_ids: HashMap<String, PathBuf> = HashMap::new();
    for path in adapter_paths(root)? {
        let config = load_config(&path)?;
        for (map, key, kind) in [
            (&mut slugs, config.slug.clone(), "slug"),
            (&mut short_names, config.short_name.clone(), "short_name"),
            (
                &mut instrument_ids,
                config.instrument_id.clone(),
                "instrument_id",
            ),
        ] {
            if let Some(existing) = map.insert(key.clone(), path.clone()) {
                bail!(
                    "duplicate adapter {kind} {key:?}: {} and {}",
                    existing.display(),
                    path.display()
                );
            }
        }
    }
    Ok(())
}

/// Generate a Diputados adapter from a batch-manifest entry. Counts and
/// the publication date start unfrozen (`null`); `batch run
/// --freeze-counts` fills them from the first successful parse.
/// `allow_article_gaps` scaffolds `false`: gap tolerance is a deliberate
/// per-instrument choice, made consciously by the operator after a first
/// parse hard-errors on a genuine gap, not a blanket default.
fn scaffold_adapter(root: &Path, manifest_path: &Path, slug: &str) -> Result<PathBuf> {
    let manifest = load_batch_manifest(manifest_path)?;
    let instrument = manifest
        .instruments
        .iter()
        .find(|entry| entry.slug == slug)
        .with_context(|| format!("{slug:?} is not in batch {}", manifest.batch_id))?;
    if instrument.adapter != "diputados" {
        bail!(
            "instrument {slug:?} uses adapter {:?}, which needs a hand-written config",
            instrument.adapter
        );
    }
    let adapter_path = root.join("adapters/diputados").join(format!("{slug}.json"));
    if adapter_path.exists() {
        bail!("adapter already exists at {}", adapter_path.display());
    }
    let (instrument_kind, urn_kind) = match instrument.instrument_type.as_str() {
        "codigo" => ("code", "code"),
        "reglamento" | "dcg" => ("regulation", "regulation"),
        _ => ("statute", "statute"),
    };
    let reference_line = instrument
        .ref_page
        .as_ref()
        .map_or_else(String::new, |url| {
            format!(
                "  \"reference_url\": {},\n",
                serde_json::json!(url.as_str())
            )
        });
    let contents = format!(
        "{{\n  \"slug\": {slug_json},\n  \"instrument_id\": \"urn:lex-mx:federal:{urn_kind}:{slug}\",\n  \"instrument_type\": \"{instrument_kind}\",\n  \"parser\": \"diputados\",\n  \"official_title\": {title},\n  \"short_name\": {short_name},\n  \"operational_source\": \"camara_de_diputados\",\n  \"source_url\": {source_url},\n{reference_line}  \"publisher\": \"Cámara de Diputados del H. Congreso de la Unión\",\n  \"publication_date\": null,\n  \"expected_articles\": null,\n  \"expected_transitories\": null,\n  \"allow_article_gaps\": false,\n  \"formal_publication_urls\": {{}}\n}}\n",
        slug_json = serde_json::json!(slug),
        title = serde_json::json!(lex_source::normalize_official_title(&instrument.title)),
        short_name = serde_json::json!(instrument.short_name()),
        source_url = serde_json::json!(instrument.source_pdf.as_str()),
    );
    fs::write(&adapter_path, &contents)?;
    load_config(&adapter_path)
        .with_context(|| format!("scaffolded adapter is invalid: {}", adapter_path.display()))?;
    println!("scaffolded {}", adapter_path.display());
    Ok(adapter_path)
}

/// Freeze machine-proposed baselines into a scaffolded adapter by exact
/// replacement of its `null` placeholder lines, so hand-written adapters
/// (which have no placeholders) can never be rewritten. Idempotent.
///
/// Frozen adapters carry only `expected_articles` (an exact count) and
/// `expected_transitories`; `expected_min_articles` is left for
/// hand-written open-instrument adapters (e.g. `lritf.json`, which sets a
/// minimum and no exact count while the law is still being amended) and is
/// never written here. The `expected_articles` placeholder is the presence
/// signal instead, so freezing tolerates adapters written with or without
/// an `expected_min_articles` placeholder.
fn freeze_adapter_baseline(
    adapter_path: &Path,
    publication_date: Option<NaiveDate>,
    article_count: usize,
    transitory_count: usize,
) -> Result<bool> {
    let original = fs::read_to_string(adapter_path)?;
    let mut updated = original.clone();
    if let Some(date) = publication_date {
        updated = updated.replace(
            "\"publication_date\": null",
            &format!("\"publication_date\": \"{}\"", date.format("%Y-%m-%d")),
        );
    }
    if updated.contains("\"expected_articles\": null") {
        updated = updated
            .replace(
                "\"expected_articles\": null",
                &format!("\"expected_articles\": {article_count}"),
            )
            .replace(
                "\"expected_transitories\": null",
                &format!(
                    "\"expected_transitories\": {transitory_count},\n  \"count_baseline\": {{\n    \"provenance\": \"machine-proposed\",\n    \"parser_version\": \"{SCHEMA_VERSION}\",\n    \"proposed_at\": \"{}\"\n  }}",
                    Utc::now().format("%Y-%m-%dT%H:%M:%SZ")
                ),
            );
    }
    if updated == original {
        return Ok(false);
    }
    fs::write(adapter_path, &updated)?;
    load_config(adapter_path)
        .with_context(|| format!("frozen adapter is invalid: {}", adapter_path.display()))?;
    Ok(true)
}

/// One instrument's structural pipeline for batch ingestion: fetch through
/// export with temporal analysis deliberately skipped (deferred per batch
/// by legal priority), freezing baselines when requested.
fn run_batch_instrument(
    root: &Path,
    slug: &str,
    freeze_counts: bool,
    keep_work: bool,
    obsidian_vault: Option<&Path>,
) -> Result<()> {
    let context = instrument_context(root, slug)?;
    run_fetch(&context)?;
    run_extract(&context)?;
    run_parse(root, &context)?;
    let report = run_validate(root, &context)?;
    if !report.valid {
        bail!("validation failed for {slug}");
    }
    if freeze_counts {
        let adapter_path = find_adapter_path(root, slug)?;
        let derived_date = if context.config.publication_date.is_none() {
            let raw = fs::read_to_string(&context.paths.text)?;
            Some(
                lex_parse::extract_dof_publication(&raw)
                    .with_context(|| format!("{slug}: no DOF publication note found to freeze"))?,
            )
        } else {
            None
        };
        if freeze_adapter_baseline(
            &adapter_path,
            derived_date,
            report.article_count,
            report.transitory_count,
        )? {
            println!(
                "froze machine-proposed baseline into {}",
                adapter_path.display()
            );
            // Re-validate with the frozen expectations so the committed
            // validation report carries the enforced counts, not the
            // unfrozen warnings.
            let context = instrument_context(root, slug)?;
            let report = run_validate(root, &context)?;
            if !report.valid {
                bail!("validation failed for {slug} after freezing its baseline");
            }
        }
    }
    let context = instrument_context(root, slug)?;
    run_export(root, &context, ExportFormat::Markdown, None)?;
    if let Some(vault) = obsidian_vault {
        run_export(root, &context, ExportFormat::Obsidian, Some(vault))?;
    }
    if !keep_work {
        cleanup_work(&context)?;
    }
    Ok(())
}

#[derive(Debug, serde::Serialize)]
struct BatchClosureReport {
    status: String,
    /// The batch's own instruments and every other instrument the corpus
    /// closure relinked.
    relinked: Vec<String>,
    /// Accepted-gap instruments outside the batch whose glossary alone the
    /// corpus closure refreshed.
    glossary_refreshed: Vec<String>,
    expected_edges: Vec<ExpectedEdgeCheck>,
}

#[derive(Debug, serde::Serialize)]
struct ExpectedEdgeCheck {
    expectation: String,
    status: String,
    detail: String,
}

#[derive(Debug)]
struct ExpectedEdgeSpec {
    source: String,
    source_article: Option<String>,
    target: String,
}

#[derive(Debug)]
struct ExpectedEdgeCorpus {
    slug: String,
    short_name: String,
    instrument_id: String,
    references: Vec<(String, String, ReferenceResolutionStatus)>,
}

/// Close the bounded graph for the successfully processed instruments.  Their
/// first parse could not see a later batch sibling, so this deliberately runs
/// after every selected source has entered the corpus. An admitted title can
/// also move another instrument's glossary usages (the title-span filter), so
/// the whole corpus is then closed once and must reject exactly the accepted
/// gaps; every instrument it relinks is validated and republished. An
/// accepted-gap instrument whose glossary alone it refreshed is republished
/// without the ordinary validation, whose recorded reference errors stand
/// (Codex review 2026-10-20); the closure already checked its coverage.
fn run_batch_closure(
    root: &Path,
    successful_slugs: &[String],
    expected_edges: &[String],
    obsidian_vault: Option<&Path>,
) -> Result<BatchClosureReport> {
    for slug in successful_slugs {
        let context = instrument_context(root, slug)?;
        run_link(root, &context)?;
    }
    let mut closed: BTreeSet<String> = successful_slugs.iter().cloned().collect();
    let mut glossary_refreshed = Vec::new();
    if !successful_slugs.is_empty() {
        let corpus = run_corpus_closure(root)?;
        if !closure_matches_accepted_gaps(root, &corpus)? {
            bail!(
                "corpus closure after the batch rejected {} instrument(s) beyond the accepted gaps; see .work/corpus-closure.json",
                corpus.rejected.len()
            );
        }
        closed.extend(corpus.relinked_instruments);
        glossary_refreshed = corpus.glossary_refreshed_instruments;
    }
    for slug in &closed {
        let context = instrument_context(root, slug)?;
        let report = run_validate(root, &context)?;
        if !report.valid {
            bail!("validation failed for {slug} after batch reverse linking");
        }
        republish_exports(root, &context, obsidian_vault)?;
    }
    for slug in &glossary_refreshed {
        republish_exports(root, &instrument_context(root, slug)?, obsidian_vault)?;
    }

    let corpora = expected_edge_corpora(root)?;
    let selected: HashSet<&str> = successful_slugs.iter().map(String::as_str).collect();
    let checks = evaluate_expected_edges(expected_edges, &selected, &corpora);
    let status = if checks
        .iter()
        .any(|check| matches!(check.status.as_str(), "missing" | "invalid"))
    {
        "failed"
    } else if successful_slugs.is_empty() {
        "skipped"
    } else {
        "ok"
    };
    Ok(BatchClosureReport {
        status: status.to_owned(),
        relinked: closed.into_iter().collect(),
        glossary_refreshed,
        expected_edges: checks,
    })
}

fn expected_edge_corpora(root: &Path) -> Result<Vec<ExpectedEdgeCorpus>> {
    read_committed_corpora(root).map(|corpora| {
        corpora
            .into_iter()
            .map(|(slug, corpus)| ExpectedEdgeCorpus {
                slug,
                short_name: corpus.instrument.short_name,
                instrument_id: corpus.instrument.id,
                references: corpus
                    .references
                    .into_iter()
                    .map(|edge| {
                        (
                            edge.source_provision_id,
                            edge.target_instrument_id,
                            edge.resolution_status,
                        )
                    })
                    .collect(),
            })
            .collect()
    })
}

fn parse_expected_edge(expectation: &str) -> Result<ExpectedEdgeSpec> {
    let (source, target) = expectation
        .split_once("->")
        .context("expected edge must contain one '->'")?;
    let source = source.trim();
    let target = target.trim();
    if source.is_empty() || target.is_empty() || target.contains("->") {
        bail!("expected edge must have one non-empty source and target");
    }
    let article = Regex::new(r"(?i)^(?P<instrument>.+?)\s+art[ií]culo\s+(?P<article>.+)$")
        .expect("expected-edge article selector regex");
    let (source, source_article) = if let Some(captures) = article.captures(source) {
        (
            captures["instrument"].trim().to_owned(),
            Some(captures["article"].trim().to_owned()),
        )
    } else {
        (source.to_owned(), None)
    };
    if source.is_empty() || source_article.as_ref().is_some_and(String::is_empty) {
        bail!("expected edge has an empty source selector");
    }
    Ok(ExpectedEdgeSpec {
        source,
        source_article,
        target: target.to_owned(),
    })
}

fn matching_expected_corpora<'a>(
    selector: &str,
    corpora: &'a [ExpectedEdgeCorpus],
) -> Vec<&'a ExpectedEdgeCorpus> {
    corpora
        .iter()
        .filter(|corpus| {
            corpus.slug.eq_ignore_ascii_case(selector)
                || corpus.short_name.eq_ignore_ascii_case(selector)
        })
        .collect()
}

fn evaluate_expected_edges(
    expected_edges: &[String],
    selected_slugs: &HashSet<&str>,
    corpora: &[ExpectedEdgeCorpus],
) -> Vec<ExpectedEdgeCheck> {
    expected_edges
        .iter()
        .map(|expectation| {
            let spec = match parse_expected_edge(expectation) {
                Ok(spec) => spec,
                Err(error) => {
                    return ExpectedEdgeCheck {
                        expectation: expectation.clone(),
                        status: "invalid".to_owned(),
                        detail: format!("{error:#}"),
                    };
                }
            };
            let sources = matching_expected_corpora(&spec.source, corpora);
            let targets = matching_expected_corpora(&spec.target, corpora);
            let source = match sources.as_slice() {
                [] => {
                    return ExpectedEdgeCheck {
                        expectation: expectation.clone(),
                        status: "invalid".to_owned(),
                        detail: format!("source {:?} is not a committed corpus", spec.source),
                    };
                }
                [source] => *source,
                _ => {
                    return ExpectedEdgeCheck {
                        expectation: expectation.clone(),
                        status: "invalid".to_owned(),
                        detail: format!("source {:?} is ambiguous", spec.source),
                    };
                }
            };
            if !selected_slugs.contains(source.slug.as_str()) {
                return ExpectedEdgeCheck {
                    expectation: expectation.clone(),
                    status: "deferred".to_owned(),
                    detail: format!("source {} was not processed in this batch run", source.slug),
                };
            }
            let target = match targets.as_slice() {
                [] => {
                    return ExpectedEdgeCheck {
                        expectation: expectation.clone(),
                        status: "deferred".to_owned(),
                        detail: format!("target {:?} is not a committed corpus", spec.target),
                    };
                }
                [target] => *target,
                _ => {
                    return ExpectedEdgeCheck {
                        expectation: expectation.clone(),
                        status: "invalid".to_owned(),
                        detail: format!("target {:?} is ambiguous", spec.target),
                    };
                }
            };
            let source_provision_id = spec.source_article.as_ref().map(|article| {
                format!(
                    "{}:article:{}",
                    source.instrument_id,
                    lex_parse::labels::canonical_slug(article)
                )
            });
            let resolved = source
                .references
                .iter()
                .any(|(edge_source, edge_target, status)| {
                    *status == ReferenceResolutionStatus::Resolved
                        && edge_target == &target.instrument_id
                        && source_provision_id
                            .as_ref()
                            .is_none_or(|expected_source| edge_source == expected_source)
                });
            ExpectedEdgeCheck {
                expectation: expectation.clone(),
                status: if resolved { "satisfied" } else { "missing" }.to_owned(),
                detail: if resolved {
                    format!("{} has a resolved edge to {}", source.slug, target.slug)
                } else {
                    let historical = source
                        .references
                        .iter()
                        .filter(|(_, _, status)| {
                            *status == ReferenceResolutionStatus::HistoricalTargetUnavailable
                        })
                        .count();
                    missing_edge_detail(
                        &source.slug,
                        spec.source_article.as_deref(),
                        &target.slug,
                        historical,
                    )
                },
            }
        })
        .collect()
}

/// Why an expected edge is missing. Historical-target edges are preserved
/// citations to superseded laws with no live text; they never satisfy an
/// expectation, and saying so keeps "missing" from reading as "not cited".
fn missing_edge_detail(
    source: &str,
    article: Option<&str>,
    target: &str,
    historical_edges: usize,
) -> String {
    let mut detail = match article {
        Some(article) => format!("{source} Article {article} has no resolved edge to {target}"),
        None => format!("{source} has no resolved edge to {target}"),
    };
    if historical_edges > 0 {
        detail = format!(
            "{detail}; {source} cites {historical_edges} superseded historical target(s), which are not live edges"
        );
    }
    detail
}

fn prepare_batch_instruments<'a>(
    root: &Path,
    manifest_path: &Path,
    selected: &[&'a lex_source::BatchInstrument],
    results: &mut Vec<serde_json::Value>,
) -> Vec<&'a lex_source::BatchInstrument> {
    let mut runnable = Vec::new();
    for instrument in selected {
        if find_adapter_path(root, &instrument.slug).is_ok() {
            runnable.push(*instrument);
        } else if instrument.adapter == "diputados" {
            match scaffold_adapter(root, manifest_path, &instrument.slug) {
                Ok(_) => runnable.push(*instrument),
                Err(error) => results.push(serde_json::json!({
                    "slug": instrument.slug,
                    "status": "failed",
                    "stage": "scaffold",
                    "error": format!("{error:#}"),
                })),
            }
        } else {
            results.push(serde_json::json!({
                "slug": instrument.slug,
                "status": "skipped",
                "reason": format!(
                    "adapter {:?} requires a hand-written config",
                    instrument.adapter
                ),
            }));
        }
    }
    runnable
}

fn batch_closure_report(
    root: &Path,
    batch_id: &str,
    failed: &[&str],
    successful_slugs: &[String],
    expected_edges: &[String],
    obsidian_vault: Option<&Path>,
) -> Result<(serde_json::Value, Option<String>)> {
    if !failed.is_empty() {
        return Ok((
            serde_json::json!({
                "status": "skipped",
                "detail": "one or more instruments failed before reverse-link closure",
            }),
            None,
        ));
    }
    match run_batch_closure(root, successful_slugs, expected_edges, obsidian_vault) {
        Ok(closure) => {
            let error = (closure.status == "failed").then(|| {
                let missing: Vec<_> = closure
                    .expected_edges
                    .iter()
                    .filter(|check| matches!(check.status.as_str(), "missing" | "invalid"))
                    .map(|check| check.expectation.as_str())
                    .collect();
                format!(
                    "batch {batch_id} did not satisfy expected edges: {}",
                    missing.join(", ")
                )
            });
            Ok((serde_json::to_value(closure)?, error))
        }
        Err(error) => Ok((
            serde_json::json!({
                "status": "failed",
                "detail": format!("{error:#}"),
            }),
            Some(format!(
                "batch {batch_id} reverse-link closure failed: {error:#}"
            )),
        )),
    }
}

fn run_batch(
    root: &Path,
    manifest_path: &Path,
    only: &[String],
    freeze_counts: bool,
    keep_work: bool,
    obsidian_vault: Option<&Path>,
) -> Result<()> {
    let manifest = load_batch_manifest(manifest_path)?;
    let selected: Vec<_> = manifest
        .instruments
        .iter()
        .filter(|instrument| only.is_empty() || only.contains(&instrument.slug))
        .collect();
    if selected.is_empty() {
        bail!("no instruments selected from batch {}", manifest.batch_id);
    }
    for blocked in &manifest.blocked {
        println!(
            "blocked (not fetched): {} — {}",
            blocked.slug, blocked.reason
        );
    }
    // Scaffold missing Diputados adapters up front, then check global
    // uniqueness before any network work.
    let mut results: Vec<serde_json::Value> = Vec::new();
    let runnable = prepare_batch_instruments(root, manifest_path, &selected, &mut results);
    preflight_adapter_uniqueness(root)?;
    for instrument in runnable {
        println!("— {} —", instrument.slug);
        match run_batch_instrument(
            root,
            &instrument.slug,
            freeze_counts,
            keep_work,
            obsidian_vault,
        ) {
            Ok(()) => results.push(serde_json::json!({
                "slug": instrument.slug,
                "status": "ok",
            })),
            Err(error) => results.push(serde_json::json!({
                "slug": instrument.slug,
                "status": "failed",
                "error": format!("{error:#}"),
            })),
        }
    }
    let failed: Vec<&str> = results
        .iter()
        .filter(|entry| entry["status"] == "failed")
        .filter_map(|entry| entry["slug"].as_str())
        .collect();
    let successful_slugs: Vec<String> = results
        .iter()
        .filter(|entry| entry["status"] == "ok")
        .filter_map(|entry| entry["slug"].as_str().map(str::to_owned))
        .collect();
    let (closure, closure_error) = batch_closure_report(
        root,
        &manifest.batch_id,
        &failed,
        &successful_slugs,
        &manifest.expected_edges,
        obsidian_vault,
    )?;
    let report_path = root
        .join(".work")
        .join(format!("batch-report-{}.json", manifest.batch_id));
    fs::create_dir_all(report_path.parent().expect("report parent"))?;
    write_pretty_json(
        &serde_json::json!({
            "batch_id": manifest.batch_id,
            "generated_at": Utc::now().to_rfc3339(),
            "results": results,
            "closure": closure,
        }),
        &report_path,
    )?;
    println!("batch report: {}", report_path.display());
    for entry in &results {
        println!(
            "{}: {}{}",
            entry["slug"].as_str().unwrap_or("?"),
            entry["status"].as_str().unwrap_or("?"),
            entry["error"]
                .as_str()
                .map(|error| format!(" — {error}"))
                .unwrap_or_default()
        );
    }
    if !failed.is_empty() {
        bail!(
            "batch {} had failures: {}",
            manifest.batch_id,
            failed.join(", ")
        );
    }
    if let Some(error) = closure_error {
        bail!("{error}");
    }
    Ok(())
}

/// Default self-reference (internal) markers for an instrument that does
/// not configure its own. The self-reference noun tracks the instrument's
/// kind: a código's "de este código" is internal, while for a ley the
/// same phrase names a different instrument and stays external (via the
/// generic external markers). "ordenamiento" is a kind-neutral
/// self-reference used by both.
fn self_reference_markers(instrument_type: &str) -> Vec<String> {
    let own = match instrument_type {
        "code" => ["de este código", "del presente código", "este código"].as_slice(),
        "regulation" => [
            "de este reglamento",
            "del presente reglamento",
            "de las presentes disposiciones",
            "de estas disposiciones",
        ]
        .as_slice(),
        // A constitution cites itself as "esta Constitución" or by the
        // adjective ("el artículo 123 constitucional"). Without these, a later
        // official title in the sentence would capture the citation.
        "constitution" => [
            "de esta constitución",
            "de la presente constitución",
            "esta constitución",
            " constitucional",
        ]
        .as_slice(),
        // statute and anything else are ley-shaped.
        _ => ["de esta ley", "de la presente ley", "esta ley"].as_slice(),
    };
    own.iter()
        .copied()
        .chain(["de este ordenamiento", "del presente ordenamiento"])
        .map(str::to_owned)
        .collect()
}

/// Parser options for a Diputados consolidated document, derived from the
/// adapter: running-header lines default to the uppercased official title.
fn diputados_options(config: &SourceConfig) -> DiputadosOptions {
    let header_lines = if config.header_lines.is_empty() {
        vec![config.official_title.to_uppercase()]
    } else {
        config.header_lines.clone()
    };
    DiputadosOptions {
        instrument_id: config.instrument_id.clone(),
        header_lines,
        stop_markers: config.main_document_stop_markers.clone(),
        annex_markers: config.substantive_annex_markers.clone(),
        separate_enactment: config.separate_enactment,
        nested_instrument: config.nested_instrument.clone(),
        separatorless_article_headings: config.separatorless_article_headings.clone(),
    }
}

/// Dispatch to the adapter-configured parser.
fn parse_by_configured_parser(
    paths: &Paths,
    config: &SourceConfig,
    raw: &str,
    publication_date: Option<NaiveDate>,
) -> Result<ParsedInstrument> {
    match config.parser.as_str() {
        "diputados" => {
            let document = lex_parse::parse_diputados_publication(
                raw,
                &diputados_options(config),
                publication_date,
                config.publication_period.as_ref(),
            )?;
            let formal_manifest = config
                .formal_source
                .as_ref()
                .map(|_| read_json(&paths.formal_manifest))
                .transpose()?;
            Ok(ParsedInstrument {
                provisions: document.provisions,
                formal_manifest,
                amendment_references: Vec::new(),
                enactment_evidence: document.enactment_evidence,
                reform_evidence: document.reform_evidence,
                latest_reform_date: None,
            })
        }
        "ifpe-dcg" => {
            let publication_date =
                publication_date.context("DCG parser requires an exact publication day")?;
            let formal_manifest: SourceManifest = read_json(&paths.formal_manifest)?;
            let annex_documents = read_annex_documents(paths, config)?;
            let provisions = parse_dcg(
                raw,
                &annex_documents,
                &config.instrument_id,
                publication_date,
                &config.definition_layout_articles,
            )?;
            Ok(ParsedInstrument {
                provisions,
                formal_manifest: Some(formal_manifest),
                amendment_references: Vec::new(),
                enactment_evidence: Vec::new(),
                reform_evidence: Vec::new(),
                latest_reform_date: None,
            })
        }
        "itf-dcg" => {
            let publication_date =
                publication_date.context("ITF parser requires an exact publication day")?;
            let annex_documents = read_annex_documents(paths, config)?;
            let document = parse_itf_dcg(
                raw,
                &annex_documents,
                &config.instrument_id,
                publication_date,
            )?;
            Ok(ParsedInstrument {
                provisions: document.provisions,
                formal_manifest: None,
                amendment_references: document.amendment_references,
                enactment_evidence: Vec::new(),
                reform_evidence: document.reform_evidence,
                latest_reform_date: document.latest_reform_date,
            })
        }
        other => bail!("unsupported parser {other:?} in adapter configuration"),
    }
}

const OGA_SERIAL_NOTE: &str = "Nueva Ordenanza publicada en el Diario Oficial de la Federación en forma seriada los días 1º al 8 de enero de 1912";

fn resolve_publication(
    config: &SourceConfig,
    manifest: &SourceManifest,
    raw: &str,
) -> Result<Option<NaiveDate>> {
    if let Some(period) = &config.publication_period {
        if config.slug != "oga"
            || config.instrument_id != "urn:lex-mx:federal:statute:oga"
            || config.publication_date.is_some()
            || period.start.to_string() != "1912-01-01"
            || period.end.to_string() != "1912-01-08"
            || period.source_note != OGA_SERIAL_NOTE
            || period.source_sha256 != manifest.source_sha256
            || !raw
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .contains(OGA_SERIAL_NOTE)
        {
            bail!("invalid OGA serialized-publication exception or mismatched source evidence");
        }
        return Ok(None);
    }
    match &config.publication_date {
        Some(date) => Ok(Some(NaiveDate::parse_from_str(date, "%Y-%m-%d")?)),
        None => Ok(Some(lex_parse::extract_dof_publication(raw).with_context(
            || {
                format!(
                    "adapter for {} has no exact publication date or supported original DOF note",
                    config.slug
                )
            },
        )?)),
    }
}

fn validate_source_metadata(corpus: &Corpus) -> Result<()> {
    lex_core::validate_publication_metadata(corpus).map_err(anyhow::Error::msg)
}

fn validate_repeal_config(
    root: &Path,
    config: &SourceConfig,
    manifest: &SourceManifest,
    raw: &str,
) -> Result<()> {
    if let Some(notice) = &config.source_repeal_notice {
        if notice.successor_clause.is_some() {
            anyhow::ensure!(
                read_sibling_corpora(root, &config.instrument_id)?
                    .iter()
                    .any(|(_, c)| notice.matches_successor(&manifest.source_sha256, c)),
                "repeal notice does not match the abrogating provision of its successor"
            );
        } else {
            anyhow::ensure!(
                notice.matches_source(&manifest.source_sha256, raw),
                "repeal notice does not match retained source bytes/header"
            );
        }
        if let Some(successor) = &notice.successor_instrument_id {
            anyhow::ensure!(
                read_sibling_corpora(root, &config.instrument_id)?
                    .iter()
                    .any(|(_, c)| &c.instrument.id == successor),
                "repeal notice successor is not in the corpus"
            );
        }
    }
    Ok(())
}

/// Recheck persisted cross-corpus evidence before validation or publication.
/// Source-header verification still belongs to parsing the retained acquisition;
/// successor clauses can be checked against canonical data without source caches.
fn validate_persisted_repeal_evidence(root: &Path, corpus: &Corpus) -> Result<()> {
    let Some(notice) = &corpus.instrument.source_repeal_notice else {
        return Ok(());
    };
    let siblings = read_sibling_corpora(root, &corpus.instrument.id)?;
    if notice.successor_clause.is_some() {
        anyhow::ensure!(
            siblings.iter().any(|(_, successor)| {
                notice.matches_successor(&corpus.instrument.source_sha256, successor)
            }),
            "persisted repeal notice successor evidence is missing or stale; audited review required"
        );
    }
    if let Some(successor) = &notice.successor_instrument_id {
        anyhow::ensure!(
            siblings.iter().any(|(_, c)| &c.instrument.id == successor),
            "persisted repeal notice successor is missing; audited review required"
        );
    }
    Ok(())
}

fn run_parse(root: &Path, context: &InstrumentContext) -> Result<()> {
    let paths = &context.paths;
    let config = &context.config;
    let manifest: SourceManifest = read_json(&paths.manifest)?;
    let raw = fs::read_to_string(&paths.text)
        .with_context(|| format!("failed to read {}", paths.text.display()))?;
    let publication_date = resolve_publication(config, &manifest, &raw)?;
    let extracted_text_sha256 = manifest
        .extracted_text_sha256
        .clone()
        .context("manifest lacks extracted text hash; run extract first")?;

    let parsed = parse_by_configured_parser(paths, config, &raw, publication_date)?;
    let (provisions, formal_manifest) = (parsed.provisions, parsed.formal_manifest);

    validate_repeal_config(root, config, &manifest, &raw)?;

    let instrument = Instrument {
        schema_version: SCHEMA_VERSION.to_owned(),
        id: config.instrument_id.clone(),
        jurisdiction: "mx".to_owned(),
        level: "federal".to_owned(),
        instrument_type: instrument_type(&config.instrument_type)?,
        official_title: config.official_title.clone(),
        short_name: config.short_name.clone(),
        operational_source: manifest.operational_source.clone(),
        formal_publication_source: manifest.formal_publication_source.clone(),
        publication_date,
        publication_period: config.publication_period.clone(),
        latest_reform_date: match &config.latest_reform_date {
            Some(date) => Some(NaiveDate::parse_from_str(date, "%Y-%m-%d")?),
            None => parsed
                .latest_reform_date
                .or_else(|| latest_reform_date(&raw)),
        },
        retrieved_at: manifest.retrieved_at,
        source_url: manifest.official_url,
        source_sha256: manifest.source_sha256,
        extracted_text_sha256,
        parser_version: env!("CARGO_PKG_VERSION").to_owned(),
        status: if config.source_repeal_notice.is_some() {
            InstrumentStatus::Repealed
        } else {
            InstrumentStatus::InForce
        },
        source_repeal_notice: config.source_repeal_notice.clone(),
        issuing_authorities: config.issuing_authorities.clone(),
        formal_publication_url: config
            .formal_source
            .as_ref()
            .map(|formal| formal.url.clone()),
        formal_publication_code: config
            .formal_source
            .as_ref()
            .map(|formal| formal.publication_code.clone()),
        formal_source_sha256: formal_manifest
            .as_ref()
            .map(|manifest| manifest.source_sha256.clone()),
        formal_extracted_text_sha256: formal_manifest
            .as_ref()
            .and_then(|manifest| manifest.extracted_text_sha256.clone()),
    };
    let (references, extraction) =
        links_for_parsed_instrument(root, context, &instrument, &provisions)?;
    // Reform evidence must be computed before reapplication: an
    // amendment-event determination's provision ID lives only in reform
    // evidence, never among canonical provisions, and reapplication needs
    // the freshly reparsed evidence, not whatever was last written to disk.
    // Every parser with a reform-evidence concept returns it from the same
    // parse of the same raw text.
    let reform_evidence = parsed.reform_evidence;
    let mut corpus = Corpus {
        enactment_evidence: parsed.enactment_evidence,
        instrument,
        provisions,
        references,
        terms: extraction.terms,
        term_usages: extraction.usages,
        amendment_references: parsed.amendment_references,
    };
    for item in &mut corpus.enactment_evidence {
        item.source_sha256
            .clone_from(&corpus.instrument.source_sha256);
    }
    validate_source_metadata(&corpus)?;
    reapply_persisted_temporal_state(paths, config, &mut corpus, &reform_evidence)?;
    run_furniture_audit(context, &corpus.provisions)?;
    write_canonical(&corpus, &paths.corpus)?;
    write_glossary_coverage(&extraction.coverage, &paths.corpus)?;
    println!("parsed {} canonical provisions", corpus.provisions.len());
    println!("extracted {} canonical references", corpus.references.len());
    println!(
        "extracted {} defined terms with {} usages",
        corpus.terms.len(),
        corpus.term_usages.len()
    );
    // Written unconditionally (even as an empty array) for every parser
    // that has a reform-evidence concept at all, so the file always
    // reflects the current parse rather than silently going stale if a
    // future reparse ever finds zero reform transitories where a prior
    // parse found some.
    if matches!(config.parser.as_str(), "diputados" | "itf-dcg") {
        write_pretty_json(&reform_evidence, &paths.reform_evidence)?;
        println!(
            "isolated {} reform transitories for temporal analysis",
            reform_evidence.len()
        );
    }
    if !corpus.amendment_references.is_empty() {
        println!(
            "recorded {} amendment-legend references",
            corpus.amendment_references.len()
        );
    }
    Ok(())
}

/// A reparse must never erase applied temporal state — including audited
/// lawyer-verified decisions. Re-apply the persisted result, using the
/// freshly reparsed evidence (ordinary transitories plus the reform
/// evidence just extracted from this same parse) so a determination is
/// re-applied only when its exact evidence text is unchanged, and an
/// amendment-event determination resolves correctly instead of being
/// treated as if its provision vanished.
fn reapply_persisted_temporal_state(
    paths: &Paths,
    config: &SourceConfig,
    corpus: &mut Corpus,
    reform_evidence: &[TemporalEvidence],
) -> Result<()> {
    if !paths.temporal_result.exists() {
        return preserve_verified_provisions(paths, corpus);
    }
    let result: TemporalAnalysisResult = read_json(&paths.temporal_result)?;
    let current_evidence: HashMap<String, String> =
        build_temporal_evidence(config, &corpus.provisions, reform_evidence)
            .into_iter()
            .map(|evidence| (evidence.provision_id, evidence.text))
            .collect();
    let outcome = reapply_temporal_determinations(
        &mut corpus.provisions,
        &result.determinations,
        &current_evidence,
    );
    anyhow::ensure!(
        !result.determinations.iter().any(|determination| {
            determination.basis == lex_core::Basis::LawyerVerified
                && outcome.stale.contains(&determination.provision_id)
        }),
        "refusing to clear a lawyer-verified decision with stale or missing evidence; audited migration required"
    );
    preserve_verified_provisions(paths, corpus)?;
    println!(
        "re-applied {} persisted temporal determinations",
        outcome.current.len()
    );
    if !outcome.stale.is_empty() {
        eprintln!(
            "warning: {} determination(s) no longer ground in the reparsed text (or lack \
             recorded evidence provenance) and were not re-applied; rerun temporal analysis \
             and review: {}",
            outcome.stale.len(),
            outcome.stale.join(", ")
        );
    }
    Ok(())
}

fn links_for_parsed_instrument(
    root: &Path,
    context: &InstrumentContext,
    instrument: &Instrument,
    provisions: &[lex_core::Provision],
) -> Result<(Vec<lex_core::ReferenceEdge>, TermExtraction)> {
    let loaded_siblings = read_sibling_corpora(root, &instrument.id)?;
    let siblings = corpus_views(&loaded_siblings);
    let references =
        extract_instrument_references(root, context, instrument, provisions, &siblings)?;
    let extraction = extract_instrument_terms(context, instrument, provisions, &siblings)?;
    Ok((references, extraction))
}

fn write_glossary_coverage(coverage: &lex_core::GlossaryCoverage, directory: &Path) -> Result<()> {
    write_pretty_json(coverage, &directory.join(GLOSSARY_COVERAGE_FILE))
}

/// A sidecar can be missing or incomplete. The existing applied human decision
/// is also authoritative and must not be silently removed by an ordinary parse.
fn preserve_verified_provisions(paths: &Paths, corpus: &Corpus) -> Result<()> {
    if !paths.provisions.is_file() {
        return Ok(());
    }
    let prior: Vec<lex_core::Provision> = read_json(&paths.provisions)?;
    for previous in prior.iter().filter(|p| {
        p.review_status == lex_core::ReviewStatus::LawyerVerified
            || p.temporal_basis == Some(lex_core::Basis::LawyerVerified)
    }) {
        let current = corpus
            .provisions
            .iter()
            .find(|p| p.id == previous.id)
            .with_context(|| {
                format!(
                    "refusing to remove lawyer-verified provision {}; audited migration required",
                    previous.id
                )
            })?;
        // Compare the full reviewed provision, including its exact source text,
        // dates and effects. Harmless-looking source changes still need review.
        anyhow::ensure!(
            serde_json::to_value(previous)? == serde_json::to_value(current)?,
            "refusing to replace lawyer-verified provision {}; audited migration required",
            previous.id
        );
    }
    Ok(())
}

/// The file beside `terms.json` recording how that file was produced.
const GLOSSARY_COVERAGE_FILE: &str = "glossary-coverage.json";

/// An instrument's defined terms, their usages and how the terms were
/// extracted ([`lex_core::GlossaryCoverage`]).
struct TermExtraction {
    terms: Vec<lex_core::DefinedTerm>,
    usages: Vec<lex_core::TermUsage>,
    coverage: lex_core::GlossaryCoverage,
}

/// Extract the instrument's glossary terms and every exact term usage.
/// A glossary that is expressly additive to another instrument's glossary
/// (the DCG's Article 1 opens "además de los términos utilizados en la
/// Ley…") resolves usages against its own terms first, then the terms of
/// each instrument listed in `additive_to`, in order. An instrument may
/// configure further glossary provisions (`additional_provisions`), each
/// defining a different set of terms; their terms add to the first's.
fn extract_instrument_terms(
    context: &InstrumentContext,
    instrument: &Instrument,
    provisions: &[lex_core::Provision],
    siblings: &[(&str, &Corpus)],
) -> Result<TermExtraction> {
    let SelectedGlossary {
        terms,
        additive_to,
        status,
        glossaries,
        hold,
    } = select_glossary(context, provisions)?;
    let usage_form = context
        .config
        .glossary
        .as_ref()
        .and_then(|glossary| glossary.usage_form.clone());
    let coverage = |imports, usage_count| lex_core::GlossaryCoverage {
        schema_version: lex_core::SCHEMA_VERSION.to_owned(),
        strategy_version: lex_core::GLOSSARY_STRATEGY_VERSION.to_owned(),
        instrument_id: instrument.id.clone(),
        source_sha256: instrument.source_sha256.clone(),
        extracted_text_sha256: instrument.extracted_text_sha256.clone(),
        status,
        glossaries: glossaries.clone(),
        hold: hold.clone(),
        imports,
        usage_form: usage_form.clone(),
        term_count: terms.len(),
        usage_count,
    };
    if terms.is_empty() {
        return Ok(TermExtraction {
            coverage: coverage(Vec::new(), 0),
            terms: Vec::new(),
            usages: Vec::new(),
        });
    }

    let adopted = additive_to
        .iter()
        .map(|additive| additive_terms(additive, siblings))
        .collect::<Result<Vec<_>>>()?;
    let mut term_sets: Vec<&[lex_core::DefinedTerm]> = vec![&terms];
    term_sets.extend(adopted.iter().map(Vec::as_slice));
    let usages = match usage_form.as_deref() {
        None => extract_term_usages(provisions, &term_sets)?,
        Some("quoted") => extract_quoted_term_usages(provisions, &term_sets)?,
        Some("with_lowercase") => extract_term_usages_with_lowercase(provisions, &term_sets)?,
        Some(other) => bail!("unsupported glossary usage_form {other:?}"),
    };
    let usages = match &context.config.glossary {
        Some(glossary) => {
            exclude_reviewed_non_usages(&instrument.id, usages, &glossary.usage_exclusions)?
        }
        None => usages,
    };
    let usages = if usage_form.as_deref() == Some("quoted") {
        usages
    } else {
        let definitions = term_definitions(&term_sets, provisions, siblings);
        let phrases = title_phrases(&context.root, instrument, siblings, &terms, provisions)?;
        let index = title_index(&phrases)?;
        drop_usages_in_titles(usages, provisions, &definitions, &index)
    };
    let imports = additive_to
        .iter()
        .zip(&adopted)
        .map(|(additive, terms)| lex_core::GlossaryImportCoverage {
            additive_to: additive.clone(),
            term_count: terms.len(),
        })
        .collect();
    Ok(TermExtraction {
        coverage: coverage(imports, usages.len()),
        terms,
        usages,
    })
}

/// Instrument titles and institution names as usage boundaries: every
/// committed instrument's official title with more than one word, every
/// curated multi-word alias whose instrument is committed, and every
/// institution's full name a glossary prints as a definition
/// ([`institution_names`]), lower-cased.
fn title_phrases(
    root: &Path,
    instrument: &Instrument,
    siblings: &[(&str, &Corpus)],
    terms: &[lex_core::DefinedTerm],
    provisions: &[lex_core::Provision],
) -> Result<Vec<String>> {
    let mut instruments: Vec<&Instrument> = vec![instrument];
    instruments.extend(siblings.iter().map(|(_, corpus)| &corpus.instrument));
    let mut phrases: BTreeSet<String> = instruments
        .iter()
        .map(|candidate| candidate.official_title.trim().to_lowercase())
        .filter(|title| title.contains(' '))
        .collect();
    let alias_path = root.join("adapters/diputados/_instrument-aliases.json");
    if alias_path.is_file() {
        let table: InstrumentAliasTable = read_json(&alias_path)?;
        let committed: HashSet<&str> = instruments
            .iter()
            .map(|candidate| candidate.short_name.as_str())
            .collect();
        for (short_name, aliases) in &table.aliases {
            if committed.contains(short_name.as_str()) {
                phrases.extend(
                    aliases
                        .iter()
                        .filter(|alias| alias.contains(' '))
                        .map(|alias| alias.to_lowercase()),
                );
            }
        }
    }
    phrases.extend(institution_names(terms, provisions));
    for (_, corpus) in siblings {
        phrases.extend(committed_institution_names(corpus)?);
    }
    Ok(phrases.into_iter().collect())
}

/// [`institution_names`] of a committed corpus, kept per instrument because
/// `link --all` asks for every sibling's once per instrument.
fn committed_institution_names(corpus: &Corpus) -> Result<Vec<String>> {
    type Cache = HashMap<String, (String, Vec<String>)>;
    static CACHE: std::sync::Mutex<Option<Cache>> = std::sync::Mutex::new(None);
    let fingerprint = corpus
        .terms
        .iter()
        .map(|term| format!("{}@{}-{}", term.id, term.start_char, term.end_char))
        .chain(std::iter::once(
            corpus.instrument.extracted_text_sha256.clone(),
        ))
        .collect::<Vec<_>>()
        .join(",");
    let mut cache = CACHE
        .lock()
        .map_err(|_| anyhow::anyhow!("institution name cache poisoned"))?;
    let cache = cache.get_or_insert_with(HashMap::new);
    if let Some((cached, names)) = cache.get(&corpus.instrument.id)
        && *cached == fingerprint
    {
        return Ok(names.clone());
    }
    let names = institution_names(&corpus.terms, &corpus.provisions);
    cache.insert(corpus.instrument.id.clone(), (fingerprint, names.clone()));
    Ok(names)
}

/// Words that name an institution or a part of one.
const INSTITUTION_WORDS: &[&str] = &[
    "Administración",
    "Agencia",
    "Auditoría",
    "Banco",
    "Cámara",
    "Centro",
    "Comisión",
    "Consejo",
    "Coordinación",
    "Dirección",
    "Fiscalía",
    "Fondo",
    "Instituto",
    "Junta",
    "Oficina",
    "Organismo",
    "Procuraduría",
    "Registro",
    "Secretaría",
    "Servicio",
    "Sistema",
    "Subsecretaría",
    "Suprema",
    "Tribunal",
    "Unidad",
];

/// The full names of institutions that glossaries define, lower-cased: a
/// definition that opens with one (`Secretaría: Secretaría de Medio Ambiente
/// y Recursos Naturales`; `Procuraduría: La Procuraduría Federal de
/// Protección al Ambiente`). A term inside such a name is the name's word,
/// not a use of the term (LGEEPA's `Ambiente` in the Procuraduría's name;
/// `Secretaría`, defined as the Secretaría de Economía, in `Secretaría de
/// Hacienda y Crédito Público`), unless the term's own definition names it
/// (operator 2026-10-20: "when in doubt use the full exact name"). A name
/// counts only with three or more words, and not when it refers to another
/// body inside it: a name ending in an institution word or an abbreviation
/// (`Consejo Técnico del Instituto`, `Consejo Consultivo de la AMEXCID`),
/// or with an institution word followed by a connector or the end
/// (`Fiscalía Especializada de la Fiscalía y de las Procuradurías`).
fn institution_names(
    terms: &[lex_core::DefinedTerm],
    provisions: &[lex_core::Provision],
) -> Vec<String> {
    static NAME: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
        let institution = INSTITUTION_WORDS.join("|");
        let word = r"[A-ZÁÉÍÓÚÑ][\wáéíóúñü-]*";
        let connector = r"(?:de|del|la|las|los|el|y|e|para|al|en|a)";
        Regex::new(&format!(
            r#"\A[\s"“”:,.;-]*(?:(?:La|El|Los|Las|la|el|los|las|a la|al|A la|Al)\s+)?((?:{institution})(?:\s+(?:{connector}\s+)*{word})+)"#
        ))
        .expect("institution name regex must compile")
    });
    let connectors = [
        "de", "del", "la", "las", "los", "el", "y", "e", "para", "al", "en", "a",
    ];
    let institution_word = |word: &str| {
        INSTITUTION_WORDS
            .iter()
            .any(|institution| word == *institution || word.strip_suffix('s') == Some(institution))
    };
    let texts: HashMap<&str, &str> = provisions
        .iter()
        .map(|provision| (provision.id.as_str(), provision.text.as_str()))
        .collect();
    let mut names = BTreeSet::new();
    for term in terms {
        let Some(text) = texts.get(term.defining_provision_id.as_str()) else {
            continue;
        };
        let entry: String = text
            .chars()
            .skip(term.start_char)
            .take(term.end_char.saturating_sub(term.start_char))
            .collect();
        let Some(after) = entry
            .find(&term.term)
            .map(|at| &entry[at + term.term.len()..])
        else {
            continue;
        };
        let Some(found) = NAME.captures(after).and_then(|captures| captures.get(1)) else {
            continue;
        };
        let mut words: Vec<&str> = found.as_str().split_whitespace().collect();
        while words.last().is_some_and(|word| connectors.contains(word)) {
            words.pop();
        }
        let Some(last) = words.last() else {
            continue;
        };
        let refers_inside = words.iter().enumerate().skip(1).any(|(index, word)| {
            institution_word(word)
                && words
                    .get(index + 1)
                    .is_none_or(|next| connectors.contains(next))
        });
        let abbreviation = last.chars().count() > 1 && last.chars().all(|c| !c.is_lowercase());
        if words.len() >= 3 && !refers_inside && !abbreviation && !institution_word(last) {
            names.insert(words.join(" ").to_lowercase());
        }
    }
    names.into_iter().collect()
}

/// Instrument titles, lower-cased with single spaces, indexed by their first
/// word and longest first within it.
struct TitleIndex {
    by_first_word: HashMap<String, Vec<Vec<char>>>,
    longest: usize,
}

/// The index of `phrases`. `link --all` asks once per instrument for the
/// same corpus-wide set, so the last index is kept.
fn title_index(phrases: &[String]) -> Result<std::sync::Arc<TitleIndex>> {
    static CACHE: std::sync::Mutex<Option<(Vec<String>, std::sync::Arc<TitleIndex>)>> =
        std::sync::Mutex::new(None);
    let mut cache = CACHE
        .lock()
        .map_err(|_| anyhow::anyhow!("title index cache poisoned"))?;
    if let Some((cached, index)) = cache.as_ref()
        && cached.as_slice() == phrases
    {
        return Ok(index.clone());
    }
    let mut by_first_word: HashMap<String, Vec<Vec<char>>> = HashMap::new();
    for phrase in phrases {
        let title: Vec<char> = normalized(phrase).chars().collect();
        let first: String = title.iter().take_while(|c| c.is_alphanumeric()).collect();
        if !first.is_empty() {
            by_first_word.entry(first).or_default().push(title);
        }
    }
    for titles in by_first_word.values_mut() {
        titles.sort_by_key(|title| std::cmp::Reverse(title.len()));
    }
    let longest = by_first_word
        .values()
        .flatten()
        .map(Vec::len)
        .max()
        .unwrap_or(0);
    let index = std::sync::Arc::new(TitleIndex {
        by_first_word,
        longest,
    });
    *cache = Some((phrases.to_vec(), index.clone()));
    Ok(index)
}

fn normalized(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn lower(c: char) -> char {
    c.to_lowercase().next().unwrap_or(c)
}

/// Where `title` ends when it is printed at `start` of `text`: any case,
/// any run of whitespace for each space, and not running on into a word.
fn title_end(text: &[char], start: usize, title: &[char]) -> Option<usize> {
    let mut at = start;
    for expected in title {
        if *expected == ' ' {
            let from = at;
            while text.get(at).is_some_and(|c| c.is_whitespace()) {
                at += 1;
            }
            if at == from {
                return None;
            }
        } else if text.get(at).map(|c| lower(*c)) == Some(*expected) {
            at += 1;
        } else {
            return None;
        }
    }
    let ends_in_word = title.last().is_some_and(|c| c.is_alphanumeric());
    (!ends_in_word || text.get(at).is_none_or(|c| !c.is_alphanumeric())).then_some(at)
}

/// The earliest position a title of `longest` characters, lower-cased with
/// single spaces, can start at and still reach `start`: a run of whitespace
/// counts as the one space it matches in the title.
fn lookback_start(text: &[char], start: usize, longest: usize) -> usize {
    let mut begin = start.min(text.len());
    let mut units = 0;
    while begin > 0 && units < longest {
        begin -= 1;
        let continues_run =
            text[begin].is_whitespace() && text.get(begin + 1).is_some_and(|c| c.is_whitespace());
        if !continues_run {
            units += 1;
        }
    }
    begin
}

/// Whether `text` contains `phrase` as whole words.
fn contains_phrase(text: &str, phrase: &str) -> bool {
    text.match_indices(phrase).any(|(at, found)| {
        let before = text[..at].chars().next_back();
        let after = text[at + found.len()..].chars().next();
        let edge = |c: Option<char>| c.is_none_or(|c| !c.is_alphanumeric());
        (!phrase.starts_with(char::is_alphanumeric) || edge(before))
            && (!phrase.ends_with(char::is_alphanumeric) || edge(after))
    })
}

/// Every title printed over `[start, end)` of `text`: a title nested in a
/// longer one (`Ley General de Salud` in `Reglamento de la Ley General de
/// Salud en Materia de …`) covers it too.
fn covering_titles(text: &[char], start: usize, end: usize, index: &TitleIndex) -> Vec<String> {
    let mut covering = Vec::new();
    for begin in
        lookback_start(text, start, index.longest)..=start.min(text.len().saturating_sub(1))
    {
        if !text[begin].is_alphanumeric() || (begin > 0 && text[begin - 1].is_alphanumeric()) {
            continue;
        }
        let word: String = text[begin..]
            .iter()
            .take_while(|c| c.is_alphanumeric())
            .map(|c| lower(*c))
            .collect();
        for title in index.by_first_word.get(&word).into_iter().flatten() {
            if title_end(text, begin, title).is_some_and(|stop| stop >= end) {
                covering.push(title.iter().collect());
            }
        }
    }
    covering
}

/// Drop usages inside an instrument title: they are the title's words, not
/// uses of the term (`Ley` in `Ley Federal de Competencia Económica`; lan's
/// `Aguas Nacionales` in `Ley de Aguas Nacionales`). A usage stays when the
/// term's own definition names a title that covers it, its source-backed
/// referent: reg-lan defines `Ley` as `la Ley de Aguas Nacionales`. Citation
/// edges are unaffected.
fn drop_usages_in_titles(
    usages: Vec<lex_core::TermUsage>,
    provisions: &[lex_core::Provision],
    definitions: &HashMap<String, String>,
    index: &TitleIndex,
) -> Vec<lex_core::TermUsage> {
    let used: HashSet<&str> = usages
        .iter()
        .map(|usage| usage.provision_id.as_str())
        .collect();
    let texts: HashMap<String, Vec<char>> = provisions
        .iter()
        .filter(|provision| used.contains(provision.id.as_str()))
        .map(|provision| (provision.id.clone(), provision.text.chars().collect()))
        .collect();
    usages
        .into_iter()
        .filter(|usage| {
            let Some(text) = texts.get(&usage.provision_id) else {
                return true;
            };
            let titles = covering_titles(text, usage.start_char, usage.end_char, index);
            titles.is_empty()
                || definitions.get(&usage.term_id).is_some_and(|definition| {
                    titles
                        .iter()
                        .any(|title| contains_phrase(definition, title))
                })
        })
        .collect()
}

/// Each term's definition entry, whitespace-collapsed and lower-cased: the
/// span of its defining provision, in this instrument or the one it adopts
/// the term from.
fn term_definitions(
    term_sets: &[&[lex_core::DefinedTerm]],
    provisions: &[lex_core::Provision],
    siblings: &[(&str, &Corpus)],
) -> HashMap<String, String> {
    let texts: HashMap<&str, &str> = provisions
        .iter()
        .chain(
            siblings
                .iter()
                .flat_map(|(_, corpus)| corpus.provisions.iter()),
        )
        .map(|provision| (provision.id.as_str(), provision.text.as_str()))
        .collect();
    term_sets
        .iter()
        .flat_map(|set| set.iter())
        .filter_map(|term| {
            let text = texts.get(term.defining_provision_id.as_str())?;
            let entry: String = text
                .chars()
                .skip(term.start_char)
                .take(term.end_char.saturating_sub(term.start_char))
                .collect();
            Some((term.id.clone(), normalized(&entry)))
        })
        .collect()
}

/// Remove the adapter's reviewed non-usages. Each must remove exactly one
/// extracted usage at its pinned provision, position, text and term;
/// otherwise the text it was reviewed against has changed and extraction
/// fails until it is re-reviewed.
fn exclude_reviewed_non_usages(
    instrument_id: &str,
    mut usages: Vec<lex_core::TermUsage>,
    exclusions: &[lex_source::UsageExclusion],
) -> Result<Vec<lex_core::TermUsage>> {
    for exclusion in exclusions {
        let provision_id = format!("{instrument_id}{}", exclusion.provision_suffix);
        let before = usages.len();
        usages.retain(|usage| {
            !(usage.provision_id == provision_id
                && usage.start_char == exclusion.start_char
                && usage.span == exclusion.text
                && usage.term_id == exclusion.term_id)
        });
        if usages.len() == before {
            bail!(
                "reviewed usage exclusion no longer matches a usage: {provision_id} at {} {:?} ({}); re-review it against the current text",
                exclusion.start_char,
                exclusion.text,
                exclusion.term_id
            );
        }
    }
    Ok(usages)
}

/// The instrument's own glossary terms and how they were found.
struct SelectedGlossary {
    terms: Vec<lex_core::DefinedTerm>,
    additive_to: Vec<String>,
    status: lex_core::GlossaryCoverageStatus,
    glossaries: Vec<lex_core::GlossaryProvisionCoverage>,
    hold: Option<lex_core::GlossaryHoldRecord>,
}

fn provision_coverage(
    provision: &lex_core::Provision,
    method: &str,
    terms: &[lex_core::DefinedTerm],
    repeal_placeholders: Option<Vec<String>>,
    unread_fractions: Option<Vec<String>>,
    invalidated_fractions: Option<Vec<String>>,
) -> lex_core::GlossaryProvisionCoverage {
    let own: Vec<_> = terms
        .iter()
        .filter(|term| term.defining_provision_id == provision.id)
        .collect();
    lex_core::GlossaryProvisionCoverage {
        provision_id: provision.id.clone(),
        provision_text_sha256: lex_core::evidence_sha256(&provision.text),
        method: method.to_owned(),
        fractions: own
            .iter()
            .filter_map(|term| term.fraction.clone())
            .collect(),
        repeal_placeholders,
        unread_fractions,
        invalidated_fractions,
        profile: None,
        term_count: own.len(),
    }
}

/// The glossary an adapter configures, with its accounting per provision.
fn select_configured_glossary(
    glossary: &lex_source::GlossaryConfig,
    provisions: &[lex_core::Provision],
) -> Result<SelectedGlossary> {
    let mut named = vec![(
        glossary.provision_suffix.as_str(),
        glossary.style.as_str(),
        glossary.profile.as_ref(),
    )];
    named.extend(glossary.additional_provisions.iter().map(|additional| {
        (
            additional.provision_suffix.as_str(),
            additional.style.as_str(),
            None,
        )
    }));
    let configured = named
        .iter()
        .map(|(suffix, style, profile)| {
            Ok((
                find_glossary_provision(provisions, suffix)?,
                GlossaryStyle::from_config(style)?,
                *profile,
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    let terms = extract_glossaries(&configured)?;
    let glossaries = configured
        .iter()
        .zip(&named)
        .map(|((provision, style, profile), (_, name, _))| {
            Ok(lex_core::GlossaryProvisionCoverage {
                profile: profile.cloned(),
                ..provision_coverage(
                    provision,
                    name,
                    &terms,
                    repeal_placeholders(provision, *style, *profile),
                    unread_fractions(provision, *style)?,
                    invalidated_fractions(provision, *style, *profile)?,
                )
            })
        })
        .collect::<Result<_>>()?;
    Ok(SelectedGlossary {
        terms,
        additive_to: glossary.additive_to.clone(),
        status: lex_core::GlossaryCoverageStatus::Configured,
        glossaries,
        hold: None,
    })
}

/// An adapter may configure its glossary explicitly (the audited committed
/// instruments do); otherwise a fraction glossary is auto-detected
/// (`detect_glossary`), so bulk instruments get their defined terms linked
/// without hand-config. A detected glossary that is incomplete is held with
/// no terms and reported here.
fn select_glossary(
    context: &InstrumentContext,
    provisions: &[lex_core::Provision],
) -> Result<SelectedGlossary> {
    use lex_core::{GlossaryCoverageStatus, GlossaryHoldRecord};
    if let Some(glossary) = &context.config.glossary {
        return select_configured_glossary(glossary, provisions);
    }
    let detection = detect_glossary(provisions)?;
    if let Some(hold) = &detection.held {
        eprintln!(
            "{}: glossary held with no terms at {}: {}",
            context.config.slug, hold.provision_id, hold.message
        );
    }
    let detected = detection
        .method
        .zip(detection.terms.first())
        .and_then(|(method, term)| {
            provisions
                .iter()
                .find(|provision| provision.id == term.defining_provision_id)
                .map(|provision| {
                    provision_coverage(
                        provision,
                        method,
                        &detection.terms,
                        detection.repeal_placeholders.clone(),
                        detection.unread_fractions.clone(),
                        detection.invalidated_fractions.clone(),
                    )
                })
        });
    let hold = detection
        .held
        .map(|hold| -> Result<GlossaryHoldRecord> {
            let provision = provisions
                .iter()
                .find(|provision| provision.id == hold.provision_id)
                .with_context(|| format!("held glossary {} not found", hold.provision_id))?;
            Ok(GlossaryHoldRecord {
                provision_id: hold.provision_id,
                provision_text_sha256: lex_core::evidence_sha256(&provision.text),
                code: hold.code,
                message: hold.message,
            })
        })
        .transpose()?;
    let status = match (&detected, &hold) {
        (Some(_), _) => GlossaryCoverageStatus::Detected,
        (None, Some(_)) => GlossaryCoverageStatus::Held,
        (None, None) => GlossaryCoverageStatus::NoneDetected,
    };
    Ok(SelectedGlossary {
        terms: detection.terms,
        additive_to: Vec::new(),
        status,
        glossaries: detected.into_iter().collect(),
        hold,
    })
}

/// The terms a glossary adopts through one `additive_to` entry: every term
/// of the sibling instrument with that exact id, or only the terms defined
/// in the sibling provision with that exact id (a regulation adopting "las
/// definiciones establecidas en el artículo 5 A de la Ley", not the law's
/// other glossaries). An entry that names neither, or a provision that
/// defines no terms, fails.
fn additive_terms(
    additive: &str,
    siblings: &[(&str, &Corpus)],
) -> Result<Vec<lex_core::DefinedTerm>> {
    if let Some((_, corpus)) = siblings
        .iter()
        .find(|(_, corpus)| corpus.instrument.id == additive)
    {
        return Ok(corpus.terms.clone());
    }
    let owners: Vec<&Corpus> = siblings
        .iter()
        .map(|(_, corpus)| *corpus)
        .filter(|corpus| {
            additive
                .strip_prefix(corpus.instrument.id.as_str())
                .is_some_and(|rest| rest.starts_with(':'))
        })
        .collect();
    let [owner] = owners.as_slice() else {
        bail!(
            "glossary is additive to {additive}, which is not an instrument or provision in the corpus"
        );
    };
    anyhow::ensure!(
        owner
            .provisions
            .iter()
            .any(|provision| provision.id == additive),
        "glossary is additive to {additive}, which is not a provision of {}",
        owner.instrument.id
    );
    let terms: Vec<_> = owner
        .terms
        .iter()
        .filter(|term| term.defining_provision_id == additive)
        .cloned()
        .collect();
    anyhow::ensure!(
        !terms.is_empty(),
        "glossary is additive to {additive}, which defines no terms"
    );
    Ok(terms)
}

/// Extract this instrument's reference edges, resolving targets against
/// every instrument committed under `corpus/mx/`.
/// Intake decisions are immutable under ordinary reparse/relink. A changed or
/// removed approval needs an explicit audited migration, not a missing config key.
fn preserve_intake_decisions(context: &InstrumentContext) -> Result<()> {
    let instrument_path = context.paths.corpus.join("instrument.json");
    if instrument_path.is_file() {
        let prior: Instrument = read_json(&instrument_path)?;
        if let Some(notice) = prior.source_repeal_notice {
            anyhow::ensure!(
                context.config.source_repeal_notice.as_ref() == Some(&notice),
                "refusing to replace or clear persisted instrument repeal decision"
            );
        }
    }
    let references_path = context.paths.corpus.join("references.json");
    if references_path.is_file() {
        let prior: Vec<lex_core::ReferenceEdge> = read_json(&references_path)?;
        for edge in prior {
            if let Some(approval) = edge.outdated_reference_approval {
                anyhow::ensure!(
                    context
                        .config
                        .outdated_reference_approvals
                        .contains(&approval),
                    "refusing to replace or clear persisted stale-reference decision"
                );
            }
            if let Some(review) = edge.legal_operator_review {
                anyhow::ensure!(
                    context.config.legal_operator_reviews.contains(&review),
                    "refusing to replace or clear persisted legal operator review"
                );
            }
        }
    }
    Ok(())
}

fn extract_instrument_references(
    root: &Path,
    context: &InstrumentContext,
    instrument: &Instrument,
    provisions: &[lex_core::Provision],
    siblings: &[(&str, &Corpus)],
) -> Result<Vec<lex_core::ReferenceEdge>> {
    preserve_intake_decisions(context)?;
    anyhow::ensure!(
        context.config.reference_policy.as_deref() != Some("internal")
            || (context.config.outdated_reference_approvals.is_empty()
                && context.config.legal_operator_reviews.is_empty()),
        "exact external reference approvals and reviews require cross-instrument extraction"
    );
    if context.config.reference_policy.as_deref() == Some("internal") {
        return extract_internal_references(provisions);
    }
    let mut known_targets: HashSet<String> =
        provisions.iter().map(|item| item.id.clone()).collect();
    for (_, corpus) in siblings {
        known_targets.extend(corpus.provisions.iter().map(|item| item.id.clone()));
    }
    if let Some(parent) = &context.config.parent_instrument_id {
        anyhow::ensure!(
            siblings
                .iter()
                .any(|(_, corpus)| &corpus.instrument.id == parent),
            "parent instrument {parent} is not a committed corpus instrument"
        );
    }
    let configured_external = context.config.external_instruments.iter().map(|external| {
        (
            external.name_marker.to_lowercase(),
            external.instrument_id.clone(),
        )
    });
    let external_instruments = merge_external_instruments(
        global_external_instruments(root, instrument, siblings)?,
        configured_external,
    )?;
    // `load_config` guarantees a historical entry names a successor and a
    // valid `historical_until`, so a malformed one cannot reach this point.
    let historical_targets = context
        .config
        .external_instruments
        .iter()
        .filter(|external| external.historical_target_unavailable)
        .filter_map(|external| {
            Some(lex_parse::HistoricalTarget {
                id: external.instrument_id.clone(),
                successor_id: external.successor_instrument_id.clone()?,
                until: NaiveDate::parse_from_str(external.historical_until.as_deref()?, "%Y-%m-%d")
                    .ok()?,
            })
        })
        .collect();
    let options = ReferenceOptions {
        policy: InstrumentContextPolicy::SentenceEarliestMarker {
            internal_markers: context
                .config
                .internal_reference_markers
                .clone()
                .unwrap_or_else(|| self_reference_markers(&context.config.instrument_type)),
            external_instruments,
        },
        transitory_citations: true,
        same_article_fractions: true,
        relative_references: true,
        historical_targets,
        parent_instrument: context.config.parent_instrument_id.clone(),
    };
    let mut references = extract_references(
        provisions,
        Some((instrument.id.as_str(), instrument.official_title.as_str())),
        &options,
        &known_targets,
    )?;
    lex_parse::apply_outdated_reference_approvals(
        &mut references,
        provisions,
        &context.config.outdated_reference_approvals,
        &instrument.source_sha256,
        &known_targets,
    )?;
    lex_parse::apply_legal_operator_reviews(
        &mut references,
        provisions,
        &context.config.legal_operator_reviews,
    )?;
    Ok(references)
}

/// Hand-curated official-title and colloquial citation aliases, keyed by an
/// instrument's stable `short_name`.
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct InstrumentAliasTable {
    schema_version: String,
    description: String,
    aliases: BTreeMap<String, Vec<String>>,
}

/// Build cross-instrument markers for aliases whose targets are present in
/// the committed corpus. Bare acronyms are excluded because they can collide
/// with ordinary legal prose.
fn global_external_instruments(
    root: &Path,
    instrument: &Instrument,
    siblings: &[(&str, &Corpus)],
) -> Result<Vec<(String, String)>> {
    let mut short_name_to_id = HashMap::new();
    let mut instruments = Vec::with_capacity(siblings.len() + 1);
    instruments.push(instrument);
    for (_, corpus) in siblings {
        instruments.push(&corpus.instrument);
    }
    for candidate in &instruments {
        if let Some(existing) =
            short_name_to_id.insert(candidate.short_name.clone(), candidate.id.clone())
            && existing != candidate.id
        {
            bail!(
                "instrument short_name {} maps to both {} and {}",
                candidate.short_name,
                existing,
                candidate.id
            );
        }
    }
    let alias_markers = configured_alias_markers(root, &instrument.id, &short_name_to_id)?;
    let title_markers = unique_official_title_markers(&instrument.id, &instruments);
    // Official titles and curated aliases must agree. `normalize` keeps the
    // longest marker first, which is what lets a full title win over a
    // shorter title that is its prefix when both start at the same character.
    normalize_external_instruments(title_markers.into_iter().chain(alias_markers))
}

/// Curated multi-word aliases whose `short_name` key is a committed instrument.
/// A missing alias table is an empty set, not an error: official titles still link.
fn configured_alias_markers(
    root: &Path,
    own_instrument_id: &str,
    short_name_to_id: &HashMap<String, String>,
) -> Result<Vec<(String, String)>> {
    let alias_path = root.join("adapters/diputados/_instrument-aliases.json");
    if !alias_path.is_file() {
        return Ok(Vec::new());
    }
    let table: InstrumentAliasTable = read_json(&alias_path)?;
    if table.schema_version != SCHEMA_VERSION {
        bail!(
            "unsupported instrument alias schema version {} in {}; expected {SCHEMA_VERSION}",
            table.schema_version,
            alias_path.display()
        );
    }
    if table.description.trim().is_empty() {
        bail!(
            "instrument alias description is empty in {}",
            alias_path.display()
        );
    }
    resolve_global_aliases(&table, own_instrument_id, short_name_to_id)
}

/// Exact official titles of other committed instruments. A title shared by more
/// than one instrument, and any title with no space, stays unlinked.
fn unique_official_title_markers(
    own_instrument_id: &str,
    instruments: &[&Instrument],
) -> Vec<(String, String)> {
    let mut grouped: BTreeMap<String, Vec<&str>> = BTreeMap::new();
    for instrument in instruments {
        let title = instrument.official_title.trim();
        if !title.contains(' ') {
            continue;
        }
        grouped
            .entry(title.to_lowercase())
            .or_default()
            .push(instrument.id.as_str());
    }
    let mut markers = Vec::new();
    for (title, mut ids) in grouped {
        ids.sort_unstable();
        ids.dedup();
        if let [target_id] = ids.as_slice()
            && *target_id != own_instrument_id
        {
            markers.push((title, (*target_id).to_owned()));
        }
    }
    markers
}

/// Official titles that more than one committed instrument uses. They are not
/// markers; closure reports them so the omission is visible.
fn ambiguous_official_titles(corpora: &[(String, Corpus)]) -> Vec<String> {
    let mut grouped: BTreeMap<String, BTreeMap<&str, &str>> = BTreeMap::new();
    for (slug, corpus) in corpora {
        let title = corpus.instrument.official_title.trim();
        if !title.contains(' ') {
            continue;
        }
        grouped
            .entry(title.to_lowercase())
            .or_default()
            .insert(corpus.instrument.id.as_str(), slug.as_str());
    }
    grouped
        .into_iter()
        .filter(|(_, owners)| owners.len() > 1)
        .map(|(title, owners)| {
            let slugs: Vec<_> = owners.into_values().collect();
            format!("{title} ({})", slugs.join(", "))
        })
        .collect()
}

fn resolve_global_aliases(
    table: &InstrumentAliasTable,
    own_instrument_id: &str,
    short_name_to_id: &HashMap<String, String>,
) -> Result<Vec<(String, String)>> {
    let mut markers = Vec::new();
    for (short_name, phrases) in &table.aliases {
        let Some(target_id) = short_name_to_id.get(short_name) else {
            continue;
        };
        if target_id == own_instrument_id {
            continue;
        }
        for phrase in phrases.iter().filter(|phrase| phrase.contains(' ')) {
            markers.push((phrase.clone(), target_id.clone()));
        }
    }
    normalize_external_instruments(markers)
}

fn normalize_external_instruments(
    instruments: impl IntoIterator<Item = (String, String)>,
) -> Result<Vec<(String, String)>> {
    let mut markers: HashMap<String, String> = HashMap::new();
    for (marker, target_id) in instruments {
        let marker = marker.to_lowercase();
        if let Some(existing) = markers.insert(marker.clone(), target_id.clone())
            && existing != target_id
        {
            bail!("instrument alias {marker:?} maps to both {existing} and {target_id}");
        }
    }
    let mut external: Vec<_> = markers.into_iter().collect();
    external.sort_by(|(left_marker, left_id), (right_marker, right_id)| {
        right_marker
            .len()
            .cmp(&left_marker.len())
            .then_with(|| left_marker.cmp(right_marker))
            .then_with(|| left_id.cmp(right_id))
    });
    Ok(external)
}

/// Adapter-scoped mappings override the global same-title alias. This is
/// required when a source predates a later law with the same official title:
/// the source can point to the historical identity without colliding with the
/// live corpus alias.
fn merge_external_instruments(
    mut global: Vec<(String, String)>,
    configured: impl IntoIterator<Item = (String, String)>,
) -> Result<Vec<(String, String)>> {
    let configured: Vec<_> = configured.into_iter().collect();
    let configured_markers: HashSet<_> = configured
        .iter()
        .map(|(marker, _)| marker.to_lowercase())
        .collect();
    global.retain(|(marker, _)| !configured_markers.contains(&marker.to_lowercase()));
    global.extend(configured);
    normalize_external_instruments(global)
}

fn corpus_views(corpora: &[(String, Corpus)]) -> Vec<(&str, &Corpus)> {
    corpora
        .iter()
        .map(|(slug, corpus)| (slug.as_str(), corpus))
        .collect()
}

fn sibling_views<'a>(
    corpora: &'a [(String, Corpus)],
    own_instrument_id: &str,
) -> Vec<(&'a str, &'a Corpus)> {
    corpora
        .iter()
        .filter(|(_, corpus)| corpus.instrument.id != own_instrument_id)
        .map(|(slug, corpus)| (slug.as_str(), corpus))
        .collect()
}

fn run_link(root: &Path, context: &InstrumentContext) -> Result<()> {
    run_link_with(root, context, false)
}

/// Single-instrument relink with the same protected-edge guard as the corpus
/// closure. A refusal happens before any canonical file is written; an
/// operator migration passes `migrate_protected` and every changed edge id is
/// printed.
fn run_link_with(root: &Path, context: &InstrumentContext, migrate_protected: bool) -> Result<()> {
    validate_persisted_repeal_evidence(root, &read_corpus(&context.paths)?)?;
    let corpora = read_committed_corpora(root)?;
    let mut corpus = read_corpus(&context.paths)?;
    let graph = relink_instrument(root, context, &corpus, &corpora)?;
    let drift = protected_edge_drift(&corpus, &graph);
    if !drift.is_empty() {
        if !migrate_protected {
            bail!(
                "relink refused; it would change protected edges (rerun with --migrate-protected only for a recorded operator migration):\n{}",
                drift.join("\n")
            );
        }
        for entry in &drift {
            println!("migrating protected edge: {entry}");
        }
    }
    corpus.references = graph.references;
    corpus.terms = graph.terms;
    corpus.term_usages = graph.usages;
    write_canonical(&corpus, &context.paths.corpus)?;
    if let Some(coverage) = &graph.coverage {
        write_glossary_coverage(coverage, &context.paths.corpus)?;
    }
    println!("extracted {} canonical references", corpus.references.len());
    println!(
        "extracted {} defined terms with {} usages",
        corpus.terms.len(),
        corpus.term_usages.len()
    );
    Ok(())
}

struct LinkedGraph {
    references: Vec<lex_core::ReferenceEdge>,
    terms: Vec<lex_core::DefinedTerm>,
    usages: Vec<lex_core::TermUsage>,
    /// Set when this graph extracted the terms itself.
    coverage: Option<lex_core::GlossaryCoverage>,
}

fn relink_instrument(
    root: &Path,
    context: &InstrumentContext,
    corpus: &Corpus,
    corpora: &[(String, Corpus)],
) -> Result<LinkedGraph> {
    let siblings = sibling_views(corpora, &corpus.instrument.id);
    let references = extract_instrument_references(
        root,
        context,
        &corpus.instrument,
        &corpus.provisions,
        &siblings,
    )?;
    let extraction =
        extract_instrument_terms(context, &corpus.instrument, &corpus.provisions, &siblings)?;
    Ok(LinkedGraph {
        references,
        terms: extraction.terms,
        usages: extraction.usages,
        coverage: Some(extraction.coverage),
    })
}

#[derive(Debug, serde::Serialize)]
struct CorpusClosureReport {
    status: String,
    instruments: usize,
    relinked: usize,
    relinked_instruments: Vec<String>,
    /// Accepted-gap instruments whose terms, usages, coverage and Markdown
    /// were refreshed while their references stayed as persisted.
    glossary_refreshed_instruments: Vec<String>,
    unchanged: usize,
    references_before: usize,
    references_after: usize,
    resolved_cross_instrument: usize,
    ambiguous_titles: Vec<String>,
    rejected: Vec<CorpusClosureRejection>,
}

#[derive(Debug, serde::Serialize)]
struct CorpusClosureRejection {
    slug: String,
    errors: Vec<String>,
}

enum ClosureDecision {
    Unchanged,
    Accepted(lex_core::ValidationReport),
    Rejected(Vec<String>),
}

const CLOSURE_ACCEPTED_GAPS: &str = "adapters/diputados/_closure-accepted-gaps.json";

/// The recorded accepted gaps; no list on disk means none is accepted.
fn read_accepted_closure_gaps(root: &Path) -> Result<AcceptedClosureGaps> {
    let path = root.join(CLOSURE_ACCEPTED_GAPS);
    if !path.exists() {
        return Ok(AcceptedClosureGaps { accepted: vec![] });
    }
    serde_json::from_str(&fs::read_to_string(&path)?)
        .with_context(|| format!("invalid {}", path.display()))
}

#[derive(serde::Deserialize)]
struct AcceptedClosureGaps {
    accepted: Vec<AcceptedClosureGap>,
}

#[derive(serde::Deserialize)]
struct AcceptedClosureGap {
    slug: String,
    errors: Vec<String>,
}

/// True only when the closure's rejections equal the recorded accepted-gap
/// list exactly (same instruments, same errors). Anything new, changed or
/// resolved is reported and the closure keeps failing, so a clean exit means
/// "nothing beyond the accepted gaps". No list on disk means no gap is accepted,
/// and a clean closure with a non-empty list is also a mismatch (stale entries).
fn closure_matches_accepted_gaps(root: &Path, report: &CorpusClosureReport) -> Result<bool> {
    let accepted = read_accepted_closure_gaps(root)?;
    // Errors are compared as recorded, with multiplicity: a new citation to
    // an already-accepted missing target must not pass.
    let normalize = |errors: &[String]| {
        let mut errors = errors.to_vec();
        errors.sort();
        errors
    };
    let mut recorded: Vec<(&str, Vec<String>)> = accepted
        .accepted
        .iter()
        .map(|gap| (gap.slug.as_str(), normalize(&gap.errors)))
        .collect();
    let mut actual: Vec<(&str, Vec<String>)> = report
        .rejected
        .iter()
        .map(|rejection| (rejection.slug.as_str(), normalize(&rejection.errors)))
        .collect();
    recorded.sort();
    actual.sort();
    let mut matches = true;
    for (slug, errors) in &actual {
        if !recorded.contains(&(*slug, errors.clone())) {
            println!("closure: new or changed rejection, not an accepted gap: {slug}");
            matches = false;
        }
    }
    for (slug, errors) in &recorded {
        if !actual.contains(&(*slug, errors.clone())) {
            println!(
                "closure: accepted gap no longer rejected as recorded (resolved or changed): {slug}"
            );
            matches = false;
        }
    }
    Ok(matches)
}

/// Relink every committed instrument once, against one in-memory corpus.
/// An instrument whose new graph does not validate is left on disk as it was.
fn run_corpus_closure(root: &Path) -> Result<CorpusClosureReport> {
    let mut corpora = read_committed_corpora(root)?;
    // Same persisted-evidence gate as `link`, before any graph is proposed.
    for (_, corpus) in &corpora {
        validate_persisted_repeal_evidence(root, corpus)?;
    }
    let contexts = load_instrument_contexts(root, &corpora)?;
    let references_before = corpora
        .iter()
        .map(|(_, corpus)| corpus.references.len())
        .sum();
    let original_terms: Vec<_> = corpora
        .iter()
        .map(|(_, corpus)| corpus.terms.clone())
        .collect();
    let original_usages: Vec<_> = corpora
        .iter()
        .map(|(_, corpus)| corpus.term_usages.clone())
        .collect();
    let ambiguous_titles = ambiguous_official_titles(&corpora);

    // Terms come only from each instrument's own provisions. The second pass
    // recomputes usages so an additive glossary sees those refreshed terms.
    refresh_linked_terms(&contexts, &mut corpora)?;
    let coverage = refresh_linked_terms(&contexts, &mut corpora)?;
    // A rejected instrument keeps its persisted terms, so its refreshed
    // coverage describes them only when the refresh reproduced them.
    let refreshed_as_persisted = terms_reproduced(&corpora, &original_terms, &original_usages)?;
    let proposals = propose_references(root, &contexts, &corpora)?;
    let provision_ids = all_provision_ids(&corpora);
    let decisions = classify_closure(
        &contexts,
        &corpora,
        &proposals,
        &original_terms,
        &original_usages,
        &provision_ids,
    )?;
    let glossary_refreshed = glossary_refreshable(
        root,
        &contexts,
        &corpora,
        &decisions,
        &refreshed_as_persisted,
    )?;
    apply_closure_decisions(
        &mut corpora,
        &proposals,
        &decisions,
        &glossary_refreshed,
        &original_terms,
        &original_usages,
    );
    preflight_closure_coverage(
        &corpora,
        &contexts,
        &coverage,
        &decisions,
        &refreshed_as_persisted,
        &glossary_refreshed,
    )?;
    let ClosureOutputs {
        relinked,
        glossary_refreshed,
        unchanged,
        rejected,
    } = write_closure_outputs(
        &corpora,
        &contexts,
        &coverage,
        &decisions,
        &refreshed_as_persisted,
        &glossary_refreshed,
    )?;
    let report = CorpusClosureReport {
        status: if rejected.is_empty() { "ok" } else { "failed" }.to_owned(),
        instruments: corpora.len(),
        relinked: relinked.len(),
        relinked_instruments: relinked,
        glossary_refreshed_instruments: glossary_refreshed,
        unchanged,
        references_before,
        references_after: corpora
            .iter()
            .map(|(_, corpus)| corpus.references.len())
            .sum(),
        resolved_cross_instrument: resolved_cross_instrument_edges(&corpora),
        ambiguous_titles,
        rejected,
    };
    write_pretty_json(&report, &root.join(".work/corpus-closure.json"))?;
    println!(
        "corpus closure: relinked {}, unchanged {}, rejected {}, references {} -> {}, resolved cross-instrument {}",
        report.relinked,
        report.unchanged,
        report.rejected.len(),
        report.references_before,
        report.references_after,
        report.resolved_cross_instrument
    );
    Ok(report)
}

/// Per instrument, whether the refreshed terms and usages equal the persisted ones.
fn terms_reproduced(
    corpora: &[(String, Corpus)],
    original_terms: &[Vec<lex_core::DefinedTerm>],
    original_usages: &[Vec<lex_core::TermUsage>],
) -> Result<Vec<bool>> {
    corpora
        .iter()
        .zip(original_terms.iter().zip(original_usages))
        .map(|((_, corpus), (terms, usages))| {
            Ok(
                serde_json::to_value(&corpus.terms)? == serde_json::to_value(terms)?
                    && serde_json::to_value(&corpus.term_usages)? == serde_json::to_value(usages)?,
            )
        })
        .collect()
}

fn load_instrument_contexts(
    root: &Path,
    corpora: &[(String, Corpus)],
) -> Result<HashMap<String, InstrumentContext>> {
    let mut configs = HashMap::new();
    for path in adapter_paths(root)? {
        let config = load_config(&path)?;
        if let Some(previous) = configs.insert(config.slug.clone(), config) {
            bail!("duplicate adapter slug {}", previous.slug);
        }
    }
    let mut contexts = HashMap::new();
    for (slug, _) in corpora {
        let config = configs.remove(slug).with_context(|| {
            format!("no adapter configuration found for instrument {slug:?} under adapters/")
        })?;
        let paths = Paths::new(root, slug, config.source_format);
        contexts.insert(
            slug.clone(),
            InstrumentContext {
                config,
                paths,
                root: root.to_path_buf(),
            },
        );
    }
    Ok(contexts)
}

/// Validate all final-state coverage before the closure writes any output.
fn preflight_closure_coverage(
    corpora: &[(String, Corpus)],
    contexts: &HashMap<String, InstrumentContext>,
    coverage: &[lex_core::GlossaryCoverage],
    decisions: &[ClosureDecision],
    refreshed_as_persisted: &[bool],
    glossary_refreshed: &[bool],
) -> Result<()> {
    // Bind coverage to the final corpus, including originals restored after a
    // rejection. Preflight every record before writing any closure output.
    for (index, (slug, corpus)) in corpora.iter().enumerate() {
        if matches!(decisions[index], ClosureDecision::Rejected(_))
            && !refreshed_as_persisted[index]
            && !glossary_refreshed[index]
        {
            bail!(
                "{slug} is rejected, and the current glossary extraction no longer \
                 reproduces its persisted terms, so its coverage would go stale; \
                 resolve the rejection or record it as an accepted gap"
            );
        }
        if !matches!(decisions[index], ClosureDecision::Rejected(_))
            || refreshed_as_persisted[index]
            || glossary_refreshed[index]
        {
            validate_glossary_coverage_inputs(
                &coverage[index],
                &contexts[slug],
                corpus,
                &sibling_views(corpora, &corpus.instrument.id),
            )?;
        }
    }
    Ok(())
}

/// Write each instrument's closure outcome and count it: relinked,
/// unchanged, or rejected (with or without a glossary-only refresh).
fn write_closure_outputs(
    corpora: &[(String, Corpus)],
    contexts: &HashMap<String, InstrumentContext>,
    coverage: &[lex_core::GlossaryCoverage],
    decisions: &[ClosureDecision],
    refreshed_as_persisted: &[bool],
    glossary_refreshed: &[bool],
) -> Result<ClosureOutputs> {
    let (targets, term_targets) = closure_link_targets(corpora);
    let mut relinked = Vec::new();
    let mut glossary_refreshed_instruments = Vec::new();
    let mut unchanged = 0;
    let mut rejected = Vec::new();
    for (index, (slug, corpus)) in corpora.iter().enumerate() {
        let context = contexts
            .get(slug)
            .with_context(|| format!("no adapter configuration found for instrument {slug:?}"))?;
        if !matches!(decisions[index], ClosureDecision::Rejected(_))
            || refreshed_as_persisted[index]
            || glossary_refreshed[index]
        {
            write_glossary_coverage(&coverage[index], &context.paths.corpus)?;
        }
        match &decisions[index] {
            ClosureDecision::Unchanged => unchanged += 1,
            ClosureDecision::Rejected(errors) => {
                if glossary_refreshed[index] {
                    write_refreshed_glossary(context, corpus, &targets, &term_targets)?;
                    glossary_refreshed_instruments.push(slug.clone());
                    println!(
                        "{slug}: rejected as its accepted gap; glossary refreshed, references left unchanged"
                    );
                } else {
                    println!("{slug}: rejected, left unchanged");
                }
                rejected.push(CorpusClosureRejection {
                    slug: slug.clone(),
                    errors: errors.clone(),
                });
            }
            ClosureDecision::Accepted(report) => {
                write_relinked_instrument(context, corpus, report, &targets, &term_targets)?;
                println!("{slug}: relinked {} references", corpus.references.len());
                relinked.push(slug.clone());
            }
        }
    }
    Ok(ClosureOutputs {
        relinked,
        glossary_refreshed: glossary_refreshed_instruments,
        unchanged,
        rejected,
    })
}

/// What [`write_closure_outputs`] wrote: relinked instruments, accepted-gap
/// instruments whose glossary alone was refreshed, and the rest.
struct ClosureOutputs {
    relinked: Vec<String>,
    glossary_refreshed: Vec<String>,
    unchanged: usize,
    rejected: Vec<CorpusClosureRejection>,
}

/// Which rejected instruments take a glossary-only refresh: an instrument
/// whose rejection is exactly its recorded accepted gap, whose refreshed
/// terms or usages differ from the persisted ones, and whose terms no other
/// instrument imports (`additive_to`). Its references stay as persisted
/// (operator decision 2026-10-19), so a grammar fix still reaches its
/// glossary while the gap stays recorded.
fn glossary_refreshable(
    root: &Path,
    contexts: &HashMap<String, InstrumentContext>,
    corpora: &[(String, Corpus)],
    decisions: &[ClosureDecision],
    refreshed_as_persisted: &[bool],
) -> Result<Vec<bool>> {
    let gaps = read_accepted_closure_gaps(root)?;
    let sorted = |errors: &[String]| {
        let mut errors = errors.to_vec();
        errors.sort();
        errors
    };
    let imported = |instrument_id: &str| {
        contexts.values().any(|context| {
            context.config.glossary.as_ref().is_some_and(|glossary| {
                glossary.additive_to.iter().any(|additive| {
                    additive == instrument_id
                        || additive
                            .strip_prefix(instrument_id)
                            .is_some_and(|rest| rest.starts_with(':'))
                })
            })
        })
    };
    Ok(corpora
        .iter()
        .enumerate()
        .map(|(index, (slug, corpus))| {
            let ClosureDecision::Rejected(errors) = &decisions[index] else {
                return false;
            };
            !refreshed_as_persisted[index]
                && gaps
                    .accepted
                    .iter()
                    .any(|gap| gap.slug == *slug && sorted(&gap.errors) == sorted(errors))
                && !imported(&corpus.instrument.id)
        })
        .collect())
}

/// Write a glossary-only refresh: terms, usages and Markdown, never the
/// persisted references or validation record.
fn write_refreshed_glossary(
    context: &InstrumentContext,
    corpus: &Corpus,
    targets: &LinkTargets,
    term_targets: &TermTargets,
) -> Result<()> {
    let directory = &context.paths.corpus;
    write_pretty_json(&corpus.terms, &directory.join("terms.json"))?;
    write_pretty_json(&corpus.term_usages, &directory.join("term-usages.json"))?;
    write_markdown(corpus, targets, term_targets, &context.paths.markdown)?;
    Ok(())
}

fn refresh_linked_terms(
    contexts: &HashMap<String, InstrumentContext>,
    corpora: &mut [(String, Corpus)],
) -> Result<Vec<lex_core::GlossaryCoverage>> {
    let mut extracted = Vec::with_capacity(corpora.len());
    for (index, (slug, corpus)) in corpora.iter().enumerate() {
        if index.is_multiple_of(50) {
            println!("refreshing terms {index}/{}", corpora.len());
        }
        let context = contexts
            .get(slug)
            .with_context(|| format!("no adapter configuration found for instrument {slug:?}"))?;
        let siblings = sibling_views(corpora, &corpus.instrument.id);
        extracted.push(extract_instrument_terms(
            context,
            &corpus.instrument,
            &corpus.provisions,
            &siblings,
        )?);
    }
    let mut coverage = Vec::with_capacity(extracted.len());
    for ((_, corpus), extraction) in corpora.iter_mut().zip(extracted) {
        corpus.terms = extraction.terms;
        corpus.term_usages = extraction.usages;
        coverage.push(extraction.coverage);
    }
    Ok(coverage)
}

fn propose_references(
    root: &Path,
    contexts: &HashMap<String, InstrumentContext>,
    corpora: &[(String, Corpus)],
) -> Result<Vec<LinkedGraph>> {
    let mut proposals = Vec::with_capacity(corpora.len());
    for (index, (slug, corpus)) in corpora.iter().enumerate() {
        println!("linking {slug} ({}/{})", index + 1, corpora.len());
        let context = contexts
            .get(slug)
            .with_context(|| format!("no adapter configuration found for instrument {slug:?}"))?;
        let siblings = sibling_views(corpora, &corpus.instrument.id);
        let references = extract_instrument_references(
            root,
            context,
            &corpus.instrument,
            &corpus.provisions,
            &siblings,
        )?;
        proposals.push(LinkedGraph {
            references,
            terms: corpus.terms.clone(),
            usages: corpus.term_usages.clone(),
            coverage: None,
        });
    }
    Ok(proposals)
}

fn all_provision_ids(corpora: &[(String, Corpus)]) -> HashSet<String> {
    corpora
        .iter()
        .flat_map(|(_, corpus)| corpus.provisions.iter().map(|item| item.id.clone()))
        .collect()
}

fn closure_term_ids(
    corpora: &[(String, Corpus)],
    original_terms: &[Vec<lex_core::DefinedTerm>],
    rejected: &[Option<Vec<String>>],
) -> HashSet<String> {
    let mut ids = HashSet::new();
    for (index, (_, corpus)) in corpora.iter().enumerate() {
        let terms = if rejected[index].is_some() {
            &original_terms[index]
        } else {
            &corpus.terms
        };
        ids.extend(terms.iter().map(|term| term.id.clone()));
    }
    ids
}

fn classify_closure(
    contexts: &HashMap<String, InstrumentContext>,
    corpora: &[(String, Corpus)],
    proposals: &[LinkedGraph],
    original_terms: &[Vec<lex_core::DefinedTerm>],
    original_usages: &[Vec<lex_core::TermUsage>],
    provision_ids: &HashSet<String>,
) -> Result<Vec<ClosureDecision>> {
    let mut rejected: Vec<Option<Vec<String>>> = vec![None; corpora.len()];
    loop {
        let term_ids = closure_term_ids(corpora, original_terms, &rejected);
        let mut changed = false;
        for (index, ((slug, corpus), proposal)) in corpora.iter().zip(proposals).enumerate() {
            if rejected[index].is_some()
                || graph_unchanged(
                    corpus,
                    proposal,
                    &original_terms[index],
                    &original_usages[index],
                )?
            {
                continue;
            }
            let drift = protected_edge_drift(corpus, proposal);
            if !drift.is_empty() {
                rejected[index] = Some(drift);
                changed = true;
                continue;
            }
            let context = contexts.get(slug).with_context(|| {
                format!("no adapter configuration found for instrument {slug:?}")
            })?;
            let report = validate_loaded(context, corpus, proposal, provision_ids, &term_ids);
            if !report.valid {
                rejected[index] = Some(validation_errors(&report));
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    let term_ids = closure_term_ids(corpora, original_terms, &rejected);
    let mut decisions = Vec::with_capacity(corpora.len());
    for (index, ((slug, corpus), proposal)) in corpora.iter().zip(proposals).enumerate() {
        if let Some(errors) = rejected[index].clone() {
            decisions.push(ClosureDecision::Rejected(errors));
            continue;
        }
        if graph_unchanged(
            corpus,
            proposal,
            &original_terms[index],
            &original_usages[index],
        )? {
            decisions.push(ClosureDecision::Unchanged);
            continue;
        }
        let context = contexts
            .get(slug)
            .with_context(|| format!("no adapter configuration found for instrument {slug:?}"))?;
        decisions.push(ClosureDecision::Accepted(validate_loaded(
            context,
            corpus,
            proposal,
            provision_ids,
            &term_ids,
        )));
    }
    Ok(decisions)
}

/// Persisted historical-identity and reviewer-approved edges record adapter or
/// reviewer decisions. A closure relink may never change or drop one, even when
/// the new graph validates; it keeps the previous graph instead.
fn protected_edge_drift(corpus: &Corpus, proposal: &LinkedGraph) -> Vec<String> {
    let proposed: HashMap<&str, &lex_core::ReferenceEdge> = proposal
        .references
        .iter()
        .map(|edge| (edge.id.as_str(), edge))
        .collect();
    corpus
        .references
        .iter()
        .filter(|edge| {
            matches!(
                edge.resolution_status,
                ReferenceResolutionStatus::HistoricalTargetUnavailable
                    | ReferenceResolutionStatus::HistoricalSupersededReferenceOutdated
            ) || edge.outdated_reference_approval.is_some()
        })
        .filter(|edge| {
            proposed.get(edge.id.as_str()).is_none_or(|new| {
                // Recording the resolution rule on an edge written before the
                // field existed is provenance, not a change to the decision.
                let mut new = (*new).clone();
                if edge.resolution_rule.is_none() {
                    new.resolution_rule = None;
                }
                serde_json::to_value(&new).ok() != serde_json::to_value(edge).ok()
            })
        })
        .map(|edge| {
            format!(
                "persisted historical or approved reference changed or dropped: {}",
                edge.id
            )
        })
        .collect()
}

fn graph_unchanged(
    corpus: &Corpus,
    proposal: &LinkedGraph,
    original_terms: &[lex_core::DefinedTerm],
    original_usages: &[lex_core::TermUsage],
) -> Result<bool> {
    Ok(
        serde_json::to_value(&corpus.references)? == serde_json::to_value(&proposal.references)?
            && serde_json::to_value(original_terms)? == serde_json::to_value(&proposal.terms)?
            && serde_json::to_value(original_usages)? == serde_json::to_value(&proposal.usages)?,
    )
}

fn validate_loaded(
    context: &InstrumentContext,
    corpus: &Corpus,
    proposal: &LinkedGraph,
    provision_ids: &HashSet<String>,
    term_ids: &HashSet<String>,
) -> lex_core::ValidationReport {
    let config = &context.config;
    let mut report = validate_corpus(
        &CorpusView {
            instrument_id: &corpus.instrument.id,
            official_title: Some(corpus.instrument.official_title.as_str()),
            provisions: &corpus.provisions,
            references: &proposal.references,
            terms: &proposal.terms,
            term_usages: &proposal.usages,
            amendment_references: &corpus.amendment_references,
        },
        &CorpusExpectations {
            min_articles: config.expected_min_articles,
            articles: config.expected_articles,
            transitories: config.expected_transitories,
            annexes: config.expected_annexes,
            require_chapter_context: config.parser == "ifpe-dcg",
            allow_article_gaps: config.allow_article_gaps,
        },
        provision_ids,
        term_ids,
    );
    append_parent_rule_issues(&mut report, config, &proposal.references);
    report
}

/// A `parent_shorthand` edge must target the adapter's recorded parent law;
/// the linker never infers a parent, so any other target is a defect.
fn append_parent_rule_issues(
    report: &mut lex_core::ValidationReport,
    config: &SourceConfig,
    references: &[lex_core::ReferenceEdge],
) {
    for edge in references {
        if edge.resolution_rule == Some(lex_core::ResolutionRule::ParentShorthand)
            && config.parent_instrument_id.as_deref() != Some(edge.target_instrument_id.as_str())
        {
            report.issues.push(lex_core::ValidationIssue {
                severity: lex_core::Severity::Error,
                code: "resolution_rule_inconsistent".to_owned(),
                message: format!(
                    "parent-shorthand edge does not target the adapter parent: {}",
                    edge.id
                ),
                provision_id: Some(edge.source_provision_id.clone()),
            });
            report.valid = false;
        }
    }
}

/// Check extraction provenance against current adapter selection and originals,
/// not merely counts in the importing instrument. Replay is read-only and also
/// catches stale term usages even when their stored counts still match.
fn validate_glossary_coverage_inputs(
    coverage: &lex_core::GlossaryCoverage,
    context: &InstrumentContext,
    corpus: &Corpus,
    siblings: &[(&str, &Corpus)],
) -> Result<()> {
    coverage
        .validate(
            &corpus.instrument,
            &corpus.provisions,
            &corpus.terms,
            &corpus.term_usages,
        )
        .map_err(anyhow::Error::msg)?;
    let expected =
        extract_instrument_terms(context, &corpus.instrument, &corpus.provisions, siblings)?;
    anyhow::ensure!(
        serde_json::to_value(coverage)? == serde_json::to_value(&expected.coverage)?,
        "glossary coverage differs from current adapter selection or imported terms; relink required"
    );
    anyhow::ensure!(
        serde_json::to_value(&corpus.terms)? == serde_json::to_value(&expected.terms)?
            && serde_json::to_value(&corpus.term_usages)?
                == serde_json::to_value(&expected.usages)?,
        "glossary terms or usages differ from current extraction; relink required"
    );
    Ok(())
}

/// `glossary-coverage.json` must describe the persisted terms and the current
/// glossary provision text; a stale record is an error. A missing record is a
/// warning, for instruments not yet relinked since the record was introduced.
fn append_glossary_coverage_issues(
    report: &mut lex_core::ValidationReport,
    corpus: &Corpus,
    directory: &Path,
    context: &InstrumentContext,
    siblings: &[(&str, &Corpus)],
) -> Result<()> {
    let path = directory.join(GLOSSARY_COVERAGE_FILE);
    let (severity, code, message) = if path.is_file() {
        let coverage: lex_core::GlossaryCoverage = read_json(&path)?;
        match validate_glossary_coverage_inputs(&coverage, context, corpus, siblings) {
            Ok(()) => return Ok(()),
            Err(error) => (
                lex_core::Severity::Error,
                "glossary_coverage_inconsistent",
                error.to_string(),
            ),
        }
    } else {
        (
            lex_core::Severity::Warning,
            "glossary_coverage_missing",
            format!("{GLOSSARY_COVERAGE_FILE} is missing; relink to record it"),
        )
    };
    if severity == lex_core::Severity::Error {
        report.valid = false;
    }
    report.issues.push(lex_core::ValidationIssue {
        severity,
        code: code.to_owned(),
        message,
        provision_id: None,
    });
    Ok(())
}

fn validation_errors(report: &lex_core::ValidationReport) -> Vec<String> {
    report
        .issues
        .iter()
        .filter(|issue| issue.severity == lex_core::Severity::Error)
        .take(12)
        .map(|issue| format!("{}: {}", issue.code, issue.message))
        .collect()
}

fn apply_closure_decisions(
    corpora: &mut [(String, Corpus)],
    proposals: &[LinkedGraph],
    decisions: &[ClosureDecision],
    glossary_refreshed: &[bool],
    original_terms: &[Vec<lex_core::DefinedTerm>],
    original_usages: &[Vec<lex_core::TermUsage>],
) {
    for (index, (_, corpus)) in corpora.iter_mut().enumerate() {
        match &decisions[index] {
            // An accepted-gap instrument keeps its refreshed terms and usages
            // with its persisted references.
            ClosureDecision::Rejected(_) if glossary_refreshed[index] => {}
            ClosureDecision::Rejected(_) => {
                corpus.terms.clone_from(&original_terms[index]);
                corpus.term_usages.clone_from(&original_usages[index]);
            }
            ClosureDecision::Accepted(_) => {
                corpus.references.clone_from(&proposals[index].references);
                corpus.terms.clone_from(&proposals[index].terms);
                corpus.term_usages.clone_from(&proposals[index].usages);
            }
            ClosureDecision::Unchanged => {}
        }
    }
}

fn closure_link_targets(corpora: &[(String, Corpus)]) -> (LinkTargets, TermTargets) {
    let inputs: Vec<_> = corpora
        .iter()
        .map(|(slug, corpus)| (corpus, slug.as_str()))
        .collect();
    let targets = link_targets(&inputs);
    let terms = term_targets(&inputs, &targets);
    (targets, terms)
}

fn resolved_cross_instrument_edges(corpora: &[(String, Corpus)]) -> usize {
    corpora
        .iter()
        .map(|(_, corpus)| {
            corpus
                .references
                .iter()
                .filter(|edge| {
                    edge.resolution_status == ReferenceResolutionStatus::Resolved
                        && edge.target_instrument_id != corpus.instrument.id
                })
                .count()
        })
        .sum()
}

fn write_relinked_instrument(
    context: &InstrumentContext,
    corpus: &Corpus,
    report: &lex_core::ValidationReport,
    targets: &LinkTargets,
    term_targets: &TermTargets,
) -> Result<()> {
    let directory = &context.paths.corpus;
    write_pretty_json(&corpus.references, &directory.join("references.json"))?;
    write_pretty_json(&corpus.terms, &directory.join("terms.json"))?;
    write_pretty_json(&corpus.term_usages, &directory.join("term-usages.json"))?;
    write_validation(report, directory)?;
    write_markdown(corpus, targets, term_targets, &context.paths.markdown)?;
    Ok(())
}

fn run_temporal_request(context: &InstrumentContext) -> Result<()> {
    let paths = &context.paths;
    let corpus = read_corpus(paths)?;
    let reform_evidence: Vec<TemporalEvidence> = if paths.reform_evidence.exists() {
        read_json(&paths.reform_evidence)?
    } else {
        Vec::new()
    };
    let evidence = build_temporal_evidence(&context.config, &corpus.provisions, &reform_evidence);
    let request = TemporalAnalysisRequest {
        schema_version: SCHEMA_VERSION.to_owned(),
        prompt_version: "temporal-v2".to_owned(),
        instrument_id: corpus.instrument.id,
        publication_date: corpus.instrument.publication_date.context(
            "temporal requests require an exact publication day; serialized range is not a day",
        )?,
        latest_reform_date: corpus.instrument.latest_reform_date,
        relevant_provisions: evidence,
        required_output_schema: "schemas/temporal-model-output-v2.schema.json".to_owned(),
    };
    write_pretty_json(&request, &paths.temporal_request)?;
    println!(
        "wrote temporal analysis request with {} evidence items",
        request.relevant_provisions.len()
    );
    Ok(())
}

/// Build the relevant-provisions evidence list: every ordinary transitory,
/// plus the reform-decree transitories the adapter configures as relevant
/// to this instrument (`relevant_reform_transitories`). Reused by the
/// temporal-analysis request and by reparse's temporal-state reapplication,
/// since an amendment-event determination's provision ID never appears
/// among canonical provisions — only among reform evidence.
fn build_temporal_evidence(
    config: &SourceConfig,
    provisions: &[lex_core::Provision],
    reform_evidence: &[TemporalEvidence],
) -> Vec<TemporalEvidence> {
    let mut evidence: Vec<TemporalEvidence> = provisions
        .iter()
        .filter(|item| item.provision_type == ProvisionType::Transitory)
        .map(|item| TemporalEvidence {
            provision_id: item.id.clone(),
            label: item.label.clone(),
            text: item.text.clone(),
            // Canonical transitory marks already live on the provision;
            // this temporal-input projection stays as it was.
            amendment_marks: Vec::new(),
        })
        .collect();
    let mut relevant_reform_evidence: Vec<TemporalEvidence> = reform_evidence
        .iter()
        .filter(|evidence| {
            config
                .relevant_reform_transitories
                .iter()
                .any(|(date, ordinals)| {
                    ordinals.iter().any(|ordinal| {
                        evidence.provision_id
                            == format!(
                                "{}:amendment:{date}:transitory:{ordinal}",
                                config.instrument_id
                            )
                    })
                })
        })
        .cloned()
        .collect();
    evidence.append(&mut relevant_reform_evidence);
    evidence
}

fn run_codex_temporal(root: &Path, context: &InstrumentContext, model: &str) -> Result<()> {
    let paths = &context.paths;
    let request: TemporalAnalysisRequest = read_json(&paths.temporal_request)?;
    let prompt = fs::read_to_string(root.join("prompts/temporal-v2.md"))?;
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
        .arg(root.join("schemas/temporal-model-output-v2.schema.json"))
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
    run_temporal_import(context, &paths.temporal_model_output, model, None)
}

fn current_temporal_evidence(
    context: &InstrumentContext,
    corpus: &Corpus,
) -> Result<Vec<TemporalEvidence>> {
    let reform = if context.paths.reform_evidence.exists() {
        read_json(&context.paths.reform_evidence)?
    } else {
        Vec::new()
    };
    Ok(build_temporal_evidence(
        &context.config,
        &corpus.provisions,
        &reform,
    ))
}

fn ensure_current_temporal_evidence(
    requested: &[TemporalEvidence],
    current: &[TemporalEvidence],
) -> Result<()> {
    let fingerprint = |items: &[TemporalEvidence]| {
        items
            .iter()
            .map(|item| {
                (
                    item.provision_id.clone(),
                    lex_core::evidence_sha256(&item.text),
                )
            })
            .collect::<BTreeMap<_, _>>()
    };
    let old = fingerprint(requested);
    let new = fingerprint(current);
    if old.len() != requested.len() || new.len() != current.len() || old != new {
        bail!("temporal request evidence is stale; regenerate the request before importing");
    }
    Ok(())
}

fn run_temporal_import(
    context: &InstrumentContext,
    response_path: &Path,
    model: &str,
    response_id: Option<String>,
) -> Result<()> {
    let paths = &context.paths;
    let request_bytes = fs::read(&paths.temporal_request)?;
    let request: TemporalAnalysisRequest = serde_json::from_slice(&request_bytes)?;
    let response_bytes = fs::read(response_path)?;
    let batch = parse_temporal_model_response(&response_bytes)?;
    let corpus = read_corpus(paths)?;
    let instrument = &corpus.instrument;
    if request.instrument_id != instrument.id
        || Some(request.publication_date) != instrument.publication_date
    {
        bail!("temporal request instrument metadata is stale");
    }
    ensure_current_temporal_evidence(
        &request.relevant_provisions,
        &current_temporal_evidence(context, &corpus)?,
    )?;
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
        let superseded = preserve_temporal_review_history(
            &mut routed.result,
            &mut routed.review_items,
            &previous_result,
            &previous_items,
        );
        if !superseded.is_empty() {
            eprintln!(
                "warning: {} previous review(s) concerned evidence that no longer matches the \
                 current text; archived under a versioned ID, not applied, and need a fresh \
                 review of the new text: {}",
                superseded.len(),
                superseded.join(", ")
            );
        }
    }
    enrich_review_context(&mut routed.review_items, &context.config);
    write_pretty_json(&routed.result, &paths.temporal_result)?;
    write_pretty_json(&routed.review_items, &paths.review_queue)?;

    let mut corpus = read_corpus(paths)?;
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

/// Deterministic, no-model temporal classification (`lex_parse::temporal_derive`).
/// Reports what it resolved or skipped and why; never fails the pipeline --
/// a skipped instrument is an expected, common outcome, not an error.
fn run_derive_command(
    root: &Path,
    instrument: &str,
    repair_dates_only: bool,
    vault: Option<&Path>,
) -> Result<()> {
    let context = instrument_context(root, instrument)?;
    run_temporal_derive(&context, repair_dates_only)?;
    republish_exports(root, &context, vault)
}

fn run_temporal_derive(context: &InstrumentContext, repair_dates_only: bool) -> Result<()> {
    let paths = &context.paths;
    let mut corpus = read_corpus(paths)?;
    let cleared = repair_article_dates(&corpus.instrument, &mut corpus.provisions);
    if cleared > 0 {
        println!(
            "repaired {cleared} article dates (original commencement for unamended articles; amended articles left unset)"
        );
    }
    let split = repair_partial_repeals(&mut corpus.provisions);
    if split > 0 {
        println!(
            "reclassified {split} partially repealed articles (repeal scope recorded; remaining text stays in force)"
        );
    }
    if repair_dates_only {
        if cleared + split > 0 {
            write_canonical(&corpus, &paths.corpus)?;
        }
        return Ok(());
    }
    let today = Utc::now().date_naive();
    let outcome =
        derive_article_temporal_determinations(&corpus.instrument, &corpus.provisions, today);
    let repealed = outcome
        .determinations
        .iter()
        .filter(|item| item.temporal_status == TemporalStatus::Repealed)
        .count();
    let commenced = outcome.determinations.len() - repealed;
    match &outcome.commencement {
        Some(Commencement::Resolved {
            effective_from,
            pattern,
            ..
        }) => {
            println!(
                "temporal derive: commencement resolved via {pattern} ({effective_from}); {repealed} repealed, {commenced} classified from commencement"
            );
        }
        Some(Commencement::Skipped { reason, .. }) => {
            println!(
                "temporal derive: commencement skipped ({}: {}); {repealed} repealed, 0 classified from commencement",
                reason.code(),
                reason.describe()
            );
        }
        None => {
            println!(
                "temporal derive: {} (no ordinary transitories); {repealed} repealed",
                corpus.instrument.id
            );
        }
    }
    if outcome.determinations.is_empty() && cleared == 0 {
        return Ok(());
    }
    apply_temporal_determinations(&mut corpus.provisions, &outcome.determinations);
    write_canonical(&corpus, &paths.corpus)?;
    Ok(())
}

fn enrich_review_context(items: &mut [ReviewItem], source: &SourceConfig) {
    for item in items {
        let publication_date = item
            .proposed_machine_conclusion
            .publication_date
            .to_string();
        item.formal_source_url = source
            .formal_publication_urls
            .get(&publication_date)
            .cloned();
        item.provision_diff.get_or_insert_with(|| {
            if item.provision_id.contains(":amendment:") {
                "Unavailable in the one-law vertical slice; an affected-provision diff requires \
                 amendment-event modeling."
                    .to_owned()
            } else {
                "Initial enactment; there is no prior provision version for this instrument."
                    .to_owned()
            }
        });
    }
}

fn run_review_command(
    root: &Path,
    context: &InstrumentContext,
    obsidian_vault: Option<&Path>,
    command: ReviewCommand,
) -> Result<()> {
    match command {
        ReviewCommand::List { all } => run_review_list(context, all),
        ReviewCommand::Legacy {
            all_instruments,
            check_upstream,
        } => run_review_legacy(root, context, all_instruments, check_upstream),
        ReviewCommand::Open {
            provision_id,
            reason,
        } => {
            run_review_open(context, &provision_id, &reason)?;
            run_export(root, context, ExportFormat::Markdown, None)?;
            if let Some(vault) = obsidian_vault {
                run_export(root, context, ExportFormat::Obsidian, Some(vault))?;
            }
            Ok(())
        }
        ReviewCommand::Resolve {
            review_id,
            resolution,
            reviewer,
            note,
            temporal_status,
            effective_from,
            effective_to,
            effects_file,
        } => {
            let effects: Option<Vec<TransitoryEffect>> = effects_file
                .as_deref()
                .map(read_json)
                .transpose()
                .with_context(|| "failed to read --effects-file")?;
            run_review_resolve(
                context,
                &review_id,
                TemporalReviewResolution {
                    resolution: resolution.into(),
                    reviewer,
                    note,
                    temporal_status: temporal_status.map(Into::into),
                    effective_from,
                    effective_to,
                    effects,
                    resolved_at: Utc::now(),
                },
            )?;
            run_export(root, context, ExportFormat::Markdown, None)?;
            if let Some(vault) = obsidian_vault {
                run_export(root, context, ExportFormat::Obsidian, Some(vault))?;
            }
            Ok(())
        }
    }
}

fn run_review_open(context: &InstrumentContext, provision_id: &str, reason: &str) -> Result<()> {
    let paths = &context.paths;
    let mut items: Vec<ReviewItem> = read_json(&paths.review_queue)
        .with_context(|| "review queue not found; run temporal analysis first")?;
    let mut result: TemporalAnalysisResult = read_json(&paths.temporal_result)
        .with_context(|| "temporal result not found; run temporal analysis first")?;
    let request: TemporalAnalysisRequest = read_json(&paths.temporal_request)
        .with_context(|| "temporal request not found; run temporal analysis first")?;
    let instrument: Instrument = read_json(&paths.instrument)?;
    open_temporal_review(
        &mut items,
        &mut result,
        &request,
        provision_id,
        reason,
        &instrument.source_url,
    )?;
    enrich_review_context(&mut items, &context.config);
    write_pretty_json(&result, &paths.temporal_result)?;
    write_pretty_json(&items, &paths.review_queue)?;
    // Reflect the pending review in the canonical provision state.
    let mut corpus = read_corpus(paths)?;
    apply_temporal_determinations(&mut corpus.provisions, &result.determinations);
    write_canonical(&corpus, &paths.corpus)?;
    println!("opened review:temporal:{provision_id}");
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ReviewedTextState {
    /// The text the reviewer saw is byte-identical to the current evidence.
    Matches,
    /// The provision's current evidence text is not what the reviewer saw.
    Differs,
    /// The provision no longer produces temporal evidence.
    Absent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LegacyReviewRow {
    review_id: String,
    status: ReviewItemStatus,
    resolved_by: Option<String>,
    state: ReviewedTextState,
}

/// Review items whose proposed conclusion carries no evidence hash predate
/// hash tracking. Their embedded evidence is still the text the reviewer saw,
/// so compare it directly with the current corpus evidence.
fn legacy_review_rows(items: &[ReviewItem], current: &[TemporalEvidence]) -> Vec<LegacyReviewRow> {
    items
        .iter()
        .filter(|item| item.proposed_machine_conclusion.evidence_sha256.is_empty())
        .map(|item| {
            let state = match current
                .iter()
                .find(|evidence| evidence.provision_id == item.provision_id)
            {
                None => ReviewedTextState::Absent,
                Some(evidence)
                    if lex_core::evidence_sha256(&evidence.text)
                        == lex_core::evidence_sha256(&item.evidence.text) =>
                {
                    ReviewedTextState::Matches
                }
                Some(_) => ReviewedTextState::Differs,
            };
            LegacyReviewRow {
                review_id: item.id.clone(),
                status: item.status,
                resolved_by: item.resolved_by.clone(),
                state,
            }
        })
        .collect()
}

/// Whether the publisher's current headers match the retained manifest.
/// `ETag` is authoritative, then `Last-Modified`; a length match alone is not
/// enough to call the source unchanged.
fn upstream_verdict(
    manifest: &lex_core::SourceManifest,
    probe: &lex_source::SourceProbe,
) -> &'static str {
    if probe.http_status != 200 {
        return "unreachable";
    }
    if let (Some(old), Some(new)) = (&manifest.etag, &probe.etag) {
        return if old == new { "unchanged" } else { "moved" };
    }
    match (&manifest.last_modified, &probe.last_modified) {
        (Some(old), Some(new)) => {
            if old == new {
                "unchanged"
            } else {
                "moved"
            }
        }
        _ => "unknown",
    }
}

fn run_review_legacy(
    root: &Path,
    context: &InstrumentContext,
    all_instruments: bool,
    check_upstream: bool,
) -> Result<()> {
    let slugs = if all_instruments {
        let mut slugs: Vec<String> = fs::read_dir(root.join("corpus/mx"))?
            .filter_map(Result::ok)
            .filter(|entry| entry.path().join("review-queue.json").exists())
            .filter_map(|entry| entry.file_name().into_string().ok())
            .collect();
        slugs.sort();
        slugs
    } else {
        vec![context.config.slug.clone()]
    };
    let mut total = 0;
    for slug in slugs {
        let instrument = instrument_context(root, &slug)?;
        let paths = &instrument.paths;
        let Ok(items) = read_json::<Vec<ReviewItem>>(&paths.review_queue) else {
            continue;
        };
        let rows = legacy_review_rows(
            &items,
            &current_temporal_evidence(&instrument, &read_corpus(paths)?)?,
        );
        if rows.is_empty() {
            continue;
        }
        let upstream = if check_upstream {
            let manifest: lex_core::SourceManifest = read_json(&paths.manifest)?;
            Some(match lex_source::probe_source(&instrument.config) {
                Ok(probe) => format!(
                    "{} (manifest retrieved {}, etag {:?})",
                    upstream_verdict(&manifest, &probe),
                    manifest.retrieved_at.format("%Y-%m-%d"),
                    manifest.etag
                ),
                Err(error) => format!("unreachable: {error:#}"),
            })
        } else {
            None
        };
        println!("{slug}");
        if let Some(upstream) = upstream {
            println!("  upstream source: {upstream}");
        }
        for row in &rows {
            println!(
                "  {}\n    status: {:?}, resolved by: {}\n    reviewed text vs current evidence: {}",
                row.review_id,
                row.status,
                row.resolved_by.as_deref().unwrap_or("-"),
                match row.state {
                    ReviewedTextState::Matches => "matches",
                    ReviewedTextState::Differs => "DIFFERS, re-review needed",
                    ReviewedTextState::Absent => "provision no longer present",
                }
            );
        }
        total += rows.len();
    }
    println!("{total} legacy review item(s)");
    Ok(())
}

fn run_review_list(context: &InstrumentContext, all: bool) -> Result<()> {
    let items: Vec<ReviewItem> = read_json(&context.paths.review_queue)
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
        let resolution = item.resolution.map_or_else(String::new, |resolution| {
            format!(
                "\n  resolution: {:?}\n  resolved by: {}",
                resolution,
                item.resolved_by.as_deref().unwrap_or("unknown")
            )
        });
        println!(
            "{}\n  {}\n  status: {:?}\n  confidence: {:.2}\n  issue: {}{}",
            item.id,
            item.evidence.label,
            item.status,
            item.proposed_machine_conclusion.confidence,
            item.exact_issue,
            resolution,
        );
    }
    println!("{} review items", visible.len());
    Ok(())
}

fn run_review_resolve(
    context: &InstrumentContext,
    review_id: &str,
    resolution: TemporalReviewResolution,
) -> Result<()> {
    let paths = &context.paths;
    let mut items: Vec<ReviewItem> = read_json(&paths.review_queue)
        .with_context(|| "review queue not found; run temporal analysis first")?;
    let mut result: TemporalAnalysisResult = read_json(&paths.temporal_result)
        .with_context(|| "temporal result not found; run temporal analysis first")?;
    let item = items
        .iter_mut()
        .find(|item| item.id == review_id)
        .with_context(|| format!("review item not found: {review_id}"))?;
    let corpus = read_corpus(paths)?;
    let current = current_temporal_evidence(context, &corpus)?;
    if !current.iter().any(|evidence| {
        evidence.provision_id == item.provision_id
            && lex_core::evidence_sha256(&evidence.text)
                == item.proposed_machine_conclusion.evidence_sha256
    }) {
        bail!(
            "review evidence is stale against the current corpus; regenerate analysis and review"
        );
    }
    resolve_temporal_review(item, &mut result.determinations, resolution)?;

    write_pretty_json(&result, &paths.temporal_result)?;
    write_pretty_json(&items, &paths.review_queue)?;
    let mut corpus = read_corpus(paths)?;
    apply_temporal_determinations(&mut corpus.provisions, &result.determinations);
    write_canonical(&corpus, &paths.corpus)?;
    println!("resolved {review_id}");
    Ok(())
}

fn validate_corpus_snapshot(
    root: &Path,
    context: &InstrumentContext,
) -> Result<lex_core::ValidationReport> {
    let paths = &context.paths;
    let config = &context.config;
    let corpus = read_corpus(paths)?;
    validate_persisted_repeal_evidence(root, &corpus)?;
    let mut external_targets = HashSet::new();
    let mut external_terms = HashSet::new();
    let siblings = read_sibling_corpora(root, &corpus.instrument.id)?;
    for (_, sibling) in &siblings {
        external_targets.extend(sibling.provisions.iter().map(|item| item.id.clone()));
        external_terms.extend(sibling.terms.iter().map(|term| term.id.clone()));
    }
    let mut report = validate_corpus(
        &CorpusView {
            instrument_id: &corpus.instrument.id,
            official_title: Some(corpus.instrument.official_title.as_str()),
            provisions: &corpus.provisions,
            references: &corpus.references,
            terms: &corpus.terms,
            term_usages: &corpus.term_usages,
            amendment_references: &corpus.amendment_references,
        },
        &CorpusExpectations {
            min_articles: config.expected_min_articles,
            articles: config.expected_articles,
            transitories: config.expected_transitories,
            annexes: config.expected_annexes,
            require_chapter_context: config.parser == "ifpe-dcg",
            allow_article_gaps: config.allow_article_gaps,
        },
        &external_targets,
        &external_terms,
    );
    append_parent_rule_issues(&mut report, config, &corpus.references);
    let sibling_views: Vec<_> = siblings
        .iter()
        .map(|(slug, sibling)| (slug.as_str(), sibling))
        .collect();
    append_glossary_coverage_issues(&mut report, &corpus, &paths.corpus, context, &sibling_views)?;
    Ok(report)
}

fn run_validate(root: &Path, context: &InstrumentContext) -> Result<lex_core::ValidationReport> {
    let paths = &context.paths;
    let report = validate_corpus_snapshot(root, context)?;
    write_validation(&report, &paths.corpus)?;
    println!(
        "validation: {}; {} articles, {} transitories, {} references, {} issues",
        if report.valid { "valid" } else { "invalid" },
        report.article_count,
        report.transitory_count,
        report.reference_count,
        report.issues.len()
    );
    Ok(report)
}

fn run_export(
    root: &Path,
    context: &InstrumentContext,
    format: ExportFormat,
    obsidian_vault: Option<&Path>,
) -> Result<()> {
    let paths = &context.paths;
    let corpus = read_corpus(paths)?;
    validate_persisted_repeal_evidence(root, &corpus)?;
    let siblings = read_sibling_corpora(root, &corpus.instrument.id)?;
    let (targets, terms) = build_link_targets(&corpus, &context.config.slug, &siblings);
    match format {
        ExportFormat::Json => write_canonical(&corpus, &paths.corpus)?,
        ExportFormat::Markdown => write_markdown(&corpus, &targets, &terms, &paths.markdown)?,
        ExportFormat::Obsidian => {
            let vault = obsidian_vault.context(
                "Obsidian export requires --obsidian-vault PATH or \
                 LEX_MEX_OBSIDIAN_VAULT",
            )?;
            let review_items = read_all_review_queues(root)?;
            write_obsidian(&corpus, &targets, &terms, &review_items, vault)?;
            println!("published Obsidian vault {}", vault.display());
        }
    }
    println!("exported {format:?}");
    Ok(())
}

fn build_link_targets(
    corpus: &Corpus,
    slug: &str,
    siblings: &[(String, Corpus)],
) -> (LinkTargets, TermTargets) {
    let mut corpora: Vec<(&Corpus, &str)> = vec![(corpus, slug)];
    corpora.extend(
        siblings
            .iter()
            .map(|(sibling_slug, sibling)| (sibling, sibling_slug.as_str())),
    );
    let targets = link_targets(&corpora);
    let terms = term_targets(&corpora, &targets);
    (targets, terms)
}

/// Read every committed corpus, keyed by its corpus directory slug.
fn read_committed_corpora(root: &Path) -> Result<Vec<(String, Corpus)>> {
    let corpus_root = root.join("corpus/mx");
    let mut corpora = Vec::new();
    if !corpus_root.is_dir() {
        return Ok(corpora);
    }
    let mut directories: Vec<_> = fs::read_dir(&corpus_root)?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|entry| entry.path())
        .filter(|path| path.join("instrument.json").is_file())
        .collect();
    directories.sort();
    for directory in directories {
        let slug = directory
            .file_name()
            .and_then(|name| name.to_str())
            .context("corpus directory name is not valid UTF-8")?
            .to_owned();
        let paths = Paths::new(root, &slug, SourceFormat::Pdf);
        let corpus = read_corpus(&paths)?;
        corpora.push((slug, corpus));
    }
    Ok(corpora)
}

/// Read every committed corpus except `own_instrument_id`, keyed by its
/// corpus directory slug.
fn read_sibling_corpora(root: &Path, own_instrument_id: &str) -> Result<Vec<(String, Corpus)>> {
    read_committed_corpora(root).map(|corpora| {
        corpora
            .into_iter()
            .filter(|(_, corpus)| corpus.instrument.id != own_instrument_id)
            .collect()
    })
}

fn read_all_review_queues(root: &Path) -> Result<Vec<ReviewItem>> {
    let corpus_root = root.join("corpus/mx");
    let mut items = Vec::new();
    if !corpus_root.is_dir() {
        return Ok(items);
    }
    let mut queues: Vec<_> = fs::read_dir(&corpus_root)?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|entry| entry.path().join("review-queue.json"))
        .filter(|path| path.is_file())
        .collect();
    queues.sort();
    for queue in queues {
        let mut queue_items: Vec<ReviewItem> = read_json(&queue)?;
        items.append(&mut queue_items);
    }
    Ok(items)
}

fn read_corpus(paths: &Paths) -> Result<Corpus> {
    fn optional<T: serde::de::DeserializeOwned>(path: &Path) -> Result<Vec<T>> {
        if path.exists() {
            read_json(path)
        } else {
            Ok(Vec::new())
        }
    }
    let corpus = Corpus {
        enactment_evidence: optional(&paths.corpus.join("enactment-evidence.json"))?,
        instrument: read_json(&paths.instrument)?,
        provisions: read_json(&paths.provisions)?,
        references: optional(&paths.references)?,
        terms: optional(&paths.terms)?,
        term_usages: optional(&paths.term_usages)?,
        amendment_references: optional(&paths.amendment_references)?,
    };
    validate_source_metadata(&corpus)?;
    Ok(corpus)
}

fn instrument_type(value: &str) -> Result<InstrumentType> {
    Ok(match value {
        "constitution" => InstrumentType::Constitution,
        "code" => InstrumentType::Code,
        "statute" => InstrumentType::Statute,
        "regulation" => InstrumentType::Regulation,
        "guideline" => InstrumentType::Guideline,
        "circular" => InstrumentType::Circular,
        "other" => InstrumentType::Other,
        unsupported => bail!("unsupported instrument type {unsupported:?}"),
    })
}

fn latest_reform_date(raw: &str) -> Option<NaiveDate> {
    let regex = Regex::new(r"Última (?:r|R)eforma(?: publicada)? DOF (\d{2}-\d{2}-\d{4})")
        .expect("static regex");
    regex
        .captures_iter(raw)
        .filter_map(|captures| NaiveDate::parse_from_str(&captures[1], "%d-%m-%Y").ok())
        .max()
}

fn cleanup_work(context: &InstrumentContext) -> Result<()> {
    let work = &context.paths.work;
    if work.exists() {
        fs::remove_dir_all(work)
            .with_context(|| format!("failed to delete temporary work {}", work.display()))?;
    }
    Ok(())
}

fn absolute_root(root: &Path) -> Result<PathBuf> {
    if root == Path::new(".") {
        std::env::current_dir().map_err(Into::into)
    } else if root.is_absolute() {
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
    work: PathBuf,
    source: PathBuf,
    text: PathBuf,
    formal_source: PathBuf,
    formal_text: PathBuf,
    formal_manifest: PathBuf,
    annex_manifests: PathBuf,
    corpus: PathBuf,
    manifest: PathBuf,
    instrument: PathBuf,
    provisions: PathBuf,
    references: PathBuf,
    terms: PathBuf,
    term_usages: PathBuf,
    amendment_references: PathBuf,
    temporal_request: PathBuf,
    temporal_model_output: PathBuf,
    temporal_result: PathBuf,
    review_queue: PathBuf,
    reform_evidence: PathBuf,
    markdown: PathBuf,
}

impl Paths {
    fn new(root: &Path, slug: &str, source_format: SourceFormat) -> Self {
        let work = root.join(".work").join(slug);
        let corpus = root.join("corpus/mx").join(slug);
        let source_extension = match source_format {
            SourceFormat::Pdf => "pdf",
            SourceFormat::Html => "html",
            SourceFormat::Doc => "doc",
        };
        Self {
            source: work.join(format!("{slug}.{source_extension}")),
            text: work.join(format!("{slug}.txt")),
            formal_source: work.join(format!("{slug}-formal.html")),
            formal_text: work.join(format!("{slug}-formal.txt")),
            formal_manifest: corpus.join("formal-source-manifest.json"),
            annex_manifests: corpus.join("annex-source-manifests.json"),
            manifest: corpus.join("source-manifest.json"),
            instrument: corpus.join("instrument.json"),
            provisions: corpus.join("provisions.json"),
            references: corpus.join("references.json"),
            terms: corpus.join("terms.json"),
            term_usages: corpus.join("term-usages.json"),
            amendment_references: corpus.join("amendment-references.json"),
            temporal_request: corpus.join("temporal-analysis-request.json"),
            temporal_model_output: work.join("temporal-model-output.json"),
            temporal_result: corpus.join("temporal-analysis-result.json"),
            review_queue: corpus.join("review-queue.json"),
            reform_evidence: corpus.join("reform-temporal-evidence.json"),
            markdown: corpus.join("markdown"),
            work,
            corpus,
        }
    }

    /// 1-indexed annex PDF/text paths, matching CNBV's own annex numbering.
    fn annex_pdf(&self, number: usize) -> PathBuf {
        self.work.join(format!("annex-{number}.pdf"))
    }

    fn annex_text(&self, number: usize) -> PathBuf {
        self.work.join(format!("annex-{number}.txt"))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        InstrumentContext, OGA_SERIAL_NOTE, SourceConfig, SourceFormat, SourceManifest,
        additive_terms, extract_instrument_terms, instrument_context, load_config,
        preserve_intake_decisions, resolve_publication, run_temporal_request,
        self_reference_markers,
    };
    use std::{
        collections::{BTreeMap, HashMap, HashSet},
        fs,
        path::Path,
    };

    use chrono::{NaiveDate, Utc};
    use clap::Parser;
    use lex_core::{
        Corpus, HeadingContext, Instrument, InstrumentStatus, InstrumentType, Provision,
        ProvisionType, ReferenceResolutionStatus, ReviewStatus, SCHEMA_VERSION, TemporalStatus,
    };
    use lex_export::write_canonical;
    use url::Url;

    #[test]
    fn usages_inside_an_instrument_title_are_dropped_unless_the_definition_names_it() {
        let text = "La Ley y la Ley Federal de\nDerechos rigen las Aguas Nacionales; \
                    la Ley de Aguas Nacionales también, y el Reglamento de la Ley de Aguas \
                    Nacionales.";
        let provision = Provision {
            text: text.to_owned(),
            ..fixture_corpus("urn:x:reg", "Reglamento", "REG", "Texto").provisions[0].clone()
        };
        let at = |needle: &str, nth: usize| {
            let byte = text.match_indices(needle).nth(nth).unwrap().0;
            text[..byte].chars().count()
        };
        let usage = |term: &str, span: &str, start: usize| lex_core::TermUsage {
            schema_version: SCHEMA_VERSION.to_owned(),
            id: format!("{}:term-usage:{start}", provision.id),
            provision_id: provision.id.clone(),
            term_id: term.to_owned(),
            span: span.to_owned(),
            start_char: start,
            end_char: start + span.chars().count(),
        };
        let usages = vec![
            usage("ley", "Ley", at("Ley", 0)),
            usage("ley", "Ley", at("Ley", 1)),
            usage("aguas", "Aguas Nacionales", at("Aguas Nacionales", 0)),
            usage("ley", "Ley", at("Ley", 2)),
            usage("aguas", "Aguas Nacionales", at("Aguas Nacionales", 1)),
            usage("ley", "Ley", at("Ley", 3)),
        ];
        // `Ley` is defined as the Ley de Aguas Nacionales; the waters are not.
        let definitions = HashMap::from([
            (
                "ley".to_owned(),
                "ley: la ley de aguas nacionales;".to_owned(),
            ),
            (
                "aguas".to_owned(),
                "aguas nacionales: las del artículo 27;".to_owned(),
            ),
        ]);
        let index = super::title_index(&[
            "Ley Federal de Derechos".to_owned(),
            "Ley de Aguas Nacionales".to_owned(),
            "Reglamento de la Ley de Aguas Nacionales".to_owned(),
        ])
        .unwrap();
        let kept: Vec<(String, usize)> =
            super::drop_usages_in_titles(usages, &[provision], &definitions, &index)
                .into_iter()
                .map(|usage| (usage.term_id, usage.start_char))
                .collect();
        assert_eq!(
            kept,
            [
                ("ley".to_owned(), at("Ley", 0)),
                ("aguas".to_owned(), at("Aguas Nacionales", 0)),
                ("ley".to_owned(), at("Ley", 2)),
                // Inside the Reglamento's title, `Ley` is still the title it
                // is defined as.
                ("ley".to_owned(), at("Ley", 3)),
            ]
        );
    }

    #[test]
    fn a_term_inside_an_institution_name_a_glossary_defines_is_not_a_usage() {
        let glossary = "Para los efectos de esta Ley se entiende por:\n\n\
             I. Ambiente: El conjunto de elementos naturales;\n\n\
             II. Procuraduría: La Procuraduría Federal de Protección al Ambiente;\n\n\
             III. Secretaría: Secretaría de Economía;\n\n\
             IV. Hacienda: Secretaría de Hacienda y Crédito Público;\n\n\
             V. Consejo: Consejo Técnico del Instituto;\n\n\
             VI. Comité: Consejo Consultivo de la AMEXCID;\n\n\
             VII. Fiscalías: Fiscalía Especializada de la Fiscalía y de las Procuradurías.";
        let provision = Provision {
            text: glossary.to_owned(),
            ..fixture_corpus("urn:x:law", "Ley", "LEY", "Texto").provisions[0].clone()
        };
        let entry = |term: &str, fraction: &str| {
            let start = glossary.find(&format!("{fraction}. {term}")).unwrap();
            let end = glossary[start..]
                .find(";\n\n")
                .map_or(glossary.len(), |end| start + end + 1);
            lex_core::DefinedTerm {
                schema_version: SCHEMA_VERSION.to_owned(),
                id: format!("urn:x:law:term:{}", term.to_lowercase()),
                term: term.to_owned(),
                instrument_id: "urn:x:law".to_owned(),
                defining_provision_id: provision.id.clone(),
                fraction: Some(fraction.to_owned()),
                inciso: None,
                start_char: glossary[..start].chars().count(),
                end_char: glossary[..end].chars().count(),
                basis: lex_core::Basis::ExpressDefinition,
            }
        };
        let terms = [
            entry("Ambiente", "I"),
            entry("Procuraduría", "II"),
            entry("Secretaría", "III"),
            entry("Hacienda", "IV"),
            entry("Consejo", "V"),
            entry("Comité", "VI"),
            entry("Fiscalías", "VII"),
        ];
        assert_eq!(
            super::institution_names(&terms, std::slice::from_ref(&provision)),
            [
                "procuraduría federal de protección al ambiente",
                "secretaría de economía",
                "secretaría de hacienda y crédito público",
            ]
        );

        let text = "La Procuraduría Federal de Protección al Ambiente cuida el Ambiente; \
                    la Secretaría informa a la Secretaría de Hacienda y Crédito Público.";
        let operative = Provision {
            id: "urn:x:law:article:2".to_owned(),
            text: text.to_owned(),
            ..provision.clone()
        };
        let usage = |term: &str, span: &str, nth: usize| {
            let byte = text.match_indices(span).nth(nth).unwrap().0;
            let start = text[..byte].chars().count();
            lex_core::TermUsage {
                schema_version: SCHEMA_VERSION.to_owned(),
                id: format!("urn:x:law:article:2:term-usage:{start}"),
                provision_id: operative.id.clone(),
                term_id: format!("urn:x:law:term:{}", term.to_lowercase()),
                span: span.to_owned(),
                start_char: start,
                end_char: start + span.chars().count(),
            }
        };
        let usages = vec![
            usage("Procuraduría", "Procuraduría", 0),
            usage("Ambiente", "Ambiente", 0),
            usage("Ambiente", "Ambiente", 1),
            usage("Secretaría", "Secretaría", 0),
            usage("Secretaría", "Secretaría", 1),
        ];
        let definitions = super::term_definitions(&[&terms], std::slice::from_ref(&provision), &[]);
        let index = super::title_index(&super::institution_names(
            &terms,
            std::slice::from_ref(&provision),
        ))
        .unwrap();
        let kept: Vec<(String, usize)> =
            super::drop_usages_in_titles(usages, &[operative], &definitions, &index)
                .into_iter()
                .map(|usage| (usage.span, usage.start_char))
                .collect();
        let at = |span: &str, nth: usize| {
            let byte = text.match_indices(span).nth(nth).unwrap().0;
            text[..byte].chars().count()
        };
        // The Procuraduría is the name its definition gives; the Ambiente in
        // its name is not, and the Secretaría (de Economía) is not the one
        // in the name of the Secretaría de Hacienda.
        assert_eq!(
            kept,
            [
                ("Procuraduría".to_owned(), at("Procuraduría", 0)),
                ("Ambiente".to_owned(), at("Ambiente", 1)),
                ("Secretaría".to_owned(), at("Secretaría", 0)),
            ]
        );
    }

    #[test]
    fn a_title_printed_with_long_whitespace_runs_still_covers_its_usages() {
        let index = super::title_index(&[
            "Ley de Aguas Nacionales".to_owned(),
            "Ley de Aguas".to_owned(),
        ])
        .unwrap();
        let usage = |text: &str, term: &str, span: &str| {
            let byte = text.rfind(span).unwrap();
            let start = text[..byte].chars().count();
            let provision = Provision {
                text: text.to_owned(),
                ..fixture_corpus("urn:x:law", "Ley", "LEY", "Texto").provisions[0].clone()
            };
            let usage = lex_core::TermUsage {
                schema_version: SCHEMA_VERSION.to_owned(),
                id: format!("{}:term-usage:{start}", provision.id),
                provision_id: provision.id.clone(),
                term_id: term.to_owned(),
                span: span.to_owned(),
                start_char: start,
                end_char: start + span.chars().count(),
            };
            (provision, usage)
        };
        let kept = |text: &str, term: &str, span: &str, definition: &str| {
            let (provision, usage) = usage(text, term, span);
            let definitions = HashMap::from([(term.to_owned(), definition.to_owned())]);
            super::drop_usages_in_titles(vec![usage], &[provision], &definitions, &index).len()
        };
        let spaces = " ".repeat(80);
        let breaks = "\n".repeat(40);
        for text in [
            format!("Ñandú. Ley{spaces}de Aguas Nacionales."),
            format!("Ñandú. Ley de{breaks}Aguas\t\t\t\t Nacionales."),
            format!("Ñandú. Ley\t{spaces}de{breaks}Aguas{spaces}Nacionales"),
        ] {
            assert_eq!(
                kept(
                    &text,
                    "nacionales",
                    "Nacionales",
                    "nacionales: las personas;"
                ),
                0,
                "{text:?}"
            );
        }
        // The definition must name the covering title as whole words: the
        // Ley de Aguascalientes does not name the Ley de Aguas.
        let text = "Conforme a la Ley de Aguas.";
        assert_eq!(
            kept(text, "ley", "Ley", "ley: la ley de aguascalientes;"),
            0
        );
        assert_eq!(kept(text, "ley", "Ley", "ley: la ley de aguas;"), 1);
    }

    #[test]
    fn a_reviewed_non_usage_removes_exactly_its_pinned_match_and_a_stale_one_fails() {
        let usage = |start: usize, span: &str| lex_core::TermUsage {
            schema_version: SCHEMA_VERSION.to_owned(),
            id: format!(
                "urn:x:law:article:3:term-usage:{start}-{}",
                start + span.len()
            ),
            provision_id: "urn:x:law:article:3".to_owned(),
            term_id: "urn:x:law:term:alcohol".to_owned(),
            span: span.to_owned(),
            start_char: start,
            end_char: start + span.len(),
        };
        let exclusion = |start: usize| lex_source::UsageExclusion {
            provision_suffix: ":article:3".to_owned(),
            start_char: start,
            text: "alcohol".to_owned(),
            term_id: "urn:x:law:term:alcohol".to_owned(),
            reason: "alcohol isopropílico".to_owned(),
        };
        let usages = vec![usage(10, "alcohol"), usage(40, "alcohol")];
        let kept =
            super::exclude_reviewed_non_usages("urn:x:law", usages.clone(), &[exclusion(40)])
                .unwrap();
        assert_eq!(kept.iter().map(|u| u.start_char).collect::<Vec<_>>(), [10]);
        // The text moved: the exclusion no longer matches and extraction fails.
        let error = super::exclude_reviewed_non_usages("urn:x:law", usages, &[exclusion(41)])
            .unwrap_err()
            .to_string();
        assert!(error.contains("no longer matches"), "{error}");
    }

    use super::{
        Cli, Command, ExpectedEdgeCorpus, InstrumentAliasTable, LinkedGraph, Paths, SearchScope,
        committed_instrument_index, evaluate_expected_edges, freeze_adapter_baseline,
        latest_reform_date, merge_external_instruments, protected_edge_drift, read_corpus,
        resolve_global_aliases, run_batch_closure, run_corpus_closure, run_link, run_validate,
        scaffold_adapter, selected_corpus_paths, unique_official_title_markers,
    };

    const STALE_RUNNING_HEADER_REFORM_DATE_FIXTURE: &str = include_str!(
        "../../../fixtures/diputados/latest-reform-date-stale-running-header-sample.txt"
    );

    #[test]
    fn temporal_import_rejects_changed_missing_and_duplicate_current_evidence() {
        let evidence = lex_core::TemporalEvidence {
            provision_id: "urn:test:transitory:one".to_owned(),
            label: "One".to_owned(),
            text: "Original source text".to_owned(),
            amendment_marks: Vec::new(),
        };
        let original = vec![evidence.clone()];
        assert!(super::ensure_current_temporal_evidence(&original, &original).is_ok());
        let mut changed = evidence.clone();
        changed.text.push_str(" changed");
        assert!(super::ensure_current_temporal_evidence(&original, &[changed]).is_err());
        assert!(super::ensure_current_temporal_evidence(&original, &[]).is_err());
        assert!(
            super::ensure_current_temporal_evidence(&[evidence.clone(), evidence], &original)
                .is_err()
        );
    }

    #[test]
    fn latest_reform_date_uses_latest_matching_date() {
        assert_eq!(
            latest_reform_date(STALE_RUNNING_HEADER_REFORM_DATE_FIXTURE),
            NaiveDate::from_ymd_opt(2025, 11, 14)
        );
    }

    #[test]
    fn scaffolded_adapter_freezes_an_exact_only_count_baseline() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path();
        let adapter_dir = root.join("adapters/diputados");
        let batch_dir = root.join("batches");
        fs::create_dir_all(&adapter_dir).unwrap();
        fs::create_dir_all(&batch_dir).unwrap();
        let manifest_path = batch_dir.join("sample.json");
        let manifest = serde_json::json!({
            "schema_version": "0.1.0",
            "batch_id": "sample_batch",
            "description": "Scaffold and freeze regression fixture.",
            "instruments": [{
                "slug": "sample-law",
                "title": "LEY de Muestra",
                "type": "ley",
                "adapter": "diputados",
                "status": "NEW",
                "ref_page": "https://example.test/ref/sample-law.htm",
                "source_pdf": "https://example.test/pdf/sample-law.pdf"
            }]
        });
        fs::write(
            &manifest_path,
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();

        let adapter_path = scaffold_adapter(root, &manifest_path, "sample-law").unwrap();
        let scaffolded = lex_source::load_config(&adapter_path).unwrap();
        assert_eq!(scaffolded.expected_min_articles, None);
        assert_eq!(scaffolded.expected_articles, None);
        assert_eq!(scaffolded.expected_transitories, None);
        assert!(!scaffolded.allow_article_gaps);
        assert!(scaffolded.count_baseline.is_none());

        let publication_date = NaiveDate::from_ymd_opt(1982, 12, 31).unwrap();
        assert!(freeze_adapter_baseline(&adapter_path, Some(publication_date), 35, 2).unwrap());
        let frozen = lex_source::load_config(&adapter_path).unwrap();
        assert_eq!(frozen.publication_date.as_deref(), Some("1982-12-31"));
        assert_eq!(frozen.expected_min_articles, None);
        assert_eq!(frozen.expected_articles, Some(35));
        assert_eq!(frozen.expected_transitories, Some(2));
        assert!(!frozen.allow_article_gaps);
        assert_eq!(
            frozen
                .count_baseline
                .as_ref()
                .map(|baseline| baseline.provenance.as_str()),
            Some("machine-proposed")
        );
        assert!(!freeze_adapter_baseline(&adapter_path, Some(publication_date), 99, 99).unwrap());
    }

    fn legacy_item() -> lex_core::ReviewItem {
        let mut items: Vec<lex_core::ReviewItem> =
            serde_json::from_str(include_str!("../../../corpus/mx/lritf/review-queue.json"))
                .unwrap();
        let mut item = items.remove(0);
        item.proposed_machine_conclusion.evidence_sha256 = String::new();
        item
    }

    #[test]
    fn legacy_reviews_are_compared_with_current_evidence() {
        let item = legacy_item();
        let same = vec![item.evidence.clone()];
        let rows = super::legacy_review_rows(std::slice::from_ref(&item), &same);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].state, super::ReviewedTextState::Matches);

        let mut changed = item.evidence.clone();
        changed.text.push_str(" nuevo texto");
        let rows = super::legacy_review_rows(std::slice::from_ref(&item), &[changed]);
        assert_eq!(rows[0].state, super::ReviewedTextState::Differs);

        let rows = super::legacy_review_rows(std::slice::from_ref(&item), &[]);
        assert_eq!(rows[0].state, super::ReviewedTextState::Absent);

        let mut hashed = item;
        hashed.proposed_machine_conclusion.evidence_sha256 = "abc".to_owned();
        let found = super::legacy_review_rows(&[hashed], &same);
        assert!(found.is_empty(), "{found:?}");
    }

    #[test]
    fn upstream_verdict_prefers_etag_then_last_modified() {
        let mut manifest: lex_core::SourceManifest =
            serde_json::from_str(include_str!("../../../corpus/mx/lacp/source-manifest.json"))
                .unwrap();
        manifest.etag = Some("\"a\"".to_owned());
        manifest.last_modified = Some("Tue, 25 Nov 2025 21:27:36 GMT".to_owned());
        let probe =
            |etag: Option<&str>, last_modified: Option<&str>, status| lex_source::SourceProbe {
                http_status: status,
                etag: etag.map(str::to_owned),
                last_modified: last_modified.map(str::to_owned),
                content_length: None,
            };
        assert_eq!(
            super::upstream_verdict(&manifest, &probe(Some("\"a\""), None, 200)),
            "unchanged"
        );
        assert_eq!(
            super::upstream_verdict(&manifest, &probe(Some("\"b\""), None, 200)),
            "moved"
        );
        manifest.etag = None;
        assert_eq!(
            super::upstream_verdict(
                &manifest,
                &probe(None, Some("Wed, 01 Jul 2026 00:00:00 GMT"), 200)
            ),
            "moved"
        );
        assert_eq!(
            super::upstream_verdict(&manifest, &probe(None, None, 200)),
            "unknown"
        );
        assert_eq!(
            super::upstream_verdict(&manifest, &probe(None, None, 503)),
            "unreachable"
        );
    }

    #[test]
    fn missing_edge_detail_explains_historical_citations() {
        assert_eq!(
            super::missing_edge_detail("lrsic", None, "lraf", 0),
            "lrsic has no resolved edge to lraf"
        );
        let detail = super::missing_edge_detail("lrsic", Some("33"), "lraf", 3);
        assert!(detail.starts_with("lrsic Article 33 has no resolved edge to lraf"));
        assert!(detail.contains("3 superseded historical target(s)"));
    }

    #[test]
    fn consumer_index_and_selection_only_expose_committed_corpora() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path();
        let first = fixture_corpus(
            "urn:lex-mx:federal:statute:first",
            "First Law",
            "FIRST",
            "first text",
        );
        let second = fixture_corpus(
            "urn:lex-mx:federal:statute:second",
            "Second Law",
            "SECOND",
            "second text",
        );
        write_canonical(&first, &root.join("corpus/mx/first")).unwrap();
        write_canonical(&second, &root.join("corpus/mx/second")).unwrap();
        fs::create_dir_all(root.join("corpus/mx/incomplete")).unwrap();
        let standard = root.join("corpus/mx/nom-999-test-2026");
        fs::create_dir_all(&standard).unwrap();
        fs::copy(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../fixtures/standards/numbered-standard-metadata.json"),
            standard.join("standard.json"),
        )
        .unwrap();

        let index = committed_instrument_index(root).unwrap();
        assert_eq!(
            index
                .iter()
                .map(|entry| entry.slug.as_str())
                .collect::<Vec<_>>(),
            vec!["first", "nom-999-test-2026", "second"]
        );

        let selected = selected_corpus_paths(root, &["second".to_owned()]).unwrap();
        assert_eq!(selected, vec![root.join("corpus/mx/second")]);
        let selected_standard =
            selected_corpus_paths(root, &["nom-999-test-2026".to_owned()]).unwrap();
        assert_eq!(selected_standard, vec![standard]);
        assert!(selected_corpus_paths(root, &["missing".to_owned()]).is_err());
        assert!(selected_corpus_paths(root, &["../second".to_owned()]).is_err());
    }

    #[test]
    fn consumer_search_defaults_to_canonical_records() {
        let cli = Cli::try_parse_from(["lex-mex", "search", "control sanitario"]).unwrap();
        assert!(matches!(
            cli.command,
            Command::Search {
                scope: SearchScope::Canonical,
                ..
            }
        ));
    }

    #[test]
    fn audit_furniture_cli_accepts_a_pdf_and_json_output() {
        let cli =
            Cli::try_parse_from(["lex-mex", "audit-furniture", "fixture.pdf", "--json"]).unwrap();
        assert!(matches!(
            cli.command,
            Command::AuditFurniture { pdf, json }
                if pdf == Path::new("fixture.pdf") && json
        ));
    }

    #[test]
    fn global_aliases_include_only_descriptive_ingested_sibling_markers() {
        let table = InstrumentAliasTable {
            schema_version: "0.1.0".to_owned(),
            description: "test aliases".to_owned(),
            aliases: BTreeMap::from([
                (
                    "LOCG".to_owned(),
                    vec![
                        "Ley Orgánica del Congreso General de los Estados Unidos Mexicanos"
                            .to_owned(),
                        "LOCG".to_owned(),
                    ],
                ),
                (
                    "RGIC".to_owned(),
                    vec![
                        "Reglamento para el Gobierno Interior del Congreso General de los Estados Unidos Mexicanos"
                            .to_owned(),
                    ],
                ),
                (
                    "NOT-INGESTED".to_owned(),
                    vec!["Ley todavía no incorporada".to_owned()],
                ),
            ]),
        };
        let instruments = HashMap::from([
            ("LOCG".to_owned(), "urn:test:locg".to_owned()),
            ("RGIC".to_owned(), "urn:test:rgic".to_owned()),
        ]);

        let markers = resolve_global_aliases(&table, "urn:test:rgic", &instruments).unwrap();

        assert_eq!(
            markers,
            vec![(
                "ley orgánica del congreso general de los estados unidos mexicanos".to_owned(),
                "urn:test:locg".to_owned(),
            )]
        );
    }

    #[test]
    fn configured_historical_alias_overrides_same_title_live_alias() {
        let marker = "ley para regular las agrupaciones financieras";
        let merged = merge_external_instruments(
            vec![(marker.to_owned(), "urn:test:lraf-current".to_owned())],
            [(marker.to_owned(), "urn:test:lraf-1990".to_owned())],
        )
        .unwrap();

        assert_eq!(
            merged,
            vec![(marker.to_owned(), "urn:test:lraf-1990".to_owned())]
        );
    }

    #[test]
    fn official_titles_link_only_when_unique_and_descriptive() {
        let repeated = fixture_corpus("urn:test:a", "Ley Repetida", "A", "Texto.");
        let again = fixture_corpus("urn:test:b", "Ley Repetida", "B", "Texto.");
        let unique = fixture_corpus("urn:test:c", "Ley Única", "C", "Texto.");
        let short = fixture_corpus("urn:test:d", "CPEUM", "D", "Texto.");
        let markers = unique_official_title_markers(
            "urn:test:a",
            &[
                &repeated.instrument,
                &again.instrument,
                &unique.instrument,
                &short.instrument,
            ],
        );
        assert_eq!(
            markers,
            vec![("ley única".to_owned(), "urn:test:c".to_owned())]
        );
    }

    #[test]
    fn link_all_does_not_require_an_instrument_slug() {
        let cli = Cli::try_parse_from(["lex-mex", "link", "--all"]).unwrap();
        assert!(matches!(
            cli.command,
            Command::Link {
                all: true,
                instrument: None,
                migrate_protected: false
            }
        ));
        assert!(Cli::try_parse_from(["lex-mex", "link", "--all", "--migrate-protected"]).is_err());
        assert!(Cli::try_parse_from(["lex-mex", "link"]).is_err());
        assert!(Cli::try_parse_from(["lex-mex", "link", "lritf", "--all"]).is_err());
    }

    fn write_title_closure_fixture(root: &Path) {
        let none = serde_json::json!([]);
        let fixtures = [
            (
                "target",
                "urn:lex-mx:test:target",
                "Ley Destino",
                "DESTINO",
                "Texto destino.",
            ),
            (
                "source",
                "urn:lex-mx:test:source",
                "Ley Fuente",
                "FUENTE",
                "En términos del artículo 1 de la Ley Destino.",
            ),
            (
                "broken",
                "urn:lex-mx:test:broken",
                "Ley Rota",
                "ROTA",
                "En términos del artículo 9 de la Ley Destino.",
            ),
            (
                "left",
                "urn:lex-mx:test:left",
                "Ley Repetida",
                "IZQ",
                "Texto.",
            ),
            (
                "right",
                "urn:lex-mx:test:right",
                "Ley Repetida",
                "DER",
                "Texto.",
            ),
            (
                "ask",
                "urn:lex-mx:test:ask",
                "Ley Consulta",
                "CONSULTA",
                "En términos del artículo 1 de la Ley Repetida.",
            ),
        ];
        for (slug, id, title, short_name, text) in fixtures {
            write_fixture_adapter(root, slug, id, title, short_name, &none);
            write_canonical(
                &fixture_corpus(id, title, short_name, text),
                &root.join("corpus/mx").join(slug),
            )
            .unwrap();
        }
    }

    #[test]
    fn corpus_closure_links_unique_official_titles_and_keeps_invalid_relinks() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path();
        write_title_closure_fixture(root);
        let target_id = "urn:lex-mx:test:target";

        let report = run_corpus_closure(root).unwrap();

        assert_eq!(report.status, "failed");
        assert_eq!(report.instruments, 6);
        assert_eq!(report.relinked, 1);
        assert_eq!(report.unchanged, 4);
        assert_eq!(report.rejected.len(), 1);
        assert_eq!(report.rejected[0].slug, "broken");
        assert!(
            report
                .ambiguous_titles
                .iter()
                .any(|title| title.contains("ley repetida"))
        );
        let source = read_corpus(&Paths::new(root, "source", SourceFormat::Pdf)).unwrap();
        assert_eq!(source.references.len(), 1);
        assert_eq!(source.references[0].target_instrument_id, target_id);
        assert_eq!(
            source.references[0].resolution_status,
            ReferenceResolutionStatus::Resolved
        );
        let broken = read_corpus(&Paths::new(root, "broken", SourceFormat::Pdf)).unwrap();
        assert!(broken.references.is_empty());
        let ask = read_corpus(&Paths::new(root, "ask", SourceFormat::Pdf)).unwrap();
        assert!(ask.references.is_empty());
    }

    #[test]
    fn expected_edges_distinguish_recall_failures_from_deferred_targets() {
        let corpora = vec![
            ExpectedEdgeCorpus {
                slug: "lic".to_owned(),
                short_name: "LIC".to_owned(),
                instrument_id: "urn:test:lic".to_owned(),
                references: vec![
                    (
                        "urn:test:lic:article:115".to_owned(),
                        "urn:test:ltosf".to_owned(),
                        ReferenceResolutionStatus::Resolved,
                    ),
                    (
                        "urn:test:lic:article:116".to_owned(),
                        "urn:test:ltosf".to_owned(),
                        ReferenceResolutionStatus::Unresolved,
                    ),
                ],
            },
            ExpectedEdgeCorpus {
                slug: "ltosf".to_owned(),
                short_name: "LTOSF".to_owned(),
                instrument_id: "urn:test:ltosf".to_owned(),
                references: Vec::new(),
            },
            ExpectedEdgeCorpus {
                slug: "lraf".to_owned(),
                short_name: "LRAF".to_owned(),
                instrument_id: "urn:test:lraf".to_owned(),
                references: Vec::new(),
            },
        ];
        let selected = HashSet::from(["lic"]);
        let expectations = vec![
            "LIC -> LTOSF".to_owned(),
            "LIC articulo 115 -> LTOSF".to_owned(),
            "LIC articulo 116 -> LTOSF".to_owned(),
            "LIC -> PLD family".to_owned(),
            "LRAF -> LIC".to_owned(),
            "not an edge".to_owned(),
        ];

        let checks = evaluate_expected_edges(&expectations, &selected, &corpora);

        assert_eq!(
            checks
                .iter()
                .map(|check| check.status.as_str())
                .collect::<Vec<_>>(),
            [
                "satisfied",
                "satisfied",
                "missing",
                "deferred",
                "deferred",
                "invalid",
            ]
        );
    }

    #[test]
    fn additive_to_adopts_a_whole_instrument_or_only_one_provisions_terms() {
        let law_id = "urn:lex-mx:test:statute:lss";
        let mut law = fixture_corpus(
            law_id,
            "Ley del Seguro Social",
            "LSS",
            "Para los efectos de esta Ley, se entiende por:\n\nI. Ley: la Ley del Seguro \
             Social;\n\nII. Instituto: el Instituto Mexicano del Seguro Social;",
        );
        let mut add_on = law.provisions[0].clone();
        add_on.id = format!("{law_id}:article:159");
        add_on.text = "Para efectos de esta Ley, se entenderá por:\n\nI. Pensión, la renta \
                       vitalicia.\n\nII. Renta vitalicia, el contrato."
            .to_owned();
        let mut plain = law.provisions[0].clone();
        plain.id = format!("{law_id}:article:2");
        plain.text = "Esta Ley es de observancia general.".to_owned();
        law.terms = lex_parse::extract_glossaries(&[
            (
                &law.provisions[0],
                lex_parse::GlossaryStyle::AutoWhole,
                None,
            ),
            (&add_on, lex_parse::GlossaryStyle::AutoEstablished, None),
        ])
        .unwrap();
        law.provisions.extend([add_on, plain]);
        let siblings = [("lss", &law)];
        let names = |additive: &str| {
            additive_terms(additive, &siblings)
                .unwrap()
                .into_iter()
                .map(|term| term.term)
                .collect::<Vec<_>>()
        };
        assert_eq!(
            names(law_id),
            ["Ley", "Instituto", "Pensión", "Renta vitalicia"]
        );
        // `reg-lss-macerf` adopts Article 5 A only, not Article 159.
        assert_eq!(names(&format!("{law_id}:article:1")), ["Ley", "Instituto"]);

        for (additive, reason) in [
            (format!("{law_id}:article:9"), "is not a provision of"),
            (format!("{law_id}:article:2"), "defines no terms"),
            (format!("{law_id}-bis"), "not an instrument or provision"),
            (
                "urn:lex-mx:test:statute:lsar:article:1".to_owned(),
                "not an instrument or provision",
            ),
        ] {
            let error = additive_terms(&additive, &siblings)
                .unwrap_err()
                .to_string();
            assert!(error.contains(reason), "{additive}: {error}");
        }
    }

    fn fixture_corpus(id: &str, title: &str, short_name: &str, text: &str) -> Corpus {
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        Corpus {
            enactment_evidence: Vec::new(),
            instrument: Instrument {
                schema_version: SCHEMA_VERSION.to_owned(),
                id: id.to_owned(),
                jurisdiction: "mx".to_owned(),
                level: "federal".to_owned(),
                instrument_type: InstrumentType::Statute,
                official_title: title.to_owned(),
                short_name: short_name.to_owned(),
                operational_source: "fixture".to_owned(),
                formal_publication_source: "dof".to_owned(),
                publication_date: Some(date),
                publication_period: None,
                latest_reform_date: None,
                retrieved_at: Utc::now(),
                source_url: Url::parse("https://example.test/source.pdf").unwrap(),
                source_sha256: "a".repeat(64),
                extracted_text_sha256: "b".repeat(64),
                parser_version: "test".to_owned(),
                status: InstrumentStatus::InForce,
                source_repeal_notice: None,
                issuing_authorities: Vec::new(),
                formal_publication_url: None,
                formal_publication_code: None,
                formal_source_sha256: None,
                formal_extracted_text_sha256: None,
            },
            provisions: vec![Provision {
                numbering_basis: None,
                schema_version: SCHEMA_VERSION.to_owned(),
                id: format!("{id}:article:1"),
                instrument_id: id.to_owned(),
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
                text: text.to_owned(),
                publication_date: Some(date),
                publication_period: None,
                effective_from: None,
                effective_to: None,
                temporal_status: TemporalStatus::Unknown,
                temporal_basis: None,
                temporal_confidence: None,
                review_status: ReviewStatus::NotAnalyzed,
                transitory_effects: Vec::new(),
                amendment_marks: Vec::new(),
                repeals: Vec::new(),
                commencement_condition: None,
            }],
            references: Vec::new(),
            terms: Vec::new(),
            term_usages: Vec::new(),
            amendment_references: Vec::new(),
        }
    }

    fn write_fixture_adapter(
        root: &std::path::Path,
        slug: &str,
        id: &str,
        title: &str,
        short_name: &str,
        external_instruments: &serde_json::Value,
    ) {
        let adapter_dir = root.join("adapters/diputados");
        fs::create_dir_all(&adapter_dir).unwrap();
        let config = serde_json::json!({
            "slug": slug,
            "instrument_id": id,
            "instrument_type": "statute",
            "parser": "diputados",
            "official_title": title,
            "short_name": short_name,
            "operational_source": "camara_de_diputados",
            "source_url": "https://example.test/source.pdf",
            "reference_url": "https://example.test/reference.htm",
            "publisher": "Fixture",
            "publication_date": "2024-01-01",
            "expected_min_articles": 1,
            "expected_articles": 1,
            "expected_transitories": 0,
            "formal_publication_urls": {},
            "external_instruments": external_instruments,
        });
        fs::write(
            adapter_dir.join(format!("{slug}.json")),
            serde_json::to_vec_pretty(&config).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn glossary_coverage_separates_held_detected_and_absent_glossaries() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path();
        let id = "urn:lex-mx:test:glosario";
        write_fixture_adapter(
            root,
            "glosario",
            id,
            "Ley Glosario",
            "GLOSARIO",
            &serde_json::json!([]),
        );
        let context = instrument_context(root, "glosario").unwrap();
        let extract = |text: &str| {
            let corpus = fixture_corpus(id, "Ley Glosario", "GLOSARIO", text);
            let extraction =
                extract_instrument_terms(&context, &corpus.instrument, &corpus.provisions, &[])
                    .unwrap();
            extraction
                .coverage
                .validate(
                    &corpus.instrument,
                    &corpus.provisions,
                    &extraction.terms,
                    &extraction.usages,
                )
                .unwrap();
            extraction.coverage
        };

        let held = extract(
            "Para los efectos de esta Ley, se entiende por:\n\nI. Agencia: La Agencia;\n\nIII. \
             Banco: El Banco.",
        );
        assert_eq!(held.status, lex_core::GlossaryCoverageStatus::Held);
        assert_eq!(held.term_count, 0);
        assert_eq!(
            held.hold.map(|hold| hold.code),
            Some(lex_core::GlossaryHoldCode::SequenceBreak)
        );

        let detected = extract(
            "Para los efectos de esta Ley, se entiende por:\n\nI. Agencia: La Agencia;\n\nII. \
             Derogada.\n\nIII. Banco: El Banco que la Agencia supervisa.",
        );
        assert_eq!(detected.status, lex_core::GlossaryCoverageStatus::Detected);
        assert_eq!(detected.glossaries[0].method, "wider_pass");
        assert_eq!(detected.glossaries[0].fractions, ["I", "III"]);
        assert_eq!(
            detected.glossaries[0].repeal_placeholders,
            Some(vec!["II".to_owned()])
        );
        assert_eq!(detected.term_count, 2);

        let absent = extract("Esta Ley regula la materia.");
        assert_eq!(
            absent.status,
            lex_core::GlossaryCoverageStatus::NoneDetected
        );
        assert!(absent.hold.is_none() && absent.glossaries.is_empty());
    }

    #[test]
    fn coverage_replay_rejects_stale_imports_and_adapter_selection() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path();
        let id = "urn:lex-mx:test:importer";
        let parent_id = "urn:lex-mx:test:parent";
        write_fixture_adapter(
            root,
            "importer",
            id,
            "Ley Fuente",
            "FUENTE",
            &serde_json::json!([]),
        );
        let mut context = instrument_context(root, "importer").unwrap();
        context.config.glossary = Some(
            serde_json::from_value(serde_json::json!({
                "provision_suffix": ":article:1", "style": "auto_whole",
                "additive_to": [format!("{parent_id}:article:1")]
            }))
            .unwrap(),
        );
        let mut parent = fixture_corpus(
            parent_id,
            "Ley Original",
            "ORIGINAL",
            "Para efectos de esta Ley se entiende por: I. Agencia: La entidad; II. Banco: La institución.",
        );
        parent.terms = lex_parse::detect_glossary_terms(&parent.provisions).unwrap();
        let mut corpus = fixture_corpus(
            id,
            "Ley Fuente",
            "FUENTE",
            "Para efectos de esta Ley se entiende por: I. Cuenta: La Agencia administra la Cuenta; II. Fondo: El Banco administra el Fondo.",
        );
        let extraction = extract_instrument_terms(
            &context,
            &corpus.instrument,
            &corpus.provisions,
            &[("parent", &parent)],
        )
        .unwrap();
        corpus.terms = extraction.terms;
        corpus.term_usages = extraction.usages;
        let check = |context: &super::InstrumentContext, parent: &Corpus| {
            super::validate_glossary_coverage_inputs(
                &extraction.coverage,
                context,
                &corpus,
                &[("parent", parent)],
            )
        };
        check(&context, &parent).unwrap();
        let mut changed = parent.clone();
        changed.terms.pop();
        assert!(
            check(&context, &changed)
                .unwrap_err()
                .to_string()
                .contains("imported terms")
        );
        // A rename with unchanged import count must also invalidate usages.
        changed = parent.clone();
        changed.terms[0].term = "Otra Agencia".to_owned();
        assert!(check(&context, &changed).is_err());
        context
            .config
            .glossary
            .as_mut()
            .unwrap()
            .additive_to
            .clear();
        assert!(check(&context, &parent).is_err());
    }

    #[test]
    fn coverage_replay_rejects_fabricated_placeholder_metadata() {
        let temporary = tempfile::tempdir().unwrap();
        let id = "urn:lex-mx:test:glosario";
        write_fixture_adapter(
            temporary.path(),
            "glosario",
            id,
            "Ley Glosario",
            "GLOSARIO",
            &serde_json::json!([]),
        );
        let context = instrument_context(temporary.path(), "glosario").unwrap();
        let mut corpus = fixture_corpus(
            id,
            "Ley Glosario",
            "GLOSARIO",
            "Para efectos de esta Ley se entiende por: I. Agencia: La entidad; II. Derogada. III. Banco: La institución.",
        );
        let mut extraction =
            extract_instrument_terms(&context, &corpus.instrument, &corpus.provisions, &[])
                .unwrap();
        corpus.terms = extraction.terms;
        corpus.term_usages = extraction.usages;
        super::validate_glossary_coverage_inputs(&extraction.coverage, &context, &corpus, &[])
            .unwrap();
        extraction.coverage.glossaries[0].repeal_placeholders = Some(vec!["IV".to_owned()]);
        assert!(
            super::validate_glossary_coverage_inputs(&extraction.coverage, &context, &corpus, &[])
                .is_err()
        );
    }

    #[test]
    fn batch_closure_reverse_links_validates_and_exports_selected_instruments() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path();
        let source_id = "urn:lex-mx:test:source";
        let target_id = "urn:lex-mx:test:target";
        write_fixture_adapter(
            root,
            "source",
            source_id,
            "Ley Fuente",
            "FUENTE",
            &serde_json::json!([{
                "name_marker": "Ley Destino",
                "instrument_id": target_id,
            }]),
        );
        write_fixture_adapter(
            root,
            "target",
            target_id,
            "Ley Destino",
            "DESTINO",
            &serde_json::json!([]),
        );
        write_canonical(
            &fixture_corpus(
                source_id,
                "Ley Fuente",
                "FUENTE",
                "En términos del artículo 1 de la Ley Destino.",
            ),
            &root.join("corpus/mx/source"),
        )
        .unwrap();
        write_canonical(
            &fixture_corpus(target_id, "Ley Destino", "DESTINO", "Texto destino."),
            &root.join("corpus/mx/target"),
        )
        .unwrap();

        let closure = run_batch_closure(
            root,
            &["source".to_owned(), "target".to_owned()],
            &["FUENTE articulo 1 -> DESTINO".to_owned()],
            None,
        )
        .unwrap();

        assert_eq!(closure.status, "ok");
        assert_eq!(closure.expected_edges[0].status, "satisfied");
        let source =
            read_corpus(&Paths::new(root, "source", lex_source::SourceFormat::Pdf)).unwrap();
        assert_eq!(source.references.len(), 1);
        assert_eq!(source.references[0].target_instrument_id, target_id);
        assert!(
            root.join("corpus/mx/source/markdown/articulo-1.md")
                .is_file()
        );

        let skipped = run_batch_closure(
            root,
            &[],
            &["FUENTE articulo 1 -> DESTINO".to_owned()],
            None,
        )
        .unwrap();
        assert_eq!(skipped.status, "skipped");
        assert_eq!(skipped.expected_edges[0].status, "deferred");
    }
    #[test]
    fn batch_closure_relinks_an_instrument_outside_the_batch_whose_usage_the_new_title_covers() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path();
        let glossary_id = "urn:lex-mx:test:glosario";
        let admitted_id = "urn:lex-mx:test:aguas";
        write_fixture_adapter(
            root,
            "glosario",
            glossary_id,
            "Ley Glosario",
            "GLOSARIO",
            &serde_json::json!([]),
        );
        write_canonical(
            &fixture_corpus(
                glossary_id,
                "Ley Glosario",
                "GLOSARIO",
                "Para los efectos de esta Ley, se entiende por:\n\nI. Aguas: Las del subsuelo;\n\n\
                 II. Banco: El Banco que vigila las Aguas conforme a la Ley de Aguas Federales.",
            ),
            &root.join("corpus/mx/glosario"),
        )
        .unwrap();
        let glossary = |root: &Path| {
            read_corpus(&Paths::new(root, "glosario", lex_source::SourceFormat::Pdf)).unwrap()
        };
        run_link(root, &instrument_context(root, "glosario").unwrap()).unwrap();
        assert_eq!(glossary(root).term_usages.len(), 2);

        // Admitting the Ley de Aguas Federales makes the second `Aguas` part
        // of its title, although the glossary is not in the batch.
        write_fixture_adapter(
            root,
            "aguas",
            admitted_id,
            "Ley de Aguas Federales",
            "LAF",
            &serde_json::json!([]),
        );
        write_canonical(
            &fixture_corpus(admitted_id, "Ley de Aguas Federales", "LAF", "Texto."),
            &root.join("corpus/mx/aguas"),
        )
        .unwrap();
        let closure = run_batch_closure(root, &["aguas".to_owned()], &[], None).unwrap();

        assert_eq!(closure.status, "ok");
        assert_eq!(closure.relinked, ["aguas", "glosario"]);
        let usages = glossary(root).term_usages;
        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0].span, "Aguas");
        let context = instrument_context(root, "glosario").unwrap();
        assert!(run_validate(root, &context).unwrap().valid);
        assert!(
            root.join("corpus/mx/glosario/markdown/articulo-1.md")
                .is_file()
        );
        // Closing again changes nothing.
        let before = fs::read(root.join("corpus/mx/glosario/term-usages.json")).unwrap();
        let again = run_batch_closure(root, &["aguas".to_owned()], &[], None).unwrap();
        assert_eq!(again.relinked, ["aguas"]);
        assert_eq!(
            fs::read(root.join("corpus/mx/glosario/term-usages.json")).unwrap(),
            before
        );
    }

    /// A Ley Rota with an accepted gap (its article 9 of the Ley Destino does
    /// not exist) and a glossary whose `Aguas` the admitted Ley de Aguas
    /// Federales' title covers. Returns Rota's persisted references.
    fn write_accepted_gap_fixture(root: &Path) -> serde_json::Value {
        const CITATION: &str = "En términos del artículo 9 de la Ley Destino.";
        let none = serde_json::json!([]);
        write_fixture_adapter(
            root,
            "target",
            "urn:lex-mx:test:target",
            "Ley Destino",
            "DESTINO",
            &none,
        );
        write_canonical(
            &fixture_corpus(
                "urn:lex-mx:test:target",
                "Ley Destino",
                "DESTINO",
                "Texto destino.",
            ),
            &root.join("corpus/mx/target"),
        )
        .unwrap();
        write_fixture_adapter(
            root,
            "rota",
            "urn:lex-mx:test:rota",
            "Ley Rota",
            "ROTA",
            &none,
        );
        write_canonical(
            &fixture_corpus("urn:lex-mx:test:rota", "Ley Rota", "ROTA", CITATION),
            &root.join("corpus/mx/rota"),
        )
        .unwrap();
        let before = run_corpus_closure(root).unwrap();
        assert_eq!(before.rejected.len(), 1);
        fs::write(
            root.join(super::CLOSURE_ACCEPTED_GAPS),
            serde_json::to_vec_pretty(&serde_json::json!({
                "accepted": [{"slug": "rota", "errors": before.rejected[0].errors}],
            }))
            .unwrap(),
        )
        .unwrap();
        // Its glossary follows the citation, so the gap's errors are unchanged;
        // its terms are not persisted yet.
        write_canonical(
            &fixture_corpus(
                "urn:lex-mx:test:rota",
                "Ley Rota",
                "ROTA",
                &format!(
                    "{CITATION}\n\nPara los efectos de esta Ley, se entiende por:\n\n\
                     I. Aguas: Las del subsuelo;\n\nII. Banco: El Banco que vigila las Aguas \
                     conforme a la Ley de Aguas Federales."
                ),
            ),
            &root.join("corpus/mx/rota"),
        )
        .unwrap();
        let references = serde_json::to_value(&rota(root).references).unwrap();

        write_fixture_adapter(
            root,
            "aguas",
            "urn:lex-mx:test:aguas",
            "Ley de Aguas Federales",
            "LAF",
            &none,
        );
        write_canonical(
            &fixture_corpus(
                "urn:lex-mx:test:aguas",
                "Ley de Aguas Federales",
                "LAF",
                "Texto.",
            ),
            &root.join("corpus/mx/aguas"),
        )
        .unwrap();
        references
    }

    fn rota(root: &Path) -> Corpus {
        read_corpus(&Paths::new(root, "rota", lex_source::SourceFormat::Pdf)).unwrap()
    }

    #[test]
    fn batch_closure_republishes_an_accepted_gap_instrument_whose_glossary_alone_it_refreshed() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path();
        let vault = temporary.path().join("vault");
        let references = write_accepted_gap_fixture(root);
        let closure = run_batch_closure(root, &["aguas".to_owned()], &[], Some(&vault)).unwrap();

        assert_eq!(closure.status, "ok");
        assert_eq!(closure.glossary_refreshed, ["rota"]);
        assert!(!closure.relinked.contains(&"rota".to_owned()));
        // Its glossary is refreshed (the `Aguas` in the admitted title is not
        // a usage), its references stay as persisted, and both exports carry it.
        let refreshed = rota(root);
        assert_eq!(refreshed.terms.len(), 2);
        assert_eq!(refreshed.term_usages.len(), 1);
        assert_eq!(
            serde_json::to_value(&refreshed.references).unwrap(),
            references
        );
        assert!(root.join("corpus/mx/rota/markdown/articulo-1.md").is_file());
        let in_vault = walk_files(&vault)
            .iter()
            .any(|path| fs::read_to_string(path).is_ok_and(|text| text.contains("Ley Rota")));
        assert!(in_vault);
        // Closing again changes nothing and refreshes nothing.
        let again = run_batch_closure(root, &["aguas".to_owned()], &[], Some(&vault)).unwrap();
        assert!(again.glossary_refreshed.is_empty());
        assert_eq!(
            serde_json::to_value(&rota(root).term_usages).unwrap(),
            serde_json::to_value(&refreshed.term_usages).unwrap()
        );
    }

    #[test]
    fn batch_closure_rereads_the_alias_catalog_as_a_usage_boundary() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path();
        write_accepted_gap_fixture(root);
        run_batch_closure(root, &["aguas".to_owned()], &[], None).unwrap();
        assert_eq!(rota(root).term_usages.len(), 1);
        // A new alias in the catalog bounds usages too: the closure rereads it.
        fs::write(
            root.join("adapters/diputados/_instrument-aliases.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "schema_version": "0.1.0",
                "description": "fixture",
                "aliases": {"ROTA": ["Banco que vigila las Aguas"]},
            }))
            .unwrap(),
        )
        .unwrap();
        let aliased = run_batch_closure(root, &["aguas".to_owned()], &[], None).unwrap();
        assert_eq!(aliased.glossary_refreshed, ["rota"]);
        assert!(rota(root).term_usages.is_empty());
    }

    fn walk_files(directory: &Path) -> Vec<std::path::PathBuf> {
        let mut files = Vec::new();
        for entry in fs::read_dir(directory).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(walk_files(&path));
            } else {
                files.push(path);
            }
        }
        files
    }

    #[test]
    fn batch_closure_fails_when_the_corpus_closure_rejects_an_unaccepted_instrument() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path();
        write_title_closure_fixture(root);
        let error = run_batch_closure(root, &["target".to_owned()], &[], None).unwrap_err();
        assert!(format!("{error:#}").contains("beyond the accepted gaps"));
    }

    fn fixture_publication_period(hash: &str) -> lex_core::PublicationPeriod {
        lex_core::PublicationPeriod {
            metadata_version: "1.0.0".to_owned(),
            kind: lex_core::PublicationPeriodKind::SerializedRange,
            start: NaiveDate::from_ymd_opt(1912, 1, 1).unwrap(),
            end: NaiveDate::from_ymd_opt(1912, 1, 8).unwrap(),
            source_note: OGA_SERIAL_NOTE.to_owned(),
            source_sha256: hash.to_owned(),
        }
    }

    #[test]
    fn oga_exception_requires_exact_retained_note_range_and_source() {
        let mut config: SourceConfig =
            serde_json::from_str(include_str!("../../../adapters/diputados/lritf.json")).unwrap();
        let manifest: SourceManifest = serde_json::from_str(include_str!(
            "../../../corpus/mx/lritf/source-manifest.json"
        ))
        .unwrap();
        config.slug = "oga".to_owned();
        config.instrument_id = "urn:lex-mx:federal:statute:oga".to_owned();
        config.publication_date = None;
        config.publication_period = Some(fixture_publication_period(&manifest.source_sha256));
        let raw = format!("{OGA_SERIAL_NOTE}\nÚltima reforma publicada DOF 19-12-1985");
        assert_eq!(resolve_publication(&config, &manifest, &raw).unwrap(), None);
        assert!(
            resolve_publication(
                &config,
                &manifest,
                "Última reforma publicada DOF 19-12-1985"
            )
            .is_err()
        );
        config
            .publication_period
            .as_mut()
            .unwrap()
            .source_note
            .push('x');
        assert!(resolve_publication(&config, &manifest, &raw).is_err());
        config.publication_period = Some(fixture_publication_period(&manifest.source_sha256));
        config.publication_period.as_mut().unwrap().start =
            NaiveDate::from_ymd_opt(1912, 1, 9).unwrap();
        assert!(resolve_publication(&config, &manifest, &raw).is_err());
        config.publication_period = Some(fixture_publication_period("wrong"));
        assert!(resolve_publication(&config, &manifest, &raw).is_err());
        config.publication_period = Some(fixture_publication_period(&manifest.source_sha256));
        config.publication_date = Some("1912-01-01".to_owned());
        assert!(resolve_publication(&config, &manifest, &raw).is_err());
    }

    #[test]
    fn range_corpus_loads_but_temporal_request_fails_before_creating_output() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        write_fixture_adapter(
            root,
            "range",
            "urn:test:range",
            "Rango",
            "RANGO",
            &serde_json::json!([]),
        );
        let context = instrument_context(root, "range").unwrap();
        let mut corpus = fixture_corpus("urn:test:range", "Rango", "RANGO", "Texto.");
        let period = fixture_publication_period(&corpus.instrument.source_sha256);
        corpus.instrument.publication_date = None;
        corpus.instrument.publication_period = Some(period.clone());
        for p in &mut corpus.provisions {
            p.publication_date = None;
            p.publication_period = Some(period.clone());
        }
        write_canonical(&corpus, &context.paths.corpus).unwrap();
        assert!(
            read_corpus(&context.paths)
                .unwrap()
                .instrument
                .publication_date
                .is_none()
        );
        let error = run_temporal_request(&context).unwrap_err().to_string();
        assert!(error.contains("require an exact publication day"));
        assert!(!context.paths.temporal_request.exists());
    }
    #[test]
    fn ordinary_reruns_cannot_clear_or_replace_persisted_intake_approvals() {
        let temp = tempfile::tempdir().unwrap();
        let config = load_config(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../adapters/diputados/lritf.json"),
        )
        .unwrap();
        let mut context = InstrumentContext {
            config,
            paths: Paths::new(temp.path(), "lritf", SourceFormat::Pdf),
            root: temp.path().to_path_buf(),
        };
        let mut corpus = fixture_corpus(&context.config.instrument_id, "Fixture", "LRITF", "Texto");
        let approval = lex_core::OutdatedReferenceApproval {
            source_provision_id: corpus.provisions[0].id.clone(),
            target_instrument_id: "urn:lex-mx:federal:statute:lmv".to_owned(),
            target_provision_id: "urn:lex-mx:federal:statute:lmv:article:16-bis-7".to_owned(),
            source_span: "Texto".to_owned(),
            start_char: 0,
            end_char: 5,
            source_text_sha256: lex_core::evidence_sha256("Texto"),
            source_sha256: corpus.instrument.source_sha256.clone(),
            reviewer: "Fixture reviewer".to_owned(),
            captured_at: "2026-10-16T00:00:00Z".to_owned(),
            reviewer_supplied_at: None,
            rationale: "Fixture only".to_owned(),
            decision_source: "fixture".to_owned(),
            decision_source_sha256: "c".repeat(64),
        };
        corpus.references.push(lex_core::ReferenceEdge {
            schema_version: SCHEMA_VERSION.to_owned(),
            id: "fixture-edge".to_owned(),
            source_provision_id: approval.source_provision_id.clone(),
            source_span: approval.source_span.clone(),
            start_char: 0,
            end_char: 5,
            target_instrument_id: approval.target_instrument_id.clone(),
            target_provision_id: approval.target_provision_id.clone(),
            qualifiers: vec![],
            basis: lex_core::Basis::ExpressCrossReference,
            confidence: 1.0,
            resolution_status: ReferenceResolutionStatus::HistoricalSupersededReferenceOutdated,
            reference_form: lex_core::ReferenceForm::Direct,
            outdated_reference_approval: Some(approval.clone()),
            legal_operator_review: None,
            resolution_rule: None,
        });
        write_canonical(&corpus, &context.paths.corpus).unwrap();
        assert!(preserve_intake_decisions(&context).is_err());
        context
            .config
            .outdated_reference_approvals
            .push(approval.clone());
        assert!(preserve_intake_decisions(&context).is_ok());
        context.config.outdated_reference_approvals[0]
            .rationale
            .push_str(" changed");
        assert!(preserve_intake_decisions(&context).is_err());
        context.config.outdated_reference_approvals[0] = approval;
        let notice = lex_core::SourceRepealNotice {
            source_note: "Ley Abrogada DOF 16-07-2025".to_owned(),
            source_sha256: corpus.instrument.source_sha256.clone(),
            formal_publication_url: "https://www.dof.gob.mx/nota_detalle.php?codigo=fixture"
                .parse()
                .unwrap(),
            formal_source_sha256: "b".repeat(64),
            successor_instrument_id: None,
            successor_clause: None,
            reviewer: "Fixture reviewer".to_owned(),
            captured_at: "2026-10-16T00:00:00Z".to_owned(),
            reviewer_supplied_at: None,
            rationale: "Fixture only".to_owned(),
            decision_source: "fixture".to_owned(),
            decision_source_sha256: "c".repeat(64),
        };
        corpus.instrument.status = InstrumentStatus::Repealed;
        corpus.instrument.source_repeal_notice = Some(notice.clone());
        write_canonical(&corpus, &context.paths.corpus).unwrap();
        assert!(preserve_intake_decisions(&context).is_err());
        context.config.source_repeal_notice = Some(notice);
        assert!(preserve_intake_decisions(&context).is_ok());
    }

    #[test]
    fn constitutions_cite_themselves_by_constitution_wording() {
        let markers = self_reference_markers("constitution");
        for marker in [
            "de esta constitución",
            "esta constitución",
            " constitucional",
        ] {
            assert!(markers.iter().any(|m| m == marker), "{marker}");
        }
        assert!(
            !self_reference_markers("statute")
                .iter()
                .any(|m| m.contains("constituci"))
        );
    }

    fn sample_closure_instrument() -> lex_core::Instrument {
        serde_json::from_value(serde_json::json!({
            "schema_version": SCHEMA_VERSION,
            "id": "urn:lex-mx:federal:statute:fixture",
            "jurisdiction": "mx",
            "level": "federal",
            "instrument_type": "statute",
            "official_title": "Ley de prueba",
            "short_name": "FIXTURE",
            "operational_source": "camara_de_diputados",
            "formal_publication_source": "dof",
            "publication_date": "2002-01-15",
            "latest_reform_date": null,
            "retrieved_at": "2026-10-16T00:00:00Z",
            "source_url": "https://www.diputados.gob.mx/LeyesBiblio/pdf/FIXTURE.pdf",
            "source_sha256": "e".repeat(64),
            "extracted_text_sha256": "f".repeat(64),
            "parser_version": "0.1.0",
            "status": "in_force"
        }))
        .unwrap()
    }

    #[test]
    fn closure_exits_clean_only_when_rejections_equal_the_accepted_gaps() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path();
        let report = |rejected: Vec<(&str, &str)>| super::CorpusClosureReport {
            status: "failed".to_owned(),
            instruments: 2,
            relinked: 0,
            relinked_instruments: vec![],
            glossary_refreshed_instruments: vec![],
            unchanged: 0,
            references_before: 0,
            references_after: 0,
            resolved_cross_instrument: 0,
            ambiguous_titles: vec![],
            rejected: rejected
                .into_iter()
                .map(|(slug, error)| super::CorpusClosureRejection {
                    slug: slug.to_owned(),
                    errors: vec![error.to_owned()],
                })
                .collect(),
        };
        let current = report(vec![("one", "missing a"), ("two", "missing b")]);
        // No recorded list accepts nothing.
        assert!(!super::closure_matches_accepted_gaps(root, &current).unwrap());
        assert!(super::closure_matches_accepted_gaps(root, &report(vec![])).unwrap());
        fs::create_dir_all(root.join("adapters/diputados")).unwrap();
        fs::write(
            root.join(super::CLOSURE_ACCEPTED_GAPS),
            r#"{"accepted":[{"slug":"one","errors":["missing a"]},{"slug":"two","errors":["missing b"]}]}"#,
        )
        .unwrap();
        assert!(super::closure_matches_accepted_gaps(root, &current).unwrap());
        // A new rejection, a changed error and a resolved gap each fail.
        for changed in [
            report(vec![
                ("one", "missing a"),
                ("two", "missing b"),
                ("three", "x"),
            ]),
            report(vec![("one", "missing a"), ("two", "missing other")]),
            report(vec![("one", "missing a")]),
            report(vec![]),
        ] {
            assert!(!super::closure_matches_accepted_gaps(root, &changed).unwrap());
        }
    }

    #[test]
    fn closure_guard_keeps_persisted_historical_and_approved_edges() {
        let edge = |status: ReferenceResolutionStatus, target: &str| lex_core::ReferenceEdge {
            schema_version: SCHEMA_VERSION.to_owned(),
            id: "fixture:reference:1:article:12".to_owned(),
            source_provision_id: "fixture:article:2".to_owned(),
            source_span: "12".to_owned(),
            start_char: 1,
            end_char: 3,
            target_instrument_id: target.to_owned(),
            target_provision_id: format!("{target}:article:12"),
            qualifiers: vec![],
            basis: lex_core::Basis::ExpressCrossReference,
            confidence: 1.0,
            resolution_status: status,
            reference_form: lex_core::ReferenceForm::Direct,
            outdated_reference_approval: None,
            legal_operator_review: None,
            resolution_rule: None,
        };
        let corpus_with = |references: Vec<lex_core::ReferenceEdge>| Corpus {
            instrument: sample_closure_instrument(),
            provisions: vec![],
            references,
            terms: vec![],
            term_usages: vec![],
            amendment_references: vec![],
            enactment_evidence: vec![],
        };
        let historical = edge(
            ReferenceResolutionStatus::HistoricalTargetUnavailable,
            "urn:lex-mx:federal:statute:lraf-1990",
        );
        let graph = |references: Vec<lex_core::ReferenceEdge>| LinkedGraph {
            references,
            terms: vec![],
            usages: vec![],
            coverage: None,
        };
        let corpus = corpus_with(vec![historical.clone()]);
        let found = protected_edge_drift(&corpus, &graph(vec![historical.clone()]));
        assert!(found.is_empty(), "{found:?}");
        // Recording a resolution rule on a protected edge that predates the
        // field is allowed; changing an already recorded rule is not.
        let mut with_rule = historical.clone();
        with_rule.resolution_rule = Some(lex_core::ResolutionRule::NamedInstrument);
        let found = protected_edge_drift(&corpus, &graph(vec![with_rule.clone()]));
        assert!(found.is_empty(), "{found:?}");
        let mut other_rule = with_rule.clone();
        other_rule.resolution_rule = Some(lex_core::ResolutionRule::ParentShorthand);
        assert_eq!(
            protected_edge_drift(&corpus_with(vec![with_rule]), &graph(vec![other_rule])).len(),
            1
        );
        let retargeted = edge(
            ReferenceResolutionStatus::Resolved,
            "urn:lex-mx:federal:statute:lraf",
        );
        assert_eq!(
            protected_edge_drift(&corpus, &graph(vec![retargeted])).len(),
            1
        );
        assert_eq!(protected_edge_drift(&corpus, &graph(vec![])).len(), 1);
        // Ordinary resolved edges may change freely.
        let ordinary = corpus_with(vec![edge(
            ReferenceResolutionStatus::Resolved,
            "urn:lex-mx:federal:statute:lraf",
        )]);
        let found = protected_edge_drift(&ordinary, &graph(vec![]));
        assert!(found.is_empty(), "{found:?}");
    }

    #[test]
    fn stale_or_missing_human_review_evidence_stops_before_canonical_writes() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        write_fixture_adapter(
            root,
            "reviewed",
            "urn:test:reviewed",
            "Fixture",
            "FIXTURE",
            &serde_json::json!([]),
        );
        let context = instrument_context(root, "reviewed").unwrap();
        let mut original = fixture_corpus(
            "urn:test:reviewed",
            "Fixture",
            "FIXTURE",
            "Esta Ley entrará en vigor al día siguiente de su publicación.",
        );
        let p = &mut original.provisions[0];
        p.id = "urn:test:reviewed:transitory:primero".to_owned();
        p.provision_type = ProvisionType::Transitory;
        p.label = "PRIMERO".to_owned();
        p.number = "PRIMERO".to_owned();
        p.review_status = ReviewStatus::LawyerVerified;
        p.temporal_basis = Some(lex_core::Basis::LawyerVerified);
        p.temporal_status = TemporalStatus::Effective;
        p.effective_from = Some(NaiveDate::from_ymd_opt(2024, 1, 2).unwrap());
        p.temporal_confidence = Some(1.0);
        let determination = lex_core::TemporalDetermination {
            provision_id: p.id.clone(),
            temporal_status: p.temporal_status.clone(),
            publication_date: p.publication_date.unwrap(),
            effective_from: p.effective_from,
            effective_to: None,
            confidence: 1.0,
            basis: lex_core::Basis::LawyerVerified,
            supporting_text: vec![p.text.clone()],
            review_required: false,
            review_reason: None,
            model: "fixture".to_owned(),
            prompt_version: "fixture".to_owned(),
            effects: vec![],
            evidence_sha256: lex_core::evidence_sha256(&p.text),
        };
        let result = lex_core::TemporalAnalysisResult {
            schema_version: SCHEMA_VERSION.to_owned(),
            instrument_id: original.instrument.id.clone(),
            request_sha256: "a".repeat(64),
            response_sha256: "b".repeat(64),
            response_id: None,
            model: "fixture".to_owned(),
            prompt_version: "fixture".to_owned(),
            analyzed_at: Utc::now(),
            determinations: vec![determination],
        };
        write_canonical(&original, &context.paths.corpus).unwrap();
        super::write_pretty_json(&result, &context.paths.temporal_result).unwrap();
        let canonical_before = fs::read(&context.paths.provisions).unwrap();
        let sidecar_before = fs::read(&context.paths.temporal_result).unwrap();
        let reapply = |parsed: &mut Corpus| {
            super::reapply_persisted_temporal_state(&context.paths, &context.config, parsed, &[])
        };
        let mut parsed = original.clone();
        parsed.provisions[0].review_status = ReviewStatus::NotAnalyzed;
        parsed.provisions[0].temporal_basis = None;
        parsed.provisions[0].temporal_status = TemporalStatus::Unknown;
        parsed.provisions[0].effective_from = None;
        parsed.provisions[0].temporal_confidence = None;
        assert!(reapply(&mut parsed).is_ok());
        assert_eq!(
            serde_json::to_value(&parsed.provisions).unwrap(),
            serde_json::to_value(&original.provisions).unwrap()
        );

        parsed.provisions[0].text.push_str(" Cambio de evidencia.");
        assert!(reapply(&mut parsed).is_err());
        parsed.provisions.clear();
        assert!(reapply(&mut parsed).is_err());
        assert_eq!(
            fs::read(&context.paths.temporal_result).unwrap(),
            sidecar_before
        );
        assert_eq!(
            fs::read(&context.paths.provisions).unwrap(),
            canonical_before
        );

        let mut parsed = original.clone();
        parsed.provisions[0].review_status = ReviewStatus::NotAnalyzed;
        let mut incomplete = result;
        incomplete.determinations.clear();
        super::write_pretty_json(&incomplete, &context.paths.temporal_result).unwrap();
        assert!(reapply(&mut parsed).is_err());
        fs::remove_file(&context.paths.temporal_result).unwrap();
        assert!(reapply(&mut parsed).is_err());
        assert_eq!(
            fs::read(&context.paths.provisions).unwrap(),
            canonical_before
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn successor_evidence_drift_blocks_validate_link_and_all_exports() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let own_id = "urn:lex-mx:federal:statute:former";
        let successor_id = "urn:lex-mx:federal:statute:successor";
        write_fixture_adapter(
            root,
            "former",
            own_id,
            "Former",
            "FORMER",
            &serde_json::json!([]),
        );
        let mut context = instrument_context(root, "former").unwrap();
        let mut former = fixture_corpus(own_id, "Former", "FORMER", "Texto anterior.");
        let successor = fixture_corpus(
            successor_id,
            "Successor",
            "SUCCESSOR",
            "Se abroga la Ley anterior.",
        );
        let notice = lex_core::SourceRepealNotice {
            source_note: successor.provisions[0].text.clone(),
            source_sha256: former.instrument.source_sha256.clone(),
            formal_publication_url: "https://www.dof.gob.mx/nota_detalle.php?codigo=fixture"
                .parse()
                .unwrap(),
            formal_source_sha256: "b".repeat(64),
            successor_instrument_id: Some(successor_id.to_owned()),
            successor_clause: Some(lex_core::SuccessorAbrogationClause {
                provision_id: successor.provisions[0].id.clone(),
                provision_text_sha256: lex_core::evidence_sha256(&successor.provisions[0].text),
            }),
            reviewer: "Fixture reviewer".to_owned(),
            captured_at: "2026-10-16T00:00:00Z".to_owned(),
            reviewer_supplied_at: None,
            rationale: "Fixture only".to_owned(),
            decision_source: "fixture".to_owned(),
            decision_source_sha256: "c".repeat(64),
        };
        former.instrument.status = InstrumentStatus::Repealed;
        former.instrument.source_repeal_notice = Some(notice.clone());
        context.config.source_repeal_notice = Some(notice);
        write_canonical(&former, &context.paths.corpus).unwrap();
        write_fixture_adapter(
            root,
            "successor",
            successor_id,
            "Successor",
            "SUCCESSOR",
            &serde_json::json!([]),
        );
        let successor_dir = root.join("corpus/mx/successor");
        write_canonical(&successor, &successor_dir).unwrap();
        assert!(super::run_validate(root, &context).unwrap().valid);
        super::run_link(root, &context).unwrap();
        // A single-instrument relink refuses to change a persisted protected
        // edge before writing anything; the audited override allows it.
        let mut protected = read_corpus(&context.paths).unwrap();
        let original_references = fs::read(&context.paths.references).unwrap();
        protected.references.push(lex_core::ReferenceEdge {
            schema_version: SCHEMA_VERSION.to_owned(),
            id: "fixture-protected-edge".to_owned(),
            source_provision_id: protected.provisions[0].id.clone(),
            source_span: "Texto".to_owned(),
            start_char: 0,
            end_char: 5,
            target_instrument_id: successor_id.to_owned(),
            target_provision_id: format!("{successor_id}:article:1"),
            qualifiers: vec![],
            basis: lex_core::Basis::ExpressCrossReference,
            confidence: 1.0,
            resolution_status: ReferenceResolutionStatus::HistoricalTargetUnavailable,
            reference_form: lex_core::ReferenceForm::Direct,
            outdated_reference_approval: None,
            legal_operator_review: None,
            resolution_rule: None,
        });
        write_canonical(&protected, &context.paths.corpus).unwrap();
        let persisted = fs::read(&context.paths.references).unwrap();
        assert!(super::run_link(root, &context).is_err());
        assert_eq!(fs::read(&context.paths.references).unwrap(), persisted);
        super::run_link_with(root, &context, true).unwrap();
        assert_eq!(
            fs::read(&context.paths.references).unwrap(),
            original_references
        );
        super::run_export(root, &context, super::ExportFormat::Markdown, None).unwrap();
        let tracked = [
            &context.paths.instrument,
            &context.paths.provisions,
            &context.paths.references,
            &context.paths.corpus.join("validation.json"),
            &context.paths.markdown.join("README.md"),
        ];
        let before: Vec<_> = tracked.iter().map(|p| fs::read(p).unwrap()).collect();
        for variant in 0..3 {
            let mut changed = successor.clone();
            match variant {
                0 => changed.provisions[0].text = "Texto sin abrogación.".to_owned(),
                1 => changed.provisions.clear(),
                _ => changed.provisions[0].instrument_id = own_id.to_owned(),
            }
            write_canonical(&changed, &successor_dir).unwrap();
            assert!(super::run_validate(root, &context).is_err());
            assert!(super::run_link(root, &context).is_err());
            assert!(super::run_corpus_closure(root).is_err());
            for format in [
                super::ExportFormat::Json,
                super::ExportFormat::Markdown,
                super::ExportFormat::Obsidian,
            ] {
                let vault = root.join("vault");
                assert!(super::run_export(root, &context, format, Some(&vault)).is_err());
                assert!(!vault.exists());
            }
            for (p, bytes) in tracked.iter().zip(&before) {
                assert_eq!(&fs::read(p).unwrap(), bytes);
            }
        }
        fs::remove_dir_all(&successor_dir).unwrap();
        assert!(super::run_validate(root, &context).is_err());
        assert!(super::run_export(root, &context, super::ExportFormat::Markdown, None).is_err());
    }
}
