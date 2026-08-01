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
- NOM/NMX support remains a separate standards-capable schema/parser boundary
  and publishes five reviewed official-source NOM compilations without
  relabeling them as statutes or regulations;
- Maximasa consumes and verifies the five-NOM bundle as candidates, without
  inferring applicability or human legal/technical approval.

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

On 2026-07-25 the operator authorized the next real standards sequence after
AD1 `lgbn`: NOM-251-SSA1-2009, NOM-247-SSA1-2008,
NOM-051-SCFI/SSA1-2010, NOM-002-STPS-2010, and
NOM-187-SSA1/SCFI-2002. The official Secretaría de Economía registry marks
NOM-187 current and supplies a consolidated PDF incorporating the 2011 and
2013 modifications. Its 2 January 2023 record is a systematic review with the
result “Modificación,” not evidence that a successor entered into force.

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
- [x] (2026-07-25) Completed AD1 by landing two fixture-backed parser fixes at
  `74fb04e98` and committing the inspected, frozen, zero-error `lgbn` corpus
  at `736146bd8`.
- [x] (2026-07-26) Extended the standards compiler only as needed to publish committed
  standards records and expose them through the consumer CLI and deterministic
  bundle boundary at `0c880735c`, with follow-up trusted-boundary and parser
  corrections committed before dependent standards.
- [x] (2026-07-26) Ingested and individually inspected the five authorized
  NOMs in order: NOM-251 (`0b49379e8`), NOM-247 (`4084597f5`), NOM-051
  (`a44f8aa94`), NOM-002-STPS (`e6e396625`), and NOM-187 (`a8d54e340`).
- [x] (2026-07-26) Returned the five-NOM, 20-file bundle lock and
  candidate-only evidence to Maximasa at `0761ee1`; its five tests pass
  without promoting applicability.
- [x] (2026-07-26) Checked platiica's individual NOM-247 and NOM-051 registry
  pages (the same records the catalog
  `https://platiica.economia.gob.mx/normalizacion/catalogo-mexicano-de-normaswd_asp-id29/`
  resolves to) and the underlying DOF modification decrees against both
  official domains (`diariooficial.gob.mx`, `www.dof.gob.mx`). NOM-247's two
  decrees are narrow numeral-level amendments with no official consolidated
  republication; NOM-051's 2020-03-27 decree is a full restatement of the
  norm. Re-sourced NOM-051 from that 2020 PDF at `7cd482e5e`, landing a
  fixture-backed parser fix first at `2e31f1106` after the refresh exposed a
  real defect (a restarted, numbered Bibliografía reference list
  coincidentally realigning with the outer top-level clause count — all five
  previously-committed NOM corpora reparse byte-identical after the fix).
  NOM-051 now carries zero `standard_unconsolidated_modification` warnings.
  NOM-247's two warnings are left as the correct, currently unclearable
  state: incorporating them would require Lex-Mex to perform its own legal
  consolidation, which the NOM-187 precedent already rejected as out of
  scope.
- (2026-07-26) Flagged, not chased further: the refreshed NOM-051's two
  `included_in_source: true` flags for the 2010-10-26 and 2014-08-14
  modifications are carried forward from the prior metadata, not
  independently re-derived against the new 2020 text. Both dates are
  confirmed genuine targeted NOM-051 amendment decrees (fetched and read
  directly), distinct from platiica's separately listed 2011-03-23 and
  2012-01-17 "Procedimientos para la evaluación de la conformidad"
  companion documents, which are correctly excluded from `modifications` as
  administrative procedure instruments rather than textual amendments — as
  are the 2019 draft (`PROYECTO de Modificación`, never finalized) and the
  2025-05-09/2025-07-31 `ACUERDO` entries extending the 2020 modification's
  phased-implementation deadlines (administrative timeline extensions, not
  text changes; the retained transitorios still name the original,
  now-superseded phase dates). A spot check for the exact numeral IDs the
  2010 decree added (e.g. `4.2.8.3.9`, `4.3.1.3`) did not find them under
  the same numbers in the 2020 text, but this is inconclusive rather than
  contrary evidence: the 2020 restatement substantially renumbered the
  standard around the new front-of-pack warning-seal system, and a
  "para quedar como sigue" full restatement is written to be the complete
  current text as of its date, not a selective one. Treat these two flags
  as an assumption pending confirmation, not an independently verified
  fact.
