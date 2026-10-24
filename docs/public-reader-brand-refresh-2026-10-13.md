# Reader brand refresh — code, Figma and publication handoff

Updated 2026-10-13. The user requested both reader code and its editable Figma
screen to fit LEG4L L4BS brand pages 00–05.

## Result

The reader uses ink `#090909`, paper `#FAFAFA` and cyan `#00C6E5`, with
Space Grotesk Medium headings/controls and Inter source text. The header is ink
with a cyan divider; navigation uses a restrained cyan selection accent; primary
controls use cyan with ink text. Qualified temporal-status labels remain neutral
and distinct from navigation. All content, controls, DOM IDs and reader behavior
are preserved. Responsive layout remains the existing stacked reading/PDF flow;
the separate mobile source view in Figma is still a design proposal.

The brand file was read only. Approved wordmark/monogram artwork was not redrawn,
retyped or changed. An outdated brand-page mention of Hernáiz Abogados was not
carried forward, following the current host brand boundary.

Code changes are limited to `reader/index.html` (two primary-button classes) and
`reader/reader.css`; there are no JS, Rust, schema or canonical corpus changes.
`reader/README.md` documents the visual system and publication identity boundary.
Fonts follow the existing host's Google Fonts convention and have sans-serif
fallbacks; the browser review confirmed both requested font families loaded.

## Editable Figma output

File: `ar2bES9p6jeUBY0w3E4eDY`, page `0:1`.

| View | Node | Link |
| --- | --- | --- |
| Reusable components | `2:25` | [Components](https://www.figma.com/design/ar2bES9p6jeUBY0w3E4eDY/?node-id=2-25) |
| Desktop 1440 | `2:41` | [Desktop](https://www.figma.com/design/ar2bES9p6jeUBY0w3E4eDY/?node-id=2-41) |
| Mobile 390 | `2:121` | [Mobile](https://www.figma.com/design/ar2bES9p6jeUBY0w3E4eDY/?node-id=2-121) |
| Mobile source | `2:148` | [Source](https://www.figma.com/design/ar2bES9p6jeUBY0w3E4eDY/?node-id=2-148) |
| Empty/verification states | `2:166` | [States](https://www.figma.com/design/ar2bES9p6jeUBY0w3E4eDY/?node-id=2-166) |
| Design handoff | `2:183` | [Handoff](https://www.figma.com/design/ar2bES9p6jeUBY0w3E4eDY/?node-id=2-183) |

Updated six existing text styles and seven main components in place (`2:27`,
`2:29`, `2:31`, `2:33`, `2:35`, `2:37`, `2:39`). Existing instances and text
properties remain intact. Eight new primitive/semantic/radius variables support
the update; existing semantic colors alias the brand primitives. Bindings cover
fills, strokes, spacing and control radii, with matching CSS syntax. Every text
node uses Space Grotesk or Inter; structural readback found no raster/flattened
UI images. All six compositions were visually inspected. Desktop contains 16
component instances; mobile contains five. Original frame and component IDs
are preserved.

## Validation

- Existing Node suites: 11 passed, zero failed.
- Chrome 155.0.8059.40: 13 behavior/HTTP/layout checks passed, no page errors,
  failed responses or network failures in the final run. Checks include exact
  pinned text, background reads, PDF digest/ranges, browser verification,
  navigation/back, reference comparison/selection, annex source switching,
  suffix search, actual loaded brand fonts/colors, no horizontal overflow at
  320/390/700/768/1100/1280px, mobile layout and no-JavaScript discovery.
- Desktop/mobile running screenshots were inspected against the brand-aligned
  Figma compositions. The desktop screenshot shows the retained LRITF PDF
  visibly rendered. Native PDF viewer interaction and desktop-app embedding
  remain separate qualification; this styling task does not change them.
- Two packages built from the same immutable release and current UI produced
  the same new publication ID. The stager verified a fresh local output.
- Receipt comparison: exactly `index.html` and `reader.css` have new hashes;
  every other artifact, the manifest hash and corpus/release identity are
  unchanged. Still 1,783 receipt-listed artifacts plus the receipt.
- Protected Git scope comparison against baseline `f2af1eff0` passed for
  corpus, Rust crates, schemas, reader JS/core and packager/stager modules.
  Diff checks passed. No Rust tests were rerun for this HTML/CSS-only change.

Temporary screenshots, probes, Figma ledger and receipt evidence are under
ignored `.work/reader-brand-alignment/`; generated assets are not Git source.

## New publication identity and receiver action

The refreshed package is staged at the established handoff path:

`/Users/jr/Documents/4ll4/Webpage/Lex-Mex/public`

```json
{
  "schema_version": "reader-publication-v1",
  "release_id": "3c04016651e2fd4169632756e96ec19136136e1f42c4ed666070f23f20b30660",
  "publication_id": "510cc5e36d0d5d184954cad83775b03ce3e903e3230316466767618c80d7815f",
  "corpus_commit": "9e990e09995718a77865993b24c5f8c8e7167b42",
  "manifest_sha256": "c2b22653d7a0b68edc1f29d2b2554fc45eb4a595f5be91f0b96868f2128aedc7"
}
```

This supersedes publication `d84847a5c49e31df359b19369e322d77bf035474000aba45bb18ff728a9172ed`
for the refreshed UI only. The original producer's content-integrity review still
applies to the unchanged immutable release; its old publication pin applies to
the old HTML/CSS. The original complete package is retained locally at
`.work/deployment-review-20261013/repacked-publication` for rollback.

No website source or host pin was edited here. The receiving Sites account must
stage this package into `LEG4L_L4BS/public/lex-mex`, update the host's
`lex-mex-release.json` to the JSON above, then run its prebuild verifier and fresh
`npm test`. Use the existing isolated Sites source/package workflow; include the
generated assets in the full host archive. Follow the original
[deployment handoff](public-reader-deployment-handoff-2026-10-13.md) for live
routes, hashes, MIME types, coverage and recovery, using this new publication ID.

The website handoff records candidate version 19 failing the live PDF range gate
and restoration of version 18. That hosting issue is not fixed or waived by this
UI update. Resolve it and repeat the remaining live acceptance checks in the
website account. User subsequently authorized the UI source commit:
`2c023c9bebfa6d768eb84f0736cbfb865d322598`. Required precommit Rust checks and
LRITF/IFPE validators passed. No deployment or push was performed.
