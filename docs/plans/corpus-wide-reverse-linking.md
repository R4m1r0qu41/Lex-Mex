# Corpus-wide reverse linking

Living execution plan for closing cross-instrument references across the committed federal corpus. A fresh agent must be able to resume from this file and the repository.

## Purpose and observable outcome

Every committed instrument under `corpus/mx/` is relinked against the instruments that exist now. Resolved citations become canonical reference edges and presentation links in Markdown. A relink that does not validate is left unchanged and named in `.work/corpus-closure.json`. Audited temporal decisions are not modified. No database index is built.

## Scope and exclusions

In scope: the Rust linker, the curated alias table, one corpus-wide `link --all` pass, validation, and Markdown republication for graphs that still validate.

Out of scope:

- SQLite or any other derived reader index. The nested reader builds that index from a committed graph, so it waits until this closure is accepted.
- Temporal model execution, legal-review resolution, and changes to audited determinations.
- Inventing colloquial or accent-stripped aliases. Those resolve only when the alias table already configures them.
- Linking the ambiguous official title `Ley de Amnistía` (`lamn` and `lamni`).
- The deferred DCGs that are not in the corpus, `egdf`, and the open `reg-lfmn` / `lfiiedb` decisions.

## Authority

`AGENTS.md`, the Rust linker, and committed corpus files outrank this plan. The predecessor checkpoint is the cluster plan's 2026-10-16 note that structural ingestion is complete (467 instruments) and corpus-wide reverse linking had not been scoped.

## Initial baseline

Local `main` at `25732f3ef` (`chore: remove misplaced nota-viabilidad comites banca multiple`). 467 `corpus/mx/*/instrument.json` files. Every stored validation report was valid. The alias table had 66 keys, 55 of which matched a committed `short_name`; 412 instruments had no alias entry. `link <slug>` reloaded every sibling corpus, which does not scale to this closure.

## Current checkpoint

