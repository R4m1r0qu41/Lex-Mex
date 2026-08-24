# Lex-Mex Project Status

- **Status date:** 2026-08-23
- **Repository:** <https://github.com/R4m1r0qu41/Lex-Mex>
- **Committed instruments:** 235 (203 federal corpus instruments plus 32 NOMs)
- **Active ingestion batch:** vault-reconciliation recovery across `tax_T1_core`, `financial_F4_popular_savings`, `financial_F5_credit_auxiliaries` — complete (9/10; `lieps` held out)
- **Next checkpoint:** normalize and admit EC1 (`cl2_EC1_competencia_inversion`)
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
| Instruments | 235 |
| Articles | 38,242 |
| Original transitory provisions | 1,529 |
| Annexes | 29 |
| Standard clauses | 3,885 |
| Standard transitory provisions | 100 |
| Standard post-transitory supplements | 78 |
| Reference edges | 20,668 |
| Unresolved reference edges | 0 |
| Generated Markdown files | 40,003 |

All 235 `validation.json` reports are valid. They contain 374 non-blocking
warnings: 238 non-numeric/suffixed-article notices, 16 unfrozen count
baselines, 35 represented supplements whose source states no explicit legal
character, 8 article-gap notices, 3
warnings for official standard modifications not incorporated in the
retained source text, 3 decree targets that match no committed clause, 4
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
`batches/administration_ad2_bienes_obras_servicios.json`), AD3 has added
all five instruments through `reg-laat` (`lspcapf`, `lfremsp`,
`locfcrl`, `reg-art121-122-lft`, `reg-laat`;
`batches/administration_ad3_servicio_publico_laboral.json` — `reg-laat`
also resolves the entry `batches/labor_L1_labor.json` had left blocked),
and AD4 has now added all five instruments through `ldcpdch` (`lgpc`,
`reg-lgpc`, `lhheum`, `ldvuma`, `ldcpdch`;
`batches/administration_ad4_proteccion_civil_misc.json`), closing Domain AD.
TX1 has added all five instruments through `reg-cff` (`lsat`, `lfdc`,
`lopdc`, `lfpca`, `reg-cff`; `batches/tax_TX1_sat_procedimiento.json`),
opening Domain TX. TX2 has added three of its four prepared instruments
(`lfd`, `lgdp`, `reg-lfprh`; `batches/tax_TX2_ingresos_presupuesto.json`)
— `lcmopfih` is held out for a structural parsing difficulty (see below),
distinct from `lif-2026`/`pef-2026`'s pre-existing reviewer-confirmation
block. TX3 has added two of its three prepared instruments (`lfisan`,
`reg-ladua`; `batches/tax_TX3_impuestos_aduanas.json`) — `lisipl` is held
out for the same structural difficulty as `lcmopfih`, a second confirmed
instance the same day, closing Domain TX. FI1 has added four of its five
prepared instruments (`lsp`, `lmeum`, `lcmm`, `ltfccg`;
`batches/financial_FI1_autoridades_pagos.json`), opening Domain FI —
`lcnbv` is held out for a new failure class, a stale cross-reference to a
provision of an already-committed instrument that has since been
repealed (see below). FI2 has added all six prepared development-bank laws
(`lonf`, `loshf`, `lobb`, `lobnce`, `lobnosp`, `lobnefaa`;
`batches/financial_FI2_banca_desarrollo.json`). FI3 has added all eight
prepared rural-insurance, agricultural-guarantee, retirement-system, and
development-convention instruments (`lfaar`, `lfgfaga`, `reg-lfgfaga`,
`reg-lfif-art95`, `reg-lsar`, `lmccbdc`, `lccaif`, `lccbid`;
`batches/financial_FI3_seguro_rural_convenios.json`). Together with the
separate Maximasa federal-gap ingestion of `reg-csps`, and the RC1 recovery
batch below, the live corpus now contains 203 federal instruments.

