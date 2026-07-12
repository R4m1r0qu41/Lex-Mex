//! Parser for the January 28, 2021 disposiciones de carácter general
//! applicable to instituciones de fondos de pago electrónico (DCG-IFPE-2021).
//!
//! The operational CNBV PDF carries the índice, considerandos, seven
//! chapters, 59 articles, and four transitories. The eight annexes are
//! *not* included in that PDF's body; CNBV publishes each one as its own
//! PDF, linked from the "Ver más" panel of the instrument's row on the
//! Normatividad page (via the `NormatividadAjax.svc/ResolucionesYAnexos`
//! endpoint, `normaId=1036`). Each annex PDF is extracted and parsed the
//! same way as the main document. The main text and each annex are
//! extracted with page breaks preserved (`pdftotext -layout`): a paragraph
//! is merged across a page break unless the previous line ends a sentence
//! or enumeration (`.`, `:`, or `;`); annex PDFs have no page furniture
//! beyond an occasional bare page-number footer line, which is dropped.

use std::collections::BTreeSet;

use anyhow::{Context, Result, bail};
use chrono::NaiveDate;
use lex_core::{HeadingContext, Provision, ProvisionType, ReviewStatus, SCHEMA_VERSION};
use regex::Regex;

pub(crate) const TRANSITORY_ORDINALS: &[&str] = &[
    "PRIMERO", "SEGUNDO", "TERCERO", "CUARTO", "QUINTO", "SEXTO", "SÉPTIMO", "OCTAVO", "NOVENO",
    "DÉCIMO",
];

/// Column threshold that separates the term column from the definition
/// column in the Article 1 two-column layout. Terms indent at most 12
/// columns in the source; definition text never starts before column 20.
const DEFINITION_COLUMN_MIN: usize = 20;
const TERM_INDENT_MAX: usize = 12;

/// Parse the main CNBV PDF body plus one already-isolated document per
/// annex, each paired with its expected annex number (1-indexed, matching
/// CNBV's own `Orden`) for a cross-check against its own "ANEXO N" heading.
pub fn parse_dcg(
    main_raw: &str,
    annex_documents: &[(u32, String)],
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
    for (number, raw) in annex_documents {
        provisions.push(parse_annex_document(
            raw,
            *number,
            instrument_id,
            publication_date,
        )?);
    }
    if provisions.is_empty() {
        bail!("no DCG provisions recognized");
    }
    Ok(provisions)
}

pub(crate) struct HeadingState {
    pub(crate) title: Option<String>,
    pub(crate) chapter: Option<String>,
    pub(crate) section: Option<String>,
    pub(crate) apartado: Option<String>,
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
        title: None,
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

pub(crate) fn amendment_marker_regex() -> Result<Regex> {
    Ok(Regex::new(r"^\((\d{1,2})\)$")?)
}

/// Amendment-marker bookkeeping shared by every compiled-CNBV-document
/// parser (this module's own `parse_annex_document`, and `crate::itf`'s
/// main-document scanner). A margin marker prints as a standalone `(N)`
/// line at the vertical position of the text it annotates, so it can
/// arrive before a heading, before a body line, or between body lines;
/// callers hold it here until the next content line resolves where it
/// belongs, or until a context boundary makes clear it belongs nowhere.
#[derive(Default)]
pub(crate) struct PendingMarks {
    marks: Vec<u32>,
    /// True immediately after a marker line, so the very next blank line
    /// is recognized as the marker's own line spacing rather than a
    /// paragraph break. Any other line — including a page-number
    /// footer — clears this before it can leak across an intervening
    /// line to a blank line the marker was never adjacent to.
    swallow_next_blank: bool,
}

impl PendingMarks {
    /// Record a marker line.
    pub(crate) fn push(&mut self, marker: u32) {
        self.marks.push(marker);
        self.swallow_next_blank = true;
    }

