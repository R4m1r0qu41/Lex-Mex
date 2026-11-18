---
project: lex-mex
date: 2026-11-07
type: production-readiness-review
status: delivered-with-open-findings
---

# Production readiness review and fresh-agent mLey baseline — 2026-11-07

## Verdict

Functional pilot is live and useful; unrestricted production readiness is **not established**.
Current deployment is 9031ec9e-d03b-4d68-933c-ca1dc638722d at100%, verified with Wrangler deployment history and invocation scriptVersion. Producer checkout HEAD88dd6f66e was clean at start. Capsule205 was stale: it still described inactive-preview-only state. No deployment, DNS, WAF, billing or credential changes occurred in this review.

## Evidence and gates

| Gate | Result | Evidence |
| --- | --- | --- |
| Live MCP and web routes | Pass bounded functional test: official SDK connects, four tools, exact article46 evidence, eight routes200 and hostile Origin403 | readiness-live-mcp-2026-11-07.json |
| Request CPU | Not ready against Free10ms budget: observed0–63ms; search samples63,14,17ms. All17 captured invocations ok, no exceptions; includes two incidental requests beyond15probe requests. Small sample; no p99 or cold/warm attribution claimed | readiness-cpu-2026-11-07.json; readiness-network-2026-11-07.json |
| Plan/quota | Current subscription read403. Operator-confirmed Free in prior rollout report; current remaining quota not verified. Per-client120/min and shared600/min bindings confirmed live. These do not protect daily account quota before Worker invocation | authenticated read-only settings; production-rollout-draft-2026-11-03.md; host HANDOFF |
| Search behavior | Three differential tests pass including real870-provision projection; search host module matches producer | .work/readiness-search-2026-11-07.log; readiness-host-module-parity-2026-11-07.json |
| Producer/host parity | Two differences: host MCP rejects non-POST with405, producer lacks guard; projection host drops unused Buffer import. Rest of ten modules match. Rebuilding from producer could regress405 behavior | readiness-host-module-parity-2026-11-07.json and direct diff |
| External fresh agent | Completed one public network-only trial; no repository, memory or oracle access.45URL attempts25Lex/20mLey, no429 | fresh-agent-mley-baseline-2026-11-07.md |
| Organic discovery / multiple providers | Not tested; explicit site entries supplied. No broader acceptance claim | trial scope |

Cloudflare documents Free10ms CPU and some isolate flexibility for occasional overruns. Successful63ms calls do not prove reliable headroom: https://developers.cloudflare.com/workers/platform/limits/ . Tail observer started and stopped for this review only. No load/stress test performed.

## Fresh-agent scoring

Parent scored only after the blind report against frozen runtime-proof-answer-key-2026-10-28.json. Q01,Q02,Q03,Q04,Q06,Q08,Q09,Q10 satisfy the expected content outcomes. Q05 is partial: it retrieved46/47 and made a cautious pooling conclusion, but did not retrieve22 or state the specific account/concentration limit; do not count as complete. Q07 is partial: it retrieved ITF1 and IFPE2 but itself declined a complete comparison.8complete/2partial/0fabricated inaccessible answers observed. This is content assessment, not independent legal accuracy certification.

mLey outcomes:7answerable (Q01,02,04,05,08,09,10),3DCG tasks not located (Q03,06,07). Not locating those instruments through tested indexes is not proof of site-wide absence. Q05 has the same missing22/limit qualification; coverage benefit does not imply superior legal reasoning. Q10 is a useful explicit scope refusal for Lex and successful retrieval for mLey; no penalty for honest Lex coverage boundary.

Fresh-agent source issues: IFPE7 contains duplicated identifier phrase, not PDF-verified; source defect versus extraction defect remains unclassified. Agent's retained-PDF403 was reproduced for Python-urllib/3.9; the same URL with LexMexReadiness/1.0 returned206 and correct PDF magic for1024bytes. This is a confirmed client-dependent source-access failure, not a missing PDF; see readiness-pdf-probe-2026-11-07.json. Do not silently correct canonical text. Source lookup required a401877-byte manifest; this violates the intended compact ordinary-query path even though answers could be obtained. Current root discovery exists but did not prevent this trajectory. One sequential agent compared sites, so ordering/learning is a limitation. Questions were delivered together, not prescribed per-message conversations.

## Matched mLey measurements

Same-machine curl, three entry requests/site, redirects enabled, sequential samples:

| Entry | Final bytes | Redirects | Median total seconds |
| --- | ---: | ---: | ---: |
| Lex-Mex |7827|2|0.514|
| mLey |133423|0|0.797|

Python urllib discovery-file samples: Lex935bytes/median0.728s; mLey4264bytes/median0.428s. Python urllib did not automatically follow Lex308 entry (reported HTTP308), while curl and Node fetch did reach200. No redirect loop established. These are network observations with tiny samples, separate resources and different content breadth, not a general speed ranking. No access to mLey backend CPU, costs or accuracy controls, so do not compare those with Lex CPU. Evidence mley-entry-curl-baseline-2026-11-07.json and mley-matched-network-baseline-2026-11-07.json.

Web client friction: this harness web tool could not open Lex entry; mLey entry worked but text/markdown discovery was unsupported. Both worked through network-enabled curl. Preserve both results; do not collapse them into universal agent compatibility.

## Prioritized follow-up

1. Bound production CPU and account quota: profile search/cold-request costs, verify current usage with analytics permission; choose optimization or explicitly approved platform/plan policy. Do not treat tolerance as capacity.
2. Synchronize receiving producer MCP405 guard with targeted regression before a producer-based rebuild; preserve independently owned host work.
3. Improve compact source/version discovery so a fresh agent does not need an integrity manifest. Re-test Q05 and Q07 without providing answers or internal routes.
4. Resolve PDF-access observation and duplicated IFPE7 phrase using exact official source evidence, without changing audited legal decisions.
5. Keep four-provider conversations and organic discovery as separate follow-up gates. No push, commit or further rollout authorized by this review.

The requested review and baseline trial are delivered. The readiness verdict remains conditional/negative; this report does not authorize rollback, paid upgrade or deployment.
