# Parser rule inventory — parser responsibility split

Status: G1 inventory, baseline `85c41f3898d86737766564fa8d3fd8f6d4ae74a3`
(line numbers refer to that commit). Part of
[the execution plan](plans/parser-responsibility-split.md). Every production
rule group in `crates/lex-parse/src` is listed once with the crate that owns it
after the split. Fixture names are the inline `#[test]` functions that pin the
rule; "negative" means the nearest ambiguous, held or fail-closed case. Nothing
here changes a rule: groups move whole, with their tests, and no predicate is
rewritten or broadened. Accepted-decision links point at the repository
documents that record the rationale.

Owners: **grammar** = `lex-grammar`, **analysis** = `lex-analysis`,
**link** = `lex-link`, **parse** = `lex-parse` (retained).
"Span effect" says whether the rule creates, preserves or narrows a character
span of the source text; none may alter source text.

## 1. Lexical grammar — owner: grammar

| ID | Rule group | Source family | Symbols today | Preconditions / state | Span effect | Hold / fail-closed | Positive fixture | Negative / ambiguous fixture | Decision |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| GR-01 | Article label grammar: number, suffix letters, digraphs, Latin qualifiers, ordinal abbreviations, thousands separators, hyphen tails | all | `labels::ArticleLabel`, `match_label_at`, `find_labels`, `parse_component`, `parse_number`, `parse_qualifier`, `match_suffix_token_at`, `match_qualifier_word_at` (labels.rs:19–543) | stateless, byte-offset cursor over one text | reports label ranges; never edits text | none; unmatched text yields no label | `compound_secondary_identifier_is_one_label`, `latin_ordinals_continue_the_qualifier_series`, `ordinal_spellings_are_semantically_equivalent`, `thousands_separators_are_part_of_the_number` | `a_digraph_never_swallows_a_following_letter`, `letter_suffix_does_not_swallow_qualifiers`, `compound_identifier_does_not_cross_paragraph_boundary`, `article_list_remains_separate_labels` | `decisions.md`, `ordinals-split-reg-lan-review-2026-10-18.md` |
| GR-02 | Label identity and ordering: slug, canonical slug, accent stripping, sort keys | all | `labels::slugify_label`, `canonical_slug`, `strip_accents`, `ArticleSortKey` | stateless | none | none | `sort_keys_order_suffixed_articles`, `enye_is_a_distinct_letter_suffix` | `quintus_is_a_ranked_qualifier_after_quater`, `slash_ordinal_normalization_is_bound_to_the_exact_numeric_suffix` | `decisions.md` |
| GR-03 | Connector words shared by glossary delimiter reading and term-variant generation | all | `grammar::is_connector` (moved from terms.rs) | stateless | none | none | `resolves_usages_with_variants_priority_and_positional_capitals` | `longest_match_wins_over_embedded_terms` | `glossary-coverage-header-review-2026-10-19.md` |
| GR-04 | Spanish calendar-date syntax: DOF date runs, list dates, `spanish_date`, month names | statute, regulation, reform evidence | `DOF_DATE`, `DOF_DATE_RUN`, `LIST_DATE`, `latest_dof_date`, `mentions_dof_date`, `spanish_date`, `MONTHS`, `month_number` (lib.rs:310–343, 2806; temporal_derive.rs:486–510) | stateless | none | none | `latest_dof_date_reads_every_date_in_a_note_list`, `literal_date_with_word_day_resolves` | `a_first_of_month_publication_date_written_with_the_ordinal_mark_is_recognized` | `held-source-parser-contracts.md` |
| GR-05 | Exact character slicing and shared whitespace/slug helpers | all | `char_slice`, `collapse_whitespace`, `slug` (lib.rs:2784, 2791, 2795) | stateless | slices only; offsets are chars | `None` outside range | covered through `validate_reference_span` fixtures | `term_slugs_keep_only_letters_digits_and_single_hyphens` | `legal-model.md` |
| GR-06 | Roman numerals (glossary fractions, transitory labels) | statute, regulation | `terms::canonical_roman`, `roman_value` (terms.rs:1315–1365); `diputados::parse_roman_transitory_start` | stateless | none | `None` for non-canonical numerals (glossary hold `NoncanonicalNumeral`) | `established_pass_reads_a_period_between_numeral_and_qualifier` | `fraction_labels_normalize_their_qualifier_case_only` | `glossary-g0-g2-outlier-review-2026-10-18.md` |