- [x] (2026-07-27) Landed Scope 1 (standards transitorio inspection):
  `StandardTransitory` type, parser, validator, CLI/bundle wiring, schema,
  and doc updates at `6c7015644`/`bc3ecf906`/`4e0794480`, with three real
  parser defects found and fixed against NOM-051's actual retained text and
  a 10-block backfill across all five committed NOMs (zero clause-count
  regression). Full detail in `docs/decisions.md` 2026-07-27 and
  `docs/standards-module.md`'s "Standards transitorio inspection" section.
- [x] (2026-07-27) Mechanically refreshed the Maximasa five-NOM bundle again
  to pick up `transitories.json` becoming a required canonical bundle file
  (4→5 files per standard, 20→25 total): regenerated
  `nom-bundle-manifest.json` via `lex-mex bundle create` at Lex-Mex
  `a3a48296f`, reran Maximasa's `build_demo_data.py`, updated the hardcoded
  bundle-lock test expectations (`selected_files_checked` 20→25, new
  sha256), and confirmed all 14 Maximasa tests pass with every locked file
  verified against the live corpus. No NOM was rechecked against its
  official source — this was the standing mechanical-only refresh per
  operator instruction. Also added a note to
  `nom-candidate-package.yaml`/`OPEN_QUESTIONS.md` O-7 recording NOM-051's
  now-machine-visible stale transitorio phase date (2025-10-01 vs.
  2028-01-01).
- [x] (2026-07-27) Staged Scope 2 (decree-diff engine) as recorded future
  scope under M4 rather than starting it; the Maximasa NOM ingestion and
  processing slice is closed for now pending further operator direction.
- [x] (2026-07-31) Landed Scope 2 Stage A (clause-level amendment marks) on
  operator sign-off: `title` on `StandardModificationSource` as pure input,
  `amended_by` derived onto `StandardClause` with the decree's own action,
  three new validation warnings distinguishing *targets found* / *title names
  nothing* / *no title recorded*, both schemas, six new tests including the two
  real NOM-247 decree titles quoted verbatim from DOF, and a new
  `standards refresh` command that re-derives a committed standard's parsed
  files from its retained text. Premise verified first: 19 of the 21 numerals
  the NOM-247 titles name match a committed clause exactly. Corpus effect is
  narrow — 26 of 29 standards refresh to a zero-byte diff; NOM-247 gains 17
  marked clauses (`clauses.json` diff is `amended_by` insertions only, no
  clause text, id, number, or span moved) and NOM-020 correctly stays at
  instrument level because the STPS "ACUERDO de Modificación a la NOM-..."
  title form names no numeral at all. `cargo fmt`, `cargo clippy
  --all-targets` (clean), and 123 workspace tests pass; all 29 standards
  revalidate. **Follow-up owed: NOM-247 is in the Maximasa five-NOM bundle, so
  that out-of-repo bundle lock is stale.** Detail in
  `docs/plans/standards-amendment-marks.md` and `docs/decisions.md`.

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

- Decision: treat NOM-187-SSA1/SCFI-2002 as current for this ingestion only
  from the official registry and its consolidated official PDF. Record the
  2023 systematic review as lifecycle evidence, not as a succession event.
  Rationale: the registry explicitly reports `Estado de la Norma: Vigente`,
  supplies the consolidated text, and labels the 2023 event `Modificación`;
  no replacement designation or effective successor is identified.

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

