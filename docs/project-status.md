# Lex-Mex Project Status

- **Status date:** 2026-08-01
- **Repository:** <https://github.com/R4m1r0qu41/Lex-Mex>
- **Committed instruments:** 193 (161 federal corpus instruments plus 32 NOMs)
- **Active ingestion batch:** `administration_ad3_servicio_publico_laboral` — complete (5/5)
- **Next checkpoint:** normalize and admit the next prepared cluster-2 batch
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
| Instruments | 193 |
| Articles | 33,984 |
| Original transitory provisions | 1,294 |
| Annexes | 29 |
| Standard clauses | 3,885 |
| Standard transitory provisions | 100 |
| Standard post-transitory supplements | 78 |
| Reference edges | 17,406 |
| Unresolved reference edges | 0 |
| Generated Markdown files | 35,468 |

All 193 `validation.json` reports are valid. They contain 243 non-blocking
warnings: 175 non-numeric/suffixed-article notices, 16 unfrozen count
baselines, 35 represented supplements whose source states no explicit legal
character, 7 article-gap notices, 3
warnings for official standard modifications not incorporated in the
retained source text, 3 decree targets that match no committed clause, 2
suffix-order notices, 1 redesignated standard (NOM-002-SEMARNAT-1996,
published as NOM-002-ECOL-1996), and 1 modification whose recorded DOF title
names no numeral at all. Validity does not imply that temporal analysis or
legal review has been performed.

## Federal structural first pass

The source inventory defines a 454-instrument Cámara universe (laws and
regulations, excluding DCGs). The pre-cluster corpus held 128 instruments;
CN1 and CN2 added 16 and are structurally closed. AD1 has added all six
instruments through `lgbn`, AD2 has added all five instruments through
`lfar` (`lspm`, `lfaebsp`, `reg-lfaebsp`, `reg-lopsrm`, `lfar`;
`batches/administration_ad2_bienes_obras_servicios.json`), and AD3 has now
added all five instruments through `reg-laat` (`lspcapf`, `lfremsp`,
`locfcrl`, `reg-art121-122-lft`, `reg-laat`;
`batches/administration_ad3_servicio_publico_laboral.json` — `reg-laat`
also resolves the entry `batches/labor_L1_labor.json` had left blocked).
Together with the separate Maximasa federal-gap ingestion of `reg-csps`, the
live corpus now contains 161 instruments.

The cluster-2 first pass contains 326 instruments in 53 batches. Its state is:

| State | Batches | Instruments |
|---|---:|---:|
| Structurally closed (CN1, CN2) | 2 | 16 |
| Structurally complete (AD1, AD2, AD3) | 3 | 16 |
| Prepared, not yet admitted | 48 | 291 |
| Explicitly blocked | 2 | 3 |

The remaining prepared cluster-2 workload is 291 instruments. `egdf`,
`lif-2026`, and `pef-2026` remain explicit
deferrals pending reviewer direction; they are not silently treated as
complete.

The separate Maximasa standards sequence added NOM-251-SSA1-2009,
NOM-247-SSA1-2008, NOM-051-SCFI-SSA1-2010, NOM-002-STPS-2010, and
NOM-187-SSA1-SCFI-2002. NOM-247 carries two
`standard_unconsolidated_modification` warnings; its retained clause text
must not be used as current obligations until those modifications are
incorporated, and no official consolidated text exists to clear them (both
are narrow numeral-level DOF decrees, not full republications). Since
2026-07-31 that staleness is located rather than instrument-wide: **17 of
NOM-247's 252 clauses carry `amended_by` marks** naming the decree and its own
verb, and three targets the decrees name resolve to no committed clause
(`5.1.5`, an *adición*; `Apéndice normativo A`, which the corpus does not
model; and `5.2.7.ii.1)`, whose numeral does not exist in the base text at
all). The marked clauses still carry their full base-publication text — the
marks record known staleness, not a correction. NOM-051 was
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
2026-07-28) is complete across all 27 candidates: **all 27 are ingested.**
All are `as_published`; NOM-020-STPS-2011 carries the only
new unconsolidated-modification warning (the ACUERDO de Modificación
published 2015-10-19, eliminating inciso j) of numeral 13.2, which the
retained text does not incorporate). That decree's DOF title names no numeral
at all — the STPS "ACUERDO de Modificación a la Norma Oficial Mexicana
NOM-020-STPS-2011, ..." form carries only the standard's identity — so
NOM-020 correctly stays at instrument level with a
`standard_modification_scope_unknown` warning rather than a guessed mark.

The six originally flagged candidates were held under the hold-out-and-flag
policy (`docs/decisions.md` 2026-07-28). They cluster into
three parser failure classes plus one metadata question, all recorded in
`docs/ingestion-difficulty-log.md`: `annex-form-numbering`
(NOM-019-STPS-2011), `annex-continues-numbering` (NOM-010-STPS-2014,
NOM-035-STPS-2018, NOM-024-STPS-2001), `indice-selected-as-body`
(NOM-052-SEMARNAT-2005), and `metadata-ambiguity` (NOM-002-SEMARNAT-1996,
whose retained text is titled NOM-002-ECOL-1996). Every one of the five
parser cases validated `valid` with 0 issues while being structurally wrong,
which is the load-bearing finding: `validation.json` checks the internal
consistency of whichever clause run was selected, never whether the correct
run was selected.

