# Parser split implementation receipt

Living narrative for [the execution plan](plans/parser-responsibility-split.md);
machine evidence is in `parser-split-implementation-evidence.json`. Status:
**IN PROGRESS**. The strict A/B queue finished: C1, C2 and C3 each match the
baseline exactly, awaiting advisor review; nothing is accepted, merged, pushed or
deployed.

## Where things stand

| Gate | Status | What exists | What is still required |
| --- | --- | --- | --- |
| G0 | baseline pair exact | strict harness (17/17 self-test), strict B3 == B4 on retained, sweep and supplement; B1 and B2 are legacy | advisor acceptance of coverage (106 parse-replay blockers) and the date-inertness record |
| G1 | A/B exact | commit `07cfa5536`, binary C1; C1 == B4 on all three run kinds; advisor accepted the symbol and dependency map | advisor gate decision |
| G2 | A/B exact | commit `fd70e4370`, binary C2; C2 == B4 on all three run kinds | advisor gate decision |
| G3 | A/B exact | commits `5c074e21d` and `1250889ce`, binary C3; C3 == B4 on all three run kinds | advisor review of the final map |
| G4 | checks met | native at C, B/B baseline, two B/C builds, verify, 21 goldens, row parity | advisor gate decision |
| G5 | not started | none | advisor final pass, commit-bound Codex handoff |

Commits on `refactor/parser-split` (baseline B = `85c41f389`): `ff38d1882`,
`07cfa5536`, `f1da6f1e7`, `fd70e4370`, `5c074e21d`, `1250889ce` (the code under
test, binary C3), then `c4439068e`, `d05d98d3b`, `73f42a983`, `ad1c8382a` and
`366cef07f` (docs, harness, self-test and driver only). `git diff
1250889ce..366cef07f` over crates, manifests, lock, toolchain, schemas, database
fixtures and adapters is empty, so HEAD production code equals C3's source. Binary
hashes and per-tree native results are in the evidence file.

G0 pass wording (plan): "Unavailable retained inputs must be resolved or
explicitly returned as a coverage blocker; do not call reduced coverage full
acceptance." 106 are returned as blockers here; this is not full coverage.

## G0 — isolation, reproducible baseline and replay coverage

- B equals the planning baseline; no HEAD drift. The operator-created worktree is
  `/Users/jr/Dev/lex-mex-parser-split`. Local milestone commits are authorised by
  the goal prompt; there is no push, merge, deployment or corpus change. The
  fifteen unrelated dirty entries on `main` were never touched.
- Native baseline at B: fmt, clippy `-D warnings`, 419 tests, none ignored.
- Harness `prompts/parser-split-ab.py` replays retained-source stages and a
  whole-corpus sweep through one frozen binary, rebuilding a disposable root from
  `git archive B` at one fixed path with a scrubbed environment, so no
  normalization is needed. Comparison is byte-exact; the comparator allowlist is
  empty. The harness hash is taken at process start for every valid run; only the
  legacy runs B1 and B2 recorded it at the end (see the evidence rules below).
- Retained replay B1 finished: 1771 rows over 355 instruments. Baseline facts the
  candidates must reproduce exactly: `validate` exits non-zero for `cjm`,
  `cub-dcg-2005`, `cucb-dcg-2004`, `laero`, `oaac-dcg-2009` and `scap-dcg-2012`;
  `lfiiedb` fails at `parse` (its retained text does not match the committed
  manifest hash); `oga` fails at the temporal-request stage. In the supplementary set,
  `lftr` fails `validate` (327 articles, 0 transitories, 106 references, 2 issues;
  cause not diagnosed). All of these are pre-existing and reproduced identically
  by every run; none was fixed here. `cjm` and `laero`
  fail adapter-frozen article counts (954 versus 952 and 109 versus 100), tied
  to the heading-leak holds recorded in
  `docs/heading-leak-permanent-acceptance-2026-10-21.md`. The other causes were
  not diagnosed.
