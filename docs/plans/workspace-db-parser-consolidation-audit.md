# Workspace, database and parser consolidation audit

2026-10-21. Initial/current baseline: branch
`parser/heading-leak-refetch-2026-10-21`, `5eca03bc5085de423ae23b1f1519712604bd28e2`.
Operator requests audit and recommendations while Claude completes permanent DB
validation. This document authorizes no deletion, merge, parser rewrite or push.
Scope: registered worktree metadata, bounded status of DB/reader worktrees,
local DB manifests, parser source boundaries and repository plans. No corpus
bulk read. Stop condition: actionable inventory and staged consolidation plan.

## 1. Housekeeping

Seven registered worktrees, sixteen local branches. Local `main` and cached
`origin/main` remain `e718da2e5`; no network fetch performed. Current branch
is three commits ahead of main. Main workspace had only three unrelated
untracked operator files before this audit; preserve all three.

| Worktree / branch | Evidence | Recommendation |
| --- | --- | --- |
| Main / heading-leak branch | Active permanent builds, +3 commits | Keep until validation, reviewed normal merge and push authorization |
| lex-mex-db / db/sqlite-projection | All commits merged into main; two modified plan/index files and six untracked review/handoff files | Reconcile/archive unique drafts into repository truth first; only then remove worktree/branch |
| lex-mex-reader / feat/public-reader | Permanent online presentation workspace for the project and DB; six commits absent from main plus local work | Retain permanently; not a stale-worktree cleanup candidate |
| OCR experiment target / experiment/ocr-foundry-2026-08-24 | One commit absent from main | Retain pending experiment-owner disposition |
| OCR target-glmocr / experiment/glmocr | Commit ancestry fully in main | Cleanup candidate, pending local-status/source/artifact and owner checks |
| OCR target-harness / experiment/perception-harness | Same merged ancestry | Same conditional cleanup |
| OCR target-olmocr / experiment/olmocr | Same merged ancestry | Same conditional cleanup |

Operator clarification (2026-10-21): the reader workspace is permanent and
presents the project and database online. Its linked-worktree form does not
make it disposable. Keep it registered and preserve its local work. A later
change to a standalone checkout is an operational choice, not required for
cleanup or DB acceptance; plan ownership, synchronization and deployment
before any such migration. No conversion is performed here.

Experiment directories were not inspected: ancestry alone cannot establish
cleanliness or disposability. No worktree was removed or pruned.

Other merged branch candidates (no registered worktree):
`fable/diputados-bulk`, `fable/ifpe-dcg-2021`, `feat/intake-ec2-20261008`,
`parser/operator-rulings-2026-10-20`, `parser/pending-batch`.
Check owner/remote references before deleting. Preserve divergent
`backup/4dcf024c3-pre-split` (+1 unique), `backup/split-v1` (+11),
`codex/lritf-reference-links` (+1). Unique counts are relative to local main,
not a semantic assessment of whether changes have been superseded.

DB outputs currently occupy about **2.7 GiB** across six nonempty roots:
`integ-e718-a/b` (~460 MiB each), `heading-repair-a/b` (~461 MiB each),
`permanent-5eca-a/b` (~461–464 MiB while active). `refetch-cd6c-a/b` are empty.
Do not delete active outputs. After permanent acceptance, retain one verified
permanent artifact with manifest and independent receipts; preserve reproducibility
hashes for the second before deleting its redundant bytes. Keep one prior-main
release for rollback until the new main release is accepted. Candidate snapshot
pairs then become removable. Never remove `.work/` sources as DB cleanup.
Document exact paths/bytes and obtain deletion authorization before cleanup;
no blanket `git clean`, forced worktree removal or recursive derived purge.

## 2. Database state and working conditions

M1–M4 implementation is present and was normally merged into main. The new
branch refreshes corpus input and implements seed-v3 retirement history, not
an unfinished database loader. Previous native gate: fmt, strict Clippy,
419 tests, required validators; two candidate builds independently verified,
21 goldens each. These are prior receipts, not tests rerun by this audit.

Both new permanent roots already contain manifests pinned to **5eca03bc5**
for corpus and executing builder, release
`afdc5acf0b5017d73cd7b8e0b8f8fc5962914b56de26467e42b0c77fd2ef1974`.
Their manifests claim database SHA
`5da6e800e4d3b362411c561d9ae67fbfe012abe3164fce30bbb8a21c9e249bf2`
and logical SHA
`8cc3736018bffae94f303d9cbfa378170c9ff0e2842ce708431f08d528b60359`.
The logical digest equals the accepted candidate; physical identity changed
with commit-bound provenance. Observing manifests is not independent acceptance
of the ongoing runs. Builder `verification.json` is not the independent
`db verify` receipt; do not confuse the two.

Census: 468 instruments, 68,341 provisions, 35,930 reference edges, 5,766 terms,
97,672 usages, 14 active debt rows; three retired records remain evidence.
32 standards remain catalogue-only; SCJN registry is evidence-only.
Most temporal state is still not analyzed: DB presence is not legal completion.

`crates/lex-db/src/build.rs::output_directory` deliberately accepts only
worktree-local, Git-ignored `derived/db/...`; rejects traversal and symlink
paths. Immutable release directories are not overwritten. Claude's new paths
are correct; an external-output override is unnecessary. Builds read pinned
Git inputs; compiled builder/dependency bytes must match the claimed builder.
A dirty unrelated file is not a DB source. Never run another agent's mutable
Cargo/build stage concurrently in its shared target directory; use a dedicated
linked worktree and target directory for independent implementation.