Grammar owns lexical helpers only. Legal classifiers (what a repeal or
commencement means) stay out of grammar. GR-06 moves only if both parse and link
call it; otherwise it stays with its single caller.

## 2. Analysis — owner: analysis

| ID | Rule group | Source family | Symbols today | Preconditions / state | Span effect | Hold / fail-closed | Positive fixture | Negative / ambiguous fixture | Decision |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| AN-01 | Initial temporal status and initial repeal notes, evaluated at parse time | statute, regulation, DCG | `initial_temporal_status`, `initial_repeals`, `ORDINAL_WORDS`, `repeal_ordinals` (lib.rs:1310–1457; `repeal_ordinals` takes a `RepealScope`, a legal classification, so it is analysis not grammar); called by `dcg.rs:535,541` and `diputados.rs:1128,1134` | parse-time call, identical timing and inputs; an explicit note only | none | ordinary provisions start `Unknown`; only narrow notes classify | `unanalyzed_temporal_status_is_unknown_except_for_an_explicit_repeal_note`, `repeal_marker_alone_on_its_line_is_a_repeal_but_a_sentence_is_not` | `partial_repeal_notes_are_classified_only_in_the_unambiguous_shape` | `legal-model.md` |
| AN-02 | Deterministic commencement derivation, article classification, date repair, partial-repeal repair | statute, regulation | `temporal_derive::*` (temporal_derive.rs:1–592): `derive_article_temporal_determinations`, `resolve_commencement*`, `repair_article_dates`, `repair_partial_repeals`, `Commencement`, `SkipReason`, `RULE_SET_VERSION` | reads transitories + publication date + today's date; only `not_analyzed` provisions | none | commencement skipped, never guessed: business days, month units, exception clauses, multiple clauses, wrapper text | `day_after_publication_classifies_all_non_repealed_articles`, `relative_days_with_word_count_resolves`, `future_publication_date_yields_future_effective` | `exception_clause_is_skipped_even_with_a_recognizable_date`, `multiple_commencement_clauses_are_skipped`, `nonexact_publication_never_guesses_an_endpoint_or_changes_dates`, `already_analyzed_articles_are_never_touched` | `legal-model.md`, `operator-decisions-2026-10-20.md` |
| AN-03 | Corpus validation: counts, order, provisions, terms, references, spans, targets, amendment marks | all | `validate_lritf`, `validate_corpus`, `CorpusExpectations`, `CorpusView`, `validate_counts`, `validate_provisions`, `validate_article_order_by_label`, `validate_terms`, `validate_references`, `validate_reference`, `validate_reference_span`, `validate_reference_target`, `resolution_rule_consistent`, `validate_repeals_and_condition`, `validate_amendment_marks`, `plain_article_number` (lib.rs:1458–1546, 1852–2800) | reads supplied records; no parser options | none | emits issues; never repairs | `validates_expected_counts_and_order`, `gap_tolerant_ordering_accepts_suffixes_and_warns_on_gaps`, `each_edge_records_how_its_target_was_chosen` | `unfrozen_count_baselines_warn_without_failing`, `strict_ordering_still_rejects_gaps_when_not_allowed`, `validation_rejects_a_resolution_rule_that_does_not_fit_the_edge`, `validation_rejects_an_effective_status_without_temporal_analysis` | `decisions.md` |
| AN-04 | Audited review application: outdated-reference approvals and legal-operator reviews | statute, regulation | `apply_outdated_reference_approvals`, `apply_legal_operator_reviews` (lib.rs:2608–2684) | exact edge identity + source hash; supplied approvals only | none; annotates one exact edge | an approval that does not match fails; other missing targets still fail | `legal_operator_review_annotates_one_exact_edge_without_changing_it`, `exact_reviewed_stale_reference_is_retained_and_other_missing_targets_fail` | same pair | `operator-decisions-2026-10-20.md` |

