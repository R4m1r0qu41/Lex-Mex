# Maximasa legal-corpus integration and standards module

This is the living execution plan for using Lex-Mex in the first real-world
downstream application: the compliance module in
`/Users/jr/Documents/MAXIMASA/maximasa-os`. Keep Maximasa-specific legal
analysis and search results in that project; keep reusable ingestion,
validation, query, bundle, and standards support in Lex-Mex.

Repository code, canonical corpus data, `AGENTS.md`, the cluster-2 ingestion
plan, official sources, and recorded human legal-review decisions outrank this
plan.

## Purpose and observable outcome

Maximasa receives a versioned, source-grounded package containing only the
federal instruments and candidate provisions relevant to its compliance
module. Lex-Mex records the resulting coverage gaps, admits missing
instruments through the normal batch order, and gains a reviewed design for
NOM/NMX standards without forcing standards into statute semantics.

The observable outcome is:

- the stale Maximasa handoff uses the actual `lex-mex instruments`, `path`,
  and `search` interface;
- the selected federal instruments are validated and searched by bounded
  domain queries, with results returned as candidates rather than legal
  conclusions;
- Ley General de Protección Civil and its regulation enter through prepared
  batch AD4, unless the operator explicitly approves a documented batch-order
  amendment;
- Reglamento Federal de Seguridad y Salud en el Trabajo receives an official
  source inventory and a reviewed cluster-2 batch placement before ingestion;
- a selected-instrument bundle contract feeds Maximasa without copying the
  whole repository;
- NOM/NMX support is specified as a separate standards-capable schema/parser
  boundary and is not implemented by relabeling a statute or regulation.

## Scope and exclusions

In scope:

- federal law/regulation coverage and article-level candidate retrieval;
- official-source inventory for missing federal instruments;
- selected canonical/Markdown bundles with provenance and validation digests;
- NOM/NMX architecture based on the Maximasa candidate register;
- the data contract between Lex-Mex and Maximasa's compliance module.

Out of scope:

- final applicability or compliance determinations;
- autonomous legal-review resolution;
- state, Ciudad de México, alcaldía, land-use, or permit law inside the
  current federal corpus;
- production implementation of `maximasa-os`, which remains gated by its
  Phase-5 roadmap;
- ingestion of a NOM/NMX before the standards schema, parser, source model,
  and validator are reviewed;
- bypassing AD1–AD4 batch order merely because an instrument is important to
  Maximasa.

## Authoritative surfaces

Lex-Mex:

- `docs/plans/cluster-2-federal-corpus-ingestion.md`
- `prompts/lex-mex-federal-cluster-2-plan.md`
- `prompts/cluster-2-batches/lex-mex-cl2-batch-AD4.json`
- `crates/lex-cli/src/main.rs`
- `crates/lex-core/src/lib.rs`
- `crates/lex-source/`, `crates/lex-parse/`, and `crates/lex-export/`
- `corpus/mx/<slug>/`

Maximasa:

- `docs/handoffs/LEX_MEX_MAXIMASA_INGESTION_PROMPT.md`
- `docs/architecture/lex-mex-integration.md`
- `docs/product/compliance-module-specification.md`
- `docs/compliance/nom-register.md`
- `research/lex-mex-results/`
- `schemas/obligation.schema.yaml`

Official-source boundary:

- Cámara de Diputados consolidated law/regulation sources;
- Diario Oficial de la Federación formal publications;
- later NOM/NMX work must use the competent official publisher plus DOF and
  current SINEC/SIICEC status where applicable.

## Initial baseline

The immutable baseline is local Lex-Mex `main` at `1382a6e9e` on 2026-07-25,
three commits ahead of `origin/main`, with a clean worktree and 149 committed
instruments. The consumer CLI milestone passes all required checks.

At this baseline:

- all 18 instrument slugs in Maximasa's candidate manifest are committed;
- neither Ley General de Protección Civil nor Reglamento Federal de Seguridad
  y Salud en el Trabajo is committed;
- `lgpc` and `reg-lgpc` are already prepared in cluster-2 batch AD4;
- Reglamento Federal de Seguridad y Salud en el Trabajo has no prepared
  cluster-2 manifest entry;
- the Maximasa handoff still pins `44fe6ade3` and incorrectly says no
  `search` subcommand exists;
- Maximasa has a 30-entry candidate NOM register but no verified NOM/NMX
  corpus or standards parser.

## Current checkpoint

Live corpus and prepared-inventory comparison is complete. The first Q-001
attempt exposed repeated Markdown-front-matter noise, so consumer search now
defaults to canonical JSON; Markdown remains an explicit presentation scope.
The bounded canonical rerun then exposed 109 provisions contaminated by the
two-line running header in `reg-lgs-mcsaeps`. Exact adapter configuration and
two regression fixtures now remove that furniture, distinguish later
regulation headings in the reform appendix, preserve the 1,356-article /
4-transitory / 38-reference structural baseline, and isolate 21 later-act
transitories under their correct 1998, 1999, and 2004 publication dates.
The official PDF and extracted-text hashes remain unchanged.