- Decision: incorporating an unconsolidated NOM modification requires an
  official consolidated/full-restatement text; Lex-Mex does not merge a
  targeted DOF amendment decree's replacement language into a standard's
  clauses itself. Checked against
  `https://platiica.economia.gob.mx/normalizacion/catalogo-mexicano-de-normaswd_asp-id29/`
  (the operator-supplied authoritative NOM/NMX catalog) and both DOF domains.
  NOM-051's 2020-03-27 decree is a full restatement (same pattern as the
  NOM-187 consolidated PDF already accepted) and was re-sourced at
  `7cd482e5e`. NOM-247's 2011 and 2012 decrees are narrow numeral-level
  amendments with no consolidated republication anywhere in that catalog, so
  its two `standard_unconsolidated_modification` warnings stay in place.
  Rationale: self-consolidating a decree's text would mean Lex-Mex
  performing the legal consolidation no publisher has performed, which
  conflicts with the NOM-187 precedent of only treating officially
  consolidated text as current.

- Discovery: the numbered-clause standard parser could misread a standard's
  own Bibliografía reference list as continuing top-level clauses when that
  list restarts its numbering at 1 and later happens to reach the value that
  would follow the Bibliografía heading in the outer section count. Exposed
  by NOM-051's 2020 source (a 157-entry numbered bibliography); fixed at
  `2e31f1106` with a fixture distinguishing it from the legitimate case
  (nested `N.1`, `N.2` bibliography sub-clauses, already present in NOM-247).
  All five previously-committed NOM corpora reparse byte-identical, so the
  fix needed no companion corpus changes. The load-bearing part: the buggy
  parse validated clean (312 clauses, `valid`, 0 issues, consecutive
  top-level numbering, correct nesting) — nothing in `validate_standard`
  can detect a wrong-but-internally-consistent clause tree. It was only
  overturned because `numbered_body_run`'s `max_by_key(selected.len())`
  happened to prefer the correct run by a 166-vs-157 node margin, a
  data-dependent margin, not a structural guarantee. The reusable check for
  the next standard with a numbered bibliography: compare the parsed
  top-level run's last clause number against the índice's own highest
  section number before trusting a clean validation report.

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

Operator-flagged future scope, not yet committed: (1) a small, reusable NOM
consolidation workflow — review the platiica catalog record, check official
DOF sources, and compile locally through the Rust pipeline once established
— generalizable to other non-compiled norms beyond NOM-247, meant to stay
lightweight rather than a large automation effort; (2) the platiica catalog
record for a NOM also names the parent law(s)/regulation(s) it derives from,
useful for backlinking and establishing competent authorities for audits and
compliance — not yet modeled in `StandardMetadata`; (3) **Scope 2 — decree-
diff engine**, staged 2026-07-27, not started. Standards transitorio
inspection ("Scope 1," `docs/decisions.md` 2026-07-27) landed first and is
closed: `StandardTransitory` blocks with span-addressable text and a
regex-scanned `asserted_dates` field, reusing the statute ordinal
recognizer, deliberately not a structural parse. Scope 2 is the harder
follow-on: parse a MODIFICACIÓN/ACUERDO decree's own ellipsis-diff
("unchanged span ... replacement in full") into a deterministic
substitution, apply it to a target clause or transitorio already in the
corpus, and track ACUERDO supersession chains (e.g. NOM-051's two 2025
ACUERDOs moving `transitory:segundo`'s phase date from 2025-10-01 to
2028-01-01 — currently machine-visible as stale via `asserted_dates` but
not resolved). It needs a new `derived_consolidation` text-basis variant
distinct from `as_published`/`official_compilation`, and a retained-text
strategy for derogation-caused span shifts. Not scoped in code; do not
start without a fresh planning pass and operator sign-off on the trusted-
boundary shape first — this is exactly the kind of schema/parser/validator/
fixture change M4 requires landing together. **Decomposed 2026-07-29** into
Stage A (clause-level amendment marks, planned in
`docs/plans/standards-amendment-marks.md`), Stage B
(multi-source provenance), and Stage C (actual consolidation, which depends on
the unresolved `transitory-absorbs-annex` defect because NOM-247's second
decree eliminates an entire Apéndice normativo).

