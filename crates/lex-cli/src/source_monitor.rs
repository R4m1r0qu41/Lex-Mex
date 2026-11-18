//! `monitor-sources`: read-only change monitor for official sources.
//!
//! Answers two questions without ever touching `corpus/`: has the official
//! file of a committed instrument changed since the hash recorded in its
//! `source-manifest.json`, and does the Cámara de Diputados index list laws or
//! regulations the corpus does not have. Everything it writes goes under
//! `.work/`. Acting on a change (refetch, review, release) stays a separate,
//! reviewed step.
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fs,
    io::Write as _,
    path::{Path, PathBuf},
    sync::LazyLock,
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail};
use chrono::{DateTime, SecondsFormat, Utc};
use clap::Args;
use lex_core::SourceManifest;
use lex_source::{
    Acquisition, SourceConfig, SourceFormat, SourceProbe, fetch, load_config, probe_source,
    sha256_hex,
};
use regex::Regex;
use serde::Serialize;

use crate::{Paths, adapter_paths};

const REPORT_SCHEMA_VERSION: &str = "1.0.0";
const REPORT_PATH: &str = ".work/source-monitor-report.json";
const LAWS_INDEX_URL: &str = "https://www.diputados.gob.mx/LeyesBiblio/index.htm";
const REGULATIONS_INDEX_URL: &str = "https://www.diputados.gob.mx/LeyesBiblio/regla.htm";
const DISCOVERY_ADAPTER_DIR: &str = "diputados";

#[derive(Debug, Args)]
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct MonitorSourcesArgs {
    /// Committed instrument slugs under corpus/mx to check.
    #[arg(conflicts_with = "all")]
    slugs: Vec<String>,
    /// Check every committed instrument that has a source-manifest.json.
    #[arg(long)]
    all: bool,
    /// Always download. When the bytes hash to exactly the recorded
    /// `source_sha256` and the pipeline's retained source file is missing, write
    /// it there (never overwriting). Changed files are saved under
    /// `.work/monitor/<slug>/` instead.
    #[arg(long)]
    retain: bool,
    /// Also compare the Cámara de Diputados laws and regulations indexes with
    /// the committed instruments.
    #[arg(long)]
    discover: bool,
    /// Minimum delay between two requests to the same host, in milliseconds.
    #[arg(long, default_value_t = 1000)]
    delay_ms: u64,
    /// Exit 1 when anything is changed, unreachable, not checked, discovered, or
    /// when an index could not be read.
    #[arg(long)]
    strict: bool,
}

// ---------------------------------------------------------------------------
// Report types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
enum Status {
    UnchangedByValidators,
    UnchangedByHash,
    Changed,
    Unreachable,
    NotChecked,
}

