# batch-v1 first round: analysis and execution plan (2026-11-09)

Protocol `batch-v1`, release `c305db4a…354c969` (live, verified 2026-11-09 by direct probe).
Plan drafted with the advisor (Opus tier; shares the session's framing, not an independent verifier).

## 1. What the four runs were

All four runs were `browse` mode with no connector attached. **No run exercised MCP**, so
"none could connect" means "none had the connector installed", not "the handshake failed".

| File | Client | Release seen | Result |
|---|---|---|---|
| `lex-mex-results.md` | Grok (xAI), open_page + browser | c305db4a (current) | 7 of 8 fully supported; Q10 correct "not in collection". Q04 found only after trial fetches of LRITF arts 22-30 |
| `lex-mex-test-run.md` | Claude Sonnet 5.5, WebFetch | c305db4a (current) | Q01, Q02, Q09 complete; Q03-Q06 partial (WebFetch returns summaries, refuses verbatim); Q04 not found after about 16 blind fetches; Q10 correct |
| `lex_mex_test_report.md` | Gemini, google:search snippets | c305db4a (current) | Q02, Q06, Q09 supported; Q03 partial; Q04, Q05, Q10 skipped (tool-call limit). Cites LRITF art 35 for the guarantee, which disagrees with art 11 in every other run |
| `Lex-Mex_Public_Service_Test_Report_2026-11-09.md` | ChatGPT (GPT-6), browse | **3c040166 (stale, retired 2026-10-22)** | 0 of 8 substantive. Stale cached `llms.txt`; its release paths now return 404; sandbox had no DNS |

Compared with the 3 Nov chat runs (pre-protocol, different client mode): Claude and Grok then got
verbatim text with direct fetch and answered essentially all questions. Gemini then and now relied on
search snippets and cited art 35. ChatGPT saw the same stale `3c040166` release on 28 Oct and again
today, so it has not refreshed its view in 12 days. The service did not regress; client capability
and discoverability explain the differences.

## 2. Verified facts (live probes, 2026-11-09 evening)

Server is healthy:
- `POST /lex-mex/mcp`: initialize 200 for protocol versions 2024-11-05, 2025-03-26, 2025-11-25 (and
  negotiates an unknown version down to 2025-11-25); `notifications/initialized` 202;
  `tools/list` returns search, get_provision, get_references, get_coverage; `tools/call get_provision`
  for LRITF art 46 returns the full record. `GET /mcp` is 405 with `Allow: POST` (expected for a
  POST-only server).
- `/lex-mex/r/<rel>/search?q=segregados` returns LRITF art 46 first. Search and article routes work.
- No user-agent filtering of consequence: ChatGPT-User, OpenAI-MCP, Claude-User, Anthropic-MCP,
  Google-Extended, node, curl all get 200. Only `Python-urllib` gets a Cloudflare 403 (even with
  JSON headers). OAuth discovery paths return 404 (correct for an authless server).
- Browser `Origin` headers (claude.ai, chatgpt.com) get 403 "Forbidden origin" (`worker/lex-mex/mcp-http.mjs:8`,
  DNS-rebinding protection). Hosted connectors call server-side and normally send no Origin.
  This matters only for in-browser MCP clients. Unverified whether any hosted connector sends one.

Findings in the service (these are the real causes):
1. **Discovery gap.** `/lex-mex/llms.txt` (the file agents land on) lists only the release index and
   manifest, with relative links. It does not mention `/lex-mex/agents/`, the search URL,
   `current.json`, the URN-to-URL rule or the MCP endpoint. The full instructions exist only in the
   root `/llms.txt`, which none of the runs read. Effect: Claude and Grok probed articles by number;
   Gemini and ChatGPT never reached the text.
2. **Relative links** in `llms.txt` produced "invalid argument" errors in ChatGPT's link follower.
3. **Client-rendered landing page.** Raw HTML has no links and shows "Preparando el corpus…".
   Text-only clients (ChatGPT, Gemini) see nothing; Grok worked only because it drove a browser.
4. **Retired release paths return a bare 404** (`/releases/3c040166…/index.md`), giving a stale-cache
   client no way back to the current release.
5. The reader's on-page search box returned unfiltered text for Grok (UI defect, not yet reproduced).

Not service faults: ChatGPT DNS failures and Gemini's tool-call limit; Claude's blocked `curl` and the
WebFetch summarizer (client behaviour; MCP or short JSON pages sidestep it); Claude's Q10 answer was
correct. The phrase "El Identificador de Cliente El Identificador de Cliente y" in DCG-IFPE art 7 is
present in the official PDF (retained text line 621): faithful transcription, **not a parser defect,
no fixture needed**. Gemini's art 35 citation is a client error as far as checked (art 35 opens with
the authorization process; I did not read all of it).

## 3. Execution plan

Phase 0 (read-only; mostly done above). Remaining operator items:
- 0.1 Operator: for each app, was the connector added before pasting, and what was the exact error?
  This decides whether MCP is a setup step or needs a fix.
- 0.2 Operator: Cloudflare dashboard, Security Events and Worker logs for `/lex-mex/*`, about
  23:59 UTC 9 Nov (ChatGPT) and 03:10-03:40 UTC 10 Nov (Gemini, Claude). No ChatGPT requests would
  confirm it answered from cache.

Phase 1 (fixes; each deploy needs operator authorization; none made yet):
- 1.1 Put the agent entry at the top of `/lex-mex/llms.txt` with absolute URLs: `current.json`,
  search URL template, article URL template, `/lex-mex/agents/`, MCP URL with tool names. First find
  which layer produces the file: `reader/package.mjs` copies `llms.txt` into the hashed publication, so
  a change alters the publication hash; the alternative is a Worker response override. Needs a
  design choice.
- 1.2 Add plain static links (agents page, llms.txt, catalog) to `/lex-mex/` raw HTML.
- 1.3 MCP: no fault proven. Do not relax the Origin check on speculation; decide only if a real client
  needs it.
- 1.4 Retired release paths: operator picks a pointer page or redirect to `/lex-mex/agents/`. Never
  silently map an old release to new content.
- 1.5 Out-of-corpus requests: confirm what an unknown instrument returns and make it an explicit
  "not in collection" listing the four slugs.
- 1.6 Reproduce and fix the reader search box.

Phase 2 (retest):
- Gate: reach LRITF art 46 from `/lex-mex/` by HTML links only, using the web-fetch agent as an
  outside client.
- Keep the `batch-v1` prompt unchanged so rounds stay comparable.
- README: add an `mcp` precondition (the four tools must be visible before pasting; otherwise save the
  connect error as the result). Check which apps accept a custom remote MCP server.
- A real MCP round needs a client that can add one: Claude Code
  (`claude mcp add --transport http lex-mex https://www.leg4ll4bs.com/lex-mex/mcp`), Claude.ai
  custom connector, ChatGPT developer-mode connector (Plus/Pro). Gemini's consumer app probably cannot.
  Run the prompt in a fresh session with the connector on.
