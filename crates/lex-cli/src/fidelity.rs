//! `verify-fidelity`: read-only source-fidelity evidence for committed
//! instruments. Reports go to `.work/`; nothing under `corpus/` is written and
//! no review state is touched.
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use clap::Args;
use lex_core::{Provision, ProvisionType, SourceManifest};
use lex_parse::fidelity::{
    FidelityReport, GapClass, InputStatus, ProvisionFidelity, ProvisionInput, ProvisionStatus,
    ReportMeta, SourceDocument, check_document,
};
use lex_source::sha256_hex;
use serde::Serialize;
use serde_json::Value;

const SUMMARY_SCHEMA_VERSION: &str = "1.0.0";

#[derive(Debug, Args)]
pub(crate) struct VerifyFidelityArgs {
    /// Committed instrument slug under corpus/mx.
    #[arg(required_unless_present = "all", conflicts_with = "all")]
    slug: Option<String>,
    /// Check every corpus/mx instrument that has instrument.json and
    /// source-manifest.json, and write .work/fidelity-summary.json.
    #[arg(long)]
    all: bool,
    /// Exit 1 when any checked instrument has a non-passing provision or
    /// could not be checked.
    #[arg(long)]
    strict: bool,
}

#[derive(Debug, Serialize)]
struct SummaryEntry {
    slug: String,
    input_status: InputStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    input_detail: Option<String>,
    provision_count: usize,
    status_counts: BTreeMap<ProvisionStatus, usize>,
    gap_class_counts: BTreeMap<GapClass, usize>,
    all_provisions_pass: bool,
}

#[derive(Debug, Serialize)]
struct SummaryTotals {
    instruments: usize,
    instruments_passing: usize,
    provisions: usize,
    status_counts: BTreeMap<ProvisionStatus, usize>,
    input_status_counts: BTreeMap<InputStatus, usize>,
}

#[derive(Debug, Serialize)]
struct Summary {
    schema_version: &'static str,
    totals: SummaryTotals,
    instruments: Vec<SummaryEntry>,
}

pub(crate) fn run(root: &Path, args: VerifyFidelityArgs) -> Result<()> {
    let slugs = if args.all {
        committed_slugs(root)?
    } else {
        let slug = args.slug.context("missing instrument slug")?;
        if !root.join("corpus/mx").join(&slug).is_dir() {
            bail!("no committed instrument corpus/mx/{slug}");
        }
        vec![slug]
    };
    let mut entries = Vec::with_capacity(slugs.len());
    for slug in &slugs {
        let entry = verify_slug(root, slug);
        println!("{}", summary_line(&entry));
        entries.push(entry);
    }
    let summary = summarize(entries);
    println!("{}", totals_line(&summary.totals));
    if args.all {
        write_json(&root.join(".work/fidelity-summary.json"), &summary)?;
    }
    if args.strict && summary.totals.instruments_passing != summary.totals.instruments {
        bail!(
            "{} of {} instrument(s) have non-passing provisions or could not be checked",
            summary.totals.instruments - summary.totals.instruments_passing,
            summary.totals.instruments
        );
    }
    Ok(())
}

fn committed_slugs(root: &Path) -> Result<Vec<String>> {
    let directory = root.join("corpus/mx");
    let mut slugs = Vec::new();
    for entry in fs::read_dir(&directory)
        .with_context(|| format!("failed to read {}", directory.display()))?
    {
        let path = entry?.path();
        if path.join("instrument.json").is_file()
            && path.join("source-manifest.json").is_file()
            && let Some(name) = path.file_name().and_then(|name| name.to_str())
        {
            slugs.push(name.to_owned());
        }
    }
    slugs.sort();
    Ok(slugs)
}

fn summary_line(entry: &SummaryEntry) -> String {
    let count = |status| entry.status_counts.get(&status).copied().unwrap_or(0);
    format!(
        "{}: input={} provisions={} exact={} modulo_classified_gaps={} unclassified_gap={} \
         text_not_in_source={} diverged={} not_located={} empty={} pass={}",
        entry.slug,
        enum_name(&entry.input_status),
        entry.provision_count,
        count(ProvisionStatus::Exact),
        count(ProvisionStatus::ExactModuloClassifiedGaps),
        count(ProvisionStatus::UnclassifiedGap),
        count(ProvisionStatus::TextNotInSource),
        count(ProvisionStatus::Diverged),
        count(ProvisionStatus::NotLocated),
        count(ProvisionStatus::Empty),
        entry.all_provisions_pass
    )
}

