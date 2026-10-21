//! M2 staging, transactional build and immutable local release publication.
use crate::{
    contract::{BuildSpecification, DatabaseManifest, VERSION, validate_manifest},
    digest,
    input::{GitInput, Snapshot},
};
use anyhow::{Context, Result, ensure};
use rusqlite::{Connection, OpenFlags};
use serde::Serialize;
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs::{self, File},
    path::{Path, PathBuf},
};

// Bind the executing projection implementation to its claimed Git commit.
// Include transitive local validator sources as well as the projection code.
const BUILDER_FILES: &[(&str, &[u8])] = &[
    ("Cargo.toml", include_bytes!("../../../Cargo.toml")),
    ("Cargo.lock", include_bytes!("../../../Cargo.lock")),
    ("crates/lex-db/Cargo.toml", include_bytes!("../Cargo.toml")),
    ("crates/lex-db/build.rs", include_bytes!("../build.rs")),
    ("crates/lex-db/schema.sql", include_bytes!("../schema.sql")),
    ("crates/lex-db/src/lib.rs", include_bytes!("lib.rs")),
    ("crates/lex-db/src/build.rs", include_bytes!("build.rs")),
    ("crates/lex-db/src/load.rs", include_bytes!("load.rs")),
    (
        "crates/lex-db/src/validation.rs",
        include_bytes!("validation.rs"),
    ),
    ("crates/lex-db/src/input.rs", include_bytes!("input.rs")),
    (
        "crates/lex-db/src/contract.rs",
        include_bytes!("contract.rs"),
    ),
    ("crates/lex-db/src/probe.rs", include_bytes!("probe.rs")),
    (
        "crates/lex-core/src/lib.rs",
        include_bytes!("../../lex-core/src/lib.rs"),
    ),
    (
        "crates/lex-parse/src/lib.rs",
        include_bytes!("../../lex-parse/src/lib.rs"),
    ),
];

#[derive(Debug, Serialize)]
pub struct BuildReport {
    pub release_id: String,
    pub directory: PathBuf,
    pub database_sha256: String,
    pub input_files: usize,
    pub table_counts: BTreeMap<String, i64>,
}

pub fn check_builder(git: &GitInput) -> Result<()> {
    for (path, bytes) in BUILDER_FILES {
        ensure!(
            git.read(path)? == *bytes,
            "executing builder differs from builder commit: {path}"
        );
    }
    // All source/schema files must match the pinned builder, including modules
    // called by shared validators; source files unknown to this binary are not
    // accepted as provenance either. Inventory is embedded at compile time below.
    for (path, bytes) in include!(concat!(env!("OUT_DIR"), "/builder_inputs.rs")) {
        ensure!(
            git.read(path)? == *bytes,
            "compiled dependency differs from builder commit: {path}"
        );
    }
    Ok(())
}

/// Both schema consumers must apply exactly the compiled contract.
pub fn check_snapshot_schemas(git: &GitInput) -> Result<()> {
    let mut expected = std::collections::BTreeSet::new();
    for (path, bytes) in include!(concat!(env!("OUT_DIR"), "/builder_inputs.rs")) {
        if path.starts_with("schemas/") {
            expected.insert((*path).to_owned());
            ensure!(
                git.read(path)? == *bytes,
                "snapshot schema differs from compiled contract: {path}"
            );
        }
    }
    let actual = git
        .paths()?
        .into_iter()
        .filter(|p| {
            p.starts_with("schemas/")
                && Path::new(p)
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("json"))
        })
        .collect();
    ensure!(
        expected == actual,
        "snapshot schema inventory differs from compiled contract"
    );
    Ok(())
}

fn no_symlinks(path: &Path) -> Result<()> {
    for ancestor in path.ancestors() {
        if let Ok(meta) = fs::symlink_metadata(ancestor) {
            ensure!(
                !meta.file_type().is_symlink(),
                "symlink output component: {}",
                ancestor.display()
            );
        }
    }
    Ok(())
}

/// Restrict outputs to the worktree's ignored derived/db subtree.
pub fn output_directory(root: &Path, requested: &Path) -> Result<PathBuf> {
    let root = fs::canonicalize(root)?;
    let relative = if requested.is_absolute() {
        requested
            .strip_prefix(&root)
            .context("output outside worktree")?
    } else {
        requested
    };
    crate::contract::safe_path(relative.to_str().context("non-UTF8 output path")?)?;
    ensure!(
        relative.starts_with("derived/db"),
        "output must be under derived/db"
    );
    let output = root.join(relative);
    no_symlinks(&output)?;
    // Check before creating directories so a rejected path has no write effects.
    let result = std::process::Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["check-ignore", "-q", "--"])
        .arg(relative.join("probe.sqlite"))
        .status()?;
    ensure!(result.success(), "output is not Git-ignored");
    fs::create_dir_all(&output)?;
    no_symlinks(&output)?;
    Ok(output)
}

