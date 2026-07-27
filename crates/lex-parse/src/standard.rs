use std::collections::HashSet;

use anyhow::{Result, bail};
use lex_core::{
    SCHEMA_VERSION, Severity, StandardClause, StandardKind, StandardMetadata, StandardStatus,
    StandardTextBasis, StandardTransitory, StandardValidationReport, ValidationIssue,
};
use regex::Regex;

use crate::{
    diputados::{parse_transitory_start, transitory_ordinals},
    slug, spanish_date,
};

/// Parse the numbered body of a NOM/NMX without treating its clauses as
/// statute articles. Character offsets address the unchanged extracted text.
pub fn parse_standard_clauses(
    source_text: &str,
    metadata: &StandardMetadata,
) -> Result<Vec<StandardClause>> {
    let heading =
        Regex::new(r"(?m)^[ \t]*(\d+(?:\.\d+)*)(?:(\.)[ \t]+|[ \t]+)([^\r\n].*?)[ \t]*\r?$")?;
    let matches = heading
        .captures_iter(source_text)
        .filter_map(|captures| {
            let whole = captures.get(0)?;
            let number = captures.get(1)?.as_str();
            let order = clause_order(number)?;
            let label = captures.get(3)?.as_str().trim_start();
            let plausible_top_level =
                captures.get(2).is_some() || label.chars().next().is_some_and(char::is_uppercase);
            let terminal_heading = order.len() == 1 && is_bibliography_heading(label);
            Some((
                whole.start(),
                number.to_owned(),
                order,
                plausible_top_level,
                terminal_heading,
            ))
        })
        .collect::<Vec<_>>();
    let Some((selected, body_end)) = matches
        .iter()
        .enumerate()
        .filter(|(_, (_, _, order, plausible, _))| {
            *plausible && order.len() == 1 && matches!(order[0], 0 | 1)
        })
        .map(|(start, _)| numbered_body_run(&matches, start))
        .max_by_key(|(selected, _)| selected.len())
    else {
        bail!("standard text has no numbered body beginning with clause 0 or 1");
    };
    let numbering_end = matches
        .get(body_end)
        .map_or(source_text.len(), |(start, _, _, _, _)| *start);
    let matches = selected
        .into_iter()
        .map(|index| matches[index].clone())
        .collect::<Vec<_>>();
    let structural_end = matches
        .last()
        .map_or(source_text.len(), |(start, _, _, _, _)| {
            standard_clause_end(source_text, *start, numbering_end)
        });

    let mut clauses = Vec::with_capacity(matches.len());
    for (index, (start, number, _, _, _)) in matches.iter().enumerate() {
        let natural_end = matches.get(index + 1).map_or(structural_end, |next| next.0);
        let end = standard_clause_end(source_text, *start, natural_end);
        let (trimmed_start, trimmed_end) = trim_span(source_text, *start, end);
        let text = source_text[trimmed_start..trimmed_end].to_owned();
        clauses.push(StandardClause {
            schema_version: SCHEMA_VERSION.to_owned(),
            id: format!("{}:clause:{}", metadata.id, clause_slug(number)),
            standard_id: metadata.id.clone(),
            number: number.clone(),
            label: first_line(&text).to_owned(),
            text,
            start_char: source_text[..trimmed_start].chars().count(),
            end_char: source_text[..trimmed_end].chars().count(),
        });
    }
    Ok(clauses)
}

