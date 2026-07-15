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

Verified against local `main` at `03ff3fbb` (two commits ahead of remote
`main` at `1ef26c28`):

- `rgic` is committed and validates with 214 articles, 2 original
  transitories, 30 references, 23 reform-transitory evidence records, and zero
  issues (`2e061724`);
- `ldofgg` is committed and validates with 20 articles, 2 original
  transitories, 1 reference, 7 reform-transitory evidence records, and zero
  issues (`727aa5d1`);
- all five operational CN1 entries now have committed adapters and canonical
  corpora;
- the reviewed global-alias linker and bounded marker search are committed at
  `942f201c`; the implementation consumes the existing underscore-prefixed
  registry, excludes bare acronyms and absent targets, rejects alias
  collisions, and leaves canonical provision text unchanged;
- the five-instrument reverse-link, validation, and Markdown pass added 22
  resolved cross-instrument edges without removing or retargeting an existing
  edge: 19 target CPEUM and 3 target LOCGEUM;
- final CN1 reference counts are 41/47/51/31/1 for
  `locg`/`reg-diputados`/`reg-senado`/`rgic`/`ldofgg`; every instrument
  validates with zero issues and every added span and target was inspected;
- prepared CN2 is normalized as
  `batches/constitutional_cn2_implementing_laws.json`: all 10 entries retain
  their verified official Cámara reference and PDF sources, none is blocked,
  and no expected edge was invented where the prepared manifest supplied none;
- the operational-manifest inventory is now 28 manifests and 150 instruments;
- `lrfiyii-art105`, the first CN2 instrument, is committed at `b7653e22` with
  74 articles, 4 original transitories, 46 references, 45 reform-transitory
  evidence records, and zero validation issues; its source SHA-256 is
  `b99f96ee0d44bd781d14cfdc7f94358bd6d3f6ed17c3bd3aadcf5d149873edeb`
  and extracted-text SHA-256 is
  `34e6b8f137e02054a2ca1567de109faa54801c7a28b40f02ca046ecd928eca43`;
- the parser hardening required by that first instrument is isolated at
  `c1952e50` with fixtures for title-case structural headings, paragraph
  boundaries after page-wrapped amendment marks, and repeated transitory
  sections within one reform decree;
- `lrart6-mdr`, the second CN2 instrument, is committed at `7a62c205` with
  42 articles, 3 original transitories, 15 references, 3 defined terms, 6
  reform-transitory evidence records, and zero validation issues; its source
  SHA-256 is
  `b3013ac434856f46bc973c4178d55c6d5d440730a42f3d7f8ca225730e0d382c`
  and extracted-text SHA-256 is
  `e7c915812ed8a49a288ace0ad3c8bebb5ed4d0901eeac84f46d6def7be3c5f4f`;
- its shared parser/linker correction is isolated at `42b8bb61`: exact
  heading grammar no longer mistakes a wrapped sentence beginning with
  `Capítulo` for structure, and an ordinal dot such as `6o.` no longer hides
  a following external-instrument marker;
- `lrfv-art76`, the third CN2 instrument, is committed at `40d41362` with 15
  articles, 1 original transitory, 10 references, no defined terms or reform
  evidence, and zero validation issues; its source SHA-256 is
  `89ae4eeddd5a5d6f2100bc85e9348f6f1fdc5f1acb92ef074c6a742a5cd2aad3`
  and extracted-text SHA-256 is
  `fbef56a05118e87a378da61cdedc37c729b1a3edaa75e849ff128cf6c5d5fb09`;
- its shared linker correction is isolated at `92004dc2`: the historical
  title `Constitución General de la República` resolves to CPEUM, `artículo
  que precede` resolves to the previous article, and numeric ordinal paragraph
  qualifiers no longer truncate the following external-instrument context;
- `lrart76-fracvi`, the fourth CN2 instrument, is committed at `99d588de` with
  25 articles, 1 original transitory, 9 references, no defined terms, 4
  reform-transitory evidence records, and zero validation issues; its source
  SHA-256 is
  `bb06062d0e0454e928090853dd4071bfd19b713abe553033285147a67c323531`
  and extracted-text SHA-256 is
  `7879ed40021c49ac411e0d50cf89620090373d5425945b4f7fa9fd4e377fcc15`;