Acceptance remaining with Claude: independent verify on both permanent files,
empty stale-input paths at the permanent commit, all 21 goldens each,
physical/logical/manifest equality, integrity/source parity and recorded
commands/environment. Then review normal merge; a new release pinned to the
merge commit is a distinct provenance identity if built. No serving/API work
or parser refactor is required to accept this DB milestone.

Documentation debt: `PLANS.md` still describes pre-M4 state and some old
"not built" work. The DB worktree has newer uncommitted plan/index drafts.
Reconcile these after Claude's receipt; do not overwrite them with another
parallel rewrite. The prior heading-repair report correctly describes its
historical candidate snapshot but is not a current permanent-build receipt.

## 3. Parser: rules and four-crate proposal

`lex-parse` has 21,714 Rust lines including tests; roughly 11,600 precede the
main test modules. `lib.rs` alone is 5,668 lines; Diputados 4,626; terms 3,938.
Tests account for much of that size. Appended regression fixtures are valuable
legal-source protection; file length and test count do not establish bad rules.
The actual coupling is extraction + citations + validation + derivation sharing
one crate and root helpers. CLI and DB both consume this surface.

Recommend **four total responsibility crates**, preserving `lex-core` as the
existing trusted data model and `lex-source` as acquisition:

| Crate | Responsibility | Dependency direction |
| --- | --- | --- |
| lex-grammar (new, only when moved code uses it) | Article/ordinal/qualifier labels, Spanish date syntax, exact span helpers | Core or standalone; no parser/DB dependency |
| lex-parse (retained, reduced) | Extracted document partitioning, source-family parsers, furniture, glossaries | Core + grammar |
| lex-link (new) | Citation extraction, target resolution, glossary usage linking | Core + grammar; explicit corpus/term inputs |
| lex-analysis (new) | Separate validation and deterministic temporal-derivation modules; audited decision application | Core + grammar; explicit parsed records |

Naming is a proposal, not a decision already implemented. Parser initialization
of temporal/repeal facts needs an explicit shared evidence classifier or a
post-parse derivation step; do not introduce parse↔analysis cycles or casually
move legal semantics into string grammar. Pure document validation rules may
need shared grammar predicates. Move integration tests that exercise multiple
stages into CLI integration fixtures, rather than reversing dependencies just
for tests. PDF command invocation can later move into the existing source
crate after provenance/replay parity; it does not require a fifth new crate.

First consolidate **rules within modules**, then extract crates. A crate split
alone would redistribute the same conditionals. Build a rule inventory linking
source family, preconditions, owner/state, exact span effect, negative cases,
hold reason and regression fixtures. Existing deterministic invariants include:
configured law ownership, document-state transitions, annex boundaries,
source-backed label equivalence and explicit exception profiles.

For every learned rule require: positive official-source fixture; nearest
negative/ambiguous example; unchanged canonical spans/IDs outside its declared
scope; explicit fail-closed hold when recognition is uncertain. Profiles must
be narrow provenance-bound data, not executable regex escape hatches. Do not
learn by statistical inference or silently generalize from one instrument.
Group test cases by invariant and use tables where cases exercise the same
rule. Keep named regressions for silent loss and reviewed legal boundaries.

## Milestones, recovery and next action

1. **DB acceptance first (Claude owner).** Capture permanent receipts and reconcile
   plan/index. Stop if identity, source parity or stale checks fail. Existing
   releases remain rollback artifacts.
2. **Housekeeping inventory disposition.** Preserve DB drafts, reader work and
   experiment evidence; obtain owner/deletion authority for exact stale paths.
   No automatic removal based only on merged ancestry.
3. **Rule inventory and module split.** Baseline current retained-source replay
   hashes, corpus IDs/text/reference spans, queues and exports. Separate root
   helpers/citations/validation without behavior changes. No source refresh.
4. **Extract four crates incrementally.** One dependency boundary per reviewed
   commit; existing public compatibility re-exports can bridge callers temporarily.
   Update workspace, CLI, DB and provenance inventory together. In particular
   `crates/lex-db/build.rs` explicitly enumerates source directories: every new
   crate's Rust sources and manifest must enter builder pinning and rebuild
   triggers, with a negative test for mismatched compiled dependency bytes.
5. **Acceptance.** Native fmt/strict Clippy/workspace tests, required validators,
   retained-source A/B with zero unexplained changes, canonical/review/export
   parity and full DB independent verification/goldens. Mechanical moves must
   not change logical corpus digest. Physical DB bytes/release ID may change
   because the builder identity changed. Any semantic improvement is a separate
   source-backed change after the move, never hidden inside refactoring.

Recovery: revert the isolated refactor commit on its dedicated branch; never
regenerate or overwrite the admitted corpus to make a refactor pass. Stop on
unclassified replay drift, lost spans, new held cases or changed reviewed facts.

Current checkpoint: permanent-5eca receipts accepted; see
`docs/heading-leak-permanent-acceptance-2026-10-21.md`.
Exactly one current next action: commit the accepted documentation and push
the current heading-leak branch under the operator's explicit authorization. This audit is complete; consolidation and
cleanup remain proposals. No code or corpus changed, no tests rerun, no commit
or push made by this audit.
