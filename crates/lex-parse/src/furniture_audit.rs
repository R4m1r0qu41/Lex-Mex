//! Read-only positional cross-check for PDF page furniture (running
//! headers, footers, and letterheads) that publisher-specific string filters
//! cannot catch. It detects furniture by Y-coordinate repetition across pages
//! and emits evidence only: it never rewrites extracted text and never
//! replaces the canonical Poppler extraction path.
//!
//! Adapted from the REPO-019 (firecrawl/anydoc) Repo Foundry prototype,
//! validated in Agent Vault EXP-REPO-019-004/005. The tool is deliberately a
//! second opinion alongside the canonical extractor.

use pdf_inspector::TextItem;
use serde::Serialize;

/// Coverage threshold separating genuine furniture from body-line
/// coincidence. This is load-bearing, not a tuning knob: the reviewed
/// experiment found genuine bands at coverage 1.0 and false positives between
/// 0.6 and 0.9. Do not lower it without renewing the regression evidence.
pub const FURNITURE_COVERAGE_THRESHOLD: f64 = 0.95;

/// Minimum number of distinct parsed provisions that must contain a recurring
/// header/footer band before ingestion treats it as contamination. One legal
/// citation of an instrument title can legitimately match its running header;
/// the recurring multi-provision shape is the anomaly this audit gates.
pub const ADMITTED_FURNITURE_PROVISION_FLOOR: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Zone {
    Header,
    Footer,
    Body,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BandKind {
    /// One string dominates the band (a title banner or letterhead).
    Static,
    /// Distinct strings share the band (page counters or dates).
    Dynamic,
}

#[derive(Debug, Clone, Serialize)]
pub struct FurnitureBand {
    pub y: i64,
    pub pages: usize,
    pub coverage: f64,
    pub zone: Zone,
    pub kind: BandKind,
    pub distinct_texts: usize,
    pub sample_texts: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct FurnitureReport {
    pub page_count: usize,
    /// Header/footer bands at or above the coverage threshold. Evidence only.
    pub furniture_bands: Vec<FurnitureBand>,
    /// Repeating body-zone bands. Never auto-classified as furniture.
    pub body_zone_repeats: Vec<FurnitureBand>,
}

/// A recurring header/footer band that survived as its own parsed block. This
/// is a potential extraction or parser-boundary failure, not an automatic
/// text-rewrite instruction.
#[derive(Debug, Clone, Serialize)]
pub struct AdmittedFurniture {
    pub band: FurnitureBand,
    pub matched_text: String,
    pub provision_ids: Vec<String>,
}

impl AdmittedFurniture {
    /// Whether this finding has the repeated-provision shape that blocks
    /// ingestion. Smaller overlaps remain in the diagnostic report.
    #[must_use]
    pub fn blocks_ingestion(&self) -> bool {
        self.provision_ids.len() >= ADMITTED_FURNITURE_PROVISION_FLOOR
    }
}

fn normalize(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// True when `furniture` appears verbatim (same casing) at the start or end
/// of `block`, on a word boundary, with other text on the other side.
fn has_exact_edge_match(block: &str, furniture: &str) -> bool {
    let trimmed = block.trim_end_matches(|c: char| c.is_ascii_punctuation() || c.is_whitespace());
    let at_end = trimmed.strip_suffix(furniture).is_some_and(|before| {
        before
            .chars()
            .next_back()
            .is_some_and(|c| !c.is_alphanumeric())
    });
    let at_start = block
        .strip_prefix(furniture)
        .is_some_and(|after| after.chars().next().is_some_and(|c| !c.is_alphanumeric()));
    at_end || at_start
}

fn zone_of(relative_y: f32) -> Zone {
    if relative_y >= 0.88 {
        Zone::Header
    } else if relative_y <= 0.12 {
        Zone::Footer
    } else {
        Zone::Body
    }
}

/// Bucket positional text by rounded Y coordinate, classify each repeating
/// band as static/dynamic, and separate header/footer bands from body repeats.
/// The explicit threshold is solely for the acceptance regression; production
/// callers use the pinned audit entry point.
#[must_use]
pub fn detect_bands(items: &[TextItem], min_coverage: f64) -> FurnitureReport {
    use std::collections::{BTreeMap, BTreeSet};

    let pages: BTreeSet<u32> = items.iter().map(|item| item.page).collect();
    let page_count = pages.len();
    // Recurrence is undefined for a single page: every line has 100%
    // coverage there, so classifying it as furniture would be speculation.
    if page_count < 2 {
        return FurnitureReport {
            page_count,
            furniture_bands: Vec::new(),
            body_zone_repeats: Vec::new(),
        };
    }
    let (mut low_y, mut high_y) = (f32::MAX, f32::MIN);
    for item in items {
        low_y = low_y.min(item.y);
        high_y = high_y.max(item.y);
    }
    let span = (high_y - low_y).max(1.0);

    let mut by_y: BTreeMap<i64, (BTreeSet<u32>, BTreeMap<String, usize>)> = BTreeMap::new();
    for item in items {
        let text = normalize(&item.text);
        if text.is_empty() {
            continue;
        }
        #[allow(clippy::cast_possible_truncation)]
        let y = item.y.round() as i64;
        let entry = by_y.entry(y).or_default();
        entry.0.insert(item.page);
        *entry.1.entry(text).or_insert(0) += 1;
    }

    let mut furniture_bands = Vec::new();
    let mut body_zone_repeats = Vec::new();
    for (y, (pages, texts)) in by_y {
        #[allow(clippy::cast_precision_loss)]
        let coverage = pages.len() as f64 / page_count as f64;
        if coverage < min_coverage {
            continue;
        }
        let dominant_count = texts.values().copied().max().unwrap_or(0);
        #[allow(clippy::cast_precision_loss)]
        let kind = if texts.len() == 1 || dominant_count as f64 >= pages.len() as f64 * 0.8 {
            BandKind::Static
        } else {
            BandKind::Dynamic
        };
        #[allow(clippy::cast_precision_loss)]
        let relative_y = (y as f32 - low_y) / span;
        let mut sample_texts: Vec<String> = texts.keys().cloned().collect();
        let band = FurnitureBand {
            y,
            pages: pages.len(),
            coverage,
            zone: zone_of(relative_y),
            kind,
            distinct_texts: texts.len(),
            sample_texts: {
                sample_texts.truncate(6);
                sample_texts
            },
        };
        if band.zone == Zone::Body {
            body_zone_repeats.push(band);
        } else {
            furniture_bands.push(band);
        }
    }

    FurnitureReport {
        page_count,
        furniture_bands,
        body_zone_repeats,
    }
}

/// Read one PDF positionally and run the detector at the pinned threshold.
/// It performs no network access, subprocess execution, or writes.
pub fn audit_pdf(path: &std::path::Path) -> Result<FurnitureReport, pdf_inspector::PdfError> {
    let items = pdf_inspector::extract_text_with_positions(path)?;
    Ok(detect_bands(&items, FURNITURE_COVERAGE_THRESHOLD))
}

/// Identify header/footer evidence that survived into parsed legal text.
///
/// Body-zone repeats are excluded deliberately: they are reported for review,
/// but may be genuine repeated legal language. The caller decides how to
/// surface a non-empty result; the ingestion pipeline persists it as a
/// diagnostic and stops before writing a candidate corpus.
#[must_use]
pub fn find_admitted_furniture<'a>(
    report: &FurnitureReport,
    texts: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> Vec<AdmittedFurniture> {
    let texts: Vec<_> = texts
        .into_iter()
        .map(|(id, text)| {
            (
                id,
                text.split("\n\n")
                    .map(normalize)
                    .map(|block| (block.to_lowercase(), block))
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    let mut findings = Vec::new();

    for band in &report.furniture_bands {
        for sample in &band.sample_texts {
            let sample_text = normalize(sample);
            let normalized = sample_text.to_lowercase();
            // Page counters and other short fragments are too ambiguous to
            // implicate legal text even when they happen to be repeated.
            if normalized.chars().count() < 12 {
                continue;
            }
            let provision_ids = texts
                .iter()
                // A title can be a substantive citation inside several
                // transitories. The Diputados parser emits source paragraphs
                // as double-newline-delimited blocks, whereas leaked running
                // furniture has its own block. Require that stronger shape so
                // the gate catches parser leakage without treating a legal
                // reference to the instrument title as contamination.
                //
                // Furniture that leaks into a paragraph lands at its edge (the
                // page break follows the last line or precedes the first), in
                // the band's exact casing; a title-case citation or one inside
                // a sentence does neither. Match that shape as well.
                .filter(|(_, blocks)| {
                    blocks.iter().any(|(lowered, original)| {
                        lowered == &normalized || has_exact_edge_match(original, &sample_text)
                    })
                })
                .map(|(id, _)| (*id).to_owned())
                .collect::<Vec<_>>();
            if !provision_ids.is_empty() {
                findings.push(AdmittedFurniture {
                    band: band.clone(),
                    matched_text: sample.clone(),
                    provision_ids,
                });
                break;
            }
        }
    }
    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(page: u32, y: f32, text: &str) -> TextItem {
        TextItem {
            text: text.to_owned(),
            x: 0.0,
            y,
            width: 10.0,
            height: 10.0,
            font: String::new(),
            font_size: 10.0,
            page,
            is_bold: false,
            is_italic: false,
            is_underline: false,
            is_strikeout: false,
            item_type: pdf_inspector::types::ItemType::default(),
            mcid: None,
        }
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn threshold_is_exactly_0_95() {
        assert_eq!(FURNITURE_COVERAGE_THRESHOLD, 0.95);
    }

    #[test]
    fn coverage_threshold_pinned_at_0_95_excludes_the_0_6_false_positive() {
        let mut items = Vec::new();
        for page in 0..10_u32 {
            items.push(item(page, 0.0, &format!("footer {page}")));
            items.push(item(page, 980.0, "DIARIO OFICIAL DE LA FEDERACIÓN"));
            items.push(item(page, 490.0, &format!("body {page}")));
        }
        for page in 0..7_u32 {
            items.push(item(
                page,
                900.0,
                "Los presentes lineamientos tienen por objeto establecer",
            ));
        }

        let loose = detect_bands(&items, 0.6);
        assert!(loose.furniture_bands.iter().any(|band| {
            band.sample_texts
                .iter()
                .any(|text| text.starts_with("Los presentes"))
        }));

        let pinned = detect_bands(&items, FURNITURE_COVERAGE_THRESHOLD);
        assert!(pinned.furniture_bands.iter().all(|band| {
            !band
                .sample_texts
                .iter()
                .any(|text| text.starts_with("Los presentes"))
        }));
        assert!(pinned.furniture_bands.iter().any(|band| {
            band.sample_texts
                .iter()
                .any(|text| text.contains("DIARIO OFICIAL"))
        }));
        assert!(
            pinned
                .body_zone_repeats
                .iter()
                .any(|band| band.zone == Zone::Body)
        );
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn dynamic_band_is_distinguished_from_static() {
        let items = (0..10_u32)
            .map(|page| item(page, 20.0, &format!("{} de 10", page + 1)))
            .collect::<Vec<_>>();
        let report = detect_bands(&items, FURNITURE_COVERAGE_THRESHOLD);
        let counter = report
            .furniture_bands
            .iter()
            .find(|band| band.zone == Zone::Footer)
            .expect("page counter must be detected as a footer band");
        assert_eq!(counter.kind, BandKind::Dynamic);
        assert_eq!(counter.coverage, 1.0);
    }

    #[test]
    fn a_single_page_cannot_establish_a_recurring_band() {
        let report = detect_bands(
            &[
                item(0, 980.0, "DIARIO OFICIAL DE LA FEDERACIÓN"),
                item(0, 490.0, "cuerpo del documento"),
            ],
            FURNITURE_COVERAGE_THRESHOLD,
        );

        assert_eq!(report.page_count, 1);
        assert!(report.furniture_bands.is_empty());
        assert!(report.body_zone_repeats.is_empty());
    }

    #[test]
    fn standalone_header_footer_block_is_reported_but_embedded_legal_citation_is_not() {
        let mut items = Vec::new();
        for page in 0..10_u32 {
            items.push(item(page, 980.0, "DIARIO OFICIAL DE LA FEDERACIÓN"));
            items.push(item(page, 490.0, "repetición legítima del cuerpo"));
            items.push(item(page, 0.0, &format!("{} de 10", page + 1)));
        }
        let report = detect_bands(&items, FURNITURE_COVERAGE_THRESHOLD);
        let findings = find_admitted_furniture(
            &report,
            [
                (
                    "urn:test:1",
                    "Texto legal.\n\nDIARIO OFICIAL DE LA FEDERACIÓN\n\nContinúa el texto.",
                ),
                (
                    "urn:test:2",
                    "La Ley se publicó en el DIARIO OFICIAL DE LA FEDERACIÓN para surtir efectos.",
                ),
            ],
        );

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].provision_ids, vec!["urn:test:1"]);
        assert_eq!(findings[0].band.zone, Zone::Header);
        assert!(!findings[0].blocks_ingestion());
    }

    #[test]
    fn furniture_leaked_at_a_paragraph_edge_is_reported_but_citations_are_not() {
        let report = FurnitureReport {
            page_count: 3,
            furniture_bands: vec![FurnitureBand {
                y: 980,
                pages: 3,
                coverage: 1.0,
                zone: Zone::Header,
                kind: BandKind::Static,
                distinct_texts: 1,
                sample_texts: vec!["DIARIO OFICIAL DE LA FEDERACIÓN".to_owned()],
            }],
            body_zone_repeats: Vec::new(),
        };
        for leaked in [
            "Texto legal. DIARIO OFICIAL DE LA FEDERACIÓN",
            "Texto legal.\nDIARIO OFICIAL DE LA FEDERACIÓN.",
            "DIARIO OFICIAL DE LA FEDERACIÓN Continúa el texto legal.",
        ] {
            let findings = find_admitted_furniture(&report, [("urn:test:leak", leaked)]);
            assert_eq!(findings.len(), 1, "{leaked:?}");
        }
        for citation in [
            "La Ley se publicó en el DIARIO OFICIAL DE LA FEDERACIÓN para surtir efectos.",
            "Publicado en el Diario Oficial de la Federación",
            "El Diario Oficial de la Federación publicará el aviso.",
            "NODIARIO OFICIAL DE LA FEDERACIÓN",
        ] {
            let findings = find_admitted_furniture(&report, [("urn:test:cite", citation)]);
            assert!(findings.is_empty(), "{citation:?}");
        }
    }

    #[test]
    fn repeated_header_in_three_provisions_blocks_ingestion() {
        let report = FurnitureReport {
            page_count: 3,
            furniture_bands: vec![FurnitureBand {
                y: 980,
                pages: 3,
                coverage: 1.0,
                zone: Zone::Header,
                kind: BandKind::Static,
                distinct_texts: 1,
                sample_texts: vec!["DIARIO OFICIAL DE LA FEDERACIÓN".to_owned()],
            }],
            body_zone_repeats: Vec::new(),
        };
        let findings = find_admitted_furniture(
            &report,
            [
                ("urn:test:1", "DIARIO OFICIAL DE LA FEDERACIÓN"),
                ("urn:test:2", "DIARIO OFICIAL DE LA FEDERACIÓN"),
                ("urn:test:3", "DIARIO OFICIAL DE LA FEDERACIÓN"),
            ],
        );

        assert_eq!(findings.len(), 1);
        assert!(findings[0].blocks_ingestion());
    }
}
