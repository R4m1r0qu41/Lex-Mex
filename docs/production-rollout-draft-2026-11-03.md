# Lex-Mex MCP pilot — production rollout DRAFT

Status: **EXECUTED 2026-11-04T06:47Z by operator instruction ("go to 100%").**
`aca7ddf1-706b-47d4-b6b4-c2f3847f19d8` deployed at 100% on `leg4ll4bs-web` (direct switch,
no canary: a split was blocked by asset version skew, see below). Rollback target
`097dc04d-5eb5-4f9a-a088-e8fac5609aa6` (previous 100% version) is retained:
`wrangler rollback 097dc04d-5eb5-4f9a-a088-e8fac5609aa6 --name leg4ll4bs-web -m "reason"`.
The CPU gate did NOT pass its pre-set cold-start criterion (first request ~40-70 ms vs
production p99.9 16.8 ms); the operator accepted that risk on a Free plan where requests
up to 141 ms CPU have so far completed with status success. Post-deploy checks: remote
SDK proof against `https://www.leg4ll4bs.com` passed (4 tools, article 46 retrieved,
hostile Origin 403); `/`, `/en`, `/lex-mex/`, `current.json`, agents page, sitemap,
search API and article route all 200; all 7 hashed assets referenced by `/` return 200.
Watch for: any exceededCpu/exceededResources/error status (needs an analytics read),
asset 404s. NOT YET DONE: external AI-client tests and organic-discovery tests; commit
of the uncommitted `reader/` changes (needs operator authorization); independent
Codex review of the retrieval change; revoke of the Cloudflare analytics token.
Original text of this draft follows (historical).

(Previous status: DRAFT, NOT APPROVED, NOT EXECUTED. Gates evaluated 2026-11-04; they
did not pass.)

## Verdict (2026-11-04)
Operator confirmed the Cloudflare plan is **Free** (10 ms CPU per request, 100,000
requests/day). Preview `834b5d37` measured CPU p90 37.0 ms / p99 113.7 ms, so the
CPU gate FAILS against the documented Free limit; the request-volume gate passes
(peak 2,418/day). Mitigating observation, not a pass: production `097dc04d` already
shows p90 10.0 / p99 20.8 ms and no CPU-exceeded status in 30 days, so the limit is
evidently not terminating requests at these levels; I cannot say where it would.
Unresolved: which routes cause the tail, and whether analytics CPU includes
cold-start work that the per-request limit does not count.
Options (operator's call): (a) upgrade to Workers Paid, then follow the rollout as
written; (b) attribute the tail per route, reduce it, re-measure; (c) accept the
risk with a 10% canary watching for CPU-exceeded errors (weakest option).
No production traffic, custom domain, DNS or billing change is authorized by this file.

## Route attribution of the CPU tail (2026-11-04, operator chose to stay on Free)
Method: one route per UTC minute sent to preview `834b5d37` only; per-minute CPU read
from analytics (same read-only token). Statuses all success.
- Sample A (6 requests/route): every minute's first request cost ~30-80 ms CPU
  regardless of route (current.json, home es/en, article all show p99 29-35 ms with
  p50 0.4-3 ms). Search and agents-page minutes had p50 22 and 29 ms.
- Sample B (30 requests/route, n=28-30): current.json p50 0.7 / p90 1.8 / p99 29.9;
  article-46 API p50 1.5 / p90 4.3 / p99 33.2; agents page p50 0.8 / p90 2.8 / p99
  102.2; **search API p50 6.8 / p90 68.8 / p99 141.1 ms**.
- Reading: (1) search is the only route heavy in steady state (about 1 in 10+ calls
  over 10 ms; MCP `search` shares the code). (2) A separate one-off cost of ~30-100 ms
  hits the first request of a fresh isolate on any route. Sitemaps and other static
  assets are not Worker invocations, so they are not in this data.
- Sample B's agents-page p99 and sample A's agents-page p50 disagree; n is small and
  the pages/search ran in different isolates. Treat per-route ordering as
  directional, not exact.
Code hypotheses (UNTESTED, from reading reader/query-memory.mjs and
query-projection.mjs): search runs a Unicode-property regex (`\p{L}`, lookbehind) over
every provision's normalized text, and `rank()` does `text.split(needle)` inside the
sort comparator (repeated work on long text per comparison). Cold path:
`projectionAdapter` hashes and JSON-parses ~3.4 MB, then `memoryAdapter` does
`structuredClone` of the dataset, validates every provision, builds two normalized
copies per provision, and a per-provision `JSON.stringify` identity check.
Candidate fixes (need regression tests, native checks and a fresh preview
measurement): precompute rank once per match; replace the Unicode regex with
indexOf plus a boundary check; drop `structuredClone` of freshly parsed data;
precompute normalized text in the projection at build time. None applied.
Remeasuring requires uploading a new inactive Worker version (external write: needs
operator approval) and a new analytics read.

## Revised acceptance criterion (set 2026-11-04 BEFORE re-measuring)
The original "p99 under 10 ms" bar is unattainable: production `097dc04d` has p99 20.8 ms
and every route, including `/` and `/en` which never touch retrieval, shows a one-off
~30-100 ms first-request cost on a fresh isolate. A version passes the Free-plan CPU gate if:
1. Steady-state (not first request on an isolate) per-route p90 is under 10 ms for the
   search API, article API, agents page and `current.json`; search included.
2. The first-request cost is no worse than production's observed tail (p99 20.8 ms,
   n=18, low-confidence), judged on the same one-route-per-minute probe.
