# Four-instrument Reader implementation review — 2026-10-22

Verdict: package integrity and retained-content disposition pass; publication
readiness is held for the active-article tracking finding below. Reviewed
`/Users/jr/Dev/lex-mex-reader-four` against accepted main85c41f389, receipt,
focused exporter/CLI integration and Reader source. No implementation/host edit,
commit, push or deploy performed. Browser/Figma validation was not independently
repeated in this review; those remain Claude's reported evidence.

## Required fix: active article can lag the reading position

`reader/reader.js:105` observes article intersection against the upper half of
the scroll viewport with default threshold0. `observed` (line156) computes the
active article from current positions, but runs only on intersection changes.
The input listeners (line230) only set userScrolled; they do not schedule active
tracking on scroll. An article's heading crossing `barHeight()+24` does not
necessarily change either article's intersection state. Therefore the active
label, source selection, copy action and share URL can lag the visible article.

Concrete geometry: with the previous article still intersecting, move the next
heading from180px below the viewport top to80px, across a124px reading line.
Both articles can remain intersecting the observer's upper-half root for the
whole movement; no threshold crossing is required. The current code does not
recompute selection in that interval. Existing pickActive unit tests verify
the pure choice given positions, not whether real scrolling invokes it.
Callback suppression during settle/lock also has no guaranteed trailing update.
This is a source-level event-delivery defect; this review did not reproduce
it in a new browser session, so record browser evidence with the repair.

Suggested remedy: retain IntersectionObserver for candidate visibility, but
schedule a single requestAnimationFrame position/selection pass on real scroll,
with a trailing pass when settling/loading suppression ends. Do not add history
entries on passive movement or reload PDFs. Test short consecutive articles,
long article tails, reading-line crossings while intersection membership stays
constant, rapid jumps and manual interruption. Verify sticky label, exact copy
body, share URL, sidebar and PDF offer against the article at the reading line.
Rebuild both packages and rerun checks/screenshots on the final shipped bytes.

## Independently verified

Mechanical checks:22/22 Node tests; diff check; all ship-a/b receipt file hashes
and sizes; byte-identical packages;1,777 retained release files identical to
baseline-unmodified-ui; exactly four UI changes;15/15 transfer/source bindings.
The first sandbox Node run hit loopback EPERM; elevated rerun passed.
Raw receipts are copied into the companion evidence file. Full424-test Rust
suite and clean-clone validators are Claude evidence, not rerun by Codex.

Focused main.rs integration: validate_corpus_snapshot contains the previous
run_validate computation, including repeal/parent/glossary coverage checks;
run_validate still writes/prints its returned report. public-release calls the
read-only computation and fails on invalid reports. No missing check found in
that extraction. Public exporter uses explicit DTOs and closed schemas; retained
source/body identity and held LIC update are appropriate. No new legal approval.

Subsequent retained-source audit clears the technical LIC hold for the exact
current-main bytes; see [consolidated review](db-reader-lic-consolidated-review-2026-10-22.md).
Public deployment still requires operator authorization. PDF following correctly remains unavailable.
Real touch, other browsers and load-time outliers remain disclosed limits; do
not claim they were cleared by these Node/package checks. The rollback packet
and source identity are useful but cannot substitute for correct active copy.

Next action: Claude follows R0–R4 in the updated four-instrument execution plan: 
repair active tracking with an event-level/browser regression, prepare the
source-verified LIC candidate, produce new immutable package IDs and evidence,
then request re-review.
No commit/deploy authorization is granted by this technical review.
