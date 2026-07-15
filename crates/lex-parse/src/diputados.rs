//! Generic parser for Cámara de Diputados consolidated law/código PDFs.
//!
//! Generalizes the original single-instrument LRITF parser: instrument
//! identity, running-header furniture, and main-document stop markers come
//! from adapter configuration; the article grammar comes from
//! [`crate::labels`]; transitory ordinals cover masculine and feminine
//! forms through the trigésimo compounds; and the reform-decree appendix
//! is extracted in the same pass so the caller never rescans the raw text.
//! The committed LRITF corpus is the byte-identity fixture for this
//! module: parsing LRITF through it must reproduce that corpus exactly.

use anyhow::{Result, bail};
use chrono::NaiveDate;
use lex_core::{
    HeadingContext, Provision, ProvisionType, ReviewStatus, SCHEMA_VERSION, TemporalEvidence,
};
use regex::Regex;

use crate::labels;
use crate::{collapse_whitespace, reform_evidence_item, slug, spanish_date};

/// Instrument-specific configuration for the generic Diputados parser,
/// built by the CLI from the adapter's `SourceConfig`.
#[derive(Debug, Clone)]
pub struct DiputadosOptions {
    pub instrument_id: String,
    /// Running-header lines (compared against whole trimmed source lines).
    /// When the adapter configures none, the caller derives the uppercased
    /// official title, which is how Diputados prints the running header.
    pub header_lines: Vec<String>,
    /// Blocks that end the main document before the reform-decree
    /// appendix, in addition to the built-in appendix markers. LRITF's
    /// consolidated PDF, for example, ends at "ARTÍCULOS SEGUNDO A DÉCIMO".
    pub stop_markers: Vec<String>,
}

#[derive(Debug)]
pub struct DiputadosDocument {
    pub provisions: Vec<Provision>,
    /// Reform-decree transitories from the consolidated document's
    /// appendix, isolated as temporal evidence.
    pub reform_evidence: Vec<TemporalEvidence>,
}

/// Every transitory ordinal word, masculine and feminine, longest first so
/// prefix matching never truncates a compound (`DÉCIMA PRIMERA` before
/// `DÉCIMA`). Includes accentless variants because older códigos print
/// `DECIMO` without the accent.
fn transitory_ordinals() -> Vec<String> {
    let units_m = [
        "PRIMERO", "SEGUNDO", "TERCERO", "CUARTO", "QUINTO", "SEXTO", "SÉPTIMO", "SEPTIMO",
        "OCTAVO", "NOVENO",
    ];
    let units_f = [
        "PRIMERA", "SEGUNDA", "TERCERA", "CUARTA", "QUINTA", "SEXTA", "SÉPTIMA", "SEPTIMA",
        "OCTAVA", "NOVENA",
    ];
    let tens_m = [
        "DÉCIMO",
        "DECIMO",
        "VIGÉSIMO",
        "VIGESIMO",
        "TRIGÉSIMO",
        "TRIGESIMO",
    ];
    let tens_f = [
        "DÉCIMA",
        "DECIMA",
        "VIGÉSIMA",
        "VIGESIMA",
        "TRIGÉSIMA",
        "TRIGESIMA",
    ];
    let mut ordinals = Vec::new();
    for (tens, units) in [(tens_m, units_m), (tens_f, units_f)] {
        for ten in tens {
            for unit in units {
                ordinals.push(format!("{ten} {unit}"));
                // Joined compounds (`DECIMOPRIMERO`) also occur.
                ordinals.push(format!("{ten}{unit}"));
            }
            ordinals.push((*ten).to_owned());
        }
        ordinals.extend(units.iter().map(|unit| (*unit).to_owned()));
    }
    for extra in [
        "UNDÉCIMO",
        "UNDECIMO",
        "DUODÉCIMO",
        "DUODECIMO",
        "UNDÉCIMA",
        "UNDECIMA",
        "DUODÉCIMA",
        "DUODECIMA",
        "ÚNICO",
        "UNICO",
        "ÚNICA",
        "UNICA",
    ] {
        ordinals.push(extra.to_owned());
    }
    ordinals.sort_by_key(|ordinal| std::cmp::Reverse(ordinal.len()));
    ordinals
}

/// Case-insensitive prefix match returning the exact matched source slice
/// and the remainder, so the ordinal is stored as the document writes it.
fn strip_prefix_ci<'a>(haystack: &'a str, needle: &str) -> Option<(&'a str, &'a str)> {
    let mut end = 0;
    let mut chars = haystack.chars();
    for needle_char in needle.chars() {
        let actual = chars.next()?;
        if !actual.to_lowercase().eq(needle_char.to_lowercase()) {
            return None;
        }
        end += actual.len_utf8();
    }
    Some((&haystack[..end], &haystack[end..]))
}

/// Recognizes a statute transitory heading in both the bare form
/// (`PRIMERA.- body`, `DÉCIMO SEGUNDO. body`, LRITF-style) and the
/// article-prefixed form many códigos use (`Artículo Primero.- body`),
/// matching the ordinal case-insensitively. Returns the ordinal exactly
/// as written and the body.
fn parse_transitory_start<'a>(
    block: &'a str,
    ordinals: &'a [String],
) -> Option<(&'a str, &'a str)> {
    let after_prefix = ["Artículo ", "ARTÍCULO ", "Articulo ", "ARTICULO "]
        .iter()
        .find_map(|prefix| block.strip_prefix(prefix))
        .unwrap_or(block);
    for ordinal in ordinals {
        if let Some((matched, rest)) = strip_prefix_ci(after_prefix, ordinal) {
            for separator in [".-", ".", "-", ":"] {
                if let Some(body) = rest.strip_prefix(separator) {
                    return Some((matched, body.trim()));
                }
            }
        }
    }
    None
}

