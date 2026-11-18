# DuckDB bounded analytics MVP

## Purpose and baseline

Operator request2026-10-25: write a minimal bounded test to run in an isolated
`.delta` worktree. Starting main HEAD05d33e11b (verify drift before execution).
This plan authorizes no worktree creation, canonical changes, production
integration, migration, commit, push or deployment. Use an operator-created
linked worktree. Preserve simultaneous writers and all existing main edits.

Question: does native DuckDB offer a useful analytical benefit for this corpus,
relative to existing indexed SQLite, with exactly equal query results?

Outputs: fresh ignored experiment directory containing source snapshot, five CSV
projections, native DuckDB file, engine receipts, exact query SQL and report.json.
No new crate, Cargo dependency, schema or server. Python dependency is opt-in and
pinned to DuckDB1.4.3, a confirmed available version, not a latest-version claim.

## Minimal structure

- `prompts/duckdb-bounded-benchmark.py`: guards, snapshot/export, isolated engine
  workers, four SQL queries, result parity and measurement report.
- `prompts/duckdb-benchmark-requirements.txt`: isolated Python environment pin.
- `prompts/duckdb-benchmark-smoke.py`: synthetic cross-engine/identity checks.
- This plan: run commands, gates, interpretation and recovery.

SQLite is opened read-only and query_only. Its immutable snapshot is hash-checked;
nonempty WAL/journal sidecars cause refusal. The database SHA, embedded release ID
and specification must match the provided manifest. Input is assumed an already
accepted release; this runner is not a replacement for `lex-mex db verify`.

Selected columns come from instruments, provisions, reference_edges,
defined_terms and term_usages. All are existing non-null TEXT columns. CSV import
specifies VARCHAR explicitly, refuses unexpected null/non-text/sentinel values,
and checks imported counts. No SQLite extension or network autoload is used:
DuckDB reads generated CSVs into its own native storage. JSON temporal states,
legal dates, provision text and original database are not rewritten.

Queries: provision counts/text lengths by instrument; citation destinations and
resolution status; term usages joined to definitions by owner; citation origin/
destination counts. The last query includes provision-anchored edges only;
title-anchored edges are not silently counted as provisions. All result sets are
ordered and compared via canonical JSON SHA-256; any mismatch fails the run.
These are analytics workloads, not FTS or authoritative legal-query tests.

## Fixed bounds

- Maximum source SQLite size1GiB; maximum500,000rows per selected table.
- One thread and256MB DuckDB memory setting;256MB temporary spill allowance.
- Each engine worker has120second timeout (operator can lower or set at most300).
- Four queries, each once as first query and three warm repetitions.
- Fresh output only; never overwrite any prior result or release.
- No downloads during benchmark execution; only explicit environment setup uses PyPI.

DuckDB's buffer setting is not a total process RSS ceiling. Peak process RSS is
measured for each worker separately. Copy/export/hash time is outside worker
timeout; source/table bounds constrain that setup. First queries are process-fresh,
not OS-cold: copy/export has already warmed filesystem cache. Do not flush system
caches or describe timings as controlled cold-disk comparisons. SQLite has the
accepted indexes/full schema; DuckDB has a five-table projection and no added
indexes. File sizes are footprints of different projections, not compression ratios.

## Run in the operator-provided worktree

Open a terminal at that isolated `.delta` linked worktree. The runner requires
both `.delta` in the resolved path and a `.git` file, and restricts output to its
own `.work/`. It refuses the primary checkout.

If this uncommitted experiment packet is not yet in that worktree's branch,
copy exactly the three prompt files from `/Users/jr/Dev/lex-mex/prompts/` and this
plan into their same relative paths. Preserve worktree instructions/other edits;
do not copy the old experimental query branch or its Reader. No commit is required
to run the packet; report.json hashes the executed runner and records worktree HEAD.

```sh
pwd
git status --short
git rev-parse HEAD
python3 -m venv .work/duckdb-venv
.work/duckdb-venv/bin/python -m pip install -r prompts/duckdb-benchmark-requirements.txt
.work/duckdb-venv/bin/python prompts/duckdb-benchmark-smoke.py
.work/duckdb-venv/bin/python prompts/duckdb-bounded-benchmark.py \
  --source /Users/jr/Dev/lex-mex/derived/db/permanent-5eca-a/afdc5acf0b5017d73cd7b8e0b8f8fc5962914b56de26467e42b0c77fd2ef1974/corpus.sqlite \
  --manifest /Users/jr/Dev/lex-mex/derived/db/permanent-5eca-a/afdc5acf0b5017d73cd7b8e0b8f8fc5962914b56de26467e42b0c77fd2ef1974/manifest.json \
  --output "$PWD/.work/duckdb-benchmark-01"
```

If the accepted release path has moved, obtain its actual manifest/database pair;
do not guess another file or relax the hash gate. Use a fresh02 directory for a
repeat. No network source acquisition or Rust build is needed.

## Acceptance gates and decision

G0: worktree/instructions/status verified; input accepted externally and matching
manifest; dependency pin installed; synthetic checks pass. Stop on any failure.

G1: full experiment exits0, status.json and report.json say pass, all table counts
and four query result hashes agree, original/snapshot hashes remain unchanged.
Failure leaves status/logs and outputs in place and exits nonzero; do not reuse
the failed output directory or interpret its partial timing results.

G2: review process-first and warm medians, import/export cost, native DuckDB size,
separate peak RSS and workload assumptions. Repeat in a fresh directory only if
it resolves noise/uncertainty. No arbitrary minimum speedup is an acceptance rule.
A parity pass can correctly lead to declining DuckDB if the practical benefit
is too small. A faster scan alone does not justify a production migration.

G3: report recommendation, measured tradeoffs, tool/source identities and limits.
Any future Rust integration/Parquet publication is a separate reviewed change.
The committed corpus stays canonical; accepted SQLite remains the serving/query
projection. This experiment cannot resolve missing parse inputs or legal reviews.

Recovery: retain failed receipts/logs. Restart with fresh output after bounded
repair, never overwrite an accepted release or reset another writer's branch.

## Validation actually performed — 2026-10-25

In a disposable `/private/tmp` Python environment, pinned DuckDB1.4.3 installed.
Synthetic checks passed actual SQLite and DuckDB queries over Unicode, CRLF,
leading newline, empty text, joins and a title-anchor case. Existing-output,
wrong-manifest and embedded-identity refusal checks passed; failure receipt verified.
Runner --help passed. Invoking the runner from main exited2 before creating output.
Diff whitespace check passed. A mistaken --help invocation against the smoke
script (which is executable checks, not a CLI) failed under system Python lacking
DuckDB; the correctly specified runner help check then passed. No corpus benchmark
was executed or speedup measured. No `.delta` worktree was created by Codex.

## Current next action

Run G0/G1 using the commands above in the operator-created `.delta` worktree,
then return report.json and both worker receipts for review. Do not integrate
DuckDB into production based on synthetic smoke results.
