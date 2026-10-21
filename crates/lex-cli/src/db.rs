use anyhow::Result;
use clap::Subcommand;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Subcommand)]
pub enum DbCommand {
    /// Build and independently read back a small in-memory adversarial fixture.
    ContractProbe,
    /// Independently verify an immutable release against pinned Git inputs.
    Verify {
        #[arg(long)]
        database: PathBuf,
        #[arg(long)]
        manifest: PathBuf,
        #[arg(long)]
        corpus_commit: String,
    },
    /// Run the fixed corpus golden-query acceptance fixtures read-only.
    GoldenQueries {
        #[arg(long)]
        database: PathBuf,
    },
    /// Build an immutable local projection from pinned corpus and builder commits.
    Project {
        #[arg(long)]
        corpus_commit: String,
        #[arg(long)]
        builder_commit: String,
        #[arg(long, default_value = "derived/db")]
        output_dir: PathBuf,
    },
    /// Inventory immutable Git inputs; requires the committed production debt seed.
    InspectInputs {
        #[arg(long)]
        corpus_commit: String,
    },
    /// Validate an explicitly proposed seed against pinned canonical Git blobs.
    CheckSeed {
        #[arg(long)]
        corpus_commit: String,
        #[arg(long)]
        seed: PathBuf,
    },
}

pub fn run(root: &Path, command: DbCommand) -> Result<()> {
    match command {
        DbCommand::Verify {
            database,
            manifest,
            corpus_commit,
        } => println!(
            "{}",
            serde_json::to_string_pretty(&lex_db::verify::verify(
                root,
                &resolve(root, &database),
                &resolve(root, &manifest),
                &corpus_commit
            )?)?
        ),
        DbCommand::GoldenQueries { database } => {
            let c = lex_db::verify::open_readonly(&resolve(root, &database))?;
            println!(
                "{}",
                serde_json::to_string_pretty(&lex_db::queries::golden(&c)?)?
            );
        }
        DbCommand::Project {
            corpus_commit,
            builder_commit,
            output_dir,
        } => println!(
            "{}",
            serde_json::to_string_pretty(&lex_db::build::project(
                root,
                &corpus_commit,
                &builder_commit,
                &output_dir
            )?)?
        ),
        DbCommand::ContractProbe => println!(
            "{}",
            serde_json::to_string_pretty(&lex_db::probe::run_probe()?)?
        ),
        DbCommand::InspectInputs { corpus_commit } => println!(
            "{}",
            serde_json::to_string_pretty(&lex_db::input::inspect(root, &corpus_commit)?)?
        ),
        DbCommand::CheckSeed {
            corpus_commit,
            seed,
        } => {
            let git = lex_db::input::GitInput::new(root, &corpus_commit)?;
            let bytes = fs::read(if seed.is_absolute() {
                seed
            } else {
                root.join(seed)
            })?;
            let result = lex_db::contract::validate_seed(&git, &bytes)?;
            println!(
                "validated {} accepted debt rows from candidate seed against {corpus_commit}",
                result.rows.len()
            );
        }
    }
    Ok(())
}

fn resolve(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_owned()
    } else {
        root.join(path)
    }
}