- Coverage: 355 of 500 committed instruments replay. Of the 145 that do not,
  32 are NOM standards with no adapter (covered by the sweep through
  `standards validate` and `standards refresh`) and 113 have no retained source in
  the main checkout. A hash search over 77,479 candidate files in the other local
  work trees recovered 7 of the 113 (both text and source matched), all from an
  earlier replay directory; they form a supplementary frozen set with its own
  runs. **106 instruments remain coverage blockers for parse replay.** The 113
  absent were 111 `diputados`-family and 2 `itf-dcg`; those families are replayed
  through 346 and 7 other instruments, but no claim of complete parse coverage is
  made. `cub-dcg-2005` and `lfiiedb` are replayed but not provenance-matched.
- Date inertness: `derive-temporal` reads today's UTC date. The 354 B1 derive
  outputs resolve commencement dates no later than 2026-01-20. An earlier note
  here said the committed corpus has no date on or after 2026-10-01; that was too
  broad. `corpus/mx/lritf/temporal-analysis-result.json` holds two `end_condition`
  fixed dates of 2027-04-01 (a model-proposed fallback for staged commencement,
  not a derived commencement). Every strict run happened between 2026-10-23T23:42Z
  and 2026-10-24T10:35Z, so that date was in the future for all of them and the run
  date cannot change classification among B3..C1. The scan covered ISO date values
  in committed JSON only, not dates written in Spanish inside transitory text, and
  shows inertness for these run dates only. B and C ran in the same window on the
  same clock, so date dependence could produce false drift but could not hide a
  real difference. The sweep derive outputs were not
  scanned separately (their stdout carries no dates and the roots are not kept).
- Self-replay. An earlier "B1 versus B2 exact" result came from a comparator
  that treated a field missing on one side as a note, and B1 lacked the
  whole-root digest and harness hash. That is legacy evidence and does not
  satisfy G0. The harness now compares strictly (below) and the valid baseline
  pair is the new **B3 and B4**; B2 versus B3 is run with `--allow-legacy-format`
  as supplemental byte-level evidence only (expected exit 2). B1, B2 and the
  killed partial sweep-B1 are legacy. G0 PASS needs strict B3 == B4 on the
  retained, sweep and supplementary runs plus the passing self-test.

### Evidence rules in force for every valid run

- Comparison is strict by default. A required evidence field missing from one or
  both results fails; formats must match; both runs must come from the same
  harness revision; every row must carry its full evidence keys.
- The harness takes its hash when the process starts, re-executes itself from a
  read-only hash-named snapshot, and records whether the snapshot or the binary
  changed during the run (exit 3), whether the harness is the committed blob,
  and whether the live file changed. An edit to the live file is recorded but
  not executed; a later run that picks up the edit has a different revision and
  fails strict comparison.
- `--allow-legacy-format` exists only for older results; it reports incomplete
  evidence and exits 2, never 0. Every comparison file records the comparator's
  own harness hash.
- Acceptance self-test (`prompts/parser-split-ab-selftest.py`): 17 of 17 pass: an
  unchanged control passes, and evidence missing from both or one result,
  missing row keys, a changed executing snapshot and a run from an edited copy
  all fail. Its first run failed three cases because the test launched harness
  copies outside the repository without the documented repository variable; that
  was a defect in the test, fixed in `73f42a983`.
- Queue: one committed driver (`prompts/parser-split-queue.py`) ran B3, B4, then
  C3, C2, C1 (each compared with B4) and stops at the first non-zero run or
  comparison. C3, the final code, ran first so the decisive answer came
  earliest; every required run still happened.

### Strict A/B result (queue complete 2026-10-24T10:35:33Z)

27 of 27 steps exited 0 with no stop marker. The harness blob (`9c329e0c`) and
driver blob (`0c3a762c`) were unchanged throughout. Every one of the 15 runs
reports the harness unchanged, run from the frozen snapshot, committed, binary
unchanged and live source not changed during the run. All 12 strict comparisons
report 0 drift, complete evidence, no legacy flag and the same harness revision on
both sides.

