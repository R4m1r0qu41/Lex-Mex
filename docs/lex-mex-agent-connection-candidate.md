# Lex-Mex agent access — candidate connection guide

Status: local candidate verified; production promotion and external-client acceptance
remain pending. These proposed URLs are not a claim that MCP is already deployed.

## Use your own agent

Your AI application plans searches, reads the retrieved legal evidence and writes
the answer. Lex-Mex supplies public read-only data. No model API key or AI account
credential belongs in Lex-Mex. A model subscription does not automatically imply
that its application supports custom MCP connections; test the specific client.

### Ordinary browsing

Start at `https://www.leg4ll4bs.com/lex-mex/` or its `llms.txt` discovery file.
The candidate exposes `/lex-mex/agents/` for search and reading, and
`/lex-mex/current.json` for the pinned release/catalogue/coverage URLs.
Search first, retrieve complete relevant provisions and follow references.
Cite stable IDs, the release and official sources. Disclose partial coverage and
freshness limits. An empty search is not proof that no obligation exists.

### Connected MCP client

After promotion, add remote Streamable HTTP server:
`https://www.leg4ll4bs.com/lex-mex/mcp`.
Use the client's supported custom-server/connector workflow. The candidate serves
public read-only evidence without login; it must not request model credentials.
Initialization instructions identify the release to pass to each tool.

| Tool | Arguments beyond release_id | Result |
| --- | --- | --- |
| search | query; optional instrument, cursor | Ranked excerpts and IDs; read full provisions next |
| get_provision | instrument, type, number | Complete text, source, digest and recorded review/status |
| get_references | instrument, type, number; optional cursor | Registered references and resolution limitations |
| get_coverage | None | Partial scope and freshness limitations |

Types are article, transitory and annex. Instruments in the pilot are lritf, lic,
itf-dcg-2018 and ifpe-dcg-2021. Keep one release throughout an answer. Finish
pagination where needed; do not download the entire collection to answer a
single question. Request bodies are capped at 8 KiB. Errors, unavailable releases
and insufficient evidence must remain visible rather than filled from memory.

MCP gives a configured agent structured access; it does not install itself,
guarantee correct reasoning or cause a search engine to rank Lex-Mex. Public
crawlable pages and organic-discovery tests remain separate requirements.

## Acceptance after promotion

Run fresh directed conversations in each supported provider with Q01–Q10 one
question per message, retaining exact requests/responses. Record tool-level text
availability separately from correctness. Run organic discovery in fresh sessions
without the domain, site name, connector or prior Lex-Mex context. Neither local
SDK tests nor the local Terra report substitute for those gates.