impl Status {
    const ALL: [Self; 5] = [
        Self::UnchangedByValidators,
        Self::UnchangedByHash,
        Self::Changed,
        Self::Unreachable,
        Self::NotChecked,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::UnchangedByValidators => "unchanged_by_validators",
            Self::UnchangedByHash => "unchanged_by_hash",
            Self::Changed => "changed",
            Self::Unreachable => "unreachable",
            Self::NotChecked => "not_checked",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Recorded {
    sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    etag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_modified: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content_length: Option<u64>,
    retrieved_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ObservedVia {
    Probe,
    Download,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Observed {
    via: ObservedVia,
    #[serde(skip_serializing_if = "Option::is_none")]
    http_status: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    etag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_modified: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content_length: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum RetainOutcome {
    /// The retained pipeline source was missing and was restored.
    Written,
    /// The retained pipeline source already exists; it was left untouched.
    AlreadyPresent,
    /// A changed file was saved under `.work/monitor/<slug>/` for review.
    SavedForReview,
}

#[derive(Debug, Serialize)]
struct InstrumentEntry {
    slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    official_url: Option<String>,
    status: Status,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    recorded: Option<Recorded>,
    #[serde(skip_serializing_if = "Option::is_none")]
    observed: Option<Observed>,
    #[serde(skip_serializing_if = "Option::is_none")]
    retained: Option<RetainOutcome>,
    /// Path relative to the repository root.
    #[serde(skip_serializing_if = "Option::is_none")]
    retained_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    retain_error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum IndexKind {
    Laws,
    Regulations,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct IndexEntry {
    index: IndexKind,
    /// Title as printed in the index row.
    title: String,
    pdf_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_reform: Option<String>,
    /// The index itself marks the instrument as abrogated (a "Reglamento
    /// Abrogado" note or an `abro/` path).
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    abrogated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct MissingFromIndex {
    slug: String,
    official_url: String,
}

#[derive(Debug, Serialize)]
struct IndexError {
    index: IndexKind,
    url: String,
    message: String,
}

#[derive(Debug, Serialize)]
struct Options {
    retain: bool,
    discover: bool,
    delay_ms: u64,
}

#[derive(Debug, Serialize)]
struct Report {
    schema_version: &'static str,
    checked_at: String,
    options: Options,
    /// Annex and formal-publication sources are not monitored yet.
    annex_sources_checked: bool,
    status_counts: BTreeMap<Status, usize>,
    instruments: Vec<InstrumentEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    index_entries_parsed: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    index_errors: Option<Vec<IndexError>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    discovered: Option<Vec<IndexEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    missing_from_index: Option<Vec<MissingFromIndex>>,
}

impl Report {
    fn needs_attention(&self) -> bool {
        self.status_counts
            .iter()
            .any(|(status, count)| *count > 0 && !is_unchanged(*status))
            || self.discovered.as_ref().is_some_and(|d| !d.is_empty())
            || self.index_errors.as_ref().is_some_and(|e| !e.is_empty())
    }
}

fn is_unchanged(status: Status) -> bool {
    matches!(
        status,
        Status::UnchangedByValidators | Status::UnchangedByHash
    )
}

// ---------------------------------------------------------------------------
// Network seam
// ---------------------------------------------------------------------------

/// The only two network operations the monitor performs. Production uses
/// `lex-source` (same client, user agent and certificates as ingestion); tests
/// substitute canned answers so no test touches the network.
trait SourceAccess {
    fn probe(&self, config: &SourceConfig) -> Result<SourceProbe>;
    fn download(&self, config: &SourceConfig) -> Result<Acquisition>;
}

struct LiveAccess;

impl SourceAccess for LiveAccess {
    fn probe(&self, config: &SourceConfig) -> Result<SourceProbe> {
        probe_source(config)
    }

    fn download(&self, config: &SourceConfig) -> Result<Acquisition> {
        fetch(config)
    }
}

/// Keeps a minimum delay between two requests to the same host.
struct Pacer {
    delay: Duration,
    last: HashMap<String, Instant>,
}

impl Pacer {
    fn new(delay: Duration) -> Self {
        Self {
            delay,
            last: HashMap::new(),
        }
    }

    fn wait(&mut self, host: &str) {
        let since_last = self.last.get(host).map(Instant::elapsed);
        let remaining = remaining_delay(since_last, self.delay);
        if !remaining.is_zero() {
            thread::sleep(remaining);
        }
        self.last.insert(host.to_owned(), Instant::now());
    }
}

fn remaining_delay(since_last: Option<Duration>, delay: Duration) -> Duration {
    since_last.map_or(Duration::ZERO, |elapsed| delay.saturating_sub(elapsed))
}

fn host_of(config: &SourceConfig) -> String {
    config.source_url.host_str().unwrap_or("").to_owned()
}

// ---------------------------------------------------------------------------
// Pure decisions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
enum ValidatorVerdict {
    /// Every recorded validator equals what the server serves now.
    Match,
    /// At least one recorded validator differs or is no longer served.
    Differs(Vec<&'static str>),
    /// Validators cannot settle the question; download and hash.
    Insufficient(&'static str),
}

/// Compare what the manifest recorded with what a HEAD probe observed.
///
/// A validator is trusted only when the probe succeeded (2xx) and at least one
/// of `ETag` or `Last-Modified` was recorded: `content_length` alone is set on
/// every manifest and a same-size revision would otherwise pass silently.
fn compare_validators(recorded: &Recorded, probe: Option<&SourceProbe>) -> ValidatorVerdict {
    let Some(probe) = probe else {
        return ValidatorVerdict::Insufficient("probe unavailable");
    };
    if !(200..300).contains(&probe.http_status) {
        return ValidatorVerdict::Insufficient("probe returned a non-success status");
    }
    if recorded.etag.is_none() && recorded.last_modified.is_none() {
        return ValidatorVerdict::Insufficient("manifest records no ETag or Last-Modified");
    }
    let mut differing = Vec::new();
    if recorded.etag.is_some() && recorded.etag != probe.etag {
        differing.push("etag");
    }
    if recorded.last_modified.is_some() && recorded.last_modified != probe.last_modified {
        differing.push("last_modified");
    }
    if recorded.content_length.is_some() && recorded.content_length != probe.content_length {
        differing.push("content_length");
    }
    if differing.is_empty() {
        ValidatorVerdict::Match
    } else {
        ValidatorVerdict::Differs(differing)
    }
}

fn classify_hash(recorded_sha256: &str, observed_sha256: &str) -> Status {
    if recorded_sha256.eq_ignore_ascii_case(observed_sha256) {
        Status::UnchangedByHash
    } else {
        Status::Changed
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RetainAction {
    None,
    /// Restore the pipeline's retained source file.
    WriteRetained,
    AlreadyRetained,
    /// Save the changed bytes at `.work/monitor/<slug>/<sha256>.<ext>`.
    SaveForReview,
}

/// Where downloaded bytes go. A changed file is always kept for review (it is
/// already downloaded) and never at the pipeline's retained path; an unchanged
/// file restores the retained path only under `--retain` and only when absent.
fn retain_decision(retain: bool, hash_matches: bool, retained_exists: bool) -> RetainAction {
    if !hash_matches {
        RetainAction::SaveForReview
    } else if !retain {
        RetainAction::None
    } else if retained_exists {
        RetainAction::AlreadyRetained
    } else {
        RetainAction::WriteRetained
    }
}

fn changed_copy_path(root: &Path, slug: &str, sha256: &str, extension: &str) -> PathBuf {
    root.join(".work/monitor")
        .join(slug)
        .join(format!("{sha256}.{extension}"))
}

fn source_extension(format: SourceFormat) -> &'static str {
    match format {
        SourceFormat::Pdf => "pdf",
        SourceFormat::Html => "html",
        SourceFormat::Doc => "doc",
    }
}

// ---------------------------------------------------------------------------
// Filesystem helpers (the only writers)
// ---------------------------------------------------------------------------

fn ensure_outside_corpus(root: &Path, path: &Path) -> Result<()> {
    if path.starts_with(root.join("corpus")) {
        bail!("refusing to write under corpus/: {}", path.display());
    }
    Ok(())
}

/// Write `bytes` to `path` without ever replacing an existing file. Returns
/// `Ok(false)` when the file already existed. The bytes land in a sibling
/// temporary file first, so a crash never leaves a partial file at `path`.
fn write_new_file(root: &Path, path: &Path, bytes: &[u8]) -> Result<bool> {
    ensure_outside_corpus(root, path)?;
    let parent = path.parent().context("path has no parent directory")?;
    fs::create_dir_all(parent)?;
    let temporary = parent.join(format!(
        ".{}.part{}",
        path.file_name().and_then(|n| n.to_str()).unwrap_or("file"),
        std::process::id()
    ));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .with_context(|| format!("failed to create {}", temporary.display()))?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    let linked = fs::hard_link(&temporary, path);
    let _ = fs::remove_file(&temporary);
    match linked {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),
        Err(error) => Err(error).with_context(|| format!("failed to write {}", path.display())),
    }
}

fn write_report(root: &Path, report: &Report) -> Result<()> {
    let path = root.join(REPORT_PATH);
    ensure_outside_corpus(root, &path)?;
    let mut bytes = serde_json::to_vec_pretty(report)?;
    bytes.push(b'\n');
    fs::create_dir_all(path.parent().context("report path has no parent")?)?;
    fs::write(&path, bytes).with_context(|| format!("failed to write {}", path.display()))
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}

// ---------------------------------------------------------------------------
// Per-instrument check
// ---------------------------------------------------------------------------

struct Settings {
    retain: bool,
    delay: Duration,
}

fn not_checked(slug: &str, detail: String) -> InstrumentEntry {
    InstrumentEntry {
        slug: slug.to_owned(),
        official_url: None,
        status: Status::NotChecked,
        detail: Some(detail),
        recorded: None,
        observed: None,
        retained: None,
        retained_path: None,
        retain_error: None,
    }
}

fn recorded_from(manifest: &SourceManifest) -> Recorded {
    Recorded {
        sha256: manifest.source_sha256.clone(),
        etag: manifest.etag.clone(),
        last_modified: manifest.last_modified.clone(),
        content_length: manifest.content_length,
        retrieved_at: manifest
            .retrieved_at
            .to_rfc3339_opts(SecondsFormat::Secs, true),
    }
}

fn observed_from_probe(probe: &SourceProbe) -> Observed {
    Observed {
        via: ObservedVia::Probe,
        http_status: Some(probe.http_status),
        sha256: None,
        etag: probe.etag.clone(),
        last_modified: probe.last_modified.clone(),
        content_length: probe.content_length,
    }
}

fn observed_from_download(acquisition: &Acquisition) -> Observed {
    Observed {
        via: ObservedVia::Download,
        http_status: Some(acquisition.manifest.http_status),
        sha256: Some(sha256_hex(&acquisition.bytes)),
        etag: acquisition.manifest.etag.clone(),
        last_modified: acquisition.manifest.last_modified.clone(),
        content_length: Some(acquisition.bytes.len() as u64),
    }
}

fn check_instrument(
    root: &Path,
    slug: &str,
    config: &Result<SourceConfig, String>,
    settings: &Settings,
    access: &dyn SourceAccess,
    pacer: &mut Pacer,
) -> InstrumentEntry {
    // Read the manifest exactly once; another process may rewrite it mid-run.
    let manifest_path = root
        .join("corpus/mx")
        .join(slug)
        .join("source-manifest.json");
    let manifest: SourceManifest = match fs::read(&manifest_path)
        .map_err(anyhow::Error::from)
        .and_then(|bytes| serde_json::from_slice(&bytes).map_err(anyhow::Error::from))
    {
        Ok(manifest) => manifest,
        Err(error) => {
            return not_checked(slug, format!("unreadable source-manifest.json: {error}"));
        }
    };
    let official_url = manifest.official_url.as_str().to_owned();
    let mut entry = not_checked(slug, String::new());
    entry.official_url = Some(official_url.clone());
    entry.detail = None;
    entry.recorded = Some(recorded_from(&manifest));
    let config = match config {
        Ok(config) => config,
        Err(message) => {
            entry.detail = Some(message.clone());
            return entry;
        }
    };
    if !config
        .source_url
        .as_str()
        .eq_ignore_ascii_case(&official_url)
    {
        entry.detail = Some(format!(
            "adapter source_url {} differs from the manifest official_url",
            config.source_url
        ));
        return entry;
    }
    let recorded = recorded_from(&manifest);
    let host = host_of(config);

    let mut probe_note = None;
    if !settings.retain {
        pacer.wait(&host);
        let probe = access.probe(config);
        let probe = match probe {
            Ok(probe) => Some(probe),
            Err(error) => {
                probe_note = Some(format!("probe failed: {error:#}"));
                None
            }
        };
        match compare_validators(&recorded, probe.as_ref()) {
            ValidatorVerdict::Match => {
                entry.status = Status::UnchangedByValidators;
                entry.observed = probe.as_ref().map(observed_from_probe);
                return entry;
            }
            ValidatorVerdict::Differs(fields) => {
                probe_note = Some(format!("validators differ: {}", fields.join(", ")));
            }
            ValidatorVerdict::Insufficient(reason) => {
                probe_note.get_or_insert_with(|| reason.to_owned());
            }
        }
    }

    pacer.wait(&host);
    let acquisition = match access.download(config) {
        Ok(acquisition) => acquisition,
        Err(error) => {
            entry.status = Status::Unreachable;
            entry.detail = Some(format!("{error:#}"));
            return entry;
        }
    };
    let observed = observed_from_download(&acquisition);
    let observed_sha = observed.sha256.clone().unwrap_or_default();
    entry.status = classify_hash(&recorded.sha256, &observed_sha);
    entry.observed = Some(observed);
    entry.detail = probe_note;

    let paths = Paths::new(root, slug, config.source_format);
    let extension = source_extension(config.source_format);
    let action = retain_decision(
        settings.retain,
        entry.status != Status::Changed,
        paths.source.exists(),
    );
    apply_retain(
        root,
        &mut entry,
        action,
        &paths.source,
        &changed_copy_path(root, slug, &observed_sha, extension),
        &acquisition.bytes,
    );
    entry
}

fn apply_retain(
    root: &Path,
    entry: &mut InstrumentEntry,
    action: RetainAction,
    retained_path: &Path,
    review_path: &Path,
    bytes: &[u8],
) {
    let (target, outcome) = match action {
        RetainAction::None => return,
        RetainAction::AlreadyRetained => {
            entry.retained = Some(RetainOutcome::AlreadyPresent);
            entry.retained_path = Some(relative(root, retained_path));
            return;
        }
        RetainAction::WriteRetained => (retained_path, RetainOutcome::Written),
        RetainAction::SaveForReview => (review_path, RetainOutcome::SavedForReview),
    };
    match write_new_file(root, target, bytes) {
        Ok(created) => {
            entry.retained = Some(if created || outcome == RetainOutcome::SavedForReview {
                outcome
            } else {
                RetainOutcome::AlreadyPresent
            });
            entry.retained_path = Some(relative(root, target));
        }
        Err(error) => entry.retain_error = Some(format!("{error:#}")),
    }
}

// ---------------------------------------------------------------------------
// Index parsing (pure)
// ---------------------------------------------------------------------------

static ROW_SPLIT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)<tr\b").expect("regex"));
static CELL_SPLIT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)<td\b[^>]*>").expect("regex"));
static ANCHOR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)<a\b([^>]*)>(.*?)</a>").expect("regex"));
static HREF: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?i)href\s*=\s*"([^"]*)""#).expect("regex"));
static PARAGRAPH: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)<p\b[^>]*>").expect("regex"));
static TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)<[^>]*>").expect("regex"));
static ENTITY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"&(#[0-9]+|#[xX][0-9a-fA-F]+|[A-Za-z]+);").expect("regex"));
static REFORM: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)Reformas?\s+(?:DOF\s+)?\d{2}/\d{2}/\d{4}(?:\s*,\s*(?:DOF\s+)?\d{2}/\d{2}/\d{4})*",
    )
    .expect("regex")
});

