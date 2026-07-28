# Ingestion difficulty log

A durable, cross-batch record of ingestion obstacles that were flagged and
held out rather than resolved on the spot. Distinct from a plan's own
`Progress`/`Surprises and discoveries` sections (which are per-plan and
close when the plan closes): this log exists so the *same failure class*
recurring across unrelated batches — NOM standards now, state/municipal
corpora later — is visible at a glance instead of buried in per-plan prose.

## Policy (set 2026-07-28)

When an instrument being ingested hits a structural difficulty that isn't a
quick, obviously-correct fix — a new signature-block/heading-collision
variant, a modifying decree whose text doesn't cleanly integrate, an
ordinal-restart, an acquisition source that can't be resolved — it is **held
out of `corpus/` entirely**, not compiled with the defect present. A report
goes here instead. This keeps a known structural defect from ever entering
committed canonical data; Maximasa's bundle lock hashes and consumes
`corpus/` directly, so anything wrong there is wrong downstream.

Ordinary parser bugs that get fixed immediately (the normal case — see
`docs/project-status.md` "Batch operating loop") still get a regression
fixture and don't need an entry here. This log is specifically for
difficulties that are *not* resolved before moving to the next instrument.

Review of ingested-but-not-yet-legally-reviewed material is a separate
question from this log — see `docs/decisions.md` 2026-07-28 for the
packet-based review policy.

## Failure classes seen so far

- `acquisition` — no adapter exists for the source; the official text has
  to be manually located/verified (platiica, DOF, or a registry mirror),
  and that sourcing step itself is ambiguous, rate-limited, or blocked
  (e.g. `dof.gob.mx`'s TLS behavior — see `nom-register.md` "How this was
  verified").
- `decree-diff` — a MODIFICACIÓN/ACUERDO decree changes a standard's text
  via ellipsis-diff ("unchanged span ... replacement in full") that isn't
  integrated into the retained source; needs the staged Scope 2 engine
  (`docs/plans/maximasa-legal-integration.md` M4).
- `transitory-ordinal-restart` — a modifying decree's own transitorios get
  appended after the base standard's, restarting the ordinal sequence
  (`PRIMERO...SEGUNDO...PRIMERO`); currently surfaces as a hard
  `standard_transitory_duplicate` validation error rather than splitting
  cleanly (flagged `docs/standards-module.md` 2026-07-27, not yet hit for
  real).
- `signature-block-bleed` — a decree's closing signature/dateline is
  mis-recognized as body text (three prior instances fixed with fixtures:
  índice heading collision, untrimmed-indentation ordinal miss, post-2016
  CDMX dateline format — see `docs/decisions.md` 2026-07-27).
- `metadata-ambiguity` — conflicting official records (e.g. a systematic
  review result vs. an actual successor) that need a judgment call before
  `standard-metadata.json` can be written correctly (see NOM-187's
  2023-record handling, `nom-register.md`).

Add a new class here the first time it's seen; do not invent a class for a
single one-off unless it plausibly recurs.

## Log

*(empty — no instrument has been held out yet under this policy. Entries
below follow this shape:)*

```
### <instrument-id> — <failure class> — <date flagged>

What's difficult: <concrete description of the actual text/structure that
doesn't fit>.

What was tried: <if anything>.

Status: open | closed (link to the decisions.md entry / commit that
resolved it, and which log entries it closes)
```
