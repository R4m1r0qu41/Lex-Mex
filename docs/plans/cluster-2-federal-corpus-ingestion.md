# Complete federal corpus ingestion and cross-instrument validation

This is a living execution plan for the prepared Lex-Mex federal corpus ingestion work. Keep `Progress`, `Surprises and Discoveries`, `Decision Log`, and `Outcomes and Retrospective` current whenever work proceeds. A fresh agent must be able to resume from this repository and this file without the original conversation.

## Purpose and observable outcome

Lex-Mex must ingest the prepared Mexican federal laws and regulations through the repository's Rust pipeline, not through the retired vault tooling. At completion, every admitted instrument has an official source manifest and hashes, canonical provisions, frozen structural baselines, validated references and defined-term usages, a zero-error validation report, and generated Markdown. Cross-instrument links must be recomputed after their targets enter the corpus, and the full repository must pass its deterministic checks without changing audited legal decisions.

This plan governs structural ingestion, processing, linking, validation, and publication preparation. Temporal model analysis is deliberately deferred and must not be run merely because an instrument was structurally ingested. Obsidian is an optional presentation target; repository files remain canonical.

## Scope and exclusions

The source inventory is `prompts/lex-mex-federal-cluster-2-plan.md` and the 53 prepared manifests under `prompts/cluster-2-batches/`. Operational manifests belong under `batches/` and must validate against `schemas/batch-manifest.schema.json`. Begin by finishing `batches/constitutional_CN1_congress.json`, then admit later prepared batches in the order recorded by the cluster plan unless a dependency, blocked source, or operator decision requires a documented change.

This plan does not authorize:

- temporal model execution or automatic resolution of legal uncertainty;
- clearing a `blocked` manifest entry without JRH confirmation;
- changing an audited reviewer identity, resolution, or review history;
- treating a consolidated Cámara text as a formal DOF source when the distinction matters;
- importing canonical facts from Obsidian or the retired Python pipeline;
- destructive cleanup of the existing dirty source wave;
- bulk commits that combine unrelated instruments or parser fixes.

## Repository authority and orientation

Read `AGENTS.md` first. It owns the architectural, source-integrity, legal-review, model-routing, and validation rules. Read `README.md` for CLI behavior, `docs/decisions.md` for accepted parser and ingestion decisions, and `docs/project-status.md` as a useful but potentially stale project snapshot. Repository code, tests, accepted decisions, manifests, canonical corpus files, and current Git state outrank this plan when they conflict.

The relevant implementation surfaces are:

- `batches/*.json`: executable batch manifests consumed by `lex-cli batch run`;
- `schemas/batch-manifest.schema.json`: the admitted batch shape;
- `prompts/cluster-2-batches/*.json`: prepared source inputs, not yet operational manifests;
- `adapters/diputados/*.json` and `adapters/cnbv/*.json`: source and parser configuration;
- `crates/lex-cli/src/main.rs`: batch orchestration and individual pipeline stages;
- `crates/lex-source/`: acquisition, hashing, adapter, and manifest types;
- `crates/lex-parse/`: canonical parsing, references, terms, and validation;
- `crates/lex-export/`: Markdown and optional Obsidian publication;
- `fixtures/`: regression evidence for every material parser defect;
- `corpus/mx/<slug>/`: committed canonical output and validation evidence;
- `.work/`: ignored downloaded and extracted work products plus batch reports.

`batch run` performs fetch, extract, parse, validation, optional count freezing, Markdown export, and optional Obsidian export. Parsing also extracts the current instrument's references and terms. The standalone `link <slug>` command recomputes references and terms for a previously parsed corpus against the siblings currently present.

## Initial baseline

The audited starting point for this plan is local `main` at `488057a5ce979d6261028e143ad7cbe6fab58fe7` (`Ingest Senate regulations and harden Diputados parsing`). At that commit:

- `locg` is committed and validates with 151 articles, 8 original transitories, and zero issues (`97fa5cbc`);
- `reg-diputados` is committed and validates with 323 articles, 13 original transitories, and zero issues (`553baa6e`);
- `reg-senado` is committed and validates with 313 articles, 4 original transitories, and zero issues (`488057a5`);
- `.gitignore`, the two federal cluster plans, and 53 prepared cluster-2
  manifests are an existing dirty source wave and must be preserved.

This initial baseline is immutable historical context. Later progress belongs
in `Current checkpoint` and `Progress`, not in this list.

## Current checkpoint

Verified against local `main` at `e503364bb` (30 commits ahead of remote
`main` at `47004f56`):

- CN1 (`locg`, `reg-diputados`, `reg-senado`, `rgic`, `ldofgg`) and CN2 (10
  instruments) are committed and structurally closed; the CN2 reverse-link,
  validation, and Markdown pass closed at `34449eb6`.
- The active AD1 batch has committed `lplan`, `lfep`, `reg-lfep`, and
  `lfrpe`; `lfrsp` and `lgbn` remain.
- Per-instrument counts, source and extracted-text hashes, and validation
  state are owned by each instrument's `corpus/mx/<slug>/validation.json` and
  `source-manifest.json` and are not restated here. The dated `Progress` log
  below is the append-only historical record of each ingestion and each
  shared parser, linker, or adapter correction, with commit ids.
- Shared-infrastructure milestones: the reviewed global-alias linker and
  bounded marker search at `942f201c`, the unanalyzed temporal default
  correction at `1f262295`, and bounded batch closure at `21863ef31`.
- The 55 prepared prompt files are committed at `ca6a4649`: 53 manifests
  under `prompts/cluster-2-batches/` and the two federal cluster plans.
- The divergent `fable/cross-linking` history is retained at the annotated
  tag `archive/fable-cross-linking` (peeling to `e7ed6369`) and is never
  merged as a unit; see the decision log.