/// Parse a standard's TRANSITORIOS section into addressable, ordinal-labeled
/// blocks without attempting to understand their internal structure.
/// Deliberately lightweight: some standards' transitorios are long and
/// structurally complex (phased criteria, tables, cross-references), and a
/// full clause-style structural parse of that would risk exactly the
/// runaway complexity this stays clear of. Each block's raw text is scanned
/// for "N de MES de AAAA" date phrases; those are recorded as
/// `asserted_dates` without any claim about what they mean (entry into
/// force, phase boundary, deadline, ...) -- reading the surrounding text is
/// still required for that. Returns an empty vector for a standard with no
/// recognizable TRANSITORIOS section, which is not an error: absence is
/// reported by the caller if it matters.
pub fn parse_standard_transitories(
    source_text: &str,
    metadata: &StandardMetadata,
) -> Result<Vec<StandardTransitory>> {
    let heading_marker =
        Regex::new(r"(?mi)^[ \t]*(?:ART[ÍI]CULOS?[ \t]+)?TRANSITORIOS?\b.*\r?$")?;
    let ordinals = transitory_ordinals();
    let line_start = Regex::new(r"(?m)^[ \t]*(\S.*)$")?;
    // A standard's índice repeats every section heading, including
    // TRANSITORIOS, before the real body -- the same false-first-match
    // hazard the bibliography heading has. Try candidates from the last
    // occurrence backwards and take the first one whose section actually
    // contains a recognized ordinal start; an índice mention never does,
    // since it is a bare heading with no transitory text following it.
    let mut heading_ends = heading_marker
        .find_iter(source_text)
        .map(|found| found.end())
        .collect::<Vec<_>>();
    heading_ends.reverse();
    let Some((section, section_offset)) = heading_ends.into_iter().find_map(|heading_end| {
        let section_end = section_end_marker(source_text, heading_end);
        let candidate = &source_text[heading_end..section_end];
        line_start
            .captures_iter(candidate)
            .any(|captures| parse_transitory_start(&captures[1], &ordinals).is_some())
            .then_some((candidate, heading_end))
    }) else {
        return Ok(Vec::new());
    };

    let starts = line_start
        .captures_iter(section)
        .filter_map(|captures| {
            let line = captures.get(1).expect("group 1 always matches with the line");
            parse_transitory_start(line.as_str(), &ordinals)
                .map(|(ordinal, _)| (line.start(), ordinal.to_owned()))
        })
        .collect::<Vec<_>>();

    // `\s+` (not `[ \t]+`) between tokens: `pdftotext -layout` output wraps
    // long lines, and a date phrase can fall across that wrap (".. el 1 de
    // octubre\nde 2023." is a real occurrence, not a contrived one).
    let date_phrase =
        Regex::new(r"(?i)(\d{1,2})[oº]?\s+de\s+([a-zá-úñ]+)\s+de\s+(\d{4})")?;
    let mut transitories = Vec::with_capacity(starts.len());
    for (index, (start, ordinal)) in starts.iter().enumerate() {
        let natural_end = starts.get(index + 1).map_or(section.len(), |next| next.0);
        let (trimmed_start, trimmed_end) = trim_span(section, *start, natural_end);
        let text = section[trimmed_start..trimmed_end].to_owned();
        let asserted_dates = date_phrase
            .captures_iter(&text)
            .filter_map(|captures| {
                spanish_date(&captures[1], &captures[2].to_lowercase(), &captures[3])
            })
            .collect();
        let absolute_start = section_offset + trimmed_start;
        let absolute_end = section_offset + trimmed_end;
        transitories.push(StandardTransitory {
            schema_version: SCHEMA_VERSION.to_owned(),
            id: format!("{}:transitory:{}", metadata.id, slug(ordinal)),
            standard_id: metadata.id.clone(),
            ordinal: ordinal.clone(),
            text,
            start_char: source_text[..absolute_start].chars().count(),
            end_char: source_text[..absolute_end].chars().count(),
            asserted_dates,
        });
    }
    Ok(transitories)
}

/// A closing dateline ("México, D.F., a ..." pre-2016; "Ciudad de México, a
/// ..." after the Distrito Federal/CDMX renaming) or a `SUFRAGIO EFECTIVO`
/// line, either of which opens a decree's signature block.
const SIGNATURE_MARKERS: &str =
    r"SUFRAGIO[ \t]+EFECTIVO\b|(?:CIUDAD[ \t]+DE[ \t]+)?M[ÉE]XICO,[ \t]+(?:D\.?[ \t]*F\.?,[ \t]+)?A\b";

