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