Do not assume these statements remain current. At every resumption, compare them with `git log`, `git status`, the operational manifest, adapter presence, corpus presence, validation files, and the active-run drift report.

## Next action

The operator accepted the bounded closure on 2026-07-16. Provisionally process
next AD1 statute `lfrsp` through the Rust pipeline, inspect and freeze its
structural baseline, and record any reusable deterministic parser, linker, or
adapter learning before moving to the next AD1 instrument. Corpus-wide
relinking and human expected-edge review remain separate work.

## Progress

- [x] (2026-07-14 20:33Z) Ingested and committed `locg` at `97fa5cbc`; validation recorded zero issues.
- [x] (2026-07-14 21:02Z) Ingested and committed `reg-diputados` at `553baa6e`; validation recorded zero issues.
- [x] (2026-07-14 22:08Z) Ingested and committed `reg-senado` plus narrowly required parser fixtures and hardening at `488057a5`; validation recorded zero issues.
- [x] (2026-07-15 02:22Z) Reconciled the active-run capsule through `fabbb2c2`, bound it to this task-named plan, and advanced it to checkpoint 5 without changing the repository worktree.
- [x] (2026-07-14 22:44Z) Ingested, inspected, froze, relinked, validated, and committed `rgic` with the required parser regression at `2e061724`.
- [x] (2026-07-15 02:28Z) Ingested, inspected, froze, relinked, validated, and committed `ldofgg` at `727aa5d1`: 20 articles, 2 original transitories, 1 reference, 7 reform-transitory evidence records, and zero issues. Added a focused stop-marker fixture so enactment signatures do not contaminate its final transitory.
- [x] (2026-07-15 02:31Z) Relinked, validated, and regenerated Markdown for all five CN1 instruments. Counts remained 31/40/47/30/1 references, every validation reported zero issues, and Git recorded no canonical diff.
- [x] (2026-07-15 02:46Z) Reviewed `fable/cross-linking`, selectively implemented the safe global-alias and bounded-marker paths at `942f201c`, added LOCGEUM/RGIC regression coverage, and closed CN1 after a five-instrument reverse relink produced 22 reviewed resolved edges and zero validation issues.
- [x] (2026-07-15 03:10Z) Normalized prepared CN2 into `batches/constitutional_cn2_implementing_laws.json`, verified the 10 official Cámara source pairs, and advanced the pinned inventory to 28 manifests and 150 instruments.
- [x] (2026-07-15 03:10Z) Ingested and committed `lrfiyii-art105` at `b7653e22` after isolating its parser hardening at `c1952e50`: 74 articles, 4 original transitories, 46 references, 45 reform-transitory evidence records, stable source hashes, and zero validation issues. The complete required gate passed, including both audited baseline validators and the new instrument validator.
- [x] (2026-07-15 03:23Z) Ingested and committed `lrart6-mdr` at `7a62c205` after isolating two shared regressions at `42b8bb61`: 42 articles, 3 original transitories, 15 references, 3 defined terms, 6 reform-transitory evidence records, stable source hashes, and zero validation issues. The complete required gate passed with 82 workspace tests and both audited baseline validators.
- [x] (2026-07-15 05:00Z) Ingested and committed `lrfv-art76` at `40d41362` after isolating its shared reference corrections at `92004dc2`: 15 articles, 1 original transitory, 10 reviewed references, stable source hashes, and zero validation issues. The complete gate passed with 84 workspace tests, both audited baseline validators, and the affected-instrument validator.
- [x] (2026-07-15 15:05Z) Ingested and committed `lrart76-fracvi` at `99d588de` after isolating its constitutional article-list correction at `eee19102`: 25 articles, 1 original transitory, 9 reviewed references, 4 reform-transitory evidence records, stable source hashes, and zero validation issues. The complete gate passed with 85 workspace tests, both audited baseline validators, and the affected-instrument validator.
- [x] (2026-07-15 16:04Z) Ingested and committed `lrfxiiib-art123` at `03ff3fbb` after isolating its latest-reform date correction at `8be59440`: 25 articles, 4 original transitories, 4 reviewed references, 19 reform-transitory evidence records, stable source hashes, and zero validation issues. The complete gate passed with 86 workspace tests, both audited baseline validators, and the affected-instrument validator.
- [x] (2026-07-15 17:06Z) Ingested and committed `lrfxviii-art73` at `015eb8ca` after isolating its ordinal-word article correction at `5e03f735`: 2 articles, 1 original transitory, 2 reviewed references, no terms or reform evidence, stable source hashes, and zero validation issues. The complete gate passed with 87 workspace tests, both audited baseline validators, and the affected-instrument validator.
- [x] (2026-07-16 17:15Z) Ingested and committed `lfcpo` at `f9506734`: 65 articles, 6 original transitories, 15 reviewed references, 2 reform-transitory evidence records, stable source hashes, and zero validation issues. An instrument-scoped stop marker excluded enactment signatures from the sixth transitory; the complete gate passed with 87 workspace tests and both audited baseline validators.
- [x] (2026-07-16 17:18Z) Resolved the divergent-worktree integration policy: do not merge `fable/cross-linking` into `main` as a unit. Its 18 unique commits combine branch-only corpora with stale bulk relinks and conflicting parser, exporter, source, schema, and decision changes; preserve the history under `archive/fable-cross-linking` and reapply only reviewed units on current `main`.
- [x] (2026-07-16 19:14Z) Ingested and committed `lfrm` at `0598a28f`: 61 articles, 5 original transitories, 27 reviewed references, 8 defined terms with 108 usages, no reform evidence, stable source hashes, and zero validation issues. An instrument-scoped stop marker excluded enactment signatures and SCJN judgment appendices from the fifth transitory; an explicit `Constitución` mapping recovered seven exact CPEUM edges while LGIPE citations remain deferred until that prepared target enters the corpus. The complete gate passed with 87 workspace tests and both audited baseline validators.
- [x] (2026-07-16 19:17Z) Ingested and committed `lsct` at `6a8348b6`: 11 articles, 1 original transitory, 6 reviewed references, 20 reform-transitory evidence records, stable source hashes, and zero validation issues. An instrument-scoped stop marker excluded the 1991 enactment signatures, and the explicit `Constitución` mapping recovered CPEUM Articles 76 and 133. The 2021 Fiscalía decree contributes 16 evidence records and the 2025 civil-procedure decree contributes 4. The complete gate passed with 87 workspace tests and both audited baseline validators.
- [x] (2026-07-16 19:20Z) Ingested and committed `latime` at `e9619503`: 14 articles, 2 original transitories, 13 reviewed references, no terms or reform evidence, stable source hashes, and zero validation issues. An instrument-scoped stop marker excluded the 2004 enactment signatures, while a full-title LFT mapping recovered the exact Article 74 citation. The complete gate passed with 87 workspace tests and both audited baseline validators.
- [x] (2026-07-16 19:21Z) Closed the ten-instrument CN2 reverse-link, validation, and Markdown pass at `34449eb6`. Final CN2 totals are 334 articles, 28 original transitories, 147 references, 11 defined terms, 108 term usages, and 96 reform-transitory evidence records. Every validator reports zero issues; the only reverse-pass canonical change preserved the official-title qualifier `párrafo primero` on the existing `lrart6-mdr` CPEUM Article 6 edge.
- [x] (2026-07-16 20:18Z) Normalized and committed AD1 at `614fe4a9`, advancing the operational inventory to 29 manifests and 156 unique instruments. All six entries were absent, unblocked, and preserved in prepared order; the workspace tests, formatting, clippy, and both audited baseline validators passed. Live Cámara verification was inconclusive because the official host failed its TLS handshake.
- [x] (2026-07-16 20:21Z) Audited the accumulated ingestion regressions after CN2. The 87-test workspace strongly covers parser and temporal primitives, but identified two correctness risks before further scale: freshly parsed, unanalyzed provisions default to `effective` even when consolidated text records SCJN invalidity, and batch success neither reverse-relinks earlier instruments nor evaluates the manifest's `expected_edges` recall oracle. Secondary debt remains in exact-title alias discovery, pre-ingestion commit provenance, and untested CLI orchestration.
- [x] (2026-07-16 20:39Z) Corrected the unanalyzed temporal boundary at `1f262295`. Ordinary provisions now begin `unknown`; explicit source-text repeal notes remain deterministically `repealed`; and validation rejects an unanalyzed status that disagrees with that rule. The reviewed migration changed only temporal-status lines for 30,124 canonical provisions and matching Markdown across 144 corpora, retained 3,592 explicit repeals and all 22 reviewed states, passed 89 workspace tests and all required gates, and left every corpus validator valid.
- [x] (2026-07-16 20:49Z) Implemented bounded batch closure at `21863ef31`: after successful selected instruments enter the corpus, `batch run` reverse-relinks, validates, and republishes them, then records `expected_edges` as satisfied, missing, deferred, or invalid in its report. Missing or malformed concrete expectations fail the batch; unavailable targets and sources not processed in the run remain explicit deferrals. The CLI regression covers whole-instrument and article-scoped checks, missing edges, unresolved edges, unavailable targets, partial runs, malformed syntax, and a full temporary-corpus reverse-link/validate/export closure. The full gate passed with 91 workspace tests and both audited baseline validators.
- [x] (2026-07-16 21:00Z) Operator accepted the bounded batch closure and set the scalable operating rule: treat each batch as a local-code learning loop. Record reusable parser/linker defects as fixtures and deterministic code changes, source-specific findings as adapter configuration, and operating discoveries in this plan before the next instrument relies on them. Refreshed the stale project-facing status and orientation documents; Git history retains the superseded checkpoint snapshots.
- [x] (2026-07-16 21:36Z) Ingested and committed AD1 hub `lplan` at `2713decd6`: 48 articles, 5 original transitories, 7 resolved references, no terms, 64 reform-transitory evidence records, stable source hashes, and zero validation issues. The provisional parse exposed the original enactment-signature block after transitory QUINTO; the exact 1982 signature-date marker is configured only in the adapter, which removes that non-canonical text and its spurious CPEUM Article 89 edge. The frozen rerun, bounded closure, full 91-test workspace gate, and audited baseline validators passed.
- [x] (2026-07-16 22:11Z) Ingested and committed `lfep` at `d168dd79c`: 69 articles (including 59 BIS), 8 original transitories, 27 resolved references, no terms, 111 reform-transitory evidence records, stable source hashes, and zero validation issues. The LFEP reform appendix revealed that an operative `ARTÍCULO ÚNICO` quoting an earlier uppercase DOF date could overwrite the containing decree's publication date and strand a later transitory. Parser correction `0cedabdc8` limits date recognition to decree-title material, normalizes valid month casing, preserves context in any future error, and adds a focused fixture. An LFEP adapter marker separately excludes the original 1986 enactment signatures and their spurious CPEUM Article 89 edge. The frozen rerun and full 92-test gate passed.
- [x] (2026-07-16 22:50Z) Ingested and committed `reg-lfep` at `e10a49da4`: 46 articles, 3 original transitories, 2 resolved references, 5 defined terms with 39 usages, 5 reform-transitory evidence records, stable source hashes, and zero validation issues. The original third transitory ended before the exact 1990 enactment-signature block; that narrow boundary is adapter configuration, not a shared parser defect. The frozen rerun and full 92-test gate passed.
- [x] (2026-07-16 23:09Z) Ingested and committed `lfrpe` at `e503364bb`: 35 articles, 2 original transitories, 6 resolved references, no terms, 21 reform-transitory evidence records, stable source hashes, and zero validation issues. The original second transitory ended before the exact 2004 enactment-signature block; that narrow adapter marker removes the spurious CPEUM Article 89 edge without a shared parser change. The frozen rerun and full 92-test gate passed.
- [x] (2026-07-18) Operator review of the lfrpe checkpoint found Article 35 contaminated with dot-redacted enactment-decree articles (`ARTÍCULO SEGUNDO.- .........`). The shared parser now recognizes an ordinal-word decree article whose body is only redaction dots as non-content decree apparatus, with fixture `fixtures/diputados/dot-redacted-decree-article-sample.txt`; the frozen rerun changed only Article 35's canonical text and Markdown, preserving both hashes and all counts. Adapter hygiene from the same review: scaffold now defaults `allow_article_gaps` to `false`, freeze writes only `expected_articles`, the validator treats an exact-only article baseline as frozen, and lfrpe's adapter drops gap tolerance and the redundant minimum. Documentation deduplication: this plan's checkpoint section defers per-instrument facts to `validation.json`/`source-manifest.json` and this log; AGENTS.md session-lifecycle rules point to Agent Vault canon, and the provider rule distinguishes within-provider routing from operator-started cross-provider switches; the project-status warning census is restored to the reproducible 187 (162/16/7/2). Full workspace gates and both audited baseline validators passed.
- [ ] Normalize and admit each remaining prepared cluster-2 batch, then ingest its instruments in dependency order.
- [ ] Complete a corpus-wide relink, expected-edge audit, deterministic validation, and publication review.

