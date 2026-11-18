# Lex-Mex external agent tests

Where results of external AI-client tests are kept. One file per provider run.

Protocol id: `batch-v1` (eight questions pasted in one message, with a built-in record template).
This differs from the earlier protocol of one question per message
(`docs/runtime-proof-test-pack-2026-10-28.md` in the lex-mex repo) because the operator has
limited time. Label every result with its protocol id; compare only runs with the same id and release.

## How to run

1. Open a fresh conversation in the provider's app. No prior Lex-Mex context.
2. Choose a mode and set it up before pasting:
   - `mcp`: add the custom remote MCP server `https://www.leg4ll4bs.com/lex-mex/mcp` (Streamable HTTP, no login).
   - `browse`: no connector; the client's own web access only.
   - `none`: no web, no connector (control; expect honest refusals, not invented law).
3. Paste the prompt below, unedited, as one message.
4. Save the complete reply (and, if the client shows them, the raw tool calls) as
   `YYYY-MM-DD_<provider>_<client>_<mode>.md` in this folder. Examples:
   `2026-11-05_claude_claude-ai_mcp.md`, `2026-11-05_openai_chatgpt_browse.md`.
5. If the agent cannot write files, paste its reply into the file yourself. Do not edit the reply.
6. A failed question may get one follow-up. Record it under a `## Follow-up` heading; never overwrite the baseline answer.

Pre-protocol results (chat sessions, 2026-10-29 to 2026-11-03, different release): the four files already in
this folder (Claude, GPT, Grok, Gemini). Keep them as they are.

## Prompt to paste

```text
You are testing a public legal-research service called Lex-Mex: https://www.leg4ll4bs.com/lex-mex/
(MCP endpoint, if your client has it connected: https://www.leg4ll4bs.com/lex-mex/mcp).
Answer the eight questions below using only what the service returns. Do not answer from memory;
if you cannot reach the service or the evidence is not there, say so instead of filling the gap.
Do not describe your hidden reasoning. Report only actions you actually took.

Questions (keep these IDs):
Q01. What is in this collection, and what is outside it?
Q02. Who authorizes an ITF (institución de tecnología financiera), and does the government guarantee customer money?
Q03. Quote the exact first paragraph of article 2 of the DCG for IFPE (2021) and say what consent it requires.
Q04. Find where the law requires an ITF to segregate customer money from its own resources.
Q05. Must every IFPE client have an individual bank account? Explain with the articles you relied on, and say what the texts do not settle.
Q06. Follow the reference from article 2 of the DCG for IFPE to article 7: what authentication is required and what evidence must be kept?
Q09. Which source and version supports your answer to Q02, and is it verified as current today?
Q10. Fetch article 27 of the Ley para Regular las Sociedades de Información Crediticia.

Reply in this exact structure, in Markdown:

# Run header
- Provider and client/app (with version if you know it):
- Mode (mcp / browse / none) and how you reached Lex-Mex:
- Release id you saw (state "none" if you saw none):
- Date/time of this run:

# Results
For each question Qxx:
## Qxx
- Answer: (concise; cite stable IDs such as urn:lex-mx:... and official source links; label quotes as quotes)
- Path taken: (numbered list, in order, of every tool call or URL you used; for each give what you asked for and what came back, including result size or "empty")
- Failures and friction: (errors, truncated or empty results, wrong instrument returned, retries, calls that were wasted, anything you could not do; write "none" if none)
- Evidence status: (supported by retrieved text / partly supported / not retrieved, answered from memory / not answerable)

# Summary for the maintainers
- Which questions you could not complete, and the layer that failed (client could not connect, tool missing, search returned nothing useful, text too long or truncated, wrong result, other):
- The three changes that would have made your task easiest (to the tool names, arguments, responses, pages or documentation):
- Anything in a response that looked wrong, inconsistent or misleading:
```

## Optional: organic-discovery check (separate, fresh session)

Do not mention the domain, the site name or the connector. Paste only:

```text
Use your web access to find a public source that lets AI agents read the text of Mexican federal financial regulation
(for example the law on financial technology institutions) with article-level citations. Say which sources you found,
how you found them, and which one you would rely on and why. List every search or URL you used, in order.
```

Record it as `YYYY-MM-DD_<provider>_<client>_organic.md`. A pass means Lex-Mex appears without being named;
absence is a finding about discoverability, not a failed run.
