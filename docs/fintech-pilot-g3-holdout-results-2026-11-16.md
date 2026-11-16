# Fintech structural pilot — G3 sealed evaluation

Date: 2026-11-16. **Prelocked contract-gold score: 39/42 (92.9%).**
There were 33 resolved requests, all exact and correct, and 9 abstentions.
Post-run independent QA attributes the three mismatches to one prelocked
expectation error across modes, not an implementation defect. The score stays
39/42; neither gold nor code was repaired after execution.

## Evaluator identity and pre-execution disposition

JRH authorized Codex to act as evaluator, privately record the earlier sealed
compatibility conflict, then run G3 once. The builder did not open the answers
or run the tool. Codex used separate evaluator preparation, independent QA and
mechanical execution contexts; this is session/role separation within Codex,
not independent model-family evidence or technical isolation from JRH's account.
The prior disclosed builder smoke-run deviation remains in the G2 review.

Frozen code: `991b5dc8592f0588e916f0b9ae302c4e5ec815a6`.
Intake checkout: clean `pilot/fintech-structural`, HEAD
`c113f934273c84d04c7b441714c36a8c0738c1ed`. Only documentation differs from
the candidate; crates, canonical corpus, schemas, G1 fixtures and Cargo lock
remain unchanged. The G1 contract section retains SHA-256
`ad607ca7d65983c05b5cf47e5350d9ed1aaab5591ed12745df8062e9a264ebf6`.

The original G0 gold remains byte-identical, SHA-256
`ce8fb8400a2a9f44835c24629037eebf4a2bdc7bd0d2f007d4cdb18941bbf317`.
Before any candidate invocation, an evaluator without candidate-output access
prepared a private compatibility disposition and versioned contract expectations.
Independent QA checked both against canonical text and the frozen contract.
The versioned gold was hash-locked at
`132905618fa83c36093fb510a16701579d5da19fb4f311c4a84b5a75ae91f580`.
Approval also binds the disposition, runner and schema validator hashes.

G0 contains 15 observations, not 15 typed lookup requests. Fourteen positive
marker observations were translated into typed locators and tested in all three
modes: 42 requests. One prose-only negative observation has no executable
structural locator and was explicitly excluded before execution. The private
record retains all original bindings, exclusions and compatibility rationale.
One original positive label expectation was privately identified as incompatible
with G1 before execution; original evidence was preserved separately.

## One execution and immutable score

The frozen example was built with
`cargo build --locked -p lex-core --example structure_pilot_lookup`.
Its SHA-256 was
`17822f0511adb181a5a021ab29afbb78e181dce829ee9c7bb09703d55439b353`.
The evaluator loaded filtered canonical copies containing exactly the eight
holdout provisions, rather than projecting every provision of the three laws.

A create-exclusive execution record was written before launch. One example
process received 42 scored requests followed by 42 predetermined identical,
unscored determinism controls. No candidate invocation was made while preparing
or reviewing gold. No retry or tuning run occurred. All 84 requests and 84
captured responses validate against the frozen schema. The two response batches
are byte-identical; this tests determinism within one process, not separate-build
or separate-process reproducibility.

| Metric | Result |
|---|---:|
| Agreement with prelocked contract expectations | 39/42 (92.9%) |
| Resolved exactness, including span, text and node/parent path | 33/33 (100%) |
| Resolved requests / all attempts | 33/42 (78.6%) |
| Abstentions / all attempts | 9/42 (21.4%) |
| Wrong resolved results | 0/33 |
| Exact substring, hash, release and node-ID binding errors | 0 |
| `not_found` results | 0/42 |
| Observed false absence results | 0; no `not_found` result was exercised |
| Original G0 positive-marker discovery agreement | 12/14 (85.7%) |
| Changed canonical files, IDs, text, review state or source manifests | 0 |

Original-v1 discovery means that a positive observation resolves at its original
Unicode-scalar marker start and that the returned marker text is a prefix of the
original quote. G0's quotes extend beyond marker labels; they are not G1 exact
label-span gold. This metric must not be called original exact-span accuracy.
The prose-only exclusion is outside that 14-query denominator.

| Mode | Prelocked matches | Resolved | Abstained |
|---|---:|---:|---:|
| label | 13/14 | 12/14 | 2/14 |
| intro | 13/14 | 12/14 | 2/14 |
| complete | 13/14 | 9/14 | 5/14 |