    /// Whether a blank line right now is the marker's own spacing. Consumes
    /// the flag: only the one blank immediately after a marker is
    /// swallowed.
    pub(crate) fn take_swallow_next_blank(&mut self) -> bool {
        std::mem::take(&mut self.swallow_next_blank)
    }

    /// Any other non-blank, non-marker line severs the marker/blank
    /// adjacency the swallow behavior depends on.
    pub(crate) fn observe_content_line(&mut self) {
        self.swallow_next_blank = false;
    }

    /// Attach every pending marker onto `builder`.
    pub(crate) fn drain_onto(&mut self, builder: &mut DcgProvisionBuilder) {
        for marker in self.marks.drain(..) {
            builder.mark(marker);
        }
    }

    /// Take every pending marker for a receiver that is not a
    /// `DcgProvisionBuilder` (a reform transitory that becomes
    /// `TemporalEvidence`). Clears the buffer like `drain_onto`.
    pub(crate) fn take(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.marks)
    }

    /// Discard pending markers at a context boundary with no provision to
    /// receive them (an attribution block, a per-resolution transitory, a
    /// considerando, or the legend). Errors instead of silently losing
    /// provenance if a marker actually reached this boundary — that would
    /// mean a real document exercises a case this parser does not yet
    /// attribute correctly, which needs a human look rather than a silent
    /// drop.
    pub(crate) fn discard(&mut self, context: &str) -> Result<()> {
        if self.marks.is_empty() {
            return Ok(());
        }
        bail!(
            "amendment marker(s) {:?} appear where no provision can receive them ({context})",
            std::mem::take(&mut self.marks)
        );
    }

    /// Discard pending markers at a structural heading (título, capítulo,
    /// sección, apartado). Unlike [`Self::discard`], this never errors: a
    /// compiled document can legitimately mark an entire heading as
    /// repealed (for example `Apartado D` followed by a lone
    /// `(Derogado)`), and `HeadingContext` has no field to receive a mark
    /// even if it wanted to — but that fact is always redundant with the
    /// same marker already recorded directly on each provision the
    /// heading covers, so nothing is lost by discarding it here.
    pub(crate) fn discard_from_heading(&mut self) {
        self.marks.clear();
    }
}

fn new_transitory_regex() -> Result<Regex> {
    Ok(Regex::new(&format!(
        r"^({})\.-\s*(.*)$",
        TRANSITORY_ORDINALS.join("|")
    ))?)
}

pub(crate) fn flush(
    current: &mut Option<DcgProvisionBuilder>,
    provisions: &mut Vec<Provision>,
    publication_date: NaiveDate,
) {
    if let Some(builder) = current.take() {
        provisions.push(builder.finish(publication_date));
    }
}