3. Zero `exceededCpu` / `exceededResources` statuses.
Plan: n about 100 per route against the NEW version ID only; no load on production.
Scope of the code change: search only (rank computed once per match; Unicode lookbehind
regex replaced by indexOf plus boundary check, with a differential regression test).
`worker/index.ts` builds the retrieval adapter at module top level, so its parse,
hash, clone and normalize work runs at startup (316 ms vs the 1 s startup limit), not
per request; those cold-path changes are deliberately NOT part of this pass.

## Search optimization candidate (2026-11-04, operator said "go ahead")
Code (uncommitted): `reader/query-memory.mjs` search only: rank computed once per
match (plain-object comparator; an array-destructuring comparator was measurably
slower when cold and was replaced); Unicode-lookbehind regex replaced by
`indexOf` plus code-point boundary check. Differential test
`reader/query-memory-search.test.mjs` against a frozen oracle
`reader/fixtures/query-memory-prior.mjs` (identical JSON over 35 queries x 3
instrument filters x 3 page sizes, cursors followed; also identical on the real
projection when `LEX_MEX_PROJECTION` is set). 7 deliberate mutations (word-char
class, boundary side, code-point handling, first-occurrence-only, rank sign) each
fail the test. Reader suite: 70 pass / 0 fail / 19 skipped (baseline 68/0/18).
Local Node CPU, real projection: warm search 'de' 18.7 -> 2.4 ms, 'cuentas'
3.6 -> 1.3 ms, 'segregados' 1.2 -> 1.3 ms (unchanged: only 3 matches); cold first
call 'segregados' 7.4 -> 6.4 ms, 'de' 41 -> 18 ms. So the change fixes broad queries
but is NOT expected to explain the remote 'segregados' tail by itself.
Uploaded INACTIVE version `aca7ddf1-706b-47d4-b6b4-c2f3847f19d8` (tag
lex-mex-search-opt) via `wrangler versions upload`; production still
`097dc04d` at 100%. Preview URL responds; remote SDK proof passes.
Re-measure sent 2026-11-04 06:32-06:36Z, 100 requests/route, one route per minute:
search 'segregados', search 'cuentas', article-46 API, agents page, current.json.
Analytics read-back PENDING (needs the read-only token again).

