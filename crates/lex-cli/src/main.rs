use std::{
    collections::{HashMap, HashSet},
    fs,
    io::Write as _,
    path::{Path, PathBuf},
    process::{Command as ProcessCommand, Stdio},
};

use anyhow::{Context, Result, bail};
use chrono::{NaiveDate, Utc};
use clap::{Parser, Subcommand, ValueEnum};
use lex_core::{
    Corpus, Instrument, InstrumentStatus, InstrumentType, ProvisionType, ReviewItem,
    ReviewItemStatus, ReviewResolution, SCHEMA_VERSION, SourceManifest, TemporalAnalysisMetadata,
    TemporalAnalysisRequest, TemporalAnalysisResult, TemporalEvidence, TemporalModelBatch,
    TemporalReviewResolution, TemporalStatus, TransitoryEffect, apply_temporal_determinations,
    open_temporal_review, preserve_temporal_review_history, reapply_temporal_determinations,
    resolve_temporal_review, route_temporal_analysis,
};
use lex_export::{
    LinkTargets, TermTargets, link_targets, term_targets, write_canonical, write_markdown,
    write_obsidian, write_validation,
};
use lex_parse::{
    CorpusExpectations, CorpusView, GlossaryStyle, InstrumentContextPolicy, ReferenceOptions,
    extract_html_text, extract_internal_references, extract_pdf, extract_references,
    extract_reform_transitories, extract_term_usages, extract_terms, find_glossary_provision,
    parse_dcg, parse_itf_dcg, parse_lritf, validate_corpus,
};
use lex_source::{
    SourceConfig, discover, fetch, fetch_annex, fetch_formal, load_config, sha256_hex,
    write_acquisition, write_manifest,
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
    Link {
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
        #[arg(long, default_value = "lritf")]
        instrument: String,
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

    match cli.command {
        Command::Discover { source } => {
            let configs = discover_source_configs(&root, &source)?;
            for config in &configs {
                println!("{}", serde_json::to_string_pretty(&discover(config))?);
            }
        }
        Command::Fetch { instrument } => {
            let context = instrument_context(&root, &instrument)?;
            run_fetch(&context)?;
        }
        Command::Extract { instrument } => {
            let context = instrument_context(&root, &instrument)?;
            run_extract(&context)?;
        }
        Command::Parse { instrument } => {
            let context = instrument_context(&root, &instrument)?;
            run_parse(&root, &context)?;
        }
        Command::Link { instrument } => {
            let context = instrument_context(&root, &instrument)?;
            run_link(&root, &context)?;
        }
        Command::AnalyzeTemporal {
            instrument,
            provider,
            model,
        } => {
            let context = instrument_context(&root, &instrument)?;
            run_temporal_request(&context)?;
            if provider == TemporalProvider::Codex {
                run_codex_temporal(&root, &context, &model)?;
                run_export(&root, &context, ExportFormat::Markdown, None)?;
                if let Some(vault) = &obsidian_vault {
                    run_export(&root, &context, ExportFormat::Obsidian, Some(vault))?;
                }
            }
        }
        Command::ImportTemporal {
            instrument,
            response,
            model,
            response_id,
        } => {
            let context = instrument_context(&root, &instrument)?;
            run_temporal_import(&context, &response, &model, response_id)?;
            run_export(&root, &context, ExportFormat::Markdown, None)?;
            if let Some(vault) = &obsidian_vault {
                run_export(&root, &context, ExportFormat::Obsidian, Some(vault))?;
            }
        }
        Command::Validate { instrument } => {
            let context = instrument_context(&root, &instrument)?;
            let report = run_validate(&root, &context)?;
            if !report.valid {
                bail!(
                    "corpus validation failed; inspect {}",
                    context.paths.corpus.join("validation.json").display()
                );
            }
        }
        Command::Export { instrument, format } => {
            let context = instrument_context(&root, &instrument)?;
            run_export(&root, &context, format, obsidian_vault.as_deref())?;
        }
        Command::Pipeline {
            instrument,
            keep_work,
            temporal_provider,
            temporal_model,
        } => {
            let context = instrument_context(&root, &instrument)?;
            run_pipeline(
                &root,
                &context,
                obsidian_vault.as_deref(),
                keep_work,
                temporal_provider,
                &temporal_model,
            )?;
        }
        Command::Review {
            instrument,
            command,
        } => {
            let context = instrument_context(&root, &instrument)?;
            run_review_command(&root, &context, obsidian_vault.as_deref(), command)?;
        }
    }
    Ok(())
}

/// One instrument's adapter configuration and working paths.
struct InstrumentContext {
    config: SourceConfig,
    paths: Paths,
}

fn instrument_context(root: &Path, slug: &str) -> Result<InstrumentContext> {
    let config = find_adapter(root, slug)?;
    let paths = Paths::new(root, slug);
    Ok(InstrumentContext { config, paths })
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
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
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
            if path
                .extension()
                .is_some_and(|extension| extension == "json")
            {
                paths.push(path);
            }
        }
    }
    paths.sort();
    Ok(paths)
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
    write_acquisition(&acquisition, &context.paths.pdf, &context.paths.manifest)?;
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
    let keep_page_breaks = context.config.parser == "ifpe-dcg";
    let extraction = extract_pdf(&paths.pdf, &paths.text, keep_page_breaks)?;
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

