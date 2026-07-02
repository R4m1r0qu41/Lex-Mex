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

The workspace currently contains 13 passing unit tests.

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

`lex-export` tests:

- Produces stable presentation filenames.
- Publishes generated notes beneath the corpus boundary without modifying
  human-authored notes.
- Emits structured transitory-effect sections in Obsidian notes.

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

## 4. What is tested: current verification results

Checks rerun successfully on 2026-07-02:

| Check | Result |
|---|---:|
| `cargo fmt --check` | Pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | Pass |
| `cargo test --workspace` | Pass: 13 tests |
| `cargo run -p lex-cli -- validate lritf` | Pass |
| LRITF articles | 145 |
| LRITF original transitories | 11 |
| Structural validation issues | 0 |
| Temporal determinations | 19 |
| Structured transitory effects | 32 |
| Machine-accepted determinations | 18 |
| Lawyer-verified determinations | 1 |
| Pending legal reviews | 0 |
| Resolved review records retained for audit | 1 |
| Markdown files in the Obsidian vault | 162 |

The tracked Git worktree was clean before this status document was created.

## 5. What is pending

### Immediate product gaps

- Canonical express-reference extraction and graph records are not implemented.
- Article references in source text are not yet converted into Markdown or
  Obsidian links, so hover previews for referenced articles are not available.
- Compound citations, paragraph/fraction qualifiers, ranges, and Bis/Ter/Quáter
  targets still need reference-parser coverage.
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
- Express-reference and graph validation tests do not exist because that layer
  has not yet been implemented.
- Obsidian link resolution and hover behavior have not yet been tested because
  internal article links have not been generated.

## 6. Suggested next steps

### Step 1 — Complete LRITF express-reference linking

Implement a canonical `ReferenceEdge` model containing:

- source provision ID;
- exact source span and character offsets;
- target instrument and provision ID;
- reference qualifiers such as paragraph, fraction, and inciso;
- basis `express_cross_reference`;
- confidence and resolution status.

Add deterministic extraction for individual, compound, and ranged LRITF
article references. Validate that every internal target exists. Keep canonical
source text unchanged and inject links only during export.

Expected Obsidian output:

```markdown
[[Corpus/LRITF/articulo-48|artículo 48]]
```

With Obsidian Page Preview enabled, clicking or hovering the reference should
show the target article.

### Step 2 — Use LRITF transitories as the first linker acceptance fixture

Transitories provide compact, high-value compound citations. Required fixtures
should include:

- a single article reference;
- `artículos 48, 54 y 56`;
- paragraph and fraction qualifiers;
- repeated references in one transitory;
- a reference to the same law versus an external instrument;
- unresolved external references that do not create broken Obsidian links.

Acceptance target: every internal LRITF article citation resolves, generated
links preserve the displayed source wording, and canonical text remains
byte-for-byte unchanged.

### Step 3 — Ingest the January 28, 2021 CNBV DCG

After the LRITF linker is reliable, add a CNBV source adapter and ingest:

> Disposiciones aplicables a las instituciones de fondos de pago electrónico a
> que se refieren los artículos 48, segundo párrafo; 54, primer párrafo y 56,
> primer y segundo párrafos de la Ley para Regular las Instituciones de
> Tecnología Financiera.

This is the preferred second instrument because it is manageable, comes from an
official CNBV/DOF source, and immediately tests cross-instrument links from the
DCG back to LRITF Articles 48, 54, and 56.

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

The next milestone should be **LRITF internal express-reference linking with
working Obsidian hover previews**. The CNBV DCG should follow immediately as the
first cross-instrument linking test.
