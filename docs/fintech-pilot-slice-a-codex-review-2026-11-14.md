# Codex review — Fintech pilot slice (a)

Reviewed HEAD `e7d67b719109f481b9550415f405bcb987f11fcf` on
`pilot/fintech-structural`, range `2d3c77a81..e7d67b719` (18684b2b4 and
e7d67b719). Initial dirty state: four unrelated untracked prompts, preserved.
Source set: the shared recognizer, parser policy/parity oracle, export anchor
registry, pilot contract, targeted V2 gate clauses and supplied scratch evidence.
Output: this verdict. Stop condition: bounded slice disposition and explicit
answers to the contract questions, without starting slice (b) or G0.

## Verdict and contract

**Accept slice (a), within its disclosed lexical and export limits.** No blocking
finding in the reviewed diff. This is not approval of a completed G0 or G1:
neither the frozen sample nor the detailed span contract has been submitted.

D1 is acceptable: I accept the engineering reviewer role for G0 and G1 under
repository decision routing. A future handoff must contain pinned sample/gold
provenance for G0, then actual boundary, identity, coverage and outcome semantics
for G1. A legal interpretation still goes to JRH. The builder must not see the
sealed holdout answers before code freeze; use a fresh agent without inherited
builder context for that task.

D2–D7 are reasonable scoped deviations. D3 avoids a new dependency edge and
keeps the lexical rule beside the existing shared qualifier vocabulary in
lex-core. D4's recorded references are useful query inputs, not gold truth:
independent annotations and negative/unsupported cases remain required. D5's
reduced document count must retain V2's frozen sample, independent gold,
provenance and results artifacts; it reduces narrative duplication, not evidence.
D7's explicitly local baseline does not establish published corpus membership.

Proceed with slice (b) as specified: migrate each terms.rs rule separately,
with byte parity or a fixture and disposition of every changed term. G0/G1
remain subsequent explicit engineering review gates before projection code.

## Code review

`lex-core::fraction_marker` is a lexical recognizer, not Roman-numeral validation
or proof of structural parenthood. Its byte slicing occurs at ASCII numeral or
character boundaries, while end_char counts Unicode scalar values. Qualifier
fallback preserves the bare reading when the qualified form cannot finish.
`marker_ending_at` requires a separator followed by whitespace, including for
the en-dash form. Existing qualifier normalization supplies the anchor key.

The parser wrapper deliberately keeps the old narrower numeral/qualifier policy,
including exact qualifier case and spacing and the bare-period fallback. The
retained legacy function matches the pre-slice implementation. The permanent
21,384-input grid exercises the policy differences rather than asserting that
the lexical recognizer accepts every shape the parser accepts unchanged.

The export registry now admits dash separators, still suppresses duplicate keys
within a provision, and shares that registry between anchor emission and link
target selection. CPF 217's distinct I and I Bis keys are a correction supported
by a verbatim fixture. No canonical text, IDs, manifests or reviewed decisions
change in this slice.

Answers to the export questions:

- Excluding unqualified lone C/D/L/M before a dash is a reasonable conservative
  policy for this slice's observed lettered headings. Period-form false positives
  and ambiguous I/V/X remain disclosed; this is not a general structural resolver.
  A future sibling-context change needs its own fixtures and impact review.
- Narrowing term links is acceptable because containing_paragraph_anchor uses
  the existing canonical definition start and the same unique-anchor registry.
  These are paragraph navigation targets, not newly certified complete-definition
  boundaries. Ambiguous duplicate labels still fall back to the note.
- No objection to the stated sequence for slice (b), independent G0 sampling,
  then G1. Do not derive expected annotations from the candidate recognizer.

## Evidence boundary

The 3,392,862-input source oracle was not independently rerun. The permanent
parser parity grid is run as part of the native tests. Supplied scratch exports
and replay trees, when compared below, are author-generated artifacts; rerunning
the comparisons does not independently regenerate those trees or prove their
build provenance. The code diff contains no corpus changes.

## Validation actually run

Mechanical delegate ran all with exit 0:
- `cargo fmt --check`
- `cargo clippy --locked --workspace --all-targets -- -D warnings`
- `cargo test --locked --workspace` (all suites/doc-tests passed)
- `cargo run --locked -p lex-cli -- validate lritf`
- `cargo run --locked -p lex-cli -- validate ifpe-dcg-2021`
- `cargo run --locked -p lex-cli -- validate lieps`

All three validators report zero issues. The parity-grid test was also confirmed
individually (one passed), and five export fraction tests passed. The delegate's
workspace output was truncated: its provisional count of 527 differs from the
handoff's 537 and cannot be substantiated from a complete retained log. This
review claims the successful suite exit, not an independently verified total.
Clippy emitted a non-failing walkdir warning.

After inspecting the read-only script, the delegate reran `python3 -I
anchorcount.py anchors-vault-before anchors-vault-after` on the supplied scratch
artifacts: 68,867 notes per side, no unmatched notes, zero findings; 8,440 gained
anchors, two lost (LFD 187 f-ii/f-iii), one rekey (CPF 217 f-i to f-i-bis), 693 new
links and 1,545 term fragments added. `diff -rq p2root/corpus slicearoot/corpus`
exited 0. These substantiate the artifact comparison claims within the provenance
limit above. `git diff --check` passed.

Only this report and the pilot plan changed in the repository. No code/corpus
edits, staging, commits, pushes, merges or deployments. Next action: slice (b)
under the accepted migration discipline, followed by the G0 submission.