/// A single-letter article suffix in a heading that the base-number
/// grammar does not fold in itself, either space-separated
/// (`Artículo 2448 A.-`) or written past the separator after a low ordinal
/// (`Artículo 4o.-A.-`). The letter must sit immediately before a body
/// separator (`.`/`-`), so an ordinary body — `4o.- A los efectos…`
/// (space before the letter) or `16. Se entenderá…` (letter starts a
/// word) — is never mistaken for a suffix. Returns the suffix letter and
/// the remainder at the body separator. The dash form (`2448-A`) is
/// handled by the grammar and never reaches here.
fn heading_letter_suffix(after: &str) -> Option<(char, &str)> {
    let body = if let Some(rest) = after.strip_prefix(".-").or_else(|| after.strip_prefix('.')) {
        rest
    } else if after.starts_with(' ') {
        after.trim_start_matches(' ')
    } else {
        return None;
    };
    let mut chars = body.chars();
    let letter = chars.next()?;
    if !letter.is_ascii_uppercase() || !matches!(chars.next(), Some('.' | '-')) {
        return None;
    }
    Some((letter, &body[letter.len_utf8()..]))
}

/// `Artículo 15-D.- body`, `ARTICULO 1o. body`, `Artículo 4o.-A.- body`,
/// `Artículo 2448 A.- body` → `(number, body)`.
fn parse_article_start(block: &str) -> Option<(String, &str)> {
    let rest = ["Artículo ", "ARTÍCULO ", "ARTICULO ", "Articulo "]
        .iter()
        .find_map(|prefix| block.strip_prefix(prefix))?;
    let rest = rest.trim_start();
    let label = labels::match_label_at(rest)?;
    let mut number = label.raw().to_owned();
    let mut after = &rest[label.raw().len()..];
    if let Some((letter, remainder)) = heading_letter_suffix(after) {
        number = format!("{number}-{letter}");
        after = remainder;
    }
    for separator in [".-", ".", "-"] {
        if let Some(body) = after.strip_prefix(separator) {
            return Some((number, body.trim()));
        }
    }
    None
}

/// The leading base number of an article label (`70-A` → 70, `1o` → 1),
/// for detecting a heading whose number regresses below the sequence.
fn article_base(number: &str) -> Option<u64> {
    let digits: String = number.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}

struct HeadingPatterns {
    libro: Regex,
    title: Regex,
    chapter: Regex,
    section: Regex,
    apartado: Regex,
}

impl HeadingPatterns {
    fn new() -> Result<Self> {
        // Roman numerals, ordinal words in either gender (accented or
        // not), ÚNICO/ÚNICA, and PRELIMINAR; the numeral part is captured
        // verbatim and stored as printed.
        let numeral = "(?:[IVXLCDM]+|[A-ZÁÉÍÓÚÑ]+(?:\\s+[A-ZÁÉÍÓÚÑ]+)?)";
        Ok(Self {
            libro: Regex::new(&format!("(?i)^LIBRO\\s+({numeral})$"))?,
            title: Regex::new(&format!("(?i)^T[ÍI]TULO\\s+({numeral})$"))?,
            chapter: Regex::new(&format!("(?i)^CAP[ÍI]TULO\\s+({numeral})$"))?,
            section: Regex::new(&format!("(?i)^SECCI[ÓO]N\\s+({numeral})$"))?,
            apartado: Regex::new(&format!("(?i)^APARTADO\\s+({numeral})$"))?,
        })
    }

    /// When `block` is a structural heading, update `heading` — resetting
    /// every deeper level, exactly as the original parser did for
    /// Título/Capítulo — and report the match.
    fn apply(&self, block: &str, heading: &mut HeadingContext) -> bool {
        if let Some(captures) = self.libro.captures(block) {
            heading.libro = Some(format!("Libro {}", &captures[1]));
            (heading.title, heading.chapter) = (None, None);
            (heading.section, heading.apartado) = (None, None);
        } else if let Some(captures) = self.title.captures(block) {
            heading.title = Some(format!("Título {}", &captures[1]));
            heading.chapter = None;
            (heading.section, heading.apartado) = (None, None);
        } else if let Some(captures) = self.chapter.captures(block) {
            heading.chapter = Some(format!("Capítulo {}", &captures[1]));
            (heading.section, heading.apartado) = (None, None);
        } else if let Some(captures) = self.section.captures(block) {
            heading.section = Some(format!("Sección {}", &captures[1]));
            heading.apartado = None;
        } else if let Some(captures) = self.apartado.captures(block) {
            heading.apartado = Some(format!("Apartado {}", &captures[1]));
        } else {
            return false;
        }
        true
    }
}

fn is_page_furniture(line: &str, options: &DiputadosOptions, page_number: &Regex) -> bool {
    options
        .header_lines
        .iter()
        .any(|header| line == header.as_str())
        || line.starts_with("CÁMARA DE DIPUTADOS DEL H. CONGRESO DE LA UNIÓN")
        || line == "Secretaría General"
        || line == "Secretaría de Servicios Parlamentarios"
        || line.starts_with("Última Reforma DOF ")
        || page_number.is_match(line)
}

