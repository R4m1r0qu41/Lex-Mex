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
    /// Re-derive a committed NOM/NMX's parsed files from its retained text.
    ///
    /// The counterpart to `validate`: where that reports committed derived
    /// files as stale for the current parser, this rewrites them. Only derived
    /// files are touched -- `standard.json` is input and `extracted-text.txt`
    /// is the retained source, and both are left exactly as committed. The
    /// retained text is checked against `extracted_text_sha256` first, so a
    /// refresh can never reparse something other than what the record claims.
    Refresh {
        /// Committed standard slug under corpus/mx.
        standard: String,
        /// Permit the refresh to change which decrees mark which clauses.
        ///
        /// Amendment marks are a legal-meaning claim, not a span offset, and
        /// they never move the clause count -- so a title-parser regression
        /// that drops or misattributes marks would otherwise pass every
        /// mechanical guard. Changing them has to be intended and stated.
        #[arg(long)]
        allow_mark_change: bool,
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
        StandardsCommand::Refresh {
            standard,
            allow_mark_change,
        } => refresh_committed_standard(root, &standard, allow_mark_change),
    }
}

fn refresh_committed_standard(root: &Path, slug: &str, allow_mark_change: bool) -> Result<()> {
    let corpus = committed_standard_dir(root, slug)?;
    let metadata: StandardMetadata = read_json(&corpus.join("standard.json"))?;
    let text = retained_text(&corpus, &metadata, slug)?;
    let clauses = parse_standard_clauses(&text, &metadata)?;
    let transitories = parse_standard_transitories(&text, &metadata)?;
    let report = validate_standard(&metadata, &clauses, &transitories, &text);

    // Every guard runs before any write: a refused or failed refresh must
    // leave the committed directory exactly as it found it. Writing first and
    // bailing after would leave invalid or partially-updated canonical files
    // behind an error exit.
    let previous: Vec<StandardClause> = read_json(&corpus.join("clauses.json"))?;
    if previous.len() != clauses.len() {
        bail!(
            "refusing to refresh {slug}: clause count would change {} -> {}; a structural change \
             this large is a parser regression to diagnose, not a file to rewrite",
            previous.len(),
            clauses.len()
        );
    }
    // Transitories get the same protection as the clause count: they never
    // affect the clause count or the amendment marks, so a transitory-parser
    // regression (e.g. a heading-recognition change that suddenly returns
    // none) would otherwise be rewritten into committed canonical data with
    // exit code 0 -- and `validate` could never flag it afterwards, because
    // the corpus would be self-consistent. Entry-into-force dates live here;
    // losing them silently is a legal-meaning failure, not a formatting one.
    let previous_transitories: Vec<StandardTransitory> =
        read_json(&corpus.join("transitories.json"))?;
    if previous_transitories.len() != transitories.len() {
        bail!(
            "refusing to refresh {slug}: transitory count would change {} -> {}; diagnose the \
             parser change, or recompile from verified inputs if the change is intended",
            previous_transitories.len(),
            transitories.len()
        );
    }
    let marks_of = |clauses: &[StandardClause]| {
        clauses
            .iter()
            .filter(|clause| !clause.amended_by.is_empty())
            .map(|clause| (clause.number.clone(), clause.amended_by.clone()))
            .collect::<Vec<_>>()
    };
    let (was, now) = (marks_of(&previous), marks_of(&clauses));
    if was != now && !allow_mark_change {
        bail!(
            "refusing to refresh {slug}: amendment marks would change ({} marked clauses -> {}); \
             marks are a legal-meaning claim the clause count cannot detect a regression in. \
             Re-run with --allow-mark-change once the new marks have been read against the \
             decrees' own DOF titles",
            was.len(),
            now.len()
        );
    }
    if !report.valid {
        bail!(
            "refusing to refresh {slug}: the reparse does not validate ({} issues); nothing was \
             written",
            report.issues.len()
        );
    }

    write_json(&clauses, &corpus.join("clauses.json"))?;
    write_json(&transitories, &corpus.join("transitories.json"))?;
    write_json(&report, &corpus.join("validation.json"))?;
    println!(
        "refreshed {slug}: {} clauses ({} amendment-marked), {} transitories, {} issues, \
         validation valid",
        clauses.len(),
        now.len(),
        transitories.len(),
        report.issues.len(),
    );
    Ok(())
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

fn committed_standard_dir(root: &Path, slug: &str) -> Result<PathBuf> {
    if slug.is_empty()
        || !slug
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        bail!("standard slug must contain only lowercase ASCII letters, digits, and hyphens");
    }
    Ok(root.join("corpus/mx").join(slug))
}