/// Dispatch to the adapter-configured parser.
fn parse_by_configured_parser(
    paths: &Paths,
    config: &SourceConfig,
    raw: &str,
    publication_date: NaiveDate,
) -> Result<ParsedInstrument> {
    match config.parser.as_str() {
        "lritf" => Ok(ParsedInstrument {
            provisions: parse_lritf(raw, publication_date)?,
            formal_manifest: None,
            amendment_references: Vec::new(),
            reform_evidence: Vec::new(),
            latest_reform_date: None,
        }),
        "ifpe-dcg" => {
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
                reform_evidence: Vec::new(),
                latest_reform_date: None,
            })
        }
        "itf-dcg" => {
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
                reform_evidence: document.reform_evidence,
                latest_reform_date: document.latest_reform_date,
            })
        }
        other => bail!("unsupported parser {other:?} in adapter configuration"),
    }
}

fn run_parse(root: &Path, context: &InstrumentContext) -> Result<()> {
    let paths = &context.paths;
    let config = &context.config;
    let manifest: SourceManifest = read_json(&paths.manifest)?;
    let raw = fs::read_to_string(&paths.text)
        .with_context(|| format!("failed to read {}", paths.text.display()))?;
    let publication_date = NaiveDate::parse_from_str(&config.publication_date, "%Y-%m-%d")?;
    let extracted_text_sha256 = manifest
        .extracted_text_sha256
        .clone()
        .context("manifest lacks extracted text hash; run extract first")?;

    let parsed = parse_by_configured_parser(paths, config, &raw, publication_date)?;
    let (provisions, formal_manifest) = (parsed.provisions, parsed.formal_manifest);

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
        latest_reform_date: parsed
            .latest_reform_date
            .or_else(|| latest_reform_date(&raw)),
        retrieved_at: manifest.retrieved_at,
        source_url: manifest.official_url,
        source_sha256: manifest.source_sha256,
        extracted_text_sha256,
        parser_version: env!("CARGO_PKG_VERSION").to_owned(),
        status: InstrumentStatus::InForce,
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
    let references = extract_instrument_references(root, context, &instrument, &provisions)?;
    let (terms, term_usages) = extract_instrument_terms(root, context, &instrument, &provisions)?;
    // Reform evidence must be computed before reapplication: an
    // amendment-event determination's provision ID lives only in reform
    // evidence, never among canonical provisions, and reapplication needs
    // the freshly reparsed evidence, not whatever was last written to disk.
    let reform_evidence = if config.parser == "lritf" {
        extract_reform_transitories(&raw)?
    } else {
        parsed.reform_evidence
    };
    let mut corpus = Corpus {
        instrument,
        provisions,
        references,
        terms,
        term_usages,
        amendment_references: parsed.amendment_references,
    };
    reapply_persisted_temporal_state(paths, config, &mut corpus, &reform_evidence)?;
    write_canonical(&corpus, &paths.corpus)?;
    println!("parsed {} canonical provisions", corpus.provisions.len());
    println!("extracted {} canonical references", corpus.references.len());
    println!(
        "extracted {} defined terms with {} usages",
        corpus.terms.len(),
        corpus.term_usages.len()
    );
    if !reform_evidence.is_empty() {
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
        return Ok(());
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

/// Extract the instrument's glossary terms and every exact term usage.
/// A glossary that is expressly additive to another instrument's glossary
/// (the DCG's Article 1 opens "además de los términos utilizados en la
/// Ley…") resolves usages against its own terms first, then the terms of
/// each instrument listed in `additive_to`, in order.
fn extract_instrument_terms(
    root: &Path,
    context: &InstrumentContext,
    instrument: &Instrument,
    provisions: &[lex_core::Provision],
) -> Result<(Vec<lex_core::DefinedTerm>, Vec<lex_core::TermUsage>)> {
    let Some(glossary) = &context.config.glossary else {
        return Ok((Vec::new(), Vec::new()));
    };
    let style = GlossaryStyle::from_config(&glossary.style)?;
    let glossary_provision = find_glossary_provision(provisions, &glossary.provision_suffix)?;
    let terms = extract_terms(glossary_provision, style)?;

    let siblings = read_sibling_corpora(root, &instrument.id)?;
    let mut term_sets: Vec<&[lex_core::DefinedTerm]> = vec![&terms];
    for additive_instrument in &glossary.additive_to {
        let sibling = siblings
            .iter()
            .find(|(_, corpus)| &corpus.instrument.id == additive_instrument)
            .with_context(|| {
                format!("glossary is additive to {additive_instrument}, which is not in the corpus")
            })?;
        term_sets.push(&sibling.1.terms);
    }
    let usages = extract_term_usages(provisions, &term_sets)?;
    Ok((terms, usages))
}

/// Extract this instrument's reference edges, resolving targets against
/// every instrument committed under `corpus/mx/`.
fn extract_instrument_references(
    root: &Path,
    context: &InstrumentContext,
    instrument: &Instrument,
    provisions: &[lex_core::Provision],
) -> Result<Vec<lex_core::ReferenceEdge>> {
    if context.config.parser == "lritf" {
        return extract_internal_references(provisions);
    }
    let mut known_targets: HashSet<String> =
        provisions.iter().map(|item| item.id.clone()).collect();
    for (_, corpus) in read_sibling_corpora(root, &instrument.id)? {
        known_targets.extend(corpus.provisions.iter().map(|item| item.id.clone()));
    }
    let options = ReferenceOptions {
        policy: InstrumentContextPolicy::SentenceEarliestMarker {
            internal_markers: context
                .config
                .internal_reference_markers
                .clone()
                .unwrap_or_default(),
            external_instruments: context
                .config
                .external_instruments
                .iter()
                .map(|external| (external.name_marker.clone(), external.instrument_id.clone()))
                .collect(),
        },
        transitory_citations: true,
        same_article_fractions: true,
        relative_references: true,
    };
    extract_references(
        provisions,
        Some((instrument.id.as_str(), instrument.official_title.as_str())),
        &options,
        &known_targets,
    )
}

fn run_link(root: &Path, context: &InstrumentContext) -> Result<()> {
    let paths = &context.paths;
    let mut corpus = read_corpus(paths)?;
    corpus.references =
        extract_instrument_references(root, context, &corpus.instrument, &corpus.provisions)?;
    let (terms, term_usages) =
        extract_instrument_terms(root, context, &corpus.instrument, &corpus.provisions)?;
    corpus.terms = terms;
    corpus.term_usages = term_usages;
    write_canonical(&corpus, &paths.corpus)?;
    println!("extracted {} canonical references", corpus.references.len());
    println!(
        "extracted {} defined terms with {} usages",
        corpus.terms.len(),
        corpus.term_usages.len()
    );
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
        publication_date: corpus.instrument.publication_date,
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
    resolve_temporal_review(item, &mut result.determinations, resolution)?;

    write_pretty_json(&result, &paths.temporal_result)?;
    write_pretty_json(&items, &paths.review_queue)?;
    let mut corpus = read_corpus(paths)?;
    apply_temporal_determinations(&mut corpus.provisions, &result.determinations);
    write_canonical(&corpus, &paths.corpus)?;
    println!("resolved {review_id}");
    Ok(())
}

fn run_validate(root: &Path, context: &InstrumentContext) -> Result<lex_core::ValidationReport> {
    let paths = &context.paths;
    let config = &context.config;
    let corpus = read_corpus(paths)?;
    let mut external_targets = HashSet::new();
    let mut external_terms = HashSet::new();
    for (_, sibling) in read_sibling_corpora(root, &corpus.instrument.id)? {
        external_targets.extend(sibling.provisions.iter().map(|item| item.id.clone()));
        external_terms.extend(sibling.terms.iter().map(|term| term.id.clone()));
    }
    let report = validate_corpus(
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
        },
        &external_targets,
        &external_terms,
    );
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

/// Read every committed corpus except `own_instrument_id`, keyed by its
/// corpus directory slug.
fn read_sibling_corpora(root: &Path, own_instrument_id: &str) -> Result<Vec<(String, Corpus)>> {
    let corpus_root = root.join("corpus/mx");
    let mut siblings = Vec::new();
    if !corpus_root.is_dir() {
        return Ok(siblings);
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
        let paths = Paths::new(root, &slug);
        let corpus = read_corpus(&paths)?;
        if corpus.instrument.id != own_instrument_id {
            siblings.push((slug, corpus));
        }
    }
    Ok(siblings)
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
    Ok(Corpus {
        instrument: read_json(&paths.instrument)?,
        provisions: read_json(&paths.provisions)?,
        references: optional(&paths.references)?,
        terms: optional(&paths.terms)?,
        term_usages: optional(&paths.term_usages)?,
        amendment_references: optional(&paths.amendment_references)?,
    })
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
        .captures(raw)
        .and_then(|captures| NaiveDate::parse_from_str(&captures[1], "%d-%m-%Y").ok())
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
    work: PathBuf,
    pdf: PathBuf,
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
    fn new(root: &Path, slug: &str) -> Self {
        let work = root.join(".work").join(slug);
        let corpus = root.join("corpus/mx").join(slug);
        Self {
            pdf: work.join(format!("{slug}.pdf")),
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
