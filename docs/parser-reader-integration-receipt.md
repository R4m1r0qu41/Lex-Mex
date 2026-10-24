# Parser and reader integration receipt, 2026-10-24

Executes I0–I3 of [the integration plan](plans/parser-reader-integration.md). Machine evidence:
`parser-reader-integration-evidence.json`. Status: **promoted and pushed (`main` and the reader branch); R7 promoted locally only; nothing published or deployed.**

## Scope and authority

The operator's instruction (2026-10-24) was: normal-merge the parser into main, sync the updated main into the
published reader branch at `b9b8be012`, and keep R7's unpublished UI changes separate. I built and verified the
candidates as normal merges on new branches in the operator-created parser worktree, then promoted both real branches
by fast-forward to those exact tested commits. Codex's A-002 closure
report and the parser audit accept the production refactor and D-001.

## The two candidates

| | Commit | Branch | Parents |
| --- | --- | --- | --- |
| P (parser into main) | `88310a8b310f53dff0125f59df323cdab7d7e42a` | `integration/parser-main-2026-10-24` | `85c41f389` (main), `11a3a9095` (parser tip) |
| R (main candidate into published reader) | `f11beebce06b90a2d590acab80078b09fbd254c4` | `integration/published-reader-parser-2026-10-24` | `b9b8be012` (published reader), `88310a8b3` (P) |

P has no conflicts and its tree equals the parser tip's tree. R has one conflict, in `Cargo.lock` only: the
`lex-cli` dependency list, where the reader side added `jsonschema` and the parser side added `lex-analysis`. I kept
both in Cargo's order (`jsonschema`, then `lex-analysis`); `cargo build --locked` at R confirms the lockfile.
`crates/lex-cli/Cargo.toml` and `main.rs` merged automatically. Source ancestry is preserved (no squash, no rebase).

Historical (tips right after the candidate merges, before promotion; the current state is in the Promotion section): `main` `85c41f389`, `refactor/parser-split` `11a3a9095`, `feat/four-instrument-reader-design`
`b9b8be012`, `feat/reader-tweaks-r7` `162128517`. R7 is not in P or R (still true).

## P gates

- Diffs from B: corpus, schemas, adapters, fixtures and `docs/decisions.md` are empty; production differs from the
  tested code C by nothing.
- Disposable clone at P: fmt and clippy clean; `cargo test --workspace` 425 passed, 0 failed, 0 ignored;
  `validate lritf` 145 articles, 11 transitories, 126 references, 0 issues; `validate ifpe-dcg-2021` 59, 4, 115, 0.
  The clone was clean afterwards.
- Read-only database check with the P binary on the existing accepted candidate DB (database SHA-256 `483264b7…`,
  unchanged afterwards): `db verify` exit 0 and output equal to the earlier one (logical digest `8cc37360…`, no stale
  inputs); 21 of 21 golden queries. No projection was rebuilt; that DB truthfully names builder C.
- Tooling self-tests in an isolated scratch clone at P: harness 54/54, G4 wrappers 16/16, plan check 15/15. The 54
  hash-bound artifacts were identical before and after: 51 match the originals and 3 match the disclosed recovery
  hashes, so the incident record is intact.

## R gates

