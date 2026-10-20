# SQLite M1 review response — 2026-10-20

## Outcome and scope

The [independent review](sqlite-m1-review-2026-10-20.md) of local DB commit
`6505baf57c7aa7051cffe62742f46f10276a834a` is addressed in the working tree.
The operator requested debt-check repairs and a numbering decision before M2.
No M2 loaders, full database artifact, canonical change or legal decision was
made. These repairs are uncommitted and unpushed.

## Review dispositions

- **P1 — exact accepted set:** seed schema v2 pins the committed review JSON by
  path and SHA-256. Every accepted review row must appear exactly once, with
  identical metadata, reviewer, decision reference and source locators. Dropped,
  added, substituted and duplicate rows fail. Existing blob/span/ownership and
  separately-parsed-target checks remain. An empty seed is valid only for a
  pinned ledger with no accepted rows. All 17 production rows are unchanged.
- **P2 — decision binding:** the source snapshot is 6505baf57, where the October
  19 review ledger first exists; its corpus baseline is still bfcda816f. The
  whole decision-log hash authenticates that source snapshot. The exact named
  level-two decision section must remain unchanged at the build input commit;
  unrelated log additions are allowed. Changed or duplicate ruling sections
  fail. A changed current review ledger also fails pending an explicit refresh.
- **P2 — numbering/provenance:** canonical array rows use their zero-based file
  index, `input_path`, and JSON pointer `/N`. Scalar objects use ordinal 0 and
  pointer `""`. Wrapped fixture arrays use `/group/N`. Direct projections have
  a foreign key to retained input bytes and uniqueness on path/ordinal and
  path/pointer. Ordered children retain parent-local ordinals and inherit its
  provenance. SQLite provision row IDs are internal FTS keys, separate from
  source order; M2 must assign them deterministically using sorted input paths
  and source ordinals. Two files can both begin at ordinal 0. Read-back verifies
  path, pointer and ordinal independently of reconstructed payloads.
- **P3 — inventory:** `_database-input-policy.json`, validated by its own closed
  schema, declares dated review/candidate/refresh evidence and reference files.
  Required seed/review/decision paths cannot be omitted; candidate hashes are
  checked. Unknown reference files fail closed. The three SCJN registry files
  are evidence only, with no promotion into law provisions or FTS. Standards
  extracted TXT files are explicitly inventoried as excluded body inputs.
  Generated Markdown directories and their duplicate sidecars remain outside
  canonical selection. Dated document paths are no longer hardcoded in Rust.
- **Other findings:** SQL now accepts canonical `unresolved` references without
  asserting a live target. Version-1 release-spec serialization field order is
  explicitly pinned in code. Slug uniqueness belongs to the M2 loader/schema
  completion. M1 fixture read-back still trusts its retained raw input; M3 must
  independently verify against pinned Git inputs and the external manifest.
  This response makes no full-release parity or verification claim.

## Verification

Mechanical final gate: `cargo fmt --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, and
`cargo test --locked --workspace --quiet` all pass: **380 tests**, zero failures
(42 + 3 + 32 + 9 + 5 + 15 + 270 + 4). The 9 lex-db contract tests and 5 pinned
input tests cover provenance collisions/corruption, decision-log evolution,
accepted-set equality, empty-ledger behavior and explicit reference inventory.
Both required locked validators passed with zero issues: LRITF (145 provisions,
11 references, 126 usages) and IFPE (59 provisions, 4 references, 115 usages).
The fixture feature probe passes FTS5/JSON/STRICT on SQLite 3.53.2 and reconstructs
12 records.

Production-seed scratch CLI controls all exit 1: dropping LGS 192 Sextus;
promoting deferred OAAC 12 Bis; promoting originally rejected LFTR 180 Quintes
by falsely marking it accepted; replacing an accepted row while retaining 17
rows; and an empty seed against the 17-accepted-row ledger. Regression tests
also reject duplicates and changed metadata.

The expanded input inspector exits 0 against unreferenced test Git object
`86c045d7c353e3cbf38fc77dd60b4a2a6c282bf7`, parent 6505baf57. No branch moved.
It inventories **4,960 files**: 3,287 normalized, 1,545 evidence-only, 128
standard body exclusions; 468 instruments, 32 standards and 17 accepted debt
rows. Receipt SHA-256:
`c97e1a6cde91e38aaec12bc37f4fbcf4b8e0f20473dfaf1fa51d7d54cdea1414`.
The temporary receipt is `/private/tmp/lex-mex-m1-repair-inventory.json`.
This is an input-policy test snapshot, not a release or an authorized branch
commit. The earlier 4,917-file measurement remains historical.

## Next action

Review and accept this M1 repair diff, then implement the M2 transactional
loaders using the chosen provenance contract. Keep M3 external verification
and M4 full-corpus acceptance separate. Main and its two unrelated untracked
suggestion documents are preserved; no corpus or legal-review record changed.
