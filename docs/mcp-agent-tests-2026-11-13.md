# MCP agent tests, local database and live site (2026-11-13)

Two fresh agents with no project context answered the same six Spanish questions. Each was
given only an endpoint and a traced MCP client; expected answers were withheld. Pass criteria
were fixed before the run. Judged from the call traces (local 50 calls, online 18 calls).

## What was tested

| Arm | Endpoint | Data | Code |
|---|---|---|---|
| Local | `reader/mcp-http.mjs` served on 127.0.0.1 over `reader/query-sqlite.mjs` | SQLite projection of the whole corpus (468 instruments), `db project` at corpus and builder commit `aed3cfe17`, release `d035ffd0…` | this branch |
| Online | `https://www.leg4ll4bs.com/lex-mex/mcp` and the `llms.txt` HTTP route | published release `c305db4a…` (4 instruments, corpus commit `d2ab64d6`) | the host's deployed code, not this branch |

## Results

| Question | Local | Online |
|---|---|---|
| 1. ITF authorization (search, no article named) | pass: LRITF art. 11, CNBV previo acuerdo del Comité Interinstitucional | pass (both routes) |
| 2. Banca y crédito | pass: LIC art. 2 | pass (both routes) |
| 3. LRITF entry into force (transitory) | pass: transitory `primera` | pass (both routes) |
| 4. LIC art. 194 references | pass: 27 edges over 6 pages; LFCE "31 bis" marked `historical_superseded_reference_outdated`; no reviewer name or rationale in any response | pass, same edges and order on both routes |
| 5. Código Civil para el Distrito Federal (not in collection) | pass: says not in the collection, no claim of legal absence | pass |
| 6. CPEUM art. 82 | pass: quotes the reformed nationality text, flags status `unknown` | pass: CPEUM not in the release, declines |

No answer came from memory. No private reviewer data appeared in either trace.

## Findings (non-legal; for the agents to fix)

1. **Instrument discovery.** There is no tool or route that lists instruments with their slugs
   over MCP (`get_coverage` returns only a notice; the online `/coverage` and `llms.txt` list the
   four). Searching by an instrument's title finds laws that cite it, not the instrument, so the
   local agent wrongly concluded LFCE was absent (it is in the collection as `lfce`) and had to
   guess `cpeum`. Proposed: list instruments in `get_coverage`, and match titles and short names in
   search.
2. **`release_id`.** This branch's MCP requires it on every call; the live host says it is
   optional. Keep one contract (repository vs. deployed host diverge).
3. **Unknown `instrument` in search** returns an empty result rather than an error, so a typo and
   a missing instrument look the same.
4. **Numbers.** Transitories use the printed ordinal's gender (`primera`, not `primero`);
   `llms.txt` does not say how numbers are formed. 68 provisions with identifiers not usable in
   URLs (ordinal signs) are not served at all.
5. **References.** Five per page with no `limit`; ranges are expanded one edge per article
   ("199 al 215" gives 17 items). `/referencias.md` is 404, so references exist only as HTML.
6. **Reviewer notes are not published.** `provision_notes` (the CPEUM 2028 note) is absent from the
   public DTO and the release, by allowlist: safe (no reviewer data leaks) but the note does not
   reach readers or agents. Publishing CPEUM needs the note carried without reviewer or rationale.
7. Search ranking favors repetition (LRITF art. 11 ranks below arts. 66–67 or 12, 103, 110).

Traces and the client are scratch artifacts, not committed.