Primary official source discovery confirms:

- current LGPC consolidated PDF:
  `https://www.diputados.gob.mx/LeyesBiblio/pdf/LGPC.pdf`;
- Cámara's current federal-regulations index lists Reglamento Federal de
  Seguridad y Salud en el Trabajo with publication date 2014-11-13 and Word
  source `https://www.diputados.gob.mx/LeyesBiblio/regla/n152.doc`;
- the matching formal DOF publication is dated 2014-11-13.

Q-001 remains a candidate package rather than an applicability conclusion.
The 1999 `Reglamento de Control Sanitario de Productos y Servicios` is now
ingested as `reg-csps` through the operator-approved
`maximasa_federal_gap_1` batch-order amendment. Its official Cámara Word
source and original DOF HTML publication are independently hashed. The
inspected frozen structure is 268 articles, five original transitories, one
substantive appendix, 185 references, and zero validation issues. Direct
current candidates now include LGS Articles 115, 194, and 201, RCSPS Article
112, and exact maize/nixtamalized-flour clauses within Appendix IX. The
remaining gate is human legal review and any fact-specific applicability
analysis, not missing federal sanitary text.

## Progress

- [x] (2026-07-25) Committed the external consumer CLI and portability design
  at `1382a6e9e`.
- [x] (2026-07-25) Inspected the Maximasa authority, handoff, compliance
  module, candidate/missing manifests, NOM register, and obligation schema.
- [x] (2026-07-25) Reverified all 18 candidate instruments against live
  `lex-mex instruments --json`.
- [x] (2026-07-25) Confirmed `lgpc`/`reg-lgpc` placement in prepared AD4 and
  absence of the federal workplace-safety regulation from prepared batches.
- [x] (2026-07-25) Used Q-001 as the real-path regression: changed the
  consumer search default from Markdown to canonical records and added a CLI
  parsing test.
- [x] (2026-07-25) Initialized the local Maximasa Git repository and committed
  its 49-file foundation at `6e54f61`; the parent client-deliverables
  directory remains outside Git.
- [x] (2026-07-25) Corrected `reg-lgs-mcsaeps` running-header contamination,
  added exact fixtures, reran its official-source pipeline, and inspected the
  canonical, temporal-evidence, validation, and representative Markdown
  diffs.
- [x] (2026-07-25) Rebaselined the Maximasa integration documents, added its
  root `AGENTS.md`, preserved the concurrent NOM/NMX research pass, and
  committed the handoff at Maximasa `a9d1364`.
- [x] (2026-07-25) Added binary Word-source acquisition/extraction and
  substantive-appendix parsing with focused fixtures, then provisionally
  inspected, froze, linked, validated, and exported `reg-csps` through the
  one-instrument `maximasa_federal_gap_1` batch. The general AD1 order is
  unchanged and `lgbn` remains its next item.
- [x] (2026-07-25) Refreshed Q-001 with `reg-csps`, executed Q-002 through
  Q-009 against bounded instrument lists, and recorded Q-010 as source-blocked
  rather than searching an empty or out-of-scope corpus.
- [x] (2026-07-25) Implemented the selected-instrument bundle manifest and
  CLI with canonical/canonical-plus-Markdown profiles, committed-state and
  validation gates, per-file/source/validation hashes, and excluded-target
  disclosure.
- [ ] Add the federal workplace-safety regulation to a reviewed prepared batch.
- [ ] Complete AD1–AD3 and ingest AD4 through the provisional-inspect-freeze
  sequence.
- [x] (2026-07-25) Added the standards-specific metadata, numbered-clause,
  source-span validation, JSON Schema, fixture, and `standards compile`
  boundary without promoting unverified Maximasa register rows.
- [x] (2026-07-25) Returned the 20-instrument, 161-file bundle lock, exact
  query/extract/gap artifacts, and consumer validation to Maximasa at
  `01c4dc2`; the reproducible 11 MB payload remains outside that repository.

## Decisions and discoveries

- Decision: “Maximasa module” means the Lex-Mex data boundary feeding the
  already authorized Maximasa compliance-module specification during Phase 1.
  It does not authorize production UI/application code before Maximasa Phase
  5.

- Decision: client-specific obligation candidates and quoted search results
  live in `maximasa-os/research/lex-mex-results/`; reusable schemas, parsers,
  validators, CLI behavior, and bundle formats live in Lex-Mex.

- Decision: do not accelerate LGPC by ingesting AD4 out of sequence.
  Rationale: its source is prepared, but the cluster plan deliberately learns
  parser behavior one instrument at a time and currently names `lgbn` as the
  next AD1 action.

- Decision: the occupational-safety regulation needs a source inventory and
  batch-placement review, not an ad hoc adapter.
  Rationale: it is an official federal regulation relevant to the client, but
  it is absent from both corpus and prepared inventory.

