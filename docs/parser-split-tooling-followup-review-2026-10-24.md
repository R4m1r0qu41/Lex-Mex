# Parser split tooling and reader follow-up — 2026-10-24

Reviewed branch `refactor/parser-split` at
`3cc9d05e9e45420f17de42460d5434e7b5adb63c`.
Production remains byte-identical to tested C `1250889ce`. No production parser,
corpus, schema or database regression found. No merge or push performed.

**Verdict: A-001 is closed. A-002 is improved but not fully closed: its new
result verifier accepts incomplete evidence. Complete the bounded repair below
before treating unattended restart reconciliation as a gate.** The completed
A/B and database acceptance remain valid; no full A/B repeat is requested.

## A-001 — independently confirmed

The corrected wrappers retain command exit codes, continue collecting diagnostics
and return nonzero if any required command fails. Comparison tooling returns
nonzero for the tested parity failures. Codex reran the current G4 self-test in
scratch: **16/16 pass**. No old wrapper was executed; originals were not modified.

The full current harness self-test was independently rerun in an isolated scratch
clone: **41/41 pass**. Those tests do not cover the completeness defects below.

## A-002 — one remaining P2 issue

In `prompts/parser-split-ab.py::result_problem` (lines 462–501 at reviewed HEAD),
trees JSON is parsed without validating its contents, the logs directory is only
checked for existence, and the row loop uses `result.get("rows", [])`. No run
selection or expected stage coverage is bound to the result. The queue uses
success from this function to mark an unfinished step complete.

Independent isolated probes, holding harness/binary/baseline/input metadata
consistent, were incorrectly accepted in all four cases:

1. Empty trees map and empty log directory despite recorded command rows.
2. Missing `rows` key.
3. Empty `rows` list.
4. Missing per-row `work_tree_sha256` for retained replay stages.

There is also a concrete scope mismatch in the new self-test itself:
`selftest-restart` was produced with `--only ccf`, but
`driver.reconcile("retained", "selftest-restart", "B", [])` accepts it as a
full run. A retained-input manifest digest identifies the available inputs;
it does not establish which inputs/stages were actually run. If complete
baseline/candidate runs are accidentally replaced by matching partial runs,
relative comparisons alone cannot establish the missing coverage.

Required narrow fix:

- Record and validate run selection/scope (selected slugs, stage plan and relevant
  `--only`, `--limit`, `--skip-extract`, supplement options). The queue must
  reconcile exactly its requested full scope, not a diagnostic subset.
- Require a correctly typed, nonempty row list with all conditional evidence
  fields, and expected ordered coverage. Preserve the intentional skip of later
  stages after a failed parse; do not count baseline failures as missing runs.
- Validate saved stage-tree maps against recorded digests and required entries;
  verify required stdout/stderr files exist and match hashes. Empty directory
  presence is not complete evidence.
- Add rejection tests for all four probes, subset-as-full recovery, and a
  positive recovery control with exactly the requested scope. Keep atomic writes,
  preservation of usable results and archival of unusable attempts.

These are evidence/restart changes only. Retain frozen gate harness `9c329e0c`
and old acceptance artifacts; use the new harness only for new self-tests or
new evidence generation. No production reparse or full replay is necessary.

## Reader integration — uncertainty resolved locally

Codex independently hashed all four release trees (1,785 files each) and four
package trees (1,791 files each): merged and R7-only outputs are identical.
Codex also reran Node tests: **24/24 pass**. An initial sandbox-only EPERM on
loopback binding cleared on the same test with local-loopback permission.
The integration commit has the recorded reader/parser parents. The two added
lockfile dependencies are retained, and the recorded locked builds pass.

The locally recorded publication pin is `2df70a8a…`, release `9626bfe4…`.
Both local host publication directories have UI source matching
`b9b8be012c0a19b50e75acd922c38a19385faded`, not R7 `162128517`.
The latter adds UI work only: no Rust source or Cargo difference exists between
those reader commits, so the exporter comparison applies to the locally
published reader's exporter as well.

Codex repackaged the merged release in scratch with the exact b9b8 UI source:
all five packaged UI files match the local publication bytes. The release
manifest's 1,782 listed files match the locally published release, and the only
manifest differences are release ID, corpus commit and commit timestamp.
Thus parser/exporter/package compatibility is verified without requiring an
R7 UI upgrade. Do not describe R7 as the deployed UI. This inspected local
publication pins/directories, not the remote live site. Browser/mutation
campaigns were not rerun; reader source was not changed.

## Evidence incident — accounted for, not hidden

The original native Clippy/test/LRITF logs were overwritten during an executor
mutation test of the old wrappers. They were regenerated; their bytes differ
from the original audit. Exactly these three of 54 bound files differ, and the
current hashes match the disclosed recovery hashes. The regenerated test log
reports 425 passes. Reconstructed candidate JSON summaries match original
hashes; DBs, binaries, A/B results/comparisons and other gate artifacts are
unchanged. The current G4 self-test changed none of these artifacts.

The previous audit remains a historical check of the originals. Current repeat
results must use the replacement hashes and incident record; do not claim the
three replacement logs are the originals. The independently verified database
and A/B conclusions are unaffected.

## Checks and next action

See [machine evidence](parser-split-tooling-followup-review-2026-10-24-evidence.json)
for full commit identities, probes and test results. All unrelated main files and the clean Claude worktree are preserved.
No code changes, commits, merge, push or host writes. Next: Claude fixes the
result-completeness/scope gap and supplies the bounded positive/negative tests
for closure of A-002, then operator decides integration.
