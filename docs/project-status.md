# Lex-Mex Project Status

**Status date:** 2026-07-03  
**Repository:** <https://github.com/R4m1r0qu41/Lex-Mex>
**Current instruments:** Ley para Regular las Instituciones de Tecnología Financiera (LRITF); Disposiciones aplicables a las instituciones de fondos de pago electrónico (DCG-IFPE-2021)  
**Current legal reviewer:** JRH

## 1. Project summary

Lex-Mex is a provenance-aware compiler and update engine for Mexican legal
materials. It is intended to acquire official legal sources, preserve their
provenance, convert them into stable canonical records, analyze temporal legal
effects, route genuine uncertainty to human review, and publish useful Markdown
and Obsidian views without treating Obsidian as the canonical database.

The corpus now contains two instruments. The consolidated LRITF from the
Cámara de Diputados remains the statute vertical slice. The jointly issued
CNBV/Banco de México disposiciones de carácter general for instituciones de
fondos de pago electrónico, published in the Diario Oficial de la Federación
on January 28, 2021 (slug `ifpe-dcg-2021`, short name DCG-IFPE-2021), is the
first regulation and the first cross-instrument reference target: its express
citations of LRITF provisions resolve against the loaded LRITF corpus.

The canonical implementation is written in Rust. Model output may propose
legal classifications, but Rust owns validation, state transitions, review
routing, canonical data, and publication.

## 2. What has been implemented

### Repository and core architecture

- Rust workspace with five focused crates:
  - `lex-core`: canonical types and temporal/review state transitions.
  - `lex-source`: source configuration, acquisition, metadata, and hashing.
  - `lex-parse`: PDF and DOF-HTML extraction, LRITF and DCG parsing,
    reform-transitory isolation, reference extraction, and structural
    validation.
  - `lex-export`: canonical JSON, standard Markdown, and Obsidian publication
    with cross-instrument link targets.
  - `lex-cli`: slug-routed commands and end-to-end pipeline orchestration.
- Stable canonical identifiers such as
  `urn:lex-mx:federal:statute:lritf:article:48` and
  `urn:lex-mx:federal:regulation:ifpe-dcg-2021:annex:8`.
- Adapter-driven multi-instrument support: instrument identity, parser
  selection, expected counts, issuing authorities, reference markers, and
  configured external instruments live in `adapters/<source>/<slug>.json`.
- Versioned source-manifest, temporal-analysis, and structured-output schemas.
- Architecture and operational decisions recorded in `docs/decisions.md`.

### Source acquisition and provenance

- Cámara de Diputados adapter for LRITF; CNBV adapter for the DCG.
- Official PDF download with HTTP metadata and SHA-256 hashing.
- Direct acquisition of the formal DOF publication for the DCG (código
  5610487), whose note uniquely contains the eight annex bodies, recorded in
  `formal-source-manifest.json`.
- Deterministic DOF HTML text extraction in Rust (block structure, table
  cells joined with ` | `, DOF character entities).
- Public intermediate CA certificates shipped in the CNBV adapter because
  both official hosts serve incomplete TLS chains; each chains to a standard
  root.
- Temporary PDF extraction through Poppler `pdftotext`; the DCG keeps
  form-feed page markers for deterministic paragraph merging.
- Source URL, publisher, retrieval timestamp, content type, content length,
  HTTP metadata, source SHA-256, extracted-text SHA-256, extraction tool,
  parser version, and schema version preserved per acquisition.
- Temporary work products excluded from Git.

Current recorded source hashes:

- LRITF PDF SHA-256:
  `d6f645e6a7d3c2eeb46905d4d24ecd8e078907057dc034cda715abf019ce8491`
- LRITF extracted-text SHA-256:
  `429a8916f3b1aa7035c0b700e27cd132a3af1662b1661ac703b9b0c7847b25a6`
- DCG CNBV PDF SHA-256:
  `493282f369e52da50db28c4777119591852a52313e5bb1cef82d1bd453899bb0`
- DCG extracted-text SHA-256:
  `ecbce994c6fe5aac9843addedc77f22db6dbdbb3a613f1873eb240c88fa751a6`
- DCG formal DOF publication SHA-256:
  `93c84d47e3e07a3e394fa56253efc3ce615eed497140d01336462c69788d8cef`
- DCG formal extracted-text SHA-256:
  `d28a869927b341c3a57fabd2075dff8a40c39fe50c37cc347282b00f4da311e8`

### Parsing and canonical corpus

- LRITF: 145 articles and 11 original transitories, with reform-decree
  transitories isolated for temporal analysis.
- DCG: 59 articles (including the `Artículo 17-` heading variant), 4
  transitories (PRIMERO–CUARTO), and 8 first-class annex provisions.
- DCG heading context preserves the 7 chapters plus Chapter II's three
  sections and three apartados through the extended
  title/chapter/section/apartado model.
- Article 1's two-column term/definition layout is reconstructed
  deterministically; all 26 defined terms survive with their definitions.