Review application (AN-04) and validation (AN-03) land in separate modules of
lex-analysis, and neither depends on parse.

NOM/NMX standard validation (`standard_validate::validate_standard` and its
helpers) is **deliberately retained in lex-parse**. At G1 it imports six
recognition helpers from `standard` (`parse_standard_modification_targets`,
`strip_page_furniture`, `real_transitorios_heading`, `closing_signature_start`,
`is_dof_page_furniture`, `derive_supplement_legal_character`) and re-runs
recognition over the source, so it is not independent of document parsing and
the plan's condition for moving it is not met. It stays a parse module
(PA-09 below); moving it would need a behaviour-preserving extraction of those
helpers first, which is out of scope.

## 3. Linking — owner: link

| ID | Rule group | Source family | Symbols today | Preconditions / state | Span effect | Hold / fail-closed | Positive fixture | Negative / ambiguous fixture | Decision |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| LK-01 | Citation extraction entry points and options | all | `extract_references`, `extract_internal_references`, `ReferenceOptions`, `InstrumentContextPolicy`, `HistoricalTarget`, `ReferencePatterns`, `canonical_article_id`, `numeric_article_number` (lib.rs:143–470, 1547–1556; the id builder is used only here) | supplied provisions, known targets, options and context; no I/O | records `[start,end)` char offsets into unchanged text | unresolved edges are kept as unresolved | `extracts_compound_qualified_repeated_and_ranged_internal_references`, `resolves_cross_instrument_and_title_references_deterministically` | `explicit_missing_self_targets_remain_unresolved_but_implicit_candidates_are_suppressed` | `parent-law-mapping-2026-10-16.md` |
| LK-02 | Own-law / parent-law resolution and bare `ley`/`reglamento` context | statute, regulation, DCG | `bare_own_regulation_context`, `bare_parent_marker`, `explicit_internal_context`, `retain_internal_candidates`, `group_target`, `sentence_target` (lib.rs:601–1016) | the instrument id and its declared parent | none | a bare `ley` never takes a dated, titled, cited or anaphoric law | `a_regulations_bare_ley_resolves_to_its_parent_only_when_the_article_exists`, `a_configured_full_title_outranks_its_bare_parent_prefix` | `a_bare_ley_never_takes_a_dated_titled_or_cited_law`, `quoted_shorthand_for_the_parent_law_is_not_retargeted_by_a_later_law`, `a_law_named_inside_a_decree_title_is_not_the_cited_law` | `parent-law-mapping-2026-10-16.md`, `linking-code-audit-2026-10-16.md` |
| LK-03 | Relative references (previous/next/same-type neighbour) | statute, regulation | `relative_neighbors`, `extract_relative_references` (lib.rs:368–600) | document order within one instrument | none | self-only in a regulation for bare `reglamento` | `resolves_relative_references_by_same_type_document_order`, `resolves_article_that_precedes_as_previous_article` | `relative_references_respect_external_governing_context`, `relative_reference_is_not_retargeted_by_a_law_named_in_a_later_clause` | `linking-code-audit-2026-10-16.md` |
| LK-04 | Qualifiers, fractions and citation-list boundaries | all | `extract_qualifiers`, `anchored_qualifier`, `pre_number_qualifiers`, `extract_same_article_fractions`, `qualifier_boundary`, `instrument_context_boundary`, `continues_citation_list`, `accepted_numbers`, `names_qualifier_number` (lib.rs:1089–1142, 1625–1848) | bounded look-ahead (`MARKER_LOOKAHEAD_CHARS` = 90) | narrows the cited range | none | `captures_pre_number_qualifiers_and_same_article_fractions`, `captures_noun_first_and_penultimate_pre_qualifiers` | `suffix_qualifier_ends_at_a_word_boundary_and_accepts_hyphenated_numbers`, `paragraph_numbers_are_qualifiers_not_article_links`, `a_later_semicolon_clause_does_not_govern_a_direct_citation` | `linking-code-audit-2026-10-16.md` |
| LK-05 | External-instrument context markers and exclusions (dated, historical, anaphoric) | all | `EXTERNAL_INSTRUMENT_MARKERS`, `has_internal_instrument_context`, `has_external_instrument_context`, `excluded_reading`, `bare_word_follows`, `find_marker` (lib.rs:778–933, 1557–1624) | marker lists are data; order preserved | none | unsupported anaphora and historical wording stay unlinked | `ordinal_title_citation_keeps_following_external_instrument_context`, `constitutional_article_list_keeps_numbers_and_noun_first_qualifiers` | `unsupported_anaphora_and_historical_wording_stay_unlinked`, `punctuation_does_not_hide_a_dated_modifier`, `quoting_ley_does_not_bypass_dated_or_anaphoric_exclusions` | `linking-code-audit-2026-10-16.md` |
| LK-06 | Transitory citations and edge construction: direct edges, range expansion, resolution rule and status | statute, regulation | `extract_transitory_citations`, `direct_reference_edges`, `range_expansion_edges`, `reference_edge`, `reference_resolution_status`, `with_rule` (lib.rs:1017–1342) | records the `ResolutionRule` for each edge | records ranges | an unmatched target is `Unresolved`, never silently dropped | `each_edge_records_how_its_target_was_chosen`, `extracts_compound_qualified_repeated_and_ranged_internal_references` | `flattened_table_zero_then_article_number_is_unlinked_without_changing_text` | `parser-linker-repair-2026-10-16.md` |
| LK-07 | Defined-term usage matching: variants, priority, longest match, positional capitals, quoted forms | all | `extract_term_usages`, `extract_term_usages_with_lowercase`, `extract_quoted_term_usages`, `term_usages`, `provision_usages`, `defined_name_spans`, `inside_own_definition`, `capitalization_is_informative`, `term_variants`, `pluralize`, `singularize`, `matches_at`, `starts_word`, `paragraphs_with_offsets` (terms.rs:1623–1993) | supplied terms and provisions; adapter opts in to lower-case matching | usage spans over unchanged text | none | `resolves_usages_with_variants_priority_and_positional_capitals`, `longest_match_wins_over_embedded_terms` | `a_term_at_its_own_definition_site_hides_shorter_terms_inside_it`, `quoted_usage_form_matches_only_the_quoted_vocable`, `lower_case_usages_match_only_when_the_adapter_opts_in` | `glossary-coverage-header-review-2026-10-19.md` |

