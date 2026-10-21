# SCJN invalidity registry — deferred integration

## Purpose and boundary

Make the committed SCJN invalidity registry independently searchable and expose
source-backed candidate links to affected provisions. Keep registry records
separate from law provisions and from reviewed legal-effect determinations.
The operator explicitly defers implementation; this plan does not expand M2.
No automatic invalidation, commencement-date assignment or change to audited
human decisions is authorized.

## Baseline and authority

Immutable initial baseline: DB branch `611171e52062c67d0ef000cf42c72b87af4c4ef3`,
corpus baseline `bfcda816f46b8b0c1d694fc1f2cfb8b4438275b2`.
Read only the task plan, applicable instructions, database contracts and the
three named files under `corpus/reference/scjn-invalidity-registry/`:
`source-manifest.json`, `DeclaracionesInvalidez.pdf` and
`DeclaracionesInvalidez_lex-mex.md`. The manifest records PDF SHA-256
83f9e9b72df3a09991baa424ebd2599c0ee8a6084b4bf1bb94b3b8f4e4355593,
1,499 converted records numbered through 1,502, gaps 266/267/519 and 19 empty
effect-date cells. These are recorded baseline expectations, to be remeasured.
The conversion is evidence, not an audited legal interpretation. Its original
source URL and retrieval date remain provenance. No live source was consulted
for this plan. Later official judgments/publications must be obtained when a
legal determination depends on them; a registry row alone does not settle scope.

## Milestones and verification

1. **Freeze and audit extraction.** Verify source hashes, count/gaps and field
   inventory against the pinned files. Sample multiline/continuation rows,
   missing dates and ambiguous affected-text descriptions against PDF pages.
   Write a closed schema and Rust types for registry identity, source row/page,
   exact source fields, parsed date alongside its raw text, provenance and
   review state. Preserve null/unknown values. Deterministic extraction must
   retain byte/character locators into unchanged committed evidence. Fixtures
   cover continuation cells, Unicode, empty/ambiguous dates and duplicate IDs.
   Stop on count or source drift; report it rather than invent missing records.
2. **Project and search separately.** Add versioned migration/build contracts
   and dedicated registry tables, ordered source fields and FTS, keyed by
   snapshot plus source number. Include input path, source pointer/span, raw
   bytes and hashes. Search results identify themselves as registry evidence
   with source pages; keep them out of provision counts and provision FTS.
   Independently reconstruct all records from SQL and compare with pinned
   inputs. Test repeat builds, null/raw-field parity and search exclusions.
3. **Generate candidate links.** Resolve explicit instrument/article citations
   using existing canonical IDs and exact source spans. Preserve unresolved,
   ambiguous, multiple-target and out-of-corpus cases. Separate deterministic
   citation matches from model proposals and reviewer-approved relationships.
   Validate every resolved target against committed corpus. Fixtures cover
   historical numbering, state/local instruments and one-to-many declarations.
   Never infer that a matched current article is the text adjudicated.
4. **Review legal effects and publish.** JRH reviews scope, effective date,
   source authority and mapping to the relevant version using formal sources
   where required. Store reviewer/time/rationale/source links/prior proposal;
   keep registry-reported effect dates distinct from reviewed effects and
   provision temporal status. Expose accepted links and clearly marked pending
   evidence through the database contract, without modifying canonical text.
   Require reviewed schema/type/validator changes, native repository gates,
   independent full-release verification and inspection of review/export output
   before acceptance. Search/UI work is separate unless explicitly authorized.

## Recovery and stop conditions

Build only into a new versioned staging release; keep the previous artifact
usable. Reject unknown schema, broken hashes/locators, duplicate identities,
missing resolved targets or parity failures before publication. Preserve raw
sources, pending reviews and prior decisions across reruns. On ambiguous legal
scope, retain evidence and queue it for JRH. No source replacement, canonical
correction or deployment is part of this deferred plan.

## Current checkpoint and next action

2026-10-20: planning only. Current DB retains all three files as evidence inputs;
no registry table, FTS or legal-effect loader is implemented. M1 is accepted;
M2 continues without SCJN semantic integration.

Exactly one next action: after the core DB milestones are accepted and this
work is authorized, execute milestone 1 against a newly pinned source snapshot.
