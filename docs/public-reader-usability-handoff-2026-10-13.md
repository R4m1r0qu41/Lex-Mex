# Website handoff — Medium identity and reader usability

## Ready package and ownership

The producer reader implementation and editable Figma changes are ready for the
receiving website account. Version23 is live, deployed by the receiver account. Producer changes from
baseline `f4991e98c474b889096bd32c08218f5a3a91f04a` are included in the
2026-10-14 authorized local session-close commit. See
`docs/reader-session-close-2026-10-14.md` for final validation and continuity.

Package:
`/Users/jr/Dev/lex-mex-reader/.work/reader-usability/public-main-favicon`

- Publication ID: `407aa3dc658a274ac712e9494045a4c97724e5df0686de4e0513bf1af039470f`
- Unchanged corpus release: `3c04016651e2fd4169632756e96ec19136136e1f42c4ed666070f23f20b30660`
- Manifest SHA-256: `c2b22653d7a0b68edc1f29d2b2554fc45eb4a595f5be91f0b96868f2128aedc7`
- Corpus commit: `9e990e09995718a77865993b24c5f8c8e7167b42`
- Five changed/added artifacts: `index.html`, `reader.css`, `reader.js`,
  `reader-core.mjs`, `lex-mex.svg`. All immutable release artifacts unchanged.
- Receipt lists 1,784 files; `publication.json` is additional (1,785 total).

Use the complete package; updating only the visible HTML misses the clipboard,
index and download behavior. The SVG is an exact outlined export of approved
Medium component `43:264`, so the Cascadia Mono l needs no visitor-local font.

## Figma specification

