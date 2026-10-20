# SQLite corpus database — implementation plan

## In plain terms

The database is a read-only copy of the committed corpus that can be searched
and queried quickly. It answers: what does this article say, what does it cite,
what cites it, where is a term defined, where did the text come from, and what
is known to be missing. It never changes the corpus, and it can be rebuilt from
any pinned commit with one command. It does not tell you whether a provision is
legally in force (most provisions are still `not_analyzed`), it does not fill
missing articles or links. Something the corpus does not contain is reported
as not found: with the recorded reason when the repository records one (not
separately parsed, cited but not in the corpus, excluded), otherwise as "not
found in this release; coverage not established" — never as non-existent. The
build happens in its own Git
worktree and branch and merges into `main` only after acceptance. The database
file lives in a Git-ignored folder inside the repository for now, and its
layout is chosen so it can later be published online without redesign.

## Objective, scope and stop condition

Produce a local, verified SQLite database from a pinned committed Lex-Mex corpus,
using Rust and `rusqlite`. Deliver a builder, a read-only verifier, schema,
FTS indexes, parity/integrity tests, reproducibility evidence and build instructions.
The observable result is a database artifact that can be rebuilt and queried
locally without altering canonical data or losing its legal qualifications.

This is the **database-only plan** requested on 2026-10-17. Planning does not
execute the build. Implementation ends when the database and its verification
report pass the acceptance gates below. Reader/UI, HTTP service, MCP/WebMCP,
hosting, deployment, PDF viewers, embeddings, model inference, automated updates,
legal review and parser/linker repairs are outside this plan. SQL query fixtures
verify the database; they do not create an application or service.

V1 includes all committed federal instruments and their existing graph,
definitions, temporal/review metadata and evidence. The 32 standards receive
catalogue/exclusion records only; their clauses and text are explicitly excluded
from provision tables and FTS. This follows the supplied suggestions and keeps
the separate standards model visible without claiming it has been projected.

## Workspace, storage and later online reachability

- **Worktree.** Database implementation runs in a dedicated linked worktree
  (`../lex-mex-db`, branch `db/sqlite-projection`). It merges into `main` by
  fast-forward or reviewed merge only after M4 acceptance and Codex review.
  Corpus or linker changes never happen in that worktree; they land on `main`
  first and the database worktree rebases onto them.
- **Storage now.** Release directories are written under the repository's
  Git-ignored `derived/db/` folder (`derived/db/<release-id>/corpus.sqlite`,
  `manifest.json`, `verification.json`). Database files are never committed:
  at roughly 340–380 MB (M1 measurement, below) they would permanently bloat Git
  history. If a tracked copy is ever wanted, use Git LFS or a release asset,
  decided separately.
- **Online later (design constraints now, no serving work in this plan).**
  Releases are immutable, content-addressed directories with a sidecar
  manifest and checksum, so any static host or object store can serve them
  unchanged and a client can verify what it downloaded. No absolute paths,
  host names or secrets inside the database or manifest. Keep the file
  readable by a plain SQLite reader (no custom extensions beyond FTS5/JSON),
  which keeps static HTTP-range readers and a later read-only API both
  possible. Operator notes, reviewer names and decision prose are included
  locally; a later publication step must decide redaction before anything is
  public. Record the file size per release so a hosting limit (for example a
  2 GB release-asset limit) is visible early.

## Prerequisite on `main`: edge resolution rule

Every reference edge currently carries the same basis
(`express_cross_reference`), so the database cannot tell an edge whose target
instrument was named in the text from one inferred from a bare `la Ley` to the
regulation's parent (5,451 edges; the advisor re-pass found 6 false links in
that group before the rule was narrowed). Before M2, the linker records an
optional `resolution_rule` on each edge (`own_instrument_default`,
`own_instrument_marker`, `named_instrument`, `parent_shorthand`,
`same_article`, `relative_position`), landed on `main` with schema, types,
validation and fixtures, and a relink. Instruments held on their previous graph
by the accepted-gap list keep edges without the field until their gap is
resolved; the database stores that as `NULL` ("rule not recorded"), never as a
guess.

## Authority and immutable planning baseline

- Repository: `/Users/jr/Dev/lex-mex`; planning HEAD
  `f0380e0fa` (resolve the full commit in the implementation manifest).
- `AGENTS.md`, `CODEX.md`, `PLANS.md`, committed Rust types/schemas and
  `docs/decisions.md` are authoritative. SQLite + `rusqlite` is already selected.
