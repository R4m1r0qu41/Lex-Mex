//! `audit-qualifiers`: read-only linker evidence. For every resolved edge that
//! cites fractions, check each cited fraction against the target provision's
//! text. Reports go to `.work/`; nothing under `corpus/` is written and no
//! reference edge is changed.
use std::{collections::HashMap, fs, path::Path};

use anyhow::{Context, Result, bail};
use clap::Args;
use lex_core::ReferenceQualifier;
use lex_link::qualifier_audit::{FractionOutcome, audit_fraction_qualifiers};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const SCHEMA_VERSION: &str = "1.0.0";

#[derive(Debug, Args)]
pub(crate) struct AuditQualifiersArgs {
    /// Committed instrument slug under corpus/mx.
    #[arg(required_unless_present = "all", conflicts_with = "all")]
    slug: Option<String>,
    /// Audit every corpus/mx instrument that has references.json and write
    /// .work/qualifier-audit-summary.json.
    #[arg(long)]
    all: bool,
}

/// Mirror of `lex_link::qualifier_audit::FractionOutcome` for report output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
enum Outcome {
    FoundAtParagraphStart,
    FoundMidParagraph,
    Absent,
}

impl From<FractionOutcome> for Outcome {
    fn from(outcome: FractionOutcome) -> Self {
        match outcome {
            FractionOutcome::FoundAtParagraphStart => Self::FoundAtParagraphStart,
            FractionOutcome::FoundMidParagraph => Self::FoundMidParagraph,
            FractionOutcome::Absent => Self::Absent,
        }
    }
}

#[derive(Debug, Deserialize)]
struct EdgeRecord {
    id: String,
    source_provision_id: String,
    target_instrument_id: String,
    target_provision_id: String,
    resolution_status: String,
    #[serde(default)]
    resolution_rule: Option<Value>,
    #[serde(default)]
    qualifiers: Vec<ReferenceQualifier>,
}

#[derive(Debug, Deserialize)]
struct ProvisionRecord {
    id: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct InstrumentRecord {
    id: String,
}

#[derive(Debug, Serialize)]
struct Finding {
    edge_id: String,
    source_provision_id: String,
    qualifier_text: String,
    label: String,
    target_provision_id: String,
    resolution_rule: Option<Value>,
    outcome: Outcome,
}

#[derive(Debug, Default, Clone, Serialize)]
struct Counts {
    found_at_paragraph_start: usize,
    found_mid_paragraph: usize,
    absent: usize,
    /// Fraction qualifiers from which no label could be parsed (for example
    /// OCR forms such as `ll`).
    qualifiers_without_labels: usize,
    /// Resolved edges with fraction qualifiers whose target provision could not
    /// be loaded, so no outcome exists for them.
    edges_target_not_loaded: usize,
}

impl Counts {
    fn labels(&self) -> usize {
        self.found_at_paragraph_start + self.found_mid_paragraph + self.absent
    }

    fn add(&mut self, other: &Counts) {
        self.found_at_paragraph_start += other.found_at_paragraph_start;
        self.found_mid_paragraph += other.found_mid_paragraph;
        self.absent += other.absent;
        self.qualifiers_without_labels += other.qualifiers_without_labels;
        self.edges_target_not_loaded += other.edges_target_not_loaded;
    }

    fn line(&self) -> String {
        format!(
            "labels={} found_at_paragraph_start={} found_mid_paragraph={} absent={} \
             qualifiers_without_labels={} edges_target_not_loaded={}",
            self.labels(),
            self.found_at_paragraph_start,
            self.found_mid_paragraph,
            self.absent,
            self.qualifiers_without_labels,
            self.edges_target_not_loaded
        )
    }
}

#[derive(Debug, Serialize)]
struct InstrumentReport {
    schema_version: &'static str,
    instrument_id: String,
    counts: Counts,
    findings: Vec<Finding>,
}

#[derive(Debug, Serialize)]
struct SummaryEntry {
    slug: String,
    instrument_id: String,
    counts: Counts,
}

#[derive(Debug, Serialize)]
struct Summary {
    schema_version: &'static str,
    totals: Counts,
    instruments: Vec<SummaryEntry>,
}

/// Lazily loaded provision text per target instrument.
struct TargetIndex {
    /// Instrument id to corpus directory name.
    directories: HashMap<String, String>,
    loaded: HashMap<String, HashMap<String, String>>,
}

impl TargetIndex {
    fn new(root: &Path) -> Result<Self> {
        let mut directories = HashMap::new();
        for slug in directory_slugs(root, "instrument.json")? {
            let path = root.join("corpus/mx").join(&slug).join("instrument.json");
            let instrument: InstrumentRecord = read_json(&path)?;
            directories.insert(instrument.id, slug);
        }
        Ok(Self {
            directories,
            loaded: HashMap::new(),
        })
    }