/// The Cámara pages are served as ISO-8859-1. Decode UTF-8 when the bytes are
/// valid UTF-8, otherwise map each byte to the Unicode code point it names.
fn decode_html(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec())
        .unwrap_or_else(|_| bytes.iter().map(|&byte| char::from(byte)).collect())
}

fn decode_entities(text: &str) -> String {
    ENTITY
        .replace_all(text, |captures: &regex::Captures<'_>| {
            let name = &captures[1];
            let decoded = if let Some(hex) = name.strip_prefix("#x").or(name.strip_prefix("#X")) {
                u32::from_str_radix(hex, 16).ok().and_then(char::from_u32)
            } else if let Some(decimal) = name.strip_prefix('#') {
                decimal.parse().ok().and_then(char::from_u32)
            } else {
                match name {
                    "nbsp" => Some(' '),
                    "quot" => Some('"'),
                    "amp" => Some('&'),
                    "lt" => Some('<'),
                    "gt" => Some('>'),
                    "apos" => Some('\''),
                    _ => None,
                }
            };
            decoded.map_or_else(|| captures[0].to_owned(), |c| c.to_string())
        })
        .into_owned()
}

/// Visible text of an HTML fragment with whitespace (including CRLF, tabs and
/// non-breaking spaces) collapsed to single spaces.
fn visible_text(fragment: &str) -> String {
    let stripped = TAG.replace_all(fragment, " ");
    decode_entities(&stripped)
        .split(|c: char| c.is_whitespace() || c == '\u{a0}')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Resolve `href` against the page URL `base` (http or https, no userinfo).
fn resolve_href(base: &str, href: &str) -> String {
    let href = href
        .trim()
        .split('#')
        .next()
        .unwrap_or("")
        .replace(' ', "%20");
    if href.contains("://") {
        return href;
    }
    let (origin, path) = base.split_once("://").map_or((base, ""), |(scheme, rest)| {
        let host_end = rest.find('/').unwrap_or(rest.len());
        (&base[..scheme.len() + 3 + host_end], &rest[host_end..])
    });
    let path = path.split(['?', '#']).next().unwrap_or("");
    let mut segments: Vec<&str> = if href.starts_with('/') {
        Vec::new()
    } else {
        let directory = path.rsplit_once('/').map_or("", |(dir, _)| dir);
        directory.split('/').filter(|s| !s.is_empty()).collect()
    };
    for segment in href.trim_start_matches('/').split('/') {
        match segment {
            "." | "" => {}
            ".." => {
                segments.pop();
            }
            other => segments.push(other),
        }
    }
    format!("{origin}/{}", segments.join("/"))
}

/// Pick the row's PDF link: an anchor whose visible text is "PDF" when there is
/// one (regulation rows also link historical PDFs), otherwise the first PDF
/// link. Mobile (`pdf_mov/`) copies are never the official text.
fn row_pdf_href(row: &str) -> Option<String> {
    let mut first = None;
    for anchor in ANCHOR.captures_iter(row) {
        let Some(href) = HREF.captures(&anchor[1]).map(|c| c[1].to_owned()) else {
            continue;
        };
        let lowered = href.to_ascii_lowercase();
        let path = lowered.split(['?', '#']).next().unwrap_or("");
        let is_pdf = Path::new(path)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("pdf"));
        if !is_pdf || path.contains("pdf_mov/") {
            continue;
        }
        if visible_text(&anchor[2]).eq_ignore_ascii_case("pdf") {
            return Some(href);
        }
        first.get_or_insert(href);
    }
    first
}

/// The title is the first non-empty paragraph of the title cell. Later
/// paragraphs carry notes ("(Antes ...)", "Reglamento Abrogado", original and
/// reform dates), and the markup around the title varies between `<b>`,
/// `<span style="font-weight: 700">` and nothing.
fn first_paragraph_text(cell: &str) -> String {
    PARAGRAPH
        .split(cell)
        .map(visible_text)
        .find(|text| !text.is_empty())
        .unwrap_or_default()
}

/// Parse one Cámara index page into its PDF-bearing rows. Rows without an
/// official PDF link (header rows, navigation tables) are ignored.
fn parse_index(html: &[u8], kind: IndexKind, base_url: &str) -> Vec<IndexEntry> {
    let html = decode_html(html);
    let mut entries = Vec::new();
    for row in ROW_SPLIT.split(&html).skip(1) {
        let Some(href) = row_pdf_href(row) else {
            continue;
        };
        let cells: Vec<&str> = CELL_SPLIT.split(row).skip(1).collect();
        let Some(title_cell) = cells.get(1) else {
            continue;
        };
        let title = first_paragraph_text(title_cell);
        if title.is_empty() {
            continue;
        }
        let last_reform = match kind {
            IndexKind::Laws => cells
                .get(2)
                .map(|cell| visible_text(cell))
                .filter(|text| !text.is_empty()),
            IndexKind::Regulations => REFORM
                .find_iter(&visible_text(title_cell))
                .last()
                .map(|found| found.as_str().to_owned()),
        };
        let pdf_url = resolve_href(base_url, &href);
        let abrogated = visible_text(title_cell).to_lowercase().contains("abrogad")
            || pdf_url.to_lowercase().contains("/abro/");
        entries.push(IndexEntry {
            index: kind,
            title,
            pdf_url,
            last_reform,
            abrogated,
        });
    }
    entries
}

