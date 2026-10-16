# Corpus-wide reverse linking — landing report, 2026-10-16

Read this before committing the worktree onto `main`. It is the landing brief for the closure in [`docs/plans/corpus-wide-reverse-linking.md`](plans/corpus-wide-reverse-linking.md). It is not legal advice, and it does not authorize a push.

## Recommendation

Land the closure on `main` as one commit after the required gate below passes. Do not push unless the operator asks in this landing turn.

The change is the corpus-wide reverse link that the 2026-10-16 coverage checkpoint left open. It recomputes reference edges against the 467 instruments that now exist, writes only graphs that validate, and leaves canonical provision text alone. The 11 instruments that would not validate stay on their previous graphs. That non-zero exit is the intended result, not a failed run to repair before commit.

Do not start the reader database in the same commit. Do not drop the 11 missing citations or invent the missing articles to make `link --all` exit 0.

## Baseline

`main` and `origin/main` are `25732f3ef` (`chore: remove misplaced nota-viabilidad comites banca multiple`). That commit has 467 `corpus/mx/*/instrument.json` files and 27,429 stored references. Every stored validation report on that commit was valid.

Two untracked files were already in the worktree before linking and are not linker output:

- `docs/federal-coverage-audit-2026-10-16.md`
- `docs/federal-coverage-audit-2026-10-16-evidence.json`

`docs/project-status.md` now links to the audit. Include both files in the landing commit, or the status link is dangling. Review them as the prior coverage audit, not as evidence produced by `link --all`.

## What the diff is

`link --all` loads the corpus once. For every committed instrument it re-extracts references and defined-term usages with the current linker, including a new marker: the instrument's exact official title, when that title contains a space and no other committed instrument uses it. Curated aliases still supply accent-stripped and colloquial forms. `Ley de Amnistía` is not a marker because `lamn` and `lamni` share it. Adapter historical overrides still win. A curated alias and an official title that assign one marker to two instruments is a hard error; this run did not hit one.

The command then validates the new graph. If it validates and differs, it writes `references.json`, `terms.json`, `term-usages.json`, `validation.json`, and Markdown. If it does not validate, that instrument is left byte-for-byte as it was. Terms and term usages did not change on disk. No `provisions.json`, `instrument.json`, source manifest, adapter count, or review record changed.

| Result | Count |
|---|---:|
| Instruments considered | 467 |
| Graphs rewritten | 305 |
| Graphs already equal to the new extraction | 151 |
| Graphs rejected and kept | 11 |
| References | 27,429 → 30,425 |
| Resolved cross-instrument edges after the pass | 4,147 |
| `terms.json` / `term-usages.json` diffs | 0 |
| `provisions.json` / `instrument.json` diffs | 0 |

Tracked diff against `25732f3ef`: 2,269 files, about +58,674 / −4,856 lines. Corpus files are `references.json` (305), `validation.json` (299), and Markdown. Six rewritten instruments changed reference content without changing `validation.json` bytes. Code and docs:

- `crates/lex-cli/src/main.rs` — official-title markers and `lex-mex link --all`
- `adapters/diputados/_instrument-aliases.json` — seven keys renamed onto the committed short name: `DCG-CUB-2005`, `DCG-CUE-2003`, `DCG-CUCB-2004`, `DCG-FI-2014`, `DCG-SOCAP-SOFIPO-2006`, `DCG-SCAP-2012`, `DCG-OAAC-2009`. The phrase lists were not rewritten. Deferred keys with no committed instrument were not renamed.
- `README.md`, `PLANS.md`, `docs/decisions.md`, `docs/plans/cluster-2-federal-corpus-ingestion.md`, `docs/project-status.md`
- `docs/plans/corpus-wide-reverse-linking.md` (untracked until added)
- this report (untracked until added)

`SERVINV-DCG-2013` was not renamed. Its curated phrase is not the official title of `servinv-dcg-2013` (`DCG-SERVINV-2013`). Leave that key until someone checks the source.

## Rejected instruments

One unresolved cross-instrument article rejects the whole new graph. These 11 therefore also did not receive their other new edges. `git diff` for each directory is empty.

| Slug | Missing target |
|---|---|
| `cpf` | LIC artículo 112 Quintus |
| `lbm` | CFF artículo 136 Ter |
| `lftr` | LGTAIP artículo 180 Bis |
| `lgmime` | LGIPE artículo 11 Bis |
| `lgs` | CPF artículo 209 Quintus |
| `lic` | LRAF artículo 28 Bis; LFCE artículo 31 Bis |
| `lmv` | LGTOC artículos 228-B and 228-H |
| `lpdusf` | CNPCF artículo 1235 |
| `lsar` | LMV artículos 16 Bis 2, 16 Bis 3, 16 Bis 7, and 16 Bis 8 |
| `reg-lan` | LFPA artículo 122 |
| `reg-lisr` | LISR artículo 216 Bis |

