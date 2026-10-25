# Public-query repair R3, slice 1 — 2026-10-25

First slice of R3 in [post-split corpus and query repair](plans/post-split-corpus-and-query-repair.md),
done regression-first in an isolated worktree, `/Users/jr/Dev/lex-mex-public-query-repair`
(branch `repair/public-query-contract-2026-10-25`, from main `e3ad94ee3`). **Nothing is committed,
merged, pushed or published.** `feat/public-query` (`d2170f9fd`) is untouched. Review tier: advisor
did not review this slice; no independent review yet. Findings come from the
[2026-10-24 branch review](public-query-branch-review-2026-10-24.md).

## What was ported

Only the contract and query components, copied from `feat/public-query` onto current main with no
history import: `query-contract`, `query-memory`, `query-http`, `query-fixture`, `query-preview`,
`query-retained`, `agent-pages` (with their tests), the two query fixtures,
`schemas/public-query-v1.schema.json` and the Rust contract `crates/lex-core/src/public_query.rs`. The old Reader UI, packager, stager and server were not
imported, so the approved R7 publication boundary and files are unchanged. Baseline before any fix:
the ported tests pass on main (16/16).

## Confirmed defects: test first, then fix

Five new Node tests were written and **all failed on the ported code**; they pass after the fixes. One more Rust test covers the URL policy.

| Finding | Fix | Test |
| --- | --- | --- |
| P1 exact text: parsed HTML text differed from the canonical bytes for a leading newline, CRLF and bare CR | `agent-pages.mjs` escapes CR as `&#13;` and wraps article text in a span inside `<pre>` (the HTTP view already did) | Parsed-text test over five inputs. **Also confirmed in real Chrome**: `DOMParser` text equals the canonical string for `"\námbito\r\nsegunda línea"`, `"\r\ncrlf inicial"`, `"uno\rdos\rtres"`, `"\n\nsolo saltos"` |
| P2 cursor: base64url of JSON `null`, a scalar or an array reached property access and returned 503 | The memory adapter requires the decoded cursor to be a plain object before reading it | Null, number, string, boolean and array cursors give 400 `bad_request` on catalog and search, JSON and HTML |
| P2 search: input such as a lone U+0301 normalized to empty and returned all provisions | The handler and the adapter reject normalized-empty queries (400 / `Invalid search`) | `q=%CC%81`, space-padded and doubled forms give 400; the adapter throws |
| Source-URL policy: contract and schema required `https`, static pages accepted any `http(s)` including credentials | One rule in all three layers: `http` or `https`, no username or password, no other scheme. JS `isOfficialSourceUrl`, the Rust `validate_response` check and the schema pattern `^https?://` | Node contract and static-page tests and a Rust test accept recorded `http`/`https` and reject credentials, `ftp`, `javascript:` and `//host` |

**Policy basis.** The inventory of committed source manifests shows 467 `https` URLs and one `http`
(`reg-lgs-insumos`: `http://www.oag.salud.gob.mx/descargas/LV/56-31052021.pdf`). Recorded official URLs are
preserved as published and never rewritten to `https`, per the plan.

**Performance, behavior-preserving.** The memory adapter now normalizes each provision once, not inside
sort comparisons, and builds excerpts only for the returned page. In Rust, the compiled schema is built once
(`OnceLock`) instead of per response; a build failure is cached and reported on every call, so rejection
behavior is unchanged. The existing fixture and HTTP tests pass unchanged; no timing claim is made.

## Checks