Link takes records, terms and context as explicit arguments. It never reads a
file or the parse options type.

## 4. Parse — retained in lex-parse

| ID | Rule group | Source family | Symbols today | Preconditions / state | Span effect | Hold / fail-closed | Positive fixture | Negative / ambiguous fixture | Decision |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| PA-01 | Extraction from retained source: PDF, DOC, HTML text | all | `extract_pdf`, `extract_doc`, `Extraction`, `html::extract_html_text`, `pdftotext_version` (lib.rs:54–142, 2825) | external `pdftotext`/`textutil`/`antiword` versions recorded | defines the extracted text | empty extraction fails | `decodes_declared_utf8`, `preserves_legacy_latin1_when_declared` | `falls_back_to_latin1_for_malformed_declared_utf8`, `ignores_commented_utf8_declarations` | `metadata-portability-and-ingestion-automation.md` |
| PA-02 | Source-law ownership and nested decree/law selection | statute (Diputados) | `partition_nested_instrument`, `partition_enactment`, `is_omnibus_enactment_title`, `is_decree_article_wrapper`, `is_plural_decree_article_wrapper`, `is_dropped_decree_block` (diputados.rs:692–760, 1143–1447) | adapter-declared law/decree titles | partitions provisions by owner; wrapper text excluded explicitly | missing, duplicate, reversed or wrong-title boundaries fail closed | `nested_omnibus_enactment_keeps_law_and_overall_decree_transitories`, `explicit_layer_partition_preserves_law_decree_and_elided_siblings` | `nested_boundaries_fail_closed_on_missing_duplicate_reversed_and_wrong_title`, `omnibus_wrapper_without_overall_transitory_header_fails_loudly`, `plural_decree_wrapper_is_inert_without_an_omnibus_title` | `held-source-parser-contracts.md` |
| PA-03 | Annex and transitory bounds, enactment signatures, stop markers | statute, DCG, ITF | `is_enactment_signature`, `is_stop_marker`, `is_configured_document_stop`, `is_substantive_annex_start`, `is_judicial_appendix`, `is_transitory_section_header`, `parse_annex_document` | adapter stop markers; order within document | ends a section before signatures | lower-case transitory heading fails closed | `configured_stop_marker_excludes_enactment_signatures_from_transitory`, `parses_transitory_boundaries_and_annex_tables` | `budget_signatures_do_not_leak_into_last_transitory_before_substantive_annex`, `judicial_dispositions_are_not_reform_transitories`, `a_lower_case_transitory_heading_fails_closed_until_a_source_needs_it` | `held-source-parser-contracts.md` |
| PA-04 | Page furniture and running headers | all | `is_page_furniture`, `is_dof_edition_header`, `publisher_title_lines`, `furniture_audit::detect_bands`, `audit_pdf`, `find_admitted_furniture`, `standard::strip_page_furniture` | positional bands; threshold 0.95 | strips furniture from parsed text; offsets still address source | reports bands; blocks ingestion on repeated header | `dof_edition_running_headers_never_enter_provision_text`, `standalone_header_footer_block_is_reported_but_embedded_legal_citation_is_not` | `coverage_threshold_pinned_at_0_95_excludes_the_0_6_false_positive`, `a_single_page_cannot_establish_a_recurring_band` | `ingestion-difficulty-log.md` |
| PA-05 | Article/heading recognition and numbering within a source family | statute, DCG, ITF | `parse_article_start`, `parse_ordinal_article_start`, `parse_separatorless_article_start`, `parse_statute_article_start`, `HeadingPatterns`, `is_provision_start`, `parse_transitory_start`, `transitory_ordinals`, `dcg::TRANSITORY_ORDINALS`, `dcg::HeadingState`, `itf::parse_main_text` | adapter heading patterns | creates provision boundaries | unread article heading after a structural heading fails closed | `parses_codigo_shapes`, `parses_quintus_article_qualifier`, `parse_fixture` (dcg, itf) | `an_unread_article_heading_after_a_structural_heading_fails_closed`, `colon_is_not_a_general_ordinal_article_separator`, `a_cover_note_quoting_another_decrees_articles_is_not_an_article` | `ordinals-split-reg-lan-review-2026-10-18.md` |
| PA-06 | Amendment marks and reform-decree evidence | DCG, ITF, statute | `dcg::PendingMarks`, `amendment_marker_regex`, `itf::flush_reform`, `finalize_reform`, `extract_reform_evidence`, `ReformEvidenceBuilder`, `reform_evidence_item`, `extract_dof_publication` | marks attach to their paragraph | marks never alter text | an orphaned or contentless marker is reported, not dropped silently | `parses_compiled_document_with_amendment_markers`, `reform_decree_transitory_sections_receive_distinct_evidence_ids` | `a_marker_with_no_following_content_is_reported_not_silently_dropped`, `a_marker_orphaned_by_a_structural_heading_is_discarded_silently` (documents current behaviour) | `held-source-parser-contracts.md` |
| PA-07 | Glossary extraction: delimiters, quoted, nested incisos, wider/established/profiled passes, repeal placeholders and holds | all | `extract_terms`, `extract_glossaries`, `detect_glossary`, `detect_glossary_terms`, `GlossaryHold`, `GlossaryDetection`, `GlossaryStyle`, `established_glossary`, `detect_wider_glossary`, `profiled_glossary_entries`, `fraction_entries`, `colon_entries`, `quoted_respectively_entries`, `complete_entries`, `repeal_placeholders`, `unread_fractions`, `invalidated_fractions`, `find_glossary_provision` (terms.rs:39–1622, 1994–2003) | adapter glossary profile; fail closed on drift or an unused hint | term spans over unchanged text | `GlossaryHoldCode`: sequence break, unknown qualifier, non-canonical numeral, no term delimiter, swallowed fraction, repeal wording | `established_pass_reads_every_qualifier_spelling_and_dof_boundary`, `profile_reads_parent_and_inciso_terms_with_inciso_spans` | `wider_pass_holds_an_incomplete_glossary_instead_of_extracting_part`, `profile_fails_closed_on_any_drift_or_unused_hint`, `quoted_respectively_refuses_an_unpaired_sentence` | `glossary-g0-g2-outlier-review-2026-10-18.md`, `glossary-coverage-header-review-2026-10-19.md` |
| PA-08 | NOM/NMX standard parsing: clauses, transitories, supplements, modification targets | standard | `parse_standard_clauses`, `parse_standard_transitories`, `parse_standard_supplements`, `parse_standard_modification_targets`, `standard_tail_layout`, `apply_amendment_marks` (standard.rs:122–745) | verified source/text hashes | exact spans over the retained text | a decree title naming no numeral yields no targets; an unmatched target is reported, not resolved to an ancestor | `parses_numbered_standard_with_exact_spans`, `a_compound_decree_title_separates_modified_added_and_eliminated` | `an_unmatched_target_is_reported_rather_than_resolved_to_an_ancestor`, `a_date_inside_a_decree_title_is_never_read_as_a_numeral` | `cnbv-consolidated-disposiciones.md` |
| PA-09 | NOM/NMX standard validation (retained) | standard | `standard_validate::validate_standard`, `validate_metadata`, `validate_standard_sources`, `validate_clauses`, `validate_clause_coverage`, `validate_no_page_furniture`, `validate_transitories`, `validate_supplements`, `validate_supplement_character`, `validate_trailing_material` | typed standard records + retained text; re-runs recognition | none | reports; furniture, coverage, trailing material, supplement character | `validate_standard_stays_quiet_once_furniture_is_stripped` | `trailing_material_after_transitorios_is_reported`, `clause_coverage_below_the_floor_is_reported`, `a_compact_annex_below_any_byte_threshold_is_still_reported_by_name` | `cnbv-consolidated-disposiciones.md` |

