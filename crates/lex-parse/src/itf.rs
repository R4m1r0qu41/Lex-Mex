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

/// Feminine ordinal forms used when transitorios are styled as
/// "disposiciones transitorias" (`PRIMERA`, `SEGUNDA`, …). Masculine forms
/// live in [`TRANSITORY_ORDINALS`]; the two are matched together.
const FEMININE_TRANSITORY_ORDINALS: &[&str] = &[
    "PRIMERA", "SEGUNDA", "TERCERA", "CUARTA", "QUINTA", "SEXTA", "SÉPTIMA", "OCTAVA", "NOVENA",
    "DÉCIMA",
];

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
    /// parenthesized block after the heading. `resolved` is the resolution's
    /// own DOF date (the block's closing date), `None` while the block is
    /// still being read. `base` is the DOF date of the resolution this one
    /// modifies, present only for an amendment of an amendment ("Resolución
    /// modificatoria de la Resolución que modifica…"); it distinguishes two
    /// such resolutions published the same day (see [`finalize_reform`]).
    ReformTransitories {
        resolved: Option<NaiveDate>,
        base: Option<NaiveDate>,
        /// The parenthesized attribution text accumulated across line
        /// wraps until its closing date resolves the section.
        attribution: String,
    },
    /// Appended per-reform CONSIDERANDO blocks: skipped entirely.
    TrailingConsiderandos,
    /// The closing REFERENCIAS legend.
    Legend,
}

