# Fintech structural pilot: contract and deviations from V2

**Status:** active, started 2026-11-14 on branch `pilot/fintech-structural` from `main`
`2d3c77a81`. Codex accepted slices (a) and (b). The G0 sample is committed (`0ecc40523`) and
checked from the builder side; Codex approved G0 with conditions on 2026-11-14
([disposition](../fintech-pilot-g0-codex-disposition-2026-11-14.md)). D6 is amended accordingly.
Codex returned **revise** on G1 draft 1
([review](../fintech-pilot-g1-codex-review-2026-11-14.md)) and on draft 2
([review](../fintech-pilot-g1-draft2-codex-review-2026-11-14.md)), the second limited to two rule
interactions. Codex approved the G1 draft-3 contract at `a6d94025f` on 2026-11-15
([review](../fintech-pilot-g1-draft3-codex-review-2026-11-15.md)). Local encrypted
backup and restore verification passed on 2026-11-16
([receipt](../fintech-pilot-holdout-custody-2026-11-16.md)); private evaluation
versioning remains required before G3.
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

## G1 review checkpoint, 2026-11-14

Codex reviewed the draft at `c35bb1f91` and returned **revise** in
`docs/fintech-pilot-g1-codex-review-2026-11-14.md`. G1 remains unapproved.
The review accepts D8 and conservative tail abstention, but requires ordered
lookup, scan precedence, ownership/span, response-schema and node-ID contract
repairs. A private sealed-answer compatibility conflict requires independent
versioning after contract freeze; G0 v1 stays unchanged. Backup custody still
blocks G2 separately.

**Current next action:** builder revises the public G1 contract/schema/examples
against the review and submits a new commit-bound G1 review. No G2 work yet.

## G1 draft-2 review checkpoint, 2026-11-15

Codex reviewed `f7641a537` and returned **revise**, limited to candidate-kind
holder selection inside tail incisos and note-aware cutoffs for no-fraction
tails. See `docs/fintech-pilot-g1-draft2-codex-review-2026-11-14.md`.
R1, R2, R4 and R5 are closed; R3 is substantially addressed with these remaining
interactions. All supplied native fixture/schema checks pass and measurement
hashes/counters reproduce. Private compatibility still requires evaluator
versioning after contract freeze; G0 v1 stays unchanged. G1 remains unapproved
and backup custody independently blocks G2.

**Current next action:** builder makes the two bounded public rule/fixture
revisions and submits a new commit-bound G1 review.

## G1 draft-3 approval checkpoint, 2026-11-15

Codex approved G1 at `a6d94025f`; both remaining draft-2 interactions are
closed. See `docs/fintech-pilot-g1-draft3-codex-review-2026-11-15.md`, including
the contract-bound compatibility record. All supplied fixture/schema checks
pass, prior expectations are preserved and measurement hashes/counters
reproduce. Original G0 gold remains unchanged; its known incompatibility
requires independent private evaluation versioning before G3.

**Current next action:** evaluator supplies encrypted holdout backup and a
successful restore/hash check before G2. G1 acceptance does not clear custody
or authorize G2/G3 execution. The frozen contract text below is unchanged.

## Holdout local recovery checkpoint, 2026-11-16

JRH authorized the encrypted backup and accepted local recovery. Codex created
an AES-256-GCM second copy outside the checkout/scratch directories, with the
key in the operator's macOS Keychain. Creation and a separate verification
invocation both passed authenticated in-memory restore and the original
SHA-256 check. See `docs/fintech-pilot-holdout-custody-2026-11-16.md`.
The original sealed gold and approved contract section remain unchanged.
The backup custody prerequisite is satisfied; no G2 execution occurred here.

**Current next action:** G2 local derived projection under approved G1.
Independent private evaluation versioning remains required before G3; the
builder continues to exclude sealed answers and holdout provisions before
code freeze under D6.

## G2 authorization and working method, 2026-11-16

JRH, 2026-11-16: "Yes switch to the leaner approach for G2". G2 runs as follows:
- **Code and tests first.** The lookup is a new read-only module on the shared fracción
  recogniser. The parser, canonical data and projection output stay untouched; output goes to
  scratch or test paths only.
- **Fixtures drive the tests.** The tests read the committed G1 fixtures (123 rows, 28 full
  responses, 32 must-fail responses) directly, never copies of them.
- **Disagreements are findings.** If the code and a row disagree and the code is wrong, the code
  is fixed. If a row or the prose is wrong or ambiguous, the fixtures, `verify.py` and the frozen
  contract stay unchanged. The disagreement is logged for the review.
- **Holdout excluded.** Development uses only the 16 development provisions and the synthetic
  rows. A wider corpus sweep skips the 8 holdout provisions and reports aggregates only. The
  measurement probe serves as a second oracle on non-holdout text.
- **One Codex review.** The code is frozen at a named commit, and one Codex handoff follows at the
  end of G2, with short records. The evaluator, not the builder, runs G3.

Before handoff, an advisor reviewed the full validation battery on 2026-11-16 (session tier, not
independent): fmt, strict clippy, 545 tests, `validate` for all 468 instruments, fidelity and
qualifier audits equal to the recorded baseline, the G0/G1 verifiers, the schema check, the probe,
and the contract digest.

## G2 checkpoint, 2026-11-16

