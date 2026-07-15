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

CURRENT SCOPE
The initial vertical slice processes the consolidated Ley para Regular las
Instituciones de Tecnología Financiera (LRITF). The committed corpus currently
has no open structural issues or pending legal reviews. Next substantive work
is regulation ingestion, starting with the CNBV disposiciones de carácter
general, followed by cross-instrument reference resolution — see
`docs/project-status.md` for exact scope and known gaps.

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

Default model for substantive work in this repo: **Claude Sonnet 5**
(`claude-sonnet-5`), effort `medium`. Raise to `high` for parser/
canonicalization changes, schema-boundary changes, or anything touching
review-state transitions.

**Haiku (`claude-haiku-4-5-20251001`) is mandatory for purely mechanical work:**
all commits, and all invocations of `cargo fmt --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`,
and `cargo run --locked -p lex-cli -- validate lritf` with pass/fail
reporting. Writing a new fixture, adding a parser rule, or diagnosing a
validation failure is judgment work and stays on Sonnet 5 — only running the
command and reporting the result routes to Haiku.

Escalate to **Opus** only when Sonnet 5 has failed the same task twice, or the
task is a genuine legal-temporal-modeling design call (new effect category,
schema version bump). Start on Sonnet 5; escalate on evidence, not by default.

Note: this routing rule governs the coding-agent side only — it has no
bearing on the separate `--provider codex` temporal-analysis path, which is a
distinct, schema-gated model call inside the pipeline itself, not a
build-agent task.

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