- its shared parser correction is isolated at `eee19102`: constitutional
  adjectives between an article-list heading and its first number are
  accepted, and noun-first paragraph qualifiers retain their exact source
  spans and target anchors;
- `lrfxiiib-art123`, the fifth CN2 instrument, is committed at `03ff3fbb` with
  25 articles, 4 original transitories, 4 references, no defined terms, 19
  reform-transitory evidence records, and zero validation issues; its source
  SHA-256 is
  `57ec1adca9a93f6403430d9a0e202d49216fa2c83997706c08d93b88cdd46dea`
  and extracted-text SHA-256 is
  `9bac25113137ed36e0a842eafd262e3de89f8c0f34825e2447060a917d85a48c`;
- its shared metadata correction is isolated at `8be59440`: when a Cámara
  consolidation contains more than one `Última Reforma` label, the canonical
  latest-reform date is the maximum valid date rather than the first match;
- exactly 55 prepared prompt files are staged for operator review: 53 manifests
  under `prompts/cluster-2-batches/` and the two federal cluster plans;
- `.gitignore`, `README.md`, and `docs/project-status.md` remain modified and
  unstaged, and `.claude/` remains untracked; these user-owned paths must not
  be folded into a corpus checkpoint without operator review;
- the divergent `fable/cross-linking` work was reviewed selectively: the
  global alias path from `02088004` and proximity fix from `a0f4d62d` were
  reimplemented on current `main`; its bundled `regulates` schema work, bulk
  relinks, and unrelated parser/export changes were not merged.

Do not assume these statements remain current. At every resumption, compare them with `git log`, `git status`, the operational manifest, adapter presence, corpus presence, validation files, and the active-run drift report.

## Next action

Provisionally process `lrfxviii-art73`, the sixth entry in the reviewed CN2
operational manifest. Inspect its official PDF, adapter boundaries, canonical
structure, reform evidence, references, validation report, and representative
Markdown before freezing any proposed counts.

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
- [ ] Normalize and admit each remaining prepared cluster-2 batch, then ingest its instruments in dependency order.
- [ ] Complete a corpus-wide relink, expected-edge audit, deterministic validation, and publication review.

## Surprises and discoveries

- Observation: the active-run capsule is behind live repository state.
  Evidence: checkpoint 3 names `reg-senado` as next, while documentation HEAD `8a3a0f9b` records valid committed corpora through `rgic`.
- Observation: a successful batch run does not close reverse cross-instrument links.
  Evidence: each instrument extracts references against siblings present during its parse, but `run_batch` does not relink instruments processed earlier after a later target is added. A separate reverse relink pass is required unless batch orchestration is enhanced.
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
CN2 has five of 10 instruments committed through `03ff3fbb`. All five CN1
instruments plus `lrfiyii-art105`, `lrart6-mdr`, `lrfv-art76`, and
`lrart76-fracvi`, and `lrfxiiib-art123` have reviewed canonical corpora and
zero-issue validation. The five CN2 ingestions have added nine fixture-backed
generic boundary and metadata corrections while preserving stable evidence
IDs and canonical source text. Historical enactment-signature cleanup and
corpus-wide relinking remain explicitly separate work;
provisional ingestion of `lrfxviii-art73` is next.

At CN1 close, record the final counts and commits for `rgic` and `ldofgg`, the reverse-link results, any parser lessons, and the chosen next operational batch. At cluster close, compare the final admitted corpus with the prepared source universe, enumerate every intentionally blocked or deferred entry, summarize linker recall evidence, and identify the next legal-temporal review program without starting it automatically.

Revision note (2026-07-14): created from live repository inspection at `488057a5`, the existing CN1 and cluster-2 plans, accepted repository decisions, the Rust batch/link/validation implementation, and the provider-neutral execution-planning standard. The plan deliberately records but does not repair the stale active-run state or discovered code defects.
