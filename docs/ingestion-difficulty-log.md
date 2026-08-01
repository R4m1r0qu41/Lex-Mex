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
  ...) whose numbering *restarts* independently, out-competing the
  standard's real numbered body on raw length for the clause parser's
  run-selection heuristic (see `nom-019-stps-2011` below).
- `annex-continues-numbering` — distinct from the above: the real body is
  selected correctly from clause 1, but the run does not *stop* at the
  terminal heading (Bibliografía/Concordancia). A following annex, table,
  or questionnaire whose rows continue the same numeric sequence is
  absorbed as if those rows were clauses. Severity scales with the annex:
  one stray table row (`nom-024-stps-2001`) to 744 phantom clauses
  (`nom-010-stps-2014`).
- `transitory-absorbs-annex` — **found 2026-07-29 while fixing the clause
  side; pre-existing, not introduced by that fix.** `section_end_marker`
  ends the transitorios section at a signature marker, `APÉNDICE`, or
  `ANEXO` only. Trailing material introduced by any other heading — most
  commonly `Guía de Referencia I`, but also bare `Tabla N.` /
  `TABLAS:` / `LISTADOS:` — is not recognized, so the *last transitory*
  swallows it. Affects committed standards (NOM-027's TERCERO is 22,370
  chars; NOM-085's QUINTO is 27,938; NOM-020's QUINTO is 15,327) and is
  why NOM-019, NOM-024 and NOM-052 remain held out. Not fixed in the same
  pass deliberately: adding `Tabla` as a section-end marker could truncate
  legitimate transitory text that references a table inline, and the
  reviewer has separately indicated post-transitorios annexes need their
  own extraction approach — so *how* to model them is an open design
  question, not a marker list.
- `indice-selected-as-body` — the índice (table of contents) is itself a
  short, complete, consecutive numbered run starting at 1, so it can win
  run-selection outright and be compiled as the entire clause body,
  leaving the real body unparsed. Detectable by clause-span coverage: the
  selected run covers a tiny fraction of the document
  (`nom-052-semarnat-2005`, 1.1%). This is the clause-side analogue of the
  índice/real-section ambiguity Scope 1 already solved for transitories.

Add a new class here the first time it's seen; do not invent a class for a
single one-off unless it plausibly recurs.

## Resolved 2026-07-29

Reviewer supplied the governing domain rule: **a NOM's normative numbered
body ends at TRANSITORIOS.** What follows may be apéndices, anexos, tablas
or listados (sometimes normative) or an explicitly non-binding "Guía de
Referencia", but it is never clause-structured. Two parser changes followed
(`docs/decisions.md` 2026-07-29):

1. The clause run is bounded at the real, índice-disambiguated TRANSITORIOS
   heading, reusing Scope 1's locator (`real_transitorios_heading`) so the
   clause and transitory paths can never disagree about where a body stops.
   Closes `annex-continues-numbering` and `annex-form-numbering`.
2. Form feed (`\x0c`) joins the leading-whitespace class. `pdftotext` emits
   a page break with no following newline, so a heading landing on a page
   boundary never matched the line-start anchor. Closes
   `indice-selected-as-body` — NOM-052's real body was not out-competed by
   its índice, it was *invisible*, leaving the índice as the only candidate.

`validate_standard` now emits `standard_trailing_material` when substantial
text follows the transitorios section, so a compiled standard cannot imply
completeness while omitting normative annexes. It fires on 13 standards.

Fixtures: `page-break-heading-sample.txt`,
`post-transitorios-annex-sample.txt`.

## Log

### standard-post-transitory-supplements — resolved 2026-07-31

The `transitory-absorbs-annex` class is closed by exact configured top-level
supplement anchors and a shared tail layout. All 29 committed standards were
inventoried; 23 carry represented supplements and six carry the required
empty file. The inventory found seven previously silent swallowed-guide cases
behind unrecognized `Dado en…` closings in addition to the known warning set.
No clause changed; every earlier transitory stayed exact; 11 final
transitories lost only closing/supplement material. NOM-019, NOM-024 and
NOM-052 were then eligible for their held-out ingestion gate.

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

Status: **clause defect closed 2026-07-29.** Now parses to 94 clauses,
tops 1–15 terminating at `15. Concordancia`. The resolution was the
rejected idea, done correctly: bounding at TRANSITORIOS *does* work once
the índice occurrence is disambiguated from the real one — which is
exactly the Scope 1 locator this entry predicted would have to be reused.
NOM-019 has TRANSITORIOS at bytes 4879 (índice) and 64447 (real); the
annex form begins at 66508, after the real section.

