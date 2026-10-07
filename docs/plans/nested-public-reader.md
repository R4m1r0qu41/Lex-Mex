# Nested public reader and agent corpus access

## Purpose and observable outcome

A lawyer opens a Lex-Mex article inside a supported desktop browser or ordinary
web browser, sees its exact supporting PDF alongside it, and follows related
provisions without losing the original view. Codex or Claude Code retrieves
Markdown/plain text in the background from the same pinned release. A public
index declares complete or partial coverage and makes available text discoverable.

This is an implementation plan requested on 2026-10-06 and updated on 2026-10-07
for execution on **2026-10-08 (America/Mexico_City)**. Planning is complete;
reader, hosting, MCP, WebMCP, and SQLite implementation have not begun.
It does not authorize a DNS change or deployment. Local development and a
reviewable release candidate precede the eventual publication action.

## Authority, baseline, and existing work

Initial baseline: `8031ea11eb1d4ad88c046ba8b6ba1ed8851eb873`, with the audited
temporal repair subsequently committed and reorganized by later work. Keep that
baseline as historical evidence, not tomorrow's build input. Current verified
implementation baseline: `9e990e09995718a77865993b24c5f8c8e7167b42` on `main`,
matching the local `origin/main` tracking ref on 2026-10-07; worktree clean before
this plan update. Repository URL: `https://github.com/R4m1r0qu41/Lex-Mex`.
Refresh HEAD, status, and the remote tracking state before execution; never reset
the checkout to the earlier repair commit to reproduce old migration results.
Repository instructions, canonical corpus, schemas, Rust code, and decisions
are authoritative. External websites are interaction references, not legal data.

Read `PLANS.md`, the temporal repair audit, the four-point report, and the
existing external CLI/metadata/ingestion plan. Reuse existing read-only
`instruments`, `path`, `search`, and bundle capabilities rather than inventing
a second corpus pipeline. Rust remains responsible for canonical publication;
the web UI and SQLite are derived consumers.

Existing Markdown exports contain source hashes and inject presentation links.
They are not an immutable public release or a byte-preserved PDF store. The
current source manifest has retrieval metadata but not a successful-upstream-
check ledger. No validated article-to-PDF page mapping was established here.
These are implementation prerequisites, not fields to guess.

## Repository and website integration boundaries

The user-created reader workspace is
`/Users/jr/Documents/4ll4/Webpage/Lex-Mex`. It was empty on inspection on
2026-10-07. It is inside the Git root `/Users/jr/Documents/4ll4`, not an independent
Lex-Mex clone. Do not clone the legal corpus over it or create a nested Git
repository without an explicit architectural reason.

| Surface | Responsibility | Write scope during implementation |
|---|---|---|
| `/Users/jr/Dev/lex-mex` | Canonical Rust pipeline, legal corpus, release builder, schemas and publication manifest | Explicit reviewed producer changes; preserve legal decisions |
| `/Users/jr/Documents/4ll4/Webpage/Lex-Mex` | Reader frontend and development fixtures/release consumption | New reader files; no canonical corpus edits |
| `/Users/jr/Documents/4ll4/Webpage/LEG4L_L4BS` | Existing firm site, navigation and hosted `/lex-mex/` integration | Named integration paths only after inspection |
| `/Users/jr/Documents/4ll4` | Parent Git boundary for website and other work | Stage exact paths; preserve unrelated Socials/configuration files |

The existing site uses React/Next-style routes through Vinext. Reuse its actual
build conventions after inspecting configuration; avoid selecting a second web
framework before checking how a reader package can be consumed by the host.
Prefer reader components developed in `Webpage/Lex-Mex` and a small host route
integration. Test cross-directory module imports and asset paths in the real
production build; if the host cannot bundle sibling imports, use an explicit
deterministic staging/package step rather than two manually maintained copies.
Do not add a package/workspace just for a placeholder: it must have running code.

The site's 2026-10-07 `HANDOFF.md` reports publication through Sites version18,
with custom domain `https://www.leg4ll4bs.com`. This is handoff evidence, not a
live availability check performed in this planning pass. The historical source
layout and release checkout differ: build in `LEG4L_L4BS/`, with a root hosting
manifest and staged root build output for release packaging. Read the current
handoff and applicable Sites skills before opening or publishing that project.
Do not configure an alternative hosting provider or run a Wrangler deployment
merely because Vinext uses Wrangler internally. Do not change mail/contact work.