/// End of a section that opened at `TRANSITORIOS`-marker byte offset
/// `section_start`: the next `APÉNDICE`/`ANEXO`/signature marker, or the
/// rest of the text if none follows (the common case -- transitorios are
/// ordinarily a standard's last section).
fn section_end_marker(source_text: &str, section_start: usize) -> usize {
    let markers = Regex::new(&format!(
        r"(?mi)^[ \t]*(?:{SIGNATURE_MARKERS}|AP[ÉE]NDICE\b|ANEXO\b)"
    ))
    .expect("standard section end-marker regex must compile");
    markers
        .find(&source_text[section_start..])
        .map_or(source_text.len(), |marker| section_start + marker.start())
}

fn standard_clause_end(source_text: &str, clause_start: usize, natural_end: usize) -> usize {
    let markers = Regex::new(&format!(
        r"(?mi)^[ \t]*(?:{SIGNATURE_MARKERS}|(?:ART[ÍI]CULOS?[ \t]+)?TRANSITORIOS?\b|AP[ÉE]NDICE\b|ANEXO\b)"
    ))
    .expect("standard end-marker regex must compile");
    markers
        .find(&source_text[clause_start..natural_end])
        .map_or(natural_end, |marker| clause_start + marker.start())
}

fn numbered_body_run(
    matches: &[(usize, String, Vec<u32>, bool, bool)],
    start: usize,
) -> (Vec<usize>, usize) {
    let mut selected = vec![start];
    let mut previous = &matches[start].2;
    let mut current_top = previous[0];
    let mut top_level_closed = false;
    for (index, (_, _, order, plausible_top_level, terminal_heading)) in
        matches.iter().enumerate().skip(start + 1)
    {
        if order.len() == 1 {
            if top_level_closed || !plausible_top_level {
                continue;
            }
            if order[0] <= current_top {
                continue;
            }
            if order[0] != current_top + 1 {
                continue;
            }
            current_top = order[0];
        } else if order[0] != current_top || order <= previous {
            continue;
        }
        selected.push(index);
        previous = order;
        // A bibliography/references heading's own entries are legitimate
        // nested clauses when they continue its number (e.g. `11.1`,
        // `11.2`, ...), but some sources instead give the reference list
        // its own independent numbering that restarts at 1. That
        // restarted count can later coincidentally reach the value that
        // would follow this heading in the outer top-level sequence, so
        // only close the top-level run when the very next candidate is
        // such a restart; nested `current_top.N` sub-clauses and any
        // genuine following section keep parsing normally.
        if *terminal_heading
            && matches
                .get(index + 1)
                .is_some_and(|(.., next_order, _, _)| next_order.as_slice() == [1])
        {
            top_level_closed = true;
        }
    }
    (selected, matches.len())
}

/// Whether a top-level heading label names the standard's bibliography or
/// references section (`Bibliografía`, any accent/case variant). Its
/// internal reference list is a numbered enumeration of sources, not
/// sub-clauses, even when formatted as a numbered list.
fn is_bibliography_heading(label: &str) -> bool {
    let bibliografia = Regex::new(r"(?i)^bibliograf[ií]a\b")
        .expect("bibliography-heading regex must compile");
    bibliografia.is_match(label.trim_start())
}

/// Validate the standards-specific identity, lifecycle, review separation,
/// clause ordering, uniqueness, hashes, and exact source spans.
#[must_use]
pub fn validate_standard(
    metadata: &StandardMetadata,
    clauses: &[StandardClause],
    transitories: &[StandardTransitory],
    source_text: &str,
) -> StandardValidationReport {
    let mut issues = Vec::new();
    validate_metadata(metadata, &mut issues);
    validate_clauses(metadata, clauses, source_text, &mut issues);
    validate_transitories(metadata, transitories, source_text, &mut issues);
    StandardValidationReport {
        schema_version: SCHEMA_VERSION.to_owned(),
        standard_id: metadata.id.clone(),
        valid: !issues.iter().any(|issue| issue.severity == Severity::Error),
        clause_count: clauses.len(),
        issues,
    }
}