## Re-measure result for `aca7ddf1` (read back 2026-11-04, n=100/route, one route/minute)
CPU ms, status all success, zero errors:
| route | p50 | p90 | p99 |
| search 'segregados' | 4.0 | 10.1 | 71.6 |
| search 'cuentas' | 4.5 | 8.7 | 12.7 |
| article-46 API | 1.3 | 2.4 | 3.9 |
| agents page | 0.6 | 1.5 | 2.9 |
| current.json | 0.6 | 1.4 | 2.4 |
Before (`834b5d37`, n=30): search p50 6.8 / p90 68.8 / p99 141; article 1.5/4.3/33;
agents 0.8/2.8/102; current.json 0.7/1.8/30. Improvement is real, largest on search.
Production baseline (clean, `097dc04d`, 30 days, n=11,957): p50 0.9 / p90 3.2 /
p99 10.4 / p99.9 16.8 ms. The earlier "p99 20.8" came from only 18 requests and is
superseded. Version-wide `aca7ddf1` (n=516, mostly these probes): p50 1.7 / p90 6.8 /
p99 39.4 / p99.9 76.8; `834b5d37` (n=318): p90 34.9 / p99 102.2 / p99.9 141.1.
Against the criteria set before measuring:
1. Steady-state per-route p90 < 10 ms: MET for article, agents, current.json and
   search 'cuentas' (8.7); search 'segregados' is 10.1, borderline (that minute also
   contained cold requests; n=100 does not separate the two).
2. First-request cost no worse than production's tail: NOT MET. Cold requests on the
   new bundle still cost roughly 40-70 ms (p99 39-72 ms) against production's p99.9
   of 16.8 ms. Not caused by search: it hits every route including current.json.
   Hypotheses (UNTESTED): lazy compilation of the larger bundle (8.6 MB assets,
   MCP SDK, 3.4 MB projection module) on first use; per-request MCP server setup.
3. Zero exceededCpu/exceededResources: MET. Observation, not a pass: requests up to
   141 ms CPU (and 10-17 ms on production) all completed with status success on this
   Free account, so the documented 10 ms limit is not terminating them in practice
   here. The enforcement behaviour is not something I can guarantee from this data.
Verdict: improved but criterion 2 unmet; rollout remains on hold pending an
operator decision (see message): canary with exceeded-CPU watch, investigate the
cold-start tail first, or upgrade.

## Canary requested, NOT executed: asset version skew (2026-11-04)
Operator chose "canary at 10%". Before deploying, compared asset references between
production (`www.leg4ll4bs.com`, 100% `097dc04d`) and preview `aca7ddf1`:
`/` and `/en` reference DIFFERENT hashed JS files on each side
(`/assets/index-PDwRclog.js` vs `index-ClsUIDHE.js`; `layout-segment-context-BxHMh_eM.js`
vs `-z7i__ugK.js`; `link-DssIA9pv.js` vs `link-CeQqXYV4.js`). Cloudflare's gradual
deployment docs state that by default "consecutive requests from the same user -
including page reloads and asset fetches - can be handled by different versions".
A plain 10/90 split would therefore route some asset fetches to the version that lacks
that hashed file, so a visible fraction of page loads on the live site would lose
their JavaScript, far beyond the Lex-Mex routes. `/lex-mex/` and `/es` returned 308
on both sides and were not compared. Version affinity (`Cloudflare-Workers-Version-Key`
set by a zone rule) is the documented mitigation; it is zone configuration and is not
set up. Therefore: `wrangler versions deploy ...@10 ...@90` was dry-run only
(dry run succeeded) and must NOT be run without affinity or an explicit decision to
accept the breakage. Production remains `097dc04d` at 100%.
Abort triggers if a canary is later run: any exceededCpu/exceededResources or error
status, or any asset 404; abort by `wrangler versions deploy 097dc04d-5eb5-4f9a-a088-e8fac5609aa6@100 --name leg4ll4bs-web -y`.
A useful canary needs about 200 canary invocations (roughly a day at ~2,400
Worker requests/day and 10%); a 30 minute window sees about 5 requests.
Do not start external AI-client or organic-discovery tests until 100%: during any
split the agent routes exist on only the canary share of requests.
Provenance: `aca7ddf1` was built from UNCOMMITTED `reader/` changes in
/private/tmp (cleared on reboot) and has had no independent Codex review.

