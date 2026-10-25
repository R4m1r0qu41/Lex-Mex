# Post-split corpus coverage and query repair

## Purpose and immutable baseline

2026-10-25. Operator asks to tackle retained-input gaps, existing replay failures,
structural/temporal debt and the rejected public-query experiment. Starting HEAD:
`e3ad94ee3bf6e5a5ec34274ad77af10ce88bcf1d`. Preserve existing plan/report edits
and unrelated `docs/lexmex-retest-2026-10-24.md`. No commit, merge, push or
publication is implied by this plan. Canonical decisions and repository rules win.

Outcome: provenance-correct replay inputs, classified/repaired baseline failures,
source-backed separation of already accepted structural headings, and an
independently accepted bounded query implementation on current main. These are
separate deliverables with separate acceptance evidence; none waits for complete
legal/temporal review of the whole corpus. DuckDB is an optional analytics spike,
not a dependency of these repairs.

## Authoritative source set

- `docs/parser-split-head-review-2026-10-24-evidence.json`: exact106-item annex.
- `docs/parser-repair-readiness-2026-10-25-evidence.json`: fresh bounded inventory,
  expected hashes/URLs, local recovery locations and14 active debt rows.
- Committed `corpus/mx/<slug>/source-manifest.json`, relevant adapters and
  `adapters/shared/_known-structural-debt.json`; accepted decisions remain immutable.
- `docs/parser-split-implementation-evidence.json`: historical failed stages.
- `docs/public-query-branch-review-2026-10-24.md`: confirmed query defects.
- `docs/plans/scjn-invalidity-registry-integration.md`: deferred evidence scope.

## Current checkpoint

106 absent retained-input cases remain historical coverage gaps. Fresh bounded
search checked SHA-indexed retained evidence plus slug-associated source/text
candidates in the three permanent project .work roots:15 files,21,955,576 bytes.
It located no exact pairs for the106 cases. This is not a new exhaustive disk
search and says nothing about official URLs still serving the pinned edition.

One of the two provenance-mismatched replay sources is recoverable: `lfiiedb`.
Its exact PDF/text pair matches the current committed manifest and has been
copied/hash-verified into `.work/parser-repair-20261025/recovered/lfiiedb` with
a manifest copy. Corrected-input parse and validation now pass on frozen B and C3 with byte-identical
output files:141articles,1transitory,27references,0issues. See
`docs/lfiiedb-corrected-input-replay-2026-10-25.json`. No new HEAD binary was
built; C3 parser sources equal HEAD. No corpus changes were applied.
`cub-dcg-2005` still lacks a located exact pair;
a newer edition must not replace its baseline evidence silently.

Active structural debt is14 rows across4 instruments: laero9,lfd3,lfdo1,lfpa1.
Three prior retirements remain intact. All14 are merged headings with existing
accepted splitting decisions; this does not authorize inferring commencement
or changing source ownership. Technical splitting and legal review are distinct.

## R0 — provenance-correct recovery and failure triage

1. Replay lfiiedb from the staged exact pair in disposable baseline and current
   roots. Use the same adapter/catalogue/options and preserved binary identities;
   reconstruct frozen binaries from the compressed archive if necessary. Inspect
   parsed records, validation, source manifest, queue and Markdown. Record exact
   commands, tool versions, hashes, exits and canonical diff. Do not rewrite the
   original split evidence: produce a new corrected-input result generation.
2. For each106 missing case and cub-dcg-2005, use the inventory's expected hashes
   and URL to seek an exact archived pair first. Any official fetch goes to a new
   acquisition directory and records URL/time/headers/hash. A current PDF whose
   hash differs is a new-edition candidate, not recovered baseline input. Keep
   separate statuses for exact pair recovered, source only, changed edition,
   unavailable URL and awaiting extraction-tool reproduction.
3. Reproduce the9 historical nonzero stages in disposable roots: cjm validate;
   cub-dcg-2005 validate; cucb-dcg-2004 validate; laero validate; lfiiedb parse;
   oaac-dcg-2009 validate; oga temporal-request; scap-dcg-2012 validate; lftr
   validate. Classify each as wrong input, actual parser bug, adapter expectation,
   intentional hold, temporal-request precondition or unresolved legal question.
   A nonzero exit alone is not a parser defect. Use corrected inputs first.
4. For genuine technical defects, add a source-backed positive fixture and nearest
   negative/ambiguous case, make the narrow responsible-crate repair, and replay
   affected families. Do not relax expected counts merely to pass validation.

Gate: every case has a disposition with exact evidence; recovered cases have
correct-input baseline/current comparison. Report unresolved edition/legal
choices individually. A missing input cannot be marked fixed by downstream parity.

## R1 — retire structural debt by four instrument clusters

Inspect only each accepted containing provision and retained official source:
- LAERO:73 DECIES through73 OCTODECIES embedded in73-nonies (9).
- LFD:257 Sextus, Séptimus and Octavus embedded in257-quintus (3).
- LFDO:12 Quintus embedded in12-quater (1).
- LFPA:69 C Bis embedded in69-c (1).