/// Compare index entries with committed official URLs (case-insensitive).
/// Returns the entries the corpus lacks (first occurrence per URL), and the
/// committed instruments hosted in a directory the indexes list from whose URL
/// no longer appears.
fn compare_with_index(
    entries: &[IndexEntry],
    committed: &BTreeMap<String, String>,
) -> (Vec<IndexEntry>, Vec<MissingFromIndex>) {
    let committed_urls: BTreeSet<String> = committed
        .values()
        .map(|url| url.to_ascii_lowercase())
        .collect();
    let indexed_urls: BTreeSet<String> = entries
        .iter()
        .map(|entry| entry.pdf_url.to_ascii_lowercase())
        .collect();
    let mut seen = BTreeSet::new();
    let discovered = entries
        .iter()
        .filter(|entry| {
            let key = entry.pdf_url.to_ascii_lowercase();
            !committed_urls.contains(&key) && seen.insert(key)
        })
        .cloned()
        .collect();
    let covered_directories: BTreeSet<String> = indexed_urls
        .iter()
        .filter_map(|url| url.rsplit_once('/').map(|(dir, _)| dir.to_owned()))
        .collect();
    let missing = committed
        .iter()
        .filter(|(_, url)| {
            let key = url.to_ascii_lowercase();
            key.rsplit_once('/')
                .is_some_and(|(dir, _)| covered_directories.contains(dir))
                && !indexed_urls.contains(&key)
        })
        .map(|(slug, url)| MissingFromIndex {
            slug: slug.clone(),
            official_url: url.clone(),
        })
        .collect();
    (discovered, missing)
}

// ---------------------------------------------------------------------------
// Orchestration
// ---------------------------------------------------------------------------

/// Slug -> committed official URL for every `corpus/mx/*/source-manifest.json`.
fn committed_urls(root: &Path) -> Result<BTreeMap<String, String>> {
    let directory = root.join("corpus/mx");
    let mut urls = BTreeMap::new();
    for entry in fs::read_dir(&directory)
        .with_context(|| format!("failed to read {}", directory.display()))?
    {
        let path = entry?.path();
        let Some(slug) = path.file_name().and_then(|n| n.to_str()).map(str::to_owned) else {
            continue;
        };
        let Ok(bytes) = fs::read(path.join("source-manifest.json")) else {
            continue;
        };
        if let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes)
            && let Some(url) = value.get("official_url").and_then(|v| v.as_str())
        {
            urls.insert(slug, url.to_owned());
        }
    }
    Ok(urls)
}

/// Load the adapter config of each slug. Adapters are normally
/// `adapters/<source>/<slug>.json`; only when that is not the case are the
/// remaining adapters scanned, once.
fn load_adapters(root: &Path, slugs: &[String]) -> BTreeMap<String, Result<SourceConfig, String>> {
    let paths = adapter_paths(root).unwrap_or_default();
    let mut by_stem: HashMap<String, Vec<&PathBuf>> = HashMap::new();
    for path in &paths {
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            by_stem.entry(stem.to_owned()).or_default().push(path);
        }
    }
    let mut loaded: BTreeMap<String, Result<SourceConfig, String>> = BTreeMap::new();
    let mut load_errors: BTreeMap<String, String> = BTreeMap::new();
    for slug in slugs {
        for path in by_stem.get(slug).into_iter().flatten() {
            match load_config(path) {
                Ok(config) if config.slug == *slug => {
                    loaded.insert(slug.clone(), Ok(config));
                    break;
                }
                Ok(_) => {}
                Err(error) => {
                    load_errors.insert(slug.clone(), format!("{error:#}"));
                }
            }
        }
    }
    if slugs.iter().any(|slug| !loaded.contains_key(slug)) {
        for path in &paths {
            if let Ok(config) = load_config(path)
                && slugs.contains(&config.slug)
                && !loaded.contains_key(&config.slug)
            {
                loaded.insert(config.slug.clone(), Ok(config));
            }
        }
    }
    for slug in slugs {
        loaded.entry(slug.clone()).or_insert_with(|| {
            Err(load_errors.remove(slug).unwrap_or_else(|| {
                "no adapter configuration found for this slug under adapters/".to_owned()
            }))
        });
    }
    loaded
}

/// A Cámara adapter config to borrow the HTTP client setup from.
fn discovery_base_config(root: &Path) -> Result<SourceConfig> {
    let directory = root.join("adapters").join(DISCOVERY_ADAPTER_DIR);
    let mut paths: Vec<PathBuf> = fs::read_dir(&directory)
        .with_context(|| format!("failed to read {}", directory.display()))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| {
            path.extension().is_some_and(|e| e == "json")
                && !path
                    .file_name()
                    .is_some_and(|n| n.to_string_lossy().starts_with('_'))
        })
        .collect();
    paths.sort();
    for path in paths {
        if let Ok(config) = load_config(&path) {
            return Ok(config);
        }
    }
    bail!("no loadable adapter under {}", directory.display())
}

fn fetch_index(
    base: &SourceConfig,
    url: &str,
    access: &dyn SourceAccess,
    pacer: &mut Pacer,
) -> Result<Vec<u8>> {
    let mut config = base.clone();
    config.source_url = url
        .parse()
        .with_context(|| format!("invalid index URL {url}"))?;
    config.source_format = SourceFormat::Html;
    pacer.wait(&host_of(&config));
    Ok(access.download(&config)?.bytes)
}

struct Discovery {
    parsed: usize,
    errors: Vec<IndexError>,
    discovered: Vec<IndexEntry>,
    missing: Option<Vec<MissingFromIndex>>,
}

fn discover(
    root: &Path,
    access: &dyn SourceAccess,
    pacer: &mut Pacer,
    committed: &BTreeMap<String, String>,
) -> Result<Discovery> {
    let base = discovery_base_config(root)?;
    let mut entries = Vec::new();
    let mut errors = Vec::new();
    for (kind, url) in [
        (IndexKind::Laws, LAWS_INDEX_URL),
        (IndexKind::Regulations, REGULATIONS_INDEX_URL),
    ] {
        match fetch_index(&base, url, access, pacer) {
            Ok(bytes) => {
                let parsed = parse_index(&bytes, kind, url);
                if parsed.is_empty() {
                    errors.push(IndexError {
                        index: kind,
                        url: url.to_owned(),
                        message:
                            "index page parsed to zero entries; the page layout may have changed"
                                .to_owned(),
                    });
                }
                entries.extend(parsed);
            }
            Err(error) => errors.push(IndexError {
                index: kind,
                url: url.to_owned(),
                message: format!("{error:#}"),
            }),
        }
    }
    let (discovered, missing) = compare_with_index(&entries, committed);
    Ok(Discovery {
        parsed: entries.len(),
        // Absence from an index proves nothing when an index was unreadable.
        missing: errors.is_empty().then_some(missing),
        errors,
        discovered,
    })
}

fn select_slugs(
    root: &Path,
    args: &MonitorSourcesArgs,
    committed: &BTreeMap<String, String>,
) -> Result<Vec<String>> {
    let mut slugs: Vec<String> = if args.all {
        committed.keys().cloned().collect()
    } else {
        for slug in &args.slugs {
            if !root
                .join("corpus/mx")
                .join(slug)
                .join("source-manifest.json")
                .is_file()
            {
                bail!("no committed source manifest at corpus/mx/{slug}/source-manifest.json");
            }
        }
        args.slugs.clone()
    };
    slugs.sort();
    slugs.dedup();
    Ok(slugs)
}

