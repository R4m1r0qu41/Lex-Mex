# Parser responsibility split — behavior-preserving execution

## Purpose and authority

Prepared 2026-10-23 for Claude Sonnet 5.5 as executor, Opus 5.5 as advisor,
and a subsequent GPT-6.1-Sol (low) independent audit. The operator reports
that the reader is deployed and undergoing tweaks. This task restructures
Rust responsibilities; it does not change ingestion results or legal facts.
The operator's goal-setting instruction activates execution of this plan;
this document alone does not authorize commits, worktree creation, or deployment.

Observable outcome: four responsibility crates replace the overloaded
lex-parse surface, with reproducible evidence of unchanged parsing, linking,
validation, temporal derivation, review application, exports and DB content.
Rules become discoverable without discarding regression coverage. Completion
means ready for independent audit, not automatically accepted or merged.

## Immutable baseline and current checkpoint

Planning baseline: main `85c41f3898d86737766564fa8d3fd8f6d4ae74a3`.
Six current workspace crates: core, source, parse, export, cli, db.
Reader-four was `b9b8be012` and reader-five `162128517` at inspection; neither
is the refactor baseline. Do not merge their evolving UI work into this task.
Main has 14 pre-existing untracked files; preserve all of them.
Accepted DB reference: permanent-5eca release
`afdc5acf0b5017d73cd7b8e0b8f8fc5962914b56de26467e42b0c77fd2ef1974`,
corpus/builder `5eca03bc5085de423ae23b1f1519712604bd28e2`.
Its census is 468 instruments, 68,341 provisions, 35,930 edges, 5,766 terms,
97,672 usages, 14 active debt rows and 3 retirement records. Counts are useful
checks, not substitutes for row/content parity. No new census was run for this plan.

2026-10-23 checkpoint: plan drafted; no refactor or runtime validation executed.
Exactly one next action: Sonnet performs G0 in an operator-provided isolated
worktree, under the goal prompt below. If HEAD differs, record a bounded diff
and a separate execution baseline; never silently rewrite this initial baseline.

## Sources and scope

Read AGENTS.md, applicable provider instructions, PLANS.md, this plan, then:
- `crates/lex-parse/src/{lib,labels,terms,temporal_derive,standard}.rs` and
  relevant source-family modules, using bounded symbol searches;
- all six Cargo manifests, workspace Cargo.toml/Cargo.lock;
- `crates/lex-cli/src/` callers and `crates/lex-db/{build.rs,src/build.rs,src/build_tests.rs,src/verify.rs}`;
- `docs/heading-leak-permanent-acceptance-2026-10-21.md` and the historical
  `docs/plans/workspace-db-parser-consolidation-audit.md` for rationale;
- targeted accepted decisions and fixtures used by a moved rule.

Allowed changes: workspace manifests/lock, Rust source and tests in existing
crates and the three new crates below, directly used parity tooling/fixtures,
this plan, rule inventory, receipt/evidence and relevant plan-index entry.
No corpus, source manifest, schema, debt seed, accepted decision, PDF/extracted
source, legal approval, public package, reader, host or provider configuration
changes. No network refetch, source regeneration, DB migration or new legal
inference. Leave acquisition commands where they are for this refactor.
Missing retained inputs are blockers for the affected coverage, not permission
to download replacements or claim complete parity from fixtures alone.

## Target ownership and dependency contract

| Owner | Contents | Permitted project dependencies |
| --- | --- | --- |
| lex-grammar (new) | labels, ordinals, date syntax, exact character spans and genuinely shared lexical helpers | lex-core only if required |
| lex-analysis (new) | validation, initial repeal/temporal evidence classification, temporal derivation and explicit audited-review application in separate modules | lex-core, lex-grammar |
| lex-link (new) | citation extraction/resolution and term-usage linking, supplied records/terms/context explicitly | lex-core, lex-grammar |
| lex-parse (retained) | source-family partitioning/parsing, furniture, glossary extraction, current extraction commands | lex-core, lex-grammar, lex-analysis for initial evidence; lex-link only for compatibility facade |

