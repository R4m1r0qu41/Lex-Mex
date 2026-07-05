# Lex-Mex Project Status

- **Status date:** 2026-07-03
- **Repository:** <https://github.com/R4m1r0qu41/Lex-Mex>
- **Current instruments:** Ley para Regular las Instituciones de Tecnología Financiera (LRITF); Disposiciones aplicables a las instituciones de fondos de pago electrónico (DCG-IFPE-2021)
- **Current legal reviewer:** JRH

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
  - `lex-parse`: PDF and DOF-HTML extraction, LRITF and DCG parsing (main
    document and independently sourced annexes), reform-transitory
    isolation, reference extraction, and structural validation.
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
- The DCG's eight annexes are acquired as CNBV's own dedicated per-annex
  PDFs, not extracted from the main document. CNBV publishes them from the
  instrument's row on the Normatividad page via a "Ver más" panel backed by
  `NormatividadAjax.svc/ResolucionesYAnexos?normaId=1036`; that endpoint
  returns each annex's URL and order (and would list any amending
  resolution, of which this instrument currently has none). Each annex PDF
  is fetched, hashed, and extracted the same way as the main document, with
  per-annex manifests in `annex-source-manifests.json`.
- Direct acquisition of the formal DOF publication for the DCG (código
  5610487) for promulgation-date provenance and cross-verification,
  recorded in `formal-source-manifest.json`; its text is not used for
  canonical content.
- Deterministic DOF HTML text extraction in Rust (block structure, table
  cells joined with ` | `, DOF character entities), retained for the formal
  source and available for future formal-source needs.
- Public intermediate CA certificates shipped in the CNBV adapter because
  both official hosts serve incomplete TLS chains; each chains to a standard
  root.
- Temporary PDF extraction through Poppler `pdftotext`; the DCG's main
  document and annexes keep form-feed page markers for deterministic
  paragraph merging.
- Source URL, publisher, retrieval timestamp, content type, content length,
  HTTP metadata, source SHA-256, extracted-text SHA-256, extraction tool,
  parser version, and schema version preserved per acquisition.
- Temporary work products excluded from Git.

Current recorded source hashes:

- LRITF PDF SHA-256:
  `d6f645e6a7d3c2eeb46905d4d24ecd8e078907057dc034cda715abf019ce8491`
- LRITF extracted-text SHA-256:
  `429a8916f3b1aa7035c0b700e27cd132a3af1662b1661ac703b9b0c7847b25a6`
- DCG CNBV main PDF SHA-256:
  `493282f369e52da50db28c4777119591852a52313e5bb1cef82d1bd453899bb0`
- DCG main extracted-text SHA-256:
  `ecbce994c6fe5aac9843addedc77f22db6dbdbb3a613f1873eb240c88fa751a6`
- DCG formal DOF publication SHA-256 (provenance only):
  `93c84d47e3e07a3e394fa56253efc3ce615eed497140d01336462c69788d8cef`
- DCG annex PDF SHA-256 values, recorded in order in
  `annex-source-manifests.json`: `668abe9a…`, `b741cb02…`, `3ea5c47d…`,
  `9baefef4…`, `7e4d63c6…`, `9d12f997…`, `3b23ad52…`, `e3192868…`
  (annexes 1–8 respectively).

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
- Each annex is parsed from its own dedicated CNBV PDF using the same
  paragraph and page-break rules as an article; its first line must be its
  own "ANEXO N" heading, cross-checked against the annex's position in the
  adapter's ordered URL list.
- Word-level fidelity checks against the extracted sources show zero missing
  or added words for articles, transitories, and all eight annexes — only
  each annex's own heading line is intentionally separated into its `label`
  field rather than kept in `text`.
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
- Pre-number qualifiers (`las fracciones II, III, IV y V del artículo 22`,
  `el séptimo párrafo del artículo 29`) are captured and attach to every
  article in the cited list; qualifiers carry validated character spans.
- Same-article fraction citations (`fracción N del presente artículo`)
  are edges targeting the containing provision, one per numeral, emitted
  only when the fraction exists as a paragraph.
- Fraction-precision previews: each fraction numeral in an anchored
  qualifier links to the target note's `^f-<n>` block, so hovering
  `XI` in `la fracción XI del artículo 36` previews only that fraction,
  while the article number keeps the whole-article hover; same-article
  numerals preview their own fraction in place.
- Current graph: LRITF 115 edges (95 article/transitory citations plus 20
  same-article fraction edges); DCG 111 edges — 82 internal article
  citations, 13 same-article fraction edges, 13 provision-level LRITF
  citations, and 3 title-anchored LRITF citations — all resolved, none
  guessed.

