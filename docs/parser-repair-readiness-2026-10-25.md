# Post-split repair readiness — 2026-10-25

Started the requested follow-up at HEAD e3ad94ee3. The living execution plan is
[post-split corpus and query repair](plans/post-split-corpus-and-query-repair.md).
It separates input recovery, baseline failure diagnosis, four structural clusters,
legal/temporal review and the isolated public-query implementation.

## First completed correction: lfiiedb replay input

Located exact retained PDF and extracted text matching the committed source
manifest, staged a fresh pair and verified both hashes. Restored the frozen B
and C3 executables from hash-verified compressed evidence. In fresh disposable
roots, both parse and validate exit0:141articles,1transitory,27references,0issues;
24terms/621usages. Resulting instrument output file hashes are identical.
No source/corpus record was overwritten. No new HEAD binary was built; the
parser/grammar/analysis/link source trees at HEAD equal tested C3. This is a
corrected-input result generation, not a rewrite of the historical failed run.

[Replay commands and hashes](lfiiedb-corrected-input-replay-2026-10-25.json).

## Remaining cases

The106 historical missing inputs remain unlocated in the bounded search. Checked
archived SHA inventory and15 slug-associated candidate files (~22MB) under the
three permanent project .work roots, not every disk directory. The second
mismatched source, cub-dcg-2005, still needs pinned-edition recovery. Newer
publisher editions must enter a separate acquisition/ingestion review.

The14 active structural rows are9LAERO,3LFD,1LFDO,1LFPA. Their accepted merged-
heading decisions support technical splitting, but do not resolve temporal or
source-ownership ambiguity. Preserve the three existing retirement records and
retire new rows only through the provenance-bound seed procedure.

Eight other historical nonzero stages remain to classify after lfiiedb's input
correction. Baseline equivalence never establishes that those failures are correct.
The query experiment remains unpublished; exact-text DOM, cursor shape,
normalized-empty search, shared source-URL policy, output bounds and federal-scale
DB serving are explicit acceptance gates. None of those implementation repairs
has been applied by this inventory milestone.

[Inventory, expected hashes/URLs and exact debt rows](parser-repair-readiness-2026-10-25-evidence.json).

## Database recommendation

Confirmed `rusqlite = "=0.40.2"` with `bundled` in crates/lex-db/Cargo.toml.
The read API in queries.rs opens SQLite read-only and enables query_only.
No DuckDB dependency is in the current manifests/lockfile.

Keep SQLite/rusqlite as the accepted lookup, FTS and corpus-projection path.
DuckDB is a reasonable optional consumer for wide scans, reference/term joins,
coverage aggregation and comparisons across releases. Its columnar-vectorized
engine targets analytical workloads ([official design](https://duckdb.org/why_duckdb)).
At68,341 provisions we have not measured a need or a speedup; benchmark actual
queries against SQLite before adding a second engine.

The [official SQLite extension](https://duckdb.org/docs/2.0/core_extensions/sqlite)
can query SQLite and export to Parquet. Attaching SQLite leaves its original
storage row-oriented; materialize selected typed data into DuckDB/Parquet for a
columnar analytical snapshot. Pin release ID, logical digest and source hash;
compare results/counts and preserve NULL/unknown legal status semantics.

Start any optional benchmark in a separate process on a disposable copy, with
no writes to the accepted release. This also avoids bringing two SQLite library
copies into the serving process, a risk called out by the extension documentation.
No engine installation, migration, DuckDB benchmark or speed claim was made here.

Next: recover/classify cub-dcg-2005's pinned source and validation hold, then work
through the baseline failures and four structural repair clusters under the plan.
Unrelated reports/plan edits preserved; no commit, push or deployment.
