# R2 — temporal and source holds queue (2026-10-25)

Instrument-specific queue for R2 of [post-split corpus and query repair](plans/post-split-corpus-and-query-repair.md),
built from the committed corpus, adapters, validation and review evidence at the tip of the local branch
`repair/itf-dcg-heading-separator-2026-10-25` (`ef28b9619`, after R0 and R1). **Read-only: no temporal state, review
state or corpus file was changed to produce it**, and nothing here fills a commencement globally or approves a review by
machine. Review tier: advisor only. Evidence: [R2 evidence](post-split-r2-queue-2026-10-25-evidence.json).

## Counts kept distinct

These measure different things and must not be added together or read as progress on each other.

| Measure | Count | Source |
| --- | ---: | --- |
| Active structural-debt rows / retired | 0 / 17 | `_known-structural-debt.json` |
| Historical missing inputs not recovered as exact pairs | 0 of 106 (4 changed editions; with `cub-dcg-2005` 5, of which 3 ingested, 2 held) | R0 acquisition |
| Closure accepted gaps | 3 (`cub-dcg-2005`, `lftr`, `socap-sofipo-dcg-2006`) | `_closure-accepted-gaps.json` |
| Legal operator reviews / outdated-reference approvals (audited, untouched) | 16 / 17 | database projection |
| Pending review items (`review list`) | 0 | CLI |
| Provisions by review status: `not_analyzed` / `machine_accepted` / `lawyer_verified` / `review_required` | 48,062 / 20,302 / 2 / 1 | corpus |
| Provisions by temporal status: `unknown` / `effective` / `repealed` / `partially_repealed` | 46,742 / 16,073 / 5,550 / 2 | corpus |
| Instruments whose every provision is `unknown` | 246 of 468 | corpus |

## Queue A — deterministic metadata and source events (no legal ruling needed)

1. **Instruments whose provisions are all `unknown`: 246.** `derive-temporal` was run on a copy-on-write clone of each and its
   reported outcome bucketed (the clone was discarded):

| Instruments | Outcome of `derive-temporal` | First examples |
| ---: | --- | --- |
| 143 | commencement resolved via day_after_publication (N-N-N); N repealed, N classified from commencement | `laem`, `lagra`, `lansi`, `larcp` … |
| 29 | commencement skipped (multiple-commencement-clauses: N ordinary transitories mention entering into force) | `cnpcf`, `cnpp`, `laassp`, `lapp` … |
| 17 | commencement resolved via relative_days (N-N-N); N repealed, N classified from commencement | `ldpam`, `lfcp`, `lfmzaah`, `lfpccs` … |
| 16 | commencement skipped (no-commencement-clause: no ordinary transitory mentions entering into force) | `lcaaa`, `lfca`, `lfgfaga`, `lse` … |
| 10 | commencement skipped (unrecognized-commencement-pattern: commencement clause found but its shape is not recognized) | `ldsc`, `lfrpe`, `lga`, `linbal` … |
| 6 | commencement skipped (exception-or-conditional-language: commencement clause carries a qualifier: "salvo") | `lgmde`, `lif-2026`, `lraf`, `lsnieg` … |
| 5 | commencement skipped (business-days-period: commencement period is in business days) | `lbogm`, `lfea`, `ligie`, `reg-lfppi` … |
| 3 | commencement skipped (exception-or-conditional-language: commencement clause carries a qualifier: "sin perjuicio") | `lamn`, `lgcs`, `reg-lfiiedb` |
| 3 | commencement resolved via same_day_as_publication (N-N-N); N repealed, N classified from commencement | `lamni`, `lfm`, `lgsmime` |
| 3 | commencement resolved via literal_date (N-N-N); N repealed, N classified from commencement | `lcmopfih`, `lcnbv`, `lnac` |
| 2 | commencement skipped (unsupported-relative-period-unit: commencement period unit "mes" is not computed) | `lfdc`, `reg-lfpc-art122` |
| 2 | commencement skipped (exception-or-conditional-language: commencement clause carries a qualifier: "gradual") | `lisf`, `lnmasc-penal` |
| 1 | urn:lex-mx:federal:statute:lexp (no ordinary transitories); N repealed | `lexp` |
| 1 | urn:lex-mx:federal:statute:lftr (no ordinary transitories); N repealed | `lftr` |
| 1 | urn:lex-mx:federal:statute:lisipl (no ordinary transitories); N repealed | `lisipl` |
| 1 | commencement skipped (exception-or-conditional-language: commencement clause carries a qualifier: "con excepción") | `lissste` |
| 1 | urn:lex-mx:federal:statute:lrascap (no ordinary transitories); N repealed | `lrascap` |
| 1 | urn:lex-mx:federal:statute:lscs (no ordinary transitories); N repealed | `lscs` |
| 1 | urn:lex-mx:federal:statute:lsrlip (no ordinary transitories); N repealed | `lsrlip` |

   **Reading the table.** 166 of the 246 (143 + 17 + 3 + 3) already have a commencement clause that the existing deterministic
   rule resolves; the corpus simply never had that step applied to them, so their provisions stay `unknown`. 74 are skipped for a
   named reason (29 multiple commencement clauses, 16 no clause, 10 unrecognized pattern, 9 qualifier language such as "salvo",
   5 business-day periods, 2 unsupported relative unit, 2 "gradual", 1 "con excepción"), and 6 have no ordinary transitories
   (`lexp`, `lftr`, `lisipl`, `lrascap`, `lscs`, `lsrlip`). Applying the rule to the 166 would write `deterministic_rule`
   temporal states (confidence 0.9 to 1.0, not legal review) to thousands of provisions. That is a bulk temporal-state
   change; it is queued and **was not applied** (no global commencement fill without an explicit instruction).

   A skip for an unresolvable commencement is a code-or-source fact about that instrument, not a legal conclusion; each bucket
   is a separate deterministic work item (a missing commencement clause, a period in business days, an unsupported form).
