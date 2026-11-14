# Codex bounded re-review — LIEPS title boundary

Handoff target and verified working HEAD:
`4fa0bcb7906063217be1a61fddd068611f5b8d02`, branch
`fix/lieps-transitory-grouping`. Reviewed diff: `7b68c3554..4fa0bcb79`.
Initial dirty state consists of four unrelated untracked prompt files, preserved.

## Verdict

**Accept the bounded repair; close the P2 title-boundary finding.** Together
with this fix, the previously reviewed `191f8299d` has no remaining blocking
finding from this review. `64afb12eb` remains accepted. This verdict does not
merge or publish the branch.

`cites_decree_mid_sentence` now lets an independently published title-case act
open despite the preceding comma/lowercase ending. The exact reproducer is a
permanent regression expecting both 2010-03-03 and 2011-04-04 `unico` records.
The real LIEPS wrapped citation remains covered by the original fixture.

`awaits_its_publication_line` prevents the new look-ahead from splitting an
unfinished surrounding title. I independently read the retained RGIC source
around lines 1994–2000: the ACLARACION title spans the cited decree line and
owns the later publication line, consistent with the added test. The uppercase
opening heuristic requires at least four uppercase characters followed by a
lowercase connector, and is used only while deciding this particular ambiguous
decree boundary. It is a source-layout heuristic, not general title proof.
The negatives cover an article heading and an already published title.

The publication search is bounded to eight following raw lines before the first
blank line. This is acceptable for the named repair and current source-backed
cases. A publication line beyond that window or across a blank/page-furniture
break remains unsupported by this narrowing; this review does not claim that
all future source layouts are covered. The handoff discloses a retained-source
scan with no such affected sites. Its scan and 465-instrument replay were not
independently repeated here.

The diff changes parser code/tests and status/plan documentation only; there is
no new corpus adoption or legal decision. Four unrelated prompts remain intact.

## Validation

Mechanical delegate ran these at the reviewed HEAD, all exit 0:
- `cargo fmt --check`
- `cargo clippy --locked --workspace --all-targets -- -D warnings`
- `cargo test --locked --workspace` (526 tests passed; doc-tests passed)
- `cargo run --locked -p lex-cli -- validate lritf`
- `cargo run --locked -p lex-cli -- validate ifpe-dcg-2021`
- `cargo run --locked -p lex-cli -- validate lieps`
- `cargo run --locked -p lex-cli -- validate rgic`

All four validators report zero issues. The delegate also confirmed the exact
comma-boundary regression and the original LIEPS grouping regression individually.
Clippy emitted a non-failing walkdir warning. `git diff --check` passed.

Only this report and the execution plan changed in the repository this session.
No code/corpus edits, staging, commits, pushes, merges or deployments.
Next action: operator-directed PR #3 update/integration; the handoff requires
JRH's instruction before merge.
