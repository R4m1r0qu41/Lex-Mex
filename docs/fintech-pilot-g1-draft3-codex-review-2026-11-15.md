# Fintech structural pilot — Codex G1 draft 3 review

Date: 2026-11-15. **Verdict: approve G1 at the reviewed commit.**
Both remaining draft-2 findings are closed. This approves the contract;
it does not clear the separate G2 custody prerequisite or authorize execution.

## Binding and reviewer context

Reviewed branch `pilot/fintech-structural`, HEAD
`a6d94025fd2b5242d397c28aba0c3f1502eb3ff2`, range
`75e19ed74..a6d94025f`. Intake was clean. This is the receiving Codex session
that issued the earlier G0/G1 reviews, carrying that review context. It did
not author the builder's drafts or inherit the builder's drafting conversation.
This is review continuity, not a new independent review of earlier verdicts.
The provider-handoff-resolution procedure continues to apply.

Evidence: the commit-bound handoff, earlier dispositions, contract changes
through G1.11, both new public witnesses, four added full responses, preservation
of prior expectations, existing schema/verifiers and reproduced measurements.
Review stayed bounded to the submitted changes and their interactions with
previously accepted rules.

## Findings closed

**A — Holder selection by kind.** G1.6 now keeps every fraction candidate held
by the root, including one in a tail inciso. Only an inciso candidate there
is held by T. The `g1-syn-24-*` witness follows the contract: the matching
fraction request takes L2; an absent fraction request takes L7 with
`partial_scan`; the tail inciso takes L1; fraction I's label and intro resolve
while its complete span remains ambiguous. The supplied full responses expose
the root's partial fraction scan, avoiding a false absence claim.

**A consequence — Nested region ranges and cuts.** Accepted. Distinguishing
the innermost region used to find a candidate from the enclosing ranges used
to propagate cuts is a necessary clarification. A root-held possible fraction
can interrupt both an inciso and its enclosing fraction, so both ranges below
the holder are affected. Listing owners innermost first is deterministic. In
the public witness, the earlier tail cutoff still determines the certain span
`[0,7)`; the response retains both applicable segments. This conservatively
propagates uncertainty and does not promote the candidate into a node or move
canonical text. The change is tied to public item A, not to private gold.

**B — No-fraction tails across notes.** G1.4 now starts this tail at the first
paragraph after the last inciso's marker paragraph. Interior notes may be
enclosed but remain certainly root-owned; ownership rule 2 takes precedence
over note interruption. In `g1-syn-25-*`, the inciso's certain span `[0,8)`
ends before the note, the tail is `[10,52)`, root parenthood stays resolved,
and the explicit root complete lookup returns `[0,52)`. This aligns the
cutoff table, the no-note guarantee and the committed probe.

No further public contract revision is required by this review. Earlier R1,
R2, R4 and R5 remain closed; these changes close the outstanding R3 interactions.
D8 and conservative tail abstention retain their earlier acceptance. No claim
is made that an implementation has achieved any expected outcome or coverage.

## Compatibility record: g1-draft3-compatibility-v1

This record describes compatibility only; it does not revise private answers.

- Contract commit: `a6d94025fd2b5242d397c28aba0c3f1502eb3ff2`.
- Contract-section SHA-256:
  `ad607ca7d65983c05b5cf47e5350d9ed1aaab5591ed12745df8062e9a264ebf6`.
  The hashed bytes begin at the `## G1 contract, pilot-v1 (draft 3 for Codex review, 2026-11-15)`
  heading in the plan and continue through EOF at the reviewed commit.
- Original G0 gold commitment:
  `ce8fb8400a2a9f44835c24629037eebf4a2bdc7bd0d2f007d4cdb18941bbf317`.
- Result: **incompatible**. A private read-only comparison confirms that a
  sealed annotation still conflicts with the frozen contract. Original gold
  bytes, canonical hashes and exact quote bindings were verified unchanged.
  This is a direct compatibility finding, not exhaustive semantic evaluation
  or a scored G3 run.

No affected provision, query, label, offset, stratum, count or case-specific
rule is disclosed. G0 v1 remains immutable. Before G3, an independent evaluator
must prepare/review any necessary versioned private gold or evaluation
disposition under G0 condition 3, binding it to this approved contract and
preserving the original evidence. No silent answer rewrite or retrospective
score repair is allowed. Disclosure or tuning on private cases requires a
fresh evaluation set. The public contract need not be changed toward those
answers, and this known incompatibility does not reopen G1's engineering gate.

## Verification actually run

A mechanical delegate ran at the reviewed HEAD, all successfully:

- G1 `verify.py`: **123 rows** (42 development, 81 synthetic), all 23 G0
  development queries and 20 label spans, branch/pair coverage and full
  response links passed.
- G0 `verify.py --holdout-gold .work/fintech-g0-holdout-2026-11-14/holdout-gold.json`:
  pinned records, provenance bindings and sealed commitment passed.
- `cargo run --offline --locked --manifest-path fixtures/fintech-structural-g1-2026-11-14/schema-check/Cargo.toml`:
  meta-schema check, **28 valid responses**, **32 must-fail rejections**,
  projection example and **346 row-request checks** passed. Invalid-request
  fixtures are expected to fail the request schema; 346 is the check count,
  not a claim that every request is well formed.
- `measure.py`: file hashes and counters match committed output, with only
  the input-commit line differing as expected.
- Preservation comparison against `75e19ed74`: all 116 draft-2 behaviour
  rows, 24 full responses and 32 must-fail cases unchanged case by case.
  The G1 schema, verifier, standalone checker, probe/output and projection
  example are unchanged; corpus, workspace crates, trusted schemas, adapters
  and G0 fixtures are unchanged across the reviewed range.
- `git diff --check`: exit 0.

Additional receiving-review checks: manual contract/witness/response alignment,
contract-section digest and private canonical/hash/quote compatibility checks.
Hand-asserted branch and scan fields remain authored expectations; the schema
and fixture checks do not substitute for G2/G3 implementation evidence.
No workspace Cargo gate was repeated for this documentation-only verdict.

## Next action and scope

**Next action:** evaluator supplies the encrypted holdout backup and a
successful restore/hash check required before G2. Record custody evidence
without plaintext answers or decryption material in builder-facing files.
G1 approval alone does not clear that prerequisite. Private evaluation
versioning remains required before G3. No G2/G3 execution is performed here.

This session writes this verdict and a plan checkpoint. No code, canonical
record, schema, fixture or gold was changed. No staging, commit, merge, push
or deployment was performed as part of this review. The external Vault
handoff remains pending; repository acceptance is not external queue or
continuity closeout.