fn is_stop_marker(block: &str, options: &DiputadosOptions) -> bool {
    block.contains("ARTÍCULOS TRANSITORIOS DE")
        || block.contains("ARTICULOS TRANSITORIOS DE")
        || options
            .stop_markers
            .iter()
            .any(|marker| block.starts_with(marker.as_str()))
}

fn is_transitory_section_header(block: &str) -> bool {
    // A trailing colon, a missing accent, and mixed case all occur
    // (`ARTICULOS TRANSITORIOS`, `TRANSITORIOS:`, `Transitorios`).
    let block = block
        .strip_suffix(':')
        .unwrap_or(block)
        .trim_end()
        .to_uppercase();
    matches!(
        block.as_str(),
        "DISPOSICIONES TRANSITORIAS"
            | "DISPOSICIONES TRANSITORIALES"
            | "TRANSITORIOS"
            | "TRANSITORIO"
            | "ARTÍCULO TRANSITORIO"
            | "ARTICULO TRANSITORIO"
            | "ARTÍCULOS TRANSITORIOS"
            | "ARTICULOS TRANSITORIOS"
    )
}

/// A decree-wrapper article line using an ordinal word instead of a
/// number: `Artículo Primero.- Se expide la Ley…` (promulgation) or
/// `Artículo Segundo a Artículo Cuarto.- ……` (elided decree articles).
/// Outside the transitory section these belong to the enacting decree,
/// not the instrument, so they are dropped rather than folded into the
/// preceding article. Inside the transitory section the same forms are
/// transitorios and are handled there.
fn is_decree_article_wrapper(block: &str, ordinals: &[String]) -> bool {
    let Some(rest) = [
        "Artículo ",
        "Artículos ",
        "ARTÍCULO ",
        "ARTÍCULOS ",
        "ARTICULO ",
        "ARTICULOS ",
    ]
    .iter()
    .find_map(|prefix| block.strip_prefix(prefix)) else {
        return false;
    };
    ordinals.iter().any(|ordinal| {
        strip_prefix_ci(rest, ordinal)
            .is_some_and(|(_, tail)| tail.chars().next().is_none_or(|c| !c.is_alphanumeric()))
    })
}

fn is_immediate_structural(line: &str, options: &DiputadosOptions) -> bool {
    let upper = line.to_uppercase();
    is_stop_marker(line, options)
        || is_decree_heading(line)
        || upper.starts_with("LIBRO ")
        || upper.starts_with("TÍTULO ")
        || upper.starts_with("TITULO ")
        || upper.starts_with("CAPÍTULO ")
        || upper.starts_with("CAPITULO ")
        || upper.starts_with("SECCIÓN ")
        || upper.starts_with("SECCION ")
        || upper.starts_with("APARTADO ")
        || is_transitory_section_header(line)
}

/// A decree title, as opposed to a wrapped sentence whose next source line
/// happens to begin with `Decreto de ...`. Diputados prints appendix decree
/// titles in uppercase; the two title-case forms cover older consolidations
/// without treating an embedded decree citation as a structural boundary.
fn is_decree_heading(block: &str) -> bool {
    block.starts_with("DECRETO ")
        || block.starts_with("Decreto por el que ")
        || block.starts_with("Decreto que ")
}

/// Line-level flush trigger, deliberately looser than the block-level
/// parse (a wrapped citation line starting `Artículo 22 de la Ley` still
/// opens a new block, exactly as the original LRITF parser behaved).
fn is_provision_start(line: &str, ordinals: &[String]) -> bool {
    line.starts_with("Artículo ")
        || line.starts_with("ARTÍCULO ")
        || line.starts_with("ARTICULO ")
        || line.starts_with("Articulo ")
        || parse_transitory_start(line, ordinals).is_some()
}

/// A line containing only an article heading, without the punctuation that
/// normally separates the identifier from its body (`Artículo 1`). Some
/// secondary regulations print the numbered paragraphs on the following
/// line, so blindly collapsing the lines would turn `Artículo 1` + `1. ...`
/// into the false compound identifier `Artículo 1 1`.
fn is_bare_article_heading(line: &str) -> bool {
    let Some(rest) = ["Artículo ", "ARTÍCULO ", "ARTICULO ", "Articulo "]
        .iter()
        .find_map(|prefix| line.strip_prefix(prefix))
    else {
        return false;
    };
    let rest = rest.trim();
    labels::match_label_at(rest).is_some_and(|label| label.raw().len() == rest.len())
}

fn is_numbered_paragraph_start(line: &str) -> bool {
    let digits = line.bytes().take_while(u8::is_ascii_digit).count();
    digits > 0
        && line.as_bytes().get(digits) == Some(&b'.')
        && line
            .as_bytes()
            .get(digits + 1)
            .is_some_and(u8::is_ascii_whitespace)
}

