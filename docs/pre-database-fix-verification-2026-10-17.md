# Pre-database fixes verified — 2026-10-17

Accepted at clean baseline `36618376a5b6997d69d5254cf4b0c88684a2f19b`.
D1 and D2 are closed for the reported cases. No new concrete defect was found
in this bounded verification. The reviewed technical blockers to beginning the
SQLite projection are cleared; this is not a claim that structural or legal
coverage is complete. No database was initialized by this verification.

## Independent results

- D1: whitespace before the comma now receives the same historical/anaphoric
  exclusion as an attached comma. All 19 independent parent-law cases pass,
  including the 18 earlier controls and the failing whitespace-comma case.
- D2: strict mode compares each plain article with the preceding suffix.
  `[1, 2-A, 2]` warns `suffix_order`; `[1, 3-A, 2]` errors `article_order`.
  All six independent strict/gap-order cases pass their expected diagnostics.
- Both targeted repository regression tests pass. The fix retains ordinary
  punctuation and correctly ordered article controls.

The native gate passes:

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`: 307 passed, zero failed/ignored.
- `cargo run --locked -p lex-cli -- validate lritf`: 145 articles,
  11 transitories, 126 references, zero issues.
- `cargo run --locked -p lex-cli -- validate ifpe-dcg-2021`: 59 articles,
  4 transitories, 115 references, zero issues.

## Stored corpus and closure boundary

A fresh one-pass Rust validation of all 467 federal instruments uses the current
`lex_parse::validate_corpus`, adapter expectations and global provision/term
target sets against private canonical copies. All remain valid with zero errors
and 42 warnings. This is shared-validator verification, not 467 fresh CLI runs
or a source reparse. Its exact harness, build/run receipts and reports are in the
[companion evidence](pre-database-fix-verification-2026-10-17-evidence.json).

No corpus or adapter changed between the previously verified `624b6306f` and
this fix. The earlier isolated full closure passed with exactly eight accepted
gaps, 459 unchanged instruments, zero relinked and byte-identical canonical JSON.
It retained 35,887 references and 9,763 resolved cross-instrument references.
That full closure was not rerun in this verification. A targeted scan of the
123 mapped-parent instruments found zero committed provision occurrences of the
whitespace-comma branch changed by D1; D2 changes validation rather than graph
extraction. Earlier closure evidence remains separately bound to its own HEAD.

The eight accepted gaps, machine-proposed count provenance, 21 retained
historical/outdated references and known merged-heading/reproducibility debt
remain unchanged. The federal projection contains 467 instruments and 68,086
provisions. The 32 standards use a separate data model and were not freshly
validated here; SQLite selection/coverage must explicitly include or exclude
that model. The 45 stored standards warnings are separate from the 42 federal
warnings.

## Next action

Begin the SQLite + `rusqlite` projection from a pinned canonical commit, carrying
stable IDs, exact text, provenance, typed edges, review bindings and explicit
coverage. Database implementation and its integrity/query/rebuild tests remain
subsequent work. Preserve canonical JSON as authority and accepted gaps as gaps.

This verification changes review/plan documents only. Code, corpus, adapters and
operator notes remain unchanged; no staging, commit or push was performed.