No analysis/link dependency on parse, CLI, DB or export; no grammar dependency
on analysis/link/parse. lex-core stays the data model. Keep legal classifiers
out of grammar. Existing parse-time calls to initial_temporal_status and
initial_repeals delegate to analysis with identical timing and inputs; do not
move them to a later stage and change initialization behavior. Analysis takes
explicit records and configuration, not parser-specific options.

Preserve the existing public lex_parse API through thin re-exports/wrappers
for reader branches and external callers, including module paths and public
types. First-party CLI/DB callers should use the responsible crates directly.
Compatibility facade may depend on link/analysis, but contains no copied
business logic. Do not deprecate with warnings that fail downstream strict
Clippy during this change. Record deliberate remaining facade dependencies.

Split terms.rs by behavior: glossary extraction stays in parse; usage matching
moves to link. Any shared string matcher goes to grammar only if truly lexical.
Move standard validation to analysis where independent of document parsing;
retain parse-specific recognition checks in parse. Share typed inputs in core
only when actually necessary and without serialization/schema changes.
Cross-stage tests belong at CLI integration level when required to avoid
reverse dev-dependency cycles. Do not move production code solely for tests.

Opus must review the actual symbol/dependency map at G1, especially these
boundaries, and the provenance/parity design. Sonnet owns implementation.
Opus advice cannot authorize legal changes, waive failed gates or expand scope.
If a specified model is unavailable, report it rather than silently substitute.

## Gate protocol and artifacts

Each gate records commands, exit codes, tool versions, elapsed time, exact
HEAD/source digest, input/output hashes, coverage counts and failures in
`docs/parser-split-implementation-evidence.json`; narrative in
`docs/parser-split-implementation-receipt.md`. Large logs and snapshots go in
fresh ignored `.work/parser-split/` or `derived/db/parser-split-*` directories,
with hashes and reproduction commands in the tracked evidence. Never overwrite
an existing release. Use one writer and a dedicated CARGO_TARGET_DIR.

Status per gate is PASS, FAIL or BLOCKED, never inferred from a previous agent's
summary. No skipped test may count as PASS. Run costly checks once per required
boundary, not repeatedly when unchanged. A failure gets a bounded diagnosis;
a behavior change goes into a separate deferred defect report, not this patch.

### G0 — isolation, reproducible baseline and replay coverage

1. Verify instructions, HEAD, status, worktree ownership and local-commit authority.
   Use an operator-created dedicated worktree, not reader-four/five or main.
   Record B = full execution-baseline SHA, toolchain and extraction tool versions.
2. Inventory all exported parse symbols and first-party callers. Record source
   families and available retained extracted-text/source hashes programmatically;
   do not load the corpus wholesale into model context.
3. Establish baseline native checks using the commands below. Run mutating
   validators only in a disposable validation checkout, and inspect their diff.
4. Implement a deterministic A/B runner before moving code, preferably as an
   integration harness or existing tooling extension. Freeze identical inputs,
   options, catalogue, exception profiles and accepted review records. Replay
   every available retained-source instrument supported by current pipelines,
   with a coverage manifest listing each included or unavailable instrument and
   exact reason. Also run all existing fixtures, including held/ambiguous inputs.
5. Compare baseline against itself twice; all deterministic outputs must match.
   Capture typed parsed records, labels/IDs/order/text, source spans, glossary
   terms/holds, references/targets/qualifiers/rules, usages, validation issues,
   temporal proposals/status, review queues and exported Markdown. Preserve
   vector order; only sort object keys for JSON canonicalization. Never mask
   source text, spans, statuses, issue messages or rule IDs. Any unavoidable
   timestamp/path normalization is an explicit field allowlist, justified and
   reviewed before candidate runs. No broad regex scrubber.
6. Capture the entire committed corpus tree hash; exercise relink/validation/
   derivation/export over all committed instruments in disposable copies on both
   sides, independently of retained-source availability. Compare stage outputs
   to baseline stage outputs, not an assumption that old persisted output is fresh.

PASS: native baseline green, self-replay exact, coverage manifest complete and
Opus accepts that all changed production paths are exercised. Unavailable
retained inputs must be resolved or explicitly returned as a coverage blocker;
do not call reduced coverage full acceptance. Pre-existing failures stop this
task for a report; do not silently fix them during extraction.

