# Four-instrument Reader — R0–R4 continuation receipt and corrected publication packet

2026-10-22. Executes the R0–R4 continuation of
`docs/plans/four-instrument-reader-design-release.md` (received from Codex at SHA-256
`14ffd7f6e9de821ac2e434c992c8bcef8ba604732d03bf710e9307da72be974a`). It supersedes the
publication proposal and the blanket LIC hold in
`docs/four-instrument-reader-implementation-receipt-2026-10-22.md`, which stays as the M0–M4
record. Machine-readable evidence: `docs/four-instrument-reader-continuation-evidence-2026-10-22.json`.
**Nothing is staged to a host or deployed. Nothing is pushed or merged.** The operator authorized
local staged commits of the finished work on this branch. The commit sequence is listed in the Codex re-review handoff.

Review tiers: this continuation was executed by **Claude Opus 5.5**. The handoff assigned execution
to Sonnet 5.5, so this is a routing deviation, recorded here rather than hidden. The Opus advisor was
consulted once before R1. It reads this session's transcript, so it is not independent. The R4 review
came from a separate Opus verifier agent that received a bounded file packet. It is not a Codex review.
Codex re-review is requested through the provider-handoff queue. No review here grants publication or
legal approval.

## Outcome

| Option | Publication | Release | Provisions | Status |
| --- | --- | --- | --- | --- |
| **Preferred** | `f5a1937aa7304375508a11dad427c6fea4d43ad74c567fb53e2fe7e85df00acf` | `9626bfe45c4ebc95b47351fd20c1de86f9b4f54856aee1de6d408e5d8389991f` (corpus `85c41f389`) | 870 (LIC 511, IFPE 71, ITF 132, LRITF 156) | ready for operator/Codex review |
| Fallback | `709b09c401f3f4176104def965d40a2b70a0114d0d1d38d6147b88db4566c004` | `3c04016651e2fd4169632756e96ec19136136e1f42c4ed666070f23f20b30660` (corpus `9e990e09`) | 867 | ready; same fixed UI over retained content |
| Rollback | `407aa3dc658a274ac712e9494045a4c97724e5df0686de4e0513bf1af039470f` | `3c04…` | 867 | currently deployed; reproduced by `.work/baseline-unmodified-ui` |

Packages (ignored): `.work/r4-preferred-a` = `-b`, `.work/r4-fallback-a` = `-b` (byte-identical pairs).
The earlier ready publication `0b797aa6…` (`.work/ship-a`) is superseded. It carries the
active-tracking defect. The first continuation build (`.work/r3-*`, publications `cb2212ff…` and
`422570a5…`) is also superseded. The R4 review found a tall-viewport defect in it, and only
`reader.css` changed afterwards.

## R0 — baseline

- Main `85c41f389`, worktree and branch confirmed, no other writer process. The review inputs were
  copied from the main workspace with identical hashes. The main-workspace plan stays at `14ffd7f…`;
  only this worktree's copy carries the new checkpoint.
- Rust, schema and Cargo inputs are byte-identical to the state that passed 424 tests. The hashes
  are bound in the evidence (`r0.native_input_sha256`). The repository's required checks were
  rerun before committing, by the mechanical tier plus one direct validator run. All exit 0:
  - `cargo fmt --all -- --check`;
  - `cargo clippy --locked --workspace --all-targets -- -D warnings`;
  - `cargo test --locked --workspace`: 424 passed, 0 failed, 0 ignored;
  - `cargo build --locked -p lex-cli`.

  The validators ran with the integrated binary (`.work/target/debug/lex-mex`) in a disposable
  clone of `85c41f389`. LRITF: valid, 145 articles, 11 transitories, 126 references, 0 issues.
  IFPE: valid, 59 articles, 4 transitories, 115 references, 0 issues. The clone's `git status`
  stayed clean.
- `release-main-a` = `release-main-b`. Its `builder_sha256` equals the current `public_release.rs`
  hash, all canonical input hashes match `85c41f389`, and all 1782 manifest files verify. It was
  reused without a rebuild.
- LIC public provisions match canonical `corpus/mx/lic/provisions.json` at `85c41f389`: 511 = 511,
  same IDs, same order, 0 text mismatches.
- `ship-a`/`ship-b`, `baseline-unmodified-ui` and `release-main-a/b` were not modified.

## R1 — active selection (Codex required finding)

