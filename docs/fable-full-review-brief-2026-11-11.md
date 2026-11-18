---
project: lex-mex
date: 2026-11-11
type: review-brief
status: for a fresh Fable 5.2 session after /clear
author: Claude (Opus 5.5), from repository state and the 2026-11-10/11 session
---

# Lex-Mex: brief for a full system review (Fable 5.2)

Purpose: give a fresh session enough to review the whole Lex-Mex system (corpus pipeline, database,
public reader, Worker/MCP agent surface, discoverability) and propose upgrades. The operator has
**frozen website/Worker changes** until this review. This brief is navigation, not authority:
`AGENTS.md`, `docs/project-status.md`, the task plans under `docs/plans/` and current Git state
outrank it. Verify before relying on any number here.

Context discipline (from `CLAUDE.md`): never bulk-read `corpus/` (479 MB, 468 instruments) or
`derived/db` (2.7 GB). Navigate index → `git grep` → one file.

## 1. Repository map

All six folders are **worktrees of one Git repo** (`origin` = github.com/R4m1r0qu41/Lex-Mex,
common dir `/Users/jr/Dev/lex-mex/.git`). Separately, the public website is another repo.

| Folder | Branch | State vs local `main` (`1ac6904bd`) |
|---|---|---|
| `/Users/jr/Dev/lex-mex` | `main` | clean; local main ahead of `origin/main` (unpushed history) |
| `/Users/jr/Dev/lex-mex-db` | `db/sqlite-projection` | merged into main; **9 dirty paths** (PLANS.md, sqlite plan, 7 untracked 2026-10-20 review docs) |
| `/Users/jr/Dev/lex-mex-integration` | `rule/bounded-commencement-2026-10-27` | **12 commits not on main**: bounded commencement rule (LFPA art. 28 business-day periods), applied to 5 instruments, 4 reverted |
| `/Users/jr/Dev/lex-mex-parser-repair` | `repair/itf-dcg-heading-separator-2026-10-25` | merged into main, clean |
| `/Users/jr/Dev/lex-mex-public-query-repair` | `repair/public-query-contract-2026-10-25` | merged into main, clean |
| `/Users/jr/Dev/lex-mex-reader` | `feat/public-reader-r7` | **11 commits not on main** (public query adapter over the DB, agent-test protocol and results, discoverability plan); **10 dirty paths**, incl. untracked `reader/retrieval-check.mjs` + test, which the website's live verifier imports |

Other branches: `backport-discoverability-pages` (2 commits, syncs `reader/` with the deployed website
Worker modules; unmerged), `codex/lritf-reference-links` (1 commit, July), several merged/backup
branches, four OCR experiment worktrees under `/Users/jr/Dev/experiments/...`.

Website repo: `/Users/jr/Documents/4ll4` (`Webpage/LEG4L_L4BS`), Cloudflare Worker `leg4ll4bs-web`.
`main` there is `d56372c` (includes held SEO commit `3dd532d`, **not deployed**). Production runs local,
unpushed branch `discoverability-agent-pages` head `1ddfde5` as Worker version `aa207c17`.
Rollback to pre-discoverability: `wrangler rollback eb73a863-5694-4260-ad78-ba912955b7cc`.

## 2. What has been built

**Pipeline (Rust, 9 crates):** `lex-source` (acquisition, hashing), `lex-parse` (+ `lex-grammar`,
`lex-link`, `lex-analysis` after the parser split), `lex-core` (types, temporal effects, review
state), `lex-export` (JSON/Markdown/Obsidian), `lex-db` (SQLite projection), `lex-cli`.
Rust owns normalization, validation, review-state changes and publication; model output
(temporal analysis) is a schema-gated proposal (`schemas/temporal-model-output-v2.schema.json`).

**Corpus (committed canonical data):** 468 federal instruments (452 of 453 current Cámara laws and
regulations; `egdf` excluded by operator; CNBV DCGs; plus 32 NOMs as a separate boundary).
About 64,859 articles, 3,506 transitories, 30 annexes, about 36,077 reference edges (closure gate:
2 accepted gaps, `lftr`, `socap-sofipo-dcg-2006`). Each instrument: source PDF hash, parser
baselines, canonical JSON, validation report, Markdown.

**Temporal / review state:** audited temporal analysis covers only LRITF, `ifpe-dcg-2021`,
`itf-dcg-2018`. Deterministic `derive-temporal` resolved commencement for many instruments (74
skipped, queued). Most provisions are `not_analyzed` with `temporal_status: unknown`; about
20,000 `machine_accepted` (deterministic); 2 `lawyer_verified`. JRH is the legal reviewer of record.

**SQLite projection (`lex-db`):** full-corpus database builds exist locally (`derived/db`, git-ignored,
A/B reproducibility builds; release `caead5ed…`: 68,395 provisions, FTS, terms). Plan
`docs/plans/sqlite-corpus-database.md`: M2 accepted; M3 repairs pending re-review; M4
reproducibility documented as independently accepted on the db branch (`047b421ef`); confirm.
**The full database is not served publicly.**

