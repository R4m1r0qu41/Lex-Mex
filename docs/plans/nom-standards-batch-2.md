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
| NOM-001-STPS-2008 | not_started |
| NOM-004-STPS-1999 | not_started |
| NOM-005-STPS-1998 | not_started |
| NOM-006-STPS-2023 | not_started |
| NOM-009-STPS-2011 | not_started |
| NOM-010-STPS-2014 | not_started |
| NOM-011-STPS-2001 | not_started |
| NOM-015-STPS-2001 | not_started |
| NOM-017-STPS-2024 | not_started |
| NOM-018-STPS-2015 | not_started |
| NOM-019-STPS-2011 | not_started |
| NOM-020-STPS-2011 | not_started |
| NOM-022-STPS-2015 | not_started |
| NOM-024-STPS-2001 | not_started |
| NOM-025-STPS-2008 | not_started |
| NOM-026-STPS-2008 | not_started |
| NOM-027-STPS-2008 | not_started |
| NOM-029-STPS-2011 | not_started |
| NOM-030-STPS-2009 | not_started |
| NOM-033-STPS-2015 | not_started |
| NOM-035-STPS-2018 | not_started |
| NOM-036-1-STPS-2018 | not_started |

### Table 3 — SEMARNAT (3 remaining)

| NOM | Status |
|---|---|
| NOM-001-SEMARNAT-2021 | not_started |
| NOM-052-SEMARNAT-2005 | not_started |
| NOM-161-SEMARNAT-2011 | not_started |

### Table 4 — gap-analysis additions (2 remaining)

| NOM | Status |
|---|---|
| NOM-002-SEMARNAT-1996 | not_started |
| NOM-085-SEMARNAT-2011 | not_started |

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

## Next action

Begin sourcing the first instrument. No priority order is asserted within
a table; work through them as sourcing succeeds, flagging what doesn't.
