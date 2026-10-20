# SQLite M3 review repairs — 2026-10-20

Incoming Claude review `2026-10-20-lex-mex-m3-review.md` identified one P1:
duplicate keys in stored JSON let serde's last-key interpretation match source
while SQLite's first-key interpretation forged generated columns and FTS labels.
The original artifact is clean; its 226,384 stored JSON values were independently
scanned by Claude. No canonical/legal changes are needed.

## Repairs

All 13 stored extras/payload columns now pass one independent verifier check:
parse, recursively sort object keys lexically, compact-serialize, and require
byte equality with the stored text before reconstructing against pinned source.
This rejects duplicate keys at any depth, alternate key order, whitespace and
alternate encodings. The source-parity check still requires the exact expected
object (including absent versus null). Version 2's canonical JSON rule is explicit
in the execution plan, independent of serde's `preserve_order` feature.

Regression mutations prepend forged provision label/number keys, rebuild FTS,
and rebind both sidecar hashes before verification; a second mutation forges a
qualifier payload. A direct test covers nested/array duplicate keys, sorting,
whitespace, Unicode and null. Positive full artifact verification is required.

Smaller findings addressed: both digest implementations select ordinary tables
through `pragma_table_list`; source-manifest and standard schemas are validated
independently; definition spans must contain the exact term; missing input lookups
return contextual errors; a new golden query pins the whole cited-but-missing
view at 6 rows and multiplicity 17. The English-error-text view remains a known
format dependency protected by golden fixtures. M4 must show no stale input
paths after the permanent builder is committed.

## Validation

Final native gate passes: `cargo fmt --check`,
`cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --locked --workspace --quiet` (399 passed, zero failures), and
both locked instrument validators (LRITF 145/11/126, IFPE 59/4/115, zero issues).
Repaired full-artifact verifier exits 0 in 156.24 seconds: 213,471 normalized
records, 4,960 raw inputs and 9,996,277 FTS token positions match. Database and
logical checksums remain `2b1aec0d…` and `435078f9…`; original bytes unchanged.
All 21 golden queries pass (0.15 seconds). The expected manifest-schema stale
path against M2 HEAD remains explicit; M4 must eliminate it after commit.
See [evidence](sqlite-m3-review-repairs-2026-10-20-evidence.json), including hashes
of the repaired verifier/test source files. The original M3 artifact and its
snapshot identity are preserved. These are verifier-only repairs plus fixtures;
no new full build or M4 reproducibility claim. The updated verifier will verify
the existing pinned artifact; its source/builder remain snapshot 6172db59d.

## State

Worktree `/Users/jr/Dev/lex-mex-db`, branch `db/sqlite-projection`, HEAD42ac990d4.
M3 and these repairs remain uncommitted; no push. Main/operator suggestion files,
canonical corpus and open legal questions are preserved. Next: independent
repair re-review before an authorized M3 commit and M4 permanent-builder builds.
