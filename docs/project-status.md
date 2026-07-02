# Lex-Mex Project Status

**Status date:** 2026-07-02  
**Repository:** <https://github.com/R4m1r0qu41/Lex-Mex>
**Current instrument:** Ley para Regular las Instituciones de Tecnología Financiera (LRITF)  
**Current legal reviewer:** JRH

## 1. Project summary

Lex-Mex is a provenance-aware compiler and update engine for Mexican legal
materials. It is intended to acquire official legal sources, preserve their
provenance, convert them into stable canonical records, analyze temporal legal
effects, route genuine uncertainty to human review, and publish useful Markdown
and Obsidian views without treating Obsidian as the canonical database.

The current vertical slice processes the consolidated LRITF published by the
Cámara de Diputados. The Cámara text is treated as the operational source for
the consolidated law, while Diario Oficial de la Federación (DOF) publications
are attached as formal promulgation and amendment sources.

The canonical implementation is written in Rust. Model output may propose legal
classifications, but Rust owns validation, state transitions, review routing,
canonical data, and publication.

## 2. What has been implemented

### Repository and core architecture

- Rust workspace with five focused crates:
  - `lex-core`: canonical types and temporal/review state transitions.
  - `lex-source`: source configuration, acquisition, metadata, and hashing.
  - `lex-parse`: PDF extraction orchestration, LRITF parsing, reform-transitory
    isolation, and structural validation.
  - `lex-export`: canonical JSON, standard Markdown, and Obsidian publication.
  - `lex-cli`: user commands and pipeline orchestration.
- Stable canonical identifiers such as
  `urn:lex-mx:federal:statute:lritf:article:48`.
- Versioned source-manifest, temporal-analysis, and structured-output schemas.
- Architecture and operational decisions recorded in `docs/decisions.md`.

### Source acquisition and provenance

- Cámara de Diputados adapter for LRITF.
- Official PDF download with HTTP metadata and SHA-256 hashing.
- Temporary PDF extraction through Poppler `pdftotext`.
- Extracted-text SHA-256 recording.
- Source URL, publisher, retrieval timestamp, content type, content length,
  parser version, and schema version preserved in the source manifest.
- Temporary work products excluded from Git.
- Formal DOF source URLs mapped by publication date for reviewed transitories.

Current recorded source hashes:

- PDF SHA-256:
  `d6f645e6a7d3c2eeb46905d4d24ecd8e078907057dc034cda715abf019ce8491`
- Extracted-text SHA-256:
  `429a8916f3b1aa7035c0b700e27cd132a3af1662b1661ac703b9b0c7847b25a6`

### Parsing and canonical corpus

- Article-level parsing of 145 LRITF articles.
- Parsing of the law's 11 original transitory provisions.
- Isolation of reform-decree transitories from ordinary provisions.
- Explicit adapter filtering of reform transitories relevant to LRITF.
- Preservation of headings, source text, order, publication dates, temporal
  annotations, review state, and structured transitory effects.
- Canonical JSON stored under `corpus/mx/lritf/`.
- Structural validation report generation.

### Express-reference graph

- Canonical `ReferenceEdge` records stored in
  `corpus/mx/lritf/references.json`.
- Exact source spans and Unicode character offsets without modifying canonical
  provision text.
- Deterministic support for individual, compound, repeated, qualified, and
  ranged article citations.
- Paragraph, fraction, and subsection qualifier metadata.
- Explicit direct versus range-expansion reference forms.
- Resolved/unresolved status, confidence, and `express_cross_reference` basis.
- Validation of source spans, offsets, duplicate IDs, internal instrument
  identity, confidence, and target existence.
- Named external-law references remain unlinked until the target instrument is
  represented in the corpus.
- A standalone `link lritf` stage regenerates references without reparsing or
  changing temporal-review state.

### Temporal analysis v2

- Versioned `temporal-v2` prompt and strict JSON output schema.
- Authenticated Codex CLI model runner and provider-neutral response importer.
- Request and raw-response SHA-256 provenance.
- Deterministic validation of:
  - exact evidence identifiers;
  - one determination per evidence item;
  - duplicate, missing, and unknown identifiers;
  - confidence ranges and date ranges;
  - exact supporting quotations against source evidence;
  - valid provision-status and transitory-effect structure.
- Separation between a transitory provision's own temporal status and the legal
  effects created by that transitory.
- Structured effect classifications for:
  - commencement;
  - implementation and regulatory deadlines;
  - adaptation periods;
  - transitional permissions;
  - procedural survival and old-rule cohorts;
  - migration to a new regime;
  - authority assignment and coordination;
  - staged commencement;
  - sunsets, repeal, and uncommon effects.
