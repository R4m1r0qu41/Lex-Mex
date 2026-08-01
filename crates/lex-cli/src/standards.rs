use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use clap::Subcommand;
use lex_core::{StandardClause, StandardMetadata, StandardSupplement, StandardTransitory};
use lex_parse::{
    parse_standard_clauses, parse_standard_supplements, parse_standard_transitories,
    validate_standard,
};
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
        /// Permit a reviewed final-transitory truncation and its corresponding
        /// post-transitory supplement derivation. Earlier transitories may not
        /// change, and all guards still run before writes.
        #[arg(long)]
        allow_tail_repartition: bool,
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
            allow_tail_repartition,
        } => refresh_committed_standard(root, &standard, allow_mark_change, allow_tail_repartition),
    }
}

fn refresh_committed_standard(
    root: &Path,
    slug: &str,
    allow_mark_change: bool,
    allow_tail_repartition: bool,
) -> Result<()> {
    let corpus = committed_standard_dir(root, slug)?;
    let metadata: StandardMetadata = read_json(&corpus.join("standard.json"))?;
    let text = retained_text(&corpus, &metadata, slug)?;
    let clauses = parse_standard_clauses(&text, &metadata)?;
    let transitories = parse_standard_transitories(&text, &metadata)?;
    let supplements = parse_standard_supplements(&text, &metadata)?;
    let report = validate_standard(&metadata, &clauses, &transitories, &supplements, &text);

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
    let supplements_path = corpus.join("supplements.json");
    let previous_supplements: Vec<StandardSupplement> = if supplements_path.is_file() {
        read_json(&supplements_path)?
    } else {
        Vec::new()
    };
    let transitories_changed = previous_transitories != transitories;
    let supplements_changed = previous_supplements != supplements;
    if (transitories_changed || supplements_changed) && !allow_tail_repartition {
        bail!(
            "refusing to refresh {slug}: transitory content or supplements would change; re-run \
             with --allow-tail-repartition only after reviewing the exact final-transitory and \
             supplement spans"
        );
    }
    if allow_tail_repartition && transitories_changed {
        let Some((previous_final, current_final)) =
            previous_transitories.last().zip(transitories.last())
        else {
            bail!("refusing tail repartition without a final transitory");
        };
        if previous_transitories[..previous_transitories.len() - 1]
            != transitories[..transitories.len() - 1]
            || previous_final.id != current_final.id
            || previous_final.ordinal != current_final.ordinal
            || previous_final.start_char != current_final.start_char
            || !previous_final.text.starts_with(&current_final.text)
            || current_final.end_char > previous_final.end_char
            || supplements.is_empty()
        {
            bail!(
                "refusing to repartition {slug}: only truncation of the final transitory paired \
                 with represented supplements is permitted"
            );
        }
    }
    let (was, now) = (amendment_marks(&previous), amendment_marks(&clauses));
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
    write_json(&supplements, &supplements_path)?;
    write_json(&report, &corpus.join("validation.json"))?;
    println!(
        "refreshed {slug}: {} clauses ({} amendment-marked), {} transitories, {} supplements, {} issues, \
         validation valid",
        clauses.len(),
        now.len(),
        transitories.len(),
        supplements.len(),
        report.issues.len(),
    );
    Ok(())
}