/// A reform transitory being accumulated: resolution date, the DOF date of
/// the resolution it modifies (if it is an amendment of an amendment),
/// ordinal, lines, and markers.
struct ReformTransitory {
    date: NaiveDate,
    base_date: Option<NaiveDate>,
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
    // Older DCGs write the low articles with the masculine ordinal
    // abbreviation (`Artículo 1o.-` … `9o.-`, also `º`/`°`); the ordinal is
    // dropped so the canonical number matches the plain form (`1o` → `1`,
    // as `8 ≡ 8o`). Group 1 is the digits, group 2 any ` Bis[ N]` tail
    // (possibly empty), group 3 the heading remainder.
    let article_re =
        Regex::new(r"^Artículo\s+(\d+)(?:o|º|°)?((?:\s+Bis(?:\s+\d+)?)?)\s*\.-\s*(.*)$")?;
    // A transitorios-section heading appears across CNBV vintages in
    // masculine plural (`TRANSITORIOS`), masculine singular (`TRANSITORIO`,
    // for a single-article modifying resolution), and the feminine
    // "disposiciones transitorias" forms (`TRANSITORIA`/`TRANSITORIAS`).
    // All open the same region; the first is the original section and every
    // later one is a per-resolution reform section (handled below).
    let transitorios_heading_re = Regex::new(r"^TRANSITORI[OA]S?$")?;
    // The per-resolution sections use ÚNICO/ÚNICA for single-article
    // resolutions and an en dash after the period in one resolution
    // (`PRIMERO. –`). Ordinals occur in both grammatical genders because a
    // "Disposición Transitoria" is feminine (`PRIMERA`, `SEGUNDA`, …).
    let transitory_re = Regex::new(&format!(
        r"^(ÚNIC[OA]|{}|{})\s*\.\s*[-–]\s*(.*)$",
        TRANSITORY_ORDINALS.join("|"),
        FEMININE_TRANSITORY_ORDINALS.join("|"),
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
    // The attribution block closes with the resolution's DOF date, e.g.
    // `… en el Diario Oficial de la Federación el 25 de marzo de 2019)`.
    // It is matched against the whole attribution text (accumulated across
    // line wraps that can split the date itself, e.g. `… el 12 de enero de`
    // / `2015)`), so the date immediately before the closing paren is the
    // section's own; the leading connector (`el`, or `de` in older
    // resolutions) is optional because a wrap can strand it.
    let attribution_date_re =
        Regex::new(r"(?:(?:el|de)\s+)?0?(\d{1,2})\s+de\s+([a-záéíóú]+)\s+de\s+(\d{4})\)")?;
    // Inside a second-order attribution ("Resolución modificatoria de la
    // Resolución que modifica… publicada … de la Federación el 23 de enero
    // de 2018, publicada … el 27 de diciembre de 2024)"), the modified
    // resolution's date is the `de la Federación el <date>,` clause that
    // ends with a comma, distinct from the block's own paren-closed date.
    let base_resolution_date_re =
        Regex::new(r"de la Federación el\s+0?(\d{1,2})\s+de\s+([a-záéíóú]+)\s+de\s+(\d{4}),")?;
    // A REFERENCIAS legend entry is `N)  text`; some vintages parenthesize
    // the number as `(N)  text`, so the leading paren is optional.
    let legend_entry_re = Regex::new(r"^\(?(\d{1,2})\)\s+(.*)$")?;

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
    let mut reform_transitories: Vec<ReformTransitory> = Vec::new();
    let mut reform_current: Option<ReformTransitory> = None;
    let mut latest_reform_date: Option<NaiveDate> = None;
    let mut amendment_references: Vec<AmendmentReference> = Vec::new();
    let mut legend_current: Option<(u32, Vec<String>)> = None;
    // The compiled document opens with a preamble and a repeated índice
    // (table of contents) that echoes headings — including a `TRANSITORIOS`
    // line and the annex list, each with its own markers — before the first
    // article. Region transitions and margin markers only count once the
    // body proper begins at the first article heading; otherwise an índice
    // `TRANSITORIOS` echo flips the scanner into the transitorios region
    // before any article is parsed (the SOCAP/OAAC failure, where every
    // body marker then piled up unattached). Índice markers are redundant
    // with the same marker on the provision they annotate — still recorded
    // in the REFERENCIAS legend — so preamble markers are dropped.
    let mut body_started = false;

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
            if body_started {
                pending_marks.push(captures[1].parse().expect("two-digit marker"));
            }
            continue;
        }
        pending_marks.observe_content_line();

        // Region transitions shared by every region. Suppressed in the
        // preamble/índice, where a `TRANSITORIOS` echo is not the real
        // section (see `body_started`).
        if body_started && transitorios_heading_re.is_match(trimmed) {
            // A marker printed at the foot of the section being left
            // annotates its last provision (the last article, transitory,
            // or reform transitory); attach it there before flushing. Any
            // marker still pending has no provision to receive it — an
            // Apartado-level "(Derogado)" note whose article was already
            // flushed by its own heading, for instance — and is heading-
            // level marginalia, redundant with the same marker on the
            // provisions it summarizes and recorded in the legend, so it is
            // dropped rather than stranded.
            if let Some(builder) = &mut current {
                pending_marks.drain_onto(builder);
            } else if let Some(reform) = &mut reform_current {
                reform.marks.extend(pending_marks.take());
            }
            flush(&mut current, &mut provisions, publication_date);
            flush_reform(&mut reform_current, &mut reform_transitories);
            pending_marks.discard_from_heading();
            region = match region {
                Region::Body => Region::OriginalTransitories,
                _ => Region::ReformTransitories {
                    resolved: None,
                    base: None,
                    attribution: String::new(),
                },
            };
            (pending_blank, crossed_page_break) = (false, false);
            continue;
        }
        if body_started
            && trimmed == "CONSIDERANDO"
            && matches!(
                region,
                Region::OriginalTransitories | Region::ReformTransitories { .. }
            )
        {
            flush(&mut current, &mut provisions, publication_date);
            flush_reform(&mut reform_current, &mut reform_transitories);
            pending_marks.discard_from_heading();
            region = Region::TrailingConsiderandos;
            (pending_blank, crossed_page_break) = (false, false);
            continue;
        }
        if body_started && trimmed == "REFERENCIAS" {
            flush(&mut current, &mut provisions, publication_date);
            flush_reform(&mut reform_current, &mut reform_transitories);
            pending_marks.discard_from_heading();
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
                    body_started = true;
                    let number = format!("{}{}", &captures[1], &captures[2]);
                    let mut article = DcgProvisionBuilder::article(
                        instrument_id,
                        number,
                        captures[3].trim(),
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
            Region::ReformTransitories {
                resolved,
                base,
                attribution,
            } => {
                // CNBV consolidated disposiciones re-amend their own reform
                // transitorios, so a marker can legitimately land here — in
                // the attribution block, before the first ordinal, or on a
                // transitory's own lines. Any such marker is held and
                // attached to the first transitory of the section (below),
                // keeping the mention as it would on an article.
                let Some(resolved_date) = *resolved else {
                    // Accumulate the parenthesized attribution across the
                    // line wraps that can split even the date (`… el 12 de
                    // enero de` / `2015)`), then read it once the closing
                    // paren appears. The date just before that paren is the
                    // resolution's own; a second-order attribution also
                    // names the modified resolution first (comma-terminated).
                    if !attribution.is_empty() {
                        attribution.push(' ');
                    }
                    attribution.push_str(trimmed);
                    if let Some(captures) = attribution_date_re.captures(attribution) {
                        let parsed = spanish_date(&captures[1], &captures[2], &captures[3])
                            .with_context(|| {
                                format!("unparseable resolution date in attribution: {attribution}")
                            })?;
                        if let Some(base_captures) = base_resolution_date_re.captures(attribution) {
                            *base = Some(
                                spanish_date(
                                    &base_captures[1],
                                    &base_captures[2],
                                    &base_captures[3],
                                )
                                .with_context(|| {
                                    format!("unparseable modified-resolution date: {attribution}")
                                })?,
                            );
                        }
                        *resolved = Some(parsed);
                        latest_reform_date =
                            Some(latest_reform_date.map_or(parsed, |seen| seen.max(parsed)));
                    }
                    (pending_blank, crossed_page_break) = (false, false);
                    continue;
                };
                if let Some(captures) = transitory_re.captures(trimmed) {
                    flush_reform(&mut reform_current, &mut reform_transitories);
                    // A marker preceding the ordinal marks this transitory,
                    // exactly as it would a new article heading.
                    reform_current = Some(ReformTransitory {
                        date: resolved_date,
                        base_date: *base,
                        ordinal: captures[1].to_owned(),
                        lines: vec![trimmed.to_owned()],
                        marks: pending_marks.take(),
                    });
                } else if let Some(reform) = &mut reform_current {
                    reform.marks.extend(pending_marks.take());
                    reform.lines.push(trimmed.to_owned());
                }
                // Otherwise the line is section text before the first
                // ordinal; any pending marker stays held for that ordinal.
            }
            // Appended per-reform CONSIDERANDO blocks and the REFERENCIAS
            // legend are non-canonical: a marker there is marginalia,
            // redundant with the same marker on the provision it annotates,
            // so it is dropped rather than surfaced. A structural mis-parse
            // instead strands markers at a TRANSITORIOS heading, which stays
            // an error.
            Region::TrailingConsiderandos => {
                pending_marks.discard_from_heading();
            }
            Region::Legend => {
                pending_marks.discard_from_heading();
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
    flush_reform(&mut reform_current, &mut reform_transitories);
    flush_legend(&mut legend_current, &mut amendment_references);

    if amendment_references.is_empty() {
        bail!("compiled ITF document has no REFERENCIAS legend");
    }
    Ok(ItfDocument {
        provisions,
        amendment_references,
        reform_evidence: finalize_reform(&reform_transitories, instrument_id),
        latest_reform_date,
    })
}

fn flush_reform(current: &mut Option<ReformTransitory>, out: &mut Vec<ReformTransitory>) {
    if let Some(mut reform) = current.take() {
        // A single resolution's marker can print at several line positions
        // within one reform transitory (once per amended paragraph); the
        // mention is "amended by resolution N", so collapse repeats, as a
        // provision's `BTreeSet`-backed marks already do.
        reform.marks.sort_unstable();
        reform.marks.dedup();
        out.push(reform);
    }
}

/// Build reform temporal evidence, giving each transitory a canonical id
/// `…:amendment:<dof-date>:transitory:<ordinal>`.
///
/// Two modifying resolutions can be published on the same DOF date, each
/// with its own `ÚNICO` — for SCAP the 19a and 20a resolutions, both DOF
/// 2024-12-27, one amending the resolution of 2018-01-23 and the other that
/// of 2024-04-09 (an amendment of an amendment, each pushing its base
/// resolution's entry into force forward). That collides the plain id, so a
/// colliding group is disambiguated by the modified resolution's own DOF
/// date (`…:modifies:<base>:transitory:<ordinal>`), which identifies each as
/// the ÚNICO of a distinct resolution rather than an anonymous duplicate.
/// If a colliding group cannot be told apart that way (no distinct base
/// dates), it falls back to a stable occurrence suffix so ids stay unique.
fn finalize_reform(
    transitories: &[ReformTransitory],
    instrument_id: &str,
) -> Vec<TemporalEvidence> {
    use std::collections::HashMap;

    let base_id = |reform: &ReformTransitory| {
        format!(
            "{instrument_id}:amendment:{}:transitory:{}",
            reform.date.format("%Y-%m-%d"),
            crate::slug(&reform.ordinal),
        )
    };

    // Group indices by the plain id they would take, preserving order.
    let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
    for (index, reform) in transitories.iter().enumerate() {
        groups.entry(base_id(reform)).or_default().push(index);
    }

    let mut evidence: Vec<Option<TemporalEvidence>> =
        (0..transitories.len()).map(|_| None).collect();
    for indices in groups.values() {
        // A base date disambiguates a colliding group only if every member
        // has one and they are all distinct.
        let bases: Vec<Option<NaiveDate>> =
            indices.iter().map(|&i| transitories[i].base_date).collect();
        let distinct_bases = indices.len() > 1
            && bases.iter().all(Option::is_some)
            && bases.iter().collect::<std::collections::HashSet<_>>().len() == indices.len();

        for (occurrence, &index) in indices.iter().enumerate() {
            let reform = &transitories[index];
            let mut item = reform_evidence_item(
                instrument_id,
                reform.date,
                &reform.ordinal,
                "Resolución",
                reform.lines.join(" "),
                reform.marks.clone(),
            );
            if indices.len() > 1 {
                if distinct_bases {
                    let base = reform.base_date.expect("checked all Some");
                    let base = base.format("%Y-%m-%d");
                    item.provision_id = item.provision_id.replacen(
                        ":transitory:",
                        &format!(":modifies:{base}:transitory:"),
                        1,
                    );
                    item.label = format!("{} (modifica resolución DOF {base})", item.label);
                } else if occurrence > 0 {
                    item.provision_id = format!("{}-{}", item.provision_id, occurrence + 1);
                    item.label = format!("{} (#{})", item.label, occurrence + 1);
                }
            }
            evidence[index] = Some(item);
        }
    }
    evidence
        .into_iter()
        .map(|item| item.expect("every index filled"))
        .collect()
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
    fn singular_and_feminine_reform_sections_become_distinct_evidence() {
        // Older CNBV compilations head a single-article modifying
        // resolution `TRANSITORIO` (singular) or style it as a feminine
        // "Disposición Transitoria" (`TRANSITORIA` / `ÚNICA`), and wrap the
        // attribution's `el`/`de` connector onto the previous line so the
        // date lands on a bare closing line. Each such section must open a
        // reform region and produce its own date-attributed evidence — not
        // collapse into repeated original `transitory:unico` provisions
        // (the servinv-dcg-2013 duplicate_id failure).
        let raw = "Artículo 1.- Objeto de las disposiciones.\n\
                   TRANSITORIOS\n\
                   PRIMERO.- Las presentes disposiciones entrarán en vigor.\n\
                   TRANSITORIO\n\
                   (Resolución que modifica las Disposiciones, publicada en el\n\
                   Diario Oficial de la Federación el\n\
                   29 de julio de 2015)\n\
                   ÚNICO.- La presente Resolución entrará en vigor al día siguiente.\n\
                   TRANSITORIA\n\
                   (Resolución que modifica las Disposiciones, publicada en el\n\
                   Diario Oficial de la Federación de\n\
                   26 de octubre de 2015)\n\
                   ÚNICA.- La presente Resolución entrará en vigor al día siguiente.\n\
                   REFERENCIAS\n\
                   1)    Reformado mediante Resolución.\n";
        let document = parse_itf_dcg(
            raw,
            &[],
            ITF_ID,
            NaiveDate::from_ymd_opt(2018, 9, 10).unwrap(),
        )
        .expect("singular/feminine reform sections parse");

        // Exactly one original transitory (PRIMERO); the reform ÚNICO/ÚNICA
        // are evidence, not provisions, so no id collides.
        let original_transitories: Vec<_> = document
            .provisions
            .iter()
            .filter(|item| item.provision_type == ProvisionType::Transitory)
            .map(|item| item.id.as_str())
            .collect();
        assert_eq!(
            original_transitories,
            [format!("{ITF_ID}:transitory:primero")]
        );

        let evidence_ids: Vec<_> = document
            .reform_evidence
            .iter()
            .map(|item| item.provision_id.as_str())
            .collect();
        assert_eq!(
            evidence_ids,
            [
                format!("{ITF_ID}:amendment:2015-07-29:transitory:unico"),
                format!("{ITF_ID}:amendment:2015-10-26:transitory:unica"),
            ]
        );
        assert_eq!(
            document.latest_reform_date,
            NaiveDate::from_ymd_opt(2015, 10, 26)
        );
    }

    #[test]
    fn ordinal_abbreviation_articles_normalize_to_the_plain_number() {
        // Older DCGs head the low articles with the masculine ordinal
        // abbreviation (`Artículo 1o.-`); the canonical number drops it so
        // `1o` is the same article as a plain `1` would be (`8 ≡ 8o`).
        let raw = "Artículo 1o.- Primero objeto.\n\
                   Artículo 2o.- Segundo objeto.\n\
                   Artículo 15 Bis.- Con qualificador.\n\
                   TRANSITORIOS\n\
                   PRIMERO.- Entra en vigor.\n\
                   REFERENCIAS\n\
                   1)    Reformado mediante Resolución.\n";
        let document = parse_itf_dcg(
            raw,
            &[],
            ITF_ID,
            NaiveDate::from_ymd_opt(2003, 3, 19).unwrap(),
        )
        .expect("ordinal-abbreviation articles parse");
        let numbers: Vec<_> = document
            .provisions
            .iter()
            .filter(|item| item.provision_type == ProvisionType::Article)
            .map(|item| item.number.as_str())
            .collect();
        assert_eq!(numbers, ["1", "2", "15 Bis"]);
        assert!(
            document
                .provisions
                .iter()
                .any(|item| item.id.ends_with(":article:1"))
        );
    }

    #[test]
    fn preamble_indice_is_skipped_and_its_markers_dropped() {
        // The índice (table of contents) echoes a TRANSITORIOS heading and
        // annex markers before the first article. Neither flips the scanner
        // into the transitorios region nor attaches to article 1 (the
        // SOCAP/OAAC failure); the body begins at Artículo 1.
        let raw = "DISPOSICIONES DE PRUEBA\n\
                   (18)\n\
                   TITULO PRIMERO\n\
                   TRANSITORIOS\n\
                   Listado de Anexos\n\
                   (9)\n\
                   Anexo 1  Algo.\n\
                   Artículo 1.- Objeto real.\n\
                   Artículo 2.- Segundo.\n\
                   TRANSITORIOS\n\
                   PRIMERO.- Entra en vigor.\n\
                   REFERENCIAS\n\
                   18)   Reformada la denominación.\n\
                   9)    Anexo reformado.\n";
        let document = parse_itf_dcg(
            raw,
            &[],
            ITF_ID,
            NaiveDate::from_ymd_opt(2009, 1, 19).unwrap(),
        )
        .expect("índice is skipped, body parses");
        let articles: Vec<_> = document
            .provisions
            .iter()
            .filter(|item| item.provision_type == ProvisionType::Article)
            .collect();
        assert_eq!(
            articles
                .iter()
                .map(|a| a.number.as_str())
                .collect::<Vec<_>>(),
            ["1", "2"]
        );
        // The índice (18)/(9) markers did not land on article 1, and the
        // índice TRANSITORIOS echo did not open a transitory.
        assert!(articles[0].amendment_marks.is_empty());
        assert_eq!(
            document
                .provisions
                .iter()
                .filter(|item| item.provision_type == ProvisionType::Transitory)
                .count(),
            1
        );
    }

    #[test]
    fn an_attribution_date_split_across_lines_resolves_the_section() {
        // A wrap can split the attribution date itself (`… el 12 de enero
        // de` / `2015)`); the accumulated attribution still resolves the
        // section rather than stranding its markers.
        let raw = "Artículo 1.- Objeto.\n\
                   TRANSITORIOS\n\
                   PRIMERO.- Entra en vigor.\n\
                   TRANSITORIO\n\
                   (Resolución que modifica las Disposiciones, publicada en el\n\
                   Diario Oficial de la Federación el 12 de enero de\n\
                   2015)\n\
                   ÚNICO.- Entra en vigor al día siguiente.\n\
                   REFERENCIAS\n\
                   1)   Reformado.\n";
        let document = parse_itf_dcg(
            raw,
            &[],
            ITF_ID,
            NaiveDate::from_ymd_opt(2006, 12, 18).unwrap(),
        )
        .expect("split date resolves");
        assert_eq!(
            document
                .reform_evidence
                .iter()
                .map(|item| item.provision_id.as_str())
                .collect::<Vec<_>>(),
            [format!("{ITF_ID}:amendment:2015-01-12:transitory:unico")]
        );
    }

    #[test]
    fn a_parenthesized_legend_number_is_accepted() {
        // Some vintages parenthesize the legend number as `(N)  text`.
        let raw = "Artículo 1.- Objeto.\n\
                   TRANSITORIOS\n\
                   PRIMERO.- Entra en vigor.\n\
                   REFERENCIAS\n\
                   (3)    Reformado por Resolución.\n";
        let document = parse_itf_dcg(
            raw,
            &[],
            ITF_ID,
            NaiveDate::from_ymd_opt(2014, 11, 24).unwrap(),
        )
        .expect("parenthesized legend parses");
        assert!(
            document
                .amendment_references
                .iter()
                .any(|reference| reference.marker == 3)
        );
    }

    #[test]
    fn same_day_reforms_are_disambiguated_by_the_resolution_each_modifies() {
        // Two modifying resolutions can share a DOF date, each with its own
        // ÚNICO amending a different base resolution (SCAP 19a/20a, both DOF
        // 2024-12-27, amending the 2018-01-23 and 2024-04-09 resolutions).
        // The plain id collides; each is disambiguated by the resolution it
        // modifies, not flattened into an anonymous duplicate.
        let raw = "Artículo 1.- Objeto.\n\
                   TRANSITORIOS\n\
                   PRIMERO.- Entra en vigor.\n\
                   TRANSITORIO\n\
                   (Resolución modificatoria de la Resolución que modifica las Disposiciones,\n\
                   publicada en el Diario Oficial de la Federación el 23 de enero de 2018,\n\
                   publicada en el citado medio de difusión el 27 de diciembre de 2024)\n\
                   ÚNICO.- Entrará en vigor al día siguiente.\n\
                   TRANSITORIO\n\
                   (Resolución modificatoria de la Resolución que modifica las Disposiciones,\n\
                   publicada en el Diario Oficial de la Federación el 9 de abril de 2024,\n\
                   publicada en el citado medio de difusión el 27 de diciembre de 2024)\n\
                   ÚNICO.- Entrará en vigor al día siguiente.\n\
                   REFERENCIAS\n\
                   1)    Reformado mediante Resolución.\n";
        let document = parse_itf_dcg(
            raw,
            &[],
            ITF_ID,
            NaiveDate::from_ymd_opt(2018, 9, 10).unwrap(),
        )
        .expect("same-day reforms parse");

        let ids: Vec<_> = document
            .reform_evidence
            .iter()
            .map(|item| item.provision_id.as_str())
            .collect();
        assert_eq!(
            ids,
            [
                format!("{ITF_ID}:amendment:2024-12-27:modifies:2018-01-23:transitory:unico"),
                format!("{ITF_ID}:amendment:2024-12-27:modifies:2024-04-09:transitory:unico"),
            ]
        );
        // Both keep the ÚNICO label, annotated with the modified resolution.
        assert!(
            document.reform_evidence[0]
                .label
                .contains("modifica resolución DOF 2018-01-23")
        );
    }

    #[test]
    fn a_marker_in_a_trailing_considerando_is_dropped_as_marginalia() {
        // A marker in an appended per-reform CONSIDERANDO annotates
        // non-canonical text; it is redundant with the same marker on the
        // provision it refers to (kept in the legend) and is dropped rather
        // than surfaced as an error. A structural mis-parse instead strands
        // markers at a TRANSITORIOS heading, which stays an error.
        let raw = "Artículo 1.- Objeto de las disposiciones.\n\
                   TRANSITORIOS\n\
                   PRIMERO.- Entra en vigor.\n\
                   CONSIDERANDO\n\
                   Que se reforma lo conducente.\n\
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
        .expect("a trailing-considerando marker is dropped, not an error");
        // It attaches to no provision, but the legend still records it.
        assert!(
            document
                .provisions
                .iter()
                .all(|item| item.amendment_marks.is_empty())
        );
        assert!(
            document
                .amendment_references
                .iter()
                .any(|reference| reference.marker == 6)
        );
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
