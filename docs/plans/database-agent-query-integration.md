# Database integration for agent retrieval

Status: **D0–D2 implemented and accepted locally. D3 accepted by independent
Codex round-2 re-review for the recorded source hashes; R1/R2 closed. D4 host
handoff is current to the reviewed owner commit; backend runtime decision open.**
Implementation authorized by the operator's session goal on 2026-10-25.
Prepared: 2026-10-24. Implemented: 2026-10-25. Nothing committed or deployed.

## Purpose and observable outcome

Connect the accepted SQLite corpus projection to a bounded public retrieval
contract. An agent should resolve a citation, find relevant provisions, retrieve
exact text and inspect express references without downloading full instruments
or the full integrity manifest. Every result must identify its source snapshot,
coverage and limitations. The human Reader remains independently usable.

This is the DB integration workstream of `agent-retrieval-optimization.md`.
That parent plan owns transport diagnostics and the broader agent evaluation.
This file alone owns the proposed DB integration sequence; it does not replace
the DB producer's existing acceptance plan.

## Immutable initial baseline

- Reader checkout: `f49c07923` on `feat/public-reader-r7`, with existing uncommitted
  retrieval harness, documentation and earlier handoff changes.
- Main: `e3ad94ee3bf6e5a5ec34274ad77af10ce88bcf1d`.
- DB checkout: `047b421efe9ef6575288bb6d35b6190fc9e2d192`.
- Experimental query branch reviewed at
  `d2170f9fd7ff8ea179512e496f22fffa2c9ddc1c`; not integration-ready.
- Current public release observed by the preceding baseline:
  `c305db4a3cfb408b76c13431e26beda72d5ce665046c95783fab36507354c969`,
  corpus commit `d2ab64d61ecfe103cc2a2980c5d04ed63c20bb7a`.

These are baseline identifiers, not permission to serve an old DB artifact.
Refresh Git state, acceptance records and artifact hashes before implementation.

## Authority and scope

Read the applicable AGENTS.md/CODEX.md and these bounded sources:

- Main: `crates/lex-db/src/queries.rs`, `verify.rs`, the DB schema and accepted
  artifact manifest; `docs/plans/sqlite-corpus-database.md` and its linked
  acceptance evidence.
- Producer: `crates/lex-cli/src/public_release.rs` and public release/instrument
  schemas; these define existing public projections and exact-text artifacts.
- Main: `docs/public-query-branch-review-2026-10-24.md`; inspect proposed reusable
  contract files with `git show` at the experimental hash above.
- Reader: `docs/agent-retrieval-baseline-2026-10-24.md` and companion evidence.
- Host owner: the current website HANDOFF, only when selecting a serving adapter.

In scope: snapshot binding, read-only query adapter, public response schemas,
citation resolution, bounded lexical search, catalog/reference pagination,
provenance, failure semantics and acceptance evidence.

Excluded: new canonical ingestion, legal-review transitions, CNBV guide
admission, inferred historical validity, model-generated legal answers,
embeddings, MCP, Reader UI/windowing changes and production deployment.
No extra worktree is needed for drafting. Later implementation must respect the
three-checkout layout and preserve existing dirty files.

## Proposed design

### 1. Explicit snapshot and public-scope binding

Before serving, verify a binding record containing the DB artifact checksum and
manifest/schema version, builder/corpus identifiers, public release ID, selected
instrument/source hashes and public projection version. Compare actual text and
source hashes for the served selection; matching commit labels alone is not
proof. Fail closed on a missing, stale or incompatible binding.

Start with the same four instruments as the current public release. A full
federal DB may back the adapter, but every operation must enforce that public
allowlist, including search, lookup, resolution, references and qualifications.
Broader publication is a separate scope decision. If the accepted DB does not
match the selected release, produce and verify a compatible snapshot through
the existing pipeline; never relabel its bytes as a different release.

### 2. Reuse the Rust query core

Use `lex_db::queries::Reader`: read-only connection, `lookup`, `article_ids`,
`context_ids` and parameterized literal-phrase `search`. Add missing bounded
operations in the existing crate. Preserve Rust responsibility for trusted
projection/validation. Do not expose arbitrary SQL or serialize raw `Lookup`
values: those contain canonical extras and internal evidence paths.