fn build_report(
    checked_at: DateTime<Utc>,
    args: &MonitorSourcesArgs,
    mut instruments: Vec<InstrumentEntry>,
    discovery: Option<Discovery>,
) -> Report {
    instruments.sort_by(|a, b| a.slug.cmp(&b.slug));
    let mut status_counts: BTreeMap<Status, usize> =
        Status::ALL.iter().map(|status| (*status, 0)).collect();
    for entry in &instruments {
        *status_counts.entry(entry.status).or_insert(0) += 1;
    }
    let (index_entries_parsed, index_errors, discovered, missing_from_index) =
        discovery.map_or((None, None, None, None), |d| {
            (
                Some(d.parsed),
                Some(d.errors),
                Some(d.discovered),
                d.missing,
            )
        });
    Report {
        schema_version: REPORT_SCHEMA_VERSION,
        checked_at: checked_at.to_rfc3339_opts(SecondsFormat::Secs, true),
        options: Options {
            retain: args.retain,
            discover: args.discover,
            delay_ms: args.delay_ms,
        },
        annex_sources_checked: false,
        status_counts,
        instruments,
        index_entries_parsed,
        index_errors,
        discovered,
        missing_from_index,
    }
}

fn short(sha: &str) -> &str {
    sha.get(..12).unwrap_or(sha)
}

fn summary_lines(report: &Report) -> Vec<String> {
    let mut lines = vec![format!(
        "monitor-sources: {} instrument(s)",
        report.instruments.len()
    )];
    lines.push(
        Status::ALL
            .iter()
            .map(|status| {
                format!(
                    "{}={}",
                    status.name(),
                    report.status_counts.get(status).copied().unwrap_or(0)
                )
            })
            .collect::<Vec<_>>()
            .join(" "),
    );
    for entry in &report.instruments {
        let observed_sha = entry
            .observed
            .as_ref()
            .and_then(|o| o.sha256.as_deref())
            .unwrap_or("?");
        match entry.status {
            Status::Changed => lines.push(format!(
                "changed: {} sha256 {} -> {}{}",
                entry.slug,
                entry.recorded.as_ref().map_or("?", |r| short(&r.sha256)),
                short(observed_sha),
                entry
                    .retained_path
                    .as_ref()
                    .map_or_else(String::new, |p| format!(" (saved {p})")),
            )),
            Status::Unreachable | Status::NotChecked => lines.push(format!(
                "{}: {}: {}",
                entry.status.name(),
                entry.slug,
                entry.detail.as_deref().unwrap_or("")
            )),
            Status::UnchangedByValidators | Status::UnchangedByHash => {}
        }
    }
    if let Some(parsed) = report.index_entries_parsed {
        lines.push(format!(
            "index entries parsed: {parsed}; not in corpus: {}; committed but missing from index: {}",
            report.discovered.as_ref().map_or(0, Vec::len),
            report
                .missing_from_index
                .as_ref()
                .map_or_else(|| "unknown".to_owned(), |m| m.len().to_string()),
        ));
    }
    for error in report.index_errors.iter().flatten() {
        lines.push(format!("index error: {}: {}", error.url, error.message));
    }
    for entry in report.discovered.iter().flatten() {
        lines.push(format!(
            "discovered: {} | {}{}{}",
            entry.title,
            entry.pdf_url,
            entry
                .last_reform
                .as_ref()
                .map_or_else(String::new, |r| format!(" | {r}")),
            if entry.abrogated {
                " | abrogated per index"
            } else {
                ""
            },
        ));
    }
    for missing in report.missing_from_index.iter().flatten() {
        lines.push(format!(
            "missing_from_index: {} {}",
            missing.slug, missing.official_url
        ));
    }
    lines
}

pub(crate) fn run(root: &Path, args: &MonitorSourcesArgs) -> Result<()> {
    if args.slugs.is_empty() && !args.all && !args.discover {
        bail!("name at least one slug, or pass --all and/or --discover");
    }
    let report = monitor(root, args, &LiveAccess, Utc::now())?;
    for line in summary_lines(&report) {
        println!("{line}");
    }
    println!("report: {REPORT_PATH}");
    if args.strict && report.needs_attention() {
        bail!("source monitor found changes, new or unreachable sources; see {REPORT_PATH}");
    }
    Ok(())
}

