# 2026-11-18 — Claude Code subagent, organic (search only, no URL given)

- Protocol: not `batch-v1`. Same family as the 2026-11-10 and 2026-11-11 `organic` runs: the agent was given no website address and told to find answers with web search only (WebSearch and WebFetch; no local files).
- "Independent" here means a fresh Sonnet 5.5 subagent with no repository, vault or conversation context. It is the same model family as the orchestrator, not a cross-family review.
- Search backend caveat: the WebSearch tool is a US-only backend that is neither Google nor Bing. This run cannot confirm or refute indexing in either engine. Confirming that needs `site:leg4ll4bs.com/lex-mex` in Google and Bing, or Search Console, run by the operator.
- Orchestrator checks: two further `site:`/brand queries by the orchestrator through the same tool also returned no leg4ll4bs.com page. The search-only agent's answers on LRITF arts. 11 and 46 came from the third-party mley.mx and agree with the committed corpus in substance; its LFT art. 47 answer comes from a third-party source and was not graded (Lex-Mex does not hold the LFT).
- Production at the time: version `aa207c17-74a1-49d7-a2c5-ac61a6f85e50` (website commit `1ddfde5`), release `c305db4a…`.
- The text below is the agent's reply, unedited.

---

# Search-only research report

The WebSearch tool returns a list of links plus a model-written summary, so "rank" below means position in the returned link list. WebFetch returns a small-model summary of the page, so quotes are capped at about 125 characters. I did not read any local file or use memory for legal content.

**One deviation from "search only":** after search results showed the domain leg4ll4bs.com (via LinkedIn snippets), I fetched `https://leg4ll4bs.com` once, to see how the site presents Lex-Mex. I did not guess any deeper URL.

## S1. Do leg4ll4bs.com / Lex-Mex show up in search?

**Query A: `Ley para Regular las Instituciones de Tecnología Financiera artículo`**
- Top results, in order:
  1. vlex.com.mx, "Ley para Regular las Instituciones de Tecnología Financiera"
  2. mexico.justia.com, same title, PDF
  3. cnbv.gob.mx, PDF
  4. ordenjuridico.gob.mx, "LEY PARA REGULAR LAS INSTITUCIONES DE TECNOLOGÍA FINANCIERA"
  5. internet2.scjn.gob.mx, 54. LRITF.docx
  6. vlex.com.mx, second copy
  7. leyes-mx.com, "Artículo 1"
  8. uacj.mx, an e-journal article
  9. mley.mx, "Ley [LRITF]", article 1
  10. journals.unam.mx
  - The list continues with mley.mx and leyes-mx.com per-article pages and a cyberpolicyportal PDF.
- **leg4ll4bs.com / Lex-Mex: not present** (not in the 16 links).
- Per-article sites that did appear: mley.mx, leyes-mx.com, sdv.com.mx.

**Query B: `LEG4L L4BS Lex-Mex`**
- Top results:
  1. l4sb.com, "New Mexico Assisting Lawyer"
  2. l4sb.com, "Meet the Law 4 Small Business Team"
  3. worldtrademarkreview.com, Basham, Ringe y Correa
  4. vlex.com.mx, "LexBlog Mexico"
  5. lawcat.berkeley.edu
  6. guides.ll.georgetown.edu, Mexican Legal Research Guide
  7. chambers.com
  8. legal500.com
  9. benchmarklitigation.com
- **Not present.** The summary read "LEG4L" and "L4BS" as the unrelated L4SB (Law 4 Small Business).

**Extra queries I ran, to probe discoverability:**
- `leg4ll4bs.com lex-mex`: not present. Results were vlex LexBlog pages, legal500 Lechuga Abogados, and restaurant pages.
- `leg4ll4bs`: not present. Results were Scratch, itch.io and esolangs pages.
- `Lex-Mex Mexican federal law articles ... machine-readable article citations`: not present. Results were official and aggregator sources.
- `"Lex-Mex" provenance-aware Mexican federal legal texts corpus Markdown Obsidian`: not present. The only relevant results were an MDPI paper (a different corpus) and vlex.
- `"LEG4L L4BS"` (exact phrase, extended mode):
  - Results were 10 LinkedIn profiles whose snippets quote the firm's rebrand posts.
  - The summary said the firm was formerly Hernáiz Abogados, uses the domain leg4ll4bs.com, and is pronounced "legal labs".
  - This was the only query where leg4ll4bs.com surfaced at all, and only as text inside LinkedIn snippets, not as a result URL. No Lex-Mex page appeared.