**Public reader (four-instrument release):** signed static publication (release `c305db4a…`:
LRITF, LIC, DCG-ITF-2018, DCG-IFPE-2021; 870 provisions) with retained official PDFs and
SHA-256 manifests, JS reader at `/lex-mex/index.html`. Producer code in `reader/` (package,
stage, reader, retrieval-check, RAG modules).

**Agent surface on the website Worker** (`worker/lex-mex/*.mjs`, memory adapter over a projection
of the four-instrument release):
- HTTP: `/lex-mex/search` (literal whole-word phrase match), stable article pages
  `/lex-mex/mx/<slug>/articulo|transitorio|anexo/<n>` (+ `.md` twins, `/referencias`), contents
  hubs `/lex-mex/mx/` and `/lex-mex/mx/<slug>/`, sitemap index + per-instrument sitemaps,
  `/lex-mex/llms.txt`, `/lex-mex/current.json`, coverage page; pinned `/lex-mex/r/<release>/…`
  addresses (retired releases 404 with a pointer, never a silent redirect).
- MCP (Streamable HTTP, POST-only, read-only): `search`, `get_provision`, `get_references`,
  `get_coverage`; tool titles; optional `release_id`; `limit` 1–10.
- Rate limits: 120/min per IP, 600/min shared (per Cloudflare location). Workers Paid plan.
- RAG answer path modules exist in `reader/` (`rag-*.mjs`, Q04/Q05 proofs 2026-11-03); not part
  of the deployed Worker.

**Verification tooling:** Rust native gate (fmt, strict clippy, about 447 tests, validators);
reader tests (`node --test reader/`: 90 tests, 71 pass, 19 real-projection-gated skips); website
`npm test` (17 tests incl. build audit); live verifier `scripts/verify-cloudflare-live.mjs`
(7,754 checks, passed on `aa207c17`); fresh-agent protocol `batch-v1` and organic check
(`lex-mex-reader/docs/Agent_Tests/README.md`).

## 3. What has been tried (condensed timeline)

- Jul–Aug 2026: LRITF/Fintech vertical slice; Diputados bulk; IFPE DCG; 242 instruments by 08-24.
- Sep–Oct 15: cluster-2 federal structural first pass; bulk admission to 467/468.
- Oct 16–19: corpus-wide reverse linking, parent-law mapping, glossary detection, merged-heading split.
- Oct 20–27: parser responsibility split (four crates), post-split corpus repair, public-query
  contract repair, SQLite projection M1–M4, DuckDB spike (declined a columnar store), bounded
  commencement rule (unmerged).
- Oct 24: R7 four-instrument reader design release; mLey surface benchmark (10-27).
- Oct 28–Nov 3: agent-first plan: runtime proof questions Q01–Q10, RAG evidence path, sitemap,
  MCP pilot; production rollout of the MCP pilot (Free plan), later moved to Workers Paid.
- Nov 7: production readiness review: functional pilot useful, readiness not established
  (CPU 0–63 ms vs Free 10 ms, quota unverified, producer/host module drift, PDF 403 for some
  clients, IFPE art. 7 duplicated phrase); fresh-agent mLey baseline.
- Nov 9–10: multi-provider batch-v1 runs; organic discovery run: **Lex-Mex never found** (mLey,
  Diputados, Corpus Iuris, Ansvar found). Discoverability plan + SEO audit.
- Nov 10–11 (M1 discoverability, deployed): stable release-free URLs, crawlable hubs, Markdown
  twins, sitemaps, neutral provenance wording ("extraído del PDF oficial", no "íntegro"), Spanish
  status labels (`vigente`, `revisada de forma automatizada`), home link to `/lex-mex/mx/`,
  "Ley Fintech" alternate name, weak-ETag fix, out-of-collection 404, MCP `limit`.
  Codex audited the branch and deployed code (findings fixed; `50594ba..1ddfde5` not Codex-reviewed).

## 4. What works (with evidence)

- Deterministic pipeline and canonical corpus with provenance; native gate and validators pass on main.
- Four-instrument agent surface: batch-v1 browse run 2026-11-11 answered 7 of 8 from retrieved
  text (Q05 partial by nature, Q10 correct refusal); Q04 in 3 requests (6 before M1); reference
  chain art. 2 → 7 → 29 followed via `/referencias`. Live verifier 7,754/7,754.
- Integrity story that competitors lack: retained PDFs, source and text SHA-256, release identity,
  explicit "no acredita vigencia". The organic agent recommended mLey while noting mLey's own
  "correspondencia de fuentes no comprobada", "SHA-256: Pendiente" and draft status.

## 5. What does not work / open issues

**Discoverability (the stated top goal):** organic agents do not find Lex-Mex. New stable pages are
hours old and unindexed; Search Console/Bing registration not done; no registry/directory listings;
the brand "Lex-Mex" collides with an unrelated paid product ("LexMex", lex-mex.xyz, 404).

