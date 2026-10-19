# Glossary G0–G2, outliers and title-prefix usage review

## Result and baseline

Reviewed main `dbc3ab17d`, from the previous `adadd9c9d` review, including the
incoming glossary-repairs/quoted-usages handoff. G0–G1 are implemented; **G2 is
partly complete, not closed**. Imports (G3) remain unconfigured. The 94 admissions
match their stated totals: +1,619 terms, +28,245 usages; no previously nonempty
terms file changed and no reference graph changed in the admission commit.

Five G2 originals now have definitions: LSH 5 (52), LGEEPA 3 (41), LSE 3 (66),
LPTE 3 (28), LGEO 2 (17). LSS still has six terms from article 159, none from
5 A. Choosing their governing scope is an operator decision; do not replace one
with the other just because it is earlier in the document.

REG-LAN retains four terms and now 706 quoted usages under the recorded operator
ruling. Preserve that opt-in: do not restore its two bare usages indirectly via
a new global matcher. The prior respectively-list guard fixes and newline/multiple
sentence regressions are present. The split plan's source-recovery gate is now
qualified correctly. This review found no reason to undo those repairs.

## Findings to fix before admitting the outliers

**P1 — A repeal prefix can bypass the completeness check.** In
`complete_entries`, `is_repeal_placeholder_body` runs before the swallowed-heading
check. The current public detector accepts only `Agencia`, with no hold, for:

```text
Para los efectos de esta Ley se entiende por:
I. Agencia: La agencia;
II. Derogada. ENCABEZADO DE PAGINA III. Secretaría: La secretaría.
```

The last fraction is hidden by furniture and discarded with the repealed entry.
Separately, `I. Derogados: Los permisos retirados; II. Vigentes: Los permisos
activos.` drops the genuine first definition because the test is `starts_with`.
These are reproduced synthetic failures, not claims of a found stored omission.
Require a whole-entry repeal-placeholder grammar (optional supported amendment
note only), inspect residual content and swallowed markers before skipping, and
hold ambiguous bodies. Add both regression fixtures. Do not let this function
make a legal finding that a substantive definition is repealed.

**P2 — Qualified fraction order/uniqueness is not checked.** The wider detector
accepts `I, I Bis, I Bis, II` if the two Bis entries have different term names.
It also lacks a Bis-before-Ter ordering test. Track the full parsed fraction key,
not only the base numeral; reject duplicates/backward order. Equivalent Roman
spellings should not be accepted by permissive arithmetic alone. Preserve the
printed label separately from its parsing key.

**P2 — Held coverage exists only in stderr.** `extract_instrument_terms` prints
`glossary held`, but the persisted empty terms list does not distinguish a held
glossary from an instrument with none. The DB must not infer either meaning from
`[]`. Persist a versioned deterministic extraction report with source hashes,
candidate provision, strategy, entry coverage and reason codes; keep it separate
from legal approval. This is also how small exceptions become auditable.

The sequence check does not prove that every period-start candidate is valid;
LFCA's `A. C.` demonstrates the false-start class. It is right to hold that case,
not to weaken the check globally. Existing-pass precedence protects this release's
old term sets, as verified by the diff; it does not prove that those old glossaries
are complete or that their scope is correct. Audit them without silently replacing
them.

## The 19 holds: inspected text and bounded remedies

Every hold was reproduced by calling the current Rust detector. Hashes and exact
reasons are in the companion evidence JSON. Labels below describe committed
extracted text; a glyph discrepancy is not declared an official-source typo
without checking the retained source.