fn totals_line(totals: &SummaryTotals) -> String {
    let count = |status| totals.status_counts.get(&status).copied().unwrap_or(0);
    let passing = count(ProvisionStatus::Exact) + count(ProvisionStatus::ExactModuloClassifiedGaps);
    let inputs: Vec<String> = totals
        .input_status_counts
        .iter()
        .map(|(status, n)| format!("{}={n}", enum_name(status)))
        .collect();
    format!(
        "totals: instruments={} passing_instruments={} provisions={} exact={} \
         modulo_classified_gaps={} unclassified_gap={} text_not_in_source={} diverged={} \
         not_located={} empty={} passing_provisions={passing} inputs[{}]",
        totals.instruments,
        totals.instruments_passing,
        totals.provisions,
        count(ProvisionStatus::Exact),
        count(ProvisionStatus::ExactModuloClassifiedGaps),
        count(ProvisionStatus::UnclassifiedGap),
        count(ProvisionStatus::TextNotInSource),
        count(ProvisionStatus::Diverged),
        count(ProvisionStatus::NotLocated),
        count(ProvisionStatus::Empty),
        inputs.join(" ")
    )
}

fn enum_name<T: Serialize>(value: &T) -> String {
    match serde_json::to_value(value) {
        Ok(Value::String(name)) => name,
        _ => String::from("unknown"),
    }
}

fn summarize(entries: Vec<SummaryEntry>) -> Summary {
    let mut status_counts: BTreeMap<ProvisionStatus, usize> =
        ProvisionStatus::ALL.iter().map(|s| (*s, 0)).collect();
    let mut input_status_counts: BTreeMap<InputStatus, usize> = BTreeMap::new();
    let mut provisions = 0;
    let mut instruments_passing = 0;
    for entry in &entries {
        provisions += entry.provision_count;
        *input_status_counts.entry(entry.input_status).or_default() += 1;
        instruments_passing += usize::from(entry.all_provisions_pass);
        for (status, count) in &entry.status_counts {
            *status_counts.entry(*status).or_default() += count;
        }
    }
    Summary {
        schema_version: SUMMARY_SCHEMA_VERSION,
        totals: SummaryTotals {
            instruments: entries.len(),
            instruments_passing,
            provisions,
            status_counts,
            input_status_counts,
        },
        instruments: entries,
    }
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, bytes).with_context(|| format!("failed to write {}", path.display()))
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("invalid JSON in {}", path.display()))
}

/// Check one instrument, write its report when `.work/<slug>/` exists, and
/// return its summary entry. Never returns an error: anything that prevents a
/// check is recorded as an input status so a batch run can continue.
fn verify_slug(root: &Path, slug: &str) -> SummaryEntry {
    match check_instrument(root, slug) {
        Ok(report) => {
            let work = root.join(".work").join(slug);
            let mut detail = report.input_detail.clone();
            if work.is_dir()
                && let Err(error) = write_json(&work.join("fidelity-report.json"), &report)
            {
                eprintln!("{slug}: {error:#}");
                detail = Some("fidelity-report.json could not be written".to_owned());
            }
            SummaryEntry {
                slug: slug.to_owned(),
                input_status: report.input_status,
                input_detail: detail,
                provision_count: report.provision_count,
                status_counts: report.status_counts,
                gap_class_counts: report.gap_class_counts,
                all_provisions_pass: report.all_provisions_pass,
            }
        }
        Err(error) => {
            eprintln!("{slug}: {error:#}");
            SummaryEntry {
                slug: slug.to_owned(),
                input_status: InputStatus::CorpusUnreadable,
                input_detail: Some(error.root_cause().to_string()),
                provision_count: 0,
                status_counts: ProvisionStatus::ALL.iter().map(|s| (*s, 0)).collect(),
                gap_class_counts: GapClass::ALL.iter().map(|c| (*c, 0)).collect(),
                all_provisions_pass: false,
            }
        }
    }
}

