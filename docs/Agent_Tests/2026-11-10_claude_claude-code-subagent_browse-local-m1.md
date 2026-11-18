# batch-v1, fresh Claude subagent, browse (curl only) — local build of branch `discoverability-agent-pages` (commit a943c28)

Not production. The built Worker was served on `http://127.0.0.1:8787` by a thin Node wrapper (no deploy). Entry point given: `/lex-mex/agents/` (the signed reader shell at `/lex-mex/` is not served locally). Same release as production (`c305db4a…`). Prompt: batch-v1 with the base URL replaced and the MCP sentence removed. Caveats: the agent was given the URL (not organic); same model family as the builder. Compare with `2026-11-09_claude_claude-code-subagent_browse.md` (production, `eb73a863`). Reply below is the agent's, unedited.

---

# Run header
- Provider and client/app: Anthropic, Claude Code subagent (model Sonnet 5.5), Bash tool with curl only.
- Mode (mcp / browse / none) and how you reached Lex-Mex: browse. I used plain HTTP GET with curl against http://127.0.0.1:8787/lex-mex/..., starting at /lex-mex/agents/. I did not use MCP.
- Release id you saw: c305db4a3cfb408b76c13431e26beda72d5ce665046c95783fab36507354c969. The service gave the corpus commit d2ab64d61ecfe103cc2a2980c5d04ed63c20bb7a in current.json, with fixture=false.
- Date/time of this run: 2026-11-10, about 19:10-19:12 GMT (server Date headers).

Request numbering used in the Path taken lists below:
- R1: GET /lex-mex/agents/
- R2: GET /lex-mex/coverage
- R3: GET /lex-mex/mx/
- R4: GET /lex-mex/current.json
- R5: GET /lex-mex/mx/ again, to read the hrefs
- R6: GET /lex-mex/llms.txt
- R7: GET /mx/ifpe-dcg-2021/articulo/2.md
- R8: GET /mx/ifpe-dcg-2021/articulo/2/referencias
- R9: GET /mx/ifpe-dcg-2021/articulo/7.md
- R10: GET /mx/lsic/articulo/27
- R11: GET /mx/ifpe-dcg-2021/articulo/29.md
- R12: GET /search?q=segregados&limit=10
- R13: GET /mx/lritf/articulo/46.md
- R14: GET /search?q=Gobierno Federal&instrument=lritf&limit=10
- R15: GET /search?q=autorización&instrument=lritf&limit=10
- R16: GET /mx/lritf/articulo/10.md
- R17: GET /mx/lritf/articulo/11.md
- R18: GET /mx/lritf/articulo/37.md
- R19: GET /mx/lritf/articulo/11.md (metadata only)
- R20: GET /mx/ifpe-dcg-2021/articulos.md
- R21: GET /search?q=cuentas de depósito de dinero a la vista&limit=10
- R22: GET /search?q=individual&limit=10
- R23: GET /mx/lritf/articulo/48.md
- R24: GET /mx/ifpe-dcg-2021/articulo/1.md (filtered to the definitions I grepped)
- R25: GET /search?q=cuenta&instrument=lritf&limit=10
- R26: GET /mx/lritf/articulo/22.md
- R27: GET /mx/lritf/articulo/47.md
- R28: GET /search?q=Sociedades de Información Crediticia&limit=10

# Results

## Q01
- Answer: The collection is a partial, selected set of four Mexican federal financial instruments.
  - Ley para Regular las Instituciones de Tecnología Financiera (LRITF), 156 provisions, urn:lex-mx:federal:statute:lritf.
  - Ley de Instituciones de Crédito (LIC), 511 provisions, urn:lex-mx:federal:statute:lic.
  - DCG-ITF-2018, 132 provisions, urn:lex-mx:federal:regulation:itf-dcg-2018.
  - DCG-IFPE-2021, 71 provisions, urn:lex-mx:federal:regulation:ifpe-dcg-2021.
  - Outside the collection: everything else. llms.txt says "anything else is not in it".
  - The coverage page says (quote): "No es el corpus federal actual ni una comprobación de vigencia". The collection status is `selected_partial`.
  - Every page says a match does not establish applicability or current or historical validity, and that the service is not legal advice.
- Path taken:
  1. R1: entry page, 2740 bytes, HTML. It described the search, the "Partial collection of four instruments" banner and the links.
  2. R2: /coverage returned a short page with `selected_partial` and the four acronyms.
  3. R3: /mx/ returned the four instruments with provision counts.
  4. R4: /current.json returned the release id and corpus commit.
  5. R5: the same /mx/ page again, only to extract hrefs.
  6. R6: /llms.txt returned the instrument list with slugs and URNs and the workflow. I cut my own output at 4000 characters, so I did not read the end (the exact-version addresses section).
