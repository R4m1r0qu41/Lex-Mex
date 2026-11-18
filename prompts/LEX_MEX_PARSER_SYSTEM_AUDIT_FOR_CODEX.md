# Lex-Mex — Parser and System-Context Architecture Audit

**Assignment:** Independent, evidence-based review of the existing Lex-Mex ingestion/parser architecture, its canonical data model, and what downstream systems actually consume.

**Mode:** Investigation and recommendations only. **Do not implement a refactor.** Do not alter canonical corpus data, adapters, fixtures, schemas, reviewer decisions, or production code. You may produce a Markdown report in `docs/reviews/` (or print the report if writing is not authorized). Do not re-ingest or adopt a corpus-wide reparse. Avoid network access and destructive operations.

**Primary question:** Does Lex-Mex turn official Mexican legal documents into a useful *machine-readable legal structure*, or does it principally retain faithful article-level text wrapped in JSON? Precisely where does structural understanding begin and end—at the parser, in the linker, in the database/indexes, in Obsidian export, or in downstream agent workflows?

**Important:** This is **not** a request to replace a fidelity-first parser with model-generated legal interpretations. Do not assume the system lacks fine-grained structure because the overview does not mention it. Verify the actual code, schemas, serialized artifacts, consumers, and tests. Equally, do not treat “it outputs JSON” as proof of a fully structured legal syntax tree.

## 0. Context and verification rules

1. First read repository-level instructions (`AGENTS.md`, `CODEX.md` if present) and applicable nested instructions. Respect repository constraints.
2. Capture the actual branch, `HEAD` SHA, working-tree status, and runtime/tool versions relevant to claims. The supplied overview describes branch `upgrade/four-goal-2026-11-12` at commit `6bf0f9615`, but the live checkout may differ. **Audit the checkout actually present** and explicitly identify divergence from that historical baseline.
3. Read the attached/background document `parser-overview-for-independent-review-2026-11-13.md` (if it exists in the repo or is provided in your context). Treat it as a **claim inventory**, not the authoritative implementation. If inaccessible, use the context below and report the gap.
4. Review, where available: `docs/parser-rule-inventory.md`, `docs/held-source-parser-contracts.md`, `docs/ingestion-difficulty-log.md`, `docs/decisions.md`, and relevant source integrity/validation guidance.
5. Every major finding needs **actual file path + line ranges or symbol names**, the concrete data path, and where possible an actual serialized example or test. Distinguish **Verified**, **Inferred**, **Documented only**, **Unverified**, and **Contradicted**. Do not manufacture samples, paths, test outcomes, schemas, or repository behavior.
6. Prioritize static inspection, targeted existing fixtures, and non-destructive commands. Run targeted tests only if safe and useful. If commands fail or cannot run, say why. Do not create new fixtures or silently 'fix' a defect during this audit.
7. Do not conflate **syntax/structure**, **citations and reference resolution**, **temporal amendment modeling**, and **substantive legal interpretation**. Assess them separately.
8. Inspect the whole relevant *system data flow*, not just `lex-parse`. Follow Rust types into JSON, storage/indexes, APIs/CLI, Obsidian/Markdown, and agent-facing retrieval **only where those consumers exist**.

### Working mental model to test (not to assume)

```text
Official source (PDF / DOC / HTML)
  -> retained source + extraction + text/provenance
  -> normalization + source-family parser
  -> canonical provision records (e.g. provisions.json)
  -> [possible subsection/structural representation?]
  -> derived temporal evidence / citation & term linking / validation
  -> database and index / exports / CLI or API
  -> human and agent retrieval, reasoning, and graph workflows
```

The parser overview claims:

- Pure Rust parsing, no language model in parsing, fail-closed handling, provenance and Unicode-character spans.
- Article/transitory/annex records persisted in `provisions.json` with IDs, labels, heading context, exact text, and initial temporal fields.
- Separate reform-temporal evidence, glossary extraction, citations/term usages, and later temporal derivation/export.
- `parse_diputados_publication` in `crates/lex-parse/src/diputados.rs` operates on normalized logical blocks and maintains current provision, heading, and transitory/annex/reform state.
- Some fraction boundaries are recognized during normalization, but the overview does **not establish** whether fracciones/incisos/paragraphs persist as independently typed children, offset ranges, anchors, or only prose.
- `verify-fidelity` checks committed provision text against official extracted source in token order; that may not, on its own, prove exhaustive coverage.
- Known issues in the 2026-11-13 overview include `lgipe` suppressed-article boundaries, `lcf` / `lacp` fraction recognition/anchors, an orphan CNBV amendment-marker path, and `lieps` reform evidence/dating under a held reparse. Check whether they still apply.