Tomorrow's target route is `https://www.leg4ll4bs.com/lex-mex/`, subject to a live
canonical-domain check and host route verification. Verify apex/www redirects
and base-path handling; the local directory name does not create a public route.
The existing parent `public/llms.txt` can link the reader's agent index without
overwriting the firm's current description or unrelated links. An article URL,
navigation entry, sitemap entry, and working static asset paths are required
integration evidence, not simply an iframe link to GitHub.

Read the applicable ancestor/site instructions on 2026-10-08. Open the website
root in the execution workspace or obtain the necessary filesystem write scope
before making site edits. This planning session edits only the corpus plan.

## Changes carried forward from 2026-10-07

Source: Agent Vault daily receipt
`AI/60_Evaluations/session-summaries/lex-mex/2026-10-07.md`, checked against current
Git history and targeted repository decisions. The handoff reports 220 workspace
tests and211 passing statute validators; these are prior-session results, not
checks rerun for this documentation update.

- The mixed repair commit was split, and `temporal-derive-v3` restored 10,208
  original dates across130 instruments for articles without amendment evidence.
  Amended wording remains unset. Use the current values and disclose that absence
  of an amendment marker is not a completed decree-by-decree historical audit.
- `partially_repealed`, `conditional_pending`, `repeals` with legislative/judicial
  cause, and `commencement_condition` now exist. Models cannot assign the new
  statuses. The release exporter/reader must preserve these values and show
  conditions as unverified where applicable; never collapse them to effective.
- Parser/source-boundary work, historical targets, legacy-review rechecks, and
  LACP/LRAF/LCEC audited baselines have advanced. Reuse current fixtures and
  recorded decisions rather than reparsing all instruments for the reader.
- Open legal/data tasks remain: unverified-condition review queue and model-output
  schema version,65 SCJN-flagged provisions across23 instruments needing review,
  ruling notice versus DOF effective-date judgment, unread repeal cases in
  LSCS207 and FI-DCG-2014 64, and LFPC reparse drift (190 versus188 articles).

These are named blockers for legal-validity claims and affected instrument
promotion, not a reason to stop unrelated structural reader development.
Do not label any collection legally complete or currently valid based on the
reader's build success. Keep unresolved instruments/records visibly qualified;
exclude affected candidates from a validated release if their required gates fail.
Do not silently resolve those tasks as part of frontend integration. The receipt's
next legal-pipeline action remains separate from tomorrow's reader workstream.

## First release scope

Start with LRITF, IFPE-DCG-2021, ITF-DCG-2018, and LIC only if each satisfies
release gates. Expose a wider coverage catalogue with explicitly unavailable
article pages where necessary. The structural reader does not claim complete
historical validity or automatically accept temporal legal conclusions.

Proposed public nesting: `https://leg4ll4bs.com/lex-mex/`. Route and asset helpers
must honor a configurable base path. Root `/llms.txt` can point to
`/lex-mex/llms.txt` when the parent site owner integrates it. A subdomain deployment
remains an alternative if the existing host cannot support the needed routes;
inspect the actual site repository before choosing infrastructure.

## Architecture

```mermaid
flowchart LR
  C[Canonical JSON and review records] --> R[Rust validated release builder]
  P[Retained official source bytes] --> R
  R --> M[Immutable release manifest]
  M --> T[Markdown and plain text shards]
  M --> D[Derived SQLite FTS and reference edges]
  M --> S[PDF snapshots and page mappings]
  T --> H[Public HTTP discovery]
  D --> A[Read-only API and MCP]
  S --> V[Nested browser reader]
  A --> V
  A --> L[Codex or Claude Code]
  H --> L
```

Build once from a clean committed corpus selection. Generate a content-addressed
release manifest excluding its own digest field when computing its identifier.
Record Git commit, builder/schema versions, included instruments, coverage,
canonical/text/artifact hashes, source snapshot references, and review summary.
Pin all assets by release ID. Publish the mutable latest pointer only after the
complete candidate passes, retaining the previous pointer for rollback.

SQLite indexes instruments, provisions, source snapshots, typed edges, and
bounded review summaries. It must be rebuildable from the release, read-only
in serving, and contain no independent editorial authority. Delay SQLite if a
static bounded catalogue is sufficient for the first text-release milestone.

## Human interaction: nesting without losing context

