# Lex-Mex | IFC Authorization Research Benchmark and Agent Optimization Plan

**Version:** 1.0  
**Assessment date:** 2026-10-24 (Mexico City)  
**Target:** https://www.leg4ll4bs.com/lex-mex/  
**Research question:** “What authorities are involved in authorizing a Mexican *institución de financiamiento colectivo* (IFC), and what does each check?”  
**Protocol:** Pass A uses only the public Lex-Mex surface; Pass B independently uses official Mexican legal sources. No Lex-Mex-derived legal text was imported into Pass B.

## 1. Executive finding

**The independent research pass answered the question, while the Lex-Mex retrieval pass did not reach the statute.** The public Lex-Mex landing page and the agent discovery document were discoverable, but the client used for this test failed to retrieve the release-pinned Markdown index and JSON manifest announced by that document. Consequently, this is **not a valid measure of Lex-Mex legal accuracy or intrinsic token efficiency**: the comparison was interrupted by an access/interoperability problem at the retrieval layer.

The product already has a sound foundation: a small `/lex-mex/llms.txt`, immutable release identifiers, references to a corpus index and manifest, explicit partial-coverage language, and source provenance claims. The highest-value next step is **proving artifact accessibility across different agent clients** before adding a more elaborate search or agent architecture.

**Recommended order:** (P0) externally verify stable text retrieval → (P0) make legal provisions directly addressable and indexable → (P1) add small, high-precision lexical search → (P1) expose law/version/effectiveness/citation metadata → (P1) instrument retrieval budgets and regression tests → (P2) only then consider structured answer APIs or a Model Context Protocol (MCP) server.

## 2. Pass A: Lex-Mex-only test

### 2.1 Actual steps and observations

| Step | Exact public route / method | Observed result | Interpretation |
|---|---|---|---|
| A1 | `https://www.leg4ll4bs.com/lex-mex/` via web retrieval | HTML landing page returned; the rendered text included “Preparando el corpus…” and “Acceso para agentes” | Landing page is discoverable, but the initial HTML is a JavaScript-dependent shell for substantive article text. |
| A2 | Click **Acceso para agentes** | Resolved to `/lex-mex/llms.txt`, delivered a **short, 11-line, plaintext** agent-discovery document | Strong lightweight discovery entry point. A direct URL-opening attempt to the same route also failed in this specific client, indicating client/path handling variation. |
| A3 | Open the exact release index advertised in `llms.txt` | Client reported `Internal Error` / “not accessible via this tool” | Unable to discover available instruments or reach provisions via that path. No verified HTTP status or origin-side diagnosis. |
| A4 | Open the exact release manifest advertised in `llms.txt` | Same client error | Unable to independently inspect coverage and source-version metadata. |
| A5 | Retry index/manifest with harmless query-string variants and alternate host | Same client error | No evidence that query-string rewriting repairs the issue. |
| A6 | Try direct Python HTTPS requests from the execution container | DNS name resolution failed in this container | **Environment limitation**, not evidence of failure by the Lex-Mex server. Prior tests from another environment may still pass. |

The public discovery document exposed this release identifier:

`3c04016651e2fd4169632756e96ec19136136e1f42c4ed666070f23f20b30660`

Its advertised paths were:

- `https://www.leg4ll4bs.com/lex-mex/releases/3c04016651e2fd4169632756e96ec19136136e1f42c4ed666070f23f20b30660/index.md`
- `https://www.leg4ll4bs.com/lex-mex/releases/3c04016651e2fd4169632756e96ec19136136e1f42c4ed666070f23f20b30660/manifest.json`

The discovery text explicitly warns that the corpus is partial, advises retrieval of only relevant shards, and states that there is no last-successful-upstream-check ledger or guarantee of historical validity. These are meaningful qualifications, not generic disclosures.

**Pass A result:** Legal answer **not completed using Lex-Mex alone**. Whether the latest LRITF or relevant articles were included in the release is **unverified**, because the index and manifest could not be retrieved. The observed symptom could arise from the particular retrieval client, response handling of `.md`/`.json` artifacts, intermediary restrictions, server configuration, or another cause. It is **not** proof that `curl` or an ordinary browser receives an error.