G2 is built in `1075a6ce7`, with no other change:
- `lex_core::structure_pilot`, the projection and the lookup;
- its fixture-driven tests;
- `examples/structure_pilot_lookup.rs`, the local command-line lookup for G3.

The tests pass against the committed fixtures:
- 123 rows, 346 requests, deciding branches included;
- 28 full responses, by exact JSON equality;
- the projection example, schema validity and determinism;
- the probe's classification and structural counters;
- a corpus sweep of all three modes, schema-validated. A later test-only commit widened these two.

The code met no row it had to disagree with. The workspace suite (556 tests), strict clippy,
both validators and the G0/G1 checks pass. Four deliberate rule breaks each failed the suite.
Six choices the contract leaves open are listed for review in the Codex handoff. **Next:** one
Codex review of `1075a6ce7` for code freeze; then the evaluator runs G3 and versions the private
gold.

## G2 code review checkpoint, 2026-11-16

Codex reviewed freeze candidate `b68cf68e5` (pilot code identical at checkout
`a6a790fd4`) and returned **revise** in
`docs/fintech-pilot-g2-codex-review-2026-11-16.md`. Native gates and supplied
fixtures pass, but public uppercase multi-letter and accented-capital marker
controls reproduce false `not_found` results. The six disclosed implementation
choices have bounded engineering dispositions in the review. The builder's
pre-freeze smoke-run holdout inclusion remains disclosed; no private answer
exposure or tuning is inferred. G3 has not run.

**Current next action:** builder repairs unsupported-marker recognition and
pins the reviewed interpretations, then submits the corrected commit for code
freeze acceptance. Frozen G1 text and gold remain unchanged.

## G2 revision checkpoint, 2026-11-16

`991b5dc85` answers the G2 review:
- **P2 fixed.** The G1.3 letter forms are read as written: capitals and lowercase letters of any
  script before `. `, two or more letters of any case before `)`, and Unicode dashes and bullets.
  No non-holdout corpus paragraph changes class.
- **Choices pinned.** Choices 1, 2, 5 and 6 are pinned by public synthetic tests.
  `set_current_text` now replaces only a loaded provision.
- **Contract gap, unchanged.** Single-letter `A)` and `á)` starts (2 in the non-holdout corpus, in
  `lic` 45-R) are not a G1.3 form. They stay content; Codex is asked to rule.
- **Next:** Codex re-review for code freeze.

## G1 contract, pilot-v1 (draft 3 for Codex review, 2026-11-15)

This section is the span, parenthood and lookup contract. D5 says this document absorbs V2's
`01-span-parenthood-contract` and `lookup-contract`. It was written from V2 §4, from the
development provisions, from synthetic examples, and from measurements on non-holdout articles.
No holdout provision and no holdout answer was read; the eight holdout provisions were
excluded from every measurement. Nothing here is code. The machine-checkable parts and the
expected behaviour are in `fixtures/fintech-structural-g1-2026-11-14/`.

Draft 2 answers the [Codex G1 review](../fintech-pilot-g1-codex-review-2026-11-14.md) (verdict:
revise). Each change is tied to one of R1–R5 and to a public witness, and G1.10 lists the
changes. No change was made for R6, and none uses the sealed answers: R6 belongs to an
independent evaluator after this contract is frozen.

Draft 3 answers the [Codex draft-2 review](../fintech-pilot-g1-draft2-codex-review-2026-11-14.md)
(verdict: revise, limited to items A and B). It changes three rules and adds the review's two
public witnesses; G1.11 lists the changes. Nothing else from draft 2 was redesigned, and again
nothing was changed toward the sealed answers.

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
- **Fail closed on any other layout.** The provision returns `not_structured`
  (`unexpected_layout`) if its text contains a lone `\n`, three or more consecutive `\n`, or
  leading or trailing whitespace. It also does so for the two marker layouts in G1.4 that the
  pilot does not model. None of the 1,092 non-holdout eligible articles shows any of these.

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
   qualifier rank), with rank 0 for no qualifier. Its printed form is `normalize_fraction_label`.
   `label_span` runs from the paragraph start through the separator (`marker.end_char`), so
   `XXVI bis.` is 9 characters.
2. **`inciso_marker`.** The paragraph starts with a single lowercase letter, then `)`, then
   whitespace. The letter order is a … n, ñ, o … z, and `label_span` is the 2 characters.
3. **`article_note`.** The paragraph starts with `Artículo adicionado`, `Artículo reformado` or
   `Artículo derogado`, contains `DOF dd-dd-dddd`, and is at most 400 characters. It belongs to
   the root, certainly.
   - A note is **closing** if every paragraph after it is also a note, and **interior**
     otherwise. A closing note never makes an attachment uncertain. An interior note does,
     for the paragraphs after it (G1.4, ownership rule 3).
   - Measured on non-holdout articles: 508 such paragraphs, including multi-clause and
     `Fe de erratas` forms. One is interior (`lic` 108 Bis), and it lies inside a tail.
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

   The pilot does not structure these. They make scans partial (G1.6) instead of being read
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

- **Root.** Every eligible provision has one synthetic `article_root`. It inherits the
  provision ID and creates no canonical provision.