- `reader/` is byte-identical to `b9b8be012`; parser, db, grammar, analysis and link sources equal P.
- Disposable clone at R: fmt and clippy clean; `cargo test --workspace` 430 passed (425 plus the 5 public-release
  exporter tests the reader branch adds); both validations valid with the same figures; `cargo build --locked -p lex-cli`
  exit 0; `node --test reader/` 22 of 22 (the b9b8 inventory; R7's 24 were not imported).
- Exporter and package parity: b9b8-only and R binaries, each built `--locked` from its exact commit, built the
  four-instrument public release twice each from one fixed root with the retained PDF store, then packaged each. All
  four release trees (1785 files) and all four package trees (1791 files) are byte-identical by path and SHA-256.
- Against the accepted local publication (`2df70a8a`, release `9626bfe4`, corpus commit `85c41f389`): the release
  manifest differs only in `release_id`, `corpus_commit` and `committed_at`; all 1782 listed files have identical hashes,
  sizes and bytes; the five UI assets are byte-identical; after normalising the release-id directory name only
  `latest.json`, `llms.txt`, `publication.json` and the release manifest differ, and each embeds the release id or commit.
  This checked the local publication directories, not the live site. No browser or mutation campaign was run (UI and
  content bytes are identical), and no host staging, publication or deployment happened.

## Preservation

The preservation inventory (path, status and SHA-256 of every dirty entry in main, reader-four and reader-five) is in
the evidence file. Main has 25 entries, reader-four 9, reader-five 2; none was touched. The only untracked collision
for promotion is `docs/plans/parser-responsibility-split.md` in main, byte-identical to its first committed version and
superseded by the progressed plan in P. `PLANS.md` and the reader-four design-release plan are tracked-modified and are
not changed by P or R. The parser worktree is now on `integration/parser-main-2026-10-24`; no stash, reset, clean, bulk
stage or deletion was used.

## Promotion (done, local only)

Rechecks first: every tip as expected, ancestry (`main` is an ancestor of P, the reader branch of R), and every dirty
entry in main (25), reader-four (9) and reader-five (2) byte-identical to the inventory; the tracked-modified `PLANS.md`
and reader design-release plan are not changed by P or R. Then:

1. In main, the one colliding untracked file (`docs/plans/parser-responsibility-split.md`, SHA-256 `c5cbcc47…`) was
   moved, not deleted, to `.work/integration-backups/parser-main-20261024T192127Z/` with a path and hash manifest
   (manifest SHA-256 `3c9776df…`); its hash is equal before and after the move.
2. `main`: `git merge --ff-only 88310a8b3`, from `85c41f389` to `88310a8b310f53dff0125f59df323cdab7d7e42a` (P).
3. `feat/four-instrument-reader-design`: `git merge --ff-only f11beebce` in the reader-four worktree, from `b9b8be012`
   to `f11beebce06b90a2d590acab80078b09fbd254c4` (R). R's second parent is P, so main is synced into the reader at P.

After: both trees equal the tested candidates' trees; the 24 remaining dirty entries in main, 9 in reader-four and 2 in
reader-five are byte-identical to the inventory, and `PLANS.md`'s local modification survives. `refactor/parser-split`
(`11a3a9095`), `feat/reader-tweaks-r7` (`162128517`) and `origin/main` (`85c41f389`) did not move. No `--no-ff`, reset,
force, stash, clean, deletion or push was used. The reader branch sits at R and lacks this receipt's docs commit, which
follows P on main.

**R7** stays separate and unpublished: no R7 file is in main or the reader branch. It now lacks the parser split, so a
future R7 release needs main merged into it first. Push and deployment still need separate authorization.

## Push (2026-10-24)

Before the push: R is exactly the union of both sides for the three files both touched (`Cargo.lock`,
`crates/lex-cli/Cargo.toml`, `crates/lex-cli/src/main.rs`); a credential-shaped scan of both push ranges found only the
Spanish word "Secretaría" and a Node test path string, and no binary files; `origin/main` was still `85c41f389` and the
reader branch was absent. Pushed with no force and no tags: `main` `85c41f389..0642bbea1` (fast-forward) and
`feat/four-instrument-reader-design` as a new remote branch at `f11beebce`. `feat/reader-tweaks-r7`,
`refactor/parser-split` and the `integration/*` branches were not pushed.

**CI is red, and it was red before this change.** Run `40126862221` (push of `0642bbea1`) failed at the Clippy step
with `clippy::assert_is_empty` at `crates/lex-db/src/build_tests.rs:237`, exactly as runs at the baseline did
(`39731598118` at B and two earlier). CI installs `stable` (clippy for Rust 1.99.0); my local toolchain is 1.95. B has
35 `assert!(…is_empty())` calls and P adds none. The merge did not cause it and I did not change lint code (out of scope
for a behaviour-preserving integration). Clippy fails first, so CI never ran the tests or validators: there is no Linux
test result for this code. The workflow has read-only permissions and no deploy job.

**Review tier:** the pre-push review was by the advisor (Opus 5.5), the executor's own model family, not independent.
Codex's status report asked for an independent review before any push; the operator chose advisor review, then push.
The independent audit is still open. Nothing was published or deployed.

Raw integration receipts (native, exporter and package logs, DB verify and golden outputs, self-test logs, the
preservation inventory, and tree digests for the four releases and four packages) are kept in the ignored
`.work/parser-reader-integration/`, bound by `hash-manifest.json` (SHA-256 `f7b92404…`).

## R7 (promoted locally), Libro fix and CI lint fix

**R7.** The updated reader was merged into R7 (`d2ab64d61`, clean), the Libro fix and a lint fix were added, and
`feat/reader-tweaks-r7` was fast-forwarded from `162128517` to `0661d5ef0ff61517287c4de0437b90f6bb60ffcc` in the
reader-five worktree (rechecks first: tip unchanged, ancestry, its 2 untracked files byte-identical before and after and
not colliding). Gates at the merge: 430 Rust tests, both validations, `--locked` builds, `node --test reader/`, and four
releases and four packages byte-identical between R7-only and merged binaries; against release `9626bfe4` only the
commit-derived fields differ. **Local only: not pushed, restaged, pinned or deployed.**

**Libro fix** (`a21459733`). `headingLevels` now starts with `libro`, and a new `indexLevels` (Libro, Título, Capítulo)
feeds the index grouping. The index change goes beyond Codex's P3 (which named only `headingPath`) but is the same
omission: two Libros repeating a Título name would otherwise share an index group. New fixture test (25 of 25; it fails
without the change). Old and new heading code give identical output for all 870 provisions of the four preferred
instruments (none has a Libro). The rebuilt package differs from the pre-fix R7 package only in `reader-core.mjs` and
`publication.json`. **No browser harness was run on it:** the equivalence check is enough to move the local branch, not to
publish; run the preferred browser harness once on the rebuilt package before any restage or pin change. Also,
`browser-check.mjs` (lines 363-364) predicts the export path as Título then Capítulo and would mis-predict for a código
with Libros; update it when such a code enters the reader.

**CI clippy lint** (`ce4bde27c` on main, cherry-picked with `-x` from `0661d5ef0`). CI installs `stable` (Rust 1.99.0,
clippy flags `assert!(x.is_empty())`); my toolchain was 1.95.0. Clippy 1.99.0 over the whole workspace, with
`--keep-going`, found exactly two lint sites, on main and on the R7 candidate (the earlier "35" counted the assertion
pattern, not the lint). Both became `assert_eq!(x, Vec::<String>::new())` (test code only). In a disposable clone with
toolchain 1.99.0 and CI's steps: fmt, clippy `-D warnings`, 425 tests, `validate lritf` and `validate ifpe-dcg-2021` all
pass; clippy is also clean on 1.95.0; main's tree equals the verified tree. Not changed: the pushed reader branch
`f11beebce` still has both lints (a PR from it would go red; it gets the fix at its next main sync), and
`rust-toolchain.toml` still says `stable`, so a future stable bump can turn CI red again (pinning to `1.99.0` is a repo
policy change for the operator). The fix touches `crates/lex-db/src/build_tests.rs`, a pinned builder directory, so builder
identity at the new main differs from P; the accepted DB still truthfully names builder C and needs no rebuild. Receipts:
`.work/parser-reader-integration/lint/` (`hash-manifest.json` SHA-256 `d0835bbe...`).

**Review tier:** the ongoing independent Codex audit is bound to P/R. The Libro fix, the lint fix and the R7 promotion have
had advisor review only.

## Handoff records

Codex's status report (`parser-reader-integration-status-2026-10-24.md`, untracked on main) noticed a gap in these
records: the earlier Codex-to-Claude integration request was still marked pending, there was no fresh P/R Codex
handoff, and this receipt kept pre-promotion wording ("Unmoved", `tips_unmoved`, `promotion_procedure_not_run`).
Codex will place its own report. Tidied here: the request is resolved in the vault with a Resolution section, the
stale fields are labelled historical, and Codex's execution plan is bound by SHA-256 (`f257f01e9a62414cbe7a582f39c41c29c5bfc0f26cf9c0fa0e29368b356ed151`; it stays
untracked and Codex-owned). I did not write a new P/R Codex handoff; the independent P/R review is Codex's to
schedule.

## Limits

106 of 500 instruments still lack a retained source (not full parse coverage); the API compile fixture proves paths, not
signatures; a P-pinned database is a separate release if wanted. P and R were promoted before an independent review
(the plan wanted Codex's review first; the operator's instruction came first), so a post-promotion review is still open.