## Surprises and discoveries

- Observation: the active-run capsule is behind live repository state.
  Evidence: checkpoint 3 names `reg-senado` as next, while documentation HEAD `8a3a0f9b` records valid committed corpora through `rgic`.
- Observation: batch closure now covers only successful selected instruments,
  not every committed corpus.
  Evidence: the closure reverse-links, validates, republishes, and evaluates
  expected edges for its successful source set; expectations with unavailable
  targets or sources not processed in the run are reported as deferred. A
  corpus-wide relink and human recall review remain intentionally separate.
- Observation: the former structural default assigned `effective` to every
  non-repeal provision before temporal analysis; this is resolved at
  `1f262295`.
  Evidence: LFRM Articles 32, 59, and 61 exposed the contradiction by retaining
  express SCJN invalidity notes while exporting `effective/not_analyzed`.
  `initial_temporal_status` now returns `Unknown` for ordinary text, the
  validator enforces that boundary, and the corpus-wide migration changed no
  content beyond the matching canonical and presentation status fields.
- Observation: exact official titles of committed siblings are not automatic
  cross-instrument markers.
  Evidence: global linking uses only the curated alias table plus per-adapter
  overrides; LATIME therefore needs an instrument-scoped full-title mapping
  for the already committed LFT target.
- Observation: regression coverage remains concentrated below the full
  fetch-through-export orchestration boundary.
  Evidence: the new CLI regression exercises expected-edge closure semantics,
  while live-source acquisition and full batch execution remain intentionally
  integration-tested manually against official hosts.
