# Independent parser split audit — 2026-10-24

**Verdict: accept the production refactor for operator integration review, with
explicit coverage limits and two nonblocking tooling findings. Accept D-001.**
No merge, push or deployment is authorized or performed by this review.

Handoff: `2026-10-24-lex-mex-parser-split-independent-audit.md`.
Reviewed HEAD: `9ad15ec9886c1e24957f93f72de6a04fbbf50d40`.
Tested production C: `1250889ce10b8ba910a5f5ca5f232fd5752e79df`.
Baseline B: `85c41f3898d86737766564fa8d3fd8f6d4ae74a3`.
Claude's worktree was clean; all changes after C are docs/prompts, not production,
corpus, schema, adapters or fixtures. Codex performed this audit; the recorded
Opus advisor opinion was considered supporting evidence, not independent review.

[Machine evidence](parser-split-independent-audit-2026-10-24-evidence.json)
contains hashes, verdicts and independent probes. The earlier
[A/B review](parser-split-ab-codex-review-2026-10-24.md) remains valid.

## Verdict per handoff claim

| Claim | Verdict and evidence |
| --- | --- |
| 1. Strict A/B parity | Confirmed. Prior independent checks cover all 15 runs, 12 pairs, 22,330 command rows and 44,660 stream files; every handoff-bound artifact hash was checked again (54/54 match). No production change followed C3. |
| 2. Harness identity | Confirmed. Gate SHA `fa321e46…`, blob `9c329e0c…`; HEAD repaired blob `24c9534b…`. These are deliberately separate evidence generations. |
| 3. Native checks at C | Confirmed from hash-bound raw logs and command exit records: all five checks exit 0; 425 tests pass, none fail/ignored; both validators valid with the stated counts. The lists preserve all 419 baseline test names and add six. Full native suite was not rerun by Codex. |
| 4. DB parity | Confirmed independently. Reran db verify and all 21 goldens for B/B and both B/C artifacts. All pass, stale paths empty. Rehashed actual databases/manifests; integrity/FK/schema checks pass. Complete ordinary table rows and FTS virtual content match, with only the allowed release provenance differences. Candidate a/b bytes and manifests are identical. |
| 5. Dependency boundaries | Confirmed by manifests and module/import review: grammar is a leaf; analysis/link use core+grammar; parse retains the compatibility facade and initial-analysis calls; DB uses core+grammar+analysis. No reverse production/dev edge. |
| 6. Comparator/restart self-test | Recorded 28/28 result confirmed by bound raw log. Independent archival, finished-label refusal and unique-output probes pass. Full self-test was not rerun. Restart coverage has the residual edge below. |

## Findings

### A-001 — P3: G4 helper scripts do not propagate command failure

`prompts/parser-split-g4-native.sh:6` echoes the command status and returns the
successful echo status. Its final status pipeline also masks earlier failures.
`prompts/parser-split-g4-candidate.sh:12–18` continues after failed project/verify/
golden commands and may finish successfully. Temporary-copy failure injection
made required commands exit 42; both wrappers still exited 0.

Fix before using these scripts as unattended gates: retain each command's exit
code, return nonzero on any failure, and make comparison tooling return nonzero
when parity fails. Keep diagnostic output. Current acceptance stands because
actual native command exits are recorded as zero and DB artifacts/verification/
goldens were independently checked; wrapper status alone was not relied upon.

### A-002 — P3: restart repair does not cover completed-result/state-write gap

`prompts/parser-split-queue.py` marks a run done only after the child exits;
`prompts/parser-split-ab.py::finish` writes its result before returning. If the
driver stops between those events, resume reruns the same label, but
`archive_interrupted` refuses any existing result. A run that writes an unusable
result and exits 3 has the same refusal on retry. Active partial-log interruption
is repaired; these result-present cases still need explicit recovery.

Reconcile a valid completed result with expected binary/harness/input identity
before marking done, or allocate a fresh attempt identity while preserving the
old result and logs. Use atomic state updates and add result-present retry tests.
This did not occur in the completed 27-step queue and does not invalidate it.

## Production and legal-review judgment

The previous independent body comparison covered 266 top-level functions modulo
namespace/whitespace changes. Additional inspection of citation targeting,
qualifiers, exact character spans, validator issue construction and audited-review
application found no changed predicate, ordering or message. Shared grammar
helpers retain lexical responsibilities. Parse-time initial repeal/temporal
classification still executes at the original point through analysis; no legal
inference was moved into grammar or delayed to a later pipeline step.

The undriven-command table is acceptable as a direct-caller map, not whole-command
coverage: review/import logic remains in unchanged core; their export paths use
unchanged export code. Standards functions remain in parse, with their shared
lexical helpers now in grammar; standards validation/refresh exercise them.
Pipeline/batch orchestration is not wholly replayed, but its moved functions are
exercised through component stages and existing unit tests. No model-provider
run was made. Public compatibility paths remain available; the 126-path fixture
is appropriately described as a path check rather than exhaustive signatures.

**D-001 should be retained.** Every compiled source must match its claimed Git
builder, and additional pinned Rust/JSON sources must also be rejected. The
unchanged builder control succeeds; changed files/manifests and newly added
sources fail for the expected paths. Codex reran the three targeted provenance
tests (exactly one test passed per invocation). The extra-source test isolates
the new inventory check: removing that check leaves all embedded bytes equal,
so its `expect_err` would fail. A guard-removal mutation was not independently
compiled during this audit. The fixture adds an unused source deliberately to
isolate inventory completeness; its name saying "consumed" is imprecise, but
its assertion honestly tests the stronger inventory restriction.

## Database verification

All three logical content hashes equal
`8cc3736018bffae94f303d9cbfa378170c9ff0e2842ce708431f08d528b60359`.
Candidate physical hash:
`483264b74cb8f4ffe25917e3876a4a5672cfdf82bce8b940e8b9f527191a9060`.
Codex compared 298,981 ordinary rows outside `corpus_release`, plus all 68,341
FTS virtual rows, for each baseline/candidate pair and candidate/candidate.
Schema definitions match. The release specification differs only in
`builder_commit` and `cargo_lock_sha256`; release ID and physical hash follow
that provenance change. Legal-review payload tables and provision extras are
included in the complete row comparison.

Census remains 468 instruments, 68,341 provisions, 35,930 reference edges,
5,766 terms, 97,672 usages, 14 active structural-debt records, and 32 catalogue
standards. No legal decision or review state changed.

## Acceptance limits and next action

106 instruments lack retained parse inputs, and two replayed sources do not match
canonical provenance. Eight retained command failures and one supplementary
failure reproduce baseline. These remain explicit ingestion/coverage debt; this
review approves the refactor within tested coverage, not every instrument's
correctness. Date equivalence is limited to observed run dates. Reader exporter
build/package verification remains an integration-commit requirement.

Only this report and evidence are added on main. All unrelated dirty/untracked
state and Claude's clean worktree are preserved. No code/corpus changes, commits,
merges, pushes or deployment. Next: operator selects integration timing; correct
the two tooling findings before unattended reuse and run the reader integration
check on the integration candidate.
