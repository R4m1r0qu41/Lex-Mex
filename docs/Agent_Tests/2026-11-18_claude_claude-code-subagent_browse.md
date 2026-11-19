# 2026-11-18 — Claude Code subagent, browse mode (domain only)

- Protocol: `batch-v1` **variant**. Q01-Q06, Q09, Q10 as in the README; Q11 (out-of-collection labour-law question) and Q12 (LRITF risk disclosure) added; the agent was told to use only WebFetch/WebSearch (curl allowed for plain GET of leg4ll4bs.com) and not to read local files. Not strictly comparable with earlier `batch-v1` runs.
- Release seen: `c305db4a3cfb408b76c13431e26beda72d5ce665046c95783fab36507354c969` (corpus `d2ab64d61`), production version `aa207c17-74a1-49d7-a2c5-ac61a6f85e50` (website commit `1ddfde5`).
- "Independent" here means a fresh Sonnet 5.5 subagent with no repository, vault or conversation context. It is the same model family as the orchestrator, so it is not a cross-family review. A cross-family run follows the README paste protocol and is the operator's.
- Orchestrator grading (targeted `git grep` against the committed corpus; legal interpretation not graded): Q03 quote matches `corpus/mx/ifpe-dcg-2021/markdown/articulo-2.md` first paragraph; Q02 (LRITF art. 11), Q04 (art. 46), Q06 (IFPE arts. 7 and 29: 180 natural days, ten business days) and Q12 (art. 34 four risk points) match the corpus. Q10 and Q11 correctly refused as outside the collection. The Q05 negative conclusion is interpretation and is not graded here.
- Findings are in `docs/project-status.md` (2026-11-18 wrap-up). The text below is the agent's reply, unedited.

---

# Run header
- Provider and client/app: Anthropic Claude (model Sonnet 5.5) running as a sub-agent in Claude Code. Client version unknown.
- Mode (browse) and how you reached Lex-Mex: Browse only. I used WebFetch first, then curl through Bash for plain GETs of leg4ll4bs.com URLs. I switched to curl because WebFetch's summarizer refuses to reproduce article text verbatim (its replies said "125 characters" or "can't reproduce"), so it could not give me exact quotes. I did not use the MCP endpoint. I read no local files.
- Release id you saw: c305db4a3cfb408b76c13431e26beda72d5ce665046c95783fab36507354c969. It comes from `current.json`, with corpus_commit d2ab64d61ecfe103cc2a2980c5d04ed63c20bb7a and fixture:false. The same id appears in each article's front matter and on `/lex-mex/agents/`.
- Date/time of this run: 2026-11-18. I did not record the clock time.

# Results

## Q01
- Answer: Lex-Mex is a partial, "selected_partial" collection of four Mexican federal financial instruments.
  - LRITF: Ley para Regular las Instituciones de Tecnología Financiera.
  - LIC: Ley de Instituciones de Crédito.
  - DCG-ITF-2018: Disposiciones de carácter general aplicables a las instituciones de tecnología financiera.
  - DCG-IFPE-2021: Disposiciones aplicables a las instituciones de fondos de pago electrónico (arts. 48, 54 and 56 of the LRITF). The index lists 71 provisions for it.
  - Each article has its URN, official source link, retrieval time and SHA-256 hashes.
  - Outside the collection: "Todo instrumento que no figure en esta lista está fuera de la colección" (quote from `/coverage`). The site says it is not the current federal corpus and does not check validity ("No es el corpus federal actual ni una comprobación de vigencia"). A search with no results does not prove a norm does not exist. It is not legal advice.
- Path taken:
  1. WebFetch `/lex-mex/` returned a short summary. The page is JavaScript-rendered ("Preparando el corpus…"). It showed no release id and "última comprobación periódica: no registrada". Its only link of interest was `./llms.txt`.
  2. WebFetch `/lex-mex/llms.txt` returned a summary listing the four instruments, the URL patterns for search, article, references and coverage, `current.json`, and an optional MCP endpoint.
  3. WebFetch `/lex-mex/coverage` returned a summary that confirmed the four instruments.
  4. curl `/lex-mex/coverage` returned 200, 1835 B, with the same content in full.
  5. curl `/lex-mex/mx/` returned 200, 2554 B. I only read the first 1500 characters. The 4 instruments are listed, and the first one shows 71 disposiciones.