| Instrument | Prelocked matches | Resolved | Abstained |
|---|---:|---:|---:|
| LRITF | 12/15 | 12/15 | 3/15 |
| LIC | 15/15 | 14/15 | 1/15 |
| LMV | 12/12 | 7/12 | 5/12 |

| Requested kind | Prelocked matches | Resolved | Abstained |
|---|---:|---:|---:|
| fraction | 27/30 | 24/30 | 6/30 |
| inciso | 12/12 | 9/12 | 3/12 |

Abstention reasons are `sequence_break` (3), `unresolved_boundary` (3), and
`same_line_marker` (3). Independent post-run QA checked the three unexpected
abstentions against source and G1. The frozen contract requires those conservative
results; the prelocked gold and its manual scan inventory missed the sequence
condition. This is an evaluation finding. No post-run gold replacement,
reclassification of the scored cases, implementation fix or corrected score is
claimed. The original 39/42 result remains the experiment's result.

## Coverage and companion evidence

Captured responses identify 12 distinct last-step parent/kind pairs: 10 complete,
1 partial and 1 ambiguous. Thus complete coverage in that observed scope is
10/12 (83.3%); it is not coverage of every projected parent or the whole corpus.
The prelocked manual inventory's different counts are retained as evidence of
the evaluation defect, not substituted for observed coverage.

Selected provisions contain 29 existing glossary records. Twelve same-fraction
resolved-result comparisons had valid definition spans: 11 lookup spans lie
inside the definition span and one equals it. Definition and requested-mode
spans serve different purposes; no equality was forced and no term was changed.
No new rendered Obsidian export/anchor comparison or live/public interface test
was run in this bounded scoring task. Earlier slice-a anchor parity and G2
fixture gates remain separate evidence. There was no holdout `not_found` output,
so this experiment supplies no positive holdout evidence for absence correctness.

Detailed per-case IDs, requests, quotes, offsets, expected/actual comparisons,
shape breakdowns, source bindings and the post-run QA are retained privately at
`.work/fintech-g3-evaluator-2026-11-16/`. They are deliberately absent from
builder-facing records. Original gold, prelocked v2 gold and captured output all
remain unchanged. Captured response SHA-256:
`3df29651ab32d5fb7d094e17f28447660dbb3be94424a8470a76d55d936790fc`.

## Validation and disposition

Mechanical preflight passed: candidate/HEAD identity for code/data/fixtures,
locked example build, G0 sealed-commitment verifier, and offline locked evaluator
schema-check build. Before execution, independent QA ran a no-lookup preflight
checking every original quote/hash binding, versioned span bounds, exclusion
coverage and request schema. After execution, captured response schema validation,
semantic comparison, canonical substring/binding audit, determinism control,
glossary-span audit and `git diff --check` passed. No workspace Cargo gate was
repeated for these documentation/evaluator-only changes; the G2 checks remain
recorded in the freeze review.

**Recommendation: iterate evaluation.** Keep the implementation frozen. A fresh,
independently prepared and reviewed evaluation set is needed before a clean G3
acceptance claim. Do not rerun or repair this experiment's score, tune on its
private cases, or treat 33/33 resolved exactness as 100% overall gold accuracy.
This turn does not start another evaluation, authorize broader adoption, or change
any public interface. No staging, commit, push, merge or deployment occurred.

## Continuity closeout

Repository truth was updated first, followed by the project-day receipt at
`/Users/jr/Vaults/Agent_Vault/AI/60_Evaluations/session-summaries/lex-mex/2026-11-16.md`.
The existing active-run capsule was required and explicitly checkpointed as
**252**, at
`AI/30_Executable/runtime/active-runs/runs/lex-mex-20260714-131904-complete-and-safely-validate-the-prepared-lex/CURRENT.json`.
It records the current task-plan path and SHA-256 without expanding its already
full 20-source navigation list. The guarded rolling-context writer refused
because writes are not enabled for `lex-mex`; the pre-existing dirty note was
preserved. No hook-based semantic update or rolling-context success is claimed.
The report, plan/status changes and receipt remain uncommitted because this
session has no commit authorization.

Private artifact-manifest SHA-256:
`905beafffdea7d7923a1eba4ee1924881b473c7da4cf410c91e17ffde4b374cf`.
This seals the original bindings, prelocked gold/disposition/approval, runner,
input/request/response captures, schema checks, comparisons and post-run QA.