### 2.2 What was **not** tested

This assessment did **not** run a real JavaScript browser session, execute client-side search, inspect HTTP status codes from the origin, inspect the deployment, review the codebase, benchmark retrieval latency, or obtain provider-metered token counts. No claim should be made about those without an instrumented follow-up.

## 3. Pass B: Independent legal answer

### 3.1 Classification and controlling framework

An IFC is a regulated *institución de tecnología financiera* undertaking professional and habitual crowdfunding facilitation as defined by Articles 15–16 of the **Ley para Regular las Instituciones de Tecnología Financiera (LRITF)**. The authorization is a financial-institution license, not an authorization for each crowdfunding transaction or project. Its legal basis is principally LRITF Articles **15, 35–40, and 58**, together with the **Disposiciones de carácter general aplicables a las instituciones de tecnología financiera** (especially Article 3) and the CNBV’s IFC authorization guide.

**Current-law caution:** The independently obtained Chamber of Deputies PDF states “Última Reforma DOF 14-11-2025.” The LRITF PDF attached to the wider project bears the earlier “Última Reforma DOF 24-01-2024.” A version-aware retrieval service must not silently use the older snapshot when answering as of October 2026.

### 3.2 Authorities and what each reviews

| Authority/body | Legally established role | Principal subjects implicated in its review | Classification of this description |
|---|---|---|---|
| **Comisión Nacional Bancaria y de Valores (CNBV)** | **Receives, evaluates and grants** authorization, subject to prior agreement of the Interinstitutional Committee; later supervises and verifies conditions to commence operations. | Corporate form and bylaws; business model and permitted IFC operations; financial projections, funding and minimum capital; controllers, shareholders, directors; governance; segregation of funds; risk disclosures; operational and technological controls; cyber and information security; providers; investor protection; anti-fraud and anti-money-laundering documentation. | **Statutory decision-making role.** Subjects arise from LRITF Arts. 36, 39–40 and general regulations Art. 3; not a claim that the CNBV alone reviews each item. |
| **Comité Interinstitucional** | Must reach the **prior agreement** necessary for the CNBV to grant an IFC authorization. Six proprietor members: two appointed by the CNBV, two by the **Secretaría de Hacienda y Crédito Público (SHCP)** and two by **Banco de México (Banxico)**. | The **whole application**, which the CNBV must make available to all Committee members (LRITF Art. 39). | **Statutory collegial approval**, not a separate licensing office issuing a parallel authorization. |
| **SHCP (through Committee members)** | Participates in approval; independently promulgates anti-money-laundering/counter-terrorist-financing (AML/CFT) rules for technology-finance institutions under LRITF Art. 58, after CNBV opinion. | Financial-sector public policy and AML/CFT framework, including source-of-funds and compliance risks in the application. | Committee participation and AML/CFT rulemaking are **express**; a formal SHCP-only review checklist or exclusive jurisdiction over those file sections is **not established** by Art. 35. |
| **Banxico (through Committee members)** | Participates in approval. Separately has relevant powers concerning operations with virtual assets and foreign currency under LRITF Art. 37, as well as statutory oversight of payment-related operations within its jurisdiction. | Payment-system implications when relevant; the operational model; technology/settlement risks; special currency or virtual-asset operations if proposed. | Committee vote and special authorizations are **express**. A mandatory separate Banxico licensing proceeding for every ordinary IFC application is **not** established. |

**Crucial voting rule:** The Committee has a quorum when at least three members attend and all three constituent authorities are represented. Resolutions are generally by simple majority, with a casting vote for the chair on a tie. For licenses to operate as a financial technology institution, **at least one representative of each authority must vote in favor** (Art. 35). Therefore none of the three authorities can be bypassed in the approval.

