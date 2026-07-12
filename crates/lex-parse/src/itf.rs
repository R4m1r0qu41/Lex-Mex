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
    DcgProvisionBuilder, HeadingState, PendingMarks, TRANSITORY_ORDINALS, amendment_marker_regex,
    flush, parse_annex_document,
};
use crate::{reform_evidence_item, spanish_date};

/// Everything the compiled main document yields besides annexes.
#[derive(Debug)]
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
    marks: Vec<u32>,
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
    // only the orphan parenthesis needs removing. Applied only as a retry
    // when the raw line fails to match an article heading, and only
    // accepted if stripping it produces a real match — so this can never
    // alter a line that isn't actually a mis-rendered article heading.
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
    // Margin markers sit at the vertical position of the text they
    // annotate, which the layout extraction can emit either just before a
    // provision's heading line or between its body lines. They are held
    // here and attached to whichever provision the next content line
    // belongs to; every context boundary either drains them onto a
    // receiving provision or explicitly discards them.
    let mut pending_marks = PendingMarks::default();
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
            if !pending_marks.take_swallow_next_blank() {
                pending_blank = true;
            }
            continue;
        }
        let trimmed = line.trim();

        // Margin markers are held pending and are invisible to paragraph
        // flow; the next content line decides which provision they mark.
        if let Some(captures) = marker_re.captures(trimmed) {
            pending_marks.push(captures[1].parse().expect("two-digit marker"));
            continue;
        }
        pending_marks.observe_content_line();

        // Region transitions shared by every region.
        if trimmed == "TRANSITORIOS" {
            flush(&mut current, &mut provisions, publication_date);
            flush_reform(&mut reform_current, instrument_id, &mut reform_evidence);
            // Leaving Body can carry a stray marker with no home (an
            // Apartado-level "(Derogado)" note with no open article, for
            // instance) — always redundant with the same marker already
            // recorded on the individual provisions it summarizes. Any
            // other region reaching a second TRANSITORIOS heading has no
            // such evidenced exception.
            if matches!(region, Region::Body) {
                pending_marks.discard_from_heading();
            } else {
                pending_marks.discard("a TRANSITORIOS heading")?;
            }
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
            pending_marks.discard("a CONSIDERANDO section")?;
            region = Region::TrailingConsiderandos;
            (pending_blank, crossed_page_break) = (false, false);
            continue;
        }
        if trimmed == "REFERENCIAS" {
            flush(&mut current, &mut provisions, publication_date);
            flush_reform(&mut reform_current, instrument_id, &mut reform_evidence);
            if matches!(region, Region::Body) {
                pending_marks.discard_from_heading();
            } else {
                pending_marks.discard("the REFERENCIAS legend")?;
            }
            region = Region::Legend;
            (pending_blank, crossed_page_break) = (false, false);
            continue;
        }

        match &mut region {
            Region::Body => {
                if let Some(captures) = titulo_re.captures(trimmed) {
                    flush(&mut current, &mut provisions, publication_date);
                    pending_marks.discard_from_heading();
                    headings = HeadingState {
                        title: Some(format!("Título {}", &captures[1])),
                        chapter: None,
                        section: None,
                        apartado: None,
                    };
                } else if let Some(captures) = chapter_re.captures(trimmed) {
                    flush(&mut current, &mut provisions, publication_date);
                    pending_marks.discard_from_heading();
                    headings.chapter = Some(format!("Capítulo {}", &captures[1]));
                    headings.section = None;
                    headings.apartado = None;
                } else if let Some(captures) = section_re.captures(trimmed) {
                    flush(&mut current, &mut provisions, publication_date);
                    pending_marks.discard_from_heading();
                    headings.section = Some(format!("Sección {}", &captures[1]));
                    headings.apartado = None;
                } else if let Some(captures) = apartado_re.captures(trimmed) {
                    flush(&mut current, &mut provisions, publication_date);
                    pending_marks.discard_from_heading();
                    headings.apartado = Some(format!("Apartado {}", &captures[1]));
                } else if let Some(captures) = article_re.captures(trimmed).or_else(|| {
                    // The one glyph-splitting artifact in the source PDF
                    // (see orphan_paren_re above): only accepted when
                    // stripping the leading `) ` turns the line into a
                    // real article heading, so this can never alter an
                    // unrelated line that merely starts the same way.
                    orphan_paren_re.captures(trimmed).and_then(|outer| {
                        article_re.captures(outer.get(1).expect("group").as_str())
                    })
                }) {
                    flush(&mut current, &mut provisions, publication_date);
                    let mut article = DcgProvisionBuilder::article(
                        instrument_id,
                        captures[1].to_owned(),
                        captures[2].trim(),
                        &headings,
                        false,
                    );
                    pending_marks.drain_onto(&mut article);
                    current = Some(article);
                } else if let Some(builder) = &mut current {
                    pending_marks.drain_onto(builder);
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
                    pending_marks.drain_onto(&mut transitory);
                    current = Some(transitory);
                } else if let Some(builder) = &mut current {
                    pending_marks.drain_onto(builder);
                    builder.push_line(line, pending_blank, crossed_page_break);
                }
            }
            Region::ReformTransitories(date) => {
                // CNBV consolidated disposiciones re-amend their own reform
                // transitorios, so a marker can legitimately land here. It
                // is kept on the reform transitory's TemporalEvidence
                // (below); only markers with no open transitory to receive
                // them — e.g. inside the parenthesized attribution block —
                // are still surfaced rather than silently dropped.
                let Some(resolved) = *date else {
                    pending_marks.discard("a per-resolution attribution block")?;
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
                    // A marker preceding the ordinal marks this transitory,
                    // exactly as it would a new article heading.
                    reform_current = Some(ReformTransitory {
                        date: resolved,
                        ordinal: captures[1].to_owned(),
                        lines: vec![trimmed.to_owned()],
                        marks: pending_marks.take(),
                    });
                } else if let Some(reform) = &mut reform_current {
                    reform.marks.extend(pending_marks.take());
                    reform.lines.push(trimmed.to_owned());
                } else {
                    // A marker after the attribution date but before the
                    // first ordinal has no transitory to receive it.
                    pending_marks.discard("a per-resolution TRANSITORIOS section")?;
                }
            }
            // Appended per-reform CONSIDERANDO blocks carry no canonical
            // content and cannot receive a marker either.
            Region::TrailingConsiderandos => {
                pending_marks.discard("a CONSIDERANDO section")?;
            }
            Region::Legend => {
                pending_marks.discard("the REFERENCIAS legend")?;
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
        evidence.push(reform_evidence_item(
            instrument_id,
            reform.date,
            &reform.ordinal,
            "Resolución",
            reform.lines.join(" "),
            reform.marks,
        ));
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

    #[test]
    fn a_marker_inside_a_reform_transitory_is_kept_on_its_evidence() {
        // CNBV consolidated disposiciones re-amend their own reform
        // transitorios, so a marker inside a per-resolution transitory is
        // kept on that transitory's TemporalEvidence — the mention is
        // preserved without linking to the (non-corpus) modifying
        // resolution, rather than surfaced as an error.
        let raw = "Artículo 1.- Objeto de las disposiciones.\n\
                   TRANSITORIOS\n\
                   TRANSITORIOS\n\
                   (Resolución publicada el 25 de marzo de 2019)\n\
                   ÚNICO.- Entra en vigor.\n\
                   (6)\n\n\
                   Segunda línea con marca pendiente.\n\
                   REFERENCIAS\n\
                   6)    Reformado mediante Resolución.\n";
        let document = parse_itf_dcg(
            raw,
            &[],
            ITF_ID,
            NaiveDate::from_ymd_opt(2018, 9, 10).unwrap(),
        )
        .expect("a marker inside a reform transitory is kept, not an error");
        let unico = document
            .reform_evidence
            .iter()
            .find(|evidence| evidence.label.contains("ÚNICO"))
            .expect("the ÚNICO reform transitory is present");
        assert_eq!(unico.amendment_marks, vec![6]);
    }

    #[test]
    fn a_marker_inside_a_considerando_section_is_reported_not_silently_dropped() {
        let raw = "TRANSITORIOS\n\
                   PRIMERO.- Entra en vigor.\n\
                   CONSIDERANDO\n\
                   Que se reforma lo conducente.\n\
                   (6)\n\n\
                   Segunda línea con marca pendiente.\n\
                   REFERENCIAS\n\
                   1)    Reformado.\n";
        let error = parse_itf_dcg(
            raw,
            &[],
            ITF_ID,
            NaiveDate::from_ymd_opt(2018, 9, 10).unwrap(),
        )
        .unwrap_err();
        assert!(error.to_string().contains('6'));
    }

    #[test]
    fn a_marker_orphaned_by_a_structural_heading_is_discarded_silently() {
        // The real document repeals an entire Apartado with no article of
        // its own — heading text followed directly by a lone "(Derogado)"
        // note, each preceded by the resolution's marker. Neither marker
        // has a provision to attach to (the heading itself flushes and
        // clears any article that was open before it, and HeadingContext
        // has no field to receive one), but that fact is always redundant
        // with the same marker already recorded directly on the
        // individual derogated articles it summarizes — so this must
        // parse successfully rather than erroring.
        let raw = "TÍTULO PRIMERO\n\
                   Artículo 25.- Texto vigente.\n\
                   Apartado D\n\
                   (9)\n\
                   (Derogado)\n\n\
                   TRANSITORIOS\n\
                   PRIMERO.- Entra en vigor.\n\
                   REFERENCIAS\n\
                   9)    Derogado por resolución.\n";
        let document = parse_itf_dcg(
            raw,
            &[],
            ITF_ID,
            NaiveDate::from_ymd_opt(2018, 9, 10).unwrap(),
        )
        .unwrap();
        let article_25 = document
            .provisions
            .iter()
            .find(|item| item.id.ends_with(":article:25"))
            .unwrap();
        assert!(article_25.amendment_marks.is_empty());
    }
}