- D1/D2 clearance: `docs/pre-database-fix-verification-2026-10-17.md`.
  Prior verification: 307 tests; all 467 federal instruments valid, zero errors,
  42 warnings. These are prior checks, not a database verification result.
- Inputs reviewed: `docs/grok_gemini_db_suggestions.md` (SHA-256
  `6b0cf9887d3875871388a8b3fa68ccf5ee94728c958d68d56602fb2983845e8f`)
  and `docs/grok-db-suggestions.md`. Both were operator-owned untracked files
  at planning start; preserve their content and Git state.
- `docs/plans/nested-public-reader.md` supplies future consumer requirements.
  Its web/serving milestones do not enter this database workstream.
- Relevant contracts: `crates/lex-core/src/lib.rs`, `lex-source` adapter types,
  shared validation in `lex-parse`, `schemas/reference-edge.schema.json`,
  definition/usage/source/enactment/review schemas and existing bundle manifest.

Canonical JSON stays authoritative. The builder does not parse official sources,
relink citations, run temporal analysis, resolve a review, invent missing dates,
retarget historical edges or export modified canonical JSON. Read-only validation
of pinned inputs **is required**; invoking mutating pipeline CLI commands is not.
This corrects the suggestions' blanket prohibition on using a validator.

### Historical census checked during October 17 planning

Counts were independently recomputed from committed-shaped JSON without printing
source text. They are acceptance expectations for this baseline, not constants
that prevent a deliberately selected later corpus commit.

| Record | Count |
|---|---:|
| Federal instruments | 467 |
| Provisions | 68,086 |
| Article / transitory / annex provisions | 64,586 / 3,470 / 30 |
| Reference edges | 35,887 |
| Resolved / approved outdated / unavailable historical | 35,866 / 17 / 4 |
| Direct / relative / range-expansion forms | 31,922 / 2,620 / 1,345 |
| Official-title source anchors | 15 |
| Qualifier rows | 12,302 |
| Fraction / paragraph / subsection qualifier types | 8,685 / 2,811 / 806 |
| Defined terms / term usages | 3,534 / 62,785 |
| Stale approvals / legal operator notes | 17 / 16 |
| Standards, catalogue only | 32 |
| Canonical JSON under `corpus/mx/*/*.json` | ~167 MB |

Preserve all 30 annex provisions. The serialized qualifier type is `subsection`,
not an invented `inciso` enum. The closure allowlist contains eight accepted
instrument gaps. Machine-proposed count baselines, merged-heading debt and the
absence of a stored link are not evidence of complete legal coverage.

## Design decisions and treatment of the suggestions

| Suggestion | Plan decision |
|---|---|
| Direct projection from canonical JSON | Adopt; additionally perform read-only input/binding validation. |
| Typed edges, separate terms, ordered qualifier children | Adopt, preserving every field and array order. |
| Polymorphic source anchors | Adopt explicit provision/title source columns and enforceable FKs/checks. |
| Flattened review columns in proposed DDL | Replace with complete independent review/approval payloads; preserve hashes, spans, reviewer timestamps and evidence locators. |
| All optional metadata made mandatory | Correct nullability from actual Rust/schema contracts, including unfrozen counts and absent formal sources. |
| Byte reproducibility plus wall-clock `built_at` | Require reproducibility within a pinned build environment; keep wall-clock receipts outside the deterministic artifact. |
| FTS external content | Adopt with real backing columns and deterministic integer row IDs; populate after bulk load. |
| BM25 guarantees article-number priority | Use exact ID/number lookup separately; weights are a measured ranking choice, not an identity guarantee. |
| Derived qualifier token expansion | Defer. V1 stores original qualifier lists exactly and supports exact stored-text queries. |
| Range qualifier inheritance | Defer to a separate linker task. Bare range edges remain bare. |
| `journal_mode=OFF`, `synchronous=OFF` | Do not adopt as the default. Use a transactional temporary build and durable finalization. |
| 80–100 MB, zero-copy and sub-millisecond promises | Treat as unverified estimates. Measure final size and query timings. |
| Snapshot comparison from enactment/repeal evidence | Exclude; those records do not constitute historical corpus snapshots. |
| MCP/reader serving step | Exclude entirely from this plan. |

## Input, release identity and completeness contract

