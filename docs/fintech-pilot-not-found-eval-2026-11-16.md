# Fintech pilot — short not-found evaluation

Date: 2026-11-16. **16/16 expected absences returned `not_found`; zero false
`not_found` results.** All four positive controls resolved correctly. All four
abstention controls abstained, with two detailed-field disagreements retained
in the immutable **22/24** full-field score.

## Scope and method

JRH adopted `991b5dc85` for internal, local, read-only use and requested this
short absence supplement. Codex acted as evaluator; the builder worked in its
separate status-pass worktree and did not prepare or see these cases before the
run. Preparation, independent review and mechanical execution used separate
Codex contexts. This is role/session separation within one provider.

Intake: clean `pilot/fintech-structural`, HEAD
`6f43dbe8da18167d807db4f99d9b83a193806ca0`. Candidate code, corpus, schemas,
fixtures and Cargo lock match frozen
`991b5dc8592f0588e916f0b9ae302c4e5ec815a6`. The G1 contract-section SHA-256 remains
`ad607ca7d65983c05b5cf47e5350d9ed1aaab5591ed12745df8062e9a264ebf6`.

The evaluator selected 24 unique requests from real canonical structure in
12 fresh articles across LRITF, LIC and LMV: 16 absences, four positives and four
required abstentions. These are structure-derived probes, not a random sample
of citation errors. Independent guards exclude provisions mentioned anywhere
in the earlier public G0/G1 inventories; the eight G3 provisions are included
in that exclusion. No selected provision overlaps those earlier samples.

The preparer read each selected article in full, and an independent reviewer
checked each expected parent/kind scan and control against G1 without candidate
output or implementation-source access. Gold, rationale, runner and validator
were approved and hash-locked before execution. Private gold SHA-256:
`50d608672a419a4c306075a2a7a27fabb9d711509405ce7b72e11cc6c0ed2101`.

One frozen example process loaded only the 12 selected records and executed
24 scored requests once. No retry, duplicate replay, tuning, gold replacement
or score repair occurred. Detailed requests, canonical quotes, offsets,
selection rationale and comparisons remain evaluator-private under
`.work/fintech-not-found-eval-2026-11-16/`.

## Results

| Requested measure | Result |
|---|---:|
| Correct `not_found` | 16/16 |
| False `not_found` | 0 |
| `not_found` where abstention was required | 0/4 controls |
| `not_found` for a present positive target | 0/4 controls |
| `not_found` without a complete scan | 0/16 returned absences |
| Abstention where `not_found` was expected | 0/16 |
| Positive controls correct | 4/4 |
| Required abstentions, correct outcome type | 4/4 |
| Required abstentions, all expected fields match | 2/4 |
| All prelocked fields match | 22/24 |
| Wrong resolved results; substring/binding errors | 0; 0 |

The 16 returned absences all had complete scans and matched their prelocked
absence evidence. The four abstentions comprised two `ambiguous` and two
`not_structured` responses. All requests and responses passed schema validation.

| Mode | Full-field matches | Returned absences |
|---|---:|---:|
| label | 12/12 | 6 |
| intro | 5/5 | 5 |
| complete | 5/7 | 5 |

The two complete-mode abstention-detail discrepancies are not absence failures;
they remain failures in the 22/24 full-field score. Post-run QA disposition is
recorded below. No new determinism experiment or full-corpus accuracy claim is
made. This small selected sample provides bounded evidence for absence handling,
not a guarantee for arbitrary structures or every provision.

## Validation and disposition

Mechanical checks passed: frozen-source/data/fixture identity, locked example
build and offline locked schema-validator build. The existing unused `walkdir`
workspace-dependency warning remains. Independent review and a no-lookup
preflight checked exclusion coverage, request schema and expectation bindings.
An evaluator-only unresolved-segment comparison was completed before the run;
the sealed gold and frozen implementation were unchanged. Captured response
schema validation and exact substring/hash/release/node-binding checks passed.
No Cargo workspace gate was repeated for documentation and private evaluator
artifacts; no repository code or canonical data changed.

Captured response SHA-256:
`ac836b3781a12f9154b5daa241d1dba109c4951368fe8832a6b8299ec4a55aef`.
The original G0/G1/G3 artifacts and G3 score **39/42** remain unchanged.

Independent post-run QA verified every critical absence count and the control
outcomes/reasons. Both detailed disagreements are prelocked expectation
representation errors: the expected unresolved-segment owner paths omitted
occurrence suffixes required by G1. The captured output is contract-correct;
QA found no implementation or scorer defect. The gold and strict **22/24**
score remain unchanged. The private QA report and all original comparison
failures are retained.

**Disposition:** this supplement supplies the requested bounded absence evidence
for the already adopted internal, local, read-only pilot. Continue that use
within its existing limits. Future evaluation preparation should verify
occurrence-qualified owner paths before locking expectations. No further run
or implementation change was made.

No staging, commit, push, merge or deployment was performed by this session.
