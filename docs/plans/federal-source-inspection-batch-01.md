# Federal source inspection — batch 01

## Purpose and scope

Inspect frozen LAEM and LAMNI candidate sources and their structural artifacts;
produce a visible per-source readiness/hold report. Two short sources bound the
first inspection to 21 articles and seven transitories. Operator authorized structural admission of both inspected candidates on
2026-10-15. Model runs, legal-status determinations, new downloads and changes
to the 14 campaign holds remain excluded.

## Authority and immutable baseline

Baseline: `12e0cc75253076d5ccb783fd1432dcdb48695bfb` on main.
Repository instructions, source bytes, Rust types and accepted decisions outrank
this plan. Navigation: `docs/federal-ingestion-integration-review-2026-10-15.md` (archived 2026-10-16 to the git-ignored `archive/`; recover from git history)
and `docs/federal-ingestion-followup-evidence.json` (archived 2026-10-16 to the git-ignored `archive/`; recover from git history). Frozen replay roots are
`.work/federal-followup-all-v1/shard-3/laem/` and
`.work/federal-followup-all-v1/shard-6/lamni/`. They are evidence, not canonical
admission authority. Initial unrelated notes deletions and external-agent audit
remain untouched; the earlier review and evidence correction also remain intact.

## Milestones and verification

1. Verify identities, retained PDF/text digests, source metadata and source
   structure against candidate articles and transitories.
2. Inspect candidate differences, validation warnings, references, review-state
   preservation and Markdown. Comparison may collapse whitespace and remove
   enumerated page furniture in memory only; never alter source bytes.
3. Record readiness and exact uncertainty per source. Stop a candidate when
   attribution, completeness or identity is uncertain. Preserve its artifacts.

Verification evidence: `docs/federal-source-inspection-batch-01-evidence.json`.
Verifier: `.work/bounded-source-inspection-01/verify.py` (local work file).
It asserts all 28 candidate bodies occur in source after whitespace/page-furniture
comparison, four retained-byte hashes match manifests, reference offsets/targets
validate, export IDs exist, review/temporal states remain unreviewed/unknown,
and retained validation reports pass with only unfrozen-count warnings.
This comparison proves text retention, not legal effect or current-law status.
Recovery: rerun against the frozen roots and compare recorded artifact digests;
if missing or different, stop rather than replace provenance from a live URL.

## Current checkpoint — 2026-10-15

Complete: admitted LAEM (17 articles, 5 law-owned transitories, 1 reference)
and LAMNI (4 articles, 2 transitories, 1 reference) to the working corpus.
Both validators report zero issues. Corpus is 232 locally versus 230 at HEAD.
No model/legal temporal review, existing corpus rewrite, commit or push.

Extract/parse/link/validate/Markdown export passed for both frozen sources.
Instrument/provisions/source manifest/reform evidence equal inspected replay
records exactly. Source offsets and target existence checked for both edges.
No unexplained canonical difference. Prior replay's missing LAEM constitutional
edge is resolved against the full corpus. Both frozen count baselines installed.

Required native gate passed: `cargo fmt --check`;
`cargo clippy --workspace --all-targets -- -D warnings`;
`cargo test --workspace` (250 passed, zero failed);
`cargo run --locked -p lex-cli -- validate lritf` (145/11/126, zero issues);
`cargo run --locked -p lex-cli -- validate ifpe-dcg-2021` (59/4/113, zero issues).
`cargo build --locked -p lex-cli` passed. Mechanical agent ran the required gate.

Exactly one next action: inspect the next bounded cluster among the remaining
225 replay candidates. Original 14 holds and broader reverse linking remain
separate. Commit/push was not performed for this admission.

## Outcome and retrospective

Source inspection raised an attribution question that operator PDF review
resolved in favor of law ownership. References to “Decreto” alone did not
establish a separate decree-owned transitory layer. The earlier proposed
migration is superseded; preserve source text and the recorded human decision.

## Admission authorization — 2026-10-15

Operator instructed “admit”. Admit only LAEM and LAMNI from frozen sources,
with inspected 17/5 and 4/2 count baselines. Preserve original manifests and
source hashes, law-owned LAEM transitories and unreviewed temporal state.
Run extract, parse, link, validate and Markdown export; review all candidate
changes and required native checks. Stop on failed checks or unexplained
source/body differences. No commit or push is part of this admission.
