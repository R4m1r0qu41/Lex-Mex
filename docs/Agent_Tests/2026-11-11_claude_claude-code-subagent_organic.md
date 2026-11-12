# Organic-discovery check — 2026-11-11

- Protocol: optional organic-discovery check (README), prompt pasted unedited.
- Client: Claude Code general-purpose subagent (Claude Opus 5.5), cold start, no session context, told to use only web tools and not to read local files.
- Live site at run time: Worker version `a550ea9a` (website commit `50594ba`), deployed the same day; stable `/lex-mex/mx/` pages and per-instrument sitemaps had been live for a few hours (not yet crawled by search engines, as far as known).
- Result: **Lex-Mex did not appear.** The agent found mLey.mx (recommended, with caveats), the Diputados PDF, the SCJN Word copy, vLex, sdv.com.mx, cyberpolicyportal, Ansvar's `@ansvar/mexican-law-mcp`, an MDPI dataset and HAQQ.

## Agent reply (unedited)

**Short answer:** The best source I found for an AI agent is **mLey.mx**. It is an independent public portal that serves Mexican federal laws one article at a time, in plain Markdown made for agents. That includes the LRITF (Ley para Regular las Instituciones de Tecnología Financiera, the fintech law). For any citation that matters, I would check the text against the **official Cámara de Diputados PDF**. mLey links to that PDF itself and says its own text is a copy that has not been verified.

I used only web search and curl/WebFetch against public URLs. I read no local files.

**How I found it**
- Search 1, in Spanish, for the LRITF text and AI agents: mLey came up with many results, one per article, such as `mley.mx/LRITF/articulo/76/`. Other hits were an SCJN Word copy of the law, a copy on database.cyberpolicyportal.org, vLex (paywalled) and sdv.com.mx (a compendium).
- Search 2, in English, for machine-readable Mexican law for AI agents: this found Ansvar's `@ansvar/mexican-law-mcp`, an open-source MCP server for AI agents. It also found an MDPI 2022 research dataset that turns the laws into JSON (static, not a live service) and HAQQ (a commercial product that does not appear to cover the LRITF). This search turned up no official government API.
- I then looked at mLey's machine-facing files and the official source.

**What I checked on mLey**
- `https://mley.mx/llms.txt` exists. It gives instructions for agents: cite the law and article, prefer the Markdown versions, keep legal text, summary and interpretation apart, check the reform date, and respect ETag, robots.txt and rate limits.
- `https://mley.mx/LRITF/llms.txt` is the index for this law.
  - It says to cite as "Ley Para Regular Las Instituciones De Tecnología Financiera, artículo {n}".
  - It reports the last DOF reform as 14-11-2025 and the last mLey update as 2026-10-12.
  - It links an article index (`/LRITF/articulos.md`) and one Markdown file per article.
- `https://mley.mx/LRITF/articulo/76/index.html.md` returns the full literal text of Article 76, which is the open-finance/API article.
  - It has YAML front matter: law_key, article_key, `source: diputados`, canonical_path.
  - It shows where the article sits: Title III, Chapter IV.
  - It carries a `status="draft"` marker.
- The HTML page for the article links the official Diputados PDF and the DOF. It also discloses the following:
  - "Correspondencia de fuentes no comprobada": the match with the official source has not been confirmed.
  - The SHA-256 field reads "Pendiente" (pending).
  - The verification-date fields contradict each other.
  - The site says its check identifies a copy, not legal validity.
- `https://mley.mx/.well-known/mcp.json` returned 404, so I found no MCP endpoint. A developer page exists, but what I fetched was mostly asset markup.
- One quality problem: the LRITF index tags the law with subjects like "salud, vivienda… derechos humanos". That is clearly a wrong automatic classification for a fintech law, so the summaries and metadata should not be trusted the way the article text can be.

**Official source**
- `https://www.diputados.gob.mx/LeyesBiblio/pdf/LRITF.pdf` returned 200. Its header says "Última Reforma DOF 14-11-2025", which matches mLey's reform date.
- `https://www.diputados.gob.mx/LeyesBiblio/ref/lritf.htm` is the reform history page.
- This is the authoritative consolidated text, but it is a PDF with no article-level addresses.

**Which I would rely on, and why**
- **mLey** is the only source I found that does all of this at once: public, no login, a stable URL per article, plain Markdown with metadata, an explicit citation format, agent instructions, and a link back to the official source for every article.
- Its own disclaimers (unverified match, pending hash, draft status) mean it is not the authority. For any citation that matters, compare the article against the Diputados PDF, check the DOF for reforms after 14-11-2025, and do not treat the output as legal advice.
- I did not test the Ansvar MCP server. It needs installing and is a community project whose own docs say to verify citations. SCJN and other copies are whole documents with no article-level addressing.

**Every search and URL, in order**
1. WebSearch: "Ley para Regular las Instituciones de Tecnología Financiera texto artículos API para agentes IA"
2. WebSearch: "Mexican federal laws machine-readable API article-level citations AI agents"
3. WebFetch: https://mley.mx/LRITF/articulo/76/
4. curl (status only): https://mley.mx/robots.txt
5. curl (status only): https://mley.mx/llms.txt
6. curl (status only): https://mley.mx/.well-known/mcp.json
7. curl (status only): https://mley.mx/
8. curl (content): https://mley.mx/llms.txt
9. curl: https://mley.mx/LRITF/llms.txt
10. curl: https://mley.mx/LRITF/articulo/76/index.html.md (the first attempt's output was cut off by a shell error, so I fetched it again)
11. curl (status): https://www.diputados.gob.mx/LeyesBiblio/pdf/LRITF.pdf
12. curl: https://www.diputados.gob.mx/LeyesBiblio/ref/lritf.htm (the first parse failed on encoding)
13. curl: https://mley.mx/LRITF/articulo/76/index.html.md (fetched again)
14. curl: https://www.diputados.gob.mx/LeyesBiblio/pdf/LRITF.pdf (status again)
15. curl: https://www.diputados.gob.mx/LeyesBiblio/ref/lritf.htm (parsed again, which got the reform-index headings)
16. curl: https://mley.mx/desarrollador/
17. curl: https://www.diputados.gob.mx/LeyesBiblio/pdf/LRITF.pdf (downloaded to read the reform date)

The URLs below came up in search results but I did not open them:
- https://www.internet2.scjn.gob.mx/red/delitos/Word/54.%20LRITF.docx
- https://vlex.com.mx/vid/ley-regular-instituciones-tecnologia-727991149
- https://sdv.com.mx/compendio/ley-para-regular-las-instituciones-de-tecnologia-financiera/
- https://database.cyberpolicyportal.org/api/files/1658413498329a22xugett05.pdf
- https://www.pkgstats.com/pkg:@ansvar/mexican-law-mcp
- https://mdpi.com/2306-5729/7/7/91
- https://www.haqq.ai/solutions/by-country/mexico
