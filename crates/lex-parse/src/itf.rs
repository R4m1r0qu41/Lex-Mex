//! Parser for the compiled CNBV "Disposiciones de carácter general
//! aplicables a las instituciones de tecnología financiera" (DOF
//! 10/09/2018), the first instrument in the corpus consolidated from
//! amending resolutions.
//!
//! The compiled document differs from the IFPE DCG in four ways this
//! module handles explicitly:
//!
//! 1. **Margin amendment markers.** Amended text carries a numbered
//!    margin marker (`(7)`) whose meaning the document's closing
//!    REFERENCIAS legend defines ("Reformado por el Artículo Primero de la
//!    Resolución publicada… el 25 de marzo de 2019."). Markers are
//!    marginalia, not provision prose: they are removed from canonical
//!    text, recorded per provision as `amendment_marks`, and the legend is
//!    parsed into the corpus-level `amendment_references`.
//! 2. **Título-level headings** above chapters, tracked into
//!    `heading_context.title`.
//! 3. **`Bis` article numbering** (`Artículo 15 Bis.-`, `Artículo 15 Bis
//!    1.-`).
//! 4. **One TRANSITORIOS section per amending resolution** after the
//!    original one. Only the original transitories are canonical
//!    provisions; each later section is attributed to its resolution by
//!    the parenthesized block that follows the heading, and its articles
//!    become reform temporal evidence
//!    (`…:amendment:<dof-date>:transitory:<ordinal>`), mirroring the
//!    LRITF reform-decree appendix.

use anyhow::{Context, Result, bail};
use chrono::NaiveDate;
use lex_core::{AmendmentReference, Provision, TemporalEvidence};
use regex::Regex;

use crate::dcg::{
    DcgProvisionBuilder, HeadingState, TRANSITORY_ORDINALS, amendment_marker_regex, flush,
    parse_annex_document,
};
use crate::{slug, spanish_date};

/// Everything the compiled main document yields besides annexes.
pub struct ItfDocument {
    pub provisions: Vec<Provision>,
    pub amendment_references: Vec<AmendmentReference>,
    pub reform_evidence: Vec<TemporalEvidence>,
    pub latest_reform_date: Option<NaiveDate>,
}

/// Parse the compiled main document plus each annex's dedicated CNBV PDF
/// text (1-indexed, ordered as on the Normatividad page).
pub fn parse_itf_dcg(
    main_raw: &str,
    annex_documents: &[(u32, String)],
    instrument_id: &str,
    publication_date: NaiveDate,
) -> Result<ItfDocument> {
    let mut document = parse_main_text(main_raw, instrument_id, publication_date)?;
    for (number, raw) in annex_documents {
        document.provisions.push(parse_annex_document(
            raw,
            *number,
            instrument_id,
            publication_date,
        )?);
    }
    if document.provisions.is_empty() {
        bail!("no ITF DCG provisions recognized");
    }
    Ok(document)
}

/// Which region of the compiled document the line scanner is in.
enum Region {
    /// Preamble, index, and body articles. The index repeats every heading
    /// line; heading state simply churns through it and is reset by the
    /// body's own headings before Artículo 1.
    Body,
    /// The original TRANSITORIOS section: canonical transitory provisions.
    OriginalTransitories,
    /// A per-resolution TRANSITORIOS section, attributed by the
    /// parenthesized block after the heading. `None` while the attribution
    /// block is still being read.
    ReformTransitories(Option<NaiveDate>),
    /// Appended per-reform CONSIDERANDO blocks: skipped entirely.
    TrailingConsiderandos,
    /// The closing REFERENCIAS legend.
    Legend,
}

/// A reform transitory being accumulated: resolution date, ordinal, lines.
struct ReformTransitory {
    date: NaiveDate,
    ordinal: String,
    lines: Vec<String>,
}

