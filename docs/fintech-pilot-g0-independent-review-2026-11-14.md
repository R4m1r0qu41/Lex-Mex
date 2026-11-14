# Fintech structural pilot — independent G0 sample review (2026-11-14)

## Scope and disposition

Independent QA reviewed the frozen G0 submission at baseline
`c337dae1af9474d0aef246b3de196c49e2527722`. This review does not approve a
gate, G1, or implementation. It finds the sample **acceptable as a G0
submission for parent disposition**, subject to the custody condition below.

## Accept findings

- **Size and split:** the manifest contains 24 unique provisions: 16
  development and 8 holdout. The provision IDs are disjoint, and the canonical
  snapshot has exactly those 24 records.
- **Provenance and pin:** the baseline commit, local-only membership claim,
  per-provision canonical-text hashes, source-file hashes, and frozen artifact
  hashes bind the records to the submitted corpus state. The submission makes
  no deployed/public-inventory claim.
- **Development seeding:** all 16 development selections have a recorded
  fraction-qualified reference-edge seed. The development gold expressly treats
  those references as query seeds rather than truth: it includes an absent-label
  control and two unsupported-format exclusions.
- **Strata:** across the two splits, the frozen set covers ordinary paragraphs
  and fractions, nested incisos, repeated labels in distinct scopes, trailing
  attachment uncertainty, qualified `Bis`, dot-dash form, cited numerals that
  are not labels, a genuine absent-label request, definition-span comparison,
  and the two DCG unsupported/abstention controls. The main statute set is
  LRITF plus LIC and LMV; the DCG records are explicitly controls while their
  parsers remain out of scope.
- **Development gold:** all submitted resolved label quotes and Rust-character
  ranges round-trip against their frozen canonical provision text; non-marker
  observations and definition witnesses are bound by the supplied verifier.
  The annotated locators are plausible label-level paths, including the
  repeated-child cases under distinct fractions. This is not a claim of a
  complete structural tree.
- **Holdout independence and custody:** the public selection lists membership,
  reasons, and strata but not queries, expected results, quotes, or offsets.
  The private gold is ignored by Git, its commitment matches the manifest, and
  its canonical bindings and supplied label quote/range invariants pass review.
  Private gold content is intentionally not reproduced here.
- **Honest G0 scope:** both gold artifacts limit assertions to candidate
  label-mode results. They explicitly defer complete spans, parent attachment,
  and complete-mode semantics to G1. No premature G1 guarantee or projection
  result is asserted.

## Revise findings / conditions

No artifact revision is required for this G0 submission.

The sealed holdout is workflow isolation rather than an OS security boundary,
as the manifest accurately states. Before code freeze and G3 evaluation, the
builder must continue not to read
`.work/fintech-g0-holdout-2026-11-14/holdout-gold.json`; any exposure or
tuning against it invalidates this holdout and requires a newly selected,
sealed set. G1 must separately specify parenthood, `label`/`intro`/`complete`
semantics, unresolved segments, per-parent coverage, and typed outcomes before
any projection work.

## Validation actually run

- `python3 fixtures/fintech-structural-g0-2026-11-14/verify.py --holdout-gold .work/fintech-g0-holdout-2026-11-14/holdout-gold.json` — exit 0; verified all frozen artifacts, source/seed/definition bindings, 24 exact pinned records, the 16/8 split, 23 development queries, and the sealed commitment.
- Read-only private-gold binding check — exit 0; verified each private record's
  canonical-text hash and every supplied label quote/range invariant without
  emitting private query content.
- `git check-ignore -v .work/fintech-g0-holdout-2026-11-14/holdout-gold.json`
  and `git ls-files --error-unmatch .work/fintech-g0-holdout-2026-11-14/holdout-gold.json`
  — confirmed the file is ignored and untracked.

No candidate scanner, parser, or recognizer was run. No production, canonical,
or protected evaluator file was changed.
