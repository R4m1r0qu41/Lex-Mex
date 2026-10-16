# Lex-Mex execution plans

This file is the stable contract and index for long-running execution plans in
Lex-Mex. It does not track task progress itself and does not authorize work
beyond the operator's request and `AGENTS.md`.

## When a plan is required

Use a task-named execution plan when work is expected to cross sessions or
contributors, contains multiple independently verifiable milestones, changes a
trusted data boundary, or needs an explicit recovery and handoff sequence.
Small fixes and routine validation do not need a plan.

## Location and naming

- Store living plans under `docs/plans/`.
- Name each plan for one concrete objective, using lowercase kebab-case.
- Do not create a generic mutable root `PLAN.md`.
- Keep source inventories and operator prompts under `prompts/`; those inputs
  do not replace an execution plan.

## Required plan shape

Each plan must remain sufficient for a fresh agent to resume from repository
state without the original conversation. Include:

- purpose, observable outcome, scope, and exclusions;
- authoritative repository surfaces and external source boundaries;
- an immutable initial baseline and a separately labeled current checkpoint;
- timestamped progress with exactly one current next action;
- discoveries and decisions with concise evidence and rationale;
- milestones with exact verification, recovery, and stop conditions;
- outcomes and retrospective notes at major checkpoints and completion.

Repository code, tests, accepted decisions, current Git state, and `AGENTS.md`
outrank a plan. Verify them whenever work resumes. A plan must not contain
secrets, transcripts, hidden reasoning, or mutable copies of external state.

## Lifecycle

1. Create or select the task-named plan before substantive multi-milestone
   work.
2. Update its current checkpoint, progress, discoveries, and next action at
   meaningful milestones.
3. Bind any external active-run capsule to the exact plan path and current
   digest; the capsule remains navigation, not authority.
4. Preserve the plan after completion with its outcome recorded. Move it to an
   archive location only through a reviewed repository change.

## Plan index

### Active

- [`docs/plans/corpus-wide-reverse-linking.md`](docs/plans/corpus-wide-reverse-linking.md)
  — Relink the committed federal corpus, then validate and republish Markdown.
  Landed locally, unpushed (304 graphs rewritten, 21 held). Held instruments and
  Codex review remain; the SQLite database is decided but not built.
- [`docs/plans/nested-public-reader.md`](docs/plans/nested-public-reader.md)
  — Publish discoverable text and a source-backed nested reader with shared MCP access.

- [`docs/plans/maximasa-legal-integration.md`](docs/plans/maximasa-legal-integration.md)
  — Use the selected federal corpus in Maximasa's compliance module, close
  missing-instrument coverage, and design a standards-capable NOM/NMX boundary.
- [`docs/plans/external-cli-metadata-ingestion-automation.md`](docs/plans/external-cli-metadata-ingestion-automation.md)
  — Expose selected committed corpus data to external projects, define the
  metadata portability boundary, and make structural ingestion resumable.
- [`docs/plans/cluster-2-federal-corpus-ingestion.md`](docs/plans/cluster-2-federal-corpus-ingestion.md)
  — Complete the prepared cluster-2 federal corpus through the Rust ingestion
  gate, close reverse links, and validate publication output.

### Completed

- [`docs/plans/parser-linker-audit-2026-10-16.md`](docs/plans/parser-linker-audit-2026-10-16.md)
  — Six defects repaired; 278 tests and 44 replay stages pass. LGDNNA source
  citation remains a separate JRH review before corpus-wide relinking.

- [`docs/plans/federal-held-source-reprocessing.md`](docs/plans/federal-held-source-reprocessing.md)
  — Ten cleared holds admitted locally (467 instruments); Sol review and native
  checks passed; three DCGs deferred and EGDF excluded. Awaiting commit authority.

- [`docs/plans/federal-bulk-structural-admission.md`](docs/plans/federal-bulk-structural-admission.md)
  — Admitted/exported 225 candidates; 457 instruments locally, 14 holds
  reported separately; GPT-6.1-Sol independent review accepted.

- [`docs/plans/federal-source-inspection-batch-01.md`](docs/plans/federal-source-inspection-batch-01.md)
  — LAEM and LAMNI inspected and admitted locally with frozen counts; 232
  instruments locally, temporal review and broader reverse linking deferred.

- `docs/plans/federal-ingestion-failure-repair.md` (archived 2026-10-16 to the git-ignored `archive/`; recover from git history)
  — Parser and follow-up source-ownership/publication repairs verified on all
  227 frozen cases; 14 holds preserved, no canonical admission.

- [`docs/plans/reader-figma-intake-dots.md`](docs/plans/reader-figma-intake-dots.md)
  — Reader Figma/Dots draft delivered and fourteen EC2/EC3 intakes committed locally.

- [`docs/plans/temporal-boundary-repair-and-public-reader.md`](docs/plans/temporal-boundary-repair-and-public-reader.md)
  — Repair temporal trust boundaries and assess public-reader and agent access.

- [`docs/plans/standard-post-transitory-supplements.md`](docs/plans/standard-post-transitory-supplements.md)
  — Modeled exact-span standard supplements, migrated committed tails, and
  ingested the three standards held on transitory/supplement partitioning.