- **Fractions.** Fractions are children of the root.
- **Regions.** A position is in the region of node N when N is the innermost node whose region
  contains it. Region ranges nest: a fraction's range contains the ranges of its incisos and, for
  T, of the tail incisos. Where G1.5 says a node's region *contains* a position, it means this
  range, so the innermost node and every node enclosing it all qualify.
  - The root's region is the whole text.
  - A fraction's region runs from its marker paragraph to the paragraph before the next
    fraction marker, or to the end of the text.
  - An inciso's region runs from its marker paragraph to the paragraph before the next marker
    paragraph of any kind, or to the end of its parent's region. It never extends past the
    start of the tail (below), so the tail lies in T's region and not in the region of an inciso
    of T's run.
  - A tail inciso's region runs from its marker paragraph to the paragraph before the next
    marker paragraph, or to the end of the text.
- **Incisos.**
  - In an article with fractions, incisos inside a fraction's region are that fraction's
    children.
  - In an article with no fraction, every inciso is a child of the root, and its parenthood is
    resolved.
  - **Unmodelled layouts (fail closed, G1.2).** An inciso before the first fraction of an
    article that has fractions makes the provision `not_structured` (`unexpected_layout`). So
    does an inciso that follows an interior note with no fraction marker between them, except
    inside the tail. Measured: neither occurs in non-holdout articles (probe counters
    `inciso_before_first_fraction` and `inciso_after_interior_note_outside_tail`, both 0).
- **The last run and the tail, in an article with fractions.** Let T be the last fraction.
  - **T's run** is T's marker paragraph followed by incisos in strictly increasing letter order,
    starting with the very next paragraph. Only `content` paragraphs may sit between two incisos
    of the run; an `unsupported_marker`, an article note or a letter that does not increase ends
    the run.
  - **The tail** runs from the first paragraph after T's run to the end of the text, leaving
    out a run of closing notes. If nothing remains, there is no tail.
  - The tail is one unresolved segment (`trailing_attachment`). Its candidate owners are the last
    marker node before it and each of that node's ancestors up to the root.
  - An inciso inside the tail has `parenthood_status` `ambiguous`, with candidate parents T and
    the root. G1.6 says how tail incisos affect scans, and G1.7 how they match.
  - Article notes inside the tail stay root-owned. The tail segment's range encloses them, but
    claims nothing about them.
  - Measured on non-holdout articles: of 357 with fractions, 245 have a tail, 71 end on a
    marker and 41 end on closing notes. 4 of the 245 tails contain an inciso.
- **The tail, in an article with no fraction.** Here T would be the root, so a tail inciso's
  candidate parents [T, root] collapse to one, and no inciso has an uncertain parent. The root
  has no run.
  - The tail runs from the first paragraph after the last inciso's marker paragraph to the end
    of the text, leaving out a run of closing notes. If nothing remains, there is no tail. This
    is the same shape as in an article with fractions, so the tail can begin with an interior
    note.
  - Article notes inside the tail stay root-owned, certainly. The tail segment's range encloses
    them but claims nothing about them. Ownership rule 2 comes before rule 3, so an unlabelled
    paragraph after such a note belongs to the tail segment, not to a `note_interruption`
    segment. The last inciso's certain span therefore ends at the end of its marker paragraph,
    before any note.
  - Its segment's candidate owners are the last inciso and the root.
  - An article with no marker at all has no tail.
- **Unlabelled paragraphs.** These are `content` and `unsupported_marker` paragraphs. For each
  one, let m be the nearest marker paragraph before it. The first rule that applies wins:
  1. **No m:** the root owns it.
  2. **Inside the tail:** it is part of the tail segment.
  3. **An interior note lies between m and the paragraph:** the paragraph is part of a
     `note_interruption` segment.
     - The segment runs from the first unlabelled paragraph after the note to the end of the
       last unlabelled paragraph before the next marker paragraph, the next note or the end of
       the text.
     - The note itself is not in the segment; it stays root-owned.
     - The candidate owners are m and each of m's ancestors up to the root.
  4. **m is a fraction F:** F owns it, certainly. This covers continuation paragraphs after F's
     marker paragraph, including one before F's first inciso.
  5. **m is an inciso and the next marker is an inciso:** m owns it, certainly. The next marker
     can only be m's sibling, because a fraction in between would be the next marker.
  6. **m is an inciso of fraction F and the next marker is a fraction:** F owns it certainly.
     Whether m does is uncertain, so it is a `post_child_attachment` segment with candidates
     [m, F].

  The list is exhaustive. If m is an inciso and there is no next marker, the paragraph is
  either inside the tail (rule 2) or after an interior note (rule 3).
- **Sequences.** Within each parent, the labels of the children whose parenthood is resolved
  must strictly increase:
  - fractions by (value, rank), so `V` < `V Bis` < `VI`;
  - incisos by letter.

  Tail incisos take part in no sequence. A label that does not increase gives that parent's scan
  for that kind the reason `sequence_break` (G1.6). Measured on non-holdout articles: 0 fraction
  sequence breaks, and 2 articles with an inciso sequence break. Gaps are allowed; one
  non-holdout article has a fraction gap, next to an `unsupported_marker`.

### G1.5 Spans and node identity

- **`label_span`.** The marker as printed, separator included, trailing whitespace excluded.
  The root has none.
- **`intro_span` of a non-root node.** It runs from the first non-whitespace character after the
  label to the end of the marker paragraph. Continuation paragraphs are not intro, even one that
  comes before the node's first child (ownership rule 4); they are reachable only in `complete`
  mode, and there they follow that mode's ambiguity rules (D8). The intro is ambiguous only when
  a same-line candidate lies inside it.
