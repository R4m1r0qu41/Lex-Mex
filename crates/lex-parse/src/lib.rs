use std::{
    collections::HashSet,
    fs,
    path::Path,
    process::{Command, Stdio},
};

use anyhow::{Context, Result, bail};
use chrono::NaiveDate;
use lex_core::{
    HeadingContext, LRITF_INSTRUMENT_ID, Provision, ProvisionType, ReviewStatus, SCHEMA_VERSION,
    Severity, TemporalEvidence, TemporalStatus, ValidationIssue, ValidationReport,
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

    ValidationReport {
        schema_version: SCHEMA_VERSION.to_owned(),
        instrument_id: LRITF_INSTRUMENT_ID.to_owned(),
        valid: !issues.iter().any(|item| item.severity == Severity::Error),
        article_count,
        transitory_count,
        issues,
    }
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
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;

    use super::{extract_reform_transitories, parse_lritf, validate_lritf};

    const FIXTURE: &str = include_str!("../../../fixtures/lritf/parser-sample.txt");

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
        let report = validate_lritf(&provisions, 2, 2);
        assert!(report.valid, "{:?}", report.issues);
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
