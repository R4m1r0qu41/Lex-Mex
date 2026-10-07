# Partial repeal and conditional commencement

Status: design proposal for operator sign-off. No schema, Rust type, validator
or corpus change has been made. Both decisions below were made by the operator
on 2026-10-07; this note turns them into a concrete shape so the trusted data
boundary can change in one reviewed step.

## Why this is one change

Both gaps come from a binary model of when a provision is in force. A
provision is currently `repealed` or not, and `effective_from` is a date or
null. The law is richer:

- A repeal can remove one paragraph, several, or one fraction, and leave the
  article in force.
- A provision's commencement can depend on a future act that may not have
  happened, so "not yet in force" is a real state, distinct from "unknown".

Because the trusted boundary changes, the JSON Schemas, Rust types,
validators, fixtures and documentation must move together.

## 1. Partial repeal

Decision: model partial repeal explicitly. It is legally relevant and common
("se derogan los párrafos 1, 2 y 4, con los subsecuentes recorriéndose"). It is
sometimes by fraction, not paragraph.

Proposed shape:

- New provision status `partially_repealed`. `repealed` keeps meaning the
  whole provision is repealed.
- A `repeals` list on the provision. Each entry has a `scope` (`paragraph`,
  `fraction`, `inciso`, `other`), the `ordinals` the source names (kept as the
  source states them, for example `["1", "2", "4"]` or `["III"]`), the DOF
  date when the source gives it, and `renumbering: true` when the source says
  the remaining units are renumbered ("recorriéndose").
- Ordinals are recorded against the text as it stood before the repeal. They
  are source statements, not offsets into the current text, because
  renumbering makes offsets unreliable.
- Deterministic parsing classifies only the unambiguous shapes. Anything the
  parser cannot place in one scope becomes `unknown` and opens a review item.
- A `partially_repealed` provision with no `repeals` entry is a validation
  error.

Rows to review once the model exists. `ltfccg` Article 16 begins "(Se deroga
el primer párrafo)" and is committed `repealed`; that is a partial repeal.
`lscs` Article 207 and `fi-dcg-2014` Article 64 are also committed `repealed`
but begin with a repeal marker followed by words ("se deroga el título VII…",
"Derogado Capítulo sexto…"). They have not been read in full, so whether they
are partial, targeted at another instrument, or fully repealed is open. A
fourth such row exists and has not been identified.

## 2. Conditional commencement

Decision: when commencement depends on a future uncertain act that cannot be
verified, the provision is not in force until that act occurs. A placeholder
is needed so the corpus can be ingested now and filled in later. These cases
should be few enough for manual review.

Proposed shape:

- New provision status `conditional_pending`: not in force until a stated
  condition is met.
- A `commencement_condition` record: the source transitory, the condition in
  the source's words, the acting authority when one is named, a
  `condition_status` of `unverified`, `met` or `unmet`, and the date and
  evidence when `met`.
- `unverified` is the placeholder. It opens a review item, so every such case
  is listed for manual review. `unmet` keeps the provision not in force.
  `met` records the date it came into force and moves the provision to
  `effective`.
- The condition record is evidence for a reviewer. The parser never sets
  `met`; only an audited resolution does, so a model cannot decide it.

Worked example (LACP). The decree's Tenth-Fourth Transitory gave the CNBV 180
days from publication to issue the general rules (DCG) the law calls for.
Anything that takes effect only once those rules are issued carries a
`commencement_condition`. Whether the DCG were issued in time, late or not at
all is a fact outside the source text, so the record starts `unverified`.

## 3. How the placeholder gets filled

The full corpus will be ingested before most conditions can be resolved. The
review list is the work queue: each `unverified` condition names the authority
and act to look for (a DOF publication, a DCG, a regulation). As instruments
are ingested, some conditions are satisfied by instruments already in the
corpus and can be resolved with a citation; the rest go to the legal reviewer.

## 4. Order of work

1. Operator sign-off on the two shapes above.
2. Schema, Rust types and validators for both, with fixtures for each
   scope and condition status, in one commit per boundary.
3. Parser classification for the unambiguous partial-repeal shapes and a
   regression fixture for each.
4. Read the four flagged rows, then correct whichever are mislabeled through
   the Rust pipeline.
5. Add the condition record to the temporal model output schema as a new
   version, since model output must validate against a versioned schema.

## Open questions

- Should `partially_repealed` articles keep `temporal_status: effective` for
  the surviving text, with the repeal carried only in `repeals`? This note
  proposes a distinct status so no consumer reads the article as untouched.
- Does an `unmet` condition ever need a deadline, so a long-overdue condition
  is flagged for re-check? Not proposed here.
- Judicial invalidation (found 2026-10-07, not yet decided). A Supreme Court
  (SCJN) general declaration of unconstitutionality can invalidate a whole
  article, a fraction or a normative portion; it is not a legislative repeal.
  The source text carries notes such as "Artículo declarado inválido por
  sentencia de la SCJN … notificada para efectos legales 10-11-2025 y publicada
  DOF 16-01-2026" (`lcm` Art. 338, committed `effective`). A text search finds
  such notes in 65 provisions across 23 instruments, 41 committed `effective`
  and 24 `unknown`; they have not been read, so some may be partial or may
  describe an older ruling. Options: a `cause` (legislative or judicial) on each
  repeal entry, a separate `invalidated` status, or both. Which date ends the
  force of a provision (notification for legal effects or DOF publication) is a
  legal decision for the operator.