Desktop layout: collapsible hierarchy; primary article; secondary source PDF.
A related provision opens in a comparison panel or replaces a selected panel,
with breadcrumbs and back navigation. Limit simultaneous comparison panels to
two; further navigation replaces the secondary panel. Compact layout uses
Article / Source / Related tabs with the same selection state.

Store `release_id`, instrument ID, provision ID, source digest, selected relation,
and validated PDF page in the shareable URL/state. Preserve user scroll and zoom
when an agent retrieves other text. An explicit "Show this evidence" action
changes the visible selection; a silent `get_provision` does not.

Use a same-origin HTML PDF viewer over retained PDF bytes. A bundled PDF.js
viewer is the preferred candidate for consistent controls; confirm its current
API/version during implementation. Native PDF embedding is a fallback, not an
assumed cross-host capability. Serve PDF with the correct media type and byte
range support; direct download remains available if inline rendering fails.
Never rely on a third-party live PDF iframe as the sole evidence view.

Acquire missing archived PDFs only from the official URL and require the bytes
to match the committed hash. If the live source differs, it is a new candidate
snapshot: keep the old release unsupported for inline source display until its
exact bytes are recovered, and label the new PDF separately. Do not change the
old manifest to make a fetched replacement fit.

PDF page mapping must be snapshot-specific. Extend the extracted-page pipeline
to map canonical spans to page indexes; validate boundaries and ambiguous matches.
Show a page jump only when mapping is verified. Otherwise open the instrument
PDF with search and say that automatic page location is unavailable.

## Public discovery and text contract

Proposed routes below are contracts to implement, not deployed endpoints.
Examples use an arbitrary release token `RELEASE_ID`; clients first resolve it
from the release catalogue and then use immutable URLs throughout a query.

| Route under `/lex-mex/` | Purpose |
|---|---|
| `llms.txt` | Small Markdown orientation, coverage and release links, usage guidance |
| `coverage.json` and `coverage.md` | Actual included instruments and independent stage states |
| `releases/latest.json` | Pointer to a fully validated immutable release |
| `releases/RELEASE_ID/manifest.json` | Digests, provenance and exact selection |
| `releases/RELEASE_ID/index.md` | Sharded instrument index |
| `releases/RELEASE_ID/mx/lritf/index.md` | Provision index and source links |
| `releases/RELEASE_ID/mx/lritf/articulo-1.md` | Article text with declared presentation links and metadata |
| `releases/RELEASE_ID/mx/lritf/articulo-1.txt` | Exact canonical provision body, UTF-8; metadata supplied separately |
| `releases/RELEASE_ID/sources/SHA256.pdf` | Exact retained source bytes |
| `reader?release=RELEASE_ID&instrument=lritf&provision=STABLE_ID` | Human nested view |
| `mcp` | Planned read-only Streamable HTTP service |

Keep `.txt` canonical body content distinct from linked Markdown presentation.
Metadata identifies language, stable IDs, source digest, retrieval/check dates,
release, coverage, review status, and missing evidence. Reuse existing filenames
where valid; don't assume numeric IDs cover Bis/suffixed articles or transitories.
Do not publish private reviewer notes or working artifacts; define a deliberately
public review summary before exposure.

HTML pages advertise their Markdown with `rel="alternate" type="text/markdown"`
and the applicable index with `rel="describedby"`. HTTP Link headers can provide
the same discovery for text responses. Publish a sitemap for human pages and
configure crawl policy deliberately. No access account or JavaScript execution
is needed to read public Markdown. Provide archive downloads and digest manifests
for offline clients, with bounded per-instrument shards instead of one forced
whole-corpus context dump.

`llms.txt` is a proposal and cannot guarantee automatic tool adoption. Offer
copyable instructions: fetch this index, select a release, search/navigate only
the relevant shards, cite stable IDs and source links. The site's root agent
index and GitHub README should link the public index once it exists. A future
optional agent skill may reinforce this workflow, but ordinary HTTP must work
without installation.

## MCP, browser tools, and desktop integration

MCP tools: `search_corpus`, `get_provision`, `get_context`, `get_source`,
`list_references`, and `compare_snapshots`. Inputs include release ID, stable
instrument/provision IDs, and bounded pagination. Outputs provide text, provenance,
coverage/uncertainty, source and viewer URLs. Invalid/unknown IDs return explicit
errors. `as_of` legal-validity queries remain unsupported until historical evidence
is implemented. Search relevance cannot create legal graph edges.