**Do not hallucinate three silos.** Article 35 does not say, for example, that “SHCP *approves only AML*” or “Banxico *approves only payment architecture*.” Those are possible areas of institutional attention, not separate binding approval tests. The shared file and collegial vote matter more than an artificial division of responsibility.

### 3.3 What is reviewed in the application, across the institutions

Under LRITF Article 39, the application includes corporate documentation, a business plan, account segregation, fee/risk disclosures, operational security and cybersecurity, client identification and selection, conflicts of interest, fraud and AML/CFT policies, key technology providers and contracts, shareholder identity and source of capital, directors’ suitability, rights to the digital interface, local service address, and IFC-specific incentive alignment. Article 3 of the CNBV general provisions adds details including financial viability for the first three years, projected financial statements, the service/fee structure and operations to be conducted, technology characteristics, operating/control/risk manuals and compliance-officer certification.

The CNBV **2022 IFC authorization guide** organizes the dossier into (1) application, (2) supporting corporate/financial/operational documentation, (3) AML/CFT documentation, and (4) commencing-operations evidence. Its guidance is informational and **not itself binding law**; the CNBV makes that distinction expressly on its public guidance page.

**Authorization versus commencement:** Under Art. 40, the licensee must demonstrate readiness to the CNBV **at least 30 business days before commencing business**, including incorporation/registration, subscribed and paid capital, eligible directors and operational/technology/control infrastructure. The CNBV may inspect. Article 40 expressly provides for joint CNBV–Banxico inspections for **electronic payment funds institutions (IFPEs)**, not as a general IFC condition; avoid importing the IFPE-specific requirement into the IFC path.

**Other bodies:** The **Comisión Nacional para la Protección y Defensa de los Usuarios de Servicios Financieros (CONDUSEF)** has subsequent consumer-protection and reporting competencies, but is **not one of the three Art. 35 voting authorities**. The **Unidad de Inteligencia Financiera (UIF)** is relevant to the AML/CFT institutional ecosystem, but is **not a separately designated seventh committee member or fourth approving authority**. Ordinary tax, company registration or privacy regulators likewise should not be described as granting the IFC license merely because their rules may apply.

### 3.4 Primary-source references

