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
- `annex-form-numbering` — a numbered fill-in-form annex (a "Guía de
  Referencia"-style annex/format with fields like `1.`, `1.1`, `2.`, `2.1`
  ...) out-competes the standard's real numbered body for the clause
  parser's run-selection heuristic (see the log entry below for the
  mechanism).

Add a new class here the first time it's seen; do not invent a class for a
single one-off unless it plausibly recurs.

## Log

### nom-019-stps-2011 — annex-form-numbering — 2026-07-28

What's difficult: `parse_standard_clauses` (`crates/lex-parse/src/standard.rs`)
finds every line matching a numbered-heading regex, then picks the
*longest consecutive run* of numbered headings starting at clause `0` or
`1` (`numbered_body_run`, `max_by_key(|(selected, _)| selected.len())`).
NOM-019-STPS-2011's real body has 15 top-level clauses (Objetivo, Campo de
aplicación, ... Concordancia) plus a handful of dotted subclauses (3.1,
4.1–4.x). But the document also carries a "Guía de Referencia I" annex —
an investigation-report fill-in form with its own independent numbering
(`1.` Identificación del centro de trabajo, `1.1` RAZON SOCIAL, `1.3`,
`1.6`, `1.8`, `2.` Datos del trabajador, `2.1`, `2.3`, ... continuing
through `4.6`). That form's field count outnumbers the real body's clause
count, so the run-selection heuristic picks the *annex form* as "the"
body. Result: `clauses.json` compiled to 365 entries, all validator checks
passed (0 issues, `valid: true`), but every real substantive clause
(Objetivo, Campo de aplicación, Obligaciones del patrón, etc.) is silently
absent — the validator checks internal consistency of whatever run got
selected, not whether the *right* run was selected. This is a different
mechanism from the three signature-block-bleed fixes (2026-07-27): those
were about a single wrong match squeezing into an otherwise-correct run;
this is the run-selection heuristic itself choosing entirely the wrong
run because a longer numbered sequence exists elsewhere in the same text.

What was tried: downloaded and hashed the official platiica PDF
(`019stps11.pdf`, DOF pub. 2011-04-13), extracted text with `pdftotext
-layout`, wrote metadata, ran `standards compile` into `.work/` only (not
copied into `corpus/`, per the hold-out policy). Confirmed via
`grep`/manual inspection that the real 15-clause body is present in the
extracted text and simply wasn't selected. Confirmed the mechanism, not
just hypothesized it: the 365 selected clauses span byte offsets
66508–111539 of a 113234-byte text — entirely inside the annex-form
region, after both the índice (~byte 2000) and the real body (~byte
3000). So this is genuinely the run-selection heuristic picking the wrong
run wholesale, not the índice/real-section collision class already fixed
for transitories on 2026-07-27 (ruled out explicitly, not assumed).

Status: open. Not committed to `corpus/`. One remediation lead, not a
prescription: prefer a candidate run that reaches a plausible terminal
heading (`is_bibliography_heading` already special-cases this for the
single-heading case; NOM-019's índice lists both "14. Bibliografía" and
"15. Concordancia con normas internacionales" as the real body's actual
end) over raw run length. A "bound candidates to before the first
TRANSITORIOS marker" idea was considered and rejected: NOM-019's índice
itself lists `TRANSITORIOS` before the annex, so cutting there would
truncate the real body to nothing — the same índice-vs-real-section
ambiguity Scope 1 already solved for transitories would have to be reused
here too, not reinvented.
