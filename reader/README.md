# Public reader

Static Spanish reader and deterministic publication packager. Rust exports an explicitly allowlisted release; the browser displays the exact retained PDF alongside provision text. Canonical corpus, private review records, model proposals and server tooling are never publication inputs.

## Build

From the isolated reader worktree:

```sh
cargo run --locked -p lex-cli -- public-release build --instrument lritf,ifpe-dcg-2021,itf-dcg-2018,lic --source-store .work/reader-sources --output .work/public-release --base-path /lex-mex/
node reader/package.mjs --release .work/public-release --output .work/public-reader
node reader/serve.mjs --dir .work/public-reader --port 4173
```

Open `http://127.0.0.1:4173/lex-mex/`. Destinations must not already exist. The source store contains `<source SHA-256>.pdf` files; every retained PDF must match its committed manifest. Annexes use their separate official source manifests.

Only copy the completed publication directory to `Webpage/Lex-Mex/public`. This directory is a deployment artifact, not a Git worktree or editable source. The host must mount its contents at `/lex-mex/`, serve `index.html` for that path, preserve query parameters, and serve `.md`, `.txt`, `.json` and `.pdf` with the appropriate MIME types. Copying the folder alone does not deploy or connect the firm website.

## Contract and limits

The visual system follows LEG4L L4BS brand pages 00–05: cyan `#00C6E5`,
ink `#090909`, paper `#FAFAFA`, Space Grotesk headings/controls and Inter source
text. Fonts use the host's established Google Fonts loading convention, with
local sans-serif fallbacks. The canonical body remains exact; typography is a
presentation choice. Cyan controls use ink text, while temporal classifications
remain qualified and visually separate from navigation selection.

Editable design: [reader desktop](https://www.figma.com/design/ar2bES9p6jeUBY0w3E4eDY/?node-id=2-41),
[components](https://www.figma.com/design/ar2bES9p6jeUBY0w3E4eDY/?node-id=2-25).
UI-only changes keep the immutable corpus release ID but produce a new publication
ID; update the host's reviewed publication pin before building the new package.

`latest.json` resolves to an immutable release. Shared links pin release, instrument and provision. `llms.txt` leads agents to per-instrument Markdown indexes; exact provision bodies are available as `.txt`. Model HTTP reads do not change the human selection. This release does not implement an MCP server or desktop browser integration.

Every provision exposes its source digest and official URL. The PDF is retained and verified; its retrieval time is distinct from a last successful upstream freshness check, which is currently absent. PDF page mappings are intentionally unset. Article navigation therefore preserves the PDF document and current viewer position; annex selection switches to the annex PDF.

Coverage is partial. Repository temporal classifications are qualified, not legal advice or official law. Private reviewer identities, review rationales, condition evidence and transitory-effect proposals are excluded. The reader shows express references within the selected release; outside targets remain marked as unpublished.

## Continuous reading (design Revision 02)

The selected instrument is one scroll of real articles. Every provision is real DOM (native find, selection and anchors work); `content-visibility: auto` defers painting and bodies are filled in chunks, the chunk holding a deep link first. The active article follows the reading line (recomputed once per frame on scroll, resize, chunk fill and observer changes, and once more when a navigation settle or lock ends) and updates the sticky header, the index (`aria-current`) and the PDF-pane state; the index scrolls independently and never moves the text. Passive scrolling replaces the URL (no history entries); index choices, search results, instrument changes and reference follows push one entry; back/forward restore the article. "Copiar artículo" copies the complete canonical body from the release JSON, never a rendering; "Copiar cita" and "Copiar enlace" name the provision and pin the release. Search runs over the release JSON, so unrendered articles are found, and counts are per provision.

View options, exports and copying (R7). Two toggles in the reading bar are remembered in this browser (defaults apply when storage is unavailable): **Ley: abreviada/completa** chooses whether *Copiar cita*, in the reading bar and in the hover window, names the law by its abbreviation or its official title; **Texto: grande/pequeño** scales article text (small is 0.65 of large, `textScales` in `reader-core.mjs`) and re-holds the article at the reading line through the ordinary navigation path. The hover window offers *Copiar artículo* (the exact body) as well as *Copiar cita*. Markdown downloads, for one article and for the whole instrument, carry the heading path (Título · Capítulo · Sección · Apartado, in that order, printed once per change) above the article label; the per-article file is `## path`, `### label`, then the exact text. The release's own `.md` files are produced by Rust and are unchanged. A plain click on *Texto exacto* opens the exact text as a UTF-8 view because a host may serve `text/plain` without a charset and browsers then guess Latin-1; the raw `.txt` URL keeps working for agents and modified clicks. The index groups articles under their Título · Capítulo. Per-article reform notes stay inside the exact text; the instrument-level latest reform date is shown in the source panel.

A reference opens in a secondary view without moving the origin article; "Volver" restores its scroll position and focus. Targets outside the published collection stay unlinked and are labelled unavailable.

PDF following ships **unavailable**: no page anchor is verified against a retained PDF hash, and a page is never inferred from article number, label occurrence or proportion. The control is disabled and says so; the PDF keeps its own page and zoom, and passive scrolling into a provision with another source (annexes) keeps the open PDF and offers "Abrir PDF de esta disposición" instead of swapping it.

Below 1100px the index and source panes become sheets over the reading column (focus moves into the sheet, Escape closes it and returns focus). The Reader reads only the public release; it needs no database or query service.

**Excluded from this publication path:** the experimental static `agents/<id>/` presentation and the query preview/service. The packager and stager allowlists accept only the files above, and regressions in `package.test.mjs` and `stage.test.mjs` fail closed on agent routes and private release files. Discovery is the release's own `llms.txt` and canonical Markdown/text.

## Verification

```sh
node --test reader/
node reader/browser-check.mjs --publication .work/public-reader --out .work/browser-check.json
```

`browser-check.mjs` drives a real headless Chrome over the DevTools protocol (no dependencies; `--chrome PATH` overrides the macOS default) against a packaged publication. It scrolls with programmatic and trusted wheel events and checks that the active label, index, title, exact copy body, share URL and replaced URL equal the article at the reading line, including when a heading crosses the line without any IntersectionObserver change, plus deep links, settle interruption, resize, history, search, references and the PDF offer. It is kept out of `node --test` because it needs Chrome; run it on the final package and record its exit code.

The packager verifies the release identifier, every artifact digest and byte count, rejects files outside the public allowlist, and creates a reproducible `publication.json`. Rust regression tests cover deterministic exact text, source digest failures, private-data exclusion and destination protection.

## Advance a staged publication

```sh
node reader/stage.mjs --publication .work/public-reader --destination /path/to/Webpage/Lex-Mex/public
node reader/stage.mjs --publication /path/to/Webpage/Lex-Mex/public --destination /path/to/Webpage/LEG4L_L4BS/public/lex-mex
```

The stager validates the publication receipt, public allowlist, release manifest
and all file hashes before writing. It rejects symlinks and altered immutable
release files, preserves earlier pinned releases, locks concurrent staging, and
advances `latest.json` last. Individual root files are atomically replaced; this
is not a transaction across all UI files, so run locally before deployment.
The host verifies its committed package pin before every build.