In the worktree (via the mech subagent; its wrapper exit status reflects `tee`, so the printed results are the evidence):
`cargo fmt --check` pass; strict workspace/all-target Clippy pass on the default toolchain and on 1.99.0;
`cargo test --workspace` 434 passed, 0 failed (the 430 on main plus 4 `public_query` tests);
`node --test reader/` 46 passed, 0 failed (R7's 25 plus 21 query/agent-page tests). Not run: the R7 browser
harness, locked LRITF/IFPE validators, any database or federal-scale benchmark.

Patch with the whole slice: `.work/parser-repair-20261025/public-query-slice1.patch` (SHA-256 `1bcb47d7d6c09b28e5a4870b2acca31ea1f006e808ea7254bf780d9a0f6e7aef`).

## Not done (R3 remainder)

- **Database-backed serving.** The adapter is still the bounded in-memory one. Bounded lookups and FTS against the accepted
  SQLite release, with documented literal/token, accent, ranking and pagination semantics, are not built.
- **Output bounds.** Term pages are still one file per distinct word with no corpus-wide budget.
- **Benchmarks** of representative full-corpus queries and **revalidation against accepted DB rows and goldens**.
- **Independent audit** before integration or publication.

## Slice 2 — database-backed adapter (same worktree, uncommitted)

Added `reader/query-sqlite.mjs` (read-only `node:sqlite`, `query_only`), `reader/query-sqlite.test.mjs` and
`reader/query-sqlite-bench.mjs`. Patch for slices 1 and 2: `.work/parser-repair-20261025/public-query-slice2.patch`
(SHA-256 `51ea10f5…`). It serves the **candidate** projection built from branch tip `ef28b9619`
(release `bb8a77dd…`), not the earlier accepted release, and it is not committed or published.

**Semantics, stated in the module header.**
- *Search* normalizes the query (lower case, accents removed), splits it into letter/digit tokens (1 to 12) and matches
  them as one adjacent-token phrase against label, heading and text through the FTS5 index (`unicode61`, diacritics
  removed). Punctuation and case are ignored; partial words do not match. A query with no token after normalization is a
  400, so an accents-only query can never become an unrestricted search.
- *Ranking:* article, transitory, annex; then bm25; then provision id.
- *Pagination:* opaque cursor bound to release, route and query; reachable depth is capped at 1,000 results. A deeper,
  foreign or malformed cursor is a 400.
- *Public numbers* are the URL slug taken from the provision id (`78-terdecies`), because the stored display number
  (`78 Terdecies`) is not URL-safe in 9,389 provisions.
- *Not addressable:* 68 provisions (63 transitories such as `transitory:1º`, 5 articles) have ids outside the
  `[a-z0-9-]` public shape. They are left out of search and index, and the coverage notice states the count; they are
  never guessed. Missing or unknown provisions return null and the transport answers 404.
- Output is built from named columns only; stored `extras` and review payloads never leave the module.

**Correctness (15 tests; 61 of 61 Node tests pass with the database, 46 pass and 15 skip without).**
Exact text and digest over a sample of 150+ provisions equals the stored rows byte for byte; search membership for
`obligación` equals an independent full-corpus tokenization (2,134 hits, golden G7); accent and case variants return the
same ordered hits; phrase and partial-word behavior; instrument filter; stable, bound and capped pagination; the catalog
is exactly the 468-row instrument table; 13 malformed-request classes are 400 and never 503; responses stay within the
16 KiB budget and never contain stored extras; golden G3 historical citations keep their status with no live target;
resolved edge targets exist; per-source edge counts equal the stored edges; golden G6 articles are served standalone.
The tests are gated on `LEX_MEX_QUERY_DB` because the database is not committed.

**Benchmark** (Node v26.8.2, this machine, 15 runs per workload, warm = after the first run;
peak process memory 78 MB). Measurements, not a latency promise.

| Workload | Response bytes | Cold ms | Warm median ms | Warm p95 ms | Warm max ms |
| --- | ---: | ---: | ---: | ---: | ---: |
| catalog first page | 1,567 | 9 | 0.1 | 0.2 | 0.2 |
| article lookup (lac 78 Terdecies) | 1,658 | 1.1 | 0.1 | 0.2 | 0.2 |
| search rare token: terdecies | 2,163 | 1.6 | 0.2 | 0.4 | 0.4 |
| search phrase: contrato de seguro | 4,592 | 8.1 | 0.8 | 0.9 | 0.9 |
| search accented: obligación (2,134 hits) | 4,520 | 24.8 | 5.7 | 6.1 | 6.1 |
| search common: ley | 4,881 | 48.5 | 51.9 | 55.8 | 55.8 |
| search stop-word: de | 5,391 | 208.4 | 130.6 | 137.9 | 137.9 |
| search filtered: contrato in lscs | 4,551 | 4.5 | 1.7 | 1.9 | 1.9 |
| index of ccf, first page | 4,244 | 0.5 | 0.2 | 0.2 | 0.2 |
| references of a provision | 1,084 | 1.7 | 0.1 | 0.1 | 0.1 |
| search deepest reachable page: ley (offset 990) | 4,882 | 78.5 | 78.2 | 82.3 | 82.3 |

Cost follows the matched set: a stop-word query sorts every match by bm25, so output is bounded by the depth cap but
time is not. The worst measured case is about 0.2 s cold. Whether to cap candidates, drop stop-words or reject
single-token queries above a hit threshold is left as a decision.

Revised "Not done": the database-backed adapter, output bounds on search, tests and the benchmark above now exist for
the *candidate* projection. Still open: static agent-page output bounds (one file per distinct term), an acceptance
rerun against the *accepted* release and its goldens (two goldens differ on the candidate, see the corpus record),
the transport routes in a deployment, and **independent audit** before any integration or publication.
