//! Parser for the January 28, 2021 disposiciones de carácter general
//! applicable to instituciones de fondos de pago electrónico (DCG-IFPE-2021).
//!
//! The operational CNBV PDF carries the índice, considerandos, seven
//! chapters, 59 articles, and four transitories; the eight annex bodies are
//! published only in the formal DOF note and are parsed from its extracted
//! text. The main text is extracted with page breaks preserved (`pdftotext
//! -layout`): every page transition in this source emits one blank line, so
//! a paragraph is merged across a page break unless the previous line ends a
//! sentence or enumeration (`.`, `:`, or `;`).

use anyhow::{Result, bail};
use chrono::NaiveDate;
use lex_core::{
    HeadingContext, Provision, ProvisionType, ReviewStatus, SCHEMA_VERSION, TemporalStatus,
};
use regex::Regex;

const TRANSITORY_ORDINALS: &[&str] = &[
    "PRIMERO", "SEGUNDO", "TERCERO", "CUARTO", "QUINTO", "SEXTO", "SÉPTIMO", "OCTAVO", "NOVENO",
    "DÉCIMO",
];

/// Column threshold that separates the term column from the definition
/// column in the Article 1 two-column layout. Terms indent at most 12
/// columns in the source; definition text never starts before column 20.
const DEFINITION_COLUMN_MIN: usize = 20;
const TERM_INDENT_MAX: usize = 12;

pub fn parse_dcg(
    main_raw: &str,
    formal_text: &str,
    instrument_id: &str,
    publication_date: NaiveDate,
    definition_layout_articles: &[String],
) -> Result<Vec<Provision>> {
    let mut provisions = parse_main_text(
        main_raw,
        instrument_id,
        publication_date,
        definition_layout_articles,
    )?;
    provisions.extend(parse_annexes(formal_text, instrument_id, publication_date)?);
    if provisions.is_empty() {
        bail!("no DCG provisions recognized");
    }
    Ok(provisions)
}

struct HeadingState {
    chapter: Option<String>,
    section: Option<String>,
    apartado: Option<String>,
}

fn parse_main_text(
    raw: &str,
    instrument_id: &str,
    publication_date: NaiveDate,
    definition_layout_articles: &[String],
) -> Result<Vec<Provision>> {
    let chapter_re = Regex::new(r"^CAPÍTULO\s+([IVXLCDM]+)$")?;
    let section_re = Regex::new(r"^Sección\s+([A-Za-zÁÉÍÓÚáéíóú]+)$")?;
    let apartado_re = Regex::new(r"^Apartado\s+([A-Z])$")?;
    let article_re = Regex::new(r"^Artículo\s+(\d+)\s*\.?-\s*(.*)$")?;
    let transitory_re = new_transitory_regex()?;

    let mut provisions = Vec::new();
    let mut current: Option<DcgProvisionBuilder> = None;
    let mut headings = HeadingState {
        chapter: None,
        section: None,
        apartado: None,
    };
    let mut in_transitories = false;
    let mut pending_blank = false;
    let mut crossed_page_break = false;

    for source_line in raw.lines() {
        let crossed_here = source_line.starts_with('\u{c}');
        if crossed_here {
            crossed_page_break = true;
        }
        let line = source_line.trim_start_matches('\u{c}');
        if line.trim().is_empty() {
            pending_blank = true;
            continue;
        }
        let trimmed = line.trim();

        if let Some(captures) = chapter_re.captures(trimmed) {
            flush(&mut current, &mut provisions, publication_date);
            headings.chapter = Some(format!("Capítulo {}", &captures[1]));
            headings.section = None;
            headings.apartado = None;
            (pending_blank, crossed_page_break) = (false, false);
            continue;
        }
        if let Some(captures) = section_re.captures(trimmed) {
            flush(&mut current, &mut provisions, publication_date);
            headings.section = Some(format!("Sección {}", &captures[1]));
            headings.apartado = None;
            (pending_blank, crossed_page_break) = (false, false);
            continue;
        }
        if let Some(captures) = apartado_re.captures(trimmed) {
            flush(&mut current, &mut provisions, publication_date);
            headings.apartado = Some(format!("Apartado {}", &captures[1]));
            (pending_blank, crossed_page_break) = (false, false);
            continue;
        }
        if trimmed == "TRANSITORIOS" {
            flush(&mut current, &mut provisions, publication_date);
            in_transitories = true;
            (pending_blank, crossed_page_break) = (false, false);
            continue;
        }
        if in_transitories && trimmed.starts_with("Ciudad de México") {
            flush(&mut current, &mut provisions, publication_date);
            (pending_blank, crossed_page_break) = (false, false);
            continue;
        }

        if !in_transitories {
            if let Some(captures) = article_re.captures(trimmed) {
                flush(&mut current, &mut provisions, publication_date);
                let number = captures[1].to_owned();
                let definition_layout = definition_layout_articles.contains(&number);
                current = Some(DcgProvisionBuilder::article(
                    instrument_id,
                    number,
                    captures[2].trim(),
                    &headings,
                    definition_layout,
                ));
                (pending_blank, crossed_page_break) = (false, false);
                continue;
            }
        } else if let Some(captures) = transitory_re.captures(trimmed) {
            flush(&mut current, &mut provisions, publication_date);
            current = Some(DcgProvisionBuilder::transitory(
                instrument_id,
                &captures[1],
                captures[2].trim(),
            ));
            (pending_blank, crossed_page_break) = (false, false);
            continue;
        }

        if let Some(builder) = &mut current {
            builder.push_line(line, pending_blank, crossed_page_break);
        }
        (pending_blank, crossed_page_break) = (false, false);
    }
    flush(&mut current, &mut provisions, publication_date);
    Ok(provisions)
}

