# Reader alignment with LEG4L L4BS brand pages 00–05

## Outcome and scope

User requested matching reader code and its editable Figma screen to the brand
identity's pages 00–05. Update the existing component board, desktop/mobile/source
views and states in Figma file `ar2bES9p6jeUBY0w3E4eDY`, and the static reader's
HTML/CSS. Preserve reader behavior, canonical text, provenance, legal qualifiers
and express-reference controls. Do not edit the brand source file
`qRykddSQGCQdn6fnQfGen1`, host serving code, legal corpus, or Sites deployment.

Initial baseline: clean `feat/public-reader` at
`f2af1eff05e53830371cd78e0a96dd6f69db96b9`. The existing immutable corpus release
`3c04016651e2fd4169632756e96ec19136136e1f42c4ed666070f23f20b30660` remains the
content input. UI changes require a new publication receipt, not new legal text.

## Authority and design decisions

Brand pages inspected: 00 `334:2`, 01 `334:3`, 02 `334:4`, 03 `334:6`,
04 `334:7`, 05 `334:9`. Foundations `335:18` establish cyan #00C6E5,
ink #090909, paper #FAFAFA; Space Grotesk Medium headings/labels and Inter body.
Existing code is dependency-free static HTML/CSS/JS. Reuse its DOM IDs, controls
and packaging pipeline. The host already uses these fonts through Google Fonts;
match that established font-loading convention, with readable local fallbacks.
Use cyan with ink text or as a decorative/state accent, avoiding cyan small text
on paper. Keep qualified legal-status labeling distinct from navigation state.
Approved wordmarks/monograms are exact artwork with restricted usage; do not
redraw them or invent a firm logo. Lex-Mex remains a textual product identity.
An older brand prose reference to Hernáiz Abogados is superseded by host AGENTS;
do not carry it into the reader.

## Milestones and checks

1. Inspect brand/screens/tokens/components; record bounded Figma state under
   ignored `.work/reader-brand-alignment/`.
2. Reconcile existing Figma variables, text styles, components and dependent
   screens in place; validate editable structure, bindings and desktop/mobile
   composition screenshots. Preserve existing node IDs when possible.
3. Implement the resulting visual spec in reader HTML/CSS; exercise actual
   search/navigation/reference/PDF controls at desktop and 390px. Fix clipped,
   overflowing or low-contrast elements. Run existing Node tests and diff checks.
4. Repackage the same immutable corpus with the new UI, validate the publication,
   and provide the receiving site account exact IDs and host-pin update steps.
   No Sites upload/deployment by this task. Preserve the previous UI package for
   rollback and unchanged release URLs.

Recovery: revert only task-owned visual changes if needed; do not modify
canonical facts or patch retained PDFs. Do not waive the website's separately
reported PDF range acceptance failure as part of this styling task.

## Progress — 2026-10-13

Discovery complete: inspected all six named brand pages, reader component board
`2:25`, desktop `2:41`, mobile `2:121`, source-mobile `2:148`, states `2:166`,
handoff `2:183`, seven local main components and existing variables/text styles.
No suitable library search result; reused local reader components and rethemed
them from the verified brand foundations.

Completed: all six Figma compositions retain editable layers, component instances
and stable IDs; six type styles, seven components and their dependent views are
brand-aligned. Bound color/radius/spacing tokens use matching CSS syntax.
Reader HTML/CSS implements the same palette and type system while preserving
its actual inline PDF and existing stacked mobile flow. Figma's separate source
view remains explicitly a proposal. Node 11/11 and Chrome 13 checks passed,
including fonts loaded, reader behavior and 320–1440px layout. Screenshots and
structure were inspected; no raster UI layers or canonical/code boundary drift.

Final publication `510cc5e36d0d5d184954cad83775b03ce3e903e3230316466767618c80d7815f`
repackages the existing release; only HTML/CSS hashes change. Repeat packaging
and staging checks pass. Details and receiver pin are in
[`../public-reader-brand-refresh-2026-10-13.md`](../public-reader-brand-refresh-2026-10-13.md).
The old complete UI package remains available for rollback. No host serving fix,
host pin change, source commit, push or Sites deployment was made by this task.

Retrospective: matching the brand required replacing serif reading typography
and green-tinted chrome, not changes to legal content or navigation logic. The
existing component/variable structure supported an in-place update; no new
framework, UI dependency or redrawn brand artwork was needed.

Current next action: receiving Sites account consumes the refreshed package,
updates its publication pin, resolves the separate PDF range blocker and repeats
fresh host/live checks before production promotion.

Commit checkpoint — 2026-10-13: user authorized committing the seven reviewed UI
paths. Commit `2c023c9bebfa6d768eb84f0736cbfb865d322598` records the refresh.
Required fmt, strict Clippy, workspace tests, LRITF/IFPE validators and staged
diff checks passed; reader worktree was clean after that commit. No push or
deployment. Subsequent logo exploration is tracked separately.
