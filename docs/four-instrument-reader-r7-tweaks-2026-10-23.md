# Reader R7 — operator UI tweaks, 2026-10-23

Executes the operator's tweak list on top of the deployed F1 Reader (`b9b8be012`, live publication `2df70a8a…`). Branch
`feat/reader-tweaks-r7` in worktree `/Users/jr/Dev/lex-mex-reader-five`, created from `b9b8be012`. The F1 worktree
(`lex-mex-reader-four`) and its uncommitted Codex files were not touched. Executor: Claude Sonnet 5.5. **Nothing is committed,
restaged to the host, pinned, deployed, pushed or merged.** Source and test changes are uncommitted in the worktree.

## What changed (Reader code only; no Rust, schema, corpus or release content)

| # | Tweak | Result |
| --- | --- | --- |
| 1 | Citation: abbreviated or full law name | `citation(document, provision, full)`; one remembered toggle "Ley: abreviada/completa", shared by Copiar cita in the reading bar and in the hover window. Full = `official_title`. |
| 2a | Reform date | **Deferred as decided.** Article reform notes stay inside the exact text (the history shows in the middle column). The instrument-level date stays visible in the right panel. See the review below. |
| 2b | Text size | Toggle "Texto: grande/pequeño" (small = 0.65, constant `textScales.small` in `reader-core.mjs`), remembered, scales article text and the hover text. Placeholder heights scale with it (`estimateMetrics`), and a toggle re-holds the article at the reading line through the existing `go()` path. |
| 3 | Hover window | Added **Copiar artículo** (exact body, `articleCopy`) beside Copiar cita; the cita follows the name toggle. |
| 4 | Right panel | PDF area larger (min 420px, takes the freed height); only "Última reforma" stays visible; publisher, retrieval date and periodic check are under a "Metadatos del documento" fold; "Verificar copia PDF", the digest and the follow-state note moved into "Proveniencia y verificación". All element ids kept. |
| 5a | Texto exacto garbled in Obsidian | A plain click opens the exact text in a new tab as a UTF-8 blob (`text/plain;charset=utf-8`); the raw `href` is kept for modified clicks and agents. Host header fix is noted below for the webpage repo. "Acceso para agentes" now reads "Para agentes e IA · llms.txt ↗" and opens in a new tab. |
| 5b | `.md` missing "Título · Capítulo" | Both exports (Descargar norma completa and the per-article Markdown) emit the heading path once per change of path, via `headingPath`/`provisionMarkdown`. The per-article file is now `## Título · Capítulo`, `### Artículo N`, then the exact text (previously text only). The Rust-made raw `.md` files in the release are unchanged (changing them would be a new release ID). |
| 6 | Left pane "Artículos ·" | Articles under a heading are grouped as "Título QUINTO · Capítulo I"; Transitorios, Anexos and headingless articles keep their name. |
| — | Heading order | The middle column showed "Capítulo I · Título QUINTO" because it followed JSON key order. One helper now fixes the hierarchy (título, capítulo, sección, apartado) for the middle column and both exports; the index keeps título and capítulo only, as asked. The order was checked against the parser: a new Sección clears the Apartado (`crates/lex-parse/src/dcg.rs:118`) and a new Capítulo clears both (`:110-111`), so Apartado nests inside Sección. |

## Reform-date review (the last-reform data point)

| Instrument | `latest_reform_date` | Check against the article texts |
| --- | --- | --- |
| LIC | 2025-11-14 | equals the latest `DOF dd-mm-yyyy` among the 511 article notes: **corroborated** |
| LRITF | 2025-11-14 | equals the latest note date among its 156 provisions: **corroborated** |
| ITF-DCG-2018 | 2026-07-06 | its article texts carry no `DOF dd-mm-yyyy` notes, so the value **cannot be corroborated from the release text**. It is derived by the ITF parser as the maximum of the reform dates it parsed (`crates/lex-parse/src/itf.rs`, around lines 401–405). Not checked against the source PDF here. |
| IFPE-DCG-2021 | none | shown as "no disponible"; no reform dates in the release |

