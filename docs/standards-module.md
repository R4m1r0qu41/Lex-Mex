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
  source-file locator, whether its text is as-published or officially
  consolidated, dated modification and systematic-review sources, official
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

## Deliberate limits

- No Maximasa NOM/NMX register entry is promoted merely because its
  designation appears plausible.
- `status: current` requires an official currency check, including
  cancellation or replacement publications where relevant.
- A current designation does not imply current clause text. As-published
  sources retain every known formal modification as an explicit
  `included_in_source: false` warning; only an official compilation may use
  `text_basis: official_consolidated`.
- Applicability remains downstream and fact-specific.
- Conformity-assessment text is a source fact, not a statement that a
  particular establishment must undergo it.
- Joint NOM prefixes and multiple issuing authorities remain explicit.
- Compilation does not infer applicability, legal approval, technical
  approval, or conformity-assessment duties from publication into the corpus.
