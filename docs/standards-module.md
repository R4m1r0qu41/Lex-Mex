# NOM/NMX standards module

Lex-Mex compiles NOM and NMX through a standards-specific trusted boundary.
They are not relabeled as statutes, regulations, or articles. This first
module is intentionally source-agnostic: official-source discovery and
acquisition remain separate from compilation until at least one reviewed
official publisher adapter establishes stable acquisition rules.

## Canonical boundary

A compiled standard directory contains:

- `standard.json`: designation, kind, issuers, domains, legal dates,
  current/cancelled/replaced status, replacement chain, the exact official
  source-file locator, whether its text is as-published or an official
  compilation, dated modification and systematic-review sources, official
  DOF and registry locators, hashes, and separate legal/technical review
  states;
- `clauses.json`: dot-numbered clauses with exact character spans into the
  unchanged extracted source text;
- `transitories.json`: ordinal-labeled blocks from the TRANSITORIOS section
  (`PRIMERO`, `SEGUNDO`, ...), addressable by exact span but deliberately
  not deeply parsed — see "Standards transitorio inspection" below;
- `extracted-text.txt`: the exact UTF-8 text used to compile and revalidate
  clause spans, retained so committed standards do not depend on an untracked
  work file;
- `validation.json`: deterministic identity, lifecycle, clause order,
  uniqueness, and source-span checks.

The external contracts are
`schemas/standard-metadata.schema.json`,
`schemas/standard-clause.schema.json`,
`schemas/standard-transitory.schema.json`, and
`schemas/standard-validation.schema.json`. Standard metadata never self-awards
`lawyer_verified` or `technical_verified`; those values record actual reviewed
state supplied through an audited future workflow.

## Compile command

Prepare metadata only from official-source facts, record the SHA-256 of both
the acquired source and unchanged extracted UTF-8 text, then run:

```bash
lex-mex standards compile \
  --metadata standard-metadata.json \
  --source official-source.pdf \
  --text official-source.txt \
  --output .work/compiled-standard
```

The command refuses an existing destination, schema/parser-version drift, or
either hash mismatch. A structurally invalid parse writes its report for
inspection and exits unsuccessfully. This makes the output useful for a
provisional-inspect-freeze ingestion sequence without representing the first
machine parse as a reviewed legal or technical conclusion.

After provisional inspection, compile the unchanged inputs into
`corpus/mx/<standard-slug>/`. A committed standard remains distinct from a
statute: its directory contains `standard.json` and `clauses.json`, never a
synthetic `instrument.json` or `provisions.json`. Revalidate it with:

```bash
lex-mex standards validate nom-251-ssa1-2009
```

Committed standards are returned by `lex-mex instruments`, accepted by
`lex-mex path` and `lex-mex search`, and supported by the canonical bundle
profile. Use `--kind standard`, `--kind clauses`, or `--kind transitories`
when requesting a specific standards path.

## Reading an official-source record before ingesting

A platiica registry record's `Historial Documental` lists several distinct
document kinds together; classify each before treating anything as a
modification (full rationale and worked example in `docs/decisions.md`,
2026-07-26):

- The NOM's own PDF — may be stale relative to later official texts.
- `Procedimientos para la evaluación de la conformidad` — a separate
  complementary instrument, not part of the NOM's text.
- `PROYECTO de Modificación` — never normative; always excluded. Identify
  it by the literal word `PROYECTO` and the absence of a vigencia date on
  the DOF page.
- `MODIFICACIÓN` — an actual normative text change.
- `ACUERDO` — the generic name for any binding DOF-published decision, not
  a fixed scope. It can be substantive (NOM-level) content, a transitorio
  date change, or both under an "modifican, adicionan y derogan" heading.
  Read its actual text every time; never infer scope from the type name.

A `MODIFICACIÓN` or `ACUERDO` states its own diff verbatim: an untouched
run of numerals or labels followed by `...` means that span is unchanged,
and whatever numeral/label follows in full is the verbatim replacement,
addition, or derogation text for that item. This is a deterministic
substitution stated by the decree itself, not Lex-Mex performing its own
legal consolidation — but it only applies when a target already exists to
apply it to (a base text, or, for a transitorio-only ACUERDO, a
transitorio provision already represented in the corpus).

`Bibliografía` splits into a real Ley/Reglamento/Acuerdo parent-authority
chain (worth backlinking once the target is committed) and non-normative
entries (ISO guides, academic citations) that are not.

## Standards transitorio inspection