- Failures and friction: The WebFetch summary of the home page was nearly empty because the page renders in JavaScript. WebFetch returns summaries, not raw text. I did not read counts of provisions for the other three instruments.
- Evidence status: supported by retrieved text.

## Q02
- Answer: Authorization to organize and operate as an ITF is granted by the CNBV, "previo acuerdo del Comité Interinstitucional" (LRITF art. 11, first paragraph, `urn:lex-mx:federal:statute:lritf:article:11`). The government does not guarantee customer money. The third paragraph of art. 11 (quote): "Ni el Gobierno Federal ni las entidades de la administración pública paraestatal podrán responsabilizarse o garantizar los recursos de los Clientes que sean utilizados en las Operaciones que celebren con las ITF…". ITFs must state this expressly on their websites, in app messages, in advertising and in contracts. Source: https://www.diputados.gob.mx/LeyesBiblio/pdf/LRITF.pdf (Cámara de Diputados).
- Path taken:
  1. curl GET `/lex-mex/mx/lritf/articulo/11.md` returned 200, 2793 B, the full article.
  2. curl GET `/search?q=organizarse%20y%20operar&limit=10` returned 200 with 10 results. LRITF 67 and 66 appear first and art. 11 does not appear on page 1. The other hits are LIC articles about banks.
  3. curl GET `/search?q=garantizar&limit=10` returned 10 results. LRITF 11 is the 8th, with a snippet that matches. I did not read page 2.
  4. curl GET `/search?q=autorización de la Comisión&limit=10` returned 10 LIC/DCG results, which were not useful.
  5. curl GET `/search?q=responsabilidad del Gobierno Federal` returned empty ("No se encontraron coincidencias").
- Failures and friction: Literal search is poor at ranking. The art. 11 authorization text did not show up in the first search. The searches in steps 2, 4 and 5 did not lead me to art. 11. I reached art. 11 through the `garantizar` search (step 3) and then fetched it directly. I fetched art. 11 first, using a slug and number I had seen only in the results, so the order above is by topic, not strict time.
- Evidence status: supported by retrieved text.

## Q03
- Answer: Quote of the first paragraph of art. 2 (`urn:lex-mx:federal:regulation:ifpe-dcg-2021:article:2`), as returned by the `.md` page: "Las instituciones de fondos de pago electrónico, al pactar la celebración de Operaciones y la prestación de servicios a través de Canales de Instrucción, deberán requerir el consentimiento expreso de sus Clientes para dichos efectos, el cual se podrá obtener a través del proceso de Autenticación referido en el artículo 7 de las presentes Disposiciones. Adicionalmente, las instituciones de fondos de pago electrónico deberán:" The consent required is the Client's express consent ("consentimiento expreso") to carry out Operaciones and services through Canales de Instrucción. It may be obtained through the Authentication process of art. 7. Source: CNBV PDF (URL in the article's `source_url`, "Disposiciones aplicables a las instituciones de fondos de pago electrónico…").
- Path taken:
  1. WebFetch `/mx/ifpe-dcg-2021/articulo/2.md` returned a paraphrase and metadata only. It refused to give the verbatim text, so the quote could not be taken from it.
  2. curl GET `/mx/ifpe-dcg-2021/articulo/2.md` returned 200, 4564 B, the full text. The quote above comes from this call.
- Failures and friction: WebFetch could not give an exact quote. I used curl (see header).
- Evidence status: supported by retrieved text.

