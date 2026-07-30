# Standards amendment marks (Scope 2, Stage A)

Planning pass, 2026-07-29. **Not started — requires operator sign-off on the
trusted-boundary shape below**, per the M4 rule governing Scope 2
(`docs/plans/maximasa-legal-integration.md`).

## Problem

A NOM's retained text is the base publication. Later DOF decrees modify it,
and no official consolidated text exists — unlike Cámara de Diputados
statutes and CNBV disposiciones, where the publisher issues a compiled
version Lex-Mex can point at and hash.

Today that is recorded at instrument granularity:
`standard_unconsolidated_modification` says *this standard has an
uncorporated modification*, which forces a reader to treat all 252 of
NOM-247's clauses as suspect when the 2011 decree touched eleven of them.

## The cheap lever

A modifying decree names its targets in its own title. NOM-247's two:

> Modificación de los numerales **1.4, 2, 3.2, 3.10, 3.12, 3.17, 3.18, 3.19,
> 3.36, 3.44 y 8** de la Norma Oficial Mexicana NOM-247-SSA1-2008…

> Modificación de los numerales **3.2, 3.10, 3.33, 4, 5.1.1, 5.2.7.ii.1)**,
> adición del numeral **5.1.5** y eliminación de los numerales **5.2.2.8,
> 5.2.3.4, 5.2.4.5** y el **Apéndice normativo A** de la Norma Oficial
> Mexicana NOM-247-SSA1-2008…

Those numerals are clause identifiers Lex-Mex already addresses. Parsing them
turns an instrument-level warning into a clause-level one **without applying
any text change**, without a consolidation engine, and without depending on
the unresolved `transitory-absorbs-annex` defect.

## Scope

**In scope.** Recording *which* clauses a decree affects, *what* it does to
them, and *where* the decree is.

**Out of scope, explicitly.** Applying the modification. Producing a
consolidated text. The `derived_consolidation` text basis. Anything that
requires reading a decree's body rather than its title. Those are Stage C and
sit on top of the annex problem — the second decree above eliminates
`Apéndice normativo A`, which the corpus does not model at all and which
`transitory-absorbs-annex` currently swallows into a transitory.

## Prerequisite: the existing model is half-built

`StandardMetadata` has no amendment fields, but the statute/DCG side does —
`amendment_marks: Vec<u32>` on a provision, resolving through
`amendment_references: Vec<AmendmentReference>` (`marker`, `description`) on
the instrument, already rendered into Markdown frontmatter.

**Correction, 2026-07-30.** The original version of this section (below,
quoted for the record) claimed the legend was empty everywhere. That was
measured against the wrong field and never checked against the actual
`amendment-references.json` sibling file each instrument carries. It is
wrong and retracted, not refined.

> That legend is empty everywhere it is used — 1,844 committed provisions
> carry markers that resolve to nothing, and not one instrument has a single
> legend entry. Decide before implementing: populate the CNBV legend from the
> compiled documents' REFERENCIAS sections, or record explicitly why it
> stays empty.

**The corrected measurement**, against the real file, across all nine
instruments plus the tenth CNBV instrument `ifpe-dcg-2021` (which carries no
marks and no legend file at all — see `docs/decisions.md`, 2026-07-30):

| Instrument | Provisions marked | Legend entries |
|---|---:|---:|
| `socap-sofipo-dcg-2006` | 507 | 97 |
| `cucb-dcg-2004` | 264 | 99 (capped)† |
| `cub-dcg-2005` | 260 | 99 (capped)† |
| `oaac-dcg-2009` | 245 | 99 (capped)† |
| `scap-dcg-2012` | 204 | 42 |
| `fi-dcg-2014` | 152 | 47 |
| `cue-dcg-2003` | 96 | 99 (capped)† |
| `itf-dcg-2018` | 88 | 18 |
| `servinv-dcg-2013` | 28 | 8 |

† = a `\d{1,2}` marker-regex cap folds legend entries ≥100 into entry 99's
text rather than splitting them out, so 99 undercounts the true legend
length for these four instruments — not a meaningful column to sum. Zero
committed provisions carry markers that resolve to nothing; every mark
resolves. The legend is already populated by
`crates/lex-parse/src/itf.rs`'s `flush_legend` — there is no "populate or
justify empty" decision left to make.

**A more serious defect than "legend gap", found during the re-pass:** the
same `\d{1,2}` cap also governs the in-body margin-marker regex
(`amendment_marker_regex`, `dcg.rs:176`), so a `(100)`-or-higher marker in
the source text fails to parse as a marker at all and is silently dropped —
confirmed present today in committed `provisions.json` for the four capped
instruments (three-digit parenthesized tokens sitting in body text). See
`docs/decisions.md`, 2026-07-30, "CNBV legend re-pass" for detail. Not fixed
here — it is a scoped follow-up needing its own fixtures, separate from
Stage A.

## Proposed trusted-boundary shape — for sign-off

Deliberately mirrors the existing statute model rather than inventing a
parallel one.

On `StandardModificationSource` (already present, currently
`publication_date` / `official_url` / `included_in_source`):

```
affects: Vec<StandardModificationTarget>   // parsed from the decree title
```

```
StandardModificationTarget {
    clause: String,          // "3.2", "5.1.1", "5.2.7.ii.1)"
    action: ModificationAction,   // modified | added | eliminated
    resolved: bool,          // whether `clause` matches a committed clause id
}
```

On `StandardClause`:

```
amended_by: Vec<usize>   // indices into StandardMetadata.modifications
```

Rendered the way Diputados and CNBV print it — a per-clause note naming the
action and the DOF publication date, linked to `official_url`:

> Numeral modificado mediante decreto publicado en el DOF el 10-05-2011

### Validation rules

- A target whose `clause` matches no committed clause sets `resolved: false`
  and raises a warning rather than an error: the decree may target a numeral
  the base text does not contain (an *adición*), which is legitimate and
  informative.
- `standard_unconsolidated_modification` keeps firing at instrument level, but
  its message names the affected clauses when they are known.
- A clause carrying `amended_by` is not thereby stale-free: the annotation
  records that its text is **known outdated**, which is the opposite claim.
  Wording in the exporter must not imply the text was updated.

### What this does not assert

The clause text remains exactly the base publication. Nothing here claims
currency; it claims *known staleness, precisely located*. That distinction is
the entire value, and it is easy to lose in presentation.

## Verification

- Fixtures from the two real NOM-247 decree titles, including the compound
  second one (modifies + adds + eliminates + eliminates an annex).
- The annex-elimination target must parse and be recorded as `resolved: false`
  with a clear reason — the corpus has no annex representation to point at.
  This is the seam where Stage C begins.
- Byte-identical reparse of all committed standards (clause text must not
  move).
- A NOM-247 record whose 252 clauses are unchanged, with 11 clauses marked
  from the 2011 decree and 10 targets from the 2012 decree.

## Open question carried from the review

NOM-247 has **at least four** modifications, not the two recorded. The 2011
decree's CONSIDERANDO cites decrees of **2010-01-22** (adding transitional
provisions, 180 days for labelling) and **2010-07-19** (extending the first
transitorio to 2010-12-31). Both are absent from `standard.json`. Whether
transitorio-only modifications belong in `modifications[]` is a modelling
question this plan does not settle.

## Next action

Operator sign-off on the `StandardModificationTarget` / `amended_by` shape
above — the only open item left, per the 2026-07-30 correction. Nothing is
implemented.