fn amendment_marks(
    clauses: &[StandardClause],
) -> Vec<(String, Vec<lex_core::StandardClauseAmendment>)> {
    clauses
        .iter()
        .filter(|clause| !clause.amended_by.is_empty())
        .map(|clause| (clause.number.clone(), clause.amended_by.clone()))
        .collect()
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
    let supplements = parse_standard_supplements(&text, &metadata)?;
    let report = validate_standard(&metadata, &clauses, &transitories, &supplements, &text);

    fs::create_dir_all(output)?;
    write_json(&metadata, &output.join("standard.json"))?;
    write_json(&clauses, &output.join("clauses.json"))?;
    write_json(&transitories, &output.join("transitories.json"))?;
    write_json(&supplements, &output.join("supplements.json"))?;
    write_json(&report, &output.join("validation.json"))?;
    fs::write(output.join("extracted-text.txt"), text.as_bytes())?;
    println!(
        "standard validation: {}; {} clauses, {} transitories, {} supplements, {} issues",
        if report.valid { "valid" } else { "invalid" },
        report.clause_count,
        transitories.len(),
        supplements.len(),
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
    let supplements: Vec<StandardSupplement> = read_json(&corpus.join("supplements.json"))?;
    let text = retained_text(&corpus, &metadata, slug)?;
    let reparsed = parse_standard_clauses(&text, &metadata)?;
    if serde_json::to_value(&reparsed)? != serde_json::to_value(&clauses)? {
        bail!("committed standard clauses are stale for the current parser");
    }
    let reparsed_transitories = parse_standard_transitories(&text, &metadata)?;
    if serde_json::to_value(&reparsed_transitories)? != serde_json::to_value(&transitories)? {
        bail!("committed standard transitories are stale for the current parser");
    }
    let reparsed_supplements = parse_standard_supplements(&text, &metadata)?;
    if serde_json::to_value(&reparsed_supplements)? != serde_json::to_value(&supplements)? {
        bail!("committed standard supplements are stale for the current parser");
    }
    let report = validate_standard(&metadata, &clauses, &transitories, &supplements, &text);
    println!(
        "standard validation: {}; {} clauses, {} transitories, {} supplements, {} issues",
        if report.valid { "valid" } else { "invalid" },
        report.clause_count,
        transitories.len(),
        supplements.len(),
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
        write_json,
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

    fn compiled_tail_fixture() -> (tempfile::TempDir, PathBuf) {
        let temporary = tempfile::tempdir().unwrap();
        let corpus = temporary.path().join("corpus/mx/nom-999-test-2026");
        let fixture_root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/standards");
        let text = fixture_root.join("post-transitorios-annex-sample.txt");
        let bytes = fs::read(&text).unwrap();
        let digest = lex_source::sha256_hex(&bytes);
        let mut metadata: serde_json::Value = serde_json::from_slice(
            &fs::read(fixture_root.join("numbered-standard-metadata.json")).unwrap(),
        )
        .unwrap();
        metadata["source_sha256"] = digest.clone().into();
        metadata["extracted_text_sha256"] = digest.into();
        let metadata_path = temporary.path().join("metadata.json");
        fs::write(
            &metadata_path,
            serde_json::to_vec_pretty(&metadata).unwrap(),
        )
        .unwrap();
        compile_standard(&metadata_path, &text, &text, &corpus).unwrap();
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
    fn refresh_refuses_to_rewrite_changed_amendment_marks_unaided() {
        let (temporary, corpus) = compiled_fixture();
        let root = temporary.path();

        // An unchanged refresh is a no-op and must stay one.
        refresh_committed_standard(root, "nom-999-test-2026", false, false).unwrap();
        validate_committed_standard(root, "nom-999-test-2026").unwrap();

        // Now retarget the decree. The clause count is identical either way, so
        // this is exactly the regression shape the count guard cannot see.
        let metadata = corpus.join("standard.json");
        let retargeted = fs::read_to_string(&metadata)
            .unwrap()
            .replace("numerales 5.1 y 5.2", "numerales 5.1 y 6");
        fs::write(&metadata, retargeted).unwrap();

        let refused =
            refresh_committed_standard(root, "nom-999-test-2026", false, false).unwrap_err();
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

        refresh_committed_standard(root, "nom-999-test-2026", true, false).unwrap();
        let rewritten: Vec<StandardClause> = read_json(&corpus.join("clauses.json")).unwrap();
        let marked = rewritten
            .iter()
            .filter(|clause| !clause.amended_by.is_empty())
            .map(|clause| clause.number.as_str())
            .collect::<Vec<_>>();
        assert_eq!(marked, ["5.1", "6"]);
    }

    #[test]
    fn refresh_requires_and_bounds_tail_repartition_authorization() {
        let (temporary, corpus) = compiled_tail_fixture();
        let root = temporary.path();
        let before_standard = fs::read(corpus.join("standard.json")).unwrap();
        let before_text = fs::read(corpus.join("extracted-text.txt")).unwrap();
        let before_clauses = fs::read(corpus.join("clauses.json")).unwrap();
        let before_transitories = fs::read(corpus.join("transitories.json")).unwrap();
        let before_supplements = fs::read(corpus.join("supplements.json")).unwrap();
        let before_validation = fs::read(corpus.join("validation.json")).unwrap();

        let metadata_path = corpus.join("standard.json");
        let mut metadata: serde_json::Value = read_json(&metadata_path).unwrap();
        metadata["supplement_starts"] = serde_json::json!([{
            "anchor": "APÉNDICE I",
            "kind": "appendix"
        }]);
        write_json(&metadata, &metadata_path).unwrap();
        let configured_standard = fs::read(&metadata_path).unwrap();

        let refused =
            refresh_committed_standard(root, "nom-999-test-2026", false, false).unwrap_err();
        assert!(refused.to_string().contains("--allow-tail-repartition"));
        assert_eq!(
            fs::read(corpus.join("clauses.json")).unwrap(),
            before_clauses
        );
        assert_eq!(
            fs::read(corpus.join("transitories.json")).unwrap(),
            before_transitories
        );
        assert_eq!(
            fs::read(corpus.join("supplements.json")).unwrap(),
            before_supplements
        );
        assert_eq!(
            fs::read(corpus.join("validation.json")).unwrap(),
            before_validation
        );

        refresh_committed_standard(root, "nom-999-test-2026", false, true).unwrap();
        assert_eq!(fs::read(&metadata_path).unwrap(), configured_standard);
        assert_eq!(
            fs::read(corpus.join("extracted-text.txt")).unwrap(),
            before_text
        );
        assert_eq!(
            fs::read(corpus.join("clauses.json")).unwrap(),
            before_clauses
        );
        assert_ne!(
            fs::read(corpus.join("transitories.json")).unwrap(),
            before_transitories
        );
        assert_ne!(
            fs::read(corpus.join("supplements.json")).unwrap(),
            before_supplements
        );
        assert_ne!(
            fs::read(corpus.join("validation.json")).unwrap(),
            before_validation
        );
        assert_ne!(before_standard, configured_standard);
        validate_committed_standard(root, "nom-999-test-2026").unwrap();
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

        let refused =
            refresh_committed_standard(root, "nom-999-test-2026", false, false).unwrap_err();
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

        let refused =
            refresh_committed_standard(root, "nom-999-test-2026", false, false).unwrap_err();
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