1. [LRITF, Chamber of Deputies, text last amended 14 November 2025](https://www.diputados.gob.mx/LeyesBiblio/pdf/LRITF.pdf) — Article 15 (classification); Article 35 (competence, composition and voting); Articles 36–39 (eligibility, license and dossier); Article 40 (operational commencement); Article 58 (AML/CFT). Particularly PDF pp. 17–20 and 27–28.
2. [Official Diario Oficial de la Federación (DOF) original 2018 LRITF publication](https://sidof.segob.gob.mx/notas/docFuente/5515623) — historical original, not a substitute for the current consolidated text.
3. [CNBV’s updated public portal listing authorization guides, 16 January 2026](https://www.gob.mx/cnbv/documentos/guias-cnbv-para-la-autorizacion-de-instituciones-supervisadas) — includes the October 2022 IFC guide.
4. [CNBV guidance page explaining legal status of its IFC/IFPE guides](https://www.gob.mx/cnbv/acciones-y-programas/proceso-de-autorizacion-y-guias-de-las-instituciones-de-tecnologia-financiera-itfs) — expressly describes guides as practical, nonbinding and non-guaranteeing.
5. [Official DOF publication of CNBV’s general fintech provisions](https://sidof.segob.gob.mx/notas/docFuente/5537450) — base text; consult later published amendments and the updated compilation for current wording.
6. [DOF modification dated 9 September 2025](https://dof.gob.mx/2025/SHCP/SHCP_090925.pdf) — example of subsequent normative changes that a static corpus must track.

## 4. Comparison: what this experiment can and cannot establish

| Criterion | Lex-Mex first pass | Independent primary-source pass |
|---|---|---|
| Discovery | Landing page and 11-line agent document recovered | Targeted official-source searches located the current LRITF immediately |
| Can the client retrieve relevant controlling provisions? | **Not observed**: release index and manifest access failed | **Yes**: official LRITF available and its article ranges searchable |
| Correct legal conclusion produced from that pass alone? | **No** | **Yes** |
| Source version identified? | Release hash identified; underlying statute's version **not verified** | LRITF consolidation explicitly says last reform **14-11-2025** |
| Citations at article level? | **Not tested**; no release-shard content retrieved | **Yes**: LRITF Arts. 15, 35–40, 58; CNBV general provisions Art. 3 |
| Search/retrieval effort | Repeated retrieval failures, client mismatch possible | One controlling PDF plus targeted lookups and corroborating CNBV/DOF sources |
| Prompt-token consumption | **Not instrumented** | **Not instrumented** |
| Latency and server byte counts | **Not instrumented** | **Not instrumented** |
| Legal precision | **Cannot assess** | Correctly distinguishes CNBV grant, the Committee’s three-authority assent, and nonexclusive review focuses |

**Verdict:** In this client's test conditions, independent research **won on task completion**, not demonstrably on token cost, speed or inherent answer quality. Claiming that Lex-Mex is “worse” because of one failed tool transport would overstate the evidence. Equally, a link present in `llms.txt` is **not proof** that an arbitrary AI client can read its target. Both facts should remain visible in reporting.

## 5. Prioritized improvements for an agent-first legal corpus

### P0. External retrieval compatibility: fix the path before optimizing inference

**Observed problem:** Main page and agent discovery route can be found, but the release-pinned Markdown index and JSON manifest were inaccessible through this tool.

**Implement/test:**

- Verify exact `GET`/`HEAD` responses for `/lex-mex/llms.txt`, pinned `/index.md`, pinned `/manifest.json`, and a real article URL using at least three independent clients: ordinary `curl` on a machine with network access, an external cloud machine, and an agent web-retrieval client.
- Capture **actual HTTP status, redirect chain, content-type, content-length, selected cache headers, final URL, first 200 body bytes and elapsed time**. Do not infer HTTP 403/404 or server blocking from a tool-level `Internal Error`.
- For text assets, return a clean `200`, UTF-8, `text/markdown; charset=utf-8` for `.md`, `application/json` for JSON and `text/plain` for `llms.txt`; honor `GET` without cookies, user session or JavaScript.
- Check content negotiation, `robots.txt`, CDN/security middleware, redirects, filename extensions, stale artifact links and case sensitivity. Establish which actually causes failures **before making configuration changes**.
- For immutable release assets, use long-lived immutable caching and stable checksums; make a small pointer manifest the only frequently-changing lookup.
- Avoid requiring the site’s visual reader to finish hydration before an agent can recover plain legal text.

**Acceptance:** A clean unauthenticated agent on two independent networks can traverse `llms.txt` → current release index → LRITF article 35 and 39 in **≤4 successful text requests**. A failed link returns an inspectable status and small error body. Test each release in Continuous Integration (CI) after deployment.

### P0. Make provisions first-class, stable resources

The existing article-per-Markdown approach is already a strong storage model. Expose it directly and deterministically. **Illustrative future routes, not verified existing routes:**

```text
/lex-mex/llms.txt
/lex-mex/corpus.json                         # current pointers + coverage
/lex-mex/laws/lritf/index.md                 # stable current human/agent law index
/lex-mex/laws/lritf/articles/35.md          # stable current citation target
/lex-mex/laws/lritf/articles/39.md
/lex-mex/releases/{release}/laws/lritf/articles/35.md  # pinned text
```

- Both a **stable current** URL and an **immutable versioned** URL are required. Link them via manifest metadata, not by silently redirecting every pinned request to current text.
- Give each provision a stable identifier (`MX-FED-LRITF-ART-35`), a precise heading, a canonical law identifier, article number, authoritative source URL, publication/update and effective dates **when verified**, corpus release hash and content hash.
- Preserve legal substructure: article, paragraph, fraction, clause, transitional provision; provide fragment identifiers for citations when reliable.
- Expose **exact legal text by default**, not an unexplained abstract or model-generated rewrite. If summaries exist, label them derived/nonauthoritative and return them only when requested.

**Acceptance:** A `GET` on one article serves that article, not the full law, a page’s UI scaffolding or unrelated backreferences; the versioned URL remains byte-identical across deployments.

### P1. Small, deterministic search before semantic complexity

**Illustrative proposed API (not currently established):**

```text
GET /lex-mex/api/v1/search?q=autorizacion%20IFC&law=lritf&limit=5&format=json
GET /lex-mex/api/v1/articles/MX-FED-LRITF-ART-35?as_of=2026-10-24&format=md
GET /lex-mex/api/v1/citations?article=MX-FED-LRITF-ART-35&direction=out
```

Start with a small **SQLite Full-Text Search 5 (FTS5)** index or equivalent inverted lexical index. Normalize accent variants and capitalization; maintain editorial aliases (IFC, `institución de financiamiento colectivo`, Ley Fintech, LRITF, `Comité Interinstitucional`, SHCP/Banxico). Search on normalized article text, article ID, law aliases and headings; keep the result set small.

A search result should contain `article_id`, `law_id`, title, a **short query-matched excerpt**, applicable date metadata, release and source URL, citation URL, and a retrieval score. A second explicit request fetches the full article. Prioritize deterministic precision and version correctness over embedding-everything or retrieval-augmented-generation infrastructure.

**Acceptance:** The IFC test returns Articles 35, 39, 40 and 58 of LRITF and the relevant general-provisions Article 3 within a small number of ranked results. No unrelated laws, large documents or decorative HTML are sent by default.

### P1. Verifiable source/version/temporal model

Each provision should distinguish at least:

```yaml
article_id: MX-FED-LRITF-ART-35
instrument_id: MX-FED-LRITF
jurisdiction: MX-FED
source_type: primary_law
text_status: exact_official_consolidation_snapshot
publication_date: null            # null until independently verified
last_reform_date: null            # populate from verified official source
valid_from: null                 # do not infer from publication date
valid_to: null
verification_status: not_verified
last_successful_upstream_check: null
official_source_url: https://www.diputados.gob.mx/LeyesBiblio/pdf/LRITF.pdf
release_id: <release-hash>
content_sha256: <actual-sha256>
related_provisions:
  - MX-FED-LRITF-ART-39
  - MX-FED-LRITF-ART-40
```

The `null` values above are intentional schema examples, **not an assertion that those facts are unknown to the publisher**. Model enactment, publication, entry into force, deferred effectiveness, amendment and repeal as separate events. Where a consolidated official text does not resolve a transitional rule, answer with uncertainty and link the decree. Distinguish **retrieved at**, **last reformed**, **last checked upstream**, and **effective as of**. Never silently treat a recent retrieval as confirmation that the source is in force.

**Acceptance:** Querying `as_of=2024-10-24` and `as_of=2026-10-24` either returns verified date-applicable content or explicitly says historical applicability is not assured. It must not quietly serve the same “current” text as authoritative for both dates.

### P1. Citation and relationship graph optimized for multi-provision questions

The IFC question spans Articles 35, 39, 40, 58 and subordinate regulations. Build a lightweight edge list grounded only in **express** references, separating these from editorial/conceptual relations:

- `explicit_cross_reference`: legal text points to another provision.
- `implements` / `delegated_regulation`: verified normative relationship.
- `amends` / `repeals`: formal legal-change relationship.
- `editorial_related`: suggested retrieval convenience, **not legal authority**.

Expose `(source, relation, target, authority, release, evidence span)` and cap returned relationships (e.g., top 10 unless expanded). Do **not** have the engine assert that SHCP alone determines AML approval merely because a relation connects Article 35 to Article 58.

**Acceptance:** An agent can fetch one small related-provision list and then retrieve **only** applicable sections of relevant instruments, preserving legal provenance and evidence type.

### P1. Token-economy contract and telemetry

The target is **minimal legal evidence sufficient to answer**, not a maximally concise legal corpus that omits material conditions.

Set provisional **retrieval budgets** (to be calibrated, **not measured in this test**):

| Operation | Provisional input-budget target |
|---|---:|
| `llms.txt` discovery | ≤200 tokens |
| Search results (5 hits) | ≤350 tokens |
| One provision + concise metadata | ≤800 tokens, except naturally long provisions |
| Relationship list | ≤250 tokens |
| Standard five-provision IFC evidence bundle | ≤3,000 tokens |

Implementation measures:

1. Default to text/Markdown and short JSON; never send frontend framework scripts, navigation, entire statutes, 20k-item indexes or all backlinks for a narrow question.
2. Server-side `limit`, `fields`, `max_chars`, `format`, `include_relations=false` and explicit `include_full_text=true`. Enforce maximums and clear truncation indicators; **never truncate a legal provision without saying so**.
3. Keep exact text in stable, cacheable artifacts; expose `ETag`, `Last-Modified` where applicable, hashes and conditional requests. Only manifest pointers need frequent refresh.
4. Use deterministic lexical search first; optionally rerank only small candidate sets. Avoid model-based query expansion, summaries and embeddings as mandatory steps.
5. Log **request count, status, bytes downloaded, body tokens (using a pinned tokenizer), agent context tokens, latency and exact article relevance**, with safe query redaction. Separate server processing from model consumption.

Recommended metrics: `tokens_per_correct_cited_answer`, `requests_per_answer`, `irrelevant_provisions_per_answer`, `relevant_provisions_recalled`, `citation_precision`, `version_mismatch_rate`, and `external-client success rate`. A smaller token count with a wrong or uncitable legal conclusion is not an optimization win.

### P2. Optional structured outputs and MCP

Once static retrieval and search pass, offer an **optional**, thin answer-support service with structured `legal_claims` referencing immutable provision IDs and passages. Avoid representing generated synthesis as a normative source. An MCP facade could expose tools such as `search_provisions`, `get_provision`, `get_relations`, and `get_source_status`. MCP is **not a prerequisite** for interoperability; the public HTTP/Markdown surface should remain usable by clients that cannot call a custom tool.

## 6. Reproducible tests and acceptance gates

### 6.1 Transport smoke test (run on a networked runner, not this container)

```bash
BASE='https://www.leg4ll4bs.com/lex-mex'
RELEASE='3c04016651e2fd4169632756e96ec19136136e1f42c4ed666070f23f20b30660'

for PATH_SUFFIX in \
  'llms.txt' \
  "releases/$RELEASE/index.md" \
  "releases/$RELEASE/manifest.json"; do
  URL="$BASE/$PATH_SUFFIX"
  echo "=== $URL"
  curl -sS -L --max-time 20 -o /tmp/lex-mex-body \
    -w 'status=%{http_code} type=%{content_type} bytes=%{size_download} redirects=%{num_redirects} seconds=%{time_total}\n' \
    "$URL"
  head -c 200 /tmp/lex-mex-body; printf '\n'
done
```

Then fetch the **actual** LRITF article URLs discovered from the current index; do not assume the proposed routes above are deployed. Execute across a noncached client and a second public network. Record which client returns a real HTTP code and which supplies only a tool-internal error.

### 6.2 Legal golden-answer suite

| Test prompt | Required verified elements | Fail conditions |
|---|---|---|
| **G1. Which authorities authorize an IFC?** | CNBV issues; Committee prior agreement; CNBV/SHCP/Banxico each represented | Calling SHCP or Banxico a separate general IFC licensing office |
| **G2. Can CNBV authorize without SHCP or Banxico assent?** | No; one favorable representative from **each** authority, plus committee decision requirements (Art. 35) | Equating overall simple majority alone with sufficiency |
| **G3. What does each check?** | General Art. 39/Art. 3 dossier; distinguish legally assigned competence from practical emphasis | Claiming the LRITF formally partitions the application into three exclusive domains |
| **G4. Is authorization sufficient to begin operations?** | No; Art. 40 readiness at least 30 **business** days before commencement | Treating IFPE-specific joint inspection as mandatory for IFCs |
| **G5. Who regulates IFC AML/CFT requirements?** | SHCP general provisions subject to CNBV opinion under Art. 58; CNBV supervisory role | Naming UIF as a separate Art. 35 approving member |
| **G6. Historical applicability** | Appropriate amendment/effectiveness metadata or honest inability to resolve `as_of` | Presenting 2024 text as proven to be the 2026 version |

For each test, record retrieval URL(s), exact excerpts, independently reviewed answer, support/failure labels, measured tokens and latency. Establish the golden answer by a human legal reviewer. This is an **evaluation suite**, not training content that should be inserted into the authoritative statutory Markdown itself.

### 6.3 Quantitative experiment design

Run the same six questions in three conditions, on the **same model, same tokenizer, same temperature, same answer format and fixed budget**:

1. **Official web only:** searches + primary legal sources, no Lex-Mex access.
2. **Lex-Mex static retrieval:** start at `/lex-mex/llms.txt`, use article-level text, no external searches except official-source verification if needed.
3. **Lex-Mex search:** use the proposed bounded search endpoint plus article fetches.

Use at least five repeated runs per question and report median/p90 latency, model input/output tokens, retrieved body tokens, request failures, accuracy, citations and coverage. Count every failed fetch; do not compare a successful official-research answer against an incomplete Lex-Mex answer as though both were equivalent-quality outputs.

**Proposed success gate:** ≥99% cross-client retrieval success for published immutable assets; 100% of mandatory G1–G5 legal elements supported and cited; zero fabricated authority roles; 100% explicit temporal uncertainty when G6 cannot be verified; ≥30% reduction in median *retrieved evidence tokens* versus official-web-only at equal or better legal accuracy. These are **proposed targets**, not observed benchmarks.

## 7. Delivery roadmap

| Priority | Deliverable | Immediate definition of done |
|---|---|---|
| **Now — P0** | Release URL compatibility audit | Verified origin HTTP statuses and bodies from ≥3 clients; root cause identified or clearly scoped |
| **Now — P0** | Article-level static agent routes | Each published article independently retrievable, stable and linked from a small index |
| **Next — P1** | Tiny corpus manifest + alias-aware full-text search | The IFC query locates the correct articles without full-corpus retrieval |
| **Next — P1** | Provision provenance and temporal status fields | Date/official source/release/hashes exposed; unknown effective state represented honestly |
| **Next — P1** | Golden tests + token measurements | CI regression report for G1–G6, client reachability and normalized retrieval cost |
| **Later — P2** | Relation-graph and optional MCP | Proven advantage over static indexed retrieval, without changing source-of-truth semantics |

## 8. Agent-harness instructions to place in repository documentation

```md
### Lex-Mex source retrieval policy

1. Begin with /lex-mex/llms.txt. Resolve and pin its release ID.
2. Check corpus coverage and applicable dates before making a completeness claim.
3. Run a narrow indexed search; fetch exact articles, not full instruments.
4. Prefer statutory text to editorial abstracts. Preserve article/fraction and source/version IDs.
5. Distinguish binding law, regulator guidance, and practical inference.
6. Fetch expressly cited implementing regulations when an answer depends on them.
7. Do not interpret 'published' or 'retrieved' as 'currently effective'.
8. If an asset is inaccessible or absent, record that fact and use an official source;
   do not silently claim the legal conclusion came from Lex-Mex.
9. Cite exact provisions and official-source URLs. Never fabricate article links.
10. Record requests, bytes, tokens, latency and retrieval errors per test case.
```

## 9. Bottom line for LEGΔL LΔBS

**The competitive advantage is not a prettier legal reader or a novel agent protocol. It is a legally reliable, version-specific, article-addressable Mexican source layer that lets an agent get to the right provisions with few requests and small payloads.** Independent legal research succeeded because the controlling statute was easy to locate and inspect. Lex-Mex can outperform that workflow when it makes those same provisions *more* directly discoverable, temporally trustworthy, citation-ready and mechanically cheap to retrieve. The first condition is dependable cross-client access to the existing release artifacts.