## Q04
- Answer: LRITF art. 46, first paragraph (`urn:lex-mx:federal:statute:lritf:article:46`). Quote: "Las ITF, respecto de las cantidades de dinero que reciban de sus Clientes para la realización de las Operaciones contratadas, estarán obligadas a mantener los recursos propios segregados de los de sus Clientes, así como mantener estos últimos identificados por cada Cliente." The article continues: funds must be deposited by the end of the day they are received in demand-deposit accounts opened in the institution's name at an authorized entity. Those accounts must be different from the ones holding the ITF's own operating resources. Alternatively the funds can be placed in one-day repos with Government/Banxico securities, or in an administration trust set up for that purpose. Source: Diputados LRITF.pdf. Also seen: LRITF art. 39 fr. IV requires an application for authorization to include the "políticas de separación de cuentas, en términos de lo establecido en el artículo 46".
- Path taken:
  1. curl GET `/search?q=segregar&limit=10` returned one result, LRITF art. 18 (segregating types of operation, not money). It did not find art. 46.
  2. curl GET `/search?q=recursos propios&limit=10` returned 5 results. The first was LRITF art. 46, with a matching snippet.
  3. curl GET `/mx/lritf/articulo/46.md` returned 200, 3971 B, the full article.
  4. curl GET `/mx/lritf/articulo/39.md` returned the application requirements, including fr. IV.
- Failures and friction: The search for "segregar" missed art. 46 because search is literal on whole words, and art. 46 uses "segregados". The site itself suggests searching "segregados". My first query was therefore wasted. A first-time agent searching the natural word would not get the answer.
- Evidence status: supported by retrieved text.