## 1. Trace the actual implementation end to end

Build an evidence-backed call/data-flow map beginning with `run_parse` (reported in `crates/lex-cli/src/main.rs`) and ending with the concrete consumer of parsed legal data. Identify:

- Entry points and commands: fetch, extract, parse, derive-temporal, link, validate, export, persistence, query, and any agent tools or protocols.
- Source-family dispatch: `diputados`, `itf-dcg`, `ifpe-dcg`, standards (`NOM` / `NMX`), plus additional families if present.
- Crate/module responsibilities: `lex-parse`, `lex-grammar`, `lex-analysis`, `lex-link`, `lex-core` and any actual storage/API/export crates.
- Structures instantiated at every stage; transformations and loss of information; data ownership and mutation boundaries.
- Which files represent immutable evidence, canonical records, derived indices, cached projections, and presentation formats.
- Whether there are **two or more structurally independent parsing/segmentation implementations** (e.g., core parser versus Obsidian anchors versus retrieval layer). Note rule drift risk.

**Deliverable A:** One Mermaid data-flow diagram with real named types/modules/artifacts, and a table `stage | input | output | authority | source location | invariant/test`.

## 2. Identify the real machine-readable representation

Inspect the *actual* Rust definitions, serialization attributes, schemas, and output artifacts. Answer:

1. What precisely is the schema of `provisions.json`? Show **one real** article record (abbreviate long body content clearly without suggesting abbreviated text is the actual stored value).
2. What exactly is an `article` record vs. a `transitory`, `annex`, `heading`, `paragraph`, `fraction`, `inciso`, `numeral`, `subinciso`, `table`, `note`, or reform evidence entry?
3. Do article components have **typed child nodes**, explicit parent IDs, sibling order, stable hierarchical paths, typed spans, or only textual delimiters and ad hoc anchors?
4. If `Artículo 15, fracción II, inciso b)` is queried, what does the system do **today**? Show the actual call path and output: direct indexed retrieval, offset extraction, regex-on-demand, Markdown anchor navigation, semantic search, or a language-model reading the full article.
5. Is heading context (`libro`, `título`, `capítulo`, `sección`, `apartado`) a first-class structured model? Are headings independently represented, just context strings, or discarded from provision text?
6. What is the semantic meaning and stability guarantee of node/provision IDs? How are suffixes, ordinals, `Bis`, `Ter`, `Ñ`, digraphs, duplicate article numbers across scopes, and law/decree contexts handled?
7. How is substantive source text **reconstructed** from a structured output? Is lossless reconstruction possible? If not, identify exactly what is retained, normalized, dropped, or kept only in the source artifact.
8. Is there any explicit **legal rule / obligation / condition / exception / subject / action** representation? If yes, trace its provenance and review status. If not, say so—this is not a default failure of structural parsing.

**Deliverable B:** A *verified* machine-readability inventory, not a binary verdict:

| Layer | Current representation | Structured fields | Retained as prose | Directly queryable? | Evidence |
|---|---|---|---|---|---|
| Instrument | ... | ... | ... | ... | ... |
| Article / transitory | ... | ... | ... | ... | ... |
| Paragraph / fraction / inciso | ... | ... | ... | ... | ... |
| Citation / defined term | ... | ... | ... | ... | ... |
| Temporal event | ... | ... | ... | ... | ... |
| Legal-rule interpretation | ... | ... | ... | ... | ... |

**Do not call a field “structured” merely because raw text is stored in JSON.** Equally, an exact text span with a typed parent relationship *is* meaningful structure even without an `AST` class.

## 3. Inspect parser mechanics, state and structural boundaries