- Observation: exact sibling-title mentions do not necessarily imply an
  article-level graph edge.
  Evidence: after the registry became active, every CN1 citation containing a
  resolvable article number acquired a reviewed edge, including three links to
  LOCGEUM. Several RGIC title mentions describe whole-instrument continuing
  effect without naming a target provision, so they correctly remain prose;
  the regression fixture separately proves that an article-qualified RGIC
  title resolves when present.
- Observation: enactment signatures contaminate the final original transitory in many older committed Diputados corpora.
  Evidence: the provisional LDOFGG parse appended the 1986 congressional signatures and presidential promulgation block to Transitory Segundo; a repository-wide exact search found the same `Rúbrica` pattern in numerous existing final transitories, including `rgic` and `reg-senado`. LDOFGG is corrected narrowly with its existing adapter stop-marker boundary, but historical cleanup requires a separately reviewed canonical migration.
- Observation: Diputados documents can use title-case structural headings and
  place a new paragraph after an amendment mark separated by page furniture.
  Evidence: the provisional `lrfiyii-art105` parse absorbed `Capítulo III` into
  Article 70 and joined the following paragraph to a preceding `DOF` amendment
  marker until focused fixtures reproduced both boundaries.
- Observation: a single reform decree can contain more than one transitory
  section and restart ordinal labels within the same publication date.
  Evidence: the 1996-11-22 appendix in `lrfiyii-art105` contains two such
  sections; date-plus-ordinal evidence IDs collided until later sections were
  deterministically qualified as `section-2`.
- Observation: a title-case structural keyword can also be the first word of
  a source line wrapped inside an ordinary legal sentence.
  Evidence: Article 25 of `lrart6-mdr` wraps after `este`, leaving `Capítulo
  deberán señalarse:` at the start of the next physical line; prefix-only
  heading recognition introduced a false paragraph break until constrained
  to the exact heading grammar.
- Observation: the period in a Mexican ordinal article spelling is not a
  sentence boundary for cross-instrument context.
  Evidence: the title's `artículo 6o., ... de la Constitución` initially
  targeted the statute's own Article 6; retaining context after the ordinal
  dot correctly produces two reviewed CPEUM Article 6 edges, from the title
  and Article 1.
