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
    /// Headings that begin a substantive appendix belonging to the
    /// instrument. Everything after the first matching heading is preserved
    /// as one canonical annex provision.
    pub annex_markers: Vec<String>,
    pub separate_enactment: bool,
    pub nested_instrument: Option<lex_core::NestedInstrumentProfile>,
    /// Article labels whose heading the source prints without a separator
    /// after the number (`Artículo 71 El médico tratante…`, `reg-pmimss`).
    /// Reviewed per instrument: each must open exactly one main-sequence
    /// block, followed by a capitalized body, or the parse fails.
    pub separatorless_article_headings: Vec<String>,
}

#[derive(Debug)]
pub struct DiputadosDocument {
    pub provisions: Vec<Provision>,
    /// Reform-decree transitories from the consolidated document's
    /// appendix, isolated as temporal evidence.
    pub reform_evidence: Vec<TemporalEvidence>,
    pub enactment_evidence: Vec<lex_core::EnactmentEvidence>,
}

/// Every transitory ordinal word, masculine and feminine, longest first so
/// prefix matching never truncates a compound (`DÉCIMA PRIMERA` before
/// `DÉCIMA`). Includes accentless variants because older códigos print
/// `DECIMO` without the accent.
pub(crate) fn transitory_ordinals() -> Vec<String> {
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
        "CUADRAGÉSIMO",
        "CUADRAGESIMO",
        "QUINCUAGÉSIMO",
        "QUINCUAGESIMO",
    ];
    let tens_f = [
        "DÉCIMA",
        "DECIMA",
        "VIGÉSIMA",
        "VIGESIMA",
        "TRIGÉSIMA",
        "TRIGESIMA",
        "CUADRAGÉSIMA",
        "CUADRAGESIMA",
        "QUINCUAGÉSIMA",
        "QUINCUAGESIMA",
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
pub(crate) fn parse_transitory_start<'a>(
    block: &'a str,
    ordinals: &'a [String],
) -> Option<(&'a str, &'a str)> {
    let after_prefix = ["Artículo ", "ARTÍCULO ", "Articulo ", "ARTICULO "]
        .iter()
        .find_map(|prefix| block.strip_prefix(prefix))
        .unwrap_or(block);
    for ordinal in ordinals {
        if let Some((matched, rest)) = strip_prefix_ci(after_prefix, ordinal) {
            // A heading capitalizes its ordinal; `décimo. Asimismo, …` is the
            // tail of a sentence carried over a page break (cpf, DOF
            // 16-06-2016, transitory Décimo Primero).
            if !matched.starts_with(char::is_uppercase) {
                return None;
            }
            for separator in [".-", ".", "-", ":"] {
                if let Some(body) = rest.strip_prefix(separator) {
                    return Some((matched, body.trim()));
                }
            }
        }
    }
    None
}

/// Bare numeric-ordinal transitory heading used by some original statutes
/// (`1o.- La presente Ley entrará en vigor…`, LFJS). Only the strict `.-`
/// separator is accepted so a body line that merely begins with a numbered
/// ordinal (`1o. de enero…`) is never read as a heading. Callers must apply
/// this only inside an established transitory section.
pub(crate) fn parse_numeric_transitory_start(block: &str) -> Option<(&str, &str)> {
    let digits = block.bytes().take_while(u8::is_ascii_digit).count();
    if digits == 0 || digits > 2 {
        return None;
    }
    let rest = &block[digits..];
    let marker = rest
        .chars()
        .next()
        .filter(|c| matches!(c, 'o' | 'º' | '°'))?;
    let end = digits + marker.len_utf8();
    let body = block[end..].strip_prefix(".-")?;
    Some((&block[..end], body.trim()))
}

/// Roman labels are accepted only by the established original-transitory
/// section mode, never by the article or reform-evidence grammar.
fn parse_roman_transitory_start(block: &str) -> Option<(&str, &str)> {
    let (ordinal, body) = block.split_once(".-")?;
    if ordinal.is_empty()
        || !ordinal
            .chars()
            .all(|c| matches!(c, 'I' | 'V' | 'X' | 'L' | 'C' | 'D' | 'M'))
    {
        return None;
    }
    Some((ordinal, body.trim()))
}

/// A single-letter article suffix in a heading that the base-number
/// grammar does not fold in itself, either space-separated
/// (`Artículo 2448 A.-`) or written past the separator after a low ordinal
/// (`Artículo 4o.-A.-`). The letter must sit immediately before a body
/// separator (`.`/`-`), so an ordinary body — `4o.- A los efectos…`
/// (space before the letter) or `16. Se entenderá…` (letter starts a
/// word) — is never mistaken for a suffix. Returns the suffix (the letter,
/// plus a qualifier when one follows it as in `1o.-A BIS.-`) and the
/// remainder at the body separator. The dash form (`2448-A`) is handled by
/// the grammar and never reaches here.
fn heading_letter_suffix(after: &str) -> Option<(String, &str)> {
    let body = if let Some(rest) = after.strip_prefix(".-").or_else(|| after.strip_prefix('.')) {
        rest
    } else if after.starts_with(' ') {
        after.trim_start_matches(' ')
    } else {
        return None;
    };
    // Uppercase only, so a lowercase word opening a body is never a suffix.
    // A digraph (`CH`, `LL`, `RR`) is one letter here, as in `26-LL`.
    let first = body.chars().next()?;
    if !first.is_uppercase() {
        return after
            .starts_with(' ')
            .then(|| lowercase_letter_suffix(body))
            .flatten();
    }
    let letter = labels::match_suffix_token_at(body)?;
    let rest = &body[letter.len()..];
    match rest.chars().next() {
        // `Artículo 4o.-A.-`: the suffix letter closes the identifier.
        Some('.' | '-') => Some((letter.to_owned(), rest)),
        // `Artículo 1o.-A BIS.-` (LIVA): a qualifier follows the suffix
        // letter, so the identifier is `1o-A BIS`. Without this the letter
        // is rejected, the heading collapses to `1o`, and `A BIS.-` is left
        // stranded at the head of the body — a duplicate `article:1`.
        Some(' ') => {
            let tail = rest.trim_start_matches(' ');
            let qualifier = labels::match_qualifier_word_at(tail)?;
            let remainder = &tail[qualifier.len()..];
            if !matches!(remainder.chars().next(), Some('.' | '-')) {
                return None;
            }
            Some((format!("{letter} {qualifier}"), remainder))
        }
        _ => None,
    }
}

/// A quoted block that opens with an article heading word (`Artículo
/// Segundo.`), numbered or ordinal.
fn is_article_shaped_word(block: &str) -> bool {
    ["Artículo ", "ARTÍCULO ", "ARTICULO ", "Articulo "]
        .iter()
        .any(|prefix| block.starts_with(prefix))
}

/// A block that opens like an article heading (`Artículo 228 a.- …`),
/// whether or not the grammar reads it.
fn is_article_shaped(block: &str) -> bool {
    ["Artículo ", "ARTÍCULO ", "ARTICULO ", "Articulo "]
        .iter()
        .find_map(|prefix| block.strip_prefix(prefix))
        .is_some_and(|rest| rest.starts_with(|c: char| c.is_ascii_digit()))
}

/// LGTOC prints `Artículo 228 a.-` and `Artículo 228 a bis.-`: a lower-case
/// suffix letter after a space, accepted only when the heading separator
/// `.-` closes it at once, so a body word is never read as a suffix.
fn lowercase_letter_suffix(body: &str) -> Option<(String, &str)> {
    let letter = body.chars().next().filter(char::is_ascii_lowercase)?;
    let rest = &body[1..];
    if rest.starts_with(".-") {
        return Some((letter.to_string(), rest));
    }
    let tail = rest.strip_prefix(' ')?;
    let qualifier = labels::match_qualifier_word_at(tail)?;
    let remainder = &tail[qualifier.len()..];
    remainder
        .starts_with(".-")
        .then(|| (format!("{letter} {qualifier}"), remainder))
}

/// `Artículo 15-D.- body`, `ARTICULO 1o. body`, `Artículo 4o.-A.- body`,
/// `Artículo 2448 A.- body` → `(number, body)`.
fn parse_article_start(block: &str) -> Option<(String, &str)> {
    let block = if block.starts_with(". Artículo ") {
        &block[2..]
    } else {
        block
    };
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

/// Numeric value for a cardinal article heading such as `Artículo Primero`.
/// Article labels elsewhere use numeric ordinal abbreviations (`1o`, `2o`),
/// so this preserves the shared ordering and reference-ID convention while
/// leaving the official provision text unchanged.
fn ordinal_article_value(ordinal: &str) -> Option<u8> {
    let normalized = ordinal
        .to_uppercase()
        .replace('Á', "A")
        .replace('É', "E")
        .replace('Í', "I")
        .replace('Ó', "O")
        .replace('Ú', "U")
        .replace(' ', "");
    let unit = |word| match word {
        "PRIMERO" | "PRIMERA" => Some(1),
        "SEGUNDO" | "SEGUNDA" => Some(2),
        "TERCERO" | "TERCERA" => Some(3),
        "CUARTO" | "CUARTA" => Some(4),
        "QUINTO" | "QUINTA" => Some(5),
        "SEXTO" | "SEXTA" => Some(6),
        "SEPTIMO" | "SEPTIMA" => Some(7),
        "OCTAVO" | "OCTAVA" => Some(8),
        "NOVENO" | "NOVENA" => Some(9),
        _ => None,
    };
    if let Some(value) = unit(&normalized) {
        return Some(value);
    }
    match normalized.as_str() {
        "UNDECIMO" | "UNDECIMA" => return Some(11),
        "DUODECIMO" | "DUODECIMA" => return Some(12),
        _ => {}
    }
    for (prefix, base) in [("DECIMO", 10), ("VIGESIMO", 20), ("TRIGESIMO", 30)] {
        if normalized == prefix {
            return Some(base);
        }
        if let Some(suffix) = normalized.strip_prefix(prefix)
            && let Some(value) = unit(suffix)
        {
            return Some(base + value);
        }
    }
    None
}

/// `Artículo Primero.- body` → `(1o, body)` when an older statute numbers
/// its operative articles with ordinal words. These are distinct from
/// decree-wrapper articles, which are filtered only when their body states an
/// unmistakable promulgation or amendment action.
fn parse_ordinal_article_start<'a>(
    block: &'a str,
    ordinals: &'a [String],
) -> Option<(String, &'a str)> {
    let after_prefix = ["Artículo ", "ARTÍCULO ", "Articulo ", "ARTICULO "]
        .iter()
        .find_map(|prefix| block.strip_prefix(prefix))?
        .trim_start();
    for ordinal in ordinals {
        if let Some((matched, rest)) = strip_prefix_ci(after_prefix, ordinal) {
            for separator in [".-", ".", "-"] {
                if let Some(body) = rest.strip_prefix(separator) {
                    let number = ordinal_article_value(matched)?;
                    return Some((format!("{number}o"), body.trim()));
                }
            }
        }
    }
    None
}

/// A configured separatorless heading: `Artículo 71 El médico…` → `("71",
/// "El médico…")`, only for a label the adapter lists.
fn parse_separatorless_article_start<'a>(
    block: &'a str,
    options: &DiputadosOptions,
) -> Option<(String, &'a str)> {
    options
        .separatorless_article_headings
        .iter()
        .find_map(|label| {
            let body = ["Artículo ", "ARTÍCULO ", "ARTICULO ", "Articulo "]
                .iter()
                .find_map(|prefix| block.strip_prefix(prefix))?
                .strip_prefix(label.as_str())?
                .strip_prefix(' ')?;
            body.starts_with(char::is_uppercase)
                .then(|| (label.clone(), body.trim()))
        })
}

fn parse_statute_article_start<'a>(
    block: &'a str,
    ordinals: &'a [String],
) -> Option<(String, &'a str)> {
    parse_article_start(block).or_else(|| parse_ordinal_article_start(block, ordinals))
}

/// A dot-redacted decree-appendix article: `ARTÍCULO SEGUNDO.- .........`
/// The consolidated PDF sometimes replaces a non-substantive enactment-
/// decree article (e.g. a repealing or budgetary clause) with redaction
/// dots rather than omitting it, immediately after the statute's own
/// numbered articles and before `TRANSITORIOS`. This is decree apparatus,
/// not statute text: the body carries no legal content, only redaction
/// dots, whitespace, or ellipsis characters. That empty-body test is what
/// distinguishes it from a genuine ordinal-numbered statute article
/// (`Artículo Primero.- Esta ley regula…`), whose substantive body must
/// still reach [`parse_ordinal_article_start`] unfiltered. Some sources
/// print the redacted article with a colon (`ARTÍCULO SEGUNDO: ..........`,
/// `reg-lgeepa-mretc`); the colon is accepted here only, never as a general
/// ordinal heading separator.
fn is_dot_redacted_decree_article(block: &str, ordinals: &[String]) -> bool {
    let Some(body) = parse_ordinal_article_start(block, ordinals)
        .map(|(_, body)| body)
        .or_else(|| colon_ordinal_article_body(block, ordinals))
    else {
        return false;
    };
    !body.is_empty()
        && body
            .chars()
            .all(|c| c == '.' || c == '…' || c.is_whitespace())
}

/// The body after `ARTÍCULO <ordinal>:`, for the dot-redaction test only.
fn colon_ordinal_article_body<'a>(block: &'a str, ordinals: &[String]) -> Option<&'a str> {
    let after_prefix = ["Artículo ", "ARTÍCULO ", "Articulo ", "ARTICULO "]
        .iter()
        .find_map(|prefix| block.strip_prefix(prefix))?
        .trim_start();
    ordinals.iter().find_map(|ordinal| {
        let (matched, rest) = strip_prefix_ci(after_prefix, ordinal)?;
        ordinal_article_value(matched)?;
        rest.strip_prefix(':').map(str::trim)
    })
}

/// Either decree-wrapper form the main article loop discards outright:
/// neither appended to the current provision's text nor started as a new
/// one.
fn is_dropped_decree_block(block: &str, ordinals: &[String]) -> bool {
    is_decree_article_wrapper(block, ordinals) || is_dot_redacted_decree_article(block, ordinals)
}

/// The leading base number of an article label (`70-A` → 70, `1o` → 1),
/// for detecting a heading whose number regresses below the sequence.
fn article_base(number: &str) -> Option<u64> {
    let digits: String = number
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == ',')
        .filter(|c| *c != ',')
        .collect();
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

    fn matches(&self, block: &str) -> bool {
        self.libro.is_match(block)
            || self.title.is_match(block)
            || self.chapter.is_match(block)
            || self.section.is_match(block)
            || self.apartado.is_match(block)
    }
}

/// Only a complete dated DOF page header is furniture. A substantive sentence
/// can cite `DIARIO OFICIAL (Primera Sección)` and must remain source text.
fn is_dof_edition_header(line: &str) -> bool {
    static HEADER: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
        let date = r"(?:Lunes|Martes|Miércoles|Miercoles|Jueves|Viernes|Sábado|Sabado|Domingo)\s+\d{1,2}\s+de\s+[a-záéíóú]+\s+de\s+\d{4}";
        let section = r"\([^()\r\n]*Secci[oó]n\)";
        Regex::new(&format!(
            r"(?i)^(?:{date}\s+DIARIO OFICIAL\s+{section}\s+\d+|\d+\s+{section}\s+DIARIO OFICIAL\s+{date})$"
        ))
        .expect("static DOF edition header regex")
    });
    HEADER.is_match(line)
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
        || (line.contains("DIARIO OFICIAL") && line.contains("PAGE"))
        // A DOF edition's running header (`Jueves 30 de noviembre de 2006
        // DIARIO OFICIAL (Séptima Sección) 107`), as `standard.rs` reads it.
        || is_dof_edition_header(line)
        || page_number.is_match(line)
}

/// A legislative closing block is apparatus, never a statutory transitory.
/// Require a place/date opening and actual signatures; an inline citation is
/// insufficient. Called on normalized blocks so wrapped signatures are whole.
fn is_enactment_signature(block: &str) -> bool {
    let upper = block.to_uppercase();
    let place = ["MÉXICO,", "CIUDAD DE MÉXICO,"]
        .iter()
        .any(|p| upper.starts_with(p));
    ((place
        || upper.starts_with("PALACIO NACIONAL,")
        || upper.starts_with("DADO EN LA RESIDENCIA")
        || upper.starts_with("DADO EN EL PALACIO"))
        && upper.contains("RÚBRICA"))
        || (upper.contains("D. P.-") && upper.contains("S. P.-") && upper.contains("RÚBRICA"))
        || upper.starts_with("POR TANTO, MANDO SE IMPRIMA, PUBLIQUE, CIRCULE")
        || upper.starts_with("EN CUMPLIMIENTO DE LO DISPUESTO POR LA FRACCIÓN I DEL ARTÍCULO 89")
        || ((upper.starts_with("DIP.") || upper.starts_with("SEN.")) && upper.contains("RÚBRICA"))
}

fn is_judicial_appendix(block: &str) -> bool {
    block.starts_with("PUNTOS RESOLUTIVOS de la sentencia")
        || block.starts_with("SENTENCIA dictada")
        || block.starts_with("VERSIÓN pública de la Sentencia")
}

fn is_configured_document_stop(block: &str, options: &DiputadosOptions) -> bool {
    options
        .stop_markers
        .iter()
        .any(|marker| block.starts_with(marker))
}

fn is_substantive_annex_start(block: &str, options: &DiputadosOptions) -> bool {
    options
        .annex_markers
        .iter()
        .any(|marker| block.starts_with(marker))
}

fn is_stop_marker(block: &str, options: &DiputadosOptions) -> bool {
    block.contains("ARTÍCULOS TRANSITORIOS DE")
        || block.contains("ARTICULOS TRANSITORIOS DE")
        || is_configured_document_stop(block, options)
}