fn specification(
    root: &Path,
    corpus: &str,
    builder: &str,
    files: Vec<crate::contract::InputFile>,
) -> Result<BuildSpecification> {
    let git = GitInput::new(root, builder)?;
    let input = GitInput::new(root, corpus)?;
    let timestamp = input
        .command(&["show", "-s", "--format=%ct", corpus])?
        .trim()
        .parse()?;
    let c = Connection::open_in_memory()?;
    let mut options = c
        .prepare("PRAGMA compile_options")?
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    options.sort();
    Ok(BuildSpecification {
        schema_version: VERSION.into(),
        corpus_commit: corpus.into(),
        builder_commit: builder.into(),
        cargo_lock_sha256: digest(&git.read("Cargo.lock")?),
        rust_toolchain: env!("LEX_DB_RUSTC").into(),
        sqlite_version: rusqlite::version().into(),
        sqlite_compile_options: options,
        source_commit_timestamp: timestamp,
        inclusion_policy: "federal_with_standards_catalogue".into(),
        logical_digest_policy: crate::contract::LOGICAL_POLICY.into(),
        inputs: files,
    })
}

fn sync_json(path: &Path, value: &impl Serialize) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    fs::write(path, bytes)?;
    File::open(path)?.sync_all()?;
    Ok(())
}

/// Preliminary M2 parity against external immutable snapshot, not a substitute
/// for M3's independent release verifier and query suite.
pub fn check_loaded(c: &Connection, data: &Snapshot) -> Result<BTreeMap<String, i64>> {
    ensure!(
        c.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))? == "ok",
        "database integrity failed"
    );
    ensure!(
        !c.prepare("PRAGMA foreign_key_check")?.exists([])?,
        "foreign key check failed"
    );
    let count: i64 = c.query_row("SELECT count(*) FROM input_files", [], |r| r.get(0))?;
    ensure!(
        count == i64::try_from(data.len())?,
        "raw input inventory differs"
    );
    for (path, bytes) in data {
        let (actual, hash): (Vec<u8>, String) = c.query_row(
            "SELECT bytes,sha256 FROM input_files WHERE path=?1",
            [path],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        ensure!(
            actual == *bytes && digest(bytes) == hash,
            "raw input parity failed: {path}"
        );
    }
    let mut counts = BTreeMap::new();
    for (filename, table, columns) in crate::load::GROUPS {
        let mut expected = 0;
        for (path, bytes) in data
            .iter()
            .filter(|(p, _)| p.starts_with("corpus/mx/") && p.ends_with(&format!("/{filename}")))
        {
            let value: Value = serde_json::from_slice(bytes)?;
            let records = crate::load::objects(&value)?;
            let actual = crate::load::reconstructed(c, table, columns, path)?;
            ensure!(
                actual
                    == records
                        .iter()
                        .map(|(_, v)| (*v).clone())
                        .collect::<Vec<_>>(),
                "normalized parity failed: {path}"
            );
            let mut stmt = c.prepare(&format!(
                "SELECT source_pointer FROM {table} WHERE input_path=?1 ORDER BY ordinal"
            ))?;
            let pointers = stmt
                .query_map([path], |r| r.get::<_, String>(0))?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            ensure!(
                pointers == records.iter().map(|(p, _)| p.clone()).collect::<Vec<_>>(),
                "source pointer parity failed: {path}"
            );
            expected += records.len();
        }
        let count: i64 = c.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))?;
        ensure!(
            count == i64::try_from(expected)?,
            "unexpected normalized rows: {table}"
        );
        counts.insert((*table).into(), count);
    }
    crate::parity::check(c, data)?;
    for table in [
        "source_manifests",
        "evidence_artifacts",
        "coverage",
        "coverage_accepted_gaps",
        "standards_catalog",
        "known_structural_debt",
        "instrument_reproducibility",
        "reference_qualifiers",
        "outdated_reference_approvals",
        "legal_operator_reviews",
        "input_files",
    ] {
        counts.insert(
            table.into(),
            c.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))?,
        );
    }
    Ok(counts)
}