    fn text(&mut self, root: &Path, instrument_id: &str, provision_id: &str) -> Option<&str> {
        if !self.loaded.contains_key(instrument_id) {
            let provisions = self
                .directories
                .get(instrument_id)
                .and_then(|slug| {
                    let path = root.join("corpus/mx").join(slug).join("provisions.json");
                    read_json::<Vec<ProvisionRecord>>(&path).ok()
                })
                .unwrap_or_default();
            self.loaded.insert(
                instrument_id.to_owned(),
                provisions.into_iter().map(|p| (p.id, p.text)).collect(),
            );
        }
        self.loaded
            .get(instrument_id)?
            .get(provision_id)
            .map(String::as_str)
    }
}

pub(crate) fn run(root: &Path, args: AuditQualifiersArgs) -> Result<()> {
    let slugs = if args.all {
        directory_slugs(root, "references.json")?
    } else {
        let slug = args.slug.context("missing instrument slug")?;
        if !root.join("corpus/mx").join(&slug).is_dir() {
            bail!("no committed instrument corpus/mx/{slug}");
        }
        vec![slug]
    };
    let mut targets = TargetIndex::new(root)?;
    let mut entries = Vec::with_capacity(slugs.len());
    for slug in &slugs {
        let report = audit_slug(root, slug, &mut targets)?;
        println!("{slug}: {}", report.counts.line());
        write_json(
            &root.join(".work").join(slug).join("qualifier-audit.json"),
            &report,
        )?;
        entries.push(SummaryEntry {
            slug: slug.clone(),
            instrument_id: report.instrument_id,
            counts: report.counts,
        });
    }
    let mut totals = Counts::default();
    for entry in &entries {
        totals.add(&entry.counts);
    }
    println!("totals: instruments={} {}", entries.len(), totals.line());
    if args.all {
        let summary = Summary {
            schema_version: SCHEMA_VERSION,
            totals,
            instruments: entries,
        };
        write_json(&root.join(".work/qualifier-audit-summary.json"), &summary)?;
    }
    Ok(())
}

fn audit_slug(root: &Path, slug: &str, targets: &mut TargetIndex) -> Result<InstrumentReport> {
    let directory = root.join("corpus/mx").join(slug);
    let instrument: InstrumentRecord = read_json(&directory.join("instrument.json"))?;
    let references = directory.join("references.json");
    let edges: Vec<EdgeRecord> = if references.is_file() {
        read_json(&references)?
    } else {
        Vec::new()
    };
    let mut counts = Counts::default();
    let mut findings = Vec::new();
    for edge in &edges {
        if edge.resolution_status != "resolved"
            || edge.target_provision_id.is_empty()
            || !edge
                .qualifiers
                .iter()
                .any(|q| q.qualifier_type == lex_core::ReferenceQualifierType::Fraction)
        {
            continue;
        }
        let Some(text) = targets.text(root, &edge.target_instrument_id, &edge.target_provision_id)
        else {
            counts.edges_target_not_loaded += 1;
            continue;
        };
        for qualifier in edge
            .qualifiers
            .iter()
            .filter(|q| q.qualifier_type == lex_core::ReferenceQualifierType::Fraction)
        {
            if lex_link::qualifier_audit::parse_fraction_labels(&qualifier.text).is_empty() {
                counts.qualifiers_without_labels += 1;
            }
        }
        for audit in audit_fraction_qualifiers(&edge.qualifiers, text) {
            let outcome = Outcome::from(audit.outcome);
            match outcome {
                Outcome::FoundAtParagraphStart => counts.found_at_paragraph_start += 1,
                Outcome::FoundMidParagraph => counts.found_mid_paragraph += 1,
                Outcome::Absent => counts.absent += 1,
            }
            if outcome != Outcome::FoundAtParagraphStart {
                findings.push(Finding {
                    edge_id: edge.id.clone(),
                    source_provision_id: edge.source_provision_id.clone(),
                    qualifier_text: audit.qualifier_text,
                    label: audit.label,
                    target_provision_id: edge.target_provision_id.clone(),
                    resolution_rule: edge.resolution_rule.clone(),
                    outcome,
                });
            }
        }
    }
    Ok(InstrumentReport {
        schema_version: SCHEMA_VERSION,
        instrument_id: instrument.id,
        counts,
        findings,
    })
}

/// Sorted corpus/mx directory names that contain `file`.
fn directory_slugs(root: &Path, file: &str) -> Result<Vec<String>> {
    let directory = root.join("corpus/mx");
    let mut slugs = Vec::new();
    for entry in fs::read_dir(&directory)
        .with_context(|| format!("failed to read {}", directory.display()))?
    {
        let path = entry?.path();
        if path.join(file).is_file()
            && let Some(name) = path.file_name().and_then(|name| name.to_str())
        {
            slugs.push(name.to_owned());
        }
    }
    slugs.sort();
    Ok(slugs)
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("failed to parse {}", path.display()))
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, bytes).with_context(|| format!("failed to write {}", path.display()))
}

