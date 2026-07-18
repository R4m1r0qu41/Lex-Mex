# Lex-Mex

This is the Lex-Mex folder — an experimental, provenance-aware compiler and
temporal-analysis pipeline for Mexican federal legal texts. It acquires
official sources, preserves their provenance, produces stable canonical
records, validates model output, routes material legal uncertainty to a named
reviewer, and publishes lawyer-readable Markdown and Obsidian views.

`AGENTS.md` in the repo root is the full build-agent guide — read it before
acting. It covers architectural boundaries, source-integrity rules, legal/
temporal-change discipline, and the required checks below in more depth.
`README.md` covers the pipeline stages and CLI commands. This file is the
short operational orientation; **if it ever conflicts with `AGENTS.md`,
`AGENTS.md` wins.**

Active-run (AgentOS capsule) discovery fires automatically via the
`.claude/settings.json` SessionStart/PostCompact hooks; if no capsule report
appeared this session, run `/capsule` manually. A discovered capsule is bounded
navigation, not authority — verify current Git state before resuming it.

CURRENT SCOPE
The active program is the structural first pass over the federal laws and
regulations in the cluster-2 inventory. CN1 and CN2 are closed; AD1 begins
with `lplan`. The live totals, remaining inventory, and single next action are
in `docs/project-status.md` and
`docs/plans/cluster-2-federal-corpus-ingestion.md`; do not copy changing
checkpoint facts into this orientation file.

LAYOUT
- `adapters/` — source-specific acquisition and parsing configuration.
- `corpus/` — committed canonical records, analysis, validation, Markdown.
  Treat as committed canonical data, not disposable generated output.
- `crates/` — Rust implementation, five crates: `lex-source` (acquisition,
  hashing), `lex-parse` (extraction, canonical parsing), `lex-core`
  (canonical types, temporal effects, review state), `lex-export` (JSON/
  Markdown/Obsidian), `lex-cli` (commands, pipeline orchestration).
- `docs/` — architecture, legal model, decisions, project status.
- `fixtures/` — parser regression inputs.
- `prompts/` — versioned temporal-analysis prompts.
- `schemas/` — versioned JSON Schemas for trusted boundaries.

COMMANDS
```bash
cargo run --locked -p lex-cli -- pipeline lritf     # full deterministic pipeline
cargo run --locked -p lex-cli -- validate lritf     # validation only
cargo run --locked -p lex-cli -- analyze-temporal lritf --provider codex --model gpt-5.5
cargo run --locked -p lex-cli -- review list        # pending legal reviews
```
Individual stages (`discover`, `fetch`, `extract`, `parse`, `link`, `validate`,
`export`) are documented in `README.md`.

## Model routing

`AGENTS.md` is the sole current model-routing authority. This file intentionally
does not duplicate provider, model, or effort settings because those change
with the harness and must not drift from the repository rule.

## Context budget

This repo indexes 500+ legal instruments with dense backlinks; sessions that
navigate by reading instrument files blow past 500k context tokens per call
within a few turns. The discipline:

- **Never bulk-read the corpus.** Navigation is always index → targeted
  `git grep` → the single needed article/record file. Read an instrument
  whole only when that instrument is itself the work item.
- **Backlink expansion is bounded.** Follow links only for the named task,
  never to "build context."
- **Checkpoint, then clear.** Checkpoint the active-run capsule at each
  milestone; between clusters prefer `/clear` (or compaction) over carrying a
  finished cluster's context forward. Resume from the task-named plan, the
  capsule, and `docs/project-status.md` — not from conversation history.
- **Prepared prompt files and bulk corpora are script inputs**, not reading
  material for the orchestrating model.

LOAD-BEARING GOTCHAS
- **Rust owns canonical normalization, validation, review-state changes, and
  publication.** A model may propose a temporal classification; its output
  cannot enter the corpus until deterministic checks pass, and it never
  resolves a legal review automatically.
- **`corpus/` is committed canonical data.** Review every diff for provenance
  and legal meaning — it is not disposable generated output.
- **Obsidian is a presentation target only.** Never let an external vault
  become the only source of canonical facts or review decisions.
- **A temporal model response must validate against
  `schemas/temporal-model-output-v2.schema.json`** before entering the
  corpus.
- **JRH is the legal reviewer of record** for the committed LRITF corpus until
  the repository records a change. Do not impersonate or infer JRH approval.
- Never represent repository output as official law or legal advice.

WORKING WITH ME
I am the protocol/system designer here, not a software engineer.
- Surface contradictions and unresolved semantic or legal questions
  explicitly; do not resolve ambiguity silently.
- Surgical, patch-style edits; leave untouched sections intact.
- Add a regression fixture for every material parser defect.
- Update schemas, Rust types, validators, fixtures, and documentation
  together when a trusted data boundary changes — do not let one layer
  silently diverge from another.