Per-article reform history is inside each article's text and shows in the middle column; the instrument-level date is in the
right panel (and stays visible when the metadata fold is closed). No middle-column header was added, as the operator allowed.
The per-article field (structured reform notes, so copies can omit them and agents can query them) remains a Rust-side project.

## Note for the webpage repo (not edited)

The live host serves text files without a charset: `llms.txt` and every `.txt` as `text/plain`, every `.md` (including the
release `index.md`) as `text/markdown`. Browsers then guess Latin-1 (the `crÃ©dito` seen in Obsidian), and some HTTP clients
decode `text/*` without a charset as Latin-1 as well. Request: serve `text/plain; charset=utf-8` and `text/markdown;
charset=utf-8` for `/lex-mex/**` (JSON is already `application/json`). I could not confirm whether the Sites setup allows
setting headers. Until then the Reader's blob view covers the human click path only; raw URLs stay as served.

## The agent link

"Para agentes e IA · llms.txt ↗" opens `./llms.txt` in a new tab; `<link rel="describedby">` already pointed there. The live
`https://www.leg4ll4bs.com/lex-mex/llms.txt` returns 200 (`text/plain`). It is short: the pinned release ID, a note not to
bulk-load, a link to the release's per-instrument Markdown index and a link to the manifest. It is enough to route an agent
but does not describe the instruments; richer agent guidance belongs to the `feat/public-reader` workstream.

## Files and hashes

Source (uncommitted, SHA-256): `reader/reader.js` `a5255724b38fa9ebd07820dbeb041b017aa73e9a88086af932ad97f13b97f0f1`;
`reader/reader-core.mjs` `a62d62744c56bbd755e4a91c8f236baa8298eb386773149858fa4b1880c10585`; `reader/index.html`
`2fc36a084ded9a698678bc4d0af8f45c4624b1ce0221e20045921b35617f7426`; `reader/reader.css`
`20830544e2ff48b03f529d7ca010ef4b3b46563e8818196a96bfb1eafa336463`; `reader/browser-check.mjs`
`1633ee64990063a2b7a3b6649f3ccd9babda33d252e3a1621ce0c923f9702bc1`; `reader/reader.test.mjs`
`3a1c39ad46314415c856900e458978ac4b445424254548eab45581c06ce2e364`; `reader/README.md` (documentation only, not a packaged asset, so no package hash depends on it)
`444db1e3acdf35c9e0b8d53447a8417aefcf58259a98debb6c3e6e85ba473094`.

Packages (ignored; `.work/r7-*`, A=B byte-identical): preferred `85b4ce98757da6c75b859de2d1eaeded369630adbbbe2734fa74caedc42501b1`
over 9626 (1790 listed, 1791 on disk, 0 bad hashes); fallback `309f8d5f5bccfb097894c1746a91510f2fa031ebdfcf109a8171d7dd3802237e`
over 3c04 (1784/1785, 0 bad). Built from the F1 release inputs; versus F1 (`2df70a8a…`, `21d4e387…`) only `index.html`,
`reader.css`, `reader.js`, `reader-core.mjs` and `publication.json` differ. Release content, PDFs and manifests are unchanged.

## Evidence

- `node --test reader/`: 24/24 (22 existing plus a heading/Markdown test and a text-scale test; the citation and index
  group tests were extended; the one expectation that changed on purpose is the index group label without "Artículos").
- Full harness through `mech`, one pass each (wall time 01:09–01:35): preferred **60/0**, fallback **57/0** plus 3 N/A
  (the three Art. 112 articles are absent from 3c04), legacy 3c04 links on a preferred-staged directory **75/0**, preferred
  without scroll anchoring **60/0**. No run stalled. Reports `.work/evidence/r7-final-*.json`, SHA-256:
  preferred-1 `df9ee10d37095b5607f3e79d9fc241711ce5d0de02ce9ab42380f592929ec54b`; fallback-1
  `4f910f26dc2aa01bec11abb97a712d595d5527427d1561c12fd6255d35eccc27`; legacy-staged
  `7cc53e02963f5de677022af378b14cc23ca012c4845731fe636969a9dbe1aaa9`; preferred-noanchor
  `31b811211ead8a3d18ba1891f6d9896df2b6cf82209e0564263f9ba95c19a666`.
