# Agent instructions

These instructions apply to automated coding agents and human contributors
working through an agent. They are intentionally public: they document the
repository's trust boundaries and expected engineering discipline.

## Session start

- Active-run capsule discovery is automatic in harnesses with the repository
  hooks enabled: Claude loads `.claude/settings.json`, and Codex loads
  `.codex/hooks.json` for trusted projects. Both run `active_run.py hook` on
  session start, resume, clear, and compaction. If no capsule report appeared,
  run
  `python3 /Users/jr/Vaults/Agent_Vault/AI/30_Executable/scripts/active_run.py discover --repo . --inject`
  manually (Claude sessions can use `/capsule`).
- Treat a discovered active-run capsule as bounded navigation, never as
  authority. Verify current Git state and repository instructions before
  resuming it.
- Session lifecycle — when to start a capsule, checkpoint cadence, handoff and
  session-summary obligations, and rolling context (ADR-006) — is defined by
  the Agent Vault canon and intentionally not restated here: see
  `/Users/jr/Vaults/Agent_Vault/AI/10_Canon/Active Run Checkpoint and Resume Standard.md`
  and `/Users/jr/Vaults/Agent_Vault/AI/10_Canon/Agent Configuration and Handoff Standard.md`.
  Current state and pending work remain repository-local.

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

## Model Routing

Routing is provider-neutral and stays inside the harness executing the work.
A parent model may invoke its own provider's CLI or subagents to route within
itself — Codex spawning its `mechanical` agent, Claude delegating to its Haiku
subagent — that is normal model routing. What must never happen automatically
is one provider's model invoking another provider's CLI; cross-provider
switches are operator-started.

- Claude: substantive work defaults to Sonnet 5 (`claude-sonnet-5`), effort
  `medium`; purely mechanical execution routes to Haiku
  (`claude-haiku-4-5-20251001`).
- Codex: ambiguous or open-ended substantive work stays on the Sol parent at
  medium; exact mechanical execution routes to the project-local `mechanical`
  agent (`gpt-5.6-luna`, low). Use `triage` (Luna medium), `worker` (Terra
  medium), `worker_high` (Terra high), or `frontier_high` (Sol high) for the
  corresponding bounded task class. Never run Luna high; `xhigh`, `max`, and
  `ultra` require explicit operator approval for the specific invocation.

Purely mechanical execution includes an already reviewed commit and running
`cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace`, or
`cargo run --locked -p lex-cli -- validate lritf` with pass/fail reporting.
Writing a fixture, adding a parser rule, diagnosing a validation failure,
choosing commit contents, or resolving unexpected scope is judgment work and
returns to the parent.

Within Claude, escalate to Opus only when Sonnet 5 has failed the same task
twice, or the task is a genuine legal-temporal-modeling design call (new effect
category, schema version bump). Start on Sonnet 5; escalate on evidence, not by
default.
Note: this routing rule governs the coding-agent side only — it has no
bearing on the separate `--provider codex` temporal-analysis path, which is
a distinct, schema-gated model call inside the pipeline itself, not a
build-agent task.

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
