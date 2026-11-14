# Fintech structural pilot: contract and deviations from V2

**Status:** active, started 2026-11-14 on branch `pilot/fintech-structural` from `main`
`2d3c77a81`. Codex accepted slices (a) and (b). The G0 sample is committed (`0ecc40523`) and
checked from the builder side; Codex approved G0 with conditions on 2026-11-14
([disposition](../fintech-pilot-g0-codex-disposition-2026-11-14.md)). D6 is amended accordingly. The
G1 contract below is a draft awaiting Codex review; G2 also needs the holdout backup evidence.
**Order:** JRH, 2026-11-13: fix the LIEPS transitory grouping first (merged as PR #3), then this
pilot, "with your recommendations applied".
**Authority:** canonical `Provision.text`, canonical IDs, reviewer decisions, temporal decisions,
source manifests and release artifacts stay unchanged. The pilot output is derived and local.
No public route, MCP tool, database migration or deployment is authorized.

## Basis

- **Plan input:** `prompts/V2_LEX_MEX_FINTECH_STRUCTURAL_PILOT_CONTRACT_FIRST_2026-11-13.md` (V2),
  preserved unchanged and tracked on JRH's 2026-11-14 instruction to commit pending work.
  This task-named document is the binding contract; departures from V2 are listed below.
- **Earlier draft V2 revises:** `prompts/LEX_MEX_FINTECH_STRUCTURAL_PILOT_AGENT_BUILD_PLAN_2026-11-13.md`.
- **Technical audit:** `prompts/LEX_MEX_PARSER_SYSTEM_INDEPENDENT_AUDIT_2026-11-13_CODEX.md`. Its
  reviewer is Codex. V2 attributes the separate plan comments to Astra.
- **Claude's recommendations:** four changes plus the advisor's additions. JRH approved them
  on 2026-11-13.

## Deviations from V2

| # | V2 says | This pilot does | Reason |
|---|---|---|---|
| D1 | The operator approves G0 (sample) and G1 (definitions) | Codex approves G0 and G1 through the provider-handoff queue. JRH hears only of a case that forces a decision about what a provision legally covers | These are engineering calls. Under the decision-routing rule in `AGENTS.md`, only strict legal questions go to JRH. Uncertain attachments return `ambiguous` by design |
| D2 | `VI.-` export parity is backlog after G1 (§9) | The first slice is one shared fracción recognizer plus the `VI.-` export fix, before G0 | About seven fracción recognizers disagree on the dash. The pilot must build on one rule, not add an eighth. Codex's audit recommends the same next step |
| D3 | The shared rule lives in `lex-grammar` (Claude's recommendation) | It lives in `lex-core`, as `fraction_marker`, beside `FRACTION_QUALIFIER_PATTERN`, `fraction_qualifier_rank` and `normalize_fraction_label` | `lex-grammar` depends on no project crate by recorded design (`docs/parser-rule-inventory.md` §6). The qualifier list lives in `lex-core`, and `lex-export` depends only on `lex-core`. No new crate edge |
| D4 | Hand-written locators | Queries come from real fraction-qualified citations that the linker has already recorded (`references.json` qualifiers of type `fraction`). Results are reported as resolved-correct, resolved-wrong (must be 0) and abstained, alongside V2 §7's coverage figures | Success then means real citations landing on exact text |
| D5 | Six roles and six documents (§10–§11) | Three documents: this contract (absorbing `00-baseline`, `01-span-parenthood-contract` and `lookup-contract`), one results document, and one decision record. Codex review files serve as `04-independent-review` | Proportionate to 20–30 provisions |
| D6 | The holdout agent selects and annotates | Kept, and treated as the step with real value. A fresh agent selects and annotates the sealed holdout from canonical text alone. **Amended 2026-11-14** ([Codex G0 disposition](../fintech-pilot-g0-codex-disposition-2026-11-14.md), condition 1): the holdout is answer-sealed and publicly identified. Its eight articles, their strata and their text are public; its queries, quotations, offsets and expected results stay private until the frozen implementation is evaluated. The builder does not use the eight provisions for development, tests, debugging or tuning before code freeze. Any earlier exposure of an answer, or tuning on a holdout provision, voids the holdout and requires a fresh sealed set. This is a workflow commitment, not technical isolation Codex reviews both gold sets. The holdout must include repeated labels, nested incisos, trailing paragraphs, `Bis`, and `.-` | Independence of the gold set is what makes the result meaningful |
| D7 | Verify the live published inventory first | The sample is the local committed corpus, labelled local. The public `llms.txt` and manifest may be read for the record, but membership is not inferred from them | No website edit or deployment is in scope. Local canonical data is what the projection reads |
| D8 | §4.2: `intro_span` is the text before the first child | `intro_span` is the rest of the marker paragraph. A continuation paragraph before the first child is reachable only in `complete` mode (G1.5) | This keeps intro certain, without an attachment question; no information is lost, because `complete` includes it |

## Not fixed by this pilot

- LGIPE's 20 folded articles.
- `353-Ñ` missing from the public lookup (Unicode routing).
- CNBV orphan amendment markers.
- Extractor-to-canonical completeness.

These affect correctness and stay separate work. The pilot adds precision: exact, checkable
subsection lookups and correct fraction links.

## Slice (a): shared fracción recognizer and `VI.-` export anchors

### Design

- **Shared core:** `lex_core::fraction_marker(text)` reads a marker at the start of `text`:
  - a Roman numeral from `I V X L C D M`;
  - optionally, a qualifier word that `fraction_qualifier_rank` knows, after whitespace or after
    `.` plus whitespace;
  - then `.`, optionally followed by `-` or `–`;
  - then the whitespace that must follow.

  It returns the numeral, the qualifier, the printed label, the separator, and `end_char`
  (Unicode scalar values through the separator). When a qualified reading does not complete,
  it falls back to the bare numeral (`XX. Bis de la ley` reads as `XX.`), as both earlier
  recognizers did. It applies no context policy.
- **Parser policy (`lex-parse` `is_fraction_start`):** acceptance is unchanged:
  - numerals of `I V X L C` only;
  - no lone `C` or `L`;
  - qualifiers only as ` Bis`, ` Ter`, ` Quáter`, ` Quater`, ` Quinquies` or ` Sexies`, after
    exactly one space;
  - the bare-numeral fallback (`XX. Bis. ` reads as `XX.`).

  This recognizer drives the page-break flush, and a changed flush would change
  `Provision.text`.
- **Export policy (`lex-export` `paragraph_anchors`):** it accepts everything the old regex
  `^([IVXLCDM]+(?:\.?\s+QUALIFIER)?)\.\s` accepted, plus the `.-` and `.–` separators.
  The dash does not anchor a lone `C`, `D`, `L` or `M`. In the corpus, all six such
  paragraphs are lettered apartados or list items: `lfd` 187 `D.- Por la expedición…`, and
  `oga` 639 and 661 `C.- Libro…`. The period form keeps its earlier treatment (see "Known
  limits").
- **Effect on output:** fraction block anchors (`^f-…`) and the fraction-anchored links built
  on them are Obsidian output only. Committed `corpus/*/markdown` carries no block anchors, so
  no committed file changes.
  - **Consumers checked.** The only consumers are `with_block_anchors` and the `obsidian` branch
    of `linked_string` (`crates/lex-export/src/lib.rs`). That covers both the export command
    and the `link --all` closure, which uses the same `link_targets`/`term_targets`. Markdown
    mode writes `[text](note.md)` with no fragment.
  - **No independent rule elsewhere.** The JavaScript reader (`reader/`) and `lex-db` have no
    fraction-anchor rule of their own.

### Proof

- **Parser oracle (one-off, not committed).** The old `is_fraction_start` and the new wrapper
  were compared on 3,392,862 inputs:
  - every trimmed line of every retained `.work/*/*.txt`;
  - every pair of adjacent lines joined as an open block would be;
  - a synthetic grid: 11 numerals × 9 separators × 12 qualifier forms × 9 tails.

  They differ on 0 inputs.
- **Corpus replay.** A full replay of 465 instruments (parse → derive-temporal → validate →
  Markdown export, in a scratch root) is byte-identical across the whole `corpus/` tree to
  the same replay with the `4fa0bcb79` parser. Against the committed corpus, both replays show
  only the standing `cue-dcg-2003`, `scap-dcg-2012` and held `lftr` differences. The three
  slugs not replayed are as in the LIEPS repair: their retained inputs do not match their
  manifests.
- **Export oracle (one-off, not committed).** The old regex and the new policy were run on
  every canonical paragraph: 251,798 paragraphs. The old regex matched 82,652 and the new
  policy matches 91,114. Every old match keeps its key except one. The 8,462 gained matches
  all use the `.-` separator.
  - **The exception** is `cpf` article 217's `I. bis.- …`. The old regex read it as `f-i`
    while skipping the real `I.- …` paragraph, so a link to fracción I previewed fracción I
    Bis. It is now `f-i-bis`, and `I.-` is `f-i`.
  - **The synthetic grid** changes keys only in that same shape: `N. QUALIFIER.- `, read as `N`
    before and as `N`+qualifier now.
  - **Reconciliation with the Obsidian figure.** 22 of the 8,462 newly matched paragraphs
    repeat another fraction label in the same provision, so neither copy is anchored. That
    leaves 8,440, the anchor gain measured below. No newly matched paragraph gives way to a
    term anchor.
- **Tests:**
  - six `lex-core` unit tests for `fraction_marker`, including the bare-numeral fallback;
  - four `lex-export` tests from verbatim canonical text:
    - `lgtoc` 176 (`I.-`…`VI.-`);
    - `cpf` 217 and `lcndh` 23 (qualified dash fractions);
    - `reg-lfmzaah` 20 (a repeated `VII.-` gets no anchor);
    - `lfd` 187 (dash lettered apartados get no fraction anchor);
  - the parser's `fraction_markers_are_recognized_narrowly`, extended with the bare-numeral
    fallback cases;
  - a permanent parser parity test, `fraction_start_acceptance_matches_the_legacy_recognizer`.
    It checks a 21,384-input grid against the pre-slice function, which is kept under
    `cfg(test)`, so slice (b) cannot drift the parser unnoticed.

### Measured Obsidian effect

The full Obsidian export of all 468 instruments was written twice, into scratch vaults only
(evidence kept in the session scratchpad: `anchors-vault-before`, `anchors-vault-after`, and the
classifier `anchorcount.py`, re-runnable as `python3 -I anchorcount.py BEFORE AFTER`),
with `LEX_MEX_OBSIDIAN_VAULT` unset: once with `main` `2d3c77a81` and once with this slice.
Both vaults hold 68,867 notes. Each differing line was reduced to plain text (anchors and link
syntax removed). No line differs in anything but fraction anchors and link targets.

| Change | Count |
|---|---|
| Fraction block anchors gained | 8,440 (80,689 → 89,127 net) |
| Anchors lost to a newly visible duplicate | 2: `lfd` 187 `f-ii` and `f-iii`. Their twins under another apartado are printed `II.-` and `III.-`. No link pointed to either |
| Anchors re-keyed | 1: `cpf` 217, `f-i` → `f-i-bis` |
| Cited numerals newly linked to their fraction (`CCom` 1154 "fracciones II y III del artículo 1151") | 693 |
| Term links narrowed from the whole defining article to its fraction (`lfaebsp` "Instituto" → `articulo-2o#^f-vi`) | 1,545 |
| Links removed, or moved to another note | 0 |

### Known limits (recorded, not fixed in slice (a))

- **Lone letters with a period.** 217 paragraphs (`C.` 117, `D.` 70, `L.` 26, `M.` 4) still
  anchor as `f-c`, `f-d`, `f-l` and `f-m`, as before slice (a). About 24 follow their Roman
  predecessor (`XLIX` before `L.`) and are real fractions. The rest are lettered apartados or
  list items. A context rule ("the preceding fraction is this numeral's predecessor") is
  proposed as a separate change.
- **Letters `I`, `V` and `X`.** In a lettered list, `I.`, `V.` or `X.` (and their dashed forms)
  are lexically identical to fracciones I, V and X (`oga` 639 `I.- Libro de instrucciones…`),
  so the dashed forms now anchor too. Only the pilot's per-parent scan, which sees the
  sibling sequence, can tell them apart.

### Shapes still not recognized

Counts are canonical paragraphs the new recognizer rejects, as of 2026-11-14. They are
recorded for the pilot's unsupported-shape report, not fixed here.

| Shape | Example | Paragraphs |
|---|---|---|
| Dash, then text with no space | `I.-Aeronave` (`reg-lgs-msi` 5) | 88 |
| Period, then text with no space | `XXVII.Información` (`fi-dcg-2014` 14 Bis) | 19 |
| Numeral, space, capitalised word, no separator | `III Asesores…` (`servinv-dcg-2013` 1) | 115 |
| Letter-suffixed fraction | `XXIX-A.` (`cpeum` 73) | 26 |
| Numeral or letter followed by `)` (includes letter incisos `C)`, `D)`) | `I). Por…` (`lfd` 40) | 102 |
| Editorial `(sic …)` after the numeral | `VI (sic DOF 02-08-2006).` (`lgimh` 38) | 12 |

## Slice (b): glossary rules

Each rule in `crates/lex-parse/src/terms.rs` that recognizes a fracción marker is migrated to
`fraction_marker` one at a time. Each migration must either reproduce the committed glossary
byte for byte or be its own commit with a fixture and a disposition of every changed term.
Reviewed glossary profiles (`provision_text_sha256`, offsets) must not move without the
existing re-confirmation path.

### Result: six rules migrated, all byte-identical

All six fracción rules in `terms.rs` now read the marker through `fraction_marker_where`, a
sibling of `fraction_marker` in `lex-core`. Each rule's context (what may precede the numeral),
separator policy and tail stay its own. `fraction_marker` itself returns exactly what it did,
since it is now `fraction_marker_where` with every reading accepted.
- **Why a sibling.** A rule that rejects the qualified reading must still get the bare one, as
  its regex did by backtracking. Under a period-only rule, `XX. Bis.- Texto` reads `XX.`. A
  plain filter on `fraction_marker` would lose that reading.
- **Commits, in order:**

  | Commit | Rule | Context before the numeral | Separator |
  |---|---|---|---|
  | 1 | `fraction_entries` | a paragraph start | `.` (one-line body) |
  | 2 | mixed | `;`, line break, `(antes fracción N) ` | `.` |
  | 3 | established | `;`, `:`, line break, a `DOF` date | `.` |
  | 4 | wider | `;`, `:`, `.`, line break, `y`, a `DOF` date | `.` or `.-` |
  | 5 | swallowed-fraction check | whitespace | `.` or `.-`, then a capital |
  | 6 | `roman_colon` | `:` or `;` and whitespace | `.`, then the term and `: ` |

  Rules 2 to 6 share `scan_fraction_markers`. It reproduces the regex scan: leftmost match
  first, no overlap with the previous match, and the start of the text tried before the first
  context match.
- **Permanent parity tests.** Each old regex is kept under `cfg(test)` in
  `terms::fraction_rule_parity`, and its rule's full output must match it.
  - The output compared is byte offsets, match ends, labels and terms, plus the matched text for
    the swallowed check.
  - The grids: 84,000 paragraph shapes for `fraction_entries`; 504,000 context, marker and body
    combinations for each scan rule; `roman_colon` gets both.
  - They include `XX. Bis.- Texto`, numerals of seven and eight letters, `.-` and `.–` under
    each rule, a body that runs onto a second line, an empty `fractions` body, and consecutive
    fractions split only by a line break.
  - The grids miss two shapes, so `edge_shapes_match_the_legacy_rules` lists them explicitly:
    - a `roman_colon` colon right after the marker's whitespace (`XX. Bis. : x`, where only
      the bare reading has a term; `I.  : x`, where the term is whitespace);
    - swallowed headings that open with an accented capital, `Ñ` or a lowercase letter.
  - Without that test, two breaks would pass: reading the `roman_colon` term after the scan
    instead of inside `accept`, or dropping `ÁÉÍÓÚÑ` from the swallowed rule's capitals.
  - A deliberate break of each kind failed at least one of these tests: dropping `.-` from the
    wider rule, skipping the start of the text, or raising the numeral limit to eight.
- **Corpus oracle** (one-off, not committed). Old and new were run over every provision text in
  `corpus/mx` (68,397 provisions): each text whole, each paragraph and line as is and trimmed,
  1,442,394 inputs in all. There were 0 differences for every rule. The old rules found:
  - `fraction_entries`: 330,608 entries;
  - mixed: 448,435 markers;
  - established: 456,260 markers;
  - wider: 507,866 markers;
  - swallowed: 385,092 matches;
  - `roman_colon`: 29,419 entries.
- **Replay sensitivity.** A deliberately broken `fraction_entries` changed `terms.json` for
  `lritf` and `itf-dcg-2018` (the two `fractions`-style instruments) after `parse` alone. So the
  replay does reach the glossary rules.
- **Replay.** parse → derive-temporal → validate → Markdown export of the 465 instruments, once
  with rule 1 alone and once with all six rules. Both `corpus/` trees are byte-identical to
  slice (a)'s replay. They have the same two known validation holds (`lftr`, `scap-dcg-2012`)
  and the same error output.
- **Intermediate commits.** Each of the six commits was checked on its own in a scratch
  worktree, with fmt, strict clippy and its parity tests. The full workspace tests and the
  validators ran on the last commit.
- **Out of scope, noted.** `lex-link`'s `fraction_labels` (`^([IVXLCDM]+)\.\s`) also
  recognizes fraction paragraphs, for same-article references. Moving it would change link
  data, so it needs its own impact review.

## G0 to G3

The gates are V2's, with D1, D4, D5 and D6 applied.

- **G0, sample (Codex approves).**
  - Local snapshot at a pinned commit.
  - The main set comes from Diputados laws: `lritf` plus one or two financial laws.
  - `itf-dcg-2018` and `ifpe-dcg-2021` act as the "different format, expected to abstain" case
    while the DCG parsers are on hold.
  - Development cases are drawn from fraction-qualified reference edges into those provisions.
  - The sealed holdout comes from the fresh agent (D6).
- **G1, contract (Codex approves).** V2 §4 semantics: `label`, `intro` and `complete` spans;
  `unresolved_segments`; one synthetic article root; human locator versus derived `node_id`;
  coverage per parent and per child kind; and typed outcomes (`resolved`, `ambiguous`,
  `not_structured`, `not_found`, `stale_projection`, `invalid_locator`). They are written into
  this document before any projection code.
- **G2, projection.** A derived, hash-bound JSON projection in scratch or test paths only,
  built from canonical `Provision.text`.
- **G3, harness and holdout.** A local command-line lookup, run once on the sealed holdout and
  reported in full, followed by the decision record (`adopt local`, `iterate` or `abandon`).

## Codex review of slice (a), 2026-11-14

At `e7d67b719`, slice (a) accepted in
`docs/fintech-pilot-slice-a-codex-review-2026-11-14.md`. Codex accepts D1's
engineering reviewer role for future G0/G1 submissions; neither gate is approved
yet. D2–D7 accepted within the recorded scope, retaining independent gold and
frozen evidence despite the reduced narrative document count. Lone-letter dash
exclusion and paragraph-level term-link narrowing are accepted with the existing
ambiguity/unsupported-shape limits. Native fmt, locked strict clippy, workspace
tests and LRITF/IFPE/LIEPS validators pass; supplied replay trees compare equal.
Next action: slice (b), one glossary rule at a time with parity or explicit
fixture/disposition, followed by a pinned G0 sample submission. Four unrelated
untracked prompts preserved. Codex made no code/corpus edits, commits or pushes.

## Codex review of slice (b), 2026-11-14

At `c337dae1a`, slice (b) accepted for the committed corpus and specified policy
shapes in `docs/fintech-pilot-slice-b-codex-review-2026-11-14.md`. This is not
universal Unicode equivalence: simple case-folding versus lowercase lookup is
a disclosed limit, with no observed corpus occurrence. lex-link fraction_labels
stays outside this slice and requires its own impact review.
Native fmt, locked strict clippy, 545 tests/doc-tests and LRITF/IFPE/LIEPS
validators pass. Supplied root1/root6 replay trees compare equal to slice a;
68,397-provision oracle input independently matches the current corpus. The
one-off oracle was not rerun; replay trees remain author-generated artifacts.
Next action: G0 submission with pinned local sample/provenance and fresh-agent
sealed holdout design; G0/G1 remain unapproved until separately reviewed.
Four unrelated prompts preserved; no code/corpus edits, commit or push by Codex.


## G0 sample submission prepared, 2026-11-14

Submission: `docs/fintech-pilot-g0-submission-2026-11-14.md`. Frozen fixtures:
`fixtures/fintech-structural-g0-2026-11-14/`. Pin: full HEAD
`c337dae1af9474d0aef246b3de196c49e2527722`, local committed corpus only.
24 provisions: 16 development (23 candidate label queries) and 8 disjoint
independent holdout provisions (15 private label queries). LRITF/LIC/LMV plus
two DCG scope-abstention controls. Repeated child labels, nested incisos,
trailing uncertainty, qualified Bis, dot-dash, absent labels, definition witness
and citation-not-marker observations are represented.

A fresh agent selected and annotated holdout from canonical text alone. Answers
remain ignored at `.work/fintech-g0-holdout-2026-11-14/holdout-gold.json`, bound
by the public manifest digest. Builder must not inspect until G1/code freeze.
No complete-span gold or structural projection is asserted before G1; this is
a G0 candidate evidence set. Independent QA report:
`docs/fintech-pilot-g0-independent-review-2026-11-14.md`.

Mechanical verification: both fixture verify.py commands pass, including sealed
commitment; 24 exact records, disjoint split, development spans and source/seed/
definition/artifact hashes verified. Diff whitespace check passes; scoped
corpus/adapters/crates/schemas diff against HEAD is empty. No Cargo gate repeated
for this sampling-only step. Prior slice-b report/plan work and four untracked
prompts preserved; no staging/commit/push, source fetch or corpus/code change.

**Current next action:** independent G0 gate review of this submission, then G1
contract freeze before projection implementation. The rolling writer remains
disabled for lex-mex; repository plan, existing capsule and project-day receipt
retain continuity. Do not start a new capsule.

Independent G0 sample QA completed: the fresh reviewer found this an acceptable
submission with no required artifact revision, missing stratum or gold-binding
failure. Report: `docs/fintech-pilot-g0-independent-review-2026-11-14.md`.
The review keeps private answers undisclosed. G0/G1 approval remains pending;
next action is the G0 engineering gate disposition, followed by G1 semantics.

## Builder-side check of the G0 sample (Claude), 2026-11-14

Claude committed Codex's slice (b) verdict and G0 package unchanged as `0ecc40523`. This
section is the builder's check of the public artifacts only. It is not a G0 disposition.

- **Who did what.** Codex curated the development gold and assembled the package, and under
  D1 Codex also approves G0. The holdout was selected and annotated by one fresh agent and
  checked by another, so the overlap is limited to the development set and the package as a
  whole. This check adds a reviewer from a different model family for the development set.
  Decision (non-legal, recorded here): G0 disposition stays with Codex under D1, in a Codex
  session that did not curate the package. JRH may override.
- **Development gold.** All 20 resolved labels sit at a line start (corrected on 2026-11-14 from
  19, a miscount). Each one falls under the
  locator's parent: the fraction heading before an inciso is the fraction named in the locator,
  including the two `b)` labels in `lritf` 98 under III and IV. `lic` 11 has no fraction heading
  at all, so the `not_found` query is consistent with its text. The `lmv` 2 `Control` witness
  runs from fraction III to the start of IV. The two DCG controls follow G0's scope rule, and
  the outcome names come from G1's list. `verify.py` passes before and after the commit.
- **What the seal covers.** The eight holdout articles and their strata are public, and their
  text is in `canonical-snapshot.json` and in the corpus. Only the queries and answers are
  sealed. So D6's "the builder does not see it" holds for the answers, not the articles. The
  builder will not use those eight provisions as development or test input before code freeze.
  Their sealed answers were not opened: only their SHA-256 was checked, and it matches the
  manifest.
- **Strata only in the holdout.** The `.-` label form and a label placed after prose on the same
  line appear in the holdout but not in the development gold. That is acceptable for a test of
  generalisation. G1 must still settle these cases in writing before code freeze:
  - whether a label after prose on the same line counts as structural;
  - how labels are normalised, for example the source's `XXVI bis.` against the locator's
    `XXVI Bis`;
  - which paragraph a trailing unlabelled paragraph belongs to;
  - when `not_found` may be returned instead of an abstention.
- **Custody.** The holdout answers exist in one unencrypted copy in the Git-ignored `.work`
  folder, which has been cleaned before. It should be backed up before any cleanup.
- **Gate with the fixtures present** (via `mech`):
  - fmt and strict clippy, exit 0;
  - `cargo test`, exit 0 with 545 passed and 0 failed;
  - `validate` for `lritf`, `ifpe-dcg-2021` and `lieps`, 0 issues;
  - `verify.py` and `git diff --check`, exit 0.

**Next action:** Codex's G0 disposition, then the G1 contract written into this document
before any projection code.

## G0 gate disposition, 2026-11-14

Codex approved the pinned G0 sample with conditions in
`docs/fintech-pilot-g0-codex-disposition-2026-11-14.md`. The disposition came
from a fresh non-curating receiving session. The answer commitment and snapshot
verifier passed; no private answers were inspected or disclosed. D6 must explicitly
record answer-sealed/public-membership isolation before G1 approval. Original
gold stays immutable; contract conflicts require independently reviewed
versioning. A recoverable evaluator-controlled backup is required before G2.

**Current next action:** builder records the D6 amendment and submits the G1
semantics for separate Codex approval. No projection work is approved here.


## Historical prompt inputs committed, 2026-11-14

JRH authorized committing all pending work in stages. The slice-b/G0 review and
sample were already committed as `0ecc40523` and the builder-side check as
`3ba9dbbf2` before this instruction. The remaining parser audit request/report
were preserved unchanged in `61bde01eb`; the original and V2 pilot briefs are
tracked unchanged with this plan correction. These are historical inputs, not
authorization to execute their superseded instructions. Current D1–D7 and gate
dispositions remain authoritative. G0 was still awaiting its non-curating review
at the prompt-archival checkpoint; the later gate disposition above records the
current conditional approval. G1 remains unapproved. The ignored holdout answers
remain private and untracked.

Checks for these documentation-only commits: UTF-8/NUL and credential-pattern
scan of all four inputs, exact staged-content comparison, and staged whitespace
checks. No Cargo gate repeated; no code, corpus or trusted schema changed.
No push or merge authorized or performed.

## G1 contract, pilot-v1 (draft for Codex review, 2026-11-14)

This section is the span, parenthood and lookup contract. D5 says this document absorbs V2's
`01-span-parenthood-contract` and `lookup-contract`. It was written from V2 §4, from the
development provisions, from synthetic examples, and from measurements on non-holdout articles.
No holdout provision and no holdout answer was read; the eight holdout provisions were
excluded from every measurement. Nothing here is code. The machine-checkable parts and the
expected behaviour are in `fixtures/fintech-structural-g1-2026-11-14/`.

### G1.1 Scope

- **Eligible provisions.** `provision_type` `article` in `lritf`, `lic` and `lmv`, with scope
  `statute_body`. This list decides scope, not a guess about the format.
- **Everything else returns `not_structured` (`unsupported_scope`) and is not scanned.** That
  covers transitories, every other instrument, and in particular `itf-dcg-2018` and
  `ifpe-dcg-2021`, even though their articles show clean Roman labels.

### G1.2 Input, offsets and layout

- **Input.** The projection reads canonical `Provision.text` only.
- **Offsets.** Spans are half-open offsets in Unicode scalar values (Rust `char`) into that exact
  text.
- **Binding.** The projection binds:
  - `canonical_text_sha256`, the SHA-256 of the text's UTF-8 bytes;
  - `projection_version` `pilot-v1`;
  - `source_release_id` `local-<40-hex commit>`.
- **Paragraphs.** A paragraph is a maximal run of text between `\n\n` separators.
- **Fail closed on any other layout.** If an eligible text contains a lone `\n`, three or more
  consecutive `\n`, or leading or trailing whitespace, the provision returns `not_structured`
  (`unexpected_layout`). None of the 1,092 non-holdout eligible articles has any of these.

### G1.3 Paragraph classes

Each paragraph gets exactly one class, decided by its start and tested in this order.

1. **`fraction_marker`.** `lex_core::fraction_marker` accepts the paragraph start, so the
   separator is `.`, `.-` or `.–` followed by whitespace, and any qualifier is one that
   `fraction_qualifier_rank` knows. The numeral must also pass this policy:
   - it is in canonical Roman form (`IV`, not `IIII`);
   - its value is 1 to 100;
   - a lone `L` or `C` counts only when the previous fraction's base value is 49 or 99;
   - a lone `D` or `M` never counts.

   A start that fails the policy is `unsupported_marker`. The label is the pair (value,
   qualifier rank). Its printed form is `normalize_fraction_label`. `label_span` runs from the
   paragraph start through the separator (`marker.end_char`), so `XXVI bis.` is 9 characters.
2. **`inciso_marker`.** The paragraph starts with a single lowercase letter, then `)`, then
   whitespace. The letter order is a … n, ñ, o … z, and `label_span` is the 2 characters.
3. **`article_note`.** The paragraph starts with `Artículo adicionado`, `Artículo reformado` or
   `Artículo derogado`, contains `DOF dd-dd-dddd`, and is at most 400 characters. It belongs to
   the root and never makes an attachment ambiguous.
   - Measured on non-holdout articles: 508 such paragraphs, including multi-clause and
     `Fe de erratas` forms.
   - Notes printed inside a paragraph (`…; Fracción adicionada DOF …`) are ordinary text and
     stay inside that paragraph's spans.
4. **`unsupported_marker`.** Any other marker-like start. That means one of:
   - Roman capitals not followed by a letter (`II En caso…`, `XV Bis 1.`);
   - a capital letter, a period and a space (`A. `, a possible apartado);
   - digits followed by `.`, `)`, `°`, `º` or `o`;
   - lowercase Roman followed by `.` or `)` (`ii)`);
   - two or more letters followed by `)`;
   - a lowercase letter, a period and a space;
   - a dash or bullet followed by a space.

   The pilot does not structure these. They make the scan partial (G1.6) instead of being read
   silently as continuation text. Measured: 37 such paragraphs in non-holdout articles.
5. **`content`.** Everything else.

**Same-line candidates.** These are possible markers inside a paragraph. The projection records
them but never turns them into nodes.
- **Fraction:** `[:;.]` and whitespace, then a reading that `fraction_marker` accepts, then
  whitespace and a letter.
- **Inciso:** `[:;.]` and whitespace, then `x)`, then whitespace and a letter.
- Citations such as `fracción I de este artículo` or `incisos a), b) y c)` do not match, because
  no `[:;.]` comes right before the label.
- **Measured, under this pattern only:** 0 fraction candidates and 26 inciso candidates in the
  non-holdout articles. One inciso candidate is in a development provision: `lmv` 2, an `e)`
  inside XIX's `d)`.

### G1.4 Tree and ownership

- **Root.** Every eligible provision has one synthetic `article_root`. Its `complete_span` is the
  whole text, it inherits the provision ID and it creates no canonical provision.
- **Fractions.** Fractions are children of the root.
- **Incisos.**
  - Incisos inside a fraction's region are that fraction's children. A fraction's region runs
    from its marker paragraph to the paragraph before the next fraction marker, or to the end of
    the text.
  - In an article with no fraction, incisos are children of the root.
  - An inciso before the first fraction of an article that has fractions makes the provision
    `not_structured` (`unexpected_layout`).
- **The last run and the tail.** Let T be the last fraction, or the root in an article with no
  fraction.
  - **T's run.** For a fraction, the run is its marker paragraph followed by incisos in strictly
    increasing letter order, starting with the very next paragraph. For the root, the run starts
    at its first inciso. Only `content` paragraphs may sit between two incisos of the run; an
    `unsupported_marker`, an article note or a letter that does not increase ends the run.
  - **The tail.** It runs from the first paragraph after T's run to the end of the text, leaving
    out a closing run of article notes. An article with no marker has no tail.
  - The tail is one unresolved segment (`trailing_attachment`). Its candidates are the last
    marker node before it and each of that node's ancestors up to the root.
  - Incisos inside the tail have `parenthood_status` `ambiguous`, with T and the root as
    candidate parents.
  - Measured on non-holdout articles: of 357 with fractions, 245 have a tail, 71 end on a
    marker and 41 end on article notes.
- **Unlabelled paragraphs.** These are `content` and `unsupported_marker` paragraphs. For each
  one, let m be the nearest marker paragraph before it. The first rule that applies wins:
  1. **No m:** the root owns it. This is the root's intro.
  2. **Inside the tail:** it is part of the tail segment.
  3. **m is a fraction F:** F owns it, certainly. This covers continuation paragraphs after F's
     marker paragraph, including one before F's first inciso.
  4. **m is an inciso and the next marker is an inciso:** m owns it, certainly. The next marker
     can only be m's sibling, because a fraction in between would be the next marker.
  5. **m is an inciso of fraction F and the next marker is a fraction:** F owns it certainly.
     Whether m does is uncertain, so it is a `post_child_attachment` segment with candidates
     [m, F].
- **Sequences.** Within each parent, sibling labels of a kind must strictly increase:
  - fractions by (value, rank), so `V` < `V Bis` < `VI`;
  - incisos by letter.

  A label that does not increase makes that parent's scan for that kind `ambiguous`. Measured
  on non-holdout articles: 0 fraction sequence breaks, and 2 articles with an inciso sequence
  break. Gaps are allowed; one non-holdout article has a fraction gap, next to an
  `unsupported_marker`.

### G1.5 Spans and node identity

- **`label_span`.** The marker as printed, separator included, trailing whitespace excluded.
  The root has none.
- **`intro_span`.**
  - For a non-root node, it runs from the first non-whitespace character after the label to the
    end of the marker paragraph. Continuation paragraphs are not intro, even one that comes
    before the node's first child (ownership rule 3); they are reachable only in `complete`
    mode. This departs from V2 §4.2, which defines intro as the text before the first child
    (D8). The intro is therefore certain unless a same-line candidate lies inside it.
  - For the root, it runs from 0 to the end of the last paragraph before the first child marker,
    and is empty (`[0,0)`) if the text starts with a marker.
- **`complete_span`.** It runs from the label start to the end of the last paragraph that the
  node or one of its descendants certainly owns. It never includes the `\n\n` after it.
  - It is `resolved` only when no unresolved segment can extend it: no segment whose candidates
    include this node (or a descendant) and also a node outside it.
  - It must also contain no candidate (G1.6) that could start a sibling or an ancestor-level
    node.
  - Otherwise `complete` returns `ambiguous`. The response gives the certain span and the
    unresolved segments, never a longer span as certain.
  - **Where the certain span ends:**
    - before a tail or `post_child_attachment` segment, at the end of the last certainly owned
      paragraph;
    - before an `unsupported_marker` paragraph, at the end of the paragraph before it;
    - before a same-line candidate inside the node, right after the `[:;.]` that precedes the
      candidate.

    An ambiguous intro reports its certain span the same way.
- **`direct_content_spans`.** Not provided in pilot-v1.
- **`unresolved_segments`.** A same-line or unsupported marker's segment runs from that marker
  to the end of its paragraph. A tail or post-child segment runs over the uncertain paragraphs.
  Each records `start_char`, `end_char`, a reason
  (`trailing_attachment`, `post_child_attachment`, `unsupported_marker`, `same_line_marker`) and
  the candidate owners' node IDs.
- **`node_id`.** `<provision_id>#pilot-v1:<first 12 hex of canonical_text_sha256>:<path>`.
  - The root's path is `root`.
  - Other paths join segments with `/`: `f.<printed normalised label, space→_>.<occurrence>` and
    `i.<letter>.<occurrence>`.
  - The occurrence is the 1-based count of same-label siblings so far, so it is always 1 unless
    a sequence breaks.
  - An inciso with an ambiguous parent uses `tail/i.<letter>.<occurrence>`.
  - IDs bind the text hash and version. No stability is promised across amendments.
- **Human locator.** `{fraction?, inciso?}` under the provision. It is not the `node_id`.

### G1.6 Scan status per parent and kind

| Status | Meaning |
|---|---|
| `complete` | No `unsupported_marker` and no same-line candidate of that kind in the parent's region, and the kind's sequence strictly increases |
| `partial` | A candidate exists that could be a missed node of that kind under this parent |
| `ambiguous` | The kind's sequence breaks under this parent |
| `unsupported` | Out of scope (G1.1) |

**Candidates.** A same-line candidate of kind k, found in the region of node N, could be a node
of kind k under P. P is N or N's nearest ancestor that can hold kind k: the root for fractions,
and the fraction (or the root, when there are no fractions) for incisos. Then:
- (P, k) is `partial`;
- every node strictly below P whose region contains the candidate gets an `ambiguous` complete
  span;
- that node's intro is also ambiguous if the candidate lies inside its intro.

**Unsupported markers.** An `unsupported_marker` owned by node N could be any kind under any
ancestor. N and each of its ancestors, root included, are therefore `partial` for every kind.
Each of them except the root gets an `ambiguous` complete span.

### G1.7 Lookup

**Request.** It carries:
- `provision_id` and `locator`; a `node_id` may stand in for both;
- `mode`, one of `label`, `intro` or `complete`, with `complete` as the default;
- the projection to read.

Locator labels are matched on (value, rank) for fractions and on the letter for incisos, never
on the printed string. So `X Quáter`, `X quater` and `X QUATER` all match a printed `X Quater.`.

**Evaluation order.**
1. **`invalid_locator`** if any of these holds:
   - the request has an unknown key, an unknown mode or an unknown `provision_id`;
   - a fraction is not a canonical Roman numeral of value 1 to 100 with an optional known
     qualifier;
   - an inciso is not one letter (an optional `)` is stripped);
   - mode `label` is requested for the root.
2. **`stale_projection`** if the provision's current text hash, the projection version or the
   source release ID differs from the projection.
3. **`not_structured` (`unsupported_scope` or `unexpected_layout`)** under G1.1 or G1.2.
4. **The path is walked from the root.** For a requested label L of kind k under parent P,
   these tests apply in order. A node whose parent is ambiguous counts as a match under each of
   its candidate parents. Candidates carry labels as follows:
   - a same-line candidate carries the label that `fraction_marker` or its letter gives it;
   - an `unsupported_marker` that starts with Roman capitals carries that numeral's value. It
     matches a requested fraction of the same value, whatever the qualifier, so `XV Bis 1.`
     matches `XV`, `XV Bis` and `XV Ter`;
   - any other `unsupported_marker` carries no label.
   - **An inciso requested with no fraction, in an article that has fractions:** `ambiguous`
     (`incomplete_path`) with every inciso of that letter as a candidate if one exists, and
     otherwise `not_structured` (`incomplete_path`). This case never returns `not_found`.
   - **Exactly one match, no same-line or unsupported candidate under P carries L, and
     (P, k) is not `ambiguous`:** continue from the match.
   - **Two or more matches, a candidate under P carrying L, a match with an ambiguous parent, or
     (P, k) `ambiguous` with a match:** `ambiguous`, with every candidate's path and label span.
     Each candidate also carries its `node_id`; a same-line candidate is not a node, so its
     `node_id` is `null`.
   - **No match, and (P, k) is `complete`:**
     - L is greater than every present label, or no label of kind k exists: `not_found`;
     - L lies below the first present label or strictly between two of them: `not_structured`
       (`sequence_gap`), since a gap is where lost text would show.
   - **No match, and (P, k) is `partial` or `ambiguous`:** `not_structured` (`partial_scan`
     or `sequence_break`).
5. **Mode applied to the resolved node:**
   - `label` returns `label_span`;
   - `intro` returns `intro_span`, or `ambiguous` if a candidate lies inside it;
   - `complete` returns `complete_span`, or `ambiguous` under G1.5.

A label lookup can therefore succeed while `complete` abstains for the same node.

**Response.** Every response carries:
- the outcome, the mode and a reason code;
- `provision_id`, `canonical_text_sha256`, `projection_version` and `source_release_id`;
- the normalised path;
- the scan status of the last (parent, kind) step;
- candidates, when the outcome is `ambiguous`.

Only a `resolved` outcome carries `node_id`, the span and the exact substring. No other outcome
returns text as an exact match, and no lookup ever returns the whole article in place of a
subsection.

### G1.8 Glossary comparison, determinism and reporting

- **Glossary spans.** `DefinedTerm` spans are comparison evidence for G3, never rewritten. One
  convention is known to differ: a `fractions`-style glossary span ends at the next label's
  start. For example, `lmv` 2 `Control` is `[273,1013)`, while `complete_span` for III ends
  before the separator, at `[273,1011)`.
- **Determinism.** Output is UTF-8 JSON with sorted keys, nodes in document order and no floats.
  Two runs on the same input must be byte-identical.
- **Reporting.** Results are reported under D4 and V2 §7.
- **Expected coverage.** The tail rule makes `complete` abstain for the last fraction in about
  69% of fraction articles (245 of 357). That is a coverage result to report, not to hide.

### G1.9 Expected behaviour and how it was derived

`fixtures/fintech-structural-g1-2026-11-14/expected-behaviour.json` holds 73 rows:
- **40 development queries.** These are the 23 G0 development queries in all three modes, plus
  17 more on development provisions: tails, continuation between siblings, the `lmv` 2 same-line
  `e)`, gaps, values above the highest label and incomplete paths. Each G0 query's label span
  equals the G0 gold span; `verify.py` checks this.
- **33 rows from 13 synthetic texts.** They cover `.-`, a same-line fraction, `Quáter`
  matching, an `unsupported_marker`, a gap, a sequence break, a closing article note, incisos
  with an ambiguous parent, a lone `L`, invalid locators, a stale projection, a
  `post_child_attachment`, and a continuation paragraph before a first inciso.

**Derivation.** Every span was read by hand from a paragraph table and entered by hand. The
table lists each paragraph's start, length and opening characters. A probe of the G1.3 patterns
and G1.4 tail rule over non-holdout articles supplied the counts quoted above. It also surfaced
the `lmv` 2 same-line `e)`, whose spans were then read and entered by hand. The probe is
session-scoped author evidence and is not committed. `verify.py` in that folder checks the
following, and implements no G1 rule:
- text hashes;
- exact label quotes, and agreement with the G0 gold label spans;
- that each span ends at a paragraph boundary or, for a certain span before a same-line
  candidate, just before that candidate's whitespace;
- that each intro starts after the label's whitespace.

**Compatibility with the sealed holdout.** This contract was frozen without the holdout answers.
A reviewer may compare it with them privately. If a rule contradicts a sealed answer, G0 v1 is
kept and the change is versioned under the disposition's condition 3; the conflicting case is
not disclosed to the builder.
