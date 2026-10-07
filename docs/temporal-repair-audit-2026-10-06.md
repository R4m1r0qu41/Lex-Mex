# Temporal repair audit — 2026-10-06

## Scope and baseline

This report records the repair of four P1 findings in the supplied repository
review, plus a related current-evidence gap found during implementation.
Baseline: `8031ea11eb1d4ad88c046ba8b6ba1ed8851eb873` on `main`.
The user authorized the local repair, later split into staged commits: model and
review evidence boundaries, the date-rule change, and the corpus date migration,
followed by one commit per document. Their identities are available from
`git log --oneline 8031ea11..HEAD`. This avoids putting a commit's own hash
inside the content that determines it.

The source audits are the Spearhead repository review dated 2026-10-06 and the
historical Vault and Worktree Cleanup findings. The cleanup report was examined
as evidence; this repair performed no new branch, worktree, or vault deletions.

## Findings and resulting behavior

### P1-01: model import bypassed the external contract

Before: `run_temporal_import` deserialized directly into canonical model types.
Canonical effect types allow reviewer fields and `externally_verified`, while
the model-output v2 schema forbids them. Consequently a model could claim a
verification that belonged to the reviewer boundary.

After: `lex_core::parse_temporal_model_response` parses raw JSON, validates it
against the embedded `schemas/temporal-model-output-v2.schema.json`, and only
then deserializes it. The typed routing boundary independently rejects external
verification and reviewer metadata. Missing required nullable keys are rejected
rather than silently accepted through deserialization defaults.

Files: `crates/lex-core/src/temporal.rs`, `crates/lex-core/src/lib.rs`, and
`crates/lex-cli/src/main.rs`. The existing schema did not need a version or
content change. The actual boundary now enforces that schema.
The workspace adds `jsonschema` 0.33 with default features disabled, plus
`serde_json` in lex-core; Cargo.lock records the dependency graph. This is a
local embedded-schema validation path, without model/provider calls.

Evidence: schema regression accepts a valid payload and rejects forbidden
verification status, reviewer-only fields, and an omitted required key. An
isolated LCEC CLI import of forbidden claims failed without changing corpus
bytes. A valid-response control subsequently created eight pending reviews.

### P1-02: archived review could overwrite the live conclusion

Before: resolution looked up the determination by provision ID. An archived
pending review for old evidence could therefore replace the newer conclusion
for that same provision.

After: resolution rejects evidence-version archive IDs. It also requires a
nonempty current determination evidence hash matching the original proposed
machine conclusion before any mutation. The CLI additionally checks the actual
current corpus evidence, as described in P1-05.

Files: `crates/lex-core/src/temporal.rs` and `crates/lex-cli/src/main.rs`.
Regression tests compare complete before/after result and review state on error.
An isolated ITF archived-review resolution failed with corpus bytes unchanged.

### P1-03: evidence omission erased review history

Before: history preservation skipped an old item when the prior determination
or new determination was absent. A new request that omitted a provision could
therefore remove its pending or resolved review record from the next queue.

After: both cases archive the old item under its proposed evidence hash, or a
legacy marker when there is no hash. Audit contents, reviewer identity,
timestamps, rationale, and original machine proposal are copied intact.
Already archived items remain archived and are not repeatedly versioned.

File: `crates/lex-core/src/temporal.rs`. Regression covers pending and resolved
items, missing new determinations, retained contents, and idempotent preservation.
Preserving a historical decision does not apply it to changed or removed text.

### P1-04: original commencement backdated later article text

Before: deterministic derivation assigned the original instrument commencement
to every non-repealed article. The audit's concrete LAERO Article 10 BIS example
received `1995-12-23` even though retained evidence records its addition in 2016.

After: `temporal-derive-v2` keeps the separate resolved instrument commencement
but leaves article `effective_from` unset. It makes no replacement historical
date claim. Existing deterministic temporal status classification is unchanged.

Files: `crates/lex-parse/src/temporal_derive.rs`, its public export, and the CLI.
The new `derive-temporal <slug> --repair-dates-only` mode creates no new
determinations. Its correction predicate requires all of:

- Article type, deterministic-rule basis, and machine-accepted review status.
- Effective or future-effective status.
- Existing start date equal to the resolved original commencement.
- No end date and no transitory effects.

Model or human-reviewed state is excluded. A regression checks later-added
wording, removal of the unsupported date, preservation of a lawyer-reviewed
record, and a second repair that makes no further changes.

### P1-05: stored request/result could be stale against current corpus

Before: a request and its result could still match each other after the corpus
text changed. Checking only those stored artifacts did not establish freshness
against the current evidence.

After: import checks instrument identity/publication metadata and compares the
complete current and requested evidence ID/text-hash maps. Missing, changed, or
duplicate evidence is rejected. Review resolution checks the proposed evidence
hash against current canonical evidence before the core resolution guard runs.

File: `crates/lex-cli/src/main.rs`. Unit coverage exercises changed, missing,
and duplicate evidence. An isolated changed-corpus import failed without further
writes; restoring the original evidence allowed the valid-response control.
These checks establish evidence consistency, not upstream currency or legal truth.