- Word-level fidelity checks against the extracted sources show no missing,
  duplicated, or reordered body text for articles, transitories, or annexes.
- Canonical JSON stored under `corpus/mx/lritf/` and
  `corpus/mx/ifpe-dcg-2021/`.
- Structural validation report generation per instrument.

### Express-reference graph

- Canonical `ReferenceEdge` records per instrument in `references.json`.
- Exact source spans and Unicode character offsets without modifying
  canonical provision text.
- Deterministic support for individual, compound, repeated, qualified, and
  ranged article citations.
- Cross-instrument resolution: a DCG citation naming the LRITF resolves
  against the loaded LRITF corpus. The audited LRITF graph keeps its original
  extraction policy and remains byte-identical.
- Title-anchored edges: the DCG title's statutory-basis citations of LRITF
  Articles 48, 54, and 56 are canonical edges validated against the official
  title, with paragraph qualifiers.
- `disposición ORDINAL Transitoria` citations: DCG transitory CUARTO resolves
  to LRITF's OCTAVA transitoria.
- Deterministic unlinked policy: short-form defined-term citations (`la Ley`)
  and named laws outside the corpus (for example, Código de Comercio) create
  no edges and no broken links.
- Current graph: LRITF 95 resolved internal edges; DCG 98 edges — 82
  internal, 13 provision-level LRITF citations, and 3 title-anchored LRITF
  citations — all resolved, none guessed.

### Temporal analysis v2

- Versioned `temporal-v2` prompt and strict JSON output schema.
- Authenticated Codex CLI model runner and provider-neutral response importer,
  now instrument-aware.