## 5. Compatibility surface and open placements

- Public `lex_parse::` paths stay valid through thin re-exports. The 88 public
  items and their first-party callers are recorded by
  `prompts/parser-split-ab.py inventory`; the hash-bound listing is in the
  evidence file.
- `standard.rs` straddles analysis (AN-05) and parse (PA-08). `terms.rs`
  straddles parse (PA-07) and link (LK-07), sharing `slug` and the roman helpers
  (GR-06). `lib.rs` carries extraction (PA-01), link (LK-01–LK-06), analysis
  (AN-01, AN-03, AN-04) and grammar (GR-03–GR-05); at G1 these become modules
  inside lex-parse with no behaviour change, so every later move is a crate
  boundary move only.
- Mapping check. `split_lib.py` sent every unlisted `lib.rs` item to the
  `references` module by default, so a successful build does not prove the
  mapping. The items that landed in `references.rs` were therefore listed and
  checked by name: `ReferencePatterns`, `ReferenceSource`,
  `InstrumentContextPolicy`, `ReferenceOptions`, `HistoricalTarget`,
  `GroupTarget`, `RelativeNeighbors`, `extract_internal_references`,
  `extract_references`, `extract_source_references`,
  `extract_relative_references`, `extract_reference_group`,
  `extract_transitory_citations`, `relative_neighbors`,
  `bare_own_regulation_context`, `with_rule`, `retain_internal_candidates`,
  `explicit_internal_context`, `group_target`, `sentence_target`, `find_marker`,
  `bare_parent_marker`, `bare_word_follows`, `excluded_reading`,
  `names_qualifier_number`, `accepted_numbers`, `direct_reference_edges`,
  `range_expansion_edges`, `reference_edge`, `reference_resolution_status`,
  `EXTERNAL_INSTRUMENT_MARKERS`, `has_internal_instrument_context`,
  `has_external_instrument_context`, `extract_qualifiers`, `anchored_qualifier`,
  `pre_number_qualifiers`, `extract_same_article_fractions`, `fraction_labels`,
  `qualifier_boundary`, `instrument_context_boundary`, `continues_citation_list`,
  `MARKER_LOOKAHEAD_CHARS`, `canonical_article_id`, `numeric_article_number`.
  Every one is a citation-link rule (LK-01 to LK-06); none is validation,
  review application, classification or parse-family logic.
