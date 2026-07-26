use std::collections::HashSet;

use anyhow::{Result, bail};
use lex_core::{
    SCHEMA_VERSION, Severity, StandardClause, StandardKind, StandardMetadata, StandardStatus,
    StandardValidationReport, ValidationIssue,
};
use regex::Regex;

/// Parse the numbered body of a NOM/NMX without treating its clauses as
/// statute articles. Character offsets address the unchanged extracted text.
pub fn parse_standard_clauses(
    source_text: &str,
    metadata: &StandardMetadata,
) -> Result<Vec<StandardClause>> {
    let heading =
        Regex::new(r"(?m)^[ \t]*(\d+(?:\.\d+)*)(?:\.[ \t]+|[ \t]+)([^\r\n].*?)[ \t]*\r?$")?;
    let mut matches = heading
        .captures_iter(source_text)
        .filter_map(|captures| {
            let whole = captures.get(0)?;
            let number = captures.get(1)?.as_str();
            let first = number.split('.').next()?.parse::<u32>().ok()?;
            Some((whole.start(), number.to_owned(), first))
        })
        .collect::<Vec<_>>();
    let Some(first_body) = matches
        .iter()
        .position(|(_, number, first)| !number.contains('.') && matches!(*first, 0 | 1))
    else {
        bail!("standard text has no numbered body beginning with clause 0 or 1");
    };
    matches.drain(..first_body);

    let mut clauses = Vec::with_capacity(matches.len());
    for (index, (start, number, _)) in matches.iter().enumerate() {
        let end = matches
            .get(index + 1)
            .map_or(source_text.len(), |next| next.0);
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

/// Validate the standards-specific identity, lifecycle, review separation,
/// clause ordering, uniqueness, hashes, and exact source spans.
#[must_use]
pub fn validate_standard(
    metadata: &StandardMetadata,
    clauses: &[StandardClause],
    source_text: &str,
) -> StandardValidationReport {
    let mut issues = Vec::new();
    validate_metadata(metadata, &mut issues);
    validate_clauses(metadata, clauses, source_text, &mut issues);
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

#[cfg(test)]
mod tests {
    use super::{parse_standard_clauses, validate_standard};
    use chrono::{NaiveDate, Utc};
    use lex_core::{
        ReviewStatus, SCHEMA_VERSION, StandardKind, StandardMetadata, StandardStatus,
        TechnicalReviewStatus,
    };

    const SAMPLE: &str = include_str!("../../../fixtures/standards/numbered-standard-sample.txt");

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
        let report = validate_standard(&metadata, &clauses, SAMPLE);
        assert!(report.valid, "{:#?}", report.issues);
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
