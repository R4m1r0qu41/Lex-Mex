# Four-instrument Reader release — implementation receipt and publication packet

> **Superseded in part, 2026-10-22.** The R0–R4 continuation (`docs/four-instrument-reader-continuation-receipt-2026-10-22.md`) found the active-article tracking defect in this package's `0b797aa6…` UI and repaired it. It cleared the blanket LIC hold for the audited bytes and replaced the publication/rollback proposal below with a full pin delta. This file remains the M0–M4 record; its ready package and pin delta are no longer the proposal.

2026-10-22. Executed from `docs/plans/four-instrument-reader-design-release.md`
(original SHA-256 `d067c29da94c1e4a72625b667a282bdde7eef2a1c23980105f366547dcbdb847`).
**Nothing is committed, pushed, merged, staged to a host or deployed.**
Machine-readable evidence: `docs/four-instrument-reader-evidence-2026-10-22.json`;
baseline comparison: `docs/four-instrument-reader-baseline-comparison-2026-10-22.json`.

Review tiers: Claude Sonnet 5.5 wrote the code and owns scope and diagnosis; the
mechanical (Haiku) tier ran the deterministic native gates; the Opus 5.5 advisor
was consulted four times (see Consultations) by reading this same transcript,
so it is **not** an independent check. No Codex review of this work exists yet; in
particular the `main.rs` conflict resolution below has had no cross-provider review
(operator option before commit or merge). Direct evidence for that merge: the validators ran the integrated binary against a clean clone of committed main and rewrote the LRITF and IFPE validation output byte-identically (`git status` clean), so the `validate_corpus_snapshot` split preserved behaviour for both.

## Outcome

A reviewable local package implementing approved Revision 02 (Figma
`ar2bES9p6jeUBY0w3E4eDY`, board `160:570`) for exactly `lritf, ifpe-dcg-2021,
itf-dcg-2018, lic`, built over the **retained** release.

| | |
|---|---|
| Ready publication | `0b797aa6d57e744a3e27a5cedda62db926e3c6dc36fa560fae1ecca874b9a09d` |
| Release (unchanged legal content) | `3c04016651e2fd4169632756e96ec19136136e1f42c4ed666070f23f20b30660`, corpus commit `9e990e09995718a77865993b24c5f8c8e7167b42` |
| Package root (ignored) | `/Users/jr/Dev/lex-mex-reader-four/.work/ship-a` — 1784 files, built twice (`ship-a`, `ship-b`) byte-identical; these are the bytes the browser checks ran on |
| Worktree / branch | `/Users/jr/Dev/lex-mex-reader-four`, `feat/four-instrument-reader-design` at accepted main `85c41f389`, uncommitted |

Only four public UI files differ from what is deployed (`index.html`, `reader.css`,
`reader.js`, `reader-core.mjs`); no release file differs. The deployed UI was
reproduced exactly: the unmodified committed packager over the retained release gives
`407aa3dc…`, the host pin, so Reader HEAD `f5330c3e9` is the deployed UI and the rollback base.

## Baseline decision: ready option vs held option

I built the same four instruments from accepted main (`85c41f389`) with the integrated
exporter: release `9626bfe45c4ebc95b47351fd20c1de86f9b4f54856aee1de6d408e5d8389991f`
(two builds identical). Per-provision comparison with the retained release:

- **LRITF:** provisions and references identical. **IFPE, ITF:** provisions identical (text, temporal, review, source digests); references differ only by added resolved links (IFPE +3/−1, ITF +5).
- **LIC: public legal text differs.** 3 new provisions (112 Quintus, Sextus, Septimus, previously inside 112 Quater) and 26 text changes: 15 paragraph-break normalisations, 8 leaked chapter/apartado headings removed (109-bis-8, 110-bis-1, 110-bis-14, 150, 155, 220, 224, 45-n), 112 Quater split, a signature block removed from the vigésimo transitory, and 108 Bis (a truncated editorial-note tail restored: "Artículo Décimo Primero del citado Decreto."). Heading context changed on 248 provisions (124 chapter, 124 apartado). References: LIC +129/−41.
- Effective dates, review status, temporal status, repeals, conditions and source digests are identical for every shared provision. Every `pdf_page` is null in both releases (867/867 and 870/870).

