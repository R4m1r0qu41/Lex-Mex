# Four-instrument Reader — R5 receipt: requested-target stability (Codex final-review follow-up)

2026-10-22. Executes R5 of `docs/plans/four-instrument-reader-design-release.md` (received from Codex at SHA-256
`f0b27d845696f8103990dc878b02a8db90529d190c378f04ae4f442bd7706c38`) after
`docs/four-instrument-reader-r0-r4-codex-review-2026-10-22.md`. It supersedes the package IDs in
`docs/four-instrument-reader-continuation-receipt-2026-10-22.md` (`f5a1937a…` and `709b09c4…`). Machine-readable
evidence: `docs/four-instrument-reader-r5-evidence-2026-10-22.json`. Executor: Claude Opus 5.5 (the plan names Sonnet;
this routing deviation is recorded here). **Nothing is committed, staged to a host, deployed, merged or pushed.**

## Outcome

| Option | Publication | Release | Status |
| --- | --- | --- | --- |
| **Preferred** | `fbcb4e98b64f5dd00c82a387ce04a35c93681f9d1b4d52ce1181ca55d22e7ea3` | `9626bfe4…` (870 provisions, corpus `85c41f389`) | ready for Codex re-review |
| Fallback | `4a3b2e2fa13871f7dbe0903e317b091cfc52091ad376c62f01f3b9b630dc742c` | `3c040166…` (867) | ready; same UI over retained content |
| Rollback | `407aa3dc…` | `3c040166…` | currently deployed |

Packages (ignored): `.work/r10-preferred-a` = `-b` and `.work/r10-fallback-a` = `-b`. Only `reader.js` and `publication.json`
differ from the R4 packages, so the release content, references and PDFs are byte-identical to the content Codex verified. Builds
`r5-*` through `r9-*` are superseded intermediate attempts. They are preserved, and their results are recorded below.

## Defect, reproduced and diagnosed

Codex's finding reproduced on the old final bytes (`.work/r4-fallback-a`). A 390px pinned link to LIC Art. 193 ended
on Art. 191 or 192, and label, index, URL and *Copiar artículo* all named the wrong article. The trace
(`.work/evidence/r5/trace-fallback-r4.json`, 12 runs) shows two mechanisms:
- At narrow widths, the estimated heights of articles that have not rendered yet are far off. After the jump, the
  target oscillated by about ±2,500px as neighbouring articles rendered, and the bounded settle (6 × ~60ms) could end
  misaligned.
- A late web-font swap (the stylesheet imports Google Fonts with `display=swap`) moved an aligned target by 30px.

In both cases the R1 trailing pass then truthfully selected whatever article was at the line. Fonts are one cause,
not the only one: with fonts blocked, 1 of 3 runs still failed. The same UI serves legacy 3c04 links after a
preferred deployment, so both options were affected.

## Repair (`reader/reader.js` only)

An explicit navigation (entry, index, search, reference, instrument switch, back/forward) **holds** its target at the
reading line until the reader moves the view or navigates again:
- **Anchoring.** While held, the browser's scroll anchoring is off on the column, and this code does the anchoring.
  A ResizeObserver on `#flow`, `#active-bar` and `#load-status`, the font `loadingdone` event, and the end of each
  settle all realign the target synchronously. A shift during a settle chain is corrected at once. Alignment is
  measured against the position the column can actually reach (`drift`), so the first and last articles settle too.
- **Intent.** The only sign of intent is the column moving away from where this code last put it. Keyboard, wheel,
  touch, scrollbar, find in page, focus and assistive technology all produce that. It is checked in the scroll
  handler, before every realign and before each settle retry. A clamp to the bottom edge caused by shrinking content
  does not count. Pressing buttons, or a tap that doesn't scroll, is not intent.
- **No release timer.** The hold has no time limit; it ends only on intent or the next explicit navigation. While held, passive
  tracking does not reassign the selection, so a held entry is never rewritten by a passive pass.
- **Bounds.**
  - Realigning is limited to 40 chains plus 400 immediate corrections. Both reset once the target is aligned.
  - A spent budget allows one more chain after 700ms of quiet, at most 30 times per hold.
  - Exhaustion is marked (`reader:hold-exhausted`, `reader:hold-recovery-exhausted`) and never ends the hold.