**Search quality:** literal phrase matching with no stemming ("autorizar" misses "autorización");
ranking buries core articles for generic words; topic/heading labels would need the producer
(only 7 of 870 provisions contain a heading line). BM25/early-occurrence experiments did not fix it.

**Data / producer defects** (need a new signed release, not Worker edits):
- DCG-IFPE art. 7 fr. I duplicated phrase (source vs extraction defect unclassified);
- LRITF art. 22 fractions II/III merged in one paragraph;
- `source_recorded_reform_date` null for the DCGs;
- references resolve to whole articles, not fractions/paragraphs; no reference purpose;
- signed reader page uses relative `./llms.txt`; signed agents sitemap lists 870 pinned URLs;
- text is extracted from PDFs but **never compared back** to the PDF (no fidelity check);
- no daily DOF/Diputados change monitor (operator wants one, in Rust, using existing hashes).

**Scale / architecture:**
- Worker uses an in-memory projection of 870 provisions; the full corpus (68k provisions, DB built)
  has no serving path. Several Worker checks call `catalog({limit:10})`, so past ten instruments a
  covered instrument would be reported as outside the collection.
- Cold LIC hub costs about 108 ms CPU (Node) on first request; edge caching of Worker output is not
  configured or verified.
- Hub JSON-LD uses short name as `legislationIdentifier`; articles use the URN.

**Process / ops hygiene:**
- Production runs an unpushed website branch; `3dd532d` (SEO, keyword removal) held and will
  conflict in `public/sitemap.xml` and tests on merge; site still emits the keywords meta.
- **At least three copies of the Worker modules** drift: website `worker/lex-mex/`, lex-mex `reader/`
  on main (stale), `backport-discoverability-pages`, and `feat/public-reader-r7`.
- The live verifier depends on untracked `reader/retrieval-check.mjs` in the lex-mex-reader worktree.
- Unmerged branches: bounded commencement (12), reader r7 (11), backport (2); dirty db and reader worktrees.
- Model-routing misses (mechanical runs not delegated to `mech`) recurred.
- Unsigned reader label live: `historical_superseded_reference_outdated` shown as
  "referencia histórica a un texto sustituido" (pending operator sign-off).

## 6. Constraints the review must respect

- `AGENTS.md` rules; Rust owns normalization/validation/publication; models propose only.
- JRH is legal reviewer of record; never infer or simulate legal approval; never present output as
  official law or legal advice; no "íntegro"/fidelity claims until a PDF comparison exists.
- No deploy, push, merge to main or history rewrite without the operator's explicit go; each deploy
  is its own go. Codex is started only by the operator.
- Name stays "Lex-Mex" for now (any rename later, inside leg4ll4bs.com, no new domain).
- Hold the full corpus from the public surface until search is stable on the four-instrument set
  (operator, 2026-11-10); revisit in this review.

## 7. What the operator asked for

A full review "to check if we can improve the system all together": data and producer issues first,
then how the pipeline, database, reader and agent surface should fit. Suggested questions:

1. One source of truth for the serving code: how to end the module copies (package, submodule, or
   producer-built artifact deployed by the website).
2. A fidelity check: compare parsed text back to the PDF (per provision), classify defects
   (source vs extraction), and what a "verified" label could honestly mean.
3. Producer upgrades for search: headings/topic labels, morphology (stemming or variant
   suggestions), fraction-level reference resolution.
4. Path from 4 to 468 instruments: serving the SQLite projection (D1/R2/static shards), CPU and
   quota, the `limit:10` assumptions, signed-release size.
5. Change monitoring: daily DOF + Diputados check against stored hashes; proportional releases.
6. Discoverability: Search Console/Bing, registries, Anthropic Connectors Directory (needs a real
   custom-connector test), full SEO upgrade for leg4ll4bs.com and Lex-Mex.
7. Branch and worktree consolidation: what to merge, retire or rebase (integration, reader-r7,
   backport, db dirty docs).

## 8. Where to read (bounded)

- `AGENTS.md`, `CLAUDE.md`, `PLANS.md` (plan index), `docs/project-status.md` (top 60 lines).
- `docs/plans/agent-first-delivery-and-change-tolerant-releases.md` (checkpoint section, line ~270).
- `docs/plans/sqlite-corpus-database.md` (current checkpoint, line ~485).
- `docs/production-readiness-review-2026-11-07.md`.
- In `/Users/jr/Dev/lex-mex-reader`: `docs/plans/discoverability-and-instrumentation-plan-2026-11-10.md`
  (sections 5–6), `docs/plans/seo-audit-2026-11-10.md`, `docs/Agent_Tests/README.md` and the
  2026-11-11 results.
- Website: `/Users/jr/Documents/4ll4/Webpage/LEG4L_L4BS/AGENTS.md`, `HANDOFF.md`,
  `DISCOVERABILITY-CODEX-AUDIT-2026-11-11.md` (on the branch), `worker/lex-mex/query-http.mjs`.
