# Parser batch review — 2026-10-20

Reviewed `parser/pending-batch` at `cf8eed626`, based on `db48f899d`.
This is a read-only audit of the seven local commits, separate from the accepted
DB milestone at `fedcbc5d`. The designated Claude integration session retains
ownership of the parser branch and DB-first merge. No parser/corpus changes,
merges or pushes were performed by this review.

## Findings

### P1 — Batch completion no longer proves corpus glossary closure

`crates/lex-cli/src/main.rs:1462`, `run_batch_closure`, relinks, validates and
republishes only `successful_slugs`. But `title_phrases` reads titles from every
committed instrument and the shared alias table. A newly admitted title can
therefore invalidate another instrument's usages and coverage while the batch
reports `ok`. The handoff and decision entry disclose this; code inspection
confirms the missing dependency. The already-relinked current batch is not
shown stale by this finding, and this is not a blocker to DB-first integration.
It is a blocker to trusting future admission completion/publication without
the documented manual corpus-wide closure.

Repair: after the batch's admissions, run the existing protected corpus-wide
closure once, require the exact accepted-gap contract (no additional failures),
validate affected glossary coverage and republish every changed export before
marking completion. Preserve audit guards and failure reporting. Prefer a
private staging snapshot before promotion; at minimum fail completion visibly
on any newly stale dependent instrument. Add a two-instrument regression where
admitting B's title changes A despite A not belonging to the batch. Also test
alias-catalog changes, idempotence and unexpected closure failures.

### P2 — Expanded whitespace defeats title containment's lookback bound

`main.rs:2537` bounds candidate starts by `index.longest`, measured in normalized
title characters, while scanning the original text's character positions.
`title_end` accepts arbitrary whitespace runs. Those two rules disagree.

Compiled the actual `lower`, `title_end`, `covering_titles` functions extracted
from the reviewed commit into a standalone Rust harness:

- title: `ley de aguas nacionales`;
- text: `Ley` + 80 spaces + `de Aguas Nacionales`;
- usage: the final `Nacionales`;
- `title_end(text, 0, title)` succeeds, but `covering_titles` returns empty.

Thus a usage inside a recognized title is retained solely because the title's
whitespace expands past the raw-character window. This is a reproduced matcher
contract defect; no current corpus misclassification is claimed. A source
parser's usual whitespace collapsing does not establish this helper's promised
behavior for every admitted source.

Repair: bound lookback in normalized whitespace units, or match normalized text
with a mapping back to unchanged canonical character offsets. Add regression
cases for long spaces, repeated line breaks, tabs and a usage near the title's
end; preserve Unicode offsets and the nested-title exception. Do not normalize
canonical source text to conceal the mismatch.

## Policy and remaining boundaries

- `la presente Ley`: retain the recorded current behavior until JRH decides
  whether own-title occurrences should bind to that shorthand. The 9 lmigra and
  4 lgscmm cases are policy questions, not automatic parser repairs.
- The title exception uses normalized `definition.contains(title)` (`main.rs:2587`),
  a lexical occurrence, not an explicit legal referent binding. It can also match
  a prefix of a longer word. Use full-boundary title matching for the mechanical
  part; do not claim this establishes a legally unambiguous referent. Incidental
  mentions and unknown/uncommitted titles remain disclosed coverage limits.
- Heading separators: the narrow numeral/capitalization guards and cpf wrapped
  sentence regression are appropriate for this batch. No new current-source
  heading failure was reproduced. They do not prove support for every heading
  form (for example, lower-case names or higher ordinal word forms).
- LAN's a./b. split is source-scoped and tested; the 11 reviewed LIEPS exclusions
  remain exact-text/offset/term pinned and fail closed when stale. Do not extend
  them to the unreviewed extra `azúcar` case.
- ltfccg still requires retained source matching its committed hash; lgcs's seven
  terms remain legal review. LGS and the specified LIEPS boundary rulings are
  now recorded as operator-confirmed on the parser branch, not inferred here.

## Independent checks

- All 253 removals at `0d7b677ee` are contained within exact printed names, in
  91 instruments; zero additions. Definition body usages are outside this rule.
- Across all seven reparsed instruments, IDs and every provision field other
  than text/heading context are unchanged against `db48f899d`. No reference
  file differs. This supports preservation of stored temporal/review fields;
  it is not a new legal ruling on removed heading/signature apparatus.
- Standalone production-function whitespace reproducer passes its assertions.
- `cargo fmt --check`, strict workspace Clippy, and
  `cargo test --locked --workspace --quiet` pass: **374 tests**, zero failures.
  All builds used isolated `/private/tmp/lex-parser-review-target`.
- No CLI `validate`, `link`, `parse` or `export` was run against shared canonical
  data. Those commands may write; the active Claude branch was kept read-only.
  The 374 parser-branch tests and DB branch's 399 tests are different suites;
  the integrated suite must be rerun after the normal merges.

## Pinned integration census

At `cf8eed626`: 5,728 terms, 99,243 usages, 35,949 references and strategy 5
across all 468 instruments. The cross-instrument-usage golden changes from
**2,581 to 2,545** (36 fewer); LRITF `autoridad-financiera` remains **88**.
These are source census observations, not edited DB expectations. Preserve
before/after evidence when updating the integrated fixtures. See the
[review evidence](parser-batch-review-2026-10-20-evidence.json) for source hashes
and the exact synthetic whitespace input.

## Integration sequence

1. Commit the accepted M4 report/evidence and normally merge the DB branch first,
   preserving `6505baf57` ancestry, in the designated integration session.
2. Address the two technical findings on the parser branch with regression
   fixtures; keep policy questions explicit. Normally merge the parser batch
   afterward, without rebasing or rewriting either branch.
3. Bind adversarial/profile fixtures to strategy 5 and deliberately reconcile
   golden expectations from committed source evidence. Rerun every golden query,
   not just the cross-instrument usage count. Keep the old M4 artifact immutable.
4. Repeat native checks and two fresh M4-style builds/verifications against the
   integrated permanent commit. Require empty stale inputs and preserve all
   accepted debt, audit decisions, source manifests and review queues.

The reviewed M4 release `9c444091…` remains correct for `fedcbc5d`; it does not
represent the pending parser corpus. Do not update its hashes or files in place.