- Observation: historical constitutional titles and numeric ordinal paragraph
  qualifiers occur together in older Cámara consolidations.
  Evidence: `lrfv-art76` Article 9 cites `artículo 115, fracción III, inciso
  b), 2o. párrafo de la Constitución General de la República`; without the
  historical CPEUM alias and ordinal-aware boundary, the resolved Article 115
  edge and its three anchored qualifiers were omitted.
- Observation: older statutes use `artículo que precede` as a singular,
  deterministic synonym for `artículo anterior`.
  Evidence: `lrfv-art76` Article 6 refers to Article 5 with that exact phrase;
  a focused fixture now preserves the source span and previous-article target.
- Observation: a wrapped two-line Cámara running header must be excluded by
  both exact physical lines.
  Evidence: the provisional `lrart76-fracvi` parse otherwise appended header
  text to Articles 3, 6, 13, and 20 and the original transitory, producing
  spurious constitutional references until both lines were adapter-scoped.
- Observation: constitutional article lists can place the adjective before
  the first number and use noun-first paragraph qualifiers after later
  numbers.
  Evidence: `lrart76-fracvi` Article 25 cites `artículos constitucionales 109,
  fracción I; 110, párrafo segundo y 114, párrafo primero`; a focused fixture
  now preserves all three exact targets and qualifiers.
- Observation: one Cámara consolidation can contain stale running-header
  reform metadata before a newer title-page reform date.
  Evidence: `lrfxiiib-art123` repeats `Última Reforma DOF 31-07-2021` in its
  page furniture but states `Última reforma publicada DOF 14-11-2025` on the
  title page and includes the matching 2025 reform appendix; first-match
  selection therefore understated the canonical latest-reform date.
- Observation: a historical Cámara statute can number its operative articles
  with cardinal words before a separately headed transitory.
  Evidence: `lrfxviii-art73` uses `ARTICULO PRIMERO` and `ARTICULO SEGUNDO`
  before `TRANSITORIO` and `ARTICULO UNICO`; the prior decree-wrapper
  heuristic dropped both operative articles and retained only the transitory.
- Observation: LFCPo's final original transitory is followed immediately by
  the 2014 legislative and promulgation signatures before the 2021 reform
  appendix.
  Evidence: the provisional parse appended both signature blocks to
  Transitory Sexto and emitted a spurious CPEUM Article 89 edge; the accepted
  instrument-scoped stop marker removed only those blocks while preserving
  the 65/6 structure and both source hashes.
- Observation: `fable/cross-linking` is not a mergeable feature branch in its
  present form.
  Evidence: it has 18 patch-unique commits and changes 369 files with about
  130,000 inserted lines, including branch-only corpora, corpus-wide generated
  relinks, and conflicting edits in the parser, exporter, source layer, CLI,
  schema, and `docs/decisions.md`; `main` has independently advanced through
  seven CN2 ingestions and already reimplemented two reviewed linker fixes.
- Observation: LFRM combines consolidated invalidity annotations in operative
  provisions with full SCJN judgment appendices after the enactment block.
  Evidence: Articles 32, 59, and 61 correctly retain the Cámara brackets and
  invalidity notes, while the provisional Transitory Quinto incorrectly
  absorbed the signatures and both later judgment publications until the
  main-document stop marker was set at the 2021 legislative signature line.
- Observation: LFRM expressly defines `Constitución` as CPEUM and `Ley
  General` as LGIPE, but only CPEUM is currently committed.
  Evidence: the instrument-scoped CPEUM marker resolves seven exact article
  citations in addition to the full-title edge; LGIPE is prepared in batch
  EL1 but has no committed adapter or corpus, so its Article 369 and 370
  citations remain prose pending target admission and reverse relinking.
- Observation: LSCT is a short statute whose consolidated source appends two
  unusually broad multi-law decrees.
  Evidence: the operative corpus remains 11 articles and one original
  transitory, while the source-grounded evidence layer correctly retains 16
  transitories from the 2021 Fiscalía decree and 4 from the 2025
  civil-procedure harmonization decree without importing either decree's
  enactment signatures into canonical provision text.
- Observation: LATIME's express external citations exercise three already
  committed target classes without requiring a shared alias change.
  Evidence: CPEUM Articles 93 and 89 resolve through accepted constitutional
  titles, RGIC Articles 58 and 59 resolve through the global registry, and an
  instrument-scoped full-title mapping resolves LFT Article 74; whole-law
  mentions of LSCT and the foreign-trade law correctly remain prose because
  they name no target provision.

## Decision log

- Decision: structural ingestion precedes temporal analysis.
  Rationale: accepted repository decision dated 2026-07-11; the Rust pipeline is the sole ingestion gate and temporal review proceeds later by legal priority.
  Date/author: 2026-07-11 / JRH.
- Decision: process one new instrument at a time during parser expansion.
  Rationale: a single official PDF can expose a new structural variant; isolating one adapter, corpus diff, fixture set, and commit keeps provenance and parser consequences reviewable.
  Date/author: 2026-07-14 / execution-plan adaptation.
- Decision: do not freeze the first machine-proposed counts until the provisional parse has been inspected against the official document.
  Rationale: unfrozen validation warnings reveal proposed counts, but mechanically freezing an incorrect first parse would turn its mistake into the drift baseline.
  Date/author: 2026-07-14 / execution-plan adaptation.
- Decision: a batch is not complete when only its newly added instruments validate.
  Rationale: instruments parsed earlier cannot acquire references to targets that did not yet exist; close each batch with reverse relinking and expected-edge review.
  Date/author: 2026-07-14 / execution-plan adaptation.