- `leg4ll4bs.com Mexican law articles ITF Ley Fintech ...` with `allowed_domains=["leg4ll4bs.com"]`: no links at all. This suggests the domain is not indexed by this search tool, or at least not for that query.
- `Lex-Mex Mexican federal law corpus lex-mex leg4ll4bs.com/lex-mex` (extended mode):
  - Result 1 was GitHub R4m1r0qu41/Lex-Mex, titled "Provenance-aware compiler and temporal-analysis pipeline for Mexican federal legal texts."
  - Competing projects also appeared: ingteranalvarez/lex-mx, Julio-Patron/lexapi-mx, glama.ai corpus-iuris, and lexifina/mexico-law.
  - leg4ll4bs.com/lex-mex itself did not appear.

**Direct fetch of `https://leg4ll4bs.com`:**
- The page showed the header "LΞG4L L4BS | Ingeniería Legal". The `<title>` and meta description were not exposed by the fetcher.
- The nav link is Lex-Mex at `/lex-mex/mx/`. The footer link is "Lex-Mex agents" at `/lex-mex/agents/`.
- Neither link has `?release=` or a hash.

## S2. Who authorizes an ITF, and does the government guarantee customer money?

- **Queries:** `LRITF artículo 11 autorización Comisión Nacional Bancaria y de Valores gobierno federal responsabilidad recursos clientes`; `Ley para Regular las Instituciones de Tecnología Financiera artículo`.
- **Top results:**
  1. mley.mx, "Ley [LRITF]", article 11
  2. sdv.com.mx, "articulo 11"
  3. mley.mx, search index
  4. alertas.directoriolegislativo.org, LRITF_090318.pdf
  5. diputados.gob.mx, LRAF ref01 PDF
  6. internet2.scjn.gob.mx, LRITF.docx
- **Page relied on:** https://mley.mx/LRITF/articulo/11/, titled "Artículo 11 de LRITF | mLey.mx". mLey is an independent portal that cites the Cámara de Diputados as its source.
- **Answer, Article 11 of the Ley para Regular las Instituciones de Tecnología Financiera (LRITF):**
  - Organizing and operating as an ITF requires an authorization granted by the CNBV, with the prior agreement of the Comité Interinstitucional. Quote: "Para organizarse y operar como ITF se requiere obtener una autorización que será otorgada por la CNBV".
  - The government does not guarantee customer money. Quote: "Ni el Gobierno Federal ni las entidades de la administración pública paraestatal podrán responsabilizarse". Per the fetch summary, they also cannot "garantizar" Clients' resources used in Operations, and they assume no responsibility for ITF obligations or one Client's obligations to another.
  - ITFs must state this on their websites, messages, advertising and contracts.
- **Supported by retrieved text:** Yes, but only through a summarized fetch of a third-party portal.
  - The page itself says its text match with the official source is "no comprobada" (unverified).
  - The wording after the opening clause is a paraphrase, not a full verbatim quote.
  - I could not read the official Cámara de Diputados PDF. WebFetch returned unreadable FlateDecode binary for `diputados.gob.mx/LeyesBiblio/pdf/LRITF.pdf`.
  - The Cámara page https://www.diputados.gob.mx/LeyesBiblio/ref/lritf.htm was retrieved. It shows the last reform as DOF 14-11-2025 (reforms of 20-05-2021 and 24-01-2024 are also listed).

## S3. Segregation of customer money from the ITF's own resources

- **Queries:** `LRITF artículo 54 ITF recursos de los Clientes no podrán formar parte de su patrimonio, separados, cuenta de depósito segregación`; the mley.mx article index.
- **Top results:**
  1. mley.mx article 46
  2. sdv.com.mx article 46
  3. sdv.com.mx article 47
  4. mley.mx article 45
  5. internet2.scjn.gob.mx, LRITF.docx
  6. sdv.com.mx article 45
  7. mley.mx article 47
  8. mley.mx article 54
  - Several irrelevant Spanish CNMV, Iberclear and hacienda.gob.es PDFs appeared further down.