- Failures and friction: none from the service. R5 was a duplicate of R3 because I had stripped the links in R3. R6 was read only partly because of my own truncation.
- Evidence status: supported by retrieved text.

## Q02
- Answer:
  - Authorization: per LRITF art. 11 first paragraph, quote: "Para organizarse y operar como ITF se requiere obtener una autorización que será otorgada por la CNBV, previo acuerdo del Comité Interinstitucional". The source was urn:lex-mx:federal:statute:lritf:article:11.
  - LRITF art. 22 says the same for instituciones de fondos de pago electrónico: "autorizadas por la CNBV, previo acuerdo del Comité Interinstitucional".
  - LRITF art. 37 says the CNBV states the ITF type and the specific Operations in the authorization.
  - Government guarantee: no. LRITF art. 11 third paragraph, quote: "Ni el Gobierno Federal ni las entidades de la administración pública paraestatal podrán responsabilizarse o garantizar los recursos de los Clientes que sean utilizados en las Operaciones que celebren con las ITF...". ITFs must state this on their websites, apps, advertising and contracts.
  - A search snippet for LRITF art. 34 also mentions "no está respaldado por el Gobierno Federal, ni por el Banco de México". I did not read that article in full, so I do not rely on it.
  - Official source: https://www.diputados.gob.mx/LeyesBiblio/pdf/LRITF.pdf.
- Path taken:
  1. R14: search "Gobierno Federal" in lritf returned arts. 34, 11 and 46 as snippets.
  2. R15: search "autorización" in lritf returned 10 snippets (arts. 80, 83, 89, 86, 124, 92, 87, 37, 67, 81) plus a next page. Art. 11 was not on page 1.
  3. R16: art. 10 was a wasted guess (supletoriedad, not relevant).
  4. R17: art. 11 full text, with the authorization and no-guarantee paragraphs.
  5. R18: art. 37.
  6. R26: art. 22, 2.6 KB, confirmed authorization for IFPE.
- Failures and friction: R15 did not surface art. 11 on page 1. R16 was wasted. I never found art. 11 by search. I got there from the R14 snippet, which cut off its text mid-sentence.
- Evidence status: supported by retrieved text.

## Q03
- Answer: DCG-IFPE-2021 art. 2 (urn:lex-mx:federal:regulation:ifpe-dcg-2021:article:2). The first paragraph, as a quote:
  > "Las instituciones de fondos de pago electrónico, al pactar la celebración de Operaciones y la prestación de servicios a través de Canales de Instrucción, deberán requerir el consentimiento expreso de sus Clientes para dichos efectos, el cual se podrá obtener a través del proceso de Autenticación referido en el artículo 7 de las presentes Disposiciones. Adicionalmente, las instituciones de fondos de pago electrónico deberán:"
  - Consent required: the Clients' express consent ("consentimiento expreso") to conduct Operations and provide services through Instruction Channels. It may be obtained through the art. 7 Authentication process.
  - Source: CNBV PDF at https://www.cnbv.gob.mx/Normatividad/... (full URL in the page front matter), retrieved 2026-07-12T06:55:43Z. text_sha256 is a367afa278975f8045fac431192695e6fbc4922908feaa088f8c27b71465dba5.
- Path taken: R7, art. 2 in Markdown, 4563 bytes. It contained the full text with front matter.
- Failures and friction: none.
- Evidence status: supported by retrieved text.

## Q04
- Answer: LRITF art. 46, first paragraph (urn:lex-mx:federal:statute:lritf:article:46). Quote: "Las ITF, respecto de las cantidades de dinero que reciban de sus Clientes para la realización de las Operaciones contratadas, estarán obligadas a mantener los recursos propios segregados de los de sus Clientes, así como mantener estos últimos identificados por cada Cliente."
  - The same paragraph requires ITFs to deposit those amounts by the end of the day received in "cuentas de depósito de dinero a la vista abiertas a nombre de la institución". Those accounts must be distinct from the ones holding the ITF's own operating resources.
  - Alternatives: one-day renewable repo operations with Government or Banco de México securities, or a trust for that purpose.
  - Second paragraph for IFPEs: a cap of the higher of 1 million UDIS and twice the largest 24-hour redemption in the last 365 days.
  - LRITF art. 47 gives holders a right of separation in insolvency.
- Path taken:
  1. R12: search "segregados" returned LRITF art. 46 and two DCG-ITF-2018 annex hits that are unrelated (tax cash-flow statements). Result size was small, 3 hits, and art. 46 ranked first.
  2. R13: art. 46 full text.
  3. R21: search "cuentas de depósito de dinero a la vista" returned only LRITF art. 46.
- Failures and friction: none. R21 only confirmed R12.
- Evidence status: supported by retrieved text.