1. Require an explicit `--corpus-commit <full-oid>` and a committed builder
   version. Never read a mixture of current dirty files and claim the named commit.
   Materialize only selected Git blobs into a private immutable input snapshot
   or use Git object reads directly. Use relative repository paths in metadata.
2. Inventory all federal canonical JSON files and required adapter metadata at
   that commit, including operational/formal/annex source manifests, enactment
   and reform evidence, amendment legends, temporal analysis and review queues
   when present. Exclude generated Markdown, downloaded files, local model work,
   credentials and `.work` inputs. Include the accepted-gap file, consumed
   schemas and documentary coverage sources in the digest inventory.
3. Assign every discovered JSON path an explicit handling policy: normalized
   rows plus preserved payload, evidence-only artifact, or declared exclusion.
   A previously unknown path/type fails with its path; it is not silently dropped.
   Missing optional files remain distinguishable from present empty files.
4. Preserve exact JSON input bytes in `input_files` BLOBs, with SHA-256 and length.
   This permits exact source-file reconstruction and keeps future canonical
   fields from disappearing behind a limited relational schema. Indexed columns
   and record payloads must still pass parity; raw storage alone is not success.
   **Storage budget:** raw bytes (~167 MB) plus a full per-row payload copy plus
   columns and FTS would roughly triple the input. Keep the raw bytes as the
   reconstruction source; store a per-row JSON payload only for record fields
   that no column represents (an "extra fields" payload), and verify
   no-field-dropped by reconstructing each record from columns plus extras and
   comparing it with the raw record. **Settled in M1:** extras-only, measured
   on the full corpus at 333.7 MB against 497.1 MB for full payloads; the
   column-mapping, absent-versus-null, number and parity rules are in
   [M1 contract decisions](../sqlite-m1-contract-decisions-2026-10-17.md).
   Store standards metadata/catalogue provenance; inventory hashes and explicit
   exclusions for their unprojected clauses/text, without claiming full inclusion.
5. Release identity hashes a versioned build specification: corpus commit,
   selected inputs/digests, inclusion policy, builder commit, Cargo.lock digest,
   database schema version, SQLite version/compile options, Rust toolchain and
   deterministic settings. The artifact stores this specification and a stable
   source-commit timestamp, not a newly generated time. Runtime timestamps,
   elapsed time and host measurements belong in a separate verification receipt.
6. The sidecar manifest records the completed database checksum and logical
   content digest. Do not store the database's own checksum inside itself.
   Avoid absolute local paths and secrets in the database/manifest.
7. **Hard gate: logical parity.** Two builds with identical input specification
   must produce an identical logical content digest (every table's rows in a
   canonical order, every payload) and identical deterministic manifest fields.
   **Recorded observation: byte identity.** Identical database bytes within one
   pinned environment are a goal and are reported, but a byte difference with
   matching logical digests does not block acceptance; it is investigated as a
   follow-up. Across environments require logical parity only.

The existing corpus-bundle manifest is useful precedent. Add a database-specific
schema with matching Rust types and fixture; do not repurpose its
`canonical_markdown` profile to mean a database. Known documentary debt may be
recorded as sourced coverage observations, not fabricated canonical findings.

## Relational model

