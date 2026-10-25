# Reader R5 — independent Codex re-review, 2026-10-22

Reviewed branch `feat/four-instrument-reader-design`, HEAD
`ca58f5879f030ac3cba604374187b350cc82c1ef`, with code from `b8c344b97`
against `d87db82c0`. The worktree was clean on entry. The two local commits
reported in the handoff exist; the earlier receipt's uncommitted wording is
historical. This review authorizes no additional commit, merge, push, host
staging, pin change or deployment.

**Verdict: the ordinary requested-target repair is supported, but both final
options remain held for one terminal recovery-cap defect.** The proposed
preferred and fallback content/source baselines remain accepted from R0–R4;
no corpus, database or legal-review change is requested.

## Finding — P2: recovery-cap exhaustion freezes a mismatched selection

`reader/reader.js:190–193` marks lifetime recovery exhaustion and returns,
leaving the target held. `track()` at line 206 then suppresses passive selection
updates indefinitely. Once layout leaves the exhausted target off-line, label,
index, URL and copy keep the requested article while the reading line shows
another provision. No user-facing failure notice is produced. Pressing Copy
does not move the column and therefore does not release the hold.

Independent Chrome probe on **unmodified shipped fallback bytes**:

1. Open LIC 193 at 390px with fonts blocked; wait for `reader:target-settled`.
2. Using a CDP breakpoint in `keepHeld`, inject the disclosed terminal state:
   `realigns=41`, `nudges=401`, `recoveries=30`, `settleTarget=null`.
3. Add synthetic padding above the target to trigger a layout change. At the
   paused callback, `drift(held)` is 1367.875px and `userScrolled` is false.
4. After 1.8s, `reader:hold-recovery-exhausted` is marked once. The visible
   reading-line provision is **191**, the requested 193 is 1375.875px below the
   bar, but label/index/URL and copied body still identify **193**.
5. A subsequent 100px reader scroll releases the hold and updates the selection
   consistently to 191.

This is **explicit state fault injection plus a synthetic layout change**. It
proves the terminal branch's behavior, not that 30 exhausting storms occur in
ordinary use. Its frequency is unmeasured. The limitation was disclosed honestly
by the executor; it is not an undisclosed normal-path regression. Nevertheless,
the bounded failure path should preserve the Reader's visible-text/copy contract.
Both options use exactly the same UI bytes, so both inherit it.

**Required repair / answer to the cap question:** end the hold when its lifetime
recovery allowance is exhausted **and the target remains misaligned**. Stop
settling/correction work, resume passive tracking and communicate that the
requested position could not be maintained. Do not silently keep a false active
selection. A later explicit navigation may start a fresh hold. This does not
require a timer ending a healthy, aligned hold; that would undermine late-font
stability. An equivalent explicit failure state that prevents misleading
selection/copy would need its own demonstrated UI contract; the smallest current
repair is terminal unholding with a visible status.

Add a bounded deterministic regression for terminal exhaustion, then assert:
label/index/URL/copy agree with the reading line or an explicit failed-navigation
state; no continuing correction loop; reader movement still wins; a new explicit
navigation resets the allowance and settles correctly. Distinguish this boundary
test from natural resize-storm coverage.

## Independent verification actually performed

- All handoff SHA-256 bindings match: final UI, harness, R5 plan, R5 receipt,
  evidence and prior review inputs. Reviewed source equals shipped `reader.js`.
- Preferred `fbcb4e98b64f5dd00c82a387ce04a35c93681f9d1b4d52ce1181ca55d22e7ea3`:
  A=B, 1791 disk files, all listed sizes/hashes valid, no extra paths; descriptor
  ID and manifest hash valid.
- Fallback `4a3b2e2fa13871f7dbe0903e317b091cfc52091ad376c62f01f3b9b630dc742c`:
  A=B, 1785 disk files, same checks valid.