| Instrument / article | Actual blocker | Bounded treatment |
|---|---|---|
| `reg-lgcc-rne` 2 | Running title hides VI; VII looks like a gap | Exact source-bound VI boundary hint; retain furniture debt; source parser cleanup separately. |
| `reg-lgeepa-maaa` 2 | Running title hides III | Same boundary/furniture treatment. |
| `reg-lgeepa-meia` 3 | Repeated headers disrupt fractions; hold reports XVI after XIV | Inspect all entry boundaries, including Bis/Ter; exact hints, not a single guessed gap. |
| `reg-lgeepa-mpcca` 6 | Running title hides III | Exact boundary hint; keep added/reform notes distinct. |
| `reg-lgeepa-mretc` 3 | Running title hides XI | Exact boundary hint; “otros ordenamientos” import remains a separate legal scope question. |
| `reg-lgs-investigacion` 40 | Running title hides IX | Exact boundary hint. Intro expressly says this Reglamento; do not infer narrower scope solely from article position. |
| `reg-lss-maeba` 2 | Running title hides IV | Exact boundary hint; importing LSS 5 A still requires resolved scope. |
| `log` 4 | `Página 1 de 9` before I prevents its detection | Exact leading furniture/boundary annotation. Do not delete canonical text during linking. |
| `fi-dcg-2014` 1 | `ll.` and `III.Certificados` (no space), not simply a source gap | Exact label/boundary hints for both shapes, verified against source; preserve literal text. |
| `servinv-dcg-2013` 1 | `III Asesores en inversiones` lacks the period | Explicit III boundary; do not invent a missing definition. Preserve its nested continuation under II. |
| `reg-lftr-mcsre` 3 | Visible numbering jumps VII→IX | Record the exact observed sequence only after source verification; never synthesize VIII. Existing historical/import qualifications remain. |
| `ladua` 2 | Both `lI.` and `lll.` occur | Source-bound interpretation of the two labels; no global lowercase-L→I substitution. |
| `lfca` 2 | `C.` inside `Centro … A. C. y Estudios …` mistaken for fraction 100 | Ignore that exact candidate via abbreviation rule/annotated boundary. The description “S. A. de C. V.” is not the inspected offending substring. |
| `lfpccs` 3 | XXVII says `SNICS. El Servicio…` in a colon glossary | Per-entry period delimiter; this entry does have a delimiter. |
| `lfra` 2 | V says `Se entiende por cadena causal…`; VI is a substantive exception, not a simple named entry | Explicit span for `cadena causal`; classify VI as non-definition/continuation with reviewed rationale. Do not force one term from every live fraction. |
| `lfvv` 2 | VII says `Secretaría; La Secretaría…` | Per-entry semicolon delimiter. No canonical punctuation edit. |
| `ltfccg` 3 | IX says `Valuador profesional es la persona…` | Exact term span before copula, not a general split on every `es`. |
| `lieps` 3 | VIII `Tabacos labrados y otros:` introduces named inciso definitions | Explicit nested entry paths/spans, reviewed parent versus child treatment. Never take only the first inciso or flatten scope without recording it. |
| `lsct` 2 | `“Tratado”: …` etc. quoted entry names | Paired-quote term-name support in the fraction style. This is not the `quoted_respectively` sentence style and does not imply quoted-only usages. |

## Rules for small, instrument-specific exceptions

Use **one typed, declarative extraction profile**, referenced by the existing
adapter, rather than instrument-name branches or a new parser for each law.
Implement only the operations exercised by these cases; no embedded code or
unbounded regular-expression patch language.

1. **Bind the input.** Profile identifies instrument, exact provision ID, exact
   UTF-8 text SHA-256, strategy version, rationale and evidence/review locator.
   Every hint includes its literal span and Unicode-character offsets. Hash or
   span drift fails closed; a new source needs review, not automatic rebinding.
2. **Describe structure, do not rewrite law.** Supported hints: exact entry
   boundaries, ignored non-entry candidates, observed label sequence, literal
   label parsing aliases, delimiter choice, quoted term-name bounds, and explicit
   term/definition spans. Keep canonical text and printed labels untouched.
   Derive offsets against that same text; no stripped scratch string offsets.
3. **Account for all material.** Every candidate entry/region is classified as
   definition, continuation, verified repeal placeholder, furniture, substantive
   non-definition, or unresolved. No missing final entry; no default ignore.
   Unknown leftovers hold the glossary. Overlap is allowed only for explicitly
   modeled nested entries/continuations, with tested ownership.
4. **Keep shared rules strict.** No `allow_gaps: true`, `ignore_errors`, global
   typo replacement or fallback that accepts the first N entries. A specific
   observed sequence may have a source-backed gap without relaxing others.
