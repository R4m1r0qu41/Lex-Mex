# Four-instrument runtime test — 2026-10-28

Purpose: find the shortest path from a real question to useful, sourced text, then fix the largest observed obstacle. This is a ten-question product trial, not another ingestion campaign. No model trials or deployed-runtime pass are claimed by preparing it.

## Start here

Give each model [the questions and common instruction](../prompts/runtime-proof-questions.md), one question at a time in a fresh conversation per model. Use Claude, Grok, Gemini and GPT with their actual available browsing/tool settings; record exact versions. Do not supply the [evaluator key](runtime-proof-answer-key-2026-10-28.json).

**Public mode is the delivery test.** Entry: `https://www.leg4ll4bs.com/lex-mex/llms.txt` (or the reviewed candidate deployment URL). Let the model discover the paths; record the actual observed release. If inaccessible, retain the error and do not fabricate URLs or answers. A web-tool request on 2026-10-28 returned an accessibility error; this does not prove site downtime or universal client failure. The current query service is not assumed deployed. Existing R7 discovery is a static text/index path; the trial can expose its friction before any new service is launched.

**Local mode is a separate diagnostic control**, useful when the provider has filesystem access: entry `/Users/jr/Dev/lex-mex/corpus/mx/`, four allowlisted instrument folders only. It tests legal reading/retrieval with access available; it does not prove public runtime. Never compare a local-access score against a public score as if the environments were equal. Browser-only models cannot read a Mac path. Do not upload all four instruments to compensate for a failed ordinary question; that tests a different product.

One baseline run per model (40 answers total); same question order, mode and observed release. Q09 follows Q02. Keep the original answers, cited links, visible tool errors and wall-clock times. Do not request hidden reasoning. One standard follow-up is allowed for a failed answer and recorded separately, never replacing baseline. If a publication changes, label a new run; compare paired runs on the same version. First try Q02, Q04 and Q05 as a short smoke test; stop a universally inaccessible entry before spending time on 40 failures.

## Questions and what each tests

| ID | Simple task | Main failure exposed |
| --- | --- | --- |
| Q01 | What is in this collection? | Discovery and honest coverage |
| Q02 | Who authorizes an ITF, and does government guarantee customer money? | Short source-backed answer, LRITF11 |
| Q03 | Exact first paragraph of IFPE DCG2 and required consent | Unicode/text fidelity and wrong-instrument risk |
| Q04 | Find customer/own-resource segregation | Thematic retrieval, LRITF46 |
| Q05 | Must every IFPE client have an individual bank account? | Multi-article reasoning and limits |
| Q06 | Follow DCG IFPE2 to7: authentication and evidence | Reference navigation without changing instrument |
| Q07 | How do the two DCG scopes differ? | Instrument ownership; do not exclude DCGs |
| Q08 | LIC2: institution types and banking service | LIC retrieval and substantive completeness |
| Q09 | Which source/version supports LRITF11; is it checked through today? | Provenance without invented freshness |
| Q10 | Fetch LRSIC27, outside these four | Honest absence of coverage, not legal nonexistence |

Frozen control: accepted SQLite `d82f53c073a53960d18f56c323d720ebce84dddcc2f70da949fa0ba25687539d`, existing path recorded in the key. Independently compared to main `f0f8a868f`: all870 IDs/texts match (LRITF156, LIC511, ITF132, IFPE71). The key includes exact selected witness text, source metadata and hashes from that read-only artifact. It is an oracle snapshot, not new canonical data or a freshness certification. Prior publicR7 pin is included as a prior observation, not asserted current deployment. No DB rebuild is required.

## Evaluate in five minutes per model

Copy [the results template](../prompts/runtime-proof-results-template.json) into a task-specific run directory under `derived/runtime-proof/` (gitignored; keep raw transcripts there). Write the model/version, time, mode, entrypoint, observed release and transcript path. For each question paste the answer, record elapsed seconds and sources, then score:

| Dimension | 0 | 1 | 2 |
| --- | --- | --- | --- |
| Access | No access: mark blocked and leave all scores null | Needed one navigation/help step | Retrieved without operator help |
| Source fidelity | Unsupported or wrong text/source | Correct source but weak/incomplete citation | Correct instrument/article, real path and faithful quotation |
| Legal reading | Materially wrong | Partial, with limits disclosed | Meets the key's material points |
| Scope | Invents absent law/currentness | Limit unclear | Distinguishes corpus facts, inference and missing coverage |
| Usability | Does not answer | Answer needs one follow-up | Clear, directly useful and concise |

Access help must be distinguished from a substantive corrective follow-up. Keep baseline/follow-up answers separately; record baseline scores and score a retry in a new run if wanted. If an answer refuses because it has no access, classify `blocked`, leave scores null, and record the actual failure layer. If it answers from memory despite no retrieval, score the unsupported answer and flag any invented norm/source. An honest Q10 outside-coverage answer is `answered`, not `blocked`, if discovery establishes the coverage boundary.

Run:

```sh
python3 prompts/runtime-proof-score.py derived/runtime-proof/claude-baseline.json > derived/runtime-proof/claude-summary.json
```

The script validates the result structure and checks supplied literal quotes against selected frozen witnesses. Enter evidence as `{"id":"urn:lex-mx:federal:statute:lritf:article:11","quote":"exact passage"}`. Unlisted citations require manual review; no evidence is not an automatic pass. The script cannot judge legal correctness, prove a fetch, or distinguish client truncation from an intentional excerpt: the evaluator must inspect the transcript and key. Automatic span matching supplements human scores.

Provisional pilot target: all10 tasks answered, each at least8/10, zero critical flags (wrong instrument, altered quotation, invented norm, false currentness or false coverage). Record per-case scores and total answered/blocked separately; never average blocked trials out and call it a pass. For latency, report median and slowest task; use the first observed runs to set budgets, rather than inventing a strict timing gate. A missing optional link is not a critical failure when text and an honest unavailable state remain useful.

## Turn failures into a bounded fix

For every failure write one row: `case | model/version | release | exact request/path | actual response | expected witness | layer | proposed fix | owner | retest`. Classify:

- Access/discovery: client egress, missing agent-visible entry, unusable route or JS-only text. Fix the common retrieval entry/route, not parser rules.
- Retrieval: search misses an existing provision, wrong instrument, missing context, a clipped body. Compare a direct article fetch; fix serving/search semantics.
- Source data: the served text differs from the accepted witness. Isolate that output and trace exporter/source ownership; preserve the accepted reference.
- Reasoning: correct complete text reached the agent but interpretation fails. Improve instructions/context or the model workflow; do not rewrite canonical law to improve an answer.
- Presentation: the answer is right but hard to use; improve concise citations, navigation or qualification.

Prioritize repeated failures across providers over a one-model oddity. Fix one shared bottleneck, add a small regression reproducing it, rerun the failed question and one adjacent success, then repeat the ten-question pack once before promotion. Preserve before/after transcripts and version pins. Do not rebuild the national database to fix a route, prompt, hover window or unsupported-link label. A service may reuse the frozen database without pretending a new code commit built it.

## Temporal milestone closed

JRH on2026-10-28 confirms the five reviewed instruments are in force despite unresolved exact business-day counts, and directs LFPA29 for month/year counting. Main already records all five as `in_force`; see the new dated entry in [decisions](decisions.md). This does not turn every article into an unqualified `effective` determination or erase repeal/amendment evidence. No general month/year calendar implementation is needed to begin this trial.

Current next action: operator runs the three-question smoke test on the public entry in each provider, retains errors/answers, then completes ten questions where access works. A common access failure is the next runtime implementation target, not another temporal review cycle.