**Resolved 2026-07-29** (`docs/decisions.md`): the reviewer supplied the
governing rule — a standard's normative numbered body ends at TRANSITORIOS —
and the clause parser now bounds the body at the real, índice-disambiguated
transitorios heading, with form feed admitted to the leading-whitespace class
so page-boundary headings stop being invisible. All five clause defects are
closed, verified by byte-identical reparse of every previously-committed
standard. NOM-010-STPS-2014 (206 clauses, was 950) and NOM-035-STPS-2018
(111, was 124) were ingested first. On 2026-07-31 the exact-span supplement
boundary closed `transitory-absorbs-annex`; fresh hash-matching official PDFs
then admitted NOM-019 (94 clauses, 3 transitories, 1 supplement), NOM-024 (87,
2, 2), and NOM-052 (76, 3, 8).
NOM-002-SEMARNAT-1996 is ingested under the reviewer-granted redesignation
rule, carrying `published_designation: NOM-002-ECOL-1996`.

Every standard now has required `supplements.json`. Twenty-six standards have
78 exact-span top-level post-transitory records and six have an empty file.
All 32 standards pass deep reparse validation; no pre-existing clause changed,
every earlier transitory stayed exact, and each of the 11 migrated final-tail
changes is a strict truncation of closing or supplement text.

**AD2 admitted, 2026-08-01.** `batches/administration_ad2_bienes_obras_servicios.json`
(normalized from `prompts/cluster-2-batches/lex-mex-cl2-batch-AD2.json`) added
all five prepared instruments: `lspm`, `lfaebsp`, `reg-lfaebsp`, `reg-lopsrm`,
`lfar`. Provisional processing of the first instrument (`lspm`) hit the same
`1o`-style ordinal-numbering case the `allow_article_gaps` review gate was
built for on 2026-07-18 (`lfrsp`, same defect): two of the five source PDFs
(`lspm`, `lfaebsp`) number their first nine articles with the Spanish
ordinal abbreviation (`ARTICULO 1o.-` … `9o.-`) before switching to plain
cardinal numbering at article 10, and the strict article-order path never
advances past a `non_numeric_article` warning, so every article from 10
onward fails as `article_order`. `labels.rs` already canonicalizes
`1o`/`2º`/`3°` ordinal marks for exactly this case; the fix, as intended, was
the reviewed adapter setting `allow_article_gaps: true`, routing both through
the label-aware ordering path instead of the strict one. No parser code
changed and no default should — the strict default is what forces this
review on every new instrument. All five instruments validate clean and
reverse-link with 0 unresolved references; `lfaebsp`
separately carries genuine `6-bis`/`6-ter`/`6-quater` suffixed articles the
same label-aware path resolves correctly. Full finding: `docs/decisions.md`
2026-08-01.

**AD3 admitted, 2026-08-01.** `batches/administration_ad3_servicio_publico_laboral.json`
(normalized from `prompts/cluster-2-batches/lex-mex-cl2-batch-AD3.json`) added
all five prepared instruments: `lspcapf`, `lfremsp`, `locfcrl`,
`reg-art121-122-lft`, `reg-laat`. Two independent findings, both resolved
without a parser code change to the underlying grammar: `lspcapf` and
`reg-art121-122-lft` hit the same `1o.`–`9o.`-prefix ordinal case as AD2 —
`allow_article_gaps: true` again — bringing the confirmed count to four
statutes (`lfrsp`, `lspm`, `lfaebsp`, and now these two). Separately,
`lspcapf`'s own reform-decree appendix cited a publication date written
`1º de septiembre de 2005` (the day-of-month ordinal mark, not the article
one); `extract_reform_evidence`'s publication-date regex required a bare
digit and had no tolerance for it, so the decree's first transitory hit a
hard "without its Diario Oficial publication date" parse error.
`extract_dof_publication`'s separate regex already tolerated `[oº]?` for
this exact case; the fix widened the reform-appendix regex to match, with a
new fixture. All five AD3 instruments validate clean and reverse-link with
0 unresolved references (49 new edges). Full finding: `docs/decisions.md`
2026-08-01.

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
  hermetic CI;
- `lex-mex review-packets generate` (landed 2026-07-31) groups the 157
  committed instruments that have a `batches/*.json` manifest into 32
  packets for reviewer assignment; the 32 standards and the CNBV DCG family
  have no batch manifest and so are not yet covered by this mechanism.

Next general cluster action: normalize the next prepared cluster-2 batch
(AD4, `cl2_AD4_proteccion_civil_misc`) into an operational manifest per the
cluster plan's admission order (`docs/plans/cluster-2-federal-corpus-ingestion.md`).
AD1–AD3 are now structurally complete. The separately authorized five-NOM
Maximasa sequence does not reorder the prepared federal batches.

## Archived divergent branches

`main` is the only active development line. The divergent `fable` worktrees
were deleted after their common superset history was retained by the annotated
tag `archive/fable-cross-linking` (peeling to
`e7ed63699f4577c78300ca379dbe431c6db1d424`). Their contents are never merged
or cherry-picked wholesale; a useful unit is reimplemented and reviewed on
current `main`.