/// Retained inputs for an instrument, once verified against its manifests.
struct Inputs {
    main_text: String,
    annex_texts: BTreeMap<usize, String>,
    source_file_present: bool,
    annex_hash_checked: bool,
}

enum InputOutcome {
    Ready(Inputs),
    Failed(InputStatus, String),
}

fn check_instrument(root: &Path, slug: &str) -> Result<FidelityReport> {
    let corpus = root.join("corpus/mx").join(slug);
    let manifest: SourceManifest = read_json(&corpus.join("source-manifest.json"))?;
    let provisions: Vec<Provision> = read_json(&corpus.join("provisions.json"))?;
    let annex_numbers: Vec<usize> = provisions
        .iter()
        .filter(|provision| provision.provision_type == ProvisionType::Annex)
        .filter_map(|provision| provision.number.parse().ok())
        .collect();
    let mut meta = ReportMeta {
        instrument_id: manifest.instrument_id.clone(),
        input_status: InputStatus::Ok,
        input_detail: None,
        source_file_present: false,
        annex_extraction_hash_checked: false,
        source_sha256: manifest.source_sha256.clone(),
        extracted_text_sha256: manifest.extracted_text_sha256.clone(),
        extraction_tool: manifest.extraction_tool.clone(),
    };
    let work = root.join(".work").join(slug);
    let annex_manifests = read_annex_hashes(&corpus.join("annex-source-manifests.json"));
    match load_inputs(&work, slug, &manifest, &annex_numbers, &annex_manifests)? {
        InputOutcome::Failed(status, detail) => {
            meta.input_status = status;
            meta.input_detail = Some(detail);
            Ok(FidelityReport::new(meta, Vec::new()))
        }
        InputOutcome::Ready(inputs) => {
            meta.source_file_present = inputs.source_file_present;
            meta.annex_extraction_hash_checked = inputs.annex_hash_checked;
            let results = evaluate(&provisions, &inputs.main_text, &inputs.annex_texts);
            Ok(FidelityReport::new(meta, results))
        }
    }
}

/// Annex extraction hashes by annex number, from `annex-source-manifests.json`
/// (entry `n - 1` describes `annex-<n>`). Absent or unreadable means no hash.
fn read_annex_hashes(path: &Path) -> Vec<Option<String>> {
    read_json::<Vec<Value>>(path)
        .map(|entries| {
            entries
                .iter()
                .map(|entry| {
                    entry
                        .get("extracted_text_sha256")
                        .and_then(Value::as_str)
                        .map(str::to_owned)
                })
                .collect()
        })
        .unwrap_or_default()
}

fn load_inputs(
    work: &Path,
    slug: &str,
    manifest: &SourceManifest,
    annex_numbers: &[usize],
    annex_hashes: &[Option<String>],
) -> Result<InputOutcome> {
    let text_path = work.join(format!("{slug}.txt"));
    let Ok(text_bytes) = fs::read(&text_path) else {
        return Ok(InputOutcome::Failed(
            InputStatus::InputsMissing,
            format!(".work/{slug}/{slug}.txt is missing"),
        ));
    };
    let Some(expected) = manifest.extracted_text_sha256.as_deref() else {
        return Ok(InputOutcome::Failed(
            InputStatus::InputsMissing,
            "source-manifest.json records no extracted_text_sha256".to_owned(),
        ));
    };
    let source_present = source_candidates(work, slug)
        .into_iter()
        .filter_map(|path| fs::read(path).ok())
        .map(|bytes| sha256_hex(&bytes))
        .collect::<Vec<_>>();
    if !source_present.is_empty() && !source_present.contains(&manifest.source_sha256) {
        return Ok(InputOutcome::Failed(
            InputStatus::SourceHashMismatch,
            "retained source file sha256 differs from source-manifest.json".to_owned(),
        ));
    }
    if sha256_hex(&text_bytes) != expected {
        return Ok(InputOutcome::Failed(
            InputStatus::ExtractionHashMismatch,
            "retained extraction sha256 differs from source-manifest.json".to_owned(),
        ));
    }
    let main_text =
        String::from_utf8(text_bytes).context("retained extraction is not valid UTF-8")?;
    let mut annex_texts = BTreeMap::new();
    let mut every_annex_checked = true;
    for number in annex_numbers {
        if annex_texts.contains_key(number) {
            continue;
        }
        let Ok(bytes) = fs::read(work.join(format!("annex-{number}.txt"))) else {
            continue;
        };
        match number
            .checked_sub(1)
            .and_then(|index| annex_hashes.get(index))
            .and_then(Option::as_deref)
        {
            Some(recorded) if sha256_hex(&bytes) != recorded => {
                return Ok(InputOutcome::Failed(
                    InputStatus::AnnexExtractionHashMismatch,
                    format!(
                        "retained annex-{number}.txt sha256 differs from annex-source-manifests.json"
                    ),
                ));
            }
            Some(_) => {}
            None => every_annex_checked = false,
        }
        let text = String::from_utf8(bytes)
            .with_context(|| format!("retained annex-{number}.txt is not valid UTF-8"))?;
        annex_texts.insert(*number, text);
    }
    Ok(InputOutcome::Ready(Inputs {
        annex_hash_checked: !annex_texts.is_empty() && every_annex_checked,
        main_text,
        annex_texts,
        source_file_present: !source_present.is_empty(),
    }))
}

