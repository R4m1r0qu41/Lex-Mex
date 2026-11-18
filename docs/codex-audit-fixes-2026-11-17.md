# Codex audit fixes, 2026-11-17: review request

Claude implemented the fixes for the Codex audit of 2026-11-17
(`docs/commencement-and-reader-branch-codex-review-2026-11-17.md`, on this branch as `f3b1039a4`).
JRH directed it: "fix all issues the goal ends with a review file". Nothing is pushed or merged, and no
corpus file changed. This file asks Codex for one bounded re-review.

## Identity

| Branch | Worktree | Range | Content |
|---|---|---|---|
| `fix/codex-audit-2026-11-17` (new, from `origin/main` `47a3ffc59`, no upstream) | `/Users/jr/Dev/lex-mex-integration` | `47a3ffc59..` this commit | the commencement port, C1–C4, C6, R5 on `main` code |
| `feat/public-reader-r7` | `/Users/jr/Dev/lex-mex-reader` | `6a239b1c5..921e1bd6c` | R1, R2, R4, R5, R6 in the D3 adapter, and the `write_binding` race |
| `rule/bounded-commencement-2026-10-27` | none (written through a temporary worktree, since removed) | `1ae2a7a90..a283f85ff` | C5 superseded notes only |

**Port branch commits.** Five commits are cherry-picked from the old branch with `-x`, all code or calendar data:
`22f284c64` (from `0a6fa9e97`), `ebc78d6ce` (`ed2abf54f`), `245a8cd78` (`7478586e2`), `f710e6b8d` (`dfd7053c0`) and
`f55e9584f` (`8fbfd9137`). They are followed by your verdict, `f3b1039a4`, and the fix commit `0c8c69dc0`. The old
branch's corpus snapshots (`2cc5c7a5d`, `230791871`) and record commits are not ported. `reg-lfpc-art122` therefore
stays `unknown` on this branch, as on `main`, until the in-force status pass applies the rule.

## What changed, by finding

| Finding | Fix | Where | Regression test |
|---|---|---|---|
| C1, plus your 10-28 limit 1 | Every legacy date path (day after, same day, literal date, calendar days) must match the whole clause, anchored from a plain subject to the end. Qualifier and business-day guards run on accent-folded, whitespace-collapsed text. A `días` count accepts only `naturales` between number and direction, so `hábiles` (accented or not) never reaches it. | `crates/lex-analysis/src/temporal_derive.rs`: `resolve_commencement_text`, `LEGACY_SUBJECT`, `LEGACY_TAIL`, `anchored`, the four `try_*` | `an_unaccented_business_day_clause_never_becomes_a_calendar_day_date`, `legacy_date_paths_accept_only_the_whole_clause` |
| C3 | Qualifiers and negation are matched as whole words on folded text: `Interno entrará` is not a negation; `no  entrará` and `no` plus a line break are. | `EXCEPTION_MARKERS`, `folded` | `negation_is_matched_as_words_across_any_whitespace`, `qualifiers_are_matched_without_accents_or_case` |
| C2, plus limit 4 | A new `schemas/business-day-calendar-v1.schema.json`: every key required, no unknown keys, `schema_version` const `"1"`, provenance strings non-empty. `parse_calendar` validates against it, then deserialises into `deny_unknown_fields` types, then checks semantics: ordered ranges, non-overlapping statute versions, real month and day pairs. The CLI's permissive loader is removed. A missing file is still an empty calendar, which resolves nothing. | `crates/lex-analysis/src/business_days.rs`, `crates/lex-cli/src/main.rs` `load_business_day_calendar` | `a_misspelled_missing_or_extra_calendar_key_is_an_error`, `calendar_semantics_are_checked_after_the_schema`, CLI `derive_rejects_a_misspelled_calendar_key_and_leaves_the_corpus_untouched` |
| Limit 3 | The 1 December entry has `last_year: 2018`. A count that touches a later occurrence year (2024, 2030, …) is `Insufficient`. This does **not** decide whether article 28 then excludes 1 October; that stays a legal question for JRH. | calendar JSON, `SexennialHoliday`, `walk` | `a_window_reaching_an_unevidenced_transmission_year_is_insufficient` |
| C4 | The derive save gate counts `split`. | `main.rs` `run_temporal_derive` | CLI `derive_saves_a_partial_repeal_reclassification_even_when_it_is_the_only_change` (fails with the old gate; checked) |
| C6 | `repair_article_dates` takes the calendar. The CLI loads it once, before both the repair and the derive. | `temporal_derive.rs`, `main.rs` | `date_repair_uses_the_same_calendar_as_the_derivation` |
| C5 | Dated superseded notes on the old branch. Historical measurements are kept. | old branch `a283f85ff` | records only |
| R5 (main code) | `lex_db::sqlite_sidecars` appends `-wal`, `-shm` and `-journal` to the full file name. It is used in `verify.rs` and `build.rs`. | `crates/lex-db/src/{lib,verify,build}.rs` | `sidecar_tests` |
| R1 | A list item that alone exceeds the limit returns `response_too_large` with a `skip_cursor` after it. The response schema gains that optional field, so the D3 schema bytes change. | reader `public.rs` `page`, `Failure::Oversized`; `schemas/public-query-response-v1.schema.json` | `an_oversized_item_returns_a_skip_cursor_and_traversal_continues` |
| R2 | Release-index links are classified without throwing. The strict pin applies to the selected link only. | `reader/retrieval-check.mjs` | four extra-link tests and one cross-origin control |
| R4 | The deadline is checked between search hits and after dispatch (`>=`). | reader `public.rs` `past_deadline`, `handle`, `search` | `work_after_sqlite_still_counts_against_the_deadline` |
| R5 (reader) | Same helper, used in `open_readonly`, `verify` and `build`. | reader `crates/lex-db/src` | `a_live_sidecar_is_found_whatever_the_database_extension` |
| R6 | Opening resolves annexes with the binder's own rule (`annex_sources_by_number`, by configured URL) and then checks the bound digest. | reader `load_scope` | `a_bound_annex_opens_when_another_manifest_shares_its_content` |
| `write_binding` race | `create_new` with `sync_all`. | reader `public.rs` | the existing `binding_is_exact_and_refuses_overwrite` |
| Number tables | One `NUMBER_WORDS` for both forms (adds `cien`, `ciento veinte`, `ciento ochenta` to calendar-day counts). | `temporal_derive.rs` | in `the_corpus_clause_shapes_still_resolve` |