- **`intro_span` of the root.** It runs from 0 to the end of the last paragraph that comes before
  the first child marker and is not an article note. With no child marker the limit is the end
  of the text, so the intro of an article with no marker is all of its text except its closing
  notes. The intro is `[0,0)` when no such paragraph exists, as when the text opens with a
  marker; an `intro` lookup then resolves to that empty span, with empty text. It is ambiguous
  when a same-line candidate or an `unsupported_marker` paragraph lies inside it.
- **`complete_span` of the root.** The whole text, always resolved.
- **`complete_span` of a non-root node.** It runs from the label start to the end of the last
  paragraph that the node or one of its descendants certainly owns. It never includes the
  `\n\n` after it, and it never includes a root-owned note: G1.4 makes every layout that would
  put a note inside it a tail, a `note_interruption` or `unexpected_layout`, and each of the
  first two cuts the span before the note.
- **Which uncertainties cut which span.** Each source below either cuts a node's span or does
  not; nothing else does.

  | Source | Cuts the `complete` span of | Certain part ends |
  |---|---|---|
  | Tail segment | each candidate owner except the root | at the end of the last paragraph before the segment |
  | `note_interruption` segment | each candidate owner except the root | at the end of the last paragraph before the note |
  | `post_child_attachment` segment [m, F] | m only; F certainly owns the paragraphs | at the end of m's last certainly owned paragraph |
  | `unsupported_marker` paragraph U, owned by N (in a tail, N is the tail's first candidate) | N and each ancestor of N except the root | at the end of the paragraph before U |
  | Same-line candidate C, held by P (G1.6) | each node strictly below P whose region range contains C (G1.4): the innermost node and every node enclosing it | right after the `[:;.]` before C |

  - The same table governs intros. A non-root intro is cut only by a same-line candidate. The
    root's intro is cut by a same-line candidate or an `unsupported_marker` inside it.
  - When several sources cut one span, the certain part ends at the earliest of their ends. The
    response lists every applicable segment, in order of `start_char`.
  - A cut span is `ambiguous`. The response gives the certain part and the segments, never a
    longer span as certain. `certain_span` is `null` when the certain part would be empty.
- **`direct_content_spans`.** Not provided in pilot-v1.
- **`unresolved_segments`.**
  - A same-line candidate's segment runs from the candidate to the end of its paragraph, and
    the segments of two candidates in one paragraph may overlap. An `unsupported_marker`'s
    segment runs from that marker to the end of its paragraph.
  - Their candidate owners are, for a same-line candidate, the nodes it cuts, innermost first,
    followed by P; for an unsupported marker, N and each of its ancestors up to the root.
  - A tail, `note_interruption` or post-child segment runs over the uncertain paragraphs.
  - Each records `start_char`, `end_char`, a reason (`trailing_attachment`,
    `note_interruption`, `post_child_attachment`, `unsupported_marker`, `same_line_marker`)
    and the candidate owners' node IDs.
- **`node_id`.** `<provision_id>#pilot-v1:<first 12 hex of canonical_text_sha256>:<path>`.
  - The root's path is `root`.
  - Other paths join segments with `/`: `f.<numeral>[_<Qualifier>].<occurrence>` and
    `i.<letter>.<occurrence>`. `<Qualifier>` is the canonical name of the rank: `Bis`, `Ter`,
    `Quater`, `Quinquies`, `Sexies`, `Septies`, `Octies`, `Nonies` or `Decies`, whatever the
    printed spelling.
  - An inciso of an article with no fraction has the path `i.<letter>.<occurrence>`, with no
    fraction segment.
  - The occurrence is the 1-based count of same-label siblings so far, so it is always 1 unless
    a sequence breaks.
  - A tail inciso uses `tail/i.<letter>.<occurrence>`, counted among tail incisos.
  - IDs bind the text hash and version. No stability is promised across amendments.
- **Parent fields.** A node's `parent_node_id` is its parent, or `null` for the root and for a
  node with ambiguous parenthood. `candidate_parent_node_ids` is filled only when parenthood is
  ambiguous, and is empty otherwise. In a projection, an ambiguous `intro_span` or
  `complete_span` holds the certain part (`null` when empty), marked by its status.

### G1.6 Scan status per parent and kind

- **Which scans exist.**
  - The root has a fraction scan. In an article with no fraction it also has an inciso scan.
  - Each fraction has an inciso scan.
  - Incisos have no scans in pilot-v1.
  - An out-of-scope provision has no scans; its status is `unsupported`.
- **Reasons.** A scan (P, k) collects every reason that applies, and the projection and the
  response keep them all, always in the order listed here:
  - `sequence_break`: the labels of P's resolved children of kind k do not strictly increase
    (G1.4).
  - `same_line_marker`: a same-line candidate of kind k is held by P.
  - `unsupported_marker`: an `unsupported_marker` is owned by P or by a descendant of P.
  - `tail_inciso`: P is T, k is inciso, and the tail contains an inciso.
- **Reduction.** The status follows from the reasons, independent of the order they were found:
  - `ambiguous` if `sequence_break` is among them, since a broken sequence outranks partial
    evidence;
  - otherwise `partial` if any reason is present;
  - otherwise `complete`.
- **Who holds a candidate.**
  - A same-line candidate of kind k, found in the region of node N (G1.4), is held by P: N
    itself or N's nearest ancestor that can hold kind k.
    - **A fraction candidate** is held by the root, wherever it sits, including inside a tail
      inciso's region. The root's fraction scan then gains `same_line_marker`, so a request for
      that fraction follows L2, and no fraction request returns `not_found`.
    - **An inciso candidate** is held by the fraction whose region it is in, or by the root in an
      article with no fraction. Inside a tail inciso's region, whose parent is uncertain, it is
      held by T.
  - **No holder.** A same-line inciso candidate in the root's own region of an article with
    fractions (its chapeau) has no holder. It adds no scan reason and takes part in no lookup.
    It still cuts the root's intro (G1.5), and its segment's only candidate owner is the root.
  - **Measured.** Of the 26 same-line inciso candidates in non-holdout articles:
    - 20 sit in an inciso's region;
    - 3 sit in a fraction's region outside any inciso;
    - 3 sit in a tail;
    - none sits in a chapeau or in an article with no fraction.
  - An `unsupported_marker` owned by N is held by N and by each of N's ancestors.
- **What a candidate carries.**
  - A same-line candidate carries the label that `fraction_marker` or its letter gives it.
  - An `unsupported_marker` that starts with Roman capitals carries that numeral's value. It
    matches a requested fraction of the same value, whatever the qualifier, so `XV Bis 1.`
    matches `XV`, `XV Bis` and `XV Ter`.
  - Any other `unsupported_marker` carries no label.
- **Tail incisos and scans.** A tail inciso is a possible child of T that is not certainly T's,
  which is why it adds `tail_inciso` to (T, inciso). So no lookup under T returns `not_found`
  while T's tail holds an inciso (G1.7, branch L7).

### G1.7 Lookup

**Request.** A request is one JSON object. The lookup runs against one loaded projection set and
the current canonical corpus at a stated release; neither is a request field.

| Key | Required | Value |
|---|---|---|
| `provision_id` | yes | string |
| `locator` | yes | object, one of the two forms below |
| `mode` | no | string: `label`, `intro` or `complete`; `complete` when absent |

- **Subsection locator.** `{"fraction": …}`, `{"fraction": …, "inciso": …}`, or, in an article
  with no fraction, `{"inciso": …}`. Each value is a string.
- **Root locator.** `{"article_root": true}`, alone. The root is returned only for this explicit
  request. It is a separate operation, not a fallback; a subsection request never returns the
  root.
- **`node_id` requests are deferred in pilot-v1.** V2's lookup takes a typed structural locator
  only. A request carrying `node_id` returns `invalid_locator` (`unsupported_request_form`).

**Locator syntax (frozen).**
- **`fraction`.**
  - A canonical uppercase Roman numeral of value 1 to 100, optionally followed by exactly one
    space (U+0020) and one qualifier word.
  - The qualifier is any word `fraction_qualifier_rank` accepts: any letter case, `Quáter` with
    or without its accent.
  - Nothing else is allowed. That excludes leading, trailing or doubled whitespace, a period
    or dash, digits (`XV Bis 1`), a lowercase numeral (`iv`) and a non-canonical numeral
    (`IIII`).
- **`inciso`.** One letter from `a`–`z` or `ñ` (U+00F1), optionally followed by `)`. Uppercase
  letters and whitespace are invalid.
- **Matching.** Labels match on (value, rank) for fractions and on the letter for incisos, never
  on the printed string. So `X Quáter`, `X quater` and `X QUATER` all match a printed `X Quater.`.
- **Normalised path.** The validated locator, written `f.<numeral>[_<Qualifier>]`,
  `f.…/i.<letter>`, `i.<letter>` or `root`, with the canonical qualifier names of G1.5. It
  names the request, not a node, and has no occurrence number.

**Evaluation order.** The first stage that applies decides the outcome.
1. **`invalid_locator`.** Its checks apply in this order, and the first failure gives the reason:
   1. `malformed_request`: the request is not a JSON object.
   2. `unsupported_request_form`: the request has a `node_id` key.
   3. `unknown_key`: the request or its locator has any other key not listed above, such as
      `apartado`.
   4. `malformed_request`:
      - `provision_id` or `locator` is missing;
      - `provision_id` is not a string, or `locator` is not an object;
      - a locator value has the wrong JSON type, or `article_root` is not `true`.
   5. `unknown_mode`: `mode` is present and is not `label`, `intro` or `complete`, including
      when it is not a string.
   6. `empty_locator`: the locator is `{}`.
   7. `conflicting_locator`: `article_root` appears with `fraction` or `inciso`.
   8. `invalid_fraction_label` and `invalid_inciso_label`: the label fails the syntax above.
   9. `label_mode_on_root`: mode `label` on the root locator.
   10. `unknown_provision`: the provision ID is not in the current corpus.
2. **`stale_projection`.** This applies when the projection's binding differs from the current
   one. The reason is the first difference, checked in this order:
   - `hash_mismatch`: the provision's current text hash;
   - `version_mismatch`: the projection version;
   - `release_mismatch`: the source release ID.

   The response reports both bindings, the projection's and the current one, in full.
3. **`not_structured` (`unsupported_scope` or `unexpected_layout`)** under G1.1, G1.2 or G1.4.
4. **The root locator** returns the root in mode `intro` or `complete` (G1.5). It walks no path.
5. **Path walk.**
   - A `fraction` is looked up under the root.
   - An `inciso` is then looked up under the fraction found.
   - In an article with no fraction, an `inciso` alone is looked up under the root.

   Each lookup is one step (P, k, L), meaning label L of kind k under parent P, decided by the
   table below. The first branch that applies wins. Two terms are used:
   - **M**, the matches: nodes of kind k labelled L whose parent is P, or whose candidate parents
     include P.
   - **C**, the candidates held by P (G1.6) that carry L.

   | Branch | Applies when | Outcome and reason |
   |---|---|---|
   | L0 | an inciso is requested without a fraction in an article that has fractions | `ambiguous` (`incomplete_path`), listing every inciso of that letter anywhere, tail included; or `not_structured` (`incomplete_path`) if there is none. Never `not_found` |
   | L1 | some node in M has ambiguous parenthood | `ambiguous` (`ambiguous_parent`) |
   | L2 | C is not empty | `ambiguous` (`same_line_marker` if C holds a same-line candidate, otherwise `unsupported_marker`) |
   | L3 | M is not empty, and (P, k) is `ambiguous` | `ambiguous` (`sequence_break`) |
   | L4 | M has exactly one node | continue from it; the step resolves |
   | L5 | M is empty, (P, k) is `complete`, and L is above every label of P's children of kind k, or P has none | `not_found` (`absent_in_complete_scan`) |
   | L6 | M is empty, (P, k) is `complete`, otherwise (L below the first label or in a gap) | `not_structured` (`sequence_gap`) |
   | L7 | M is empty, and (P, k) is `partial` or `ambiguous` | `not_structured` (`partial_scan` or `sequence_break`, by the reduced status) |

   - M has two or more nodes only when parenthood is ambiguous (L1), or when two resolved
     children share a label, which is a sequence break (L3). So L4 always meets exactly one
     node with resolved parenthood.
   - Every `ambiguous` branch is a path ambiguity. Its response lists M and C together, in
     document order. Each candidate gives:
     - its `node_id`, or `null` for a same-line or unsupported candidate, which is not a node;
     - its path: a node's path, or for a candidate that is not a node, the parent's path
       followed by `/(same-line <label>)` or `/(unsupported <numeral>)`, with no prefix when the
       parent is the root;
     - its `label_span`.
6. **Mode applied to the resolved node.**
   - `label` returns `label_span`.
   - `intro` returns `intro_span`.
   - `complete` returns `complete_span`.
   - When the span is cut (G1.5), the outcome is `ambiguous` (`unresolved_boundary`): a span
     ambiguity.

A label lookup can therefore succeed while `complete` abstains for the same node.

**Response.** Every response echoes the raw `request` exactly as received. Every outcome except
`resolved` carries a `reason`. `mode` is the request's mode when that is `label`, `intro` or
`complete`, `complete` when the request is an object without a `mode` key, and `null` otherwise;
it does not depend on which check failed. The schema enforces the presence rules in this table:

| Field | `resolved` | `ambiguous`, path | `ambiguous`, span | `not_found` | `not_structured`, scope or layout | `not_structured`, path | `stale_projection` | `invalid_locator` |
|---|---|---|---|---|---|---|---|---|
| `reason` | absent | path reason | `unresolved_boundary` | `absent_in_complete_scan` | yes | yes | yes | yes |
| binding: `provision_id`, `canonical_text_sha256`, `projection_version`, `source_release_id` | yes | yes | yes | yes | yes | yes | absent; `projection_binding` and `current_binding` instead | absent |
| `normalised_path` | yes | yes | yes | yes | yes | yes | yes | absent |
| `scan_status`, `scan_reasons` | yes | yes | yes | yes, `complete` | absent | yes | absent | absent |
| `ambiguity` | absent | `path` | `span` | absent | absent | absent | absent | absent |
| `node_id` | yes | absent | yes | absent | absent | absent | absent | absent |
| `span`, `text` | yes | absent | absent | absent | absent | absent | absent | absent |
| `certain_span`, `unresolved_segments` | absent | absent | yes | absent | absent | absent | absent | absent |
| `candidates` | absent | yes, at least one | absent | absent | absent | absent | absent | absent |

- **Scan fields.** `scan_status` and `scan_reasons` describe the last step's (P, k), with the
  reasons in G1.6's order. They are `null` and `[]` when no step ran: the root locator, and
  branch L0.
- **Text.** `text` is the exact substring of the current canonical text at `span`.
- **What no other outcome does.** No outcome other than `resolved` returns text as an exact
  match.
- **What the schema cannot check.** JSON Schema cannot check the semantic links: `text` against
  `span`, the hash against the text, the `node_id` prefix against the binding. The fixture
  verifier checks those separately.

### G1.8 Glossary comparison, determinism and reporting

- **Glossary spans.** `DefinedTerm` spans are comparison evidence for G3, never rewritten. One
  convention is known to differ: a `fractions`-style glossary span ends at the next label's
  start. For example, `lmv` 2 `Control` is `[273,1013)`, while `complete_span` for III ends
  before the separator, at `[273,1011)`.
- **Determinism.** Output is UTF-8 JSON with sorted keys, nodes in document order and no floats.
  Two runs on the same input must be byte-identical.
- **Reporting.** Results are reported under D4 and V2 §7: resolved-correct, resolved-wrong and
  abstained, by mode.
- **Expected coverage.** The tail rule makes `complete` abstain for the last fraction in about
  69% of fraction articles (245 of 357). That is a coverage result to report, not to hide.
- **Where the counts come from.** Every count in this contract comes from the committed probe
  `measure.py` in the fixture folder (G1.9). Its full output, with the input commit and the full
  SHA-256 of each corpus file it read, is committed beside it as `measure-output.txt`. The input
  files are byte-identical to the G0 pin `c337dae1a`. G3 can re-run the probe and bind the
  counts to its own input.

### G1.9 Expected behaviour and how it was derived

The fixture folder `fixtures/fintech-structural-g1-2026-11-14/` holds:
- **`expected-behaviour.json`.** One row per query, giving each mode's outcome, reason,
  ambiguity kind and spans. Each row also names the deciding stage or branch (`V`, `S`, `N`, `R`
  or L0–L7) and lists the other branches whose conditions also hold. Row counts are in that
  file and in the verifier's output.
  - **Development rows.** These are the 23 G0 development queries in all three modes, plus rows
    on development provisions: tails, continuation between siblings, the `lmv` 2 same-line `e)`,
    gaps, values above the highest label, incomplete paths, and two root lookups. Each G0
    query's label span equals the G0 gold span.
  - **Synthetic rows.** They cover:
    - `.-`, a same-line fraction, `Quáter` matching, an `unsupported_marker`, a gap, a sequence
      break and a closing note;
    - incisos with an ambiguous parent, a lone `L`, invalid requests of every reason, and a
      stale projection;
    - a `post_child_attachment` and a continuation before a first inciso;
    - Codex's interrupted-note witness;
    - scan precedence, with a repeated label and an unsupported marker together;
    - a same-line candidate inside a tail;
    - a node cut by both a same-line candidate and a tail;
    - an article with no fraction, with root-child incisos and a root lookup;
    - layouts that fail closed: a lone `\n`, an inciso before the first fraction, and an inciso
      after an interior note;
    - a same-line inciso in a fraction article's chapeau;
    - a same-line fraction candidate inside a tail inciso (draft-2 review, item A);
    - a no-fraction tail that begins with an interior note (draft-2 review, item B).
- **`responses.json`.** Full response instances for every outcome and both ambiguity kinds.
  Among them are:
  - an invalid mode, an unknown key, a `node_id` request and a malformed value;
  - a successful root lookup, an empty root intro and a root-child inciso;
  - a stale binding and an `unexpected_layout`;
  - a root-held same-line fraction candidate, with the root's fraction scan `partial`, and a
    no-fraction tail segment that encloses a note.

  Each is tied to a row and a mode.
- **`responses-must-fail.json`.** Responses that break one presence or type rule each, which
  the schema must reject.
- **`projection-example.json`.** One full projection, for the synthetic article with no
  fraction.
- **`projection.schema.json`.** The pilot-only schema for projections, requests and responses.
  It is not under `schemas/`.
- **`schema-check/`.** A standalone Rust checker, outside the workspace and built offline from
  the workspace's pinned `jsonschema` 0.33. It checks:
  - the schema against its declared draft 2020-12 meta-schema;
  - every `responses.json` instance and the projection example against the schema;
  - every must-fail instance, which must be rejected;
  - each response's echoed `request`, which must match the request definition exactly when the
    outcome is not `invalid_locator`, and must not match it otherwise.

  Run it with
  `cargo run --offline --locked --manifest-path fixtures/fintech-structural-g1-2026-11-14/schema-check/Cargo.toml`.
- **`verify.py`.** It implements no G1 rule. It checks:
  - text hashes, exact quotes, and agreement with the G0 gold label spans;
  - span boundaries;
  - coverage of every branch, and of the branch pairs that show an earlier branch winning: L1
    over L2, L1 over L4, L2 over L3, L2 over L4, and L3 over L4;
  - the outcome and reason that each branch allows;
  - each full response against its row, and the semantic links that the schema cannot check.
- **`measure.py`.** The probe behind the quoted counts. It reads only
  `corpus/mx/{lritf,lic,lmv}/provisions.json` and drops the eight holdout IDs before reading any
  text. It prints the input commit, the file hashes and every named counter, zeros included;
  `measure-output.txt` is its committed output. It is evidence for G1.3, G1.4, G1.6 and G1.8,
  not the projection.

**Derivation.** Every span was read by hand from a paragraph table and entered by hand. The
table lists each paragraph's start, end and text or opening characters. A script added
substring hashes, quotes, the node-ID hash prefix and the JSON layout; no scanner, recognizer
or probe produced any expectation.

**Compatibility with the sealed holdout.** This contract was frozen without the holdout answers.
Codex's review found at least one sealed G0 answer incompatible with draft 1 (R6), without
naming it. Draft 2 was not adjusted toward it. After freeze, an independent evaluator compares
the frozen contract with the sealed answers privately. Any versioned gold or compatibility
record follows G0 condition 3: G0 v1 is kept unchanged, and no case is disclosed to the builder.

### G1.10 Changes in draft 2

| Review item | Change | Public witness |
|---|---|---|
| R1 | The lookup is a strict branch table (L0–L7). Ambiguous parenthood (L1) comes before unique resolution (L4). Tail incisos match under each candidate parent, take part in no sequence, and add `tail_inciso` to T's inciso scan, so they block `not_found` | `g1-syn-08-tail-incisos-q2`, new `-q4`, `g1-syn-17-*` |
| R2 | Scans keep every reason and reduce deterministically: `sequence_break` makes a scan `ambiguous`, any other reason makes it `partial`. Candidates that carry the label (L2) come before a sequence break (L3) | `g1-syn-15-*`, `g1-syn-16-*` |
| R3 | Closing and interior notes are defined. An interior note starts a `note_interruption` segment for the unlabelled paragraphs after it. An inciso after an interior note, outside the tail, fails closed. A table says which source cuts which span, and the earliest cut wins. A post-child segment cuts only the inciso. The intro and complete spans of the root, including in an article with no marker, are explicit. In an article with no fraction, incisos have resolved root parenthood | `g1-syn-14-*` (Codex's text), `g1-syn-18-*`, `g1-syn-19-*`, `g1-syn-12-*` unchanged, `g1-dev-root-*` |
| R4 | The request and the response are fully specified: a raw request echo, a nullable mode, a reason on every outcome except `resolved`, fields per outcome, `path` versus `span` ambiguity, and both bindings on a stale response. The node-ID pattern now accepts `i.a.1`. There are full and must-fail instances and a re-runnable meta-schema check | `responses.json`, `responses-must-fail.json`, `projection-example.json`, `schema-check/` |
| R5 | `node_id` requests are deferred, matching V2. The root has its own locator, `{"article_root": true}`, so `{}` is invalid. The locator syntax is frozen | `g1-syn-10-*` |
| R3 | Regions are defined for every node kind, so the same-line cut rule has a defined input. A same-line inciso in a fraction article's chapeau has no holder. Inciso layouts the pilot does not model fail closed | `g1-syn-20-*` to `g1-syn-23-*`, `g1-dev-sameline-lmv2-xix-d` unchanged |
| R3 | The root intro leaves out article notes, and an empty root intro resolves to `[0,0)`. This answers "define root-without-markers intro/complete explicitly" | `g1-dev-root-lic11`, `g1-syn-14-interior-note-q3` |
| R4 | Node paths and normalised paths use the canonical qualifier name (`Quater`), so one spelling serves the response's `normalised_path` and every print variant | `g0-dev-10-q3` response (`f.XXVI_Bis.1`) |
| R6 | No change. Left to the independent evaluator after freeze | none |

**Changed expectations from draft 1:**
- **Representation only, for all rows.** Each row now names its deciding branch, and each
  ambiguous result names its ambiguity kind. `not_found` rows gain the reason
  `absent_in_complete_scan`, and `invalid_locator` rows gain their reasons.
- **`g1-syn-10-invalid-locators-q5`.** The locator `{}` was invalid only in `label` mode,
  standing for the root. It is now `invalid_locator` (`empty_locator`) in every mode. The root
  is requested with `{"article_root": true}`.
- **`g1-syn-10-invalid-locators-q6`.** The invalid mode `body` is now given as a raw request,
  under the key `request`, with the reason `unknown_mode`.
- **Node paths.** Qualifier spelling in a path is now canonical (`Quater`). No draft-1 row
  showed a `Quáter` path.
- **No span or outcome of a draft-1 row changed.** The truncation table was checked against
  every draft-1 row. In particular, `g1-syn-12-post-child-attachment-q1` stays resolved, because
  the post-child segment cuts only the inciso `b)`.
- **No-fraction tails.** No draft-1 row covered an article with no fraction that has incisos
  (measured: 4 such articles outside the holdout, none with paragraphs after the last inciso),
  so the new rule changes no earlier row.

### G1.11 Changes in draft 3

| Review item | Change | Public witness |
|---|---|---|
| A | The holder rule is qualified by kind. A fraction candidate is held by the root everywhere, including inside a tail inciso's region; only an inciso candidate there is held by T. The root's fraction scan becomes `partial`, the matching fraction request follows L2, and no fraction request returns `not_found` | `g1-syn-24-*`, Codex's text; full responses for q1, q2 and q4 |
| A, consequence | The cut rule needed a defined input once a candidate's holder can sit two levels above it. Region ranges nest, and a same-line candidate cuts every node strictly below its holder whose range contains it, innermost first. In the witness this cuts T as well as the tail inciso; T's certain part still ends where the tail cuts it | `g1-syn-24-*-q4` (two segments) |
| B | The tail of an article with no fraction runs from the first paragraph after the last inciso's marker paragraph, as in an article with fractions. It encloses interior notes, which stay root-owned, and rule 2 keeps the paragraphs after them in the tail segment. The last inciso's certain span ends before the note. Root parenthood and the root's `complete` span stay resolved | `g1-syn-25-*`, Codex's text; full response for q1 |
| R6 | No change. Left to the independent evaluator after freeze | none |

**Changed expectations from draft 2.** None. All 116 draft-2 rows and all 24 full responses are
unchanged, compared case by case; only the fixtures' `contract` label now says draft 3. No
draft-2 row had a same-line fraction candidate outside a chapeau. The one draft-2 no-fraction
tail, in `g1-syn-19`, holds no note, so both tail definitions give it the same span. The probe
already starts a no-fraction tail at the first paragraph after the last inciso, so `measure.py`
and `measure-output.txt` are unchanged. Measured on non-holdout articles, neither layout occurs:
the probe counts 0 same-line fraction candidates (`same_line_fraction`) and 0 tails in the 4
articles with incisos and no fraction (`no_fraction_inciso_articles_with_tail`).
