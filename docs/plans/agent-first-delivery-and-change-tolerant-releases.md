# Agent-first delivery and change-tolerant corpus releases

## Purpose, authority and baseline

Operator priorities2026-10-27: deliver a useful agent tool with a human interface;
tolerate missing links and badly drafted cross-references; automate ordinary and
model-readable commencement cases; stop rebuilding the whole corpus for routine
changes; benchmark mley.mx; preserve the four-instrument fintech test; keep wider
DCG/NOM ingestion separate; design for recurring DOF amendments.

These priorities guide this plan. They do not assert legal approval of a date,
permit model proposals to overwrite audited decisions, or implement a new trusted
schema by prose. Existing instructions and source/legal-review boundaries remain.
No deployment, paid resource, migration, worktree creation or commit is performed
or newly authorized by writing this plan. Use isolated operator-created worktrees
for implementation, with one writer per checkout.

Immutable implementation baseline: main `f0f8a868f` (integration record; accepted
normal-merge code at `dbc88a314`). The independently verified SQLite reference is release
`d82f53c073a53960d18f56c323d720ebce84dddcc2f70da949fa0ba25687539d`,
currently under `/Users/jr/Dev/lex-mex-parser-repair/derived/db/repair-a/`.
It and publicR7 are reusable inputs. Current site pin is public release `c305db4a…`, publication
`0d142a9a…`. Freeze exactly these instrument identities for the delivery pilot:
`lritf`, `lic`, `itf-dcg-2018`, `ifpe-dcg-2021`.

The two DCGs already selected remain in the package. New DCG families, source
parameters and NOM ingestion are excluded from its delivery gate. The future
DOF watcher can discover such documents without automatically admitting them.

## Current diagnosis and observable outcome

The accepted SQLite projection and tested retrieval contracts exist. The
repository's default query-preview CLI still opens the fixture or retained
four-instrument JSON adapter, not a production SQLite-backed service. The web
publication remains its independently stagedR7 artifact. Local database tests
and corpus rebuilds do not connect those surfaces automatically.

Outcome: one deployed, version-consistent four-instrument retrieval path that an
actual agent can use and a human can inspect, plus an update design that preserves
source evidence without whole-corpus work for every small change. No requirement
for perfect linking, all temporal questions answered, or full-corpus publication.

**M0–M1 are the current delivery milestone. M2–M4 are separate follow-up work
packages. Do not run the entire roadmap as one prerequisite for a useful reader.**

Sources: `docs/post-split-integration-readiness-2026-10-27.md`, accepted independent
reviews, `reader/query-{http,sqlite,preview}.mjs`, current public package manifest,
`crates/lex-db/src/{build,logical}.rs`, the operator retest
`docs/lexmex-retest-2026-10-24.md`, and
[the mLey public-code benchmark](../mley-surface-benchmark-2026-10-27.md).

## Product and integrity policy

| Situation | Delivery treatment |
| --- | --- |
| No extracted link, ambiguous target, bad publisher cross-reference, or target outside the package | Keep exact text/citation and explicit unresolved/ambiguous/unavailable state; reading and search continue. Aggregate in reports; no hand-maintained exception per instance is a launch prerequisite. |
| Resolved canonical target with no public route | Preserve target ID/status and show route unavailable; never invent an alias or throw away the whole page. |
| Source text, law ownership, IDs or exact spans are wrong; a resolved edge asserts a nonexistent target; audited decision overwritten | Integrity defect: stop promotion of the affected change. Preserve last accepted data while isolating the bad candidate. |
| An ordinary commencement date is computable | Apply the approved deterministic rule with source basis and recorded machine status. |
| A complex commencement clause is interpretable by a model | Model proposes a structured rule/evidence; deterministic validation and computation decide whether it fits an approved automation policy. Keep model origin visible. |
| Genuine legal ambiguity, missing triggering event or unsupported interpretation | Keep that determination pending; refer a concrete question/evidence to JRH. Other instruments remain usable. |
| PDF positioning or optional reader enhancement fails | Keep text/source links available; show a local retry/unavailable state. |

Known omissions become coverage states, not asserted legal nonexistence. A
citation of an article that the publisher failed to define must not be “fixed”
by guessing. Distinguish publisher defects from parser defects. Structural loss
that swallows text into the wrong article remains a real defect, not an acceptable
missing-link exception.

Changing current blanket closure gates/accepted-gap inventories requires a small
explicit implementation: validators, typed states, CLI exit policy and regression
fixtures together. Replace exhaustive “every unresolved target must already be in
this hand-maintained gap list” release gating with integrity checks plus coverage
change reports. Keep a compact representative regression set; changing coverage
counts alone must not force hand-editing acceptance fixtures every day.

## M0 — freeze and measure one delivery baseline

1. Reuse the named accepted SQLite reference; verify its stored identity and
   manifest. Do not rebuild it to name currentHEAD. Text/ID parity is already
   checked: all870 provisions match publicR7 (IFPE71, ITF132, LIC511, LRITF156).
   Complete source-metadata/coverage checks and preserve that pilot baseline;
   report any actual differences rather than silently updating it. If the artifact
   has moved, locate its accepted copy instead of automatically rebuilding.
2. Create the four-instrument allowlist/read projection and explicit coverage
   notice. No other federal/DCG/NOM work becomes a dependency. Cross-package
   references remain visible with unavailable routes as needed.
3. Freeze ten end-to-end tasks: exact article lookup, thematic search, reading
   context, following a valid reference, handling an unavailable target, source
   comparison, temporal qualification and one honest outside-coverage case. Include
   the operator's IFPE account-separation question with a bounded expected outcome:
   cite what these four instruments establish and disclose missing guidance.
4. Time source work, compile, DB load, verification, export and browser tests
   separately. Record CPU and wall time; mark sleeps/pauses where known. The
   historic10h53m was the strict split queue, not a measured duration of one SQL
   database write. A reported10+hour session must be attributed before optimizing.

Gate: reusable baseline, exact allowlist, task expectations and stage timings
recorded. No parser cleanup, model sweep or national database rebuild to start.

## M1 — connect the actual runtime and the reader

Use the existing query DTO contract and corrected SQL semantics. The public
service and human article route must read the same selected release. An agent
must be able to discover it, search, retrieve full exact text and inspect sources
without loading the integrity manifest or entire instrument.

Hosting fact checked2026-10-27: Cloudflare lists `node:sqlite` as a **module stub**,
not a working filesystem database implementation. The current Node adapter cannot
be assumed to run unchanged on the existing Workers site. Resolve this now.

Preferred bounded path for the existing host: prove a thin D1-backed read adapter
using only the four-instrument query projection. D1 supports SQLite/FTS5/JSON;
keep canonical Git and the accepted localSQLite artifact unchanged. Compare
queries/DTOs against the local reference adapter; no direct whole500MB audit DB
upload, no hosted canonical authority and no new ingestion parameters. A small
serving projection is a delivery artifact, not a redesign of the legal model.

Before creating remote resources, run a local runtime/parity proof and record
account limits/usage. If query semantics or the current free-host envelope fail,
stop this one technical choice with evidence; choose an explicit compatible
service runtime rather than starting another general database/platform project.
No paid upgrade or new hosting commitment is implied by this plan.

Build a direct article URL with useful server-readable HTML, machine JSON/text,
source identity, compact uncertainty metadata and previous/next context. KeepR7
branding. PDF viewing and overlays are progressive enhancements; no dependence
on JavaScript, viewport measurement or successful PDF layout to access legal text.
Reuse useful reader code; do not redesign the whole UI.

