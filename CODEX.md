# Codex conventions for Lex-Mex

Read `AGENTS.md` for shared repository rules. Inherits the global Codex
working agreements in `~/.codex/AGENTS.md`.

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