The server does not control a desktop browser simply because it returns a URL.
The host/agent opens the viewer using its browser capability. For optional live
synchronization, the top-level reader can expose read-only WebMCP retrieval plus
`get_reader_selection` and an explicit `show_evidence` UI action, using the same
release contract and state. Background retrieval never silently changes the view.
No canonical ingestion or review-resolution tools belong to this public surface.

OpenAI's current documentation says the built-in browser is in the ChatGPT
desktop app, not Codex CLI/IDE. It can display public/local pages. WebMCP tools
must be registered on the top-level page: the documented browser does not discover
iframe tools. Therefore register Lex-Mex tools on the reader shell even if PDF
rendering uses a nested frame. Availability depends on host/model/policy; feature
detection and ordinary HTTP/MCP fallbacks are required.

Claude Code documents remote HTTP MCP and a desktop Browser preview. Use a local
reader server for acceptance testing in that pane. Treat arbitrary public-page
preview, PDF rendering, and any shared-selection behavior as tests to execute,
not capabilities established by the documentation alone. CLI users can use a
separate ordinary browser with the same URLs. Codex must not edit Claude-owned
configuration or invoke its CLI; provide setup instructions for the operator.

The universal contract is text + viewer/source URLs + pinned release. It works
even when a host cannot embed an MCP UI. An optional MCP Apps widget can be
evaluated later for compatible hosts; it is not a prerequisite for public access.

## Milestones and acceptance gates

1. **Release/source foundation.** Implement a Rust release builder over an
   explicit corpus selection; retain/recover matching PDFs; declare missing
   snapshots. Acceptance: deterministic rebuild, all hashes check, human
   decisions preserved, no working/private files leaked, failed builds leave the
   previous release intact. Update schemas/types/fixtures together for new fields.
2. **Discoverable text release.** Generate indexes, coverage, immutable `.md` and
   `.txt`, manifests, and archives. Acceptance: no-JavaScript HTTP client follows
   index to an article, exact `.txt` body matches canonical text, Markdown links
   resolve within the published release, partial coverage is explicit.
3. **Nested reader candidate.** Build hierarchy/article/PDF/related panes and
   snapshot-specific page mappings. Acceptance: PDF digest matches manifest;
   unmapped articles have honest fallback; share/back navigation retains context;
   compact and keyboard views work; two-panel limit prevents window proliferation.
4. **SQLite and read-only MCP.** Build serving projection and bounded tools.
   Acceptance: HTTP, MCP, and reader agree on IDs/text/release; invalid IDs and
   unsupported historical queries are explicit; release swaps cannot mix content
   across a pinned session; corpus and reviews remain untouched by all reads.
5. **Desktop qualification.** Test the actual ChatGPT desktop built-in browser
   and operator-run Claude Desktop preview. Acceptance: human PDF remains visible
   while agent fetches multiple text articles; explicit show-evidence changes the
   pane; same digest/release throughout; unavailable WebMCP/inline PDF paths fall
   back cleanly. Record versions and results separately, not as a generic pass.
6. **Publication candidate.** Inspect LEG4L L4BS hosting integration; prepare
   base-path routes, atomic release promotion, cache rules, source-check ledger,
   and rollback. Local/preview evidence must be reviewable before deployment.
   Publication approval and authenticated hosting/DNS access are final inputs.

Run the repository's required checks for Rust/canonical changes. Add meaningful
release/route integration tests for the new boundary, rather than tests that only
mirror UI implementation. Use frozen exact-source fixtures for page mapping and
include repeated article labels, page furniture, and spanning articles.

## Freshness, operation, and stop conditions

Check official sources on a configured cadence. Record last attempt, last
successful check, HTTP metadata, and observed digest in a derived operational
ledger. New digest -> candidate ingestion -> validation/inspection -> new release.
Never automatically replace published text merely because the upstream file
changed. An unreachable source keeps its last verified snapshot and shows the
failed check. Retain DOF sources for formal amendment/commencement evidence.

Pause integration on unexplained source/text differences, lost human decisions,
unvalidated page locations, or missing exact PDF bytes presented as verified.
Workers can continue unrelated instruments. Do not autoapprove legal ambiguity.
Rollback changes the latest pointer to a retained known release, preserving history.

## Current checkpoint and next action