### G1 — rule inventory and modules within lex-parse

Create `docs/parser-rule-inventory.md`: rule ID/name, source family, current
symbol, future owner, preconditions/state, source-span effect, hold reason,
positive fixture, nearest negative/ambiguous fixture, accepted-decision link
where relevant. Cover all moved production rule groups, not every test assertion.
Keep source-law ownership, nested decree/law selection, annex/transitory bounds,
page furniture, numbering/ordinals, glossary delimiter/quoted/nested cases,
citation context/ranges/qualifiers and repeal/temporal boundaries explicit.

Move root responsibilities into modules first; no predicate rewrites or rule
broadening. Group identical fixture setups only where assertions remain equally
strong; preserve named silent-loss/legal-boundary regressions. Do not impose
arbitrary file-length targets. Inventory unmapped rules as a failure to finish G1.

PASS: Opus records approval of dependency/symbol map and comparator allowlist;
format/Clippy/workspace tests and full A/B pass with zero unexplained drift.

### G2 — extract grammar and analysis

Move grammar first, then analysis, preserving compatibility paths and the
parse-time initial classification contract. Update manifests and source
provenance with each new crate as soon as it becomes a dependency. No empty
scaffolding crate. Run format/Clippy/workspace tests after each boundary;
run complete A/B after both. Ensure validation and audited-review application
preserve ordering and exact decisions, including rejected/unresolved records.

PASS: acyclic production and dev-dependency graph, public API compatibility
compiles, source-family and analysis tests retain coverage, A/B exact.

### G3 — extract linking, migrate consumers and pin every source

Move citations and term usages; migrate CLI/DB imports, leaving thin compatible
lex_parse paths. Verify all old public symbols with a compile-only downstream
fixture using the G0 inventory (not just current internal call sites).

Update BOTH build-time inventory/triggers (`lex-db/build.rs`) and runtime
builder/verification expectations and test fixture setup. Pin all new crate
Rust sources and manifests, workspace manifests/lock and any non-Rust rule
assets actually consumed. Track directories so adding a new file triggers
rebuild as well as editing an existing file. Review hard-coded parse paths in
`src/build.rs`, `src/build_tests.rs`, `src/verify.rs`; do not blindly replace them.

Negative provenance tests must prove rejection of a claimed builder whose
compiled inputs differ: changed source in EACH new crate, changed manifest,
and a newly added consumed source/rule asset. Assert the mismatch path/reason;
include an unchanged control that succeeds. Test no-rebuild stale-binary
mismatch separately from Cargo rebuild-trigger behavior. No test may disable
or mock away the provenance check it claims to verify.

PASS: native checks, API compatibility, exact A/B, dependency graph audit,
positive/negative provenance tests. Opus reviews remaining coupling and
provenance coverage before final acceptance builds.

### G4 — final corpus and database acceptance

