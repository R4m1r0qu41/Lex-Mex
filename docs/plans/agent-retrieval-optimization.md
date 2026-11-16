# Agent retrieval optimization

## Purpose and scope

Turn the two 2026-10-24 external reports into a measured path from discovery to
small, exact, source-backed evidence. Preserve canonical text and the accepted
human Reader. This checkpoint is a review and implementation plan, not a new
publication, legal acceptance, or authorization to ingest guidance.

## Baseline and authority

- Reader: `f49c07923`; main: `e3ad94ee3`; DB: `047b421ef`.
- Inputs: `docs/Lex-Mex_IFC_Agent_Retrieval_Benchmark_2026-10-24.md` and
  `docs/lexmex-retest-2026-10-24.md`, retained unchanged.
- Current producer: `reader/package.mjs`, `crates/lex-cli/src/public_release.rs`.
- Main's `docs/public-query-branch-review-2026-10-24.md` governs the unresolved
  experiment at `d2170f9fd7ff8ea179512e496f22fffa2c9ddc1c`.
- Website owner checkout's HANDOFF records version26/current-only publication.
  Git, exact artifacts and live results outrank older reports and capsules.
- The discovered AgentOS capsule concerns an older discovery experiment and has
  missing source bindings. It is not authority for this plan.

## Windowed-reader reservation

Operator decision, 2026-10-24: the windowed-reader spike is coming later. Keep
the idea/branch reservation, but no extra worktree now. Create its worktree when
implementation begins. The permanent top-level layout remains lex-mex,
lex-mex-db and lex-mex-reader. Retrieval work does not require the UI spike.

## Review findings

1. **R7 deployment and retrieval improvements are different deliverables.**
   `builder_sha256` hashes only `public_release.rs` (line389), not Reader assets
   or the complete backend. The packager preserves release text and discovery
   while adding UI assets. An unchanged exporter hash does not prove a failed
   deployment. The retest correctly observes that query/discovery work has not
   shipped; its deployment inference is too strong.
2. **Transport remains client-dependent.** Fresh checks this turn: curl reads
   live llms.txt pointing to c305db4a…c969; current index returns200,
   text/markdown,260bytes. Python urllib receives403 on that same index and
   current manifest; web retrieval reports tool-level Internal Error on root
   and llms.txt. Sandbox curl initially fails DNS, then succeeds outside it.
   These are distinct observations, not proof of origin blocking, a MIME root
   cause, or universal success. Compare headers/response bodies and network
   intermediaries before changing security or content types.
3. **The IFC report is not a current-release transport benchmark.** It names
   3c040166…30660. Host HANDOFF records current-only deployment and old-release
   URLs404. That retention policy conflicts with the proposed permanently
   retrievable citation URLs: either retain previously published immutable
   artifacts or explicitly document expiry/archival resolution. Never redirect
   an old citation silently to new legal text.
4. **Hashed provision URLs are deterministic already**, but agents cannot
   derive them from an article number. Add a bounded citation resolver or
   collision-safe aliases using instrument + provision kind + exact locator;
   retain original IDs/hashes. Bis, fractions and transitories need fixtures.
5. **Small discovery is preferable to loading the manifest.** Serve a compact
   coverage/source summary and paginated instrument navigation; keep the full
   integrity manifest available for verification. Headings and express links
   can be deterministic; editorial topics/aliases must remain labeled derived
   metadata. Do not invent statutory headings or silently truncate provisions.
6. **Reuse the database work; do not create another index architecture.** Inspect
   the DB owner's current acceptance and query interface before adapting its
   FTS. The old public-query branch is not publication-ready: HTML text/hash
   mismatch, null cursor503, normalized-empty broad search and HTTP/HTTPS policy
   mismatch remain. It also has unbounded term-page output and an in-memory
   adapter unsuitable as proof of federal-scale performance. Port only bounded
   query contracts onto current main, not the old UI or whole packager.
7. **Coverage differs from retrieval quality.** CNBV guides may materially help
   these questions, but need their own provenance/version and nonbinding
   guidance classification. First verify the cited guide page and source rights/
   availability. Neither report authorizes canonical ingestion or establishes a
   human-reviewed legal golden answer. No legal claims in either report were
   independently accepted in this engineering review.
8. **MCP is an optional adapter, not a guaranteed bypass.** Tool clients can
   still summarize, truncate or apply output policies. Test exact bytes/hash
   fidelity in actual clients. Public HTTP retrieval must remain useful without
   a connector; no promise of escaping quote policies or egress restrictions.
9. **The token comparison is directional, not controlled evidence.** The retest
   admits warm knowledge and estimates tokens; the IFC pass failed before legal
   retrieval. Separate transferred bytes, tool-visible text, tokenizer counts
   and model billing. Do not claim a measured optimization percentage.

## Milestones and acceptance

Execution authorized 2026-10-24. Parent owns scope, integration and evidence;
GPT-6.1 Sol implements bounded changes and investigates the DB integration;
Luna runs named deterministic checks. Workers own disjoint paths in the existing
Reader checkout; no extra worktree, commit, deployment or canonical ingestion.
The earlier review-only checkpoint below is historical, not the current authority.

- M1 tooling complete: `reader/retrieval-check.mjs` and15 fixture tests; live
  default client passes, controlled Python-user-agent probe403. Cross-client
  acceptance remains open; see `docs/agent-retrieval-baseline-2026-10-24.md`.