- Decision: `batch run` closes only its successfully selected instrument set,
  then treats concrete `expected_edges` as a deterministic recall gate.
  Rationale: this repairs the default batch-success claim without triggering a
  corpus-wide rewrite. Missing or malformed concrete expectations fail; an
  unavailable target or a source not processed in the run remains an explicit
  deferral for later review.
  Date/author: 2026-07-16 / bounded batch-closure implementation.
- Decision: parser or linker defects discovered during execution receive their own fixtures, focused implementation diff, validation, and commit.
  Rationale: `AGENTS.md` requires regression evidence and forbids mixing unreviewed semantic changes into canonical data commits.
  Date/author: 2026-07-14 / repository policy.
- Decision: keep the LDOFGG signature correction instrument-scoped and defer the broader historical cleanup.
  Rationale: `main_document_stop_markers` is the accepted deterministic boundary for instrument-specific consolidated-document endings; changing the generic parser during checkpoint 5 would alter many committed canonical corpora without the required corpus-wide review.
  Date/author: 2026-07-15 / checkpoint 5 execution.
- Decision: selectively reimplement the two safe linker mechanisms from
  `fable/cross-linking` instead of merging or cherry-picking that branch.
  Rationale: the branch diverged before CN1 and bundles a new `regulates`
  field, broad adapter rewrites, bulk relinks, and unrelated parser/export
  work. Current `main` already contains the same alias registry under its
  intentionally ignored underscore filename, so consuming that file plus the
  independently reviewed marker-start bound closes CN1 with a narrow diff.
  Date/author: 2026-07-15 / checkpoint 6 execution.
- Decision: do not merge `fable/cross-linking` into `main`, now or after CN2,
  and do not use a synthetic conflict-resolution merge to mark it integrated.
  Rationale: the archived history is an aggregate execution line, not one coherent
  feature. A whole-branch merge would admit stale generated relinks and
  obsolete shared-code states alongside useful new corpora, obscuring the
  provenance and validation of each trusted boundary. Keep it as an archival
  source under the annotated tag `archive/fable-cross-linking` and transplant
  or reimplement only bounded commits after verifying them against current
  `main`; branch-only corpora should enter through the
  normal one-instrument ingestion gate, and semantic/schema features require
  their own fixtures and review.
  Date/author: 2026-07-16 / divergent-worktree integration review.
- Decision: preserve the established evidence IDs for the first transitory
  section of a reform decree and qualify only later same-decree sections with
  `section-N`; reject any remaining duplicate evidence ID.
  Rationale: this retains stable identifiers where unambiguous while making
  repeated ordinal labels unique without inventing legal-temporal conclusions.
  Date/author: 2026-07-15 / CN2 parser execution.
- Decision: recognize an immediate structural heading only when the entire
  physical line matches the accepted heading grammar, and treat `o.`, `a.`,
  `º.`, and `ª.` immediately after a citation number as ordinal punctuation
  when locating the governing instrument.
  Rationale: both rules preserve official line-wrap semantics while keeping
  true title-case headings and exact cross-instrument spans deterministic.
  Date/author: 2026-07-15 / CN2 `lrart6-mdr` execution.
- Decision: map the historical official title `Constitución General de la
  República` to the committed CPEUM target and treat numeric ordinal dots as
  qualifier punctuation rather than sentence boundaries.
  Rationale: the official 1978 source uses both shapes in express
  constitutional citations; dedicated fixtures preserve exact source and
  qualifier spans while resolving only targets already present in the corpus.
  Date/author: 2026-07-15 / CN2 `lrfv-art76` execution.
- Decision: keep the shorthand `constitucional` mapping adapter-scoped to
  CPEUM for `lrart76-fracvi`; the shared parser accepts the adjective syntax
  and noun-first qualifiers without globally assigning an instrument target.
  Rationale: this resolves the official source's express citations while
  avoiding a corpus-wide semantic reinterpretation of otherwise ambiguous
  shorthand.
  Date/author: 2026-07-15 / CN2 `lrart76-fracvi` execution.
- Decision: derive a Diputados consolidation's latest-reform date as the
  maximum valid date across all matching `Última Reforma` labels.
  Rationale: repeated running headers can lag a corrected title page; choosing
  the maximum recorded label is deterministic and remains grounded in the
  downloaded official source rather than inferred amendment effects.
  Date/author: 2026-07-15 / CN2 `lrfxiiib-art123` execution.
- Decision: normalize ordinal-word operative article headings to numeric
  ordinal labels, and classify an ordinal heading as a decree wrapper only
  when its body begins with a clear promulgation or amendment action.
  Rationale: the canonical label and ID retain established numeric ordering
  and reference semantics, while the official provision text remains
  unchanged and genuine enactment-decree articles stay excluded.
  Date/author: 2026-07-15 / CN2 `lrfxviii-art73` execution.

## Milestone 1: reconcile state and finish CN1

At the start of the next session, inspect the live repository before running any pipeline command:

    cd /Users/jr/Dev/lex-mex
    git status --short
    git branch --show-current
    git rev-parse HEAD
    git log -5 --oneline
    python3 /Users/jr/Vaults/Agent_Vault/AI/30_Executable/scripts/active_run.py discover --repo . --inject

Preserve the unstaged `.gitignore` and the staged 55-file `prompts/` source
wave. Reconcile the active-run checkpoint through the reviewed checkpoint
command so its completed milestones include `reg-senado` and `rgic`, its plan
binding names `docs/plans/cluster-2-federal-corpus-ingestion.md`, and its next
action is `ldofgg`. If live HEAD, index state, dirty state, or source digests
have moved again, record that drift before changing the capsule.

RGIC completed the following provisional structural sequence before its counts
were frozen; retain it as the checkpoint audit trail:

    cargo run --locked -p lex-cli -- adapter scaffold batches/constitutional_CN1_congress.json rgic
    cargo run --locked -p lex-cli -- batch run batches/constitutional_CN1_congress.json --only rgic --keep-work

