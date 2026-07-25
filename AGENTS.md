# Agent instructions

These instructions apply to automated coding agents and human contributors
working through an agent. They are intentionally public: they document the
repository's trust boundaries and expected engineering discipline.

## Codex instruction inheritance

Inherits: `~/.codex/AGENTS.md`. This file adds Lex-Mex legal-corpus,
provenance, implementation, validation, and escalation rules.

## Agent Vault continuity

- Vault project ID: `lex-mex`. Repository hooks may report an active capsule;
  otherwise the global discovery rule applies. For multi-milestone work with
  no capsule:
  `python3 /Users/jr/Vaults/Agent_Vault/AI/30_Executable/scripts/active_run.py start --repo . --project-id lex-mex --objective "<named task>"`.
- Detailed capsule, handoff, receipt, and rolling-context procedure remains in
  `/Users/jr/Vaults/Agent_Vault/AI/10_Canon/Active Run Checkpoint and Resume Standard.md`
  and
  `/Users/jr/Vaults/Agent_Vault/AI/10_Canon/Agent Configuration and Handoff Standard.md`.
- Current repository state and pending work remain repository-local.

## Architectural boundaries

- Keep Rust responsible for canonical normalization, validation, review state
  changes, and publication.
- Treat `corpus/` as committed canonical data, not disposable generated output.
  Review every corpus diff for provenance and legal meaning.
- Treat Obsidian as a presentation target. Never make an external vault the
  only source of canonical facts or review decisions.
- Keep model proposals distinct from deterministic facts in types, schemas,
  stored artifacts, and exports.
- Keep canonical source text free of presentation links. Store express
  references as validated graph edges and inject links only during export.
- A temporal model response must validate against
  `schemas/temporal-model-output-v2.schema.json` before entering the corpus.
- Never allow a model run, import, export, or rerun to resolve or overwrite an
  audited human legal decision.

## Source integrity

- Never silently alter official source text. Every normalization must be
  deterministic, narrow, and covered by a fixture.
- Reference character offsets and exact source spans must validate against the
  unchanged canonical provision text (or the instrument's official title for
  title-anchored edges), and every resolved target — internal or
  cross-instrument — must exist in its instrument's committed corpus.
- Preserve the official source URL, publisher metadata, retrieval time, source
  SHA-256, extracted-text SHA-256, parser version, and schema version.
- Attach formal DOF sources when a decision depends on promulgation, amendment,
  commencement, or a later official act.
- Do not treat a consolidated Cámara text as a substitute for its cited formal
  publication when the distinction matters.

## Legal and temporal changes

- Separate a provision's temporal status from the legal effects it creates.
- Distinguish legal ambiguity from factual verification of a later official
  event.
- Preserve reviewer identity, timestamp, rationale, source links, and prior
  machine proposal for every legal-review resolution.
- Until the repository records a change, JRH is the legal reviewer for the
  committed LRITF corpus. Do not impersonate or infer JRH approval.
- Do not represent repository output as official law or legal advice.

## Implementation discipline

- Add a regression fixture for every material parser defect.
- Update schemas, Rust types, validators, fixtures, and documentation together
  when a trusted data boundary changes.
- Do not add a new crate or top-level directory without code or data that uses
  it now.
- Keep generated Obsidian output inside `Corpus/<instrument>/`; never overwrite
  human-authored vault directories.
- Keep credentials, tokens, personal vaults, downloaded work files, and local
  environment configuration out of Git.
- Preserve unrelated local changes and avoid destructive Git operations.

## Execution planning

- Read `PLANS.md` before work that spans multiple milestones, sessions, or
  contributors, or that requires an explicit recovery and handoff sequence.
- Keep living execution state in one task-named file under `docs/plans/`; do
  not create a generic mutable root `PLAN.md`.
- Keep prepared source inventories and prompts distinct from execution plans,
  and bind any external active-run capsule to the applicable task plan by path
  and digest.

## Context budget

- Never bulk-read the corpus. Navigate from an index through targeted
  `git grep` results to the single needed article or record; read an entire
  instrument only when that instrument is the work item.
- Keep backlink expansion bounded to the named task. Do not follow links merely
  to build ambient context.
- Checkpoint the active-run capsule at milestones. Between completed clusters,
  start a fresh bounded session in the same harness and resume from the
  task-named plan, capsule, and repository state rather than carrying
  finished-cluster context.
- Treat prepared prompt files and bulk corpora as script inputs, not reading
  material for the orchestrating model.

## Repository escalation and mechanical triggers

Global Codex routing applies. Parser/canonicalization changes, schema
boundaries, review-state transitions, legal-temporal modeling, new effect
categories, and schema-version changes remain parent judgment and may use
`frontier_high` when the global escalation criteria are met.

Running `cargo fmt --check`,
`cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace`, or
`cargo run --locked -p lex-cli -- validate lritf` with pass/fail reporting is
mechanical only after the parent fixes the scope. Writing a fixture, adding a
parser rule, diagnosing a failure, choosing commit contents, or resolving
unexpected scope is judgment work. These coding-agent rules do not govern the
separate schema-gated `--provider codex` temporal-analysis pipeline.

## Required checks

Run these before committing changes that affect code or canonical data:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run --locked -p lex-cli -- validate lritf
cargo run --locked -p lex-cli -- validate ifpe-dcg-2021
```

For a source or pipeline change, also run the affected end-to-end stage and
inspect the source manifest, validation report, canonical diff, review queue,
and exported Markdown.