- M2/M3 dependency review complete: implementation uses the producer and DB APIs
  specified below. Preserve the publication allowlist until a
  separately verified additive artifact has a defined host integration path.
- M4 queued after M1: cold-client evaluation requires actual independent clients;
  local fixture tests cannot substitute for their results or human legal review.
- M5 optional and deferred until the preceding evidence supports it.

1. Transport baseline: capture GET/HEAD status, redirects, MIME, cache headers,
   bytes, hashes and timing for discovery, current index, manifest and two
   actual provisions. Use curl, an independent network and real agent clients.
   Record environment failures separately. Resolve old-release retention policy.
2. Small static discovery/resolution: coverage summary, bounded law navigation,
   citation resolver, exact article retrieval and provenance. Pin release for
   every traversal; test ambiguous/missing locators and byte/hash agreement.
   No per-word HTML explosion. Keep agent controls off the human canvas.
3. Bounded DB query: repair public-query regressions first, then integrate with
   an accepted read-only DB snapshot. Cap query length, result count, excerpts,
   graph expansion and response size; validate cursor scope and reject empty
   normalized queries. Measure full-corpus latency/memory/output, not fixtures
   alone. Preserve express-edge evidence; separate editorial relationships.
4. Evaluation: cold isolated runs of IFC and IFPE questions, plus negative,
   out-of-coverage and historical-date cases. Human reviewer approves legal
   expectations. Measure requests/failures, recall, citations, versions, bytes,
   tokens and latency at equal answer quality. Long exact articles are allowed;
   token budgets must never silently cut controlling conditions.
5. Optional MCP: reuse the accepted query contract only after HTTP retrieval
   works; separately validate exact-text delivery and client truncation.

For code changes run repository-required Rust checks and affected Node tests,
plus targeted regression fixtures. For release changes validate manifests,
canonical diffs and host integration; owner-account deployment and live client
verification remain separate gates. Stop on source/hash drift or unreviewed
legal/schema decisions. Keep the existing publication as rollback; do not
rewrite immutable artifacts or publish the experimental branch directly.

## DB and producer integration decision — 2026-10-24

The detailed DB workstream is drafted in
[`database-agent-query-integration.md`](database-agent-query-integration.md).
The operator's latest instruction is **draft only**; DB implementation has not
started. Transport acceptance and offline DB development have separate gates.

Read-only GPT-6.1 Sol investigation confirmed these reusable boundaries:

- `crates/lex-cli/src/public_release.rs`: `stem`, `public_provision`,
  `source_info` and instrument assembly already provide canonical IDs, public
  projections, provenance and exact `.txt`. Extend those instead of inventing
  a parallel authoritative representation.
- Main/DB `crates/lex-db/src/queries.rs`: `Reader::open` is read-only;
  `article_ids` resolves instrument/kind/number but may return multiple IDs;
  `lookup` distinguishes qualified absence from ordinary not-found; `search`
  is parameterized literal-phrase FTS, bounded to 512 characters/100 results.
  Never serialize raw lookup JSON: it reconstructs canonical data, not a public
  allowlist. Fractions remain validated spans, not invented canonical records.
- Adapt the old public query contract's 10-result/240-character-excerpt/16 KiB
  response bounds. Full articles are a separate operation. Add public DTOs,
  catalog/reference pagination, bounded excerpts and cursor scope; do not
  materialize all matching full texts before pagination.
- Accepted DB work at `047b421ef` and integrated evidence at `e718da2e5` do not by
  themselves match current public release `c305db4a`. Bind public release, corpus
  commit, DB checksum/manifest and source hashes before serving any DB results.
  If incompatible, build/verify a new snapshot through the existing pipeline.
- The Reader branch lacks the newer main DB integration. Reconcile it with main
  while preserving existing dirty handoff/plan files before changing trusted
  producer/DB boundaries. No experiment branch wholesale merge.

Current-only hosting remains the operator's existing decision; this task does
not reverse it. Document the availability limitation in the next public
discovery revision. A release ID identifies bytes; it is not a promise that
the host retains that release forever. Never replace expired pinned text with
current text at the same URL.

## Historical review checkpoint — 2026-10-24

Review complete; no runtime code, canonical data, host configuration or published
assets changed. Existing dirty/untracked inputs preserved. No build/test suite
was warranted for this documentation-only review. Live checks are limited to
the observations above, not a three-client acceptance result.

## Current execution checkpoint — 2026-10-24

Applied M1 with GPT-6.1 Sol implementation/independent review and Luna6 checks.
40/40 Reader Node tests, syntax and diff checks pass. Live default client reaches
Articles35/39 and verifies6 artifacts; controlled Python-user-agent discovery
fails403. No full-manifest download or native Rust checks (no commit), no new
worktree, corpus changes or deployment. Existing dirty handoffs/plans preserved.
The old capsule remains stale and is not rebound to this different objective.
Durable repository evidence and this plan are the resume authority.

**Exactly one next action:** use the recorded failed request IDs/timestamps on
the existing host owner account to identify the denying rule, then verify a
scoped correction with these probes and an independent network/agent client.
Cross-client acceptance is not complete merely because Node/curl pass; source
or query implementation must not be presented as resolving that transport gate.
