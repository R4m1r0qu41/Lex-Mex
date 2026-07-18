# Lex-Mex Project Status

- **Status date:** 2026-07-16
- **Repository:** <https://github.com/R4m1r0qu41/Lex-Mex>
- **Committed instruments:** 148
- **Active ingestion batch:** `administration_ad1_planning_paraestatals`
- **Next checkpoint:** `lfrsp`
- **Current legal reviewer:** JRH

## Current state

Lex-Mex is a provenance-aware compiler and temporal-analysis pipeline for
Mexican federal legal materials. Rust owns acquisition metadata, canonical
normalization, validation, reference extraction, review-state transitions, and
publication. Model output is a schema-gated proposal and cannot overwrite an
audited human decision.

The committed corpus contains official consolidated texts from Cámara de
Diputados and CNBV sources. Obsidian is a presentation target only; generated
content is confined to `Corpus/<instrument>/`.

Current committed-corpus totals:

| Artifact | Count |
|---|---:|
| Instruments | 148 |
| Articles | 32,711 |
| Original transitory provisions | 1,215 |
| Annexes | 28 |
| Reference edges | 16,887 |
| Unresolved reference edges | 0 |
| Generated Markdown files | 34,102 |

All 148 `validation.json` reports are valid. They contain 187 non-blocking
warnings: 162 suffixed-article ordering notices, 16 unfrozen count baselines,
7 article-gap notices, and 2 suffix-order notices. Validity does not imply
that temporal analysis or legal review has been performed.

## Federal structural first pass

The source inventory defines a 454-instrument Cámara universe (laws and
regulations, excluding DCGs). The pre-cluster corpus held 128 instruments;
CN1 and CN2 added 16 and are structurally closed. AD1 has now added `lplan`,
`lfep`, `reg-lfep`, and `lfrpe`; the current corpus therefore contains 148 of the 454
instruments.

The cluster-2 first pass contains 326 instruments in 53 batches. Its state is:

| State | Batches | Instruments |
|---|---:|---:|
| Structurally closed (CN1, CN2) | 2 | 16 |
| Completed within active AD1 batch | 1 | 4 |
| Admitted, remaining in AD1 | 1 | 2 |
| Prepared, not yet admitted | 50 | 301 |
| Explicitly blocked | 2 | 3 |

The remaining active workload is 303 instruments: 202 laws, 98 regulations,
2 codes, and 1 ordinance. `egdf`, `lif-2026`, and `pef-2026` remain explicit
deferrals pending reviewer direction; they are not silently treated as
complete.

The active plan is
[`cluster-2-federal-corpus-ingestion.md`](plans/cluster-2-federal-corpus-ingestion.md).
It is the authoritative source for batch order, source inventories, recovery,
and historical receipts. Earlier status snapshots and superseded checkpoint
narratives are preserved in Git history rather than duplicated as live docs.

## Batch operating loop

Process the first instrument of each batch provisionally, inspect its source
manifest and canonical diff, then freeze reviewed structural counts and run
the bounded batch closure. The closure relinks, validates, and republishes the
successful selected instruments, and evaluates concrete `expected_edges` as
`satisfied`, `missing`, `deferred`, or `invalid`.

Every reusable learning must land before the next instrument uses it:

- parser or linker behavior: focused regression fixture and deterministic
  implementation change;
- source-specific boundary, stop marker, or title mapping: reviewed adapter
  configuration;
- operating discovery: the plan's timestamped `Progress` and `Surprises and
  discoveries` sections;
- durable semantic or architecture decision: `docs/decisions.md`.

This makes later batches faster through local deterministic code while keeping
canonical source text, legal ambiguity, and reviewer decisions protected.

## Temporal and review scope

Structural ingestion and temporal analysis are separate programs. Newly
normalized provisions remain `review_status: not_analyzed`; ordinary
provisions start `temporal_status: unknown`, while an express source-text
repeal note starts `repealed`. The audited temporal vertical slice remains
`lritf`, `ifpe-dcg-2021`, and `itf-dcg-2018`. JRH is the legal reviewer of
record; ITF DCG transitory SÉPTIMO remains pending formal-boundary review.

## Known gaps and next action

- corpus-wide relinking and human expected-edge recall review are deferred
  until the broader target set is admitted;
- exact-title aliases not in the curated registry still need an
  adapter-scoped mapping or a reviewed registry expansion;
- no automated official-source change monitor, candidate-version flow, or
  provision-level update diff exists;
- `source-manifest.resulting_git_commit` still records the pre-ingestion HEAD;
- live network/model flows remain integration-tested manually rather than in
  hermetic CI.

Next: provisionally ingest `lfrsp`, record any reusable deterministic learning,
then inspect and freeze its structural baseline before continuing AD1.

## Archived divergent branches

`main` is the only active development line. The divergent `fable` worktrees
were deleted after their common superset history was retained by the annotated
tag `archive/fable-cross-linking` (peeling to
`e7ed63699f4577c78300ca379dbe431c6db1d424`). Their contents are never merged
or cherry-picked wholesale; a useful unit is reimplemented and reviewed on
current `main`.
