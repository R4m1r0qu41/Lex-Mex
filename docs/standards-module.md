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
  unchanged extracted source text, each optionally carrying `amended_by` —
  see "Clause-level amendment marks" below;
- `transitories.json`: ordinal-labeled blocks from the TRANSITORIOS section
  (`PRIMERO`, `SEGUNDO`, ...), addressable by exact span but deliberately
  not deeply parsed — see "Standards transitorio inspection" below;
- `supplements.json`: source-ordered, exact-span top-level appendices, annexes,
  reference guides, standalone tables, and lists following a genuine
  TRANSITORIOS section. Their internal rows, forms, and subsections remain
  opaque; see "Post-transitory supplements" below;
- `extracted-text.txt`: the exact UTF-8 text used to compile and revalidate
  clause spans, retained so committed standards do not depend on an untracked
  work file;
- `validation.json`: deterministic identity, lifecycle, clause order,
  uniqueness, and source-span checks.

The external contracts are
`schemas/standard-metadata.schema.json`,
`schemas/standard-clause.schema.json`,
`schemas/standard-transitory.schema.json`,
`schemas/standard-supplement.schema.json`, and
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
profile. Use `--kind standard`, `--kind clauses`, `--kind transitories`, or
`--kind supplements`
when requesting a specific standards path.

## Refresh command

`standards validate` reports committed derived files as stale for the current
parser; `standards refresh` rewrites them:

```bash
lex-mex standards refresh nom-247-ssa1-2008
```

It re-derives `clauses.json`, `transitories.json`, `supplements.json`, and `validation.json` from
the committed record. `standard.json` is input and `extracted-text.txt` is the
retained source; neither is ever written. The retained text is checked against
`extracted_text_sha256` first, so a refresh cannot reparse something other than
what the record claims — which is what makes a parser change backfillable
across all committed standards without re-acquiring original PDFs.

Four guards, all of which run **before any file is written** — a refused or
failed refresh leaves the committed directory exactly as it found it:

- a changed clause count aborts outright — that size of structural change is a
  parser regression to diagnose, not a file to rewrite;
- any transitory-content or supplement change aborts by default. A reviewed
  `--allow-tail-repartition` run may truncate only the final transitory while
  creating/changing its corresponding supplements; earlier transitories must
  compare deeply byte-for-byte. This permission is independent of
  `--allow-mark-change`;
- a changed transitory count aborts outright. Transitories never move the
  clause count or the marks, so a transitory-parser regression that suddenly
  returns none would otherwise be written into committed data with exit code
  0 — and `validate` could never flag it afterwards, because the corpus would
  be self-consistent. Entry-into-force dates live there;
- a change to any clause's amendment marks aborts unless `--allow-mark-change`
  is passed. Marks are a legal-meaning claim and never move the clause count,
  so nothing else in the pipeline would catch a title-parser regression that
  drops or misattributes them;
- a reparse that does not validate aborts with nothing written, rather than
  leaving invalid derived files behind a non-zero exit for a batch loop to
  miss.

## Clause-level amendment marks

A NOM's retained text is its base publication, and no official consolidated
text exists. Recording that at instrument granularity forces a reader to treat
all 252 of NOM-247's clauses as suspect when the 2011 decree touched eleven.

A modifying decree names its targets in its own DOF title, so
`StandardModificationSource` carries that title verbatim as pure input, and the
parser derives from it:

- `amended_by` on each matching `StandardClause` — the modification's index and
  the decree's own verb (`modified` / `added` / `eliminated`);
- validation warnings for every named unit that matches no committed clause.

Nothing is applied. A marked clause's text remains exactly the base
publication; the mark records that the text is **known outdated, precisely
located**, which is the opposite of a currency claim. In particular a clause
marked `eliminated` still carries its full live text — the mark records the
repeal, and consolidating it is Scope 2 Stage C.

Resolution is exact-match only, never to a nearest committed ancestor: an
unmatched target is real information (an *adición* of a numeral the base text
does not contain, an annex the corpus does not model, or a numeral that ought
to exist and does not), and attaching its mark to a parent clause would claim a
decree addressed text it never named.

Title parsing is deliberately conservative. Everything from the standard's own
identity onward is discarded first, so a designation's digits
("NOM-247-SSA1-2008") can never be read as a numeral; the remainder is
segmented at its own action verbs; a segment must carry a target noun
(`numeral`, `apéndice`, `anexo`, ...) before any token in it counts; date
phrases inside a segment ("del diverso publicado el 30 de junio de 2011") are
excluded before numeral matching, because a date's bare day or year can
collide with a real top-level clause number and stamp a false mark on an
unrelated clause; and an annex identifier must start with a genuine capital or
digit — case-insensitivity is scoped to the keyword, so "Anexo de la ..."
(prose) names nothing while "Anexo 1" does.