### Defined-term glossary layer

- Canonical `DefinedTerm` records (`terms.json`) extracted from each
  instrument's configured glossary provision: LRITF Article 4
  (fraction-style, 23 terms) and DCG Article 1 (colon-style, 26 terms),
  each anchored to the exact span of its definition entry including
  continuation paragraphs.
- Canonical `TermUsage` records (`term-usages.json`): every exact
  occurrence at word boundaries, longest match first, case-sensitive, with
  one deterministic singular/plural variant per term (`Operación` matches
  the defined `Operaciones`; `Comisión Supervisora` matches `Comisiones
  Supervisoras`). Currently 1,091 LRITF usages and 816 DCG usages.
- The DCG glossary is expressly additive to the LRITF's (configured
  `additive_to`), so DCG usages resolve against DCG Article 1 first and
  fall back to LRITF Article 4: `Cliente` (174 uses), `Operaciones`,
  `Infraestructura Tecnológica`, and `CNBV` in the DCG resolve to the
  statute's definitions.
- Positional capitals carry no signal: a term whose only capital is its
  initial letter does not match at sentence, list-item, or table-cell
  starts (`I. Controles de acceso…` is not the defined `Control`), while
  acronyms and multi-word terms match anywhere. A term never matches
  inside its own definition entry.
- Both artifacts are schema-bound (`defined-term.schema.json`,
  `term-usage.schema.json`) and validated: unique term IDs, existing
  defining provisions, definition spans containing the term, exact usage
  spans, cross-instrument resolvability, non-overlapping usages.
- Obsidian notes carry block anchors on every fraction paragraph (`^f-xi`)
  and definition entry (`^t-<slug>`); a term links to its definition's
  block so hovering shows only the definition. First usage per provision
  is linked (all usages remain canonical); term links never overlap
  reference links. 732 block-anchored links published, all targets
  verified.

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
  from an individual authorization. Initially all 4 were machine-accepted
  at confidence 0.97–0.99 with 0 pending reviews; JRH subsequently opened
  and resolved a review of CUARTO through the audited workflow, so that
  determination is now lawyer-verified: the authorization that starts its
  six-month clock is granted by the CNBV previo acuerdo del Comité
  Interinstitucional (SHCP, Banco de México, CNBV; LRITF art. 35),
  recorded in `responsible_authorities`.
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
- `review open` lets the designated reviewer open a review on an already
  machine-accepted determination; the determination and provision reflect
  the pending review immediately, and Markdown/Obsidian regenerate exactly
  as `review resolve` does.
- Reparsing re-applies the persisted temporal result rather than resetting
  it, so a default pipeline rerun never erases applied temporal state.
  Reapplication requires an exact hash match of the evidence text a
  determination was made against (`evidence_sha256`), not a supporting
  quotation merely surviving somewhere in changed text, and uses the same
  evidence construction as the analysis request so amendment-event
  determinations reapply correctly. A determination with no recorded hash
  has no verifiable provenance and is marked stale, never grandfathered in
  through a substring check.
- A pending or resolved review is never cleared by a model rerun regardless
  of the rerun's confidence, but restoration itself requires the previous
  determination's evidence hash to match the freshly routed current
  determination's hash; changed evidence means the old review no longer
  applies, so the fresh determination stands instead of silently
  reinstating a decision made about different text.

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
review [--instrument <slug>] list [--all]
review [--instrument <slug>] open <provision-id> --reason TEXT
review [--instrument <slug>] resolve <review-id>
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
- Transitory starts/boundaries and independent per-annex PDF parsing,
  including page-number-footer removal and a mismatched-heading rejection.
- Cross-instrument reference policy: full-name LRITF citations resolve,
  title citations carry qualifiers, the short-form `de la Ley,` citation
  creates no edge (regression for a defect found during implementation),
  transitory citations resolve, unconfigured external laws stay unlinked.
- Deterministic HTML extraction of blocks, table cells, and DOF entities
  (used for the formal source, retained for future formal-source needs).

`lex-export` (3): stable filenames, human-note boundary, link injection
without canonical-text changes.

`lex-source` (2): SHA-256 and source-format verification.

### Manual and integration testing completed

- All ten DCG sources (main PDF, formal DOF note, and all 8 annex PDFs)
  fetched through the pipeline; every byte hash matched independently
  downloaded copies exactly, confirming deterministic acquisition.