Gate: ten tasks work through the real deployment route in a supported agent
client and desktop/mobile browser. Verify text actually reaches the agent;
summarizer/quote caps, egress rejection and truncation are distinct client issues,
not solved by HTTP200. If needed, add a thin supported tool transport over the
same retrieval API; do not create a second legal-data backend. Record requests,
bytes/observed tokens, latency and exact-text/source fidelity. Establish realistic
budgets from the pilot, including a short common-task path, and keep them stable.
Any wrong-source answer or silent text loss fails; honest scope limitations pass
when expected. Validate optional-link failures without requiring a perfect graph.

## M2 — release identity and proportional verification

Keep provenance, but separate content, evidence/build receipts and serving code.
Do not redefine old manifest/digest versions. Current logical_content_v2 includes
input_files evidence and excludes corpus_release; it is not a pure normative-text
hash. Add any narrower semantic content fingerprint explicitly with a versioned
policy, compatible readers and fixtures.

- Exact source SHA/retrieval metadata remains per acquisition/instrument. Content
  and analysis components have stable hashes; release manifests compose them.
  A service build records its own code identity separately.
- A request resolves `current` once and stays on that release. Publishing a new
  manifest advances the pointer atomically; never mix versions within a response
  or silently substitute current content for an old pinned request.
- Preserve immutable source evidence and prior artifacts for audit/rollback.
  Public historical-serving retention is a separate choice; the existing
  current-only website policy does not require publishing every old site tree.
  Unavailable old versions return an explicit result, not misleading new text.
- Existing compiled-builder checks remain: never claim an old builder produced
  bytes compiled from different sources. Reusing an accepted DB with a newer
  compatible query service is legitimate and does not relabel its builder.
- Move incidental execution/review commentary out of content identity where it
  has no legal-data role. Keep legally relevant decisions/evidence pinned with
  their existing immutable acceptance links. Never edit an old accepted decision
  merely to simplify hashing.

| Change | Default verification/release action |
| --- | --- |
| Reader CSS/layout or query error handling | Targeted checks plus existing accepted dataset; no DB projection. |
| New retrieval service build, compatible data contract | Service/contract gates against frozen DB; dataset ID unchanged. |
| One source/instrument corrected | Changed-instrument parse/validation, affected dependency closure and query checks; publish a new data release when ready. |
| No-change official source check | Append freshness/check evidence; no text reparse or DB rebuild. |
| Schema/loader/digest boundary or substantial parser rule change | Broader tests and independent source/DB checks appropriate to that boundary. |
| Scheduled release audit | Full from-scratch comparison/reproducibility outside the ordinary UI development loop. |

Do not blindly run two full builds for each minor commit. Prove the build path's
determinism at the appropriate boundary and reuse its receipts for unchanged
inputs/toolchain. Add timings and explicit invalidation reasons so the runner
can explain why expensive work is necessary.

Gate: a reader-only change leaves accepted DB bytes/identity untouched; a no-change
source check does the same. A real data update creates an auditable new release
without relabeling or overwriting the parent. Existing provenance negatives and
human-decision protection still pass. ShipM1 before making this an expansive
identity migration.

## M3 — model-assisted temporal coverage without per-instrument rewiring

Retain the already accepted166 rule applications. Process remaining cases by
clause family, using a model to read the complete relevant clause and propose
base event, offset, calendar/business-day unit, exceptions, scope and exact spans.
Dates, triggers and applicability stay separate. Distinguish instrument original
commencement from when amended current wording took effect; neither retrieval
time nor consolidation date substitutes for them.

Validate model output against the temporal output schema before corpus entry.
Compute dates deterministically for approved, evidenced forms. Automatically
accept only the preauthorized unambiguous pattern policy and label the result
as machine-derived; do not silently upgrade it to lawyer_verified. Deterministic
arithmetic does not turn a model-interpreted legal premise into a deterministic
fact: retain that premise and its model provenance separately. A business-day
formula must identify its applicable calendar rather than assume weekends alone. New
interpretations remain model proposals until the matching policy/review exists.
Evidence of a future authorization/notice must be checked, not invented by a model.

Pilot a small representative batch, score against source/operator expectations,
then apply reusable rules to each matching family. Humans resolve the residual
ambiguous questions, not every ordinary row. Publish unresolved-rate and reason
counts; do not promise that all complex clauses will resolve before measuring.
These improvements must not block the four-instrument product milestone.

## M4 — incremental projections and recurring DOF updates

Separate frequent detection from accepted publication. Poll official DOF editions
with a durable cursor, captured URLs/timestamps/hashes, retry/backoff and an explicit
last-successful-check time. Identify candidate affected laws and put DCG/NOM/other
source classes in their own queues. Discovery does not silently expand this package.

Treat an amendment decree as an event, with publication, possible commencement,
target version and operations kept distinct. A model can propose typed insert,
replace, repeal or renumber operations using exact official text/spans. Apply
supported unambiguous operations only through deterministic Rust validation and
preauthorized policy. Incomplete text, ambiguous ownership, conditions or absent
base versions remain isolated proposals for review. Do not invent omitted text.

Stage an update against an expected parent hash, preserving original source and
before/after versions; make the event idempotent. One event affecting several
instruments publishes as one consistent release. Re-reading it is a no-op. Corrections
are new events, never hidden rewrites. Preserve every audited decision and prior
machine proposal. Roll back by restoring the pointer, not erasing event history.

The current db-project command performs a full fresh projection; an incremental
path is new implementation work. Prototype on a disposable copy of an accepted
artifact: one transaction updates changed inputs/rows/FTS and affected graph scopes,
then publishes a fresh immutable snapshot/manifest. Audit bundles stay separate
from the smaller serving projection.

Dependency closure must include outgoing edges and usages, incoming edges to
changed IDs, imported definitions and temporal dependencies. Alias/title changes
can require a broader relink; report that invalidation explicitly. An unresolved
citation becoming resolvable is a normal update, not a reason to maintain a new
manual exception. Global invariants remain checked; unchanged component hashes
must really correspond to unchanged inputs.

Gate: unchanged input, single-article replacement, new target ID with existing
incoming citations, and an amendment with unresolved commencement are tested.
Repeated events are no-ops; multi-law events are atomic; crash leaves the last
release usable. Compare incremental logical rows against a full rebuild for the
pilot, without requiring identical page layout between different construction
methods. Thereafter use full checks periodically and after boundary changes,
with measured change-scoped gates for routine daily/weekly updates. No weekly
release may depend on an operator manually repairing every broken reference.

## Verification, recovery and scope control

Before implementation commits follow repository-required checks. Change schemas,
Rust types, validators and fixtures together at trusted boundaries. Use data-driven
rule-family cases and nearest negatives; do not add an outlier-specific rule just
to make a count pass. Execute only affected E2E stages plus required release checks.
Preserve raw evidence and all unrelated work; no bulkclean/reset or overwriting
existing artifacts. On uncertainty quarantine the candidate component, not the
entire accepted corpus or the deployed four-instrument reader.

