# Lex-Mex: parser value, public reader, and unattended ingestion

2026-10-06. Assessment against repository baseline 8031ea11 and the two
user-supplied Spearhead audits. Implementation status belongs to
`docs/plans/temporal-boundary-repair-and-public-reader.md`.

## Judgment

Keep the parser. Reduce the amount of enrichment required before a source becomes
usable. A document-level Markdown copy is a useful acquisition/search fallback,
but it does not provide stable article identities, validated citation targets,
source-span checks, deterministic diffs, or review history. Those capabilities
are valuable for agents and humans. They must earn their maintenance cost through
observable retrieval accuracy and useful published workflows.

Lex-Mex has a stronger *potential* evidence model than a plain document mirror;
it is not yet demonstrably the better user product. mLey is shipped, searchable,
and easy to verify visually. A GitHub corpus is valuable infrastructure, but it
asks the lawyer to perform integration work before receiving the benefit.

## What the comparison establishes

The live mLey homepage provides law/regulation discovery, article search,
record-update labels, and links to Markdown and llms.txt. It explicitly says its
record-update dates are not legal-reform dates. The supplied Article 22 screenshot
shows an index, readable provision, adjacent PDF, and a relationship view. These
are good interaction choices. They do not establish the completeness, correctness,
or freshness of the site's entire underlying corpus; no backend audit was done.

Sources consulted:
- https://mley.mx/
- https://mley.mx/CCF/articulo/22/

SQLite or a larger graph does not automatically improve on that product. The
advantage would be reliable answers to “which text did this answer use?”, “what
changed?”, “which relationships are express?”, and “what remains unverified?”.
A link from Article 22 to its containing code is useful navigation, but does not
by itself demonstrate substantive cross-law reasoning.

## Publish useful coverage before exhaustive enrichment

Maintain independent coverage states:

| Stage | Usable output | Acceptance |
|---|---|---|
| Acquired | Official PDF snapshot, source metadata, full extracted text | Format and digest checks; explicit extraction status |
| Structured | Article/section pages and navigable hierarchy | Count, source-boundary, span, and heading checks |
| Linked | Express citations and backlinks | Exact source spans and existing targets; disclose unresolved references |
| Analyzed | Temporal/effect proposals and reviews | Model schema, evidence freshness, audit preservation, qualified human decisions |

Failure to parse one document should leave an accessible source entry and a
quarantined structured candidate, not block acquisition of unrelated instruments.
Do not present an acquired-only document as structurally validated. Do not require
complete temporal analysis to offer source-grounded reading and search.

The code already separates much of this data. The unfinished resumable-ingestion
plan is the main operational gap: one long agent session is doing the coordinator's
job. Prefer persistent work records and bounded retries over another general
multi-agent framework.

## Public reader on LEG4L L4BS

Start with a useful vertical slice (for example LRITF, its two DCGs, LIC, and
related references already represented), plus a catalogue showing wider coverage.
A small published reader with honest limitations is more valuable than waiting for
the last federal instrument. Hosting under a subdomain such as
`lex.leg4ll4bs.com` is a proposal, not a DNS or deployment decision made here.
The supplied domain could not be inspected through the browser tool in this pass.

Minimum reader:
1. Search and stable shareable provision URLs, including neighboring text.
2. Side-by-side official PDF, opened to the relevant page when a validated page
   mapping exists. Never invent page numbers from article numbers.
3. Source status in ordinary language: last successful upstream check, source
   retrieved date, source-stated last reform, and pending detected changes.
4. “See source” and “What changed” actions. Put complete hashes and manifests in
   expandable provenance details, with a one-click digest verification workflow.
5. Explicit citations/backlinks as a list first; a bounded graph is an optional
   view with edge types and unresolved/historical distinctions.
6. Visible coverage and legal-review status. A current download is not proof
   that every provision is currently effective.

A hash identifies a particular byte sequence; it does not establish freshness
or authenticate the publisher by itself.
The PDF displayed must be the snapshot whose hash supports the text. Retain source
PDFs in an artifact store keyed by SHA-256 for published releases; today's pipeline
can delete .work PDFs, and a live Diputados URL can later point to different bytes.
Offer the current upstream PDF separately when it differs. Recheck sources on a
schedule, record failed checks, and retain old snapshots for reproducible answers.

## One corpus, several interfaces

Canonical source files + review records -> validated release manifest -> derived
SQLite/FTS index -> public reader / read-only API / MCP / Obsidian.

Keep JSON authority for now. SQLite is a rebuildable serving index, not another
place to hand-edit legal facts. Ordinary tables for instruments, provisions,
source snapshots, references, and review summaries are enough for the first
bounded graph; a graph database is not a prerequisite.

First MCP tools should be narrow: search, get provision plus context, get source
snapshot, list references/backlinks, and compare two retained snapshots. Return
stable IDs, exact excerpts, source URLs/digests, retrieval/check times, corpus
release ID, and uncertainty. Pin a query session to a release so several tool
calls cannot silently combine versions. Historical effective-date queries must
return unsupported/unknown until provision-version evidence exists. Keep ingestion
and review mutation tools out of the public read interface.

An “agentic legal brain” becomes useful when it can assemble an evidence packet,
identify missing related instruments, explain a change with before/after citations,
and distinguish a source fact from a proposed legal effect. Let the reasoning model
work over this interface rather than memorize an entire vault.

## Unattended workers and evals