The 2026-08-23 vault-reconciliation recovery is not a new batch. A 2026-08-23
reconciliation between this repository and the Obsidian vault found ten
instruments that existed only as pre-2026-07-11 Python-era vault renders,
with no committed corpus record and no entry in the cluster-2 inventory:
the ISR, IVA and IEPS laws with their reglamentos, plus `lacp`, `lgoaac`,
`lrascap` and `luc`. Four of them were named in the vault's own cluster-2
ingestion note among the 18 instruments whose Python renders folded
letter-suffixed articles into their parents, so they were both uncommitted
and known-defective where they did exist. Nine admitted clean (`lisr`,
`reg-lisr`, `liva`, `reg-liva`, `reg-lieps`, `lacp`, `lgoaac`, `lrascap`,
`luc`), adding 1,524 articles, 54 original transitories and 2,053 resolved
reference edges with 0 unresolved. These ten were already listed in their owning cluster-1 manifests
(`tax_T1_core`, `financial_F4_popular_savings`,
`financial_F5_credit_auxiliaries`) and had simply never been run through the
Rust gate, so the recovery was recorded in those manifests rather than in a
new one — a slug belongs to exactly one batch manifest. `lieps` is held out
under `multi-character-article-suffix`
(`docs/ingestion-difficulty-log.md`) and moved to `tax_T1_core`'s `blocked`
list, which is why `lex-source`'s frozen unique-slug baseline moves 200 → 199.
`lacp`, `lgoaac`, `lrascap` and `luc` were recorded in the cluster-1
manifests against CNBV's Normatividad mirror despite being federal laws;
operator decision 2026-08-23 re-pointed them at the Cámara de Diputados
consolidated texts and the `diputados` adapter, consistent with every other
committed federal law. `liva`, `lacp` and `lgoaac` needed the reviewed
`allow_article_gaps: true` adapter setting for the familiar ordinal-mark
case; `liva` additionally required a parser fix
(`article-letter-suffix-qualifier-heading`, resolved the same day).

The cluster-2 first pass contains 326 instruments in 53 batches. Its state is:

| State | Batches | Instruments |
|---|---:|---:|
| Structurally closed (CN1, CN2) | 2 | 16 |
| Structurally complete (AD1–AD4, TX1–TX3, FI1–FI3) | 10 | 49 |
| Prepared, not yet admitted | 41 | 255 |
| Explicitly blocked or held out | 3 | 6 |

The remaining prepared cluster-2 workload is 255 instruments. `egdf`,
`lif-2026`, and `pef-2026` remain explicit deferrals pending reviewer
direction; `lcmopfih` and `lisipl` are held out per
`docs/ingestion-difficulty-log.md`'s `nested-law-in-enacting-article`
class, and `lcnbv` is held out per that same log's newly added
`stale-cross-reference-to-repealed-provision` class. None of the six are
silently treated as complete.

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

**AD4 admitted, 2026-08-01, closing Domain AD.**
`batches/administration_ad4_proteccion_civil_misc.json` (normalized from
`prompts/cluster-2-batches/lex-mex-cl2-batch-AD4.json`) added all five
prepared instruments: `lgpc`, `reg-lgpc`, `lhheum`, `ldvuma`, `ldcpdch`. All
five ran clean on the first pass at the default `allow_article_gaps: false`
— the first AD batch with neither an ordinal-numbering review case nor a
parser defect. All five validate clean and reverse-link with 0 unresolved
references (50 new edges; 233 new articles, 29 new original transitories).
Full finding: `docs/decisions.md` 2026-08-01.

**TX1 admitted, 2026-08-01, opening Domain TX.**
`batches/tax_TX1_sat_procedimiento.json` added all five prepared
instruments: `lsat`, `lfdc`, `lopdc`, `lfpca`, `reg-cff`. `lsat`, `lfdc`,
`lfpca` hit the familiar `1o.`–`9o.` ordinal-numbering case; reviewed
`allow_article_gaps: true`, no parser change. All five validate clean and
reverse-link with 0 unresolved references (181 new edges; 341 new
articles, 28 new original transitories). Full finding:
`docs/decisions.md` 2026-08-01.

