# Fintech structural pilot: contract and deviations from V2

**Status:** active, started 2026-11-14 on branch `pilot/fintech-structural` from `main`
`2d3c77a81`. Codex accepted slices (a) and (b). The G0 sample is committed (`0ecc40523`) and
checked from the builder side; G0 awaits Codex's disposition, and G1 has not started.
**Order:** JRH, 2026-11-13: fix the LIEPS transitory grouping first (merged as PR #3), then this
pilot, "with your recommendations applied".
**Authority:** canonical `Provision.text`, canonical IDs, reviewer decisions, temporal decisions,
source manifests and release artifacts stay unchanged. The pilot output is derived and local.
No public route, MCP tool, database migration or deployment is authorized.

## Basis

- **Plan:** `prompts/V2_LEX_MEX_FINTECH_STRUCTURAL_PILOT_CONTRACT_FIRST_2026-11-13.md` (V2). It
  stays untracked and unedited. This document is the binding contract, and every departure
  from V2 is listed below.
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
| D6 | The holdout agent selects and annotates | Kept, and treated as the step with real value. A fresh agent selects and annotates the sealed holdout from canonical text alone, and the builder does not see it until code freeze. Codex reviews both gold sets. The holdout must include repeated labels, nested incisos, trailing paragraphs, `Bis`, and `.-` | Independence of the gold set is what makes the result meaningful |
| D7 | Verify the live published inventory first | The sample is the local committed corpus, labelled local. The public `llms.txt` and manifest may be read for the record, but membership is not inferred from them | No website edit or deployment is in scope. Local canonical data is what the projection reads |

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
- **Development gold.** All 19 resolved labels sit at a line start. Each one falls under the
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