2026-10-16, branch `feat/corpus-reverse-linking`, rebased onto `7326c41aa`, not on `main`. After the tightening pass below, `target/release/lex-mex link --all` exits non-zero by design because instruments are held at their previous graphs. After the Codex repair commit that follows, `lgdnna` is rejected by rule as well as by hand (its previous graph was restored at the operator's request). A second run relinks 0 and produces no diff beyond `lgdnna`.

| Result | Count |
|---|---:|
| Instruments | 467 |
| Relinked and validated | 304 |
| Unchanged | 150 |
| Held at previous graph | 21 (see below) |
| References | 27,429 → 35,893 |
| Resolved cross-instrument edges | 9,769 |
| Retargeted edges (all reviewed) | 44 |
| Removed internal edges (all reviewed) | 54 (13 re-spanned to the full suffix, 41 false links to another named law) |
| Removed cross-instrument edges | 0 |

No `provisions.json`, `instrument.json`, `terms.json`, `term-usages.json` or source manifest changed, and every `validation.json` is valid. Persisted historical and approved edges are byte-identical.

Held instruments: 11 cite an article absent from the target corpus (`cpf`, `lbm`, `lftr`, `lgmime`, `lgs`, `lic`, `lmv`, `lpdusf`, `lsar`, `reg-lan`, `reg-lisr`) `lrsic` is held by the historical-edge guard, and `lgdnna` is held by operator request pending JRH review. After the parser/linker repair, eight more are held because an explicit citation to a missing own-article is now retained as unresolved: `cub-dcg-2005`, `cucb-dcg-2004`, `lgcs`, `lgoaac`, `oaac-dcg-2009`, `reg-lagra-mcdets`, `reg-lfce`, `socap-sofipo-dcg-2006` (unverified; several look like `Bis`-suffix parser gaps). `link --all` on the landed tree: relinked 0, unchanged 446, rejected 21. Per-target evidence and classification (target parser defects vs stale citations) is in the `corpus-wide-reverse-linking` entry of `docs/ingestion-difficulty-log.md`.

## Tightening pass (2026-10-16, before landing)

The first closure was reviewed against `25732f3ef` by diffing the whole graph (removed, retargeted and added cross-instrument edges), not a sample. Findings that gate landing:

| # | Class | Evidence | Action |
|---|---|---|---|
| T1 | Constitution self-citation retargeted | CPEUM Art. 3 "artículo 123 de esta Constitución … Ley Federal del Trabajo" now links to LFT Art. 123; Art. 73 "artículo 21 de esta Constitución … Ley Nacional sobre el Uso de la Fuerza" links to LNUF. `self_reference_markers("constitution")` has only ley-shaped phrases, so a later official title wins. | **Fixed.** Added constitution self markers (`de esta constitución`, `de la presente constitución`, `esta constitución`, ` constitucional`). Fixture + test. |
| T2 | Decree/regulation/title mentions linked to the embedded law | PEF "artículo 5 del DECRETO por el que se abroga la Ley Federal de Remuneraciones…" links to LFREMSP; "Reglamento de los Artículos 121 y 122 de la LFT" links to LFT. | **Fixed.** Unambiguous instrument-title openings (`del decreto por el que`, `del decreto que`, `del acuerdo por el que`, `del acuerdo sobre`, `del estatuto de`, `de las reglas de carácter general`, `de las reglas generales`, `de los lineamientos para`) are generic markers: with no configured title at that noun the edge is skipped. Bare `del acuerdo` and `de la norma` were rejected (ordinary Spanish). A title-mention guard for `Reglamento/Reglamentaria de(l/los)` was tried and dropped: regulation titles cite the articles they regulate (existing ordinal-title fixture). |
| T3 | Persisted historical edge changes (LRSIC) | LRSIC Art. 2 → `lraf` ("resolved"); base stored `lraf-1990` historical. **Not caused by the closure:** `main`'s own `link lrsic` produces the same `lraf` edge. The source note on fraction V reads `DOF 23-01-2004, 01-02-2008, 09-03-2018`, after `historical_until` 2014-01-10, so the existing dating rule selects the successor. | Do **not** change the rule. **Guard implemented.** Closure guard: reject any instrument whose new graph changes or drops a persisted `historical_target_unavailable` or approved-outdated edge. LRSIC joins the rejected list. **Operator/JRH question:** is the 2018 re-enactment of fraction V a citation of the 2014 LRAF (rule-consistent) or should the stored 1990 identity stand? |
| T4 | Ley de Amparo running title | Uppercase "LEY REGLAMENTARIA DE LOS ARTÍCULOS 103 Y 107 DE LA CONSTITUCIÓN…" leaked into `provisions.json` and links to CPEUM. | Furniture defect in canonical text. Out of scope (would change provisions). Count and log in `docs/ingestion-difficulty-log.md`. |
| T5 | 11 rejected instruments | One missing target article rejects the graph. | Classify each read-only (stale citation vs parser defect). No fixes in this landing. |

Acceptance for every change: no diff to `provisions.json`, `instrument.json`, `terms.json`, `term-usages.json`; persisted historical and approved edges byte-identical; a targeted scan of new/retargeted cross edges for (a) a self-reference phrase between span and matched marker and (b) a head noun between span and matched marker returns zero; every retarget and removed cross-instrument edge reviewed; a second `link --all` produces no diff; every rejection explained. Stop and ask if any check fails or a removed resolved cross-instrument edge cannot be explained.

## Next action

Operator approval to land the branch on `main` (one commit, explicit paths, no push without a separate request). Then: answer the `lrsic` question (T3), and schedule the target-side parser fixes and approved outdated references for the held instruments as separate bounded passes. Do not drop the missing citations, do not invent the missing articles, and do not start the reader database. `SERVINV-DCG-2013` stays an unmatched alias key.

## Decision log

- 2026-10-16: Backlink the canonical graph before any database. The reader SQLite index is a rebuildable derivative of instruments, provisions, and typed edges. Indexing first would store an incomplete graph and have to be rebuilt.
- 2026-10-16: A committed official title is a marker when it contains a space and is unique among committed instruments. Curated aliases still supply accent-stripped and colloquial forms. A marker that the alias table and an official title assign to different instruments is an error, not a silent choice. Adapter historical overrides still win.
- 2026-10-16: `link --all` loads the corpus once. It writes `references.json`, `terms.json`, `term-usages.json`, `validation.json`, and Markdown only when the new graph validates and differs. Provisions and instrument records stay untouched.
- 2026-10-16: Seven in-corpus DCG alias keys were renamed to the committed `short_name` (`DCG-CUB-2005` and the six siblings). Deferred DCG keys with no committed instrument were left unchanged.
- 2026-10-16: Closure review found three linker defects, fixed before landing: constitution self-citations captured by a later official title; decree titles embedding a law title; and an over-broad head-noun marker. A closure guard now rejects any relink that changes a persisted historical or approved edge. The title-mention guard was tried and removed because regulation titles cite the articles they regulate (existing ordinal-title fixture).

## Progress

- [x] (2026-10-16) Tightening pass: constitution self markers, decree head nouns, historical-edge guard, regression tests (lex-parse 198, lex-cli 34 pass). Re-ran the closure from the `25732f3ef` baseline and again after the rebase; full-graph audit clean (see checkpoint).
- [x] (2026-10-16) Unique official-title markers, `link --all`, and the seven confirmed DCG alias-key corrections. `cargo test -p lex-cli --bin lex-mex` (32 tests) and `cargo clippy -p lex-cli --all-targets -- -D warnings` passed.
- [x] (2026-10-16) Release closure recorded above. Provision text unchanged. Full workspace `cargo test` / `cargo clippy` / the two audited validators were not re-run after the corpus write.

## Milestones

1. Linker accepts unique official titles and `link --all`, with fixtures for a resolved title, a duplicate title, and a missing article that is not written. Done.
2. Corpus-wide closure. Done, with the 11 rejections preserved on their previous graphs.
3. Record the outcome here and in `docs/project-status.md`. Done. Stop. Do not index a database and do not commit unless the operator asks.

## Outcome

The worktree corpus now has 2,996 additional reference edges and 4,147 resolved cross-instrument edges. 305 instruments were republished. 151 needed no graph change. 11 stay on their pre-closure graphs because each cites at least one article the target corpus does not contain. `Ley de Amnistía` remains deliberately unlinked. Nothing was committed.

## Recovery

The closure computes every new graph before writing. A failure while extracting links leaves the corpus unchanged. A failure while writing leaves the instruments printed as `relinked` updated and the rest unchanged; rerun `link --all` after fixing the reported error. Do not delete corpus directories to retry.

## Follow-up 2026-10-16: suffix-citation fixes (operator review of held list)

Operator review of the held instruments found three citation-parser defects, fixed in `lex-parse` with one regression test each and relinked (8 instruments, 19 remain held; `link --all` now rejects 19):

- **Suffix word boundary:** `136 tercer párrafo` was read as `136 Ter` (`lbm`, released from hold).
- **Hyphenated suffix numbers:** `5 Bis-1` lost its number (`lgoaac`, released; false edges removed in `ccom` and `lcm`).
- **`decies` family headings:** running-text headings such as `Artículo 78 Terdecies.` linked to `78 Ter` or the parent article (`lac`, `lgamvlv`, `lgmdfp`, `reg-lsem`); those false edges are removed.

Every added, removed and retargeted edge in the 8 relinked graphs was read against its provision text. Still held and awaiting decisions or other fixes: see `docs/held-instruments-review-2026-10-16.md`.

## Follow-up 2026-10-16: operator decisions on the held list

Nine held instruments were admitted (ten at first; `reg-lan` was withdrawn the same day, see the decision entry) with exact stale-citation approvals plus a `legal_operator_review` note each (see the 2026-10-16 decision entry): `lgdnna`, `lgmime`, `lsar`, `lgcs`, `reg-lfce`, `lic`, `reg-lisr`, `reg-lagra-mcdets`, `lpdusf` (17 citations). `link --all` relinked these 10 (30,407 → 30,575 references; every added, removed and retargeted edge read; the removals are false links such as headings and mis-spanned `74` for `74 quinquies`) and rejects 9: `cpf`, `lgs`, `lmv` (target-side suffix parser gaps), `lftr` (embedded invalidated articles and a misattributed governing law), `lrsic` (operator decided fraction V resolves to the current LRAF; migration pending), and `cub-dcg-2005`, `cucb-dcg-2004`, `oaac-dcg-2009`, `socap-sofipo-dcg-2006` (for a DCG pass).

Correction the same day: `reg-lan` was relinked without an approval. The quoted `de la “Ley”` / `de la "Ley"` forms are now generic external-instrument markers, which removed 124 false internal edges (citations of the Ley de Aguas Nacionales read as articles of the Reglamento, plus the LFPA misattribution); `lic` carries the corrected LFCE article 3 fracción IX note. Totals: 30,451 references, 4,312 resolved cross-instrument edges.