**Stage A is signed off and landed, 2026-07-31** — schema, `lex-core` types,
title parser, validator, fixtures, CLI, corpus backfill, and documentation
together, per this milestone's own rule. A modifying decree's DOF title is
recorded verbatim as pure input on `StandardModificationSource`; the parser
derives `amended_by` marks on matching clauses (carrying the decree's own verb,
so an eliminated clause is never rendered as merely modified) and validation
warnings for every named unit that resolves to no committed clause. **No text
is applied and no consolidated text is produced** — a marked clause's text
remains exactly the base publication, and the mark asserts *known staleness,
precisely located*. Full acceptance table, the three deliberate deviations from
the signed-off shape, and two findings the plan did not anticipate are in
`docs/plans/standards-amendment-marks.md` and `docs/decisions.md` (same date).
**Stage B is signed off and landed, 2026-07-31.** `source_sha256: Option<String>`
on each `modifications[]` entry (`StandardModificationSource`) pins that
decree's own source bytes, reusing the per-decree list that already existed
via `official_url` rather than adding a second array. Additive: unset by
default, so all 29 committed standards revalidate byte-identically (confirmed
via `standards refresh` corpus-wide, zero diff) and both schemas re-validate
at 0 violations. No decree PDF is fetched or hashed by this change — that
stays deferred, same acquisition-drift reasoning as the CNBV marker-cap fix
the same day. Full reasoning: `docs/decisions.md` 2026-07-31.

Stage C remains unstarted, still sitting on `transitory-absorbs-annex` and
the annex-modeling decision.

**Design proposal, 2026-07-31 (`docs/decisions.md` same date), leading
candidate for Stage C's engine — awaiting sign-off, not implemented.**
Validated against a real pilot outside the standards module (SHCP/CNBV art.
115 LIC disposiciones, no consolidated text either, `docs/plans/cnbv-art115-
lic-consolidation.md`), which found the *same* ellipsis-diff mechanism NOM
MODIFICACIÓN decrees use, just nested one level deeper (inside a named
unit's sub-parts, not only across a whole clause). One model now covers
both: build canonical text as a **strict left fold over the decree history
in chronological order** (`canonical := base; for decree in
chronological order: canonical := apply(canonical, decree)`), where
`apply()` has exactly three per-unit operations — **`replace`** (decree
gives explicit new text), **`keep`** (ellipsis: content and position both
unchanged), and **`shift`** (a "recorriéndose los demás en su orden"
renumbering named in the decree's own resolving-clause prose: content
unchanged, position moves — collapsing this into `keep` is the specific way
a naive implementation silently mislabels a provision). The resolving
clause's own prose descriptions (particularly derogations and reordering)
must be applied even when the decree gives no text restatement for that
unit, and the clause's REFORMAN/DEROGAN/ADICIONAN lists must be checked
against what its replacement body actually contains before applying
anything — that cross-check is what surfaced both a 5-item gap list and 46
ellipsis-affected provisions in the pilot compiled draft. This also answers
the open "retained-text strategy for derogation-caused span shifts"
question directly above: **there is no span shift, because nothing is ever
deleted or renumbered** — a repealed unit becomes `derogado` with the
repealing decree's date/codigo recorded in place, matching how DOF's own
compiled texts render repeals. The fold itself is a plain sequential
reduction per instrument (no graph structure needed); cross-instrument
reference resolution stays the separate, already-deferred concern it was.

(4) **packet-based review assignment, signed off and landed 2026-07-31**
(`docs/decisions.md` same date): `lex-mex review-packets generate` groups
already-committed instruments by `batches/*.json`'s `batch_id` (30 packets,
147 of 181 committed instruments today) and `assign`/`list` hand each to a
reviewer, tracked in a new `ReviewPacket` record distinct from
`legal_review_status`/`technical_review_status` — assignment workflow only,
no verdict. Standards and the CNBV DCG family have no batch manifest, so
neither is covered yet. A way for a reviewer to flag a missing backlink on
the fly stays explicitly deferred, as originally scoped.

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