Walk through `normalized_blocks`, article-label recognition, `parse_diputados_publication`, enactment partitioning, glossary extraction, CNBV parser families, standards, and any subsection handling. Answer:

- What lexer/tokenizer, regex, scanner, state machine, stack, grammar, or parser-combinator approaches are used? Is there actually an Abstract Syntax Tree (AST), or a simpler flat provision collection with context state?
- Which transitions open/close articles, transitories, nested statutes, annexes, reform appendices, chapters, fracciones, incisos, and unnumbered paragraphs?
- When does the parser merely detect a fraction boundary to join/reflow lines versus preserve a first-class fraction boundary in output?
- Can it distinguish headings from citations in prose, quotations of other decrees, separatorless headings, page artifacts, signatures, elided decree apparatus, annexes, and `Artículo [n] Bis`?
- How does it resolve ambiguous parenthood: after `I.`, `a)`, `b)`, `II.`, then a non-enumerated paragraph—whose child is that paragraph? Does it resolve, leave unspecified, or guess?
- Where do adapter-specific exceptions live? Are they source-/text-hash-bound and tested? Are family-specific parsers warranted for the four older DCGs borrowing `itf-dcg`?
- What triggers fail-closed behavior, a hold, a warning, an accepted gap, or silent fallback? Enumerate any **silent-loss paths** and whether they violate stated invariants.

**Deliverable C:** A state-transition summary and a focused gap table with exact code references and reproduction case(s).

## 4. Fidelity, provenance and completeness are different properties

Do **not** collapse these into one “valid” flag. Determine independently:

- **Text fidelity:** What does `verify-fidelity` precisely verify? Token equality? Word order? Unicode normalization? Whitespace? Inclusion of output in source? Does it validate *every* source segment's representation or only the converse?
- **Coverage:** Is there a source-to-canonical segmentation/coverage ledger that accounts for every relevant character/token/block as included, intentionally excluded, or unresolved? If not, how are omissions detected?
- **Provenance:** Where are official URL, source/extracted hashes, extractor version, parser/schema versions, retrieval time, page/position data, and transformations stored? Can an emitted fraction/inciso be traced to a source page and span?
- **Normalization:** Exactly which content is dropped as page furniture, which whitespace is reflowed, which text is retained verbatim? Is 'unchanged provision text' defined relative to extracted text, normalized text, or source file bytes?
- **Offsets:** Verify Unicode **character** indexing rather than Rust UTF-8 byte indexing at each public boundary. Check agreement across linker, glossary, re-confirmation mapping, database, Markdown anchors, and exported representations.
- **Stability:** When whitespace-only reparses alter hashes or offsets, what remains stable? Does `reconfirms` preserve reviewer approvals without masking a substantive change?
- **Fail-closed integrity:** Check known CNBV orphan marker behavior and any other paths where material can disappear without failure.

**Deliverable D:** An invariant matrix: `invariant | claimed by docs | enforced in code | test | counterexample/gap | consequence`.

## 5. Trace downstream use, not merely parser output

For **each implemented consumer**, answer what it reads and what it knows without performing new text interpretation:

- Persistent database/index architecture: actual technology (do not assume SQLite/PostgreSQL/DuckDB), schemas, FTS or other indexes, relations, referential integrity, versioning, and retrieval granularity.
- CLI or API query surfaces: whether users and agents can address an article, fracción, inciso, historical version, transitory, defined term, or cross-reference as a deterministic identifier.
- Obsidian/Markdown export and anchors: where anchors come from; whether they are canonical parser artifacts or rederived presentation conveniences; known `VI.-` issue; behavior on source reflows.
- Citation/defined-term linking: what is linked to what, with what spans; when references are unresolved, cross-instrument, ambiguous, stale, or time-dependent.
- Temporal model: differentiate statute text, consolidated text, reform decree evidence, initial status, amended/effective versions, and reviewer-approved facts. Are historical queries genuinely version-aware?
- Agents and applications: what data and tools are actually supplied to an agent; whether a language model must reparse prose to reach sub-article targets; whether retrieval outputs source citations and stable IDs.
- Legal graphs: identify **existing** graph primitives/edges versus desired future constructs for operational compliance, litigation procedure, and legal reasoning. Do not infer those features from ordinary citation links.

