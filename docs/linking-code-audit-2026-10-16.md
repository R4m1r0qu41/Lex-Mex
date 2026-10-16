# Linking code audit for Claude — 2026-10-16

Baseline: `main` at `667e33a2c99d7a3c4a7e927b2efd8c82222f8669`, initially
clean. This is a fresh audit after the six repairs committed in `078359f72`.
Those repairs are not being reported again as outstanding. No source code,
canonical data, held-instrument decisions, staging or commits were changed.

Independent Sol reviews reproduced **seven additional findings**. Three are
confirmed against stored corpus data or its actual rendered output; four are
synthetic boundary cases. Parser tests (206) and exporter tests (10) all pass,
so the existing suite does not cover these cases.

## Findings, in recommended order

### L1 — Paragraph numbers become article links (P2; corpus-confirmed)

`crates/lex-parse/src/lib.rs:835` (`accepted_numbers`) accepts a numeric
paragraph qualifier as another article number. In **LGPP Article 95**, the
source reads `artículo 10, párrafo 2, inciso c), de esta Ley`. Stored edges
include both Article 10 (2383..2385) and an erroneous Article 2 (2395..2396);
`inciso c` is attached to that erroneous second edge. The correct target scope
is Article 10, paragraph 2, inciso c. Source text must remain unchanged.

Fix direction: parse qualifier-number spans separately from article-list
numbers; retain exact offsets and compound scope. Test `2o. párrafo`, `párrafo
2`, ordinary word ordinals, compound qualifiers and genuine article lists.
Thirteen matching source shapes were found by a bounded metadata probe; only
this LGPP example was verified against its stored edges here.

### L2 — Duplicate fraction anchors are ambiguous (P2; actual export)

`crates/lex-export/src/lib.rs:52`, `:529`, `:614` represent fractions as a set
of Roman labels and emit anchors without section identity. **CPEUM Article
20** emits three `^f-i` anchors under different apartados. **LFTR Article 180
Bis** expressly cites Article 20, Apartado B, fraction I, but emits
`[[Corpus/CPEUM/articulo-20#^f-i|I]]`. That destination cannot uniquely identify
B.I. This audit does not assume which duplicate Obsidian selects at runtime.

Fix direction: use a shared registry of unique, actually emitted anchors. Until
section-qualified destinations can be represented and validated, fall back to
the whole article rather than guessing a repeated fraction. A section-aware
model would require coordinated extraction, type/schema, validation and export
changes; merely adding occurrence numbers is insufficient.

### L3 — Definition links target nonexistent anchors (P2; actual export)

`crates/lex-export/src/lib.rs:84`, `:114`, `:529` derive fraction anchors for
definitions even when rendering never emits them. **LFCSQ Article 2** emits
links for **Desvío** to `#^f-xii` and **Polígono de Inspección** to `#^f-xxvi`,
but the target note has zero matching anchors. Their definitions begin inline
rather than at paragraph starts. A metadata inventory identified 142 candidate
fraction-term targets with usages and no paragraph-leading anchor; only the
named LFCSQ cases were rendered and independently confirmed.

Fix direction: derive term destinations from the same actual-anchor registry.
Either emit unique definition anchors at validated rendering boundaries or use
a whole-note fallback. Preserve canonical definition text and offsets.

### L4 — A later semicolon clause retargets a direct citation (P2; synthetic)

`crates/lex-parse/src/lib.rs:1489`, `:1520`, `:603` delimit instrument context
at periods/newlines but not an independent semicolon clause. Fixture:
`Se aplicará el artículo 1; la Constitución Política de los Estados Unidos
Mexicanos reconoce derechos.` It resolves to CPEUM Article 1 because the later
alias is within the lookahead. Replacing the semicolon with a period keeps the
local article. Do not infer the governing instrument from that later statement.

Fix direction: constrain governing context by grammatical attachment/clause
boundaries while preserving semicolon-separated article lists. Claude's landed
relative-citation guard handles later-clause context; this finding concerns the
separate direct-citation path.

### L5 — Configured aliases match inside longer words (P2; synthetic)

`crates/lex-parse/src/lib.rs:736`, `:744` use substring search for configured
aliases. A fixture naming `Ley del Seguro Socialista` falsely resolves to LSS
because it contains `ley del seguro social`. The invented title is only a
lexical-boundary test, not a claim that such an instrument exists.

Fix direction: require lexical boundaries, preserve exact-title punctuation,
and retain longest configured-title precedence. Unrecognized titles must not
become links to a different law through a partial-word match.

### L6 — Display acronym controls Markdown target identity (P2; synthetic)

`crates/lex-export/src/lib.rs:722` uses equality of `short_name` to choose a
local path. In a private fixture changing only CPEUM's display acronym to LFTR,
a genuine LFTR-to-CPEUM Article 20 edge renders `[20](articulo-20.md)` instead
of `[20](../../cpeum/markdown/articulo-20.md)`. The wrong local LFTR article
exists. All 467 current short names are unique; no live collision is claimed.

Fix direction: compare canonical instrument identity, using the target slug
only for path construction. Display labels must not decide identity.

### L7 — Parenthesized compound qualifiers disappear (P3; synthetic)

`crates/lex-parse/src/lib.rs:169`, `:178`, `:1402` fail to retain scope in
`la fracción II, inciso a) del artículo 1 de esta Ley`. The article edge is
present but has no qualifiers. The simpler word-form fraction control works.

Fix direction: retain standard closing-parenthesis syntax and compound
fraction/inciso scope, with exact Unicode spans. Do not change the provision
text or invent finer target anchors from unsupported scope.

## Review of Claude's integration handoff

Reviewed the incoming request
`2026-10-16-lex-mex-relative-citation-regression-review.md` in the Agent Vault
pending Codex queue. The landed relative rule cuts later clauses and requires
attached genitive context; `run_corpus_closure` checks persisted repeal evidence
before proposing any graph. No new concrete regression in those changes or in
the six earlier fixes was demonstrated in this bounded review. This is not a
rerun of the full corpus closure or a resolution of the 21 held instruments.

## Verification and limits

- `cargo test --locked -p lex-parse --lib`: 206 passed, zero failed.
- `cargo test --locked -p lex-export --lib`: 10 passed, zero failed.
- Thirteen private extraction cases with positive controls; actual private
  Obsidian exports of CPEUM, LFTR and LFCSQ; one synthetic Markdown export.
- Actual corpus examples above were checked against exact stored records and
  rendered destinations. Metadata-only candidates are explicitly qualified.
- No full native gate or corpus-wide relink was run in this audit. No code fix
  was applied. Unsupported ranges/Unicode forms beyond the bounded probes have
  not been exhaustively tested.

Evidence: `docs/linking-code-audit-2026-10-16-evidence.json`.
Claude entry point: `docs/linking-code-claude-handoff-2026-10-16.md`.
