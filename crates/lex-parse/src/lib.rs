use std::{
    collections::{HashMap, HashSet},
    fs,
    ops::Range,
    path::Path,
    process::{Command, Stdio},
};

use anyhow::{Context, Result, bail};
use chrono::NaiveDate;
use lex_core::{
    Basis, HeadingContext, LRITF_INSTRUMENT_ID, Provision, ProvisionType, ReferenceEdge,
    ReferenceForm, ReferenceQualifier, ReferenceQualifierType, ReferenceResolutionStatus,
    ReviewStatus, SCHEMA_VERSION, Severity, TemporalEvidence, TemporalStatus, ValidationIssue,
    ValidationReport,
};
use regex::Regex;

const SOURCE_HEADER: &str = "LEY PARA REGULAR LAS INSTITUCIONES DE TECNOLOGÍA FINANCIERA";
const TRANSITORY_ORDINALS: &[&str] = &[
    "PRIMERA",
    "SEGUNDA",
    "TERCERA",
    "CUARTA",
    "QUINTA",
    "SEXTA",
    "SÉPTIMA",
    "OCTAVA",
    "NOVENA",
    "DÉCIMA",
    "DÉCIMA PRIMERA",
];

#[derive(Debug)]
pub struct Extraction {
    pub text: String,
    pub tool_version: String,
}

pub fn extract_pdf(pdf_path: &Path, text_path: &Path) -> Result<Extraction> {
    if let Some(parent) = text_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let status = Command::new("pdftotext")
        .args(["-layout", "-nopgbrk"])
        .arg(pdf_path)
        .arg(text_path)
        .stdin(Stdio::null())
        .status()
        .context("failed to execute pdftotext; install Poppler")?;
    if !status.success() {
        bail!("pdftotext failed with status {status}");
    }
    let text = fs::read_to_string(text_path)
        .with_context(|| format!("failed to read {}", text_path.display()))?;
    if text.trim().is_empty() {
        bail!("pdftotext produced empty output");
    }
    Ok(Extraction {
        text,
        tool_version: pdftotext_version(),
    })
}

pub fn parse_lritf(raw: &str, publication_date: NaiveDate) -> Result<Vec<Provision>> {
    let article_re = Regex::new(r"^Artículo\s+(\d+(?:\s+(?:Bis|Ter|Quáter))?)\.-\s*(.*)$")?;
    let title_re = Regex::new(r"^TÍTULO\s+([IVXLCDM]+)$")?;
    let chapter_re = Regex::new(r"^CAPÍTULO\s+([IVXLCDM]+)$")?;
    let blocks = normalized_blocks(raw);

    let mut provisions = Vec::new();
    let mut current: Option<ProvisionBuilder> = None;
    let mut current_title: Option<String> = None;
    let mut current_chapter: Option<String> = None;
    let mut in_statute_transitories = false;

    for block in blocks {
        if block.starts_with("ARTÍCULOS SEGUNDO A DÉCIMO") {
            break;
        }
        if block == "DISPOSICIONES TRANSITORIAS" {
            if let Some(builder) = current.take() {
                provisions.push(builder.finish(publication_date));
            }
            in_statute_transitories = true;
            current_title = None;
            current_chapter = None;
            continue;
        }

        if !in_statute_transitories {
            if let Some(captures) = title_re.captures(&block) {
                if let Some(builder) = current.take() {
                    provisions.push(builder.finish(publication_date));
                }
                current_title = Some(format!("Título {}", &captures[1]));
                current_chapter = None;
                continue;
            }
            if let Some(captures) = chapter_re.captures(&block) {
                if let Some(builder) = current.take() {
                    provisions.push(builder.finish(publication_date));
                }
                current_chapter = Some(format!("Capítulo {}", &captures[1]));
                continue;
            }
            if let Some(captures) = article_re.captures(&block) {
                if let Some(builder) = current.take() {
                    provisions.push(builder.finish(publication_date));
                }
                let number = captures[1].to_owned();
                current = Some(ProvisionBuilder::article(
                    number,
                    captures[2].trim(),
                    current_title.clone(),
                    current_chapter.clone(),
                ));
                continue;
            }
        } else if let Some((ordinal, body)) = parse_transitory_start(&block) {
            if let Some(builder) = current.take() {
                provisions.push(builder.finish(publication_date));
            }
            current = Some(ProvisionBuilder::transitory(ordinal, body));
            continue;
        }

        if let Some(builder) = &mut current {
            builder.push_block(&block);
        }
    }

    if let Some(builder) = current {
        provisions.push(builder.finish(publication_date));
    }
    if provisions.is_empty() {
        bail!("no LRITF provisions recognized");
    }
    Ok(provisions)
}

