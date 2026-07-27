use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use clap::Subcommand;
use lex_core::{StandardClause, StandardMetadata, StandardTransitory};
use lex_parse::{parse_standard_clauses, parse_standard_transitories, validate_standard};
use lex_source::sha256_hex;

#[derive(Debug, Subcommand)]
pub(crate) enum StandardsCommand {
    /// Compile verified source/text inputs into standards-specific records.
    Compile {
        #[arg(long)]
        metadata: PathBuf,
        #[arg(long)]
        source: PathBuf,
        #[arg(long)]
        text: PathBuf,
        /// New directory to create. Existing paths are never overwritten.
        #[arg(long)]
        output: PathBuf,
    },
    /// Revalidate one committed NOM/NMX corpus against its retained text.
    Validate {
        /// Committed standard slug under corpus/mx.
        standard: String,
    },
}

pub(crate) fn run_standards_command(root: &Path, command: StandardsCommand) -> Result<()> {
    match command {
        StandardsCommand::Compile {
            metadata,
            source,
            text,
            output,
        } => compile_standard(&metadata, &source, &text, &output),
        StandardsCommand::Validate { standard } => validate_committed_standard(root, &standard),
    }
}

fn compile_standard(
    metadata_path: &Path,
    source_path: &Path,
    text_path: &Path,
    output: &Path,
) -> Result<()> {
    if output.exists() {
        bail!(
            "standards output already exists; refusing to overwrite {}",
            output.display()
        );
    }
    let metadata_bytes = fs::read(metadata_path)
        .with_context(|| format!("failed to read {}", metadata_path.display()))?;
    let metadata: StandardMetadata = serde_json::from_slice(&metadata_bytes)
        .with_context(|| format!("failed to parse {}", metadata_path.display()))?;
    if metadata.schema_version != lex_core::SCHEMA_VERSION {
        bail!(
            "unsupported standard schema version {:?}",
            metadata.schema_version
        );
    }
    if metadata.parser_version != env!("CARGO_PKG_VERSION") {
        bail!(
            "metadata parser_version {:?} does not match compiler version {:?}",
            metadata.parser_version,
            env!("CARGO_PKG_VERSION")
        );
    }
    let source_bytes = fs::read(source_path)
        .with_context(|| format!("failed to read {}", source_path.display()))?;
    let text_bytes =
        fs::read(text_path).with_context(|| format!("failed to read {}", text_path.display()))?;
    if sha256_hex(&source_bytes) != metadata.source_sha256 {
        bail!("source SHA-256 does not match standard metadata");
    }
    if sha256_hex(&text_bytes) != metadata.extracted_text_sha256 {
        bail!("extracted-text SHA-256 does not match standard metadata");
    }
    let text = String::from_utf8(text_bytes).context("extracted standard text is not UTF-8")?;
    let clauses = parse_standard_clauses(&text, &metadata)?;
    let transitories = parse_standard_transitories(&text, &metadata)?;
    let report = validate_standard(&metadata, &clauses, &transitories, &text);

    fs::create_dir_all(output)?;
    write_json(&metadata, &output.join("standard.json"))?;
    write_json(&clauses, &output.join("clauses.json"))?;
    write_json(&transitories, &output.join("transitories.json"))?;
    write_json(&report, &output.join("validation.json"))?;
    fs::write(output.join("extracted-text.txt"), text.as_bytes())?;
    println!(
        "standard validation: {}; {} clauses, {} transitories, {} issues",
        if report.valid { "valid" } else { "invalid" },
        report.clause_count,
        transitories.len(),
        report.issues.len()
    );
    if !report.valid {
        bail!(
            "standard validation failed; inspect {}",
            output.join("validation.json").display()
        );
    }
    Ok(())
}

fn validate_committed_standard(root: &Path, slug: &str) -> Result<()> {
    if slug.is_empty()
        || !slug
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        bail!("standard slug must contain only lowercase ASCII letters, digits, and hyphens");
    }
    let corpus = root.join("corpus/mx").join(slug);
    let metadata: StandardMetadata = read_json(&corpus.join("standard.json"))?;
    let clauses: Vec<StandardClause> = read_json(&corpus.join("clauses.json"))?;
    let transitories: Vec<StandardTransitory> = read_json(&corpus.join("transitories.json"))?;
    let text_bytes = fs::read(corpus.join("extracted-text.txt"))
        .with_context(|| format!("failed to read retained text for standard {slug}"))?;
    if sha256_hex(&text_bytes) != metadata.extracted_text_sha256 {
        bail!("retained extracted text does not match standard metadata");
    }
    let text =
        String::from_utf8(text_bytes).context("retained extracted standard text is not UTF-8")?;
    let reparsed = parse_standard_clauses(&text, &metadata)?;
    if serde_json::to_value(&reparsed)? != serde_json::to_value(&clauses)? {
        bail!("committed standard clauses are stale for the current parser");
    }
    let reparsed_transitories = parse_standard_transitories(&text, &metadata)?;
    if serde_json::to_value(&reparsed_transitories)? != serde_json::to_value(&transitories)? {
        bail!("committed standard transitories are stale for the current parser");
    }
    let report = validate_standard(&metadata, &clauses, &transitories, &text);
    println!(
        "standard validation: {}; {} clauses, {} transitories, {} issues",
        if report.valid { "valid" } else { "invalid" },
        report.clause_count,
        transitories.len(),
        report.issues.len()
    );
    if !report.valid {
        bail!("committed standard validation failed");
    }
    let committed_report: lex_core::StandardValidationReport =
        read_json(&corpus.join("validation.json"))?;
    if serde_json::to_value(&report)? != serde_json::to_value(&committed_report)? {
        bail!("committed standard validation report is stale");
    }
    Ok(())
}

fn write_json<T: serde::Serialize>(value: &T, path: &Path) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    fs::write(path, bytes)?;
    Ok(())
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("failed to parse {}", path.display()))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{compile_standard, validate_committed_standard};

    #[test]
    fn compiled_standard_retains_text_and_revalidates_as_committed_corpus() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path();
        let fixture_root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/standards");
        let corpus = root.join("corpus/mx/nom-999-test-2026");
        compile_standard(
            &fixture_root.join("numbered-standard-metadata.json"),
            &fixture_root.join("numbered-standard-sample.txt"),
            &fixture_root.join("numbered-standard-sample.txt"),
            &corpus,
        )
        .unwrap();

        assert_eq!(
            fs::read(corpus.join("extracted-text.txt")).unwrap(),
            fs::read(fixture_root.join("numbered-standard-sample.txt")).unwrap()
        );
        validate_committed_standard(root, "nom-999-test-2026").unwrap();
    }
}
