# NOM standards batch 2

Second NOM ingestion batch, staged 2026-07-28. Source list: Maximasa's
`docs/compliance/nom-register.md` (officially verified 2026-07-25/26),
minus the five instruments already canonical (NOM-002-STPS-2010,
NOM-051-SCFI/SSA1-2010, NOM-187-SSA1-SCFI-2002, NOM-247-SSA1-2008,
NOM-251-SSA1-2009 — all of Table 2). Table 5 (NADF-016/011, NOM-117,
possible NOM-036-2) is excluded: not real NOM candidates yet (CDMX-local
instruments, a reference-only norm, and an unverified designation
respectively).

**27 candidates total**, not the round 25 first estimated — see
`docs/decisions.md` 2026-07-28.

## Process

Same discipline as the general federal cluster-2 "Batch operating loop"
(`docs/project-status.md`), with one addition: a difficult instrument is
**held out and flagged, not forced through**. See
`docs/ingestion-difficulty-log.md` for the flag format and failure-class
taxonomy, and `docs/decisions.md` 2026-07-28 for why review no longer
gates each instrument.

Per instrument: locate and verify the official source (no NOM acquisition
adapter exists yet — this is manual per `docs/standards-module.md`), write
`standard-metadata.json`, extract text, `compile`, `validate`. On success,
mark `ingested` below. On a structural difficulty that isn't a quick fix,
mark `flagged` and add an entry to the difficulty log instead of compiling
it.

## Candidates

### Table 1 — STPS (22 remaining; NOM-002-STPS-2010 already canonical)

| NOM | Status |
|---|---|
| NOM-001-STPS-2008 | ingested (86 clauses, 3 transitories) |
| NOM-004-STPS-1999 | ingested (12 clauses, 3 transitories) |
| NOM-005-STPS-1998 | ingested (72 clauses, 3 transitories) |
| NOM-006-STPS-2023 | ingested (127 clauses, 3 transitories) |
| NOM-009-STPS-2011 | ingested (155 clauses, 3 transitories) |
| NOM-010-STPS-2014 | ingested 2026-07-29 (206 clauses, 6 transitorios) |
| NOM-011-STPS-2001 | ingested (105 clauses, 2 transitories) |
| NOM-015-STPS-2001 | ingested (99 clauses, 2 transitories) |
| NOM-017-STPS-2024 | ingested (51 clauses, 3 transitories) |
| NOM-018-STPS-2015 | ingested (147 clauses, 3 transitories) |
| NOM-019-STPS-2011 | ingested 2026-07-31 (94 clauses, 3 transitories, 1 non-binding guide supplement) |
| NOM-020-STPS-2011 | ingested (162 clauses, 5 transitories, 1 unconsolidated-modification warning) |
| NOM-022-STPS-2015 | ingested (83 clauses, 3 transitories) |
| NOM-024-STPS-2001 | ingested 2026-07-31 (87 clauses, 2 transitories, 2 non-binding guide supplements) |
| NOM-025-STPS-2008 | ingested (81 clauses, 3 transitories) |
| NOM-026-STPS-2008 | ingested (96 clauses, 3 transitories) |
| NOM-027-STPS-2008 | ingested (81 clauses, 3 transitories) |
| NOM-029-STPS-2011 | ingested (116 clauses, 3 transitories) |
| NOM-030-STPS-2009 | ingested (70 clauses, 3 transitories) |
| NOM-033-STPS-2015 | ingested (121 clauses, 3 transitories) |
| NOM-035-STPS-2018 | ingested 2026-07-29 (111 clauses, 2 transitorios) |
| NOM-036-1-STPS-2018 | ingested (102 clauses, 3 transitories) |

### Table 3 — SEMARNAT (3 remaining)

| NOM | Status |
|---|---|
| NOM-001-SEMARNAT-2021 | ingested (151 clauses, 7 transitories) |
| NOM-052-SEMARNAT-2005 | ingested 2026-07-31 (76 clauses, 3 transitories, 8 supplements) |
| NOM-161-SEMARNAT-2011 | ingested (86 clauses, 5 transitories) |

### Table 4 — gap-analysis additions (2 remaining)