fn monitor(
    root: &Path,
    args: &MonitorSourcesArgs,
    access: &dyn SourceAccess,
    checked_at: DateTime<Utc>,
) -> Result<Report> {
    let committed = committed_urls(root)?;
    let slugs = select_slugs(root, args, &committed)?;
    let adapters = load_adapters(root, &slugs);
    let settings = Settings {
        retain: args.retain,
        delay: Duration::from_millis(args.delay_ms),
    };
    let mut pacer = Pacer::new(settings.delay);
    let mut instruments = Vec::with_capacity(slugs.len());
    for slug in &slugs {
        let config = &adapters[slug];
        let entry = check_instrument(root, slug, config, &settings, access, &mut pacer);
        eprintln!("{}: {}", slug, entry.status.name());
        instruments.push(entry);
    }
    let discovery = if args.discover {
        Some(discover(root, access, &mut pacer, &committed)?)
    } else {
        None
    };
    let report = build_report(checked_at, args, instruments, discovery);
    write_report(root, &report)?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use std::cell::RefCell;

    const SCHEMA: &str = include_str!("../../../schemas/source-monitor-report-v1.schema.json");
    const LAWS_SAMPLE: &[u8] = include_bytes!("../../../fixtures/monitor/index-laws-sample.htm");
    const REGS_SAMPLE: &[u8] =
        include_bytes!("../../../fixtures/monitor/index-regulations-sample.htm");

    fn recorded(etag: Option<&str>, last_modified: Option<&str>, length: Option<u64>) -> Recorded {
        Recorded {
            sha256: "a".repeat(64),
            etag: etag.map(str::to_owned),
            last_modified: last_modified.map(str::to_owned),
            content_length: length,
            retrieved_at: "2026-01-01T00:00:00Z".to_owned(),
        }
    }

    fn probe(
        status: u16,
        etag: Option<&str>,
        last_modified: Option<&str>,
        length: Option<u64>,
    ) -> SourceProbe {
        SourceProbe {
            http_status: status,
            etag: etag.map(str::to_owned),
            last_modified: last_modified.map(str::to_owned),
            content_length: length,
        }
    }

    #[test]
    fn validators_match_when_every_recorded_validator_is_equal() {
        let rec = recorded(None, Some("Mon, 01 Jan 2026"), Some(10));
        let p = probe(200, None, Some("Mon, 01 Jan 2026"), Some(10));
        assert_eq!(compare_validators(&rec, Some(&p)), ValidatorVerdict::Match);
        // A validator absent from the manifest is not compared.
        let rec = recorded(None, Some("x"), None);
        let p = probe(200, Some("\"new\""), Some("x"), Some(99));
        assert_eq!(compare_validators(&rec, Some(&p)), ValidatorVerdict::Match);
    }

    #[test]
    fn any_differing_validator_or_missing_observation_forces_download() {
        let rec = recorded(Some("\"a\""), Some("x"), Some(10));
        let p = probe(200, Some("\"b\""), Some("x"), Some(10));
        assert_eq!(
            compare_validators(&rec, Some(&p)),
            ValidatorVerdict::Differs(vec!["etag"])
        );
        let p = probe(200, Some("\"a\""), None, Some(11));
        assert_eq!(
            compare_validators(&rec, Some(&p)),
            ValidatorVerdict::Differs(vec!["last_modified", "content_length"])
        );
    }

    #[test]
    fn content_length_alone_never_settles_unchanged() {
        let rec = recorded(None, None, Some(10));
        let p = probe(200, None, None, Some(10));
        assert!(matches!(
            compare_validators(&rec, Some(&p)),
            ValidatorVerdict::Insufficient(_)
        ));
        assert!(matches!(
            compare_validators(&rec, None),
            ValidatorVerdict::Insufficient(_)
        ));
        let rec = recorded(None, Some("x"), None);
        let p = probe(405, None, Some("x"), None);
        assert!(matches!(
            compare_validators(&rec, Some(&p)),
            ValidatorVerdict::Insufficient(_)
        ));
    }

    #[test]
    fn hash_comparison_classifies_status() {
        let a = "a".repeat(64);
        assert_eq!(
            classify_hash(&a, &a.to_uppercase()),
            Status::UnchangedByHash
        );
        assert_eq!(classify_hash(&a, &"b".repeat(64)), Status::Changed);
    }

    #[test]
    fn retain_decision_never_targets_the_retained_path_for_changed_bytes() {
        use RetainAction::{AlreadyRetained, None as Nothing, SaveForReview, WriteRetained};
        assert_eq!(retain_decision(true, true, false), WriteRetained);
        assert_eq!(retain_decision(true, true, true), AlreadyRetained);
        assert_eq!(retain_decision(false, true, false), Nothing);
        assert_eq!(retain_decision(false, true, true), Nothing);
        for retain in [true, false] {
            for exists in [true, false] {
                assert_eq!(retain_decision(retain, false, exists), SaveForReview);
            }
        }
    }

    #[test]
    fn pacing_waits_only_for_the_unelapsed_part_of_the_delay() {
        let delay = Duration::from_secs(1);
        assert_eq!(remaining_delay(None, delay), Duration::ZERO);
        assert_eq!(
            remaining_delay(Some(Duration::from_millis(400)), delay),
            Duration::from_millis(600)
        );
        assert_eq!(
            remaining_delay(Some(Duration::from_secs(5)), delay),
            Duration::ZERO
        );
    }

    #[test]
    fn href_resolution_handles_relative_absolute_and_parent_links() {
        let base = "https://www.diputados.gob.mx/LeyesBiblio/index.htm";
        assert_eq!(
            resolve_href(base, "pdf/LFT.pdf"),
            "https://www.diputados.gob.mx/LeyesBiblio/pdf/LFT.pdf"
        );
        assert_eq!(
            resolve_href(base, "../Otro/A b.pdf#x"),
            "https://www.diputados.gob.mx/Otro/A%20b.pdf"
        );
        assert_eq!(
            resolve_href(base, "/x/y.pdf"),
            "https://www.diputados.gob.mx/x/y.pdf"
        );
        assert_eq!(
            resolve_href(base, "http://e.test/a.pdf"),
            "http://e.test/a.pdf"
        );
    }

    #[test]
    fn laws_index_sample_parses_title_pdf_and_last_reform() {
        let entries = parse_index(LAWS_SAMPLE, IndexKind::Laws, LAWS_INDEX_URL);
        let titles: Vec<&str> = entries.iter().map(|e| e.title.as_str()).collect();
        assert_eq!(titles.len(), 5, "{titles:?}");
        assert!(
            titles.contains(&"CÓDIGO Civil Federal"),
            "decodes ISO-8859-1"
        );
        let lft = entries
            .iter()
            .find(|e| e.title == "LEY Federal del Trabajo")
            .unwrap();
        assert_eq!(
            lft.pdf_url,
            "https://www.diputados.gob.mx/LeyesBiblio/pdf/LFT.pdf"
        );
        assert_eq!(lft.last_reform.as_deref(), Some("DOF 14/05/2026"));
        let lfcpq = entries
            .iter()
            .find(|e| e.pdf_url.ends_with("/LFCPQ.pdf"))
            .unwrap();
        assert!(
            lfcpq
                .title
                .starts_with("LEY Federal para el Control de Precursores")
        );
        assert!(lfcpq.title.ends_with("Tabletas y/o Comprimidos"));
        assert!(!lfcpq.title.contains("  ") && !lfcpq.title.contains('\n'));
        // The official copy, never the mobile pdf_mov copy; abrogated rows are kept.
        assert!(entries.iter().all(|e| !e.pdf_url.contains("pdf_mov")));
        assert!(
            entries
                .iter()
                .any(|e| e.pdf_url.ends_with("/abro/levct/LEVCT_abro.pdf"))
        );
        let renamed = entries
            .iter()
            .find(|e| e.title.starts_with("LEY de Ascensos"))
            .unwrap();
        // The "(Antes &quot;...&quot;)" note below the bold title is not part of it.
        assert_eq!(
            renamed.title,
            "LEY de Ascensos y Recompensas del Ejército, Fuerza Aérea y Guardia Nacional"
        );
    }

    #[test]
    fn regulations_index_sample_uses_the_pdf_column_and_reform_text() {
        let entries = parse_index(REGS_SAMPLE, IndexKind::Regulations, REGULATIONS_INDEX_URL);
        assert_eq!(entries.len(), 5, "{entries:?}");
        let aduanera = &entries[0];
        assert_eq!(aduanera.title, "REGLAMENTO de la Ley Aduanera");
        assert_eq!(
            aduanera.pdf_url,
            "https://www.diputados.gob.mx/LeyesBiblio/regley/Reg_LAdua.pdf"
        );
        assert_eq!(
            aduanera.last_reform.as_deref(),
            Some("Reforma DOF 23/02/2026")
        );
        assert_eq!(entries[1].last_reform, None);
        // Historical PDFs in the same row are not the current text.
        let militar = &entries[2];
        assert!(
            militar.pdf_url.ends_with("/regley/Reg_LSM_060647.pdf"),
            "{militar:?}"
        );
        // A title wrapped in a styled span, after an empty bold paragraph.
        let naval = &entries[3];
        assert_eq!(naval.title, "REGLAMENTO de la Ley de Educación Naval");
        assert!(!naval.abrogated);
        // The "Reglamento Abrogado" note is not part of the title but is kept as a flag.
        let geotermica = &entries[4];
        assert_eq!(
            geotermica.title,
            "REGLAMENTO de la Ley de Energía Geotérmica"
        );
        assert!(geotermica.abrogated);
        assert!(entries[..4].iter().all(|e| !e.abrogated));
    }

    #[test]
    fn header_and_navigation_rows_are_ignored() {
        let html = b"<table><tr><td>No.</td><td><b>Ley</b></td></tr>\
            <tr><td>1</td><td><b>LEY X</b></td><td>-</td>\
            <td><a href=\"pdf_mov/X.pdf\">m</a></td></tr></table>";
        assert_eq!(parse_index(html, IndexKind::Laws, LAWS_INDEX_URL).len(), 0);
    }

    fn committed(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(s, u)| ((*s).to_owned(), (*u).to_owned()))
            .collect()
    }

    #[test]
    fn index_comparison_is_case_insensitive_and_scoped_to_listed_directories() {
        let entries = parse_index(LAWS_SAMPLE, IndexKind::Laws, LAWS_INDEX_URL);
        let committed = committed(&[
            (
                "lft",
                "https://www.diputados.gob.mx/LeyesBiblio/PDF/lft.PDF",
            ),
            (
                "ccf",
                "https://www.diputados.gob.mx/LeyesBiblio/pdf/CCF.pdf",
            ),
            (
                "gone",
                "https://www.diputados.gob.mx/LeyesBiblio/pdf/GONE.pdf",
            ),
            (
                "abro",
                "https://www.diputados.gob.mx/LeyesBiblio/abro/lftr/LFTR_abro.pdf",
            ),
            ("cnbv", "https://www.cnbv.gob.mx/Normatividad/x.pdf"),
        ]);
        let (discovered, missing) = compare_with_index(&entries, &committed);
        let urls: Vec<&str> = discovered.iter().map(|e| e.pdf_url.as_str()).collect();
        assert_eq!(discovered.len(), 3, "{urls:?}");
        assert!(
            urls.iter()
                .all(|u| !u.ends_with("/LFT.pdf") && !u.ends_with("/CCF.pdf"))
        );
        // Other hosts and directories the index does not list from are not judged.
        assert_eq!(
            missing,
            vec![MissingFromIndex {
                slug: "gone".to_owned(),
                official_url: "https://www.diputados.gob.mx/LeyesBiblio/pdf/GONE.pdf".to_owned(),
            }]
        );
    }

    // ----- end-to-end with a fake network over a synthetic root -----------

    struct FakeAccess {
        /// Bytes served for each URL (lowercased). Missing -> HTTP 404 error.
        files: HashMap<String, Vec<u8>>,
        probes: HashMap<String, SourceProbe>,
        requests: RefCell<Vec<String>>,
    }

    impl SourceAccess for FakeAccess {
        fn probe(&self, config: &SourceConfig) -> Result<SourceProbe> {
            self.requests
                .borrow_mut()
                .push(format!("HEAD {}", config.source_url));
            self.probes
                .get(&config.source_url.as_str().to_ascii_lowercase())
                .cloned()
                .context("probe refused")
        }

        fn download(&self, config: &SourceConfig) -> Result<Acquisition> {
            self.requests
                .borrow_mut()
                .push(format!("GET {}", config.source_url));
            let bytes = self
                .files
                .get(&config.source_url.as_str().to_ascii_lowercase())
                .cloned()
                .context("source returned HTTP 404 Not Found")?;
            let manifest: SourceManifest = serde_json::from_value(manifest_json(
                config.source_url.as_str(),
                &sha256_hex(&bytes),
                None,
                Some("x"),
            ))
            .unwrap();
            Ok(Acquisition { bytes, manifest })
        }
    }

    fn manifest_json(url: &str, sha: &str, etag: Option<&str>, modified: Option<&str>) -> Value {
        json!({
            "schema_version": "1.0.0", "instrument_id": "urn:test", "operational_source": "x",
            "formal_publication_source": "dof", "publisher": "p", "official_url": url,
            "reference_url": null, "retrieved_at": "2026-01-01T00:00:00Z", "http_status": 200,
            "content_type": null, "content_length": 5, "etag": etag, "last_modified": modified,
            "source_sha256": sha, "extracted_text_sha256": null, "extraction_tool": null,
            "parser_version": "0", "schema_version_used": "1.0.0", "resulting_git_commit": null
        })
    }

    fn adapter_json(slug: &str, url: &str) -> Value {
        json!({
            "slug": slug, "instrument_id": format!("urn:test:{slug}"),
            "instrument_type": "law", "parser": "diputados", "official_title": slug,
            "short_name": slug, "operational_source": "camara_de_diputados",
            "source_url": url, "reference_url": null, "publisher": "p",
            "expected_articles": 1, "expected_transitories": 0,
            "formal_publication_urls": {}
        })
    }

    struct Fixture {
        root: tempfile::TempDir,
        fake: FakeAccess,
    }

    const URL_A: &str = "https://www.diputados.gob.mx/LeyesBiblio/pdf/AAA.pdf";
    const URL_B: &str = "https://www.diputados.gob.mx/LeyesBiblio/pdf/BBB.pdf";
    const URL_C: &str = "https://www.diputados.gob.mx/LeyesBiblio/pdf/CCC.pdf";
    const URL_D: &str = "https://www.diputados.gob.mx/LeyesBiblio/pdf/DDD.pdf";

    fn fixture() -> Fixture {
        let root = tempfile::tempdir().unwrap();
        let mut files = HashMap::new();
        let mut probes = HashMap::new();
        let specs: [(&str, &str, &[u8], &[u8]); 4] = [
            // slug, url, bytes recorded in the manifest, bytes served now
            ("aaa", URL_A, b"%PDF-a", b"%PDF-a"),
            ("bbb", URL_B, b"%PDF-b", b"%PDF-b"),
            ("ccc", URL_C, b"%PDF-c", b"%PDF-c2"),
            ("ddd", URL_D, b"%PDF-d", b""),
        ];
        for (slug, url, recorded_bytes, served) in specs {
            let corpus = root.path().join("corpus/mx").join(slug);
            fs::create_dir_all(&corpus).unwrap();
            let modified = (slug != "bbb").then_some("Mon, 01 Jan 2026 00:00:00 GMT");
            fs::write(
                corpus.join("source-manifest.json"),
                serde_json::to_vec_pretty(&manifest_json(
                    url,
                    &sha256_hex(recorded_bytes),
                    None,
                    modified,
                ))
                .unwrap(),
            )
            .unwrap();
            fs::write(corpus.join("provisions.json"), b"[1, 2, 3]").unwrap();
            let adapters = root.path().join("adapters/diputados");
            fs::create_dir_all(&adapters).unwrap();
            fs::write(
                adapters.join(format!("{slug}.json")),
                serde_json::to_vec(&adapter_json(slug, url)).unwrap(),
            )
            .unwrap();
            if !served.is_empty() {
                files.insert(url.to_ascii_lowercase(), served.to_vec());
            }
            probes.insert(
                url.to_ascii_lowercase(),
                probe(200, None, Some("Mon, 01 Jan 2026 00:00:00 GMT"), Some(5)),
            );
        }
        // aaa has the recorded Last-Modified and an unchanged probe; ccc has a
        // newer Last-Modified; bbb has no recorded validator at all; ddd's
        // probe differs and its download then fails.
        probes.insert(
            URL_D.to_ascii_lowercase(),
            probe(200, None, Some("Wed, 03 Mar 2026 00:00:00 GMT"), Some(5)),
        );
        probes.insert(
            URL_C.to_ascii_lowercase(),
            probe(200, None, Some("Tue, 02 Feb 2026 00:00:00 GMT"), Some(6)),
        );
        // The synthetic Cámara index lists aaa and a law the corpus lacks.
        let index = b"<table><tr><td>1</td><td><b>LEY Nueva</b></td><td>DOF 01/01/2026</td>\
            <td><a href=\"pdf/NUEVA.pdf\">x</a></td></tr>\
            <tr><td>2</td><td><b>LEY A</b></td><td></td><td><a href=\"pdf/aaa.pdf\">x</a></td></tr>\
            </table>";
        files.insert(LAWS_INDEX_URL.to_ascii_lowercase(), index.to_vec());
        files.insert(REGULATIONS_INDEX_URL.to_ascii_lowercase(), index.to_vec());
        Fixture {
            root,
            fake: FakeAccess {
                files,
                probes,
                requests: RefCell::new(Vec::new()),
            },
        }
    }

    fn args(slugs: &[&str], retain: bool, discover: bool) -> MonitorSourcesArgs {
        MonitorSourcesArgs {
            slugs: slugs.iter().map(|s| (*s).to_owned()).collect(),
            all: false,
            retain,
            discover,
            delay_ms: 0,
            strict: false,
        }
    }

    fn snapshot(directory: &Path) -> BTreeMap<String, Vec<u8>> {
        fn walk(base: &Path, path: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
            for entry in fs::read_dir(path).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    walk(base, &path, out);
                } else {
                    out.insert(relative(base, &path), fs::read(&path).unwrap());
                }
            }
        }
        let mut out = BTreeMap::new();
        walk(directory, directory, &mut out);
        out
    }

    fn status_of(report: &Report, slug: &str) -> Status {
        report
            .instruments
            .iter()
            .find(|e| e.slug == slug)
            .unwrap()
            .status
    }

    fn validate_against_schema(report: &Report) {
        let schema: Value = serde_json::from_str(SCHEMA).unwrap();
        let value = serde_json::to_value(report).unwrap();
        jsonschema::validator_for(&schema)
            .unwrap()
            .validate(&value)
            .unwrap_or_else(|error| panic!("report violates schema: {error}"));
    }

    #[test]
    fn statuses_without_retain_and_corpus_stays_untouched() {
        let fx = fixture();
        let before = snapshot(&fx.root.path().join("corpus"));
        let report = monitor(
            fx.root.path(),
            &args(&["ddd", "ccc", "bbb", "aaa"], false, true),
            &fx.fake,
            Utc::now(),
        )
        .unwrap();
        assert_eq!(status_of(&report, "aaa"), Status::UnchangedByValidators);
        assert_eq!(status_of(&report, "bbb"), Status::UnchangedByHash);
        assert_eq!(status_of(&report, "ccc"), Status::Changed);
        assert_eq!(status_of(&report, "ddd"), Status::Unreachable);
        let slugs: Vec<&str> = report.instruments.iter().map(|e| e.slug.as_str()).collect();
        assert_eq!(slugs, ["aaa", "bbb", "ccc", "ddd"]);
        // aaa was settled by its probe alone: no GET for it.
        assert!(!fx.fake.requests.borrow().contains(&format!("GET {URL_A}")));
        // The changed bytes are kept for review, not at the retained path.
        let ccc = report.instruments.iter().find(|e| e.slug == "ccc").unwrap();
        assert_eq!(ccc.retained, Some(RetainOutcome::SavedForReview));
        let saved = ccc.retained_path.as_ref().unwrap();
        assert!(
            saved.starts_with(".work/monitor/ccc/")
                && Path::new(saved).extension().is_some_and(|e| e == "pdf")
        );
        assert_eq!(fs::read(fx.root.path().join(saved)).unwrap(), b"%PDF-c2");
        assert!(!fx.root.path().join(".work/ccc/ccc.pdf").exists());
        // Discovery: the synthetic index lists one law the corpus lacks.
        let discovered = report.discovered.as_ref().unwrap();
        assert_eq!(
            discovered.len(),
            1,
            "both fake indexes list it; reported once"
        );
        assert!(discovered.iter().all(|e| e.pdf_url.ends_with("/NUEVA.pdf")));
        let missing: Vec<&str> = report
            .missing_from_index
            .as_ref()
            .unwrap()
            .iter()
            .map(|m| m.slug.as_str())
            .collect();
        assert_eq!(missing, ["bbb", "ccc", "ddd"]);
        assert!(report.needs_attention());
        validate_against_schema(&report);
        let written: Value =
            serde_json::from_slice(&fs::read(fx.root.path().join(REPORT_PATH)).unwrap()).unwrap();
        assert_eq!(written["schema_version"], "1.0.0");
        assert_eq!(written["annex_sources_checked"], false);
        assert_eq!(snapshot(&fx.root.path().join("corpus")), before);
    }

    #[test]
    fn retain_restores_missing_source_only_when_hash_matches_and_never_overwrites() {
        let fx = fixture();
        let before = snapshot(&fx.root.path().join("corpus"));
        let retained = fx.root.path().join(".work/aaa/aaa.pdf");
        let existing = fx.root.path().join(".work/bbb/bbb.pdf");
        fs::create_dir_all(existing.parent().unwrap()).unwrap();
        fs::write(&existing, b"keep me").unwrap();

        let report = monitor(
            fx.root.path(),
            &args(&["aaa", "bbb", "ccc"], true, false),
            &fx.fake,
            Utc::now(),
        )
        .unwrap();
        // --retain always downloads: no HEAD requests at all.
        assert!(
            fx.fake
                .requests
                .borrow()
                .iter()
                .all(|r| r.starts_with("GET "))
        );
        assert_eq!(status_of(&report, "aaa"), Status::UnchangedByHash);
        assert_eq!(fs::read(&retained).unwrap(), b"%PDF-a");
        assert_eq!(fs::read(&existing).unwrap(), b"keep me");
        assert!(!fx.root.path().join(".work/ccc/ccc.pdf").exists());
        let outcome = |slug: &str| {
            report
                .instruments
                .iter()
                .find(|e| e.slug == slug)
                .unwrap()
                .retained
        };
        assert_eq!(outcome("aaa"), Some(RetainOutcome::Written));
        assert_eq!(outcome("bbb"), Some(RetainOutcome::AlreadyPresent));
        assert_eq!(outcome("ccc"), Some(RetainOutcome::SavedForReview));
        assert!(report.discovered.is_none());
        validate_against_schema(&report);
        assert_eq!(snapshot(&fx.root.path().join("corpus")), before);
    }

    #[test]
    fn an_unreadable_manifest_or_missing_adapter_is_not_checked_and_the_run_continues() {
        let fx = fixture();
        fs::write(
            fx.root.path().join("corpus/mx/aaa/source-manifest.json"),
            b"{not json",
        )
        .unwrap();
        fs::remove_file(fx.root.path().join("adapters/diputados/bbb.json")).unwrap();
        let report = monitor(
            fx.root.path(),
            &args(&["aaa", "bbb", "ccc"], false, false),
            &fx.fake,
            Utc::now(),
        )
        .unwrap();
        assert_eq!(status_of(&report, "aaa"), Status::NotChecked);
        assert_eq!(status_of(&report, "bbb"), Status::NotChecked);
        assert_eq!(status_of(&report, "ccc"), Status::Changed);
        validate_against_schema(&report);
    }

    #[test]
    fn write_helpers_refuse_corpus_paths_and_existing_files() {
        let root = tempfile::tempdir().unwrap();
        let in_corpus = root.path().join("corpus/mx/x/file.pdf");
        assert!(write_new_file(root.path(), &in_corpus, b"x").is_err());
        assert!(!in_corpus.exists());
        let target = root.path().join(".work/x/file.pdf");
        assert!(write_new_file(root.path(), &target, b"one").unwrap());
        assert!(!write_new_file(root.path(), &target, b"two").unwrap());
        assert_eq!(fs::read(&target).unwrap(), b"one");
        let leftovers: Vec<_> = fs::read_dir(target.parent().unwrap()).unwrap().collect();
        assert_eq!(leftovers.len(), 1, "temporary file is removed");
    }

    #[test]
    fn an_unreadable_index_suppresses_missing_from_index_and_is_reported() {
        let mut fx = fixture();
        fx.fake
            .files
            .remove(&REGULATIONS_INDEX_URL.to_ascii_lowercase());
        let report = monitor(
            fx.root.path(),
            &args(&[], false, true),
            &fx.fake,
            Utc::now(),
        )
        .unwrap();
        assert!(report.instruments.is_empty());
        assert!(report.missing_from_index.is_none());
        assert_eq!(report.index_errors.as_ref().unwrap().len(), 1);
        validate_against_schema(&report);
    }

    #[test]
    fn status_enum_matches_schema() {
        let schema: Value = serde_json::from_str(SCHEMA).unwrap();
        let mut listed: Vec<String> = schema
            .pointer("/$defs/status/enum")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_owned())
            .collect();
        let mut actual: Vec<String> = Status::ALL.iter().map(|s| s.name().to_owned()).collect();
        listed.sort();
        actual.sort();
        assert_eq!(listed, actual);
        for status in Status::ALL {
            assert_eq!(
                serde_json::to_value(status).unwrap(),
                Value::String(status.name().to_owned())
            );
        }
    }

    #[test]
    fn report_serialisation_is_deterministic() {
        let fx = fixture();
        let at: DateTime<Utc> = "2026-11-12T00:00:00Z".parse().unwrap();
        let first = monitor(
            fx.root.path(),
            &args(&["aaa", "ccc"], false, false),
            &fx.fake,
            at,
        )
        .unwrap();
        let second = monitor(
            fx.root.path(),
            &args(&["ccc", "aaa"], false, false),
            &fx.fake,
            at,
        )
        .unwrap();
        assert_eq!(
            serde_json::to_string_pretty(&first).unwrap(),
            serde_json::to_string_pretty(&second).unwrap()
        );
        assert_eq!(first.checked_at, "2026-11-12T00:00:00Z");
    }
}