**Deliverable E:** A consumer matrix: `consumer | source artifact | granularity | deterministic lookup? | semantic inference needed? | provenance carried? | code evidence`.

## 6. Work through representative real cases

Use a small, high-information sample of **existing** fixtures and actual retained sources. Prefer (when available):

1. **A clean, nested statute provision** with multiple fracciones and incisos (show the original text, persisted JSON, and a real query/export for one inciso).
2. **A numbering edge case:** `LIVA` ordinal suffix, `LGTOC` lowercase suffix, `LFT` `353-N`/`353-Ñ`, or another live supported equivalent.
3. **A boundary defect:** `lgipe` suppressed article or `lcf` / `lacp` fraction reflow; check whether still open.
4. **A decree-versus-statute/temporal case:** a reform appendix, nested enactment, or `lieps` held reparse. Explain risk of confusing statute transitories with amending-decree transitories.
5. **A CNBV compilation example:** amendment margin mark with legend resolution and an orphan/missing-legend case, if available.
6. **A standards example** (NOM/NMX) if the output schema differs materially from statutes.

For each, show a miniature trace:

`official extracted segment -> parser state/rule -> serialized record -> validation -> downstream lookup/export`.

Avoid copying large protected or sensitive text into the report; use limited excerpts and exact local file references. If a sample is inaccessible, report that and use the next available fixture. **Do not construct fictional serialized records and label them actual.**

## 7. Evaluate architecture options without presupposing a refactor

Consider three options:

**A. Keep existing article-level canonical records, no extra structural layer.**

**B. Keep source-fidelity canonical records and add a deterministic derived structural projection** containing typed sub-article nodes (paragraph/fracción/inciso/etc.), parent IDs, sequence, canonical source spans, and validation. No duplicated authoritative wording is required; span-based nodes may suffice.

**C. Enrich or replace the canonical schema with a first-class full legal AST**, migrating consumers and reviewer bindings.

Compare options on:

- Legal fidelity and preservation of official wording;
- Machine-query capability and deterministic agent access;
- Compatibility with current canonical IDs, offsets, hashes, APIs, database, Obsidian export, tests and review decisions;
- Handling ambiguities and jurisdiction-/publisher-specific drafting conventions;
- Temporal versioning and reform provenance;
- Performance and complexity at ~current corpus scale (verify actual counts, no assumed benchmarks);
- Re-ingestion/migration risk, regression burden and long-term maintenance.

**Deliverable F:** A decision table, with a recommended option **only after actual inspection**. Prefer the *smallest additive change with measurable benefit* if a gap is confirmed. Explain why not to change the existing architecture if the functionality is already present or demand is not established.

For any proposed derived layer, specify a minimal **candidate** data contract (not a claim about the current schema). Include:

- `instrument_id`, `provision_id`, `node_kind`, `node_id`, `parent_id`, `order`, `label`, `source_span_start`, `source_span_end`, `canonical_text_hash`/version or equivalent;
- Exact source-text offset semantics (Unicode scalar values vs grapheme clusters vs bytes; follow existing contract);
- Whether implicit paragraphs and ambiguous children are expressible;
- Handling of headings and of decree versus consolidated-statute scope;
- How to invalidate/rebuild derived nodes without mutating canonical source or bypassing reviewer guards;
- Provenance and lossless recoverability guarantees;
- The minimum tests and invariants for adopting it.

Do **not** design substantive obligations/exceptions as if they were parser grammar. Legal classifications belong in a distinct, reviewable layer.

## 8. Tests, regressions and production readiness

Review existing fixtures/tests and run focused checks if safe. Evaluate whether test coverage is concentrated on known historical bugs rather than general invariants. In particular:

- Regression tests for block reflow, numeral parsing, structural boundaries, transitories, nested enactments, annexes, CNBV margins, defined terms, and reviewer re-confirmations.
- Property/invariant tests for non-overlapping spans (as appropriate), stable ordering, unique IDs, source coverage, round-trip text recovery, deterministic reruns, and preservation of unresolved text.
- Tests proving that valid records are *complete*, not merely source-backed.
- Round-trip or golden tests asserting what agent-facing/Obsidian exports actually expose, not just parser output.
- The current status of the problems documented for `lgipe`, `lcf`, `lacp`, `lieps`, and CNBV missing/orphaned marks.
- Whether a scratch reparse/diff is operationally mandatory before adoption, and which reviewers must approve legal or temporal decisions.

