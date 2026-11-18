# Fintech pilot G0 — local sample submission

Status: sample prepared at `c337dae1af9474d0aef246b3de196c49e2527722`
on `pilot/fintech-structural`; G0 disposition and G1 contract remain separate.
This submission contains 24 provisions: 16 development and 8 independently
selected holdout provisions. It is an explicitly local corpus snapshot, with
no assertion about public release membership or current legal force.

## Frozen artifacts

All builder-facing artifacts are under
`fixtures/fintech-structural-g0-2026-11-14/`:

- `sample-manifest.json`: pinned baseline, sample IDs/split/rationale, artifact
  hashes, pre-task dirty paths and sealed-gold commitment.
- `canonical-snapshot.json`: exact JSON provision records selected from the
  pinned commit, retaining unchanged text, IDs and metadata.
- `source-snapshots.json`: instrument and source-manifest records for the five
  instruments, plus SHA-256 of each pinned provisions/references/terms/metadata
  input file. Official URLs, publisher, retrieval time and acquisition/extraction
  hashes are retained as provenance, not independently re-fetched or re-certified.
- `development-selection.json`: recorded fraction-qualified edge identities
  that seeded selection. Edges are not assumed to be correct annotations.
- `development-gold.json`: 23 manually selected candidate label-mode queries,
  literal label spans, one inline-citation non-marker observation and an existing
  definition-span witness. No candidate recognizer generated these expectations.
- `holdout-selection.json`: public membership/strata only, without answer offsets.
- `verify.py`: structural integrity checker; it imports no parser or recognizer.

The frozen artifact hashes are in the manifest. Source records match both the
pinned Git objects and current canonical files. A new baseline or changed text
requires a new snapshot/version and a reviewed reason; do not silently refresh.

## Development sample

| Instrument | Articles | Purpose |
|---|---|---|
| LRITF | 22, 23, 36, 61, 82, 89, 98 | Ordinary fractions, nested incisos, continuation/trailing paragraphs, repeated child labels and inline citations |
| LIC | 11, 24, 46 | Absent-label negative, final paragraph, nested incisos and qualified fraction |
| LMV | 2, 32, 71, 128 | Definition witness, qualified fractions, short ordinary sequences and trailing/cited text |
| ITF DCG 2018 | 18 | Explicit unsupported-format control |
| IFPE DCG 2021 | 8 | Second unsupported-format control |

The primary queries come from the stored reference edges; additional manually
chosen queries cover nesting and qualifiers. LIC article 11 deliberately tests
a request for fraction I that its entire current text does not contain. This
records absence in the pinned text; it makes no legal/historical interpretation
of why the citation differs. The expected `not_found` is conditional on G1's
eligible complete fraction scan. An incomplete implementation must abstain.

The LMV article 2 definition witness is the existing `Control` record. It is a
comparison witness, not a newly resolved glossary decision. Label spans count
Unicode scalar values and are half-open into the exact stored text. Python's
literal-index operations calculated offsets for human-selected quotations;
no production recognizer selected or classified those quotations.

The two DCG controls expect `not_structured` under the pilot's declared scope,
even though their text contains readable Roman labels. This tests honest scope
abstention rather than rewarding accidental extraction from an unsupported format.

## Independent holdout and custody

A fresh agent, with no inherited builder context, selected and annotated eight
disjoint articles from canonical LRITF/LIC/LMV JSON only. It did not use reference
edges, parser code or the candidate recognizer. Public selection contains all
required strata: repeated labels in distinct scopes, nested incisos, uncertain
trailing attachment, a qualified Bis fraction and the dot-dash form. An initial
selection covering only Bis *article numbers* was corrected before freeze to
include a qualified subsection marker. No projection was developed against it.

There are 15 private candidate label queries. Gold is held persistently at
`.work/fintech-g0-holdout-2026-11-14/holdout-gold.json`, ignored by Git, bound
by SHA-256 `ce8fb8400a2a9f44835c24629037eebf4a2bdc7bd0d2f007d4cdb18941bbf317`.
The builder-facing files expose no holdout expected answers or answer offsets.
The builder must not read this file until G1/code freeze for the G3 evaluation.
Reviewers may inspect it. This is a workflow seal and hash commitment, not an
OS security boundary; the file is not encrypted and agents share the filesystem.
Preserve/back it up before any .work cleanup or cross-machine handoff. If it is
lost, do not silently regenerate it and call it the same holdout.

## G0/G1 boundary

This is a sampling submission, not a structural implementation or completed
pilot. Gold currently freezes candidate label evidence and scope/absence cases.
It does not assert full subsection spans or settle trailing-paragraph ownership.
G1 must define label/intro/complete spans, coverage eligibility, parent paths,
identity and typed ambiguity before any projection code. If that contract changes
an expectation, retain this G0 evidence version and explicitly version the change
before implementation; do not alter answers after evaluating the holdout.

The same parent curated development cases and integrated this package, so it
must not be described as independent review of its own gold. A separate fresh
QA agent reviews the sample and private gold; its spoiler-free report is
`docs/fintech-pilot-g0-independent-review-2026-11-14.md`. G0's engineering
reviewer still owns the gate disposition; no legal determination is requested.

## Verification and recovery

Run from repository root:

```bash
python3 fixtures/fintech-structural-g0-2026-11-14/verify.py
```

Reviewer-only commitment verification (checks bytes/hash, does not print answers):

```bash
python3 fixtures/fintech-structural-g0-2026-11-14/verify.py \
  --holdout-gold .work/fintech-g0-holdout-2026-11-14/holdout-gold.json
```

The mechanical delegate ran both commands successfully: 24 exact pinned records,
16/8 disjoint split, 23 development queries, source/seed/definition/artifact hashes
verified; sealed commitment verified. `git diff --check` and
`git diff --exit-code HEAD -- corpus adapters crates schemas` both exit 0.
No Rust code or canonical data changed, so native Cargo gates were not repeated
for this data-only sampling step. Independent QA may add a disposition below.

Stop if any snapshot/hash/offset/disjointness check fails. Restore the pinned
inputs or create a reviewed new snapshot; do not train against an altered gold
set. Next action: G0 review, then freeze G1 semantics. No prototype, database
projection, public route, export, deployment or legal-status update was made.

The prior slice-b review/plan edits and four untracked prompts were preserved.
Artifacts and documentation remain uncommitted; no commit or push was authorized.

## Independent QA outcome

The fresh QA reviewer found the package acceptable as a G0 submission, with no
missing required stratum, snapshot/gold binding defect or required artifact
revision. The reviewer independently checked private label bindings and custody
without publishing holdout answers. The full report is linked above. This QA
recommendation does not itself grant G0/G1 approval; the prepared package is now
ready for the engineering gate disposition.
