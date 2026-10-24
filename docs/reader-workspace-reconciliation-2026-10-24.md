# Permanent reader workspace reconciliation — 2026-10-24

Executes step 1 of `reader-r7-final-review-2026-10-24.md`. Workspace: `/Users/jr/Dev/lex-mex-reader` (formerly on
`feat/public-reader` at `f5330c3e9`). Executor: Claude Sonnet 5.5. Nothing was reset, cleaned, force-moved or deleted; no
push, host staging, pin change or deployment was performed.

## Preservation (made before any move)

- Tag `preserve/public-reader-pre-reconcile-2026-10-24` → `f5330c3e9` (the six commits).
- Off-repo backup `/Users/jr/Dev/.work-backups/lex-mex-reader-preserve-2026-10-24/`: porcelain status (43 entries),
  `tracked-changes.patch`, a verbatim copy of every untracked file with `untracked.sha256`, and a git bundle of the six commits.

## The six unique commits (`4072dd53b` … `f5330c3e9`)

Blob-ancestry check of every file they touch against `0661d5ef0` (R7):

- **Identical in R7:** `crates/lex-cli/src/public_release.rs`, both public schemas, `reader/{lex-mex.svg,package.mjs,serve.mjs,stage.mjs}`.
- **Superseded by function:** `reader/{index.html,reader.css,reader.js,reader-core.mjs,README.md,*.test.mjs}`, `crates/lex-cli/*`,
  `Cargo.lock`. R7 holds no commit with those exact blobs because the four-instrument reader reworked them. R7 still has no
  HEAD-only CSS variable or `reader-core` export. The HEAD-only element ids belong to the old single-article view that the
  continuous reader replaced by design. The logo, exact-text and PDF-panel features are present.
- **Unique, carried verbatim onto the R7 line by this commit** (nine documents, byte-identical to the tag) plus the three
  matching `PLANS.md` index entries: `agent-discovery-experiment-receipt`, `plans/reader-brand-alignment`,
  `plans/reader-logo-exploration`, `plans/reader-usability-and-medium-identity`, `public-reader-brand-refresh-2026-10-13`,
  `public-reader-build-audit-2026-10-07`, `public-reader-deployment-handoff-2026-10-13`,
  `public-reader-usability-handoff-2026-10-13`, `reader-session-close-2026-10-14`.
- **Not carried, left on the tag/branch:** the older `docs/plans/nested-public-reader.md` (289 added lines against R7's 151;
  it needs a human merge) and the older `PLANS.md`. Retrieve with `git show f5330c3e9:docs/plans/nested-public-reader.md`.

## The 43 local changes

| Disposition | Items |
| --- | --- |
| Committed to new branch `feat/public-query` (`d2170f9fd`, off `f5330c3e9`) | query contract/schema, `lex-core::public_query` and its one-line `lib.rs` export, `query-*` and `agent-pages*` modules with tests and fixtures, the agent-access plans/reviews/prompts, the `PLANS.md` plan entries, and the agent-page edits to `package.mjs`, `stage.mjs`, `stage.test.mjs` and `reader/README.md` |
| Left in place, uncommitted | `AGENTS.md` (one added line; `AGENTS.md` changes go in their own reviewed commit), `CODEX.md` (Codex-owned, untracked) |
| Left in place, uncommitted, for the operator | deletion of `nota-viabilidad-comites-banca-multiple.{md,docx}` (unrelated; still present in the tag and the branch) |

`feat/public-query` is unreviewed, carries the earlier review's open findings, and its `package.mjs`/`stage.mjs` would add
agent pages and a different `llms.txt` to any package built from it. **Never build or stage a publication from it**; the
reviewed R7 package comes only from the R7 line.

## Open

Moving the permanent workspace onto the R7 line, merging R7 into `main` and the published-reader branch, and publication are
separate steps.
