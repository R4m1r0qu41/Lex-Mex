# Fintech structural pilot — Codex G1 draft 2 review

Review date: 2026-11-15, America/Mexico_City. The filename retains the handoff's
requested date. **Verdict: revise, limited to the rule interactions below.**
G1 is not yet approved; G2 remains blocked separately by evaluator custody.

## Binding and independence

Reviewed `pilot/fintech-structural` at
`f7641a53780753137671e2ab709be8c82a13842f`, range
`c1f3cbf06..f7641a537`. Intake was clean and HEAD remained unchanged through
the evidence checks. This is the same receiving Codex review context as the
G0 and draft-1 dispositions, not an independent fresh review of those earlier
verdicts. Codex did not author either builder draft or inherit its drafting
conversation. The provider-handoff-resolution procedure continues to apply.

Sources: the commit-bound handoff; earlier G0/G1 dispositions; G1.1–G1.10;
the revised schema, verifier, standalone Rust checker, public row/full-response
fixtures, projection example, and committed measurement probe/output. Private
gold was read only for compatibility and was neither changed nor disclosed.

## Disposition of the earlier findings

| Item | Draft-2 disposition |
|---|---|
| R1 | Closed: L1 precedes L4; tail incisos have explicit matching/scan treatment. |
| R2 | Closed: reason collection and status reduction are deterministic. |
| R3 | Substantially addressed: note interruption, earliest cut, root spans, regions and resolved no-fraction parenthood are explicit. Two interactions below still need correction/clarification. |
| R4 | Closed for the submitted representations: raw request echo, outcome-specific fields, both ambiguity forms, stale bindings, root-child IDs, full examples and executable schema validation are supplied. |
| R5 | Closed: node-ID requests are explicitly deferred; root requests and locator syntax are defined. |
| R6 | Compatibility conflict persists. Preserve G0 v1; independent private versioning remains a separate evaluator task after contract freeze. |

D8's marker-paragraph intro and conservative tail abstention remain acceptable.
The measurement probe and its recorded hashes/counters now reproduce; the
245/357 coverage estimate can be treated as reproduced input evidence, not as
an implemented lookup result. Hand-entered branches and scan fields remain
expectations; passing their checks does not prove a projection implements them.

## A — Candidate holder must remain valid for its kind inside a tail inciso

G1.6 first assigns fraction candidates to the root and inciso candidates to
the fraction (or root without fractions). It then says, without restricting
the candidate kind, “For a candidate in a tail inciso's region, P is T.”
In an article with fractions T is a fraction, which has no fraction scan.

Public synthetic witness, created for this review:

```text
I. Uno.

Cierre.

a) Alfa; II. Dos.
```

The committed probe confirms classes F/C/I, a tail beginning at the middle
paragraph, and a same-line fraction candidate in the final paragraph. The
literal tail override places that fraction candidate under T; the root's
fraction scan can then miss it and permit a false `not_found` for fraction II.
Applying the earlier kind rule instead gives a root-held candidate and L2.

**Required:** qualify the tail override by kind: fraction candidates remain
root-held everywhere; inciso candidates inside tail incisos are held by T.
Confirm the root fraction scan becomes partial and the matching fraction
request follows L2. Add this witness to public expectations, with a full
response if needed to cover the representation. No private case motivated it.

## B — No-fraction tails must define how interior root-owned notes affect the cutoff

G1.4 defines a no-fraction tail as the **unlabelled paragraphs** after the last
inciso. Its unlabelled class definition excludes article notes. G1.5 then
cuts the last inciso at the end of the paragraph immediately before the tail
segment, while promising that a non-root span never includes a root-owned note.

Public synthetic witness:

```text
a) Alfa.

Artículo reformado DOF 01-01-2020

Cierre.
```

The classes are I/N/C. If the segment starts at the first unlabelled paragraph
(`Cierre.`), the cutoff table ends the inciso's certain span after the note,
contradicting the no-note guarantee. If the segment encloses the note from its
start, the cutoff is safe, but that is a broader interval than the no-fraction
definition specifies. The committed probe's `tail_start` returns the note's
index; it therefore embodies the second interpretation without the prose
making that exception explicit. Ownership rule 2 precedes note interruption,
so rule 3 cannot silently repair the first interpretation.

**Required:** explicitly define the no-fraction tail interval across interior
notes, while retaining their certain root ownership, or specify the note-aware
cutoff directly. Add the witness and assert the inciso's certain span ends
before the note. Keep the explicit resolved root parenthood and resolved root
complete span. This is a remaining R3 interaction, not a request to redesign
the tail policy.

## Private compatibility and custody

A private read-only comparison verified the canonical text hashes and exact
quote bindings and confirmed that a sealed G0 annotation remains incompatible
with draft 2. No case identity, query, label, offset, stratum, count or private
corrective rule is disclosed. This was a compatibility check, not a scored
G3 run or exhaustive implementation evaluation.

G0 v1 and its commitment remain unchanged. First freeze the revised contract
using public/development evidence; then an independent evaluator records any
versioned compatibility/gold change privately under G0 condition 3. Do not
tune the builder's rules to private answers. No backup, decryption material,
versioned private gold or restore receipt was created by this review. The
evaluator-controlled encrypted backup/restore check still blocks G2 separately.

## Verification actually run

A mechanical delegate ran these successfully at the reviewed HEAD:

- G1 `verify.py`: 116 rows (42 development, 74 synthetic), G0 coverage and
  20 label spans, branch/pair coverage, and 24 full response links passed.
- G0 `verify.py --holdout-gold .work/fintech-g0-holdout-2026-11-14/holdout-gold.json`:
  frozen records, provenance bindings and sealed commitment passed.
- `cargo run --offline --locked --manifest-path fixtures/fintech-structural-g1-2026-11-14/schema-check/Cargo.toml`:
  draft-2020-12 meta-validation, 24 valid responses, 32 rejected must-fail
  responses, projection example and 326 row requests passed.
- `measure.py`: all corpus-file hashes and counters equal the committed
  `measure-output.txt`; only the reported input commit differs from its
  recorded `24587218f` to reviewed `f7641a537`.
- `git diff --check` and
  `git diff --exit-code HEAD -- corpus crates schemas adapters fixtures/fintech-structural-g0-2026-11-14`:
  exit 0.

Reviewer checks additionally exercised the committed probe on the two public
synthetic witnesses above and performed the private binding/compatibility
comparison. Probe classification is evidence for the interactions, not a
production recognizer acceptance test. No package installation or network
retrieval was needed. No workspace Cargo gate was repeated for this
documentation-only disposition.

**Next action:** builder makes the two bounded public rule/fixture revisions
and submits a new commit-bound G1 review. Earlier closed findings need not be
redesigned. This session writes this verdict and a plan checkpoint only; no
code, canonical data, schema or gold is changed, and nothing is staged,
committed, merged, pushed or deployed. The external Vault handoff remains
pending; no external queue or continuity closeout is claimed.