- Measured cross-module edges at G1 (production code): references and usages
  depend only on grammar; validate depends on grammar, labels and initial;
  initial depends on nothing in-crate after `repeal_ordinals` moved into it;
  temporal_derive depends on initial and grammar; review depends on nothing.
  There is no analysis to link edge and no analysis or link edge into any
  parse-family module. The parse family (dcg, diputados, itf, terms, standard,
  standard_validate, reform) depends on initial, grammar, labels, usages and
  each other. The only edge that crosses the planned boundary the wrong way is
  `standard_validate` to `standard`, which is why that module is retained in
  parse.

## 6. G2 status — grammar and analysis extracted

Moved whole, with their tests where those tests build their own inputs:

- **lex-grammar** (`crates/lex-grammar`): `labels` (GR-01, GR-02), the DOF and
  Spanish date helpers (GR-04), `char_slice`, `collapse_whitespace`, `slug`
  (GR-05) and `is_connector` (GR-03). Dependencies: `chrono`, `regex` only; no
  project crate.
- **lex-analysis** (`crates/lex-analysis`): `initial` (AN-01), `temporal_derive`
  (AN-02), `validate` (AN-03) and `review` (AN-04), as separate modules.
  Dependencies: `lex-core`, `lex-grammar`.
- **lex-parse** keeps every public path through re-exports: `lex_parse::labels`
  and `lex_parse::temporal_derive` are module re-exports, and the former
  top-level functions are `pub use` of the analysis crate.