- **Other code paths.** *Volver* re-aligns a held target instead of restoring a stale scroll position. A stale
  fill's final realign only applies to the target that is still held.
- **Marks.** Informational performance marks: `reader:target-settled` and `reader:intent`.

## Review history (fresh bounded Opus verifier each round; none saw this transcript)

| Round | Build | Verdict | Defects found and fixed |
| --- | --- | --- | --- |
| R5 | first hold (timer release, event-list intent) | partial | undetected scrolls snapped back; unenforced 8s cap; pointerdown/keydown false intent; late fonts after cap |
| R6 | anchoring-owned hold | partial | *Volver* restored a stale position; budget exhaustion unheld a misaligned target (URL rewrite); wheel/touch false intent; realign could swallow a pending scroll; bottom clamp as intent |
| R7 | movement-only intent | not ready | endless realign loop on first-provision targets (unreachable −8px; budget restarted) |
| R8 | reachable alignment, nudges | partial | no recovery after exhaustion in a long storm; stale `fillAll` could re-select an old target |
| R9 | quiet-spell recovery | partial (no code blocker) | the storm test did not assert exhaustion; recovery had no lifetime cap |
| R10 | recovery cap (final) | code ready; test partly proven | no code defect. Disclosed: the storm does not prove when exhaustion happened; the cap is untested; open choice whether to end the hold at the cap. The mutation rerun on the final harness was requested and is in the evidence. |

R10 adds the lifetime cap of 30 recoveries per hold and the storm assertions. Its review saw only the R9→R10 delta
(`.work/evidence/r10-review-packet.md`). Every reviewer was a Claude Opus verifier, not an independent model family.
Codex is the next independent reviewer of the final bytes.

## Verification of final bytes

Final bytes:
- `reader/reader.js` `2ab4d37360a233e63c833f163b9bf8c2256cb4ece82e412b61591830a954a57d` (identical in all four packages);
- harness `reader/browser-check.mjs` `c647db43fd96024ca24b931f6b1e9e4c9cf2aca154e71356ef3819106fcfe79c`.

**Harness change during the final run.** The first r10 batch used harness `643d628c`. Scenario 26 failed on 2 of 3
preferred runs because the 300-step storm never used up the realign budget (`exhaustedMarks 0`). In those runs the
target ended on Art. 193 at 8px. Measured standalone, exhaustion landed 5.3–6.2s into a 6.7s storm, so the
precondition was marginal. The storm now runs 800 steps (about 17.8s); no assertion changed. Everything below was
rerun on the new harness. The first batch is kept in `.work/evidence/r5/superseded-harness-643d628c/`.

| Check (final harness) | Result |
| --- | --- |
| `node --test reader/` | 22/22 |
| Preferred `r10-preferred-a`, run 1 and run 2 | 56 passed, 0 failed (each) |
| Fallback `r10-fallback-a`, run 1 and run 2 | 53 passed, 0 failed, 3 N/A (the 112 articles are not in 3c04) |
| Legacy 3c04 links on the preferred-staged directory holding both releases (`.work/r10-legacy-stage`, `--legacy-release 3c04…`) | 71 passed, 0 failed |
| Preferred with browser scroll anchoring disabled | 56 passed, 0 failed |
| Same harness on old builds | R4 final fallback (`.work/r4-fallback-a`): 28 passed, 25 failed. R8 fallback (no quiet recovery): 52 passed, 1 failed (storm: Art. 192 at the line, offset 110) |
| Mutation run (`.work/evidence/r10h-mutation-run.json`) | 11 of 17 mutants killed. Killed: no lock-expiry timer, no scroll listener, no hold, no layout observer, no movement intent, no clamp exception, browser anchoring kept while held, *Volver* restoring a stale position, no reachable alignment, no immediate nudge, and no quiet recovery (fails only the storm). Survivors: no hold suppression, no `#load-status` observe, no fonts listener, no settle-end realign, no `fillAll` target guard, no intent check before a settle retry |

The stability matrix (scenario 16) checks the requested article in all of these: label, ground truth at the line,
URL provision, exact *Copiar artículo* text, 8±3px below the bar, the settle mark, and no intent or exhaustion marks.
It covers LIC Art. 193 at 390px with fonts normal, 1.5s late and blocked (3 fresh browsers each) and 10s late, and at
1440px with all four. On the old R4 bytes, all 7 runs at 390px with fonts not blocked land on the wrong article: Art. 192 at 38px
or Art. 191 at 2,587px. With fonts blocked, and at 1440px, they reach 193 at 8px. They still fail there, because
R4 has no settle mark: the settle check fails by construction, not because of a misplaced target.

