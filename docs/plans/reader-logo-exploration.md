# Lex-Mex logo exploration

## Scope and outcome

User authorized committing the completed reader UI refresh, then requested four
editable logo directions on reader Figma page `16:27` (`02 — Logo`). Produce two
shared-EX symbols and two wordmarks, plus a comparison of their four pairings.
Do not select a finalist, replace the running reader logo, edit firm artwork,
or deploy anything. Attached images are concept references, not production assets.

Reader refresh committed as `2c023c9bebfa6d768eb84f0736cbfb865d322598`; required
Rust checks, LRITF/IFPE validators and staged diff passed; reader worktree clean
before this logo plan. The current publication remains `510cc5e36d0d...`.

## Source and design contract

Target file `ar2bES9p6jeUBY0w3E4eDY`, empty existing logo page `16:27`.
Brand reference `qRykddSQGCQdn6fnQfGen1`, read only. Uppercase artwork uses
Space Grotesk Bold (verified at `8:2` and `8:3`). Three-bar reference geometry:
top `12:2` (96×15), middle `12:3` (88×16), bottom `12:4` (96×15), at relative
Y 0/51/103. Reuse that geometry with current brand cyan #00C6E5; its legacy
source cyan differs from the current brand foundations. Ink #090909 and paper
#FAFAFA match the reader's established tokens.

1. Uppercase L and M stacked left of one shared EX; each occupies half-height
   of the EX with optical spacing. E uses the source's three bars.
2. Lowercase l flows into the shared E's middle bar; M sits below, X at right.
3. Uppercase LEX-MEX, Space Grotesk Bold, both E's rendered as cyan three bars.
4. Lowercase lex-mex, Space Grotesk, only the initial l cyan.

Main assets remain editable vector/text components outside the review board.
Show instances on paper and ink; add four paired lockups for choosing a finalist.
Keep fills token-bound, use auto-layout for review cards and related containers,
and preserve the running reader page.

## Milestones and current state

Discovery complete: target empty; exact typography/bar geometry read and exported
without changing brand source; local tokens available; logo library search empty.

Completed 2026-10-13: four directions built as four component sets with eight
Light/Dark variants, plus shared three-bar E component `20:27`. Main assets sit
outside the review boards. Uppercase letters are editable Space Grotesk Bold
outlines; the lowercase wordmark remains editable text with a shared text style
and range-colored initial l. No raster images or flattened full-logo screenshots
are used as artwork. The connected l arm matches the E middle bar's exact
scaled top/bottom coordinates; L/M have half-height slots with optical spacing.

Review boards:

- [Four directions, `22:60`](https://www.figma.com/design/ar2bES9p6jeUBY0w3E4eDY/?node-id=22-60)
  — 1760×1088, eight theme previews.
- [Four pairings, `22:168`](https://www.figma.com/design/ar2bES9p6jeUBY0w3E4eDY/?node-id=22-168)
  — 1760×1048, four symbol/wordmark combinations on both surfaces.

Component sets: uppercase symbol `20:34`, connected-l symbol `20:65`, uppercase
wordmark `20:94`, lowercase wordmark `20:143`. The boards contain 24 top-level
logo instances plus nested shared-E instances, native vector/text layers and
token-bound fills. Full composition screenshots, actual fonts, transparent SVG
containers, labels and theme contrast inspected. Corrected row sizing,
unintended SVG-container fills and caption alignment before final inspection.

Running reader UI and staged publication unchanged by logo exploration. No
finalist selected, logo installed, push or deployment. Working source, final
screenshots and exact ID ledger remain in ignored `.work/reader-logo-options/`.

## Lowercase direction chosen — 2026-10-13

User rejected the connected-l/E concept and deleted it in Figma, changed the
initial lowercase l to Cascadia Mono in selected wordmark instances, and chose
the lowercase direction. Preserve those edits; do not recreate deleted options.
User then requested applying spacing/weight suggestions in reader visual copies
on the logo page, removing their L·M mark and uppercase title, while keeping
standalone shared LM/EX and uppercase LEX-MEX references.

Created [Regular/Medium reader comparison, `43:267`](https://www.figma.com/design/ar2bES9p6jeUBY0w3E4eDY/?node-id=43-267)
on page `16:27`, containing full editable desktop copies `43:27` and `43:91`.
Both headers use a 28px lowercase wordmark: Cascadia Mono Regular initial l in
cyan, Space Grotesk Regular or Medium for ex-mex, first-letter tracking -1px and
remaining tracking -0.15px. Header masters are component variants `43:262` and
`43:264`, set `43:266`. The copies replace the original uppercase title and
remove the separate L·M glyph; original reader frame `2:41` remains unchanged.

Created [separate alternatives reference, `43:313`](https://www.figma.com/design/ar2bES9p6jeUBY0w3E4eDY/?node-id=43-313)
with shared LM/EX and uppercase LEX-MEX instances. Existing logo-page content
and user edits remain intact. Readback confirms correct mixed fonts, weight,
range colors, exactly one lowercase wordmark per copied header and no uppercase
title/L·M glyph there. Both composition screenshots and actual-size header
comparisons were visually inspected; all layers remain editable, with no image
fills. State and screenshots are in ignored `.work/reader-logo-options/`.

This step changes Figma comparison copies and this plan only; running reader
code, canonical data and staged publication are unchanged. Medium is the current
recommendation for compact header recognition; Regular remains available for
comparison. No weight was installed or deployed.

Historical next action (superseded by Medium approval below): choose weight;
apply the chosen lowercase identity to the production reader after that decision.


## Medium approved — 2026-10-13

User selected Medium and authorized its installation on main Figma reader page
0:1 and in code. This supersedes the exploration-only exclusion above. Exact
vector export from43:264 is reader/lex-mex.svg; L·M removed. Original comparison
43:267 and separate references43:313 remain. Usability work and website handoff
continue in docs/plans/reader-usability-and-medium-identity.md.