- Structured triggers, end conditions, affected scope, responsible authorities,
  application rules, and verification status.
- Open-ended cohort exhaustion is represented as `open_ended_by_design`, not as
  legal uncertainty.
- Clear rules requiring a later factual check are represented as
  `external_verification_required`, not as legal ambiguity.
- Confirmed external events can record an official source URL, event date, and
  verification note.

Current temporal result:

- 19 determinations.
- 32 structured legal effects.
- 18 machine-accepted determinations.
- 1 lawyer-verified determination.
- 0 pending legal reviews.

### Human review workflow

- Confidence- and ambiguity-based review routing.
- Review queue with exact issue, evidence, proposed conclusion, Cámara source,
  DOF source, review options, reviewer note, and audit metadata.
- `review list`, `review list --all`, and audited `review resolve` commands.
- Reviewer identity required for resolution.
- Lawyer override support for provision status, dates, and complete structured
  effects supplied through `--effects-file`.
- Resolved decisions survive later model-response imports.
- Resolved items remain in JSON audit history but disappear from the pending
  Obsidian dashboard.
- JRH's resolution of LRITF SÉPTIMA is recorded as `lawyer_verified`:
  - the twelve-month period begins on 2018-03-10;
  - the relevant dispositions were published on 2021-01-28;
  - their late publication is recorded as externally verified;
  - separate factual verification of the Article 71 coordination agreement
    remains outstanding but is not a pending legal-review question.

### Markdown and Obsidian

- Standard Markdown export for every provision.
- Optional external Obsidian vault (for example, `$HOME/Vaults/Lex-Mex`).
- Generated LRITF index and article/transitory notes.
- Frontmatter with canonical identity, temporal status, review status, source
  URL, and source hash.
- Structured transitory effects displayed beneath the source text.
- Standard Markdown links and Obsidian wikilinks injected from the validated
  reference graph at export time.
- 95 resolved internal LRITF links across 26 generated provision notes,
  including all numeric references in SÉPTIMA.
- Generated pending-review dashboard.
- Export ownership limited to generated corpus paths; human-authored `Notas/`
  and `Revisiones/` remain outside the exporter boundary.
- Current vault contains 162 Markdown files and reports no pending temporal
  reviews in its generated dashboard.

### CLI workflow

Implemented commands include:

```text
discover diputados
fetch lritf
extract lritf
parse lritf
link lritf
analyze-temporal lritf
import-temporal lritf <response>
validate lritf
export lritf --format json|markdown|obsidian
pipeline lritf
review list [--all]
review resolve <review-id>
```

## 3. What has been tested

### Automated test coverage

The workspace currently contains 15 passing unit tests.

`lex-core` tests:

- Rejects supporting quotations not found verbatim in source evidence.
- Accepts open-ended procedural survival without unnecessary legal review.
- Separates external factual verification from legal ambiguity.
- Routes materially unknown effects to review.
- Rejects unaudited or incomplete lawyer overrides.
- Applies audited lawyer overrides and preserves `lawyer_verified` basis.
- Preserves resolved human decisions across later model reruns.

`lex-parse` tests:

- Parses articles and original law transitories without page furniture.
- Verifies expected article/transitory counts and coherent ordering.
- Isolates reform-decree transitories as separate temporal evidence.
- Extracts compound, qualified, repeated, ranged, and same-law references while
  excluding a named external-law reference.

`lex-export` tests:

- Produces stable presentation filenames.
- Publishes generated notes beneath the corpus boundary without modifying
  human-authored notes.
- Emits structured transitory-effect sections in Obsidian notes.
- Injects resolved standard Markdown and Obsidian links without changing
  canonical source text.

`lex-source` test:

- Produces a known SHA-256 value correctly.

### Manual and integration testing completed

- Official LRITF PDF pages were visually compared against selected extracted
  articles and transitories.
- A complete network acquisition, extraction, parsing, temporal analysis,
  validation, Markdown export, and Obsidian publication cycle was executed.
- The v2 model returned all 19 requested determinations and 32 effects.
- Deterministic import rejected an initially over-restrictive invariant; the
  invariant was corrected and regression-tested before import.
- An incomplete lawyer override was rejected without changing corpus hashes.
- JRH's SÉPTIMA decision was applied through the actual CLI review workflow.
- The original model response was reimported afterward to verify that JRH's
  resolved decision survived unchanged.
- The pending Obsidian review dashboard was verified to be empty after the
  resolution.
- The reference graph was generated from the reviewed corpus without changing
  `provisions.json`, the instrument, temporal result, or review queue.
