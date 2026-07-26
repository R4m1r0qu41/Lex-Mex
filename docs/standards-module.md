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
- `extracted-text.txt`: the exact UTF-8 text used to compile and revalidate
  clause spans, retained so committed standards do not depend on an untracked
  work file;
- `validation.json`: deterministic identity, lifecycle, clause order,
  uniqueness, and source-span checks.

The external contracts are
`schemas/standard-metadata.schema.json`,
`schemas/standard-clause.schema.json`, and
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
profile. Use `--kind standard` or `--kind clauses` when requesting a specific
standards path.

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

## Known gap: transitorios have no structured representation

`StandardClause` parsing stops at the `TRANSITORIOS` marker (or an
`APÉNDICE`/`ANEXO` heading first); transitorio text is retained in
`extracted-text.txt` but is not a clause, has no ID, and is not queryable.
`StandardModificationSource` has no notion of one ACUERDO superseding
another, or of a phase/transitorio's effective date changing independent
of clause text. Found while attempting a NOM-051 pass under the reading
procedure above: two 2025 ACUERDOs push its 2020 modification's final
implementation phase from 2025-10-01 to 2028-01-01, and neither is
represented anywhere in the committed corpus or its
`standard_unconsolidated_modification` warning — the zero-warning state
is correct for clause *text* but says nothing about transitorio currency.
Not yet resolved; a schema/parser design is needed before it can be.

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