Resume the independent federal cluster-2 sequence at AD2 by normalizing its
prepared inventory and provisionally ingesting `lspm`. The Maximasa NOM
ingestion and processing slice is closed for now: NOM-051 is re-sourced and
zero-warning, transitorio inspection (Scope 1) is landed and backfilled
across all five standards, and the Maximasa bundle/candidate-package/test
lock are all refreshed and passing against the current Lex-Mex HEAD.
NOM-247 has no viable official consolidated text and is left as correctly
unconsolidated — but as of 2026-07-31 its staleness is located at clause level
rather than instrument level (Scope 2 Stage A, landed). Scope 2 Stages B and C
are staged as recorded future scope, not started. Any further work on NOM-247,
Stages B/C, the flagged NOM consolidation workflow, or parent-law/regulation
backlinking waits on operator direction rather than proceeding unprompted.

**Owed, 2026-07-31:** Stage A changed `corpus/mx/nom-247-ssa1-2008/clauses.json`
(`amended_by` insertions only), and NOM-247 is one of the five NOMs in the
Maximasa bundle — so the bundle lock below is stale and needs the standing
mechanical-only refresh (regenerate `nom-bundle-manifest.json`, rerun
`build_demo_data.py`, update the hardcoded lock sha256, confirm 14 tests). Not
done here: it is a cross-repository write.

The five-NOM bundle returned to Maximasa was current as of Lex-Mex
`a3a48296f` (mechanically refreshed 2026-07-27 to pick up the new
`transitories.json` canonical file); Maximasa's 14-test suite passed with
every locked file verified against the live corpus at that point.

## Outcomes and retrospective

The full Maximasa return and five-standard sequence is complete through
Lex-Mex `a8d54e340` and Maximasa `0761ee1`. The real consumer work closed the
RCSPS source gap, exposed and fixed parser/search defects, proved both
selected-corpus portability contracts, and published five standards without
inferring legal applicability.

The follow-up modification-incorporation slice re-sourced NOM-051 from its
2020 official consolidated text (`7cd482e5e`) after a fixture-backed parser
fix (`2e31f1106`) that the refresh's real content exposed; the corpus now
carries 193 non-blocking warnings (down from 194), and NOM-051 has zero
`standard_unconsolidated_modification` warnings. NOM-247 keeps both of its
warnings by design: no official consolidated text exists for its two DOF
decrees, and Lex-Mex does not perform its own legal consolidation. NOM-187
and NOM-251 remain zero-warning canonical candidate sources; NOM-002-STPS is
unaffected. Remaining work is human/fact gated, the operator-flagged NOM
consolidation workflow and parent-law backlinking (not yet scoped), or the
independent cluster-2 ingestion plan.

Standards transitorio inspection (Scope 1) followed, adding addressable
`StandardTransitory` blocks and a narrow `asserted_dates` scan for all five
standards without attempting a structural parse of transitorio content.
It made one fact machine-visible that clause-level validation cannot see:
NOM-051's transitorio SEGUNDO still asserts its original 2020 phase date
even though two later ACUERDOs moved it — text currency and transitorio-
date currency are separate claims. Closing that gap is Scope 2 (the
decree-diff engine), staged as future scope rather than started. The
Maximasa bundle/candidate-package/test lock were refreshed mechanically to
match, closing the Maximasa NOM ingestion and processing slice for now.