| Kind | Rows | Non-zero stages | Corpus tree (all five runs) | Root tree (all five runs) |
| --- | --- | --- | --- | --- |
| retained | 1771 | 8 | `08f0ca73fb81…` | `009366407bc0…` |
| sweep | 2660 | 0 | `8349dee2eecb…` | `891d4f879950…` |
| supplement | 35 | 1 | `a47619a8d1ed…` | `2657a292bb1f…` |

Pairs: B3 vs B4 (baseline self-replay, binary `5c4a3f29`), then B4 vs C3
(`31e21bc7`), C2 (`17b09ead`) and C1 (`90cc2657`) for each kind. `retained-B2` vs
`retained-B3` with `--allow-legacy-format` exits 2 (1771 rows, drift 0, evidence
incomplete); it is supplemental only. This shows identical output for the 355
replayable instruments, the 32 standards and the 7 recovered inputs; it says
nothing about the 106 instruments with no retained source.

## G1 — modules inside lex-parse (candidate)

`lib.rs`, `terms.rs` and `standard.rs` production code moved into modules with
no predicate change. A normalized line-multiset comparison before and after
differs only in import wrapping and three re-wrapped signatures. The rule
inventory (`parser-rule-inventory.md`) lists every moved rule group, and the
`references` module's contents were checked item by item, because the split
script sent every unlisted item there by default. The advisor accepted the
measured cross-module edges: link depends on grammar only; analysis on grammar
and its own modules; no analysis-to-link edge; `standard_validate` retained in
parse because it re-runs recognition.

## G2 — grammar and analysis crates (candidate)

`lex-grammar` (no project dependency) and `lex-analysis` (core and grammar).
Placements by single owner: `repeal_ordinals` and `ORDINAL_WORDS` to analysis
(it takes a legal `RepealScope`), `canonical_article_id` to link,
`plain_article_number` to validation. `validate_reference(s)` became public,
generic over the hasher, because three cross-stage tests that need the parser
drive them; clippy first flagged the hashers and the fix was to generalize, not
to allow. Tests that build their own inputs moved with their code; the total
count is unchanged. The line-multiset comparison against G1 shows only path and
visibility changes, hasher generics, two test collect annotations and docs.
Production code has no crate-name, path or asset macros, so no emitted string
depends on crate placement.

## G3 — link crate, consumers, provenance (candidate)

`lex-link` (core and grammar). `lex-parse` depends on analysis and link only
through the compatibility facade and the parse-time `initial_*` calls; glossary
code does not call the usage matcher (an earlier scan suggesting otherwise was a
false positive from the test region). `lex-cli` and `lex-db` import the
responsible crates directly, and `lex-db` no longer links `lex-parse`. A
compile-only fixture generated from the baseline's 126 public paths compiles
against both the baseline and the candidate and fails on a wrong path; it proves
paths, not signatures.

Provenance: the builder pins every new crate's sources and manifest, emits one
pinned-directory list, and reruns on any file added to a pinned directory
(verified manually: no change reruns nothing, adding a file reruns the build
script, removing it reruns it). Negative tests cover a changed source in each new
crate through both the inventory path and the hard-coded `BUILDER_FILES` entry, a
changed manifest in each new crate, and unchanged controls. Stale-binary
mismatch and the cargo rebuild trigger are tested separately.

Dependency audit against the plan's ownership table (measured with `cargo tree`):
grammar has no project dependency; analysis and link depend on core and grammar
only; parse depends on core, grammar, analysis and link (initial evidence plus the
compatibility facade); db depends on analysis, core and grammar; no crate in
analysis, link or grammar depends on parse, CLI, DB or export. This matches the
table.

## G4 — native checks, DB builds and parity (C = `1250889ce`)

