# Independent P/R audit at main HEAD — 2026-10-24

Status: **P and R hold at main `e3ad94ee3`.** No parser-behavior defect found.
This packet is for Codex review. It does not authorize promotion, push, publication,
or deployment, and it does not replace the bound integration logs.

Reviewed tip: `e3ad94ee3bf6e5a5ec34274ad77af10ce88bcf1d` (`main` in
`/Users/jr/Dev/lex-mex`). `origin/main` at the same inspection is `a07a93f31`; the
four commits after it are documentation and cleanup only (empty diff over `crates/`,
`reader/`, schemas, and Cargo manifests).

## What was rechecked

Git identities and trees were recomputed at this HEAD. The P/R native, exporter,
database, and self-test logs were not rerun. Those logs remain the ones bound by
`.work/parser-reader-integration/hash-manifest.json` (SHA-256
`f7b9240463d3120504c5831315383433c8034b5d082038db502c5e305d0fbc04`), re-hashed in
the earlier pass of this audit. The delta since P and R was read as patches.

Candidates, unchanged:

| | Commit | Role at this HEAD |
| --- | --- | --- |
| B | `85c41f389` | Pre-split main |
| C | `1250889ce` | Tested parser production |
| P | `88310a8b3` | Parser normal-merge into main. Ancestor of HEAD. Tree equals parser tip `11a3a9095`. |
| R | `f11beebce` | Published reader `b9b8be012` plus P. Ancestor of HEAD. Still the tip of `feat/four-instrument-reader-design` and of `origin/feat/four-instrument-reader-design`. |

## Verdict

The composition checked before push is still the composition on main.

- P is an ancestor of HEAD. `git diff 11a3a9095 P` is empty. `git diff C P` over
  `crates/`, `Cargo.toml`, and `Cargo.lock` is empty. `git diff B P` over corpus,
  schemas, adapters, fixtures, and `docs/decisions.md` is empty.
- R is an ancestor of HEAD. `reader/` at R equals `b9b8be012`. Parser, core, db,
  grammar, analysis, link, source, and export at R equal P. The only paths both
  parents changed since B are `Cargo.lock`, `crates/lex-cli/Cargo.toml`, and
  `crates/lex-cli/src/main.rs`. R keeps both `jsonschema` and the split crates
  (`lex-analysis`, `lex-grammar`, `lex-link`). `main.rs` is the parser import move
  plus the reader's `public-release` command, including `validate_corpus_snapshot`
  so a release build does not write `validation.json`.
- From C to HEAD, `lex-parse`, `lex-grammar`, `lex-analysis`, `lex-link`,
  `lex-core`, `lex-source`, and `lex-export` are unchanged. The only `lex-db`
  source change is the test assertion below.
- From B to HEAD, corpus, adapters, fixtures, and `docs/decisions.md` are unchanged.
  Schemas differ from B only by `public-instrument-v1` and `public-release-v1`,
  which are unchanged from `b9b8be012`.
- Bound logs, not rerun here: P 425 tests, R 430 tests (the five extra are
  `public_release`, all ok), both validators at the recorded figures (LRITF
  145/11/126/0, IFPE 59/4/115/0), Node 22/22 on the b9b8 inventory, self-tests
  54/54 and 16/16, plan check 15 OK lines. The four release tree digests match
  (`3632b415…`, 1,785 files) and the four package digests match (`aeb36f30…`,
  1,791 files). Both exporter logs print release `ec4b5ee0…`.

## What HEAD adds after P and R

These are on main now. They are not silent edits to the parser split.

1. Clippy fix `ce4bde27c`, cherry-picked from R7 `0661d5ef0`. Two test assertions
   change from `assert!(x.is_empty())` to `assert_eq!(x, Vec::<String>::new())`:
   `crates/lex-db/src/build_tests.rs` (the pre-existing CI site) and one closure
   test in `crates/lex-cli/src/main.rs`. No production branch changed. Both sites
   are gone at HEAD. `f11beebce` still has `assert!(verified.stale_input_paths.is_empty())`
   at `build_tests.rs:237`, so a CI run of that branch alone would still fail
   `clippy::assert_is_empty` on Rust 1.99. `rust-toolchain.toml` remains `stable`.
2. R7 line, including Libro fix `a21459733`, merged by `976040a2f` (rehearsal) and
   present on `origin/main` `a07a93f31`. `git diff 0661d5ef0 HEAD` over `reader/`
   and the crates is empty, so main's reader and CLI are the R7 tip: R, plus the
   R7 UI, plus Libro, plus the same clippy patch. The `PLANS.md` conflict in that
   merge keeps both parents' index entries; each of the parser-split, integration,
   and four-instrument reader plans appears once.
3. `headingLevels` is `libro`, `title`, `chapter`, `section`, `apartado`, matching
   `lex-db` and the Diputados parser. The index uses `indexLevels` (`libro`,
   `title`, `chapter`). A missing, null, or empty `libro` is dropped, so the four
   preferred instruments are unchanged. `node --test reader/` at that tree passes
   25/25. `reader/README.md` still describes the path as Título · Capítulo ·
   Sección · Apartado and the index as Título · Capítulo.

## Evidence limits (not merge defects)

- `db verify` in the bound receipt records `head_commit` `c4439068` (docs-only
  "Record G0-G3 state"), not P. Crates at `c4439068`, at C, and at P are the same.
  `stale_input_paths` is empty. Logical digest `8cc37360…`, database `483264b7…`.
  The check supports the code. It does not record a binary built from P's Git id.
  The accepted database still names builder C. The clippy commit touches
  `build_tests.rs`, which the receipt already notes changes builder identity
  relative to P. No rebuild was required for this audit, and none is recommended
  merely to relabel that database.
- Release and package parity is an aggregate tree SHA. Per-file maps and the trees
  were not kept, so those hashes were not recomputed.
- 106 instruments still have no retained source. The API compile fixture proves
  paths, not signatures. Both limits predate the merge.
- This pass did not re-run fmt, Clippy, `cargo test`, the validators, the exporter,
  or a browser campaign, and it did not fetch or probe the live host. Later notes
  in `docs/r7-integration-and-work-cleanup-2026-10-24.md` claim a deployed
  publication `0d142a9a…`; that claim was not rechecked here.

## Open items outside the P/R parser contract

Present at HEAD because R7 is present. Not regressions of the parser merge.

- Text-size toggle calls `go(..., 'resize')`. `sourceDecision` swaps the open PDF
  for every trigger other than `scroll`, so sizing text while an annex offer is
  showing replaces the PDF and drops its page and zoom. IFPE and ITF have annexes.
- `public-release` rejects a null `publication_date`. Canonical dates are optional.
  Ordenanza General de la Armada (`oga`) has a null instrument date and 1,869
  provisions with a null date, so that slug cannot be published. The four preferred
  instruments all have dates.
- `browser-check.mjs` still predicts Título · Capítulo only. The receipt already
  says it would mis-predict a código that has Libros. The four preferred
  instruments have none.

## For Codex

P and R do not need a repair. The independent review the integration receipt left
open can close on this packet for the parser merge and the published-reader sync.
R7, the Libro fix, and the lint fix were not part of that original P/R request;
they are on this HEAD and are reported above so the tip is not mistaken for R.
