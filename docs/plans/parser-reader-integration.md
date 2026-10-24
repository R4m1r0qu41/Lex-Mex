# Integrate the parser split and sync the published reader

## Purpose, authority and scope

2026-10-24 recommendation: integrate the accepted parser into main, then sync
that main into the published reader branch. Keep R7's unpublished UI separate.
The operator may choose a broader scope; this plan's default is not authorization
to merge, push, publish or move another writer's branch. A goal prompt below
explicitly authorizes local candidate work only. Canonical instructions, code,
accepted decisions and Git state outrank this plan.

Outcome: two reviewable normal-merge candidates, exact ancestry and test/package
receipts, then an operator-authorized promotion. No corpus regeneration, source
refetch, legal review change, database migration, UI redesign or deployment.
Do not replay the completed multi-hour parser queue for tooling-only changes.

## Immutable baseline and current checkpoint

- Main B: `85c41f3898d86737766564fa8d3fd8f6d4ae74a3`.
- Parser candidate: `11a3a90952aa05beeb9db62be095c6b54da6bbad`.
- Tested parser production C: `1250889ce10b8ba910a5f5ca5f232fd5752e79df`.
  Later parser commits change only docs/prompts.
- Published reader source: `b9b8be012c0a19b50e75acd922c38a19385faded`, branch
  `feat/four-instrument-reader-design`, permanent reader-four workspace.
- Deferred R7: `162128517576b043a389f753578c5cabb4540898`, branch
  `feat/reader-tweaks-r7`, permanent reader-five workspace. R7 has no Rust/Cargo
  changes relative to b9b8, but does change UI and tests. Do not import it here.
- Local publication pin: release `9626bfe45c4ebc95b47351fd20c1de86f9b4f54856aee1de6d408e5d8389991f`,
  publication `2df70a8ab0f43571ed6ca1500ff4637e4de252ee1edc190befeda8b938b8f1d4`.
  Pin and UI directories were checked locally, not against the remote site.

Current checkpoint: planning only; no integration candidate has been created by
Codex. The earlier disposable R7/parser merge `f8909420…` supports exporter
compatibility but is not either final integration commit below. Baseline native,
A/B and DB evidence remains bound to its original source commits.

Exactly one next action: operator activates the local-candidate goal below with
this default scope or supplies another scope. Record a bounded diff and stop
for scope reassessment if any named source branch has advanced or gained a writer.
Do not silently rebase the baseline or include fresh UI work.

## Sources and working conditions

Read applicable AGENTS/provider instructions, PLANS.md, this plan, accepted
parser audits and the latest A-002 closure report, parser implementation
receipt/evidence and published-reader receipt/README. Use targeted source reads.
Keep the corpus out of model context; comparisons operate programmatically.

Sonnet 5.5 executor, Opus 5.5 advisor for nontrivial merge conflicts only;
subsequent Codex audit of exact candidate commits. No provider impersonation.
Use the clean operator-created parser-split worktree for fresh integration
branches if its writer has handed it over. Preserve `refactor/parser-split` at
its reviewed tip. Do not switch or alter the active reader-four/five worktrees.
Otherwise request an operator-created isolated integration worktree.
Use disposable clones and distinct CARGO_TARGET_DIR values for mutating checks.
Do not invoke old G4 wrappers, write into gate-output roots, or overwrite releases.

## I0 — inventory and preservation

Record current branch tips, clean/dirty state, writer ownership, local main/origin
state without assuming a stale remote ref is current. No network push/fetch is
needed to prepare candidates. Bind report and machine evidence under
`docs/parser-reader-integration-{receipt,evidence}.md/json` (use `.json` for the
machine evidence). Hash all task-relevant local drafts before promotion.

Main currently has unrelated files and uncommitted Codex review reports. Do not
stash, reset, clean, bulk-stage or discard them. Parser promotion has one known
untracked collision: `docs/plans/parser-responsibility-split.md`, whose local
planning copy differs from the branch's progressed plan. Reader promotion/scope
expansion has eight known untracked overlaps: seven identical review/evidence
files and the differing `docs/plans/four-instrument-reader-design-release.md`.
Recompute this inventory at promotion time; do not rely on the historical count.

Candidate preparation does not require moving these main files. Before any
later authorized promotion, preserve every colliding local file in a fresh
ignored `.work/integration-backups/<unique>/` directory with exact path/hash
manifest, verify backup bytes, then move only the colliding untracked paths out
of the way. Reconcile the old plans as historical planning copies; incoming
progressed plans are the execution truth. Never restore old draft bytes over a
progressed plan. Noncolliding audit reports and PLANS.md edits remain intact.
Stage/commit reviewed Codex documentation only with explicit path-level scope
and local-commit authorization; preserve the three operator-owned suggestion/
prompt files. No worktree/branch deletion or derived-artifact cleanup.

PASS: exact source identities and preservation inventory recorded; one writer.

## I1 — parser-to-main candidate

On a new local branch such as `integration/parser-main-2026-10-24` at B,
normal-merge the reviewed parser tip with `--no-ff`. Record P, both parents and
conflict resolution. Do not move `main` yet. Never squash/rebase: preserve C,
6505baf57 and the corpus/debt-seed provenance ancestry.