The sources below are research references, not authority over repository decisions:
- [mLey benchmark](../mley-surface-benchmark-2026-10-27.md)
- [Workers Node compatibility](https://developers.cloudflare.com/workers/runtime-apis/nodejs/)
- [D1 SQL/FTS5 support](https://developers.cloudflare.com/d1/sql-api/sql-statements/)
- [D1 limits](https://developers.cloudflare.com/d1/platform/limits/)

## Checkpoint and exactly one next action

2026-10-27: operator constraints captured; public mLey frontend/discovery inspected;
current runtime mismatch verified against official docs. M0 text/ID comparison
completed:870/870 match existing publicR7 and accepted d82SQLite. Source-metadata,
task expectations and runtime proof remain. No product, schema, DB,
cloud resource, source corpus or legal review was changed. Research/plan only.

Next: executeM0 and a local four-instrument Workers/D1 retrieval-parity proof forM1
against a frozen accepted release; stop unrelated corpus/DCG/NOM cleanup during
that delivery milestone. Deployment remains a final reviewable operator action.


### 2026-10-28 — Bounded temporal closure and concrete runtime trial

JRH confirms the five instruments are in force; main already records their
instrument status accordingly. LFPA29 governs month/year counting per operator;
exact dates remain separate and do not gate runtime. No canonical status edits,
calendar expansion or database rebuild required for this closure.

M0 task pack prepared: `docs/runtime-proof-test-pack-2026-10-28.md`, evaluator key,
shared questions, results template and score helper. All870reference IDs/texts
were rechecked against main, unchanged. Four provider runs have not occurred;
no live-runtime pass claimed. Public-entry web-tool access failed on this attempt,
which is recorded as a client observation, not proof of host downtime.

Current next action (supersedes the earlier next-action wording): run Q02/Q04/Q05
public smoke trials with Claude/Grok/Gemini/GPT; retain answers/errors and complete
the ten tasks where access works. Use those failures to target the local runtime
adapter/retrieval proof. No broader temporal work is a prerequisite.


### 2026-11-03 — Executable RAG, access and discovery repair

This section supersedes the pending provider-trial next action. Operator requests
an executable repair plan and favors RAG. Implement a retrieval-augmented answer
path over the accepted four-instrument release, alongside public searchable text.
RAG does not itself fix crawler access or make search engines select the site.
No implementation, deployment or new legal approval is claimed by this plan.

#### Ownership and execution contract

Use GPT-6.1-Sol for initial diagnosis, release/runtime and authentication decisions,
evidence contracts, acceptance review and integration. Use GPT-6.1-Terra for
bounded implementation after those contracts are fixed. Sol reviews changes to
legal meaning or evidence boundaries; Terra must not infer JRH approval. Delegate
named deterministic validation to the mechanical role under repository rules.

Work in the reader/service checkout selected after inspecting current branches,
instructions and writer ownership. Source reports are in
`/Users/jr/Dev/lex-mex-reader/docs/Agent_Tests/`; that path is not proof that it is
the correct implementation checkout. Use operator-created isolated worktrees if
simultaneous writers exist. Preserve existing dirty work. Commit and deployment
require current authorization; prepare and validate the complete candidate first.
Keep living execution state here; implementation handoffs reference this section.

Baseline: parser main f0f8a868fd78ce1cf0ed939ab82511daf5659725;
accepted d82 SQLite and four-instrument allowlist above. Inspect and record reader
HEAD, deployed release/publication and serving configuration before changing them.

#### Evidence and diagnosis

Read the four provider reports, Clara note, runtime proof pack and evaluator key,
and mLey benchmark already linked above. Reports are answers, not full transport
traces. Claude reports article access but misses article 46; Gemini uses snippets
and chooses article 35 for Q04; GPT reports inaccessible articles and a different
release ID; Grok supplies answers with quotation and qualification gaps. Do not
label all four as identical access failures. The screenshot shows mLey found by
ordinary article searches, but Claude received summaries, not full article text.
Our web-tool attempts at Lex-Mex discovery failed; this does not establish origin
downtime, WAF responsibility or authentication failure.

#### P0 — Reproduce and classify (Sol)

Capture timestamp, client/model/tool, exact URL/query, redirect chain, status,
content type, response body or tool error, release identity, latency and bytes.
Compare ordinary HTTP, browser, and actual provider tools for homepage, discovery,
coverage, article HTML, article text/JSON and search. Inspect robots, sitemap,
canonical URLs, caching, www redirects and deployed routes. Inspect available
Cloudflare logs/settings read-only; correlate requests before blaming bot controls.
Distinguish DNS/egress, WAF/challenge, 401/403 auth, missing route, stale release,
client summarization, truncation and search failure. Do not spoof trusted crawlers
or disable protections globally. Output a failure matrix and smallest repair.
Gate: each observed failure has evidence and either a demonstrated cause or an
explicit unresolved hypothesis. Do not repeat corpus builds to diagnose transport.

#### P1 — Shared retrieval and public evidence (Terra; Sol approves contract)

Expose search, full provision, references and coverage from one pinned release.
Reuse existing query DTOs and lexical retrieval. First prove local adapter parity
against accepted SQLite. Resolve Workers compatibility through the bounded D1
projection proof above or an evidenced compatible service choice. Docker can run
the Node/SQLite proof locally; installing Docker does not authorize remote hosting.
Do not add vectors until held-out queries demonstrate a lexical retrieval gap.

Provide stable human-readable article routes with server-delivered full text,
ordinary links and matching text/JSON endpoints. Map these to immutable provision
IDs and release-specific citations; never silently serve new text at an old pin.
Return source URL, source hash, retrieval date, recorded reform date, review status,
coverage and freshness limitations with each provision. Keep summaries separate.
Search returns ranked excerpts and IDs, followed by explicit full-text retrieval.
Support pagination and complete long-article retrieval without silent truncation.

Public legal reading/discovery should work anonymously without cookies, login,
JavaScript challenges or API keys. Separate public retrieval from administrative
and publication operations, which remain authenticated and authorized. If a model
answer endpoint is public, protect model spend with scoped authentication or
quotas/rate limits without blocking public text. Add MCP only when client evidence
shows it is needed, backed by the same API; a connector is not a discovery fix.
Gate: exact ID/text and DTO parity, release consistency, anonymous read success,
admin rejection for anonymous requests, explicit 404/coverage/rate-limit behavior.

#### P2 — Bounded RAG answers (Terra; Sol evaluates)

Implement question -> search -> full relevant provisions -> reference expansion
within a fixed budget -> answer with citations. Start with the four instruments.
Answer claims must link to retrieved provisions and exact supporting spans; the
model cannot invent source IDs, edit canonical records or confer legal approval.
Distinguish quotes, interpretation and uncertainty. Verify quoted spans and cited
IDs deterministically. A citation's existence alone does not prove entailment:
Sol reviews material claims against evidence, especially omissions/qualifications.
Treat source text as data, not instructions; test prompt injection in retrieved
content. Log retrieval and model steps without credentials or user-sensitive data.
If retrieval fails say so; if coverage is absent say so. External-source fallback
is an explicitly labeled separate mode and cannot count as a Lex-Mex-only pass.
Gate: ten-question pack, Q04/Q05 article 46 recovery, exact quotes, full-source
provenance and honest Q10 absence. Add IFC authorization as a held-out integration
case; flag missing supplementary AML/CONDUSEF materials instead of asserting a
complete filing checklist. Money transmitter is an outside-coverage case.

#### P3 — Discoverability (Terra; Sol reviews)

Publish instrument indexes and article pages with meaningful Spanish titles,
article numbers, official instrument names/aliases, descriptions, canonical URLs,
internal links, crawlable text, sitemap and accurate robots directives. Keep
llms.txt compact and useful, but do not assume providers read it or rank it.
Avoid duplicated indexed releases and fabricated freshness/authority claims.
Use existing site navigation to expose Lex-Mex. Prepare search-console ownership
and sitemap submission steps; execute external submissions only when authorized.
Gate: crawler-visible HTML and links verified, sitemap URLs resolve, indexability
checked. Search indexing and ranking remain external outcomes, not unit-test passes.

#### P4 — Promotion and real-provider acceptance (Sol)

Run applicable repository native checks and affected service/browser/E2E checks;
record actual commands, outputs and timings. Prepare reviewed deployment diff,
release pin and rollback to the last accepted publication. Obtain authorization
for deployment if none exists. After promotion repeat real-client trials.

Track three separate gates:
1. Directed access: fresh sessions in GPT, Claude, Gemini and Grok receive only
   the Lex-Mex entry URL and the question. Each must retrieve full evidence and
   answer the in-scope pack correctly. Unsupported clients remain documented
   failures or exclusions explicitly accepted by the operator; do not count them
   as passes. Retain exact transcripts/tool requests where available.
2. RAG correctness: all ten expected outcomes pass; zero wrong-source claims,
   invented citations or altered literal quotes. Record retrieved IDs, completeness,
   latency and token/cost measurements. Add paraphrased held-out questions so the
   service cannot pass through hard-coded answers. Keep oracle out of prompts.
3. Organic discovery: freeze six Spanish/English queries (three exact-article,
   three thematic), no site name/domain or prior Lex-Mex context. Run fresh sessions
   across the four providers after indexing, with dated follow-ups at 7/14/30 days.
   Record search results, position, selected source and actual evidence use.
   Target: every provider independently discovers and uses Lex-Mex on at least one
   query, and at least half of the 24 trials use it with supported citations.
   Repeat on a second date. These are product targets, not guaranteed SEO outcomes.

Directed and RAG success may ship while organic discovery remains pending, but do
not call the overall objective complete until the organic target passes or the
operator explicitly revises it. No whole-corpus expansion, temporal overhaul,
vector migration, new chatbot channel or visual redesign is a prerequisite.

Recovery: preserve raw evidence and accepted data; isolate failed adapter/service
changes; restore the serving pointer on regression. Stop promotion for wrong
source/text, silent version mixing, lost audited decisions or unauthorized access.

Current next action: Sol executes P0 and fixes the retrieval/auth/evidence contract,
then hands Terra P1–P3 as bounded tasks with exact paths and acceptance commands.


### P0 checkpoint — first independent HTTP comparison

Read-only Python urllib probe outside the network sandbox: discovery llms.txt
returned 200 with the expected c305 release pin; homepage, robots.txt and
sitemap.xml returned 403. Inside the sandbox all four failed DNS resolution.
Evidence: `docs/agent-access-p0-2026-11-03-evidence.json` and
`docs/agent-access-p0-2026-11-03-network-evidence.json`. The first probe's DNS
failure must not be attributed to the host. The external probe establishes
route-dependent access, not the cause of 403; error headers/body still need capture.
No service, corpus, authentication or hosting configuration changed.
Next action: capture 403 bodies/headers and probe the pinned index and article
shards, then inspect the actual serving configuration before choosing a repair.


### P0 checkpoint — route-specific rejection and working exact-text path

Evidence: `docs/agent-access-p0-route-details-2026-11-03.json` retains
headers and bodies. Homepage, robots and sitemap return HTTP 403 with body
`error code: 1010`. Pinned release index, coverage, LRITF index and article 11
Markdown return HTTP 200. Article 11 response is 1,843 bytes with stable ID,
official source and source hash. This disproves blanket public-corpus outage.
The response code narrows investigation to the host security/browser-signature
policy; account configuration/logs remain necessary to identify the exact rule.
Existing reader/query-http.mjs already renders full evidence HTML; reader/agent-pages.mjs
already generates crawlable article HTML. Reuse these rather than implementing a
second renderer. Actual host is the existing Sites project named in the R7
handoff; reader source alone cannot establish its deployed edge configuration.
Next action: inspect public agent entry and compare ordinary browser/HTTP access;
locate the Sites project's serving source and security settings read-only before
proposing the smallest route/security and retrieval connection change.


### P0 checkpoint — descriptive client and hosting access verified

`docs/agent-access-p0-application-client-2026-11-03.json`: identical public
routes with User-Agent LexMexAccessAudit/1.0 return 200 for homepage, robots
and sitemap; /lex-mex/agents/ returns 404. This demonstrates client-signature
sensitivity for the default urllib failures, without impersonating a browser.
Robots permits all crawlers and explicitly named providers; sitemap contains only
home, /en and Lex-Mex landing page, with no instrument/article URLs. Robots
permission does not override edge rejection. Sites get_site read-only confirmed
the existing project is active, access_mode public, current user owner, latest
version 26. No audience change or bypass token is needed for public corpus reads.
Do not persist credential-bearing get_site responses; retain only these facts.
Missing article sitemap entries and absent /agents/ route are concrete deployment
investigation targets, not proof that all hashed agent pages are absent.
Next action: inspect saved version 26 source provenance and the actual agent-page
entry emitted by the current package; prepare the public route/retrieval repair
against that existing hosting source, retaining accepted corpus bytes.


### P0 checkpoint — source integration gap found

Sites lists saved version 26 at source a0e7f884a7254dc01b952265e03ce3e38e615176
with deployment appgdep_6add4d9ca8a48191b301fff3adede899. A saved version's
latest deployment handle is not by itself proof that deployment succeeded.
Reader checkout is now 44f230ea4b69767c6aff59d1615f0cda0679f6c4 and dirty;
older handoff HEADs must not override it. Local host checkout also contains
substantial uncommitted Worker/build work; do not overwrite or publish it wholesale.

Concrete source gap on parser main: reader/agent-pages.mjs exports generated
hashed agent HTML, but reader/package.mjs never invokes it; stage.mjs rejects
agent HTML paths. Existing renderer presence therefore did not imply delivery.
Next bounded repair: integrate generated HTML into publication descriptor and
stager with integrity/path checks, add discovery entry and sitemap representation,
and prove immutable release bytes remain identical. Review current reader branch
implementation before choosing source base; preserve its retrieval-check work.
Host changes require its local instructions and comparison with saved source.


### Implementation checkpoint — explicit presentation publication option

Correction to the previous diagnosis: README and regression tests explicitly
exclude experimental agent routes from R7. This is an intentional boundary,
not an accidental missing call. New implementation preserves the default and
adds packageReader(..., {includeAgentPages:true}) for a new presentation package.
Generated hashed HTML and a small agents/index.html entry are hashed in its
publication descriptor; discovery is extended only in that opt-in package.
Stager allows narrowly shaped generated HTML paths while keeping canonical
release allowlists unchanged. Release input bytes and identity remain unchanged.
CLI exposure, host verifier compatibility, sitemap/canonical routes, real-data
staging and RAG remain unfinished. Syntax checks passed for package and stage.
Next action: finish targeted tests in parser main (not the separate reader
checkout), then exercise opt-in package and stage against accepted release input.


### Real-data packaging milestone

Opt-in package built and staged from accepted R7 publication in
`.work/reader-r7-review-20261024/publication`. All 1,783 release-file hashes
remain identical. Evidence: `docs/agent-publication-real-data-proof-2026-11-03.json`.
Candidate publication ee6bc12b4f17170ea4ef325d9c10075582674c46de5945ce2f503b9291d35070;
release c305 unchanged. Temporary candidate path is in that proof; regenerate if
cleaned. Both package and stage now expose --include-agent-pages; package test
checks default staging rejection and explicit opt-in success. Documentation updated.
No public deployment or query/model connection claimed.
Next action: run the complete affected Node suite, then implement canonical
article routes/sitemap and host verifier support before preparing promotion.


### Sitemap milestone

Opt-in packaging accepts an explicit HTTPS public origin and emits
agents/sitemap.xml listing full-provision pages, excluding term-result pages.
Real R7 proof: 870 URLs, all targets present in staged output; evidence
`docs/agent-sitemap-real-data-proof-2026-11-03.json`. New presentation publication
09852a3579aa04ffa758eeb80b696849e5a0a75ee1b0963bc55aff07f1fe7b29;
canonical release remains c305. Node package/stage/agent-page suite: 19 pass.
No indexing, host integration or organic-discovery success is claimed.
Next action: integrate canonical route metadata and host verifier/sitemap support,
then connect query retrieval and bounded RAG. Existing default R7 stays unchanged.


### RAG evidence retrieval first proof

Added reader/rag-evidence.mjs: bounded search-to-full-provision retrieval using
the shared adapter interface, release consistency and exact-text hash checks.
It records search IDs, coverage and budget exhaustion without truncating provisions.
The accepted SQLite adapter currently exposes the wider projection; do not connect
it to the four-instrument public path without a scoped serving adapter. First
proof therefore uses the already verified four-instrument retained adapter, not
SQLite runtime parity. Q04 queries segregados / identificados por cada cliente
recover full LRITF article 46 (3,364 serialized bytes). Evidence:
`docs/rag-q04-retrieval-proof-2026-11-03.json`. Syntax and real retrieval executed.
This is retrieval plumbing, not a model answer or completed RAG acceptance.
Next action: add meaningful integrity/budget/coverage regression tests and model
answer validation, then wire the bounded path into the actual runtime contract.


### RAG integrity and answer boundary milestone

Added reader/rag-answer.mjs with injected generator and deterministic validation
of release identity, cited retrieved IDs, exact supporting substrings, literal
quotation fidelity and explicit insufficient-evidence envelopes. This validates
citation mechanics only; a cited substring does not establish entailment or legal
completeness. No model provider has yet been wired or executed. Evidence retrieval
now validates source DTOs and checks release stability after coverage retrieval.
`node --test reader/rag.test.mjs`: 3 pass, covering full retrieval/deduplication,
whole-provision budget exclusion, altered text, release drift, invented citations,
altered quotations, wrong release and explicit abstention. Fixture generator is
synthetic; it is not a real-model answer acceptance result.
Next action: select and connect an available real model execution interface with
bounded cost/auth controls and run the frozen questions; connect the retrieval
service to the supported hosting runtime and finish host discovery integration.


### First real model evaluation — Q04 evidence-only trial

A fresh GPT-6.1-Terra medium read-only trial received only the saved Q04 evidence
bundle, with no oracle or other legal sources. It identified LRITF article 46
and supplied supporting spans. Current saved bundle actually records
budget_exceeded=true; an intermediate commentary incorrectly said false before
reading the artifact. That proposed failure finding is withdrawn. Reconcile saved
bundle contents and generation parameters before attributing budget behavior.
Retrieval now validates complete article and search-hit DTOs; RAG tests rerun:
3 pass. This trial is a local evidence-only model run, not deployed RAG acceptance.
Next action: preserve and mechanically validate the full model response against
current saved evidence, review qualifications and run Q05 before host integration.


### Q05 real model and answer transport milestone

Saved Q05 retrieval bundle and complete fresh Terra answer in
`docs/rag-q05-{retrieval-proof,terra-answer}-2026-11-03.json`.
Retrieved full articles 22 and 46. validateAnswer ran successfully against the
saved response: all supporting spans exist and release matches. Parent review
confirms the response distinguishes per-client electronic accounts from deposit
accounts, segregation/identification, end-of-day treatment and the IFPE maximum
of 1m UDIS versus twice peak 24-hour redemptions over 365 days. Model explicitly
qualifies partial evidence and freshness. This remains local evaluation, not
legal approval or a full ten-question pass.

Added reader/rag-http.mjs Node integration boundary for POST /lex-mex/api/answer:
explicit injected adapter/generator, bearer requirement, request size cap,
no-store responses, required model-budget authorization and fail-closed invalid
answer behavior. No production credentials or provider hard-coded. It is not yet
mounted or deployed; no public-auth success claimed. Tests:
node --test reader/rag.test.mjs reader/rag-http.test.mjs — 5 pass.
Next action: complete transport error/abuse cases and mount local end-to-end
retrieval/generation, then solve Workers-compatible serving and host integration.


### Local HTTP integration milestone

queryServer now mounts the answer handler only on /lex-mex/api/answer, passes
streamed body/headers, and preserves public anonymous retrieval. Answer generation
still requires explicit generator, bearer and budget callback; CLI defaults do
not enable it. Budget-service failure returns 503 without model execution.
21 query/RAG/HTTP tests pass on the exact rerun with loopback permission after
sandbox EPERM (19 initially passed, two socket tests could not bind). Parent ran
the permitted rerun because mechanical delegation could not complete socket checks
inside its sandbox. No live model over HTTP, Workers compatibility or deployment
is established by synthetic-generator transport tests.
Next action: implement bounded real provider execution and Workers-compatible
retrieval; finish host verifier integration before preparing promotion.


### Bounded filesystem-free projection candidate

Refactored retainedData export from the verified public loader; existing
retainedAdapter behavior preserved. Added query-projection.mjs to load pinned
bytes without filesystem access, reject digest/scope violations and bound pilot
size. Candidate generated from accepted R7 public files: 870 provisions,
3,350,705 JSON bytes, SHA 8d2dd5aece532d0bf13240d09b3496a3f4c29d0b2f90c73545b96c9e138390a6.
Artifact .work/rag-serving/projection.json; measurement receipt
`docs/rag-serving-projection-proof-2026-11-03.json`. Node load/export ~157ms,
heap delta ~24.6MB; these are neither Workers memory nor CPU measurements.
No SQLite rebuild or source change. This is an alternative candidate for the
bounded four-instrument runtime, not a decision to replace the preferred D1
proof or a national-scale in-memory service. Actual Worker runtime and account
limits must decide viability before selection. Six retained/projection/RAG tests
pass. Next action: prove actual Workers runtime compatibility and measured limits
for candidate retrieval, compare with D1 needs, then integrate the selected host.


### Actual local Workers runtime proof

Used existing host esbuild and Miniflare/workerd, without deployment or install.
Bundled accepted 3.35MB public projection with query-projection/memory/http modules,
explicit node:buffer global shim, nodejs_compat and existing host date 2026-05-15.
Local workerd returns 200 for discovery, search segregados and full article 46;
article response contains expected text. Evidence:
`docs/rag-worker-runtime-proof-2026-11-03.json`. First request wall ~656ms
(including runtime initialization), warm search ~6.5ms, article ~1.7ms.
These are local wall times, not CPU, startup-limit or memory-limit acceptance.
Prototype entry/bundle under .work/rag-worker-proof; not yet production source.
Node builtin compatibility is demonstrated locally; full host integration,
compressed bundle limits, cold CPU and memory still need verification. Do not
claim this completes D1 parity or select a wider in-memory backend by implication.
Next action: promote reproducible runtime harness into source, measure bundle
limits and compare serving semantics with accepted SQLite, then choose and
integrate the bounded host adapter and real model execution.


### Full provision parity and reproducible workerd harness

Compared all 870 projection provisions against accepted d82 SQLite lookup:
IDs/text, temporal and review status agree. 28 source DTO differences remain,
including separately acquired IFPE annex PDFs attributed by query-sqlite.mjs to
the parent instrument PDF. Keep published annex provenance; do not overwrite it
to make adapter parity pass. Evidence:
`docs/rag-sqlite-projection-parity-2026-11-03.json`. Four sampled LRITF searches
agree for segregados, capital mínimo and cuentas de fondos; autorización ranks
differently. Full search-contract parity is not established.

Tracked reader/worker-runtime-proof.mjs reproduces local bundling and actual
workerd responses using caller-selected installed tooling and new output dir.
Executed successfully with accepted projection; output under
.work/rag-worker-reproducible-1/proof.json. Bundle 3,387,172 bytes raw,
573,367 gzip; all three runtime requests 200. No production limit claim.
Next action: fix or explicitly isolate SQLite annex-source attribution with a
regression fixture, settle search semantics and runtime choice, then integrate
host/verifier and provider execution without changing accepted source artifacts.


### SQLite annex source defect isolated

Accepted database source_manifests retains annex acquisitions but source_anchors
contains only id/kind/instrument_id, and provision extras have no manifest binding.
Do not infer mapping from annex number or manifest order. query-sqlite now rejects
annex lookup when separate annex manifests exist, rather than citing parent PDF.
Regression demonstrates explicit rejection and unaffected ordinary article lookup.
Portable regression suite: 6 passed, 3 real-DB cases skipped (env not set).
This is containment, not complete annex mapping support. Correct annex provenance
continues in accepted public projection, so pilot delivery need not lose annexes.
Next action: select measured bounded public projection for pilot with its documented
lexical semantics (not SQLite ranking parity), finish host source integration and
real model execution; retain explicit SQLite annex limitation as follow-up.


### Reusable Worker integration boundary

Added reader/query-worker.mjs factory over pinned projection, public retrieval
and gated answer handler. Replaced implicit Buffer globals with explicit imports;
tracked runtime harness now uses this same factory, with no global Buffer shim.
Local workerd rerun: discovery/search/article 200; gzip bundle 575,027 bytes.
Proof .work/rag-worker-integrated-1/proof.json. README specifies exact host routes,
asset-first interaction, per-request credentials and fail-closed answer setup.
Read host AGENTS/CODEX and current Worker/verifier. Current host source includes
uncommitted migration routing; no edits or deployment to that checkout performed.
Host verifier already hashes descriptor-listed files, but build audit and
run_worker_first routes still require a reviewed integration candidate.
Next action: finish that candidate in an isolated source copy, run host build and
runtime checks, then connect a supported real generator and authorization budget.


### Isolated host candidate build milestone

Authority correction: current host HANDOFF records own-zone Worker production
since 2026-10-25; Sites26 is rollback, not canonical serving. Earlier Sites status
inspection did not establish live routing. Preserve this distinction.
Temporary isolated source copy path is in docs/rag-host-integration-proof-2026-11-03.json.
Candidate imports the reusable retrieval Worker, routes dynamic retrieval before
framework fallback, extends run_worker_first, stages opt-in publication, updates
pin and adds all 870 article URLs to root sitemap. Canonical host source unchanged.
Integration patch retained at docs/rag-host-integration-candidate-2026-11-03.patch;
requires copying the named reader modules/projection and staging publication too.
Exact npm test passes: production build, 7700 artifact verification, 7818 asset
build audit, 32 PDFs and 8 host tests. Initial sandbox Vite cache EPERM cleared on
permitted rerun; parent executed because delegated sandbox cannot write shared
Vite cache. This does not test the new dynamic routes in the built host yet.
Next action: exercise built-host retrieval and answer denial in local workerd,
measure actual bundled startup/CPU limits, and connect real generator before
operator-reviewed deployment. No commit/push/deploy performed.


### Built host runtime acceptance (local)

Ran isolated candidate's dist/server/wrangler.json through Wrangler local.
Verified authored / and /en, dynamic discovery, search, full article46, static
agent entry and both sitemaps return 200; sitemaps contain >=870 URLs. Unconfigured
POST answer returns 503 without generation. Evidence:
`docs/rag-built-host-runtime-proof-2026-11-03.json`. Local server explicitly stopped.
Wrangler check startup completed and wrote candidate worker-startup.cpuprofile;
this is local profiling only, not production CPU/memory/startup-limit acceptance.
No relevant provider API keys present in session (presence-only check, no values
read/output). Asked operator to select hosted model connection or visiting-agent
generation; credential values must use secret mechanisms. This choice does not
block continued public retrieval/discovery work. No deployment performed.
Next action: finish canonical page metadata and ten-question retrieval acceptance
while model connection choice is pending, then prepare the reviewed deployment.


### Article metadata milestone

Generated article HTML now includes descriptions and explicit release-pinned
canonical retrieval URLs when packaging with publicOrigin. Metadata is generated
before the presentation content ID, so changed bytes change its hashed prefix.
20 agent-page/package/stage tests pass. Real accepted-data package verifies
metadata on all 870 article pages; evidence:
`docs/agent-canonical-metadata-proof-2026-11-03.json`. Existing isolated host
candidate predates this new presentation ID and must be restaged/revalidated;
do not deploy its older patch/pin as if it includes this change. Canonical URL
resolution and sitemap canonical alignment still need integrated verification.
Next action: align sitemap URLs with canonical article routes, restage host
candidate and verify all 870 canonical routes against the same release.


### 2026-11-03 — Visiting-agent generation direction

Operator asks whether RAG should use the user's own AI agent and whether MCP is
needed. Preferred architecture: Lex-Mex hosts deterministic retrieval and evidence;
the visiting agent performs query planning, generation and citations using its
own model/account. Hosted model API keys and a Lex-Mex answer-model endpoint are
not prerequisites for this architecture. Existing rag-answer/http code remains
an optional experiment, not a new launch dependency. User retains client/model
subscription and permissions; we cannot guarantee every client can connect.

Deliver two transports over the same retrieval implementation: public crawlable
HTML/text/JSON for ordinary web discovery, and a thin read-only remote MCP adapter
for explicitly connected compatible clients. Tools: search, get_provision,
get_references and get_coverage with release-bound results and full provenance.
MCP is useful for reliable structured access but does not cause automatic search
ranking/discovery or automatically install a connector. Public reading needs no
user model credentials. Any MCP authentication protects Lex-Mex access/quotas,
not access to the user's AI subscription. Client-specific compatibility must be
actually tested; unsupported clients retain the web path.

Acceptance remains answers generated by real visiting agents from fetched full
provisions, plus unprompted organic discovery trials. Local injected-generator
HTTP tests cannot substitute for that. Next action: finish canonical/sitemap
alignment and add the read-only MCP transport with actual supported-client tests;
keep hosted generation out of the critical path pending any contrary direction.


### Read-only tool facade milestone

Added reader/mcp-tools.mjs: four annotated read-only tools and strict argument /
release pin checks; calls reuse handlePublicQuery rather than duplicate legal
retrieval. Search and references expose pagination; results carry original public
DTO plus text content. No write tools, user model keys or hosted generation.
node --test reader/mcp-tools.test.mjs: 1 pass (provision evidence, release
substitution and unknown arguments). This is a protocol-independent facade;
MCP SDK registration, transport lifecycle/security and actual client connection
remain unfinished. Do not call it a deployed MCP server.
Next action: register facade with current supported MCP SDK and test remote HTTP
transport with a real client, while completing canonical sitemap alignment.


### MCP SDK transport milestone and install incident

Pinned official @modelcontextprotocol/sdk 1.32.2 in reader/package.json and
package-lock.json; reader/node_modules ignored. Added stateless Web Standard
Streamable HTTP server over the four read-only tools, Origin validation and
JSON responses. Official SDK client initializes, lists four tools and retrieves
pinned evidence; hostile Origin returns 403. Two MCP tests pass. Transport not
yet mounted/deployed; rate limits, request caps and actual external clients remain.

Installation incident: an initial command used reader/package.json relative to
reader cwd, so manifest creation failed. npm then resolved /Users/jr and reported
removing 279 packages there. /Users/jr/package-lock.json now exists (81 bytes),
no parent package.json found. Previous dependency inventory is unknown; no claim
of restoration. Do not guess or reinstall unknown ancestor dependencies. Subsequent
install used explicit --prefix /Users/jr/Dev/lex-mex/reader and succeeded (95
packages). Report this unrelated dependency side effect to operator. Repository
source and canonical host checkout were not overwritten by that npm operation.
Next action: harden and mount MCP in the candidate Worker, test official client
against actual workerd HTTP, then finish canonical sitemap and provider trials.


### MCP in actual workerd milestone

Mounted /lex-mex/mcp in reusable Worker factory. Added streamed 8KiB request cap,
Origin rejection and no-store protocol responses. Official SDK client against
local workerd successfully initializes, lists four tools and retrieves exact
article46 with provenance. Evidence: docs/mcp-workerd-client-proof-2026-11-03.json.
MCP-inclusive bundle gzip 660,412 bytes. esbuild harness now explicitly resolves
module/main dependency entry points. No private model keys or generation needed.
This is local protocol interoperability, not external ChatGPT/Claude/Gemini/Grok
client acceptance or deployed service. Existing host candidate lacks SDK dependency
and /mcp routing; refresh it and its lockfile before any promotion. Host-level
abuse limits remain required. Next action: canonical sitemap alignment, updated
host candidate MCP route/dependency integration, and actual visiting-agent trial.


### All canonical article routes verified

Sitemap now lists the same release-pinned canonical URLs advertised in generated
article HTML, instead of alternate hashed presentation URLs. Real accepted R7
package a3a0d276af9e2ab6fc2d4a5c39097e4a585d4c5df89d9c0ef885c886a5ff435d
has 870 unique canonical URLs. Every route returned 200 through shared retrieval
handler and exact escaped full text matched all 870 projection records.
Evidence docs/agent-canonical-route-parity-2026-11-03.json. This is complete
handler-level route/text parity, not network acceptance of the built/deployed host.
Candidate host must restage this latest publication and add MCP route/dependencies.
Next action: refresh isolated host candidate with canonical publication and MCP,
run integrated host build/client proof, then prepare operator-reviewable promotion.


### Refreshed host MCP build; integrated runtime failure remains

Temporary candidate now stages canonical publication a3a0d276, adds /lex-mex/mcp
routing/run_worker_first and pins SDK1.32.2 in candidate package/lock. Candidate
has an independent node_modules link directory; canonical host dependencies were
not changed. npm test passes production build, 7700 listed artifact hashes and
8 host tests. Audit shows 13726 assets because an older generated agents tree was
retained; remove obsolete candidate-only generated output before promotion.

Integrated Wrangler dev failed before accepting connections: first process
session76118 exited1 with ERR_IPC_CHANNEL_CLOSED; SDK HTTP attempt got ECONNREFUSED.
Retained-terminal retry session53077 also exited1 after parsing header rules,
without diagnostic. Neither is live; do not claim a built-host MCP HTTP pass or
poll/restart these handles as running. Earlier standalone workerd MCP proof still
stands. No production deployment. Next action: diagnose built-host startup using
Wrangler logs/dry-run bundle and remove obsolete candidate-only generated tree,
then rerun actual SDK-over-HTTP acceptance before promotion.


### Built-module startup diagnosis

Fresh-port Wrangler debug retry exited1 with LocalRuntimeController spawn EBADF;
no live process remains. Log /private/tmp/lex-host-mcp-startup.log. This occurs
while starting workerd, before application requests, not an MCP response failure.
Removed one obsolete hashed agent tree from candidate-generated public output.

Loaded exact dist/server built host JS modules directly in Miniflare with explicit
module inventory (automatic discovery rejects Vinext dynamic imports). Actual
built host /lex-mex/mcp initialize returns 200 and advertises four-tool capability
and release instructions. Evidence docs/mcp-built-modules-initialize-proof-2026-11-03.json.
Assets binding and SDK full calls not exercised in this narrower diagnostic.
This establishes built application can initialize under workerd; Wrangler launcher
failure remains separate. Next action: run official SDK tool discovery/calls and
asset-bound route checks against explicitly inventoried built modules, then
resolve launcher environment and refresh final build audit/promotion artifacts.


### Exact built host + native assets + SDK client pass

Tracked reader/host-runtime-proof.mjs enumerates built modules explicitly and
configures native Miniflare assets with has_user_worker=true plus actual selective
routing. Official SDK initialization/tool listing/article46 call passes against
that exact built host; authored /, /en, agent entry, both >=870 canonical sitemaps
and pinned article46 HTML return 200. Evidence:
`docs/mcp-built-host-runtime-proof-2026-11-03.json`. No substitute renderer or mock
legal backend. Rebuilt output after obsolete-tree removal audits 7818 assets,
7700 receipt-listed artifacts and 32 PDFs. No production CPU/memory guarantee.
Wrangler dev launcher EBADF remains an environment/tooling issue; exact built
application and native asset binding work in direct workerd proof.
Next action: run ten-question visiting-agent acceptance through the real tool
surface, full required checks and independent integration review; refresh stale
candidate patch/receipts and prepare concrete deployment approval request only
when all remaining promotion checks and actual account-limit evidence are ready.


### Blind connected-agent evaluation started

Fresh GPT-6.1-Terra medium visiting_agent_ten_questions is running with only
question pack plus reader/mcp-trial-cli.mjs access to exact built-host MCP.
Oracle/corpus/report reads prohibited. Harness records actual calls/results in
.work/visiting-agent-mcp-trace.jsonl; final report requested under docs.
No pass claimed while the agent is running. Tool inspection found instrument was
not marked required for provision/reference schemas; fixed in source, plus cursor
budget aligned to 1024. Three MCP tests pass. Built candidate predates those two
schema fixes; refresh it after trial so trial evidence identifies actual version.
Next action: receive blind report, inspect tool trace and score against frozen
oracle separately; fix any retrieval/answer defects before promotion.


### Blind local ten-question content assessment

Fresh Terra completed docs/visiting-agent-terra-ten-questions-2026-11-03.md.
Parent compared content to frozen oracle and MCP trace: all 10 expected content
outcomes met; all required provision IDs actually retrieved; Q03 first paragraph
matches exactly. Trace has 33 calls and 11 full provisions. Score receipt:
docs/visiting-agent-terra-ten-questions-score-2026-11-03.json. Q04 recovered46;
Q05 distinguishes electronic accounts and bank deposits, segregation and limits;
Q09 discloses freshness; Q10 refuses outside coverage.
Limitations: one Terra model, local built-host harness, all questions presented
in one file and answers composed together rather than per-message conversation.
Therefore not full prescribed runtime/protocol gate or four-provider acceptance.
Independent static review of six retrieval/MCP/RAG modules found no concrete
blocking issue, but did not independently test every tool/deployment path.
Required native checks are running in mechanical agent; no pass claimed yet.
Next action: collect required checks, refresh candidate tool-schema changes and
integration artifacts, then assess deployment readiness and remaining actual
account/runtime limits before requesting concrete promotion authorization.


### Final candidate checks and account limitation

Refreshed candidate tool schemas; npm test rerun passes full build/audit and
8 host tests. Wrangler deploy --dry-run succeeds without publication:
8645.82KiB total / 1594.61KiB gzip, ASSETS only. Output directory
/private/tmp/lex-host-dry-run-final. Integration patch refreshed for Worker,
routing, canonical sitemap, pin and SDK package dependency; it still requires
module/projection copy and package-lock update described above, not patch alone.
Cloudflare whoami reports not authenticated; actual account usage/settings and
promotion remain unverified. Do not initiate login or claim account checks passed.

Mechanical fmt/Clippy passed. Cargo test observation lost; parent read-only ps
verified three existing cargo-test processes (55993,56055,56073) with lex_db
children. No restart/kill performed. Their terminal results are not captured, so
no full native-test pass is claimed. Next action: observe those live processes to
terminal, obtain durable native test/validator results and actual account limit
checks after sign-in; prepare concrete reviewed promotion once gates are met.


### Required native validation captured

Confirmed old Cargo processes absent before starting one durable captured run.
cargo test --workspace, cargo run --locked -p lex-cli -- validate lritf, and
validate ifpe-dcg-2021 all exit0. Logs under .work/final-native-validation;
receipt docs/rag-native-validation-2026-11-03.json includes counts and commands.
Fmt and strict workspace/all-target Clippy passed in prior mechanical run.
Parent captured final run because mechanical observation lost its wait handle;
no unknown live process was restarted. This verifies current parser checkout,
not a committed/released artifact (no commit authorization).
Connection guide docs/lex-mex-agent-connection-candidate.md distinguishes public
browsing and connected MCP, visiting-agent generation, proposed endpoints and
remaining external-client/organic discovery gates. Next action: final source
review and production-runtime/account readiness; prepare bounded promotion for
operator authorization. CLI Cloudflare sign-in still absent; no deployment.


### 2026-11-03 — Account-switch resumption and readiness check

Wrangler OAuth login succeeded. whoami confirms configured Cloudflare account
ebde350a36f10106d626326a6089d869. Read-only deployment history confirms active
version 097dc04d-5eb5-4f9a-a088-e8fac5609aa6, matching host handoff. Account
settings report default_usage_model standard; this does not establish billing
plan, remaining quota or CPU headroom. Worker settings retain nodejs_compat,
2026-05-15 compatibility date and ASSETS binding only.

Re-executed exact built-host/native-assets official SDK proof against retained
candidate /private/tmp/lex-host-candidate-2z13gshb/site. All four tools listed;
article46 evidence retrieved; authored pages, agent entry and both sitemaps pass.
Publication a3a0d276 remains consistent. Proof .work/resumed-host-runtime-proof.json.
Initial sandbox listen EPERM cleared by approved local-port rerun.

Promotion review found MCP handler explicitly delegates abuse limits to host,
but candidate Worker/config shows no rate-limit binding. Production CPU, plan
and quota readiness remain unverified; do not infer them from dry-run bundle size.
No deployment, canonical host edit, commit or push performed.
Next action: finish host abuse-limit implementation and account/runtime budget
verification, then request approval for the exact bounded promotion. Preserve
existing production version as rollback and keep external-client/organic tests separate.


### Resumed host hardening and reviewable next step

Isolated candidate now binds LEX_MEX_LIMITER, namespace2026110301, at600
requests/60seconds per location with one shared anonymous-pilot key. Dynamic
retrieval fails503 if binding is missing/errors and returns429/Retry-After60
when exceeded. Authored pages and static assets remain outside this budget.
This is approximate burst protection, not a global daily usage/billing cap.
Cloudflare reference: https://developers.cloudflare.com/workers/runtime-apis/bindings/rate-limit/

Generated candidate Worker types; npm test passed full build plus8host tests.
Updated host-runtime-proof.mjs supplies native rate-limit binding and tests
actual excess-request rejection plus continued home availability. Proof
docs/mcp-resumed-host-runtime-proof-2026-11-03.json passes exact built host/MCP.
Dry-run passes:8646.33KiB total,1594.82KiB gzip; ASSETS and limiter only.
Log /private/tmp/lex-host-resumed-dry-run.log. Candidate patch refreshed; as
before it additionally needs module/projection copy, staged publication and
package-lock/types. No canonical host source changed.

Next action: obtain operator approval to upload this exact candidate as an
inactive Worker version with a version preview URL for remote CPU/runtime
validation; do not change production traffic, custom domains, DNS or billing.
Production rollout remains separate until actual remote acceptance. Account
settings do not prove paid/free subscription or remaining quota.


### Authorized inactive upload and remote checks

Operator explicitly approved candidate upload for remote runtime checks.
Uploaded inactive version834b5d37-284d-4550-8b52-90be61a980ed with tag
lex-mex-mcp-pilot; Cloudflare reports startup316ms. Version URL:
https://834b5d37-leg4ll4bs-web.r4m1r0qu41.workers.dev . Enabled version
preview URLs (previously disabled), preserving existing workers.dev enabled
state. This exposes version previews; no production traffic deployment,
custom-domain/trigger/DNS/billing change occurred.

Official remote SDK connects, lists four tools and retrieves article46 with
segregation text. Eight HTTP routes200 include authored languages, reader,
discovery, canonical article and both870-entry sitemaps. Hostile Origin403.
Evidence docs/mcp-remote-runtime-proof-2026-11-03.json, reproducible command
node reader/remote-runtime-proof.mjs <origin> <output>. This verifies remote
functionality, not per-invocation CPU/memory headroom or provider conversations.
No production promotion is authorized by the inactive-upload approval.
Next action: assess production CPU/quota readiness and obtain separate
production promotion approval; external provider/organic trials remain pending.


### Operator preview acceptance and commit authorization

Operator reports the interactive preview works well and explicitly authorizes
committing all pending Lex-Mex work. This records interactive acceptance without
asserting external-provider/organic tests or production rollout authorization.
Scope is this repository; canonical website checkout remains untouched.
Wrangler local cache is ignored; credentials and generated dependencies excluded.
Remote runtime harness now accepts candidate host directory explicitly rather
than retaining a temporary checkout path in executable code.
Next action: finish required commit checks and create a scoped local commit;
production readiness/rollout remains a later separately authorized step.

Commit gate: cargo fmt --check, strict workspace/all-target Clippy, all447Rust
tests and both required validators passed. Reader tests68passed/18skipped/0failed
with local-port permission; logs .work/commit-validation/. Remote portable
harness also passed all8routes and official SDK retrieval. Staged diff reviewed;
no canonical corpus changes, credentials or Wrangler cache included.
Integration patch uses zero context (apply with git apply --unidiff-zero).
Next action after local commit: assess CPU/quota headroom before any separately
authorized production rollout; external-provider and organic acceptance remain.


### 2026-11-07 — Production readiness review and fresh-agent mLey baseline

Live production9031ec9e verified, superseding stale capsule205 navigation.
Read-only official SDK/eightHTTP paths pass. Actual tail CPU0–63ms with
search63/14/17ms, no exception/outcome failure in17events. Free10ms headroom
not established; subscription read403, current quota unavailable. Both limiter
bindings live. Fresh no-context agent45URL attempts against both public sites;
Lex8complete/2partial outcomes versus frozen key. mLey offers broader law coverage
but DCG tasks not located; no universal correctness/speed ranking. Matched
curl entry timings and client-specific transport failures recorded. Python
PDF access403 reproduced while alternate user-agent206; official source remains
available to that client. Host MCP405 guard absent in producer; no code edited.
Report docs/production-readiness-review-2026-11-07.md and linked evidence.
No deploy/commit/push, no canonical data edits.
Next action: profile/reduce CPU and verify quota, synchronize producer405 guard,
and improve compact source discovery before repeating partial fresh-agent tasks.
