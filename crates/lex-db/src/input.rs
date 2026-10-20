use crate::{
    contract::{DEBT_PATH, Handling, InputFile, validate_seed},
    digest,
};
use anyhow::{Context, Result, ensure};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    process::Command,
};

pub struct GitInput {
    root: PathBuf,
    pub commit: String,
}

impl GitInput {
    pub fn new(root: &Path, commit: &str) -> Result<Self> {
        ensure!(
            commit.len() == 40 && commit.bytes().all(|b| b.is_ascii_hexdigit()),
            "require full 40-character commit OID"
        );
        let git = Self {
            root: root.to_owned(),
            commit: commit.to_owned(),
        };
        ensure!(
            git.command(&["rev-parse", "--verify", &format!("{commit}^{{commit}}")])?
                .trim()
                == commit,
            "not a commit object"
        );
        Ok(git)
    }
    fn command(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.root)
            .args(args)
            .output()?;
        ensure!(
            output.status.success(),
            "git failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(String::from_utf8(output.stdout)?)
    }
    pub fn require_ancestor(&self, commit: &str) -> Result<()> {
        Self::new(&self.root, commit)?;
        let result = Command::new("git")
            .arg("-C")
            .arg(&self.root)
            .args(["merge-base", "--is-ancestor", commit, &self.commit])
            .status()?;
        ensure!(
            result.success(),
            "seed source commit is not an ancestor of input commit"
        );
        Ok(())
    }
    pub fn read(&self, path: &str) -> Result<Vec<u8>> {
        crate::contract::safe_path(path)?;
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.root)
            .args(["cat-file", "blob", &format!("{}:{path}", self.commit)])
            .output()?;
        ensure!(
            output.status.success(),
            "missing pinned input {path}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(output.stdout)
    }
    pub fn blob_oid(&self, path: &str) -> Result<String> {
        crate::contract::safe_path(path)?;
        Ok(self
            .command(&["rev-parse", &format!("{}:{path}", self.commit)])?
            .trim()
            .to_owned())
    }
    pub fn paths(&self) -> Result<Vec<String>> {
        let output = self.command(&["ls-tree", "-r", "-z", "--name-only", &self.commit])?;
        Ok(output
            .split('\0')
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect())
    }
}

/// Complete per-path policy for the declared federal model and standards catalogue.
/// Unknown canonical JSON fails, even when a caller could preserve its raw bytes.
pub fn handling(path: &str, standards: &BTreeSet<String>) -> Result<Handling> {
    let parts: Vec<_> = path.split('/').collect();
    ensure!(
        parts.len() == 4 && parts[..2] == ["corpus", "mx"],
        "unsupported canonical layout: {path}"
    );
    let name = parts[3];
    if standards.contains(parts[2]) {
        return match name {
            "standard.json" | "validation.json" => Ok(Handling::EvidenceOnly),
            "clauses.json" | "transitories.json" | "supplements.json" => {
                Ok(Handling::ExcludedStandardBody)
            }
            _ => anyhow::bail!("unknown standard input: {path}"),
        };
    }
    match name {
        "instrument.json"
        | "provisions.json"
        | "references.json"
        | "terms.json"
        | "term-usages.json"
        | "glossary-coverage.json"
        | "enactment-evidence.json"
        | "source-manifest.json"
        | "formal-source-manifest.json"
        | "annex-source-manifests.json" => Ok(Handling::Normalized),
        "validation.json"
        | "reform-temporal-evidence.json"
        | "amendment-references.json"
        | "temporal-analysis-request.json"
        | "temporal-analysis-result.json"
        | "review-queue.json" => Ok(Handling::EvidenceOnly),
        _ => anyhow::bail!("unknown federal input: {path}"),
    }
}

#[derive(Debug, Serialize)]
pub struct Inspection {
    pub schema_version: String,
    pub corpus_commit: String,
    pub files: Vec<InputFile>,
    pub federal_instruments: usize,
    pub standards_catalogue: usize,
    pub accepted_debt_rows: usize,
}

pub fn inspect(root: &Path, commit: &str) -> Result<Inspection> {
    let git = GitInput::new(root, commit)?;
    let paths = git.paths()?;
    let standards: BTreeSet<_> = paths
        .iter()
        .filter(|p| p.starts_with("corpus/mx/") && p.ends_with("/standard.json"))
        .map(|p| p.split('/').nth(2).unwrap_or_default().to_owned())
        .collect();
    let federal: Vec<_> = paths
        .iter()
        .filter(|p| p.starts_with("corpus/mx/") && p.ends_with("/instrument.json"))
        .collect();
    let present: BTreeSet<_> = paths.iter().map(String::as_str).collect();
    for instrument in &federal {
        let directory = instrument
            .rsplit_once('/')
            .context("invalid instrument path")?
            .0;
        for name in [
            "instrument.json",
            "provisions.json",
            "references.json",
            "terms.json",
            "term-usages.json",
            "source-manifest.json",
            "glossary-coverage.json",
            "validation.json",
        ] {
            ensure!(
                present.contains(format!("{directory}/{name}").as_str()),
                "missing required input: {directory}/{name}"
            );
        }
    }
    let seed = validate_seed(&git, &git.read(DEBT_PATH)?)?;
    let mut selected = BTreeMap::new();
    for path in &paths {
        // Exporters can place JSON sidecars beneath markdown/. They are
        // presentation artifacts, not another canonical evidence source.
        if path.starts_with("corpus/mx/") && path.split('/').nth(3) == Some("markdown") {
            continue;
        }
        if path.starts_with("corpus/mx/")
            && Path::new(path.as_str())
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        {
            selected.insert(path.clone(), handling(path, &standards)?);
        } else if ((path.starts_with("adapters/") || path.starts_with("schemas/"))
            && Path::new(path.as_str())
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("json")))
            || path == "docs/decisions.md"
            || path == "docs/parser-sweep-review-2026-10-19.md"
            || path == "docs/parser-sweep-review-2026-10-19-evidence.json"
        {
            selected.insert(path.clone(), Handling::EvidenceOnly);
        }
    }
    for path in [
        DEBT_PATH,
        "adapters/diputados/_closure-accepted-gaps.json",
        "docs/decisions.md",
    ] {
        ensure!(
            selected.contains_key(path),
            "required inventory input absent: {path}"
        );
    }
    let files = selected
        .into_iter()
        .map(|(path, handling)| {
            let bytes = git.read(&path)?;
            if Path::new(path.as_str())
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
            {
                let _: serde_json::Value = serde_json::from_slice(&bytes)
                    .with_context(|| format!("invalid JSON: {path}"))?;
            }
            Ok(InputFile {
                path,
                sha256: digest(&bytes),
                bytes: u64::try_from(bytes.len())?,
                handling,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Inspection {
        schema_version: crate::contract::VERSION.into(),
        corpus_commit: git.commit,
        files,
        federal_instruments: federal.len(),
        standards_catalogue: standards.len(),
        accepted_debt_rows: seed.rows.len(),
    })
}
