# Fintech structural pilot — Codex G0 disposition

Date: 2026-11-14. **Verdict: approve with conditions.** G1 remains unapproved.

## Identity and independence

Reviewed branch `pilot/fintech-structural`, HEAD
`3ba9dbbf233345b19f27bb0675d74334825b40a4`, including the package committed at
`0ecc40523`. The sample baseline is
`c337dae1af9474d0aef246b3de196c49e2527722`.

This receiving Codex session did not curate the package and inherited no
curating-session conversation or private annotation context. It received the
navigation capsule and read the handoff and listed repository evidence. The
development curator and gate reviewer share a provider; this is session-level
independence, not independent model-family acceptance. The builder-side check
and fresh-agent QA supply the separately attributed evidence. This disposition
did not inspect private answer content; its delegate verified the byte commitment.

## G0 findings

The frozen package meets the sample requirements in the plan's G0 section and
D4: 24 exact pinned provision records, a disjoint 16/8 split, LRITF plus LIC and
LMV, two explicitly unsupported DCG controls, and development selections bound
to recorded fraction-qualified reference edges. Those edges seed queries and
do not establish answer correctness. The public holdout selection and independent
QA document all five D6 strata, including a qualified fraction marker rather
than merely a Bis article number. The package is local-only and label-level;
it makes no current-law, public-release, complete-span or projection claim.

Evidence read: the pilot plan, G0 submission, independent G0 review, builder-side
check, sample manifest, public holdout selection and verifier. Earlier private
annotation checks are attributed to independent QA, not repeated here.

## Binding conditions and answers

1. **D6 needs an explicit amendment before G1 approval.** Public membership,
   strata and canonical text do not literally satisfy an unseen-article holdout.
   Accept this package as an **answer-sealed, publicly identified holdout**.
   Record that narrowing in D6, with this disposition as authority. The builder's
   recorded commitment excludes all eight provisions from development, tests,
   debugging and tuning before code freeze. Keep queries, quotations, offsets
   and expected results private until the frozen implementation is evaluated.
   The workflow promise is sufficient for this bounded local pilot; it does not
   establish technical access isolation or fully blind generalisation. Any
   pre-freeze answer exposure or use of holdout provisions for tuning invalidates
   their independence and requires a fresh sealed evaluation set.
2. **Holdout-only strata are acceptable.** G0 does not require every stratum in
   both splits. G1 must define same-line markers, qualifier normalisation,
   trailing-paragraph attachment and the coverage needed for `not_found` before
   projection code. Use independent development/synthetic examples for these
   rules, never holdout provisions. Incomplete coverage must abstain rather than
   claim absence. G1 must also settle the remaining span, identity, parenthood,
   coverage and typed-outcome requirements already listed in the plan.
3. **Preserve the original gold on a contract conflict.** Freeze the G1 contract
   independently of private answers. A reviewer may then assess compatibility
   privately. Retain G0 v1 and its commitment unchanged; version any revised
   gold/contract with rationale, timing and independent review before use. Do
   not disclose the conflicting private case to tune implementation. If such a
   disclosure occurs, replace that evaluation set. After evaluation, a revision
   is a new experiment and cannot repair the original score retrospectively.
4. **Keep a second recoverable copy before G2 or any cleanup/transfer.** One
   ignored `.work` copy is insufficient custody. Use an evaluator-controlled,
   encrypted backup outside the checkout and disposable scratch directories,
   preferably on separate backed-up storage; keep the key outside builder
   access. The custodian chooses the actual private location and records only
   the custodian, backup date, original SHA-256 and successful restore/hash check
   in builder-facing evidence. Do not put plaintext gold or decryption material
   in Git, shared plans, receipts or the handoff. Another shared `.work` copy
   does not meet this condition. No backup was created or verified in this
   disposition; the custodian must supply that evidence before G2.

G0 sample selection need not be redone. G1 drafting may proceed; G1 approval
requires the D6 amendment and written semantics. G2 additionally requires the
custody evidence above. This disposition authorizes no projection, public
interface, canonical change, legal determination, merge, push or deployment.

## Verification and closeout

A mechanical delegate ran the following read-only checks at the reviewed HEAD;
all exited 0:

- `python3 fixtures/fintech-structural-g0-2026-11-14/verify.py --holdout-gold .work/fintech-g0-holdout-2026-11-14/holdout-gold.json`
  — 24 pinned records, 16/8 split, 23 development queries, artifact/source/seed/
  definition bindings and sealed commitment verified.
- `git diff --check`.
- `git diff --exit-code HEAD -- corpus adapters crates schemas`.
- `git check-ignore -v .work/fintech-g0-holdout-2026-11-14/holdout-gold.json`
  — ignored by `/.work/`.

The 545-test Cargo gate is prior builder-reported evidence, not a fresh run by
this session. No code or canonical data changed, so Cargo checks were not
repeated for this documentation disposition. This session did not edit or stage
the four pre-existing prompt changes. No commit or push was made by this session.
At closeout a concurrent writer had advanced HEAD to
`61bde01eb0643259b2c55a37fd6c13e39e567953` and staged plan/prompt changes.
This verdict remains bound to `3ba9dbbf2`; it does not accept the concurrent
commit or staged changes. Integration requires a drift check against the
reviewed evidence. Further shared-worktree edits were stopped on detection.
The external Vault handoff remains pending; this repository
verdict is the review result, not external queue resolution.

**Next action:** builder records the D6 amendment and submits the written G1
contract for separate Codex review, with custody evidence required before G2.
