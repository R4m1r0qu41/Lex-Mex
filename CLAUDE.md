# CLAUDE.md — Lex-Mex Behavioral Guidelines

**Mission:** Maintain a deterministic, provenance-aware compiler and temporal-analysis pipeline for Mexican federal legal texts.

This repo prioritizes **source integrity, schema validation, and human legal review** over speed. Read `README.md` and `AGENTS.md` before implementing changes.

---

## Core Principles (from Karpathy baseline, adapted for provenance systems)

### 1. Source Integrity First
- Official sources and their metadata (URL, retrieval time, SHA-256) are immutable facts.
- Never silently alter official text. Every normalization must be deterministic, narrow, and covered by a fixture.
- Preserve the distinction between the official source, extracted text, and canonicalized provision.
- Character offsets and source spans must validate against unchanged canonical text.

### 2. Simplicity & Determinism
- No speculative optimizations. Code must be reproducible and auditable.
- Model output is a **proposal**, never a conclusion. It cannot enter the corpus until deterministic checks pass.
- Items routed to legal review require an identified human reviewer and cannot
  be resolved by model confidence alone. Schema-valid, high-confidence
  determinations may remain explicitly labeled `machine_accepted`.
- Keep the system's responsibilities clear: Rust owns integrity, validation, and review state. Models propose.

### 3. Surgical Changes
- When modifying parsing logic, add a regression fixture first. Commit the fixture and the fix together.
- Don't refactor unrelated code during a parse fix.
- Schema, types, validators, and documentation must be updated together.
- If you add a crate or directory, it must contain code or data used now—no empty scaffolding.

### 4. Goal-Driven Verification
- Test changes against the full pipeline: `cargo run --locked -p lex-cli -- validate lritf`
- Inspect the validation report, review queue, exported Markdown, and reference graph.
- For a temporal model change, verify the output validates against `schemas/temporal-model-output-v2.schema.json`.
- Ensure no audit history is overwritten by model reruns.

---

## Project-Specific Rules

### Trust Boundaries
- Official source URL, response metadata, and SHA-256 hashes are the root of trust.
- Extracted text and parsed provisions form a canonical record (committed to `corpus/`).
- Model proposals are separate from canonical facts (stored separately, schema-validated).
- Review state transitions are audited (reviewer identity, timestamp, rationale).

### Temporal Model
- Separate a provision's **temporal status** from the **legal effects** it creates.
- Distinguish **legal ambiguity** from **external verification required** (e.g., checking for a later official act).
- Every legal-review resolution is immutable and audited.
- JRH is the designated legal reviewer for the LRITF corpus unless the repo records a change.

### Obsidian Vault Discipline
- Generated output lives only in `Corpus/<instrument>/`.
- Human-authored notes stay in `Notas/`, `Revisiones/`, or other non-generated directories.
- The generated boundary is strict; never overwrite human work.
- The vault is a **presentation target**, not the source of canonical facts.

### Before Committing
```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run --locked -p lex-cli -- validate lritf
```

For source or pipeline changes, also:
- Run the affected end-to-end stage manually.
- Inspect the source manifest, validation report, canonical diff, review queue, and exported Markdown.

### Repository Structure Discipline
- `adapters/` — source-specific acquisition and parsing config.
- `corpus/` — committed canonical records, analysis, validation, Markdown (immutable facts).
- `crates/` — Rust implementation (source, parse, core, export, cli).
- `docs/` — architecture, legal model, decisions, status.
- `fixtures/` — parser regression test inputs (each parser change adds a fixture).
- `prompts/` — versioned temporal-analysis prompts.
- `schemas/` — JSON Schemas for trusted boundaries.

---

## Definition of Done

A change is complete when:
- It preserves source provenance and integrity.
- All deterministic checks pass (fmt, clippy, test, validate).
- A new parser fixture exists (if parsing changed).
- Schema, types, and validators are updated together (if schema changed).
- No audit history or legal-review decisions are overwritten.
- The change doesn't represent the output as official law or legal advice.