fn is_transitory_section_header(block: &str) -> bool {
    // The period extension covers printed headings, not a lowercase
    // continuation line such as "... de este" + "transitorio.".
    if block.ends_with('.') && block.chars().next().is_some_and(char::is_lowercase) {
        return false;
    }
    // A trailing colon, a missing accent, and mixed case all occur
    // (`ARTICULOS TRANSITORIOS`, `TRANSITORIOS:`, `Transitorios`).
    let block = block
        .strip_suffix([':', '.'])
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

/// A decree-wrapper article line using an ordinal word instead of a number:
/// `Artículo Primero.- Se expide la Ley…` (promulgation) or `Artículos
/// Segundo a Artículo Cuarto.- ……` (elided decree articles). An ordinal
/// heading alone is insufficient because older statutes can use `Artículo
/// Primero` and `Artículo Segundo` for their operative provisions.
fn is_decree_article_wrapper(block: &str, ordinals: &[String]) -> bool {
    let Some((rest, plural)) = [
        ("Artículo ", false),
        ("Artículos ", true),
        ("ARTÍCULO ", false),
        ("ARTÍCULOS ", true),
        ("ARTICULO ", false),
        ("ARTICULOS ", true),
    ]
    .iter()
    .find_map(|(prefix, plural)| block.strip_prefix(prefix).map(|rest| (rest, *plural))) else {
        return false;
    };
    let Some((_, tail)) = ordinals.iter().find_map(|ordinal| {
        strip_prefix_ci(rest, ordinal)
            .filter(|(_, tail)| tail.chars().next().is_none_or(|c| !c.is_alphanumeric()))
    }) else {
        return false;
    };
    if plural {
        return true;
    }
    let body = tail
        .trim_start_matches(['.', '-'])
        .trim_start()
        .to_uppercase();
    [
        "SE EXPIDE ",
        "SE APRUEBA LA LEY ",
        "SE CREA LA LEY ",
        "SE REFORMA ",
        "SE ADICIONA ",
        "SE DEROGA ",
        "SE ABROGA ",
    ]
    .iter()
    .any(|action| body.starts_with(action))
}

fn is_plural_decree_article_wrapper(block: &str, ordinals: &[String]) -> bool {
    ["Artículos ", "ARTÍCULOS ", "ARTICULOS "]
        .iter()
        .any(|prefix| block.starts_with(prefix))
        && is_decree_article_wrapper(block, ordinals)
}

/// The enacting decree's own title says it issues a law *and* amends others
/// ("SE EXPIDE LA LEY … Y SE REFORMAN …", or the reverse order). Only such an
/// omnibus enactment has a law whose transitories sit between decree articles
/// and a separate overall-decree transitory section; scoping the parser's
/// omnibus path to this title keeps an ordinary law's "Artículos …" block from
/// being read as one.
fn is_omnibus_enactment_title(block: &str) -> bool {
    let title = collapse_whitespace(&block.to_uppercase());
    (title.starts_with("SE ") || title.starts_with("DECRETO "))
        && title.contains("SE EXPIDE")
        && ["SE REFORMAN", "SE ADICIONAN", "SE DEROGAN", "SE ABROGAN"]
            .iter()
            .any(|action| title.contains(action))
}

fn is_immediate_structural(
    line: &str,
    options: &DiputadosOptions,
    headings: &HeadingPatterns,
) -> bool {
    is_stop_marker(line, options)
        || is_judicial_appendix(line)
        || line == "FE DE ERRATAS"
        || is_substantive_annex_start(line, options)
        || is_decree_heading(line)
        || is_reform_regulation_heading(line)
        || headings.matches(line)
        || is_transitory_section_header(line)
}

/// A decree title, as opposed to a wrapped sentence whose next source line
/// happens to begin with `Decreto de ...`. Diputados prints appendix decree
/// titles in uppercase; the title-case forms cover older consolidations
/// without treating an embedded decree citation as a structural boundary. The
/// bare `Decreto de ` prefix is deliberately not matched (it is common inside
/// ordinary prose); `Decreto de reformas ` is narrow enough to admit safely —
/// a decree citing itself as reforms to another instrument, distinct from
/// `Decreto que`/`Decreto por el que` only in its opening connector.
fn is_decree_heading(block: &str) -> bool {
    block.starts_with("DECRETO ")
        || block.starts_with("Decreto por el que ")
        || block.starts_with("Decreto que ")
        || block.starts_with("Decreto de reformas ")
}

/// A later regulation named in a Diputados reform appendix. The mixed
/// uppercase-name/lowercase-connector form distinguishes publication headings
/// (`REGLAMENTO de ...`, `REGLAMENTO en ...`) from the consolidated
/// instrument's all-uppercase running title and ordinary prose.
fn is_reform_regulation_heading(block: &str) -> bool {
    block.starts_with("REGLAMENTO de ") || block.starts_with("REGLAMENTO en ")
}

/// A reform-decree appendix entry whose amending act is itself a *ley*
/// (`LEY que reforma ...`, `Ley que establece ...`) rather than a decreto or
/// reglamento — older consolidations record some reforms this way. The
/// uppercase-name/lowercase-connector `LEY del ...` form is a complete law
/// title in the same publisher style as `REGLAMENTO de ...`; keeping the
/// match to that exact form avoids treating ordinary title-case prose as a
/// structural boundary.
fn is_reform_ley_heading(block: &str) -> bool {
    block.starts_with("LEY que ") || block.starts_with("Ley que ") || block.starts_with("LEY del ")
}

/// Line-level flush trigger, deliberately looser than the block-level
/// parse (a wrapped citation line starting `Artículo 22 de la Ley` still
/// opens a new block, exactly as the original LRITF parser behaved).
fn is_provision_start(line: &str, ordinals: &[String]) -> bool {
    (line.starts_with("Artículo ") || line.starts_with(". Artículo "))
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
fn publisher_title_lines(lines: &[&str]) -> std::collections::HashSet<usize> {
    let mut publisher_title_lines = std::collections::HashSet::new();
    for (index, line) in lines.iter().enumerate() {
        if !line
            .trim()
            .starts_with("CÁMARA DE DIPUTADOS DEL H. CONGRESO DE LA UNIÓN")
        {
            continue;
        }
        // Status notes sharing the publisher's Secretaría line are header
        // metadata too; do not apply this rule to matching body prose.
        for (metadata, content) in lines.iter().enumerate().skip(index + 1).take(2) {
            if content.trim().starts_with("Secretaría ") {
                publisher_title_lines.insert(metadata);
            }
        }
        let start = index.saturating_sub(3);
        for candidate in start..index {
            let title = lines[candidate].trim();
            if ["LEY ", "REGLAMENTO ", "CÓDIGO ", "CODIGO ", "ORDENANZA "]
                .iter()
                .any(|prefix| title.starts_with(prefix))
                && lines[candidate..index].iter().all(|part| {
                    let part = part.trim();
                    !part.is_empty() && part.chars().all(|c| !c.is_lowercase() || c == 'o')
                })
            {
                publisher_title_lines.extend(candidate..index);
                break;
            }
        }
    }
    publisher_title_lines
}

fn normalized_blocks(
    raw: &str,
    options: &DiputadosOptions,
    ordinals: &[String],
    headings: &HeadingPatterns,
) -> Vec<String> {
    let page_number = Regex::new(r"^\d+\s+de\s+\d+$").expect("static regex");
    let nested_signature_date =
        Regex::new(r"^(?:Ciudad de México|México), a \d{1,2} de [a-záéíóúñ]+ de \d{4}\.-")
            .expect("static regex");
    let amendment_mark_end =
        Regex::new(r"(?i)\bDOF\s+\d{2}-\d{2}-\d{4}[.,;:]?$").expect("static regex");
    // A signature closes its decree, so the next page's text is a new block
    // (cff: `…Santiago Creel Miranda.- Rúbrica.` then, after the page
    // break, the next appendix title).
    let signature_end = Regex::new(r#"(?i)\br[úu]bricas?\.?["”]?\.?$"#).expect("static regex");
    let mut blocks = Vec::new();
    let mut current = String::new();
    let mut pending_blank = false;
    let mut crossed_page_furniture = false;

    let lines: Vec<_> = raw.lines().collect();
    let publisher_title_lines = publisher_title_lines(&lines);
    for (index, source_line) in lines.into_iter().enumerate() {
        let line = source_line.trim();
        // These source-backed nested profiles quote each numeric article.
        // Strip only a structural opening quote immediately before its heading;
        // retain all body quotation marks and never mutate acquisition bytes.
        let line = if options.nested_instrument.is_some()
            && ["\"ARTICULO ", "\"ARTÍCULO ", "\"Artículo "]
                .iter()
                .any(|p| line.starts_with(p))
        {
            &line[1..]
        } else {
            line
        };
        if line.is_empty() {
            pending_blank = true;
            continue;
        }
        if publisher_title_lines.contains(&index) || is_page_furniture(line, options, &page_number)
        {
            if !current.is_empty() {
                crossed_page_furniture = true;
            }
            pending_blank = false;
            continue;
        }
        if crossed_page_furniture
            && (amendment_mark_end.is_match(&current) || signature_end.is_match(&current))
        {
            flush_block(&mut current, &mut blocks);
            crossed_page_furniture = false;
        }
        // A Ley-form reform-act heading ("Ley que ...", "LEY que ...") is
        // not made an unconditional `is_immediate_structural` flush trigger
        // like Decreto/Reglamento headings: unlike those, "Ley que" is
        // ordinary Spanish legal prose ("esta Ley que hayan quedado
        // firmes...", "en términos de la Ley que la rige...") and matching
        // it unconditionally corrupted body text wherever a PDF line wrap
        // happened to start with it. Scoped to `crossed_page_furniture`
        // instead: the failure this exists for is specifically a heading
        // silently absorbed into unrelated content across a page break
        // (confirmed against lfd's real committed source), so only force a
        // split there, where the false-positive risk is negligible.
        if crossed_page_furniture && (is_reform_ley_heading(line) || is_short_ley_title_line(line))
        {
            flush_block(&mut current, &mut blocks);
            crossed_page_furniture = false;
        }
        if pending_blank && !crossed_page_furniture {
            flush_block(&mut current, &mut blocks);
        }
        pending_blank = false;
        crossed_page_furniture = false;
        // Formal DOF editions need not leave a blank before wrapped signatures.
        // Only an opted-in nested source's place/date closing line creates this
        // boundary; the whole block must still pass the signature classifier.
        if options.nested_instrument.is_some()
            && (nested_signature_date.is_match(line)
                || line.starts_with(
                    "En cumplimiento de lo dispuesto por la fracción I del Artículo 89",
                ))
        {
            flush_block(&mut current, &mut blocks);
        }
        if !options.annex_markers.is_empty()
            && line.starts_with(
                "SALÓN DE SESIONES DE LA CÁMARA DE DIPUTADOS DEL HONORABLE CONGRESO DE LA UNIÓN.-",
            )
        {
            flush_block(&mut current, &mut blocks);
        }
        if is_immediate_structural(line, options, headings) {
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
    implicit_singleton: bool,
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
            implicit_singleton: false,
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
            implicit_singleton: false,
        };
        builder.push_block(first_block);
        builder
    }

    fn annex(number: &str, label: &str) -> Self {
        Self {
            provision_type: ProvisionType::Annex,
            number: number.to_owned(),
            label: label.to_owned(),
            heading: HeadingContext {
                libro: None,
                title: None,
                chapter: None,
                section: None,
                apartado: None,
            },
            blocks: Vec::new(),
            implicit_singleton: false,
        }
    }

    fn push_block(&mut self, value: &str) {
        let value = value.trim();
        if !value.is_empty() {
            self.blocks.push(value.to_owned());
        }
    }

    fn finish(
        self,
        instrument_id: &str,
        publication_date: Option<NaiveDate>,
        period: Option<&lex_core::PublicationPeriod>,
    ) -> Provision {
        let (kind, canonical_number) = match self.provision_type {
            // Ordinal marks carry no identity: article "2o" is canonical
            // id ":article:2", so a citation of bare "2" resolves to it.
            ProvisionType::Article => ("article", labels::canonical_slug(&self.number)),
            ProvisionType::Transitory => ("transitory", slug(&self.number)),
            ProvisionType::Annex => ("annex", slug(&self.number)),
        };
        let text = self.blocks.join("\n\n");
        let text = if self.provision_type == ProvisionType::Transitory {
            if let Some(index) = ["Dado en la Residencia", "Dado en la residencia"]
                .iter()
                .filter_map(|marker| text.find(marker))
                .min()
            {
                text[..index].trim_end().to_owned()
            } else {
                text
            }
        } else {
            text
        };
        Provision {
            numbering_basis: self
                .implicit_singleton
                .then_some(lex_core::NumberingBasis::ImplicitSingleton),
            schema_version: SCHEMA_VERSION.to_owned(),
            id: format!("{instrument_id}:{kind}:{canonical_number}"),
            instrument_id: instrument_id.to_owned(),
            provision_type: self.provision_type,
            label: self.label,
            number: self.number,
            heading_context: self.heading,
            text: text.clone(),
            publication_date,
            publication_period: period.cloned(),
            effective_from: None,
            effective_to: None,
            temporal_status: crate::initial_temporal_status(&text),
            temporal_basis: None,
            temporal_confidence: None,
            review_status: ReviewStatus::NotAnalyzed,
            transitory_effects: Vec::new(),
            amendment_marks: Vec::new(),
            repeals: crate::initial_repeals(&text),
            commencement_condition: None,
        }
    }
}

#[allow(clippy::too_many_lines)]
/// Partition only an adapter's explicitly reviewed source boundaries. No title,
/// ordinal, or "La presente Ley" phrase alone infers legal ownership.
fn partition_nested_instrument(
    blocks: &[String],
    raw: &str,
    options: &DiputadosOptions,
    profile: &lex_core::NestedInstrumentProfile,
    date: Option<NaiveDate>,
    period: Option<&lex_core::PublicationPeriod>,
) -> Result<(Vec<String>, Vec<lex_core::EnactmentEvidence>)> {
    use lex_core::{EnactmentEvidence, EnactmentEvidenceKind, EnactmentOwner};
    let unique = |marker: &str| -> Result<usize> {
        anyhow::ensure!(
            !marker.trim().is_empty(),
            "empty nested-instrument boundary"
        );
        let found: Vec<_> = blocks
            .iter()
            .enumerate()
            .filter(|(_, b)| b.starts_with(marker))
            .map(|(i, _)| i)
            .collect();
        anyhow::ensure!(
            found.len() == 1,
            "nested-instrument boundary must occur once: {marker:?} (found {})",
            found.len()
        );
        Ok(found[0])
    };
    let wrapper = unique(&profile.wrapper_marker)?;
    let start = unique(&profile.body_start_marker)?;
    let end = unique(&profile.body_end_marker)?;
    anyhow::ensure!(
        wrapper < start && start < end,
        "nested-instrument boundaries are out of order"
    );
    anyhow::ensure!(
        !profile.containing_act_title.trim().is_empty()
            && blocks[..wrapper]
                .join(" ")
                .contains(&profile.containing_act_title),
        "nested-instrument containing title not corroborated by source"
    );
    let stop = blocks
        .iter()
        .enumerate()
        .skip(end + 1)
        .find(|(_, b)| is_enactment_signature(b) || is_stop_marker(b, options))
        .map(|(i, _)| i)
        .ok_or_else(|| anyhow::anyhow!("nested-instrument lacks corroborated source closure"))?;
    let outer: Vec<_> = blocks
        .iter()
        .enumerate()
        .take(stop)
        .skip(end)
        .filter(|(_, b)| b.starts_with(&profile.outer_transitory_marker))
        .map(|(i, _)| i)
        .collect();
    anyhow::ensure!(
        !profile.outer_transitory_marker.trim().is_empty() && outer.len() == 1,
        "nested-instrument outer transitory boundary must occur once before source closure"
    );
    let outer = outer[0];
    anyhow::ensure!(
        outer > end,
        "nested-instrument lacks containing sibling article"
    );
    let mut evidence = Vec::new();
    let mut add = |kind: EnactmentEvidenceKind,
                   number: String,
                   heading: String,
                   text: String,
                   section: Option<String>| {
        evidence.push(EnactmentEvidence {
            schema_version: SCHEMA_VERSION.to_owned(),
            id: lex_core::containing_act_evidence_id(&options.instrument_id, date, &kind, &number),
            instrument_id: options.instrument_id.clone(),
            owner: EnactmentOwner::ContainingAct,
            kind,
            containing_act_title: profile.containing_act_title.clone(),
            section_heading: section,
            source_heading: heading,
            number,
            text_sha256: lex_core::evidence_sha256(&text),
            text,
            publication_date: date,
            publication_period: period.cloned(),
            source_sha256: String::new(),
            extracted_text_sha256: lex_core::evidence_sha256(raw),
        });
    };
    // The first record includes all source preamble and elided earlier siblings;
    // the second preserves substantive quoted amendments as one containing-act
    // region, never feeding their article numbers into the nested-law parser.
    add(
        EnactmentEvidenceKind::ContainingArticle,
        "wrapper".to_owned(),
        blocks[wrapper].clone(),
        blocks[..start].join("\n\n"),
        None,
    );
    add(
        EnactmentEvidenceKind::ContainingArticle,
        "following".to_owned(),
        blocks[end].clone(),
        blocks[end..outer].join("\n\n"),
        None,
    );
    let ordinals = transitory_ordinals();
    let mut current: Option<(String, String, String)> = None;
    let mut seen = std::collections::HashSet::new();
    for block in &blocks[outer + 1..stop] {
        if is_transitory_section_header(block) {
            bail!("repeated containing-act transitory section");
        }
        if let Some((number, body)) = parse_transitory_start(block, &ordinals)
            .or_else(|| parse_numeric_transitory_start(block))
        {
            anyhow::ensure!(
                seen.insert(slug(number)),
                "duplicate containing-act transitory ordinal"
            );
            if let Some((n, h, text)) = current.take() {
                add(
                    EnactmentEvidenceKind::Transitory,
                    n,
                    h,
                    text,
                    Some(blocks[outer].clone()),
                );
            }
            current = Some((
                number.to_owned(),
                block[..block.len() - body.len()].trim().to_owned(),
                body.to_owned(),
            ));
        } else if let Some((_, _, text)) = &mut current {
            text.push_str("\n\n");
            text.push_str(block);
        } else {
            bail!("unattributed containing-act transitory text");
        }
    }
    if let Some((n, h, text)) = current {
        add(
            EnactmentEvidenceKind::Transitory,
            n,
            h,
            text,
            Some(blocks[outer].clone()),
        );
    }
    anyhow::ensure!(
        !seen.is_empty(),
        "nested-instrument lacks containing-act transitories"
    );
    Ok((blocks[start..end].to_vec(), evidence))
}

#[allow(clippy::too_many_lines)]
fn partition_enactment(
    blocks: Vec<String>,
    raw: &str,
    options: &DiputadosOptions,
    date: Option<NaiveDate>,
    period: Option<&lex_core::PublicationPeriod>,
) -> Result<(Vec<String>, Vec<lex_core::EnactmentEvidence>)> {
    use lex_core::{EnactmentEvidence, EnactmentEvidenceKind, EnactmentOwner};
    if let Some(profile) = &options.nested_instrument {
        anyhow::ensure!(
            !options.separate_enactment,
            "nested and legacy enactment modes conflict"
        );
        return partition_nested_instrument(&blocks, raw, options, profile, date, period);
    }
    if !options.separate_enactment {
        return Ok((blocks, Vec::new()));
    }
    let ordinals = transitory_ordinals();
    let wrapper = blocks
        .iter()
        .position(|b| is_decree_article_wrapper(b, &ordinals))
        .ok_or_else(|| anyhow::anyhow!("enactment partition lacks enacting article wrapper"))?;
    let containing_act_title = blocks[..wrapper]
        .iter()
        .enumerate()
        .find_map(|(i, _)| {
            let title = blocks[i..wrapper].join(" ");
            is_omnibus_enactment_title(&title).then_some(title)
        })
        .ok_or_else(|| {
            anyhow::anyhow!("enactment partition lacks corroborating containing-decree title")
        })?;
    let sibling = blocks
        .iter()
        .enumerate()
        .skip(wrapper + 1)
        .find(|(_, b)| {
            let upper = b.to_uppercase();
            (upper.starts_with("ARTÍCULOS SEGUNDO A ")
                || upper.starts_with("ARTICULOS SEGUNDO A ")
                || upper.starts_with("ARTÍCULO SEGUNDO:")
                || upper.starts_with("ARTÍCULO SEGUNDO."))
                && (b.contains('…') || is_dot_redacted_decree_article(b, &ordinals))
        })
        .map(|(i, _)| i)
        .ok_or_else(|| {
            anyhow::anyhow!("enactment partition lacks elided sibling decree article")
        })?;
    let mut main = blocks[..sibling].to_vec();
    let mut evidence: Vec<EnactmentEvidence> = Vec::new();
    let mut in_outer = false;
    let mut heading = String::new();
    let mut seen = std::collections::HashSet::new();
    let date_key = date.map_or_else(|| "serialized-range".to_owned(), |d| d.to_string());
    let mut consumed_header = false;
    for block in &blocks[sibling..] {
        if is_stop_marker(block, options) || is_enactment_signature(block) {
            break;
        }
        let upper = block.to_uppercase();
        if is_transitory_section_header(block)
            || upper.starts_with("TRANSITORIOS DEL DECRETO POR EL QUE ")
        {
            if consumed_header {
                bail!("ambiguous repeated containing-decree transitory header");
            }
            in_outer = true;
            consumed_header = true;
            heading.clone_from(block);
            continue;
        }
        if !in_outer {
            if !block.contains('…') && !is_dot_redacted_decree_article(block, &ordinals) {
                bail!("unrecognized sibling decree material: {block}");
            }
            evidence.push(EnactmentEvidence {
                schema_version: SCHEMA_VERSION.to_owned(),
                id: format!(
                    "{}:enactment:{date_key}:decree:elided:{}",
                    options.instrument_id,
                    evidence.len() + 1
                ),
                instrument_id: options.instrument_id.clone(),
                owner: EnactmentOwner::ContainingDecree,
                kind: EnactmentEvidenceKind::ElidedSiblingArticle,
                containing_act_title: containing_act_title.clone(),
                section_heading: None,
                source_heading: block.clone(),
                number: (evidence.len() + 1).to_string(),
                text: block.clone(),
                publication_date: date,
                publication_period: period.cloned(),
                source_sha256: String::new(),
                extracted_text_sha256: lex_core::evidence_sha256(raw),
                text_sha256: String::new(),
            });
            continue;
        }
        if let Some((ordinal, body)) = parse_transitory_start(block, &ordinals)
            .or_else(|| parse_numeric_transitory_start(block))
        {
            if !seen.insert(slug(ordinal)) {
                bail!("duplicate containing-decree ordinal {ordinal}");
            }
            evidence.push(EnactmentEvidence {
                schema_version: SCHEMA_VERSION.to_owned(),
                id: format!(
                    "{}:enactment:{date_key}:decree:transitory:{}",
                    options.instrument_id,
                    slug(ordinal)
                ),
                instrument_id: options.instrument_id.clone(),
                owner: EnactmentOwner::ContainingDecree,
                kind: EnactmentEvidenceKind::Transitory,
                containing_act_title: containing_act_title.clone(),
                section_heading: Some(heading.clone()),
                source_heading: block[..block.len() - body.len()].trim().to_owned(),
                number: ordinal.to_owned(),
                text: body.to_owned(),
                publication_date: date,
                publication_period: period.cloned(),
                source_sha256: String::new(),
                extracted_text_sha256: lex_core::evidence_sha256(raw),
                text_sha256: String::new(),
            });
        } else if let Some(item) = evidence
            .last_mut()
            .filter(|e| e.kind == EnactmentEvidenceKind::Transitory)
        {
            item.text.push_str("\n\n");
            item.text.push_str(block);
        } else {
            bail!("unattributed containing-decree body");
        }
    }
    for item in &mut evidence {
        item.text_sha256 = lex_core::evidence_sha256(&item.text);
    }
    if !consumed_header || seen.is_empty() {
        bail!("containing decree lacks transitories");
    }
    // Keep main parsing on its existing stop semantics without consuming outer text.
    main.push("ARTÍCULOS TRANSITORIOS DE DECRETOS DE REFORMA".to_owned());
    Ok((main, evidence))
}

pub fn parse_diputados(
    raw: &str,
    options: &DiputadosOptions,
    publication_date: NaiveDate,
) -> Result<DiputadosDocument> {
    parse_diputados_publication(raw, options, Some(publication_date), None)
}

#[allow(clippy::too_many_lines)]
pub fn parse_diputados_publication(
    raw: &str,
    options: &DiputadosOptions,
    publication_date: Option<NaiveDate>,
    publication_period: Option<&lex_core::PublicationPeriod>,
) -> Result<DiputadosDocument> {
    let patterns = HeadingPatterns::new()?;
    let ordinals = transitory_ordinals();
    let blocks = normalized_blocks(raw, options, &ordinals, &patterns);
    let (blocks, enactment_evidence) =
        partition_enactment(blocks, raw, options, publication_date, publication_period)?;

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
    let mut separatorless_found: Vec<String> = Vec::new();
    let mut roman_transitory_section = false;
    let mut singleton_heading: Option<String> = None;
    let mut in_substantive_annex = false;
    let mut in_annex_apparatus = false;
    // The omnibus path is gated on the decree title in the preamble, before
    // the law's own text begins.
    let omnibus_enactment = blocks
        .iter()
        .take(80)
        .any(|block| is_omnibus_enactment_title(block));
    let mut pending_overall_decree_transitory = false;
    let mut enactment_layer_ordinals: std::collections::HashSet<String> =
        std::collections::HashSet::new();
    let mut last_article_base: Option<u64> = None;
    let mut seen_ordinals: std::collections::HashSet<String> = std::collections::HashSet::new();

    let mut singleton_closed = false;
    let mut preamble_quote_depth: usize = 0;
    for block in blocks {
        // Before the instrument's first article, a cover note may quote
        // another decree's articles (cfpc: "Nota sobre la vigencia…: “Artículo
        // Segundo. … Artículo Tercero. …”"). A block that opens such a quoted
        // article, or continues its quotation, is preamble, never an article.
        // A quoted enacting formula (lgdlpi: `“EL CONGRESO…`) is not one.
        if provisions.is_empty() && current.is_none() {
            let opens = block.strip_prefix('“').is_some_and(is_article_shaped_word);
            if preamble_quote_depth > 0 || opens {
                let opened = block.matches('“').count();
                let closed = block.matches('”').count();
                preamble_quote_depth = (preamble_quote_depth + opened).saturating_sub(closed);
                continue;
            }
        }
        if in_substantive_annex {
            // Annex contents may quote appendix headings, errata or judgments.
            // Only an explicitly configured block-prefix boundary ends this
            // opaque source region; generic heading heuristics do not.
            if is_configured_document_stop(&block, options) {
                break;
            }
            if let Some(builder) = &mut current {
                builder.push_block(&block);
            }
            continue;
        }
        if is_stop_marker(&block, options)
            || block == "FE DE ERRATAS"
            || is_judicial_appendix(&block)
            || (current.is_some()
                && options.annex_markers.is_empty()
                && is_enactment_signature(&block))
        {
            singleton_closed = current.as_ref().is_some_and(|b| b.implicit_singleton)
                && (is_enactment_signature(&block) || is_stop_marker(&block, options));
            break;
        }
        if is_substantive_annex_start(&block, options) {
            if let Some(builder) = current.take() {
                provisions.push(builder.finish(
                    &options.instrument_id,
                    publication_date,
                    publication_period,
                ));
            }
            // The printed description is part of the annex heading, including
            // text after a numbered heading's period (PEF Annex1).
            let label = block.trim();
            current = Some(ProvisionBuilder::annex("1", label));
            in_substantive_annex = true;
            continue;
        }
        if in_statute_transitories
            && !options.annex_markers.is_empty()
            && (block.starts_with("Dado en la Residencia")
                || block.starts_with("Dado en la residencia")
                || block.starts_with("SALÓN DE SESIONES DE LA CÁMARA DE DIPUTADOS DEL HONORABLE CONGRESO DE LA UNIÓN.-"))
        {
            if let Some(builder) = current.take() {
                provisions.push(builder.finish(
                    &options.instrument_id,
                    publication_date,
                    publication_period,
                ));
            }
            in_annex_apparatus = true;
            continue;
        }
        if in_annex_apparatus {
            // The configured substantive annex marker above ends this skip.
            // Source signatures stay in acquisition bytes, not in a provision.
            continue;
        }
        if is_transitory_section_header(&block) {
            // A statute has a single transitory section; a second header
            // is a reform decree's, so its transitorios (repeating
            // PRIMERO, SEGUNDO, …) do not belong to the instrument. The
            // narrow exception is a law issued inside an omnibus decree:
            // after the law's own transitory decree article and ellipsized
            // sibling decree articles, the overall decree has its own
            // transitory section and both layers belong to the enactment.
            if in_statute_transitories {
                if pending_overall_decree_transitory {
                    pending_overall_decree_transitory = false;
                    enactment_layer_ordinals = std::mem::take(&mut seen_ordinals);
                    continue;
                }
                break;
            }
            if let Some(builder) = current.take() {
                provisions.push(builder.finish(
                    &options.instrument_id,
                    publication_date,
                    publication_period,
                ));
            }
            in_statute_transitories = true;
            if block
                .trim_end_matches([':', '.'])
                .to_uppercase()
                .ends_with("TRANSITORIO")
            {
                singleton_heading = Some(block.clone());
            }
            heading = HeadingContext {
                libro: None,
                title: None,
                chapter: None,
                section: None,
                apartado: None,
            };
            continue;
        }
        if pending_overall_decree_transitory {
            bail!(
                "{}: omnibus decree article wrapper was not followed by the overall decree's transitory section (found {:?})",
                options.instrument_id,
                block.chars().take(60).collect::<String>()
            );
        }

        if in_statute_transitories {
            if singleton_heading.is_some()
                && current.as_ref().is_some_and(|b| b.implicit_singleton)
                && (parse_transitory_start(&block, &ordinals).is_some()
                    || parse_article_start(&block).is_some())
            {
                bail!("mixed numbered and unnumbered transitories are not supported");
            }
            // A reform decree following the statute transitories ends the
            // main document; its transitories belong to reform evidence.
            if block.starts_with("DECRETO por") || block.starts_with("REFORMAS Y ADICIONES") {
                break;
            }
            if omnibus_enactment && is_plural_decree_article_wrapper(&block, &ordinals) {
                if let Some(builder) = current.take() {
                    provisions.push(builder.finish(
                        &options.instrument_id,
                        publication_date,
                        publication_period,
                    ));
                }
                pending_overall_decree_transitory = true;
                continue;
            }
            if let Some((ordinal, body)) = parse_transitory_start(&block, &ordinals)
                .or_else(|| parse_numeric_transitory_start(&block))
            {
                // A repeated ordinal starts a reform decree's transitorios
                // (unmarked by a header); the instrument's own transitorios
                // are unique, so stop rather than duplicate them.
                if enactment_layer_ordinals.contains(&ordinal.to_uppercase()) {
                    bail!(
                        "{}: overall decree transitory {ordinal} repeats an ordinal already used by the law's own transitories",
                        options.instrument_id
                    );
                }
                if !seen_ordinals.insert(ordinal.to_uppercase()) {
                    break;
                }
                if let Some(builder) = current.take() {
                    provisions.push(builder.finish(
                        &options.instrument_id,
                        publication_date,
                        publication_period,
                    ));
                }
                current = Some(ProvisionBuilder::transitory(ordinal, body));
                continue;
            }
            // Some regulations number the original transitory section I, II,
            // ... . Enter this mode only on its initial I heading, before a
            // named transitory exists, so internal roman fractions stay text.
            if (roman_transitory_section || current.is_none())
                && let Some((ordinal, body)) = parse_roman_transitory_start(&block)
                && (roman_transitory_section || ordinal == "I")
            {
                roman_transitory_section = true;
                if let Some(builder) = current.take() {
                    provisions.push(builder.finish(
                        &options.instrument_id,
                        publication_date,
                        publication_period,
                    ));
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
                    provisions.push(builder.finish(
                        &options.instrument_id,
                        publication_date,
                        publication_period,
                    ));
                }
                current = Some(ProvisionBuilder::transitory(&number, body));
                continue;
            }
        } else {
            if is_dropped_decree_block(&block, &ordinals) {
                if let Some(builder) = current.take() {
                    provisions.push(builder.finish(
                        &options.instrument_id,
                        publication_date,
                        publication_period,
                    ));
                }
                continue;
            }
            if patterns.apply(&block, &mut heading) {
                if let Some(builder) = current.take() {
                    provisions.push(builder.finish(
                        &options.instrument_id,
                        publication_date,
                        publication_period,
                    ));
                }
                continue;
            }
            let separatorless = parse_separatorless_article_start(&block, options);
            if let Some((label, _)) = &separatorless {
                separatorless_found.push(label.clone());
            }
            if let Some((number, body)) =
                separatorless.or_else(|| parse_statute_article_start(&block, &ordinals))
            {
                let base = article_base(&number);
                // A consolidated código quotes other laws' articles inside
                // editorial notes ("…el artículo 54 de la citada Ley, a la
                // letra señalaba: Artículo 54.- …"). A heading whose base
                // number falls below the sequence position already reached
                // is such a quote, not a new article; keep it in the
                // current provision's body instead of restarting the count.
                let regresses =
                    matches!((base, last_article_base), (Some(b), Some(last)) if b < last);
                // With no provision open (just after a structural heading)
                // there is no body to quote into, so the heading would be
                // silently lost.
                if regresses && current.is_none() {
                    bail!(
                        "{}: article {number} follows article {} after a heading, with no provision to hold it",
                        options.instrument_id,
                        last_article_base.unwrap_or_default()
                    );
                }
                if !regresses {
                    if let Some(builder) = current.take() {
                        provisions.push(builder.finish(
                            &options.instrument_id,
                            publication_date,
                            publication_period,
                        ));
                    }
                    last_article_base = base.or(last_article_base);
                    current = Some(ProvisionBuilder::article(number, body, heading.clone()));
                    continue;
                }
            }
        }

        if let Some(builder) = &mut current {
            builder.push_block(&block);
        } else if !provisions.is_empty() && is_article_shaped(&block) {
            // After a structural heading no provision is open, so an article
            // the grammar did not read (lgtoc's `Artículo 228 a.-` before its
            // suffix was read) would be silently discarded.
            bail!(
                "{}: unread article heading after a structural heading: {:?}",
                options.instrument_id,
                block.chars().take(100).collect::<String>()
            );
        } else if let Some(label) = singleton_heading
            .as_ref()
            .filter(|_| in_statute_transitories)
        {
            // No printed ordinal is inserted into source text or display label.
            let mut builder = ProvisionBuilder::transitory("UNICO", &block);
            builder.label.clone_from(label);
            builder.implicit_singleton = true;
            current = Some(builder);
        } else if in_statute_transitories {
            bail!(
                "{}: unnumbered transitory body requires an explicit source identity: {:?}",
                options.instrument_id,
                block.chars().take(100).collect::<String>()
            );
        }
    }

    if current.as_ref().is_some_and(|b| b.implicit_singleton) && !singleton_closed {
        bail!("unnumbered singleton has no verified closing boundary");
    }
    if let Some(builder) = current {
        provisions.push(builder.finish(
            &options.instrument_id,
            publication_date,
            publication_period,
        ));
    }
    if provisions.is_empty() {
        bail!("no provisions recognized for {}", options.instrument_id);
    }
    for label in &options.separatorless_article_headings {
        let found = separatorless_found
            .iter()
            .filter(|found| *found == label)
            .count();
        anyhow::ensure!(
            found == 1,
            "{}: configured separatorless heading `Artículo {label}` opens {found} blocks, not exactly one",
            options.instrument_id
        );
    }
    let reform_evidence = extract_reform_evidence(raw, options)?;
    Ok(DiputadosDocument {
        provisions,
        reform_evidence,
        enactment_evidence,
    })
}

/// The original DOF publication date from the consolidated document's own
/// header note ("Nueva Ley publicada en el Diario Oficial de la Federación
/// el 16 de abril de 2025"). Deterministic: first match wins, and the
/// header note always precedes any reform-decree publication line.
#[must_use]
pub fn extract_dof_publication(raw: &str) -> Option<NaiveDate> {
    let note = Regex::new(
        r"(?i)publicad[oa] en (?:el|la) (?:Diario Oficial de la Federación|DOF) (?:el )?(\d{1,2})[oº]? de ([a-zá-úñ]+) (?:de )?(\d{4})",
    )
    .ok()?;
    let end = ["ARTÍCULOS TRANSITORIOS DE", "ARTICULOS TRANSITORIOS DE"]
        .iter()
        .filter_map(|marker| raw.find(marker))
        .min()
        .unwrap_or(raw.len());
    let normalized = collapse_whitespace(&raw[..end]);
    let captures = note.captures(&normalized)?;
    spanish_date(&captures[1], &captures[2].to_lowercase(), &captures[3])
}

struct ReformEvidenceBuilder {
    date: NaiveDate,
    act_kind: &'static str,
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
        let act_kind = builder.act_kind;
        let decree_occurrence = builder.decree_occurrence;
        let transitory_section_occurrence = builder.transitory_section_occurrence;
        let ordinal = builder.ordinal;
        let text = builder
            .blocks
            .into_iter()
            .filter(|block| !block.is_empty())
            .collect::<Vec<_>>()
            .join("\n\n");
        let text = ["Dado en la Residencia", "Dado en la residencia"]
            .iter()
            .filter_map(|marker| text.find(marker))
            .min()
            .map_or(text.as_str(), |index| text[..index].trim_end())
            .to_owned();
        let mut item = reform_evidence_item(
            instrument_id,
            date,
            &ordinal,
            act_kind,
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
                    "Transitorio {ordinal} — Sección transitoria {section}, {act_kind} DOF {date}"
                ),
                (decree, 1) => {
                    format!("Transitorio {ordinal} — {act_kind} {decree} DOF {date}")
                }
                (decree, section) => format!(
                    "Transitorio {ordinal} — {act_kind} {decree}, sección transitoria {section} DOF {date}"
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

/// An all-caps `LEY <title>` line in the reform appendix is an amending act
/// title only when it is a short, title-shaped line. A long block, a
/// continuation or running header, a line citing a DOF date, or a sentence
/// that merely begins `LEY ` is not.
fn is_short_ley_title_line(block: &str) -> bool {
    // The appendix prints the title and, often, its publication note in one
    // block ("LEY <title>. Publicada en el Diario Oficial …"); anything else
    // after the title sentence means the block is prose.
    let (title, rest) = block
        .split_once(". ")
        .map_or((block, ""), |(title, rest)| (title, rest.trim_start()));
    (title.starts_with("LEY ") || title.starts_with("Ley de "))
        && title.chars().count() <= 160
        && !title.contains("DOF")
        && !title.to_lowercase().contains("continuación")
        && (rest.is_empty() || rest.starts_with("Publicad"))
}

fn reform_act_heading_kind(block: &str) -> Option<&'static str> {
    if is_decree_heading(block) {
        Some("Decreto")
    } else if is_reform_regulation_heading(block) {
        Some("Reglamento")
    // This runs only after the explicit reform-appendix boundary. In that
    // narrowed context, an all-caps `LEY <title>` is an amending act title,
    // including the LIE appendix's `LEY Reglamentaria del Servicio
    // Ferroviario.` form. Keep the broader recognition here rather than in
    // the main-document structural parser, where `LEY ` can begin ordinary
    // source text.
    } else if is_reform_ley_heading(block) || is_short_ley_title_line(block) {
        Some("Ley")
    } else if is_reforms_heading(block) {
        Some("Reformas")
    } else {
        None
    }
}

/// An appendix entry titled by its reforms (cpf: `REFORMAS al Código Penal
/// para el Distrito y Territorios Federal.`, DOF 14-01-1966), in the same
/// uppercase-name/lowercase-connector style as `DECRETO que` and
/// `REGLAMENTO de`.
fn is_reforms_heading(block: &str) -> bool {
    ["REFORMAS ", "REFORMA "]
        .iter()
        .find_map(|prefix| block.strip_prefix(prefix))
        .is_some_and(|rest| rest.starts_with(|c: char| c.is_lowercase()))
}

/// Whether a block is exactly this instrument's running header, which by
/// default is its uppercased official title, ignoring accents and a final
/// period (`PRODUCCION` for `PRODUCCIÓN`).
fn names_this_instrument(block: &str, options: &DiputadosOptions) -> bool {
    let key =
        |value: &str| labels::strip_accents(value.trim().trim_end_matches('.')).to_uppercase();
    let block = key(block);
    options.header_lines.iter().any(|line| key(line) == block)
}

fn is_reform_closing_furniture(uppercase: &str) -> bool {
    [
        "CIUDAD DE MÉXICO",
        "MÉXICO, D",
        "SALÓN DE SESIONES",
        "SALON DE SESIONES",
        "FE DE ERRATAS",
        "DADO EN LA RESIDENCIA",
    ]
    .iter()
    .any(|prefix| uppercase.starts_with(prefix))
}

fn decree_publication_date(block: &str, publication_re: &Regex) -> Option<NaiveDate> {
    let captures = publication_re.captures(block)?;
    spanish_date(&captures[1], &captures[2].to_lowercase(), &captures[3])
}

/// Isolate the consolidated document's reform-decree appendix
/// (`ARTÍCULOS TRANSITORIOS DE DECRETOS DE REFORMA`) into per-decree
/// transitory evidence for temporal analysis.
#[allow(clippy::too_many_lines)]
pub fn extract_reform_evidence(
    raw: &str,
    options: &DiputadosOptions,
) -> Result<Vec<TemporalEvidence>> {
    let headings = HeadingPatterns::new()?;
    let ordinals = transitory_ordinals();
    let publication_re = Regex::new(
        r"(?i)publicad[oa] en (?:el|la) (?:Diario Oficial de la Federación|DOF) (?:el )?(\d{1,2})[oº]? de ([a-zá-úñ]+) (?:de )?(\d{4})",
    )?;
    // An appendix title of any form followed by its publication note, in
    // one block (cff: `VIGESIMA Cuarta Resolución de Modificaciones a la
    // Resolución Miscelánea Fiscal para 2000 y anexos 5, 7 y 14. Publicada
    // en el Diario Oficial de la Federación el 12 de febrero de 2002`).
    let titled_by_note = Regex::new(
        r"\S.*\s+Publicad[oa] en el Diario Oficial de la Federación el \d{1,2}[oº]? de [a-záéíóúñ]+ (?:de )?\d{4}\.?$",
    )?;
    let mut in_reform_appendix = false;
    let mut in_substantive_annex = false;
    let mut in_judicial_appendix = false;
    let mut in_transitories = false;
    let mut publication_date: Option<NaiveDate> = None;
    let mut decree_heading: Option<String> = None;
    let mut act_kind: Option<&'static str> = None;
    let mut decree_occurrence: Option<usize> = None;
    let mut transitory_section_occurrence = 0;
    let mut section_ordinals: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut decrees_by_date: std::collections::HashMap<NaiveDate, usize> =
        std::collections::HashMap::new();
    let mut current: Option<ReformEvidenceBuilder> = None;
    let mut evidence = Vec::new();

    for block in normalized_blocks(raw, options, &ordinals, &headings) {
        if in_substantive_annex {
            if !is_configured_document_stop(&block, options) {
                continue;
            }
            in_substantive_annex = false;
        }
        if is_substantive_annex_start(&block, options) {
            flush_reform(&options.instrument_id, &mut current, &mut evidence);
            in_substantive_annex = true;
            in_reform_appendix = false;
            in_transitories = false;
            publication_date = None;
            decree_heading = None;
            act_kind = None;
            decree_occurrence = None;
            transitory_section_occurrence = 0;
            continue;
        }
        // Judicial dispositions quote prior decrees; they are retained in the
        // source manifest, not relabeled as new legislative transitories.
        if is_judicial_appendix(&block) {
            flush_reform(&options.instrument_id, &mut current, &mut evidence);
            in_transitories = false;
            in_judicial_appendix = true;
            publication_date = None;
            act_kind = None;
            continue;
        }
        if in_judicial_appendix {
            if reform_act_heading_kind(&block).is_none() {
                continue;
            }
            in_judicial_appendix = false;
        }
        if !in_reform_appendix
            && block.contains("ARTÍCULOS TRANSITORIOS DE")
            && block.contains("DECRETO")
        {
            in_reform_appendix = true;
            continue;
        }
        // Some official compilations omit the Diputados appendix banner.
        // A configured main-document stop marker may opt in only when the
        // same block is independently recognized as an amending act title.
        if !in_reform_appendix
            && is_stop_marker(&block, options)
            && reform_act_heading_kind(&block).is_some()
        {
            in_reform_appendix = true;
        }
        if !in_reform_appendix {
            continue;
        }
        let uppercase = block.to_uppercase();
        // An omnibus decree heads the part that amends this instrument with
        // its title (`LEY DEL IMPUESTO ESPECIAL SOBRE PRODUCCION Y SERVICIOS`
        // after `Ley que establece las reducciones impositivas… Publicada
        // en el Diario Oficial…`). Inside that open decree, before its
        // transitories, the title is a section heading, not a new act.
        let section_of_open_decree = publication_date.is_some()
            && !in_transitories
            && names_this_instrument(&block, options)
            && decree_publication_date(&block, &publication_re).is_none();
        // While an act with a date is open, a further title with its own
        // publication note starts the next act even when its form is not
        // otherwise recognized.
        let noted_title =
            (publication_date.is_some() && titled_by_note.is_match(&block)).then(|| {
                if block.contains("Resolución") {
                    "Resolución"
                } else {
                    "Acto"
                }
            });
        if !section_of_open_decree
            && let Some(kind) = reform_act_heading_kind(&block).or(noted_title)
        {
            flush_reform(&options.instrument_id, &mut current, &mut evidence);
            in_transitories = false;
            publication_date = None;
            decree_heading = Some(block.clone());
            act_kind = Some(kind);
            decree_occurrence = None;
            transitory_section_occurrence = 0;
        }
        // Wrapped decree titles can share a block with their publication
        // note. A decree's operative article can quote an earlier decree's
        // publication date, so only title material—not a provision—may set
        // the containing decree's date. Once the transitory section has
        // begun, the same wording is likewise a citation inside legal text.
        if !in_transitories
            && publication_date.is_none()
            && let Some(date) = decree_publication_date(&block, &publication_re)
        {
            publication_date = Some(date);
            let occurrence = decrees_by_date.entry(date).or_default();
            *occurrence += 1;
            decree_occurrence = Some(*occurrence);
            continue;
        }
        if is_reform_transitory_section_header(&block) {
            if in_transitories {
                flush_reform(&options.instrument_id, &mut current, &mut evidence);
            }
            in_transitories = true;
            transitory_section_occurrence += 1;
            section_ordinals.clear();
            continue;
        }
        if !in_transitories {
            continue;
        }
        if is_reform_closing_furniture(&uppercase) {
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
            // An omnibus decree goes on with its operative articles after one
            // law's transitories (cff, DOF 15-12-1995: `Cuarto.` then
            // `Artículo Cuarto. Se realizan las modificaciones siguientes al
            // Código Fiscal…`). An `Artículo` heading that repeats an ordinal
            // already read in this section is such an article, so the section
            // has ended.
            let repeated = !section_ordinals.insert(slug(&ordinal));
            if repeated && is_article_shaped_word(&block) {
                flush_reform(&options.instrument_id, &mut current, &mut evidence);
                in_transitories = false;
                continue;
            }
            flush_reform(&options.instrument_id, &mut current, &mut evidence);
            let date = publication_date.ok_or_else(|| {
                anyhow::anyhow!(
                    "found reform transitory {ordinal:?} without its Diario Oficial publication date after decree heading {:?}",
                    decree_heading.as_deref().unwrap_or("<none>")
                )
            })?;
            current = Some(ReformEvidenceBuilder {
                date,
                act_kind: act_kind.ok_or_else(|| {
                    anyhow::anyhow!("found reform transitory without its containing act heading")
                })?,
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

    use super::{
        DiputadosOptions, is_dot_redacted_decree_article, is_omnibus_enactment_title,
        parse_diputados, parse_ordinal_article_start, transitory_ordinals,
    };

    const CODIGO_FIXTURE: &str = include_str!("../../../fixtures/diputados/codigo-sample.txt");
    const NUMBERED_PARAGRAPH_FIXTURE: &str =
        include_str!("../../../fixtures/diputados/separate-heading-numbered-paragraph-sample.txt");
    const TITLE_CASE_HEADING_FIXTURE: &str =
        include_str!("../../../fixtures/diputados/title-case-heading-sample.txt");
    const TITLE_CASE_WRAPPED_WORD_FIXTURE: &str =
        include_str!("../../../fixtures/diputados/title-case-wrapped-word-sample.txt");
    const PAGE_BOUNDARY_AMENDMENT_FIXTURE: &str =
        include_str!("../../../fixtures/diputados/page-boundary-amendment-mark-sample.txt");
    const ORIGINAL_TRANSITORY_SIGNATURE_FIXTURE: &str = include_str!(
        "../../../fixtures/diputados/original-transitory-signature-boundary-sample.txt"
    );
    const REFORM_VARIANTS_FIXTURE: &str =
        include_str!("../../../fixtures/diputados/reform-appendix-variants-sample.txt");
    const REFORM_PUBLICATION_CITATION_FIXTURE: &str =
        include_str!("../../../fixtures/diputados/reform-publication-citation-sample.txt");
    const REFORM_REGULATION_HEADINGS_FIXTURE: &str =
        include_str!("../../../fixtures/diputados/reform-appendix-regulation-headings-sample.txt");
    const REFORM_WRAPPED_TITLE_ARTICLE_FIXTURE: &str =
        include_str!("../../../fixtures/diputados/reform-wrapped-title-article-sample.txt");
    const MULTIPLE_TRANSITORY_SECTIONS_FIXTURE: &str =
        include_str!("../../../fixtures/diputados/reform-multiple-transitory-sections-sample.txt");
    const ORDINAL_STATUTE_ARTICLES_FIXTURE: &str =
        include_str!("../../../fixtures/diputados/ordinal-statute-articles-sample.txt");
    const DOT_REDACTED_DECREE_ARTICLE_FIXTURE: &str =
        include_str!("../../../fixtures/diputados/dot-redacted-decree-article-sample.txt");
    const COLON_DOT_REDACTED_DECREE_ARTICLE_FIXTURE: &str =
        include_str!("../../../fixtures/diputados/colon-dot-redacted-decree-article-sample.txt");
    const WRAPPED_RUNNING_HEADER_REGULATION_FIXTURE: &str =
        include_str!("../../../fixtures/diputados/wrapped-running-header-regulation-sample.txt");
    const SUBSTANTIVE_APPENDIX_FIXTURE: &str =
        include_str!("../../../fixtures/diputados/substantive-appendix-sample.txt");
    const DOF_EDITION_PAGE_HEADER_FIXTURE: &str =
        include_str!("../../../fixtures/diputados/dof-edition-page-header-sample.txt");
    const SEPARATORLESS_ARTICLE_HEADING_FIXTURE: &str =
        include_str!("../../../fixtures/diputados/separatorless-article-heading-sample.txt");
    const ARTICLE_QUINTUS_FIXTURE: &str =
        include_str!("../../../fixtures/diputados/article-quintus-sample.txt");
    const REFORM_ORDINAL_FIRST_DAY_FIXTURE: &str =
        include_str!("../../../fixtures/diputados/reform-ordinal-first-day-publication-sample.txt");
    const REFORM_LEY_HEADING_ACROSS_PAGE_FURNITURE_FIXTURE: &str = include_str!(
        "../../../fixtures/diputados/reform-ley-heading-across-page-furniture-sample.txt"
    );
    const REFORM_DECRETO_DE_REFORMAS_HEADING_FIXTURE: &str =
        include_str!("../../../fixtures/diputados/reform-decreto-de-reformas-heading-sample.txt");
    const REFORM_BARE_LEY_TITLE_FIXTURE: &str =
        include_str!("../../../fixtures/diputados/reform-bare-ley-title-sample.txt");
    const REFORM_UPPERCASE_LEY_TITLE_FIXTURE: &str =
        include_str!("../../../fixtures/diputados/reform-uppercase-ley-title-sample.txt");
    const COVER_NOTE_QUOTED_DECREE_ARTICLES_FIXTURE: &str =
        include_str!("../../../fixtures/diputados/cover-note-quoted-decree-articles-sample.txt");
    const ARTICLE_LOWERCASE_LETTER_SUFFIX_FIXTURE: &str =
        include_str!("../../../fixtures/diputados/article-lowercase-letter-suffix-sample.txt");
    const REFORM_OMNIBUS_OPERATIVE_ARTICLE_AND_RESOLUTION_FIXTURE: &str = include_str!(
        "../../../fixtures/diputados/reform-omnibus-operative-article-and-resolution-sample.txt"
    );
    const REFORM_REFORMAS_TITLE_AND_PAGE_SPLIT_ORDINAL_FIXTURE: &str = include_str!(
        "../../../fixtures/diputados/reform-reformas-title-and-page-split-ordinal-sample.txt"
    );
    const REFORM_OMNIBUS_OWN_TITLE_SECTION_FIXTURE: &str =
        include_str!("../../../fixtures/diputados/reform-omnibus-own-title-section-sample.txt");
    const ARTICLE_LATIN_ORDINAL_AFTER_CHAPTER_FIXTURE: &str =
        include_str!("../../../fixtures/diputados/article-latin-ordinal-after-chapter-sample.txt");
    const ARTICLE_LETTER_SUFFIX_QUALIFIER_FIXTURE: &str =
        include_str!("../../../fixtures/diputados/article-letter-suffix-qualifier-sample.txt");
    const NESTED_OMNIBUS_TRANSITORY_FIXTURE: &str =
        include_str!("../../../fixtures/diputados/nested-omnibus-transitory-sample.txt");

    fn options(instrument_id: &str, title: &str) -> DiputadosOptions {
        DiputadosOptions {
            instrument_id: instrument_id.to_owned(),
            header_lines: vec![title.to_uppercase()],
            stop_markers: Vec::new(),
            annex_markers: Vec::new(),
            separate_enactment: false,
            nested_instrument: None,
            separatorless_article_headings: Vec::new(),
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
    fn parses_quintus_article_qualifier() {
        let document = parse_diputados(
            ARTICLE_QUINTUS_FIXTURE,
            &options(
                "urn:lex-mx:federal:statute:sample",
                "Ley General de Muestra",
            ),
            NaiveDate::from_ymd_opt(2004, 5, 20).expect("valid date"),
        )
        .expect("quintus fixture parses");
        let numbers = document
            .provisions
            .iter()
            .map(|provision| provision.number.as_str())
            .collect::<Vec<_>>();
        assert_eq!(numbers, ["54 Quáter", "54 Quintus", "55"]);
    }

    /// Regression (merged-heading split, 2026-10-18): a Latin ordinal beyond
    /// `Nonies` was not a heading, so `33 Decies` stayed inside `33 Nonies`
    /// and the text after a following chapter heading (`33 Undecies`) was
    /// lost from the corpus (`reg-lsem`, `lgmdfp`).
    #[test]
    fn latin_ordinal_headings_after_a_chapter_are_separate_articles() {
        let document = parse_diputados(
            ARTICLE_LATIN_ORDINAL_AFTER_CHAPTER_FIXTURE,
            &options(
                "urn:lex-mx:federal:statute:sample",
                "Ley General de Muestra",
            ),
            NaiveDate::from_ymd_opt(2004, 5, 20).expect("valid date"),
        )
        .expect("latin ordinal fixture parses");
        let numbers = document
            .provisions
            .iter()
            .map(|provision| provision.number.as_str())
            .collect::<Vec<_>>();
        assert_eq!(numbers, ["33 Nonies", "33 Decies", "33 Undecies", "34"]);
        assert!(!document.provisions[0].text.contains("Decies"));
        assert!(document.provisions[2].text.contains("undecies"));
        assert_eq!(
            document.provisions[2].heading_context.chapter.as_deref(),
            Some("Capítulo III")
        );
    }

    #[test]
    fn preserves_configured_substantive_appendix_as_one_annex() {
        let mut options = options(
            "urn:lex-mx:federal:regulation:sample",
            "Reglamento de Control Sanitario de Productos y Servicios",
        );
        options.annex_markers = vec!["APÉNDICE DEL REGLAMENTO DE CONTROL SANITARIO".to_owned()];
        let document = parse_diputados(
            SUBSTANTIVE_APPENDIX_FIXTURE,
            &options,
            NaiveDate::from_ymd_opt(1999, 8, 9).expect("valid date"),
        )
        .expect("substantive appendix fixture parses");

        assert_eq!(document.provisions.len(), 3);
        assert_eq!(
            document.provisions[0].id,
            format!("{}:article:1", options.instrument_id)
        );
        assert_eq!(
            document.provisions[1].provision_type,
            ProvisionType::Transitory
        );
        assert!(!document.provisions[1].text.contains("Residencia"));
        assert_eq!(document.provisions[2].provision_type, ProvisionType::Annex);
        assert_eq!(document.provisions[2].number, "1");
        assert!(
            document.provisions[2]
                .text
                .contains("I.1. Los establecimientos")
        );
        assert!(document.provisions[2].text.contains("ARTÍCULO 99."));
    }

    #[test]
    fn ordinal_word_statute_articles_are_not_dropped_as_decree_wrappers() {
        let document = parse_diputados(
            ORDINAL_STATUTE_ARTICLES_FIXTURE,
            &options(
                "urn:lex-mx:federal:statute:sample",
                "Ley Reglamentaria de Muestra",
            ),
            NaiveDate::from_ymd_opt(1982, 12, 27).expect("valid date"),
        )
        .expect("ordinal statute-article fixture parses");

        let articles: Vec<_> = document
            .provisions
            .iter()
            .filter(|provision| provision.provision_type == ProvisionType::Article)
            .collect();
        assert_eq!(articles.len(), 2);
        assert_eq!(articles[0].number, "1o");
        assert_eq!(articles[0].text, "Regla operativa primera.");
        assert_eq!(articles[1].number, "2o");
        assert_eq!(articles[1].text, "Regla operativa segunda.");
        let transitories: Vec<_> = document
            .provisions
            .iter()
            .filter(|provision| provision.provision_type == ProvisionType::Transitory)
            .collect();
        assert_eq!(transitories.len(), 1);
        assert_eq!(transitories[0].number, "UNICO");
        assert_eq!(transitories[0].text, "Entrada en vigor.");
    }

    #[test]
    fn nested_omnibus_enactment_keeps_law_and_overall_decree_transitories() {
        let document = parse_diputados(
            NESTED_OMNIBUS_TRANSITORY_FIXTURE,
            &options(
                "urn:lex-mx:federal:statute:lraf",
                "Ley para Regular las Agrupaciones Financieras",
            ),
            NaiveDate::from_ymd_opt(2014, 1, 10).expect("valid date"),
        )
        .expect("nested omnibus fixture parses");

        let transitories: Vec<_> = document
            .provisions
            .iter()
            .filter(|provision| provision.provision_type == ProvisionType::Transitory)
            .collect();
        assert_eq!(transitories.len(), 2);
        assert_eq!(transitories[0].number, "QUINCUAGÉSIMO SEGUNDO");
        assert!(
            transitories[0]
                .text
                .contains("publicada el 18 de julio de 1990")
        );
        assert!(!transitories[0].text.contains("QUINCUAGÉSIMO TERCERO"));
        assert_eq!(transitories[1].number, "ÚNICO");
        assert_eq!(
            transitories[1].text,
            "El presente Decreto entrará en vigor al día siguiente de su publicación."
        );
        assert!(document.provisions.iter().all(|provision| {
            !provision
                .text
                .contains("El presente Decreto entrará en vigor al día siguiente.")
        }));
    }

    #[test]
    fn reform_appendix_ley_heading_must_be_a_short_title_line() {
        for title in [
            "LEY Reglamentaria del Servicio Ferroviario.",
            "LEY DEL IMPUESTO SOBRE LA RENTA",
            "LEY de Ingresos de la Federación para el ejercicio fiscal de 2010",
        ] {
            assert_eq!(
                super::reform_act_heading_kind(title),
                Some("Ley"),
                "{title}"
            );
        }
        for not_a_title in [
            "LEY DEL IMPUESTO SOBRE LA RENTA (continuación)",
            "LEY GENERAL DE SOCIEDADES MERCANTILES Artículo reformado DOF 12-01-2010",
            "LEY ORGÁNICA. El Congreso de la Unión expedirá las disposiciones necesarias.",
            "LEY APLICABLE A LOS CONTRATOS QUE CELEBREN LAS DEPENDENCIAS Y ENTIDADES DE LA ADMINISTRACIÓN PÚBLICA FEDERAL CON PERSONAS FÍSICAS O MORALES NACIONALES O EXTRANJERAS, EN LOS TÉRMINOS Y CONDICIONES QUE SE ESTABLEZCAN EN EL REGLAMENTO",
        ] {
            assert_eq!(
                super::reform_act_heading_kind(not_a_title),
                None,
                "{not_a_title}"
            );
        }
    }

    #[test]
    fn omnibus_enactment_is_recognised_from_the_decree_title_only() {
        for title in [
            "SE EXPIDE LA LEY DE AHORRO Y CRÉDITO POPULAR Y SE REFORMAN Y DEROGAN\nDIVERSAS DISPOSICIONES DE LA LEY GENERAL DE ORGANIZACIONES",
            "SE REFORMAN, ADICIONAN Y DEROGAN DIVERSAS DISPOSICIONES EN MATERIA FINANCIERA Y SE EXPIDE LA LEY PARA REGULAR LAS AGRUPACIONES FINANCIERAS.",
        ] {
            assert!(is_omnibus_enactment_title(title), "{title}");
        }
        for title in [
            "SE EXPIDE LA LEY FEDERAL DE CÁMARAS EMPRESARIALES.",
            "SE REFORMAN Y ADICIONAN DIVERSAS DISPOSICIONES DE LA LEY FEDERAL DEL TRABAJO.",
            "Artículos Primero y Segundo se reforman y se expide el reglamento.",
        ] {
            assert!(!is_omnibus_enactment_title(title), "{title}");
        }
    }

    fn omnibus_fixture_without_title() -> String {
        NESTED_OMNIBUS_TRANSITORY_FIXTURE
            .lines()
            .skip(2)
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn plural_decree_wrapper_is_inert_without_an_omnibus_title() {
        let document = parse_diputados(
            &omnibus_fixture_without_title(),
            &options(
                "urn:lex-mx:federal:statute:lraf",
                "Ley para Regular las Agrupaciones Financieras",
            ),
            NaiveDate::from_ymd_opt(2014, 1, 10).expect("valid date"),
        )
        .expect("ordinary law still parses");
        // Without the title the overall decree's transitory is not admitted.
        assert!(
            document
                .provisions
                .iter()
                .all(|provision| provision.number != "ÚNICO")
        );
    }

    #[test]
    fn omnibus_wrapper_without_overall_transitory_header_fails_loudly() {
        let raw = NESTED_OMNIBUS_TRANSITORY_FIXTURE.replace(
            "TRANSITORIO\n\nÚNICO.-",
            "ARTÍCULO QUINCUAGÉSIMO QUINTO.- Texto de otro artículo del decreto.\n\nÚNICO.-",
        );
        let error = parse_diputados(
            &raw,
            &options(
                "urn:lex-mx:federal:statute:lraf",
                "Ley para Regular las Agrupaciones Financieras",
            ),
            NaiveDate::from_ymd_opt(2014, 1, 10).expect("valid date"),
        )
        .expect_err("silent truncation is rejected");
        assert!(
            error
                .to_string()
                .contains("overall decree's transitory section")
        );
    }

    #[test]
    fn omnibus_overall_transitory_reusing_a_law_ordinal_fails_loudly() {
        let raw = NESTED_OMNIBUS_TRANSITORY_FIXTURE.replace(
            "ÚNICO.- El presente Decreto entrará en vigor al día siguiente de su publicación.",
            "QUINCUAGÉSIMO SEGUNDO.- El presente Decreto entrará en vigor al día siguiente de su publicación.",
        );
        let error = parse_diputados(
            &raw,
            &options(
                "urn:lex-mx:federal:statute:lraf",
                "Ley para Regular las Agrupaciones Financieras",
            ),
            NaiveDate::from_ymd_opt(2014, 1, 10).expect("valid date"),
        )
        .expect_err("duplicate canonical ids are rejected");
        assert!(error.to_string().contains("repeats an ordinal"));
    }

    #[test]
    fn dot_redacted_decree_articles_are_dropped_not_appended_or_created() {
        // Regression for the lfrpe defect: the consolidated PDF's dot-
        // redacted enactment-decree articles ("ARTÍCULO SEGUNDO.- ........."
        // / "ARTÍCULO TERCERO.- .........") between the last numbered
        // statute article and TRANSITORIOS must neither be appended to that
        // article's text nor create spurious ordinal-numbered articles.
        let document = parse_diputados(
            DOT_REDACTED_DECREE_ARTICLE_FIXTURE,
            &options("urn:lex-mx:federal:statute:sample", "Ley de Muestra"),
            NaiveDate::from_ymd_opt(2004, 12, 31).expect("valid date"),
        )
        .expect("dot-redacted decree article fixture parses");

        let articles: Vec<_> = document
            .provisions
            .iter()
            .filter(|provision| provision.provision_type == ProvisionType::Article)
            .collect();
        assert_eq!(articles.len(), 1);
        assert_eq!(articles[0].number, "3");
        assert_eq!(
            articles[0].text,
            "Texto sustantivo del último artículo numerado de la ley."
        );
        assert!(!articles[0].text.contains("ARTÍCULO SEGUNDO"));
        assert!(!articles[0].text.contains("ARTÍCULO TERCERO"));

        let transitories: Vec<_> = document
            .provisions
            .iter()
            .filter(|provision| provision.provision_type == ProvisionType::Transitory)
            .collect();
        assert_eq!(transitories.len(), 2);
        assert_eq!(transitories[0].number, "PRIMERO");
        assert_eq!(transitories[1].number, "SEGUNDO");
        assert_eq!(transitories[1].text, "Disposición transitoria segunda.");
    }

    #[test]
    fn colon_dot_redacted_decree_article_is_dropped_not_appended() {
        // Regression for reg-lgeepa-mretc: "ARTÍCULO SEGUNDO: .........."
        // after Article 32 was appended to that article's text.
        let document = parse_diputados(
            COLON_DOT_REDACTED_DECREE_ARTICLE_FIXTURE,
            &options(
                "urn:lex-mx:federal:regulation:sample",
                "Reglamento de Muestra",
            ),
            NaiveDate::from_ymd_opt(2004, 6, 3).expect("valid date"),
        )
        .expect("colon dot-redacted decree article fixture parses");

        let articles: Vec<_> = document
            .provisions
            .iter()
            .filter(|provision| provision.provision_type == ProvisionType::Article)
            .collect();
        assert_eq!(articles.len(), 1);
        assert_eq!(articles[0].number, "32");
        assert!(
            articles[0]
                .text
                .ends_with("Artículo reformado DOF 31-10-2014")
        );
        assert!(!articles[0].text.contains("ARTÍCULO SEGUNDO"));

        let transitories: Vec<_> = document
            .provisions
            .iter()
            .filter(|provision| provision.provision_type == ProvisionType::Transitory)
            .collect();
        assert_eq!(transitories.len(), 2);
        assert!(
            transitories[1]
                .text
                .ends_with("entrada en vigor del presente Decreto.")
        );
    }

    #[test]
    fn colon_is_not_a_general_ordinal_article_separator() {
        let ordinals = transitory_ordinals();
        assert!(is_dot_redacted_decree_article(
            "ARTÍCULO SEGUNDO: ..........",
            &ordinals
        ));
        assert!(!is_dot_redacted_decree_article(
            "ARTÍCULO SEGUNDO: Se reforma el artículo 3.",
            &ordinals
        ));
        assert!(
            parse_ordinal_article_start("ARTÍCULO SEGUNDO: Se reforma el artículo 3.", &ordinals)
                .is_none()
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
    fn title_case_heading_word_inside_wrapped_sentence_stays_inline() {
        let document = parse_diputados(
            TITLE_CASE_WRAPPED_WORD_FIXTURE,
            &options(
                "urn:lex-mx:federal:statute:sample",
                "Ley Reglamentaria de Muestra",
            ),
            NaiveDate::from_ymd_opt(2015, 11, 4).expect("valid date"),
        )
        .expect("wrapped title-case word fixture parses");

        assert_eq!(document.provisions.len(), 1);
        assert_eq!(
            document.provisions[0].text,
            "En el escrito por el que se solicite el inicio del procedimiento a que se refiere este Capítulo deberán señalarse:\n\nI. Nombre de la parte solicitante."
        );
        assert_eq!(
            document.provisions[0].heading_context.chapter.as_deref(),
            Some("Capítulo III")
        );
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
            annex_markers: Vec::new(),
            separate_enactment: false,
            nested_instrument: None,
            separatorless_article_headings: Vec::new(),
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
    fn dof_section_citations_in_substantive_prose_are_not_headers() {
        let text =
            include_str!("../../../fixtures/diputados/dof-section-citation-prose-sample.txt");
        let document = parse_diputados(
            text,
            &options(
                "urn:lex-mx:federal:regulation:sample",
                "Reglamento de Muestra",
            ),
            NaiveDate::from_ymd_opt(2006, 11, 30).unwrap(),
        )
        .unwrap();
        assert_eq!(document.provisions.len(), 2);
        assert!(document.provisions[0].text.contains(
            "El aviso se publicará en el DIARIO OFICIAL (Primera Sección) y surtirá efectos al día siguiente."
        ));
        assert!(document.provisions[1].text.contains(
            "La referencia DIARIO OFICIAL (Séptima Sección) 107 identifica la publicación del aviso."
        ));
    }

    #[test]
    fn dof_edition_running_headers_never_enter_provision_text() {
        // reg-pmimss is the DOF publication itself; both page parities'
        // headers fall inside a paragraph.
        let document = parse_diputados(
            DOF_EDITION_PAGE_HEADER_FIXTURE,
            &options(
                "urn:lex-mx:federal:regulation:sample",
                "Reglamento de Muestra",
            ),
            NaiveDate::from_ymd_opt(2006, 11, 30).expect("valid date"),
        )
        .expect("DOF edition fixture parses");
        assert_eq!(document.provisions.len(), 2);
        assert!(document.provisions.iter().all(|provision| {
            !provision.text.contains("DIARIO OFICIAL") && !provision.text.contains("Sección")
        }));
        assert!(
            document.provisions[0]
                .text
                .contains("órganos de operación administrativa desconcentrada")
        );
        // Ordinary prose that cites the Diario Oficial stays.
        assert!(
            document.provisions[1]
                .text
                .contains("publicación en el Diario Oficial de la Federación")
        );
    }

    #[test]
    fn configured_separatorless_article_heading_opens_its_article() {
        // reg-pmimss prints `Artículo 71 El médico…` without the period.
        let parse = |labels: &[&str]| {
            let mut options = options(
                "urn:lex-mx:federal:regulation:sample",
                "Reglamento de Muestra",
            );
            options.separatorless_article_headings =
                labels.iter().map(|label| (*label).to_owned()).collect();
            parse_diputados(
                SEPARATORLESS_ARTICLE_HEADING_FIXTURE,
                &options,
                NaiveDate::from_ymd_opt(2006, 11, 30).expect("valid date"),
            )
        };
        let numbers = |document: &super::DiputadosDocument| {
            document
                .provisions
                .iter()
                .map(|provision| provision.number.clone())
                .collect::<Vec<_>>()
        };

        let configured = parse(&["71"]).expect("configured heading parses");
        assert_eq!(numbers(&configured), ["70", "71", "72"]);
        assert!(
            configured.provisions[1]
                .text
                .starts_with("El médico tratante, en todos")
        );
        assert!(!configured.provisions[0].text.contains("Artículo 71"));

        // Unconfigured, the heading stays inside article 70: no global rule.
        let unconfigured = parse(&[]).expect("unconfigured fixture parses");
        assert_eq!(numbers(&unconfigured), ["70", "72"]);
        assert!(
            unconfigured.provisions[0]
                .text
                .contains("Artículo 71 El médico")
        );

        // A configured label that never opens a block fails the parse.
        let error = parse(&["71", "73"]).unwrap_err().to_string();
        assert!(error.contains("`Artículo 73` opens 0 blocks"), "{error}");
    }

    #[test]
    fn configured_wrapped_running_header_never_enters_provision_text() {
        let options = DiputadosOptions {
            instrument_id: "urn:lex-mx:federal:regulation:sample".to_owned(),
            header_lines: vec![
                "REGLAMENTO DE LA LEY GENERAL DE SALUD EN MATERIA DE CONTROL SANITARIO DE ACTIVIDADES,"
                    .to_owned(),
                "ESTABLECIMIENTOS, PRODUCTOS Y SERVICIOS".to_owned(),
            ],
            stop_markers: Vec::new(),
            annex_markers: Vec::new(),
            separate_enactment: false, nested_instrument: None,
            separatorless_article_headings: Vec::new(),
        };
        let document = parse_diputados(
            WRAPPED_RUNNING_HEADER_REGULATION_FIXTURE,
            &options,
            NaiveDate::from_ymd_opt(1988, 1, 18).expect("valid date"),
        )
        .expect("wrapped running-header fixture parses");

        assert_eq!(document.provisions.len(), 2);
        assert_eq!(document.provisions[0].text, "Texto inicial.");
        assert_eq!(
            document.provisions[1].text,
            "Primer párrafo que continúa en la siguiente página."
        );
        assert!(
            document
                .provisions
                .iter()
                .all(|provision| !provision.text.contains("REGLAMENTO DE LA LEY GENERAL"))
        );
    }

    #[test]
    fn numeric_original_transitories_are_parsed_in_the_transitory_section() {
        let source = "LEY DE PRUEBA\n\nARTICULO 1o.- Primer artículo.\n\nARTICULO 2o.- Segundo artículo.\n\nTRANSITORIOS:\n\n1o.- La presente Ley entrará en vigor cinco días después de su publicación.\n\n2o.- Se derogan las disposiciones que se opongan a la presente.\n";
        let options = options("urn:lex-mx:federal:statute:sample", "Ley de Prueba");
        let document = parse_diputados(
            source,
            &options,
            NaiveDate::from_ymd_opt(1947, 12, 31).expect("valid date"),
        )
        .expect("numeric transitory fixture parses");
        let transitories: Vec<_> = document
            .provisions
            .iter()
            .filter(|provision| provision.provision_type == ProvisionType::Transitory)
            .collect();
        assert_eq!(transitories.len(), 2);
        assert_eq!(transitories[0].number, "1o");
        assert_eq!(
            transitories[1].text,
            "Se derogan las disposiciones que se opongan a la presente."
        );
    }

    #[test]
    fn numeric_transitory_heading_requires_dot_dash_separator() {
        assert_eq!(
            super::parse_numeric_transitory_start("5o.- Se concede un plazo."),
            Some(("5o", "Se concede un plazo."))
        );
        assert_eq!(
            super::parse_numeric_transitory_start("1o. de enero de 1948 entra en vigor."),
            None
        );
        assert_eq!(super::parse_numeric_transitory_start("100o.- texto"), None);
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
    fn a_first_of_month_publication_date_written_with_the_ordinal_mark_is_recognized() {
        // "el 1º de septiembre de 2005" -- the day-of-month ordinal mark
        // (º, or its ASCII "o" spelling) is common specifically for the
        // first of a month. Before this fixture the publication-date regex
        // inside extract_reform_evidence required a bare digit immediately
        // before " de", so this decree's date was never recognized and its
        // first transitory hit the "without its Diario Oficial publication
        // date" hard error -- confirmed against a real committed source
        // (lspcapf) where an Artículo Único plus a nested TRANSITORIOS
        // heading sits between the decree heading and its transitories.
        // extract_dof_publication's own regex already tolerates `[oº]?`;
        // this one did not.
        let evidence = super::extract_reform_evidence(
            REFORM_ORDINAL_FIRST_DAY_FIXTURE,
            &options("urn:lex-mx:federal:statute:sample", "Ley de Muestra"),
        )
        .expect("a 1º publication date must not strand the transitory that follows it");

        assert_eq!(
            evidence
                .iter()
                .map(|item| item.provision_id.as_str())
                .collect::<Vec<_>>(),
            [
                "urn:lex-mx:federal:statute:sample:amendment:2005-09-01:transitory:primero",
                "urn:lex-mx:federal:statute:sample:amendment:2005-09-01:transitory:segundo",
            ]
        );
    }

    #[test]
    fn a_ley_heading_split_from_prior_content_by_page_furniture_is_recognized() {
        // A real omnibus reform recorded as a Ley (not a Decreto or
        // Reglamento) -- confirmed against lfd's real committed source,
        // where an unrelated correction note and this heading share a page
        // break -- would get silently absorbed into the preceding block,
        // leaving `act_kind` unset and crashing on the first transitory
        // that followed. Unlike Decreto/Reglamento headings,
        // `is_reform_ley_heading` is deliberately NOT an unconditional
        // `is_immediate_structural` flush trigger: "Ley que" is ordinary
        // Spanish legal prose, and matching it unconditionally corrupted
        // body text wherever a PDF line wrap happened to start with it
        // (found by regression-testing against the existing corpus before
        // this landed). It is instead scoped to `crossed_page_furniture`,
        // matching the `amendment_mark_end` precedent already in
        // `normalized_blocks` -- narrow enough to fix this real failure
        // without the false-positive risk.
        let evidence = super::extract_reform_evidence(
            REFORM_LEY_HEADING_ACROSS_PAGE_FURNITURE_FIXTURE,
            &options("urn:lex-mx:federal:statute:sample", "Ley de Muestra"),
        )
        .expect(
            "a Ley heading split by page furniture must not strand the transitory that follows it",
        );

        assert_eq!(evidence.len(), 1);
        assert_eq!(
            evidence[0].provision_id,
            "urn:lex-mx:federal:statute:sample:amendment:1985-05-05:transitory:primero"
        );
        assert!(evidence[0].label.contains("Ley DOF 1985-05-05"));
    }

    #[test]
    fn a_decreto_de_reformas_heading_is_recognized_as_a_decree_heading() {
        // `is_decree_heading` deliberately does not match the bare
        // `Decreto de ` prefix (it is common inside ordinary prose), but
        // `Decreto de reformas ` is narrow enough to admit safely --
        // confirmed against lgdp's real committed source, whose first
        // reform decree is titled "Decreto de reformas y adiciones a
        // diversos artículos..." rather than the already-recognized
        // "Decreto que"/"Decreto por el que" forms.
        let evidence = super::extract_reform_evidence(
            REFORM_DECRETO_DE_REFORMAS_HEADING_FIXTURE,
            &options("urn:lex-mx:federal:statute:sample", "Ley de Muestra"),
        )
        .expect("a Decreto de reformas heading must be recognized as the containing act heading");

        assert_eq!(evidence.len(), 1);
        assert_eq!(
            evidence[0].provision_id,
            "urn:lex-mx:federal:statute:sample:amendment:1983-12-28:transitory:unico"
        );
        assert!(evidence[0].label.contains("Decreto DOF 1983-12-28"));
    }

    #[test]
    fn a_bare_ley_del_title_is_recognized_as_a_reform_act_heading() {
        // Regression for LFGFAGA: its reform appendix includes the complete
        // "LEY del Banco de México." as the amending act, rather than a
        // Decreto or a "Ley que ..." reform heading. The preceding FE DE
        // ERRATAS entry has its own publication date but no transitories, so
        // the bare Ley title must reset the containing-act identity before
        // the Banco de México transitory section begins.
        let evidence = super::extract_reform_evidence(
            REFORM_BARE_LEY_TITLE_FIXTURE,
            &options("urn:lex-mx:federal:statute:sample", "Ley de Muestra"),
        )
        .expect("a bare LEY del title must identify the containing reform act");

        assert_eq!(evidence.len(), 2);
        assert_eq!(
            evidence[0].provision_id,
            "urn:lex-mx:federal:statute:sample:amendment:1993-12-23:transitory:primero"
        );
        assert_eq!(
            evidence[1].provision_id,
            "urn:lex-mx:federal:statute:sample:amendment:1993-12-23:transitory:decimo-octavo"
        );
        assert!(evidence.iter().all(|item| item.label.contains("Ley DOF")));
    }

    #[test]
    fn an_uppercase_ley_title_in_the_reform_appendix_is_a_containing_act() {
        let evidence = super::extract_reform_evidence(
            REFORM_UPPERCASE_LEY_TITLE_FIXTURE,
            &options("urn:lex-mx:federal:statute:sample", "Ley de Muestra"),
        )
        .expect("an uppercase LEY title in the reform appendix identifies the act");

        assert_eq!(evidence.len(), 1);
        assert_eq!(
            evidence[0].provision_id,
            "urn:lex-mx:federal:statute:sample:amendment:1995-05-12:transitory:primero"
        );
        assert!(evidence[0].label.contains("Ley DOF 1995-05-12"));
    }

    #[test]
    fn a_cover_note_quoting_another_decrees_articles_is_not_an_article() {
        // cfpc's cover note quotes the CNPCF decree's `“Artículo Segundo. …
        // Artículo Tercero. …”`. Read as Article 3, it made the real
        // Articles 1 and 2 "regress" after `LIBRO PRIMERO` and vanish.
        let document = parse_diputados(
            COVER_NOTE_QUOTED_DECREE_ARTICLES_FIXTURE,
            &options(
                "urn:lex-mx:federal:code:cfpc",
                "Código Federal de Procedimientos Civiles",
            ),
            NaiveDate::from_ymd_opt(1943, 2, 24).unwrap(),
        )
        .unwrap();
        let numbers: Vec<&str> = document
            .provisions
            .iter()
            .map(|provision| provision.number.as_str())
            .collect();
        assert_eq!(numbers, ["1º", "2º", "3º"]);
        assert!(
            document.provisions[0]
                .text
                .starts_with("Sólo puede iniciar")
        );
        assert_eq!(
            document.provisions[0].heading_context.chapter.as_deref(),
            Some("Capítulo I")
        );
    }

    #[test]
    fn lower_case_letter_suffix_articles_after_a_chapter_heading_are_read() {
        // lgtoc prints `Artículo 228 a.-` and `228 a bis.-` after `CAPITULO V
        // Bis`. Unread, they were silently dropped, and lmv's citations of
        // LGTOC 228-B and 228-H could not resolve.
        let document = parse_diputados(
            ARTICLE_LOWERCASE_LETTER_SUFFIX_FIXTURE,
            &options(
                "urn:lex-mx:federal:statute:lgtoc",
                "Ley General de Títulos y Operaciones de Crédito",
            ),
            NaiveDate::from_ymd_opt(1932, 8, 27).unwrap(),
        )
        .unwrap();
        let ids: Vec<&str> = document
            .provisions
            .iter()
            .map(|provision| provision.id.rsplit(':').next().unwrap())
            .collect();
        assert_eq!(ids, ["228", "228-a", "228-a-bis", "228-b"]);
        assert!(!document.provisions[0].text.contains("CAPITULO"));
        assert!(
            document.provisions[1]
                .text
                .starts_with("Los certificados de participación")
        );
        // A body word after a space is still never a suffix.
        assert!(super::parse_article_start("Artículo 5 a la fecha.").is_none());
    }

    #[test]
    fn an_unread_article_heading_after_a_structural_heading_fails_closed() {
        let source = "   Artículo 1.- Primero.\n\n                CAPÍTULO II\n\n   Artículo 2 zz.- Ilegible.\n";
        let error = parse_diputados(
            source,
            &options("urn:lex-mx:federal:statute:sample", "Ley de Muestra"),
            NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(),
        )
        .unwrap_err();
        assert!(
            error.to_string().contains("unread article heading"),
            "{error}"
        );
    }

    #[test]
    fn a_lower_case_transitory_heading_fails_closed_until_a_source_needs_it() {
        // Codex review 2026-10-19 (P2): requiring a capitalized ordinal keeps
        // `décimo. Asimismo…` (cpf) a continuation, but it also refuses a
        // genuinely lower-case heading. No retained source has one (309-source
        // A/B census); if one arrives, discriminate by context instead.
        let source = |ordinal: &str| {
            format!(
                "   Artículo 1.- Texto.\n\n                TRANSITORIOS\n\n   {ordinal}.- El decreto entrará en vigor mañana.\n"
            )
        };
        let parse = |text: &str| {
            parse_diputados(
                text,
                &options("urn:lex-mx:federal:statute:sample", "Ley de Muestra"),
                NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(),
            )
        };
        let capitalized = parse(&source("Primero")).unwrap();
        assert!(
            capitalized
                .provisions
                .iter()
                .any(|p| p.id.ends_with(":transitory:primero"))
        );
        let error = parse(&source("primero")).unwrap_err();
        assert!(
            error.to_string().contains("unnumbered transitory body"),
            "{error}"
        );
    }

    #[test]
    fn omnibus_operative_articles_and_noted_titles_close_a_transitory_section() {
        // cff: after one law's transitories (`Cuarto.`), the 1995 omnibus
        // decree goes on with `Artículo Cuarto.`, an operative article; and a
        // Miscelánea resolution's title, after a signature and a page break,
        // was merged into the previous decree. Both gave duplicate IDs.
        let evidence = super::extract_reform_evidence(
            REFORM_OMNIBUS_OPERATIVE_ARTICLE_AND_RESOLUTION_FIXTURE,
            &options(
                "urn:lex-mx:federal:code:cff",
                "Código Fiscal de la Federación",
            ),
        )
        .expect("no duplicate reform evidence");
        let ids: Vec<&str> = evidence
            .iter()
            .map(|item| item.provision_id.rsplit(":amendment:").next().unwrap())
            .collect();
        assert_eq!(
            ids,
            [
                "1995-12-15:transitory:primero",
                "1995-12-15:transitory:cuarto",
                "1995-12-15:section-2:transitory:primero",
                "2000-12-31:transitory:primero",
                "2000-12-31:transitory:segundo",
                "2002-02-12:transitory:unico",
            ]
        );
        assert!(!evidence[4].text.contains("Resolución"));
        assert!(evidence[5].label.contains("Resolución DOF 2002-02-12"));
    }

    #[test]
    fn reformas_titles_and_page_split_ordinals_are_read() {
        // cpf: `REFORMAS al Código Penal…` (1966) was not an act title, so its
        // transitory joined the 1965 decree; and `décimo. Asimismo, …`, the
        // tail of a sentence after a page break, was read as a transitory.
        let evidence = super::extract_reform_evidence(
            REFORM_REFORMAS_TITLE_AND_PAGE_SPLIT_ORDINAL_FIXTURE,
            &options("urn:lex-mx:federal:code:cpf", "Código Penal Federal"),
        )
        .expect("no duplicate reform evidence");
        let ids: Vec<&str> = evidence
            .iter()
            .map(|item| item.provision_id.rsplit(":amendment:").next().unwrap())
            .collect();
        assert_eq!(
            ids,
            [
                "1965-01-13:transitory:unico",
                "1966-01-14:transitory:unico",
                "2016-06-16:transitory:decimo-primero",
                "2016-06-16:transitory:decimo-segundo",
            ]
        );
        assert!(evidence[2].text.contains("décimo. Asimismo"));
    }

    #[test]
    fn the_instruments_own_title_inside_an_open_omnibus_decree_is_a_section() {
        // lieps: each omnibus decree heads its LIEPS part with the law's
        // title (unaccented), between the decree's publication note and its
        // transitories. Read as a new act, it dropped the decree's date.
        let evidence = super::extract_reform_evidence(
            REFORM_OMNIBUS_OWN_TITLE_SECTION_FIXTURE,
            &options(
                "urn:lex-mx:federal:statute:lieps",
                "Ley del Impuesto Especial sobre Producción y Servicios",
            ),
        )
        .expect("the section heading keeps its decree's publication date");
        let ids: Vec<&str> = evidence
            .iter()
            .map(|item| item.provision_id.as_str())
            .collect();
        assert_eq!(
            ids,
            [
                "urn:lex-mx:federal:statute:lieps:amendment:1993-12-03:transitory:primero",
                "urn:lex-mx:federal:statute:lieps:amendment:1994-12-28:transitory:unico",
            ]
        );
        // Another law's title in the same position is still a new act, as
        // before.
        assert!(
            super::extract_reform_evidence(
                REFORM_OMNIBUS_OWN_TITLE_SECTION_FIXTURE,
                &options("urn:lex-mx:federal:statute:sample", "Ley de Muestra"),
            )
            .is_err()
        );
    }

    #[test]
    fn reform_publication_citation_does_not_replace_containing_decree_date() {
        let evidence = super::extract_reform_evidence(
            REFORM_PUBLICATION_CITATION_FIXTURE,
            &options("urn:lex-mx:federal:statute:sample", "Ley de Muestra"),
        )
        .expect("publication citation in the operative article does not strand a transitory");

        assert_eq!(evidence.len(), 1);
        assert_eq!(
            evidence[0].provision_id,
            "urn:lex-mx:federal:statute:sample:amendment:2015-12-18:transitory:primero"
        );
    }

    #[test]
    fn reform_wrapped_title_may_start_with_article_before_publication_date() {
        let evidence = super::extract_reform_evidence(
            REFORM_WRAPPED_TITLE_ARTICLE_FIXTURE,
            &options("urn:lex-mx:federal:statute:sample", "Ley de Muestra"),
        )
        .expect("wrapped decree title preserves its publication date");

        assert_eq!(evidence.len(), 1);
        assert_eq!(
            evidence[0].provision_id,
            "urn:lex-mx:federal:statute:sample:amendment:2018-01-19:transitory:unico"
        );
        assert_eq!(
            evidence[0].text,
            "El presente Decreto entrará en vigor al día siguiente de su publicación."
        );
    }

    #[test]
    fn reform_regulation_headings_reset_dates_and_exclude_signatures() {
        let evidence = super::extract_reform_evidence(
            REFORM_REGULATION_HEADINGS_FIXTURE,
            &options(
                "urn:lex-mx:federal:regulation:sample",
                "Reglamento de Muestra",
            ),
        )
        .expect("regulation headings in reform appendix parse");

        let ids: Vec<&str> = evidence
            .iter()
            .map(|item| item.provision_id.as_str())
            .collect();
        assert_eq!(
            ids,
            [
                "urn:lex-mx:federal:regulation:sample:amendment:1998-02-04:transitory:primero",
                "urn:lex-mx:federal:regulation:sample:amendment:1999-08-09:transitory:primero",
                "urn:lex-mx:federal:regulation:sample:amendment:2004-12-28:transitory:primero",
            ]
        );
        assert!(
            evidence
                .iter()
                .all(|item| item.label.contains("Reglamento DOF"))
        );
        assert!(
            evidence
                .iter()
                .all(|item| !item.text.contains("Residencia"))
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

    #[test]
    fn letter_suffix_heading_keeps_its_qualifier() {
        // LIVA writes `Artículo 1o.-A BIS.-`. The suffix letter used to be
        // rejected because a space (not `.`/`-`) follows it, collapsing the
        // heading to `1o` and stranding `A BIS.-` in the body — which then
        // collided with the real article 1o as a duplicate canonical id.
        let document = parse_diputados(
            ARTICLE_LETTER_SUFFIX_QUALIFIER_FIXTURE,
            &options(
                "urn:lex-mx:federal:statute:muestra",
                "Ley de Muestra del Impuesto",
            ),
            NaiveDate::from_ymd_opt(1978, 12, 29).expect("valid date"),
        )
        .expect("fixture parses");
        let numbers: Vec<&str> = document
            .provisions
            .iter()
            .filter(|provision| provision.provision_type == ProvisionType::Article)
            .map(|provision| provision.number.as_str())
            .collect();
        assert_eq!(
            numbers,
            [
                "1o", "1o-A", "1o-A BIS", "1o-B", "26-L", "26-LL", "26-M", "26-Ñ", "2448-A"
            ]
        );
    }
    #[test]
    fn campaign_signatures_end_transitories_without_adapter_stop_markers() {
        let document = parse_diputados(
            ORIGINAL_TRANSITORY_SIGNATURE_FIXTURE,
            &options(
                "urn:lex-mx:federal:statute:sample",
                "Ley del Diario Oficial de la Federación y Gacetas Gubernamentales",
            ),
            NaiveDate::from_ymd_opt(1986, 12, 24).unwrap(),
        )
        .unwrap();
        assert_eq!(
            document.provisions.last().unwrap().text,
            "Se derogan las disposiciones que se opongan a la presente ley."
        );
        assert_eq!(document.reform_evidence.len(), 1);
        assert!(!super::is_enactment_signature(
            "La comisión en México, D.F., examinará las rúbricas."
        ));
        assert!(!super::is_enactment_signature(
            "El aviso citará: En cumplimiento de lo dispuesto por la fracción I del Artículo 89, sin alterar la obligación."
        ));
    }

    #[test]
    fn publisher_adjacent_wrapped_title_is_removed_but_substantive_titles_survive() {
        let source = "LEY DE PRUEBA\nTÍTULO LARGO\nCÁMARA DE DIPUTADOS DEL H. CONGRESO DE LA UNIÓN\nSecretaría General\n\nArtículo 1. Texto.\n\nREGLAMENTO INTERNO\n\nSOBRE ATRIBUCIONES\n\n2 de 3\nLEY DE PRUEBA\nTÍTULO LARGO\nCÁMARA DE DIPUTADOS DEL H. CONGRESO DE LA UNIÓN\nSecretaría General\n\nque continúa.\n\nArtículo 2. Fin.";
        let doc = parse_diputados(
            source,
            &options(
                "urn:lex-mx:federal:statute:sample",
                "Ley de Prueba Título Largo",
            ),
            NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(),
        )
        .unwrap();
        assert_eq!(doc.provisions.len(), 2);
        assert!(doc.provisions[0].text.contains("REGLAMENTO INTERNO"));
        assert!(doc.provisions[0].text.contains("SOBRE ATRIBUCIONES"));
        assert!(!doc.provisions[0].text.contains("TÍTULO LARGO"));
        assert!(doc.provisions[0].text.contains("que continúa."));
    }

    #[test]
    fn ordinal_repealed_article_is_not_a_wrapper() {
        let doc = parse_diputados(
            include_str!("../../../fixtures/diputados/repair-lcnp-excerpt.txt"),
            &options("urn:lex-mx:federal:statute:sample", "Ley de Muestra"),
            NaiveDate::from_ymd_opt(1939, 11, 10).unwrap(),
        )
        .unwrap();
        assert_eq!(
            doc.provisions
                .iter()
                .map(|p| p.number.as_str())
                .collect::<Vec<_>>(),
            ["1o", "2o", "3o", "4o"]
        );
        assert!(doc.provisions[1].text.starts_with("Se deroga."));
    }

    #[test]
    fn explicit_approval_and_creation_wrappers_do_not_duplicate_article_one() {
        for source in [
            include_str!("../../../fixtures/diputados/repair-lfdefp-excerpt.txt"),
            include_str!("../../../fixtures/diputados/repair-lgdlpi-excerpt.txt"),
        ] {
            let doc = parse_diputados(
                source,
                &options("urn:lex-mx:federal:statute:sample", "Ley de Muestra"),
                NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(),
            )
            .unwrap();
            assert_eq!(
                doc.provisions
                    .iter()
                    .filter(|p| p.id.ends_with(":article:1"))
                    .count(),
                1
            );
            assert!(!doc.provisions[0].text.starts_with("Se crea la Ley"));
            assert!(!doc.provisions[0].text.starts_with("Se aprueba la Ley"));
        }
    }

    #[test]
    fn a_single_extracted_dot_before_article_heading_preserves_article_forty() {
        let doc = parse_diputados(
            include_str!("../../../fixtures/diputados/repair-reg-lfcine-excerpt.txt"),
            &options(
                "urn:lex-mx:federal:regulation:sample",
                "Reglamento de Muestra",
            ),
            NaiveDate::from_ymd_opt(2001, 3, 29).unwrap(),
        )
        .unwrap();
        assert_eq!(
            doc.provisions
                .iter()
                .map(|p| p.number.as_str())
                .collect::<Vec<_>>(),
            ["39", "40"]
        );
        assert!(doc.provisions[1].text.starts_with("Al momento"));
    }

    #[test]
    fn publisher_publication_note_typos_preserve_real_dates() {
        for (source, expected) in [
            (
                include_str!("../../../fixtures/diputados/repair-reg-lfafe-excerpt.txt"),
                (1972, 5, 6),
            ),
            (
                include_str!("../../../fixtures/diputados/repair-louaaan-excerpt.txt"),
                (2006, 4, 26),
            ),
            (
                include_str!("../../../fixtures/diputados/repair-lgdnna-excerpt.txt"),
                (2018, 6, 20),
            ),
        ] {
            assert_eq!(
                super::extract_dof_publication(source),
                NaiveDate::from_ymd_opt(expected.0, expected.1, expected.2)
            );
        }
        assert!(
            super::extract_dof_publication("El contrato surtirá efectos el 6 de mayo de 1972.")
                .is_none()
        );
    }

    #[test]
    fn judicial_dispositions_are_not_reform_transitories() {
        let source = format!(
            "ARTÍCULOS TRANSITORIOS DE DECRETOS DE REFORMA\n\nDECRETO por el que se reforma la Ley.\nPublicado en el Diario Oficial de la Federación el 9 de septiembre de 2022\n\nTRANSITORIOS\n\nSEGUNDO. Texto legal.\n\nMéxico, D.F., a 8 de septiembre de 2022.- Dip. Persona.- Rúbrica.\n1 de 2\nLEY DE PRUEBA\nCÁMARA DE DIPUTADOS DEL H. CONGRESO DE LA UNIÓN\nSecretaría General\n\n{}",
            include_str!("../../../fixtures/diputados/repair-larefam-excerpt.txt")
        );
        let evidence = super::extract_reform_evidence(
            &source,
            &options("urn:lex-mx:federal:statute:sample", "Ley de Prueba"),
        )
        .unwrap();
        assert_eq!(evidence.len(), 1);
        assert_eq!(evidence[0].text, "Texto legal.");
        let source = format!(
            "{source}\nDECRETO por el que se reforma la Ley.\nPublicado en el Diario Oficial de la Federación el 16 de julio de 2025\n\nTRANSITORIOS\n\nÚnico. Otra reforma."
        );
        let evidence = super::extract_reform_evidence(
            &source,
            &options("urn:lex-mx:federal:statute:sample", "Ley de Prueba"),
        )
        .unwrap();
        assert_eq!(evidence.len(), 2);
        assert_eq!(evidence[1].text, "Otra reforma.");
        assert!(evidence[1].provision_id.contains("2025-07-16"));
    }
    #[test]
    fn old_cjm_numeric_transitories_end_before_presidential_order() {
        let doc = parse_diputados(
            include_str!("../../../fixtures/diputados/repair-cjm-excerpt.txt"),
            &options(
                "urn:lex-mx:federal:code:sample",
                "Código de Justicia Militar",
            ),
            NaiveDate::from_ymd_opt(1933, 8, 31).unwrap(),
        )
        .unwrap();
        assert_eq!(
            doc.provisions
                .iter()
                .map(|p| p.number.as_str())
                .collect::<Vec<_>>(),
            ["1o", "2o", "3o", "4o"]
        );
        assert!(doc.provisions[3].text.ends_with("éste se concedan."));
        assert!(!doc.provisions[3].text.contains("Por tanto"));
        assert!(!doc.provisions[3].text.contains("Al C...."));
    }
    #[test]
    fn military_slash_ordinals_and_thousands_do_not_disappear_or_regress() {
        for (source, numbers) in [
            (
                include_str!("../../../fixtures/diputados/repair-loefam-excerpt.txt"),
                vec!["1/o", "1/o BIS"],
            ),
            (
                include_str!("../../../fixtures/diputados/repair-oga-excerpt.txt"),
                vec!["999", "1,000", "1,001", "1,002"],
            ),
        ] {
            let doc = parse_diputados(
                source,
                &options("urn:lex-mx:federal:statute:sample", "Ley de Prueba"),
                NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(),
            )
            .unwrap();
            assert_eq!(
                doc.provisions
                    .iter()
                    .map(|p| p.number.as_str())
                    .collect::<Vec<_>>(),
                numbers
            );
            assert_eq!(
                doc.provisions[1].id,
                if numbers[0] == "1/o" {
                    "urn:lex-mx:federal:statute:sample:article:1-bis"
                } else {
                    "urn:lex-mx:federal:statute:sample:article:1000"
                }
            );
        }
    }

    #[test]
    fn publisher_status_note_is_removed_only_at_publisher_header() {
        let source = "Artículo 1. Texto.\n\n1 de 2\nLEY DE SEGURIDAD INTERIOR\nCÁMARA DE DIPUTADOS DEL H. CONGRESO DE LA UNIÓN\nSecretaría General   Declaración de Invalidez de la Ley 15-11-2018 / DOF 30-05-2019\nSecretaría de Servicios Parlamentarios\n\ncontinuación.\n\nArtículo 2. Declaración de Invalidez de la Ley 15-11-2018 / DOF 30-05-2019 citada por la disposición.";
        let doc = parse_diputados(
            source,
            &options(
                "urn:lex-mx:federal:statute:sample",
                "Ley de Seguridad Interior",
            ),
            NaiveDate::from_ymd_opt(2017, 12, 21).unwrap(),
        )
        .unwrap();
        assert!(!doc.provisions[0].text.contains("Declaración"));
        assert!(doc.provisions[1].text.contains("Declaración de Invalidez"));
    }
    #[test]
    fn original_publication_range_never_falls_through_to_a_reform_date() {
        assert!(
            super::extract_dof_publication(include_str!(
                "../../../fixtures/diputados/repair-oga-publication-range.txt"
            ))
            .is_none()
        );
        let source = "Artículo 1. Texto.\n\nTRANSITORIO\n\nLa presente Ordenanza comenzará a surtir efectos.";
        let error = parse_diputados(
            source,
            &options("urn:lex-mx:federal:statute:sample", "Ordenanza de Muestra"),
            NaiveDate::from_ymd_opt(1912, 1, 1).unwrap(),
        )
        .unwrap_err();
        assert!(error.to_string().contains("no verified closing boundary"));
    }

    #[test]
    fn period_section_heading_and_roman_transitories_preserve_source_labels() {
        let opts = options(
            "urn:lex-mx:federal:regulation:sample",
            "Reglamento de Muestra",
        );
        let source = include_str!("../../../fixtures/diputados/repair-reg-lfafe-transitories.txt");
        let doc =
            parse_diputados(source, &opts, NaiveDate::from_ymd_opt(1972, 5, 6).unwrap()).unwrap();
        assert_eq!(
            doc.provisions
                .iter()
                .filter(|p| p.provision_type == ProvisionType::Transitory)
                .count(),
            8
        );
        assert_eq!(doc.provisions[1].number, "PRIMERO");
        let source = include_str!("../../../fixtures/diputados/repair-reg-lsm-transitories.txt");
        let doc = parse_diputados(
            source,
            &opts,
            NaiveDate::from_ymd_opt(1942, 11, 10).unwrap(),
        )
        .unwrap();
        assert_eq!(
            doc.provisions
                .iter()
                .map(|p| p.number.as_str())
                .collect::<Vec<_>>(),
            ["I", "II", "III", "IV", "V"]
        );
        let nested = "Artículo 1. Texto.\n\nTRANSITORIOS\n\nPRIMERO. Se observará:\n\nI.- Primera fracción.\n\nII.- Segunda fracción.\n\nSEGUNDO. Fin.";
        let doc =
            parse_diputados(nested, &opts, NaiveDate::from_ymd_opt(2000, 1, 1).unwrap()).unwrap();
        assert_eq!(doc.provisions.len(), 3);
        assert!(doc.provisions[1].text.contains("II.- Segunda fracción."));
    }
    #[test]
    fn lowercase_wrapped_transitorio_is_body_not_a_second_section() {
        let source = format!(
            "TRANSITORIOS:\n\nNOVENO. {}",
            include_str!("../../../fixtures/diputados/repair-reg-lfsv-continuation.txt")
        );
        let doc = parse_diputados(
            &source,
            &options(
                "urn:lex-mx:federal:regulation:sample",
                "Reglamento de Muestra",
            ),
            NaiveDate::from_ymd_opt(2016, 7, 15).unwrap(),
        )
        .unwrap();
        assert_eq!(
            doc.provisions
                .iter()
                .map(|p| p.number.as_str())
                .collect::<Vec<_>>(),
            ["NOVENO", "DÉCIMO", "DÉCIMO PRIMERO"]
        );
        assert!(
            doc.provisions[0]
                .text
                .ends_with("fracción III de este transitorio.")
        );
        assert!(super::is_transitory_section_header("Transitorios."));
        assert!(super::is_transitory_section_header("TRANSITORIO."));
        assert!(!super::is_transitory_section_header("transitorio."));
    }
    #[test]
    fn explicit_layer_partition_preserves_law_decree_and_elided_siblings() {
        for (raw, own, decree, elided) in [
            (
                include_str!("../../../fixtures/diputados/followup-lmigra.txt"),
                10,
                4,
                1,
            ),
            (
                include_str!("../../../fixtures/diputados/followup-reg-lmigra.txt"),
                0,
                13,
                2,
            ),
        ] {
            let mut opts = options("urn:lex-mx:federal:statute:sample", "Ley de Migración");
            opts.separate_enactment = true;
            let doc =
                parse_diputados(raw, &opts, NaiveDate::from_ymd_opt(2011, 5, 25).unwrap()).unwrap();
            assert_eq!(
                doc.provisions
                    .iter()
                    .filter(|p| p.provision_type == ProvisionType::Transitory)
                    .count(),
                own
            );
            assert_eq!(
                doc.enactment_evidence
                    .iter()
                    .filter(|e| e.kind == lex_core::EnactmentEvidenceKind::Transitory)
                    .count(),
                decree
            );
            assert_eq!(doc.enactment_evidence.len(), decree + elided);
            assert!(
                doc.enactment_evidence
                    .iter()
                    .all(|e| e.text_sha256 == lex_core::evidence_sha256(&e.text))
            );
            assert!(
                doc.enactment_evidence
                    .iter()
                    .all(|e| e.containing_act_title.starts_with("DECRETO"))
            );
            assert!(
                doc.provisions
                    .iter()
                    .all(|p| !p.text.contains("ARTÍCULOS SEGUNDO A NOVENO"))
            );
            if own == 10 {
                assert!(!doc.reform_evidence.is_empty());
            }
        }
        let mut opts = options("urn:lex-mx:federal:statute:sample", "Ley de Prueba");
        opts.separate_enactment = true;
        assert!(
            parse_diputados(
                "Artículo 1. Texto.\nTRANSITORIOS\nPRIMERO. Texto.\nPRIMERO. Otra cosa.",
                &opts,
                NaiveDate::from_ymd_opt(2000, 1, 1).unwrap()
            )
            .is_err()
        );
    }

    #[test]
    fn sole_source_clause_keeps_printed_heading_and_explicit_identity_basis() {
        for (source, heading) in [
            (
                include_str!("../../../fixtures/diputados/followup-lrcdn.txt"),
                "ARTICULO TRANSITORIO:",
            ),
            (
                include_str!("../../../fixtures/diputados/followup-oga.txt"),
                "TRANSITORIO",
            ),
        ] {
            let doc = parse_diputados(
                source,
                &options("urn:lex-mx:federal:statute:sample", "Ley de Prueba"),
                NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(),
            )
            .unwrap();
            let item = doc.provisions.last().unwrap();
            assert_eq!(item.label, heading);
            assert_eq!(item.number, "UNICO");
            assert_eq!(
                item.numbering_basis,
                Some(lex_core::NumberingBasis::ImplicitSingleton)
            );
            assert!(!item.text.contains("UNICO"));
        }
        let mixed = "Artículo 1. Texto.\nTRANSITORIO\n\nCláusula sin número.\n\nPRIMERO. Otra cláusula.\n\nMéxico, D.F., a 1 de enero de 2000.- Rúbrica.";
        assert!(
            parse_diputados(
                mixed,
                &options("urn:lex-mx:federal:statute:sample", "Ley de Prueba"),
                NaiveDate::from_ymd_opt(2000, 1, 1).unwrap()
            )
            .unwrap_err()
            .to_string()
            .contains("mixed numbered")
        );
    }
    fn nested_options(slug: &str) -> DiputadosOptions {
        let mut opts = options(
            &format!("urn:lex-mx:federal:statute:{slug}"),
            "Unused header",
        );
        let (title, wrapper, start, end, transitory, headers) = match slug {
            "lcmopfih" => (
                "LEY QUE ESTABLECE, REFORMA, ADICIONA Y DEROGA DIVERSAS DISPOSICIONES FISCALES Y QUE REFORMA OTRAS LEYES FEDERALES",
                "ARTICULO VIGESIMO SEXTO.-", "\"LEY DE CONTRIBUCION DE MEJORAS",
                "ARTICULOS VIGESIMO SEPTIMO A TRIGESIMO TERCERO.-", "TRANSITORIOS",
                vec!["LEY DE CONTRIBUCIÓN DE MEJORAS POR OBRAS PÚBLICAS FEDERALES DE INFRAESTRUCTURA HIDRÁULICA".to_owned()]),
            "lisipl" => (
                "LEY QUE ESTABLECE, REFORMA Y ADICIONA LAS DISPOSICIONES RELATIVAS A DIVERSOS IMPUESTOS.",
                "ARTICULO NOVENO.-", "ARTICULO 1o.-", "ARTICULO DECIMO.-", "TRANSITORIOS.",
                vec!["IMPUESTO SOBRE SERVICIOS EXPRESAMENTE DECLARADOS DE INTERÉS PÚBLICO POR LEY, EN LOS QUE INTERVENGAN EMPRESAS CONCESIONARIAS DE".to_owned(), "BIENES DEL DOMINIO DIRECTO DE LA NACIÓN (LEY QUE ESTABLECE, REFORMA Y ADICIONA LAS DISPOSICIONES RELATIVAS A DIVERSOS IMPUESTOS)".to_owned()]),
            "lfiiedb" => (
                "SE EXPIDE LA LEY PARA EL FOMENTO DE LA INVERSIÓN EN INFRAESTRUCTURA ESTRATÉGICA PARA EL DESARROLLO CON BIENESTAR, Y SE REFORMAN Y ADICIONAN DIVERSAS DISPOSICIONES DE LA LEY FEDERAL DE PRESUPUESTO Y RESPONSABILIDAD HACENDARIA",
                "Artículo Primero.-", "TÍTULO PRIMERO",
                "Artículo Segundo.-", "Transitorios", vec![]),
            _ => unreachable!(),
        };
        opts.header_lines = headers;
        if slug == "lfiiedb" {
            opts.header_lines.extend(
                include_str!("../../../fixtures/diputados/held-nested-lfiiedb.txt")
                    .lines()
                    .map(str::trim)
                    .filter(|line| {
                        line.contains("DIARIO OFICIAL") && line.contains("Edición Vespertina")
                    })
                    .map(str::to_owned),
            );
        }
        opts.nested_instrument = Some(lex_core::NestedInstrumentProfile {
            containing_act_title: title.to_owned(),
            wrapper_marker: wrapper.to_owned(),
            body_start_marker: start.to_owned(),
            body_end_marker: end.to_owned(),
            outer_transitory_marker: transitory.to_owned(),
        });
        opts
    }

    fn assert_lisipl_source_headers_absent(doc: &super::DiputadosDocument) {
        assert!(doc.provisions.iter().all(|p| {
            !p.text
                .contains("IMPUESTO SOBRE SERVICIOS EXPRESAMENTE DECLARADOS")
                && !p
                    .text
                    .contains("BIENES DEL DOMINIO DIRECTO DE LA NACIÓN (LEY QUE ESTABLECE")
        }));
        let fifth = doc
            .enactment_evidence
            .iter()
            .find(|e| e.number == "QUINTO")
            .unwrap();
        assert!(fifth.text.ends_with("51% de unidades y permisos."));
        assert!(
            !fifth
                .text
                .contains("IMPUESTO SOBRE SERVICIOS EXPRESAMENTE DECLARADOS")
        );
        assert!(
            !fifth
                .text
                .contains("BIENES DEL DOMINIO DIRECTO DE LA NACIÓN (LEY QUE ESTABLECE")
        );
    }

    #[test]
    fn reviewed_nested_sources_separate_own_and_containing_act_provisions() {
        for (slug, raw, articles, transitories, outer) in [
            (
                "lcmopfih",
                include_str!("../../../fixtures/diputados/held-nested-lcmopfih.txt"),
                15,
                5,
                8,
            ),
            (
                "lisipl",
                include_str!("../../../fixtures/diputados/held-nested-lisipl.txt"),
                7,
                0,
                7,
            ),
            (
                "lfiiedb",
                include_str!("../../../fixtures/diputados/held-nested-lfiiedb.txt"),
                141,
                1,
                8,
            ),
        ] {
            let opts = nested_options(slug);
            let doc =
                parse_diputados(raw, &opts, NaiveDate::from_ymd_opt(2000, 1, 1).unwrap()).unwrap();
            assert_eq!(
                doc.provisions
                    .iter()
                    .filter(|p| p.provision_type == lex_core::ProvisionType::Article)
                    .count(),
                articles,
                "{slug}"
            );
            assert_eq!(
                doc.provisions
                    .iter()
                    .filter(|p| p.provision_type == lex_core::ProvisionType::Transitory)
                    .count(),
                transitories,
                "{slug}"
            );
            assert_eq!(
                doc.enactment_evidence
                    .iter()
                    .filter(|e| e.kind == lex_core::EnactmentEvidenceKind::Transitory)
                    .count(),
                outer,
                "{slug}"
            );
            assert_eq!(
                doc.enactment_evidence
                    .iter()
                    .filter(|e| e.kind == lex_core::EnactmentEvidenceKind::ContainingArticle)
                    .count(),
                2
            );
            assert!(
                doc.enactment_evidence
                    .iter()
                    .all(|e| e.owner == lex_core::EnactmentOwner::ContainingAct)
            );
            assert!(
                doc.provisions
                    .iter()
                    .all(|p| !p.text.contains("DIARIO OFICIAL")
                        && !p.text.contains("CÁMARA DE DIPUTADOS")),
                "page furniture in {slug}"
            );
            if slug == "lisipl" {
                assert_lisipl_source_headers_absent(&doc);
            }
            if slug == "lfiiedb" {
                assert!(
                    doc.provisions[0]
                        .heading_context
                        .title
                        .as_deref()
                        .is_some_and(|h| h.starts_with("Título PRIMERO"))
                );
                assert!(
                    doc.provisions[0]
                        .heading_context
                        .chapter
                        .as_deref()
                        .is_some_and(|h| h.starts_with("Capítulo ÚNICO"))
                );
                let sibling = doc
                    .enactment_evidence
                    .iter()
                    .find(|e| e.number == "following")
                    .unwrap();
                assert!(sibling.text.contains("Artículo 108.-"));
                assert!(!doc.provisions.last().unwrap().text.contains("Se reforman"));
                assert!(!doc.enactment_evidence.iter().any(|e| {
                    e.text
                        .contains("SE REFORMA EL ARTÍCULO 141 DEL CÓDIGO FISCAL")
                }));
            }
        }
    }

    #[test]
    fn nested_boundaries_fail_closed_on_missing_duplicate_reversed_and_wrong_title() {
        let raw = include_str!("../../../fixtures/diputados/held-nested-lcmopfih.txt");
        let opts = nested_options("lcmopfih");
        for altered in [
            raw.replace("ARTICULO VIGESIMO SEXTO.-", "ARTICULO AUSENTE.-"),
            format!("{raw}\n\nARTICULO VIGESIMO SEXTO.- Duplicado."),
            raw.replace("ARTICULO VIGESIMO SEXTO.-", "TEMPORARY")
                .replace(
                    "ARTICULOS VIGESIMO SEPTIMO A TRIGESIMO TERCERO.-",
                    "ARTICULO VIGESIMO SEXTO.-",
                )
                .replace(
                    "TEMPORARY",
                    "ARTICULOS VIGESIMO SEPTIMO A TRIGESIMO TERCERO.-",
                ),
            raw.replace(
                "Y QUE REFORMA OTRAS LEYES FEDERALES",
                "TITULO NO CORROBORADO",
            ),
        ] {
            assert!(
                parse_diputados(
                    &altered,
                    &opts,
                    NaiveDate::from_ymd_opt(2000, 1, 1).unwrap()
                )
                .is_err()
            );
        }
    }

    #[test]
    fn ascii_dot_only_sibling_preserves_lgag_law_and_outer_transitories() {
        let raw = include_str!("../../../fixtures/diputados/held-lgag-ascii-elision.txt");
        let mut opts = options("urn:lex-mx:federal:statute:lgag", "LEY GENERAL DE AGUAS");
        opts.separate_enactment = true;
        let doc =
            parse_diputados(raw, &opts, NaiveDate::from_ymd_opt(2025, 12, 11).unwrap()).unwrap();
        assert_eq!(
            doc.provisions
                .iter()
                .filter(|p| p.provision_type == lex_core::ProvisionType::Article)
                .count(),
            45
        );
        assert_eq!(
            doc.provisions
                .iter()
                .filter(|p| p.provision_type == lex_core::ProvisionType::Transitory)
                .count(),
            2
        );
        assert_eq!(doc.enactment_evidence.len(), 4);
        assert_eq!(
            doc.enactment_evidence
                .iter()
                .filter(|e| e.kind == lex_core::EnactmentEvidenceKind::Transitory)
                .count(),
            3
        );
        assert!(doc.enactment_evidence.iter().any(|e| e.kind
            == lex_core::EnactmentEvidenceKind::ElidedSiblingArticle
            && e.text.contains("..........")));
        let substantive = raw.replace(
            "Artículo Segundo.- ..........",
            "Artículo Segundo.- Se conserva contenido sustantivo.",
        );
        assert!(
            parse_diputados(
                &substantive,
                &opts,
                NaiveDate::from_ymd_opt(2025, 12, 11).unwrap()
            )
            .is_err()
        );
    }

    #[test]
    fn budget_signatures_do_not_leak_into_last_transitory_before_substantive_annex() {
        let raw = include_str!("../../../fixtures/diputados/held-pef-annex-boundary.txt");
        let mut opts = options(
            "urn:lex-mx:federal:decree:pef-2026",
            "PRESUPUESTO DE EGRESOS DE LA FEDERACIÓN PARA EL EJERCICIO FISCAL 2026",
        );
        opts.annex_markers = vec!["ANEXO 1. GASTO NETO TOTAL".to_owned()];
        let doc =
            parse_diputados(raw, &opts, NaiveDate::from_ymd_opt(2025, 11, 21).unwrap()).unwrap();
        assert_eq!(
            doc.provisions
                .iter()
                .filter(|p| p.provision_type == lex_core::ProvisionType::Article)
                .count(),
            1
        );
        assert_eq!(
            doc.provisions
                .iter()
                .filter(|p| p.provision_type == lex_core::ProvisionType::Transitory)
                .count(),
            16
        );
        let last = doc
            .provisions
            .iter()
            .rfind(|p| p.provision_type == lex_core::ProvisionType::Transitory)
            .unwrap();
        assert!(
            last.text
                .contains("Título Tercero Bis de la Ley General de Salud.")
        );
        assert!(
            !last.text.contains("SALÓN DE SESIONES")
                && !last.text.contains("En cumplimiento")
                && !last.text.contains("ANEXO 1")
        );
        let annexes: Vec<_> = doc
            .provisions
            .iter()
            .filter(|p| p.provision_type == lex_core::ProvisionType::Annex)
            .collect();
        assert_eq!(annexes.len(), 1);
        assert_eq!(annexes[0].label, "ANEXO 1. GASTO NETO TOTAL (pesos)");
        assert!(annexes[0].text.contains("RAMOS AUTÓNOMOS"));
        assert!(raw.contains("SALÓN DE SESIONES"));
    }

    #[test]
    fn configured_oag_appendix_with_dof_dates_preserves_all_ten_reform_sections() {
        let raw = include_str!("../../../fixtures/diputados/held-reg-insumos-reforms.txt");
        let mut opts = options(
            "urn:lex-mx:federal:regulation:reg-lgs-insumos",
            "REGLAMENTO DE INSUMOS PARA LA SALUD",
        );
        opts.header_lines.extend(
            [
                "OFICINA DEL ABOGADO GENERAL",
                "Última reforma D. O. F. 31-05-2021",
                "Publicado en el Diario Oficial de la Federación el 4 de febrero de 1998.",
            ]
            .map(str::to_owned),
        );
        // No blanket inference from a DECRETO heading before an explicit gate.
        assert!(
            super::extract_reform_evidence(raw, &opts)
                .unwrap()
                .is_empty()
        );
        opts.stop_markers = vec!["DECRETO que reforma, adiciona y deroga diversas disposiciones del Reglamento de Insumos".to_owned()];
        let evidence = super::extract_reform_evidence(raw, &opts).unwrap();
        let mut counts = std::collections::BTreeMap::new();
        for item in &evidence {
            *counts
                .entry(item.provision_id.split(":amendment:").nth(1).unwrap()[..10].to_owned())
                .or_insert(0usize) += 1;
            assert!(
                !item.text.contains("OFICINA DEL ABOGADO GENERAL")
                    && !item.text.contains("Última reforma D. O. F.")
                    && !item.text.contains(
                        "Publicado en el Diario Oficial de la Federación el 4 de febrero de 1998."
                    )
            );
            assert!(!item.text.contains("Rúbrica"));
        }
        assert_eq!(
            counts,
            std::collections::BTreeMap::from([
                ("2008-01-02".to_owned(), 8),
                ("2008-08-05".to_owned(), 3),
                ("2010-08-17".to_owned(), 1),
                ("2011-01-13".to_owned(), 3),
                ("2011-05-26".to_owned(), 1),
                ("2011-10-19".to_owned(), 3),
                ("2012-05-17".to_owned(), 1),
                ("2012-10-09".to_owned(), 2),
                ("2014-03-14".to_owned(), 1),
                ("2021-05-31".to_owned(), 6),
            ])
        );
        assert_eq!(evidence.len(), 29);
    }
    #[test]
    fn opaque_annex_keeps_quoted_reform_errata_and_judicial_text_without_fake_evidence() {
        let raw = include_str!("../../../fixtures/diputados/annex-quoted-reform-banner.txt");
        let mut opts = options("urn:lex-mx:federal:statute:sample", "LEY DE PRUEBA");
        opts.annex_markers = vec!["ANEXO 1.".to_owned()];
        opts.stop_markers = vec!["DECRETO por el que se reforma la ley al margen".to_owned()];
        let doc =
            parse_diputados(raw, &opts, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()).unwrap();
        assert_eq!(doc.provisions.len(), 3);
        let annex = &doc.provisions[2];
        assert_eq!(annex.provision_type, lex_core::ProvisionType::Annex);
        for text in [
            "Contenido inicial conservado.",
            "ARTÍCULOS TRANSITORIOS DE DECRETOS DE REFORMA",
            "Esta cláusula es una cita del cuadro",
            "La tabla cita DECRETO por el que se reforma la ley al margen",
            "FE DE ERRATAS",
            "PUNTOS RESOLUTIVOS de la sentencia",
            "CONTENIDO FINAL QUE DEBE CONSERVARSE.",
        ] {
            assert!(annex.text.contains(text), "lost annex text: {text}");
        }
        assert!(doc.reform_evidence.is_empty());
        assert!(
            super::extract_reform_evidence(raw, &opts)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn configured_reform_boundary_after_annex_ends_container_and_isolates_real_evidence() {
        let raw = include_str!("../../../fixtures/diputados/annex-quoted-reform-banner.txt");
        let raw = format!(
            "{raw}\nDECRETO por el que se reforma la ley al margen.\n\nPublicado en el DOF el 6 de junio de 2025\n\nTRANSITORIOS\n\nPRIMERO. Cláusula de la reforma real fuera del anexo.\n"
        );
        let mut opts = options("urn:lex-mx:federal:statute:sample", "LEY DE PRUEBA");
        opts.annex_markers = vec!["ANEXO 1.".to_owned()];
        opts.stop_markers = vec!["DECRETO por el que se reforma la ley al margen".to_owned()];
        let doc =
            parse_diputados(&raw, &opts, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()).unwrap();
        let annex = doc.provisions.last().unwrap();
        assert!(
            annex
                .text
                .ends_with("CONTENIDO FINAL QUE DEBE CONSERVARSE.")
        );
        assert!(!annex.text.contains("Cláusula de la reforma real"));
        assert_eq!(doc.reform_evidence.len(), 1);
        assert!(
            doc.reform_evidence[0]
                .provision_id
                .contains(":amendment:2025-06-06:")
        );
        assert_eq!(
            doc.reform_evidence[0].text,
            "Cláusula de la reforma real fuera del anexo."
        );
    }

    #[test]
    fn configured_main_appendix_without_annex_keeps_ordinary_ownership() {
        let raw = "Artículo 1. Texto propio.\n\nTRANSITORIOS\n\nPRIMERO. Cierre propio.\n\nDECRETO por el que se reforma la ley al margen.\n\nPublicado en el DOF el 6 de junio de 2025\n\nTRANSITORIOS\n\nPRIMERO. Cláusula de la reforma real.\n";
        let mut opts = options("urn:lex-mx:federal:statute:sample", "LEY DE PRUEBA");
        opts.stop_markers = vec!["DECRETO por el que se reforma la ley al margen".to_owned()];
        let doc =
            parse_diputados(raw, &opts, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()).unwrap();
        assert_eq!(doc.provisions.len(), 2);
        assert_eq!(doc.provisions[1].text, "Cierre propio.");
        assert_eq!(doc.reform_evidence.len(), 1);
        assert_eq!(doc.reform_evidence[0].text, "Cláusula de la reforma real.");
    }
}