**TX2 admitted, 2026-08-01; two genuine parser gaps plus one hold-out.**
`batches/tax_TX2_ingresos_presupuesto.json` added three of its four
prepared instruments: `lfd`, `lgdp`, `reg-lfprh`. `lgdp` and `lfd` each
hit a reform-appendix heading form the parser had no case for at all —
`Decreto de reformas ` and `Ley que `/`LEY que ` respectively — fixed by
recognizing both. Regression-testing those fixes against the existing
committed corpus (137 candidate instruments, isolated in a disposable
worktree) caught a real problem before it landed: the first attempt made
the Ley heading an unconditional block-split trigger like Decreto/
Reglamento headings already are, but "Ley que" is ordinary Spanish legal
prose, and matching it unconditionally silently corrupted body text in
three already-committed instruments (`lic`, `lmv`, `ltosf`) wherever a PDF
line wrap happened to start with it. Corrected to scope the split to
`crossed_page_furniture` only — narrow enough to fix `lfd`'s real failure
(the heading and an unrelated preceding correction note shared a page
break) without the false-positive risk; a second regression pass then
came back with only 2 of 128 comparable instruments changed (`ladua`,
`linfonavit`) — both a correctness improvement, not corruption: a
transitory a prior unrecognized "Ley que ..." heading had been
misattributing to a nearby *different* decree is now correctly dated and
labeled under its own heading, with `provisions.json`/`references.json`
untouched in both. That same process surfaced a third, deeper gap in
`reg-lfprh`: two
independently-repealed glossary fractions both read the standard
placeholder "Se deroga.", and the auto-detect glossary scanner derives its
whole term/definition delimiter from whichever fraction comes first —
poisoning the delimiter for every other, correctly colon-delimited
fraction, so the fix had to be "pick the delimiter from the first
non-placeholder entry," not just "skip placeholder entries." The
regression pass also surfaced nine already-committed instruments that
independently fail a fresh re-parse on unmodified `main`
(`ccf`, `ccom`, `cff`, `cpf`, `lac`, `lamp`, `lcf`, `lfcpq`, `lins`) — a
pre-existing latent-defect population, not introduced this session and
not re-derived or re-committed as part of it; two (`ccom`, `lac`) hit a
broader Ley-heading variant (a bare law title, no "que reforma..."
phrasing) deliberately left unfixed as too risky to pattern-match safely
in the same pass. `lcmopfih` is held out
(`nested-law-in-enacting-article`,
`docs/ingestion-difficulty-log.md`): its real 15 articles and their own
five closing transitorios are enacted verbatim inside a single article of
an unrelated 1990 instrument that is itself formally titled and enacted
as a *Ley* despite being materially a reform decree, and that enclosing
instrument closes the whole document with its own, separate eight
transitorios — two transitorios sections belonging to two different
instruments, distinguishable only by position. `lif-2026`/`pef-2026` stay
blocked.
All three admitted instruments validate clean and reverse-link with 0
unresolved references (587 new edges; 1,171 new articles, 24 new original
transitories). Full finding: `docs/decisions.md` 2026-08-01.

**TX3 admitted, 2026-08-01.** `batches/tax_TX3_impuestos_aduanas.json`
added two of its three prepared instruments: `lfisan`, `reg-ladua`.
`lfisan` hit the familiar `1o.`–`9o.` ordinal case (eleventh confirmed
instance). `reg-ladua` hit a distinct, genuine article-number gap —
article 88's numeral is entirely absent from the source text, not even a
repealed-fraction placeholder — resolved by the same reviewed
`allow_article_gaps: true` adapter setting, no parser change; recorded
separately from the ordinal class since the mechanism differs even though
the fix is identical. `lisipl` is held out: a second confirmed instance
of `nested-law-in-enacting-article` the same day, in fact the simpler of
the two cases — the nested tax has no closing transitorios of its own, so
the document's single `TRANSITORIOS` section belongs unambiguously to the
enclosing instrument, unlike `lcmopfih`'s dual-transitorios ambiguity.
Both admitted instruments
validate clean and reverse-link with 0 unresolved references (54 new
edges; 295 new articles, 13 new original transitories). Full finding:
`docs/decisions.md` 2026-08-01. This closed Domain TX (TX1–TX3).

