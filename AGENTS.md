# Agent instructions

These instructions apply to automated coding agents and human contributors
working through an agent. They are intentionally public: they document the
repository's trust boundaries and expected engineering discipline.

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

## Model Routing

Default model for substantive work: Claude Sonnet 5 (`claude-sonnet-5`),
effort `medium`. Raise to `high` for parser/canonicalization changes,
schema-boundary changes, or anything touching review-state transitions.

Haiku (`claude-haiku-4-5-20251001`) is mandatory for purely mechanical work:
all commits, and all invocations of `cargo fmt --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`,
and `cargo run --locked -p lex-cli -- validate lritf` with pass/fail
reporting. Writing a new fixture, adding a parser rule, or diagnosing a
validation failure is judgment work and stays on Sonnet 5 — only running the
command and reporting the result routes to Haiku.

Escalate to Opus only when Sonnet 5 has failed the same task twice, or the
task is a genuine legal-temporal-modeling design call (new effect category,
schema version bump). Start on Sonnet 5; escalate on evidence, not by default.
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