Prefer a shared narrow heading-recognition rule with explicit preconditions;
use an exception profile only for an irreducibly source-specific layout. Do not
create a broad numeral rule that mistakes body references for new articles.
Preserve exact source spans/text, prior IDs, review decisions and ownership.
For each cluster, inspect ID/order/text/reference/term/temporal diffs and retain
any legal ambiguity as a hold rather than guessing. Do not bulk-admit changed
publisher editions encountered while finding input files.

Gate: expected standalone IDs exist without unrelated losses; regression and
affected E2E checks pass; accepted debt rows retire through the existing immutable
history mechanism with resolving commit/source evidence. Source commits used by
retirement records require scoped authorization before producing final seeds.
Use normal merges to preserve debt provenance. Active+retired remains the exact
accepted identity set. Technical hold clearance never implies a legal ruling.

## R2 — temporal and source holds

Build an instrument-specific queue from validation/review evidence after R0/R1.
Separate deterministic missing metadata or source events from legal interpretation.
No global commencement fill or machine approval of unresolved reviews. Request
JRH only for an explicit unresolved question, citing the exact text and formal
DOF act needed. Counts of structural debt, legal reviews and missing inputs must
remain distinct;14 structural rows do not measure all legal work.

Gate: every touched legal state retains reviewer, date, rationale, source and prior
proposal. No existing audited decision is overwritten. Unresolved cases stay held.

## R3 — public-query repair on current main

Use an operator-created isolated worktree if simultaneous writers exist. Preserve
`feat/public-query` at `d2170f9fd`; do not rebase accepted integration history or
import its old Reader wholesale. Port only reviewed contract/query components.

- Fix parsed HTML exact-text mismatch for leading newline, CRLF and bare CR;
  compare real DOM text and UTF-8 hash with canonical bytes, including no-JS output.
- Validate decoded cursor shape before field access: null/scalars/arrays/malformed
  values produce400, not503; release/scope mismatch fails predictably.
- Reject normalized-empty search before execution; accents-only input must never
  become an unrestricted full-corpus search.
- Unify source-URL policy across Rust/JS/static output after inventorying committed
  HTTP URLs. Preserve recorded official URLs; never silently rewrite them to HTTPS.
- Use bounded database-backed lookups/FTS rather than treating the four-instrument
  in-memory adapter as a federal backend. Explicitly document literal/token,
  accent, ranking and pagination semantics; preserve missing/held qualifications.
- Bound term-page output and result/excerpt work; avoid one file per unrestricted
  corpus word and repeated normalization inside sort comparisons. Cache response
  validators only with equivalent rejection behavior.

Gate: confirmed defect regressions fail old code and pass new code; contract,
exact-text, invalid-input and release-bound cursor tests pass. Benchmark representative
full-corpus queries with measured time/memory/output and recorded limits, not a
fabricated latency promise. Revalidate against accepted DB rows/goldens. Existing
R7 package/UI remains unchanged unless a separately reviewed interface change
requires otherwise. Independent audit precedes integration or publication.

## Shared validation and recovery

Before code/canonical commits: fmt, strict workspace Clippy, workspace tests,
locked LRITF andIFPE validators in disposable roots. Run affected E2E stages and
inspect manifests, reports, canonical diff, queues and Markdown. Run DB verify
and21 goldens after DB-facing changes. For accepted corpus changes, create two
fresh committed-input DB projections and prove reproducibility; do not overwrite
accepted releases. For tooling-only edits, run their bounded checks instead of
repeating the complete split queue. New source-family behavior requires its own
coverage rationale, not a reuse of the old zero-drift claim.

On failure preserve evidence and stop dependent stages. Never edit official text,
weaken goldens, replace a seed's historical decision text or conceal a changed
edition. Recover in a fresh disposable root; no reset/clean of another writer.

## Analytics decision: DuckDB alongside rusqlite

`crates/lex-db/Cargo.toml` pins rusqlite0.40.2 with bundledSQLite; queries.rs opens
read-only and sets query_only. DuckDB is not a workspace dependency. Keep the
accepted SQLite release as the serving/query projection and committed corpus as
canonical. DuckDB can be an optional analytical consumer for coverage summaries,
reference/term joins and release-to-release comparisons. Its columnar/vectorized
engine targets scans and aggregation, not a guarantee that this corpus needs
another engine. Attaching SQLite does not convert its storage to columnar form.

If justified, benchmark a separate process against a disposable accepted-release
copy, then materialize selected typed tables into DuckDB or Parquet. Record
release ID/logical digest/source SHA, tool/extension versions, types, counts and
query-result parity. Measure cold/warm queries and import cost against SQLite.
Never write back to accepted SQLite, infer legal dates, or interpret missing dates
as effective dates. Keep it separate from the Rust serving binary initially;
DuckDB's SQLite extension warns about multiple linked SQLite copies in one process.
No dependency or installation is authorized by this recommendation alone.

Primary references checked2026-10-25:
- https://duckdb.org/why_duckdb (analytical/columnar-vectorized design)
- https://duckdb.org/docs/2.0/core_extensions/sqlite (SQLite access, type mapping,
  Parquet export and linked-SQLite caveat)

## Next action

Inspect the pinned cub-dcg-2005 source identity and existing heading-leak hold
before acquiring any edition, then triage its historical validation failure.
The lfiiedb wrong-input failure is resolved in new replay evidence; the original
split evidence remains historical. Broad acquisition and query implementation
remain queued under the gates above.
