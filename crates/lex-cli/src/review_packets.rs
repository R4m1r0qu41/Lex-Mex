use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use chrono::Utc;
use clap::Subcommand;
use lex_core::{PacketReviewStatus, ReviewPacket, SCHEMA_VERSION};
use lex_source::load_batch_manifest;

#[derive(Debug, Subcommand)]
pub(crate) enum ReviewPacketsCommand {
    /// Generate one packet per `batches/*.json` manifest that has at least
    /// one already-committed instrument. Never overwrites an existing
    /// packet file, so a packet's assignment state is never clobbered by a
    /// re-run.
    Generate,
    /// Assign a reviewer to a packet. Refuses if the packet is not
    /// currently `unassigned` -- reassignment is a deliberate, separate
    /// act, not a silent overwrite.
    Assign {
        packet_id: String,
        #[arg(long)]
        reviewer: String,
    },
    /// List every packet's status and reviewer.
    List,
}

pub(crate) fn run_review_packets(root: &Path, command: ReviewPacketsCommand) -> Result<()> {
    match command {
        ReviewPacketsCommand::Generate => generate_packets(root),
        ReviewPacketsCommand::Assign {
            packet_id,
            reviewer,
        } => assign_packet(root, &packet_id, &reviewer),
        ReviewPacketsCommand::List => list_packets(root),
    }
}

fn packets_dir(root: &Path) -> PathBuf {
    root.join("review-packets")
}

fn committed_slugs(root: &Path) -> Result<BTreeSet<String>> {
    let corpus_root = root.join("corpus/mx");
    let mut slugs = BTreeSet::new();
    for entry in fs::read_dir(&corpus_root)
        .with_context(|| format!("failed to read {}", corpus_root.display()))?
    {
        let entry = entry?;
        if entry.file_type()?.is_dir()
            && let Some(name) = entry.file_name().to_str()
        {
            slugs.insert(name.to_owned());
        }
    }
    Ok(slugs)
}