fn new_transitory_regex() -> Result<Regex> {
    Ok(Regex::new(&format!(
        r"^({})\.-\s*(.*)$",
        TRANSITORY_ORDINALS.join("|")
    ))?)
}

fn flush(
    current: &mut Option<DcgProvisionBuilder>,
    provisions: &mut Vec<Provision>,
    publication_date: NaiveDate,
) {
    if let Some(builder) = current.take() {
        provisions.push(builder.finish(publication_date));
    }
}

fn parse_annexes(
    formal_text: &str,
    instrument_id: &str,
    publication_date: NaiveDate,
) -> Result<Vec<Provision>> {
    let heading_re = Regex::new(r"(?i)^anexo\s+(\d+)$")?;
    let end_re = Regex::new(r"^_{4,}$")?;
    let mut annexes = Vec::new();
    let mut current: Option<(String, String, Vec<String>)> = None;

    for line in formal_text.lines() {
        let trimmed = line.trim();
        if end_re.is_match(trimmed) {
            break;
        }
        if let Some(captures) = heading_re.captures(trimmed) {
            if let Some(annex) = current.take() {
                annexes.push(annex);
            }
            current = Some((trimmed.to_owned(), captures[1].to_owned(), Vec::new()));
            continue;
        }
        if let Some((_, _, lines)) = &mut current
            && !trimmed.is_empty()
        {
            lines.push(trimmed.to_owned());
        }
    }
    if let Some(annex) = current.take() {
        annexes.push(annex);
    }

    annexes
        .into_iter()
        .map(|(label, number, lines)| {
            if lines.is_empty() {
                bail!("annex {label} has no body text in the formal source");
            }
            Ok(Provision {
                schema_version: SCHEMA_VERSION.to_owned(),
                id: format!("{instrument_id}:annex:{number}"),
                instrument_id: instrument_id.to_owned(),
                provision_type: ProvisionType::Annex,
                label,
                number,
                heading_context: HeadingContext {
                    title: None,
                    chapter: None,
                    section: None,
                    apartado: None,
                },
                text: lines.join("\n\n"),
                publication_date,
                effective_from: None,
                effective_to: None,
                temporal_status: TemporalStatus::Unknown,
                temporal_basis: None,
                temporal_confidence: None,
                review_status: ReviewStatus::NotAnalyzed,
                transitory_effects: Vec::new(),
            })
        })
        .collect()
}