Relevant commands **from the historical overview**, to verify against the current CLI before use (do not blindly invoke at scale):

```bash
cargo test -p lex-parse -p lex-grammar -p lex-analysis
cargo run --locked -p lex-cli -- parse <slug>
cargo run --locked -p lex-cli -- validate <slug>
cargo run --locked -p lex-cli -- verify-fidelity --all
```

**Warning:** `parse <slug>` may write canonical artifacts. Do **not** run it against the working corpus; only consider an isolated scratch root with documented safe arguments, or inspect existing output. Full-corpus verification may be expensive; report what was actually run.

**Deliverable G:** `test/invariant | current evidence | status | residual risk | recommended test`, plus exact commands *actually executed* and their outcomes.

## 9. Required audit report format

Return a self-contained Markdown report titled:

`Lex-Mex Parser: Fidelity, Structure, and System Integration — Independent Audit`

Use these sections in order:

1. **Executive verdict (max ~300 words).** Answer the core question plainly: what is structured, what is preserved as text, and what downstream components add. Separate verified facts from hypotheses.
2. **Scope and evidence.** Repo state, reviewed files, tested commands, unavailable artifacts, confidence limits.
3. **Actual end-to-end architecture.** Mermaid flow and stage/invariant table (Deliverable A).
4. **Data-model findings.** Real record/schema sample and machine-readability matrix (Deliverable B).
5. **Structural parser mechanics.** State transitions, ambiguity, missing nodes, silent-loss cases (Deliverable C).
6. **Source integrity review.** Fidelity vs completeness, provenance, offsets and review bindings (Deliverable D).
7. **Downstream consumers.** Database, index, CLI/API, Obsidian, link, temporal and agent paths (Deliverable E).
8. **Case studies.** At least three traced real fixtures, and ideally all six representative categories.
9. **Architecture options and recommendation.** A/B/C comparison, minimal suggested schema only if justified (Deliverable F).
10. **Validation/test findings.** What was checked and what remains unproven (Deliverable G).
11. **Prioritized findings.** `P0` (silent loss/wrong-law/unsafe assertion), `P1` (structural correctness or data contract), `P2` (ergonomics/performance), or `No action`. For each: impact, evidence, reproducibility, blast radius, proposed bounded remedy, approval owner, and regression test.
12. **Open questions for JRH only.** Ask only actual *legal classification, substantive law, or risk-acceptance* questions that cannot be resolved from code and retained official source. No delegation of ordinary debugging or source-format decisions.
13. **Next implementation slice (proposal only).** One small, reversible change with acceptance criteria; or say “none warranted” if the current system already meets the requirement. **Do not implement it.**

Finish with two concise answers, explicitly labeled:

- **“Can Lex-Mex retrieve a particular fracción/inciso today without reparsing article prose?”** Answer `Yes / Partly / No / Unverified`, with a real example and evidence.
- **“Would a derived typed subsection tree materially improve the system?”** Answer `Yes / Maybe / No / Unverified`, explaining concrete benefit and tradeoff.

## 10. Stop conditions and guardrails

- **No code changes, migrations, canonical writes, fixture rewrites, or changes to reviewer approvals.** Producing this review report is the only intended new artifact.
- **Do not assume a database technology, an agent protocol, an AST, or a graph engine exists.** Discover and cite implementations.
- **Do not conflate formatting a paragraph for Obsidian with semantically structuring it.** Conversely, recognize an offset-backed typed span as structure when present.
- **Do not treat unsupported factual, legal, or temporal interpretation as an automatic parsing objective.** Structural parsing must remain deterministic and source-traceable.
- **Do not recommend an expensive rewrite on architectural aesthetics alone.** Ground each recommendation in actual user-facing or agent-facing failure modes.
- **If the checkout/attachments are missing, stop short of an implementation verdict and return a well-defined list of missing evidence; do not invent a review.**