fn batch_manifest_paths(root: &Path) -> Result<Vec<PathBuf>> {
    let batches_dir = root.join("batches");
    let mut paths: Vec<PathBuf> = fs::read_dir(&batches_dir)
        .with_context(|| format!("failed to read {}", batches_dir.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .collect();
    paths.sort();
    Ok(paths)
}

fn generate_packets(root: &Path) -> Result<()> {
    let committed = committed_slugs(root)?;
    let output_dir = packets_dir(root);
    fs::create_dir_all(&output_dir)?;

    let mut created = 0u32;
    let mut skipped_existing = 0u32;
    let mut skipped_empty = 0u32;
    for manifest_path in batch_manifest_paths(root)? {
        let manifest = load_batch_manifest(&manifest_path)?;
        let instruments: Vec<String> = manifest
            .instruments
            .iter()
            .map(|instrument| instrument.slug.clone())
            .filter(|slug| committed.contains(slug))
            .collect();
        if instruments.is_empty() {
            skipped_empty += 1;
            continue;
        }
        let packet_path = output_dir.join(format!("{}.json", manifest.batch_id));
        if packet_path.exists() {
            skipped_existing += 1;
            continue;
        }
        let packet = ReviewPacket {
            schema_version: SCHEMA_VERSION.to_owned(),
            packet_id: manifest.batch_id.clone(),
            grouping_key: "batch_id".to_owned(),
            instruments,
            status: PacketReviewStatus::Unassigned,
            reviewer: None,
            assigned_at: None,
            notes: Vec::new(),
        };
        write_json(&packet, &packet_path)?;
        created += 1;
    }
    println!(
        "generated {created} packet(s); {skipped_existing} already existed and were left \
         untouched; {skipped_empty} batch manifest(s) had no committed instruments yet"
    );
    Ok(())
}

fn assign_packet(root: &Path, packet_id: &str, reviewer: &str) -> Result<()> {
    let path = packets_dir(root).join(format!("{packet_id}.json"));
    let mut packet: ReviewPacket = read_json(&path)?;
    if packet.status != PacketReviewStatus::Unassigned {
        bail!(
            "packet {packet_id:?} is already {:?}; reassignment is a deliberate separate act, \
             not this command",
            packet.status
        );
    }
    packet.status = PacketReviewStatus::Assigned;
    packet.reviewer = Some(reviewer.to_owned());
    packet.assigned_at = Some(Utc::now());
    write_json(&packet, &path)?;
    println!("assigned {packet_id} to {reviewer}");
    Ok(())
}

fn list_packets(root: &Path) -> Result<()> {
    let dir = packets_dir(root);
    if !dir.is_dir() {
        println!("no review packets; run `review-packets generate` first");
        return Ok(());
    }
    let mut paths: Vec<PathBuf> = fs::read_dir(&dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .collect();
    paths.sort();
    for path in paths {
        let packet: ReviewPacket = read_json(&path)?;
        println!(
            "{}\t{:?}\t{}\t{} instrument(s)",
            packet.packet_id,
            packet.status,
            packet.reviewer.as_deref().unwrap_or("-"),
            packet.instruments.len()
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

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("failed to parse {}", path.display()))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use lex_core::{PacketReviewStatus, ReviewPacket};

    use super::{assign_packet, generate_packets, list_packets, read_json};

    fn fixture_root(temporary: &tempfile::TempDir) -> PathBuf {
        let root = temporary.path();
        std::fs::create_dir_all(root.join("batches")).unwrap();
        std::fs::create_dir_all(root.join("corpus/mx/lft")).unwrap();
        std::fs::create_dir_all(root.join("corpus/mx/lftse")).unwrap();
        std::fs::write(
            root.join("batches/labor_L1_labor.json"),
            serde_json::json!({
                "schema_version": "0.1.0",
                "batch_id": "labor_L1_labor",
                "description": "test",
                "instruments": [
                    {
                        "slug": "lft",
                        "title": "LEY Federal del Trabajo",
                        "type": "ley",
                        "adapter": "diputados",
                        "status": "NEW",
                        "source_pdf": "https://example.test/lft.pdf"
                    },
                    {
                        "slug": "lftse",
                        "title": "LEY Federal de los Trabajadores al Servicio del Estado",
                        "type": "ley",
                        "adapter": "diputados",
                        "status": "NEW",
                        "source_pdf": "https://example.test/lftse.pdf"
                    },
                    {
                        "slug": "not-yet-ingested",
                        "title": "Not yet committed",
                        "type": "ley",
                        "adapter": "diputados",
                        "status": "NEW",
                        "source_pdf": "https://example.test/nope.pdf"
                    }
                ]
            })
            .to_string(),
        )
        .unwrap();
        std::fs::write(
            root.join("batches/empty_batch.json"),
            serde_json::json!({
                "schema_version": "0.1.0",
                "batch_id": "empty_batch",
                "description": "nothing committed yet",
                "instruments": [
                    {
                        "slug": "not-committed",
                        "title": "Nothing here",
                        "type": "ley",
                        "adapter": "diputados",
                        "status": "NEW",
                        "source_pdf": "https://example.test/none.pdf"
                    }
                ]
            })
            .to_string(),
        )
        .unwrap();
        root.to_path_buf()
    }

    #[test]
    fn generate_only_includes_committed_instruments_and_skips_empty_batches() {
        let temporary = tempfile::tempdir().unwrap();
        let root = fixture_root(&temporary);
        generate_packets(&root).unwrap();

        let packet: ReviewPacket =
            read_json(&root.join("review-packets/labor_L1_labor.json")).unwrap();
        assert_eq!(packet.instruments, ["lft", "lftse"]);
        assert_eq!(packet.status, PacketReviewStatus::Unassigned);
        assert_eq!(packet.grouping_key, "batch_id");
        assert!(
            !root.join("review-packets/empty_batch.json").exists(),
            "a batch with no committed instruments must not produce a packet"
        );
    }

    #[test]
    fn generate_never_overwrites_an_existing_packet() {
        let temporary = tempfile::tempdir().unwrap();
        let root = fixture_root(&temporary);
        generate_packets(&root).unwrap();
        assign_packet(&root, "labor_L1_labor", "jrh").unwrap();

        // Re-running generate must not clobber the assignment just made.
        generate_packets(&root).unwrap();
        let packet: ReviewPacket =
            read_json(&root.join("review-packets/labor_L1_labor.json")).unwrap();
        assert_eq!(packet.status, PacketReviewStatus::Assigned);
        assert_eq!(packet.reviewer.as_deref(), Some("jrh"));
    }

    #[test]
    fn assign_refuses_a_packet_that_is_not_unassigned() {
        let temporary = tempfile::tempdir().unwrap();
        let root = fixture_root(&temporary);
        generate_packets(&root).unwrap();
        assign_packet(&root, "labor_L1_labor", "jrh").unwrap();

        let error = assign_packet(&root, "labor_L1_labor", "someone_else").unwrap_err();
        assert!(error.to_string().contains("already"), "{error}");
        let packet: ReviewPacket =
            read_json(&root.join("review-packets/labor_L1_labor.json")).unwrap();
        assert_eq!(
            packet.reviewer.as_deref(),
            Some("jrh"),
            "a refused reassignment must not have written anything"
        );
    }

    #[test]
    fn list_runs_against_generated_packets_without_error() {
        let temporary = tempfile::tempdir().unwrap();
        let root = fixture_root(&temporary);
        generate_packets(&root).unwrap();
        list_packets(&root).unwrap();
    }
}