Use SQLite `STRICT` ordinary tables with explicit nullability and checks.
Represent JSON as `TEXT` with `json_valid` checks, booleans as checked integers,
and raw input bytes as `BLOB`; SQLite STRICT does not accept `JSON` or `BOOLEAN`
as type names. FTS virtual tables are separate.
[SQLite STRICT documentation](https://www.sqlite.org/stricttables.html).

Every canonical record stays completely reconstructible as a logical record:
mapped columns plus its `extras` object, ordered child rows and attached
review objects, equal to the raw record as a JSON value (M1 contract
decisions). Only the approval/operator-review objects keep a full `payload`.
Preserve null/omitted distinctions; never manufacture review, temporal or
baseline values to satisfy SQL NOT NULL.
Unknown schema versions/enum values fail with a useful diagnostic until mapped.

| Table/group | Key and required behavior |
|---|---|
| `corpus_release` | One row per database; deterministic specification, versions, counts and declared scope. |
| `input_files` | Relative path PK; exact JSON bytes, digest, byte length, handling policy, optional instrument owner. |
| `instruments` | Canonical ID PK, slug UNIQUE; actual instrument metadata, extras, optional adapter parent and count-baseline payload. No default 'audited' or 'clean'. |
| `source_manifests` | Instrument + source role + ordinal/path; all operational/formal/annex provenance, extras. Missing formal metadata stays absent. |
| `provisions` | Explicit deterministic `INTEGER PRIMARY KEY` row ID for FTS; canonical ID UNIQUE, owning instrument FK, original array ordinal, type including annex, label/number/headings/text and temporal/review columns plus extras. UNIQUE input_path/ordinal and ID/owner pair; input_path FK and source_pointer preserve file provenance. |
| `reference_edges` | Canonical edge ID PK; full source/target strings, source kind, spans, basis/confidence/form/status and extras. See constraints below. |
| `reference_qualifiers` | Composite PK `(edge_id, ordinal)`; exact type/text/optional offsets and extras. Preserve list text and repeated types; no span uniqueness or AUTOINCREMENT. |
| `outdated_reference_approvals` / `legal_operator_reviews` | Independent optional one-to-one rows keyed by edge ID; complete canonical payload and queryable reviewer/time fields. One can exist without the other. |
| `defined_terms` | Canonical ID PK; owning instrument, defining provision, offsets, basis and extras. Optional fraction/inciso keys remain in extras with queryable generated projections; preserve source-bound definition spans. |
| `glossary_coverage` | One reconstructible record per instrument: status/strategy, provision methods, ordered fractions/imports, hold, profile, usage form and counts. Preserve absent/empty/null distinctions, unread fractions and inciso profile metadata under the [current glossary contract](../sqlite-contract-refresh-2026-10-19.md#glossary-preservation-contract). Raw bytes plus extras/columns reconstruct all values. |
| `term_usages` | Canonical ID PK; provision and term FKs, exact span/offsets and extras. Cross-instrument usages are valid and remain separate from reference edges. |
| `enactment_evidence` | Canonical ID PK; owner, kind, containing-act title, source heading, text, publication date/period and extras; not law provisions. |
| `evidence_artifacts` | Path/kind/owner; preserved reform evidence, amendment legends, temporal proposals/results and review queues with schema/version/hash. Define query views only over real existing fields. |
| `coverage` / `coverage_accepted_gaps` | Explicit included/excluded/unavailable/not-assessed states, the pinned five current gap entries with exact error multiplicity/reason and source provenance. Keep instrument validation and closure completeness distinct. |
| `standards_catalog` | Standard ID/designation/title and exact metadata payload, stored validation summary, exclusion reason; declare clauses/text/links unprojected and fresh validation not performed. |
| `known_structural_debt` | One row per sourced known defect: instrument, missing or affected label, kind (`merged_heading`, `label_grammar_gap`, …), where the text actually lives (containing provision ID and exact character locator), source path/blob/hash and decision reference. Loaded only from operator-accepted rows of `adapters/shared/_known-structural-debt.json`; the `docs/decisions.md` aggregate (about 77 in 14 instruments) is not a seed. Never inferred by the builder. |
| `instrument_reproducibility` | Recorded measurements only: `identical`, `differs`, `fails` or `no_retained_text`, with the measuring commit, method and evidence hash. A view over `instruments` supplies `not_assessed` where no row exists; no stored `not_assessed` rows. No whole-instrument measurements are asserted by this refresh; scope-aware evidence is required before promotion. |
| `coverage_status_counts` | Per instrument: provisions by temporal status and review status, so `not_analyzed` is visible and dates are not read as reviewed. |

Indexes: provision `(instrument_id, ordinal)` and `(instrument_id, number)`;
edge source, live target, canonical target and resolution/form; term ownership,
term usage source and term target; review lookup; evidence owner/kind. Add further
indexes only for named query fixtures with measured query plans. Do not require
instrument/number uniqueness across articles, transitories and annexes.

### Source, target, span and audit invariants

- Edge `source_instrument_id` always names its actual owner. For a provision
  anchor, `source_provision_id` is non-null and a composite FK to provision
  ID/owner; its canonical source ID must equal that provision ID. For an
  instrument/title anchor it is null, canonical source ID equals the owning
  instrument ID, and the source text is the exact official title. Enforce these
  alternatives with CHECKs, FKs and builder validation, not a bare untyped ID.
- Preserve `target_instrument_id` and `target_provision_id` exactly for every
  edge. Add a nullable **derived live target** column: required and equal to the
  canonical target only for `resolved`, checked with a composite target/owner FK;
  null for historical/outdated edges even when a similarly named row exists.
  Do not attach unconditional live-target FKs to unavailable historical URNs.
- Store `resolution_rule` as a checked enum column, `NULL` when the edge
  predates the field. Consistency checks: `parent_shorthand` targets the
  adapter parent; `own_instrument_*` targets the owner; `relative_position`
  pairs with the `relative` form.
- Expose a `cited_but_missing` view from the accepted-gap error records (source
  instrument, missing target ID, multiplicity, reason), so an absent target
  reads as "cited, not in corpus" rather than "does not exist". It is
  instrument-level: held graphs do not store the rejected edge, so no citing
  provision or span is claimed.
- Preserve every distinct edge ID; shared source spans/range expansion are valid.
  Do not deduplicate on `(source, start, end)` or infer parent-shorthand origin
  from target identity. Adapter parent is separate provenance.
- Offsets retain Rust Unicode scalar/character semantics, exclusive end. Validate
  by the existing Rust span functions; never use UTF-8 byte slicing or treat
  FTS highlight offsets as canonical evidence offsets. Preserve exact text.
- Full approval/review payloads include source/decision hashes, target, span,
  capture timestamp and optional reviewer-supplied timestamp. Check bindings
  against canonical edge/source and corresponding pinned adapter record. Review
  prose stays prose. Do not reinterpret an old decision-source digest as a claim
  that today's mutable note has identical bytes or overwrite it on mismatch.
- Derive text hashes for provision search/parity from exact UTF-8 text and label
  them as projection-derived: `Provision` currently has no `text_sha256` field.
- Keep effects, partial repeals, commencement conditions, publication periods,
  amendment marks and review state together with the provision. Date queries
  expose recorded data/uncertainty; they do not assert historical applicability.

## Full-text and local SQL query contract

Use external-content `provisions_fts` over real `provisions` columns `label`,
`heading_title` and `text`. Materialize `heading_title` deterministically from
heading metadata; retain the complete original context separately. Back it with
explicit stable integer row IDs. Bulk-load content, then run FTS `rebuild` and
its content-aware integrity check. An immutable one-shot builder needs no insert-
only maintenance trigger. Configure `unicode61 remove_diacritics 2`; search is
accent-insensitive while canonical text is unchanged. BM25 sorts lower scores
first; start testing weights 10/3/1 with a canonical-ID tie-breaker.
[SQLite FTS5 documentation](https://www.sqlite.org/fts5.html).

Provide parameterized query fixtures and a small read-only database library API
for acceptance, not HTTP/MCP endpoints:

1. Canonical-ID lookup and instrument/type/number lookup (including suffixes).
2. Ordered local context using canonical provision ordinals, not numeric labels.
3. Outgoing/incoming live citations; separate all-status history view that exposes
   qualifiers and legal annotations without rendering non-live edges as live.
4. Definition lookup, exact term usages and cross-instrument glossary navigation.
5. Source provenance, full evidence/review payload and coverage retrieval.
6. FTS text search with instrument/type filters, bounded results and deterministic
   ordering. Exact article lookup is a separate operation; BM25 weights alone do
   not guarantee that an article outranks every mention of its number.

An exact lookup has three outcomes: found; not found with one or more sourced
qualifications (`not_separately_parsed` from an accepted debt row,
`cited_not_in_corpus` from an accepted-gap record, `excluded` from a coverage
exclusion); or "not found in this release; coverage not established".
Qualifications combine (LIC 112 Quintus has both a debt candidate and the
`cpf` gap).

**Golden queries** (expected answers taken from cases the operator has
reviewed; each is an M3 acceptance fixture):

1. Every `reg-lan` `parent_shorthand` edge (126 at this baseline) targets LAN:
   the definition in `reg-lan` article 1 makes every bare `la Ley` the Ley de
   Aguas Nacionales (operator, 2026-10-17); examples are articles 153 and 188
   citing LAN article 122.
2. `lic`'s citation of LFCE article 31 bis is approved-outdated (rule
   `named_instrument`) and carries the legal operator note naming the current
   article 3 fracción IX; the database never retargets it.
3. The four `lrsic` Transitory Quinto edges to `lraf-1990` appear in the
   history view, never as live citations; article 2 fraction V (current LRAF)
   stays live.
4. An approved-outdated edge returns its approval and its operator note
   together.
5. `cited_but_missing` lists `lftr` → LGTAIP article 180 Bis, multiplicity 1
   (instrument-level; no citing provision). CPF → LIC 112 Quintus is repaired and must be absent from that view.
6. LGTOC 228-B returns its separate provision with no active debt qualification.
   LFD 257 Sextus returns "not separately parsed" with its containing article
   257 Quintus and the accepted seed locator [100, 121).
7. FTS for `obligacion` and `obligación` return the same provisions.
8. Exact lookup of a `Terdecies` article returns that article.

Quote/escape literal FTS input separately from SQL parameter binding. Add Spanish
accent fixtures and document tokenizer collisions such as año/ano; FTS is lexical
retrieval, not legal equivalence. No English stemming or inferred synonyms.
Defer fraction-list token indexes, semantic search and snapshot comparison.

## Builder, verification and artifact lifecycle

Proposed commands (new implementation, not existing CLI claims):

```text
lex-mex project-db --corpus-commit <oid> --output-dir <derived-directory>
lex-mex verify-db --database <path> --manifest <path> --corpus-commit <oid>
```

Default selection is the complete federal model plus standards catalogue only;
record it explicitly in the manifest. V1 does not add arbitrary partial selections
that could sever valid target FKs. Validate output paths before writing: reject
canonical `corpus/`, adapters, source/schema paths and unsafe symlink aliases.
Use a Git-ignored derived output directory; never commit database files.

1. Snapshot/inventory/hash pinned inputs; freeze ordered source arrays; check
   canonical schemas, exact span/target/approval bindings and accepted coverage.
   Use shared validators in memory; do not call CLI commands that write reports
   into canonical directories. Fail on new errors; preserve accepted warnings.
2. Create a unique temporary sibling build directory on the destination filesystem.
   Use `foreign_keys=ON` before a transaction; normal rollback journaling and
   `synchronous=FULL` are the initial durability settings. Insert with prepared
   statements and deterministic ordering, preserving canonical arrays/ordinals.
   [SQLite pragma documentation](https://www.sqlite.org/pragma.html).
3. Create FTS and required indexes; run fixed finalization steps declared in the
   build specification. Avoid adaptive settings or memory-dependent layout choices.
   Start with a single transaction and ordinary journal finalization; optimize
   only against measured evidence while retaining reproducibility.
4. Run database integrity, FK, FTS and full parity checks; close all write handles,
   ensure there are no required WAL/SHM sidecars, and reopen with read-only flags.
   `verify-db` must use read-only access; perform write-command FTS integrity work
   in the temporary build or a disposable verification copy. `verify-db` also
   reports when the database's corpus commit differs from the repository's
   current `HEAD` for any input path, so a stale database cannot pass as current.
5. Produce `corpus.sqlite`, deterministic manifest and verification receipt;
   flush files and atomically rename the completed sibling directory to
   `<output-dir>/<release-id>/`. Readers can only discover complete directories.
   If it already exists, verify identity or refuse; never overwrite an existing
   release silently. No `latest` public pointer or deployment in this scope.
6. On interruption/failure discard only the owned temporary directory or leave it
   clearly marked failed. Prior successful artifacts remain intact. Recovery is
   a fresh build from the same pinned inputs, not repair of a partial database.

Read-only query handles use SQLite read-only open flags and `query_only`; no
write-capable service is built. Do not tune mmap/cache sizes or promise latency
before measurement. A local artifact containing exact review notes is not thereby
approved for public distribution; publication/redaction is outside this plan.

## Implementation milestones and ownership

Parent engineer owns schema/trust-boundary decisions and integration. Bounded
coding work can be delegated after paths/contracts are fixed; mechanical work
runs the named checks and reports exact results. No concurrent writes to the same
files. Fresh review should inspect the projection against canonical contracts.

### M1 — Input contract and schema

Implement database manifest types/schema, complete input-file handling matrix,
relational DDL and a small adversarial fixture. Create `crates/lex-db` only in the
same change as working builder/verification code consumed by `lex-cli`; no empty
crate. Keep `lex-core` independent of SQLite. Reuse `lex-source`/`lex-parse`
validation contracts without duplicating legal rules or introducing dependency
cycles. Pin a supported `rusqlite` release and bundled SQLite in Cargo.lock at
implementation time; do not copy the suggestion's `0.32` version blindly. Verify
FTS5/JSON/STRICT support with executable feature probes.

M1 also measures the storage layout (item 4 above) and records the projected
size, and fixes the `known_structural_debt` and reproducibility seed sources.
Settled 2026-10-17 in the
[M1 contract decisions](../sqlite-m1-contract-decisions-2026-10-17.md):
extras-only layout, seed contract and reviewed seed (historical counts; see the [current refresh](../sqlite-contract-refresh-2026-10-19.md) for 17 accepted, 30 rejected, 10 deferred and 60 resolved rows),
three-outcome lookup and view-supplied `not_assessed`.

Acceptance: every present canonical field/file has a preservation mapping;
title anchors, missing historical targets, absent metadata, annex provisions,
Unicode spans and independent approval/review records load correctly. Malformed
versions, duplicate IDs and illegal live targets fail with path/record diagnostics.

### M2 — Deterministic transactional builder

Implement pinned input loading, prepared bulk inserts, ordered child rows,
complete evidence preservation, coverage/catalogue records and the CLI builder.
Add manifest validation and safe output-path handling. No source pipeline calls.

Acceptance: fixture build succeeds; interrupted/corrupt-input runs cannot replace
a prior good artifact; dirty working-tree corpus changes are ignored by commit
object reads (or rejected by an explicitly verified snapshot path), never mislabeled.
Unknown input files cannot disappear unnoticed. All-source read hashes remain
unchanged. No fake unresolved rows or live links to historical targets.

### M3 — FTS, queries and independent verifier

Implement FTS rebuild, named query fixtures and `verify-db`. Compare normalized
SQL rows and complete payloads with pinned inputs independently of insertion code.
Preserve raw source JSON byte parity as a separate check. Add read-only handle tests.

Acceptance: exact/article/context/reference/term/source/coverage queries pass;
FTS accent handling, escaped syntax, ranking examples and FTS/content consistency
pass. Tests reject corrupted review hashes, changed source spans, missing live
FK targets, swapped qualifier ordinals and silent dropped payload fields.

### M4 — Full corpus build, reproducibility and handoff

Build the full declared selection twice in fresh directories with the same pinned
environment. Record all table counts, input digests, database hashes, logical
hashes, schema version, builder identity, file size and timings. Measure representative
local queries and inspect their query plans; do not create a performance SLA
from the suggestions' estimates. Document commands and artifact paths in a
repository report. Rebaseline counts only with a deliberately changed input commit.

Acceptance and stop condition:

- All record counts match the selected input census, including 30 annexes,
  15 title anchors, 12,302 ordered qualifiers, 17 approvals and 16 notes.
- Every provision text and derived hash matches, not just a 1,000-row sample;
  every edge/qualifier/term/usage/approval/review field and required evidence
  payload matches. Existing validated float values use a defined lossless
  representation/round-trip policy; no lossy rounding of confidence.
- `PRAGMA integrity_check` is `ok`, `foreign_key_check` is empty, FTS integrity
  passes, and the manifest/database/input checksums agree.
- Two clean pinned builds have identical logical content digests and all local
  query results agree deterministically; byte identity is reported. A changed
  input or toolchain produces a distinct specification/release identity, not
  an overwrite.
- All updated golden queries return their recorded answers.
- Five current accepted gaps, exclusions, machine baselines and stored uncertainty are
  queryable. Standards catalogue is distinct and their body coverage is excluded.
- Required native gate: `cargo fmt --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace`, plus locked LRITF and IFPE CLI validation checks.
  Run write-capable validation only against private snapshots when checking
  canonical immutability. Inspect input manifests, new database verification
  report, preserved review queues and canonical Git diff. This database-only
  path generates no Markdown; preserve existing exports rather than rerunning them.
- Repository report and task plan state the result, remaining coverage limits
  and recovery command. No UI, MCP, hosting, deployment or legal normalization
  work is added before stopping.

## Current checkpoint and next action

2026-10-20 (M1 review repairs, uncommitted): the debt validator now requires
exact equality with the pinned accepted review set, including metadata and
locators. Seed schema v2 pins review evidence at 6505baf57 and binds the exact
human-decision section, allowing unrelated decision-log additions. All 17
accepted rows are unchanged. Source arrays use zero-based ordinals scoped to
input path, plus a JSON pointer; scalar objects use ordinal 0 and pointer "".
This applies to direct projections, including debt and accepted gaps; children
inherit file provenance through their parent and retain local ordinals.
[Review response](../sqlite-m1-review-response-2026-10-20.md) records the repairs
and verification. fmt, strict workspace Clippy and all 380 tests pass. M2 loaders
have not started. Historical measurements below retain their original scope.

2026-10-20 (M1 implementation): main bfcda816f is integrated in the DB worktree;
local integration commit follows the final gate. [Implementation report](../sqlite-m1-implementation-2026-10-20.md)
and [evidence](../sqlite-m1-implementation-2026-10-20-evidence.json) record
manifest/seed schemas and types, all 17 accepted seed rows, the pinned input
handling matrix, SQL DDL and a working in-memory fixture builder/read-back
probe consumed by lex-cli. Bundled SQLite 3.53.2 supports FTS5/JSON/STRICT.
Final fmt/Clippy/373 tests pass, required instrument validators pass, seed
bindings pass and 4,917 real input files classify successfully. No full-corpus
DB artifact or new canonical/legal determination. M2 loaders/output and M3
release verification are not implemented. Earlier dates below are historical.

2026-10-19 (contract/debt refresh): current corpus input is
`bfcda816f46b8b0c1d694fc1f2cfb8b4438275b2`, following the reviewed sweep
`d8ecaeaa0`. DB branch HEAD remains `04cc039af`; no history was changed.
The [refresh](../sqlite-contract-refresh-2026-10-19.md) is the current binding
contract for census, glossary preservation, sourced debt and query expectations.
It supersedes dated counts and repaired missing-article examples below; the
immutable initial baseline above and historical measurements stay unchanged.

Current census: 468 instruments, 68,328 provisions, 35,949 references, 5,734
terms and 102,204 usages. Glossary strategy 3; 18 configured, 299 detected,
6 held, 145 none detected. Five accepted closure gaps. The current reviewed
seed has 17 accepted, 30 rejected and 10 deferred candidates, plus 60 repaired
old rows recorded separately. No new legal ruling or database artifact.

### Historical checkpoints

2026-10-17 (operator rulings): Latin ordinals beyond `Nonies` are the same
numbering series; the label grammar now ranks them (`fe4a53f1e` on `main`,
full native gate passed, this branch rebased). The candidate classes are
reviewed: 77 proposed for acceptance (operator yes pending), 27 rejected (legally removed placeholders), 10 deferred
to the DCG pass, and 3 LFTR bracketed-body rows still unreviewed. G1 now rests
on the `reg-lan` article 1 definition of "Ley". Open, not blocking: 51 stored
false self-link edges from embedded headings, the `cpf`/`lmv` stale-reference
contradiction, and the missing `reg-lan` definition term.

2026-10-17 (M1 contract, `db/sqlite-projection`): the three review details
are settled in the
[M1 contract decisions](../sqlite-m1-contract-decisions-2026-10-17.md).
Full-corpus prototype measurement chose extras-only (333.7 MB versus
497.1 MB) and reconstructed every provision, edge, term and usage with zero
mismatches. A read-only
detector found 117 unreviewed structural-debt candidates in 22 instruments
(the recorded aggregate was 77 in 14); no seed row is accepted. Golden fixtures
pinned to edge IDs; G1 corrected to `reg-lan` 153/188 pending operator
confirmation. No Rust code, dependency or database file exists yet.

2026-10-17 (Codex review): strengthened plan and `resolution_rule` prerequisite
reviewed at `cd49b9db4`; fmt/clippy/309 tests pass. Before M2, M1 must settle
physical payload layout, source-backed structural-debt seeds, and explicit
unknown/not-assessed semantics. See [review](../sqlite-plan-review-2026-10-17.md).
No database work started.

2026-10-17 (later): operator review adopted with changes — plain summary,
dedicated worktree and in-repo Git-ignored storage with online-later
constraints, `resolution_rule` prerequisite, structural-debt and
reproducibility tables, storage budget, logical digest as the hard gate,
golden queries, stale-commit check and temporal coverage counts. Database work
starts only after the `resolution_rule` change lands and the operator says go.


2026-10-17: database-only plan drafted from both suggestion documents, current
Rust/schema contracts, the cleared pre-database review and primary SQLite docs.
Census independently checked. No Rust, dependency, canonical-data or database
file changed. The operator's two suggestion files remain untouched/untracked.

`resolution_rule` landed (see `docs/decisions.md`).

Exactly one current next action: review the M1 repair diff and accept its debt/provenance boundary before implementing M2's full pinned-input transactional builder and recovery gates.
Keep M3 independent verification and M4 full-corpus reproducibility separate.