Current official docs describe persistent cloud work and scheduled tasks for dots
and Grok Bot. Product availability, quotas, and approved integrations still govern
actual runtime; “24/7” should describe a recoverable service, not an unlimited turn.
No provider jobs or spending were enabled in this pass.

Use a persisted queue with source-hash idempotency, job leases, retry budgets,
per-instrument branches/artifacts, and one integration writer. States:
queued -> fetched -> extracted -> provisionally parsed -> inspected -> validated
-> proposed for publication, with explicit blocked/retry states. Never let two
workers mutate the same canonical checkout. A parser-code change becomes its own
reviewed fix and regression run instead of being auto-merged by an ingestion job.

Roles worth testing:
- Worker: acquire, run the deterministic pipeline, and produce a candidate diff.
- Independent checker: inspect omitted/duplicated headings, signatures, exact
  source coverage, citation targets, and known pathological pages without seeing
  the first model's verdict.
- Coordinator: enforce deterministic checks and queue transitions; route failures
  and legal judgments to the appropriate review. Model agreement is evidence, not
  proof and never reviewer identity.

Measure against a frozen, human-labelled challenge set: ordinal article headings,
Bis/letter/digraph suffixes, nested decrees, page furniture, historical same-title
references, and changed/removed review evidence. Score exact article recovery,
text preservation, citation precision AND recall, false clean passes, cost per
accepted instrument, time to detect a changed source, and recovery from a killed
worker. Compare a single model to cross-provider checking on the same cases before
paying for every job to run twice.

Jev is plausible for narrow routing/relevance judgments: “does this paragraph look
like signature apparatus?”, “does the quoted evidence support this proposed edge?”,
or “does this source need review?”. Deterministic exact-span/hash checks should run
first. Calibrate thresholds on Lex-Mex examples; probability and model agreement
cannot certify legal currency. Existing AgentOS evaluation patterns may be reused
as a design reference, but Lex-Mex should own its contracts and environment.

Official capability references:
- https://learn.chatgpt.com/docs/dots/controls
- https://learn.chatgpt.com/docs/automations
- https://docs.x.ai/grok-bot/overview
- https://docs.typesafe.ai/cookbooks/citation_check

## Immediate engineering disposition

| Severity | Confirmed failure | Repair |
|---|---|---|
| P1 | `run_temporal_import` accepted canonical fields forbidden by the model contract, including external verification | Validate raw JSON before deserialization; reject reviewer fields in the typed routing boundary too |
| P1 | Archived pending reviews could overwrite the live determination for the same provision | Reject archived IDs and mismatched evidence hashes before mutation |
| P1 | `preserve_temporal_review_history` dropped prior items when evidence disappeared | Archive both pending and resolved records, retaining their audit contents |
| P1 | Article dates inherited original law commencement even for later-added wording | Leave dates unset; narrowly migrate old machine-generated dates |
| P1 | A request or review could still match its stored result after the current corpus changed | Compare import/request and resolution evidence to current canonical text |

Code locations: `crates/lex-cli/src/main.rs` (import, current-evidence checks,
review resolution); `crates/lex-core/src/temporal.rs` (schema gate, routing,
history, resolution); `crates/lex-parse/src/temporal_derive.rs` (dates and repair).
The first four reproduce the supplied audit; the current-corpus gate extends it.

The four temporal review findings were confirmed in code. Repairs in this pass
validate raw model JSON against the embedded v2 schema, prevent archived or stale
reviews from mutating current evidence, retain omitted-evidence history, and stop
assigning original-law commencement dates to later article text. A related stale
request/current-corpus check was added on import and review resolution.

The date repair does not prove every existing machine `effective` status.
The existing classifier still uses original instrument commencement; a later
reform can have deferred or conditional commencement even when its wording is
already in a consolidated PDF. Publication must expose that machine status as
qualified, and authoritative historical/current-validity claims need a separate
reform-aware temporal review. Structural validation cannot certify legal validity.

The older cleanup report is historical and largely completed. It should not drive
new branch deletions or vault rewrites. Its LFT heading-drift concern remains a
separate reparse comparison: the retained LFT work source is absent locally, so
this pass does not claim to have resolved that issue. Existing parser failure
classes and the lack of upstream freshness monitoring remain tracked follow-ups.

Recommended delivery order: land temporal repairs and evidence; publish a bounded
source-backed reader; add the shared SQLite/MCP serving projection; then automate
known adapter families with a measured queue and eval suite. Broader corpus coverage
continues concurrently as acquisition/structure work, not as a launch prerequisite.

## Validation and delivery record

Completed locally against baseline `8031ea11`: formatting, strict locked Clippy,
198 workspace tests, locked CLI build, required LRITF and IFPE-DCG validation,
and all 132 affected instrument validators passed. Invalid-input CLI checks used
isolated corpus copies; no provider calls or human review approvals were made.
The valid-response control produced eight pending review items in an isolated copy.

Rust removed 15,974 unsupported date values. An independent post-validation
comparison to Git HEAD confirmed no other provision JSON values changed and no
other tracked corpus files changed except the one LPUE Markdown synchronization.
The per-instrument receipt is `docs/temporal-date-repair-2026-10-06.json`.
The user authorized the local repair commit on 2026-10-06; its identity is recorded
in Git and the session receipt. Nothing was pushed, deployed, or scheduled. The existing
Obsidian vault was not regenerated by the repair sweep.
