# SQLite M4 reproducibility — 2026-10-20

M3 was committed with operator authorization at
`fedcbc5dbe1ae7d87a6dce0e0e521f56b28b9a58`, parent `42ac990d4`.
The exact 23-path milestone contains the independent verifier, query acceptance,
Claude-review duplicate-key repairs, regression fixtures and reports. No corpus
or derived artifact was committed; no push. M3's historical validation artifacts
retain their original unreferenced snapshot identities.

## Independent acceptance

Claude's `2026-10-20-lex-mex-m4-review.md` accepts both M3 repairs and M4.
In a fresh scratch clone, a separately compiled release binary reproduced the
same database and manifest bytes (third build C). Its independent verification
has empty stale inputs and the same full counts; all 21 goldens pass. Claude
also re-ran the rebound-checksum duplicate-key forgery and confirmed rejection.
These are reviewer-reported checks; the A/B measurements below are Codex's.
All three builds were on the same host/toolchain; no cross-environment claim.
Low-priority follow-ups: include row/table context in canonical JSON errors;
consider reference-query planner tuning in a new release rather than modifying
an immutable artifact. DB-first integration remains with the designated
integration operator; the parser batch must be merged afterward and rebuilt.

## M4 scope and method

Build the full corpus twice from that same permanent corpus/builder commit in
fresh `derived/db/m4-validation-a/` and `derived/db/m4-validation-b/` roots.
Both jobs use the same compiled executable and pinned environment and run
concurrently. Compare manifest, database SHA, logical SHA, complete counts and
all golden query results. Independently reconstruct every normalized record
and raw input for each artifact, compare full FTS positions, and require empty
`stale_input_paths`. No source/corpus/legal changes are part of M4.

Measure eight representative read-only SQL queries on both artifacts after
build/verification jobs finish: exact ID, number/type lookup, outgoing and
reverse live references, term usages, ranked literal FTS, pipeline coverage
and cited-but-missing coverage. Capture plans, deterministic result hashes,
and 20 warm repetitions per query. These are local observations, not an SLA
or cold-cache/cross-environment benchmark.

## Results

Both fresh builds pass and produce identical manifests, database bytes and
logical digests. Independent verification passes on each with empty
`stale_input_paths`; all 21 golden queries agree. Eight additional query result
hashes and complete census checks also agree; checksums remain unchanged after
read-only measurements. `integrity_check` is `ok`; no foreign-key violations.

| Measurement | A | B |
| --- | ---: | ---: |
| Full build (seconds) | 213.67 | 213.81 |
| Independent verification (seconds) | 162.79 | 162.65 |
| Golden queries (seconds) | 0.14 | 0.15 |
| Normalized records verified | 213,471 | 213,471 |
| Raw inputs verified | 4,960 | 4,960 |
| FTS token positions verified | 9,996,277 | 9,996,277 |

Release: `9c44409186e24c88b61c7e5050d6a629dcb4ec9c7fa0e05da05d9966cf5236f6`.
Each database is 485,859,328 bytes (463.352 MiB).
Database SHA: `16ec392e67baf6e1b0bdf94bb939d1e0ae2ebaf0549cba32b66e087e86ca654f`.
Logical SHA: `435078f9e530c4a8fca9954ad56ce1b7915db35b8ebda60fe476a735a8da9f2c`.
Artifacts are under each output root above, in the release-ID subdirectory.

The selected census is 468 instruments; 68,328 provisions (64,822 articles,
3,476 transitories, 30 annexes); 35,949 references, including 15 title-anchored
edges; 12,360 qualifiers; 17 outdated approvals and 16 operator notes; 5,727
terms and 102,432 usages. The plan's older 12,302-qualifier estimate is superseded
by the reviewed M2 census, not a new data change. Inputs classify as 3,287
normalized, 1,545 evidence-only and 128 excluded standard bodies. There are 32
standards, 17 accepted structural-debt rows, 5 accepted-gap instruments and 6
cited-but-missing rows with multiplicity 17. Full table counts, manifest hashes,
input-descriptor digest, environment and all query plans are in the
[evidence](sqlite-m4-reproducibility-2026-10-20-evidence.json). Full per-input
hash descriptors remain in both immutable local manifests.

The committed Rust builder/verifier uses rustc 1.95.0 and bundled SQLite 3.53.2;
its manifest captures compile options and lockfile hash. Supplementary query
measurements use Python SQLite 3.53.4 on macOS arm64, after the concurrent build
and verification jobs finished. They are not a benchmark of the Rust Reader API.
Both artifacts give identical results. Warm median ranges across A/B:

| Query | Median milliseconds |
| --- | ---: |
| Exact ID | 0.0045 |
| Number/type within instrument | 0.0820–0.0825 |
| Outgoing live references | 3.5456–3.5742 |
| Reverse live references | 4.2315–4.2794 |
| Term usages | 0.0352 |
| Ranked literal FTS, first 20 | 2.0570–2.1291 |
| Pipeline coverage | 0.2473–0.2492 |
| Cited-but-missing | 0.0269–0.0272 |

Plans use primary-key/owner/term indexes and FTS. The supplementary SQLite planner
chooses the low-selectivity status index for both live-reference queries and
uses temporary sort/group structures where recorded. This is an observed tuning
opportunity, not a correctness failure or a performance guarantee. No indexes,
statistics or database bytes were changed to obtain these measurements.

Native checks at the permanent commit pass: `cargo fmt --check`,
`cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --locked --workspace --quiet` (399 tests), and locked LRITF/IFPE
validators (145/11/126 and 59/4/115 respectively, zero issues).
No cross-environment build reproducibility is claimed. Neither M4 output is
published externally; the two local roots are retained for review.

## Recovery and boundaries

Use `cargo build --locked -p lex-cli`, then:

```sh
target/debug/lex-mex --root /Users/jr/Dev/lex-mex-db db project \
  --corpus-commit fedcbc5dbe1ae7d87a6dce0e0e521f56b28b9a58 \
  --builder-commit fedcbc5dbe1ae7d87a6dce0e0e521f56b28b9a58 \
  --output-dir derived/db/m4-new-validation
```

Use a fresh output directory; immutable publication refuses overwrites. Verify
the resulting database/manifest with `db verify --corpus-commit <same commit>`,
then `db golden-queries --database <database>`. A future checkout must use this
committed builder's source/schema bytes to reproduce it. Historical manifest-v1
M2 artifacts require rebuilding for version-2 verification.

SCJN stays evidence-only, standards bodies remain excluded, and analyzed
coverage stays `not_assessed`. All existing operator decisions and pending
LAN/LGCS/LGS/LIEPS legal questions remain unchanged. Canonical source text,
review queues and existing Obsidian exports are preserved; this database-only
path generates no Markdown exports. Missing links and structural debt remain
explicit rather than becoming assertions of legal absence.

Next: commit the accepted M4 reports, then the designated integration operator integrates the DB branch into main by a
normal merge preserving `6505baf57` ancestry. No hosting/UI/MCP or deployment.