/// Parse one annex's own dedicated CNBV PDF text. The first non-blank,
/// non-page-number line must be its "ANEXO N" / "Anexo N" heading, which
/// becomes the label; `expected_number` cross-checks that heading against
/// the number CNBV's webservice associated with this document's URL. Every
/// following line, including the subtitle, becomes body text using the same
/// paragraph accumulation and page-break merging as an article: a bare
/// 1-3 digit line is a page-number footer and is dropped without affecting
/// paragraph boundaries.
pub(crate) fn parse_annex_document(
    raw: &str,
    expected_number: u32,
    instrument_id: &str,
    publication_date: NaiveDate,
) -> Result<Provision> {
    // A landscape-format annex can print its margin marker on the heading
    // line itself (`ANEXO 14        (2)`).
    let heading_re = Regex::new(r"(?i)^anexo\s+(\d+)(?:\s+\((\d{1,2})\))?$")?;
    let page_number_re = Regex::new(r"^\d{1,3}$")?;
    let marker_re = amendment_marker_regex()?;
    let mut builder: Option<DcgProvisionBuilder> = None;
    let mut pending_blank = false;
    let mut crossed_page_break = false;
    let mut pending_marks = PendingMarks::default();

    for source_line in raw.lines() {
        if source_line.starts_with('\u{c}') {
            crossed_page_break = true;
        }
        let line = source_line.trim_start_matches('\u{c}');
        let trimmed = line.trim();
        if trimmed.is_empty() {
            // A blank immediately following a margin marker is part of the
            // marker's own line box, not a paragraph boundary.
            if !pending_marks.take_swallow_next_blank() {
                pending_blank = true;
            }
            continue;
        }
        // A page-number footer is invisible to paragraph flow, but it is
        // still a distinct line: it must not let a marker's "swallow the
        // next blank" carry across it to a blank line the marker was
        // never actually adjacent to.
        if page_number_re.is_match(trimmed) {
            pending_marks.observe_content_line();
            continue;
        }
        if let Some(captures) = marker_re.captures(trimmed) {
            pending_marks.push(captures[1].parse().expect("two-digit marker"));
            continue;
        }
        pending_marks.observe_content_line();
        if builder.is_none() {
            let captures = heading_re.captures(trimmed).with_context(|| {
                format!("annex {expected_number} does not start with an ANEXO heading")
            })?;
            let found_number: u32 = captures[1]
                .parse()
                .context("annex heading number is not numeric")?;
            if found_number != expected_number {
                bail!("expected annex {expected_number}, found heading for annex {found_number}");
            }
            let mut annex = DcgProvisionBuilder::annex(
                instrument_id,
                expected_number.to_string(),
                captures
                    .get(0)
                    .expect("full heading")
                    .as_str()
                    .split_whitespace()
                    .take(2)
                    .collect::<Vec<_>>()
                    .join(" "),
            );
            if let Some(inline) = captures.get(2) {
                annex.mark(inline.as_str().parse().expect("two-digit marker"));
            }
            pending_marks.drain_onto(&mut annex);
            builder = Some(annex);
            (pending_blank, crossed_page_break) = (false, false);
            continue;
        }
        if let Some(b) = &mut builder {
            pending_marks.drain_onto(b);
            b.push_line(line, pending_blank, crossed_page_break);
        }
        (pending_blank, crossed_page_break) = (false, false);
    }
    let builder =
        builder.with_context(|| format!("annex {expected_number} has no recognizable heading"))?;
    pending_marks.discard("the end of the annex document")?;
    Ok(builder.finish(publication_date))
}

pub(crate) struct DcgProvisionBuilder {
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
    /// Amendment markers printed in the compiled document's margin within
    /// this provision, deduplicated and ordered.
    amendment_marks: BTreeSet<u32>,
}

struct RawLine {
    text: String,
    after_blank: bool,
    after_page_break: bool,
}

