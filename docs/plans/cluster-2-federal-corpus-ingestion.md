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

Verified against local `main` at `727aa5d1` (two commits ahead of remote
`main` at `8a3a0f9b`):

- `rgic` is committed and validates with 214 articles, 2 original
  transitories, 30 references, 23 reform-transitory evidence records, and zero
  issues (`2e061724`);
- `ldofgg` is committed and validates with 20 articles, 2 original
  transitories, 1 reference, 7 reform-transitory evidence records, and zero
  issues (`727aa5d1`);
- all five operational CN1 entries now have committed adapters and canonical
  corpora;
- the five-instrument reverse-link, validation, and Markdown pass completed
  without canonical diffs, and every instrument remains valid with zero
  issues, but the graph still contains zero cross-instrument edges despite
  explicit sibling-title citations;
- exactly 55 prepared prompt files are staged for operator review: 53 manifests
  under `prompts/cluster-2-batches/` and the two federal cluster plans;
- `.gitignore`, `README.md`, and `docs/project-status.md` remain modified and
  unstaged and must not be folded into the CN1 closure without operator review;
- the Agent Vault active-run capsule was reconciled at checkpoint sequence 4,
  bound to this plan, and advanced to LDOFGG before checkpoint 5 execution;
- `fable/cross-linking` contains an existing global-alias and relinking
  implementation beginning at `02088004`, based on pre-CN1 commit `92774db4`;
  inspect and reconcile that divergent work before implementing another alias
  solution on `main`.

Do not assume these statements remain current. At every resumption, compare them with `git log`, `git status`, the operational manifest, adapter presence, corpus presence, validation files, and the active-run drift report.

## Next action

Review and reconcile the existing `fable/cross-linking` implementation against
current `main`, then integrate or deliberately replace its global-alias path
before claiming CN1 cross-instrument closure. Preserve the zero-diff relink
baseline and add fixtures for the known LOCGEUM/RGIC citations.

## Progress

- [x] (2026-07-14 20:33Z) Ingested and committed `locg` at `97fa5cbc`; validation recorded zero issues.
- [x] (2026-07-14 21:02Z) Ingested and committed `reg-diputados` at `553baa6e`; validation recorded zero issues.
- [x] (2026-07-14 22:08Z) Ingested and committed `reg-senado` plus narrowly required parser fixtures and hardening at `488057a5`; validation recorded zero issues.
- [x] (2026-07-15 02:22Z) Reconciled the active-run capsule through `fabbb2c2`, bound it to this task-named plan, and advanced it to checkpoint 5 without changing the repository worktree.
- [x] (2026-07-14 22:44Z) Ingested, inspected, froze, relinked, validated, and committed `rgic` with the required parser regression at `2e061724`.
- [x] (2026-07-15 02:28Z) Ingested, inspected, froze, relinked, validated, and committed `ldofgg` at `727aa5d1`: 20 articles, 2 original transitories, 1 reference, 7 reform-transitory evidence records, and zero issues. Added a focused stop-marker fixture so enactment signatures do not contaminate its final transitory.
- [x] (2026-07-15 02:31Z) Relinked, validated, and regenerated Markdown for all five CN1 instruments. Counts remained 31/40/47/30/1 references, every validation reported zero issues, and Git recorded no canonical diff.
- [ ] Resolve the demonstrated cross-instrument recall gap, rerun the clean reverse-link baseline, and close CN1 with an updated plan checkpoint.
- [ ] Normalize and admit each remaining prepared cluster-2 batch, then ingest its instruments in dependency order.
- [ ] Complete a corpus-wide relink, expected-edge audit, deterministic validation, and publication review.

## Surprises and discoveries

- Observation: the active-run capsule is behind live repository state.
  Evidence: checkpoint 3 names `reg-senado` as next, while documentation HEAD `8a3a0f9b` records valid committed corpora through `rgic`.
- Observation: a successful batch run does not close reverse cross-instrument links.
  Evidence: each instrument extracts references against siblings present during its parse, but `run_batch` does not relink instruments processed earlier after a later target is added. A separate reverse relink pass is required unless batch orchestration is enhanced.
- Observation: `adapters/diputados/_instrument-aliases.json` is not consumed by the current Rust linking path.
  Evidence: repository search finds the alias file mentioned as folded configuration, but `external_instruments` in each `SourceConfig` is the only configured external-name input passed to reference extraction. All five CN1 adapters currently have empty `external_instruments` arrays. The closure pass emitted zero cross-instrument edges even though CN1 provisions contain at least 10 exact mentions of LOCGEUM's title and 4 exact mentions of RGIC's title. The divergent `fable/cross-linking` branch already implements a global alias input; cross-instrument completeness on `main` cannot be claimed until that work is reviewed and integrated or deliberately replaced.
- Observation: enactment signatures contaminate the final original transitory in many older committed Diputados corpora.
  Evidence: the provisional LDOFGG parse appended the 1986 congressional signatures and presidential promulgation block to Transitory Segundo; a repository-wide exact search found the same `Rúbrica` pattern in numerous existing final transitories, including `rgic` and `reg-senado`. LDOFGG is corrected narrowly with its existing adapter stop-marker boundary, but historical cleanup requires a separately reviewed canonical migration.

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

Current outcome: all five CN1 instruments are structurally ingested through
`ldofgg` at `727aa5d1`, and their reverse-link/validation/export pass is clean
and deterministic. CN1 remains open because the graph emitted zero
cross-instrument edges despite known sibling-title citations. The missing
batch reverse-link phase, disconnected alias registry on `main`, and
historical enactment-signature contamination remain flagged follow-ups; the
existing `fable/cross-linking` implementation is the next review target.

At CN1 close, record the final counts and commits for `rgic` and `ldofgg`, the reverse-link results, any parser lessons, and the chosen next operational batch. At cluster close, compare the final admitted corpus with the prepared source universe, enumerate every intentionally blocked or deferred entry, summarize linker recall evidence, and identify the next legal-temporal review program without starting it automatically.

Revision note (2026-07-14): created from live repository inspection at `488057a5`, the existing CN1 and cluster-2 plans, accepted repository decisions, the Rust batch/link/validation implementation, and the provider-neutral execution-planning standard. The plan deliberately records but does not repair the stale active-run state or discovered code defects.