`parse_standard_transitories` (in `crates/lex-parse/src/standard.rs`) turns
a standard's TRANSITORIOS section into addressable `StandardTransitory`
blocks (mirroring statutes' `ProvisionType::Transitory`, reusing the same
ordinal recognizer from `diputados.rs` — masculine/feminine forms,
`Artículo Primero`-prefixed forms, joined compounds). This is deliberately
lightweight, not a structural parse: each block's internal content (phased
criteria, tables, cross-references) stays as opaque retained text. The one
thing extracted from it is `asserted_dates` — every "N de MES de AAAA"
phrase found in the block's raw text, in order, via a plain regex scan. An
`asserted_date` is not a claim about what the date means (entry into
force, phase boundary, deadline, ...); reading the surrounding text is
still required for that.

Absence is not an error: a standard whose retained text has no
recognizable TRANSITORIOS section yields an empty `transitories.json`
(true for NOM-251, NOM-247, and NOM-187 — their retained as-published PDFs
end at appendices/annexes without ever reaching a transitorios section).

Three defects surfaced compiling this against NOM-051's real retained
text, all fixed with regression fixtures (`fixtures/standards/
indexed-transitorios-sample.txt`, `cdmx-signature-sample.txt`,
`transitorios-with-dates-sample.txt`): the índice repeats the TRANSITORIOS
heading before the real section (the same false-first-match hazard as the
Bibliografía heading for clauses — fixed by scanning candidates from the
last occurrence and requiring an actual ordinal start to follow); the
line-matching regex was passing an untrimmed line (with its leading
indentation) to the ordinal recognizer, which never matches with
leading whitespace present; and the signature-block marker recognized
only the pre-2016 "México, D.F., a ..." dateline, not the post-CDMX-
renaming "Ciudad de México, a ..." form, letting a decree's closing
signature (and its own sign-off date) bleed into the last transitorio.

**Concrete finding, now machine-visible for the first time:** NOM-051's
`transitory:segundo` still asserts `2025-10-01` as its third
implementation-phase start date — the original 2020 decree's own text,
retained unchanged. Two 2025 ACUERDOs (not part of the retained source;
see `docs/decisions.md` 2026-07-26) since pushed that date to `2028-01-01`.
Zero `standard_unconsolidated_modification` warnings is correct for clause
*text* currency; it says nothing about transitorio-date currency, and
transitorio inspection does not yet close that gap — it only makes the
staleness checkable rather than invisible. Closing it (applying an
ACUERDO's own stated date substitution to a transitorio) is Scope 2, the
decree-diff engine, not yet built.

Not every `asserted_date` belongs to the standard itself: NOM-002-STPS-2010's
`transitory:tercero` asserts `2000-09-08`, which is its predecessor
NOM-002-STPS-2000's own DOF publication date, stated verbatim in the
derogation clause ("quedará sin efectos la NOM-002-STPS-2000 ... publicada
... de 8 de septiembre de 2000") — correct extraction, not contamination,
but a reminder that `asserted_dates` records every date phrase found, not
only ones about this standard's own timing.

**Known boundary, not yet hit:** the ordinal recognizer accepts `ÚNICO`/
`ÚNICA` and any statute-style ordinal, and `validate_transitories`
deliberately does not check ordering or sequence gaps (unlike clause
validation). A retained text whose TRANSITORIOS section restarts — e.g. a
later decree's own transitorios concatenated after the base standard's,
producing `PRIMERO ... SEGUNDO ... PRIMERO ...` — will currently surface as
a `standard_transitory_duplicate` validation error rather than being split
into separate decree-scoped groups. None of the five committed standards
hit this today; it is the first failure mode Scope 2 (decree-diff engine)
will need to resolve, since that is exactly the shape a retained text
gains once a modifying decree's transitorios are appended.

## Deliberate limits

- No Maximasa NOM/NMX register entry is promoted merely because its
  designation appears plausible.
- `status: current` requires an official currency check, including
  cancellation or replacement publications where relevant.
- A current designation does not imply current clause text. As-published
  sources retain every known formal modification as an explicit
  `included_in_source: false` warning. An official compilation records each
  incorporated act as `included_in_source: true`; its unconsolidated count
  must still be zero before a consumer treats its clauses as current.
- Applicability remains downstream and fact-specific.
- Conformity-assessment text is a source fact, not a statement that a
  particular establishment must undergo it.
- Joint NOM prefixes and multiple issuing authorities remain explicit.
- Compilation does not infer applicability, legal approval, technical
  approval, or conformity-assessment duties from publication into the corpus.