#[allow(clippy::too_many_lines)]
fn parse_main_text(
    raw: &str,
    instrument_id: &str,
    publication_date: NaiveDate,
) -> Result<ItfDocument> {
    let titulo_re = Regex::new(r"^TÍTULO\s+([A-ZÁÉÍÓÚ]+)$")?;
    let chapter_re = Regex::new(r"^Capítulo\s+([IVXLCDM]+)$")?;
    let section_re = Regex::new(r"^Sección\s+([A-Za-zÁÉÍÓÚáéíóú]+)$")?;
    let apartado_re = Regex::new(r"^Apartado\s+([A-Z])$")?;
    let article_re = Regex::new(r"^Artículo\s+(\d+(?:\s+Bis(?:\s+\d+)?)?)\s*\.-\s*(.*)$")?;
    // The per-resolution sections use ÚNICO for single-article resolutions
    // and an en dash after the period in one resolution (`PRIMERO. –`).
    let transitory_re = Regex::new(&format!(
        r"^(ÚNICO|{})\s*\.\s*[-–]\s*(.*)$",
        TRANSITORY_ORDINALS.join("|")
    ))?;
    let marker_re = amendment_marker_regex()?;
    // The one glyph-splitting artifact in the source PDF: article 21's
    // margin marker renders its closing parenthesis as a separate run that
    // lands at the start of the heading line (`) Artículo 21.- …`). The
    // marker number itself arrives as a normal standalone `(7)` line, so
    // only the orphan parenthesis needs removing.
    let orphan_paren_re = Regex::new(r"^\)\s+(Artículo\s.*)$")?;
    let attribution_date_re =
        Regex::new(r"el\s+0?(\d{1,2})\s+de\s+([a-záéíóú]+)\s+de\s+(\d{4})\)$")?;
    let legend_entry_re = Regex::new(r"^(\d{1,2})\)\s+(.*)$")?;

    let mut provisions = Vec::new();
    let mut current: Option<DcgProvisionBuilder> = None;
    let mut headings = HeadingState {
        title: None,
        chapter: None,
        section: None,
        apartado: None,
    };
    let mut region = Region::Body;
    let mut pending_blank = false;
    let mut crossed_page_break = false;
    let mut swallow_blank = false;
    // Margin markers sit at the vertical position of the text they
    // annotate, which the layout extraction can emit either just before a
    // provision's heading line or between its body lines. They are held
    // here and attached to whichever provision the next content line
    // belongs to; structural headings clear them.
    let mut pending_marks: Vec<u32> = Vec::new();
    let mut reform_evidence: Vec<TemporalEvidence> = Vec::new();
    let mut reform_current: Option<ReformTransitory> = None;
    let mut latest_reform_date: Option<NaiveDate> = None;
    let mut amendment_references: Vec<AmendmentReference> = Vec::new();
    let mut legend_current: Option<(u32, Vec<String>)> = None;

    for source_line in raw.lines() {
        if source_line.starts_with('\u{c}') {
            crossed_page_break = true;
        }
        let line = source_line.trim_start_matches('\u{c}');
        if line.trim().is_empty() {
            // A blank immediately following a margin marker is part of the
            // marker's own line box, not a paragraph boundary.
            if swallow_blank {
                swallow_blank = false;
            } else {
                pending_blank = true;
            }
            continue;
        }
        let trimmed_line = line.trim();
        let trimmed = orphan_paren_re
            .captures(trimmed_line)
            .map_or(trimmed_line, |captures| {
                captures.get(1).expect("heading capture").as_str()
            });

        // Margin markers are held pending and are invisible to paragraph
        // flow; the next content line decides which provision they mark.
        if let Some(captures) = marker_re.captures(trimmed) {
            pending_marks.push(captures[1].parse().expect("two-digit marker"));
            swallow_blank = true;
            continue;
        }
        swallow_blank = false;

        // Region transitions shared by every region.
        if trimmed == "TRANSITORIOS" {
            flush(&mut current, &mut provisions, publication_date);
            flush_reform(&mut reform_current, instrument_id, &mut reform_evidence);
            pending_marks.clear();
            region = match region {
                Region::Body => Region::OriginalTransitories,
                _ => Region::ReformTransitories(None),
            };
            (pending_blank, crossed_page_break) = (false, false);
            continue;
        }
        if trimmed == "CONSIDERANDO"
            && matches!(
                region,
                Region::OriginalTransitories | Region::ReformTransitories(_)
            )
        {
            flush(&mut current, &mut provisions, publication_date);
            flush_reform(&mut reform_current, instrument_id, &mut reform_evidence);
            region = Region::TrailingConsiderandos;
            (pending_blank, crossed_page_break) = (false, false);
            continue;
        }
        if trimmed == "REFERENCIAS" {
            flush(&mut current, &mut provisions, publication_date);
            flush_reform(&mut reform_current, instrument_id, &mut reform_evidence);
            region = Region::Legend;
            (pending_blank, crossed_page_break) = (false, false);
            continue;
        }

        match &mut region {
            Region::Body => {
                if let Some(captures) = titulo_re.captures(trimmed) {
                    flush(&mut current, &mut provisions, publication_date);
                    pending_marks.clear();
                    headings = HeadingState {
                        title: Some(format!("Título {}", &captures[1])),
                        chapter: None,
                        section: None,
                        apartado: None,
                    };
                } else if let Some(captures) = chapter_re.captures(trimmed) {
                    flush(&mut current, &mut provisions, publication_date);
                    pending_marks.clear();
                    headings.chapter = Some(format!("Capítulo {}", &captures[1]));
                    headings.section = None;
                    headings.apartado = None;
                } else if let Some(captures) = section_re.captures(trimmed) {
                    flush(&mut current, &mut provisions, publication_date);
                    pending_marks.clear();
                    headings.section = Some(format!("Sección {}", &captures[1]));
                    headings.apartado = None;
                } else if let Some(captures) = apartado_re.captures(trimmed) {
                    flush(&mut current, &mut provisions, publication_date);
                    pending_marks.clear();
                    headings.apartado = Some(format!("Apartado {}", &captures[1]));
                } else if let Some(captures) = article_re.captures(trimmed) {
                    flush(&mut current, &mut provisions, publication_date);
                    let mut article = DcgProvisionBuilder::article(
                        instrument_id,
                        captures[1].to_owned(),
                        captures[2].trim(),
                        &headings,
                        false,
                    );
                    for marker in pending_marks.drain(..) {
                        article.mark(marker);
                    }
                    current = Some(article);
                } else if let Some(builder) = &mut current {
                    for marker in pending_marks.drain(..) {
                        builder.mark(marker);
                    }
                    builder.push_line(line, pending_blank, crossed_page_break);
                }
            }
            Region::OriginalTransitories => {
                if let Some(captures) = transitory_re.captures(trimmed) {
                    flush(&mut current, &mut provisions, publication_date);
                    let mut transitory = DcgProvisionBuilder::transitory(
                        instrument_id,
                        &captures[1],
                        captures[2].trim(),
                    );
                    for marker in pending_marks.drain(..) {
                        transitory.mark(marker);
                    }
                    current = Some(transitory);
                } else if let Some(builder) = &mut current {
                    for marker in pending_marks.drain(..) {
                        builder.mark(marker);
                    }
                    builder.push_line(line, pending_blank, crossed_page_break);
                }
            }
            Region::ReformTransitories(date) => {
                let Some(resolved) = *date else {
                    // Reading the parenthesized attribution block; its
                    // final line carries the resolution's DOF date.
                    if let Some(captures) = attribution_date_re.captures(trimmed) {
                        let parsed = spanish_date(&captures[1], &captures[2], &captures[3])
                            .with_context(|| {
                                format!("unparseable resolution date in attribution: {trimmed}")
                            })?;
                        *date = Some(parsed);
                        latest_reform_date =
                            Some(latest_reform_date.map_or(parsed, |seen| seen.max(parsed)));
                    }
                    (pending_blank, crossed_page_break) = (false, false);
                    continue;
                };
                if let Some(captures) = transitory_re.captures(trimmed) {
                    flush_reform(&mut reform_current, instrument_id, &mut reform_evidence);
                    reform_current = Some(ReformTransitory {
                        date: resolved,
                        ordinal: captures[1].to_owned(),
                        lines: vec![trimmed.to_owned()],
                    });
                } else if let Some(reform) = &mut reform_current {
                    reform.lines.push(trimmed.to_owned());
                }
            }
            // Appended per-reform CONSIDERANDO blocks carry no canonical
            // content; the shared REFERENCIAS transition above exits them.
            Region::TrailingConsiderandos => {}
            Region::Legend => {
                if let Some(captures) = legend_entry_re.captures(trimmed) {
                    flush_legend(&mut legend_current, &mut amendment_references);
                    let marker: u32 = captures[1].parse().expect("two-digit marker");
                    legend_current = Some((marker, vec![captures[2].trim().to_owned()]));
                } else if let Some((_, lines)) = &mut legend_current {
                    lines.push(trimmed.to_owned());
                }
            }
        }
        (pending_blank, crossed_page_break) = (false, false);
    }
    flush(&mut current, &mut provisions, publication_date);
    flush_reform(&mut reform_current, instrument_id, &mut reform_evidence);
    flush_legend(&mut legend_current, &mut amendment_references);

    if amendment_references.is_empty() {
        bail!("compiled ITF document has no REFERENCIAS legend");
    }
    Ok(ItfDocument {
        provisions,
        amendment_references,
        reform_evidence,
        latest_reform_date,
    })
}

