# Public corpus — four-point assessment — 2026-10-06

## Scope and judgment

This report develops the four recommendations from the prior response: publish
a useful reader, make trust understandable, serve humans and agents from one
release, and separate coverage stages. It also addresses the original comparison
points: currency, embedded PDF, nested navigation, related regulations, and graphs.
Implementation detail and acceptance gates belong to
`docs/plans/nested-public-reader.md`.

Keep the parser. A PDF-to-Markdown mirror is a worthwhile acquisition fallback;
the parser earns its extra complexity through stable provision identity, exact
source text, validated references, reproducible changes, and preserved decisions.
Its value must be measured in retrieval correctness and reduced review work.
The time spent ingesting should not prevent delivery of a useful partial reader.

The mLey homepage and Article 22 view establish a published reading workflow.
They do not establish backend completeness or accuracy. Lex-Mex's evidence
model is a potential advantage; it does not make an unpublished repo a better
product for a lawyer who wants to read and verify an article now.
[mLey](https://mley.mx/), [Article 22](https://mley.mx/CCF/articulo/22/).

## 1. Publish a useful reader

Start with the financial-law group already useful to this project: LRITF,
IFPE-DCG-2021, ITF-DCG-2018, and LIC, subject to per-instrument release gates.
Show the wider catalogue with actual coverage instead of implying that only
these instruments exist or that the whole corpus is finished.

Use a collapsible hierarchy, a central article view, and an adjacent official
PDF. Related articles can open a secondary comparison pane while the original
article remains visible. Navigation should preserve the provision, selected
source snapshot, and return path. On smaller windows use tabs instead of a
stack of tiny nested windows. Keyboard navigation and direct share links matter
more for the first release than an elaborate graph.

The proposed public home is `https://leg4ll4bs.com/lex-mex/`; a subdomain remains
an alternative if hosting constraints require it. This is a planned route, not
a claim of existing publication. Keep the viewer usable as a standalone page
so the same route can open inside a supported desktop browser pane.

Success means a lawyer can find an article, compare it with the supporting PDF,
follow its explicit regulation links, and share the same view without installing
Obsidian or understanding the repository.

## 2. Make trust understandable

The friend's trust observation is a UX requirement, not permission to replace
evidence with an assurance. Put the ordinary questions first:

- Which official source supports this text?
- When was that source successfully checked?
- Does a newly detected upstream file await inspection?
- Which parts are machine-derived or legally reviewed?

Keep retrieval time, last successful upstream check, source-stated last reform,
and review status distinct. A recent HTTP check is not necessarily a reform;
an old original enactment is not necessarily stale consolidated text. mLey
itself distinguishes record-update labels from reform dates on its homepage.

The PDF pane must show retained bytes matching the manifest digest. Label the
current upstream PDF separately if its bytes differ. The manifest's existing
retrieval time is usable now; a new successful-check ledger and publication
artifact mapping require implementation. Do not invent those values or infer
them from the latest Git commit.

For hashes, provide a simple verification action and expandable details. The
digest fingerprints bytes; trust in publisher identity requires the official
source and acquisition evidence too. A human should not need to manually locate
a JSON file to understand why the article and PDF belong together.

Current machine temporal classifications must be qualified. The repaired dates
and passing structural validators do not certify legal currency. When formal
publication or commencement matters, present the supporting DOF act separately
from the consolidated Cámara PDF.

## 3. Serve humans and agents from the same release

Publish a versioned corpus release with stable IDs and direct HTML, Markdown,
plain-text, and source links. Agents should fetch text while the human looks at
the PDF for that exact release. Background retrieval should not hijack the human's
pane each time the model explores a related article.

Expose three complementary entry paths:

1. Public HTTP discovery: small `llms.txt`, coverage index, linked per-instrument
   indexes, direct `.md`/`.txt`, HTML alternate links, and a sitemap.
2. Read-only MCP: bounded search, provision/context retrieval, source identity,
   references/backlinks, and snapshot comparisons. Every response pins a release.
3. Desktop browser: the same source-backed reader, optionally with top-level
   WebMCP tools for sharing selection state with the agent.

`llms.txt` is a discovery proposal, not a guarantee that Codex, Claude Code, or
every crawler automatically reads it. Give users a copyable starting URL,
setup instructions, and a direct MCP option. A complete corpus can be exposed
through linked shards without loading all of it into a model's context. A
partial release must declare precisely which instruments and stages are included.
[llms.txt proposal](https://llmstxt.org/).

OpenAI documents external MCP connections and a desktop built-in browser.
Claude Code documents remote HTTP MCP and a desktop Browser preview pane.
These are documented integration paths; the Lex-Mex PDF workflow still needs
acceptance testing on the actual installed hosts.
[OpenAI Docs MCP](https://developers.openai.com/learn/docs-mcp),
[OpenAI Browser](https://learn.chatgpt.com/docs/browser),
[Claude MCP](https://code.claude.com/docs/en/mcp),
[Claude Desktop](https://code.claude.com/docs/en/desktop).

SQLite can be a rebuildable serving projection with FTS search and relational
reference edges. A graph is another view of those typed edges. Keep containment,
express citation, defined-term usage, and unresolved/historical targets distinct.
Do not present inferred semantic similarity as a source citation. Fetch a bounded
one-hop neighborhood first; expansion should answer a user question.

## 4. Separate coverage stages and automate bounded work

| Coverage | Honest user benefit | Gate |
|---|---|---|
| Acquired | Retained official PDF; extracted text if successful | Acquisition metadata, byte digest, declared extraction status |
| Structured | Article pages and hierarchy | Canonical boundary/count/text validation |
| Linked | Explicit references and backlinks | Valid source spans and existing resolved targets |
| Analyzed/reviewed | Qualified proposals and recorded decisions | Schema/evidence gates; authorized human judgment where required |

Coverage is multidimensional: a parsed law can have unresolved external targets,
and reviewed transitories do not imply a complete historical article model.
Publishing an acquisition entry is useful even when structured ingestion fails,
but provisional article candidates must remain outside the validated reader.

For automation, persist a queue with source-hash idempotency, leases, bounded
retries, failure classes, and recovery after interruption. Workers produce
per-instrument candidates; one integrator owns canonical writes. Reuse adapters
for known document families and route unfamiliar boundaries to a focused parser
fix with a regression fixture. This reduces unattended work lost to long sessions.

Dots and Grok Bot are possible worker hosts, subject to actual access and quotas.
Their persistent-work features do not supply this publication protocol by
themselves. Cross-provider checks should first be benchmarked on the same frozen
cases against a single-worker baseline; count shared failures and false clean
passes, not just agreement. Jev is an optional later narrow semantic checker,
after exact deterministic tests, rather than a publication authority.
[Dots controls](https://learn.chatgpt.com/docs/dots/controls),
[Grok Bot](https://docs.x.ai/grok-bot/overview),
[TypeSafe citation check](https://docs.typesafe.ai/cookbooks/citation_check).

## Delivery decision

First deliver source-retaining releases, direct text discovery, and a bounded
reader. Add the shared SQLite/MCP layer and desktop synchronization against that
release contract. Scale unattended ingestion after recovery and false-pass evals
are measured. The system's useful distinction is a traceable evidence workflow
available to both a lawyer and an agent, with honest partial coverage.