Native commands ran in a disposable clone at C and left it clean: fmt and clippy
clean; `cargo test --workspace` 425 passed, 0 failed, 0 ignored; `validate lritf`
valid (145 articles, 11 transitories, 126 references, 0 issues);
`validate ifpe-dcg-2021` valid (59, 4, 115, 0 issues). Figures are from the raw
logs. The corpus tree, schemas, adapters, fixtures and decision docs are
byte-identical to B (empty diff); only manifests, crates, docs and prompts changed.

Builds used one disposable root, corpus_commit B, and the frozen binaries:
baseline B/B (release `2ba8d0f9`, database `27f7599c`) and two B/C builds with
builder_commit C (release `0ea9c945`, database `483264b7`). `db verify` passed on
all three with no stale inputs (4960 input files, 208,744 normalized records), all
21 golden queries pass on each, `PRAGMA integrity_check` is ok and
`PRAGMA foreign_key_check` returns no rows on each, the schema (`sqlite_master`,
78 objects) is equal, and the logical content digest is `8cc37360…` on all three.
Golden definitions, the debt seed and schemas have an empty diff from B. A row-level comparison of every table
(`g4_compare.py`, keyed row hashes) shows B/B equal to B/C in every table except
`corpus_release`, whose specification column differs only in `builder_commit` and
`cargo_lock_sha256` (checked on the stored column, not inferred from the manifest).

Enumerated B-to-C differences, and nothing else: release id; physical database
hash; `specification.builder_commit` (B to `1250889ce`);
`specification.cargo_lock_sha256` (the lockfile now lists three more crates). The
two B/C builds are identical in release id, database bytes and manifest bytes;
their `verification.json` differs only in the recorded output directory path.

The plan's "inspect ... for each source family plus all differing records" maps to
the byte-exact A/B with an empty allowlist: no record differs, so none is left to
inspect. The commands are committed as `prompts/parser-split-g4-candidate.sh`,
`prompts/parser-split-g4-native.sh` and `prompts/parser-split-g4-compare.py`.

Not run here, as the plan says: the reader UI integration check (exporter build
and package verification) belongs to the integration commit.

## Tooling defect R-001 (repaired after the queue)

Codex found that an interrupted run could not restart under the same label: the
harness refused because `logs-<label>` existed, and the driver opened each step's
output in write mode, so a same-label resume would also have truncated the
interrupted attempt's record. Nothing was changed while the queue ran. After it
finished the repair landed in `e09aec70d`: `--archive-interrupted` renames the
interrupted logs aside (never deleting them, and refusing a label that has a
finished result) and records the move in `attempts-<label>.json`; the driver gives
every attempt its own output file, keeps the stop marker, and counts attempts in
`queue-state.json`. The self-test gained four case groups and passes 28 of 28 on
the repaired harness. The repair changed the harness revision, so **every gate
run above used harness `fa321e46…` (blob `9c329e0c`), and the repaired harness
(blob `24c9534b`) produced no gate evidence.**

## Found defects

- **D-001 (baseline).** `check_builder` accepted a builder commit that adds a
  `.rs` or `.json` source file to a pinned directory, despite a comment saying
  unknown sources are rejected. Fixed in the isolated commit `1250889ce` with a
  test that fails when the check is disabled. This is the only behaviour change
  in the whole task and is a tightening of provenance acceptance; it does not
  change release content. The operator may accept or revert it alone.

## Deliberate remaining coupling

- `lex-parse` stays pinned in the DB builder although `lex-db` no longer links
  it, so a change to `lex-parse` alone now changes builder identity.
- `validate_standard` stays in `lex-parse` (rule inventory PA-09).
- Citation regression tests stay in `lex-parse`: they build provisions through
  parser fixtures, and the dependency from parse to link points forward.

## Process note

Validation figures here come from raw logs and written `validation.json` files.
Two `mech` hand-back summaries swapped the `lritf` and `ifpe-dcg-2021` labels
(`lritf`: 145 articles, 11 transitories, 126 references; `ifpe-dcg-2021`: 59, 4,
115); the raw evidence agrees across every tree.

## Next action

G5: bounded Opus read-only final pass, fixes for in-scope findings, then the
commit-bound Codex handoff.
