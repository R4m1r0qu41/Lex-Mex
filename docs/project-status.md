# Lex-Mex Project Status

- **Status date:** 2026-07-28
- **Repository:** <https://github.com/R4m1r0qu41/Lex-Mex>
- **Committed instruments:** 160 (151 federal corpus instruments plus 9 NOMs)
- **Active ingestion batch:** `administration_ad2_public_service_mobility`
- **Next checkpoint:** `lspm`
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
| Instruments | 160 |
| Articles | 33,231 |
| Original transitory provisions | 1,240 |
| Annexes | 29 |
| Standard clauses | 1,487 |
| Standard transitory provisions | 22 |
| Reference edges | 17,173 |
| Unresolved reference edges | 0 |
| Generated Markdown files | 34,651 |

All 160 `validation.json` reports are valid. They contain 193 non-blocking
warnings: 166 non-numeric/suffixed-article notices, 16 unfrozen count
baselines, 7 article-gap notices, 2 suffix-order notices, and 2 warnings for
official standard modifications not incorporated in the retained source
text. Validity does not imply that temporal analysis or legal review has been
performed.

## Federal structural first pass

The source inventory defines a 454-instrument Cámara universe (laws and
regulations, excluding DCGs). The pre-cluster corpus held 128 instruments;
CN1 and CN2 added 16 and are structurally closed. AD1 has now added all six
instruments through `lgbn`. Together with the separate Maximasa federal-gap
ingestion of `reg-csps`, the live corpus now contains 151 instruments.

The cluster-2 first pass contains 326 instruments in 53 batches. Its state is:

| State | Batches | Instruments |
|---|---:|---:|
| Structurally closed (CN1, CN2) | 2 | 16 |
| Structurally complete AD1 batch | 1 | 6 |
| Prepared, not yet admitted | 50 | 301 |
| Explicitly blocked | 2 | 3 |

The remaining prepared cluster-2 workload is 301 instruments. `egdf`,
`lif-2026`, and `pef-2026` remain explicit
deferrals pending reviewer direction; they are not silently treated as
complete.

The separate Maximasa standards sequence added NOM-251-SSA1-2009,
NOM-247-SSA1-2008, NOM-051-SCFI-SSA1-2010, NOM-002-STPS-2010, and
NOM-187-SSA1-SCFI-2002. NOM-247 carries two
`standard_unconsolidated_modification` warnings; its retained clause text
must not be used as current obligations until those modifications are
incorporated, and no official consolidated text exists to clear them (both
are narrow numeral-level DOF decrees, not full republications). NOM-051 was
refreshed from the official 2020-03-27 DOF publication, which is a full
restatement of the standard rather than a targeted amendment; it now carries
zero unconsolidated-modification warnings. NOM-187's 2023 record is a
systematic review with result `Modificación`, not a succession event.

All five NOMs were backfilled with `transitories.json` (10 addressable
transitorio blocks total: 6 for NOM-051, 4 for NOM-002-STPS; NOM-251,
NOM-247, and NOM-187 have none — their retained as-published texts never
reach a transitorios section). This is a lightweight span-and-date
inspection, not a structural parse; see "Standards transitorio inspection"
in `docs/standards-module.md`. Standards have no Markdown export profile
(`collect_standard` bails on `CanonicalMarkdown`), so `Generated Markdown
files` above is unaffected by this addition.

The batch-2 NOM ingestion (`docs/plans/nom-standards-batch-2.md`, staged
2026-07-28) added four more STPS standards on its first tranche:
NOM-001-STPS-2008, NOM-004-STPS-1999, NOM-005-STPS-1998, and
NOM-009-STPS-2011 (86, 12, 72, and 155 clauses; 3 transitories each; all
`as_published`, zero unconsolidated-modification warnings). One candidate
from the same tranche, NOM-019-STPS-2011, was held out and flagged rather
than committed — see `docs/ingestion-difficulty-log.md`
(`annex-form-numbering`).

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

The NOM standards batch (`docs/plans/nom-standards-batch-2.md`, staged
2026-07-28) runs this same loop with one addition: an instrument whose
difficulty isn't a quick fix is held out of `corpus/` and flagged in
`docs/ingestion-difficulty-log.md` instead of being forced through — see
`docs/decisions.md` 2026-07-28.

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

Next general cluster action: normalize AD2 and provisionally ingest `lspm`.
The separately authorized five-NOM Maximasa sequence does not reorder that
prepared federal batch.

## Archived divergent branches

`main` is the only active development line. The divergent `fable` worktrees
were deleted after their common superset history was retained by the annotated
tag `archive/fable-cross-linking` (peeling to
`e7ed63699f4577c78300ca379dbe431c6db1d424`). Their contents are never merged
or cherry-picked wholesale; a useful unit is reimplemented and reviewed on
current `main`.
