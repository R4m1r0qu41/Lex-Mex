# Lex-Mex retest: corpus route vs no corpus

Date: 24 Oct 2026. Model: Claude Opus 5.5, claude.ai Project session (WebFetch, WebSearch, sandboxed shell).
Question: "What are the requirements you must demonstrate in terms of account separation policies of an IFPE?"
Release tested: `c305db4a…c969`, corpus commit `d2ab64d`, committed 2026-10-24 13:49 (-06:00).

## 1. Verdict

The changes did not reach the agent-facing surface, as far as an agent can observe. Indexes, provision files, `llms.txt` structure and the manifest behave as in the 14 Oct release, and the manifest's `builder_sha256` (`791b90e6…`) is identical to the previous release. Check whether the backend change was deployed into the release output, or whether it exposes a new endpoint that `llms.txt` does not advertise.

On this question, the no-corpus baseline gave the better answer for fewer tokens. It found CNBV's IFPE authorization guide, which itemizes what the applicant must demonstrate; the corpus does not hold that guide. The corpus still has one decisive advantage: cnbv.gob.mx blocks the fetch tool, so the 2018 CNBV disposiciones were reachable only through Lex-Mex.

## 2. Measurements

| Route | Calls | Failed | Tokens returned to model (est.) | Found the operative answer source |
|---|---|---|---|---|
| Round 1, old release (14 Oct) | 45 fetches | 0 | 90 to 110K | Statute only |
| Round 2, Lex-Mex route | 12 fetches, 1 shell | 1 (shell) | about 9 to 11K | Statute plus CUITF art. 3; not the guide |
| Round 2, baseline, no corpus | 4 searches, 5 fetches | 2 (CNBV PDF robots; 2018 guide URL) | about 5 to 7K | Yes: Guía IFPE v3.0, Mar 2021, p. 30 |

Token figures are rough, based on the size of tool outputs. The round 2 Lex-Mex route is contaminated: I already knew from round 1 that LRITF arts. 39 and 46 were the targets, so I skipped the discovery a cold agent would pay. Most of the drop from round 1 comes from that knowledge, from not loading the manifest, and from targeted prompts. It does not come from the new release.

Note on cost: through WebFetch, what the model pays is the summarizer's output, not the file's size. Page size drives tool latency; context cost is driven by what the agent asks for.

## 3. Status of the 14 Oct fixes

| ID | Item | Status |
|---|---|---|
| F1 | Static or noscript fallback on the portal | Not fixed. Still "Preparando el corpus…". The agent link is now labeled "Para agentes e IA · llms.txt" (improvement). |
| F2 | Shell access | Not fixed. `connect_rejected` by the session egress allowlist. This is a claude.ai settings item (Settings > Capabilities), not a site defect. |
| F3 | Deterministic article URLs | Not fixed. Same hash filenames. |
| F4 | Headings, summaries, tags in indexes | Not fixed. Flat "Artículo N" lists. |
| F5 | Search endpoint | Not fixed or not advertised. |
| F6 | Monolithic manifest | Not fixed. Still truncates mid-list. |
| F7 | Header overhead, `.md` vs `.txt` guidance | Not fixed. |
| F8 | Last-reform dates | Not fixed. |
| F9 | Delegations and gaps metadata | Not fixed. Art. 46 still carries no references. |
| F10 | Prev/next links | Not fixed. |
| F11 | Full-instrument download in `llms.txt` | Not fixed. |

New finding (tool side, not site side). WebFetch now applies its short-quote cap to Lex-Mex files too: LRITF art. 46 came back summarized in both `.md` and `.txt`. In round 1 the same file came back verbatim. Through this tool, exact text is no longer a corpus advantage. The fix is a delivery channel that does not pass through a summarizer, see section 5.

## 4. Answer to the test question (from both routes combined)

Facts:
- LRITF art. 39 fr. IV: the authorization application must include "las políticas de separación de cuentas, en términos de lo establecido en el artículo 46".
- CUITF (2018 CNBV disposiciones) art. 3 fr. V requires draft operating, internal control and risk management manuals. It does not itemize the separation policy.
- The CNBV IFPE authorization guide (v3.0, March 2021, Sección II, fr. V, numeral 1.2, p. 30) places the policy inside the operating manual and requires the applicant to explain: (a) how own resources stay segregated from client resources; (b) how client resources are identified per client; (c) the control mechanisms during and at the end of the day; (d) whether funds will be used in repos and, if so, only with Federal Government or Banco de México securities with credit institutions at one-day renewable terms, or through an administration trust that only does those repos.
- LRITF art. 46 sets the standards the policy implements: segregation and per-client identification; end-of-day deposit in demand-deposit accounts in the institution's name at an authorized entity, distinct from own-funds accounts (or the repo and trust alternatives); and for IFPEs a cap of the greater of 1,000,000 UDIs and twice the largest 24-hour redemption in the prior 365 days.
- LRITF art. 40: before starting operations (at least 30 business days ahead), the IFPE accredits its controls, policies and manuals, with CNBV and Banco de México inspection visits.

Interpretation: as summarized by the fetch tool, the guide's section does not mention the art. 46 cap, the art. 45 rails or the art. 47 per-holder registry and insolvency separation right. A well-built policy should still address all three, because CNBV and Banco de México will test against the statute, not only the guide.

To verify: the guide was read through a summarizing fetch. Check p. 30 directly before citing it. The guide calls itself "una herramienta de apoyo", but the policy items are worded as mandatory.

## 5. Recommendations

1. Confirm the deploy. The builder hash did not change, so the agent-facing output looks unchanged.
2. Add the CNBV authorization guides (IFPE and IFC) as a "guidance" instrument class. On this question the guide was the operative source, and the corpus did not have it.
3. Ship headings or topic tags in the indexes (F4). That alone would have pointed a cold agent from "separación de cuentas" to LRITF arts. 39 and 45 to 47, CUITF art. 3 and the guide.
4. Expose Lex-Mex as an MCP server (connector). A connector returns tool output directly, so it bypasses both the summarizer's quote cap and the sandbox egress allowlist. That restores verbatim delivery and makes shell-free agents first-class.
5. Keep CNBV texts in the corpus. cnbv.gob.mx blocks agent fetches, so Lex-Mex is the only route to them here.