- The CNBV per-annex PDFs were discovered by inspecting the Normatividad
  page's "Ver más" panel and its backing
  `NormatividadAjax.svc/ResolucionesYAnexos` endpoint directly (JRH pointed
  out this mechanism); an initial implementation had instead sourced annex
  bodies from the formal DOF note. A word-level comparison confirmed the DOF
  note and the dedicated CNBV PDFs carry identical annex content, and the
  pipeline was switched to the CNBV PDFs as the correct operational source.
- Word-level fidelity comparison of canonical text against the extracted
  CNBV PDF body and against each of the eight dedicated annex PDFs: zero
  missing or added words; only each provision's own heading line is
  structurally separated into `label`.
- All 16 DCG→LRITF edges reviewed individually against their source
  sentences; all internal transitory citations (SEGUNDO→15, TERCERO→16/17,
  CUARTO→44–47) verified.
- A completeness scan of every numeric `artículo N` citation in the DCG found
  edges for all except the three Código de Comercio citations in Anexo 8 and
  the short-form `la Ley` citations, which the documented policy leaves
  unlinked.
- The complete network acquisition, extraction, parsing, temporal analysis
  (Codex gpt-5.5, run twice — once before and once after the annex-source
  correction, since reparsing resets provision-level temporal state and the
  transitories' evidence text was unaffected by the annex change), reference
  linking, validation, Markdown export, and Obsidian publication cycle was
  executed for the DCG.
- LRITF regression: `link lritf`, `validate lritf`, and both exports rerun
  after the annex-source correction; the committed LRITF corpus, its 95
  references, and its audit history are byte-for-byte unchanged.
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
| LRITF articles / transitories / references | 145 / 11 / 115 |
| DCG articles / transitories / annexes | 59 / 4 / 8 |
| DCG reference edges (internal / same-article / LRITF / title) | 82 / 13 / 13 / 3 |
| Unresolved or guessed references | 0 |
| Structural validation issues (both instruments) | 0 |
| LRITF temporal determinations / effects | 19 / 32 |
| DCG temporal determinations (machine-accepted) | 4 |
| Lawyer-verified determinations | 2 (LRITF SÉPTIMA, unchanged; DCG CUARTO, JRH) |
| Pending legal reviews | 0 |
| Markdown files in the Obsidian vault | 235 |
| Obsidian unresolved links | 0 |
| Defined terms (LRITF / DCG) | 23 / 26 |
| Term usages (LRITF / DCG) | 1,091 / 816 |
| Block-anchored links (terms + fractions), targets verified | 819 |

## 5. What is pending

### Immediate product gaps

- Relative references such as `artículo anterior` and `este artículo` are not
  canonical graph edges; the DCG contains several (for example, Articles 47
  and 57 citing `el artículo anterior`).
- Fraction ranges (`fracciones I a IV`) link only the listed numerals, not
  the intermediate ones; inciso- and apartado-level anchors do not exist
  yet (fractions only).
- Citation-style uses of `la Ley` as a bare shorthand remain unlinked: the
  DCG does not expressly define `Ley` in its glossary, so linking it would
  be inference rather than express definition.
- Term-usage matching is case-sensitive with one deterministic
  singular/plural variant per term; unusual inflections or mid-sentence
  lowercase uses of defined terms are not matched, and a term whose only
  capital is its initial letter never matches at sentence/item starts (a
  conservative rule that can also skip rare genuine sentence-initial
  usages).
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
- Annex text uses the same prose-paragraph normalization as articles, not a
  table-cell reconstruction. Anexo 1's dense multi-column risk-indicator
  matrix therefore renders as long, column-interleaved paragraphs rather
  than a readable grid; no content is lost (word-level fidelity is exact),
  only that table's visual row/column structure.

## 6. Suggested next steps

1. **Relative article references** — resolve `artículo anterior`,
   `artículo siguiente`, and `este artículo` using provision order; the DCG
   provides immediate test material.
2. **Pre-number qualifier coverage for noun-first forms** — `párrafos
   segundo y tercero del artículo N` (noun before ordinal) is not yet
   captured.
3. **JRH review pass over the DCG temporal determinations** — three of the
   four machine-accepted determinations remain unverified (CUARTO is
   lawyer-verified).
4. **Expand the corpus** toward the remaining MVP statutes — the general
   Fintech DCG (10/09/2018) next, using its compiled document as the
   consolidated operational source with its numbered resoluciones
   modificatorias as amendment provenance — then build the update engine
   (source monitoring, diffs, DOF early warning).
