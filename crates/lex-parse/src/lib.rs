pub mod dcg;
pub mod diputados;
pub mod furniture_audit;
pub mod html;
pub mod itf;
pub mod standard;
pub mod terms;

pub use dcg::parse_dcg;
pub use diputados::{
    DiputadosDocument, DiputadosOptions, extract_dof_publication, extract_reform_evidence,
    parse_diputados, parse_diputados_publication,
};
pub use furniture_audit::{audit_pdf, find_admitted_furniture};
pub use html::extract_html_text;
pub use itf::{ItfDocument, parse_itf_dcg};
pub use lex_grammar::labels;
pub use standard::{
    StandardModificationTarget, parse_standard_clauses, parse_standard_modification_targets,
    parse_standard_supplements, parse_standard_transitories, strip_page_furniture,
    validate_standard,
};
pub use temporal_derive::{
    Commencement, DerivationOutcome, SkipReason, derive_article_temporal_determinations,
    repair_article_dates, repair_partial_repeals,
};
pub use terms::{
    GlossaryDetection, GlossaryHold, GlossaryStyle, detect_glossary, detect_glossary_terms,
    extract_glossaries, extract_quoted_term_usages, extract_term_usages,
    extract_term_usages_with_lowercase, extract_terms, find_glossary_provision,
    invalidated_fractions, repeal_placeholders, unread_fractions,
};