- New scenarios, passing in all four: **28** (citation toggle remembered across reload and shared with the hover window;
  hover Copiar artículo exact; heading path in middle column, index group and both Markdown exports; exact-text tab is
  `text/plain;charset=utf-8` and equals the provision text; verify inside the provenance fold, reform date visible, agent link
  in a new tab) and **29** (390px, fonts blocked, LIC 193: toggle small, reload, toggle large; label/index/URL/exact copy equal
  the reading-line article, 8px below the bar, no hold-exhausted or intent marks, zero scrolls while idle, font ratio 0.65 and 1.0).
  Scenarios 26 and 27 (resize storm and the terminal-cap fault injection) still pass.
- Screenshots at 1440, 1024 and 390px (desktop with the hover window and small text, tablet, mobile, mobile source sheet) were
  inspected. The first layout wrapped the toggles into a second row of full-size buttons; they were changed to a slim row of
  small text controls before the final packages.
- Control: the new harness on the F1 fallback package aborts at scenario 28 (`Cannot read properties of null (reading
  'click')` because the toggle button does not exist there) after 50 passes and writes no report. That shows the new scenarios
  need the new controls; it is an abort, not a recorded FAIL. Scenario 28 could be made to record a failure instead of
  throwing if that matters to the reviewer.

## Not done or not claimed

- No Rust, schema, corpus or release change; no Rust gates rerun. No mutation campaign, Safari/Firefox or real-device test.
- Small text is 0.65 of large, not exactly 50%; one constant to change.
- The blob view is not a shareable address; the raw `.txt` URL is unchanged and still depends on the host header.
- The reading-bar toggles share the bar with the copy buttons; the bar is one slim row taller.
- The windowed-reader spike was not started (it is a separate task).

Exactly one next action: Codex focused review of this R7 source/package/evidence packet, then, if the operator authorizes each
step, restage the selected package, change the pin and deploy with the host `npm test` and `npm run build`. Commit, restaging,
pin, deployment, push and merge are operator decisions not given for R7.

## Handoff for Codex focused review

Review target: the R7 commit on branch `feat/reader-tweaks-r7` (worktree `/Users/jr/Dev/lex-mex-reader-five`, parent
`b9b8be012`). The operator started this handoff; Claude Sonnet 5.5 is the implementer, so the notes above are not independent.
Scope is the Reader UI diff (`reader/*` and this receipt) and the two ignored R7 packages in `.work/r7-*`. No Rust, schema,
corpus or release content changed. Please do not edit source; findings go to a review document.

Suggested checks, in order of risk:

1. Reproduce package identity: rebuild A/B from the F1 release inputs and compare with preferred `85b4ce98…` and fallback
   `309f8d5f…`; confirm only `index.html`, `reader.css`, `reader.js`, `reader-core.mjs` and `publication.json` differ from F1.
2. Hold and scroll safety: `setSize` recomputes `containIntrinsicSize` and re-holds through `go(…,'resize')`. Does a size
   toggle during an active hold, during `fillAll`, or after a terminal unhold leave label, index, URL, exact copy and the
   physical reading line in agreement? Scenario 29 covers one path (LIC 193, 390px, fonts blocked) only.
3. Exports: `provisionMarkdown` and `headingPath` ordering (título, capítulo, sección, apartado), and that Copiar artículo
   and the exact-text view still equal the provision text byte for byte (no heading, no citation added).
4. Exact-text blob view: UTF-8 content type, `tab.opener=null`, 10-minute revoke, and that only a plain left click is
   intercepted (modified clicks and agents keep the raw `href`).
5. Preferences: `lex-mex:` localStorage reads and writes are wrapped and the page works with storage blocked.
6. Reform-date review above: confirm the ITF value is parser-derived and not corroborated, and that no wording implies more.
7. Harness: scenario 28 and 29 oracles; the control run aborts at scenario 28 rather than recording a FAIL.

Questions for Codex: (a) is small text at 0.65 acceptable or should it be exactly 0.5; (b) accept the `.md` per-article file
now carrying `## path` and `### Artículo N` headings; (c) any blocker before the operator considers restaging.

Not authorized and not done: restaging, pin change, host `npm test`/`npm run build`, deployment, push, merge.