2. **`oga`:** the adapter has no exact publication day, so the temporal request stops ("requires an exact publication day;
   serialized range is not a day") and all 1,672 provisions are `unknown`. Needs the exact DOF day from the source.
3. **`latest_reform_date` missing or behind the newest decree evidence (9):** `cpeum` (null, newest 2026-06-02), `lgs` (null,
   2026-01-15), `lmeum` (null, 2026-01-19), `lcndh` (null, 2024-04-01), `lffmaa` (null, 2023-03-27), `cfpc` (2021-06-07 vs
   2023-06-07), `lfpc` (2025-12-12 vs 2025-12-23), `reg-lspee-ma` (2011-12-16 vs 2025-10-03), `ligie` (2025-12-29 vs 2026-10-13,
   the publisher header lags its own text). Source-reading metadata.
4. **Held source editions:** `cub-dcg-2005` (legend repeats markers 325 and 326) and `socap-sofipo-dcg-2006` (legend omits
   entry 91 while the text uses mark (91)). Either a disclosed rule for identical duplicates (cub) or a corrected source.
5. **Stale committed records:** `cjm` (+2 articles, `30 Sextus`, `30 Séptimus`) and `lftr` (+3, `180 Quintes/Sextus/Séptimus`)
   split headings the parser already recognizes; `scap-dcg-2012` keeps an inline `(261)` the current parser removes (held
   marginal-number drift); `lrascap` has its 15 law transitories inside article 132 and needs an enactment-layer ID scheme.
6. **Static agent pages and the two goldens** are R3 and fixture decisions, not R2.

## Queue B — legal interpretation (ask JRH only for these, citing the exact text and the formal DOF act)

| Item | Exact question | Why it is not deterministic |
| --- | --- | --- |
| `lscs` 16 extracted reform transitories | Which of the 2006 (5) and 2013 (3) transitories govern LSCS? The 2006 decree amends three laws (its SEGUNDO and QUINTO concern LGISMS); the 2013 decree issues the LISF and the compilation reproduces only the LSCS-relevant ones | Applicability of an omnibus decree's transitories |
| `lscs` article 207 | Confirm it is an in-force provision that repeals other law (now `unknown`, formerly `repealed` by the deterministic rule) | Self-repeal versus repeal of other law |
| `itf-dcg-2018` transitory Séptimo | `review_required`, `repealed`, basis `llm_inference` | Existing open review; unchanged |
| `lritf` transitory Séptima, `ifpe-dcg-2021` transitory Cuarto | `lawyer_verified`; none | Audited decisions; do not overwrite |
| 16 legal operator reviews, 17 approvals | none | Audited; preserved |

## Gate for this queue

Every touched legal state must keep reviewer, date, rationale, source and the prior proposal. None was touched. Unresolved
items stay held. R2 is a queue, not a result: no item above is closed by this document.
