# R6 code-review findings — Codex ruling, 2026-10-22

**F1 requires repair before deployment. F2 retains the reviewed lifetime cap.** This supplements and narrows the earlier R6 acceptance: the post-fill terminal repair remains verified, but the loading-state terminal path does not meet the same visible-text/copy contract. No source, package, pin, commit or host state was changed by this ruling.

## F1 — confirmed combined-state defect; publication hold

`unhold()` clears `held` and schedules tracking, but leaves `loading=true` and `userScrolled=false`. `track()` then returns at its loading guard. `readerMoved()` returns immediately once no hold exists, so subsequent scrolling does not clear this guard either. `fillAll()` completion eventually clears loading and schedules tracking; there is no wall-clock bound on its completion in source. The terminal notice can therefore assert that selection follows visible text while selection and copy still refer to the abandoned target.

Independent Chrome probe against unchanged preferred package `8eb8d8b4…`, at390px/fonts blocked: reuse the accepted terminal-cap probe and additionally inject `loading=true` at the same CDP breakpoint. Result: physical article191, label/index/URL/copy193; visible failure notice; one exhaustion mark, zero intent marks and zero idle corrections. After100px reader scroll the physical article remains191 and copy/selection still193. A new explicit navigation correctly selects193. This confirms F1's state logic.

**Evidence limit:** loading was explicitly injected after initial fill completed, not produced by a naturally stalled `fillAll`. The spent counters and padding remain injected too. Natural combined-state frequency is unmeasured; this is not evidence that ordinary loads take20seconds or routinely exhaust30 recoveries. Nevertheless this is the same failure-state contract used for the original cap repair, and pending loading is not excluded from that contract. The earlier R6 probe only covered loading=false. My earlier acceptance did not cover this interaction and must be narrowed.

Required repair: terminal failure must permit passive visible-text tracking even while filling continues, without fabricating a reader-intent performance mark or weakening explicit-target stability. Add a bounded regression covering terminal exhaustion while loading remains pending, consistent label/index/URL/exact copy before and after reader movement, subsequent fill completion without restoring the abandoned target, and later explicit navigation resetting/settling normally. Preserve healthy aligned holds. Choose the implementation in the repair round rather than prescribing a flag assignment in this review.

A source edit requires fresh packages/IDs, affected browser reports and focused re-review, then restaging/revalidating the selected host package. Existing host pin commit `7d61f95f143f8763371221b483e556d92c0831a7` remains intact and is not deployment approval. The previous npm results remain valid evidence for that old package only.

## F2 — no behavior change required

Keep `recoveries` as a lifetime counter per explicit hold. The original review deliberately requested terminal unholding when the lifetime allowance is exhausted AND the target remains misaligned. Aligned holds are not ended by a timer; a later misaligned burst can exhaust the remaining lifetime allowance. Resetting on alignment would remove that reviewed lifetime bound and is not necessary to fix F1. Clarify comment wording in a future source round if useful; comment ambiguity alone does not warrant another package.

## F3–F7

F3/F4 are accurately disclosed limitations; toast acceptance still depends on correctness of tracking, which F1 now qualifies. F5–F7 are nonblocking harness maintenance/diagnostic issues in this review. A failed settled wait loses diagnostic strength, but independent geometry/copy checks remain meaningful. No additional scope is assigned. The appendix concerns a separate repository/workstream and was not reviewed or authorized for changes here.

Exactly one next action: owning implementer repairs F1, adds the pending-loading terminal regression, and returns a newly bound package/evidence packet for focused re-review. Do not deploy the current R6 package pending that result.