struct ReferencePatterns {
    article: Regex,
    number: Regex,
    separator: Regex,
    paragraph: Regex,
    fraction: Regex,
    subsection: Regex,
}

impl ReferencePatterns {
    fn new() -> Result<Self> {
        Ok(Self {
            article: Regex::new(r"(?i)\bartículos?\s+")?,
            number: Regex::new(r"(?i)\d{1,3}(?:-[A-Z])?(?:\s+(?:Bis|Ter|Quáter))?")?,
            separator: Regex::new(
                r"(?ix)^(?:
            [\s,;:/()\-]+ |
            y\b | o\b | a\b | al\b | hasta\b |
            primer(?:o)?\b | segund[oa]\b | tercer(?:o)?\b | cuart[oa]\b |
            quint[oa]\b | sext[oa]\b | séptim[oa]\b | octav[oa]\b |
            noven[oa]\b | décim[oa]\b | últim[oa]\b |
            párrafos?\b | fracci(?:ón|ones)\b | incisos?\b | apartados?\b |
            [IVXLCDM]+\b
        )*$",
            )?,
            paragraph: Regex::new(
                r"(?i)\b(?:primer|primero|segundo|tercer|tercero|cuarto|quinto|sexto|séptimo|octavo|noveno|décimo|último)(?:\s+(?:,|y|o)\s+(?:primer|primero|segundo|tercer|tercero|cuarto|quinto|sexto|séptimo|octavo|noveno|décimo|último))*\s+párrafos?\b",
            )?,
            fraction: Regex::new(
                r"(?i)\bfracci(?:ón|ones)\s+[IVXLCDM]+(?:\s*(?:,|y|o)\s*[IVXLCDM]+)*\b",
            )?,
            subsection: Regex::new(r"(?i)\bincisos?\s+[a-z](?:\s*(?:,|y|o)\s*[a-z])*\b")?,
        })
    }
}

pub fn extract_internal_references(provisions: &[Provision]) -> Result<Vec<ReferenceEdge>> {
    let patterns = ReferencePatterns::new()?;
    let target_ids: HashSet<&str> = provisions.iter().map(|item| item.id.as_str()).collect();
    let mut references = Vec::new();
    for provision in provisions {
        references.extend(extract_provision_references(
            provision,
            &patterns,
            &target_ids,
        ));
    }
    references.sort_by(|left, right| {
        left.source_provision_id
            .cmp(&right.source_provision_id)
            .then(left.start_char.cmp(&right.start_char))
            .then(left.end_char.cmp(&right.end_char))
            .then(left.target_provision_id.cmp(&right.target_provision_id))
    });
    Ok(references)
}

fn extract_provision_references(
    provision: &Provision,
    patterns: &ReferencePatterns,
    target_ids: &HashSet<&str>,
) -> Vec<ReferenceEdge> {
    let headers: Vec<_> = patterns.article.find_iter(&provision.text).collect();
    let mut references = Vec::new();
    for (index, header) in headers.iter().enumerate() {
        let group_end = headers
            .get(index + 1)
            .map_or(provision.text.len(), regex::Match::start);
        let group = &provision.text[header.end()..group_end];
        references.extend(extract_reference_group(
            provision,
            header.end(),
            group,
            patterns,
            target_ids,
        ));
    }
    references
}

fn extract_reference_group(
    provision: &Provision,
    group_start: usize,
    group: &str,
    patterns: &ReferencePatterns,
    target_ids: &HashSet<&str>,
) -> Vec<ReferenceEdge> {
    let accepted = accepted_numbers(group, patterns);
    if accepted.is_empty() {
        return Vec::new();
    }
    let lower_group = group.to_lowercase();
    if has_external_instrument_context(&lower_group)
        && !has_internal_instrument_context(&lower_group)
    {
        return Vec::new();
    }
    let mut references = direct_reference_edges(
        provision,
        group_start,
        group,
        &accepted,
        patterns,
        target_ids,
    );
    references.extend(range_expansion_edges(
        provision,
        group_start,
        group,
        &accepted,
        target_ids,
    ));
    references
}