- Only `reader.js` and `publication.json` differ from the corresponding R4
  packages. Native/schema/Cargo/corpus Git delta from the independently reviewed
  R4 baseline is empty. Prior native and source/content audits are reused,
  explicitly not rerun. The current host pin still equals the packet's old pin;
  no host files were written.
- `node --test reader/`: **22/22 PASS** after retrying outside the sandbox that
  denied loopback listening. The first 21/22 result was `listen EPERM`, not an
  application assertion defect.
- Independent targeted legacy-release probe against `.work/r10-legacy-stage`:
  **4/4 PASS** (390px normal, 1.5s late fonts, blocked fonts; 1440px normal).
  Requested article 193, reading-line truth, index, URL and exact copy agree;
  share preserves release 3c04. Offset below the bar is 7.875–8.125px. The staged
  UI hash matches the final source.
- Full preferred harness: **43 PASS lines**, then no final completion/report;
  reviewer terminated it with exit 130 after about 178s total (175s polling).
  Time since the last PASS was not separately recorded; this is an incomplete
  reviewer run, not proof of a harness or application defect. No Chrome stderr
  or executor error was emitted. No final JSON exists. Fallback's full
  independent run was not attempted after that stop. Do not promote this partial
  run to 56/0 or claim independent reproduction of the executor's complete runs.
- Isolated next touch case with per-CDP-operation timeouts **passes**: delayed
  fonts loading at input, 2px tap, zero intent marks, requested193 remains
  visible/selected and copy-exact. All three touch dispatches completed in
  12–17ms. This does not establish why the full run stalled.
- The fault-injection boundary probe above reproduced the cap mismatch.
- `git diff --check` on the reviewed clean tree passes. No Rust gates repeated
  for this UI-only re-review; no source/package edits by Codex.

Evidence and reproducible probe scripts are embedded in
`docs/four-instrument-reader-r5-codex-review-2026-10-22-evidence.json`.

## Harness proof value and limits

The new stability matrix checks the requested provision, physical layout,
selection, URL and complete copy text; this is stronger than internal agreement
on the wrong article. Real-input and unattributed-scroll cases exercise the new
intent rule. Redundant surviving hooks are not inherently defects.

The new settled-mark assertion causes old R4 builds to fail even where they land
correctly, because they lack that mark. Therefore “R4 fails25” does not mean 25
independent user-visible defects. The executor explicitly distinguishes these.
The separate wrong-target geometry/copy results are the meaningful regression
proof.

The 800-step storm and no-quiet-recovery mutant support a need for recovery, but
scenario26 samples exhaustion after the storm and recovery waits. It does not
prove when exhaustion happened, that the budget was continuously exhausted
through the storm, or the lifetime cap. Do not claim those stronger properties.
For stronger chronology, observe the exhaustion mark before quiet recovery and
record ordered exhaustion/recovery/settling events. The terminal-cap regression
above is required; an expanded mutation campaign or removal of surviving hooks
is not.

The executor's 56/0 preferred, 53/0+3N/A fallback, 71/0 legacy, no-anchoring,
fail-closed and host-copy rollback receipts remain attributed evidence. This
review verified package identity rather than rerunning every rehearsal. Late
font displacement after genuine reader intent, Safari/Firefox/real-device
coverage and single-frame painted drift remain disclosed limitations; no new
finding about their frequency is asserted here.

## Readiness and next action

- **Preferred:** content/source baseline accepted; UI publication held for the
  terminal-cap fix and regression.
- **Fallback:** same UI hold; retained-content baseline accepted.
- **Recovery cap:** terminal unholding (or a demonstrated explicit failure UI)
  is required when exhausted and unaligned. A healthy aligned hold need not end.

Exactly one next action: the owning implementer repairs the terminal cap path,
adds the bounded regression, then returns fresh final UI/package hashes and
completed preferred/fallback/legacy browser reports for a focused re-review.
Keep approved corpus and DB work closed. Operator choice of package and all
publication/integration actions remain separate.