**Reproduced on the old shipped bytes** (`ship-a`, real headless Chrome, both programmatic scrolling
and trusted wheel events). LIC Art. 121→122, reading line 107px:
- The observer membership stayed exactly {121, 122} at every step.
- After Art. 122's heading crossed the line, the label, index, title and replaced URL still said
  Art. 121.
- *Copiar artículo* copied Art. 121's body, and *Copiar enlace* produced a link to Art. 121.

The same harness also fails on the old bytes for three more cases:
- the long-article tail;
- an interrupted deep-link settle (stranded on 261 while 260 was at the line);
- the IFPE annex PDF offer: the label was stuck on CUARTO with ANEXO 1 at the line, so no offer
  appeared.

The final harness, which adds the R4 scenarios, fails 7 of 25 applicable scenarios on `ship-a`.
The two additional failures are a wheel that ends inside the navigation lock (Art. 29 Bis 3 stays
selected and copied while 29 Bis 4 is at the line) and the LRITF last-provision deep link at
1440×3000.

**Repair** (`reader/reader.js`, `reader/reader-core.mjs`, plus one `reader/reader.css` line added
after R4; no packager or stager change):
- **Scheduling.** One requestAnimationFrame pass (`schedule`/`track`) is fed by scroll on the reading
  column, IntersectionObserver callbacks, window resize, the 1100px breakpoint change and each fill
  chunk.
- **Selection.** The active article comes from a binary search over article tops (`activeIndex`),
  not from observer membership. It reads about 10 positions per pass.
- **Trailing passes.** A pass blocked by the navigation lock retries when the lock expires. Settle
  end and loading end each schedule a pass. Pending frames and timers are cancelled when the flow is
  rebuilt.
- **Spacer (added after R4 finding 2).** `.flow` trailing space grew from `70vh` to `100dvh`. With
  `70vh`, a last provision could not reach the reading line on tall viewports. The first
  continuation build then let the new trailing pass move the selection off an explicit deep link and
  rewrite its URL. Reproduced at 1440×3000: LIC VIGESIMO became DECIMONOVENO, and LRITF DÉCIMA
  PRIMERA became DÉCIMA. The old bytes also left the LRITF target short of the line. With at least
  one full viewport of trailing space, any article can reach the line.
- **Unchanged.** Passive history still uses replace (250ms). Deliberate navigation still pushes. PDF
  following stays unavailable.

**Regression:** `reader/browser-check.mjs` is a zero-dependency DevTools-protocol harness. It is
deliberately outside `node --test`, because it needs Chrome. Truth is computed from layout alone,
and copy output is captured without modifying the shipped bytes. Scenarios:
- constant-membership crossing (scrollTop and wheel), with a geometric precondition;
- short adjacent articles and a long article tail;
- resize across 1100px;
- six deep links, including 112 Quintus, Sextus and Séptimus, the last transitory and a 390px layout;
- a wheel during the navigation lock;
- an index push that is not overwritten, and back;
- an interrupted settle;
- two quick instrument switches (exactly 2 pushes);
- search, the reference view and *Volver*;
- the PDF offer with no auto-follow;
- added after R4: a wheel that ends inside the navigation lock with no later event, and last-provision
  deep links for all four instruments at 1440×3000 and 390×844.

Results on the final bytes:

| Package | Passed | Failed | Not applicable |
| --- | --- | --- | --- |
| Preferred (two runs) | 28 | 0 | — |
| Fallback | 25 | 0 | 3 (the 112 articles are absent from that release) |
| Old `ship-a` | 18 | 7 | 3 |

**Mutation run** (R4 finding 1). Each new scheduling hook was removed in turn from a copy of the
tested package, and the harness was run on that copy.

| Hook removed | Scenarios failing |
| --- | --- |
| Scroll listener | 4 |
| Lock-expiry timer | 2 (the interrupted settle and the wheel ending inside the lock) |
| Resize listener, breakpoint pass, settle-end pass, loading-end pass, chunk pass, observer feed | 0 |

So these last six hooks are not independently demonstrated. A second run disabled scroll anchoring
(`--no-scroll-anchoring`, simulating browsers without it). It produced no failure attributable to
any of the six; the only failures were intermittent deep-link landing offsets, which also affect
the unmutated build and the old bytes. The six hooks are kept as defense in depth: the plan
requires trailing passes when settle and loading end, and Codex suggested keeping the observer.
They are not claimed as tested.