## Gate 1 — per-request CPU (OPEN)
Preview version `834b5d37-284d-4550-8b52-90be61a980ed`, Worker `leg4ll4bs-web`.
Sample sent 2026-11-04T05:35:05Z–05:35:24Z (UTC) plus earlier route loop: about 63
requests (10 each to search API, article-46 API, current.json, /lex-mex/agents/, /,
agents sitemap) and 5 runs of `reader/remote-runtime-proof.mjs` (MCP initialize,
tools/list, get_provision, 8 HTTP routes, 1 hostile-Origin POST each).
`wrangler tail` (with and without `--version-id`) captured no events, so no CPU
figure exists. Wall-clock timings in earlier proofs are NOT CPU.
Operator to read from dashboard (Workers & Pages → leg4ll4bs-web → Metrics, filter
version 834b5d37): CPU p50 / p99 / max, request count, CPU-exceeded errors.
Record (Cloudflare GraphQL `workersInvocationsAdaptive`, script `leg4ll4bs-web`,
queried 2026-11-04T05:57Z over the prior 3 h; CPU in microseconds, converted):
- Preview `834b5d37`, status success: 137 requests, 0 errors. CPU p50 1.58 ms,
  p90 37.0 ms, p99 113.7 ms, p99.9 126.9 ms. Wall p50 4.8 ms.
- Preview `834b5d37`, status clientDisconnected: 7 requests, 0 errors, CPU p50 3.9 ms,
  p90/p99 48.2 ms (open-stream or aborted requests).
- Production `097dc04d` (18 requests, 0 errors): CPU p50 4.2 ms, p90 10.0 ms,
  p99 20.8 ms. The preview is therefore not worse than what is serving today.
- Sample includes my ~130 test requests; the analytics API does not split by route,
  so the heavy tail is NOT attributed. Likely cold-isolate and the ~125 KB/47 KB
  pages, but that is a hypothesis. Warm p50 (1.6 ms) matches the local proxy.
- No `exceededCpu`/`exceededResources` status anywhere in the account over 30 days.
Result: p50 passes on any plan. p90/p99 (37/114 ms) FAIL a 10 ms Free-plan limit
and pass the Paid default (30 s) with large headroom.

Local proxy only (NOT Cloudflare CPU, does not close this gate): exact built host
in Miniflare/workerd, CPU = delta of workerd process cputime (`ps`, 10 ms
resolution) / N requests, after 20 warm-up requests, limiter raised to 1e6.
Per request, warm: current.json 0.30 ms; agents page 0.30; article-46 API 0.33;
article-46 HTML 0.33; MCP get_provision 0.60; home 0.70 (n=100); agents sitemap
0.70 (n=100); API search 1.77; MCP search 1.97 (n=300 unless noted).
First request on a fresh local workerd (3 runs each; module startup excluded):
wall 11.6–25.3 ms for search/home/article/current/MCP search; `ps` CPU reading 0 or
10 ms (resolution too coarse to separate, bounded above by the wall time, so
under ~25 ms). Not conclusive against the Free-plan 10 ms limit; remote
cold-request CPU (p99/max) is still required.
Harness: scratchpad `local-cpu.mjs` (not committed).

## Gate 2 — account plan and quota (OPEN)
Operator to read Billing / Workers overview: plan (Free or Workers Paid) and usage.
Limits per Cloudflare docs (fetched 2026-11-03/04, limits and pricing pages):
- Free: 10 ms CPU per request; 100,000 requests/day (reset 00:00 UTC). Dynamic
  MCP/search paths must fit 10 ms at p99 or the rollout fails on Free.
- Paid: 5 min CPU max (30 s default); 10M requests/month included, then $0.30/M;
  30M CPU-ms/month included, then $0.02/M CPU-ms.
- Both: startup 1 s (preview reported 316 ms); uncompressed Worker size 64 MiB
  (bundle 8.6 MiB uncompressed, 1.6 MiB gzip). Static asset requests are free.
- Rate-limit binding `LEX_MEX_LIMITER` (600/60 s per location, one shared key) is
  burst protection only, not a daily or billing cap; its binding limits were not
  confirmed from docs.