The serving adapter should be thin and consume only validated public responses.
Do not assume the current website runtime can execute Rust or open the SQLite
artifact. Confirm runtime, artifact size, filesystem and resource constraints
before choosing that adapter. No replacement site or second database engine is
implied. A local validated CLI/adapter can be accepted before hosting is selected.

### 3. Public operations and semantics

| Operation | Proposed behavior |
|---|---|
| Coverage/catalog | Small snapshot descriptor, published scope, source dates and paginated instrument metadata; no full manifest required |
| Resolve citation | Instrument + provision kind + exact printed locator; return zero, one or bounded ambiguous candidates |
| Search | Bounded literal lexical search with instrument/kind filters; IDs, headings, short excerpts and source/citation pointers |
| Get provision | Exact UTF-8 text with content hash, canonical ID, source and pinned release; complete text or an explicit oversized response outcome |
| Express references | Paginated validated edges, evidence spans and qualified target availability; no inferred legal relationships |

Proposed defaults: at most 10 results, 240 Unicode characters per excerpt,
512 characters per search query and 16 KiB per non-article response. Specify
UTF-8 byte limits separately from character limits. Validate raw and normalized
queries; empty normalized input is a client error. Do not silently change phrase
search into an OR query or add editorial aliases without versioned semantics.

Full articles are a separate operation. Determine its maximum body size from
the selected corpus and hosting limits; never silently truncate legal text.
Return an exact existing artifact link when appropriate. Excerpts are visibly
partial and must not be presented as the complete provision.

Preserve distinctions among ambiguous locator, unpublished target, structurally
unseparated provision, cited-but-absent provision and not-established coverage.
Convert these to public-safe fields; remove internal paths, reviewer identities
and private rationales. Do not infer legal nonexistence from a missing DB row.
Do not expose unavailable historical applicability as a successful `as_of`
answer. Retrieved, last-reformed and legally effective dates remain distinct.

### 4. Stable bounded pagination

Define deterministic ordering and cursors bound to release, operation, query,
filters, sort, page size and projection version. Reject null/scalar/malformed,
cross-query and cross-release cursors as client errors. Prefer seek pagination;
if score ordering requires another strategy, document and measure its bound.
Never sort/materialize all full texts or compute every excerpt before paging.
Add execution deadlines, concurrency limits and cancellation at the adapter.

### 5. Selective reuse of the experiment

Reuse only reviewed contract/schema/test ideas from `feat/public-query`, not its
packager, UI or in-memory federal search. Repair null cursor and normalized-empty
query behavior before porting. Decide one source-URL policy based on actual
provenance; do not rewrite official URLs merely to satisfy HTTPS validation.
If HTML is included later, repair and test its parsed text/hash mismatch first.
HTML is not required for this DB adapter.

## Proposed milestones

| Gate | Deliverable | Acceptance and stop condition |
|---|---|---|
| D0 — reconcile inputs | Refreshed baseline, protected-dirty-file inventory, exact accepted DB artifact and four-instrument binding design | Independent check of artifact/manifest/source compatibility; stop on unaccepted or mismatched inputs |
| D1 — contract | Schemas, Rust types, public field allowlist, error/cursor rules and synthetic fixtures | Resolve Bis/transitory/annex/duplicate locators without choosing arbitrarily; no private data leakage or silent truncation |
| D2 — adapter | Read-only DB implementation with scoped resolver/search/catalog/references and local executable entry point | Same exact text/source hashes as pinned artifacts; all operations enforce public scope; wrong/missing binding fails closed |
| D3 — acceptance | Regression, concurrency/resource and representative full-DB performance evidence | Required native checks, DB verifier/goldens, public contract tests and independent review pass; no writes to accepted DB |
| D4 — host handoff | Runtime decision, exact artifact/contract bindings, operational limits and rollback instructions | Owner can review concrete integration; no deployment or broadened public scope within this plan's draft authority |

For later delegation, the parent owns schema/privacy/release decisions and
integration. GPT-6.1 Sol can implement disjoint bounded modules and independently
review them; Luna6 can run exact deterministic checks after scope is fixed.
No implementation agents are started by this draft.

## Validation and recovery

