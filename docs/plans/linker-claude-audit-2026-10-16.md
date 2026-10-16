# Linking code audit and Claude handoff — 2026-10-16

## Objective and baseline

Operator requests a fresh linking-code audit while reviewing held instruments,
with findings handed to Claude. Initial `main` is clean at `667e33a2c`.
Earlier six parser/linker fixes are committed in `078359f72`; do not report
them as unfixed. Scope extraction context/qualifiers and exported link targets.
The held-instrument review and legal decisions are outside this audit.

## Execution and stop conditions

Independent read-only Sol reviews cover extraction and export separately.
Private minimal reproductions establish findings; main source and corpus are
not modified. Mechanical parser/exporter test runs provide current baselines.
Publish a repository report, evidence and a bounded Claude handoff with source
digests and clear actual-versus-synthetic findings. Do not invoke Claude's CLI,
edit provider-owned configuration, stage, commit or push.

Verify current HEAD and code hashes before finalizing. Preserve user edits to
held-instrument review documents. Any source change invalidates the relevant
reproduction and requires refreshing it.

## Current checkpoint

Audit complete at unchanged `667e33a2c`. Tests pass: 206 parser and ten exporter.
Seven additional findings are independently reproduced: three actual stored or
rendered corpus cases, four synthetic boundary cases. The incoming Claude
relative-citation/closure review was checked without a new concrete regression.
Report/evidence and Claude handoff are in `docs/linking-code-*.md/json`.
No source/corpus/held-review changes, staging, commit, push or Claude invocation.

Exactly one next action: Claude independently reproduce the three actual cases
and prepare bounded corrective patches using the handoff's acceptance checks.