fn accepted_numbers<'a>(group: &'a str, patterns: &ReferencePatterns) -> Vec<regex::Match<'a>> {
    let candidates: Vec<_> = patterns.number.find_iter(group).collect();
    let Some(first) = candidates.first() else {
        return Vec::new();
    };
    if !group[..first.start()].trim().is_empty() {
        return Vec::new();
    }
    let mut accepted = vec![*first];
    for candidate in candidates.iter().skip(1) {
        let Some(previous) = accepted.last() else {
            break;
        };
        if !patterns
            .separator
            .is_match(&group[previous.end()..candidate.start()])
        {
            break;
        }
        accepted.push(*candidate);
    }
    accepted
}

fn direct_reference_edges(
    provision: &Provision,
    group_start: usize,
    group: &str,
    accepted: &[regex::Match<'_>],
    patterns: &ReferencePatterns,
    target_ids: &HashSet<&str>,
) -> Vec<ReferenceEdge> {
    accepted
        .iter()
        .enumerate()
        .map(|(index, number_match)| {
            let qualifier_end = accepted
                .get(index + 1)
                .map_or(group.len(), regex::Match::start);
            let qualifier_text = &group[number_match.end()..qualifier_end];
            let qualifier_text = if index + 1 == accepted.len() {
                &qualifier_text[..qualifier_boundary(qualifier_text)]
            } else {
                qualifier_text
            };
            reference_edge(
                provision,
                number_match.as_str(),
                (group_start + number_match.start())..(group_start + number_match.end()),
                canonical_article_id(number_match.as_str()),
                extract_qualifiers(
                    qualifier_text,
                    &patterns.paragraph,
                    &patterns.fraction,
                    &patterns.subsection,
                ),
                ReferenceForm::Direct,
                target_ids,
            )
        })
        .collect()
}

fn range_expansion_edges(
    provision: &Provision,
    group_start: usize,
    group: &str,
    accepted: &[regex::Match<'_>],
    target_ids: &HashSet<&str>,
) -> Vec<ReferenceEdge> {
    let mut references = Vec::new();
    for pair in accepted.windows(2) {
        let separator = group[pair[0].end()..pair[1].start()].trim().to_lowercase();
        let (Some(start), Some(end)) = (
            numeric_article_number(pair[0].as_str()),
            numeric_article_number(pair[1].as_str()),
        ) else {
            continue;
        };
        if !matches!(separator.as_str(), "a" | "al" | "hasta") || end <= start || end - start > 200
        {
            continue;
        }
        let range = (group_start + pair[0].start())..(group_start + pair[1].end());
        let source_span = &provision.text[range.clone()];
        for expanded in (start + 1)..end {
            references.push(reference_edge(
                provision,
                source_span,
                range.clone(),
                canonical_article_id(&expanded.to_string()),
                Vec::new(),
                ReferenceForm::RangeExpansion,
                target_ids,
            ));
        }
    }
    references
}

fn reference_edge(
    provision: &Provision,
    source_span: &str,
    source_range: Range<usize>,
    target_provision_id: String,
    qualifiers: Vec<ReferenceQualifier>,
    reference_form: ReferenceForm,
    target_ids: &HashSet<&str>,
) -> ReferenceEdge {
    let start_char = provision.text[..source_range.start].chars().count();
    let end_char = start_char + provision.text[source_range].chars().count();
    let target_slug = target_provision_id.rsplit(':').next().unwrap_or("unknown");
    let form_slug = match reference_form {
        ReferenceForm::Direct => "direct",
        ReferenceForm::RangeExpansion => "range",
    };
    let resolution_status = if target_ids.contains(target_provision_id.as_str()) {
        ReferenceResolutionStatus::Resolved
    } else {
        ReferenceResolutionStatus::Unresolved
    };
    ReferenceEdge {
        schema_version: SCHEMA_VERSION.to_owned(),
        id: format!(
            "{}:reference:{start_char}-{end_char}:{target_slug}:{form_slug}",
            provision.id
        ),
        source_provision_id: provision.id.clone(),
        source_span: source_span.to_owned(),
        start_char,
        end_char,
        target_instrument_id: provision.instrument_id.clone(),
        target_provision_id,
        qualifiers,
        basis: Basis::ExpressCrossReference,
        confidence: 1.0,
        resolution_status,
        reference_form,
    }
}

fn canonical_article_id(number: &str) -> String {
    let canonical_number = number
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
        .to_lowercase();
    format!("{LRITF_INSTRUMENT_ID}:article:{canonical_number}")
}

fn numeric_article_number(number: &str) -> Option<u32> {
    number.trim().parse().ok()
}

fn has_internal_instrument_context(value: &str) -> bool {
    [
        "de esta ley",
        "de la presente ley",
        "de este ordenamiento",
        "del presente ordenamiento",
    ]
    .iter()
    .any(|marker| value.contains(marker))
}

fn has_external_instrument_context(value: &str) -> bool {
    [
        "de la ley ",
        "del código ",
        "de la constitución ",
        "del reglamento ",
        "de dicha ley",
        "de esa ley",
        "de este código",
        "del presente código",
    ]
    .iter()
    .any(|marker| value.contains(marker))
}

fn extract_qualifiers(
    value: &str,
    paragraph_re: &Regex,
    fraction_re: &Regex,
    subsection_re: &Regex,
) -> Vec<ReferenceQualifier> {
    let mut matches = Vec::new();
    matches.extend(paragraph_re.find_iter(value).map(|item| {
        (
            item.start(),
            ReferenceQualifier {
                qualifier_type: ReferenceQualifierType::Paragraph,
                text: item.as_str().to_owned(),
            },
        )
    }));
    matches.extend(fraction_re.find_iter(value).map(|item| {
        (
            item.start(),
            ReferenceQualifier {
                qualifier_type: ReferenceQualifierType::Fraction,
                text: item.as_str().to_owned(),
            },
        )
    }));
    matches.extend(subsection_re.find_iter(value).map(|item| {
        (
            item.start(),
            ReferenceQualifier {
                qualifier_type: ReferenceQualifierType::Subsection,
                text: item.as_str().to_owned(),
            },
        )
    }));
    matches.sort_by_key(|(start, _)| *start);
    matches
        .into_iter()
        .map(|(_, qualifier)| qualifier)
        .collect()
}

fn qualifier_boundary(value: &str) -> usize {
    value
        .char_indices()
        .find_map(|(index, character)| matches!(character, '.' | '\n').then_some(index))
        .unwrap_or(value.len())
}

pub fn extract_reform_transitories(raw: &str) -> Result<Vec<TemporalEvidence>> {
    let publication_re = Regex::new(
        r"Publicado en el Diario Oficial de la Federación el (\d{1,2}) de ([a-z]+) de (\d{4})",
    )?;
    let ordinal_re = Regex::new(
        r"^(Primero|Segundo|Tercero|Cuarto|Quinto|Sexto|Séptimo|Octavo|Noveno|Décimo(?:\s+(?:Primero|Segundo|Tercero|Cuarto|Quinto|Sexto))?)\.(?:-)?\s*(.*)$",
    )?;
    let mut in_reform_appendix = false;
    let mut in_transitories = false;
    let mut publication_date: Option<NaiveDate> = None;
    let mut current: Option<ReformEvidenceBuilder> = None;
    let mut evidence = Vec::new();

    for block in normalized_blocks(raw) {
        if block.contains("ARTÍCULOS TRANSITORIOS DE DECRETOS DE REFORMA") {
            in_reform_appendix = true;
            continue;
        }
        if !in_reform_appendix {
            continue;
        }
        if block.starts_with("DECRETO por el que") {
            flush_reform_evidence(&mut current, &mut evidence);
            in_transitories = false;
        }
        if let Some(captures) = publication_re.captures(&block) {
            publication_date = spanish_date(&captures[1], &captures[2], &captures[3]);
            continue;
        }
        if block.eq_ignore_ascii_case("Transitorios") || block.ends_with(" Transitorios") {
            in_transitories = true;
            continue;
        }
        if !in_transitories {
            continue;
        }
        if block.starts_with("Ciudad de México") {
            flush_reform_evidence(&mut current, &mut evidence);
            in_transitories = false;
            continue;
        }
        if let Some(captures) = ordinal_re.captures(&block) {
            flush_reform_evidence(&mut current, &mut evidence);
            let date = publication_date
                .context("found reform transitory without its Diario Oficial publication date")?;
            current = Some(ReformEvidenceBuilder {
                date,
                ordinal: captures[1].to_owned(),
                blocks: vec![captures[2].trim().to_owned()],
            });
        } else if let Some(builder) = &mut current {
            builder.blocks.push(block);
        }
    }
    flush_reform_evidence(&mut current, &mut evidence);
    Ok(evidence)
}

#[must_use]
pub fn validate_lritf(
    provisions: &[Provision],
    references: &[ReferenceEdge],
    expected_min_articles: usize,
    expected_transitories: usize,
) -> ValidationReport {
    let article_count = provisions
        .iter()
        .filter(|item| item.provision_type == ProvisionType::Article)
        .count();
    let transitory_count = provisions
        .iter()
        .filter(|item| item.provision_type == ProvisionType::Transitory)
        .count();
    let mut issues = Vec::new();

    if article_count < expected_min_articles {
        issues.push(error(
            "article_count",
            format!("expected at least {expected_min_articles} articles, found {article_count}"),
            None,
        ));
    }
    if transitory_count != expected_transitories {
        issues.push(error(
            "transitory_count",
            format!("expected {expected_transitories} transitories, found {transitory_count}"),
            None,
        ));
    }

    let mut ids = HashSet::new();
    let mut expected_number = 1_u32;
    for provision in provisions {
        if !ids.insert(&provision.id) {
            issues.push(error(
                "duplicate_id",
                "duplicate canonical identifier".to_owned(),
                Some(provision.id.clone()),
            ));
        }
        if provision.text.trim().is_empty() {
            issues.push(error(
                "empty_text",
                "provision has no body text".to_owned(),
                Some(provision.id.clone()),
            ));
        }
        if provision.text.contains("CÁMARA DE DIPUTADOS")
            || provision
                .text
                .contains("Secretaría de Servicios Parlamentarios")
        {
            issues.push(error(
                "header_contamination",
                "page header leaked into provision text".to_owned(),
                Some(provision.id.clone()),
            ));
        }
        if provision.provision_type == ProvisionType::Article {
            match provision.number.parse::<u32>() {
                Ok(number) if number == expected_number => expected_number += 1,
                Ok(number) => issues.push(error(
                    "article_order",
                    format!("expected article {expected_number}, found {number}"),
                    Some(provision.id.clone()),
                )),
                Err(_) => issues.push(ValidationIssue {
                    severity: Severity::Warning,
                    code: "non_numeric_article".to_owned(),
                    message: "article suffix requires ordering review".to_owned(),
                    provision_id: Some(provision.id.clone()),
                }),
            }
        }
    }

    validate_references(provisions, references, &mut issues);

    ValidationReport {
        schema_version: SCHEMA_VERSION.to_owned(),
        instrument_id: LRITF_INSTRUMENT_ID.to_owned(),
        valid: !issues.iter().any(|item| item.severity == Severity::Error),
        article_count,
        transitory_count,
        reference_count: references.len(),
        issues,
    }
}

fn validate_references(
    provisions: &[Provision],
    references: &[ReferenceEdge],
    issues: &mut Vec<ValidationIssue>,
) {
    let provisions_by_id: HashMap<_, _> = provisions
        .iter()
        .map(|provision| (provision.id.as_str(), provision))
        .collect();
    let mut reference_ids = HashSet::new();
    for reference in references {
        if !reference_ids.insert(&reference.id) {
            issues.push(error(
                "duplicate_reference_id",
                "duplicate canonical reference identifier".to_owned(),
                Some(reference.source_provision_id.clone()),
            ));
        }
        validate_reference(reference, &provisions_by_id, issues);
    }
}

fn validate_reference(
    reference: &ReferenceEdge,
    provisions_by_id: &HashMap<&str, &Provision>,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(source) = provisions_by_id.get(reference.source_provision_id.as_str()) else {
        issues.push(error(
            "reference_source_missing",
            format!(
                "reference source does not exist: {}",
                reference.source_provision_id
            ),
            Some(reference.source_provision_id.clone()),
        ));
        return;
    };
    validate_reference_span(reference, source, issues);
    if reference.target_instrument_id != source.instrument_id {
        issues.push(error(
            "reference_instrument_mismatch",
            "internal reference target instrument differs from its source instrument".to_owned(),
            Some(reference.source_provision_id.clone()),
        ));
    }
    validate_reference_target(reference, provisions_by_id, issues);
    if reference.basis != Basis::ExpressCrossReference {
        issues.push(error(
            "reference_basis",
            "canonical reference must use express_cross_reference basis".to_owned(),
            Some(reference.source_provision_id.clone()),
        ));
    }
    if !(0.0..=1.0).contains(&reference.confidence) {
        issues.push(error(
            "reference_confidence",
            "reference confidence must be between zero and one".to_owned(),
            Some(reference.source_provision_id.clone()),
        ));
    }
}

fn validate_reference_span(
    reference: &ReferenceEdge,
    source: &Provision,
    issues: &mut Vec<ValidationIssue>,
) {
    match char_slice(&source.text, reference.start_char, reference.end_char) {
        Some(span) if span == reference.source_span => {}
        Some(span) => issues.push(error(
            "reference_span_mismatch",
            format!(
                "reference span {:?} does not match source text {:?}",
                reference.source_span, span
            ),
            Some(reference.source_provision_id.clone()),
        )),
        None => issues.push(error(
            "reference_offsets_invalid",
            format!(
                "reference character offsets {}..{} are outside the source text",
                reference.start_char, reference.end_char
            ),
            Some(reference.source_provision_id.clone()),
        )),
    }
}

fn validate_reference_target(
    reference: &ReferenceEdge,
    provisions_by_id: &HashMap<&str, &Provision>,
    issues: &mut Vec<ValidationIssue>,
) {
    let target_exists = provisions_by_id.contains_key(reference.target_provision_id.as_str());
    let (code, message) = match (&reference.resolution_status, target_exists) {
        (ReferenceResolutionStatus::Resolved, false) => (
            "resolved_reference_target_missing",
            format!(
                "resolved reference target does not exist: {}",
                reference.target_provision_id
            ),
        ),
        (ReferenceResolutionStatus::Unresolved, true) => (
            "reference_resolution_stale",
            format!(
                "reference target exists but is marked unresolved: {}",
                reference.target_provision_id
            ),
        ),
        (ReferenceResolutionStatus::Unresolved, false) => (
            "unresolved_internal_reference",
            format!(
                "internal reference target does not exist: {}",
                reference.target_provision_id
            ),
        ),
        (ReferenceResolutionStatus::Resolved, true) => return,
    };
    issues.push(error(
        code,
        message,
        Some(reference.source_provision_id.clone()),
    ));
}

fn char_slice(value: &str, start: usize, end: usize) -> Option<String> {
    if start > end || end > value.chars().count() {
        return None;
    }
    Some(value.chars().skip(start).take(end - start).collect())
}

fn normalized_blocks(raw: &str) -> Vec<String> {
    let page_number = Regex::new(r"^\d+\s+de\s+\d+$").expect("static regex");
    let mut blocks = Vec::new();
    let mut current = String::new();
    let mut pending_blank = false;
    let mut crossed_page_furniture = false;

    for source_line in raw.lines() {
        let line = source_line.trim();
        if line.is_empty() {
            pending_blank = true;
            continue;
        }
        if is_page_furniture(line, &page_number) {
            if !current.is_empty() {
                crossed_page_furniture = true;
            }
            pending_blank = false;
            continue;
        }
        if pending_blank && !crossed_page_furniture {
            flush_block(&mut current, &mut blocks);
        }
        pending_blank = false;
        crossed_page_furniture = false;
        if is_immediate_structural(line) {
            flush_block(&mut current, &mut blocks);
            blocks.push(collapse_whitespace(line));
            continue;
        }
        if is_provision_start(line) {
            flush_block(&mut current, &mut blocks);
            current.push_str(line);
            continue;
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(line);
    }
    flush_block(&mut current, &mut blocks);
    blocks
}

fn is_page_furniture(line: &str, page_number: &Regex) -> bool {
    line == SOURCE_HEADER
        || line.starts_with("CÁMARA DE DIPUTADOS DEL H. CONGRESO DE LA UNIÓN")
        || line == "Secretaría General"
        || line == "Secretaría de Servicios Parlamentarios"
        || line.starts_with("Última Reforma DOF ")
        || page_number.is_match(line)
}

fn is_immediate_structural(line: &str) -> bool {
    line.starts_with("ARTÍCULOS SEGUNDO A DÉCIMO")
        || line.starts_with("TÍTULO ")
        || line.starts_with("CAPÍTULO ")
        || line == "DISPOSICIONES TRANSITORIAS"
}

fn is_provision_start(line: &str) -> bool {
    line.starts_with("Artículo ")
        || TRANSITORY_ORDINALS
            .iter()
            .any(|ordinal| line.starts_with(&format!("{ordinal}.-")))
}

fn flush_block(current: &mut String, blocks: &mut Vec<String>) {
    if !current.is_empty() {
        blocks.push(collapse_whitespace(current));
        current.clear();
    }
}

fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn parse_transitory_start(block: &str) -> Option<(&str, &str)> {
    TRANSITORY_ORDINALS.iter().find_map(|ordinal| {
        block
            .strip_prefix(&format!("{ordinal}.-"))
            .map(|body| (*ordinal, body.trim()))
    })
}

fn slug(value: &str) -> String {
    value
        .to_lowercase()
        .replace('á', "a")
        .replace('é', "e")
        .replace('í', "i")
        .replace('ó', "o")
        .replace(['ú', 'ü'], "u")
        .replace(' ', "-")
}

fn spanish_date(day: &str, month: &str, year: &str) -> Option<NaiveDate> {
    let month = match month {
        "enero" => 1,
        "febrero" => 2,
        "marzo" => 3,
        "abril" => 4,
        "mayo" => 5,
        "junio" => 6,
        "julio" => 7,
        "agosto" => 8,
        "septiembre" => 9,
        "octubre" => 10,
        "noviembre" => 11,
        "diciembre" => 12,
        _ => return None,
    };
    NaiveDate::from_ymd_opt(year.parse().ok()?, month, day.parse().ok()?)
}

fn pdftotext_version() -> String {
    Command::new("pdftotext")
        .arg("-v")
        .output()
        .ok()
        .and_then(|output| {
            let text = if output.stderr.is_empty() {
                String::from_utf8(output.stdout).ok()?
            } else {
                String::from_utf8(output.stderr).ok()?
            };
            text.lines().next().map(str::to_owned)
        })
        .unwrap_or_else(|| "pdftotext (version unavailable)".to_owned())
}

fn error(code: &str, message: String, provision_id: Option<String>) -> ValidationIssue {
    ValidationIssue {
        severity: Severity::Error,
        code: code.to_owned(),
        message,
        provision_id,
    }
}

struct ReformEvidenceBuilder {
    date: NaiveDate,
    ordinal: String,
    blocks: Vec<String>,
}

impl ReformEvidenceBuilder {
    fn finish(self) -> TemporalEvidence {
        let date = self.date.format("%Y-%m-%d");
        TemporalEvidence {
            provision_id: format!(
                "{LRITF_INSTRUMENT_ID}:amendment:{date}:transitory:{}",
                slug(&self.ordinal)
            ),
            label: format!("Transitorio {} — Decreto DOF {date}", self.ordinal),
            text: self
                .blocks
                .into_iter()
                .filter(|block| !block.is_empty())
                .collect::<Vec<_>>()
                .join("\n\n"),
        }
    }
}

fn flush_reform_evidence(
    current: &mut Option<ReformEvidenceBuilder>,
    evidence: &mut Vec<TemporalEvidence>,
) {
    if let Some(builder) = current.take() {
        evidence.push(builder.finish());
    }
}

struct ProvisionBuilder {
    provision_type: ProvisionType,
    number: String,
    label: String,
    title: Option<String>,
    chapter: Option<String>,
    blocks: Vec<String>,
}

impl ProvisionBuilder {
    fn article(
        number: String,
        initial: &str,
        title: Option<String>,
        chapter: Option<String>,
    ) -> Self {
        let mut value = Self {
            label: format!("Artículo {number}"),
            number,
            provision_type: ProvisionType::Article,
            title,
            chapter,
            blocks: Vec::new(),
        };
        value.push_block(initial);
        value
    }

    fn transitory(ordinal: &str, initial: &str) -> Self {
        let mut value = Self {
            label: ordinal.to_owned(),
            number: ordinal.to_owned(),
            provision_type: ProvisionType::Transitory,
            title: None,
            chapter: None,
            blocks: Vec::new(),
        };
        value.push_block(initial);
        value
    }

    fn push_block(&mut self, value: &str) {
        let value = value.trim();
        if !value.is_empty() {
            self.blocks.push(value.to_owned());
        }
    }

    fn finish(self, publication_date: NaiveDate) -> Provision {
        let kind = match self.provision_type {
            ProvisionType::Article => "article",
            ProvisionType::Transitory => "transitory",
        };
        let canonical_number = if self.provision_type == ProvisionType::Article {
            self.number.to_lowercase().replace(' ', "-")
        } else {
            slug(&self.number)
        };
        Provision {
            schema_version: SCHEMA_VERSION.to_owned(),
            id: format!("{LRITF_INSTRUMENT_ID}:{kind}:{canonical_number}"),
            instrument_id: LRITF_INSTRUMENT_ID.to_owned(),
            provision_type: self.provision_type,
            label: self.label,
            number: self.number,
            heading_context: HeadingContext {
                title: self.title,
                chapter: self.chapter,
            },
            text: self.blocks.join("\n\n"),
            publication_date,
            effective_from: None,
            effective_to: None,
            temporal_status: TemporalStatus::Unknown,
            temporal_basis: None,
            temporal_confidence: None,
            review_status: ReviewStatus::NotAnalyzed,
            transitory_effects: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use lex_core::{ReferenceForm, ReferenceQualifierType};
    use pretty_assertions::assert_eq;

    use super::{
        extract_internal_references, extract_reform_transitories, parse_lritf, validate_lritf,
    };

    const FIXTURE: &str = include_str!("../../../fixtures/lritf/parser-sample.txt");
    const REFERENCE_FIXTURE: &str = include_str!("../../../fixtures/lritf/reference-sample.txt");

    #[test]
    fn parses_articles_and_statute_transitories_without_page_furniture() {
        let date = NaiveDate::from_ymd_opt(2018, 3, 9).unwrap();
        let provisions = parse_lritf(FIXTURE, date).unwrap();

        assert_eq!(provisions.len(), 4);
        assert_eq!(
            provisions[0].id,
            "urn:lex-mx:federal:statute:lritf:article:1"
        );
        assert_eq!(
            provisions[0].heading_context.title.as_deref(),
            Some("Título I")
        );
        assert_eq!(provisions[1].text, "Texto inicial que cruza una página.");
        assert_eq!(
            provisions[2].id,
            "urn:lex-mx:federal:statute:lritf:transitory:primera"
        );
        assert!(
            !provisions
                .iter()
                .any(|item| item.text.contains("CÁMARA DE DIPUTADOS"))
        );
        assert!(
            !provisions
                .iter()
                .any(|item| item.text.contains("reforma posterior"))
        );
    }

    #[test]
    fn validates_expected_counts_and_order() {
        let date = NaiveDate::from_ymd_opt(2018, 3, 9).unwrap();
        let provisions = parse_lritf(FIXTURE, date).unwrap();
        let report = validate_lritf(&provisions, &[], 2, 2);
        assert!(report.valid, "{:?}", report.issues);
    }

    #[test]
    fn extracts_compound_qualified_repeated_and_ranged_internal_references() {
        let date = NaiveDate::from_ymd_opt(2018, 3, 9).unwrap();
        let provisions = parse_lritf(REFERENCE_FIXTURE, date).unwrap();
        let references = extract_internal_references(&provisions).unwrap();

        let article_one: Vec<_> = references
            .iter()
            .filter(|edge| edge.source_provision_id.ends_with(":article:1"))
            .collect();
        assert_eq!(article_one.len(), 3);
        assert_eq!(article_one[0].source_span, "2");
        assert_eq!(
            article_one[0].qualifiers[0].qualifier_type,
            ReferenceQualifierType::Fraction
        );
        assert_eq!(article_one[0].qualifiers[0].text, "fracción II");
        assert_eq!(
            article_one[1].qualifiers[0].qualifier_type,
            ReferenceQualifierType::Paragraph
        );
        assert_eq!(article_one[1].qualifiers[0].text, "segundo párrafo");

        let article_eight: Vec<_> = references
            .iter()
            .filter(|edge| edge.source_provision_id.ends_with(":article:8"))
            .collect();
        assert_eq!(article_eight.len(), 4);
        assert!(article_eight.iter().any(|edge| {
            edge.target_provision_id.ends_with(":article:6")
                && edge.reference_form == ReferenceForm::RangeExpansion
                && edge.source_span == "5 al 7"
        }));
        assert!(
            references
                .iter()
                .all(|edge| !edge.target_provision_id.ends_with(":article:89"))
        );
        assert!(
            article_eight
                .iter()
                .any(|edge| edge.target_provision_id.ends_with(":article:6-bis"))
        );

        let repeated_article_two = references
            .iter()
            .filter(|edge| {
                edge.source_provision_id.ends_with(":transitory:primera")
                    && edge.target_provision_id.ends_with(":article:2")
            })
            .count();
        assert_eq!(repeated_article_two, 2);

        let report = validate_lritf(&provisions, &references, 8, 1);
        assert!(report.valid, "{:?}", report.issues);
        assert_eq!(report.reference_count, references.len());
    }

    #[test]
    fn extracts_reform_decree_transitories_as_separate_evidence() {
        let raw = r"
ARTÍCULOS TRANSITORIOS DE DECRETOS DE REFORMA

DECRETO por el que se reforman diversas disposiciones.
Publicado en el Diario Oficial de la Federación el 14 de noviembre de 2025

Transitorios

Primero. El presente Decreto entrará en vigor al día siguiente.

Segundo. La aplicación será gradual.

Ciudad de México, a 1 de octubre de 2025.
";
        let evidence = extract_reform_transitories(raw).unwrap();
        assert_eq!(evidence.len(), 2);
        assert_eq!(
            evidence[1].provision_id,
            "urn:lex-mx:federal:statute:lritf:amendment:2025-11-14:transitory:segundo"
        );
        assert_eq!(evidence[1].text, "La aplicación será gradual.");
    }
}
