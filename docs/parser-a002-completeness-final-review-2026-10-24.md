# A-002 completeness closure and integration recommendation

2026-10-24. **A-002 is closed. No remaining blocker was found in the reviewed
parser/tooling scope.** Earlier production-refactor acceptance and D-001
acceptance stand. This does not resolve the 106 missing retained inputs or imply
a remote publication, merge, push or deployment.

Reviewed HEAD: `11a3a90952aa05beeb9db62be095c6b54da6bbad`.
Repair: `4d98f72c2753d280cd8315fb3bb76276b66bcf0f`.
Tested production C: `1250889ce10b8ba910a5f5ca5f232fd5752e79df`;
no production/corpus/schema/adapter/fixture changes followed C.
[Machine evidence](parser-a002-completeness-final-review-2026-10-24-evidence.json).

## Verification and claim dispositions

| Item | Disposition |
| --- | --- |
| Recorded/requested scope binding | Confirmed. Command/options, selected slugs and stage plan are compared; a one-instrument diagnostic result is rejected as a full replay. |
| Complete ordered rows | Confirmed. Missing/empty rows, missing conditional evidence and dropped stages are rejected. Exact-scope control succeeds. |
| Required logs and stage maps | Confirmed. Required streams are read and hashed; required nonempty stage maps must match recorded digests. Independent tampering/missing-file probes fail closed. |
| Intentional parse failure | Confirmed. The baseline lfiiedb failure is accepted with one parse row and subsequent per-instrument stages skipped, rather than being mistaken for incomplete execution. |
| Atomic state/results and preservation | Confirmed by retained implementation and positive/negative self-test coverage; valid results are protected and invalid attempts archived. |
| Full self-test | Independently rerun in a fresh isolated scratch clone: **54/54 pass, exit 0**. |
| Plan fidelity | Independently rerun read-only against all 15 real gate results: **15/15 pass, exit 0**; row counts 1771/2660/35 match the executed plans. |
| Artifact preservation | All 54 entries checked; 51 original hashes match and exactly three regenerated native logs match their disclosed recovery hashes. No artifact changed during testing. |

Additional independent probes used a known exact-scope control and changed one
thing per negative case. All four prior gaps now reject, as do missing logs,
subset-as-full, tampered stage tree and missing scheduled row. No full A/B or
native rebuild was necessary for these tooling-only changes. The current
comparator deliberately rejects old gate results as a different evidence format;
use gate harness `9c329e0c` to reproduce the old strict comparisons. The new
plan-fidelity check exercises completeness without pretending to regenerate the
old harness identity.

## Known gaps: no additional fix required for closure

Unreferenced extra tree-map entries or diagnostic log files do not satisfy any
required selected entry: all requested scope rows, required maps and streams are
verified. Ignoring these extras is acceptable here; they are not canonical
corpus or publication inputs. It is not a reason to remove preserved diagnostics.

A live inventory change that causes requested scope to differ rejects the
result. That is a safe failure, not silent admission of reduced coverage. Keep
the original manifests, inputs and source commit for later reproduction; a
pinned-inventory convenience improvement may be considered separately.

## Integration decision

Recommended default: **normal-merge parser into main, then sync that main into
the published reader branch at b9b8be012; keep R7 separate.** This integrates the
accepted parser without making an unpublished UI upgrade part of its release.
It matches the local publication pin/UI established by the previous review.
R7's Rust/exporter code equals b9b8, and remains available for a separate UI
release decision. No remote live-site assertion is made from the local pin.

Prepared [integration execution plan](plans/parser-reader-integration.md) with
two local normal-merge candidates, full native gates, read-only existing-DB
verification/goldens, reader exporter/package byte parity, preservation inventory
and commit-bound audit before actual branch promotion. Candidate work can use
the handed-over clean operator-created parser worktree on new branches; reader
worktrees stay untouched. Preserve source ancestry; do not squash or rebase.

Main has one parser-plan untracked collision; a later broader reader-to-main
merge would have eight additional overlaps. The plan binds exact-byte backups
and reconciliation before any authorized promotion. Unrelated drafts/operator
suggestions and existing audit reports must survive intact. No blanket stash,
clean, reset or branch/worktree deletion.

The operator may choose another integration scope. This recommendation is not
permission to move actual main/reader branches, push or deploy. The prepared goal
prompt authorizes local reviewable candidates when explicitly activated.

## Outcome and next action

Only this report/evidence, the integration plan and its PLANS.md entry were added
on main; unrelated work and the clean parser worktree remain preserved. No code
change, commit, merge, push, promotion or deployment. Next: operator activates the
integration goal with the recommended scope or chooses the desired broader scope.