Inspect at minimum:

- `adapters/diputados/rgic.json` for exact title, source URL, reference URL, publisher, parser, instrument type, and unfrozen fields;
- `.work/rgic/` and `.work/batch-report-constitutional_CN1_congress.json` for acquisition and stage results;
- `corpus/mx/rgic/source-manifest.json` for response metadata and both hashes;
- `corpus/mx/rgic/provisions.json` against the official PDF's article headings, original transitories, reform appendices, running headers, and page breaks;
- `references.json`, `terms.json`, `term-usages.json`, `reform-temporal-evidence.json`, and `validation.json` for structural plausibility rather than counts alone;
- generated Markdown for representative early, middle, final, bis/letter-suffixed, transitory, and reform-evidence cases;
- the Git diff for changes outside the expected adapter, corpus, Markdown, fixture, parser, and decision paths.

If the provisional parse is wrong, stop. Add the smallest representative fixture under `fixtures/diputados/`, write a failing regression test, make a narrow deterministic parser correction, and rerun the provisional pass. Do not freeze counts merely to make validation quiet. Do not alter official text manually in canonical JSON.

After the provisional counts and structure are defensible, freeze the machine-proposed baseline and immediately validate it again:

    cargo run --locked -p lex-cli -- batch run batches/constitutional_CN1_congress.json --only rgic --freeze-counts --keep-work
    cargo run --locked -p lex-cli -- link rgic
    cargo run --locked -p lex-cli -- validate rgic
    cargo run --locked -p lex-cli -- export rgic --format markdown

Run the same sequence for `ldofgg`, replacing the slug. Commit each instrument separately after reviewing its complete canonical diff. A parser fix may be a preceding separate commit when it affects existing instruments or is easier to review independently.

CN1 is complete only when all five operational manifest entries have adapters and valid corpus directories, every non-blocked batch result is `ok`, the frozen counts match the inspected official structures, no prior audited temporal or review state changed, and the reverse-link pass in Milestone 4 has run.

## Milestone 2: normalize the next prepared batch

The 53 files under `prompts/cluster-2-batches/` are prepared source manifests, while `lex-cli batch run` consumes the normalized schema under `batches/`. Admit one prepared batch at a time.

For each prepared batch:

1. Compare its entries with live `corpus/mx/` and `adapters/`; omit already ingested instruments from the operational manifest and explain each omission in `notes`.
2. Preserve every blocked entry and reason. Never convert a blocked entry into `NEW` without the recorded operator or legal-review decision.
3. Verify the official title, source PDF, reference page, source type, and dependency order from the prepared evidence. Do not repair URLs from memory.
4. Normalize into `batches/<domain-and-batch>.json` using only fields allowed by `schemas/batch-manifest.schema.json`.
5. Preserve `expected_edges` when the prepared plan predicts important cross-instrument relationships; treat them as a recall oracle, not as permission to invent a link.
6. Load the manifest through `adapter scaffold` or a restricted `batch run --only <slug>` before broad execution, so Rust deserialization and adapter uniqueness fail early.

The next default after CN1 is CN2 because it contains constitutional implementing statutes and high-value reference targets. Change that order only when the cluster plan, a source blocker, or a documented dependency justifies it.

## Milestone 3: ingest and process one operational batch

Process hubs before regulations and peers as specified by the operational manifest. For every slug, repeat the provisional-then-frozen sequence from Milestone 1. Do not run an entire unproven batch with `--freeze-counts` on its first encounter.

Once each instrument has a reviewed adapter or a successfully scaffolded Diputados adapter, a batch-level command may be used for the remaining admitted slugs:

    cargo run --locked -p lex-cli -- batch run batches/<manifest>.json --only <comma-separated-reviewed-slugs> --keep-work

After inspecting each provisional result, freeze only the approved subset:

    cargo run --locked -p lex-cli -- batch run batches/<manifest>.json --only <comma-separated-approved-slugs> --freeze-counts --keep-work

For CNBV entries, do not use the Diputados scaffold path. The current CLI intentionally requires a hand-written adapter. Verify the official CNBV source, annex and amending-resolution discovery, formal-source requirements, parser selection, and certificate behavior against the accepted CNBV decisions before running it.

Each instrument must leave these observable artifacts:

- a source adapter with reviewed identity and frozen structural expectations;
- source and extracted-text hashes in `source-manifest.json`;
- canonical instrument, provision, reference, term, usage, reform-evidence, and validation files appropriate to that instrument;
- zero validation errors, with every warning explained before commit;
- Markdown whose representative samples preserve official content and inject only presentation links;
- unchanged audited temporal results and review history unless the exact evidence-preserving reapplication rule applies.

Record the exact command, article/transitory/reform-evidence counts, reference and term counts, warnings, parser discoveries, commit, and next slug in this plan and the active-run checkpoint after every committed instrument.

## Milestone 4: close cross-instrument links

Do not claim cross-instrument completion while `_instrument-aliases.json` remains disconnected from the Rust linker. First address that defect as a separate reviewed implementation task with fixtures proving that:

- an official full-title or accepted alias maps to the intended committed instrument;
- accent-stripped and colloquial aliases resolve only when configured;
- ambiguous or short-form names stay unlinked;
- source spans remain exact and canonical text is unchanged;
- an instrument ingested earlier can acquire a link to a target added later;
- unknown external laws remain deliberately unlinked rather than becoming broken edges.