| NOM | Status |
|---|---|
| NOM-002-SEMARNAT-1996 | ingested 2026-07-29 (73 clauses, 3 transitorios; `published_designation: NOM-002-ECOL-1996`) |
| NOM-085-SEMARNAT-2011 | ingested (73 clauses, 5 transitories) |

## Packets (deferred, not built yet)

Once a meaningful chunk of this batch is ingested, group the canonical
instruments into review packets (e.g. "industrial food processing pack")
that a colleague reviewer can be handed end-to-end, including any
backlinked parent law/regulation. Not scoped in code — staged the same way
Scope 2 was staged (`docs/plans/maximasa-legal-integration.md` M4): do not
start without a fresh planning pass. A packet needs at minimum a defined
grouping key, a reviewer-assignment record distinct from
`legal_review_status`/`technical_review_status`, and (explicitly deferred
further by the operator) a way for a reviewer to flag a missing backlink
on the fly.

## Outcome (2026-07-28)

All 27 candidates processed in two passes: **21 ingested, 6 flagged.**
Ingestion of this batch is closed; the flagged six await reviewer
direction and are recorded in `docs/ingestion-difficulty-log.md`.

| Failure class | Flagged instruments |
|---|---|
| `annex-continues-numbering` | NOM-010-STPS-2014, NOM-035-STPS-2018, NOM-024-STPS-2001 |
| `annex-form-numbering` | NOM-019-STPS-2011 |
| `indice-selected-as-body` | NOM-052-SEMARNAT-2005 |
| `metadata-ambiguity` | NOM-002-SEMARNAT-1996 |

Five of the six are clause-parser run-selection defects, and all five
validated `valid; 0 issues` while being structurally wrong — the validator
checks the selected run's internal consistency, not whether the right run
was selected. Two cheap discriminators caught every case and are worth
promoting into `validate` itself rather than staying manual triage:
clause-span coverage as a fraction of document length (the índice case
scored 0.011; every correct instrument scored ≥0.31), and whether the
selected run terminates at a Bibliografía/Concordancia heading.

One acquisition note for future batches: platiica's PDF filenames are not
derivable from the designation. NOM-010-STPS-2014's registry page links
`NOM-010-STPS-2014.pdf`, which returns an HTML error page; the real file is
`010stps2014.pdf`. Downloads must be content-type checked, not assumed.

## Resolution (2026-07-29)

Reviewer answered every flag in the Spearhead report, supplying the
governing rule: **a NOM's normative body ends at TRANSITORIOS**; trailing
apéndices, anexos, tablas and guías need different extraction rules. Both
parser defects are fixed (`docs/decisions.md` 2026-07-29), verified by
byte-identical reparse of all 26 previously-committed standards.

**24 of 27 now ingested.** NOM-010-STPS-2014 (206 clauses, was 950) and
NOM-035-STPS-2018 (111, was 124) landed under the fix.

Four remain out, for two reasons that are *not* the original flags:

| Instrument | Why it is still out |
|---|---|
| NOM-019-STPS-2011 | `transitory-absorbs-annex` — TERCERO is 46,521 chars |
| NOM-024-STPS-2001 | `transitory-absorbs-annex` — SEGUNDO is 17,094 chars |
| NOM-052-SEMARNAT-2005 | `transitory-absorbs-annex` — TERCERO is 83,226 chars, and no PRIMERO parses |
| ~~NOM-002-SEMARNAT-1996~~ | **resolved and ingested** — see `docs/decisions.md` 2026-07-29 |

The clause defects on the first three are fixed; what blocks them is a
separate, pre-existing defect found during this work, where the last
transitory swallows trailing guide/table material because
`section_end_marker` recognizes only signature markers, `APÉNDICE` and
`ANEXO`. It affects committed standards too (NOM-027's TERCERO is 22,370
chars; NOM-085's QUINTO 27,938).

## Resolution (2026-07-31)

Exact-span opaque `StandardSupplement` records close the post-transitory
modeling decision without parsing tables, forms, or inferred legal effect.
Fresh Platiica downloads matched every recorded source and extracted-text
SHA-256 before compile. NOM-019, NOM-024 and NOM-052 now pass with the
expected 94/3/1, 87/2/2 and 76/3/8 clause/transitory/supplement counts. All 27
batch candidates are canonical; the batch has no held instrument remaining.