**Packages and content.**
- Rebuilt pairs are byte-identical: `r10-preferred-a` = `-b` and `r10-fallback-a` = `-b`.
- Against the R4 packages, only `reader.js` and `publication.json` differ.
- `publication.json` SHA-256: preferred `07a70bdc…`, fallback `75ae9ac3…`.
- Package verification:

  | Check | Preferred | Fallback |
  | --- | --- | --- |
  | Files listed = files on disk | 1791 | 1785 |
  | Provision files present | 1740 | 1734 |

  Both: 0 forbidden paths, 0 denylist hits, 32 PDFs all referenced and present, 6 routes served with the right MIME
  types, a bad route returns 404.
- Fail-closed: 6 of 6 tampered inputs were refused and nothing was written (`.work/evidence/r10-failclosed.json`).

**Pin delta** (`.work/evidence/r10-pin-delta.json`, proposed pins in `.work/evidence/proposed-pins-r10/`):

| Option | release_id | publication_id | corpus_commit | manifest_sha256 |
| --- | --- | --- | --- | --- |
| Preferred | 3c04… → `9626bfe4…` | 407aa3dc… → `fbcb4e98…` | 9e990e09 → `85c41f389` | c2b22653… → `8fb340f2…` |
| Fallback | unchanged | 407aa3dc… → `4a3b2e2f…` | unchanged | unchanged |
| Rollback | current host pin | — | — | — |

**Rollback rehearsal** (`.work/evidence/r10-rehearsal.log`), using a copy of the host verifier on a copy of the host
directory:
- Preferred staged and verified. The old pin is correctly rejected.
- Staging the deployed publication back verifies; only `releases/9626…` remains as an addition.
- An exact directory restore is identical to the pre-state.
- The fallback stages, verifies and rolls back.

**Screenshots** (`.work/evidence/screenshots-r10/`, both packages): five views per package, matching the R4 set.
In the LIC 193 view at 390px, the label, heading and reading line all show Art. 193 after the settle mark.

**Native gates.** Native inputs are unchanged from the 424-test receipt: no Rust, schema or Cargo change. The hashes
are bound in the evidence JSON, so the gates were not rerun.

## Holds and limitations

1. **After a hold ends**, a late web-font swap can still move the view through the browser's own scroll anchoring.
   This was seen after a PageDown with fonts 1.5s late: about 2.8k px. The selection follows the view, so label,
   URL and copy stay consistent, but the reader is displaced. This predates the hold and affects every build.
2. **Requires reader input.** While held without intent, passive tracking is off. Once the bounds are spent,
   including the 30-recovery cap, the label and URL stay on the requested article even if the target is
   misaligned, until the reader scrolls. Before the cap, recovery always realigned it. Reaching the cap takes about
   30 budget-exhausting layout storms in one hold; this was never observed outside the test. Ending the hold at the
   cap instead is an open design choice, raised by the R10 reviewer.
3. **Mutation evidence.** Some hooks survive mutation, so their necessity is not proven. They are listed in the
   evidence. The `fillAll` target guard covers a race that is narrower than the harness can reach. No test reaches
   the 30-recovery cap. Scenario 26 checks that the budget ran out, but not that it ran out during the storm rather
   than in a failed recovery afterwards. On the final harness, the `no-quiet-recovery` mutant still fails scenario 26, so the scenario requires recovery. The `#load-status` observer was killed only by the old 300-step storm and now survives.
4. **Not tested:** real Safari, Firefox, real touch or real devices. Smooth scrolling was tested only for PageDown,
   during font loading.
5. **Painted drift.** During loading a target can be painted off-line for about one frame. The maximum observed was
   90px; it is informational, not asserted.
6. **Carried over:** when no target is held, *Volver* restores the stored scroll position.

## Decisions requested

1. Codex re-review of this R5 packet.
2. Local commit authorization for the R5 changes. The earlier staged-commit instruction covered R0–R4; this follow-up
   is not assumed covered.
3. After Codex: choose the preferred or fallback option, and separately authorize host staging, the pin change,
   deployment, and merge/push.

This receipt is not a legal determination and does not represent official law or current validity.