## Q05
- Answer: Not on the texts I retrieved. I found no text requiring that every IFPE client have an individual bank account.
  - DCG-IFPE art. 1 defines "Cuenta" as an accounting record kept by the institution ("aquel registro contable en el que la institución de fondos de pago electrónico realiza… las anotaciones de abonos… y cargos"). It is the institution's internal ledger of a client's electronic payment funds, not a bank deposit account.
  - LRITF art. 46 requires client money to be segregated and identified per client, and the deposits to be made in demand-deposit accounts "abiertas a nombre de la institución" (the ITF's name) at an authorized entity, separate from its own funds. So the bank accounts are in the institution's name, not necessarily one per client.
  - The search snippet for DCG-IFPE art. 46 mentions "Apertura de cuentas de fondos de pago electrónico" as an activity. I did not fetch that article.
  - The texts I retrieved do not settle: whether a client needs their own bank account (for example to fund or redeem), whether the institution's pooled accounts may be one or several, or whether any other provision outside what I read imposes an individual bank account. Searches for "cuenta bancaria" returned no matches. A search with no match does not prove absence.
- Path taken:
  1. curl `/mx/ifpe-dcg-2021/articulo/1.md` returned 200, 15565 B, the definitions.
  2. curl `/mx/lritf/articulo/46.md` (see Q04).
  3. curl `/search?q=cuenta bancaria&limit=10` returned empty.
  4. curl `/search?q=cuentas de depósito de dinero a la vista&limit=10` returned 1 result, LRITF art. 46.
  5. curl `/search?q=apertura de Cuentas&limit=10&instrument=ifpe-dcg-2021` returned 2 results: DCG-IFPE art. 46 (snippet only) and art. 1.
- Failures and friction: The empty "cuenta bancaria" search wasted a call. I did not retrieve DCG-IFPE art. 46 in full or any other article on funds redemption.
- Evidence status: partly supported. I have the texts for arts. 1 and 46; the negative conclusion comes from limited search, not an exhaustive read.

## Q06
- Answer: The reference from art. 2 to art. 7 resolves to `urn:lex-mx:federal:regulation:ifpe-dcg-2021:article:7`. The references page says "resuelta; regla: no registrada. No equivale a un juicio de aplicabilidad."
  - Authentication required: art. 7 says institutions must perform Client Authentication to allow access to the Canales de Instrucción and must "recabar y validar, al menos": I. the Identificador de Cliente, and II. a Factor de Autenticación. The text prints fr. I as "El Identificador de Cliente El Identificador de Cliente y" (the phrase is duplicated in the service's text). The Identificador must be unique per client and linked to all that client's Operaciones.
  - Evidence to keep: quote, "las instituciones de fondos de pago electrónico deberán guardar evidencia de la Autenticación, conforme a lo establecido en el artículo 29, fracción IV". Art. 29 fr. IV requires logs of client accesses and activities with at least the date/time to the second, the account numbers involved, identification of the Canal de Instrucción and the Factores de Autenticación used, and the IP address or phone number. These must be stored securely for at least 180 natural days with tamper-prevention. The institution must give the client that information within 10 business days on request, for the 180 days before the request.
- Path taken:
  1. curl `/mx/ifpe-dcg-2021/articulo/2/referencias` returned 200, 1006 B, a single reference to art. 7.
  2. curl `/mx/ifpe-dcg-2021/articulo/7.md` returned 200, 3126 B.
  3. curl `/mx/ifpe-dcg-2021/articulo/29.md` returned 200, 6902 B.
- Failures and friction: The duplicated text in art. 7 fr. I looks like an extraction artifact. I could not check it against the CNBV PDF. Art. 1 defines "Factor de Autenticación" as referring to art. 5, which I did not fetch, so I cannot say which factor types are acceptable.
- Evidence status: supported by retrieved text.

## Q09
- Answer: The Q02 answer rests on LRITF art. 11 (`urn:lex-mx:federal:statute:lritf:article:11`), from the Cámara de Diputados PDF https://www.diputados.gob.mx/LeyesBiblio/pdf/LRITF.pdf, retrieved 2026-07-12T06:57:26.609663Z, release c305db4a3cfb408b76c13431e26beda72d5ce665046c95783fab36507354c969, corpus commit d2ab64d61ecfe103cc2a2980c5d04ed63c20bb7a. The source's recorded reform date is 2025-11-14 (seen in the front matter of art. 46 from the same PDF; I filtered the art. 11 front matter lines out of my output, so I cannot confirm that its own value matches). The record is labelled `temporal_status: effective` ("vigente") and `review_status: machine_accepted` ("revisada de forma automatizada"). It is not verified as current today (2026-11-18). The page says "No acredita vigencia" and "Una coincidencia no establece aplicabilidad ni vigencia actual o histórica". The home page showed the last periodic check as "no registrada". Retrieval was about four months before today.
- Path taken: curl `/mx/lritf/articulo/11.md` (see Q02), `/lex-mex/current.json` (200, 739 B), WebFetch `/lex-mex/` and `/coverage`, curl `/agents/` (200, 2740 B).
- Failures and friction: WebFetch could not reproduce `current.json` verbatim; curl did.
- Evidence status: supported by retrieved text.

## Q10
- Answer: I could not fetch it. The Ley para Regular las Sociedades de Información Crediticia is not one of the four instruments in the collection. The service says so: "not_found: Instrumento fuera de la colección: esta publicación solo contiene los instrumentos listados en /lex-mex/coverage. No demuestra inexistencia jurídica." I did not give the content of article 27 from memory. (The LRITF text I retrieved mentions the law only in passing, in art. 18 fr. VI.)
- Path taken:
  1. curl `/lex-mex/mx/lsic/articulo/27` returned HTTP 404, 953 B, the message above. I guessed the slug `lsic` (it is not listed anywhere).
- Failures and friction: Guessed slug. The 404 message was clear and honest. I did not look for the other slug names, since the instrument list shows only four.
- Evidence status: not answerable (outside the collection).

## Q11
- Answer: The service does not hold the Ley Federal del Trabajo, so I cannot say what it states about rescission without liability for the employer. I did not answer from memory.
- Path taken:
  1. curl `/lex-mex/mx/lfpt/` returned HTTP 404, 848 B: "not_found: Instrumento no disponible en esta publicación." (slug guessed.)
  2. curl `/lex-mex/search?q=rescisión&limit=10` returned 200 with no results ("No se encontraron coincidencias en esta selección"). The coverage page lists the four instruments and no labour law.
- Failures and friction: The slug in step 1 was a guess. The two answers differ in wording ("fuera de la colección" in Q10 versus "no disponible en esta publicación" here), but both are clear.
- Evidence status: not answerable (outside the collection).

## Q12
- Answer: Retrieved articles of the LRITF that bear on risk disclosure to clients.
  - Art. 11, second paragraph (all ITFs), quote: "las ITF deberán difundir la información que permita a sus Clientes identificar los riesgos de las Operaciones que celebren con o a través de ellas, conforme a lo previsto en esta Ley." It also requires measures to prevent false or misleading information being spread through them.
  - Art. 34 (ITFs that operate with virtual assets): they must disclose the risks, at minimum in simple, clear terms on their website or service medium. The four points are: the asset is not legal tender and is not backed by the Federal Government or Banxico; operations cannot be reversed once executed, where applicable; the asset's value is volatile; and the technological, cyber and fraud risks.
  - Art. 39 fr. V: an application for authorization must include "Las políticas de divulgación de riesgos y responsabilidades por la realización de las Operaciones en la ITF". These must cover commissions and charges and warnings shown on the interface, in line with CNBV rules.
  - Art. 18 fr. II and III (crowdfunding institutions only): analyze and inform investors of the risk of applicants and projects, and obtain from investors an electronic record that they know the risks of their investment.
  - Art. 87 fr. VIII (Modelo Novedoso temporary authorization): applicants must state the means they will use to inform Clients of the risks they face.
  - Art. 46 and the rest I read do not add a general risk-disclosure duty. I did not read the search hits on LRITF arts. 7, 19, 21, 36 and 92 (snippets only), nor page 2 of the results. So this list may be incomplete.
  - Source: Diputados LRITF.pdf (same release).
- Path taken:
  1. curl `/search?q=riesgos&limit=10` returned 10 results across all four instruments (LRITF 34 first), with a next page.
  2. curl `/search?q=riesgos&limit=10&instrument=lritf` returned 10 LRITF results (arts. 34, 7, 19, 36, 18, 39, 11, 21, 87, 92), with a next page. The `instrument` filter worked.
  3. curl `/mx/lritf/articulo/11.md`, `/34.md`, `/18.md`, `/39.md`, `/87.md`, `/48.md`. Each returned 200 with the full article.
- Failures and friction: I did not follow the next-page link, so results on page 2 are unseen.
- Evidence status: supported by retrieved text for the articles cited; partly complete as a list.

# Overall friction
- URLs that returned errors: `/lex-mex/mx/lsic/articulo/27` returned 404 (instrument outside the collection, guessed slug); `/lex-mex/mx/lfpt/` returned 404 (not available, guessed slug). No other URL failed. Every other request returned 200, including the search pages with no results.
- WebFetch could not be used for exact quotes. It returned summaries and, for the article and `current.json`, said it could not reproduce more than 125 characters. A first-time agent with only WebFetch could not complete the verbatim-quote questions (Q03 and Q06) and would have to fall back to curl or the `.md` URL through another tool.
- The home page `/lex-mex/` is JavaScript-rendered, so a plain fetch got almost nothing ("Preparando el corpus…"), with no release id and no link to search. A first-time agent has to find `llms.txt` (the one link visible) to get started.
- Search is literal, whole-word, no stemming. "segregar" missed LRITF art. 46 ("segregados"); "cuenta bancaria" and "responsabilidad del Gobierno Federal" found nothing. A first-time agent could wrongly conclude the rule is absent. The site warns that zero results prove nothing, but only if the agent reads it.
- Results have snippets only; you have to open the article. The best hit was not always on page 1 (art. 11 was not found by "organizarse y operar").
- Article text can contain extraction artifacts (the duplicated phrase in art. 7 fr. I).
- The site gives no verification of current validity (`No acredita vigencia`, last check "no registrada"), and retrieval dates from 2026-07-12, so agents should not treat any answer as current law.
- Side note on my own run: one curl call wrote its output to `/tmp/x.out` (a throwaway file for the out-of-collection probes). I did not read any other local file.