## Canonical migration and invariants

Rust repaired 15,974 date values in 132 instruments. The committed
`docs/temporal-date-repair-2026-10-06.json` contains each slug and count.
Independent structured comparisons to baseline were performed before and after
the affected corpus validator sweep. Every changed provision retained its ID and
all other values; only `effective_from` changed from a date to `null`.

Source text, source/extracted-text hashes, source manifests, references, temporal
review queues, and human decisions were preserved. No canonical source was
redownloaded, and no provider or legal-review approval was invoked. Exporter
serialization removes some pre-existing trailing newlines; those are byte-format
differences without JSON-value changes.

One Markdown difference is intentional: LPUE transitory TERCERO's stale rendered
`repealed` status was synchronized to its already-canonical `unknown` status.
This does not introduce a legal conclusion. The external Obsidian vault was not
regenerated by the sweep and may still show pre-repair presentation state.

## Validation actually executed

The mechanical agent executed the repository gates; the parent inspected the
semantic changes and performed the isolated CLI and canonical-diff checks.

| Check | Result |
|---|---|
| `cargo fmt --check` | Pass |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | Pass |
| `cargo test --locked --workspace` | Pass: 198 unit tests; doc tests pass |
| `cargo build --locked -p lex-cli` | Pass |
| `cargo run --locked -p lex-cli -- validate lritf` | Pass, 0 issues; 145 articles, 11 transitories, 126 references |
| `cargo run --locked -p lex-cli -- validate ifpe-dcg-2021` | Pass, 0 issues; 59 articles, 4 transitories, 113 references |
| All 132 affected instrument validators | Pass, no command failures |
| Isolated invalid-model, stale-request, archived-review CLI cases | Reject without corpus mutation |
| Isolated valid-response control | Accept into eight pending reviews |
| Baseline/post-validation canonical comparisons | Only the stated dates and one Markdown synchronization |
| `git diff --check` | Pass |

Test distribution: 22 CLI, 20 core, 5 exporter, 148 parser, 3 source.
The earlier full-repository validator pass (211 law/regulation instruments and
32 standards) belongs to the baseline review; it was not repeated after this
repair. This pass validates the affected set and required baseline instruments.

The isolated CLI reproduction script and repair sweep are local `.work` evidence.
Their disposable work paths are not durable public dependencies. The regression
tests, repair command, Git baseline, and committed count receipt are durable.

## Limitations and next delivery

These repairs prevent the reproduced mutations; they do not establish complete
temporal correctness. Existing machine `effective` status is driven by original
commencement, and later reforms may have deferred or conditional commencement.
Consolidated wording alone cannot establish validity on a chosen historical date.
That requires formal act evidence and separate reform-aware review.

The old LFT heading-drift report remains unresolved because its retained work
source is absent. Upstream freshness monitoring, exact retained-PDF publication,
the public reader, SQLite serving index, and MCP service are planned, not built.
No push, DNS change, deployment, worker scheduling, or provider spending is part
of this local commit. The next implementation is the bounded reader described
in `docs/plans/nested-public-reader.md`.

## Post-repair review findings

An independent review of the repair confirmed the schema, stale-resolution,
omitted-evidence, and evidence-freshness fixes and the 15,974-date count. It
raised the following points; none blocks the repair.

### F1: the migration cleared correct dates as well as unsupported ones

The date an article's current text took effect is exactly one of two knowable
values: the original instrument commencement, if the article was never amended,
or the commencement of the reform that last touched it. The v2 rule and the
repair remove both, because the repair cannot tell an unamended article from an
amended one. Unamended articles lost a date that is correct. This is a lossy
but safe choice, and it should be revisited, not accepted as final.

Commencement is knowable but hard to automate. It is the next day, a stated
date, a formula, or a condition on a future fact. Where a condition is unmet
(for example, a period counted from a regulation not yet issued), the provision
is not in force until that fact occurs. The correct value is a recorded
condition with a not-yet-in-force status, not an unset date and not an inferred
one. Whether the condition has been met is the harder question and needs
act-level evidence.

Next step: count how many of the 15,974 cleared articles carry no amendment
marks, then restore the original date for those only, with the amendment-mark
check as the gate. Amended articles need reform-aware derivation of the
reform's own commencement. That is separate, larger work.

### F2: legacy review items cannot be resolved

Review resolution rejects a determination whose `evidence_sha256` is empty. No
committed review queue currently contains such an item, but the check used a
single JSON spacing style and is not a proof. Old-format queues would be
blocked. Add a test, or a documented migration path, for empty-hash items.

### F4: CLI rejections lack durable end-to-end tests

The invalid-model, stale-request, and archived-review rejections were verified
by isolated manual runs. The durable tests cover the core functions and the
evidence-fingerprint check, not these commands end to end. The summary's
validation table does not mark which rows are durable tests and which are
one-off isolated runs.

### Open from the earlier review

The repair does not address the parser findings from the second review, notably
the repeal-newline regression in `crates/lex-parse/src/lib.rs`.