pub fn project(root: &Path, corpus: &str, builder: &str, output: &Path) -> Result<BuildReport> {
    check_builder(&GitInput::new(root, builder)?)?;
    check_snapshot_schemas(&GitInput::new(root, corpus)?)?;
    let output = output_directory(root, output)?;
    let (inspection, data) = crate::input::snapshot(root, corpus)?;
    crate::validation::validate(&data)?;
    let spec = specification(root, corpus, builder, inspection.files.clone())?;
    let spec_json = serde_json::to_string(&spec)?;
    let release = digest(spec_json.as_bytes());
    let final_path = output.join(&release);
    ensure!(
        !final_path.exists(),
        "release already exists; refusing overwrite: {}",
        final_path.display()
    );
    let staging = tempfile::Builder::new()
        .prefix(".building-")
        .tempdir_in(&output)?;
    let db = staging.path().join("corpus.sqlite");
    let mut c = Connection::open(&db)?;
    c.execute_batch("PRAGMA journal_mode=DELETE; PRAGMA synchronous=FULL;")?;
    c.execute_batch(include_str!("../schema.sql"))?;
    crate::load::populate(&mut c, &inspection, &data, &release, &spec_json)?;
    let table_counts = check_loaded(&c, &data)?;
    c.close().map_err(|(_, e)| e)?;
    for extension in ["sqlite-wal", "sqlite-shm", "sqlite-journal"] {
        ensure!(
            !db.with_extension(extension).exists(),
            "required SQLite sidecar remains"
        );
    }
    let readonly = Connection::open_with_flags(&db, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    readonly.execute_batch("PRAGMA query_only=ON")?;
    ensure!(
        check_loaded(&readonly, &data)? == table_counts,
        "read-only parity changed"
    );
    readonly.close().map_err(|(_, e)| e)?;
    File::open(&db)?.sync_all()?;
    let manifest = DatabaseManifest {
        schema_version: VERSION.into(),
        release_id: release.clone(),
        specification: spec,
        database_sha256: digest(&fs::read(&db)?),
        logical_content_sha256: crate::logical::hash(&Connection::open_with_flags(
            &db,
            OpenFlags::SQLITE_OPEN_READ_ONLY,
        )?)?,
    };
    validate_manifest(&serde_json::to_value(&manifest)?)?;
    sync_json(&staging.path().join("manifest.json"), &manifest)?;
    let report = BuildReport {
        release_id: release,
        directory: final_path.clone(),
        database_sha256: manifest.database_sha256,
        input_files: data.len(),
        table_counts,
    };
    sync_json(&staging.path().join("verification.json"), &report)?;
    File::open(staging.path())?.sync_all()?;
    publish(&output, &staging, &final_path, &report.release_id)?;
    Ok(report)
}

fn publish(
    output: &Path,
    staging: &tempfile::TempDir,
    final_path: &Path,
    release: &str,
) -> Result<()> {
    // Complete sibling-directory rename; cooperating builders claim a lock.
    let lock = output.join(format!(".{release}.lock"));
    let lock_file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock)
        .context("release publication already claimed")?;
    let result = (|| {
        ensure!(!final_path.exists(), "release appeared during build");
        fs::rename(staging.path(), final_path)?;
        File::open(output)?.sync_all()?;
        Ok(())
    })();
    drop(lock_file);
    fs::remove_file(lock)?;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn publication_refuses_existing_release_and_stale_claim() {
        let root = tempfile::tempdir().unwrap();
        let final_path = root.path().join("release");
        let stage = tempfile::tempdir_in(root.path()).unwrap();
        fs::write(stage.path().join("manifest.json"), b"new").unwrap();
        fs::create_dir(&final_path).unwrap();
        fs::write(final_path.join("manifest.json"), b"previous").unwrap();
        assert!(publish(root.path(), &stage, &final_path, "release").is_err());
        assert_eq!(
            fs::read(final_path.join("manifest.json")).unwrap(),
            b"previous"
        );
        assert!(stage.path().join("manifest.json").exists());
        let next = root.path().join("next");
        fs::write(root.path().join(".next.lock"), b"interrupted claim").unwrap();
        assert!(publish(root.path(), &stage, &next, "next").is_err());
        assert!(!next.exists());
        assert_eq!(
            fs::read(root.path().join(".next.lock")).unwrap(),
            b"interrupted claim"
        );
        let good = root.path().join("good");
        publish(root.path(), &stage, &good, "good").unwrap();
        assert_eq!(fs::read(good.join("manifest.json")).unwrap(), b"new");
        assert!(!root.path().join(".good.lock").exists());
    }
}

#[cfg(test)]
#[path = "build_tests.rs"]
mod fixture_tests;