**FI1 admitted, 2026-08-02, opening Domain FI.**
`batches/financial_FI1_autoridades_pagos.json` added four of its five
prepared instruments: `lsp`, `lmeum`, `lcmm`, `ltfccg` — all four hit the
familiar `1o.`/`1º` ordinal case (twelfth through fifteenth confirmed
instances); same reviewed `allow_article_gaps: true` fix, no parser
change. `lcnbv` is held out: a new failure class,
`stale-cross-reference-to-repealed-provision`. Its article 15 cites LMV's
"artículo 16 Bis 7," verified absent from both LMV's committed corpus
text and a fresh, independent refetch of LMV's current source PDF (which
runs Artículo 16 straight to Artículo 17) — a real citation to a
provision of an already-ingested instrument that has since been repealed
or renumbered, not a wiring gap this batch can fix. `docs/decisions.md`
2026-08-02 has the full finding; `docs/ingestion-difficulty-log.md` has
the new class definition and `lcnbv`'s report. All four admitted
instruments validate clean and reverse-link with 0 unresolved references
(83 new edges; 108 new articles, 26 new original transitories).

**FI2 admitted, 2026-08-05.**
`batches/financial_FI2_banca_desarrollo.json` added all six prepared
development-bank organic laws: `lonf`, `loshf`, `lobb`, `lobnce`, `lobnosp`,
and `lobnefaa`. Five instruments hit the established `1o.`–`9o.` ordinal
case (sixteenth through twentieth confirmed instances) and received the same
reviewed `allow_article_gaps: true` adapter setting; `lobb` stayed on the
strict path, with only three reviewed Bis-suffix warnings. All six needed an
exact adapter-scoped boundary before their original enactment signatures,
removing one spurious CPEUM Article 89 edge apiece without changing parser
code. The batch adds 249 articles, 38 original transitories, and 144 resolved
references; all six validate with zero errors and bounded closure is clean.
Full finding: `docs/decisions.md` 2026-08-05.

**FI3 admitted, 2026-08-06.**
`batches/financial_FI3_seguro_rural_convenios.json` added all eight prepared
entries: `lfaar`, `lfgfaga`, `reg-lfgfaga`, `reg-lfif-art95`, `reg-lsar`,
`lmccbdc`, `lccaif`, and `lccbid`. The batch adds 337 articles, 23 original
transitories, 110 resolved references, 49 defined terms with 1,280 usages,
and 52 reform-transitory evidence records. Exact adapter-scoped running-header
and enactment-signature boundaries keep page furniture out of canonical text.
The legacy `reg-lfif-art95` identity remains tied to the abrogated LFIF named
by the official source; the corpus does not substitute LISF or infer the
regulation's present legal effect. LFGFAGA also required one focused parser
correction: the reform appendix's complete `LEY del Banco de México.` heading
now resets the containing act before its 1993 transitories. A disposable
before/after reparse found no existing provision or reference movement; LIC's
matching evidence would be correctly reattributed on a separately reviewed
future reparse, while CCF's pre-existing duplicate-evidence failure is
unchanged. All eight validate, bounded closure is clean, and the full required
gate passes with 154 workspace tests. Full finding: `docs/decisions.md`
2026-08-06.

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
- `lex-mex review-packets generate` (landed 2026-07-31) groups the 200
  committed instruments that have a `batches/*.json` manifest into 39
  packets for reviewer assignment; the 32 standards and the CNBV DCG family
  have no batch manifest and so are not yet covered by this mechanism.

Next general cluster action: normalize the next prepared cluster-2 batch
(EC1, `cl2_EC1_competencia_inversion`) into an operational manifest per the
cluster plan's admission order
(`docs/plans/cluster-2-federal-corpus-ingestion.md`). AD1–AD4 are
structurally complete, closing Domain AD; TX1–TX3 closed Domain TX; FI1–FI3
opened, continued, and completed the prepared FI sequence. The separately
authorized five-NOM Maximasa sequence does not reorder the prepared federal
batches.

## Archived divergent branches

`main` is the only active development line. The divergent `fable` worktrees
were deleted after their common superset history was retained by the annotated
tag `archive/fable-cross-linking` (peeling to
`e7ed63699f4577c78300ca379dbe431c6db1d424`). Their contents are never merged
or cherry-picked wholesale; a useful unit is reimplemented and reviewed on
current `main`.
