# Reader R6 — focused Codex re-review, 2026-10-22

**Verdict: prior P2 terminal recovery-cap finding resolved in both R6 packages. No new blocking finding within this focused scope.** Publication and integration remain operator decisions.

Reviewed HEAD `256d1b410917d435b1e355c3b84a71d190f658bc`, branch `feat/four-instrument-reader-design`. R6 is committed at this HEAD; the R6 plan heading's “uncommitted” label describes its preparation state. Entry worktree contains only three pre-existing untracked R5 review/account-switch documents. Codex preserved them and made no source, package, host, pin, corpus, schema or database edits.

## Repair and regression assessment

The terminal branch now calls `unhold()` only after the recovery allowance is exhausted while the target remains unaligned and no settle chain is active. `unhold()` cancels the timer, clears `held`, restores browser anchoring and schedules passive tracking. The failure notice is emitted without calling `intent()`. The aligned path does not expire a healthy hold, and `hold()` explicitly resets all three counters for later navigation. This meets the prior P2 repair contract.

Scenario 27 correctly distinguishes an injected terminal state from natural-storm coverage. Its geometry oracle reads the physical reading line; it checks the requested article before the fault and compares label/index/URL/exact copy against physical truth after the fault. The same final harness fails on unchanged R5 bytes, supporting discrimination rather than merely internal agreement. Its scroll check stays within article 191; it proves continued consistency, while the initial 193→191 transition demonstrates resumed tracking. Counter reset is established by source inspection; the browser check demonstrates successful re-navigation, not direct counter observation.

The 5.5-second replaceable status toast is acceptable for this repair: tracking/copy correctness follows from releasing the hold and does not depend on retaining the toast. No persistent failure banner is required by the reviewed contract. Natural frequency of exhausting 30 chains remains unmeasured.

## Independent verification

- Source SHA-256: `reader/reader.js` = `68344a3001803e3282c135259383d07f1ad3225f7e934c5f71476d4b2879a28a`; `reader/browser-check.mjs` = `954951a3a292af81eda17304e9ba076574edf0004d24c7e5d8d64fd8f89f35f9`. Both match the plan and committed source; shipped JS matches.
- Preferred publication `8eb8d8b4fcfd09c7b9414263494789207de38ea1968b01103fbb010df2b65a6f`: 1790 listed files / 1791 disk files. Fallback `9afcaedbd7aad277cdd4d0c63ee0a019675cf0b933454b2594d1dfa299b2a30a`: 1784 / 1785. All listed sizes/hashes, descriptor IDs and manifest hashes verify; no extra files; A/B pairs byte-identical. Each differs from its corresponding R5 package only in `reader.js` and `publication.json`.
- All five R6 final/discrimination report hashes match the plan. Recounted result arrays agree with reported totals. Preferred 57/0; fallback 54/0 + 3 N/A; legacy-staged 72/0; preferred without anchoring 57/0. R5 control 53/1 + 3 N/A, failing only scenario 27. These complete-suite runs are attributed to Claude, not new Codex reruns.
- **Fresh independent focused Chrome rerun: 4/4 R6 configurations PASS**, preferred, fallback, legacy-staged and preferred without anchoring. Used scenario 27 extracted into an isolated runner with original shared setup/helpers; no shipped bytes modified. Each records visible/tracked article 191, exact copy, notice, one exhaustion mark, zero intent marks, zero idle corrections and successful new navigation to 193 at 8px below the bar. No page errors.
- **Fresh independent R5 control: expected FAIL**, reproducing visible 191 with label/index/URL/copy on 193 and no notice. The later reader scroll generates intent, unlike R6.
- Node tests passed **22/22** in the preceding verification turn on these same source hashes. Sandbox `listen EPERM` cleared on elevated rerun. Current `git diff --check` passes via mechanical check. R6 commit touches only the plan, UI JS and browser harness; no Rust/native/canonical gates repeated for this UI-only review.

## Limits and next action

This is a focused R6 review, not a new full publication audit. Full browser suites, mutation campaign, rollback/fail-closed rehearsal, Safari/Firefox and real-device behavior were not independently rerun here. Prior accepted content/source baselines remain the basis for package choice. Evidence includes reproducible isolated probe source and all focused results in `four-instrument-reader-r6-codex-review-2026-10-22-evidence.json`.

Exactly one next action: operator selects the preferred or fallback package and grants the specific publication/integration actions desired. Technical R6 UI hold is cleared for the reviewed hashes; this review performs no commit, merge, push, host staging, pin change or deployment.