fn validate_metadata(metadata: &StandardMetadata, issues: &mut Vec<ValidationIssue>) {
    let prefix = match metadata.kind {
        StandardKind::Nom => "NOM-",
        StandardKind::Nmx => "NMX-",
    };
    if !metadata.designation.starts_with(prefix) {
        issues.push(error(
            "standard_designation",
            format!("{:?} designation must start {prefix}", metadata.kind),
            None,
        ));
    }
    if metadata.issuing_authorities.is_empty() {
        issues.push(error(
            "standard_issuing_authority",
            "at least one issuing authority is required".to_owned(),
            None,
        ));
    }
    if metadata.regulatory_domains.is_empty() {
        issues.push(error(
            "standard_regulatory_domain",
            "at least one regulatory domain is required".to_owned(),
            None,
        ));
    }
    if metadata.source_sha256.len() != 64 || metadata.extracted_text_sha256.len() != 64 {
        issues.push(error(
            "standard_source_hash",
            "source and extracted-text SHA-256 values must have 64 hex characters".to_owned(),
            None,
        ));
    }
    if metadata.status == StandardStatus::Replaced && metadata.replaced_by.is_empty() {
        issues.push(error(
            "standard_replacement",
            "a replaced standard must identify at least one replacement".to_owned(),
            None,
        ));
    }
    if metadata.status == StandardStatus::Cancelled && metadata.cancellation_date.is_none() {
        issues.push(error(
            "standard_cancellation_date",
            "a cancelled standard must record its cancellation date".to_owned(),
            None,
        ));
    }
    if metadata
        .effective_date
        .is_some_and(|date| date < metadata.publication_date)
    {
        issues.push(error(
            "standard_effective_date",
            "effective date cannot precede publication".to_owned(),
            None,
        ));
    }
    if metadata
        .cancellation_date
        .is_some_and(|date| date < metadata.publication_date)
    {
        issues.push(error(
            "standard_cancellation_date",
            "cancellation date cannot precede publication".to_owned(),
            None,
        ));
    }
    validate_standard_sources(metadata, issues);
}

fn validate_standard_sources(metadata: &StandardMetadata, issues: &mut Vec<ValidationIssue>) {
    let included_modifications = metadata
        .modifications
        .iter()
        .filter(|source| source.included_in_source)
        .count();
    match metadata.text_basis {
        StandardTextBasis::AsPublished if included_modifications > 0 => {
            issues.push(error(
                "standard_text_basis",
                "an as-published source cannot include later modifications".to_owned(),
                None,
            ));
        }
        _ => {}
    }
    for source in &metadata.modifications {
        if source.publication_date <= metadata.publication_date {
            issues.push(error(
                "standard_modification_date",
                "a modification must postdate the standard publication".to_owned(),
                None,
            ));
        }
        if !source.included_in_source {
            issues.push(warning(
                "standard_unconsolidated_modification",
                format!(
                    "source text does not incorporate modification published {}",
                    source.publication_date
                ),
            ));
        }
    }
    if metadata
        .systematic_review
        .as_ref()
        .is_some_and(|review| review.review_date < metadata.publication_date)
    {
        issues.push(error(
            "standard_systematic_review_date",
            "systematic review cannot predate the standard publication".to_owned(),
            None,
        ));
    }
}

