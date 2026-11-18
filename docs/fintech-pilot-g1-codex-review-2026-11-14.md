# Fintech structural pilot — Codex G1 review

Date: 2026-11-14. **Verdict: revise. G1 is not approved; do not start G2.**

## Review binding and independence

Repository `/Users/jr/Dev/lex-mex`, branch `pilot/fintech-structural`, HEAD
`c35bb1f91ab8b8fb3d5ed4349b246a2db7b1c639`. Reviewed range
`868d92588..c35bb1f91`: the plan/contract, project status and G1 fixtures.
The working tree was clean at intake. This Codex session did not author G1;
it previously issued the G0 disposition and inherits that review context, not
the builder's drafting context or the development curator's conversation.

The receiving review applies the provider-handoff-resolution procedure. Scope
is contract review and this verdict, not implementation or gold repair. Source
evidence is the handoff, G0 disposition and frozen G0 artifacts, G1.1–G1.9,
D6/D8, all 73 expected-behaviour rows, the proposed schema and verifier, and
the existing `lex-core` marker/normalisation definitions. Private gold was
examined only for compatibility; no private case is reproduced below.

## Accepted design choices

- **D6 amendment:** satisfies the requested answer-sealed/public-membership
  distinction and records the builder's pre-freeze non-use commitment.
- **D8:** marker-paragraph-only `intro_span` is an acceptable explicit pilot
  deviation. Retain the distinction from continuation text. Availability in
  `complete` mode is subject to that mode's ambiguity rules; do not promise
  that every continuation will be returned as a resolved span.
- **Tail abstention:** conservative abstention is acceptable for pilot-v1.
  Report resolved-correct, resolved-wrong and abstained by mode, including the
  last-fraction coverage cost. The quoted 245/357 count is builder-measured
  context, not a reproduced measurement in this review. The disposable probe
  is insufficient for an independently reproducible G3 result; retain the
  measurement method/input binding with the eventual results.
- **Absence versus gaps:** `not_found` only after an eligible complete scan,
  with `sequence_gap` for a missing interior/leading label, is an acceptable
  declared policy. This establishes absence in the pinned text, not whether a
  provision legally exists. Scope abstention for both DCGs remains correct.

These choices do not eliminate the blocking contradictions below.

## Required revisions

### R1 — The ordered unique-match branch bypasses ambiguous parenthood

G1.7 says tests apply in order. A tail inciso counts as a match under each
candidate parent. The unique-match branch checks only count, same-line/
unsupported candidates and scan ambiguity; it does not exclude a match whose
parenthood is ambiguous. It therefore continues before the following branch
can apply `ambiguous_parent`.

Public witness: `g1-syn-08-tail-incisos-q2`, locator `{fraction: I, inciso: a}`.
There is exactly one matching tail node under I, no carrying marker candidate,
and no specified sequence break. The fixture expects `ambiguous_parent` in
every mode; the earlier branch permits continuation and a label resolution.

**Required:** test ambiguous parenthood before unique resolution, or explicitly
require resolved parenthood in that branch. Specify how tail nodes participate
in each candidate parent's scans and sequence checks. Keep/add fixtures proving
the ordered decision table, rather than relying on the span verifier.

### R2 — Scan states have no precedence when several conditions coexist

G1.6 assigns `ambiguous` for a sequence break, but independently assigns
`partial` when a same-line or unsupported candidate occurs. A parent can have
both. This matters: a unique label can resolve under `partial` but must abstain
under `ambiguous`. Coding order must not decide the outcome.

**Required:** define a deterministic reduction/precedence for all applicable
scan reasons, retaining all reasons. For example, a sequence break can dominate
partial evidence. Add a synthetic case combining a repeated label with an
unsupported or same-line candidate and query a different, unique label.

### R3 — Ownership and contiguous span rules leave interrupted cases open

G1.3 assigns article notes certainly to the root. G1.5 nevertheless takes a
subsection's contiguous span through the last certainly owned paragraph.
For the synthetic text `I. Uno.\n\nArtículo reformado DOF 01-01-2020\n\nContinuación.\n\nII. Dos.`,
ownership rule 3 assigns the continuation to I. A span extending through it
also includes the intervening root-owned article note. There is no specified
clipping, exclusion or abstention rule for that hole. The note rule says notes
never introduce attachment ambiguity, so that cannot silently be the fallback.

The certain-span cutoff also needs explicit scope: the post-child segment in
`g1-syn-12-post-child-attachment-q1` is certainly inside fraction I, and its
fixture correctly includes it in I's resolved complete span. G1.5's blanket
instruction to end before a `post_child_attachment` must apply only to a node
whose boundary that segment actually makes uncertain, not all ancestors.

**Required:** define what happens across root-owned interior notes and other
ownership interruptions; define root-without-markers intro/complete explicitly.
State which segments can truncate which node, and use the earliest applicable
uncertain boundary when several exist. Add the interrupted-note example and
a combination of candidate/attachment reasons. In no-fraction tails, T and
root are identical: deduplicate candidate parents and distinguish uncertain
content attachment from actual uncertainty about an inciso's parent.

### R4 — Request/error semantics and the schema disagree

The proposed schema cannot encode several required responses, and permits
responses missing fields the prose requires:

- `mode` permits only the three valid values, although
  `g1-syn-10-invalid-locators-q6` requires a response for mode `body`.
  The response must specify whether it preserves the raw invalid value,
  carries a nullable validated mode, or uses another explicit representation.
- The response's `locator` rejects unknown keys, although q4 requires an
  `invalid_locator` for a locator containing `apartado`. Define raw versus
  normalised request fields and behaviour for wrong JSON types/missing fields.
- Every response is said to carry a reason, but `reason` is not required and
  its enum has no success or absence reason. Hash/path/scan metadata cannot
  always exist for an unknown provision, malformed locator, or root lookup
  with no parent/kind step. Define outcome-specific required/null/absent fields
  and which current/projected binding is reported on stale requests.
- The node-ID regex rejects a valid root-child inciso path `i.a.1`, permitted
  by G1.4/G1.5. A read-only regex probe confirmed this rejection.
- The prose requires candidates for every ambiguous response; the schema
  instead accepts unresolved-segment-only responses, as several fixtures use.
  Choose and document the distinction between path ambiguity and span ambiguity.

**Required:** align prose, response schema and representative full response
fixtures for every outcome, including the invalid-mode/unknown-key cases,
root-child incisos, successful root lookup and stale binding. Validate the
schema against its declared meta-schema in the eventual validation environment
and validate those full fixtures. Current expected-behaviour rows are partial
expectations, not full schema instances; do not describe their verifier as
response-schema validation. Enforce semantic span/identity invariants separately
where JSON Schema cannot express them.

### R5 — The node-ID request alternative is not specified

G1.7 permits a `node_id` instead of provision ID plus locator, but only defines
the latter's path walk. It does not settle malformed/missing IDs, conflicts
when both request forms are supplied, old version/hash IDs, or whether direct
IDs bypass sequence/parenthood ambiguity. G1.7 also says no lookup returns a
whole article in place of a subsection while deliberately supporting a root;
state that an explicit root request is a distinct operation, with its own
mode/scan behaviour, rather than an implicit fallback.

**Required:** either defer node-ID requests explicitly for pilot-v1 or specify
their full decision table, including stale IDs and ambiguous nodes, with public
synthetic cases. Freeze locator syntax too: allowed inciso alphabet/case,
whitespace handling and optional qualifier spelling must not be left to coding.

### R6 — Private compatibility review found a sealed-answer conflict

At least one G0 sealed label annotation is incompatible with the proposed G1
contract. Canonical text hashes and exact private quote bindings passed. A
direct rule comparison confirmed a conflict; this is not a G2 implementation
run or a scored G3 evaluation. No affected provision, query, label, offset,
stratum, count or corrective case-specific rule is disclosed here.

**Required:** preserve G0 v1 and its original commitment unchanged. Freeze the
revised G1 contract on public/development evidence, then have an independent
evaluator create and review any necessary versioned gold/compatibility record
privately under G0 condition 3. Record the timing, versions and rationale
without leaking cases to the builder. Do not silently rewrite old gold or use
the private conflict to tune implementation. Disclosure/tuning requires a
fresh evaluation set; later changes cannot repair an earlier score.

## What the 73 rows establish

All rows were inspected. Hashes, quoted text, span boundary invariants and G0
query coverage pass the supplied verifier. They do **not** establish that all
73 rows follow the written contract: R1 is a concrete decision-order mismatch,
and R4 prevents representing some requested errors consistently. The fixtures
also contain no successful root query. Extend the examples for the revisions
above; retain existing evidence and explain any changed expectation.

## Validation actually run

A mechanical delegate ran, all with exit 0:

- `python3 fixtures/fintech-structural-g1-2026-11-14/verify.py` — 73 rows,
  40 development/33 synthetic, all 23 G0 queries covered and all 20 G0 label
  spans equal, with hashes/quotes/boundaries checked.
- `python3 fixtures/fintech-structural-g0-2026-11-14/verify.py --holdout-gold .work/fintech-g0-holdout-2026-11-14/holdout-gold.json`
  — snapshot and sealed byte commitment verified.
- `git diff --check`.
- `git diff --exit-code HEAD -- corpus crates schemas adapters fixtures/fintech-structural-g0-2026-11-14`.

Additional reviewer checks: JSON parsing, node-ID regex probe, fixture inventory,
private canonical hash/quote validation and direct compatibility comparison.
An exploratory private witness assertion was inconclusive and failed; it was
discarded, and the conflict finding above rests on a separate successful direct
comparison. No private content was written to a review artifact.

`jsonschema` is unavailable in the current Python environment; no package was
installed and no meta-schema validation is claimed. No Cargo checks were
repeated because this disposition changes documentation only. No recognizer,
projection, canonical record, G0 gold or G1 fixture was modified.

## Next action and custody

Builder revises the public contract/schema/examples for R1–R5 and submits a new
commit-bound G1 review. R6 is handled by an independent evaluator after contract
freeze. The evaluator-controlled encrypted backup/restore check is still absent
and independently blocks G2; G1 approval alone would not clear it.

This session writes this verdict and a plan checkpoint only. No staging,
commit, push, merge, deployment or legal-status decision is authorized or made.
The external Vault handoff remains pending; this is the repository review
result, not a claim of external queue or continuity closeout.