2026-10-07: read today's Lex-Mex receipt and current decisions, verified clean
corpus HEAD/tracking baseline `9e990e099`, inspected the empty user-created reader
directory and parent Git boundary, and read the firm site's instructions, package
scripts and dated hosting handoff. Updated this plan for execution tomorrow.
No reader files, corpus changes, hosting settings or scheduled jobs were created.

The previous 2026-10-06 planning checkpoint remains historical. Its repaired-date
behavior and original mixed commit are superseded by the documented October7
decisions; do not use the old report as the current implementation specification.

## Execution runbook — Thursday, 2026-10-08

Goal for the first build session: a local integrated `/lex-mex/` preview with an
explicit selected release, discoverable text, one working article/source flow,
and evidence that firm routes still work. Finish the foundation before pursuing
MCP, full corpus ingestion, or an elaborate graph. This is a scheduled work plan,
not an automation that has been installed to run without the operator.

1. **Resume and lock scope.** Read this plan, today's receipt, applicable
   instructions and site handoff. Discover the corpus capsule. Check both Git
   roots and preserve unrelated files; record actual HEADs and source versions.
   Confirm the reader workspace and target host route. Refresh hosting/docs only
   as needed for the selected integration; credentials stay outside reports.
2. **Define selected-release contract.** Use the four named instruments with
   independent gates. Inventory source manifests, matching retained PDFs,
   current fields and any selection-specific blockers. Add release schema,
   Rust builder and meaningful fixtures; support qualified missing PDFs rather
   than silently substituting the live upstream file. Do not reparse the entire
   corpus or run temporal models as a frontend build dependency.
3. **Produce the first consumable bundle.** Build immutable manifest, coverage,
   per-instrument indexes and `.md`/exact `.txt` shards. Verify repeatable output,
   source hashes and text equality. Keep bulky release/PDF artifacts in a derived
   ignored staging destination or selected artifact store, not in the website's
   source history. Give the reader an explicit release path/configuration rather
   than hardcoding the developer's `/Users/jr` paths.
4. **Build and nest the reader.** Implement used frontend components in the new
   directory; wire the host's `/lex-mex/` route and generated assets through its
   actual build. Deliver Spanish-first catalogue/article/PDF panes, typed related
   links, copyable share URLs, and an honest unmapped/missing-PDF fallback.
   Integrate navigation and the parent agent index in named files only. Keep the
   existing English offer, Spanish root and `/es` redirect intact.
5. **Validate the integrated preview.** Run required Rust gates for producer
   changes and `npm test` in `LEG4L_L4BS` for site changes, using mechanical
   delegation under applicable instructions. Check `/`, `/en`, `/es`, the nested
   reader, source/text endpoints and asset paths in the production build. Inspect
   a real PDF pane while text is fetched independently from the same release.
   Qualify desktop hosts separately when available; record unavailable tests.
6. **Close with a reviewable candidate.** Inspect exact diffs in both repositories;
   record release/source digests, passed checks, partial coverage and blockers.
   Update this plan and continuity before stopping. Prepare a preview/release
   package and rollback instructions. Commits and deployment follow the user's
   current authorization and site instructions; this request authorizes planning,
   not a future publication or unattended job.

First-day completion gate: working local host route and no regression to firm
pages; at least one selected article has manifest-matching inline PDF (or this
goal is explicitly blocked by missing source bytes); no-JavaScript clients can
find the available text; model text and visible source share a release/digest.
MCP/SQLite/WebMCP remain subsequent milestones if foundation work uses the session.
Do not present a mock interface alone as completed corpus integration.

Next action on 2026-10-08: resume the two repository boundaries, inventory the
four-instrument exact sources, and implement milestone1 before host integration.
Start or bind the reader execution capsule to this plan and its digest; preserve
the separate legal-pipeline next action from the October7 receipt.

## Sources

- [llms.txt discovery proposal](https://llmstxt.org/): Markdown indexes and alternate links; not automatic adoption.
- [OpenAI built-in browser](https://learn.chatgpt.com/docs/browser): desktop shared page view and CLI/IDE distinction.
- [OpenAI Site tools](https://learn.chatgpt.com/docs/webmcp): top-level WebMCP, iframe limitation and availability conditions.
- [OpenAI Docs MCP](https://developers.openai.com/learn/docs-mcp): external read-only documentation MCP pattern.
- [Claude Desktop](https://code.claude.com/docs/en/desktop): local dev-server Browser preview.
- [Claude MCP](https://code.claude.com/docs/en/mcp): remote HTTP tool connections.