[Main reader page](https://www.figma.com/design/ar2bES9p6jeUBY0w3E4eDY/?node-id=0-1)
was updated in place. Desktop `2:41`, mobile `2:121`, source mobile `2:148`,
About desktop `65:50`, About mobile `65:59`. Desktop complete index `67:37`
and mobile index `69:185` each contain all 156 published LRITF provisions in a
scrollable viewport. Components/layers remain editable. The logo-page Regular
comparison and separate LM/EX / uppercase references remain preserved.

## Delivered behavior

1. Lowercase Medium wordmark replaces LEX-MEX; separate L·M is removed.
2. Left navigation includes all provisions of the selected instrument and groups
   by article/transitory/annex and recorded title/chapter. Search remains across
   the four published instruments; query results retain the existing 100-hit cap.
3. PDF verification shows the actual source URL (Diputados, CNBV or another
   recorded publisher), and says the retained PDF's SHA-256 matches the digest
   recorded for the source. Success is shown only after a successful hash check.
   It does not claim today's remote file was fetched or that PDF bytes hash to
   the text's digest. Selection changes clear the result; in-flight results from
   another provision are not attached to the newly selected provision.
4. Consultation notice: “Corpus de consulta. Nada aquí constituye asesoría
   jurídica. La fuente de publicación oficial de la normatividad federal es el
   DOF.” This avoids describing every Mexican norm as exclusively federal.
5. Copiar texto, beneath Copiar enlace, copies only `provision.text` with literal
   accents, tildes, whitespace and line breaks; no heading, links or metadata.
6. Clicking Markdown downloads the article's exact UTF-8 text as `.md`, without
   injected metadata or presentation links. The immutable raw Markdown URL and
   HTML alternate link remain available for agents; raw `.txt` remains exact.
7. Left-side Descargar norma completa downloads an actual `.md` containing every
   provision in publication order, with instrument/article headings and unchanged
   body strings. This includes published transitories and annexes. It is not a
   PDF reconstruction or a promise of content outside the published corpus.
8. About opens a native modal with LEG4L L4BS attribution, Markdown/agent-search
   purpose, source provenance, partial coverage and active database/concept/search
   development. Escape closes it and focus returns to its trigger.
9. Footer adds © LEG4L L4BS 2026.

The reported encoded accents were not reproduced in legal-body Markdown: all
867 public `.md` body sections and `.txt` strings match their public canonical
strings. Percent escapes in an upstream source URL are URL syntax, not text
corruption. New download/clipboard routes avoid copying that metadata into an
article's text. Do not globally URL-decode or entity-decode canonical legal text.

## Favicon — accepted site monogram; standalone 4 retired

The reader must match the main site's square black `[4ll4]` monogram. The
standalone `4` favicon is retired and must not be used as a fallback in future
producer packages or deployment instructions. Use the existing main-site assets:

```html
<link rel="icon" href="/favicon-monogram-square.png" sizes="512x512" type="image/png">
<link rel="shortcut icon" href="/favicon-monogram-square.png">
<link rel="apple-touch-icon" href="/avatar-dark-512.png">
```

These links are now in producer HTML before packaging. Version23 already uses
them live; no icon artwork changes or additional deployment are required.

## Receiving account steps

1. Preserve the complete prior UI package for rollback. Stage the new package
   using this checkout's updated `reader/stage.mjs`; older staging scripts may
   reject the new SVG asset. The update permits that exact optional SVG while
   preserving support for prior publications.
2. Update the website's `lex-mex-release.json` publication pin to the exact new
   receipt, leaving the corpus release and manifest hash unchanged. If changing
   the icon or any reader bytes, repackage first and use that resulting ID.
3. Run the website's verifier, tests and fresh build, confirming SVG serving,
   complete index, article clipboard, full `.md` download and About on mobile.
4. Follow the account's existing Sites deployment workflow and repeat live
   acceptance, including PDF Range 206 / Content-Range checks. The current host handoff records version 21 live with a repaired PDF range
   implementation. Preserve that Worker and PDF build-staging layout; this
   update changes reader assets only. Record the newly deployed version and
   actual live acceptance result in the host HANDOFF.md.

Example staging (not executed against the host by this task):

```bash
node /Users/jr/Dev/lex-mex-reader/reader/stage.mjs \
  --publication /Users/jr/Dev/lex-mex-reader/.work/reader-usability/public-main-favicon \
  --destination /Users/jr/Documents/4ll4/Webpage/Lex-Mex/public
```

## Validation

- Node suite: 12 passed. JavaScript syntax and diff whitespace checks passed.
- Installed headless Chrome 155: 21 targeted checks passed, including failed re-verification clearing prior success and exact
  clipboard/article/full-instrument downloads, complete grouped index, pinned
  navigation/references, annex PDF switching, PDF SHA-256 and local Range,
  About Escape/focus restoration, loaded fonts, SVG MIME and no overflow at
  320/390/700/768/1100/1280/1440px. Zero page errors, failed requests/responses.
- All 867 raw Markdown bodies and exact-text files matched public canonical
  strings. All release-file hashes unchanged; new UI receipt validated through
  packaging and staging. Staging prior then new publication passed locally.
- Figma native layers, Cascadia Mono + Space Grotesk Medium logo segments,
  Space Grotesk/Inter UI fonts, 156-item indices and desktop/mobile/About
  compositions inspected. No complete-UI raster layers.
- Headless checks do not certify mobile-native PDF rendering or live Sites
  range behavior. No Rust/canonical changes; no commit, push or deployment.

Evidence: ignored `.work/reader-usability/browser-audit.json`, browser script,
PNG screenshots, Figma mutation source and `.work/reader-usability/public-main-favicon`.


## Historical deployment request — resolved by receiver

The producer account could not access the persisted Sites project. The owner
account subsequently deployed the usability update, corrected the favicon and
published version23. The earlier local staging directory and deployment blocker
are historical evidence; use the final package/receipt above for future work.
The site's current version23 preserves the repaired PDF Worker and serving
layout, and its accepted square-monogram icon supersedes the earlier 4-only
fallback. Do not reuse that retired fallback or old candidate packages.

## Live acceptance and producer reconciliation — 2026-10-13

Live HTTPS reader HTML and receipt confirm publication407aa3dc658a274ac712e9494045a4c97724e5df0686de4e0513bf1af039470f,
Medium logo, copy-text control and the main site's square-monogram favicon.
The host HANDOFF.md records version23 deployment and acceptance. This supersedes
the historical current-account access blocker above: deployment was completed by
the owner account. Producer HTML now carries exactly the receiver's three icon
links; repackaging matches the live publication ID. Standalone4 is retired.
No additional deployment or host artwork/source mutation was needed here.