After the linker can consume the accepted alias data, relink and validate the newly completed batch plus every previously committed instrument that may cite one of its targets. At cluster completion, perform a corpus-wide closure pass:

    for dir in corpus/mx/*; do
      slug=${dir##*/}
      cargo run --locked -p lex-cli -- link "$slug" || exit 1
      cargo run --locked -p lex-cli -- validate "$slug" || exit 1
      cargo run --locked -p lex-cli -- export "$slug" --format markdown || exit 1
    done

Review both precision and recall. A zero-broken-target validator proves that emitted edges are internally valid; it does not prove that every intended citation was emitted. Compare each operational manifest's `expected_edges`, known hub relationships, and representative official citations with `references.json`. Investigate missing expected edges before accepting the batch. Never add an edge merely to satisfy an expected-edge note.

Re-run validation after every relink because sibling target sets and additive glossary terms can change. Review generated Markdown after relinking so newly resolved cross-instrument targets appear correctly and no presentation link enters canonical provision text.

## Milestone 5: deterministic validation and review

Before each code or canonical-data commit, run the checks required by `AGENTS.md`:

    cargo fmt --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace
    cargo run --locked -p lex-cli -- validate lritf
    cargo run --locked -p lex-cli -- validate ifpe-dcg-2021

Also run the affected instrument's end-to-end structural stage, `link`, `validate`, and Markdown export. Inspect its source manifest, validation report, canonical diff, review queue when present, and representative Markdown. If a shared parser or linker changed, re-run the affected historical fixtures and a representative set of already committed corpora that exercise the changed behavior.

Before staging, prove scope explicitly:

    git status --short
    git diff --check
    git diff --stat
    git diff -- adapters/ corpus/ crates/ fixtures/ docs/ batches/ PLANS.md

Stage an exact reviewed path list. Never use a broad add command in the dirty worktree. Verify `git diff --cached --name-only` and `git diff --cached --check` before committing. Routine commit execution may route to the harness's approved mechanical model tier, but the parent session owns the path selection, semantic review, and message. Never push unless the operator separately requests it.

## Milestone 6: completion, state, and recovery

The work is complete when all admitted, non-blocked prepared instruments have passed the structural pipeline; parser-proposed baselines have been reviewed and frozen; all emitted references validate; expected-edge recall has been reviewed; reverse and corpus-wide relinking have completed; deterministic tests pass; representative Markdown is correct; and blocked/legal/temporal items remain explicitly deferred.

At every meaningful stop:

1. update `Progress`, `Surprises and Discoveries`, `Decision Log`, and the single next action in this plan;
2. checkpoint the active run with the completed milestone, exact validation, touched paths, current plan digest, and next slug;
3. keep the source worktree and ignored `.work` artifacts intact when they are needed to diagnose the next step;
4. write a concise historical receipt when handing off or closing a major batch;
5. update `docs/project-status.md` only when current repository truth changes, and add `docs/decisions.md` entries only for accepted semantic or architectural decisions.

The ingestion commands are intended to be repeatable. `adapter scaffold` refuses to overwrite an existing adapter, baseline freezing rewrites only scaffolded `null` placeholders, and validation must pass after freezing. If a run fails after acquisition, keep `.work/<slug>/`, inspect the batch report, and resume from the narrowest failed stage. Do not delete or regenerate unrelated corpus directories. If a parser change damages existing output, stop, retain the failing fixture and evidence, and revert only that reviewed parser change through a new commit rather than discarding unrelated work.

## Validation and acceptance summary

A human reviewer should be able to choose any admitted slug and verify all of the following:

- the adapter points to the intended official source and carries the reviewed instrument identity;
- source and extracted text hashes are present and reproducible;
- canonical provisions preserve official text after only documented deterministic normalization;
- frozen counts and structural ordering agree with the official document;
- reform transitories are separated from original transitories without creating temporal conclusions;
- every emitted reference span matches the canonical source text and every target exists;
- representative expected cross-instrument citations are present, not merely free of broken targets;
- defined terms and usages have valid spans and targets;
- validation reports zero errors and all warnings are explained;
- Markdown renders the canonical content with presentation-only links;
- audited human legal decisions and pending reviews survive unchanged;
- the commit contains only the intended instrument, narrowly necessary parser/linker work, fixtures, evidence, and plan/state updates.

## Outcomes and retrospective

Current outcome: CN1 is structurally and graphically closed at `942f201c`, and
CN2 is structurally and graphically closed at `34449eb6`. All five CN1
instruments plus `lrfiyii-art105`, `lrart6-mdr`, `lrfv-art76`, and
`lrart76-fracvi`, `lrfxiiib-art123`, `lrfxviii-art73`, `lfcpo`, `lfrm`,
`lsct`, and `latime` have reviewed canonical corpora and zero-issue
validation. The ten CN2 ingestions have added ten fixture-backed generic corrections plus reviewed
instrument-scoped source boundaries and aliases while preserving stable
evidence IDs and canonical source text.
Historical enactment-signature cleanup and corpus-wide relinking remain
explicitly separate work; `archive/fable-cross-linking` preserves the divergent
history for bounded reapplication rather than a future merge. The next
prepared batch is AD1; corpus-wide closure remains deferred until the broader
cluster target set is admitted.

At CN1 close, record the final counts and commits for `rgic` and `ldofgg`, the reverse-link results, any parser lessons, and the chosen next operational batch. At cluster close, compare the final admitted corpus with the prepared source universe, enumerate every intentionally blocked or deferred entry, summarize linker recall evidence, and identify the next legal-temporal review program without starting it automatically.

Revision note (2026-07-14): created from live repository inspection at `488057a5`, the existing CN1 and cluster-2 plans, accepted repository decisions, the Rust batch/link/validation implementation, and the provider-neutral execution-planning standard. The plan deliberately records but does not repair the stale active-run state or discovered code defects.