`lsar` cites the same absent LMV Artículo 16 Bis family already recorded for `lcnbv`. `lcnbv` was not rewritten; its approved stale edge is unchanged. Do not delete these citations and do not mint the missing articles.

## Re-extraction, not an append

The pass runs the current extractor over every instrument. It can remove a stored edge the current rules no longer emit. Net reference counts fell by one in four instruments:

| Slug | Count | What the sample shows |
|---|---:|---|
| `cpeum` | 170 → 169 | Article 2, “artículo 3o. constitucional”, no longer links to CPEUM Article 3. The generic marker ` constitucional` skips that group. This is a real self-citation the current linker drops. |
| `reg-lgs-investigacion` | 61 → 60 | Article 82, “artículo 4o. de la Ley.”, is no longer an internal Article 4 edge. Bare “de la Ley” stays unresolved on purpose. |
| `reg-senado` | 51 → 50 | Article 8, “artículo 4o. de la Constitución”, lost a false internal Article 4 edge. Bare “Constitución” is not a CPEUM alias, so the CPEUM edge was not created. Article 56 lost a false edge whose span was the heading “Artículo 56 Quinquies”. Article 22 gained a real internal Article 110 edge. |
| `reg-lgeepa-mpcca` | 14 → 13 | Articles 3 and 4 lost internal edges for “artículo 5o. de la Ley” and “artículo 6o. de la Ley”. Article 21 gained a resolved edge to `reg-lgeepa-mretc` Article 10 from the full regulation title. |

295 other rewritten validation reports increased `reference_count`. None of the four decreases is a cross-instrument edge being deleted. A landing reviewer should read the CPEUM Article 2 case before committing; the other three are the existing bare-“Ley” / bare-“Constitución” rule, plus one heading false positive removed.

Zero validation errors do not prove every intended citation was emitted. Ambiguous and bare short names stay unlinked. Accent-stripped forms resolve only when the alias table already lists them.

## Checks already run

- `cargo fmt --check --all`
- `cargo clippy -p lex-cli --all-targets -- -D warnings`
- `cargo test -p lex-cli --bin lex-mex` — 32 tests, including unique-title, ambiguous-title, and rejected-missing-article fixtures
- `cargo build --release --locked -p lex-cli`, then `./target/release/lex-mex link --all`

Not run after the corpus write: `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `validate lritf`, and `validate ifpe-dcg-2021`. Run those before the commit. `lritf` and `ifpe-dcg-2021` were inside the closure; this is the repository gate, not a second linking pass.

## How to land

From a clean inspection of `git status` and `git diff --stat`:

1. Run the required gate. Stop if it fails. Do not relink again unless a gate failure is in the linker.
2. Confirm `git diff --name-only -- 'corpus/mx/*/provisions.json' 'corpus/mx/*/instrument.json'` is empty, and that the 11 rejected directories still have no diff.
3. Stage an explicit path list. Do not use `git add -A` or `git add corpus`. Include the corpus files this diff already shows, the Rust and alias changes, the plan, this report, the status and decision updates, and the two pre-existing coverage-audit files.
4. Commit on `main` only if this landing brief was the operator's request to land. Suggested subject: `Relink the federal corpus against committed official titles`.
5. Do not push in that commit step.

Suggested message body:

```
Recompute reference edges for all 467 committed instruments with link --all.
Official titles are markers when unique; Ley de Amnistía stays unlinked.
305 validating graphs are rewritten (27,429 to 30,425 references). Eleven
instruments that cite a missing article keep their previous graphs.
Provision text, instrument records, and audited reviews are unchanged.
```

## Stop

Do not treat this commit as temporal or legal review. Do not build the SQLite reader index from the uncommitted worktree; build it from the commit that lands, if it lands. Do not retarget `SERVINV-DCG-2013` inside the landing commit.

## Addendum — tightening pass and rebase (2026-10-16, later)

This addendum supersedes the brief's numbers and landing steps; the body above is kept as written.

- The closure was re-derived from the `25732f3ef` baseline after three linker fixes (constitution self markers, decree-title head nouns, historical-edge guard), then rebased onto `7326c41aa`. Result: 305 graphs rewritten, 150 unchanged, 12 held (the 11 above plus `lrsic`), 27,429 → 30,414 references, 4,138 resolved cross-instrument edges. A second `link --all` on the rebased tree changes nothing.
- The two coverage-audit files named in "How to land" are already on `main` in newer form and must **not** be copied from the worktree.
- Reasons for the fixes, the per-target classification of the held instruments, and the `lrsic` question are in `docs/ingestion-difficulty-log.md` (`corpus-wide-reverse-linking`).