- **Page relied on:** https://mley.mx/LRITF/articulo/46/, titled "Artículo 46 de LRITF | mLey.mx".
- **Answer, Article 46 LRITF:**
  - ITFs must "mantener los recursos propios segregados de los de sus Clientes" and keep Clients' funds identified per Client.
  - Client money not yet delivered to the beneficiary or transferred must be deposited "a más tardar al final del día en que las hayan recibido". It goes in demand-deposit accounts in the ITF's own name at an authorized financial entity, separate from the ITF's own-resources accounts.
  - Alternatives are one-day reporto operations on Government or Banxico securities, or an administration trust that only does those operations.
  - Electronic payment fund institutions have a cap on demand-account balances: the greater of 1,000,000 UDIS and twice the largest 24-hour redemption in the last 365 days.
  - For crowdfunding institutions, Client funds are not a direct or contingent liability of the institution.
  - Article 47 (by SDV's summary) gives owners of undelivered funds a "derecho de separación" over the ITF's accounts and assets. I did not fetch that article, so treat it as unverified.
  - Article 54 is not about segregation. It covers contracting external services.
- **Supported by retrieved text:** Yes, through the mley.mx fetch summary. The page carries the same "no comprobada" caveat, and the quotes are limited to short fragments. I did not obtain a verbatim official text. The official PDF was unreadable, and I did not try the ordenjuridico.gob.mx or cnbv.gob.mx copies.

## S4. DCG for IFPE (2021), article 2, customer consent

- **Queries:** `Disposiciones de carácter general aplicables a las instituciones de fondos de pago electrónico 2021 artículo 2 consentimiento del cliente`.
- **Top results:**
  1. whitecase.com, "secondary fintech regulations related to e-money institutions (IFPEs)", Spanish PDF, 2021-02
  2. vlex.com.mx, "Disposiciones ... Instituciones de Tecnología Financiera"
  3. sidof.segob.gob.mx, docFuente/5610487
  4. hacienda.gob.es (irrelevant)
  5. dof.gob.mx, nota_detalle_popup.php?codigo=5610487
  6. deloitte.com, 2018 ITF disposiciones
  7. banxico.org.mx, CUFPE form
- **Page relied on:** https://dof.gob.mx/nota_detalle_popup.php?codigo=5610487, cross-checked with https://sidof.segob.gob.mx/notas/docFuente/5610487.
  - Title: "DISPOSICIONES aplicables a las instituciones de fondos de pago electrónico a que se refieren los artículos 48, segundo párrafo; 54, primer párrafo, y 56, primer y segundo párrafos de la Ley para Regular las Instituciones de Tecnología Financiera".
  - Issued jointly by Banco de México and CNBV.
- **Caveat:** the fetcher did not show a publication date. I could not confirm that this is the "2021" instrument you meant. White & Case's January 2021 alert supports a 2021 timeframe but is not proof. Only the first 100,000 of about 195,000 characters were read.
- **Answer, Article 2, as summarized by the fetcher:**
  - IFPEs must obtain the client's express consent before agreeing to Operations and services through Canales de Instrucción. Quote: "deberán requerir el consentimiento expreso de sus Clientes para dichos efectos".
  - That consent "se podrá obtener a través del proceso de Autenticación referido en el artículo 7".
  - The contract must set out the operations and services available through each channel, the authentication mechanisms, each party's responsibilities, how clients are notified, the cancellation procedure and operational restrictions.
  - Clients must receive the terms of use before signing up, and the institution must inform them of risks and suggest ways to prevent unauthorized operations.
  - Related consent provisions: Article 3, fracción II, requires "previo consentimiento expreso" for changes that could affect the client financially. Article 4's last paragraph covers disabling notifications only on express request, through Article 7 authentication.
- **Supported by retrieved text:** Partly. The quotes are verbatim fragments from the official DOF page, but the full article text was not retrievable verbatim.

## S5. Ley Federal del Trabajo, rescission without liability for the employer

- **Queries:** `Ley Federal del Trabajo artículo 47 causas de rescisión de la relación de trabajo sin responsabilidad para el patrón`.
- **Top results:**
  1. mexico.justia.com, LFT Título II Capítulo IV
  2. ru.dgb.unam.mx, theses
  3. foxsports.com.mx
  4. vlex.com.mx
  5. idconline.mx
  6. leyes-mx.com, LFT 47
  7. mley.mx, LFT 47
  8. sdv.com.mx, articles 46 and 47
- **Page relied on:** https://mley.mx/LFT/articulo/47/, titled "Artículo 47 de la Ley Federal del Trabajo | mLey.mx". Its source is shown as Cámara de Diputados, with the same "comprobación pendiente / correspondencia no comprobada" flags. File version 12f09393a195.
- **Answer, Article 47 LFT (employer may rescind without liability):**
  - The page lists 16 fracciones; the analogous-causes clause is fracción XVI, not XV.
  - I. Deception with false certificates or references, limited to the first 30 days of service.
  - II. Faltas de probidad u honradez, violence, threats, insults or mistreatment toward the employer, family, staff, clients or suppliers, unless provoked or in self-defence.
  - III. The same acts against coworkers, if they disrupt discipline.
  - IV. Serious off-duty acts against the employer, family or management that make the relationship impossible to continue.
  - V. Intentional damage to tools, materials or premises.
  - VI. Serious damage through negligence, where negligence is the sole cause.
  - VII. Inexcusable carelessness endangering the establishment or people.
  - VIII. Immoral acts or harassment, including sexual harassment.
  - IX. Disclosing manufacturing or confidential secrets.
  - X. More than three unexcused absences in 30 days.
  - XI. Unjustified disobedience.
  - XII. Refusing preventive or safety measures.
  - XIII. Working intoxicated or under narcotics, unless there is a prescription presented in advance.
  - XIV. A final prison sentence that prevents performing the job.
  - XV. Missing legally required documents, with the worker responsible and the delay past the period in art. 43 fr. IV.
  - XVI. Analogous causes of equal gravity.
  - **Last paragraph:** the employer must give written notice clearly stating the conduct and the dates. It is delivered personally at dismissal, or filed with the tribunal within 5 business days, with the worker's last registered address. Without notice, the separation is presumed unjustified. The limitation period runs only from the worker's personal receipt.
- **Supported by retrieved text:** Yes, in substance (a paraphrase with short quotes). It comes from an unverified third-party portal. Cross-confirmation came from search-snippet summaries of 1a./J. 47/2022 and Justia, which show the fracciones and notice rule are consistent.

# Discoverability summary

- **Did a search-only agent land on leg4ll4bs.com/lex-mex?** No. Across 9 queries, leg4ll4bs.com/lex-mex never appeared as a result URL.
- **Queries that touched the domain at all:**
  - The exact phrase `"LEG4L L4BS"` in extended mode returned only LinkedIn profile posts that mention leg4ll4bs.com. The text there was "Hernáiz Abogados is becoming LEG4L L4BS", "pronounced 'legal labs'", and "fixed scope, fixed fee".
  - No Lex-Mex page or corpus was mentioned, and the page title and description of leg4ll4bs.com itself were not shown in any result.
  - The same name unquoted (`LEG4L L4BS Lex-Mex`) was misread as "L4SB / Law 4 Small Business".
  - `leg4ll4bs.com ... allowed_domains=["leg4ll4bs.com"]` returned no links, so the domain appears unindexed in this tool.
- **Queries that surfaced Lex-Mex-like results, but not the target:**
  - `Lex-Mex Mexican federal law corpus lex-mex leg4ll4bs.com/lex-mex` surfaced the GitHub repo R4m1r0qu41/Lex-Mex, in position 1. Its title is "Provenance-aware compiler and temporal-analysis pipeline for Mexican federal legal texts". Its snippet mentions a temporal corpus of LRITF, ifpe-dcg-2021 and itf-dcg-2018.
  - The same query also returned the competitors lex-mx, lexapi-mx, corpus-iuris and mexico-law.
- **Natural legal queries** (S2-S5 wording, e.g. "artículo 11 LRITF autorización CNBV") returned only mley.mx, leyes-mx.com, sdv.com.mx, vlex, justia, ordenjuridico, cnbv, diputados and dof. The target never appeared.
- **Google/Bing-style variation:** I tried exact-phrase quotes, the bare domain, the domain plus path, and domain-restricted search. Only the quoted brand string led to the domain, and only indirectly.
- **What the landing looked like (direct fetch of the root only):**
  - Header text "LΞG4L L4BS | Ingeniería Legal".
  - Nav link to Lex-Mex at `/lex-mex/mx/`.
  - Footer "Lex-Mex agents" at `/lex-mex/agents/`.
  - I fetched nothing deeper than the root, so I have no title or description for the Lex-Mex pages.
  - There was no `?release=` or hash in the two links shown. I cannot say whether deeper pages use release hashes.
- **Comparison:** the article-level, per-page, machine-friendly sites that did rank (mley.mx, leyes-mx.com, sdv.com.mx) show up for article-specific queries. mley.mx even exposes a file-version string and a SHA-256 field, though both were "pending" or unverified when fetched. A search-only agent today would reach article text through those competitors and through official sources (cámara de diputados, DOF, CNBV), not through leg4ll4bs.com/lex-mex.

## Sources used
- https://mley.mx/LRITF/articulo/11/
- https://mley.mx/LRITF/articulo/46/
- https://mley.mx/LRITF/articulos/
- https://mley.mx/LFT/articulo/47/
- https://www.diputados.gob.mx/LeyesBiblio/ref/lritf.htm (the PDF itself was unreadable)
- https://dof.gob.mx/nota_detalle_popup.php?codigo=5610487
- https://sidof.segob.gob.mx/notas/docFuente/5610487
- https://leg4ll4bs.com (single root fetch)
- Search-snippet summaries from sdv.com.mx, vlex.com.mx, justia, whitecase.com, LinkedIn, GitHub (R4m1r0qu41/Lex-Mex) and others, used only for context and not as legal authority.