Run in disposable validation checkout at final candidate C:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run --locked -p lex-cli -- validate lritf
cargo run --locked -p lex-cli -- validate ifpe-dcg-2021
```

Also run the G0 whole-corpus stage suite and retained-source A/B using frozen B
and C binaries, with the same frozen inputs and toolchain. Inspect source
manifest, validation report, canonical diff, review queue and Markdown for each
source family plus all differing records (expected: none). Corpus Git tree,
schemas, seeds and accepted decisions must remain byte-identical to B.

DB builds require a committed candidate source identity; the goal prompt
explicitly authorizes scoped local milestone commits. Final receipts may use
a later docs-only commit while stating exact tested code C. Never claim the
docs HEAD executed if it did not. Before each code commit run required checks;
validators use disposable copies, never the authoritative corpus.

Use corpus_commit=B for baseline and both candidate builds, builder_commit=B
for baseline and builder_commit=C for candidate. Build from the corresponding
source checkout/binary. Commands (substitute recorded full hashes and actual
manifest/database paths returned by project; do not guess filenames):

```sh
cargo run --locked -p lex-cli -- db project --corpus-commit "$B" --builder-commit "$C" --output-dir derived/db/parser-split-candidate-a
cargo run --locked -p lex-cli -- db project --corpus-commit "$B" --builder-commit "$C" --output-dir derived/db/parser-split-candidate-b
cargo run --locked -p lex-cli -- db verify --database "$DATABASE" --manifest "$MANIFEST" --corpus-commit "$B"
cargo run --locked -p lex-cli -- db golden-queries --database "$DATABASE"
```

Build the B/B baseline once in a fresh ignored baseline output root; an existing
artifact is usable only if identity and independent verification match B/B.
Independently verify baseline and both C artifacts and run all current golden
queries (21 at planning baseline) on each. Require integrity/FK/source parity,
no unexpected stale inputs, full census/debt history equality and identical
logical corpus digest between B/B and B/C. Compare complete logical rows,
including review/temporal provenance, not counts alone. Explicit builder/source
provenance metadata may differ between B and C and must be enumerated. Physical
DB bytes and release identity can differ B→C; repeated C builds must have
identical physical DB SHA, logical digest and deterministic manifest identity.
Any other difference fails. Do not edit goldens/debt expectations to obtain PASS.

Reader UI is outside scope. Compile old API paths; record the future reader
integration check (exporter build and package verification on the integration
commit) as an integration requirement, not a reason to import UI changes or
rerun its full mutation campaign here.

### G5 — handoff ready for independent audit

Sonnet reviews the final diff and asks Opus for a bounded read-only final pass:
rule inventory completeness, graph boundaries, review preservation, comparator
blind spots and builder pinning. Fix within-scope findings and repeat affected
gates; unresolved issues remain FAIL/BLOCKED, not an advisor waiver.

Deliver plan checkpoint, inventory, receipt, machine evidence, exact B/C and
commit list, API mapping, reproducible commands, hash-bound coverage manifest,
artifact paths, native results, A/B zero-diff summaries, DB verification/goldens,
advisor findings/dispositions and deferred defects. Include expected logical
provenance differences explicitly and preserved unrelated status.

Prepare a fresh commit-bound pending/codex handoff. Requested reviewer:
GPT-6.1-Sol (low). It verifies evidence and independently samples the highest-risk
boundaries; any material ambiguity is reported for a separate higher-effort
review, never rubber-stamped to preserve the requested tier. No automatic
merge, push, deployment, reader changes or deletion. Finish with one next action:
operator starts Codex audit of C and the packet.

## Recovery and stop conditions

Keep B and all baseline evidence. On a failed gate, stop dependent stages,
report exact failing case and minimal reproducer, and preserve working changes.
Repair only mechanical move errors within this task. No broad reset/clean,
forced worktree deletion, test weakening, corpus rewriting, review re-approval,
new exception profile, source refetch or automatic golden updates. Recover by
an explicit corrective commit or isolated checkout of the last passing stage;
never rewind another writer. Missing worktree, retained evidence, commit
permission or model availability gets a precise blocker and resumable receipt.

## Goal prompt for the operator

> Execute docs/plans/parser-responsibility-split.md through G5 in the dedicated
> operator-created parser worktree. Use Claude Sonnet 5.5 as executor and Opus
> 5.5 as bounded advisor at G1, G3 and final review. I authorize scoped local
> milestone commits after required checks, but no merge, push, deployment,
> corpus/legal changes or deletion. Continue across gates without routine
> confirmation; stop only on the plan's substantive blockers. Preserve reader
> work and all unrelated state. Finish with reproducible evidence and a fresh
> commit-bound Codex handoff for GPT-6.1-Sol (low). A goal budget ending is not
> acceptance: checkpoint the last passing gate and remaining work.

## Outcome / retrospective

Execution status (updated by the executor): G0–G4 executed on branch
`refactor/parser-split`; G5 handoff is the last step. The living record is
`docs/parser-split-implementation-receipt.md` with machine evidence in
`docs/parser-split-implementation-evidence.json`. Nothing is merged, pushed or
deployed. The sentence that follows is the original planning-time statement:
planning only, gates NOT RUN. No runtime behavior, corpus, database or
reader changed at planning time. The historical consolidation audit remains historical; this
file is the living execution plan for the parser split.