Deliberate public-surface decisions at this boundary:

- `initial_temporal_status` and `initial_repeals` are public because the parse
  families call them at parse time with unchanged timing and inputs.
- `validate_references` and `validate_reference` became public (generic over the
  map and set hasher, as the workspace's pedantic lints require) because three
  cross-stage tests drive them with edges produced by the parser and linker.
  Those tests stay in lex-parse; they need the parser, so they cannot move into
  analysis without a reverse dev-dependency.
- Tests that build their own inputs moved with their code: eight chunks into
  `lex-analysis::validate` (four tests and four fixture helpers:
  `plain_article_number_reads_bare_ordinals_only`,
  `partial_repeal_notes_are_classified_only_in_the_unambiguous_shape`,
  `partial_repeal_and_condition_records_are_required_where_the_status_needs_them`,
  `header_contamination_requires_standalone_page_furniture`, plus
  `repeal_fixture_provision`, `repeal_codes`, `sample_repeal`,
  `sample_condition`), two tests into `lex-analysis::initial`
  (`unanalyzed_temporal_status_is_unknown_except_for_an_explicit_repeal_note`,
  `repeal_marker_alone_on_its_line_is_a_repeal_but_a_sentence_is_not`) and one
  test into lex-grammar (`latest_dof_date_reads_every_date_in_a_note_list`).
  `repeal_codes` is a helper, not a test. The lex-parse test count fell from 283
  to 238 and lex-grammar plus lex-analysis hold 17 and 28, so the sum is
  unchanged (283).

Measured production dependency graph after G2: grammar has no project
dependency; analysis depends on core and grammar; parse depends on analysis,
core and grammar. No edge points back from analysis or grammar to parse, link,
cli, db or export.

Remaining before G3: link (LK-01 to LK-07) still lives in lex-parse as the
`references` and `usages` modules, and `standard_validate` stays in parse as
recorded above.
