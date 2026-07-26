# NOM/NMX standards module

Lex-Mex compiles NOM and NMX through a standards-specific trusted boundary.
They are not relabeled as statutes, regulations, or articles. This first
module is intentionally source-agnostic: official-source discovery and
acquisition remain separate from compilation until at least one reviewed
official publisher adapter establishes stable acquisition rules.

## Canonical boundary

A compiled standard directory contains:

- `standard.json`: designation, kind, issuers, domains, legal dates,
  current/cancelled/replaced status, replacement chain, official DOF and
  registry locators, hashes, and separate legal/technical review states;
- `clauses.json`: dot-numbered clauses with exact character spans into the
  unchanged extracted source text;
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

## Deliberate limits

- No Maximasa NOM/NMX register entry is promoted merely because its
  designation appears plausible.
- `status: current` requires an official currency check, including
  cancellation or replacement publications where relevant.
- Applicability remains downstream and fact-specific.
- Conformity-assessment text is a source fact, not a statement that a
  particular establishment must undergo it.
- Joint NOM prefixes and multiple issuing authorities remain explicit.
- The module does not yet publish standards into `corpus/mx/`; that step
  follows a real official-source ingestion and representative output review.
