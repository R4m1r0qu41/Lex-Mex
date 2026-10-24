# Parser and reader integration receipt, 2026-10-24

Executes I0–I3 of [the integration plan](plans/parser-reader-integration.md). Machine evidence:
`parser-reader-integration-evidence.json`. Status: **promoted locally (fast-forwards to the tested candidates); not pushed, published or deployed.**

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

Unmoved: `main` `85c41f389`, `refactor/parser-split` `11a3a9095`, `feat/four-instrument-reader-design`
`b9b8be012`, `feat/reader-tweaks-r7` `162128517`. R7 is not in P or R.

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

## Limits

106 of 500 instruments still lack a retained source (not full parse coverage); the API compile fixture proves paths, not
signatures; a P-pinned database is a separate release if wanted. P and R were promoted before an independent review
(the plan wanted Codex's review first; the operator's instruction came first), so a post-promotion review is still open.