These match the accepted heading-leak repairs, but they are public legal-text changes.
Following the plan, **the ready option keeps the retained release** and the baseline
update is a **separate held option** needing operator/JRH review. No approval is inferred.

## What was built

- **Integration (transferred, not merged):** the `public-release` exporter (`crates/lex-cli/src/public_release.rs`; three visibility edits in `bundle.rs`; a `PublicRelease` command and a `validate_corpus_snapshot` split in `main.rs`, **resolved by hand against current main's repeal-evidence, parent-rule and glossary checks, so public-release builds now run them**), two schemas, `Cargo.toml` (+`jsonschema`, `tempfile` as normal dependencies), a one-line `Cargo.lock` change, and the committed `reader/` files from Reader `f5330c3e9`. The path manifest with source blob ids and hashes is in the evidence JSON. The permanent Reader's dirty files (agent pages, query preview, documents, deletions) were not copied and their hashes are unchanged.
- **Reader (M2):** one continuous scroll per instrument of real `<article>` elements (native find, selection, anchors); `content-visibility: auto` with per-article height estimates; chunked fill with the deep-link chunk first; active tracking by `IntersectionObserver` against a reading line; independent index with roving keyboard focus; sticky header (with the Índice/Fuente buttons on narrow screens) holding *Copiar artículo* (exact `provision.text`), *Copiar cita*, *Copiar enlace*; search over release JSON with per-provision counts; a secondary reference view whose *Volver* restores scroll and focus; sheets for index and source below 1100px.
- **PDF pane:** following is **unavailable** (no verified anchors; a page is never inferred), disabled and labelled. The open PDF keeps its page and zoom while the reader scrolls; only deliberate navigation (index, search, reference, instrument, entry, back/forward) swaps documents. On passive scroll into a provision with another source (IFPE and ITF annexes each have their own PDF) the reader sees an "Abrir PDF de esta disposición" offer; source lines, digest and *Verificar copia PDF* always describe the loaded document.
- **Exclusions:** the experimental `agents/<id>/` generation and the query preview/service are not in this publication path. Allowlists accept only the listed files, with regressions that fail closed. Discovery is the release's own `llms.txt` and canonical Markdown/text.
- **Public field allowlist:** the exporter validates every produced instrument and manifest against `schemas/public-instrument-v1.schema.json` and `schemas/public-release-v1.schema.json` (both `additionalProperties: false`) at build time (`validate_schema`, `public_release.rs`), and the test `release_is_reproducible_exact_and_excludes_review_artifacts` covers review-artifact exclusion. The denylist scan below only supplements this.

## Consultations

| When | Question | What changed |
|---|---|---|
| After M1, before M2 | LIC disposition; continuous-reader architecture | Confirmed the retained-release disposition; adopted render-all with `content-visibility`; added deployed-UI reproduction, a transfer manifest and the comparison artifact |
| Before completion | Final readiness | Found PDF reload on passive scroll, browser checks not run on shipped bytes, a mislabelled 1440 screenshot, missing allowlist evidence and continuity steps; all fixed and re-verified on `ship-a` |
| After the Stop-hook objection | Final package and packet claims | Found that the backup command would have been published by the host; backup moved outside any served tree |
| Final sign-off | Packet wording and backup location | Signed off; required this wording to match the table and the backup path to leave the Git tree at `/Users/jr/Documents/4ll4` (now under the ignored `.work/`) |

The LIC legal-text difference was not treated as an advisor trigger because the plan already prescribes the disposition; it is held for you and JRH.

## Verification evidence

Repository-required checks (`AGENTS.md` → Required checks) were run, with `--locked` added; the two validators ran in a disposable clone because they write.

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo test --locked --workspace` | exit 0; 424 passed, 0 failed, 0 ignored (main baseline was 419) |
| `cargo build --locked -p lex-cli` | exit 0 |
| `node --test reader/` | 22 pass, 0 fail (rerun after the last Reader edit; the Haiku run counted 21 before two late changes) |
| LRITF / IFPE validators, disposable clone of `85c41f389` | valid (145 art/11 trans/126 refs; 59 art/4 trans/115 refs), 0 issues; clone `git status` clean |
| Rust gates vs later work | Rust code has not changed since the gate; only `reader/` and docs changed afterwards |
| Package twice (`ship-a`/`ship-b`) | identical |
| Receipt vs disk | 1785 = 1785; no forbidden paths; private-field denylist 0 hits |
| Source PDFs | 32 present, name = SHA-256, all referenced; 1734 provision files present; routes and `llms.txt` links 200; `agents/…` 404 |
| Fail closed (copies of `ship-a`) | tampered file, `agents/…` path and invalid release id refused; destination not written |
| Staging rehearsal (isolated dir) | deployed `407aa3dc` → `0b797aa6` → `407aa3dc`, 1784 files verified against the receipt each time |
| Browser suite on `ship-a` (Chrome, local server) | all four instruments load (71/132/508/156); LIC deep links to Art. 193 and the last transitory land 8px under the header (also at 390px); passive scroll adds 0 history entries and the active article equals the one at the reading line; index clicks land at +8px; back/forward restore (273 → 27 Bis 5 → forward 273); comparison leaves origin article and scroll untouched and *Volver* restores scroll and focus; copy equals JSON; DOM text equals JSON for all 508 LIC provisions; arrow-key index focus; search 49 hits / empty state; PDF verification; IFPE annex passive scroll keeps the PDF and shows the offer, accepting it swaps and re-verifies; no console errors; no horizontal overflow |

Screenshots (ignored root `/Users/jr/Dev/lex-mex-reader-four/.work/evidence/screenshots`): desktop at a true 1440px viewport (scaled iframe), narrow 528px reading/index sheet/source sheet, 390px deep links.

Measurements (indicative, no budget claimed): three clean LIC loads of the shipped bytes gave first chunk 139–148 ms and fill complete 301–320 ms (10,415 DOM nodes); an instrumented copy of the same code measured 332 ms and the previous build 318 ms; relayout ≈2.5 ms with `content-visibility` vs 74–86 ms without it. **Unexplained outliers:** three other loads, made right after heavy tab activity or a window resize, took 3.9–5.3 s to complete; I could not attribute them (timer throttling suspected) and did not reproduce them on clean loads. Treat load time as variable.

Defects found and fixed during verification: layout-estimate drift on index jumps (height estimate plus settle loop); a race where the observer could reassign the selection mid-settle and strand a deep link on a neighbouring article (settling and loading now suppress passive activation); the PDF pane reloading while scrolling through annexes; the sidebar index starved of height; the Índice/Fuente buttons scrolling away (now sticky).

## Holds and limitations

1. **Held:** baseline update to `9626bfe4…` (LIC legal-text changes above) — operator/JRH review required; it produces a *different* release id and a new pin.
2. **PDF follow unavailable:** no verified article-to-page anchors; enabling it needs anchors built and verified against retained PDF hashes.
3. Not tested: real touch/wheel gestures (scrolling was driven by script plus synthetic wheel events), real phones, Safari/Firefox; 390px was checked in an iframe and 528px in a window.
4. `history.length` readings saturated at Chrome's 50-entry cap in a long-lived test tab, so push counts were confirmed through back/forward behaviour there (earlier clean-tab runs showed +1 per index click and +0 for scroll).
5. Pre-existing and unchanged: 3 IFPE references are attached to the instrument rather than a provision and appear nowhere; a provision link absent from the retained release (e.g. 112 Quintus) falls back to the instrument's first provision; the About text still calls agent search tools "en desarrollo activo"; favicon links are host-absolute paths.
6. The integration branch is uncommitted; merging it puts the exporter on main (needs your authorization). The permanent Reader branch must not be merged wholesale.
7. Ignored artifact roots (`.work/ship-a`, `.work/baseline-unmodified-ui`, evidence, screenshots) must not be cleaned until after publication.

## Publication and rollback proposal (not run; needs separate authorization)

Host facts (read-only): the pin `/Users/jr/Documents/4ll4/Webpage/LEG4L_L4BS/lex-mex-release.json` is `407aa3dc…`; the host's `npm run build` runs `scripts/verify-lex-mex.mjs`, which compares `publication_id` and `release_id` with the staged `public/lex-mex/publication.json`. No host test depends on the Reader's element ids.

1. Commit the integration branch (operator authorization): exporter, schemas, `reader/`, docs.
2. Back up both destinations **outside every served or built tree and outside any Git-tracked folder** (a backup under `public/` would be built and published by the host, and `/Users/jr/Documents/4ll4` is a Git tree where it would appear as untracked files; `.work/` is ignored):
   ```sh
   mkdir -p /Users/jr/Dev/lex-mex-reader-four/.work/host-backup-2026-10-22
   cp -R /Users/jr/Documents/4ll4/Webpage/Lex-Mex/public /Users/jr/Dev/lex-mex-reader-four/.work/host-backup-2026-10-22/Lex-Mex-public
   cp -R /Users/jr/Documents/4ll4/Webpage/LEG4L_L4BS/public/lex-mex /Users/jr/Dev/lex-mex-reader-four/.work/host-backup-2026-10-22/LEG4L-public-lex-mex
   ```
3. Stage:
   ```sh
   cd /Users/jr/Dev/lex-mex-reader-four
   node reader/stage.mjs --publication /Users/jr/Dev/lex-mex-reader-four/.work/ship-a --destination /Users/jr/Documents/4ll4/Webpage/Lex-Mex/public
   node reader/stage.mjs --publication /Users/jr/Documents/4ll4/Webpage/Lex-Mex/public --destination /Users/jr/Documents/4ll4/Webpage/LEG4L_L4BS/public/lex-mex
   ```
4. **Pin delta** in `lex-mex-release.json`: only `publication_id` changes, `407aa3dc658a274ac712e9494045a4c97724e5df0686de4e0513bf1af039470f` → `0b797aa6d57e744a3e27a5cedda62db926e3c6dc36fa560fae1ecca874b9a09d` (`release_id`, `corpus_commit`, `manifest_sha256` unchanged).
5. Host `npm test` and `npm run build`; deploy per the host's own procedure; check live routes under `/lex-mex/`.

**Rollback:** the release directory is identical, so rollback only reverts UI root files and the pin. Stage the deployed publication back (rehearsed):
```sh
cd /Users/jr/Dev/lex-mex-reader-four
node reader/stage.mjs --publication /Users/jr/Dev/lex-mex-reader-four/.work/baseline-unmodified-ui --destination /Users/jr/Documents/4ll4/Webpage/Lex-Mex/public
node reader/stage.mjs --publication /Users/jr/Documents/4ll4/Webpage/Lex-Mex/public --destination /Users/jr/Documents/4ll4/Webpage/LEG4L_L4BS/public/lex-mex
```
(or restore the two directories saved under `/Users/jr/Dev/lex-mex-reader-four/.work/host-backup-2026-10-22/`), set `publication_id` back to `407aa3dc…`, rebuild and redeploy. The baseline package was produced by the unmodified committed packager and reproduces the deployed id.

## Decisions requested

1. Authorize committing the integration branch (and, separately, its merge route to main); optionally request a Codex review of the `main.rs` merge first.
2. Authorize host staging, the pin change and deployment of `0b797aa6…`.
3. Rule on the held LIC baseline update (and JRH review of the legal-text changes).
4. Prioritise PDF page-anchor verification if PDF following is wanted.

This receipt is not a legal determination, and an advisor review does not substitute for JRH or operator approval.
