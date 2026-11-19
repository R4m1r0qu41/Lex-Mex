# 2026-11-19 — Claude Code subagent, organic (search only, no URL given)

- Protocol: the 2026-11-18 `organic` prompt, unchanged. The agent was given no website address and told to answer with web search only (WebSearch and WebFetch; no local files).
- "Independent" here means a fresh Sonnet subagent with no repository, vault or conversation context. It is the same model family as the orchestrator, not a cross-family review.
- Search backend caveat: the WebSearch tool is a single backend that is neither Google nor Bing. This run cannot measure the Bing work done the same morning (100 URLs submitted through Bing Webmaster Tools about an hour before the run). Treat it as a baseline; the Bing check is Site Explorer's indexed count and a `site:leg4ll4bs.com/lex-mex` search in Bing after a few days.
- Bing state at the time (operator's Bing Webmaster account, read by the orchestrator): homepage indexed; Site Explorer showed no indexed Lex-Mex URLs; articles "Discovered but not crawled" since 2026-11-14 (so the sitemap index was read); live tests of `/lex-mex/mx/`, `/lex-mex/mx/lritf/` and LRITF art. 7 said "URL can be indexed by Bing" (the last two with "Title too long" and "Meta Description too long or too short"); one live test of LRITF art. 19 said "fetched but not able to process" (not reproduced on art. 7).
- Result compared with 2026-11-18: unchanged. No leg4ll4bs.com result in four queries; answers came from mley.mx (LRITF arts. 11 and 46, LFT art. 47) and the official DOF page (DCG-IFPE-2021 art. 2). This run quoted the DCG-IFPE art. 2 first paragraph verbatim from the DOF.
- Production at the time: version `fef9af53-063d-4815-bf4d-38b9c11420c8` (website commit `ca8c0bc`), release `c305db4a…`.
- The text below is the agent's reply, unedited.

---

# Web-only research report (WebSearch and WebFetch only; no local files, repo or memory)

Search rankings below are the order of the result list the tool returned. I never saw a rendered Google or Bing page, so treat "rank" as position in that list.

## S1. Does leg4ll4bs.com / Lex-Mex appear?

**No, for every query tried.**

| Query | Top results (domain, title) | Lex-Mex / leg4ll4bs.com? |
|---|---|---|
| `Ley para Regular las Instituciones de Tecnología Financiera artículo` | 1 vlex.com.mx "Ley para Regular las Instituciones de Tecnología Financiera"; 2 mexico.justia.com (PDF); 3 cnbv.gob.mx (PDF); 4 ordenjuridico.gob.mx "LEY PARA REGULAR LAS INSTITUCIONES DE TECNOLOGÍA FINANCIERA"; 5 scjn.gob.mx (docx); 6 vlex.com.mx (second copy); 7 leyes-mx.com "…Artículo 1 México"; 8 uacj.mx; 9 mley.mx "Ley [LRITF]"; 10 unam.mx; 11 mley.mx; then more leyes-mx.com and mley.mx article pages (15, 16, 24, 26, 27) | Not present in the 16 results |
| `LEG4L L4BS Lex-Mex` | l4sb.com ("New Mexico Assisting Lawyer", "Meet the Law 4 Small Business Team"), worldtrademarkreview.com (Basham, Ringe y Correa), vlex.com.mx "LexBlog Mexico", lawcat.berkeley.edu, guides.ll.georgetown.edu "Mexican Legal Research Guide", chambers.com, legal500.com, benchmarklitigation.com | Not present. The search tool reported no match for "LEG4L" or "Lex-Mex". Nearest false match was L4SB (a New Mexico firm). |
| `leg4ll4bs.com lex-mex` (extra) | vlex.com.mx "LexBlog Mexico", legal500.com (Lechuga Abogados), usmcocca.org "mex legal", plus unrelated restaurants (toasttab, spoton, latinolife, lataco, micocina) | Not present |
| `leyes federales México artículo por artículo legible por máquina JSON API Markdown corpus jurídico` (extra, to look for a machine-readable source) | ordenjuridico.gob.mx; mley.mx/leyes.md; mley.mx/llms.txt; scribd (JSON-database paper); diputados.gob.mx LeyesBiblio; sdv.com.mx/compendio ("834 Leyes · 48,028 Artículos"); mley.mx/reglamentos.md; gist.github.com | Not present |

Wording differences: the brand-name queries ("LEG4L L4BS Lex-Mex", "leg4ll4bs.com lex-mex") returned only unrelated results. The topical query and the generic "machine-readable laws" query returned other sites (mley.mx, sdv.com.mx, leyes-mx.com, vlex, Justia). I did not run Google or Bing directly, and WebSearch is a single search backend.

## S2. Who authorizes an ITF, and does the government guarantee customer money?

- **Queries:** "Ley para Regular las Instituciones de Tecnología Financiera artículo" (results above), then a direct fetch of the article page.
- **Page relied on:** https://mley.mx/LRITF/articulo/11/ (Art. 11 LRITF). This is an unofficial portal that labels itself independent and shows verification as "Pendiente" (text match not confirmed). I fetched the official PDF at diputados.gob.mx (`/LeyesBiblio/pdf/LRITF.pdf`) but it came back as compressed binary and could not be read. The CNBV PDF failed with a certificate error.
- **Answer:**
  - The CNBV authorizes ITFs, after agreement of the Comité Interinstitucional. Retrieved text: "Para organizarse y operar como ITF se requiere obtener una autorización que será otorgada por la CNBV".
  - No, the government does not guarantee customer money. Art. 11 says "Ni el Gobierno Federal ni las entidades de la administración pública paraestatal podrán responsabilizarse … o garantizar los recursos de los Clientes". ITFs must state this expressly on their websites, in app messages and electronic communications, and in advertising and client contracts.
- **Supported by retrieved text?** Yes, by the mley.mx page. Not confirmed against the official text. The article number comes from the page URL and heading.

## S3. Segregation of customer money

- **Queries:** "LRITF instituciones de fondos de pago electrónico recursos de clientes separados patrimonio propio artículo 54 mley". Top results: mley.mx/LRITF/articulo/46, sdv.com.mx compendio art. 46, mexico.justia.com, then unrelated Spanish and BOE e-money documents.
- **Page relied on:** https://mley.mx/LRITF/articulo/46/ (page title "Artículo 46 de LRITF | mLey.mx"; the page cites the official PDF at page 22; file version `d6f645e6a7d3`).
- **Answer:** Art. 46 LRITF requires ITFs to "mantener los recursos propios segregados de los de sus Clientes" and to keep records of client funds client by client.
  - Money not yet delivered must be deposited by the end of the day it was received, in designated accounts or through repo operations or a trust.
  - For IFPEs, the amount held in demand-deposit accounts may not exceed the greater of 1,000,000 UDIS and twice the highest amount redeemed in 24 consecutive hours.
  - For crowdfunding institutions, client funds cannot be treated as the institution's liability and cannot be withdrawn until release conditions are met.
- **Supported by retrieved text?** Yes, by mley.mx page text, with paraphrase of the details. I did not verify against the official PDF. The sdv.com.mx search snippet independently describes the same segregation rule.

## S4. DCG for IFPE (2021), Art. 2, customer consent

- **Queries:** "Disposiciones de carácter general aplicables a las instituciones de fondos de pago electrónico 2021 artículo 2 consentimiento del cliente". Top results: whitecase.com (IFPE secondary regulation summary), vlex.com.mx (DCG for ITF), sidof.segob.gob.mx, dof.gob.mx nota 5610487, arcusfi.com, treezor.com, and Spanish BOE documents.
- **Page relied on:** official DOF page https://dof.gob.mx/nota_detalle_popup.php?codigo=5610487. Its title is "DISPOSICIONES aplicables a las instituciones de fondos de pago electrónico a que se refieren los artículos 48, segundo párrafo; 54, primer párrafo, y 56, primer y segundo párrafos de la Ley para Regular las Instituciones de Tecnología Financiera". It was issued by Banco de México and the CNBV. The fetch did not show a publication date. White & Case separately says the rules were published in the DOF on 28 January 2021.
- **Answer (quoted from the DOF page):** Art. 2, first paragraph: "Las instituciones de fondos de pago electrónico, al pactar la celebración de Operaciones y la prestación de servicios a través de Canales de Instrucción, deberán requerir el consentimiento expreso de sus Clientes para dichos efectos, el cual se podrá obtener a través del proceso de Autenticación referido en el artículo 7 de las presentes Disposiciones."
  - In short: express customer consent is required, and it may be obtained through the Art. 7 authentication process.
  - Only the first paragraph was retrieved. The page is about 194,000 characters and I read only the first 100,000, so I did not check for further paragraphs in Art. 2.
- **Supported by retrieved text?** Yes, official source, quoted. One caveat: the publication date and amendment status were not seen on the page.

## S5. Ley Federal del Trabajo, rescission without employer liability

- **Query:** "Ley Federal del Trabajo artículo 47 causas de rescisión de la relación de trabajo sin responsabilidad para el patrón". Top results: mexico.justia.com LFT, unam.mx theses, foxsports.com.mx, idconline.mx, vlex.com.mx, leyes-mx.com/ley_federal_del_trabajo/47.htm, mley.mx/LFT/articulo/47/, sdv.com.mx art. 47.
- **Page relied on:** https://mley.mx/LFT/articulo/47/ (unofficial; shows "Comprobación pendiente", last check 15/09/2026, text match not verified, source stated as Cámara de Diputados).
- **Answer:** Art. 46 sets the general rule of rescission for just cause. Art. 47 lists the causes of rescission without liability for the employer, as a list of XVI fractions (paraphrased from the page):
  - I: deception with false certificates or references (this ground lapses after 30 days of service).
  - II: dishonesty, violence, insults or mistreatment toward the employer, its staff, clients or suppliers (not if provoked or in self-defense).
  - III: the same conduct against a coworker, if it disrupts workplace discipline.
  - IV: serious off-duty acts against the employer, family or management.
  - V: intentional material damage.
  - VI: serious damage through negligence.
  - VII: endangering the establishment or people through imprudence.
  - VIII: immoral acts or harassment, including sexual harassment.
  - IX: disclosing secrets.
  - X: more than three unjustified absences in 30 days.
  - XI: unjustified disobedience.
  - XII: refusing preventive measures.
  - XIII: attending work intoxicated or under narcotics without prescription.
  - XIV: a final prison sentence that prevents work.
  - XV: lacking legally required documents.
  - XVI: analogous grave causes.
  - The employer must give written notice of the date and causes. Failure to give it presumes the separation was unjustified unless the employer proves otherwise.
- **Supported by retrieved text?** Yes, by mley.mx page text, with short quotes. I did not verify against the official Cámara de Diputados text. A search snippet from leyes-mx.com suggests the text was updated in July 2026, so check the current version.

## Machine-readable source note

The source closest to "article-level and machine-readable" that I found is mley.mx. It offers per-article pages, Markdown copies (e.g. `/LRITF/articulo/46/index.html.md`), `/LRITF/articulos.md`, `llms.txt`, and links to the official Cámara de Diputados PDF. It states its own verification is "Pendiente", so it is unofficial. sdv.com.mx/compendio and leyes-mx.com also give article-by-article pages. For the official text I used dof.gob.mx (S4). The Diputados PDF was binary and the CNBV PDF failed.

# Discoverability summary

- A search-only agent did not land on leg4ll4bs.com or Lex-Mex in any of the four queries I ran:
  - the topical LRITF query;
  - "LEG4L L4BS Lex-Mex";
  - "leg4ll4bs.com lex-mex";
  - a generic query for machine-readable Mexican laws.
- The brand and domain queries returned unrelated results: L4SB (New Mexico), law-firm directories, vLex blog aggregators and restaurants.
- Topical queries (LRITF, LFT art. 47, IFPE rules) returned mley.mx, sdv.com.mx, leyes-mx.com, vlex, Justia, dof.gob.mx and diputados.gob.mx, not Lex-Mex.
- Because the site never appeared, there is no landing title, description, URL form, `?release=` parameter or release hash to report.
- Only one search backend was available. These results do not show how Google or Bing would rank the site.
