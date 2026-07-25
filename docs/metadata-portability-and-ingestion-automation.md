# Metadata portability and ingestion automation

This document reviews the proposed YAML metadata upgrade against the current
Lex-Mex architecture and folds the useful parts into the external-consumer and
unattended-ingestion roadmap.

## Executive decision

Adopt the proposal as a requirements inventory, not as a migration
specification.

The strongest recommendations are portable schemas, explicit schema
compatibility, stable field semantics, controlled values, canonical URN rules,
separate legal dates, provenance, and deterministic migration reports. Several
other recommendations already exist. The central proposal that Markdown plus
YAML should become canonical is rejected because Lex-Mex deliberately makes
committed JSON canonical and Markdown/Obsidian generated presentation targets.

No bulk corpus rewrite should begin from the proposed example front matter.
First define the application/bundle boundary, inventory the actual corpus, and
add read-only schema validation.

## Current reality versus suggestions

| Suggestion | Current repository state | Decision |
|---|---|---|
| String provision numbers | `Provision.number` is already a `String`; labels such as `1o` and `77 Bis` are preserved. | Already satisfied. |
| Explicit display label | `Provision.label` is canonical and exported as the heading. | Already satisfied. |
| Identity and instrument linkage | Provisions already carry stable `id`, `instrument_id`, `provision_type`, and `schema_version`. | Already satisfied; document the URN grammar. |
| Separate legal dates | Publication, effective-from, effective-to, latest reform, retrieval, and formal-source facts are already distinct where known. | Preserve; do not infer missing dates. |
| Jurisdiction, type, title, and authority | Instrument records already contain jurisdiction, level, instrument type, official/short title, source roles, and optional issuing authorities. | Export more of these into presentation or bundles when useful; do not duplicate them canonically per provision. |
| Controlled vocabularies | Rust enums own instrument, provision, temporal, review, reference, and effect values. | Generate or validate portable schemas from one authority; do not create independent YAML lists that can drift. |
| Extraction provenance and hashes | Source manifests and instrument records preserve source URL, retrieval time, source and extracted-text hashes, parser/schema versions, and formal/annex provenance. | Extend only for a demonstrated source class such as OCR; keep document facts at document level. |
| Structured references | `references.json` records exact source spans, offsets, target IDs, qualifiers, basis, confidence, resolution status, and reference form. | Keep as canonical sidecars. Semantic relations such as `implements` require a separate reviewed fact/proposal model. |
| Structure | Canonical provisions carry heading context; document order is canonical. | Derive `previous`, `next`, and sort indexes unless a consumer proves they must be persisted. |
| Granular review | Temporal review state, queues, named resolutions, rationales, timestamps, and history already exist. | Do not invent `approved` or reviewer identities. Add content/citation review workflows only with defined transitions and an actual reviewer process. |
| Version identity | The current corpus represents consolidated snapshots, not a complete interval-version history. | Valuable future model, but blocked on source/version acquisition and legal-temporal semantics. Do not synthesize `version_id` from one consolidation date. |
| Amendment events | CNBV amendment legends and reform-transitory evidence are structured for supported sources. | Generalize only from formal publication evidence; paragraph-level action classification may be a reviewed proposal. |
| Citation fields | No canonical official/short/machine citation object exists. | Good candidate after citation grammar and source authority are defined. |
| Article content hashes | Document and extracted-text hashes exist; per-provision content hashes are not stored. | Prefer a derived bundle/index digest first. Persist only if it becomes a compatibility or audit boundary. |
| Schema identity/version | Canonical objects already carry `schema_version = 0.1.0`; several trusted model and manifest boundaries have JSON Schema. | Add portable instrument/provision schemas before migration. Do not relabel the existing format as `1.x` without a compatibility decision. |

## Suggestions not accepted as written

1. **Markdown is not canonical.** `corpus/mx/<slug>/*.json` remains canonical.
   Generated Markdown can be deleted and deterministically rebuilt.
2. **YAML does not become a second database.** Front matter contains
   presentation and query conveniences derived from canonical records.
3. **Validation state is not self-certified metadata.** A field such as
   `schema_status: valid` is stale the moment content changes; validation
   reports and command exit status own that fact.
4. **Derived navigation stays derived.** `sort_key`, `previous_id`,
   `next_id`, inbound counts, indexes, and search caches should be rebuilt
   unless a versioned external format requires them.
5. **No speculative historical versions.** `version_id`, `supersedes`, and
   effective intervals require evidence for the exact historical text, not a
   naming convention applied to a consolidated snapshot.
6. **No parallel enum authority.** YAML enum files are acceptable generated
   documentation, but Rust types and external schemas cannot silently diverge.
7. **No proposed top-level reorganization.** The existing lowercase
   `corpus/`, `schemas/`, crates, and sidecars remain the repository layout
   until code uses a reviewed new surface.

## Recommended consumer architecture

### Stage 1: mounted corpus and CLI

Install one binary and point it at the authoritative checkout:

```bash
cargo install --locked --path crates/lex-cli
export LEX_MEX_ROOT=/Users/jr/Dev/lex-mex

lex-mex instruments --json
lex-mex search "juicio político" --instrument lfrsp
lex-mex search "artículo 110" --instrument lfrsp,cpeum -- --ignore-case
rg '"temporal_status": "repealed"' "$(lex-mex path lfrsp --kind provisions)"
```

This mode is read-only, sees live committed data, and can select only the
instruments the application needs.

### Stage 2: deterministic subset bundles

Add a future command shaped like:

```text
lex-mex bundle create --instrument cpeum,lritf --format canonical \
  --output ./vendor/lex-mex
```

The bundle manifest must include:

- bundle schema and Lex-Mex canonical schema versions;
- selected slug and canonical instrument ID for every instrument;
- exact included artifact classes;
- source, extracted-text, and included-file SHA-256 digests;
- generation commit and time;
- validation result digests;
- dependency policy for referenced but unbundled instruments.

Profiles should include `canonical`, `markdown`, and `canonical+markdown`.
References to excluded instruments remain explicit edges; the bundle must not
pretend those targets were included.

### Stage 3: library/API boundary

Only after two real consumers establish stable needs should common read/query
logic move behind a reusable crate or service API. Do not expose parser and
review internals as a public compatibility promise prematurely.

## Metadata upgrade sequence

1. Inventory real values from canonical JSON without bulk-reading them into a
   model context.
2. Write read-only external JSON Schemas for instrument and provision records.
3. Define compatibility rules for existing `0.1.0` and the next version.
4. Specify the canonical URN grammar and citation grammar.
5. Generate a migration report; make no corpus changes.
6. Validate representative instruments covering statutes, codes, CNBV
   regulations, unusual labels, repealed provisions, references, terms, and
   reviewed temporal state.
7. Add only fields required by an actual bundle or application.
8. Migrate a small sample idempotently, review legal-content diffs, then decide
   whether a corpus-wide migration is warranted.

This sequence uses the useful discipline from the proposal while avoiding a
presentation-led schema rewrite.

## Continuous ingestion design

The unattended mechanism should have two layers:

- **Deterministic executor:** the Rust CLI fetches, extracts, parses, validates,
  exports, records structured receipts, and reports an exact next state.
- **Harness controller:** a same-provider coding agent inspects provisional
  output, diagnoses anomalies, writes required fixtures or adapter changes,
  runs gates, checkpoints, and starts a fresh bounded session after a completed
  cluster.

The controller does not invoke another provider automatically and does not run
temporal analysis as part of structural ingestion.

### Instrument state machine

```text
queued
  -> prepared
  -> provisional
  -> inspected
  -> frozen
  -> validated
  -> exported
  -> checkpointed
  -> batch_closure
  -> complete
```

`provisional -> frozen` is never a blind loop. Inspection must compare the
official source, proposed structural counts, provision labels/order, document
boundaries, hashes, reference behavior, reform evidence, validation report,
canonical diff, and generated Markdown. A reusable parser defect receives a
fixture and focused code change before retry.

### Resume receipt

Each completed transition records under ignored `.work/`:

- repository commit and dirty-state summary;
- plan and batch-manifest paths plus SHA-256;
- adapter path and digest;
- instrument slug and source URLs;
- source and extracted-text hashes;
- proposed/frozen counts;
- validation and batch-closure report paths/digests;
- changed paths;
- retry count and backoff deadline;
- exact next state and stop reason, if any.

The task plan records accepted milestone facts. A fresh session verifies the
receipt against Git and repository files before continuing.

### Automatic continuation budgets

Every run declares bounds such as:

```text
maximum instruments
maximum wall-clock duration
maximum commits
per-host request interval
maximum retries
stop milestone (instrument, batch, or prepared inventory boundary)
```

HTTP 429/503 responses honor `Retry-After`; repeated acquisition failure uses
bounded exponential backoff and checkpoints instead of spinning. The
controller stops cleanly when its time/context/rate budget is reached and
leaves the exact resumption state.

Harness/provider rate limits likewise trigger a persisted checkpoint and
operator-visible scheduled resume; they never trigger an automatic
cross-provider switch. A launch policy may explicitly authorize narrow
per-instrument commits and pushes after every gate passes. Without that
authorization, the controller stops with a verified, uncommitted diff.

### Hard stop gates

Stop and checkpoint on:

- an unexpected dirty worktree or change outside the named instrument/fixture;
- changed source identity, publisher mismatch, or unresolved formal-source
  requirement;
- ambiguous document boundaries, article gaps/order anomalies, duplicate IDs,
  or a count proposal that cannot be justified from the official source;
- a shared parser/schema change not covered by a focused regression fixture;
- any validator, expected-edge, source-span, hash, or Markdown inspection
  failure;
- a change to audited legal review state or a request for reviewer identity;
- a blocked manifest entry requiring JRH confirmation;
- exhausted retry, rate, time, instrument, or commit budget.

### First bounded exercise

After the CLI milestone and bundle contract are settled, implement the state
machine with `lgbn` as a one-instrument exercise. It is already the
repository-authorized next AD1 item and will expose whether the receipt and
inspection gates are sufficient before allowing multi-instrument continuation.