mod extraction;
mod reform;
mod standard_validate;
pub use extraction::{Extraction, extract_doc, extract_pdf};
pub use lex_analysis::temporal_derive;
pub use lex_analysis::{
    CorpusExpectations, CorpusView, apply_legal_operator_reviews,
    apply_outdated_reference_approvals, validate_corpus, validate_lritf,
};
pub(crate) use lex_analysis::{initial_repeals, initial_temporal_status};
pub(crate) use lex_grammar::{collapse_whitespace, slug, spanish_date};
pub use lex_link::{
    HistoricalTarget, InstrumentContextPolicy, ReferenceOptions, extract_internal_references,
    extract_references,
};
pub(crate) use reform::reform_evidence_item;

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use chrono::NaiveDate;
    use lex_core::{
        Provision, ReferenceForm, ReferenceQualifierType, ReferenceResolutionStatus, TemporalStatus,
    };
    use pretty_assertions::assert_eq;

    use super::{
        CorpusExpectations, CorpusView, DiputadosOptions, HistoricalTarget,
        InstrumentContextPolicy, ReferenceOptions, extract_internal_references, extract_references,
        extract_reform_evidence, parse_dcg, parse_diputados, validate_corpus, validate_lritf,
    };

    const FIXTURE: &str = include_str!("../../../fixtures/lritf/parser-sample.txt");
    const REFERENCE_FIXTURE: &str = include_str!("../../../fixtures/lritf/reference-sample.txt");
    const RELATIVE_FIXTURE: &str =
        include_str!("../../../fixtures/lritf/relative-reference-sample.txt");
    const RELATIVE_PRECEDES_FIXTURE: &str =
        include_str!("../../../fixtures/lritf/relative-precedes-reference-sample.txt");
    const DCG_FIXTURE: &str = include_str!("../../../fixtures/ifpe-dcg-2021/parser-sample.txt");
    const DCG_ANNEX_1_FIXTURE: &str =
        include_str!("../../../fixtures/ifpe-dcg-2021/annex-1-sample.txt");
    const MARKER_LOOKAHEAD_FIXTURE: &str =
        include_str!("../../../fixtures/diputados/reference-marker-lookahead-sample.txt");
    const ORDINAL_TITLE_FIXTURE: &str =
        include_str!("../../../fixtures/diputados/reference-ordinal-title-sample.txt");
    const NUMERIC_ORDINAL_QUALIFIER_FIXTURE: &str =
        include_str!("../../../fixtures/diputados/reference-numeric-ordinal-qualifier-sample.txt");
    const CONSTITUTIONAL_ARTICLE_LIST_FIXTURE: &str = include_str!(
        "../../../fixtures/diputados/reference-constitutional-article-list-sample.txt"
    );
    const DCG_ID: &str = "urn:lex-mx:federal:regulation:ifpe-dcg-2021";
    const LRITF_ID: &str = "urn:lex-mx:federal:statute:lritf";
    const LOOKAHEAD_ID: &str = "urn:lex-mx:federal:statute:marker-lookahead-fixture";
    const LOCG_ID: &str = "urn:lex-mx:federal:statute:locg";
    const LPDUSF_ID: &str = "urn:lex-mx:federal:statute:lpdusf";
    const RGIC_ID: &str = "urn:lex-mx:federal:regulation:rgic";
    const ORDINAL_TITLE_ID: &str = "urn:lex-mx:federal:statute:ordinal-title-fixture";
    const CPEUM_ID: &str = "urn:lex-mx:federal:constitution:cpeum";
    const HISTORICAL_LRAF_ID: &str = "urn:lex-mx:federal:statute:lraf-1990";

    /// The exact options the CLI derives for the committed LRITF adapter;
    /// the committed corpus is the byte-identity fixture for them.
    fn lritf_options() -> DiputadosOptions {
        DiputadosOptions {
            instrument_id: LRITF_ID.to_owned(),
            header_lines: vec![
                "LEY PARA REGULAR LAS INSTITUCIONES DE TECNOLOGÍA FINANCIERA".to_owned(),
            ],
            stop_markers: vec!["ARTÍCULOS SEGUNDO A DÉCIMO".to_owned()],
            annex_markers: Vec::new(),
            separate_enactment: false,
            nested_instrument: None,
            separatorless_article_headings: Vec::new(),
        }
    }

    fn parse_lritf(
        raw: &str,
        date: chrono::NaiveDate,
    ) -> Result<Vec<lex_core::Provision>, anyhow::Error> {
        parse_diputados(raw, &lritf_options(), date).map(|document| document.provisions)
    }

    fn dcg_reference_options() -> ReferenceOptions {
        ReferenceOptions {
            policy: InstrumentContextPolicy::SentenceEarliestMarker {
                internal_markers: [
                    "de estas disposiciones",
                    "de las presentes disposiciones",
                    "del presente instrumento",
                    "de este instrumento",
                ]
                .iter()
                .map(|marker| (*marker).to_owned())
                .collect(),
                external_instruments: vec![(
                    "ley para regular las instituciones de tecnología financiera".to_owned(),
                    LRITF_ID.to_owned(),
                )],
            },
            transitory_citations: true,
            same_article_fractions: true,
            relative_references: true,
            historical_targets: Vec::new(),
            parent_instrument: None,
        }
    }

    fn marker_lookahead_options() -> ReferenceOptions {
        ReferenceOptions {
            policy: InstrumentContextPolicy::SentenceEarliestMarker {
                internal_markers: ["de esta ley", "de la presente ley", "esta ley"]
                    .iter()
                    .map(|marker| (*marker).to_owned())
                    .collect(),
                external_instruments: vec![
                    (
                        "ley orgánica del congreso general de los estados unidos mexicanos"
                            .to_owned(),
                        LOCG_ID.to_owned(),
                    ),
                    (
                        "ley de protección y defensa al usuario de servicios financieros"
                            .to_owned(),
                        LPDUSF_ID.to_owned(),
                    ),
                    (
                        "reglamento para el gobierno interior del congreso general de los estados unidos mexicanos"
                            .to_owned(),
                        RGIC_ID.to_owned(),
                    ),
                ],
            },
            transitory_citations: true,
            same_article_fractions: true,
            relative_references: true,
            historical_targets: Vec::new(),
            parent_instrument: None,
        }
    }

    const DCG_TITLE: &str = "Disposiciones aplicables a las instituciones de fondos de pago electrónico \
         a que se refieren los artículos 48, segundo párrafo; 54, primer párrafo, y \
         56, primer y segundo párrafos de la Ley para Regular las Instituciones de \
         Tecnología Financiera";

    fn dcg_fixture_graph() -> (
        Vec<lex_core::Provision>,
        Vec<lex_core::ReferenceEdge>,
        HashSet<String>,
    ) {
        let date = NaiveDate::from_ymd_opt(2021, 1, 28).unwrap();
        let provisions = parse_dcg(
            DCG_FIXTURE,
            &[(1, DCG_ANNEX_1_FIXTURE.to_owned())],
            DCG_ID,
            date,
            &["1".to_owned()],
        )
        .unwrap();
        let mut known_targets: HashSet<String> =
            provisions.iter().map(|item| item.id.clone()).collect();
        for target in [
            format!("{LRITF_ID}:article:48"),
            format!("{LRITF_ID}:article:54"),
            format!("{LRITF_ID}:article:56"),
            format!("{LRITF_ID}:transitory:octava"),
            // Article 15 exists in the full DCG though not in this sample;
            // its internal citation resolves in production and so must be
            // a known target here, otherwise the dropped-unresolved rule
            // removes the edge under test.
            format!("{DCG_ID}:article:15"),
        ] {
            known_targets.insert(target);
        }
        let references = extract_references(
            &provisions,
            Some((DCG_ID, DCG_TITLE)),
            &dcg_reference_options(),
            &known_targets,
        )
        .unwrap();
        (provisions, references, known_targets)
    }

    #[test]
    fn resolves_cross_instrument_and_title_references_deterministically() {
        let (_provisions, references, _known_targets) = dcg_fixture_graph();

        // Title citations resolve against LRITF with their paragraph
        // qualifiers preserved.
        let title_edges: Vec<_> = references
            .iter()
            .filter(|edge| edge.source_provision_id == DCG_ID)
            .collect();
        assert_eq!(title_edges.len(), 3);
        assert_eq!(
            title_edges
                .iter()
                .map(|edge| edge.target_provision_id.as_str())
                .collect::<Vec<_>>(),
            [
                format!("{LRITF_ID}:article:48"),
                format!("{LRITF_ID}:article:54"),
                format!("{LRITF_ID}:article:56"),
            ]
        );
        assert_eq!(title_edges[0].qualifiers[0].text, "segundo párrafo");
        assert_eq!(
            title_edges[2].qualifiers[0].text,
            "primer y segundo párrafos"
        );
        assert!(
            title_edges
                .iter()
                .all(|edge| edge.resolution_status == ReferenceResolutionStatus::Resolved)
        );

        // A full-name LRITF citation inside Article 1 resolves cross-instrument.
        assert!(references.iter().any(|edge| {
            edge.source_provision_id.ends_with(":article:1")
                && edge.target_provision_id == format!("{LRITF_ID}:article:48")
                && edge.resolution_status == ReferenceResolutionStatus::Resolved
        }));

        // The short-form defined-term citation "artículo 22, fracción I de la
        // Ley," must not create any edge (regression: it previously became a
        // false internal edge).
        assert!(
            !references
                .iter()
                .any(|edge| edge.target_provision_id.ends_with(":article:22"))
        );

        // Internal citations with DCG markers stay internal.
        assert!(references.iter().any(|edge| {
            edge.source_provision_id.ends_with(":transitory:segundo")
                && edge.target_provision_id == format!("{DCG_ID}:article:15")
        }));

        // CUARTO cites LRITF's OCTAVA transitory expressly.
        let octava = references
            .iter()
            .find(|edge| edge.target_provision_id == format!("{LRITF_ID}:transitory:octava"))
            .expect("transitory citation extracted");
        assert!(octava.source_provision_id.ends_with(":transitory:cuarto"));
        assert_eq!(octava.source_span, "OCTAVA Transitoria");
        assert_eq!(
            octava.resolution_status,
            ReferenceResolutionStatus::Resolved
        );

        // Named external laws that are not configured stay unlinked.
        assert!(
            !references
                .iter()
                .any(|edge| edge.target_provision_id.contains("codigo"))
        );
    }

    #[test]
    fn a_citation_in_text_amended_after_the_successor_resolves_to_the_successor() {
        let source_id = "urn:lex-mx:federal:statute:lrsic";
        let successor = "urn:lex-mx:federal:statute:lraf";
        let document = parse_diputados(
            "Artículo 1.- Se derogan los artículos 33 de la Ley para Regular las Agrupaciones Financieras.\nArtículo reformado DOF 15-03-2016",
            &DiputadosOptions {
                instrument_id: source_id.to_owned(),
                header_lines: Vec::new(),
                stop_markers: Vec::new(),
                annex_markers: Vec::new(), separate_enactment: false, nested_instrument: None,
                separatorless_article_headings: Vec::new(),
            },
            NaiveDate::from_ymd_opt(2002, 1, 15).unwrap(),
        )
        .unwrap();
        let known_targets = HashSet::from([format!("{successor}:article:33")]);
        let references = extract_references(
            &document.provisions,
            None,
            &ReferenceOptions {
                policy: InstrumentContextPolicy::SentenceEarliestMarker {
                    internal_markers: vec!["de esta ley".to_owned()],
                    external_instruments: vec![(
                        "ley para regular las agrupaciones financieras".to_owned(),
                        HISTORICAL_LRAF_ID.to_owned(),
                    )],
                },
                transitory_citations: true,
                same_article_fractions: true,
                relative_references: true,
                historical_targets: vec![HistoricalTarget {
                    id: HISTORICAL_LRAF_ID.to_owned(),
                    successor_id: successor.to_owned(),
                    until: NaiveDate::from_ymd_opt(2014, 1, 10).unwrap(),
                }],
                parent_instrument: None,
            },
            &known_targets,
        )
        .unwrap();
        assert_eq!(references.len(), 1);
        assert_eq!(references[0].target_instrument_id, successor);
        assert_eq!(
            references[0].resolution_status,
            ReferenceResolutionStatus::Resolved
        );
    }

    #[test]
    fn preserves_same_title_historical_citations_without_linking_the_successor_law() {
        let source_id = "urn:lex-mx:federal:statute:lrsic";
        let date = NaiveDate::from_ymd_opt(2002, 1, 15).unwrap();
        let document = parse_diputados(
            "Artículo 1.- Se derogan los artículos 33, 33-A y 33-B de la Ley para Regular las Agrupaciones Financieras.",
            &DiputadosOptions {
                instrument_id: source_id.to_owned(),
                header_lines: Vec::new(),
                stop_markers: Vec::new(),
                annex_markers: Vec::new(), separate_enactment: false, nested_instrument: None,
                separatorless_article_headings: Vec::new(),
            },
            date,
        )
        .unwrap();
        let successor_target = "urn:lex-mx:federal:statute:lraf:article:33".to_owned();
        let known_targets = HashSet::from([successor_target]);
        let references = extract_references(
            &document.provisions,
            None,
            &ReferenceOptions {
                policy: InstrumentContextPolicy::SentenceEarliestMarker {
                    internal_markers: vec!["de esta ley".to_owned()],
                    external_instruments: vec![(
                        "ley para regular las agrupaciones financieras".to_owned(),
                        HISTORICAL_LRAF_ID.to_owned(),
                    )],
                },
                transitory_citations: true,
                same_article_fractions: true,
                relative_references: true,
                historical_targets: vec![HistoricalTarget {
                    id: HISTORICAL_LRAF_ID.to_owned(),
                    successor_id: "urn:lex-mx:federal:statute:lraf".to_owned(),
                    until: NaiveDate::from_ymd_opt(2014, 1, 10).unwrap(),
                }],
                parent_instrument: None,
            },
            &known_targets,
        )
        .unwrap();

        assert_eq!(references.len(), 3);
        assert!(references.iter().all(|reference| {
            reference.target_instrument_id == HISTORICAL_LRAF_ID
                && reference
                    .target_provision_id
                    .starts_with(HISTORICAL_LRAF_ID)
                && reference.resolution_status
                    == ReferenceResolutionStatus::HistoricalTargetUnavailable
        }));
        assert!(
            references
                .iter()
                .all(|reference| reference.target_provision_id
                    != "urn:lex-mx:federal:statute:lraf:article:33")
        );

        let report = validate_corpus(
            &CorpusView {
                instrument_id: source_id,
                official_title: None,
                provisions: &document.provisions,
                references: &references,
                terms: &[],
                term_usages: &[],
                amendment_references: &[],
            },
            &CorpusExpectations {
                min_articles: Some(1),
                articles: Some(1),
                transitories: Some(0),
                annexes: 0,
                require_chapter_context: false,
                allow_article_gaps: false,
            },
            &HashSet::new(),
            &HashSet::new(),
        );
        assert!(report.valid);
        assert_eq!(
            report
                .issues
                .iter()
                .filter(|issue| issue.code == "historical_reference_target_unavailable")
                .count(),
            3
        );
    }

    #[test]
    fn ordinal_title_citation_keeps_following_external_instrument_context() {
        let target = format!("{CPEUM_ID}:article:6");
        let known_targets = HashSet::from([target.clone()]);
        let references = extract_references(
            &[],
            Some((ORDINAL_TITLE_ID, ORDINAL_TITLE_FIXTURE.trim())),
            &ReferenceOptions {
                policy: InstrumentContextPolicy::SentenceEarliestMarker {
                    internal_markers: ["de esta ley", "de la presente ley", "esta ley"]
                        .iter()
                        .map(|marker| (*marker).to_owned())
                        .collect(),
                    external_instruments: vec![(
                        "constitución política de los estados unidos mexicanos".to_owned(),
                        CPEUM_ID.to_owned(),
                    )],
                },
                transitory_citations: false,
                same_article_fractions: false,
                relative_references: false,
                historical_targets: Vec::new(),
                parent_instrument: None,
            },
            &known_targets,
        )
        .unwrap();

        assert_eq!(references.len(), 1);
        assert_eq!(references[0].source_provision_id, ORDINAL_TITLE_ID);
        assert_eq!(references[0].source_span, "6");
        assert_eq!(references[0].target_instrument_id, CPEUM_ID);
        assert_eq!(references[0].target_provision_id, target);
        assert_eq!(
            references[0].resolution_status,
            ReferenceResolutionStatus::Resolved
        );
    }

    #[test]
    fn numeric_ordinal_qualifier_keeps_external_instrument_context() {
        let target = format!("{CPEUM_ID}:article:115");
        let known_targets = HashSet::from([target.clone()]);
        let references = extract_references(
            &[],
            Some((ORDINAL_TITLE_ID, NUMERIC_ORDINAL_QUALIFIER_FIXTURE.trim())),
            &ReferenceOptions {
                policy: InstrumentContextPolicy::SentenceEarliestMarker {
                    internal_markers: ["de esta ley", "de la presente ley", "esta ley"]
                        .iter()
                        .map(|marker| (*marker).to_owned())
                        .collect(),
                    external_instruments: vec![(
                        "constitución general de la república".to_owned(),
                        CPEUM_ID.to_owned(),
                    )],
                },
                transitory_citations: false,
                same_article_fractions: false,
                relative_references: false,
                historical_targets: Vec::new(),
                parent_instrument: None,
            },
            &known_targets,
        )
        .unwrap();

        assert_eq!(references.len(), 1);
        assert_eq!(references[0].source_span, "115");
        assert_eq!(references[0].target_provision_id, target);
        assert_eq!(
            references[0]
                .qualifiers
                .iter()
                .map(|qualifier| qualifier.text.as_str())
                .collect::<Vec<_>>(),
            ["fracción III", "inciso b", "2o. párrafo"]
        );
    }

    #[test]
    fn constitutional_article_list_keeps_numbers_and_noun_first_qualifiers() {
        let targets = [109, 110, 114].map(|number| format!("{CPEUM_ID}:article:{number}"));
        let known_targets = targets.iter().cloned().collect();
        let references = extract_references(
            &[],
            Some((ORDINAL_TITLE_ID, CONSTITUTIONAL_ARTICLE_LIST_FIXTURE.trim())),
            &ReferenceOptions {
                policy: InstrumentContextPolicy::SentenceEarliestMarker {
                    internal_markers: Vec::new(),
                    external_instruments: vec![("constitucional".to_owned(), CPEUM_ID.to_owned())],
                },
                transitory_citations: false,
                same_article_fractions: false,
                relative_references: false,
                historical_targets: Vec::new(),
                parent_instrument: None,
            },
            &known_targets,
        )
        .unwrap();

        assert_eq!(
            references
                .iter()
                .map(|reference| reference.target_provision_id.as_str())
                .collect::<Vec<_>>(),
            targets.iter().map(String::as_str).collect::<Vec<_>>()
        );
        assert_eq!(references[0].qualifiers[0].text, "fracción I");
        assert_eq!(references[1].qualifiers[0].text, "párrafo segundo");
        assert_eq!(references[2].qualifiers[0].text, "párrafo primero");
    }

    #[test]
    fn bounds_late_markers_without_truncating_long_official_titles() {
        let date = NaiveDate::from_ymd_opt(2026, 7, 14).unwrap();
        let provisions = parse_diputados(
            MARKER_LOOKAHEAD_FIXTURE,
            &DiputadosOptions {
                instrument_id: LOOKAHEAD_ID.to_owned(),
                header_lines: Vec::new(),
                stop_markers: Vec::new(),
                annex_markers: Vec::new(),
                separate_enactment: false,
                nested_instrument: None,
                separatorless_article_headings: Vec::new(),
            },
            date,
        )
        .unwrap()
        .provisions;
        let mut known_targets: HashSet<String> =
            provisions.iter().map(|item| item.id.clone()).collect();
        for suffix in ["48-bis-5", "94-bis", "96-bis"] {
            known_targets.insert(format!("{LOOKAHEAD_ID}:article:{suffix}"));
        }
        known_targets.insert(format!("{LOCG_ID}:article:20"));
        known_targets.insert(format!("{RGIC_ID}:article:89"));

        let references = extract_references(
            &provisions,
            None,
            &marker_lookahead_options(),
            &known_targets,
        )
        .unwrap();

        for suffix in ["48-bis-5", "94-bis", "96-bis"] {
            assert!(references.iter().any(|edge| {
                edge.target_provision_id == format!("{LOOKAHEAD_ID}:article:{suffix}")
                    && edge.resolution_status == ReferenceResolutionStatus::Resolved
            }));
        }
        assert!(references.iter().any(|edge| {
            edge.target_provision_id == format!("{LOCG_ID}:article:20")
                && edge.resolution_status == ReferenceResolutionStatus::Resolved
        }));
        assert!(references.iter().any(|edge| {
            edge.target_provision_id == format!("{RGIC_ID}:article:89")
                && edge.resolution_status == ReferenceResolutionStatus::Resolved
        }));
        assert!(
            references
                .iter()
                .all(|edge| edge.target_instrument_id != LPDUSF_ID)
        );
    }

    #[test]
    fn captures_pre_number_qualifiers_and_same_article_fractions() {
        let (provisions, references, known_targets) = dcg_fixture_graph();

        // A qualifier written before the number — `la fracción XI del
        // artículo 36` in Anexo 1 — is captured with its anchored span.
        let annex_citation = references
            .iter()
            .find(|edge| {
                edge.source_provision_id.ends_with(":annex:1")
                    && edge.target_provision_id.ends_with(":article:36")
            })
            .expect("annex 1 cites article 36");
        assert_eq!(annex_citation.qualifiers.len(), 1);
        let qualifier = &annex_citation.qualifiers[0];
        assert_eq!(qualifier.text, "fracción XI");
        assert_eq!(qualifier.qualifier_type, ReferenceQualifierType::Fraction);
        let (start, end) = (
            qualifier.start_char.expect("anchored"),
            qualifier.end_char.expect("anchored"),
        );
        let annex_text = &provisions
            .iter()
            .find(|item| item.id.ends_with(":annex:1"))
            .expect("annex 1")
            .text;
        let span: String = annex_text.chars().skip(start).take(end - start).collect();
        assert_eq!(span, "fracción XI");

        // `las fracciones III y IV de este artículo` in Article 36 becomes
        // one self-targeting edge per numeral, each spanning the numeral.
        let same_article: Vec<_> = references
            .iter()
            .filter(|edge| {
                edge.source_provision_id.ends_with(":article:36")
                    && edge.target_provision_id == edge.source_provision_id
            })
            .collect();
        assert_eq!(
            same_article
                .iter()
                .map(|edge| edge.source_span.as_str())
                .collect::<Vec<_>>(),
            ["III", "IV"]
        );
        assert!(same_article.iter().all(|edge| {
            edge.resolution_status == ReferenceResolutionStatus::Resolved
                && edge.qualifiers.len() == 1
        }));

        // The full graph passes validation, including qualifier spans.
        let report = super::validate_corpus(
            &super::CorpusView {
                instrument_id: DCG_ID,
                official_title: Some(DCG_TITLE),
                provisions: &provisions,
                references: &references,
                terms: &[],
                term_usages: &[],
                amendment_references: &[],
            },
            &super::CorpusExpectations::default(),
            &known_targets,
            &HashSet::new(),
        );
        // The fixture is an excerpt, so count/order/unresolved-target
        // issues are expected; span integrity must hold regardless.
        assert!(
            !report.issues.iter().any(|issue| {
                matches!(
                    issue.code.as_str(),
                    "qualifier_span_mismatch"
                        | "reference_span_mismatch"
                        | "reference_offsets_invalid"
                )
            }),
            "{:?}",
            report.issues
        );
    }

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
    fn validation_rejects_an_effective_status_without_temporal_analysis() {
        let date = NaiveDate::from_ymd_opt(2018, 3, 9).unwrap();
        let mut provisions = parse_lritf(FIXTURE, date).unwrap();
        provisions[0].temporal_status = TemporalStatus::Effective;

        let report = validate_lritf(&provisions, &[], 2, 2);

        assert!(!report.valid, "{:?}", report.issues);
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.code == "unanalyzed_temporal_status")
        );
    }

    const GAPPED_CODIGO: &str = "Artículo 1o.- Uno.\n\nArtículo 2o.- Dos.\n\nArtículo 15.- Quince.\n\nArtículo 15-A.- Quince A.\n\nArtículo 17.- Diecisiete.\n\nTRANSITORIOS\n\nPRIMERO.- Entrará en vigor.\n";

    fn validate_gapped(expectations: &crate::CorpusExpectations) -> lex_core::ValidationReport {
        validate_sample(GAPPED_CODIGO, expectations)
    }

    fn validate_sample(
        text: &str,
        expectations: &crate::CorpusExpectations,
    ) -> lex_core::ValidationReport {
        validate_sample_with(text, expectations, |_| {})
    }

    fn validate_sample_with(
        text: &str,
        expectations: &crate::CorpusExpectations,
        reorder: impl FnOnce(&mut Vec<lex_core::Provision>),
    ) -> lex_core::ValidationReport {
        let date = NaiveDate::from_ymd_opt(1981, 12, 31).unwrap();
        let options = DiputadosOptions {
            instrument_id: "urn:lex-mx:federal:code:sample".to_owned(),
            header_lines: vec!["CÓDIGO DE MUESTRA".to_owned()],
            stop_markers: Vec::new(),
            annex_markers: Vec::new(),
            separate_enactment: false,
            nested_instrument: None,
            separatorless_article_headings: Vec::new(),
        };
        let mut document = parse_diputados(text, &options, date).unwrap();
        reorder(&mut document.provisions);
        crate::validate_corpus(
            &crate::CorpusView {
                instrument_id: "urn:lex-mx:federal:code:sample",
                official_title: None,
                provisions: &document.provisions,
                references: &[],
                terms: &[],
                term_usages: &[],
                amendment_references: &[],
            },
            expectations,
            &std::collections::HashSet::new(),
            &std::collections::HashSet::new(),
        )
    }

    #[test]
    fn gap_tolerant_ordering_accepts_suffixes_and_warns_on_gaps() {
        let report = validate_gapped(&crate::CorpusExpectations {
            min_articles: Some(5),
            articles: Some(5),
            transitories: Some(1),
            annexes: 0,
            require_chapter_context: false,
            allow_article_gaps: true,
        });
        assert!(report.valid, "{:?}", report.issues);
        let gaps: Vec<&str> = report
            .issues
            .iter()
            .filter(|issue| issue.code == "article_gap")
            .filter_map(|issue| issue.provision_id.as_deref())
            .collect();
        // 2o -> 15 and 15-A -> 17 skip base numbers; 15 -> 15-A does not.
        assert_eq!(
            gaps,
            [
                "urn:lex-mx:federal:code:sample:article:15",
                "urn:lex-mx:federal:code:sample:article:17"
            ]
        );
        assert!(
            !report
                .issues
                .iter()
                .any(|issue| issue.code == "non_numeric_article"),
            "suffixed articles are first-class under the label grammar"
        );
    }

    #[test]
    fn strict_ordering_checks_suffixes_by_sort_key() {
        let strict = |articles| crate::CorpusExpectations {
            min_articles: None,
            articles: Some(articles),
            transitories: Some(1),
            annexes: 0,
            require_chapter_context: false,
            allow_article_gaps: false,
        };
        let text = |body: &str| {
            format!("CÓDIGO DE MUESTRA\n\n{body}\n\nTRANSITORIOS\n\nPRIMERO.- Entrará en vigor.\n")
        };
        let ordered = validate_sample(
            &text("Artículo 1.- Uno.\n\nArtículo 1-A.- Uno A.\n\nArtículo 2.- Dos."),
            &strict(3),
        );
        assert!(ordered.valid, "{:?}", ordered.issues);
        assert!(
            ordered
                .issues
                .iter()
                .all(|i| i.code != "non_numeric_article" && i.code != "suffix_order")
        );
        // Same-base suffixes out of order warn; they do not fail the report.
        let swapped = validate_sample(
            &text(
                "Artículo 1.- Uno.\n\nArtículo 1-B.- Uno B.\n\nArtículo 1-A.- Uno A.\n\nArtículo 2.- Dos.",
            ),
            &strict(4),
        );
        assert!(
            swapped.issues.iter().any(|i| i.code == "suffix_order"),
            "{:?}",
            swapped.issues
        );
        // A plain article after a suffix of the same or a later base. The
        // parser never emits these orders, so relabel the parsed provisions.
        let relabel = |second: &'static str, third: &'static str| {
            validate_sample_with(
                &text("Artículo 1.- Uno.\n\nArtículo 2.- Dos.\n\nArtículo 3.- Tres."),
                &strict(3),
                move |provisions| {
                    provisions[1].number = second.to_owned();
                    provisions[2].number = third.to_owned();
                },
            )
        };
        let same_base = relabel("2-A", "2");
        assert!(
            same_base.issues.iter().any(|i| i.code == "suffix_order"),
            "{:?}",
            same_base.issues
        );
        let later_base = relabel("3-A", "2");
        assert!(
            later_base.issues.iter().any(|i| i.code == "article_order"),
            "{:?}",
            later_base.issues
        );
        let ordered_again = relabel("2", "3");
        assert!(ordered_again.valid && ordered_again.issues.is_empty());
        // A suffixed article whose base runs backwards is an error. The
        // parser never emits this order, so reorder the parsed provisions.
        let backwards = validate_sample_with(
            &text("Artículo 1.- Uno.\n\nArtículo 1-A.- Uno A.\n\nArtículo 2.- Dos."),
            &strict(3),
            |provisions| provisions.swap(1, 2),
        );
        assert!(
            backwards.issues.iter().any(|i| i.code == "article_order"),
            "{:?}",
            backwards.issues
        );
    }

    #[test]
    fn exact_only_article_baseline_is_frozen_without_a_minimum() {
        // A new-style frozen adapter (freeze_adapter_baseline writes only
        // `expected_articles`, leaving `expected_min_articles` unset) must
        // not carry a spurious counts_not_frozen warning: the article-count
        // baseline is frozen once either the minimum or the exact count is
        // set, not only when both are.
        let report = validate_gapped(&crate::CorpusExpectations {
            min_articles: None,
            articles: Some(5),
            transitories: Some(1),
            annexes: 0,
            require_chapter_context: false,
            allow_article_gaps: true,
        });
        assert!(report.valid, "{:?}", report.issues);
        assert!(
            !report
                .issues
                .iter()
                .any(|issue| issue.code == "counts_not_frozen"),
            "{:?}",
            report.issues
        );
    }

    #[test]
    fn unfrozen_count_baselines_warn_without_failing() {
        let report = validate_gapped(&crate::CorpusExpectations {
            min_articles: None,
            articles: None,
            transitories: None,
            annexes: 0,
            require_chapter_context: false,
            allow_article_gaps: true,
        });
        assert!(report.valid, "{:?}", report.issues);
        assert_eq!(
            report
                .issues
                .iter()
                .filter(|issue| issue.code == "counts_not_frozen")
                .count(),
            2
        );
    }

    #[test]
    fn strict_ordering_still_rejects_gaps_when_not_allowed() {
        let report = validate_gapped(&crate::CorpusExpectations {
            min_articles: Some(5),
            articles: None,
            transitories: Some(1),
            annexes: 0,
            require_chapter_context: false,
            allow_article_gaps: false,
        });
        assert!(!report.valid, "{:?}", report.issues);
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.code == "article_order")
        );
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
    fn explicit_missing_self_targets_remain_unresolved_but_implicit_candidates_are_suppressed() {
        let date = NaiveDate::from_ymd_opt(2018, 3, 9).unwrap();
        let mut provisions = parse_lritf(RELATIVE_FIXTURE, date).unwrap();
        provisions[1].text = "Se aplicarán los artículos 998 y 999 de esta Ley.".to_owned();
        let references = extract_internal_references(&provisions).unwrap();
        let edges: Vec<_> = references
            .iter()
            .filter(|edge| edge.source_provision_id == provisions[1].id)
            .collect();
        assert_eq!(edges.len(), 2);
        assert!(
            edges
                .iter()
                .all(|edge| edge.resolution_status == ReferenceResolutionStatus::Unresolved)
        );
        let report = validate_lritf(&provisions, &references, 4, 2);
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.code == "unresolved_internal_reference")
        );
        provisions[1].text =
            "Se aplicará el artículo 999. Las obligaciones de esta Ley subsisten.".to_owned();
        let references = extract_internal_references(&provisions).unwrap();
        assert!(
            !references
                .iter()
                .any(|edge| edge.source_provision_id == provisions[1].id)
        );
        provisions[1].text = "Se aplicará el artículo 999 del Código ajeno.".to_owned();
        assert!(
            !extract_internal_references(&provisions)
                .unwrap()
                .iter()
                .any(|edge| edge.source_provision_id == provisions[1].id)
        );
    }

    #[test]
    fn relative_references_respect_external_governing_context() {
        let date = NaiveDate::from_ymd_opt(2018, 3, 9).unwrap();
        let mut provisions = parse_lritf(RELATIVE_FIXTURE, date).unwrap();
        provisions[1].text = "Se aplicará el artículo anterior de la Constitución Política de los Estados Unidos Mexicanos.".to_owned();
        let options = ReferenceOptions {
            policy: InstrumentContextPolicy::SentenceEarliestMarker {
                internal_markers: vec!["de esta ley".to_owned()],
                external_instruments: vec![(
                    "constitución política de los estados unidos mexicanos".to_owned(),
                    CPEUM_ID.to_owned(),
                )],
            },
            transitory_citations: true,
            same_article_fractions: true,
            relative_references: true,
            historical_targets: vec![],
            parent_instrument: None,
        };
        let targets = provisions.iter().map(|p| p.id.clone()).collect();
        let references = extract_references(&provisions, None, &options, &targets).unwrap();
        assert!(
            !references
                .iter()
                .any(|edge| edge.source_provision_id == provisions[1].id)
        );
        provisions[1].text = "Se aplicará el artículo anterior de esta Ley y el artículo 73 de la Constitución Política de los Estados Unidos Mexicanos.".to_owned();
        let references = extract_references(&provisions, None, &options, &targets).unwrap();
        assert!(
            references
                .iter()
                .any(|edge| edge.source_provision_id == provisions[1].id
                    && edge.reference_form == ReferenceForm::Relative
                    && edge.target_provision_id == provisions[0].id)
        );
    }

    #[test]
    fn suffix_qualifier_ends_at_a_word_boundary_and_accepts_hyphenated_numbers() {
        let date = NaiveDate::from_ymd_opt(2018, 3, 9).unwrap();
        let mut provisions = parse_lritf(RELATIVE_FIXTURE, date).unwrap();
        // LBM article 64: "...136 tercer párrafo, 137..." is article 136, not 136 Ter.
        provisions[1].text =
            "Se aplicarán los artículos 1, 3 tercer párrafo y 4 de esta Ley.".to_owned();
        let article_three = provisions[2].id.clone();
        let edges = extract_internal_references(&provisions).unwrap();
        assert!(
            edges
                .iter()
                .any(|e| e.source_provision_id == provisions[1].id
                    && e.target_provision_id == article_three
                    && e.source_span == "3"),
            "{edges:?}"
        );
        // LGOAAC article 5 Bis-2: "artículo 5 Bis-1 de esta Ley" names article 5 Bis 1.
        let bis = format!("{LRITF_ID}:article:3-bis-1");
        provisions[2].id = bis.clone();
        provisions[2].number = "3 Bis 1".to_owned();
        for text in [
            "Aplica el artículo 3 Bis-1 de esta Ley.",
            "Aplica el artículo 3 Bis 1 de esta Ley.",
        ] {
            provisions[1].text = text.to_owned();
            let edges = extract_internal_references(&provisions).unwrap();
            assert!(
                edges
                    .iter()
                    .any(|e| e.source_provision_id == provisions[1].id
                        && e.target_provision_id == bis),
                "{text}: {edges:?}"
            );
        }
    }

    #[test]
    fn decies_family_suffix_headings_are_not_read_as_a_shorter_suffix() {
        let date = NaiveDate::from_ymd_opt(2018, 3, 9).unwrap();
        let mut provisions = parse_lritf(RELATIVE_FIXTURE, date).unwrap();
        // LAC and LGAMVLV print "Artículo 78 Terdecies." inside a parent article.
        provisions[1].text =
            "Artículo adicionado DOF 03-05-2023 Artículo 3 Terdecies. La Agencia dispondrá."
                .to_owned();
        let edges = extract_internal_references(&provisions).unwrap();
        assert!(
            !edges
                .iter()
                .any(|e| e.source_provision_id == provisions[1].id),
            "{edges:?}"
        );
    }

    #[test]
    fn quoted_shorthand_for_the_parent_law_is_not_retargeted_by_a_later_law() {
        let date = NaiveDate::from_ymd_opt(2018, 3, 9).unwrap();
        let mut provisions = parse_lritf(RELATIVE_FIXTURE, date).unwrap();
        let options = ReferenceOptions {
            policy: InstrumentContextPolicy::SentenceEarliestMarker {
                internal_markers: vec!["de este reglamento".to_owned()],
                external_instruments: vec![(
                    "ley federal de procedimiento administrativo".to_owned(),
                    "urn:lex-mx:federal:statute:lfpa".to_owned(),
                )],
            },
            transitory_citations: true,
            same_article_fractions: true,
            relative_references: true,
            historical_targets: vec![],
            parent_instrument: None,
        };
        let targets = provisions.iter().map(|p| p.id.clone()).collect();
        // REG-LAN article 153: article 122 is the Ley de Aguas Nacionales' (the
        // quoted "Ley"), not LFPA's.
        for text in [
            "Se aplicará, en lo conducente, lo dispuesto en el artículo 3 de la “Ley” y en la Ley Federal de Procedimiento Administrativo.",
            "Se aplicará, en lo conducente, lo dispuesto en el artículo 3 de la \"Ley\" y en la Ley Federal de Procedimiento Administrativo.",
        ] {
            provisions[1].text = text.to_owned();
            let edges = extract_references(&provisions, None, &options, &targets).unwrap();
            assert!(
                !edges
                    .iter()
                    .any(|e| e.source_provision_id == provisions[1].id),
                "{text}: {edges:?}"
            );
        }
        // A citation naming the other law directly still links to it.
        provisions[1].text =
            "Se aplicará el artículo 3 de la Ley Federal de Procedimiento Administrativo."
                .to_owned();
        let edges = extract_references(&provisions, None, &options, &targets).unwrap();
        assert!(
            edges
                .iter()
                .any(|e| e.source_provision_id == provisions[1].id
                    && e.target_instrument_id == "urn:lex-mx:federal:statute:lfpa")
        );
    }

    const LAN: &str = "urn:lex-mx:federal:statute:lan";
    const REG_LAN: &str = "urn:lex-mx:federal:regulation:reg-lan";

    fn parent_edges(text: &str, known: &[&str]) -> Vec<lex_core::ReferenceEdge> {
        let document = parse_diputados(
            &format!("Artículo 1.- {text}"),
            &DiputadosOptions {
                instrument_id: REG_LAN.to_owned(),
                header_lines: Vec::new(),
                stop_markers: Vec::new(),
                annex_markers: Vec::new(),
                separate_enactment: false,
                nested_instrument: None,
                separatorless_article_headings: Vec::new(),
            },
            NaiveDate::from_ymd_opt(2002, 1, 15).unwrap(),
        )
        .unwrap();
        extract_references(
            &document.provisions,
            None,
            &ReferenceOptions {
                policy: InstrumentContextPolicy::SentenceEarliestMarker {
                    internal_markers: ["de este reglamento", "del presente reglamento"]
                        .iter()
                        .map(|m| (*m).to_owned())
                        .collect(),
                    external_instruments: vec![
                        (
                            "ley federal de procedimiento administrativo".to_owned(),
                            "urn:lex-mx:federal:statute:lfpa".to_owned(),
                        ),
                        (
                            "ley en materia de telecomunicaciones y radiodifusión".to_owned(),
                            "urn:lex-mx:federal:statute:lmtr".to_owned(),
                        ),
                    ],
                },
                transitory_citations: true,
                same_article_fractions: true,
                relative_references: true,
                historical_targets: Vec::new(),
                parent_instrument: Some(LAN.to_owned()),
            },
            &known
                .iter()
                .map(|id| (*id).to_owned())
                .collect::<HashSet<String>>(),
        )
        .unwrap()
    }

    #[test]
    fn a_regulations_bare_ley_resolves_to_its_parent_only_when_the_article_exists() {
        let known = [
            "urn:lex-mx:federal:statute:lan:article:122",
            "urn:lex-mx:federal:statute:lfpa:article:66",
        ];
        // REG-LAN article 153: article 122 is the Ley's, not LFPA's.
        for text in [
            "Se aplicará el artículo 122 de la “Ley” y en la Ley Federal de Procedimiento Administrativo.",
            "Se aplicará el artículo 122 de la \"Ley\" y lo previsto en la Ley Federal de Procedimiento Administrativo.",
            "Se aplicará lo dispuesto en el artículo 122 de la Ley, para los efectos de este artículo.",
        ] {
            let edges = parent_edges(text, &known);
            assert_eq!(edges.len(), 1, "{text}: {edges:?}");
            assert_eq!(edges[0].target_provision_id, known[0], "{text}");
            assert_eq!(
                edges[0].resolution_status,
                ReferenceResolutionStatus::Resolved
            );
        }
        // A missing parent article is neither an edge nor an unresolved edge.
        assert!(parent_edges("Se aplicará el artículo 999 de la “Ley”.", &known).is_empty());
        // A later-named law is still linked on its own citation.
        let edges = parent_edges(
            "Se aplicará el artículo 66 de la Ley Federal de Procedimiento Administrativo.",
            &known,
        );
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].target_provision_id, known[1]);
    }

    #[test]
    fn a_bare_ley_never_takes_a_dated_titled_or_cited_law() {
        let known = ["urn:lex-mx:federal:statute:lan:article:10"];
        for text in [
            // REG-LISR article 10: the law in force until 2001 is not the parent.
            "Se aplicará el artículo 10 de la Ley vigente hasta el 31 de diciembre de 2001, podrán actualizar.",
            "Se aplicará el artículo 10 de la Ley vigente a partir de 2014 y de otras.",
            // REG-LOPGJDF article 7: a different law whose title starts "que".
            "Se aplicará el artículo 10 de la Ley que Regula el Uso de la Fuerza de los Cuerpos.",
            // REG-LFPPI article 12: "la Ley en cita" refers to an earlier-cited law.
            "Se aplicará el artículo 10 de la Ley en cita, o de su equivalente.",
        ] {
            let edges: Vec<_> = parent_edges(text, &known)
                .into_iter()
                .filter(|e| e.target_instrument_id == LAN)
                .collect();
            assert!(edges.is_empty(), "{text}: {edges:?}");
        }
        // Relative clauses about the parent and plain "vigente" still link.
        for text in [
            "Se aplicará el artículo 10 de la Ley que será presentada al Mando.",
            "Se aplicará el artículo 10 de la Ley vigente, conforme a lo anterior.",
            "Se aplicará el artículo 10 de la Ley en lo que refiere a los planteles.",
        ] {
            assert_eq!(parent_edges(text, &known).len(), 1, "{text}");
        }
    }

    #[test]
    fn each_edge_records_how_its_target_was_chosen() {
        let known = [
            "urn:lex-mx:federal:statute:lan:article:10",
            "urn:lex-mx:federal:statute:lfpa:article:66",
            "urn:lex-mx:federal:regulation:reg-lan:article:1",
            "urn:lex-mx:federal:regulation:reg-lan:article:5",
        ];
        let rule_for = |text: &str, target: &str| {
            parent_edges(text, &known)
                .into_iter()
                .find(|edge| edge.target_provision_id == target)
                .and_then(|edge| edge.resolution_rule)
        };
        assert_eq!(
            rule_for("Se aplicará el artículo 10 de la Ley.", known[0]),
            Some(lex_core::ResolutionRule::ParentShorthand)
        );
        assert_eq!(
            rule_for(
                "Se aplicará el artículo 66 de la Ley Federal de Procedimiento Administrativo.",
                known[1]
            ),
            Some(lex_core::ResolutionRule::NamedInstrument)
        );
        assert_eq!(
            rule_for("Se aplicará el artículo 5 de este Reglamento.", known[3]),
            Some(lex_core::ResolutionRule::OwnInstrumentMarker)
        );
        assert_eq!(
            rule_for("Se aplicará el artículo 5.", known[3]),
            Some(lex_core::ResolutionRule::OwnInstrumentDefault)
        );
    }

    #[test]
    fn validation_rejects_a_resolution_rule_that_does_not_fit_the_edge() {
        let known = ["urn:lex-mx:federal:statute:lan:article:10"];
        let edge = parent_edges("Se aplicará el artículo 10 de la Ley.", &known)
            .into_iter()
            .next()
            .unwrap();
        let document = parse_diputados(
            "Artículo 1.- Se aplicará el artículo 10 de la Ley.",
            &DiputadosOptions {
                instrument_id: REG_LAN.to_owned(),
                header_lines: Vec::new(),
                stop_markers: Vec::new(),
                annex_markers: Vec::new(),
                separate_enactment: false,
                nested_instrument: None,
                separatorless_article_headings: Vec::new(),
            },
            NaiveDate::from_ymd_opt(2002, 1, 15).unwrap(),
        )
        .unwrap();
        let issues_for = |candidate: &lex_core::ReferenceEdge| {
            let mut issues = Vec::new();
            lex_analysis::validate_references(
                REG_LAN,
                None,
                &document.provisions,
                std::slice::from_ref(candidate),
                &known
                    .iter()
                    .map(|id| (*id).to_owned())
                    .collect::<HashSet<String>>(),
                &mut issues,
            );
            issues
                .iter()
                .filter(|i| i.code == "resolution_rule_inconsistent")
                .count()
        };
        // An own-instrument or relative rule on a cross-instrument edge fails;
        // the edge's own rule and a missing (legacy) rule pass this check.
        for (rule, expected) in [
            (Some(lex_core::ResolutionRule::ParentShorthand), 0),
            (None, 0),
            (Some(lex_core::ResolutionRule::OwnInstrumentDefault), 1),
            (Some(lex_core::ResolutionRule::RelativePosition), 1),
            (Some(lex_core::ResolutionRule::SameArticle), 1),
        ] {
            let mut candidate = edge.clone();
            candidate.resolution_rule = rule;
            assert_eq!(issues_for(&candidate), expected, "{rule:?}");
        }
    }

    #[test]
    fn a_configured_full_title_outranks_its_bare_parent_prefix() {
        let known = [
            "urn:lex-mx:federal:statute:lan:article:1",
            "urn:lex-mx:federal:statute:lmtr:article:1",
        ];
        let edges = parent_edges(
            "Se aplicará el artículo 1 de la Ley en Materia de Telecomunicaciones y Radiodifusión.",
            &known,
        );
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].target_provision_id, known[1]);
        // A genuine parent reference before a separate later citation stays.
        let edges = parent_edges(
            "Se aplicará el artículo 1 de la Ley, y el artículo 1 de la Ley en Materia de Telecomunicaciones y Radiodifusión.",
            &known,
        );
        let targets: Vec<_> = edges
            .iter()
            .map(|e| e.target_provision_id.as_str())
            .collect();
        assert_eq!(targets, [known[0], known[1]]);
    }

    #[test]
    fn quoting_ley_does_not_bypass_dated_or_anaphoric_exclusions() {
        let known = ["urn:lex-mx:federal:statute:lan:article:10"];
        for text in [
            "Se aplicará el artículo 10 de la “Ley” vigente hasta el 31 de diciembre de 2001, y otros.",
            "Se aplicará el artículo 10 de la \"Ley\" vigente hasta el 31 de diciembre de 2001, y otros.",
            "Se aplicará el artículo 10 de la “Ley” en cita, o de su equivalente.",
            "Se aplicará el artículo 10 de la \"Ley\" en cita, o de su equivalente.",
        ] {
            assert!(parent_edges(text, &known).is_empty(), "{text}");
        }
        for text in [
            "Se aplicará el artículo 10 de la “Ley”, conforme a lo anterior.",
            "Se aplicará el artículo 10 de la \"Ley\" y de otras.",
        ] {
            assert_eq!(parent_edges(text, &known).len(), 1, "{text}");
        }
    }

    #[test]
    fn punctuation_does_not_hide_a_dated_modifier() {
        let known = ["urn:lex-mx:federal:statute:lan:article:1"];
        for text in [
            "Se aplicará el artículo 1 de la “Ley”, vigente hasta el 31 de diciembre de 1990.",
            "Se aplicará el artículo 1 de la “Ley” (vigente hasta el 31 de diciembre de 1990).",
            "Se aplicará el artículo 1 de la \"Ley\", vigente hasta el 31 de diciembre de 1990.",
            "Se aplicará el artículo 1 de la Ley, vigente hasta el 31 de diciembre de 1990.",
            "Se aplicará el artículo 1 de la Ley (vigente hasta el 31 de diciembre de 1990).",
            "Se aplicará el artículo 1 de la Ley, en cita.",
            "Se aplicará el artículo 1 de la “Ley” , vigente hasta el 31 de diciembre de 1990.",
            "Se aplicará el artículo 1 de la \"Ley\" , vigente hasta el 31 de diciembre de 1990.",
            "Se aplicará el artículo 1 de la Ley , vigente hasta el 31 de diciembre de 1990.",
        ] {
            assert!(parent_edges(text, &known).is_empty(), "{text}");
        }
        for text in [
            "Se aplicará el artículo 1 de la “Ley”, y de otras.",
            "Se aplicará el artículo 1 de la Ley. Vigente hasta mañana.",
            "Se aplicará el artículo 1 de la Ley, vigente conforme a lo anterior.",
            "Se aplicará el artículo 1 de la Ley (aplicable).",
            "Se aplicará el artículo 1 de la Ley , y de otras.",
            "Se aplicará el artículo 1 de la “Ley” , conforme a lo anterior.",
        ] {
            assert_eq!(parent_edges(text, &known).len(), 1, "{text}");
        }
    }

    #[test]
    fn unsupported_anaphora_and_historical_wording_stay_unlinked() {
        let known = ["urn:lex-mx:federal:statute:lan:article:1"];
        for text in [
            "Conforme a la Ley Federal del Trabajo, aplica el artículo 1 de la Ley en comento.",
            "Conforme a la Ley Federal del Trabajo, aplica el artículo 1 de la Ley que se cita.",
            "Se aplicará el artículo 1 de la Ley en vigor al 31 de diciembre de 1990.",
            "Se aplicará el artículo 1 de la Ley que estaba vigente al 31 de diciembre de 1990.",
        ] {
            assert!(parent_edges(text, &known).is_empty(), "{text}");
        }
        assert_eq!(
            parent_edges(
                "Se aplicará el artículo 1 de la Ley en vigor, y otras.",
                &known
            )
            .len(),
            1
        );
    }

    #[test]
    fn a_bare_ley_never_takes_a_titled_or_mixed_or_anaphoric_citation() {
        let known = ["urn:lex-mx:federal:statute:lan:article:122"];
        for text in [
            // another law's title follows "de la Ley"
            "Se aplicará el artículo 122 de la Ley de Pesca.",
            // an internal marker shares the citation window
            "Se aplicará el artículo 122 de la “Ley” y 122 de este Reglamento.",
            // anaphoric reference stays skipped
            "Se aplicará el artículo 122 de dicha Ley.",
        ] {
            let edges: Vec<_> = parent_edges(text, &known)
                .into_iter()
                .filter(|e| e.target_instrument_id == LAN)
                .collect();
            assert!(edges.is_empty(), "{text}: {edges:?}");
        }
    }

    #[test]
    fn relative_reference_is_not_retargeted_by_a_law_named_in_a_later_clause() {
        let date = NaiveDate::from_ymd_opt(2018, 3, 9).unwrap();
        let mut provisions = parse_lritf(RELATIVE_FIXTURE, date).unwrap();
        // Shape printed in committed LBOGM, LGPAS, LFCC and 23 more instruments.
        provisions[1].text = "Para la revisión a que se refiere el artículo anterior, se estará a lo dispuesto en el Título Sexto de la Ley Federal de Procedimiento Administrativo.".to_owned();
        let references = extract_internal_references(&provisions).unwrap();
        assert!(
            references
                .iter()
                .any(|edge| edge.source_provision_id == provisions[1].id
                    && edge.reference_form == ReferenceForm::Relative
                    && edge.target_provision_id == provisions[0].id)
        );
        let options = ReferenceOptions {
            policy: InstrumentContextPolicy::SentenceEarliestMarker {
                internal_markers: vec!["de esta ley".to_owned()],
                external_instruments: vec![(
                    "ley federal de procedimiento administrativo".to_owned(),
                    "urn:lex-mx:federal:statute:lfpa".to_owned(),
                )],
            },
            transitory_citations: true,
            same_article_fractions: true,
            relative_references: true,
            historical_targets: vec![],
            parent_instrument: None,
        };
        let targets = provisions.iter().map(|p| p.id.clone()).collect();
        let references = extract_references(&provisions, None, &options, &targets).unwrap();
        assert!(
            references
                .iter()
                .any(|edge| edge.source_provision_id == provisions[1].id
                    && edge.reference_form == ReferenceForm::Relative
                    && edge.target_provision_id == provisions[0].id)
        );
        for text in [
            "Las infracciones señaladas en el artículo anterior serán sustanciadas con arreglo supletorio en la Ley Federal de Procedimiento Administrativo.",
            "Los anteproyectos a que se refiere el artículo anterior y que estén sujetos a la Ley Federal de Procedimiento Administrativo, podrán hacerse públicos.",
        ] {
            provisions[1].text = text.to_owned();
            assert!(
                extract_references(&provisions, None, &options, &targets)
                    .unwrap()
                    .iter()
                    .any(|edge| edge.source_provision_id == provisions[1].id
                        && edge.target_provision_id == provisions[0].id),
                "{text}"
            );
        }
        // Without the comma the law directly governs the citation: no internal link.
        provisions[1].text =
            "Se aplicará el artículo anterior de la Ley Federal de Procedimiento Administrativo."
                .to_owned();
        assert!(
            !extract_references(&provisions, None, &options, &targets)
                .unwrap()
                .iter()
                .any(|edge| edge.source_provision_id == provisions[1].id)
        );
    }

    #[test]
    fn bare_reglamento_relative_is_self_only_in_a_regulation() {
        let date = NaiveDate::from_ymd_opt(2018, 3, 9).unwrap();
        let mut provisions = parse_lritf(RELATIVE_FIXTURE, date).unwrap();
        // Exact opening sentence of committed REG-LGN article216; its local
        // predecessor is article215, not an external regulation's article.
        let text = "Después de recibido formalmente el recurso de inconformidad y la documentación mencionada en el artículo anterior del Reglamento, el Consejo de Disciplina u Órgano Revisor deberá:";
        provisions[1].text = text.to_owned();
        assert!(
            !extract_internal_references(&provisions)
                .unwrap()
                .iter()
                .any(|edge| edge.source_provision_id == provisions[1].id)
        );
        let id = "urn:lex-mx:federal:regulation:reg-lgn";
        for (index, provision) in provisions.iter_mut().take(3).enumerate() {
            provision.instrument_id = id.to_owned();
            provision.number = (215 + index).to_string();
            provision.id = format!("{id}:article:{}", provision.number);
        }
        let references = extract_internal_references(&provisions).unwrap();
        assert!(
            references
                .iter()
                .any(|edge| edge.source_provision_id == provisions[1].id
                    && edge.target_provision_id == provisions[0].id)
        );
        provisions[1].text =
            "Se aplicará el artículo anterior del Reglamento de otra Ley.".to_owned();
        assert!(
            !extract_internal_references(&provisions)
                .unwrap()
                .iter()
                .any(|edge| edge.source_provision_id == provisions[1].id)
        );
    }

    #[test]
    fn existing_external_target_must_belong_to_declared_instrument() {
        fn view<'a>(
            provisions: &'a [Provision],
            references: &'a [lex_core::ReferenceEdge],
        ) -> CorpusView<'a> {
            CorpusView {
                instrument_id: LRITF_ID,
                official_title: None,
                provisions,
                references,
                terms: &[],
                term_usages: &[],
                amendment_references: &[],
            }
        }
        let date = NaiveDate::from_ymd_opt(2018, 3, 9).unwrap();
        let mut provisions = parse_lritf(RELATIVE_FIXTURE, date).unwrap();
        provisions[1].text = "Se aplicará el artículo 1 de esta Ley.".to_owned();
        let mut references = extract_internal_references(&provisions).unwrap();
        references.retain(|edge| edge.source_provision_id == provisions[1].id);
        references[0].target_provision_id = format!("{CPEUM_ID}:article:1");
        references[0].target_instrument_id = LOCG_ID.to_owned();
        // A hand-retargeted edge no longer fits its own-instrument rule; this
        // test concerns target ownership, so treat it as a legacy edge.
        references[0].resolution_rule = None;
        let external_targets = HashSet::from([references[0].target_provision_id.clone()]);
        let expectations = CorpusExpectations {
            articles: Some(4),
            transitories: Some(2),
            ..Default::default()
        };
        let report = validate_corpus(
            &view(&provisions, &references),
            &expectations,
            &external_targets,
            &HashSet::new(),
        );
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.code == "reference_target_owner_mismatch")
        );
        references[0].target_instrument_id = CPEUM_ID.to_owned();
        let report = validate_corpus(
            &view(&provisions, &references),
            &expectations,
            &external_targets,
            &HashSet::new(),
        );
        assert!(report.valid, "{:?}", report.issues);
    }

    #[test]
    fn resolves_relative_references_by_same_type_document_order() {
        let date = NaiveDate::from_ymd_opt(2018, 3, 9).unwrap();
        let provisions = parse_lritf(RELATIVE_FIXTURE, date).unwrap();
        let references = extract_internal_references(&provisions).unwrap();
        let relatives: Vec<_> = references
            .iter()
            .filter(|edge| edge.reference_form == ReferenceForm::Relative)
            .collect();

        // Article 1 has no previous article, so its `artículo anterior`
        // produces no edge; the plural `artículos anteriores` and the word
        // `anteriormente` never match.
        assert!(
            relatives
                .iter()
                .all(|edge| !edge.source_provision_id.ends_with(":article:1"))
        );
        assert_eq!(relatives.len(), 4);

        // Article 2: `anterior` resolves backward, `siguiente` forward,
        // and the ordinal-first pre-qualifier attaches to the edge.
        let article_two: Vec<_> = relatives
            .iter()
            .filter(|edge| edge.source_provision_id.ends_with(":article:2"))
            .collect();
        assert_eq!(article_two.len(), 2);
        assert!(article_two[0].target_provision_id.ends_with(":article:1"));
        assert_eq!(article_two[0].source_span, "artículo anterior");
        assert!(
            article_two[0].qualifiers.is_empty(),
            "{:?}",
            article_two[0].qualifiers
        );
        assert!(article_two[1].target_provision_id.ends_with(":article:3"));
        assert_eq!(article_two[1].source_span, "artículo siguiente");
        assert_eq!(article_two[1].qualifiers.len(), 1);
        assert_eq!(article_two[1].qualifiers[0].text, "primer párrafo");

        // Article 3: `del citado artículo anterior` still resolves, but the
        // intervening word keeps the pre-qualifier from attaching.
        let article_three = relatives
            .iter()
            .find(|edge| edge.source_provision_id.ends_with(":article:3"))
            .unwrap();
        assert!(article_three.target_provision_id.ends_with(":article:2"));
        assert!(
            article_three.qualifiers.is_empty(),
            "{:?}",
            article_three.qualifiers
        );

        // A relative reference inside a transitory resolves against the
        // transitory sequence, never the article sequence.
        let transitory = relatives
            .iter()
            .find(|edge| edge.source_provision_id.ends_with(":transitory:segunda"))
            .unwrap();
        assert!(
            transitory
                .target_provision_id
                .ends_with(":transitory:primera")
        );

        let report = validate_lritf(&provisions, &references, 4, 2);
        assert!(report.valid, "{:?}", report.issues);
    }

    #[test]
    fn resolves_article_that_precedes_as_previous_article() {
        let date = NaiveDate::from_ymd_opt(2018, 3, 9).unwrap();
        let provisions = parse_lritf(RELATIVE_PRECEDES_FIXTURE, date).unwrap();
        let references = extract_internal_references(&provisions).unwrap();
        let relatives: Vec<_> = references
            .iter()
            .filter(|edge| edge.reference_form == ReferenceForm::Relative)
            .collect();

        assert_eq!(relatives.len(), 1);
        assert!(relatives[0].source_provision_id.ends_with(":article:2"));
        assert_eq!(relatives[0].source_span, "artículo que precede");
        assert!(relatives[0].target_provision_id.ends_with(":article:1"));
    }

    #[test]
    fn captures_noun_first_and_penultimate_pre_qualifiers() {
        let date = NaiveDate::from_ymd_opt(2018, 3, 9).unwrap();
        let provisions = parse_lritf(RELATIVE_FIXTURE, date).unwrap();
        let references = extract_internal_references(&provisions).unwrap();

        // Noun-first form: `los párrafos segundo y tercero del artículo 2`.
        let noun_first = references
            .iter()
            .find(|edge| {
                edge.source_provision_id.ends_with(":article:4")
                    && edge.target_provision_id.ends_with(":article:2")
            })
            .unwrap();
        assert_eq!(noun_first.reference_form, ReferenceForm::Direct);
        assert_eq!(noun_first.qualifiers.len(), 1);
        assert_eq!(
            noun_first.qualifiers[0].qualifier_type,
            ReferenceQualifierType::Paragraph
        );
        assert_eq!(noun_first.qualifiers[0].text, "párrafos segundo y tercero");

        // Ordinal-first `penúltimo párrafo del artículo 1`.
        let penultimate = references
            .iter()
            .find(|edge| {
                edge.source_provision_id.ends_with(":article:3")
                    && edge.target_provision_id.ends_with(":article:1")
                    && edge.reference_form == ReferenceForm::Direct
            })
            .unwrap();
        assert_eq!(penultimate.qualifiers.len(), 1);
        assert_eq!(penultimate.qualifiers[0].text, "penúltimo párrafo");
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
        let evidence = extract_reform_evidence(raw, &lritf_options()).unwrap();
        assert_eq!(evidence.len(), 2);
        assert_eq!(
            evidence[1].provision_id,
            "urn:lex-mx:federal:statute:lritf:amendment:2025-11-14:transitory:segundo"
        );
        assert_eq!(evidence[1].text, "La aplicación será gradual.");
    }
    #[test]
    #[allow(clippy::too_many_lines)]
    fn exact_reviewed_stale_reference_is_retained_and_other_missing_targets_fail() {
        let instrument_id = "urn:lex-mx:federal:statute:lcnbv";
        let options = DiputadosOptions {
            instrument_id: instrument_id.to_owned(),
            header_lines: vec!["LEY DE LA COMISIÓN NACIONAL BANCARIA Y DE VALORES".to_owned()],
            stop_markers: vec![],
            annex_markers: vec![],
            separate_enactment: false,
            nested_instrument: None,
            separatorless_article_headings: Vec::new(),
        };
        let doc = parse_diputados(
            include_str!("../../../fixtures/diputados/held-lcnbv-article15.txt"),
            &options,
            NaiveDate::from_ymd_opt(1995, 4, 28).unwrap(),
        )
        .unwrap();
        let target = "urn:lex-mx:federal:statute:lmv";
        let opts = ReferenceOptions {
            policy: InstrumentContextPolicy::SentenceEarliestMarker {
                internal_markers: vec!["de esta ley".to_owned()],
                external_instruments: vec![(
                    "ley del mercado de valores".to_owned(),
                    target.to_owned(),
                )],
            },
            transitory_citations: true,
            same_article_fractions: true,
            relative_references: true,
            historical_targets: vec![],
            parent_instrument: None,
        };
        let mut edges = extract_references(&doc.provisions, None, &opts, &HashSet::new()).unwrap();
        let stale = edges
            .iter()
            .find(|e| e.target_provision_id == format!("{target}:article:16-bis-7"))
            .unwrap()
            .clone();
        assert_eq!(
            stale.resolution_status,
            ReferenceResolutionStatus::Unresolved
        );
        let mut issues = Vec::new();
        let by_id: std::collections::HashMap<&str, &Provision> =
            doc.provisions.iter().map(|p| (p.id.as_str(), p)).collect();
        lex_analysis::validate_reference(
            instrument_id,
            None,
            &stale,
            &by_id,
            &HashSet::new(),
            &mut issues,
        );
        assert!(
            issues
                .iter()
                .any(|i| i.code == "unresolved_internal_reference")
        );
        let source = doc
            .provisions
            .iter()
            .find(|p| p.id == stale.source_provision_id)
            .unwrap();
        let approval = lex_core::OutdatedReferenceApproval {
            source_provision_id: stale.source_provision_id.clone(),
            target_instrument_id: target.to_owned(),
            target_provision_id: stale.target_provision_id.clone(),
            source_span: stale.source_span.clone(),
            start_char: stale.start_char,
            end_char: stale.end_char,
            source_text_sha256: lex_core::evidence_sha256(&source.text),
            source_sha256: "a".repeat(64),
            reviewer: "Fixture reviewer".to_owned(),
            captured_at: "2026-10-16T00:00:00Z".to_owned(),
            reviewer_supplied_at: None,
            rationale: "Explicit fixture approval for this one stale citation".to_owned(),
            decision_source: "fixture decision; not a canonical operator resolution".to_owned(),
            decision_source_sha256: "c".repeat(64),
        };
        super::apply_outdated_reference_approvals(
            &mut edges,
            &doc.provisions,
            std::slice::from_ref(&approval),
            &"a".repeat(64),
            &HashSet::new(),
        )
        .unwrap();
        let approved = edges.iter().find(|e| e.id == stale.id).unwrap();
        assert_eq!(
            approved.resolution_status,
            ReferenceResolutionStatus::HistoricalSupersededReferenceOutdated
        );
        assert_eq!(approved.source_span, stale.source_span);
        assert_eq!(
            approved.outdated_reference_approval.as_ref(),
            Some(&approval)
        );
        let mut issues = Vec::new();
        lex_analysis::validate_reference(
            instrument_id,
            None,
            approved,
            &by_id,
            &HashSet::new(),
            &mut issues,
        );
        assert!(
            !issues
                .iter()
                .any(|i| i.severity == lex_core::Severity::Error)
        );
        assert!(
            issues
                .iter()
                .any(|i| i.code == "historical_superseded_reference_outdated")
        );
        // A second missing citation does not inherit the approval.
        let mut other = stale.clone();
        other.target_provision_id = format!("{target}:article:999");
        let mut issues = Vec::new();
        lex_analysis::validate_reference(
            instrument_id,
            None,
            &other,
            &by_id,
            &HashSet::new(),
            &mut issues,
        );
        assert!(
            issues
                .iter()
                .any(|i| i.code == "unresolved_internal_reference")
        );
        // Tampered approvals, source text, present targets and duplicates fail closed.
        let mut tampered = approved.clone();
        tampered
            .outdated_reference_approval
            .as_mut()
            .unwrap()
            .source_text_sha256 = "0".repeat(64);
        let mut issues = Vec::new();
        lex_analysis::validate_reference(
            instrument_id,
            None,
            &tampered,
            &by_id,
            &HashSet::new(),
            &mut issues,
        );
        assert!(
            issues
                .iter()
                .any(|i| i.code == "outdated_reference_approval_invalid")
        );
        let mut raw = vec![stale.clone()];
        assert!(
            super::apply_outdated_reference_approvals(
                &mut raw,
                &doc.provisions,
                std::slice::from_ref(&approval),
                &"b".repeat(64),
                &HashSet::new()
            )
            .is_err()
        );
        let mut changed = doc.provisions.clone();
        changed[0].text.push(' ');
        assert!(
            super::apply_outdated_reference_approvals(
                &mut raw,
                &changed,
                std::slice::from_ref(&approval),
                &"a".repeat(64),
                &HashSet::new()
            )
            .is_err()
        );
        assert!(
            super::apply_outdated_reference_approvals(
                &mut raw,
                &doc.provisions,
                std::slice::from_ref(&approval),
                &"a".repeat(64),
                &HashSet::from([stale.target_provision_id.clone()])
            )
            .is_err()
        );
        let mut duplicate = vec![stale.clone(), stale];
        assert!(
            super::apply_outdated_reference_approvals(
                &mut duplicate,
                &doc.provisions,
                &[approval],
                &"a".repeat(64),
                &HashSet::new()
            )
            .is_err()
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn legal_operator_review_annotates_one_exact_edge_without_changing_it() {
        let instrument_id = "urn:lex-mx:federal:statute:lcnbv";
        let options = DiputadosOptions {
            instrument_id: instrument_id.to_owned(),
            header_lines: vec!["LEY DE LA COMISIÓN NACIONAL BANCARIA Y DE VALORES".to_owned()],
            stop_markers: vec![],
            annex_markers: vec![],
            separate_enactment: false,
            nested_instrument: None,
            separatorless_article_headings: Vec::new(),
        };
        let doc = parse_diputados(
            include_str!("../../../fixtures/diputados/held-lcnbv-article15.txt"),
            &options,
            NaiveDate::from_ymd_opt(1995, 4, 28).unwrap(),
        )
        .unwrap();
        let target = "urn:lex-mx:federal:statute:lmv";
        let opts = ReferenceOptions {
            policy: InstrumentContextPolicy::SentenceEarliestMarker {
                internal_markers: vec!["de esta ley".to_owned()],
                external_instruments: vec![(
                    "ley del mercado de valores".to_owned(),
                    target.to_owned(),
                )],
            },
            transitory_citations: true,
            same_article_fractions: true,
            relative_references: true,
            historical_targets: vec![],
            parent_instrument: None,
        };
        let mut edges = extract_references(&doc.provisions, None, &opts, &HashSet::new()).unwrap();
        let edge = edges
            .iter()
            .find(|e| e.target_provision_id == format!("{target}:article:16-bis-7"))
            .unwrap()
            .clone();
        let source = doc
            .provisions
            .iter()
            .find(|p| p.id == edge.source_provision_id)
            .unwrap();
        let review = lex_core::LegalOperatorReview {
            source_provision_id: edge.source_provision_id.clone(),
            target_instrument_id: target.to_owned(),
            target_provision_id: edge.target_provision_id.clone(),
            source_span: edge.source_span.clone(),
            start_char: edge.start_char,
            end_char: edge.end_char,
            source_text_sha256: lex_core::evidence_sha256(&source.text),
            reviewer: "Fixture reviewer".to_owned(),
            captured_at: "2026-10-16T00:00:00Z".to_owned(),
            note: "Fixture note: the reviewer names a different current article.".to_owned(),
            decision_source: "fixture decision".to_owned(),
            decision_source_sha256: "c".repeat(64),
        };
        super::apply_legal_operator_reviews(
            &mut edges,
            &doc.provisions,
            std::slice::from_ref(&review),
        )
        .unwrap();
        let reviewed = edges.iter().find(|e| e.id == edge.id).unwrap();
        assert_eq!(reviewed.legal_operator_review.as_ref(), Some(&review));
        // The note changes neither the cited target nor the resolution status.
        assert_eq!(reviewed.target_provision_id, edge.target_provision_id);
        assert_eq!(reviewed.resolution_status, edge.resolution_status);
        assert_eq!(
            edges
                .iter()
                .filter(|e| e.legal_operator_review.is_some())
                .count(),
            1
        );
        // Validation rechecks the persisted binding: a valid control passes;
        // a tampered hash, span, target or metadata is an error.
        let issues_for = |candidate: &lex_core::ReferenceEdge| {
            let mut issues = Vec::new();
            lex_analysis::validate_references(
                instrument_id,
                None,
                &doc.provisions,
                std::slice::from_ref(candidate),
                &HashSet::new(),
                &mut issues,
            );
            issues
                .iter()
                .filter(|i| i.code == "legal_operator_review_invalid")
                .count()
        };
        let reviewed_edge = reviewed.clone();
        assert_eq!(issues_for(&reviewed_edge), 0);
        let tamper: [fn(&mut lex_core::LegalOperatorReview); 4] = [
            |r| r.source_text_sha256 = "0".repeat(64),
            |r| {
                r.start_char += 1;
                r.end_char += 1;
            },
            |r| r.target_provision_id = format!("{}:article:999", r.target_instrument_id),
            |r| r.note = " ".to_owned(),
        ];
        for mutate in tamper {
            let mut bad = reviewed_edge.clone();
            mutate(bad.legal_operator_review.as_mut().unwrap());
            assert_eq!(issues_for(&bad), 1);
        }
        // Changed provision text, an unmatched citation, a duplicate and a
        // second review of the same edge all fail closed.
        let mut raw = vec![edge.clone()];
        let mut changed = doc.provisions.clone();
        for provision in &mut changed {
            provision.text.push(' ');
        }
        assert!(
            super::apply_legal_operator_reviews(&mut raw, &changed, std::slice::from_ref(&review))
                .is_err()
        );
        let mut elsewhere = review.clone();
        elsewhere.start_char += 1;
        elsewhere.end_char += 1;
        assert!(
            super::apply_legal_operator_reviews(&mut raw, &doc.provisions, &[elsewhere]).is_err()
        );
        let mut duplicate = vec![edge.clone(), edge];
        assert!(
            super::apply_legal_operator_reviews(
                &mut duplicate,
                &doc.provisions,
                std::slice::from_ref(&review)
            )
            .is_err()
        );
        let mut twice = raw.clone();
        assert!(
            super::apply_legal_operator_reviews(
                &mut twice,
                &doc.provisions,
                &[review.clone(), review.clone()]
            )
            .is_err()
        );
        let mut blank = review;
        blank.note = "  ".to_owned();
        assert!(blank.validate().is_err());
    }

    #[test]
    fn flattened_table_zero_then_article_number_is_unlinked_without_changing_text() {
        let source = "urn:lex-mx:federal:decree:pef-2026";
        let target = "urn:lex-mx:federal:constitution:cpeum";
        let opts = DiputadosOptions {
            instrument_id: source.to_owned(),
            header_lines: vec![],
            stop_markers: vec![],
            annex_markers: vec![],
            separate_enactment: false,
            nested_instrument: None,
            separatorless_article_headings: Vec::new(),
        };
        let refs = ReferenceOptions {
            policy: InstrumentContextPolicy::SentenceEarliestMarker {
                internal_markers: vec![],
                external_instruments: vec![(
                    "constitución política de los estados unidos mexicanos".to_owned(),
                    target.to_owned(),
                )],
            },
            transitory_citations: false,
            same_article_fractions: false,
            relative_references: false,
            historical_targets: vec![],
            parent_instrument: None,
        };
        for (citation, expected) in [("0 74", 0), ("74", 1)] {
            let raw = format!(
                "Artículo 1. Se refiere el artículo {citation}, fracción IV, de la Constitución Política de los Estados Unidos Mexicanos."
            );
            let doc = parse_diputados(&raw, &opts, NaiveDate::from_ymd_opt(2025, 11, 21).unwrap())
                .unwrap();
            let before = doc.provisions[0].text.clone();
            let edges = extract_references(
                &doc.provisions,
                None,
                &refs,
                &HashSet::from([format!("{target}:article:74")]),
            )
            .unwrap();
            assert_eq!(edges.len(), expected);
            if expected == 1 {
                assert_eq!(edges[0].target_provision_id, format!("{target}:article:74"));
            }
            assert_eq!(doc.provisions[0].text, before);
            assert!(before.contains(&format!("artículo {citation}")));
        }
    }

    fn closure_references(
        text: &str,
        instrument_id: &str,
        internal_markers: &[&str],
        external: &[(&str, &str)],
        known: &[&str],
    ) -> Vec<lex_core::ReferenceEdge> {
        let document = parse_diputados(
            text,
            &DiputadosOptions {
                instrument_id: instrument_id.to_owned(),
                header_lines: Vec::new(),
                stop_markers: Vec::new(),
                annex_markers: Vec::new(),
                separate_enactment: false,
                nested_instrument: None,
                separatorless_article_headings: Vec::new(),
            },
            NaiveDate::from_ymd_opt(2002, 1, 15).unwrap(),
        )
        .unwrap();
        extract_references(
            &document.provisions,
            None,
            &ReferenceOptions {
                policy: InstrumentContextPolicy::SentenceEarliestMarker {
                    internal_markers: internal_markers.iter().map(|m| (*m).to_owned()).collect(),
                    external_instruments: external
                        .iter()
                        .map(|(marker, id)| ((*marker).to_owned(), (*id).to_owned()))
                        .collect(),
                },
                transitory_citations: true,
                same_article_fractions: true,
                relative_references: true,
                historical_targets: Vec::new(),
                parent_instrument: None,
            },
            &known
                .iter()
                .map(|id| (*id).to_owned())
                .collect::<HashSet<String>>(),
        )
        .unwrap()
    }

    const CPEUM: &str = "urn:lex-mx:federal:constitution:cpeum";
    const LFT: &str = "urn:lex-mx:federal:statute:lft";

    #[test]
    fn constitution_self_citation_is_not_captured_by_a_later_official_title() {
        let text = "Artículo 3.- Se normarán por el apartado A del artículo 123 de esta Constitución, en los términos que establezca la Ley Federal del Trabajo.";
        let known = [
            "urn:lex-mx:federal:constitution:cpeum:article:123",
            "urn:lex-mx:federal:statute:lft:article:123",
        ];
        let external = [("ley federal del trabajo", LFT)];
        let with_markers = closure_references(
            text,
            CPEUM,
            &[
                "de esta constitución",
                "esta constitución",
                " constitucional",
            ],
            &external,
            &known,
        );
        assert_eq!(with_markers.len(), 1);
        assert_eq!(with_markers[0].target_instrument_id, CPEUM);
        // Documents the defect the markers prevent: ley-shaped markers alone
        // let the later title capture the citation.
        let ley_shaped = closure_references(text, CPEUM, &["de esta ley"], &external, &known);
        assert_eq!(ley_shaped[0].target_instrument_id, LFT);
        let adjectival = closure_references(
            "Artículo 3.- En términos del artículo 123 constitucional, la Ley Federal del Trabajo regula la materia.",
            CPEUM,
            &[
                "de esta constitución",
                "esta constitución",
                " constitucional",
            ],
            &external,
            &known,
        );
        assert_eq!(adjectival.len(), 1);
        assert_eq!(adjectival[0].target_instrument_id, CPEUM);
    }

    const LSS: &str = "urn:lex-mx:federal:statute:lss";

    #[test]
    fn paragraph_numbers_are_qualifiers_not_article_links() {
        // L1, LGPP article 95: "artículo 10, párrafo 2, inciso c)".
        let known = [
            "urn:lex-mx:federal:statute:fix:article:10",
            "urn:lex-mx:federal:statute:fix:article:2",
        ];
        let edges = closure_references(
            "Artículo 95.- Lo previsto en el artículo 10, párrafo 2, inciso c), de esta Ley.",
            "urn:lex-mx:federal:statute:fix",
            &["de esta ley"],
            &[],
            &known,
        );
        let direct: Vec<_> = edges
            .iter()
            .filter(|e| e.reference_form == ReferenceForm::Direct)
            .collect();
        assert_eq!(direct.len(), 1, "{direct:?}");
        assert_eq!(direct[0].target_provision_id, known[0]);
        let texts: Vec<_> = direct[0]
            .qualifiers
            .iter()
            .map(|q| q.text.as_str())
            .collect();
        assert_eq!(texts, ["párrafo 2", "inciso c"]);
        // A genuine article list and word ordinals still link every article.
        let edges = closure_references(
            "Artículo 95.- Lo previsto en los artículos 10, párrafo segundo, y 2 de esta Ley.",
            "urn:lex-mx:federal:statute:fix",
            &["de esta ley"],
            &[],
            &known,
        );
        assert_eq!(
            edges
                .iter()
                .filter(|e| e.reference_form == ReferenceForm::Direct)
                .count(),
            2
        );
    }

    #[test]
    fn a_later_semicolon_clause_does_not_govern_a_direct_citation() {
        // L4: the Constitution named in a new clause must not take article 1.
        let known = [
            "urn:lex-mx:federal:statute:fix:article:1",
            "urn:lex-mx:federal:constitution:cpeum:article:1",
        ];
        let external = [(
            "constitución política de los estados unidos mexicanos",
            CPEUM,
        )];
        let edges = closure_references(
            "Artículo 5.- Se aplicará el artículo 1; la Constitución Política de los Estados Unidos Mexicanos reconoce derechos.",
            "urn:lex-mx:federal:statute:fix",
            &["de esta ley"],
            &external,
            &known,
        );
        assert!(
            edges.iter().all(|e| e.target_instrument_id != CPEUM),
            "{edges:?}"
        );
        // A semicolon inside the citation list does not close it.
        let edges = closure_references(
            "Artículo 5.- Se aplicará lo previsto en los artículos 1; 2 y 3 de la Constitución Política de los Estados Unidos Mexicanos.",
            "urn:lex-mx:federal:statute:fix",
            &["de esta ley"],
            &external,
            &[
                "urn:lex-mx:federal:constitution:cpeum:article:1",
                "urn:lex-mx:federal:constitution:cpeum:article:2",
                "urn:lex-mx:federal:constitution:cpeum:article:3",
            ],
        );
        assert_eq!(
            edges
                .iter()
                .filter(|e| e.target_instrument_id == CPEUM)
                .count(),
            3,
            "{edges:?}"
        );
    }

    #[test]
    fn a_longer_title_does_not_match_a_shorter_configured_alias() {
        // L5: "Ley del Seguro Socialista" contains "ley del seguro social".
        let known = ["urn:lex-mx:federal:statute:lss:article:3"];
        let external = [("ley del seguro social", LSS)];
        let edges = closure_references(
            "Artículo 5.- Conforme al artículo 3 de la Ley del Seguro Socialista.",
            "urn:lex-mx:federal:statute:fix",
            &["de esta ley"],
            &external,
            &known,
        );
        assert!(
            edges.iter().all(|e| e.target_instrument_id != LSS),
            "{edges:?}"
        );
        let edges = closure_references(
            "Artículo 5.- Conforme al artículo 3 de la Ley del Seguro Social.",
            "urn:lex-mx:federal:statute:fix",
            &["de esta ley"],
            &external,
            &known,
        );
        assert!(edges.iter().any(|e| e.target_instrument_id == LSS));
    }

    #[test]
    fn parenthesized_inciso_after_a_fraction_keeps_both_scopes() {
        // L7: "la fracción II, inciso a) del artículo 1 de esta Ley".
        let known = ["urn:lex-mx:federal:statute:fix:article:1"];
        let edges = closure_references(
            "Artículo 5.- Conforme a la fracción II, inciso a) del artículo 1 de esta Ley.",
            "urn:lex-mx:federal:statute:fix",
            &["de esta ley"],
            &[],
            &known,
        );
        let edge = edges
            .iter()
            .find(|e| e.target_provision_id == known[0])
            .unwrap();
        let texts: Vec<_> = edge.qualifiers.iter().map(|q| q.text.as_str()).collect();
        assert_eq!(texts, ["fracción II", "inciso a)"], "{edge:?}");
    }

    #[test]
    fn a_law_named_inside_a_decree_title_is_not_the_cited_law() {
        let external = [(
            "ley federal de remuneraciones de los servidores públicos",
            "urn:lex-mx:federal:statute:lfremsp",
        )];
        let known = ["urn:lex-mx:federal:statute:lfremsp:article:5"];
        let skipped = closure_references(
            "Artículo 1.- En términos del artículo 5 del DECRETO por el que se abroga la Ley Federal de Remuneraciones de los Servidores Públicos, se pagará.",
            "urn:lex-mx:federal:statute:pef",
            &["de esta ley"],
            &external,
            &known,
        );
        assert!(skipped.is_empty());
        let cited = closure_references(
            "Artículo 1.- En términos del artículo 5 de la Ley Federal de Remuneraciones de los Servidores Públicos, se pagará.",
            "urn:lex-mx:federal:statute:pef",
            &["de esta ley"],
            &external,
            &known,
        );
        assert_eq!(cited.len(), 1);
        assert_eq!(
            cited[0].target_instrument_id,
            "urn:lex-mx:federal:statute:lfremsp"
        );
        // Ordinary words must not hide an internal citation.
        let internal = closure_references(
            "Artículo 1.- Se allanen a los artículos 15 y 117, en los términos del acuerdo que suscriban los acreedores.",
            "urn:lex-mx:federal:statute:lcm",
            &["de esta ley"],
            &[],
            &[
                "urn:lex-mx:federal:statute:lcm:article:15",
                "urn:lex-mx:federal:statute:lcm:article:117",
            ],
        );
        assert_eq!(internal.len(), 2);
    }

    #[test]
    fn regulation_titles_keep_citing_the_regulated_articles() {
        // Like "Ley Reglamentaria del artículo 6o.… de la Constitución", a
        // regulation title cites the articles it regulates; only a decree's
        // embedded law title is not a citation (see the decree test).
        let known = [
            "urn:lex-mx:federal:statute:lft:article:121",
            "urn:lex-mx:federal:statute:lft:article:122",
        ];
        let external = [("ley federal del trabajo", LFT)];
        let title = closure_references(
            "Artículo 1.- Se abroga el Reglamento de los Artículos 121 y 122 de la Ley Federal del Trabajo, publicado en el Diario Oficial.",
            "urn:lex-mx:federal:regulation:reg-x",
            &["de este reglamento"],
            &external,
            &known,
        );
        assert_eq!(title.len(), 2);
        let cited = closure_references(
            "Artículo 1.- Conforme a los artículos 121 y 122 de la Ley Federal del Trabajo, se resolverá.",
            "urn:lex-mx:federal:regulation:reg-x",
            &["de este reglamento"],
            &external,
            &known,
        );
        assert_eq!(cited.len(), 2);
        // A regulation named by its own configured title still receives the edge.
        let reg = closure_references(
            "Artículo 1.- Los artículos 262 y 269 del Reglamento de la Ley Federal del Trabajo aplican.",
            "urn:lex-mx:federal:statute:cff",
            &["de este código"],
            &[
                ("ley federal del trabajo", LFT),
                (
                    "reglamento de la ley federal del trabajo",
                    "urn:lex-mx:federal:regulation:reg-lft",
                ),
            ],
            &[
                "urn:lex-mx:federal:regulation:reg-lft:article:262",
                "urn:lex-mx:federal:regulation:reg-lft:article:269",
            ],
        );
        assert_eq!(reg.len(), 2);
        assert!(
            reg.iter()
                .all(|edge| edge.target_instrument_id == "urn:lex-mx:federal:regulation:reg-lft")
        );
    }
}
