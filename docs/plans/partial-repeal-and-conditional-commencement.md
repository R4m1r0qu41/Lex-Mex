# Partial repeal and conditional commencement

Status: shape approved by the operator on 2026-10-07 and implemented in steps
2 to 4 below (types, validators, fixtures, parser rule, the two corrected
rows). Step 5, the model-output schema version for the condition record, and
the review-queue entry for `unverified` conditions are not built; no parser or
model sets `conditional_pending` yet.

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
- Judicial invalidation (found 2026-10-07). Decision (operator, same day):
  an SCJN invalidation of an article, fraction or paragraph is expressed like a
  partial repeal, so it follows the same rule for now. It uses the
  `partially_repealed` shape with a `cause` of `judicial` on the `repeals`
  entry (legislative repeals carry `legislative`), and the whole-article case
  uses `repealed` with the same `cause`. A dedicated rule can come later.
  Open: which date ends force (notified for legal effects or DOF publication)
  is a legal call, and the 65 provisions in 23 instruments flagged by text
  search (41 committed `effective`, 24 `unknown`; example `lcm` Art. 338) have
  not been read.

## Implementation record (2026-10-07)

- `TemporalStatus::PartiallyRepealed` and `ConditionalPending`; `Provision`
  gained `repeals` (scope, ordinals, `cause` legislative or judicial,
  optional DOF date, `renumbering`) and `commencement_condition` (source
  provision, wording, authority, `condition_status`, `met_on`, `evidence`).
  Both fields are omitted from JSON when empty.
- Validators in `validate_corpus` reject a partial repeal with no scope, repeals
  on an unrepealed provision, a pending provision with no condition, a met
  condition without a date and evidence, and a condition on the wrong status.
- A model cannot set either status: the v2 output schema does not list them and
  the routing check rejects them; a test covers both.
- The parser classifies only a parenthesised note at the start of a provision
  that names paragraphs or fractions, in either word order ("el primer
  párrafo", "los párrafos segundo a cuarto", "las fracciones III y V"). A note
  with prose inside it ("con los subsecuentes recorriéndose"), or one naming
  another article or law, stays `unknown` or `repealed` for review.
- `derive-temporal` now reclassifies committed deterministic `repealed` rows
  whose note matches: `lfpc` Art. 122 and `ltfccg` Art. 16. A census of all
  132 instruments finds no other match.
- Read, not changed: `lscs` Art. 207 holds a derogation clause that repeals
  other instruments plus promulgation text, so it is probably not itself
  repealed; `fi-dcg-2014` Art. 64 is a repealed-chapter placeholder. Both stay
  `repealed` until someone reads the source.