5. **Keep furniture repair separate.** Boundary hints can extract definitions
   from unchanged contaminated text now. They do not clean the canonical body
   or falsely certify a furniture-free definition span. Record that debt. Removing
   headers from canonical text still needs the narrow parser repair, retained
   source, reviewed diff and offset/review-binding checks.
6. **Escalate only semantics.** Formatting, an exact missing delimiter, and a
   source-verified numeral shape are technical. Which glossary governs, multiple
   imported laws' precedence, implicit definitions and nested semantic scope
   require a recorded operator decision. An exception is not legal approval.
7. **Verify once per reusable operation.** Add adversarial shared fixtures plus
   source-bound fixtures for every changed instrument. Private replay must show
   expected terms/usage changes and unchanged references, canonical provisions,
   review state and unrelated instruments. Persist extraction reports and inspect
   Markdown. Schema/types/validators change together for the new profile boundary.

This permits small reviewed data changes for later instruments once the operation
exists, instead of repeatedly changing the common parser.

## The Ley / Reglamento / Código prefix fix

The bare matcher treats the first word of a different instrument's full title as
a glossary usage. A blanket ban on `Ley de`, `Ley Federal`, etc. would also drop
true uses of the defined instrument's own full title. Quoted-only matching for
every instrument would lose legitimate bare uses and exceed the REG-LAN ruling.

Add a **title-span check before accepting a term candidate**, shared with the
existing official-title/alias registry where practical:

- Preserve explicit `usage_form: quoted` precedence. Do not widen REG-LAN.
- Identify complete title spans using pinned corpus titles and approved aliases,
  longest match first with lexical boundaries. Quoted longer titles also count.
- If `Ley` lies inside a title naming a different instrument from the term's
  source-backed referent, suppress that glossary usage; do not modify citation
  edges or turn the term occurrence into a new edge.
- If the title names the term's expressly defined referent, retain it. Record
  referent identity only from a reviewed exact definition/profile, not the word
  “Ley”, adapter parent, or an LLM guess. Existing glossary types do not currently
  supply this identity; add a validated optional binding or keep the case held.
- For unrecognized/ambiguous title-like continuations or an unbound referent,
  emit an auditable ambiguity record. Do not call the occurrence resolved based
  on a capitalization heuristic. Preserve before/after coverage and make any
  suppression policy explicit in its own reviewed change.
- Keep bare `la Ley`, `dicha Ley`, `esta Ley` under the existing applicable scope
  policy; full-title suppression alone does not prove every anaphor is correct.
- Unmatched quote pairs should produce a diagnostic, not quote repair or a
  guessed term usage. Exact matched quotes remain required in quoted mode.

Acceptance fixtures must contrast bare Ley, the defined law's own full title,
another law, titles absent from the corpus, shared/ambiguous titles, quoted full
titles, longer defined terms, punctuation/Unicode offsets and imported-term
precedence. Recompute the actual delta; **1,045 is a reported candidate upper
bound, not a verified false-positive count**. Review removals and retained true
positives before pinning the database release. This filter belongs in a separate
small change, not a rewrite of citations or canonical text.

## Execution order and validation

1. Fix repeal/qualified-sequence completeness guards and add deterministic
   persisted coverage reports. Audit the 94 admissions for the reproduced defect
   shapes; do not automatically withdraw or expand them based on synthetic cases.
2. Introduce the minimal source-bound profile and test easy delimiter/quote/
   abbreviation cases first, then label and furniture boundary cases.
3. Resolve LFRA/LIEPS entry semantics and LSS/multi-glossary scope independently;
   configure imports only after the relevant original is accepted.
4. Implement/review the title-span usage filter with source-backed referent
   bindings, then rerun release-wide term coverage and pin the DB input commit.

Fresh checks: fmt, Clippy with warnings denied, **324 tests**, and diff checks
pass. The total includes three integration tests. All 19 detector holds
reproduce. Independent JSON diff verifies the G1 totals and unchanged references.
The isolated public-API probe reproduces the three completeness counterexamples.
No full closure/CLI validation/export or corpus-wide legal review is claimed.

No production code, adapters, corpus, operator suggestions, DB worktree or Git
history changed in this review. Next action: bounded completeness-guard repair
and coverage-report design, followed by the first small exception batch.