struct DcgProvisionBuilder {
    instrument_id: String,
    provision_type: ProvisionType,
    number: String,
    label: String,
    heading_context: HeadingContext,
    /// Completed paragraphs, whitespace-collapsed.
    paragraphs: Vec<String>,
    /// Lines of the paragraph currently being accumulated.
    current_paragraph: Vec<String>,
    /// Raw source lines retained for definition-layout reconstruction.
    definition_layout: bool,
    raw_lines: Vec<RawLine>,
}

struct RawLine {
    text: String,
    after_blank: bool,
    after_page_break: bool,
}

impl DcgProvisionBuilder {
    fn article(
        instrument_id: &str,
        number: String,
        initial: &str,
        headings: &HeadingState,
        definition_layout: bool,
    ) -> Self {
        let mut builder = Self {
            instrument_id: instrument_id.to_owned(),
            provision_type: ProvisionType::Article,
            label: format!("Artículo {number}"),
            number,
            heading_context: HeadingContext {
                title: None,
                chapter: headings.chapter.clone(),
                section: headings.section.clone(),
                apartado: headings.apartado.clone(),
            },
            paragraphs: Vec::new(),
            current_paragraph: Vec::new(),
            definition_layout,
            raw_lines: Vec::new(),
        };
        builder.push_line(initial, false, false);
        builder
    }

    fn transitory(instrument_id: &str, ordinal: &str, initial: &str) -> Self {
        let mut builder = Self {
            instrument_id: instrument_id.to_owned(),
            provision_type: ProvisionType::Transitory,
            label: ordinal.to_owned(),
            number: ordinal.to_owned(),
            heading_context: HeadingContext {
                title: None,
                chapter: None,
                section: None,
                apartado: None,
            },
            paragraphs: Vec::new(),
            current_paragraph: Vec::new(),
            definition_layout: false,
            raw_lines: Vec::new(),
        };
        builder.push_line(initial, false, false);
        builder
    }

