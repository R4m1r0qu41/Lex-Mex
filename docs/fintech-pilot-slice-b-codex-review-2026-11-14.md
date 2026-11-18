# Codex review — Fintech pilot slice (b)

Reviewed `38717248d..c337dae1af9474d0aef246b3de196c49e2527722` on
`pilot/fintech-structural`: six migrations and two documentation/test commits.
Initial worktree: four unrelated untracked prompts, preserved. Source set:
lex-core marker callback, six terms.rs policies and their legacy parity tests,
pilot contract, and the named session-scoped evidence. Output: this verdict.
Stop condition: slice disposition and next-gate recommendation; no G0 selection
or prototype implementation in this review.

## Verdict

**Accept slice (b) as a migration preserving the recorded corpus behavior and
specified policy shapes.** No blocking finding in this bounded diff. This is
not a proof of equivalence on every Unicode string: the disclosed qualifier
case-folding difference remains a real limitation, described below.

The six rules were committed separately; no corpus, source manifest, canonical
text, reviewer record or term data changes in the reviewed range. I agree that
lex-link's fraction_labels must remain outside this slice and receive its own
impact review if migrated. It changes link-data acceptance, so parity evidence
for these glossary rules would not establish its correctness.

No objection to submitting G0 next. Pin the local sample, source identities,
exact hashes and development rationale. Select/annotate the sealed holdout in a
fresh agent without builder context, and keep expected answers from the builder
until code freeze. Existing reference edges are query inputs, not gold truth.
This review does not approve a not-yet-submitted G0 sample or G1 span contract.

## Code and equivalence assessment

- `fraction_marker_where` preserves fraction_marker's earlier behavior when
  accept always returns true: qualified success still wins, otherwise the bare
  reading is attempted. Caller rejection can now expose the regex's bare-label
  fallback. The core regression tests that distinction.
- `scan_fraction_markers` tries the absolute start once, then uses find_at on
  the original text, preserving word-boundary context for `\by`. A failed
  context advances one Unicode scalar from its start, so overlapping possible
  starts are still considered; successful matches resume at their consumed end.
  Current context patterns cannot match the empty string. Their alternatives
  cannot supply two useful marker starts at the same byte position. Backtracking
  optional punctuation after a DOF date would leave punctuation where a Roman
  numeral must begin, so it does not supply a missed reading.
- Gap trimming preserves each rule's distinction between space/tab, arbitrary
  whitespace, and no gap. A leading newline is not treated as an absolute-start
  match for the space/tab rules; it can still serve as an explicit context.
  Numeral-length and period/hyphen/en-dash policies are per caller.
- fraction_entry_paragraph preserves the one-line body restriction after the
  greedy whitespace prefix; its callback may reject a qualified reading before
  trying the bare one. The scanner's byte offsets remain private slicing offsets;
  existing output construction still converts to character spans.
- swallowed_fraction keeps the capital set A–Z/ÁÉÍÓÚÑ and returns context through
  the first capital, as the old regex did. roman_colon retains its anchored tail
  regex inside candidate acceptance, preserving its backtracking/term boundary
  rather than parsing the term only after choosing a marker.

The retained legacy regexes match the pre-slice patterns. Permanent tests compare
full private outputs (labels, offsets, ends and terms), with explicit accented
capital and colon-adjacent cases supplementing the grids. Removing Result from
two now-infallible vector-producing functions does not alter their output.

## Known limitation and evidence provenance

Regex Unicode simple case-folding accepts some characters, such as long s in
`quinquieſ`, that fraction_qualifier_rank's lowercase lookup does not. The
migration is therefore not universally regex-equivalent. It is disclosed and
not observed in the committed-corpus oracle; acceptance here is scoped to the
current corpus and the explicitly covered shape policies. Before claiming
universal equivalence or expanding qualifier input policy, add a permanent
case-folding regression and choose/document that policy explicitly.

The one-off 1,442,394-input oracle was inspected but not independently executed.
The scratch comparisons below use supplied author-generated replay trees; they
do not establish their build provenance or substitute for a fresh final-HEAD
replay. In particular root6 predates the disclosed final mechanical edits.
Permanent tests run against the reviewed HEAD supply the final-tree checks.
Author development checks ran directly; author gate/commits used a mechanical
delegate. That procedural distinction does not invalidate their technical data.

## Validation actually run

Mechanical delegate ran all with exit 0:
- `cargo fmt --check`
- `cargo clippy --locked --workspace --all-targets -- -D warnings`
- `cargo test --locked --workspace` (545 non-doc tests passed; doc-tests passed;
  complete output retained locally at `/tmp/lex-sliceb-gate.log`)
- `cargo run --locked -p lex-cli -- validate lritf`
- `cargo run --locked -p lex-cli -- validate ifpe-dcg-2021`
- `cargo run --locked -p lex-cli -- validate lieps`

Validators report zero issues; clippy's walkdir warning is non-failing. All
permanent glossary parity tests and the earlier parser/export tests run in the
workspace suite.

Both supplied replay comparisons (`sliceb/root1/corpus` and
`sliceb/root6/corpus` against `slicearoot/corpus`) exited 0 with no differences.
After inspection of dump_texts.py, the delegate regenerated its NUL-separated
input under `/tmp`: 68,397 provisions; cmp against the supplied texts.bin
exited 0. This independently binds that oracle input to the current corpus,
while the one-off oracle result itself remains author evidence.
`git diff --check` passed.

Only this review and the pilot plan were changed in the repository; four
unrelated untracked prompts were preserved. No code/corpus edits, staging,
commits, pushes, merges or deployments. Next action: submit the pinned G0 sample
and independent-evaluation design for review.

Continuity note: the required AgentOS rolling_context.py writer rejected the
prepared update because writes are not enabled for lex-mex. The rolling file
was left unchanged; repository plan, daily receipt and existing local capsule
carry this result. This was a tool configuration rejection, not an approval-review
rejection; no AgentOS settings were changed.
