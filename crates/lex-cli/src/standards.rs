use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use clap::Subcommand;
use lex_core::StandardMetadata;
use lex_parse::{parse_standard_clauses, validate_standard};
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
}

pub(crate) fn run_standards_command(command: StandardsCommand) -> Result<()> {
    match command {
        StandardsCommand::Compile {
            metadata,
            source,
            text,
            output,
        } => compile_standard(&metadata, &source, &text, &output),
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
    let report = validate_standard(&metadata, &clauses, &text);

    fs::create_dir_all(output)?;
    write_json(&metadata, &output.join("standard.json"))?;
    write_json(&clauses, &output.join("clauses.json"))?;
    write_json(&report, &output.join("validation.json"))?;
    println!(
        "standard validation: {}; {} clauses, {} issues",
        if report.valid { "valid" } else { "invalid" },
        report.clause_count,
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

fn write_json<T: serde::Serialize>(value: &T, path: &Path) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    fs::write(path, bytes)?;
    Ok(())
}
