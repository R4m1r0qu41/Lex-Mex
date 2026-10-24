# Reader F1 repair — 2026-10-22

F1 repaired: after terminal unholding, passive selection follows the reading line while article filling is still pending. Ready for focused independent re-review on the new package hashes. Source changes are uncommitted; the host remains pinned to the old R6 package.

## Implementation

Removed the redundant `loading&&!userScrolled` suppression from `track()`. Existing settle-target and held-target guards continue to protect explicit navigation during filling. Once neither owns the selection, tracking may follow visible text. This does not manufacture reader intent, change the lifetime recovery cap, or expire an aligned hold. `fillAll()` already guards its final re-alignment by `held===target`, so an abandoned target is not restored on completion. `go()` synchronously installs the new settle/hold before a scheduled tracking frame can run.

The scenario27 regression now runs with filling complete and with the real `fillAll` promise paused at its first zero-delay yield. Other timer delays run normally. After terminal counter injection and synthetic padding it checks visible text against label/index/URL/exact copy before and after reader scrolling while filling remains pending. It resumes the real fill loop, verifies tracking/copy consistency with zero corrective scroll calls, then explicitly navigates back and requires a new settled mark and correct geometry/copy. The previously swallowed re-navigation timeout now fails the test.

## Verification

Parent-run isolated regression: both modes pass on the repaired preferred package; the unchanged R6 control passes post-fill but fails pending-fill with visible191/selection193. On the old control, resuming filling restores agreement, confirming the guard interaction. New tests pause real filling rather than assigning `loading=true`. Recovery counters and layout displacement are still injected; natural lifetime-cap frequency remains unmeasured.

Required commit gates also pass: `cargo fmt --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace` (424 tests across suites, all pass); `cargo run --locked -p lex-cli -- validate lritf` (valid, 0 issues); and the IFPE validator (valid, 0 issues). Node tests:22/22 pass. Full browser runs (fallback/legacy via mechanical agents; no-anchoring directly by parent):

| Configuration | Result |
|---|---|
| legacy | 73 passed, 0 failed, 0 N/A |
| fallback | 55 passed, 0 failed, 3 N/A |
| noanchor | 58 passed, 0 failed, 0 N/A |

Legacy run serves the preferred release and also checks retained3c04 links. Preferred without anchoring covers the full preferred suite with browser anchoring disabled. No Rust/schema/corpus source changed; required native gates above pass with canonical validators at 0 issues. Diff check passes. No Safari/Firefox or real-device testing added.

Preferred publication: `2df70a8ab0f43571ed6ca1500ff4637e4de252ee1edc190befeda8b938b8f1d4` (`.work/f1-preferred-a`, identical `-b`).
Fallback publication: `21d4e3870bb0baf70c3761451ea053909364204529dad899d82406991c5b720a` (`.work/f1-fallback-a`, identical `-b`).
Both descriptor IDs, all listed file hashes/sizes, file sets and A/B equality verify. Only `reader.js` and `publication.json` differ from R6. Immutable corpus data, PDFs and other UI assets are unchanged.

Source SHA-256:
- reader.js: `5a9f16e0fb71c498def651d29641928ac33c881fe5f36203af5077df514649d2`
- browser-check.mjs: `29e9a6c82e1cf3b8035cad2f0ef40c392cc076a613dd7a70f6a83766dc1085d6`

Full reports and reproducible focused runner are embedded in `four-instrument-reader-f1-repair-2026-10-22-evidence.json`.

Exactly one next action: focused independent re-review of this source/package/evidence packet before restaging the already-selected preferred option. Existing host pin commit7d61f95 remains historical R6; its npm results do not validate these new UI bytes. No new commit, host staging, pin change, push or deployment performed.