fn retained_text(corpus: &Path, metadata: &StandardMetadata, slug: &str) -> Result<String> {
    let text_bytes = fs::read(corpus.join("extracted-text.txt"))
        .with_context(|| format!("failed to read retained text for standard {slug}"))?;
    if sha256_hex(&text_bytes) != metadata.extracted_text_sha256 {
        bail!("retained extracted text does not match standard metadata");
    }
    String::from_utf8(text_bytes).context("retained extracted standard text is not UTF-8")
}

fn validate_committed_standard(root: &Path, slug: &str) -> Result<()> {
    let corpus = committed_standard_dir(root, slug)?;
    let metadata: StandardMetadata = read_json(&corpus.join("standard.json"))?;
    let clauses: Vec<StandardClause> = read_json(&corpus.join("clauses.json"))?;
    let transitories: Vec<StandardTransitory> = read_json(&corpus.join("transitories.json"))?;
    let text = retained_text(&corpus, &metadata, slug)?;
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
    use std::{fs, path::PathBuf};

    use lex_core::StandardClause;

    use super::{
        compile_standard, read_json, refresh_committed_standard, validate_committed_standard,
    };

    fn compiled_fixture() -> (tempfile::TempDir, PathBuf) {
        let temporary = tempfile::tempdir().unwrap();
        let corpus = temporary.path().join("corpus/mx/nom-999-test-2026");
        let fixture_root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/standards");
        compile_standard(
            &fixture_root.join("numbered-standard-metadata.json"),
            &fixture_root.join("numbered-standard-sample.txt"),
            &fixture_root.join("numbered-standard-sample.txt"),
            &corpus,
        )
        .unwrap();
        (temporary, corpus)
    }

    #[test]
    fn compiled_standard_retains_text_and_revalidates_as_committed_corpus() {
        let (temporary, corpus) = compiled_fixture();
        let root = temporary.path();
        let fixture_root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/standards");

        assert_eq!(
            fs::read(corpus.join("extracted-text.txt")).unwrap(),
            fs::read(fixture_root.join("numbered-standard-sample.txt")).unwrap()
        );
        // The fixture's modification title names 5.1, 5.2, 5.3 and an annex;
        // only 5.1 and 5.2 exist in the sample body. Asserting the marks
        // survive `compile` -> serialize -> `validate` is the whole
        // determinism claim: `amended_by` is derived, so a title-parser change
        // must surface as stale committed data rather than diverge silently.
        let clauses: Vec<StandardClause> = read_json(&corpus.join("clauses.json")).unwrap();
        let marked = clauses
            .iter()
            .filter(|clause| !clause.amended_by.is_empty())
            .map(|clause| clause.number.as_str())
            .collect::<Vec<_>>();
        assert_eq!(marked, ["5.1", "5.2"]);
        validate_committed_standard(root, "nom-999-test-2026").unwrap();
    }

    #[test]
    fn a_modification_source_hash_is_absent_by_default_and_present_when_set() {
        // Stage B (docs/decisions.md 2026-07-31) added an optional
        // per-modification source_sha256, additive to the existing
        // modifications[] list rather than a new top-level array. The
        // property that actually protects the 29 already-committed
        // standard.json files from an unwanted diff is that omitting the
        // field serializes with no key at all, not `null` -- this is what
        // that guarantees.
        let (_temporary, corpus) = compiled_fixture();
        let raw: serde_json::Value =
            read_json(&corpus.join("standard.json")).expect("standard.json parses");
        let modification = &raw["modifications"][0];
        assert!(
            modification.get("source_sha256").is_none(),
            "an unset modification source_sha256 must serialize with no key: {modification}"
        );

        let mut metadata: lex_core::StandardMetadata =
            read_json(&corpus.join("standard.json")).unwrap();
        let hash = "a".repeat(64);
        metadata.modifications[0].source_sha256 = Some(hash.clone());
        let round_tripped = serde_json::to_value(&metadata).unwrap();
        assert_eq!(
            round_tripped["modifications"][0]["source_sha256"],
            serde_json::Value::String(hash)
        );
    }

    #[test]
    fn refresh_refuses_to_rewrite_changed_amendment_marks_unaided() {
        let (temporary, corpus) = compiled_fixture();
        let root = temporary.path();

        // An unchanged refresh is a no-op and must stay one.
        refresh_committed_standard(root, "nom-999-test-2026", false).unwrap();
        validate_committed_standard(root, "nom-999-test-2026").unwrap();

        // Now retarget the decree. The clause count is identical either way, so
        // this is exactly the regression shape the count guard cannot see.
        let metadata = corpus.join("standard.json");
        let retargeted = fs::read_to_string(&metadata)
            .unwrap()
            .replace("numerales 5.1 y 5.2", "numerales 5.1 y 6");
        fs::write(&metadata, retargeted).unwrap();

        let refused = refresh_committed_standard(root, "nom-999-test-2026", false).unwrap_err();
        assert!(
            refused.to_string().contains("amendment marks would change"),
            "{refused}"
        );
        let unchanged: Vec<StandardClause> = read_json(&corpus.join("clauses.json")).unwrap();
        assert!(
            unchanged
                .iter()
                .any(|clause| clause.number == "5.2" && !clause.amended_by.is_empty()),
            "a refused refresh must not have written anything"
        );

        refresh_committed_standard(root, "nom-999-test-2026", true).unwrap();
        let rewritten: Vec<StandardClause> = read_json(&corpus.join("clauses.json")).unwrap();
        let marked = rewritten
            .iter()
            .filter(|clause| !clause.amended_by.is_empty())
            .map(|clause| clause.number.as_str())
            .collect::<Vec<_>>();
        assert_eq!(marked, ["5.1", "6"]);
    }

    #[test]
    fn refresh_refuses_a_transitory_count_change() {
        // Transitories never affect the clause count or the amendment marks,
        // so a transitory-parser regression (a heading-recognition change
        // that suddenly returns none) would pass both other guards and be
        // written into committed canonical data with exit code 0 -- and
        // `validate` could never flag it afterwards, because the corpus would
        // be self-consistent. Simulated here from the other side: the
        // committed file records one more transitory than the parser now
        // produces.
        let (temporary, corpus) = compiled_fixture();
        let root = temporary.path();

        let path = corpus.join("transitories.json");
        let mut committed: Vec<lex_core::StandardTransitory> = read_json(&path).unwrap();
        committed.push(lex_core::StandardTransitory {
            schema_version: lex_core::SCHEMA_VERSION.to_owned(),
            id: "urn:lex-mx:federal:nom:nom-999-test-2026:transitory:regression-witness".to_owned(),
            standard_id: "urn:lex-mx:federal:nom:nom-999-test-2026".to_owned(),
            ordinal: "PRIMERO".to_owned(),
            text: "PRIMERO. Testigo de regresión: el parser ya no lo produce.".to_owned(),
            start_char: 0,
            end_char: 1,
            asserted_dates: Vec::new(),
        });
        let witness = serde_json::to_vec_pretty(&committed).unwrap();
        fs::write(&path, &witness).unwrap();

        let refused = refresh_committed_standard(root, "nom-999-test-2026", false).unwrap_err();
        assert!(
            refused
                .to_string()
                .contains("transitory count would change"),
            "{refused}"
        );
        assert_eq!(
            fs::read(&path).unwrap(),
            witness,
            "a refused refresh must not have rewritten transitories.json"
        );
    }

    #[test]
    fn refresh_writes_nothing_when_the_reparse_does_not_validate() {
        // The guards run before any write: previously the three derived files
        // were rewritten first and the validity bail fired after, leaving
        // invalid canonical data behind a non-zero exit -- and a batch loop
        // that missed one exit code would stage it. An effective date before
        // publication is a metadata-level validation error that leaves the
        // clause count, transitory count, and marks all unchanged, so it
        // reaches the validity guard and nothing else.
        let (temporary, corpus) = compiled_fixture();
        let root = temporary.path();

        let metadata_path = corpus.join("standard.json");
        let broken = fs::read_to_string(&metadata_path).unwrap().replace(
            "\"effective_date\": null",
            "\"effective_date\": \"2020-01-01\"",
        );
        fs::write(&metadata_path, broken).unwrap();
        let before = [
            fs::read(corpus.join("clauses.json")).unwrap(),
            fs::read(corpus.join("transitories.json")).unwrap(),
            fs::read(corpus.join("validation.json")).unwrap(),
        ];

        let refused = refresh_committed_standard(root, "nom-999-test-2026", false).unwrap_err();
        assert!(
            refused.to_string().contains("does not validate"),
            "{refused}"
        );
        let after = [
            fs::read(corpus.join("clauses.json")).unwrap(),
            fs::read(corpus.join("transitories.json")).unwrap(),
            fs::read(corpus.join("validation.json")).unwrap(),
        ];
        assert_eq!(before, after, "a refused refresh must write nothing");
    }
}