Require empty diff B..P for corpus, schemas, adapters, accepted decisions and
existing fixture bytes; any new implementation-test fixture must be explained,
not a changed golden expectation. Parser production sources in P must equal C.
Inspect the diff rather than inferring equality from branch names.

In a disposable validation clone at P, run:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run --locked -p lex-cli -- validate lritf
cargo run --locked -p lex-cli -- validate ifpe-dcg-2021
```

Inspect post-check diff; no unreviewed corpus/queue/export changes. Run the new
bounded tooling self-tests in isolated scratch, keeping gate harness and the
54 original/recovery-bound artifacts immutable. The historical log incident
must remain explicitly accounted for: 51 original hashes plus three replacement
native-log hashes. Do not overwrite evidence to make hashes match.

Using the P binary, independently verify an existing accepted candidate DB and
run all 21 goldens read-only. Record manifest/database hashes and verify receipt;
no projection rebuild is needed merely because P has a new Git identity. The
existing DB still truthfully names builder C. A new P-pinned DB is a separate
release if the operator later requests it.

PASS: full native gate, clean canonical diff, provenance ancestry and DB checks.
Stop on drift; never change parser rules/goldens/legal facts in this integration.

## I2 — sync-main-into-published-reader candidate

Create another local branch such as `integration/published-reader-parser-2026-10-24`
at b9b8; normal-merge P to create R. Do not move the permanent reader branch.
Record the source parents. The expected Cargo.lock conflict is the CLI dependency
list: retain both `jsonschema` and the split crates (`lex-analysis`, etc.) in
Cargo's generated order, and validate with `cargo build --locked`. Inspect any
additional conflicts with the advisor; do not blindly prefer a side. Rust,
manifests and facade changes must compose without altering parser behavior.

In a separate disposable validation clone at R, run the five native commands
above and `cargo build --locked -p lex-cli`, plus `node --test reader/` with local
loopback permission if the sandbox blocks preview binding. Use the selected
reader base's test inventory; do not import R7 tests to obtain an arbitrary count.
Require byte-identical b9b8 UI source files and unchanged core parser sources.

Exporter/package parity:

1. Build b9b8-only and merged-R binaries from their exact committed sources with
   separate targets. Both must build locked.
2. From one fixed disposable corpus root, with the same retained source-store
   hashes, build four-instrument public releases twice per binary into fresh
   output paths. Run the package verifier/packager for each with b9b8 UI bytes.
3. All four release trees and all four package trees must be byte-identical.
   Compare paths and actual file hashes, not counts alone. Freeze input files,
   tool versions and source-store identity; no source download.
4. Against the local accepted release, allow only the demonstrated commit-derived
   manifest fields (release_id, corpus_commit, committed_at), plus references to
   that new release ID in package descriptors/latest/llms. Every listed content
   file/hash and all five UI assets must match. Validate IDs, manifests and PDF
   bytes/range preview through the existing package/Node gates.

No browser/mutation campaign is required for identical UI/content bytes; if any
UI/content byte changes unexpectedly, stop and report rather than broadening
scope or treating it as metadata. No host staging, publication or deployment.

PASS: full native and locked build, Node/package/source parity and clean snapshot.

## I3 — integration handoff and later promotion

Deliver P and R full SHAs and parent lists, conflict patch, exact command/exit
records, native/test inventory, unchanged corpus/schema/decision hashes,
read-only DB verify/goldens, public release/package maps, local-source UI match,
source hashes, preserved local-state inventory and unresolved issues. Record
that R7 remains unmerged and unpublished by this task. Publish nothing externally.

Prepare a fresh commit-bound pending/codex handoff for review of P/R. Operator
then authorizes promotion explicitly. Recommended promotion order: main to P,
then published reader branch to R, preserving normal merge commits. Recheck tips,
writer ownership and untracked collisions immediately before any branch move.
Use a fast-forward to the already reviewed normal-merge candidate where possible;
if a tip advanced, stop for a fresh integration review. Do not reset or force-move.
Remote push and deployment require separate explicit authorization.

Recovery: preserve branch tips, logs and backups. On failure retain candidates
and exact reproducer, and return to the last passing stage in a separate branch
or clone; no destructive rollback or mutation of accepted releases.

## Goal prompt

> Prepare I0–I3 of docs/plans/parser-reader-integration.md with Sonnet 5.5 as
> executor and Opus 5.5 as conflict advisor. Scope: parser-to-main candidate,
> then sync that candidate into the published b9b8 reader; leave R7 separate.
> I authorize scoped local candidate branches/normal-merge commits and review
> documentation commits in the handed-over operator-created parser worktree.
> Preserve actual main and reader branch tips and all unrelated work. Run the
> exact gates and produce one commit-bound Codex handoff. Do not promote the
> real branches, push, deploy, refetch, modify corpus/legal facts, delete work,
> or repeat the full parser A/B queue. Stop only on substantive drift or failed
> gates, and checkpoint the last passing stage if the goal budget ends.

## Outcome

Planning only. I0–I3 have not run; no branch or deployment changed.
