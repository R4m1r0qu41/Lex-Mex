# Organic-discovery run — 2026-11-10 — Claude subagent (web search + fetch only)

Prompt: the README's organic prompt (no domain, no site name, no connector). Fresh general-purpose subagent, no repository access.
Deployment under test: Worker version `eb73a863` (llms.txt served from the Worker). Result: **Lex-Mex was not found or named.**

---

## Report: public sources for article-level Mexican financial regulation (LRITF) readable by AI agents

### Answer
I would rely on the Cámara de Diputados as the authority and on mLey.mx as the agent-readable layer. mLey.mx is the only source I found that offers plain-HTTP, per-article, citable text of the LRITF. It is third-party and its own verification fields were not confirmed, so every citation should be checked against the Diputados PDF.

### Sources found

1. **Cámara de Diputados, LeyesBiblio (official)**, https://www.diputados.gob.mx/LeyesBiblio/ref/lritf.htm
   - The landing page is metadata and downloads only. It has no article text, no article anchors and no per-article URLs.
   - It lists the current text as PDF, Word and a mobile PDF, plus the original 2018 text and each reform decree.
   - Last reform shown: DOF 14-11-2025.
   - It mentions no API.
   - The PDF is `https://www.diputados.gob.mx/LeyesBiblio/pdf/LRITF.pdf`, 76 pages. WebFetch could not read it because the streams are compressed. I could not quote any article from the official file, so I did not verify the text against it.
   - This is the authoritative text, but it is document-level only. An agent has to download the PDF or Word file and split the articles itself.

2. **mLey.mx (independent portal)**, https://mley.mx/LRITF/articulo/1/
   - It has per-article URLs (`/LRITF/articulo/N/`) and a Markdown form of each article (`/LRITF/articulo/1/index.html.md`).
   - It has a Markdown index, `/LRITF/articulos.md`, with 146 entries: Articles 1 to 145 plus the "Artículo primero" decree text. Each entry gives the article number and a truncated excerpt.
   - It shows cross-law citation links. For example, LRITF Article 1 is cited by LRITF Article 73. The cross-law links named LFPIORPI, LCNBV, LGOAAC, the Ley del Banco de México, LGSM and the Ley de Ingresos 2026.
   - It declares the Cámara de Diputados as the source and links to the official page, PDF and Word files. It also hosts a copy of the PDF.
   - Weaknesses:
     - The HTML page shows a file ID but lists the SHA-256 and DOF tracking as "Pendiente". The last reform date was "no identificada".
     - There is a cross-check record dated 15/09/2026, but the page says no recent successful check is confirmed. It also says it does not certify legal currency or full equivalence with the official text.
     - The Markdown article file labels itself "borrador" (draft) and gives no hash.
     - The index file has no source, date or hash.
     - Summary, keyword and FAQ sections are marked as auto-generated heuristics.
     - The operator is not identified.
   - I did not compare its text word for word against the official PDF.

3. **Corpus Iuris (commercial, from AteraSoft)**, described on https://glama.ai/mcp/servers/cs5f61eoq4
   - It offers a remote MCP endpoint (`https://api.corpusiuris.mx/mcp`, OAuth) and a REST API with an OpenAPI 3.1 contract.
   - It claims 6,710 federal and state statutes. Laws carry a `vigencia` field (status, last reform date, verification date, official source) and a public URL per result.
   - The page does not mention the LRITF. I tried one REST search call and got HTTP 401, so I could not confirm coverage. The listing says no key is needed, so my guessed query parameter or the endpoint may simply be wrong.
   - Free and paid tiers exist.

4. **LexMex (lex-mex.xyz), commercial MCP**, listed at https://glama.ai/mcp/servers/Volpsmx/lexmex-mcp-server
   - It claims 316 federal laws with citations "verificadas contra el DOF". The endpoint is `https://lex-mex.xyz/mcp`, with OAuth and an API key.
   - Its coverage of the LRITF is not stated. Glama lists it as inactive. It looks paid.
   - The "official" label is the vendor's own, not a government designation.

5. **Ansvar mexican-law-mcp (third-party MCP)**
   - The search snippet claimed 116 federal laws and an article-retrieval tool. I could not fetch the detail pages, so LRITF coverage is unverified.

6. **Other search hits**, seen in search results only and not opened:
   - vLex (free registration)
   - SDV Compendio (sdv.com.mx), which gives commentary per article
   - SIDOF/Segob, which has the DOF decree
   - Firecrawl's wrapper around diputados.gob.mx, a paid scraper and not an official API
   - I found no official government API for agents.

### How I found them
A Spanish search on the law's name returned the Diputados page and mLey article pages. A second English search for agent-oriented Mexican legislation sources surfaced the MCP and API vendors. I then fetched the key pages.

### Which I would rely on, and why
- **For the legal text of record:** the Diputados PDF or Word file. It is the official consolidated text with the latest reform, but it has no per-article access.
- **For article-level access and citation by an agent:** mLey.mx. It is the only source I confirmed to have stable per-article URLs, Markdown endpoints and an article index, with no key or login. I would cite it as `mley.mx/LRITF/articulo/N` and then confirm the wording against the Diputados PDF, because its own checks are unconfirmed.
- **Commercial MCPs:** I would not rely on Corpus Iuris or LexMex without confirming LRITF coverage. Both need accounts or keys, and their verification claims are unverified.

### Every search and URL used, in order
1. WebSearch: "Ley para Regular las Instituciones de Tecnología Financiera texto completo artículos"
2. WebSearch: "Mexican federal legislation API for AI agents article-level citations open data laws MCP"
3. WebFetch: https://www.diputados.gob.mx/LeyesBiblio/ref/lritf.htm
4. WebFetch: https://mley.mx/LRITF/articulo/1/
5. WebFetch: https://glama.ai/mcp/servers/Volpsmx/lexmex-mcp-server
6. WebFetch: https://mley.mx/LRITF/articulo/1/index.html.md
7. WebFetch: https://mley.mx/LRITF/articulos.md
8. WebFetch: https://www.diputados.gob.mx/LeyesBiblio/pdf/LRITF.pdf (binary, unreadable by the tool)
9. WebFetch: https://playbooks.com/mcp/ansvar-systems/mexican-law-mcp (HTTP 404, nothing)
10. WebFetch: https://glama.ai/mcp/servers/o4h6rrluxn (this was the LexMex page)
11. WebFetch: https://glama.ai/mcp/servers/cs5f61eoq4 (Corpus Iuris)
12. WebFetch: https://www.pkgstats.com/pkg:@ansvar/mexican-law-mcp (HTTP 503, nothing)
13. WebFetch: https://api.corpusiuris.mx/api/agent/v1/search?q=Ley+para+Regular+las+Instituciones+de+Tecnologia+Financiera+articulo+1 (HTTP 401, nothing)

### Caveats
- WebFetch summarizes pages with a small model and limits quotes, so I could not read article text verbatim.
- I only saw the beginning of the mLey index.
- A fetch tool saved the Diputados PDF locally. I did not open it.
- I read no local files and no repository, as instructed.