/// Merge raw layout lines into logical blocks: a blank line separates
/// blocks unless it coincides with page furniture (a paragraph continuing
/// across a page boundary), and structural headings and provision starts
/// always open a block of their own.
fn normalized_blocks(raw: &str, options: &DiputadosOptions, ordinals: &[String]) -> Vec<String> {
    let page_number = Regex::new(r"^\d+\s+de\s+\d+$").expect("static regex");
    let amendment_mark_end =
        Regex::new(r"(?i)\bDOF\s+\d{2}-\d{2}-\d{4}[.,;:]?$").expect("static regex");
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
        if is_page_furniture(line, options, &page_number) {
            if !current.is_empty() {
                crossed_page_furniture = true;
            }
            pending_blank = false;
            continue;
        }
        if crossed_page_furniture && amendment_mark_end.is_match(&current) {
            flush_block(&mut current, &mut blocks);
            crossed_page_furniture = false;
        }
        if pending_blank && !crossed_page_furniture {
            flush_block(&mut current, &mut blocks);
        }
        pending_blank = false;
        crossed_page_furniture = false;
        if is_immediate_structural(line, options) {
            flush_block(&mut current, &mut blocks);
            blocks.push(collapse_whitespace(line));
            continue;
        }
        if is_provision_start(line, ordinals) {
            flush_block(&mut current, &mut blocks);
            current.push_str(line);
            continue;
        }
        if is_bare_article_heading(&current) && is_numbered_paragraph_start(line) {
            // Preserve the legal paragraph numeral as body text while
            // supplying the separator omitted by the PDF layout.
            current.push_str(". ");
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

fn flush_block(current: &mut String, blocks: &mut Vec<String>) {
    if !current.is_empty() {
        blocks.push(collapse_whitespace(current));
        current.clear();
    }
}

struct ProvisionBuilder {
    provision_type: ProvisionType,
    number: String,
    label: String,
    heading: HeadingContext,
    blocks: Vec<String>,
}

impl ProvisionBuilder {
    fn article(number: String, first_block: &str, heading: HeadingContext) -> Self {
        let label = format!("Artículo {number}");
        let mut builder = Self {
            provision_type: ProvisionType::Article,
            number,
            label,
            heading,
            blocks: Vec::new(),
        };
        builder.push_block(first_block);
        builder
    }

    fn transitory(ordinal: &str, first_block: &str) -> Self {
        let mut builder = Self {
            provision_type: ProvisionType::Transitory,
            number: ordinal.to_owned(),
            label: ordinal.to_owned(),
            heading: HeadingContext {
                libro: None,
                title: None,
                chapter: None,
                section: None,
                apartado: None,
            },
            blocks: Vec::new(),
        };
        builder.push_block(first_block);
        builder
    }

    fn push_block(&mut self, value: &str) {
        let value = value.trim();
        if !value.is_empty() {
            self.blocks.push(value.to_owned());
        }
    }

    fn finish(self, instrument_id: &str, publication_date: NaiveDate) -> Provision {
        let (kind, canonical_number) = match self.provision_type {
            // Ordinal marks carry no identity: article "2o" is canonical
            // id ":article:2", so a citation of bare "2" resolves to it.
            ProvisionType::Article => ("article", labels::canonical_slug(&self.number)),
            ProvisionType::Transitory => ("transitory", slug(&self.number)),
            ProvisionType::Annex => ("annex", slug(&self.number)),
        };
        let text = self.blocks.join("\n\n");
        Provision {
            schema_version: SCHEMA_VERSION.to_owned(),
            id: format!("{instrument_id}:{kind}:{canonical_number}"),
            instrument_id: instrument_id.to_owned(),
            provision_type: self.provision_type,
            label: self.label,
            number: self.number,
            heading_context: self.heading,
            text: text.clone(),
            publication_date,
            effective_from: None,
            effective_to: None,
            temporal_status: crate::initial_temporal_status(&text),
            temporal_basis: None,
            temporal_confidence: None,
            review_status: ReviewStatus::NotAnalyzed,
            transitory_effects: Vec::new(),
            amendment_marks: Vec::new(),
        }
    }
}

pub fn parse_diputados(
    raw: &str,
    options: &DiputadosOptions,
    publication_date: NaiveDate,
) -> Result<DiputadosDocument> {
    let patterns = HeadingPatterns::new()?;
    let ordinals = transitory_ordinals();
    let blocks = normalized_blocks(raw, options, &ordinals);

    let mut provisions = Vec::new();
    let mut current: Option<ProvisionBuilder> = None;
    let mut heading = HeadingContext {
        libro: None,
        title: None,
        chapter: None,
        section: None,
        apartado: None,
    };
    let mut in_statute_transitories = false;
    let mut last_article_base: Option<u64> = None;
    let mut seen_ordinals: std::collections::HashSet<String> = std::collections::HashSet::new();

    for block in blocks {
        if is_stop_marker(&block, options) {
            break;
        }
        if is_transitory_section_header(&block) {
            // A statute has a single transitory section; a second header
            // is a reform decree's, so its transitorios (repeating
            // PRIMERO, SEGUNDO, …) do not belong to the instrument.
            if in_statute_transitories {
                break;
            }
            if let Some(builder) = current.take() {
                provisions.push(builder.finish(&options.instrument_id, publication_date));
            }
            in_statute_transitories = true;
            heading = HeadingContext {
                libro: None,
                title: None,
                chapter: None,
                section: None,
                apartado: None,
            };
            continue;
        }

        if in_statute_transitories {
            // A reform decree following the statute transitories ends the
            // main document; its transitories belong to reform evidence.
            if block.starts_with("DECRETO por") || block.starts_with("REFORMAS Y ADICIONES") {
                break;
            }
            if let Some((ordinal, body)) = parse_transitory_start(&block, &ordinals) {
                // A repeated ordinal starts a reform decree's transitorios
                // (unmarked by a header); the instrument's own transitorios
                // are unique, so stop rather than duplicate them.
                if !seen_ordinals.insert(ordinal.to_uppercase()) {
                    break;
                }
                if let Some(builder) = current.take() {
                    provisions.push(builder.finish(&options.instrument_id, publication_date));
                }
                current = Some(ProvisionBuilder::transitory(ordinal, body));
                continue;
            }
            // Some códigos number their enactment transitorios as articles
            // ("Artículo 1o.- Este Código comenzará a regir…"); inside the
            // transitory section these are transitorios, not a restart of
            // the article sequence.
            if let Some((number, body)) = parse_article_start(&block) {
                if let Some(builder) = current.take() {
                    provisions.push(builder.finish(&options.instrument_id, publication_date));
                }
                current = Some(ProvisionBuilder::transitory(&number, body));
                continue;
            }
        } else {
            if is_decree_article_wrapper(&block, &ordinals) {
                if let Some(builder) = current.take() {
                    provisions.push(builder.finish(&options.instrument_id, publication_date));
                }
                continue;
            }
            if patterns.apply(&block, &mut heading) {
                if let Some(builder) = current.take() {
                    provisions.push(builder.finish(&options.instrument_id, publication_date));
                }
                continue;
            }
            if let Some((number, body)) = parse_article_start(&block) {
                let base = article_base(&number);
                // A consolidated código quotes other laws' articles inside
                // editorial notes ("…el artículo 54 de la citada Ley, a la
                // letra señalaba: Artículo 54.- …"). A heading whose base
                // number falls below the sequence position already reached
                // is such a quote, not a new article; keep it in the
                // current provision's body instead of restarting the count.
                let regresses =
                    matches!((base, last_article_base), (Some(b), Some(last)) if b < last);
                if !regresses {
                    if let Some(builder) = current.take() {
                        provisions.push(builder.finish(&options.instrument_id, publication_date));
                    }
                    last_article_base = base.or(last_article_base);
                    current = Some(ProvisionBuilder::article(number, body, heading.clone()));
                    continue;
                }
            }
        }

        if let Some(builder) = &mut current {
            builder.push_block(&block);
        }
    }

    if let Some(builder) = current {
        provisions.push(builder.finish(&options.instrument_id, publication_date));
    }
    if provisions.is_empty() {
        bail!("no provisions recognized for {}", options.instrument_id);
    }
    let reform_evidence = extract_reform_evidence(raw, options)?;
    Ok(DiputadosDocument {
        provisions,
        reform_evidence,
    })
}

/// The original DOF publication date from the consolidated document's own
/// header note ("Nueva Ley publicada en el Diario Oficial de la Federación
/// el 16 de abril de 2025"). Deterministic: first match wins, and the
/// header note always precedes any reform-decree publication line.
#[must_use]
pub fn extract_dof_publication(raw: &str) -> Option<NaiveDate> {
    let note = Regex::new(
        r"(?i)publicad[oa] en el Diario Oficial de la Federación el (\d{1,2})[oº]? de ([a-zá-úñ]+) de (\d{4})",
    )
    .ok()?;
    let captures = note.captures(raw)?;
    spanish_date(&captures[1], &captures[2].to_lowercase(), &captures[3])
}

struct ReformEvidenceBuilder {
    date: NaiveDate,
    decree_occurrence: usize,
    transitory_section_occurrence: usize,
    ordinal: String,
    blocks: Vec<String>,
}

fn flush_reform(
    instrument_id: &str,
    current: &mut Option<ReformEvidenceBuilder>,
    evidence: &mut Vec<TemporalEvidence>,
) {
    if let Some(builder) = current.take() {
        let date = builder.date;
        let decree_occurrence = builder.decree_occurrence;
        let transitory_section_occurrence = builder.transitory_section_occurrence;
        let ordinal = builder.ordinal;
        let text = builder
            .blocks
            .into_iter()
            .filter(|block| !block.is_empty())
            .collect::<Vec<_>>()
            .join("\n\n");
        let mut item = reform_evidence_item(
            instrument_id,
            date,
            &ordinal,
            "Decreto",
            text,
            // Diputados reform decretos carry no CNBV amendment markers.
            Vec::new(),
        );
        // A consolidated appendix can contain several decrees published on
        // the same date, or several transitory sections inside one decree.
        // Preserve the established ID for the first decree/section and
        // qualify later occurrences so every evidence identity is unique.
        if decree_occurrence > 1 || transitory_section_occurrence > 1 {
            let date = date.format("%Y-%m-%d");
            let base = format!(":amendment:{date}:");
            let mut qualifiers = Vec::new();
            if decree_occurrence > 1 {
                qualifiers.push(format!("decree-{decree_occurrence}"));
            }
            if transitory_section_occurrence > 1 {
                qualifiers.push(format!("section-{transitory_section_occurrence}"));
            }
            let qualified = format!(":amendment:{date}:{}:", qualifiers.join(":"));
            item.provision_id = item.provision_id.replacen(&base, &qualified, 1);
            item.label = match (decree_occurrence, transitory_section_occurrence) {
                (1, section) => format!(
                    "Transitorio {ordinal} — Sección transitoria {section}, Decreto DOF {date}"
                ),
                (decree, 1) => {
                    format!("Transitorio {ordinal} — Decreto {decree} DOF {date}")
                }
                (decree, section) => format!(
                    "Transitorio {ordinal} — Decreto {decree}, sección transitoria {section} DOF {date}"
                ),
            };
        }
        evidence.push(item);
    }
}

fn is_reform_transitory_section_header(block: &str) -> bool {
    if is_transitory_section_header(block) {
        return true;
    }
    let uppercase = block.to_uppercase();
    uppercase.starts_with("ARTÍCULOS TRANSITORIOS DEL DECRETO")
        || uppercase.starts_with("ARTICULOS TRANSITORIOS DEL DECRETO")
}

/// Isolate the consolidated document's reform-decree appendix
/// (`ARTÍCULOS TRANSITORIOS DE DECRETOS DE REFORMA`) into per-decree
/// transitory evidence for temporal analysis.
pub fn extract_reform_evidence(
    raw: &str,
    options: &DiputadosOptions,
) -> Result<Vec<TemporalEvidence>> {
    let ordinals = transitory_ordinals();
    let publication_re = Regex::new(
        r"(?i)publicad[oa] en el Diario Oficial de la Federación el (\d{1,2}) de ([a-zá-úñ]+) de (\d{4})",
    )?;
    let mut in_reform_appendix = false;
    let mut in_transitories = false;
    let mut publication_date: Option<NaiveDate> = None;
    let mut decree_occurrence: Option<usize> = None;
    let mut transitory_section_occurrence = 0;
    let mut decrees_by_date: std::collections::HashMap<NaiveDate, usize> =
        std::collections::HashMap::new();
    let mut current: Option<ReformEvidenceBuilder> = None;
    let mut evidence = Vec::new();

    for block in normalized_blocks(raw, options, &ordinals) {
        if !in_reform_appendix
            && block.contains("ARTÍCULOS TRANSITORIOS DE")
            && block.contains("DECRETO")
        {
            in_reform_appendix = true;
            continue;
        }
        if !in_reform_appendix {
            continue;
        }
        let uppercase = block.to_uppercase();
        if is_decree_heading(&block) {
            flush_reform(&options.instrument_id, &mut current, &mut evidence);
            in_transitories = false;
            publication_date = None;
            decree_occurrence = None;
            transitory_section_occurrence = 0;
        }
        // Wrapped decree titles can share a block with their publication
        // note. Once the transitory section has begun, however, the same
        // wording is a citation inside legal text and must not replace the
        // containing decree's date or consume that transitory block.
        if !in_transitories && let Some(captures) = publication_re.captures(&block) {
            publication_date = spanish_date(&captures[1], &captures[2], &captures[3]);
            if let Some(date) = publication_date {
                let occurrence = decrees_by_date.entry(date).or_default();
                *occurrence += 1;
                decree_occurrence = Some(*occurrence);
            }
            continue;
        }
        if is_reform_transitory_section_header(&block) {
            if in_transitories {
                flush_reform(&options.instrument_id, &mut current, &mut evidence);
            }
            in_transitories = true;
            transitory_section_occurrence += 1;
            continue;
        }
        if !in_transitories {
            continue;
        }
        if uppercase.starts_with("CIUDAD DE MÉXICO")
            || uppercase.starts_with("MÉXICO, D")
            || uppercase.starts_with("SALÓN DE SESIONES")
            || uppercase.starts_with("SALON DE SESIONES")
            || uppercase.starts_with("FE DE ERRATAS")
        {
            flush_reform(&options.instrument_id, &mut current, &mut evidence);
            in_transitories = false;
            continue;
        }
        let transitory_start = parse_transitory_start(&block, &ordinals)
            .map(|(ordinal, body)| (ordinal.to_owned(), body))
            // Older decrees also number their transitories as articles
            // (`ARTICULO 1o.-`, `ARTICULO 2o.-`). Once the appendix has
            // entered a transitory section, these are evidence headings,
            // not operative decree articles.
            .or_else(|| parse_article_start(&block));
        if let Some((ordinal, body)) = transitory_start {
            flush_reform(&options.instrument_id, &mut current, &mut evidence);
            let date = publication_date.ok_or_else(|| {
                anyhow::anyhow!(
                    "found reform transitory {ordinal:?} without its Diario Oficial publication date"
                )
            })?;
            current = Some(ReformEvidenceBuilder {
                date,
                decree_occurrence: decree_occurrence.ok_or_else(|| {
                    anyhow::anyhow!(
                        "found reform transitory without its containing decree identity"
                    )
                })?,
                transitory_section_occurrence,
                ordinal,
                blocks: vec![body.to_owned()],
            });
        } else if let Some(builder) = &mut current {
            builder.blocks.push(block);
        }
    }
    flush_reform(&options.instrument_id, &mut current, &mut evidence);
    let mut evidence_ids = std::collections::HashSet::new();
    for item in &evidence {
        if !evidence_ids.insert(item.provision_id.as_str()) {
            bail!("duplicate reform evidence id {}", item.provision_id);
        }
    }
    Ok(evidence)
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use lex_core::ProvisionType;

    use super::{DiputadosOptions, parse_diputados};

    const CODIGO_FIXTURE: &str = include_str!("../../../fixtures/diputados/codigo-sample.txt");
    const NUMBERED_PARAGRAPH_FIXTURE: &str =
        include_str!("../../../fixtures/diputados/separate-heading-numbered-paragraph-sample.txt");
    const TITLE_CASE_HEADING_FIXTURE: &str =
        include_str!("../../../fixtures/diputados/title-case-heading-sample.txt");
    const PAGE_BOUNDARY_AMENDMENT_FIXTURE: &str =
        include_str!("../../../fixtures/diputados/page-boundary-amendment-mark-sample.txt");
    const ORIGINAL_TRANSITORY_SIGNATURE_FIXTURE: &str = include_str!(
        "../../../fixtures/diputados/original-transitory-signature-boundary-sample.txt"
    );
    const REFORM_VARIANTS_FIXTURE: &str =
        include_str!("../../../fixtures/diputados/reform-appendix-variants-sample.txt");
    const MULTIPLE_TRANSITORY_SECTIONS_FIXTURE: &str =
        include_str!("../../../fixtures/diputados/reform-multiple-transitory-sections-sample.txt");

    fn options(instrument_id: &str, title: &str) -> DiputadosOptions {
        DiputadosOptions {
            instrument_id: instrument_id.to_owned(),
            header_lines: vec![title.to_uppercase()],
            stop_markers: Vec::new(),
        }
    }

    #[test]
    fn parses_codigo_shapes() {
        let document = parse_diputados(
            CODIGO_FIXTURE,
            &options("urn:lex-mx:federal:code:sample", "Código de Muestra"),
            NaiveDate::from_ymd_opt(1981, 12, 31).expect("valid date"),
        )
        .expect("codigo fixture parses");
        let articles: Vec<&str> = document
            .provisions
            .iter()
            .filter(|provision| provision.provision_type == ProvisionType::Article)
            .map(|provision| provision.number.as_str())
            .collect();
        // Letter suffixes, ordinal marks, ARTICULO capitalization,
        // period-only separators, and the `2o.-A` past-separator letter
        // suffix all survive as written.
        assert_eq!(
            articles,
            ["1o", "2o", "2o-A", "15", "15-A", "15-B Bis", "16"]
        );
        let letter_suffix = document
            .provisions
            .iter()
            .find(|provision| provision.number == "15-A")
            .expect("15-A present");
        assert_eq!(
            letter_suffix.id,
            "urn:lex-mx:federal:code:sample:article:15-a"
        );
        // Ordinal marks are dropped from the canonical id so a citation of
        // bare "2" resolves to article "2o"; the `2o.-A` heading yields a
        // distinct "2-a".
        let ordinal = document
            .provisions
            .iter()
            .find(|provision| provision.number == "2o")
            .expect("2o present");
        assert_eq!(ordinal.id, "urn:lex-mx:federal:code:sample:article:2");
        let ordinal_letter = document
            .provisions
            .iter()
            .find(|provision| provision.number == "2o-A")
            .expect("2o-A present");
        assert_eq!(
            ordinal_letter.id,
            "urn:lex-mx:federal:code:sample:article:2-a"
        );
        assert_eq!(
            letter_suffix.heading_context.libro.as_deref(),
            Some("Libro PRIMERO")
        );
        assert_eq!(
            letter_suffix.heading_context.chapter.as_deref(),
            Some("Capítulo II")
        );
        let transitories: Vec<&str> = document
            .provisions
            .iter()
            .filter(|provision| provision.provision_type == ProvisionType::Transitory)
            .map(|provision| provision.number.as_str())
            .collect();
        // The narrative TRANSITORIOS mention inside article 16 must not
        // open the transitory section; the DECRETO cut keeps the reform
        // decree's own transitories out of the instrument. Both the
        // "Artículo Primero" and bare "DÉCIMO SEGUNDO" heading forms are
        // recognized and stored as written.
        assert_eq!(transitories, ["Primero", "DÉCIMO SEGUNDO"]);
        assert_eq!(document.reform_evidence.len(), 1);
        assert_eq!(
            document.reform_evidence[0].provision_id,
            "urn:lex-mx:federal:code:sample:amendment:2020-07-01:transitory:vigesimo"
        );
    }

    #[test]
    fn separate_heading_does_not_absorb_first_numbered_paragraph() {
        let document = parse_diputados(
            NUMBERED_PARAGRAPH_FIXTURE,
            &options(
                "urn:lex-mx:federal:regulation:sample",
                "Reglamento de Muestra",
            ),
            NaiveDate::from_ymd_opt(2010, 6, 4).expect("valid date"),
        )
        .expect("separate-heading fixture parses");

        assert_eq!(document.provisions.len(), 2);
        assert_eq!(document.provisions[0].number, "1");
        assert_eq!(document.provisions[0].label, "Artículo 1");
        assert_eq!(
            document.provisions[0].id,
            "urn:lex-mx:federal:regulation:sample:article:1"
        );
        assert!(
            document.provisions[0]
                .text
                .starts_with("1. Este Reglamento")
        );
        assert_eq!(document.provisions[1].number, "15 Bis 1");
    }

    #[test]
    fn title_case_structural_heading_applies_to_the_following_article() {
        let document = parse_diputados(
            TITLE_CASE_HEADING_FIXTURE,
            &options(
                "urn:lex-mx:federal:statute:sample",
                "Ley Reglamentaria de Muestra",
            ),
            NaiveDate::from_ymd_opt(1995, 5, 11).expect("valid date"),
        )
        .expect("title-case heading fixture parses");

        assert_eq!(document.provisions.len(), 2);
        assert_eq!(document.provisions[0].text, "Texto del artículo anterior.");
        assert_eq!(
            document.provisions[1].heading_context.chapter.as_deref(),
            Some("Capítulo III")
        );
        assert_eq!(document.provisions[1].text, "Texto bajo el capítulo.");
    }

    #[test]
    fn amendment_mark_closes_paragraph_across_wrapped_page_header() {
        let options = DiputadosOptions {
            instrument_id: "urn:lex-mx:federal:statute:sample".to_owned(),
            header_lines: vec![
                "LEY REGLAMENTARIA DE MUESTRA DE LOS".to_owned(),
                "ESTADOS UNIDOS MEXICANOS".to_owned(),
            ],
            stop_markers: Vec::new(),
        };
        let document = parse_diputados(
            PAGE_BOUNDARY_AMENDMENT_FIXTURE,
            &options,
            NaiveDate::from_ymd_opt(1995, 5, 11).expect("valid date"),
        )
        .expect("page-boundary fixture parses");

        assert_eq!(document.provisions.len(), 2);
        assert_eq!(
            document.provisions[0].text,
            "Primer párrafo. Párrafo reformado DOF 03-04-2025\n\nSegundo párrafo."
        );
        assert!(!document.provisions[0].text.contains("LEY REGLAMENTARIA"));
    }

    #[test]
    fn configured_stop_marker_excludes_enactment_signatures_from_transitory() {
        let mut options = options(
            "urn:lex-mx:federal:statute:sample",
            "Ley del Diario Oficial de la Federación y Gacetas Gubernamentales",
        );
        options.stop_markers = vec!["México, D. F., a 9 de diciembre de 1986".to_owned()];

        let document = parse_diputados(
            ORIGINAL_TRANSITORY_SIGNATURE_FIXTURE,
            &options,
            NaiveDate::from_ymd_opt(1986, 12, 24).expect("valid date"),
        )
        .expect("signature-boundary fixture parses");

        let transitories: Vec<_> = document
            .provisions
            .iter()
            .filter(|provision| provision.provision_type == ProvisionType::Transitory)
            .collect();
        assert_eq!(transitories.len(), 2);
        assert_eq!(
            transitories[1].text,
            "Se derogan las disposiciones que se opongan a la presente ley."
        );
        assert!(!transitories[1].text.contains("Rúbrica"));
        assert_eq!(document.reform_evidence.len(), 1);
    }

    #[test]
    fn reform_appendix_keeps_decrees_and_transitories_separate() {
        let evidence = super::extract_reform_evidence(
            REFORM_VARIANTS_FIXTURE,
            &options(
                "urn:lex-mx:federal:regulation:sample",
                "Reglamento de Muestra",
            ),
        )
        .expect("reform variants parse");

        let ids: Vec<&str> = evidence
            .iter()
            .map(|item| item.provision_id.as_str())
            .collect();
        assert_eq!(
            ids,
            [
                "urn:lex-mx:federal:regulation:sample:amendment:2010-12-20:transitory:primero",
                "urn:lex-mx:federal:regulation:sample:amendment:2010-12-20:transitory:segundo",
                "urn:lex-mx:federal:regulation:sample:amendment:2018-05-23:transitory:unico",
                "urn:lex-mx:federal:regulation:sample:amendment:2022-11-11:transitory:primero",
                "urn:lex-mx:federal:regulation:sample:amendment:2022-11-11:transitory:segundo",
                "urn:lex-mx:federal:regulation:sample:amendment:2023-09-22:transitory:unico",
                "urn:lex-mx:federal:regulation:sample:amendment:2023-09-22:decree-2:transitory:unico",
                "urn:lex-mx:federal:regulation:sample:amendment:2024-01-02:transitory:1o",
                "urn:lex-mx:federal:regulation:sample:amendment:2024-01-02:transitory:2o",
            ]
        );
        assert!(
            evidence
                .iter()
                .all(|item| !item.text.contains("SALÓN DE SESIONES"))
        );
        assert!(
            evidence[4]
                .text
                .contains("previsiones presupuestales necesarias")
        );
        assert!(
            evidence[1]
                .text
                .contains("Decreto de fecha 3 de enero de 2008")
        );
        assert!(evidence[1].provision_id.contains("2010-12-20"));
        assert!(!evidence[4].text.contains("reforma el artículo 10"));
        assert!(
            evidence
                .iter()
                .all(|item| !item.text.contains("Se reforma el artículo operativo"))
        );
        assert!(
            evidence
                .iter()
                .all(|item| !item.text.contains("Fe de erratas"))
        );
        assert_eq!(
            evidence[6].label,
            "Transitorio Único — Decreto 2 DOF 2023-09-22"
        );
    }

    #[test]
    fn reform_decree_transitory_sections_receive_distinct_evidence_ids() {
        let evidence = super::extract_reform_evidence(
            MULTIPLE_TRANSITORY_SECTIONS_FIXTURE,
            &options(
                "urn:lex-mx:federal:statute:sample",
                "Ley Reglamentaria de Muestra",
            ),
        )
        .expect("multiple transitory sections parse");

        let ids: Vec<&str> = evidence
            .iter()
            .map(|item| item.provision_id.as_str())
            .collect();
        assert_eq!(
            ids,
            [
                "urn:lex-mx:federal:statute:sample:amendment:1996-11-22:transitory:primero",
                "urn:lex-mx:federal:statute:sample:amendment:1996-11-22:transitory:segundo",
                "urn:lex-mx:federal:statute:sample:amendment:1996-11-22:section-2:transitory:primero",
                "urn:lex-mx:federal:statute:sample:amendment:1996-11-22:section-2:transitory:segundo",
            ]
        );
        assert_eq!(
            evidence[2].label,
            "Transitorio PRIMERO — Sección transitoria 2, Decreto DOF 1996-11-22"
        );
    }
}
