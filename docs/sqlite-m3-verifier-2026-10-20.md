# SQLite M3 verifier and queries — 2026-10-20

M2 commit `42ac990d4fad5a49c1d68ecfbf3c0c3fb85069fb` was accepted in the incoming
`2026-10-20-lex-mex-m2-review.md` handoff. Claude independently reconstructed
213,471 normalized records and checked 4,960 raw inputs with zero mismatches.
This report covers the subsequent M3 implementation; it does not claim M4.

Subsequent independent review found one P1; see the [repair response](sqlite-m3-review-repairs-2026-10-20.md) for the current status. The measurements below describe the original M3 validation run.

## Review dispositions

- Logical digest: manifest/build specification version 2 explicitly identifies
  `relational_content_v2`. Hash every ordinary relational table except
  `corpus_release`; exclude FTS virtual/shadow/internal tables. Release/environment
  provenance is checked separately against the sidecar, pinned Git and SQL row.
  This keeps SQLite/Rust build metadata out of the corpus-content checksum.
- Independent verification: reconstruct records from pinned Git in separate
  `verify.rs`/`verify_rows.rs`, without loader, probe, parity or handling helpers.
  Shared utilities are Git transport, SHA and manifest types/schema. Verify raw
  bytes, source ordinals/pointers, mapped fields/extras, ordered children, exact
  spans, owners, live targets, audit bindings, supporting evidence and counts.
- FTS verification: independently rebuild the expected index, run integrity
  checks, and compare every term/document/column/offset. Writes are confined to
  memory and a disposable database copy; the original opens read-only and its
  checksum is checked again afterward.
- Snapshot schemas must match compiled schema bytes and inventory; builder and
  schema identity checks run before expensive corpus validation/output creation.
- Raw glossary coverage now validates before deserialization can erase an
  invalid known `hold: null` value; a regression covers this boundary.
- Typed SourceConfig unification remains a low-priority follow-up. Permanent
  builder repinning and two full reproducibility builds remain M4 work.

Historical M2 version-1 artifacts remain unchanged. The version-2 verifier
rejects them explicitly and requires rebuilding rather than reinterpreting
an old checksum.

## Query surface

Read-only APIs provide exact/qualified/not-established lookup, number-and-type
lookup, bounded source context and parameterized literal FTS search with owner
and type filters. Views separate live citations, historical citations with
independent review notes, and accepted cited-but-missing instrument coverage.
An absent exact ID is never treated as proof of legal absence. Golden SQL is
fixed, read-only and bound to parameters.

All 20 golden checks pass on the final artifact: regulation parent authority,
approved outdated and unavailable history, current live targets, accepted gaps,
structural debt, accent-insensitive search, ordinal lookup, definitions/usages,
source provenance, explicit analysis completeness and standards exclusion.

## Validation artifact

Final test snapshot (source and builder):
`6172db59d9ab8b093df48b7ae15d551c28f4df24`, an unreferenced Git object; branch
HEAD remains M2. This is a local validation artifact, not a production release.
Directory: `derived/db/m3-validation-final/6125b449ac0ce12b0c872a64e8964c33d617f96290f2e0a89390eb7d94b27a0a/`.

Build exits 0 in 183.52 seconds. Database: 485,859,328 bytes.
SHA-256: `2b1aec0d1000ab5c88ae3c964bd237a1473fb241ad0c7b65b1ed6295765aa777`.
Logical SHA-256: `435078f9e530c4a8fca9954ad56ce1b7915db35b8ebda60fe476a735a8da9f2c`.
Counts: 468 instruments, 68,328 provisions, 35,949 references, 5,727 terms,
102,432 usages; 4,960 raw inputs. Detailed counts and verification receipt
are in the companion evidence JSON.

Independent full verification exits 0 in 163.90 seconds: all 213,471 normalized
records, 4,960 raw inputs and 9,996,277 FTS token positions match. The original
database SHA remains unchanged. Staleness reporting identifies exactly
`schemas/database-manifest.schema.json` against M2 HEAD: the version-2 schema
exists in the test snapshot and uncommitted worktree, not yet on the branch.
This expected difference prevents claiming the artifact represents current HEAD.

## Native and adversarial checks

`cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
and `cargo test --locked --workspace --quiet` pass: 398 tests, zero failures.
Required locked validators passed: LRITF 145 articles/11 transitories/126
references; IFPE 59/4/115; both zero issues. No canonical-data changes followed.

Mutation tests rebind artifact and logical checksums before verification, so
checks must detect source mismatch rather than merely checksum changes:
optional-key deletion, pointer/ordinal/span/review corruption, absent live
targets, changed raw input bytes, erased FTS, a fake FTS-prefixed ordinary table,
and release metadata tampering. Positive checks include unchanged original
bytes, new-input staleness, qualified debt/standard exclusions, literal search
and bounds. Full-corpus double-build/cross-environment reproducibility has not
been run and is reserved for M4.

## State and next action

All changes are uncommitted; no push. No corpus, parser, accepted debt or legal
review decisions changed. SCJN remains evidence-only. LAN/LGCS/LGS/LIEPS legal
questions remain with the operator. Main and its two untracked suggestion
files are preserved. The earlier development M3 artifact is not acceptance
evidence for the final digest policy.

Next: independent M3 review, then an authorized commit and M4 builds pinned to
the permanent builder commit. Integrate through a normal merge preserving
`6505baf57` ancestry.