fn flush_reform(
    current: &mut Option<ReformTransitory>,
    instrument_id: &str,
    evidence: &mut Vec<TemporalEvidence>,
) {
    if let Some(reform) = current.take() {
        let date = reform.date.format("%Y-%m-%d");
        evidence.push(TemporalEvidence {
            provision_id: format!(
                "{instrument_id}:amendment:{date}:transitory:{}",
                slug(&reform.ordinal)
            ),
            label: format!("Transitorio {} — Resolución DOF {date}", reform.ordinal),
            text: reform.lines.join(" "),
        });
    }
}

fn flush_legend(
    current: &mut Option<(u32, Vec<String>)>,
    references: &mut Vec<AmendmentReference>,
) {
    if let Some((marker, lines)) = current.take() {
        references.push(AmendmentReference {
            marker,
            description: lines.join(" "),
        });
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use lex_core::ProvisionType;

    use super::parse_itf_dcg;

    const FIXTURE: &str = include_str!("../../../fixtures/itf-dcg-2018/parser-sample.txt");
    const ANNEX_PRE_MARKER: &str =
        include_str!("../../../fixtures/itf-dcg-2018/annex-marker-sample.txt");
    const ANNEX_INLINE_MARKER: &str =
        include_str!("../../../fixtures/itf-dcg-2018/annex-inline-marker-sample.txt");
    const ITF_ID: &str = "urn:lex-mx:federal:regulation:itf-dcg-2018";

    fn parse_fixture() -> super::ItfDocument {
        parse_itf_dcg(
            FIXTURE,
            &[
                (2, ANNEX_PRE_MARKER.to_owned()),
                (14, ANNEX_INLINE_MARKER.to_owned()),
            ],
            ITF_ID,
            NaiveDate::from_ymd_opt(2018, 9, 10).unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn parses_compiled_document_with_amendment_markers() {
        let document = parse_fixture();
        let by_id = |suffix: &str| {
            document
                .provisions
                .iter()
                .find(|item| item.id.ends_with(suffix))
                .unwrap_or_else(|| panic!("missing {suffix}"))
        };

        // The index at the top never produces provisions; body headings
        // reset the churned heading state before Artículo 1.
        let articles = document
            .provisions
            .iter()
            .filter(|item| item.provision_type == ProvisionType::Article)
            .count();
        assert_eq!(articles, 7);

        // A mid-article margin marker records onto that article and its
        // paragraph flows across the marker without a break.
        let two = by_id(":article:2");
        assert_eq!(two.amendment_marks, vec![1]);
        assert!(two.text.starts_with(
            "En adición a las definiciones contenidas en la Ley, para efectos de las \
             presentes disposiciones se entenderá, en singular o plural, por:"
        ));
        assert!(two.text.contains("\n\nI. Autenticación, al conjunto"));
        assert!(!two.text.contains("(1)"));

        // Bis numbering canonicalizes with hyphens and keeps the heading
        // context of the surrounding título/capítulo/apartado.
        let bis1 = by_id(":article:15-bis-1");
        assert_eq!(bis1.label, "Artículo 15 Bis 1");
        assert_eq!(
            bis1.heading_context.title.as_deref(),
            Some("Título SEGUNDO")
        );
        assert_eq!(bis1.heading_context.chapter.as_deref(), Some("Capítulo IV"));
        assert_eq!(bis1.heading_context.apartado.as_deref(), Some("Apartado A"));

        // The orphan-parenthesis artifact on Artículo 21 is repaired.
        let twenty_one = by_id(":article:21");
        assert_eq!(twenty_one.amendment_marks, vec![7]);
        assert!(
            twenty_one
                .text
                .starts_with("Las ITF deberán notificar por escrito")
        );

        // A derogated article keeps its official body and its marker.
        let derogated = by_id(":article:26");
        assert_eq!(derogated.text, "(Derogado)");
        assert_eq!(derogated.amendment_marks, vec![9]);

        // A marker printed just before a transitory heading belongs to
        // that transitory, not the previous one.
        let quinto = by_id(":transitory:quinto");
        assert_eq!(quinto.amendment_marks, vec![4]);
        let septimo = by_id(":transitory:septimo");
        assert_eq!(septimo.text, "Derogado.");
        assert_eq!(septimo.amendment_marks, vec![5]);

        // Only the original transitories are canonical.
        let transitories = document
            .provisions
            .iter()
            .filter(|item| item.provision_type == ProvisionType::Transitory)
            .count();
        assert_eq!(transitories, 3);
    }

    #[test]
    fn attributes_reform_transitories_and_parses_the_legend() {
        let document = parse_fixture();

        let ids: Vec<_> = document
            .reform_evidence
            .iter()
            .map(|item| item.provision_id.as_str())
            .collect();
        assert_eq!(
            ids,
            [
                format!("{ITF_ID}:amendment:2019-03-25:transitory:primero"),
                format!("{ITF_ID}:amendment:2019-03-25:transitory:segundo"),
                format!("{ITF_ID}:amendment:2025-09-09:transitory:unico"),
            ]
        );
        // The en-dash ordinal separator (`ÚNICO. –`) parses, and evidence
        // text keeps the full article.
        assert!(
            document.reform_evidence[2]
                .text
                .contains("entrará en vigor el 1 de enero de 2026")
        );
        assert_eq!(
            document.latest_reform_date,
            NaiveDate::from_ymd_opt(2025, 9, 9)
        );

        let legend: Vec<_> = document
            .amendment_references
            .iter()
            .map(|item| item.marker)
            .collect();
        assert_eq!(legend, [1, 4, 5, 7, 9]);
        assert_eq!(
            document.amendment_references[2].description,
            "Derogado por el Artículo Segundo de la Resolución publicada en el Diario \
             Oficial de la Federación el 25 de marzo de 2019."
        );
    }

    #[test]
    fn annex_markers_attach_without_touching_inline_parenthesized_numbers() {
        let document = parse_fixture();
        let annex_two = document
            .provisions
            .iter()
            .find(|item| item.id.ends_with(":annex:2"))
            .unwrap();
        // Marker line before the heading; the inline `un (1) reporte` is
        // ordinary prose and stays in the text.
        assert_eq!(annex_two.amendment_marks, vec![2]);
        assert_eq!(annex_two.label, "ANEXO 2");
        assert!(annex_two.text.contains("un (1) reporte"));

        // Landscape annex with the marker on the heading line itself.
        let annex_fourteen = document
            .provisions
            .iter()
            .find(|item| item.id.ends_with(":annex:14"))
            .unwrap();
        assert_eq!(annex_fourteen.amendment_marks, vec![2]);
        assert_eq!(annex_fourteen.label, "ANEXO 14");
        assert!(!annex_fourteen.text.contains("(2)"));
    }
}