Before any code commit, run repository-required fmt, workspace clippy with
warnings denied, workspace tests and LRITF/IFPE validators. Also run the accepted
DB verifier and golden queries, contract regressions, public-scope leakage tests,
text/hash parity and affected pipeline stages. Keep their results separate.

Measure cold/warm query latency, p50/p95, concurrency, memory, query plans,
response bytes and pagination stability on the actual accepted DB. Set explicit
performance ceilings after measuring the selected deployment environment and
before acceptance; synthetic fixtures alone do not establish scale readiness.
Evaluate IFC/IFPE question retrieval separately from human-approved legal-answer
correctness. The existing 403/client failures remain a deployment acceptance
gate, not a reason to block offline adapter development once authorized.

Keep DB snapshots immutable. Stage a new binding/adapter beside the accepted
one, verify it, and switch atomically where the selected runtime supports it.
On verification failure leave the accepted service and static publication
unchanged. Existing current-only hosting policy remains; unavailable old pins
must produce an explicit outcome, never silently resolve to current text.

## Gate outcomes (2026-10-25)

Evidence: `docs/database-agent-query-acceptance-2026-10-25.md` and its
companion JSON; host handoff: `docs/database-agent-query-host-handoff-2026-10-25.md`.

| Gate | Outcome |
|---|---|
| D0 | **Passed.** Refreshed: Reader `f49c07923` (code identical to main `e3ad94ee3`), DB checkout `047b421ef`. Accepted DB release `afdc5acf…1974` (corpus/builder `5eca03bc5`, physical `5da6e800…`, logical `8cc37360…`) against public release `c305db4a…c969` (corpus `d2ab64d61`): all 4 × canonical input hashes and all 870 provision texts identical, 0 mismatches. ifpe-dcg-2021 shows 115 release edges vs 112 provision-anchored DB edges; the 3 extra are title-anchored (`source_anchors.kind='instrument_title'`) and are served. Builder-commit verifier (binary built from `5eca03bc5`) exits 0 with 21/21 goldens; its only stale paths are the two later public-release schemas. The HEAD binary cannot verify this artifact (it requires schemas absent at `5eca03bc5`) — pre-existing, not caused by this work. |
| D1 | **Passed.** `schemas/public-query-binding-v1.schema.json`, `schemas/public-query-response-v1.schema.json`, Rust `lex_db::public`, synthetic fixture tests. Duplicate printed locators return `ambiguous` with all candidates; distinct absence qualifications; no private fields representable. |
| D2 | **Passed.** `lex-mex db public-bind` / `db public-query`. Real-data acceptance: all 870 provisions field-identical to the release, every edge set identical, broad searches never leave the selection, out-of-scope IDs disclosed only as outside scope; tampered binding/DB/release fails closed. |
| D3 | **Accepted by independent round-2 re-review; R1/R2 closed.** Codex review `docs/database-agent-query-codex-review-2026-10-25.md` (commit `af6de68a8`). R1: binding compared reference IDs only, so a rehashed release with a changed span bound. Now `bind` validates the release and instrument schemas and requires the complete served projection to equal the DB-derived one: instrument metadata, all 15 provision fields and artifact paths, main and annex source metadata (annexes mapped through the adapter's `annex_pdf_urls`, as the producer does), and every edge field including `target_published`. R2: CLI stdin framing was unbounded. Now byte-bounded frames reject an over-limit frame as soon as it exceeds 8,192 bytes, drain the rest unstored and serve later frames. Evidence: `docs/database-agent-query-acceptance-2026-10-25.md` (repair section). |
| D4 | **Handoff corrected; owner decision open.** The website host is the owner's customer-owned Worker + Static Assets on Workers Free, live in production with observation pending (owner commit `41243b07`), not GPT Sites. The website migration is independent of DB hosting. The native adapter stays local until the owner chooses a backend; no API route, container, D1/R2, paid plan or public DB download is implied. |

Implementation proposals awaiting operator ratification (not settled policy):

- Source URLs are emitted verbatim as recorded (`^https?://`), never rewritten;
  all four selected instruments currently record HTTPS.
- Oversized threshold 256 KiB for complete provisions (largest selected is
  209,814 bytes); hosting limits may lower it, which yields explicit
  `oversized` outcomes with exact artifact links.
- Search semantics `literal_phrase_accent_insensitive` over FTS5 `unicode61
  remove_diacritics 2`; input with no letter or digit is `empty_query`.
- Locator resolution matches the canonical label slug only; no editorial
  aliases (for example `Décimo Primero` does not match `DECIMOPRIMERO`).
- Incoming references list only edges from published instruments; outgoing
  edges disclose out-of-scope targets as `outside_published_scope`, as the
  static release already does.
- The adapter lives in `lex-db` (as planned); since `crates/lex-db/src`,
  `schemas/` and `crates/lex-cli/src/db.rs` are builder-pinned, a future DB
  build at a commit containing this work has a new builder identity. The
  accepted artifact remains valid against its own builder commit.
- rusqlite's `hooks` feature is enabled for the deadline interrupt; Cargo.lock
  is unchanged.
- (R1 repair) An annex's expected source comes from the DB's copy of the
  adapter's `annex_pdf_urls`, matched to retained manifests as parsed
  `url::Url` values, exactly as the producer compares them (`a%2Fb` and `a/b`
  differ); every match must be unique. This adds the existing workspace `url`
  crate to `lex-db`: one `Cargo.lock` dependency line, no new package. `public-bind` refuses an instrument
  without exactly one adapter configuration.
- (R2 repair) An over-limit stdin frame gets `bad_request` immediately, and
  its remainder is drained unstored while serving continues; the process is
  not terminated. Invalid UTF-8 gets `bad_request`. A final unterminated frame
  is accepted at EOF. One CR before LF is part of the delimiter and is not
  counted.

## Current checkpoint and next action

2026-10-25: Implemented D0–D4 as recorded above, uncommitted on
`feat/public-reader-r7` in the Reader checkout. Pre-existing dirty files were
preserved. Branch placement (this R7 line versus a branch from main) is an
operator decision; the touched code paths are identical on both.

2026-10-25 (re-review round 1, operator relay): R2 accepted. R1 still
blocked on two points: a duplicate edge could stand in for another (count
plus membership check), and percent-decoded annex URLs diverged from the
producer's parsed URLs. Both repaired. The release's edges now become a map
with duplicate rejection and are compared whole; annex URLs compare as parsed
`url::Url`. D4 status updated: owner production is live on Workers Free and
observation is pending (owner commit `41243b07`). The DB adapter remains
local.

2026-10-25 (repairs): R1 and R2 repaired per the Codex repair request; the
D4 handoff is corrected. Implementation is still uncommitted. The accepted DB,
public release and original binding are untouched; re-binding with the
stricter checks reproduces the original binding byte for byte.

2026-10-25 (Codex round-2 verdict): D3 accepted at the exact hashes in
`docs/database-agent-query-codex-rereview-2026-10-25-evidence.json`. Independent
13 DB + 2 CLI + 1 real-artifact tests pass; control repack binds while duplicate
and span repacks fail. R1/R2 closed. D4 remains runtime-decision-open, native
adapter local. Round-1/round-2 review requests resolved; result delivered to
Claude's pending queue. See `docs/database-agent-query-codex-rereview-2026-10-25.md`.

2026-10-25 (commit): the operator approved the exact-path proposal. The reviewed
implementation and its records are committed on `feat/public-reader-r7` (the
commit that carries this paragraph); no reviewed source byte changed. Not
pushed or deployed. Unrelated Reader work stays uncommitted. `derived/` stays
Git-ignored, including the binding, the accepted DB and the signed release.

**Exactly one next action:** the website owner decides the D4 backend runtime
(the options are in `docs/database-agent-query-host-handoff-2026-10-25.md`).
Until then the adapter stays local. Any hosting needs its own plan and review.
Placing the branch relative to `main` and pushing it are separate operator
decisions.

2026-10-24: Draft completed from repository interfaces and prior review evidence.
No code/schema/DB/host changes, tests, builds, agents, commits or deployment
performed for this draft. Existing implementation and dirty files are preserved.
The stale discovery capsule is not rebound; this repository plan is the draft
resume point. Outcomes and retrospective will be recorded at each future gate.

Superseded draft next action (2026-10-24): review the four-instrument scope and
D0 acceptance boundary before authorizing implementation — authorized 2026-10-25.
