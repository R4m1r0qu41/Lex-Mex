# DB-first integration handoff — 2026-10-20

## Operator authorization

Operator: "Commit M4 report. Then Claude merge in order (the database branch,
the batch, the rulings branch), update the database test expectations, and push
main after Codex's review of the merged result."

M4 report/evidence committed as `047b421efe9ef6575288bb6d35b6190fc9e2d192`,
parent `fedcbc5dbe1ae7d87a6dce0e0e521f56b28b9a58`. Exact two documentation
paths; no code/corpus/derived artifacts and no push. Required source-code gates
at fedcbc5d passed previously (399 tests and required validators); the doc
commit passed JSON/evidence identity and staged-diff checks. Independent M4
acceptance includes Claude's third separately compiled same-host release build.

## Receiving owner and sequence

Claude owns integration. Codex has not invoked another provider or merged.
Verify current branches and dirty state; use normal merges, no squash/rebase:

1. Merge `db/sqlite-projection` into main, including `047b421ef`, preserving
   `6505baf57c7aa7051cffe62742f46f10276a834a` ancestry.
2. Merge `parser/pending-batch`.
3. Merge `parser/operator-rulings-2026-10-20`.

The parser workspace was last observed at `4069fe65d`, after `b16ebc0ba`.
Those commits arrived after the reviewed `1647daaa8`: their new institution-name
filter/imports and the accepted-gap/partial-repeal repairs have not been reviewed
by Codex. Do not treat older counts or findings as their acceptance. Inspect
current operator decision records and keep unanswered rulings explicit.

Preserve all unrelated work: the parser workspace contains untracked operator
suggestions and `prompts/adr-citation-grammar-and-tickets.md`. The DB worktree
also retains draft plan/review documents and their evidence; do not discard or
sweep them into a code commit. This handoff and the latest plan are draft
navigation, while the committed M4 report is the accepted database receipt.

## Expectations and acceptance

Update adversarial fixture strategy and every affected golden expectation at
the appropriate merge, recording before/after values and source-based reasons.
Use the actual final strategy/census; do not assume strategy 6 or the previous
99,516 usage count is still current after b16ebc0ba. No blanket schema relaxation
or changing goldens merely to make tests pass.

Review the previously reported paths, especially:

- Accepted-gap glossary-only refreshes must reach optional external Obsidian
  publication and the batch report without falsely marking accepted reference
  gaps fully valid. Test external export, expected/extra errors and idempotence.
- ltfccg Article 16's scoped partial repeal must retain evidence-backed machine
  provenance after parse→derive→reparse→derive, with commencement unknown and
  human/model-review guards preserved.
- New authority-name filtering, import readings and remaining operator rulings.

Run fmt, strict workspace Clippy, workspace tests and locked LRITF/IFPE
validators; use private snapshots for write-capable checks of immutability.
Run affected ingestion/link/export stages and inspect manifests, validation,
canonical/review diffs and exports. Then two fresh M4-style builds against the
permanent integrated commit, independent full verification with empty stale
inputs, all goldens, and byte/logical/count comparisons. Preserve historical
release `9c444091…`; no in-place changes to published local artifacts.

## Push condition

Return the exact merged commit, merge parents, source/census/fixture deltas,
validation results and artifact evidence to Codex for review. **Do not push
main until Codex reviews and accepts that exact integrated result.** Existing
M4 acceptance is for fedcbc5d only and is not this integration approval.

Next action: Claude performs the ordered local integration and checks, then
hands the result back to Codex. This handoff does not start Claude automatically.
