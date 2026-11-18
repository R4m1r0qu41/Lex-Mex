# DuckDB bounded analytics

Checked 2026-10-25. This directory is the record of that check. It is not a
production dependency, and it does not make DuckDB, DuckLake, or Parquet part
of the corpus.

## Decision

A columnar store would not add value to this corpus. SQLite remains the
serving and query projection. DuckDB was not integrated. DuckLake was not
introduced.

SQLite and DuckDB do answer different kinds of questions. SQLite answers row
and graph questions: the provision text, the citation, the definition, and the
provenance and review state attached to that row. DuckDB answers column scans:
counts, rates, and cuts that skip the rest of the row. DuckLake is a catalog
of Parquet snapshots for several consumers over object storage.

The accepted release is none of those analytical cases. It has 468 instruments,
68,341 provisions, and 35,930 reference edges. That is a small relational
database. The legal questions need the whole row. A census of a release can be
computed in either engine, and it creates no new legal fact. This corpus
already has its snapshot mechanism: one immutable SQLite file, a manifest, and
a hash. A lake catalog would not make the text more complete or the citations
more reliable.

An analyst extract, rebuilt from an accepted SQLite release and discarded with
it, would be a separate review. It would be justified only by a question about
the corpus itself, such as how citation or coverage counts move between
releases. This run did not find that workload.

## What was measured

The question was whether native DuckDB, given exactly the same rows, returns
the same aggregates as indexed SQLite and whether that difference is useful.
It was not a search benchmark, a full-text benchmark, or a test of legal
inference.

Input was the accepted release `permanent-5eca-a`, release id and database
SHA-256 `5da6e800e4d3b362411c561d9ae67fbfe012abe3164fce30bbb8a21c9e249bf2`.
Logical content SHA-256
`8cc3736018bffae94f303d9cbfa378170c9ff0e2842ce708431f08d528b60359`.
The source and the snapshot still matched that hash after the run. The runner
opened SQLite read-only and `query_only`. DuckDB read five CSV projections
into its own file, with one thread, a 256 MB buffer, and a 256 MB spill cap.
No SQLite extension was loaded.

The projection was `instruments.id`; `provisions.id`, `instrument_id`, `text`;
`reference_edges.source_provision_id`, `target_instrument_id`,
`resolution_status`; `defined_terms.id`, `instrument_id`; and
`term_usages.provision_id`, `term_id`. All selected columns were non-null
text. The citation-origin query joins edges to provisions, so a title-anchored
edge is not counted as a provision.

Counts agreed on both engines: instruments 468, provisions 68,341, reference
edges 35,930, defined terms 5,766, term usages 97,672. All four result
SHA-256 values agreed. `receipts/pass/status.json` says `pass`.

Warm median seconds. The copy and the CSV export had already warmed the
filesystem cache, so these are not cold-disk times.

| Query | SQLite 3.53.4 | DuckDB 1.4.3 |
|---|---:|---:|
| provision_volume | 0.0576 | 0.0099 |
| citation_destinations | 0.0112 | 0.0006 |
| term_usage_by_owner | 0.0054 | 0.0027 |
| citation_origin_destination | 0.0285 | 0.0046 |

SQLite's first in-process runs were 0.0605, 0.0112, 0.0057, and 0.0284
seconds. DuckDB's were 0.0216, 0.0007, 0.0036, and 0.0049 seconds. DuckDB
then spent 0.252 seconds importing before any of those scans. Its worker took
0.399 seconds in all and peaked at 266.1 MiB. The SQLite worker took 0.431
seconds, imported nothing, and peaked at 35.6 MiB. CSV export took 0.822
seconds. End-to-end wall time, including the snapshot copy, was 2.494 seconds.

The native DuckDB file is 95.5 MiB. The SQLite snapshot is 459.7 MiB. Those
are different projections, not a compression ratio. SQLite keeps the accepted
indexes and the full schema. DuckDB stored the five tables above and no added
indexes. The 256 MB setting is DuckDB's buffer limit, not a process cap; the
measured resident set was higher.

Every SQLite scan was already under 61 ms. The absolute gain is smaller than
the export that fed DuckDB.

## The failed attempt

`receipts/fail-sniffer/` is the first corpus run. DuckDB 1.4.3 sniffed `escape`
as empty because its sample contained no doubled quote, then rejected a later
RFC 4180 `""` in `provisions.csv`. The worker exited before any DuckDB query.
The source hash was unchanged. Timings from that directory are not evidence,
and its partial SQLite receipt was not copied here.

The repair is confined to the opt-in runner. `load_projection_csv` pins the
`csv.writer` dialect: no sniffing, comma separator, `"` quote and escape,
`\r\n` newlines, null string `\N`, and quoted nulls refused. The smoke script
checks a quote-free prefix of 2,046 comma-rows followed by one doubled-quote
field. That is a harness fix. It is not a change to the corpus or to SQLite.

## Identities

| Item | Value |
|---|---|
| Pass receipt | `receipts/pass/` |
| Worktree HEAD recorded by the runner | `a07a93f31a726c70adecd0fba53c84b3105f2e10` |
| Python | 3.14.7 |
| Platform | macOS 26.5.1, arm64 |
| DuckDB pin | 1.4.3 (`runner/duckdb-benchmark-requirements.txt`) |
| Requirements SHA-256 | `201b0fb013906656a17fdbdb5e2de992d89de8f49a11cf0e6311b821380822bb` |
| Runner SHA-256 | `c3bed3db5c3ad0668ead374b7e1dfd05b2a9a7bfd515aa2b8e0f6998a88ca088` |
| SQLite library | 3.53.4, as reported by the pass receipt |

The runner recorded the worktree HEAD above. This commit is on the primary
checkout, which had moved on. The benchmark did not execute those later
commits. It read the pinned release named in the pass receipt.

The pass receipt's `source` field is the local absolute path of that release.
The database file is not in Git.

## What is not in Git

The snapshot, the five CSV exports, and `analytics.duckdb` stay in the
worktree's ignored `.work/` directory. Together they are several hundred
megabytes, and they can be rebuilt by the runner. The SQLite release itself
stays in the ignored `derived/db/` directory.

## Limits

This run does not resolve missing parse inputs or legal reviews. It does not
authorize a Rust dependency, a Parquet publication, or a second query
authority. The committed corpus stays canonical.