- All 95 canonical LRITF references resolve to existing article records.
- Standard Markdown and the external Obsidian vault were republished with 95
  links; SÉPTIMA links Articles 48, 54, 56, and 71.

## 4. What is tested: current verification results

Checks rerun successfully on 2026-07-02:

| Check | Result |
|---|---:|
| `cargo fmt --check` | Pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | Pass |
| `cargo test --workspace` | Pass: 15 tests |
| `cargo run -p lex-cli -- validate lritf` | Pass |
| LRITF articles | 145 |
| LRITF original transitories | 11 |
| Resolved internal article references | 95 |
| Provision notes containing internal links | 26 |
| Structural validation issues | 0 |
| Temporal determinations | 19 |
| Structured transitory effects | 32 |
| Machine-accepted determinations | 18 |
| Lawyer-verified determinations | 1 |
| Pending legal reviews | 0 |
| Resolved review records retained for audit | 1 |
| Markdown files in the Obsidian vault | 162 |

Canonical provision text, temporal results, and the audited review queue were
unchanged by the reference-graph migration.

## 5. What is pending

### Immediate product gaps

- Relative references such as `artículo anterior`, `artículo siguiente`, and
  `este artículo` are not yet canonical graph edges.
- Bis/Ter/Quáter syntax is recognized, but the current LRITF corpus has no
  internal suffixed article target with which to exercise resolved export.
- Defined terms and defined-term usage are not yet extracted or linked.
- Cross-instrument references are not yet resolved.
- The January 28, 2021 CNBV Disposiciones de Carácter General (DCG) have not yet
  been ingested as a second instrument.
- Factual verification of the LRITF Article 71 coordination agreement remains
  outstanding.

### Corpus and update-engine gaps

- The remaining MVP statutes have not been ingested.
- The structural stress-test law and short CNBV test instrument have not been
  formally added to the corpus.
- No Cámara source-change monitor, candidate-version workflow, or
  provision-level diff engine exists yet.
- No automated DOF early-warning monitor or amendment reconciliation exists.
- Amendment-event records and affected-provision diffs are not implemented.
- The source manifest's `resulting_git_commit` field is currently `null` and
  still needs pipeline integration.

### Test gaps

- No automated CLI integration tests currently exercise full command flows.
- Network acquisition and live model execution are manually exercised but not
  suitable for deterministic unit tests.
- No regression corpus yet covers multiple statutes and non-statute CNBV
  document structures.
- No multi-instrument fixture yet exercises resolved cross-instrument links.
- Obsidian CLI unresolved-link verification requires the desktop application to
  be running; this run verified every target deterministically in Rust and
  inspected the generated wikilinks, but did not exercise the live hover UI.

## 6. Suggested next steps

### Step 1 — Ingest the January 28, 2021 CNBV DCG

After the LRITF linker is reliable, add a CNBV source adapter and ingest:

> Disposiciones aplicables a las instituciones de fondos de pago electrónico a
> que se refieren los artículos 48, segundo párrafo; 54, primer párrafo y 56,
> primer y segundo párrafos de la Ley para Regular las Instituciones de
> Tecnología Financiera.

This is the preferred second instrument because it is manageable, comes from an
official CNBV/DOF source, and immediately tests cross-instrument links from the
DCG back to LRITF Articles 48, 54, and 56.

### Step 2 — Resolve cross-instrument references

Generalize reference targets beyond the current instrument. Resolve the DCG's
express references to LRITF Articles 48, 54, and 56, while preserving the exact
DCG citation spans and paragraph qualifiers. Add multi-instrument graph
validation and Obsidian paths.

### Step 3 — Add relative article references

Resolve `artículo anterior`, `artículo siguiente`, `este artículo`, and similar
forms using provision order and exact linguistic constraints. Keep these edges
distinguishable from explicit numeric references.

### Step 4 — Add definitions and defined-term usage

After express article references work:

- extract definition provisions;
- assign stable identifiers to defined terms;
- link exact term usage to its defining provision;
- distinguish express definitions from inferred semantic relationships;
- export backlinks and term indexes to Obsidian.

### Step 5 — Expand the corpus and then implement updates

Proceed with the remaining MVP laws and selected stress-test instrument. Add
parser fixtures and quality metrics for each source. Only after the multi-source
corpus is stable should the project add source-change monitoring,
provision-level diffs, DOF early warning, and amendment reconciliation.

## Recommended immediate milestone

The next milestone should be **ingestion of the January 28, 2021 CNBV DCG**,
followed immediately by its resolved cross-instrument links to LRITF Articles
48, 54, and 56.