fn validate_clauses(
    metadata: &StandardMetadata,
    clauses: &[StandardClause],
    source_text: &str,
    issues: &mut Vec<ValidationIssue>,
) {
    if clauses.is_empty() {
        issues.push(error(
            "standard_clause_count",
            "standard has no numbered clauses".to_owned(),
            None,
        ));
        return;
    }
    let source_chars = source_text.chars().collect::<Vec<_>>();
    let mut ids = HashSet::new();
    let mut numbers = HashSet::new();
    let mut previous: Option<Vec<u32>> = None;
    let mut previous_top_level: Option<u32> = None;
    for clause in clauses {
        if clause.standard_id != metadata.id {
            issues.push(error(
                "standard_clause_instrument",
                "clause points to a different standard".to_owned(),
                Some(clause.id.clone()),
            ));
        }
        if !ids.insert(&clause.id) || !numbers.insert(&clause.number) {
            issues.push(error(
                "standard_clause_duplicate",
                "duplicate clause identifier or number".to_owned(),
                Some(clause.id.clone()),
            ));
        }
        let Some(order) = clause_order(&clause.number) else {
            issues.push(error(
                "standard_clause_number",
                "clause number is not a dot-separated numeric identifier".to_owned(),
                Some(clause.id.clone()),
            ));
            continue;
        };
        if previous.as_ref().is_some_and(|prior| order <= *prior) {
            issues.push(error(
                "standard_clause_order",
                "clause numbers do not increase in source order".to_owned(),
                Some(clause.id.clone()),
            ));
        }
        if order.len() == 1 {
            if previous_top_level.is_some_and(|prior| order[0] != prior + 1) {
                issues.push(error(
                    "standard_clause_gap",
                    "top-level clause numbers must be consecutive".to_owned(),
                    Some(clause.id.clone()),
                ));
            }
            previous_top_level = Some(order[0]);
        } else if previous_top_level != Some(order[0]) {
            issues.push(error(
                "standard_clause_parent",
                "nested clause appears outside its current top-level clause".to_owned(),
                Some(clause.id.clone()),
            ));
        }
        previous = Some(order);
        if clause.start_char >= clause.end_char || clause.end_char > source_chars.len() {
            issues.push(error(
                "standard_clause_span",
                "clause character span is outside the extracted text".to_owned(),
                Some(clause.id.clone()),
            ));
            continue;
        }
        let anchored = source_chars[clause.start_char..clause.end_char]
            .iter()
            .collect::<String>();
        if anchored != clause.text {
            issues.push(error(
                "standard_clause_span",
                "clause text does not match its exact extracted-text span".to_owned(),
                Some(clause.id.clone()),
            ));
        }
    }
}

/// Identity, uniqueness, and exact-span checks for transitorios. No absence
/// or ordering check: an empty list is not itself an error (some retained
/// texts genuinely lack a recognizable TRANSITORIOS section), and ordinal
/// sequencing is deliberately not enforced -- this stays a lightweight
/// inspection, not a structural parse.
fn validate_transitories(
    metadata: &StandardMetadata,
    transitories: &[StandardTransitory],
    source_text: &str,
    issues: &mut Vec<ValidationIssue>,
) {
    let source_chars = source_text.chars().collect::<Vec<_>>();
    let mut ids = HashSet::new();
    let mut ordinals = HashSet::new();
    for transitory in transitories {
        if transitory.standard_id != metadata.id {
            issues.push(error(
                "standard_transitory_instrument",
                "transitory points to a different standard".to_owned(),
                Some(transitory.id.clone()),
            ));
        }
        if !ids.insert(&transitory.id) || !ordinals.insert(&transitory.ordinal) {
            issues.push(error(
                "standard_transitory_duplicate",
                "duplicate transitory identifier or ordinal".to_owned(),
                Some(transitory.id.clone()),
            ));
        }
        if transitory.start_char >= transitory.end_char
            || transitory.end_char > source_chars.len()
        {
            issues.push(error(
                "standard_transitory_span",
                "transitory character span is outside the extracted text".to_owned(),
                Some(transitory.id.clone()),
            ));
            continue;
        }
        let anchored = source_chars[transitory.start_char..transitory.end_char]
            .iter()
            .collect::<String>();
        if anchored != transitory.text {
            issues.push(error(
                "standard_transitory_span",
                "transitory text does not match its exact extracted-text span".to_owned(),
                Some(transitory.id.clone()),
            ));
        }
    }
}

fn clause_order(number: &str) -> Option<Vec<u32>> {
    number.split('.').map(|part| part.parse().ok()).collect()
}

