# Reader R6 — `/code-review` findings for Codex, 2026-10-22

Requested by the operator while Codex ran the host `npm test` / `npm run build` on the staged preferred package.
Reviewer: a Claude `/code-review` fork (read-through of the diff and surrounding code; it did **not** run the browser
check or reproduce anything). Triage notes are by Claude Sonnet 5.5, the R6 implementer, so they are not independent;
Codex should judge each item itself.

**Scope reviewed:** `git diff ca58f5879..256d1b410 -- reader/reader.js reader/browser-check.mjs` on branch
`feat/four-instrument-reader-design` (current HEAD `57016e627`, docs-only on top). `reader/reader.js` SHA-256
`68344a3001803e3282c135259383d07f1ad3225f7e934c5f71476d4b2879a28a`; `reader/browser-check.mjs`
`954951a3a292af81eda17304e9ba076574edf0004d24c7e5d8d64fd8f89f35f9`. Staged preferred package
`8eb8d8b4fcfd09c7b9414263494789207de38ea1968b01103fbb010df2b65a6f`.

**Status:** nothing was changed in response. Any `reader.js` edit would void the package hash, the host staging and the
R6 browser reports and needs a new review round. No finding below claims to reverse the R6 verdict; items 1 and 2 are the
ones worth a ruling.

## Findings that need a ruling

### F1 — P3: selection can lag after the terminal unhold while articles are still loading (`reader/reader.js:194`, `:208`)
After `unhold()`, `userScrolled` stays false. `track()` returns early on `(loading&&!userScrolled)` (line 208), and with
`held===null` `readerMoved()` no longer detects scrollbar/find-in-page/AT movement, so nothing sets `userScrolled`. If the
terminal branch fires while `fillAll` is still running, label/index/URL/copy stay on the requested article while the reader
sees other text, and the notice says the opposite until loading ends.
- *Claude triage:* real but narrow. Reaching the terminal branch needs 30 recovery chains (≥ ~20 s of misalignment) while
  `loading` is still true; `fillAll` normally finishes in seconds, and it ends with `schedule()` (line 133), so tracking
  resumes then. The failure Codex reproduced (post-fill) is unaffected. Not reproduced.
- *Question for Codex:* accept as a bounded edge, or require the terminal branch to clear the loading gate (for example by
  setting `userScrolled` or scheduling a pass independent of `loading`)?

### F2 — design: `recoveries` is a lifetime counter, so a very long hold can end (`reader/reader.js:190–194`)
`aligned` resets `realigns` and `nudges` (line 190) but not `recoveries`. A long-lived hold that survives about 30
budget-exhausting layout bursts and is misaligned at the 31st gets unheld. The new header comment says "A healthy aligned
hold never expires", which is true at any instant it is aligned but reads stronger than the lifetime cap.
- *Claude triage:* matches the R5 review's wording ("lifetime recovery allowance … exhausted and the target remains
  misaligned"); the terminal branch fires only when misaligned. Resetting `recoveries` on alignment would be a design change
  (it removes the bound on repeated recovery loops over a very long hold), not a bug fix. Comment wording could be
  tightened ("an aligned hold is never ended by time; only a spent lifetime allowance while misaligned ends it").
- *Question for Codex:* keep the lifetime cap as reviewed, or reset `recoveries` when aligned?

## Already disclosed (no new information)

- **F3 — scenario 27 injects the spent counters** (`reader/browser-check.mjs:300–`): the 40/400/30 counting logic is not
  exercised; nothing drives 30 natural recoveries. Disclosed in the R6 plan section and in the R6 review (natural frequency
  unmeasured; counter reset by source inspection).
- **F4 — the notice is a 5.5 s replaceable toast** (`reader.js:194`, `message()`): a missed notice leaves a reader with a
  different article than the link requested, and the original target is not one click away. Disclosed; Codex's R6 review
  accepted it because correctness follows from releasing the hold.

## Minor harness points (no action proposed)

- **F5:** the breakpoint line comes from `lines.findIndex(line=>line.startsWith('function keepHeld(){'))+1`
  (`browser-check.mjs:311`). A reformat or rename makes `findIndex` return -1, the breakpoint lands on line 0 and the test
  fails only after a 10 s generic timeout ("keepHeld breakpoint never paused"). The `scriptParsed` regex
  `/\/reader\.js$/` misses a URL with a query string; listeners and `Debugger.enable` are not undone on the failure path.
- **F6:** the re-navigation wait ends in `.catch(()=>{})` (`:327`) and the scenario uses fixed sleeps (2000/700/1800 ms).
  The final assertions still require label 193, reading-line 193, offset ≈ 8 px and zero idle scrolls, so a missed re-hold
  would fail the scenario; the swallowed timeout only loses the diagnostic. Same pattern exists elsewhere in the harness
  (`:235`, `:347`).
- **F7:** scenario 27 sits between 26 and 25 in the file; scenario 26 asserts nothing about the terminal outcome, so a
  future change that makes a natural storm cross the cap would turn 26 red without pointing at the new behaviour.

## Appendix — first `/code-review` run reviewed the wrong tree (out of R6 scope)

The first run covered the **uncommitted `feat/public-reader` work** in `/Users/jr/Dev/lex-mex-reader`, not R6. Its findings
concern files that are not in the R6 commits and are listed only so they are not lost; nothing was changed. They were not
reproduced (read-through only; the reviewer ran Node 29/29 and `cargo test --locked -p lex-core public_query` 3/3).
- `reader/agent-pages.mjs:74` exact text in a bare `<pre>` loses a leading newline and normalizes `\r\n`, so a parsing agent
  can see bytes that differ from the SHA-256 shown (`query-http.mjs` already has a workaround with tests).
- `reader/query-memory.mjs:8` a cursor that decodes to JSON `null` throws and is reported as 503 instead of 400.
- `reader/query-memory.mjs:17–18` a query that normalizes to empty matches every provision; search normalizes full texts
  inside the sort comparator and builds an excerpt for every match before paginating.
- `reader/agent-pages.mjs:12` `safeUrl` accepts `http://` source links while the public query contract is https-only.
- `reader/agent-pages.mjs:42` one static page per distinct corpus word, with no cap on term files.
- `reader/stage.mjs:27` any `agents/<64-hex>/…` path is accepted without tying the id to the release.
- `reader/package.mjs:28` dead read of `llms.txt`; `reader/README.md:33` contradicts the generated `llms.txt`.
- `crates/lex-core/src/public_query.rs:138` schema re-parsed and recompiled on every `validate_response` call.
- The working tree there also shows `nota-viabilidad-comites-banca-multiple.{md,docx}` as deleted (pre-existing, unrelated).

## Next action

Codex rules on F1 and F2 (accept as-is, or request a repair round with a new package, new browser reports and a new
staging). All other items need no action for R6. Commit, deployment and host-repo decisions remain with the operator.