**Disabled anchoring, disclosed:** deep links can land 88–190px past the bar (LRITF Art. 4, LIC 112
Quintus). This also happens on the old bytes. The selection, label, URL and copy remain correct.
Real Safari, which historically lacks scroll anchoring, was not tested.

`node --test reader/` passes 22/22, and `pickActive` was replaced by `activeIndex` with a
position-only test.

## R2 — preferred LIC candidate

Full file-level comparison of 3c04 → 9626 (`r2` in the evidence): 1777 → 1783 files, 1662
identical, 115 changed, 6 added, 0 removed. Every change is classified:

- **LIC text:** exactly the 26 audited IDs changed (15 whitespace, 8 leaked headings, 112 Quáter
  split, 108 Bis tail, Vigésimo signature). Exactly the 3 audited IDs were added and none removed.
  Both public file hashes equal the audit's `old_/new_public_file_sha256`. Heading contexts changed
  on 248 provisions, matching the audit. No other shared field changed apart from the derived
  `text_sha256` of those 26.
- **Recovered articles:** 112 Quáter no longer contains the three bodies, and each body was present
  in the old 112 Quáter. 108 Bis now contains `Artículo Décimo Primero del citado Decreto.`. The new
  articles keep `effective` / `machine_accepted` with null `effective_from` and `effective_to`. No
  date or review status was invented.
- **Other changed files:** 57 provision Markdown files changed without a body change. All 57 are
  reference-list lines:
  - 56 add outside-selection references;
  - 1 retargets IFPE Art. 46 "9" from IFPE Art. 9 to ITF Art. 9. The source text reads "artículo 9 de
    las Disposiciones de carácter general aplicables a las Instituciones de Tecnología Financiera",
    and the change came from main's committed relink.

  The remaining changes are `index.md` (the three new links), `coverage.json` and `manifest.json`
  (input hashes and counts), `llms.txt` (the release ID) and the IFPE, ITF and LIC `instrument.json`
  files.
- **References:**

  | Instrument | Before → after | Change |
  | --- | --- | --- |
  | LIC | 1049 → 1137 | −41 are all from text-changed sources; +129 |
  | IFPE | 113 → 115 | 1 retarget |
  | ITF | 144 → 149 | — |
  | LRITF | 126 | unchanged |

  Span validation by code point, as the reader renders it: 0 mismatches in all four instruments, and
  0 inconsistent `target_published` flags. Resolved targets outside the selection are rendered
  unlinked: LIC 84, ITF 5, IFPE 2. LIC has 2 references with status
  `historical_superseded_reference_outdated` (Art. 187 → 28 Bis, Art. 194 → 31 bis). They render as
  unresolved. The 3 known IFPE references attached to the instrument rather than a provision still
  appear nowhere.
- **LRITF:** all 314 public files are byte-identical. The 32 source PDFs are identical, and each is
  named by its SHA-256.
- **Direct links:** links to 112 Quintus, Sextus and Séptimus land on those articles, with label,
  reading-line and URL asserted, in the preferred package. In the fallback package they fall back to
  the first LIC provision (a pre-existing behavior, disclosed).

No difference falls outside the audit evidence and the review's reference note, so nothing new is
held. No JRH ruling was sought or inferred. Classifications and dates are as recorded.

## R3 — verification of final bytes

| Check | Preferred | Fallback |
| --- | --- | --- |
| Built twice, identical | yes | yes |
| Receipt vs disk | 1791 = 1791 | 1785 = 1785 |
| Forbidden paths / denylist hits | 0 / 0 | 0 / 0 |
| PDFs (name = SHA-256, referenced) | 32 / 32 | 32 / 32 |
| Provision files missing | 0 of 1740 | 0 of 1734 |
| Routes and llms links 200; `agents/` 404 | yes | yes |
| Browser harness | 28 / 0 (two runs) | 25 / 0 (3 N/A) |

**Fail closed** (copies of the preferred package): six cases were tried, and each was refused with
the destination left unwritten:
- tampered `reader.js`;
- a tampered LIC provision;
- an `agents/` route with the receipt resealed;
- `review-queue.json` with the receipt resealed;
- an invalid release ID;
- a receipt digest mismatch.