R3 needs no change: you refuted it as a version-name collision.

## Corpus reach

`crates/lex-cli/examples/commencement_census.rs` is new and read-only: it prints every instrument's commencement verdict,
treating every provision as untouched and using an empty calendar. Comparing the ported baseline with the fix:

- 468 instruments; 8 change verdict; **no date changes** among instruments that still resolve.
- **To review (7).** In each, the commencement text is not a single plain clause:
  - Ley de la Casa de Moneda de México (a second sentence);
  - Ley Federal de Defensoría Pública (a subject naming two instruments);
  - Ley de la Fiscalía General de la República ("y se expide en cumplimiento…");
  - Ley General en Materia de Desaparición Forzada de Personas, Desaparición Cometida por Particulares y del Sistema Nacional de Búsqueda de Personas (a second sentence);
  - Ley Nacional del Sistema Integral de Justicia Penal para Adolescentes (a declaratoria);
  - Ley de Transparencia y de Fomento a la Competencia en el Crédito Garantizado (a second sentence, and the transitory was reformed);
  - Reglamento del Senado de la República (a second verb, then a second sentence).
- **Newly resolved (1):** Ley de Desarrollo Sustentable de la Cafeticultura, "a los ciento ochenta días de su publicación", resolving to 2026-06-17 (published 2025-12-19).
- **The tail allows two kinds of extraction leftover**, both measured in the corpus:
  - a legislative signature block (place and date, an optional quoted motto, then only capitalised-name `, D. P.-` style signatures ending in `Rúbricas`), in 4 instruments, among them the Ley de Vías Generales de Comunicación;
  - a bare page number, in 1 instrument.

  The names are matched case-sensitively, and a test shows that a sentence cannot pass as a name. Please judge whether
  the signature allowance is acceptable or whether those instruments should go to review instead.

Committed determinations are untouched: derive acts only on `not_analyzed` provisions, and the repair changes nothing
when commencement does not resolve.

## Checks run

- Port branch at `0c8c69dc0`, through `mech`:
  - `cargo fmt --all --check`: pass.
  - `cargo test --workspace --locked --offline`: 573 passed, 0 failed.
  - `validate` passes for `lritf`, `ifpe-dcg-2021`, `lic`, `lmv`, `lfpa`, `reg-lfpc-art122` and `lvgc`. `lic` reports "valid … 2 issues": both are `historical_superseded_reference_outdated` warnings (reviewed stale citations kept without a live link, art. 187 → `lraf` 28 Bis and art. 194 → `lfce` 31 Bis), and `corpus/mx/lic` is identical to `origin/main`.
  - `cargo clippy --workspace --all-targets --locked --offline -- -D warnings`: clean (run directly).
- Reader branch at `921e1bd6c`:
  - `cargo test -p lex-db`: 31 lib tests pass. The four new R1, R4, R5 and R6 tests fail against the previous `public.rs` (checked).
  - `node --test reader/retrieval-check.test.mjs`: 20 pass, 5 fail against the previous checker (checked).
  - Clippy is clean for `lex-db`. Two clippy errors in `lex-cli` (`main.rs:1905`, `db.rs:318`) are present without this commit and are not touched.
  - Full reader workspace, through `mech`: `cargo fmt --all --check` passes; `cargo test --workspace --locked --offline` exits 0 with 449 passed, 0 failed, 1 ignored doc-test.
- Not run: national replay, DB rebuild, holdout access or the pilot lookup.

## Questions

1. Do the fixes close C1–C4, C6 and R1, R2, R4, R5, R6 as you dispositioned them? Do they close your 10-28 limits 1, 3 and 4?
2. Is the anchored subject and tail too strict or too loose? In particular, is the signature-block allowance acceptable?
3. R1: is a `skip_cursor` on the error an acceptable explicit oversized-item policy, given that it changes the D3 response schema?
4. Is the port branch a sound base for the corpus-wide in-force rule (JRH decision 5)?

## Constraints

- Read-only except your verdict file: `docs/codex-audit-fixes-codex-review-2026-11-17.md` in this worktree, uncommitted.
- Don't push, merge or deploy.
- The fintech holdout stays sealed: don't open `holdout-gold.json`, don't pass `--holdout-gold`, and don't run `structure_pilot_lookup` on `lritf`, `lic` or `lmv`.
- Route any question that turns on law, such as the article 28 treatment of 2024 onward, to JRH.
