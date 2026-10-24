# Four-instrument Reader R0–R4 final review — 2026-10-22

**Verdict: changes requested; publication readiness held for one reproduced
navigation defect.** Reviewed frozen branch feat/four-instrument-reader-design
at d87db82c082f3f0041070e52eb1636b1e46f98cf in lex-mex-reader-four. Its working
tree is clean. No implementation, package or host files were modified.

## Blocking finding: narrow pinned deep link selects the wrong article

Independent execution of committed reader/browser-check.mjs against final
`.work/r4-fallback-a` fails `narrow 390px deep link` on **both runs**:
requested LIC Article193, but layout truth, active label/index/title and replaced
URL settle on Article191. No horizontal overflow and no UI/truth disagreement:
the tracking repair correctly describes the wrong landing position. This is
not the disclosed disabled-scroll-anchoring offset limitation: both independent
runs used the default scroll-anchoring behavior.

The preferred870 package passes28/28, but the same reader UI can load the retained
3c04 release through an old pinned URL after preferred deployment: the stager
preserves that release and reader.js selects the requested release. The failure
therefore affects backward-compatible reading, not just an unused fallback
artifact. Do not advertise either rollout as fully ready until fixed, or until
an alternative explicit legacy-link behavior is reviewed and authorized.

Reproduce with:
`node reader/browser-check.mjs --publication .work/r4-fallback-a --out <new-temp-receipt>`.
Do not overwrite existing artifacts or erase failing receipts. Expected LIC193
exists in that release; this is not the disclosed unknown-ID fallback behavior.

Likely area to investigate: bounded scrollToIndex realignment, deferred article
layout and late font/layout changes before passive tracking replaces an explicit
navigation. reader.css imports Google fonts with display=swap. A font-related
cause is a hypothesis, not proven by this review. Capture timing, requested ID,
actual URL, font readiness, target offset and settle/lock state before choosing
a repair. Preserve user scroll interruption. Do not fix a user-visible wrong
article by weakening the assertion or merely increasing the harness sleep.

## Independent checks completed

| Check | Result |
| --- | --- |
| Clean frozen branch; diff check | pass |
| Node |22/22 pass |
| Preferred browser |28/28 pass |
| Fallback browser run1 |24 pass,1 fail,3 N/A |
| Fallback browser run2 |24 pass,1 fail,3 N/A; same failure |
| Preferred A/B receipts, all file hashes/bytes and exact pair identity | pass;1790 listed files each |
| Fallback A/B same checks | pass;1784 listed files each |
| Current sourceUI versus packagedUI |4/4 exact |
| Native input hashes versus reported424-test state |10/10 exact |
| Selected references |1527 spans/target-published flags;0 mismatches |
| LIC public versus canonical |511 IDs/bodies exact; audited old/new file hashes exact |
| LRITF old/new public files |314 byte-identical |
| Retained source PDFs |32 identical, names match SHA256 |
| Proposed preferred pin versus actual publication/manifest | all fields match |

The fallback's three N/A cases are recovered112 Quintus/Sextus/Septimus, absent
from that historical release. Preserve those exclusions; the actual failure is
Article193, which exists. File totals above exclude publication.json itself;
Claude's1791/1785 disk counts include it and are consistent.

Evidence: [full receipts](four-instrument-reader-r0-r4-codex-review-2026-10-22-evidence.json).
No new full Rust gate, full DB build, mutation campaign or host test was run by
Codex. Claude's424Rust/isolated validators, six fail-closed cases, rollback
rehearsal and mutation receipts remain attributed to Claude. No live or Safari/
Firefox/real-device validation is claimed. Browser harness runs used disposable
headless Chrome profiles and temporary read-only local servers.

## Repair, mutation and content verdicts

The R1 repair addresses the original observer-only failure: scroll/rAF position
tracking plus a trailing lock timer. Mutation evidence supports the scroll
listener and lock-expiry timer. Removing six other feeds caused no attributable
failure; redundant scheduling is acceptable, but this does not prove all six
are necessary. No demand to remove them simply to satisfy mutation counts.
The100dvh spacer fixes the reported tall-viewport terminal-article issue; those
cases passed independent final-package runs.

LIC technical/source fidelity remains cleared:26 classified shared changes,
three recovered separate articles, no removed IDs, no shared temporal/review
changes, matching accepted corpus and DB. Do not reparse LIC or ask JRH to
re-approve these same deterministic repairs. DB M1–M4 remains accepted.

**IFPE46 retarget accepted:** exact source span2108..2109 names Article9 of the
ITF Disposiciones. ITF Article9 exists in the selected release. The retained
IFPE9 target was incorrect; no new legal-effect ruling follows from this
reference correction. Preserve the correction in any rebuilt package.

## Publication, rollback and merge disposition

Preferred f5a1937a/release9626 and fallback709b09c4/release3c04 are the tested
baselines, not approved deploy IDs. A sharedUI repair will create new publication
IDs for both. Recompute all pin fields from actual final files. Preserve the old
407aa3dc publication/complete pin as rollback; LIC content does not need changing.

Accept the distinction between exact directory restoration and non-destructive
stage rollback. The latter leaves9626 published. An exact rollback can preserve
new directories by moving them into an ignored non-served quarantine instead
of permanently deleting them; no blanket cleanup is required. Do not implement
host writes or changes to old PDF routing under this review.

Six untracked main-workspace review inputs overlap branch additions. Five are
byte-identical; only the plan differs (branch has later checkpoint). On an
authorized merge the integrating agent should preserve all six in an ignored
hash-bound backup before moving the overlapping untracked files out of the way,
then merge normally. The user does not need to delete them manually. Preserve
all other untracked operator/review files. No merge/push authorized or done here.

## Next action

Claude completes the R5 navigation follow-up in the task plan: diagnose exact
narrow landing, repair, demonstrate failure on old bytes and pass on new bytes,
rebuild both package pairs and rerun final Node/browser/integrity/pin checks.
Return a fresh frozen packet for Codex re-review. Preserve native receipts if
native inputs remain exact; rerun native gates if those inputs change.