- Request and raw-response SHA-256 provenance per instrument.
- LRITF: 19 determinations, 32 structured effects, 18 machine-accepted, 1
  lawyer-verified (JRH's SÉPTIMA resolution, untouched), 0 pending reviews.
- DCG: 4 determinations covering the 90-natural-day commencement
  (2021-04-28), the six-month adaptation period for Article 15 (to
  2021-10-28), the nine-month adaptation period for Articles 16 and 17 (to
  2022-01-28), and CUARTO's authorization-triggered six-month cohort rule,
  classified `external_verification_required` because each obligation runs
  from an individual authorization. All 4 machine-accepted at confidence
  0.97–0.99; 0 pending reviews; nothing labeled lawyer-verified.
- The DCG is modeled as implementing statutory delegations with its own
  commencement and transitory effects; it does not amend the LRITF.

### Human review workflow

- Unchanged from the LRITF slice: confidence- and ambiguity-based routing,
  audited resolution requiring reviewer identity, lawyer overrides, history
  preserved across reruns.
- JRH's LRITF SÉPTIMA decision, its reviewer identity, dates, basis, note,
  and audit history are byte-for-byte unchanged by this work.
- The pending-review dashboard now aggregates review queues across all
  committed instruments.

### Markdown and Obsidian

- Standard Markdown and Obsidian notes for every provision of both
  instruments, including all eight DCG annexes.
- Cross-instrument links: DCG citations of LRITF render as
  `[[Corpus/LRITF/<note>|<span>]]` wikilinks and as relative links in
  standard Markdown; the DCG index title carries the three statutory-basis
  links.
- Generated index links now use full `Corpus/<instrument>/<note>` paths so
  identical note stems across instruments cannot collide.
- Export ownership remains limited to `Corpus/<instrument>/`; human-authored
  `Notas/`, `Revisiones/`, `Adjuntos/`, `Inicio.md`, and `.obsidian/` were
  verified byte-for-byte unchanged after publication.
- Obsidian CLI (application running) reports zero unresolved links across the
  235-file vault; a deterministic Rust-side check of all 652 generated
  wikilinks confirms every target file exists.

### CLI workflow

Implemented commands, each taking an instrument slug (`lritf` or
`ifpe-dcg-2021`):

```text
discover diputados|cnbv
fetch <slug>
extract <slug>
parse <slug>
link <slug>
analyze-temporal <slug> [--provider codex --model MODEL]
import-temporal <slug> <response> --model MODEL
validate <slug>
export <slug> --format json|markdown|obsidian
pipeline <slug>
review list [--all]
review resolve <review-id>
```

## 3. What has been tested

### Automated test coverage

The workspace currently contains 23 passing unit tests.

`lex-core` (7): quotation grounding, review routing, lawyer overrides, audit
preservation across reruns.

`lex-parse` (11):

- LRITF parsing, counts, reform-transitory isolation, and reference
  extraction (unchanged).
- DCG article/transitory/annex parsing with chapter/section/apartado context.
- The `Artículo 17-` heading variant.
- Article 1 definition-layout reconstruction, including a term wrapped over
  four lines, a definition crossing a page break, and the page-shifted UDI
  block.
- Page-break paragraph behavior in both directions: sentence-final breaks
  preserve the paragraph boundary; mid-sentence breaks merge.
- Transitory starts/boundaries and annex table-row extraction.
- Cross-instrument reference policy: full-name LRITF citations resolve,
  title citations carry qualifiers, the short-form `de la Ley,` citation
  creates no edge (regression for a defect found during implementation),
  transitory citations resolve, unconfigured external laws stay unlinked.
- Deterministic HTML extraction of blocks, table cells, and DOF entities.

`lex-export` (3): stable filenames, human-note boundary, link injection
without canonical-text changes.

`lex-source` (2): SHA-256 and source-format verification.

### Manual and integration testing completed

- Both DCG sources fetched through the pipeline; byte hashes matched the
  documented reconnaissance values exactly and the DOF fetch was verified
  byte-deterministic across repeated downloads.
- Word-level fidelity comparison of canonical text against the extracted
  CNBV PDF body and the DOF annex region: no missing, duplicated, or
  reordered content; the only differences are article heading tokens and
  structural heading subject lines.
- All 16 DCG→LRITF edges reviewed individually against their source
  sentences; all internal transitory citations (SEGUNDO→15, TERCERO→16/17,
  CUARTO→44–47) verified.
- A completeness scan of every numeric `artículo N` citation in the DCG found
  edges for all except the two Código de Comercio citations in Anexo 8 and
  the short-form `la Ley` citations, which the documented policy leaves
  unlinked.
- The complete network acquisition, extraction, parsing, temporal analysis
  (Codex gpt-5.5), validation, Markdown export, and Obsidian publication
  cycle was executed for the DCG.
- LRITF regression: `link lritf`, `validate lritf`, and both exports rerun;
  the committed LRITF corpus, its 95 references, and its audit history are
  byte-for-byte unchanged.
- Obsidian vault republished with both instruments; the live Obsidian CLI
  reported no unresolved links, and cross-instrument backlinks (for example,
  LRITF Article 48 ← DCG) navigate correctly.

## 4. What is tested: current verification results

Checks rerun successfully on 2026-07-03:

| Check | Result |
|---|---:|
| `cargo fmt --check` | Pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | Pass |
| `cargo test --workspace` | Pass: 23 tests |
| `cargo run -p lex-cli -- validate lritf` | Pass |
| `cargo run -p lex-cli -- validate ifpe-dcg-2021` | Pass |
| LRITF articles / transitories / references | 145 / 11 / 95 |
| DCG articles / transitories / annexes | 59 / 4 / 8 |
| DCG reference edges (internal / LRITF / title) | 82 / 13 / 3 |
| Unresolved or guessed references | 0 |
| Structural validation issues (both instruments) | 0 |
| LRITF temporal determinations / effects | 19 / 32 |
| DCG temporal determinations (machine-accepted) | 4 |
| Lawyer-verified determinations | 1 (LRITF SÉPTIMA, unchanged) |
| Pending legal reviews | 0 |
| Markdown files in the Obsidian vault | 235 |
| Obsidian unresolved links | 0 |

## 5. What is pending

### Immediate product gaps

- Relative references such as `artículo anterior` and `este artículo` are not
  canonical graph edges; the DCG contains several (for example, Articles 47
  and 57 citing `el artículo anterior`).
- Qualifiers written before the article number (`las fracciones II, III, IV y
  V del artículo 22`, `el séptimo párrafo del artículo 29`) resolve to the
  correct article but are not captured as qualifier metadata; only
  post-number qualifiers are.
- Defined terms and defined-term usage are not extracted or linked; DCG
  citations through the defined term `la Ley` therefore remain deliberately
  unlinked.
- Chapter/section/apartado subject lines are not stored in heading context
  (only the labels, matching the LRITF chapter model).
- Factual verification of the LRITF Article 71 coordination agreement remains
  outstanding, as does per-cohort verification of DCG CUARTO authorizations.

### Corpus and update-engine gaps

- The remaining MVP statutes have not been ingested.
- No Cámara or CNBV source-change monitor, candidate-version workflow, or
  provision-level diff engine exists yet.
- No automated DOF early-warning monitor or amendment reconciliation exists.
- The source manifest's `resulting_git_commit` field still needs pipeline
  integration.

### Test gaps

- No automated CLI integration tests exercise full command flows.
- Network acquisition and live model execution remain manually exercised.
- Nested DOF tables flatten their inner rows into the containing cell's line;
  annex content is complete but deep table nesting loses row boundaries.

## 6. Suggested next steps

1. **Relative article references** — resolve `artículo anterior`,
   `artículo siguiente`, and `este artículo` using provision order; the DCG
   provides immediate test material.
2. **Pre-number qualifier capture** — extend qualifier extraction to
   `fracción/párrafo ... del artículo N` forms on both instruments.
3. **Defined terms** — extract Article 1 definitions as structured terms and
   link exact usage, which would also resolve the DCG's `la Ley` citations.
4. **JRH review pass over the DCG temporal determinations** — the four
   machine-accepted determinations are schema-valid and source-grounded but
   not lawyer-verified.
5. **Expand the corpus** toward the remaining MVP statutes, then build the
   update engine (source monitoring, diffs, DOF early warning).
