# Integrated main review — 2026-10-20

Candidate: `e718da2e5ee463446bfe5987ce9a352913d8280b` on main.
Incoming packet: `2026-10-20-lex-mex-integrated-main-review.md`.
This review is read-only against main; report/evidence live in the DB worktree.

## Verdict

**Accept `e718da2e5ee463446bfe5987ce9a352913d8280b` for the operator-authorized
Claude push of main.** No blocking finding remains in the reviewed integration.
Approval is for this exact commit; it does not approve later source changes.

## Code and integration review

The three normal merges follow the requested DB → batch → rulings order.
The original seed source `6505baf57` remains in history. No conflict-resolution
changes were introduced in the reviewed parser fixes; DB CLI wiring is the
expected addition to the rulings branch.

- Accepted-gap publication: a separate `glossary_refreshed` outcome is collected
  from the actual write path and republished to the requested vault. It does not
  claim ordinary validation passed for the known reference gaps. The corpus
  closure retains exact accepted-gap comparison and glossary preflight. New
  tests cover external export, identical references, idempotence and reread
  aliases. Previous P1 is closed for this candidate.
- Partial-repeal provenance: only untouched partial repeals with nonempty scope
  exactly matching freshly parsed opening-note evidence are promoted. No date
  is invented for remaining text; reviewed provisions are excluded. Scope
  mismatch, human review and idempotence are tested. Article 16's temporal
  fields equal their pre-reparse values at `7ae8329ab`, including null dates.
  Previous P2 is closed.
- Institution names: source-bound glossary definitions supply the full-name
  boundary list. Exact-name matching uses the corrected whitespace/word-boundary
  machinery and preserves the definition-names-referent exception. The operator
  D1/D2/D3 answers and standing context instruction are recorded; no fresh legal
  ruling is inferred by this review. Imports and eleven exclusions are explicit
  configuration, and the ltfccg IX profile is narrow.
- Quoted sense additions, printed-name suppression and heading repairs remain
  the previously reviewed implementations, now integrated with DB code. Human
  review/audit guards and accepted gaps are preserved.

Expectation deltas are deliberate: adversarial strategy 4→5→6; cross-instrument
usage golden 2,581→2,545→2,469; synthetic build coverage binds the core strategy
constant. No schema was weakened. All other golden expectations remain fixed.
The final **2,469** is independently confirmed by a relational join, not inferred
from the fixture value. The integrated source census must not be confused with
historical M4 release `9c444091…`.

## Validation

Codex independently ran fmt, strict workspace Clippy, all **412 tests** and a
fresh CLI build in `/private/tmp/lex-integrated-review-target`: all pass.
Required LRITF and IFPE validators pass with zero issues in a private local
clone pinned to the candidate; counts are 145/11/126 and 59/4/115 respectively.
The seed check validates all 17 accepted debt rows.

The separately compiled verifier passes on artifact A: **208,888 records**,
**4,960 raw inputs**, **9,997,279 FTS positions**, empty stale inputs. Both
artifacts pass all **21 goldens**. Codex independently checked both database
and manifest hashes: identical and matching the upstream receipt. The original
M4 artifact hash remains unchanged. Native gates and these checks did not write
shared canonical files.

The upstream packet reports the two fresh builds, full verification of B and
corpus-wide stage checks. Codex did not rebuild both databases or rerun those
write-capable corpus stages; it independently reconstructed A against pinned
Git and verified B is byte-identical. No broader reproducibility claim follows.
See [evidence](integrated-main-review-2026-10-20-evidence.json) for full hashes,
merge parents, counts, private validator results and scope.

Supplementary read-only SQL confirms 5,766 definitions, 2,469 cross-instrument
usages, and pipeline coverage: 468 acquired, 468 structured, 463 linked with
stored validated graphs plus 5 accepted gaps, all 468 analyzed `not_assessed`.
The latter is explicit unknown analysis coverage, not a claim analysis completed.

## Scope and push condition

Operator authorized Claude to push main only after Codex reviews the integrated
result. Codex performs no push in this turn. Approval, when recorded, binds to
this exact main commit and its reviewed artifacts; a changed source commit needs
review before push. Push main only, using a normal fast-forward push; preserve
the other branches, local draft reports and three operator-untracked files.
No history rewrite, tag push or artifact modification is authorized by this
review. Preserve historical release files and use new release directories.