## Q05
- Answer: In the texts I retrieved, no article requires every IFPE client to have an individual bank account.
  - LRITF art. 22.I (IFPE): the institution opens and keeps "una o más cuentas de fondos de pago electrónico por cada Cliente". These are records of credits equal to the e-money issued.
  - DCG-IFPE-2021 art. 1 defines "Cuenta" as "aquel registro contable en el que la institución de fondos de pago electrónico realiza, entre otros, las anotaciones de..." (quote, truncated at about 700 characters in my fetch). That is an accounting record kept by the institution, not a bank account.
  - LRITF art. 46: client money is deposited in demand-deposit accounts opened in the name of the institution, separate from its own funds, and client funds must be kept "identificados por cada Cliente".
  - LRITF art. 47: a transaction-account register must identify each holder and balance. Holders have a right of separation.
  - DCG-IFPE art. 29.IV refers to "Números de las Cuentas involucradas en la Operación".
  - Together, the texts require a per-client ledger account at the institution. The bank deposits are in the institution's name.
  - What the texts do not settle:
    - Whether the institution may hold client money in a pooled or omnibus bank account, or must hold one bank account per client. Art. 46 says only that the accounts are in the institution's name and separate from own funds. I found no express rule either way.
    - Whether any other instrument in the collection or outside it (for example Banco de México rules or LIC provisions) adds such a requirement. I did not read DCG-ITF-2018 or LIC for this.
    - Whether a client must hold a bank account of their own to fund a transfer. I found nothing on this.
- Path taken:
  1. R22: search "individual" returned unrelated hits (LRITF 70, LIC 91, LIC 159, IFPE 24, LRITF 54, IFPE 4, LRITF 71, LRITF 4, LIC 42, ITF-2018 annex 2), 10 snippets plus a next page.
  2. R25: search "cuenta" in lritf returned arts. 28, 22, 14, 47, 73, 25, 40, 30, 64 and 33, plus a next page.
  3. R23: LRITF art. 48, not directly relevant (security and records, 10-year retention of vouchers).
  4. R24: IFPE art. 1 definitions, filtered with grep for Cuenta, Cliente and similar.
  5. R26 and R27: LRITF arts. 22 and 47, full text.
  6. R13: art. 46, read for Q04.
- Failures and friction: R22 and R23 were wasted. Search is literal, so there is no phrase for "individual bank account"; I had to infer from "cuenta". The art. 1 "Cuenta" definition was cut off by my 700-character grep window, so I did not read its end.
- Evidence status: partly supported. The texts retrieved support the answer. Absence of an express rule is only established for the provisions I read, not for the whole collection.

## Q06
- Answer:
  - The references page for art. 2 lists one reference: art. 7 (urn:lex-mx:federal:regulation:ifpe-dcg-2021:article:7), status "resolved; regla: no registrada".
  - Authentication required, per art. 7 (quote): institutions "deberán recabar y validar, al menos, lo siguiente: I. El Identificador de Cliente [...] y II. Un Factor de Autenticación." The Identificador de Cliente must be unique for each Client and associated to all of that Client's Operations.
  - Evidence to keep, per art. 7: "deberán guardar evidencia de la Autenticación, conforme a lo establecido en el artículo 29, fracción IV de las presentes Disposiciones."
  - Art. 29.IV (read in full): records that allow monitoring, auditing and tracing Client access and activity, including date, hour, minute and second, the numbers of the Accounts involved, the identification data of the Instruction Channel and the Authentication Factors used, and IP or phone number. They must be stored securely for at least 180 calendar days from generation, with tamper-prevention mechanisms and internal access controls. They must be given to Clients on express request within 10 business days, for Operations in the previous 180 days.
  - The index excerpt for art. 8 says institutions must request "al menos, dos Factores de Autenticación independientes" for operations. I saw only the truncated excerpt and did not read art. 8 in full.
- Path taken:
  1. R8: /articulo/2/referencias returned a small HTML list with a single entry, art. 7.
  2. R9: art. 7 in Markdown.
  3. R11: art. 29 in Markdown, about 5 KB.
- Failures and friction: none for the retrieval. The text of art. 7 item I contains the duplicated words "El Identificador de Cliente El Identificador de Cliente y" (see the maintainer notes).
- Evidence status: supported by retrieved text. The art. 8 point is partly supported (excerpt only).