Both the segmenting regex and the verb→action classifier are generated from a
single `VERB_FAMILIES` table, so a family cannot be added to one and not the
other. Every family covers the same grammatical forms — nominal ("eliminación
de los numerales") and conjugated ("se eliminan los numerales") — because
segments are cut at verb matches: a form the regex cannot see is not skipped,
its targets are absorbed by the preceding family and mislabelled. An
asymmetric set where `reforman` matched but `derogan` did not would read "se
reforman los numerales 3.2 y 3.4 y se derogan los numerales 5.1 y 5.2" as one
*modified* segment and record two repeals as modifications. That is the one
failure direction amendment marks exist to prevent, and it has its own
regression test.

The marks reach consumers through `lex-mex instruments --json`: each standard
entry carries `amendment_marked_clauses` (how many clauses are known outdated)
and `published_designation` (when the registry has redesignated the standard,
so a reader is never shown a designation that appears nowhere in the record's
own retained text). Per-clause rendering waits on a standards Markdown export
profile, which does not exist yet — `collect_standard` deliberately bails on
`CanonicalMarkdown` — and is recorded as deferred, not omitted, in
`docs/plans/standards-amendment-marks.md`.

**The title form is not universal.** SSA1 publishes "Modificación de los numerales 3.2, 3.10 ...
de la Norma Oficial Mexicana NOM-247-SSA1-2008"; STPS publishes "ACUERDO de
Modificación a la Norma Oficial Mexicana NOM-020-STPS-2011, ..." naming no
numeral at all. The second yields no targets, and the validator says the scope
is unknown rather than that nothing was affected — a title recorded that names
nothing is a distinct fact from a title never recorded, and both are reported
separately.

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

## Post-transitory supplements

`StandardMetadata.supplement_starts` records reviewed exact anchors and kinds,
in source order. Multi-line anchors distinguish duplicate visible headings
(including NOM-036-1's repeated `GUÍA DE REFERENCIA I`). The parser resolves
each anchor exactly once after the real TRANSITORIOS heading, ends the final
transitory at the earliest applicable closing-signature marker or first
anchor, excludes closing furniture, and slices each supplement to the next
configured anchor or retained-text end. An inline reference to a table or
guide is never an implicit boundary.

Each `StandardSupplement` carries a one-based sequence, kind, collapsed
heading, exact text and character span. Legal character is derived only from
explicit source language: a Normativo/No Normativo heading or a statement
such as `no es de cumplimiento obligatorio`. Conflicting explicit signals are
an error; absence is the non-blocking
`standard_supplement_character_unspecified` warning. Kind alone never implies
normativity. `standards validate` reparses and deeply compares the required
`supplements.json`, and canonical bundles require it for every standard.

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
  `included_in_source: false` warning, located to specific clauses via
  `amended_by` where the decree's own title names them (see "Clause-level
  amendment marks"). An official compilation records each incorporated act as
  `included_in_source: true`; its unconsolidated count must still be zero
  before a consumer treats its clauses as current.
- An `amended_by` mark locates staleness; it never resolves it. No committed
  clause text has ever had a modification applied to it, including clauses a
  decree marked `eliminated`.
- Applicability remains downstream and fact-specific.
- Conformity-assessment text is a source fact, not a statement that a
  particular establishment must undergo it.
- Joint NOM prefixes and multiple issuing authorities remain explicit.
- Compilation does not infer applicability, legal approval, technical
  approval, or conformity-assessment duties from publication into the corpus.

## Redesignated standards

A Mexican norm's prefix names its issuing authority, and authorities are
occasionally reorganized. Two different things can follow, and only one is
recorded:

- **The registry redesignates the instrument.** NOM-002-SEMARNAT-1996's own
  retained text is titled NOM-002-**ECOL**-1996 and names the Secretaría de
  Medio Ambiente, Recursos Naturales y Pesca (SEMARNAP). `designation`
  carries the current registry form and `published_designation` preserves
  the published one, which raises a `standard_redesignated` warning. Without
  that field the record would assert a designation appearing nowhere in its
  own source text, silently breaking an invariant that otherwise holds for
  every committed standard.
- **The registry keeps the historical prefix.** SCFI persists in
  NOM-051-SCFI/SSA1-2010 and NOM-187-SSA1/SCFI-2002 even though the
  Secretaría de Comercio y Fomento Industrial became the Secretaría de
  Economía. Nothing is recorded: the designation is unchanged, and only the
  authority's name moved.

`published_designation` is therefore not a general "former name" field. It
is recorded only when the registry's designation and the published
designation actually differ, and `validate_metadata` rejects it when the two
are equal.