    fn push_line(&mut self, line: &str, after_blank: bool, after_page_break: bool) {
        if self.definition_layout {
            self.raw_lines.push(RawLine {
                text: line.to_owned(),
                after_blank,
                after_page_break,
            });
            return;
        }
        let continues_paragraph =
            !after_blank || (after_page_break && !ends_paragraph(self.current_paragraph.last()));
        if !continues_paragraph {
            self.finish_paragraph();
        }
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            self.current_paragraph.push(trimmed.to_owned());
        }
    }

    fn finish_paragraph(&mut self) {
        if !self.current_paragraph.is_empty() {
            self.paragraphs.push(self.current_paragraph.join(" "));
            self.current_paragraph.clear();
        }
    }

    fn finish(mut self, publication_date: NaiveDate) -> Provision {
        if self.definition_layout {
            self.paragraphs = reconstruct_definition_layout(&self.raw_lines);
        } else {
            self.finish_paragraph();
        }
        let (kind, canonical_number) = match self.provision_type {
            ProvisionType::Article => ("article", self.number.to_lowercase()),
            ProvisionType::Transitory => ("transitory", slug(&self.number)),
            ProvisionType::Annex => ("annex", self.number.to_lowercase()),
        };
        Provision {
            schema_version: SCHEMA_VERSION.to_owned(),
            id: format!("{}:{kind}:{canonical_number}", self.instrument_id),
            instrument_id: self.instrument_id,
            provision_type: self.provision_type,
            label: self.label,
            number: self.number,
            heading_context: self.heading_context,
            text: self.paragraphs.join("\n\n"),
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

fn ends_paragraph(last_line: Option<&String>) -> bool {
    last_line.is_some_and(|line| line.trim_end().ends_with(['.', ':', ';']))
}

/// Reconstruct the two-column term/definition layout used by Article 1.
///
/// Lines indented at least [`DEFINITION_COLUMN_MIN`] columns continue the
/// current definition. Lines starting within the term column are split on
/// their first run of three or more spaces into a term fragment and a
/// definition fragment; term fragments accumulate until one ends with `:`,
/// which completes the term. A new entry begins whenever a term fragment
/// appears after the previous term was completed. Lines before the first
/// term/definition line are ordinary intro paragraphs. Blank lines caused by
/// page breaks do not split entries or definitions.
fn reconstruct_definition_layout(raw_lines: &[RawLine]) -> Vec<String> {
    let mut paragraphs = Vec::new();
    let mut intro: Vec<String> = Vec::new();
    let mut entries: Vec<DefinitionEntry> = Vec::new();
    let mut in_table = false;

    for line in raw_lines {
        let indent = line.text.len() - line.text.trim_start().len();
        let split = split_definition_columns(&line.text);
        if !in_table {
            if indent <= TERM_INDENT_MAX && split.is_some() {
                in_table = true;
            } else {
                if line.after_blank && !line.after_page_break && !intro.is_empty() {
                    paragraphs.push(intro.join(" "));
                    intro.clear();
                }
                intro.push(line.text.trim().to_owned());
                continue;
            }
        }
        let real_blank = line.after_blank && !line.after_page_break;
        if indent >= DEFINITION_COLUMN_MIN || (split.is_none() && indent > TERM_INDENT_MAX) {
            if let Some(entry) = entries.last_mut() {
                entry.push_definition(line.text.trim(), real_blank);
            }
            continue;
        }
        let (term_fragment, definition_fragment) = match split {
            Some(parts) => parts,
            // A term-column line without a definition fragment alongside.
            None => (line.text.trim().to_owned(), String::new()),
        };
        let start_new_entry = entries.last().is_none_or(DefinitionEntry::term_complete);
        if start_new_entry {
            entries.push(DefinitionEntry::default());
        }
        let entry = entries.last_mut().expect("entry exists");
        entry.push_term(&term_fragment);
        if !definition_fragment.is_empty() {
            entry.push_definition(&definition_fragment, false);
        }
    }

    if !intro.is_empty() {
        paragraphs.push(intro.join(" "));
    }
    for entry in entries {
        paragraphs.extend(entry.into_paragraphs());
    }
    paragraphs
}

fn split_definition_columns(line: &str) -> Option<(String, String)> {
    let indent = line.len() - line.trim_start().len();
    if indent > TERM_INDENT_MAX {
        return None;
    }
    let trimmed = line.trim_start();
    let mut spaces = 0;
    for (offset, character) in trimmed.char_indices() {
        if character == ' ' {
            spaces += 1;
            continue;
        }
        if spaces >= 3 && indent + offset >= DEFINITION_COLUMN_MIN {
            let term = trimmed[..offset - spaces].trim().to_owned();
            let definition = trimmed[offset..].trim().to_owned();
            if !term.is_empty() && !definition.is_empty() {
                return Some((term, definition));
            }
        }
        spaces = 0;
    }
    None
}

#[derive(Default)]
struct DefinitionEntry {
    term_fragments: Vec<String>,
    definition_paragraphs: Vec<Vec<String>>,
}

impl DefinitionEntry {
    fn term_complete(&self) -> bool {
        self.term_fragments
            .last()
            .is_some_and(|fragment| fragment.ends_with(':'))
    }

    fn push_term(&mut self, fragment: &str) {
        self.term_fragments.push(fragment.to_owned());
    }

    fn push_definition(&mut self, fragment: &str, paragraph_break: bool) {
        if paragraph_break || self.definition_paragraphs.is_empty() {
            self.definition_paragraphs.push(Vec::new());
        }
        self.definition_paragraphs
            .last_mut()
            .expect("definition paragraph exists")
            .push(fragment.to_owned());
    }

    fn into_paragraphs(self) -> Vec<String> {
        let term = self.term_fragments.join(" ");
        let mut definitions = self
            .definition_paragraphs
            .into_iter()
            .map(|paragraph| paragraph.join(" "));
        let first = definitions.next().unwrap_or_default();
        let mut paragraphs = vec![format!("{term} {first}").trim().to_owned()];
        paragraphs.extend(definitions);
        paragraphs
    }
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