- Decision: the initial standards module is a schema/ingestion design, not a
  claim that the 30 client-listed NOMs are current.
  Rationale: the register explicitly marks those identifiers/titles
  unverified; source currency and cancellation/replacement chains are trusted
  data boundaries.

- Discovery: structural validation alone did not detect an exact configured
  title split across two PDF running-header lines. The real consumer query
  found the defect because the repeated title dominated its results. Exact
  source-layout fixtures and representative canonical searches are therefore
  part of the downstream integration gate.

- Discovery: `reg-lgs-mcsaeps` contains later regulation transitories for
  1998, 1999, and 2004. The generic appendix parser previously recognized
  only `DECRETO` headings, so a current rerun initially assigned all later
  sections to 1998 and retained signatures. Mixed-case official
  `REGLAMENTO de/en ...` headings now reset act identity and publication
  date, and signature furniture is excluded.

- Decision: ingestion of the 1999 replacement regulation closes the structural
  source gap but does not itself promote either regulation's matched
  provisions into Maximasa obligations. Authorized legal review and
  fact-specific applicability analysis remain required.

## Milestones and gates

### M1 — Maximasa handoff rebaseline

Update Maximasa's integration and search manifests to:

- distinguish Lex-Mex repository commit from canonical corpus-data commit;
- use `lex-mex instruments --json`, selected `search`, and `path`;
- record Q-001–Q-010 execution status and exact command/query terms;
- preserve every result as `candidate`, `insufficient_company_information`,
  `requires_legal_review`, or `requires_technical_review`.

Stop on an unverified corpus path, dirty/unversioned Maximasa authority, or a
result shape that cannot preserve exact provision ID, text, official source,
source hash, and Lex-Mex commit.

### M2 — Selected-instrument bundle

Implement the bundle contract from
`docs/metadata-portability-and-ingestion-automation.md` with:

- selected slugs and canonical IDs;
- canonical/Markdown profile;
- schema/parser/repository versions;
- source, extracted-text, validation, and included-file hashes;
- explicit external reference targets omitted from the bundle;
- deterministic ordering and reproducible manifest output.

Verify bundle creation for the selected Maximasa slugs without modifying
canonical data.

### M3 — Missing federal instruments

Keep `lgpc`/`reg-lgpc` in AD4. Prepare Reglamento Federal de Seguridad y Salud
en el Trabajo in the closest semantically coherent cluster-2 batch only after
official-source and adapter-family review. Each instrument follows
provisional parse, source inspection, frozen baseline, validation, export, and
individual commit before batch closure.

### M4 — NOM/NMX standards extension

Produce a reviewed trusted-boundary proposal covering at least:

- `nom` and `nmx` identity and instrument types;
- designation, issuing authorities, regulatory domain, publication/current
  status, effective/cancellation dates, replacement chain, and joint prefixes;
- clause/section structure rather than statute-only article assumptions;
- `objetivo`, `campo de aplicación`, conformity-assessment procedure,
  bibliography, concordance, transitories, annexes/tables, and incorporated
  references;
- official DOF source, current SINEC/SIICEC record, extraction method, and
  source/content hashes;
- human legal/technical review states kept distinct;
- fixtures for each source/layout class.

This milestone changes schemas, Rust types, parsers, validators, fixtures,
exporters, and documentation together or not at all.

### M5 — Maximasa return handoff

Return:

- corpus and tooling commits;
- validated included/missing/out-of-scope inventory;
- exact commands and query log;
- extracted candidate provisions and unresolved factual predicates;
- bundle path and digest;
- failures/warnings and reviewer gates;
- recommended compliance-module import action.

## Recovery and stop conditions

Checkpoint after each query domain, bundle implementation, parser/adapter
defect, and ingested instrument. Stop on:

- a source rate limit or official-source identity change;
- unexpected Git drift in either repository;
- an unreviewed schema/trusted-boundary change;
- parser or validator failure;
- a required official source that cannot be attached;
- any attempt to infer JRH/JRHA legal approval;
- any change from candidate to applicable without cited company facts and
  review;
- NOM/NMX status that has not been verified against official sources.

## Next action

JRH reviews the returned Maximasa candidate package while the operator supplies
the missing permits, workforce, wastewater, waste, and emissions facts. Select
one officially current NOM for the first provisional standards real-path
exercise. Independently, the general federal ingestion sequence remains at
AD1 `lgbn`; this client-specific sequence did not advance or reorder it.

## Outcomes and retrospective

The full Maximasa return sequence is complete through Lex-Mex `83fa5d981` and
Maximasa `01c4dc2`. The real consumer work closed the RCSPS source gap, exposed
and fixed parser/search defects, proved the selected-corpus portability
contract, and established a standards-specific compiler without claiming that
any NOM/NMX is already ingested or current. Remaining work is explicitly
human/fact gated or belongs to the independent cluster-2 ingestion plan.