fn trim_span(text: &str, start: usize, end: usize) -> (usize, usize) {
    let candidate = &text[start..end];
    let leading = candidate.len() - candidate.trim_start().len();
    let trailing = candidate.len() - candidate.trim_end().len();
    (start + leading, end - trailing)
}

fn first_line(text: &str) -> &str {
    text.lines().next().unwrap_or(text).trim()
}

fn clause_slug(number: &str) -> String {
    number.replace('.', "-")
}

fn error(code: &str, message: String, provision_id: Option<String>) -> ValidationIssue {
    ValidationIssue {
        severity: Severity::Error,
        code: code.to_owned(),
        message,
        provision_id,
    }
}

fn warning(code: &str, message: String) -> ValidationIssue {
    ValidationIssue {
        severity: Severity::Warning,
        code: code.to_owned(),
        message,
        provision_id: None,
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_standard_clauses, parse_standard_transitories, validate_standard};
    use chrono::{NaiveDate, Utc};
    use lex_core::{
        ReviewStatus, SCHEMA_VERSION, StandardKind, StandardMetadata, StandardModificationSource,
        StandardStatus, StandardTextBasis, TechnicalReviewStatus,
    };

    const SAMPLE: &str = include_str!("../../../fixtures/standards/numbered-standard-sample.txt");
    const INDEX_AND_APPENDIX_SAMPLE: &str =
        include_str!("../../../fixtures/standards/index-and-appendix-sample.txt");
    const BIBLIOGRAPHY_CONTINUATION_SAMPLE: &str =
        include_str!("../../../fixtures/standards/bibliography-continuation-sample.txt");
    const TRANSITORIOS_WITH_DATES_SAMPLE: &str =
        include_str!("../../../fixtures/standards/transitorios-with-dates-sample.txt");
    const INDEXED_TRANSITORIOS_SAMPLE: &str =
        include_str!("../../../fixtures/standards/indexed-transitorios-sample.txt");
    const CDMX_SIGNATURE_SAMPLE: &str =
        include_str!("../../../fixtures/standards/cdmx-signature-sample.txt");

    #[test]
    fn transitory_text_excludes_a_ciudad_de_mexico_signature_block() {
        // The signature-block marker previously recognized only the
        // pre-2016 "México, D.F., a ..." dateline; the modern "Ciudad de
        // México, a ..." form (used since the Distrito Federal/CDMX
        // renaming) fell through and bled into the last transitory's text
        // -- found compiling NOM-051's real retained text, whose transitory
        // SEXTO absorbed its decree's closing signature and the decree's
        // own sign-off date as a false-positive asserted date.
        let metadata = metadata();
        let transitories = parse_standard_transitories(CDMX_SIGNATURE_SAMPLE, &metadata).unwrap();
        assert_eq!(transitories.len(), 1);
        assert!(!transitories[0].text.contains("Director General"));
        assert!(!transitories[0].text.contains("Rúbrica"));
        assert_eq!(
            transitories[0].asserted_dates,
            vec![NaiveDate::from_ymd_opt(2025, 10, 1).unwrap()]
        );
    }

    #[test]
    fn transitory_parsing_skips_the_index_mention_and_finds_the_real_section() {
        let metadata = metadata();
        // The índice repeats "ARTÍCULOS TRANSITORIOS" as a bare heading
        // with no ordinal content directly beneath it -- the same
        // false-first-match hazard the bibliography heading has for
        // clauses. Reproduces the real defect found compiling NOM-051's
        // retained text, where its índice occurrence (line 220) preceded
        // the real section (line 1735) and was matched first.
        let transitories = parse_standard_transitories(INDEXED_TRANSITORIOS_SAMPLE, &metadata).unwrap();
        assert_eq!(
            transitories
                .iter()
                .map(|t| t.ordinal.as_str())
                .collect::<Vec<_>>(),
            vec!["PRIMERO", "SEGUNDO"]
        );
        assert_eq!(
            transitories[0].asserted_dates,
            vec![NaiveDate::from_ymd_opt(2025, 10, 1).unwrap()]
        );
        assert!(transitories[1].asserted_dates.is_empty());
    }

    #[test]
    fn parses_ordinal_transitories_and_scans_their_dates() {
        let metadata = metadata();
        let clauses = parse_standard_clauses(TRANSITORIOS_WITH_DATES_SAMPLE, &metadata).unwrap();
        assert_eq!(
            clauses
                .iter()
                .map(|clause| clause.number.as_str())
                .collect::<Vec<_>>(),
            vec!["1", "2"]
        );
        assert!(!clauses[1].text.contains("TRANSITORIOS"));

        let transitories =
            parse_standard_transitories(TRANSITORIOS_WITH_DATES_SAMPLE, &metadata).unwrap();
        assert_eq!(
            transitories
                .iter()
                .map(|t| t.ordinal.as_str())
                .collect::<Vec<_>>(),
            vec!["PRIMERO", "SEGUNDO", "TERCERO"]
        );
        assert_eq!(
            transitories[0].asserted_dates,
            vec![NaiveDate::from_ymd_opt(2025, 10, 1).unwrap()]
        );
        assert_eq!(
            transitories[1].asserted_dates,
            vec![
                NaiveDate::from_ymd_opt(2023, 9, 30).unwrap(),
                NaiveDate::from_ymd_opt(2023, 10, 1).unwrap(),
            ]
        );
        assert_eq!(
            transitories[2].asserted_dates,
            vec![NaiveDate::from_ymd_opt(2021, 3, 31).unwrap()]
        );
        assert_eq!(
            transitories[0].id,
            format!("{}:transitory:primero", metadata.id)
        );

        let report = validate_standard(
            &metadata,
            &clauses,
            &transitories,
            TRANSITORIOS_WITH_DATES_SAMPLE,
        );
        assert!(report.valid, "{:#?}", report.issues);
    }

    #[test]
    fn stops_numbered_body_at_bibliography_and_articulos_transitorios() {
        let metadata = metadata();
        let clauses = parse_standard_clauses(BIBLIOGRAPHY_CONTINUATION_SAMPLE, &metadata).unwrap();
        // The bibliography's own restarted list (1., 2., ...) must not be
        // read as clauses continuing the outer top-level count, even
        // though its second entry coincidentally reaches the count that
        // would follow clause "3" (Bibliografía) if the run continued.
        assert_eq!(
            clauses
                .iter()
                .map(|clause| clause.number.as_str())
                .collect::<Vec<_>>(),
            vec!["0", "1", "1.1", "1.2", "2", "2.1", "2.2", "3"]
        );
        let bibliography = clauses.last().unwrap();
        assert_eq!(bibliography.number, "3");
        assert!(bibliography.text.contains("Segunda referencia bibliográfica"));
        assert!(!bibliography.text.contains("TRANSITORIOS"));
        assert!(!bibliography.text.contains("ÚNICO"));
        let transitories =
            parse_standard_transitories(BIBLIOGRAPHY_CONTINUATION_SAMPLE, &metadata).unwrap();
        assert_eq!(transitories.len(), 1);
        assert_eq!(transitories[0].ordinal, "ÚNICO");
        assert!(transitories[0].text.contains("Entrará en vigor"));
        assert!(transitories[0].asserted_dates.is_empty());
        let report = validate_standard(
            &metadata,
            &clauses,
            &transitories,
            BIBLIOGRAPHY_CONTINUATION_SAMPLE,
        );
        assert!(report.valid, "{:#?}", report.issues);
    }

    #[test]
    fn parses_numbered_standard_with_exact_spans() {
        let metadata = metadata();
        let clauses = parse_standard_clauses(SAMPLE, &metadata).unwrap();
        assert_eq!(
            clauses
                .iter()
                .map(|clause| clause.number.as_str())
                .collect::<Vec<_>>(),
            vec!["0", "1", "2", "3", "4", "5", "5.1", "5.2", "6"]
        );
        assert_eq!(clauses[6].label, "5.1 El establecimiento debe medir.");
        let transitories = parse_standard_transitories(SAMPLE, &metadata).unwrap();
        assert!(transitories.is_empty());
        let report = validate_standard(&metadata, &clauses, &transitories, SAMPLE);
        assert!(report.valid, "{:#?}", report.issues);
    }

    #[test]
    fn skips_index_and_stops_before_signature_and_appendix_numbering() {
        let metadata = metadata();
        let clauses = parse_standard_clauses(INDEX_AND_APPENDIX_SAMPLE, &metadata).unwrap();
        assert_eq!(
            clauses
                .iter()
                .map(|clause| clause.number.as_str())
                .collect::<Vec<_>>(),
            vec!["1", "1.1", "1.2", "2", "2.1", "3", "3.1", "4"]
        );
        assert!(!clauses[6].text.contains("Sufragio"));
        assert!(!clauses[7].text.contains("México, D.F."));
        assert!(!clauses[7].text.contains("APENDICE"));
        let transitories = parse_standard_transitories(INDEX_AND_APPENDIX_SAMPLE, &metadata).unwrap();
        // This fixture has no TRANSITORIOS heading at all -- just a bare
        // signature/date line -- so no transitorios should be recognized.
        assert!(transitories.is_empty());
        let report = validate_standard(
            &metadata,
            &clauses,
            &transitories,
            INDEX_AND_APPENDIX_SAMPLE,
        );
        assert!(report.valid, "{:#?}", report.issues);
    }

    #[test]
    fn current_designation_does_not_hide_unconsolidated_modification() {
        let mut metadata = metadata();
        metadata.modifications.push(StandardModificationSource {
            publication_date: NaiveDate::from_ymd_opt(2026, 7, 26).unwrap(),
            official_url: "https://example.test/dof/modification".parse().unwrap(),
            included_in_source: false,
        });
        let clauses = parse_standard_clauses(SAMPLE, &metadata).unwrap();
        let transitories = parse_standard_transitories(SAMPLE, &metadata).unwrap();
        let report = validate_standard(&metadata, &clauses, &transitories, SAMPLE);
        assert!(report.valid, "{:#?}", report.issues);
        assert!(report.issues.iter().any(|issue| {
            issue.code == "standard_unconsolidated_modification"
                && issue.severity == lex_core::Severity::Warning
        }));
    }

    fn metadata() -> StandardMetadata {
        StandardMetadata {
            schema_version: SCHEMA_VERSION.to_owned(),
            id: "urn:lex-mx:federal:nom:nom-999-test-2026".to_owned(),
            kind: StandardKind::Nom,
            designation: "NOM-999-TEST-2026".to_owned(),
            official_title: "Norma de prueba".to_owned(),
            issuing_authorities: vec!["Secretaría de Prueba".to_owned()],
            regulatory_domains: vec!["fixture".to_owned()],
            publication_date: NaiveDate::from_ymd_opt(2026, 7, 25).unwrap(),
            effective_date: None,
            cancellation_date: None,
            status: StandardStatus::Unknown,
            replaces: Vec::new(),
            replaced_by: Vec::new(),
            joint_prefixes: Vec::new(),
            objective: None,
            scope: None,
            conformity_assessment: None,
            text_basis: StandardTextBasis::AsPublished,
            modifications: Vec::new(),
            systematic_review: None,
            source_url: "https://example.test/source.pdf".parse().unwrap(),
            official_dof_url: "https://example.test/dof".parse().unwrap(),
            official_registry_url: None,
            publisher: "Fixture".to_owned(),
            retrieved_at: Utc::now(),
            source_sha256: "a".repeat(64),
            extracted_text_sha256: "b".repeat(64),
            parser_version: env!("CARGO_PKG_VERSION").to_owned(),
            legal_review_status: ReviewStatus::NotAnalyzed,
            technical_review_status: TechnicalReviewStatus::NotAnalyzed,
        }
    }
}
