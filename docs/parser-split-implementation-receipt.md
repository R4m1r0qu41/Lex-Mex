# Parser split implementation receipt

Living narrative for [the execution plan](plans/parser-responsibility-split.md);
machine evidence is in `parser-split-implementation-evidence.json`. Status:
**IN PROGRESS**. G1, G2 and G3 are candidates awaiting their A/B comparisons;
nothing is accepted, merged, pushed or deployed.

## Where things stand

| Gate | Status | What exists | What is still required |
| --- | --- | --- | --- |
| G0 | IN PROGRESS | harness, frozen baseline B binary, B1 replay (1771 rows), coverage manifest, supplement set | B1 versus B2 self-replay, baseline sweeps S-B1 and S-B2, advisor acceptance of coverage |
| G1 | candidate | commit `07cfa5536`, binary C1; advisor accepted the symbol and dependency map, comparator allowlist empty | C1 A/B exact |
| G2 | candidate | commit `fd70e4370`, binary C2 | C2 A/B exact |
| G3 | candidate | commits `5c074e21d` and `1250889ce`, binary C3 | C3 A/B exact, advisor review of the final map |
| G4 | not started | none | DB builds, verify, goldens, B/B versus B/C parity |
| G5 | not started | none | advisor final pass, commit-bound Codex handoff |

Commits on `refactor/parser-split` (baseline B = `85c41f389`): `ff38d1882`,
`07cfa5536`, `f1da6f1e7`, `fd70e4370`, `5c074e21d`, `1250889ce`. Binary hashes
and per-tree native results are in the evidence file.

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
  empty. The harness hash is recorded when a run ends, not when it starts (see
  the caveat in the evidence file).
- Retained replay B1 finished: 1771 rows over 355 instruments. Baseline facts the
  candidates must reproduce exactly: `validate` exits non-zero for `cjm`,
  `cub-dcg-2005`, `cucb-dcg-2004`, `laero`, `oaac-dcg-2009` and `scap-dcg-2012`;
  `lfiiedb` fails at `parse` (its retained text does not match the committed
  manifest hash); `oga` fails at the temporal-request stage. `cjm` and `laero`
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
- Date inertness: `derive-temporal` reads today's UTC date. The committed corpus
  has no relevant date on or after 2026-10-01, and the 354 B1 derive outputs
  resolve commencement dates no later than 2026-01-20, so the run date cannot
  change them. The same scan of the sweep outputs is still pending.
- Self-replay: a one-instrument replay and a one-instrument sweep were each exact
  across two runs. The full B1 versus B2 comparison is pending.

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

When retained-B2 finishes, compare it with retained-B1 (the G0 self-replay gate)
and read each candidate comparison as soon as it lands; stop the queued later
runs on any drift.