**Rehearsal** (isolated `.work/r4-rehearsal`, with a copy of the host's own `verify-lex-mex.mjs`):

| Step | Result |
| --- | --- |
| Deployed → preferred | staging passes; the host verifier passes with the new pin and refuses the old pin |
| Stage-based rollback | the deployed publication verifies again with the old pin, but `releases/9626…` (1783 files) remains, so it is not an exact restore |
| Exact directory restore from backup | identical to the pre-state under `diff -r` and hash lists |
| Fallback, then stage-based rollback | identical to the pre-state |

LIC load timings (headless, indicative, no budget): flow complete in about 96–125ms in both packages.
The 3.9–5.3s outliers reported in the original receipt were not reproduced headless. They are
not explained.

Screenshots (`.work/evidence/screenshots-r4/`), for each package:
- desktop 1440: LIC 112 Quintus, the Art. 122 crossing, 108 Bis;
- mobile 390: Art. 193;
- narrow 900: IFPE.

## R4 — fresh review

A separate Opus verifier agent received only `.work/evidence/r4-review-packet.md`, the plan and the
Codex review. It did not see this transcript. Verdict: **partial — ready for Codex and operator
review, nothing blocking**. It found no wrong article, copy or URL at common sizes. Its findings and
how each was handled:

1. **Medium — several new paths had no test that fails on the old bytes.** Two scenarios were
   added: a wheel ending inside the lock, and last-provision deep links. A mutation run was also
   done. The scroll listener and the lock timer are now proven. The six hooks that survive are
   disclosed as defense in depth (see R1).
2. **Low to medium — a new path could rewrite an explicit navigation on tall viewports.** Confirmed
   at 1440×3000 and fixed with the `100dvh` spacer. Both packages were rebuilt and every check was
   rerun on the new bytes.
3. **Low — the non-destructive rollback is not a restore, and the host verifier cannot tell.** The
   exact directory restore is now the default rollback for the 870 option, and the receipt states
   that a verifier pass is not evidence of restoration.
4. **Low — the old receipt's proposal was stale and actionable.** A supersession banner was added
   to its top.
5. **Informational** — the IFPE retarget (asks Codex to confirm), duplicate outside-target lines in
   provision Markdown, the generic label for the 2 historical references, and the pre-existing
   `pointerdown`/`keydown` settle interruption. All are disclosed under holds.

The verifier ran no browser, tests or host build. Its tall-viewport threshold was a CSS estimate,
later confirmed by measurement here.

## Publication proposal (not run; needs separate operator authorization)

The host facts below were read read-only.
- The pin file is `/Users/jr/Documents/4ll4/Webpage/LEG4L_L4BS/lex-mex-release.json`.
- `prebuild` (`scripts/verify-lex-mex.mjs`) checks `publication_id`, `release_id`, the receipt
  digest and every file hash. `corpus_commit` and `manifest_sha256` are pinned but not checked.
- `scripts/prepare-lex-mex-pdfs.mjs` moves only the **current** receipt's PDFs behind the range
  Worker. After a release change, the older pinned release's PDFs under `public/lex-mex/releases/3c04…`
  stay in the static output as plain assets: about 15MB of duplicate bytes, served without the range
  handler. Pinned old links still resolve. This is a host behavior to accept or adjust; it was not
  changed here.

**Full pin delta, preferred** (generated from the final receipt; `.work/evidence/proposed-pins-r4/preferred_870.json`):

| Field | From | To |
| --- | --- | --- |
| `release_id` | `3c04016651e2fd4169632756e96ec19136136e1f42c4ed666070f23f20b30660` | `9626bfe45c4ebc95b47351fd20c1de86f9b4f54856aee1de6d408e5d8389991f` |
| `publication_id` | `407aa3dc658a274ac712e9494045a4c97724e5df0686de4e0513bf1af039470f` | `f5a1937aa7304375508a11dad427c6fea4d43ad74c567fb53e2fe7e85df00acf` |
| `corpus_commit` | `9e990e09995718a77865993b24c5f8c8e7167b42` | `85c41f3898d86737766564fa8d3fd8f6d4ae74a3` |
| `manifest_sha256` | `c2b22653d7a0b68edc1f29d2b2554fc45eb4a595f5be91f0b96868f2128aedc7` | `8fb340f2f249f40fc9474d7a035b0d06c3f6bc4b0cc975099f79c214a13b6713` |
| `schema_version` | unchanged | |

**Fallback delta:** only `publication_id` changes, `407aa3dc…` → `709b09c401f3f4176104def965d40a2b70a0114d0d1d38d6147b88db4566c004`.

Steps (preferred; replace the package path for the fallback):
1. Back up outside every served, built or Git tree, including the complete old pin:
   ```sh
   B=/Users/jr/Dev/lex-mex-reader-four/.work/host-backup-2026-10-22-r4; mkdir "$B"
   cp -R /Users/jr/Documents/4ll4/Webpage/Lex-Mex/public "$B/Lex-Mex-public"
   cp -R /Users/jr/Documents/4ll4/Webpage/LEG4L_L4BS/public/lex-mex "$B/LEG4L-public-lex-mex"
   cp /Users/jr/Documents/4ll4/Webpage/LEG4L_L4BS/lex-mex-release.json "$B/lex-mex-release.json"
   ```
2. Stage:
   ```sh
   cd /Users/jr/Dev/lex-mex-reader-four
   node reader/stage.mjs --publication .work/r4-preferred-a --destination /Users/jr/Documents/4ll4/Webpage/Lex-Mex/public
   node reader/stage.mjs --publication /Users/jr/Documents/4ll4/Webpage/Lex-Mex/public --destination /Users/jr/Documents/4ll4/Webpage/LEG4L_L4BS/public/lex-mex
   ```
3. Replace the pin file with `.work/evidence/proposed-pins-r4/preferred_870.json`, which is all four
   changed fields, not only `publication_id`.
4. Run the host's `npm test` and `npm run build`. Then deploy per host procedure and check
   `/lex-mex/`, a 112 Quintus link, a PDF range request and an old pinned 3c04 link.

**Rollback:**

| Option | Use when | Steps | Result |
| --- | --- | --- | --- |
| **Exact directory restore (default for the 870 option; rehearsed)** | the preferred option was deployed | replace both destination directories from the backup taken in step 1 and restore the backed-up pin file, then rebuild and redeploy | identical to the pre-state under `diff -r`, `latest.json` included; deletes the `releases/9626…` directory, so the operator must authorize that deletion when choosing rollback |
| Non-destructive (rehearsed; not a restore) | the deletion is not authorized | stage `.work/baseline-unmodified-ui` into both destinations and restore the backed-up pin | `latest.json`, the root files and the pin return to `407aa3dc…`, and the host verifier passes; `releases/9626…` (1783 files) stays published, `?release=9626…` still loads through the old UI, and the host's PDF step leaves its PDFs as plain static assets |
| Fallback option | the fallback was deployed | stage-based rollback | identical to the pre-state |

A passing host verifier is **not** evidence that a restore is complete. The verifier checks only
the files the current receipt lists.

## Holds and limitations

1. PDF following is unavailable: there are no verified article-to-page anchors.
2. The real-touch, Safari/Firefox and real-device checks were not run. The browser evidence is
   headless Chrome with synthetic trusted wheel events.
3. Pre-existing, unchanged:
   - the 3 IFPE references attached to the instrument appear nowhere;
   - unknown provision links fall back to the first provision;
   - About still says agent tools are "en desarrollo activo";
   - favicon links are host-absolute.
4. The Rust/Cargo/schema integration (`main.rs` validation split) has still had no Codex rerun; it
   remains Claude's 424-test evidence.
5. Host PDF preparation for older pinned releases: see the proposal above.
6. Disclosed for Codex confirmation: the IFPE Art. 46 "9" retarget (IFPE Art. 9 → ITF Art. 9). The
   consolidated review's count-only reference note cannot cover a retarget, so Codex should confirm
   it.
7. The 2 LIC `historical_superseded_reference_outdated` references display the generic label
   "Destino fuera de colección o sin resolver". Some provision Markdown reference lists name the same
   outside target twice, once per occurrence in the text.
8. Pre-existing: any `pointerdown` or `keydown` inside the reading column, including on the copy
   buttons, counts as reader scrolling, so it ends a pending deep-link settle early.

## Decisions requested

1. Choose the preferred (`f5a1937a…`, 870) or fallback (`709b09c4…`, 867) option for host staging, and
   authorize staging, the pin change and deployment.
2. Authorize merge and push of this branch, separately from the local commits.
3. Request a Codex re-review of this packet (handoff filed).

This receipt is not a legal determination and does not represent official law or current validity.