**Still held out** on `transitory-absorbs-annex`: its TERCERO transitory
runs to 46,521 chars, having swallowed the whole Guía de Referencia I
(the reviewer confirms this guide is explicitly *not* binding — "no es de
cumplimiento obligatorio" — so it should not be inside a transitory).

### nom-010-stps-2014 — annex-continues-numbering — 2026-07-28

What's difficult: the real body parses correctly as clauses 1–20
(`1. Objetivo` … `20. Concordancia con normas internacionales`), but the
run continues straight past the terminal heading into Apéndice I's
chemical-substances table, whose rows are numbered `21.`, `22.`, … `764.`
in the same continuous sequence. Those 744 table rows are compiled as
top-level clauses: `764. Yoduro de metilo | Daño a ojos…` is recorded as
a clause of the standard. Result: 950 clauses instead of ~150, validating
clean (0 issues) because the validator checks internal consistency of the
selected run, not whether the run should have ended.

What was tried: compiled to `.work/` only. Note the acquisition detail —
the platiica PDF for this standard is at `010stps2014.pdf`, not the
`NOM-010-STPS-2014.pdf` pattern the registry page links (that URL returns
an HTML error page, not a PDF).

Status: **closed 2026-07-29.** Ingested at 206 clauses (tops 1-20, terminating at `20. Concordancia`), 6 transitorios, carrying a `standard_trailing_material` warning for Apéndice I. Fixed by bounding the clause run at TRANSITORIOS.

### nom-035-stps-2018 — annex-continues-numbering — 2026-07-28

What's difficult: same mechanism as `nom-010-stps-2014`, smaller. Real
body parses correctly as clauses 1–13 (`1. Objetivo` …
`13. Concordancia con normas internacionales`), then the Guía de
Referencia questionnaire's items continue the sequence as `14.` … `26.`
and are absorbed as clauses — e.g. `26. Puedo decidir cuánto trabajo
realizo durante la jornada`. Two of the absorbed rows (`14`, `17`) are
byte-identical questionnaire items, which is itself a signal they are
not clauses.

What was tried: compiled to `.work/` only.

Status: **closed 2026-07-29.** Ingested at 111 clauses (tops 1-13, terminating at `13. Concordancia`), 2 transitorios, carrying a `standard_trailing_material` warning for the Guía de Referencia. Fixed by bounding the clause run at TRANSITORIOS.

### nom-024-stps-2001 — annex-continues-numbering — 2026-07-28

What's difficult: the mildest instance of the class — the body parses
correctly through `12. Concordancia con normas internacionales`, then a
single numeric table row is absorbed as clause `12.5`, with a label
consisting only of figures (`0.024  0.025  0.0…`). One phantom clause, not
744, but a clause that does not exist in the standard would still enter
canonical data.

What was tried: compiled to `.work/` only. Detected by a label-content
check (a clause label containing no alphabetic character), not by the
validator, which reports 0 issues.

Status: **clause defect closed 2026-07-29** (the `12.5` table row is now excluded — the body bounds at TRANSITORIOS). **Still held out** on `transitory-absorbs-annex`: its SEGUNDO transitory runs to 17,094 chars, having swallowed the reference guide on vibration-exposure calculation and its untitled frequency-band table (whose headings are `Número de Banda de Frecuencia | Frecuencia Central (Hz) | Factor de Ponderación`, which is why the absorbed row read as bare numbers).
### nom-052-semarnat-2005 — indice-selected-as-body — 2026-07-28

What's difficult: the compiled `clauses.json` contains exactly 11 clauses
spanning bytes 9444–9918 of a 122,575-byte document — 1.1% coverage. That
span is the índice, not the body: the parser selected the table of
contents (`1. Introducción` … `11. Vigilancia de esta Norma`, each a
single line) as the standard's entire clause structure. Every substantive
provision, including the hazardous-waste listings this standard exists
for, is absent. Validates clean (0 issues) for the same reason as the
entries above.

What was tried: compiled to `.work/` only. Surfaced by a clause-span
coverage check (selected-run byte span ÷ document length), which is a
cheap and apparently reliable discriminator for this class — every other
instrument in this batch scored ≥0.31.

Status: **clause defect closed 2026-07-29** (the form-feed fix reveals the real body: 76 clauses, tops 1-11, 55% coverage, starting at the true `1. Introducción`). **Still held out** on `transitory-absorbs-annex`: its TERCERO transitory runs to 83,226 chars, having swallowed the Tablas, Listados 1-5, Figura and Anexo 1 — which the reviewer confirms are **normative**, not a guide. It also parses only SEGUNDO and TERCERO, with no PRIMERO, which needs checking before ingestion.
### nom-002-semarnat-1996 — metadata-ambiguity — 2026-07-28

What's difficult: not a parser problem — the text parses cleanly (72
clauses, 3 transitories, 0 issues, 78% coverage, terminating correctly).
The problem is identity. The official retained text is titled **NOM-002-
ECOL-1996**, issued by the *Secretaría de Medio Ambiente, Recursos
Naturales y Pesca* (SEMARNAP). The platiica registry indexes the same
instrument as **NOM-002-SEMARNAT-1996** under SEMARNAT. The ECOL→SEMARNAT
redesignation follows the SEMARNAP→SEMARNAT reorganization and is
conventionally treated as the same instrument, but this repository does
not resolve a legal-identity question by convention: committing it would
mean asserting a `designation` that appears nowhere in its own retained
source text, and `standard.json` has no field to record that the
published designation differs from the current registry designation.

What was tried: compiled to `.work/` only, with `designation` set to the
registry form and `publisher`/`issuing_authorities` set to the SEMARNAP
form actually named in the text — an inconsistency that is itself the
evidence this needs a decision.

Status: **closed 2026-07-29 — ingested.** The reviewer granted authority to
apply the rename where the registry shows one, and to leave the prefix alone
where it does not — SCFI persists in NOM-051 and NOM-187 even though the
Secretaría de Comercio y Fomento Industrial became the Secretaría de
Economía, whereas ECOL did become SEMARNAT. So the designation follows the
*registry's* redesignation, not the authority's rename.

Verified rather than assumed, per the reviewer's "run that diff": across the
whole committed corpus, every standard's `designation` appears in its own
retained text. NOM-002 would have been the first exception, so applying the
rename silently would have broken a corpus-wide invariant with nothing
recording it. `StandardMetadata` gained an optional `published_designation`
(schema, type, validator, test and `docs/standards-module.md` together): it
records `NOM-002-ECOL-1996`, raises a `standard_redesignated` warning, and
the validator errors if it is set when the two designations are equal, so it
cannot drift into a general "former name" field.

Ingested at 73 clauses, 3 transitorios — all small, no annex absorption, so
unlike the other three it was not blocked by `transitory-absorbs-annex`. See
`docs/decisions.md` 2026-07-29.