impl DcgProvisionBuilder {
    pub(crate) fn article(
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
                libro: None,
                title: headings.title.clone(),
                chapter: headings.chapter.clone(),
                section: headings.section.clone(),
                apartado: headings.apartado.clone(),
            },
            paragraphs: Vec::new(),
            current_paragraph: Vec::new(),
            definition_layout,
            raw_lines: Vec::new(),
            amendment_marks: BTreeSet::new(),
        };
        builder.push_line(initial, false, false);
        builder
    }

    pub(crate) fn annex(instrument_id: &str, number: String, label: String) -> Self {
        Self {
            instrument_id: instrument_id.to_owned(),
            provision_type: ProvisionType::Annex,
            label,
            number,
            heading_context: HeadingContext {
                libro: None,
                title: None,
                chapter: None,
                section: None,
                apartado: None,
            },
            paragraphs: Vec::new(),
            current_paragraph: Vec::new(),
            definition_layout: false,
            raw_lines: Vec::new(),
            amendment_marks: BTreeSet::new(),
        }
    }

    pub(crate) fn transitory(instrument_id: &str, ordinal: &str, initial: &str) -> Self {
        let mut builder = Self {
            instrument_id: instrument_id.to_owned(),
            provision_type: ProvisionType::Transitory,
            label: ordinal.to_owned(),
            number: ordinal.to_owned(),
            heading_context: HeadingContext {
                libro: None,
                title: None,
                chapter: None,
                section: None,
                apartado: None,
            },
            paragraphs: Vec::new(),
            current_paragraph: Vec::new(),
            definition_layout: false,
            raw_lines: Vec::new(),
            amendment_marks: BTreeSet::new(),
        };
        builder.push_line(initial, false, false);
        builder
    }

    pub(crate) fn mark(&mut self, marker: u32) {
        self.amendment_marks.insert(marker);
    }

    pub(crate) fn push_line(&mut self, line: &str, after_blank: bool, after_page_break: bool) {
        if self.definition_layout {
            self.raw_lines.push(RawLine {
                text: line.to_owned(),
                after_blank,
                after_page_break,
            });
            return;
        }
        // A page break is a potential paragraph boundary: the paragraph
        // continues only when the previous line ends mid-sentence. A blank
        // line elsewhere always ends the paragraph.
        let continues_paragraph = if after_page_break {
            !ends_paragraph(self.current_paragraph.last())
        } else {
            !after_blank
        };
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
            self.paragraphs
                .push(collapse_whitespace(&self.current_paragraph.join(" ")));
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
            // Multi-word numbers (`15 Bis 1`) canonicalize with hyphens,
            // matching the LRITF builder and the reference extractor.
            ProvisionType::Article => ("article", self.number.to_lowercase().replace(' ', "-")),
            ProvisionType::Transitory => ("transitory", slug(&self.number)),
            ProvisionType::Annex => ("annex", self.number.to_lowercase()),
        };
        let text = self.paragraphs.join("\n\n");
        Provision {
            schema_version: SCHEMA_VERSION.to_owned(),
            id: format!("{}:{kind}:{canonical_number}", self.instrument_id),
            instrument_id: self.instrument_id,
            provision_type: self.provision_type,
            label: self.label,
            number: self.number,
            heading_context: self.heading_context,
            text: text.clone(),
            publication_date,
            effective_from: None,
            effective_to: None,
            temporal_status: crate::initial_temporal_status(&text),
            temporal_basis: None,
            temporal_confidence: None,
            review_status: ReviewStatus::NotAnalyzed,
            transitory_effects: Vec::new(),
            amendment_marks: self.amendment_marks.into_iter().collect(),
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
                    paragraphs.push(collapse_whitespace(&intro.join(" ")));
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
        paragraphs.push(collapse_whitespace(&intro.join(" ")));
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
        self.term_fragments.push(collapse_whitespace(fragment));
    }

    fn push_definition(&mut self, fragment: &str, paragraph_break: bool) {
        if paragraph_break || self.definition_paragraphs.is_empty() {
            self.definition_paragraphs.push(Vec::new());
        }
        self.definition_paragraphs
            .last_mut()
            .expect("definition paragraph exists")
            .push(collapse_whitespace(fragment));
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

fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
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

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use lex_core::ProvisionType;
    use pretty_assertions::assert_eq;

    use super::parse_dcg;

    const MAIN_FIXTURE: &str = include_str!("../../../fixtures/ifpe-dcg-2021/parser-sample.txt");
    const ANNEX_1_FIXTURE: &str =
        include_str!("../../../fixtures/ifpe-dcg-2021/annex-1-sample.txt");
    const ANNEX_8_FIXTURE: &str =
        include_str!("../../../fixtures/ifpe-dcg-2021/annex-8-sample.txt");
    const INSTRUMENT_ID: &str = "urn:lex-mx:federal:regulation:ifpe-dcg-2021";

    fn parse_fixture() -> Vec<lex_core::Provision> {
        parse_dcg(
            MAIN_FIXTURE,
            &[
                (1, ANNEX_1_FIXTURE.to_owned()),
                (8, ANNEX_8_FIXTURE.to_owned()),
            ],
            INSTRUMENT_ID,
            NaiveDate::from_ymd_opt(2021, 1, 28).unwrap(),
            &["1".to_owned()],
        )
        .unwrap()
    }

    #[test]
    fn parses_articles_transitories_and_annexes_with_hierarchy() {
        let provisions = parse_fixture();
        let articles: Vec<_> = provisions
            .iter()
            .filter(|item| item.provision_type == ProvisionType::Article)
            .collect();
        let transitories: Vec<_> = provisions
            .iter()
            .filter(|item| item.provision_type == ProvisionType::Transitory)
            .collect();
        let annexes: Vec<_> = provisions
            .iter()
            .filter(|item| item.provision_type == ProvisionType::Annex)
            .collect();

        assert_eq!(
            articles
                .iter()
                .map(|item| item.number.as_str())
                .collect::<Vec<_>>(),
            ["1", "2", "17", "36", "42"]
        );
        assert_eq!(
            transitories
                .iter()
                .map(|item| item.id.rsplit(':').next().unwrap())
                .collect::<Vec<_>>(),
            ["primero", "segundo", "tercero", "cuarto"]
        );
        assert_eq!(
            annexes
                .iter()
                .map(|item| item.label.as_str())
                .collect::<Vec<_>>(),
            ["ANEXO 1", "Anexo 8"]
        );

        // Chapter II hierarchy: section and apartado context on Article 2.
        let article_2 = &articles[1];
        assert_eq!(
            article_2.heading_context.chapter.as_deref(),
            Some("Capítulo II")
        );
        assert_eq!(
            article_2.heading_context.section.as_deref(),
            Some("Sección Primera")
        );
        assert_eq!(
            article_2.heading_context.apartado.as_deref(),
            Some("Apartado A")
        );
    }

    #[test]
    fn parses_article_17_heading_variant_without_dot() {
        let provisions = parse_fixture();
        let article_17 = provisions
            .iter()
            .find(|item| item.id.ends_with(":article:17"))
            .expect("article 17 parsed");
        assert_eq!(article_17.label, "Artículo 17");
        assert!(
            article_17
                .text
                .starts_with("Las instituciones de fondos de pago electrónico")
        );
    }

    #[test]
    fn reconstructs_article_1_definition_layout() {
        let provisions = parse_fixture();
        let article_1 = provisions
            .iter()
            .find(|item| item.id.ends_with(":article:1"))
            .expect("article 1 parsed");
        // Term and definition columns are re-associated, not interleaved.
        assert!(article_1.text.contains(
            "Administrador de Comisionistas: a la persona que, en términos del artículo 46"
        ));
        // A term wrapped over four source lines is joined.
        assert!(article_1.text.contains(
            "Política Estratégica de Continuidad de Negocio y de Seguridad de la Información: \
             al documento"
        ));
        // A definition that crosses a page break stays one entry.
        assert!(
            article_1
                .text
                .contains("del Banco de México, en moneda extranjera, objeto de una")
        );
        // The page-shifted UDI block is still recognized as one entry.
        assert!(
            article_1
                .text
                .contains("UDI: a las unidades de cuenta llamadas")
        );
    }

    #[test]
    fn merges_paragraphs_across_page_breaks_only_mid_sentence() {
        let provisions = parse_fixture();
        let article_36 = provisions
            .iter()
            .find(|item| item.id.ends_with(":article:36"))
            .expect("article 36 parsed");
        // Page break after a completed sentence keeps the paragraph break.
        assert!(
            article_36
                .text
                .contains("fecha de inicio y de fin de la asignación.\n\nV.")
        );
        let article_42 = provisions
            .iter()
            .find(|item| item.id.ends_with(":article:42"))
            .expect("article 42 parsed");
        // Mid-sentence page break is merged into one paragraph.
        assert!(
            article_42
                .text
                .contains("imágenes de identificaciones oficiales e información")
        );
    }

    #[test]
    fn parses_transitory_boundaries_and_annex_tables() {
        let provisions = parse_fixture();
        let cuarto = provisions
            .iter()
            .find(|item| item.id.ends_with(":transitory:cuarto"))
            .expect("transitory cuarto parsed");
        assert!(cuarto.text.starts_with("Las personas a que se refiere"));
        assert!(cuarto.text.ends_with("presentes Disposiciones."));
        let primero = provisions
            .iter()
            .find(|item| item.id.ends_with(":transitory:primero"))
            .expect("transitory primero parsed");
        assert!(primero.text.contains("noventa días naturales"));
        // The signature block is not part of any transitory.
        assert!(!cuarto.text.contains("Ciudad de México"));

        let annex_1 = provisions
            .iter()
            .find(|item| item.id.ends_with(":annex:1"))
            .expect("annex 1 parsed");
        assert_eq!(annex_1.label, "ANEXO 1");
        // The enumerated intro list is preserved verbatim.
        assert!(
            annex_1
                .text
                .contains("5. En caso de que no apliquen todos los supuestos")
        );
        // The bare page-number footer ("1") is dropped, not treated as text.
        assert!(!annex_1.text.contains(
            "todos los supuestos, indicar que no son aplicables y explicar el motivo. 1"
        ));
        // The table header, reached only after crossing a page break, is
        // its own paragraph and survives page-furniture removal.
        assert!(
            annex_1
                .text
                .contains("Tipo Definición Sub Tipo Sub Clase de Eventos Ejemplos")
        );
    }

    #[test]
    fn parses_annex_document_heading_and_wrapped_subtitle() {
        let provisions = parse_fixture();
        let annex_8 = provisions
            .iter()
            .find(|item| item.id.ends_with(":annex:8"))
            .expect("annex 8 parsed");
        assert_eq!(annex_8.label, "Anexo 8");
        // The two-line wrapped subtitle merges into one paragraph.
        assert!(annex_8.text.contains(
            "Especificaciones del sistema de información desarrollado por un tercero para el \
             cifrado de información compartida con la Comisión Nacional Bancaria y de Valores y \
             el Banco de México"
        ));
        assert!(annex_8.text.contains("Para efectos de este anexo"));
    }

    #[test]
    fn rejects_annex_document_with_mismatched_heading_number() {
        let error = super::parse_annex_document(
            ANNEX_1_FIXTURE,
            2,
            INSTRUMENT_ID,
            NaiveDate::from_ymd_opt(2021, 1, 28).unwrap(),
        )
        .unwrap_err();
        assert!(error.to_string().contains("expected annex 2"));
    }

    #[test]
    fn keeps_a_paragraph_break_after_a_marker_followed_by_a_page_footer() {
        // A page-number footer between a margin marker and the following
        // genuine blank line must not let the marker's "swallow the next
        // blank" reach across it: the footer is a distinct line, so the
        // blank after it is a real paragraph boundary, not the marker's
        // own line spacing.
        let raw = "ANEXO 99\nPrimer parrafo texto.\n(3)\n42\n\nSegundo parrafo texto.\n";
        let provision = super::parse_annex_document(
            raw,
            99,
            INSTRUMENT_ID,
            NaiveDate::from_ymd_opt(2021, 1, 28).unwrap(),
        )
        .unwrap();
        assert_eq!(
            provision.text,
            "Primer parrafo texto.\n\nSegundo parrafo texto."
        );
        assert_eq!(provision.amendment_marks, vec![3]);
    }

    #[test]
    fn a_marker_with_no_following_content_is_reported_not_silently_dropped() {
        // A marker as the very last line of an annex has no content line
        // left to attach to; this must surface as an error, not vanish.
        let raw = "ANEXO 99\nTexto final.\n(4)\n";
        let error = super::parse_annex_document(
            raw,
            99,
            INSTRUMENT_ID,
            NaiveDate::from_ymd_opt(2021, 1, 28).unwrap(),
        )
        .unwrap_err();
        assert!(error.to_string().contains('4'));
    }
}