#[cfg(test)]
mod qualifier_audit_tests {
    use std::{collections::BTreeSet, path::PathBuf};

    use serde_json::json;

    use super::*;

    fn write(root: &Path, relative: &str, value: &Value) {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
    }

    fn snapshot(root: &Path) -> BTreeSet<PathBuf> {
        fn walk(directory: &Path, root: &Path, out: &mut BTreeSet<PathBuf>) {
            for entry in fs::read_dir(directory).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    walk(&path, root, out);
                } else {
                    out.insert(path.strip_prefix(root).unwrap().to_path_buf());
                }
            }
        }
        let mut out = BTreeSet::new();
        walk(root, root, &mut out);
        out
    }

    fn edge(id: &str, target_instrument: &str, target: &str, text: &str, status: &str) -> Value {
        json!({
            "id": id,
            "source_provision_id": "urn:a:article:1",
            "target_instrument_id": target_instrument,
            "target_provision_id": target,
            "qualifiers": [{"qualifier_type": "fraction", "text": text}],
            "resolution_status": status,
            "resolution_rule": "own_instrument_marker"
        })
    }

    #[test]
    fn qualifier_audit_cross_instrument_writes_only_work() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        write(root, "corpus/mx/a/instrument.json", &json!({"id": "urn:a"}));
        write(root, "corpus/mx/b/instrument.json", &json!({"id": "urn:b"}));
        write(
            root,
            "corpus/mx/a/provisions.json",
            &json!([{"id": "urn:a:article:1", "text": "I. Uno\n\nII. Dos"}]),
        );
        write(
            root,
            "corpus/mx/b/provisions.json",
            &json!([
                {"id": "urn:b:article:7", "text": "Intro:\n\nI. Uno; II. Dos\n\nIII. Tres"},
                {"id": "urn:b:article:8", "text": "Nada"}
            ]),
        );
        write(
            root,
            "corpus/mx/a/references.json",
            &json!([
                edge(
                    "e1",
                    "urn:b",
                    "urn:b:article:7",
                    "fracciones I, II, III y IV",
                    "resolved"
                ),
                edge("e2", "urn:a", "urn:a:article:1", "fracción I", "resolved"),
                edge("e3", "urn:b", "urn:b:article:8", "fracción I", "unresolved"),
                edge("e4", "urn:b", "urn:b:article:99", "fracción I", "resolved"),
                edge("e5", "urn:b", "urn:b:article:8", "fracción ll", "resolved")
            ]),
        );
        write(root, "corpus/mx/b/references.json", &json!([]));
        let before = snapshot(root);

        run(
            root,
            AuditQualifiersArgs {
                slug: None,
                all: true,
            },
        )
        .unwrap();

        let after = snapshot(root);
        let added: Vec<_> = after.difference(&before).collect();
        assert_ne!(added.len(), 0);
        assert!(
            added.iter().all(|path| path.starts_with(".work")),
            "unexpected writes: {added:?}"
        );
        assert!(before.is_subset(&after));
        for kept in &before {
            assert!(after.contains(kept));
        }
        let report: Value = read_json(&root.join(".work/a/qualifier-audit.json")).unwrap();
        assert_eq!(report["schema_version"], "1.0.0");
        assert_eq!(report["instrument_id"], "urn:a");
        assert_eq!(report["counts"]["found_at_paragraph_start"], 3);
        assert_eq!(report["counts"]["found_mid_paragraph"], 1);
        assert_eq!(report["counts"]["absent"], 1);
        assert_eq!(report["counts"]["qualifiers_without_labels"], 1);
        assert_eq!(report["counts"]["edges_target_not_loaded"], 1);
        let findings = report["findings"].as_array().unwrap();
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0]["edge_id"], "e1");
        assert_eq!(findings[0]["label"], "II");
        assert_eq!(findings[0]["outcome"], "found_mid_paragraph");
        assert_eq!(findings[0]["target_provision_id"], "urn:b:article:7");
        assert_eq!(findings[1]["label"], "IV");
        assert_eq!(findings[1]["outcome"], "absent");
        let summary: Value = read_json(&root.join(".work/qualifier-audit-summary.json")).unwrap();
        assert_eq!(summary["totals"]["absent"], 1);
        assert_eq!(summary["instruments"].as_array().unwrap().len(), 2);
        assert!(root.join(".work/b/qualifier-audit.json").is_file());
    }
}
