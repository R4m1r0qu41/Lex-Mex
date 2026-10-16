# Federal bulk structural admission — 2026-10-15

## Outcome

Admitted and exported **225/225 candidates**, with **zero exceptions**.
The local corpus now contains **457 instruments** (230 committed at HEAD,
227 locally added across batch 01 and this bulk pass). Of the original
332 campaign identities, 318 are now present (91 existing + 227 admitted);
the original 14 holds remain excluded.

This bulk pass added 24,989 provisions, 5,480 reference edges and 25,216
Markdown files. All 44,323 pre-existing corpus files from the 232-instrument
baseline remain byte-identical. Exact frozen canonical equality and all new
reference spans/resolved targets passed the independent scripted audit.

Validation produced no blocking errors. Remaining warnings are **450
`counts_not_frozen`** observations (two per new adapter) and **596
`non_numeric_article`** ordering-review notices for suffixed article numbers.
They are structural review work within the admitted corpus, not cleared
findings. Per-instrument issues are in each canonical `validation.json` and
in the public evidence.

## Scope and method

Operator authorized 225 additional frozen passing candidates, excluding the
already admitted LAEM/LAMNI and all 14 original holds. Baseline: 230 committed
instruments plus two locally admitted = 232. Every original campaign identity
marked already present (91) is in that baseline, independently confirmed by
GPT-6.1-Sol. The corpus includes instruments outside this campaign, so the
campaign count and whole-repository count describe different sets.

The admission runner verified source PDF/text hashes, identity uniqueness and
absence of destination collisions before copying adapters and acquisition
manifests. It reran the current Rust extraction/parser on retained bytes. Each
new instrument, provision set, source manifest, enactment evidence and reform
evidence must equal the repaired frozen candidate JSON. No live source fetch,
source editing, ownership migration or temporal model run was performed.
Original retrieval metadata and acquisition-era `resulting_git_commit` fields
are retained, rather than relabeled as this admission's revision.

After every target was populated, the runner linked, validated and exported
Markdown for each new candidate. Existing corpus files and audited human
review decisions are outside the write scope and checked against a pre-run
SHA-256 snapshot. Source manifests, validations, canonical differences,
review-queue absence and representative exports are audited in addition to
stage exit codes. New reference spans and resolved targets receive a complete
scripted check. Canonical body retention is checked against the frozen repaired
candidates; this is not a fresh independent proof of each source's legal
completeness or current effect.

## Structural counts and review boundary

The new 225 adapters keep their count baselines unfrozen. Their parser counts
are observations, not independently reviewed source counts. Validation warnings
remain visible in canonical validation reports and companion evidence. The
independently inspected LAEM/LAMNI baselines remain frozen. All candidate
provision review states stay `not_analyzed`; deterministic source-derived
statuses are preserved, and no legal commencement/effect decision is inferred.
Unknown commencement and incomplete cross-instrument discovery do not prevent
this structural admission. Resolved graph edges must still have exact spans
and existing targets; no validator rule was weakened.

## Export and remaining work

Generated Markdown lives under each new `corpus/mx/<slug>/markdown/` directory.
This is the repository export, not deployment to a personal vault or public
site. Existing instruments have not been reverse-relinked or re-exported.
That corpus-wide linking pass is separate work now that targets are populated.

The 14 original holds remain excluded and are documented individually, with
clusters and suggested remedies, in
[federal-held-source-review-2026-10-15.md](federal-held-source-review-2026-10-15.md).
Acquisition, nested containing-act modeling, identity/reference policy and
scope questions are distinct; no hold is cleared merely because this bulk
admission ran.

## Verification and provenance limits

Required native gate passed: `cargo fmt --check`,
`cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace` (250 passed, zero failed/ignored),
`cargo run --locked -p lex-cli -- validate lritf` (145/11/126, zero issues),
and `cargo run --locked -p lex-cli -- validate ifpe-dcg-2021` (59/4/113,
zero issues). `cargo build --locked -p lex-cli` also passed.

GPT-6.1-Sol's initial audit verified frozen identity/source hashes and confirmed
no collision beyond LAEM/LAMNI. Five follow-up fixture files differ from the
frozen replay implementation hashes; Rust, schemas, replay runner and overrides
match. This distinction does not change retained sources or the required exact
canonical comparison; do not claim the complete historical implementation
snapshot is unchanged.

All 225 first-pass extract/parse/link/validate/export sequences exited zero.
No target was removed. With reviewer agreement, the task-owned runner was
stopped after that complete first pass to avoid its redundant second
link/validate/export loop; any active child was allowed to finish. The runner's
exit 143 is this controlled stop, not a failed ingestion stage. A separate
audit exited zero and verified row coverage, graph closure, exact frozen
records, export identities and the unchanged baseline. The audit supports both
provision-body and official-title reference anchors. Completion is based on
these stage receipts and the independent audit, not a claimed normal runner exit.

Final GPT-6.1-Sol review: **accepted, no blockers**. Independent read-only
audit exited zero and verified all 225 identities/stage receipts, exact frozen
records/source hashes, 44,323 baseline file hashes, 457 unique instruments,
all 5,480 reference edges (including three title anchors), complete Markdown
provision-ID coverage, public tree digests, warning totals and exclusion of
all 14 holds.

Public evidence:
[federal-bulk-admission-evidence-2026-10-15.json](federal-bulk-admission-evidence-2026-10-15.json).
Detailed receipts: `.work/federal-bulk-admission/` (runner, stage logs, baseline
snapshot and per-candidate results). Public companion evidence records
final per-instrument manifests/counts/artifact-tree digests and aggregate checks.
Unrelated local changes are preserved. No commit, push or external publication
is performed by this task.

## Commit preparation

Removed one extra terminal blank line from each generated LMIGRA and
REG-LMIGRA `markdown/decreto-expedicion.md` to pass staged whitespace checks.
No other export bytes or canonical JSON changed. Regenerated the two public
artifact-tree digests and reran the independent structural audit successfully.
