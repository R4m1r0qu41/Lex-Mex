# Heading-leak parser and debt repair

2026-10-21. Source branch parser/heading-leak-refetch-2026-10-21 at
9a2df2529, based on reviewed/pushed e718da2e5. Operator requests review and
technical parser repairs; no commit, merge or push is authorized for this branch.

Scope: review the three parser changes and 39 reparses; bind wrapper/reopened
transitory recognition to the configured source title; retire exactly three
resolved LGS debt rows with verifiable history; restore the full pinned DB gate.
Keep LRITF and unrelated operator files unchanged. New editions remain held.

Retirement design: keep the accepted review ledger and October18 decision bytes
immutable. Seed version3 separates active rows from retirement history. Active
plus retired identities/fields must equal the original accepted set exactly.
Each retirement retains its accepted row and pins an ancestor resolving commit,
source blob/hash and exact standalone target. Verify target identity/ownership
at resolution and current input, and absence at historical seed source. No new
legal decision or unvalidated ledger status is introduced. DB active debt tables
contain only active rows; raw seed input retains retirement evidence.

Milestones:
1. Narrow parser predicates and add false-positive/owner regression fixtures.
2. Update seed schema/types/validator/negative tests and actual 14+3 seed.
3. Independent native checks, source/temporal audit and affected end-to-end parse;
   two fresh pinned candidate DB builds, independent verify and 21 goldens.
4. Repository report and explicit continuity checkpoint; leave changes uncommitted.

Validation: fmt, strict Clippy, workspace tests, locked LRITF/IFPE validators in
private snapshots; read-only source hashes, old/new parser comparisons on named
fixtures and retained sources; review canonical/queue/export diffs. No official
source text edits or model/human decision overwrites. Candidate Git snapshots
may be unreferenced test objects; they are not permanent branch commits.

All four milestones complete: 419 tests; private affected E2E; two byte-identical
DB builds with independent verification and 21/21 goldens each. See
`docs/heading-leak-refetch-review-and-repair-2026-10-21.md` and companion evidence.
Current checkpoint: repair committed as5eca03bc5; permanent-commit builds
accepted with zero stale inputs and21/21 goldens each. See
`docs/heading-leak-permanent-acceptance-2026-10-21.md` and evidence. The earlier
candidate stage above is historical; no further rebuild is needed for its
three stale inputs.
Current next action: operator-authorized documentation commit and reviewed
normal merge preserving cd6cd2245; no merge or push performed here.