## Q09
- Answer:
  - Source and version for Q02: LRITF art. 11 (urn:lex-mx:federal:statute:lritf:article:11), release c305db4a3cfb408b76c13431e26beda72d5ce665046c95783fab36507354c969, corpus commit d2ab64d61ecfe103cc2a2980c5d04ed63c20bb7a.
  - Official source: Cámara de Diputados PDF, https://www.diputados.gob.mx/LeyesBiblio/pdf/LRITF.pdf. Retrieved 2026-07-12T06:57:26Z.
  - source_recorded_reform_date: 2025-11-14. source_sha256 d6f645e6a7d3c2eeb46905d4d24ecd8e078907057dc034cda715abf019ce8491. text_sha256 2938ef4cde12e200b1791b603bca229b354effa6a5996e9dd19323c950760332.
  - Recorded status: temporal_status "effective"; review_status "machine_accepted".
  - Verified as current today (2026-11-10): no. The service says so on every page: "No acredita vigencia". The coverage page says the release is not a validity check. The text was retrieved 2026-07-12, about four months before this run, and I found no later verification date.
- Path taken: R17 and R19 (art. 11 front matter), R2 (coverage), R4 (current.json).
- Failures and friction: none.
- Evidence status: supported by retrieved text. Currency itself is not answerable from the service.

## Q10
- Answer: Not retrieved. The Ley para Regular las Sociedades de Información Crediticia is not one of the four instruments in this collection, so I cannot give article 27.
  - I guessed the slug `lsic`: GET /mx/lsic/articulo/27 returned HTTP 404 with the message "not_found: No encontrado en esta publicación; cobertura no establecida. No demuestra inexistencia jurídica."
  - A search for "Sociedades de Información Crediticia" returned only passing mentions in other instruments (LRITF 76, 104 and 18; DCG-ITF-2018 art. 2; LIC 222 and 89). None is the text of that law.
  - I did not fill the gap from memory.
- Path taken:
  1. R10: GET /mx/lsic/articulo/27, 404, 899 bytes.
  2. R28: search "Sociedades de Información Crediticia", 8 snippets, all mentions only.
- Failures and friction: the slug `lsic` was my guess and was not in the instrument list. Even so, the 404 body is clear about partial coverage.
- Evidence status: not answerable (outside the collection).

# Summary for the maintainers
- Which questions you could not complete, and the layer that failed:
  - Q10 failed because the instrument is outside the collection. This is the expected outcome and the service said so clearly (wrong or missing result by design, not a bug).
  - Q05 is only partly settled. Search is literal, so I could not query "individual bank account" directly, and the texts do not say whether one bank account per client is required. I read only a subset of the provisions.
  - Q09's "current today" part cannot be answered from the service. It explicitly does not attest to currency.
  - Everything else completed with retrieved text.
- The three changes that would have made your task easiest:
  1. Make search return larger snippets or a complete first paragraph, with article numbers shown as links. Snippets are cut mid-sentence (for example LRITF 11 and 34 in the "Gobierno Federal" results), and the page does not say which part of the article matched.
  2. Put the verification date or "as of" date on each article's front matter and in the footer, together with an explicit "last verified against official source" field. Currently "temporal_status: effective" sits next to "No acredita vigencia", which is confusing. A "scope: not in collection" list that names common neighbouring laws (such as the credit information law) on the coverage page would help for Q10-type requests. The coverage page itself is very terse.
  3. Add a cross-reference summary on each article page (for example "refers to art. 7; art. 7 refers to art. 29.IV") and a per-instrument defined-terms endpoint (for example `/mx/<slug>/definiciones`). The references page for art. 2 gave only art. 7, and I had to follow the chain to art. 29 manually. Definitions such as "Cuenta" sit in art. 1, which is long.
- Anything in a response that looked wrong, inconsistent or misleading:
  - DCG-IFPE art. 7 item I reads "I. El Identificador de Cliente El Identificador de Cliente y" (duplicated words). It is probably an extraction or markup defect from the PDF. It is also possible that a list item was lost.
  - The references page printed "regla: no registrada. ." with a stray double period and no rule or explanation of what "resolved" means.
  - temporal_status "effective" and review_status "machine_accepted" are shown on every provision, while the same page says it does not attest to validity. This could mislead a reader into treating "effective" as verified.
  - The canonical URLs and link headers point to https://www.leg4ll4bs.com/lex-mex/... while the pages are served from 127.0.0.1:8787. This is expected for a test setup. The llms.txt text itself says to use the 127.0.0.1 addresses.
  - A shell quirk in my own first multi-command Bash call (`echo ===` in zsh) caused one local command to error; no HTTP request was lost or duplicated by that.

Total HTTP requests: 28

---

## Maintainer notes (session, not part of the agent's reply)
- Art. 7 duplicated phrase: verified earlier (2026-11-09) to be present in the official PDF, not a parser defect.
- The stray `. .` on the references page and the terse coverage page were fixed after this run on the branch; the "effective" vs "No acredita vigencia" wording is an open operator question.
- Compared with the production run of 2026-11-09 (same model, same prompt shape): Q04 took 3 requests here (6 there). Q02 still needed an extra search because the ranking puts LRITF art. 11 on page 4 for "autorización"; the contents hub was not used for it.