Record (scoped analytics-read token; billing/subscriptions and Worker settings
endpoints returned "Authentication error", so the plan itself is NOT read directly):
- Usage, all scripts on the account (Worker invocations only; static assets not
  counted): 12,584 requests in 30 days; peak day 2,418 (2026-11-03). That is 2.4% of
  the Free 100k/day cap and 0.13% of the Paid 10M/month inclusion.
- No CPU-exceeded or resource-exceeded invocation in 30 days, even though requests
  of 21–114 ms CPU were served (production p99 20.8 ms, preview p99 113.7 ms).
  A Free plan should terminate requests over 10 ms CPU, which points toward Paid and
  is consistent with `default_usage_model: standard` recorded earlier. BUT Wrangler
  version/deployment messages call production a "Free-tier migration candidate"
  (2026-10-25), so the plan may have changed or the analytics CPU figure may not map
  to the enforced limit. Evidence conflicts; the plan is UNVERIFIED. Still needed:
  operator reads the plan on the Billing page, or supplies a token with billing read.
- Rate-limit binding and daily/billing caps: unchanged, see above.
Result: request quota passes on either plan; CPU headroom depends on the plan.

## Pass criteria
Gate 1 p99 comfortably under the plan's CPU limit (Free: 10 ms) with zero
CPU-exceeded errors; Gate 2 plan and projected pilot traffic within quota.

## Proposed rollout (only after the plan is confirmed, gates pass and the operator approves)
Current active production: `097dc04d-5eb5-4f9a-a088-e8fac5609aa6`, 100% since
2026-10-25T06:40Z, deployment message "Final free-tier migration candidate on
staging". Syntax below verified with `--help` (wrangler 4.92.0); run from the
candidate host directory, which holds `wrangler.jsonc`.

PRECONDITION (plan): CONFIRMED FREE by the operator on 2026-11-04 (consistent with
Wrangler history labelling production a "Free-tier" candidate; the earlier Paid
inference was wrong).
- Paid: proceed. CPU tail (p99 114 ms) is far under the 30 s default.
- Free (current): DO NOT roll out as is. A 10 ms limit would break the heavy tail; first find the
  routes behind p90/p99 (per-route measurement or `wrangler tail` once it works),
  reduce them, and re-measure, or upgrade the plan.

Steps (each needs the operator's explicit go-ahead; none has been run):
1. Dry run: `wrangler versions deploy 834b5d37-284d-4550-8b52-90be61a980ed@10 097dc04d-5eb5-4f9a-a088-e8fac5609aa6@90 --name leg4ll4bs-web --dry-run`
2. Gradual 10% / 90%: same command without `--dry-run`, with
   `--message "Lex-Mex MCP pilot 10%"`. Observe 15–30 min: errors, CPU p99, 429/503.
3. Full: `wrangler versions deploy 834b5d37-284d-4550-8b52-90be61a980ed@100 --name leg4ll4bs-web --message "Lex-Mex MCP pilot 100%"`.
4. Smoke on the production origin:
   `node reader/remote-runtime-proof.mjs https://www.leg4ll4bs.com <out.json> <candidate host dir>`
   (the script's recorded `version` string is the preview ID; expect to adjust it
   or ignore that field for production).
5. Rollback at any step, no rebuild: `wrangler rollback 097dc04d-5eb5-4f9a-a088-e8fac5609aa6 --name leg4ll4bs-web --message "reason"`.
   `097dc04d` stays in the version list; no step above deletes it.
6. Decide whether version preview URLs stay enabled (enabled for the inactive upload).

Note: `wrangler.jsonc` in the candidate host sets `preview_urls: false` while the
preview was enabled out-of-band; a full config deploy would flip that, a
`versions deploy` of the uploaded version does not. Keep using `versions deploy`.

## Next acceptance steps (not run)
External AI-client tests (Q01–Q10, one question per message, per provider, exact
requests/responses kept) and organic-discovery tests in fresh sessions without the
domain or prior context. See `docs/lex-mex-agent-connection-candidate.md`.