fn source_candidates(work: &Path, slug: &str) -> Vec<PathBuf> {
    ["pdf", "html", "doc"]
        .iter()
        .map(|extension| work.join(format!("{slug}.{extension}")))
        .filter(|path| path.is_file())
        .collect()
}

/// Align every provision against its source document: an annex provision uses
/// `annex-<n>.txt` when retained, otherwise the main text.
fn evaluate(
    provisions: &[Provision],
    main_text: &str,
    annex_texts: &BTreeMap<usize, String>,
) -> Vec<ProvisionFidelity> {
    let annex_of = |provision: &Provision| -> Option<usize> {
        if provision.provision_type == ProvisionType::Annex {
            provision
                .number
                .parse()
                .ok()
                .filter(|number| annex_texts.contains_key(number))
        } else {
            None
        }
    };
    let mut main_group: Vec<usize> = Vec::new();
    let mut annex_groups: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (position, provision) in provisions.iter().enumerate() {
        match annex_of(provision) {
            Some(number) => annex_groups.entry(number).or_default().push(position),
            None => main_group.push(position),
        }
    }
    let mut results: Vec<Option<ProvisionFidelity>> = vec![None; provisions.len()];
    let mut run_group = |label: &str, text: &str, group: &[usize]| {
        let document = SourceDocument::new(text);
        let inputs: Vec<ProvisionInput<'_>> = group
            .iter()
            .map(|&position| ProvisionInput {
                id: &provisions[position].id,
                text: &provisions[position].text,
            })
            .collect();
        for (&position, result) in group.iter().zip(check_document(&document, label, &inputs)) {
            results[position] = Some(result);
        }
    };
    run_group("main", main_text, &main_group);
    for (number, group) in &annex_groups {
        run_group(&format!("annex-{number}"), &annex_texts[number], group);
    }
    results.into_iter().flatten().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCHEMA: &str = include_str!("../../../schemas/fidelity-report-v1.schema.json");

    fn fixtures() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/fidelity")
    }

    fn fixture_provisions() -> Vec<Provision> {
        read_json(&fixtures().join("provisions.json")).unwrap()
    }

    /// Lay the fixture out as a repository root: corpus/mx/fix + .work/fix.
    fn fixture_root(extraction: &str, recorded_extraction_sha: Option<&str>) -> tempfile::TempDir {
        let root = tempfile::tempdir().unwrap();
        let corpus = root.path().join("corpus/mx/fix");
        let work = root.path().join(".work/fix");
        fs::create_dir_all(&corpus).unwrap();
        fs::create_dir_all(&work).unwrap();
        fs::copy(
            fixtures().join("provisions.json"),
            corpus.join("provisions.json"),
        )
        .unwrap();
        fs::write(corpus.join("instrument.json"), "{}").unwrap();
        let manifest = fs::read_to_string(fixtures().join("source-manifest.json"))
            .unwrap()
            .replace(
                "EXTRACTION_SHA256_PLACEHOLDER",
                &recorded_extraction_sha
                    .map_or_else(|| sha256_hex(extraction.as_bytes()), str::to_owned),
            );
        fs::write(corpus.join("source-manifest.json"), manifest).unwrap();
        fs::write(work.join("fix.txt"), extraction).unwrap();
        root
    }

    fn fixture_extraction() -> String {
        fs::read_to_string(fixtures().join("extraction.txt")).unwrap()
    }

    fn validate_against_schema(report: &FidelityReport) {
        let schema: Value = serde_json::from_str(SCHEMA).unwrap();
        let value = serde_json::to_value(report).unwrap();
        jsonschema::validator_for(&schema)
            .unwrap()
            .validate(&value)
            .unwrap_or_else(|error| panic!("report violates schema: {error}"));
    }

    #[test]
    fn fixture_exercises_every_class_and_status() {
        let provisions = fixture_provisions();
        let results = evaluate(&provisions, &fixture_extraction(), &BTreeMap::new());
        let statuses: Vec<(&str, ProvisionStatus)> = results
            .iter()
            .map(|r| (r.provision_id.rsplit(':').next().unwrap(), r.status))
            .collect();
        assert_eq!(
            statuses,
            vec![
                ("1", ProvisionStatus::Exact),
                ("2", ProvisionStatus::ExactModuloClassifiedGaps),
                ("3", ProvisionStatus::ExactModuloClassifiedGaps),
                ("4", ProvisionStatus::ExactModuloClassifiedGaps),
                ("5", ProvisionStatus::UnclassifiedGap),
                ("6", ProvisionStatus::TextNotInSource),
                ("1", ProvisionStatus::Exact),
            ]
        );
        assert_eq!(results[1].gaps[0].class, GapClass::AmendmentMark);
        assert_eq!(results[2].gaps[0].class, GapClass::EditorialReformNote);
        assert_eq!(results[2].gaps[0].text, "Párrafo reformado DOF 09-03-2018");
        assert_eq!(results[3].gaps[0].class, GapClass::PageFurniture);
        assert_eq!(
            results[4].gaps[0].text,
            "Esta oración fue omitida en la copia comprometida."
        );
        assert_eq!(results[5].insertions[0].text, "por correo electrónico");
    }

    #[test]
    fn generated_report_validates_against_the_schema() {
        let extraction = fixture_extraction();
        let root = fixture_root(&extraction, None);
        let report = check_instrument(root.path(), "fix").unwrap();
        assert_eq!(report.input_status, InputStatus::Ok);
        assert!(!report.source_file_present);
        assert!(!report.all_provisions_pass);
        assert_eq!(report.provision_count, 7);
        validate_against_schema(&report);

        let mut failed = report.clone();
        failed.input_status = InputStatus::ExtractionHashMismatch;
        failed.input_detail = Some("detail".to_owned());
        failed.provisions.clear();
        validate_against_schema(&FidelityReport {
            provision_count: 0,
            ..failed
        });
    }

    #[test]
    fn schema_enums_match_the_rust_enums() {
        let schema: Value = serde_json::from_str(SCHEMA).unwrap();
        let listed = |pointer: &str| -> Vec<String> {
            schema
                .pointer(pointer)
                .and_then(Value::as_array)
                .unwrap_or_else(|| panic!("missing {pointer}"))
                .iter()
                .map(|v| v.as_str().unwrap().to_owned())
                .collect()
        };
        let mut statuses: Vec<String> = ProvisionStatus::ALL.iter().map(enum_name).collect();
        let mut gap_classes: Vec<String> = GapClass::ALL.iter().map(enum_name).collect();
        let mut inputs: Vec<String> = [
            InputStatus::Ok,
            InputStatus::InputsMissing,
            InputStatus::SourceHashMismatch,
            InputStatus::ExtractionHashMismatch,
            InputStatus::AnnexExtractionHashMismatch,
            InputStatus::CorpusUnreadable,
        ]
        .iter()
        .map(enum_name)
        .collect();
        let mut schema_statuses = listed("/$defs/provision_status/enum");
        let mut schema_classes = listed("/$defs/gap_class/enum");
        let mut schema_inputs = listed("/$defs/input_status/enum");
        for list in [
            &mut statuses,
            &mut gap_classes,
            &mut inputs,
            &mut schema_statuses,
            &mut schema_classes,
            &mut schema_inputs,
        ] {
            list.sort();
        }
        assert_eq!(statuses, schema_statuses);
        assert_eq!(gap_classes, schema_classes);
        assert_eq!(inputs, schema_inputs);
    }

    #[test]
    fn report_is_deterministic_and_round_trips() {
        let extraction = fixture_extraction();
        let root = fixture_root(&extraction, None);
        let first = check_instrument(root.path(), "fix").unwrap();
        let second = check_instrument(root.path(), "fix").unwrap();
        let text = serde_json::to_string_pretty(&first).unwrap();
        assert_eq!(text, serde_json::to_string_pretty(&second).unwrap());
        assert!(!text.contains(root.path().to_str().unwrap()));
        let back: FidelityReport = serde_json::from_str(&text).unwrap();
        assert_eq!(back, first);
    }

    #[test]
    fn extraction_hash_mismatch_fails_closed_without_aligning() {
        let extraction = fixture_extraction();
        let root = fixture_root(&extraction, Some(&"f".repeat(64)));
        let report = check_instrument(root.path(), "fix").unwrap();
        assert_eq!(report.input_status, InputStatus::ExtractionHashMismatch);
        assert_eq!(report.provisions.len(), 0);
        assert!(!report.all_provisions_pass);
        validate_against_schema(&report);
    }

    #[test]
    fn missing_extraction_is_inputs_missing_and_source_mismatch_is_reported() {
        let extraction = fixture_extraction();
        let root = fixture_root(&extraction, None);
        fs::write(
            root.path().join(".work/fix/fix.pdf"),
            b"not the recorded pdf",
        )
        .unwrap();
        let report = check_instrument(root.path(), "fix").unwrap();
        assert_eq!(report.input_status, InputStatus::SourceHashMismatch);
        fs::remove_file(root.path().join(".work/fix/fix.pdf")).unwrap();
        fs::remove_file(root.path().join(".work/fix/fix.txt")).unwrap();
        let report = check_instrument(root.path(), "fix").unwrap();
        assert_eq!(report.input_status, InputStatus::InputsMissing);
    }

    #[test]
    fn verify_writes_only_under_work() {
        let extraction = fixture_extraction();
        let root = fixture_root(&extraction, None);
        let corpus_before = fs::read(root.path().join("corpus/mx/fix/provisions.json")).unwrap();
        let entry = verify_slug(root.path(), "fix");
        assert_eq!(entry.input_status, InputStatus::Ok);
        assert!(root.path().join(".work/fix/fidelity-report.json").is_file());
        let text = fs::read_to_string(root.path().join(".work/fix/fidelity-report.json")).unwrap();
        assert!(text.ends_with("}\n"));
        assert_eq!(
            fs::read(root.path().join("corpus/mx/fix/provisions.json")).unwrap(),
            corpus_before
        );
        let corpus_files: Vec<String> = fs::read_dir(root.path().join("corpus/mx/fix"))
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(corpus_files.len(), 3);
    }

    #[test]
    fn annex_provision_uses_its_own_annex_text() {
        let mut provisions = fixture_provisions();
        let mut annex = provisions[0].clone();
        annex.provision_type = ProvisionType::Annex;
        annex.number = "2".to_owned();
        annex.id = "urn:test:annex:2".to_owned();
        annex.text = "ANEXO 2 Columna uno Columna dos Columna tres".to_owned();
        provisions.push(annex);
        let annexes = BTreeMap::from([(
            2,
            "ANEXO 2 Columna uno\nColumna dos Columna tres\n".to_owned(),
        )]);
        let results = evaluate(&provisions, &fixture_extraction(), &annexes);
        assert_eq!(results.len(), 8);
        assert_eq!(results[7].source_document, "annex-2");
        assert_eq!(results[7].status, ProvisionStatus::Exact);
        assert_eq!(results[0].source_document, "main");
        // Without the annex file the annex provision falls back to the main text.
        let results = evaluate(&provisions, &fixture_extraction(), &BTreeMap::new());
        assert_eq!(results[7].source_document, "main");
    }
}
