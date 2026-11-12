# Four-goal system upgrade: fidelity, navigation database, human reading surfaces, discoverability

Task-named execution plan per `PLANS.md`. `AGENTS.md`, current Git state and accepted decisions
outrank this file. Input brief: [fable-full-review-brief-2026-11-11](../fable-full-review-brief-2026-11-11.md).

## Purpose and observable outcome

The operator asked (2026-11-11) for the system to reach four outcomes:

1. **Discoverability at the mLey level.** An agent with only web access, not told about Lex-Mex,
   finds and uses it.
2. **A robust database for agents** to navigate the whole corpus, not only four instruments.
3. **Robust provenance.** The published Markdown text is shown to match the official PDF, so an
   agent or a lawyer can trust it.
4. **Working links for humans.** Links in the Markdown work locally in the Obsidian vault, and a
   similar reading interface exists online.

Order of work: 3 → 4 (local) → 2 → 1. Goal 3 is first because it is what Lex-Mex has and
competitors lack, because goals 1 and 2 publish more text and should not publish unverified text,
and because it needs no website change (the website is frozen).

## Scope and exclusions

In scope: the Rust pipeline, schemas, fixtures, the committed corpus (only through the pipeline),
the generated part of the Obsidian vault, plans and recommendations for the website Worker.
Out of scope without a separate operator go: any deploy, push, merge to `main`, history rewrite,
website or Worker edit (frozen 2026-11-11), temporal or legal review decisions (JRH), starting Codex.

## Initial baseline (2026-11-11, `main` at `1ac6904bd`, immutable)

Measured in this session, read-only:

- Corpus: 468 instruments with `instrument.json`; 32 further NOM directories.
- Repository Markdown: 68,864 files, 156,446 relative links (10,393 cross-instrument), **0 broken**.
- Obsidian vault `~/Vaults/Lex-Mex`: last export 2026-08-24; 234 folders; **262 of 468
  instruments missing**; 123,698 wikilinks, 27 broken (all `LGOAAC`, case of `95-Bis`).
- Retained sources in `.work/`: 355 of 468 instruments have the source file; 353 match the
  manifest `source_sha256` and `extracted_text_sha256`; 2 mismatch (`cub-dcg-2005`, `lfiiedb`);
  113 have no retained source on `main`.
- No check compares provision text back to the source (brief, section 5).
- Prototype fidelity check (Python, scratch only) over the 353 verifiable instruments, 53,016
  provisions: 43,360 exact; 9,300 exact once repeated page headers, Cámara editorial reform notes
  and CNBV amendment marks are set aside; 356 need triage (0.7%). LRITF: 156 of 156 pass.
- Search engines: a site-restricted search for `leg4ll4bs.com` returns nothing; mLey ranks for
  article-level queries with URLs of the form `mley.mx/LRITF/articulo/7/`.
- Full-corpus SQLite projection exists locally (`derived/db`, 461 MB per release, FTS included);
  it is not served. The public surface serves four instruments (870 provisions) from memory.

## Discoveries and decisions

- **D-1 (2026-11-11). DCG-IFPE art. 7 fr. I is a source defect, not an extraction defect.** The
  official CNBV PDF, page 13, prints "El Identificador de Cliente El Identificador de Cliente y"
  (rendered page inspected). Lex-Mex reproduces the source faithfully. Do not "correct" it; a
  source-anomaly annotation is the right treatment (needs a small schema decision, milestone F4).
- **D-2 (2026-11-11). LRITF art. 22 fr. II/III is one instance of a systemic parser defect.** When
  a page break falls between two fractions, the parser joins them into one paragraph. Counted:
  2,376 such paragraphs in 319 instruments; in the 244 instruments with retained text, 1,937 of
  1,941 candidates sit on their own line in the PDF and 1,880 directly follow a page break. Words
  are intact; paragraph structure is wrong. Repair is milestone P1.
- **D-3. What a "verified" label can honestly mean** (proposal, operator and JRH to accept wording):
  "every word of this provision appears, in order, in the text extracted from the official file
  whose SHA-256 is recorded; skipped source text is limited to named classes (page header/footer,
  editorial reform note, amendment mark), each listed in the report". It does not mean: optical
  equality with the printed page, that the source is the text in force, or legal review.
- **D-4. Fidelity reports start as evidence in `.work/`, not as canonical corpus data.** Promoting
  a per-instrument `fidelity.json` into `corpus/` changes a trusted boundary (schema, types,
  validator, export, DB projection together) and is milestone F4, after operator acceptance of D-3.
- **D-5. Serving path recommendation for the full corpus: Cloudflare D1.** The projection is
  already SQLite with FTS5, 461 MB is inside D1's size limit, and the query contract in
  `reader/query-sqlite.mjs` already targets SQLite. Static shards or R2 would need a second query
  implementation. Not started; needs the website freeze lifted (milestone N2).
- **D-6. Fidelity and change monitoring share one mechanism** (fetch, hash, compare with the
  manifest). The 113 instruments without retained sources need a refetch that must not rewrite
  `corpus/` manifests; a changed hash on refetch is exactly the monitor signal (milestone F3).

## Milestones

Each milestone: verification, recovery, stop condition. One writer in `crates/` at a time.

### Goal 3: provenance

- **F1. `lex-mex verify-fidelity <slug>|--all`** (Rust, read-only, report to
  `.work/<slug>/fidelity-report.json`, schema `schemas/fidelity-report-v1.schema.json`, fixtures
  under `fixtures/fidelity/`). Checks that provision text is an in-order token subsequence of the
  hash-pinned extraction; classifies every skipped source span; fails closed when hashes differ.
  Verify: native gate; LRITF 156/156; corpus totals near the prototype. Recovery: revert the new
  files. Stop: any need to loosen the matcher to pass.
- **F2. Coverage direction.** Classify the source text between consecutive provisions (headings
  known from `heading_context`, labels, notes, furniture) so truncated provisions are detected.
- **F3. Source refetch and change monitor** (`monitor-sources`): refetch to `.work/`, compare
  SHA-256/ETag/Last-Modified with the manifest, never write `corpus/`. Closes the 113-instrument
  gap and gives the daily Diputados/DOF check the operator asked for.
- **F4. Publish the result.** Operator/JRH accept D-3 wording; then committed `fidelity.json`,
  validator rule, DB column, reader label, and a source-anomaly annotation type (D-1).
- **F5. Second extractor.** Compare the Poppler token stream with the `pdf_inspector` stream
  already used by the furniture audit, per provision, so the claim does not rest on one tool.
- **P1. Page-break fraction split** (D-2): parser fix plus regression fixture; reparse; accept a
  corpus diff only when F1 shows identical tokens and the only change is new paragraph breaks
  before fraction markers. Proof on LRITF first; corpus-wide reparse is an operator go because
  older instruments may pick up unrelated parser changes.

### Goal 4: human reading surfaces

- **H1. Re-export the Obsidian vault** from `main` for all 468 instruments; verify wikilinks; the
  vault repository commit is the operator's.
- **H2. Online reader for humans** uses the same stable article pages agents use; add visible
  cross-reference links and backlinks there (website, after the freeze).

### Goal 2: navigation database

- **N1. Search quality in the producer**: Spanish morphology (variant table or stemmed FTS column
  built in Rust), heading/topic labels, fraction-level reference targets (depends on P1).
- **N2. Serve the full projection** (D-5), remove the `catalog({limit:10})` assumptions, measure CPU.
- **N3. One source for the serving modules**: the producer builds the Worker modules as a release
  artifact; the website consumes that artifact and stops carrying copies.

### Goal 1: discoverability

- **S1. Operator actions (no code):** Google Search Console and Bing Webmaster registration for
  `leg4ll4bs.com`, sitemap submission; these need the operator's accounts.
- **S2. After the freeze:** IndexNow key, article URLs without the `/lex-mex/mx/` depth if the
  rename happens, registry listings (official MCP registry, Anthropic directory), full SEO pass.
- **S3. Re-run the organic fresh-agent check** after indexing; success is an unprompted find.

## Current checkpoint

2026-11-11 (later): uncommitted work in the `main` worktree; nothing deployed, website untouched.

- **F1 built.** `lex-mex verify-fidelity <slug>|--all [--strict]` (`crates/lex-parse/src/fidelity.rs`,
  `crates/lex-cli/src/fidelity.rs`, `schemas/fidelity-report-v1.schema.json`, `fixtures/fidelity/`).
  Run over the corpus: 353 instruments checkable (113 `inputs_missing`, 2 `source_hash_mismatch`);
  53,016 provisions: 43,382 `exact`, 9,270 `exact_modulo_classified_gaps`, 337 `unclassified_gap`,
  21 `diverged`, 5 `not_located`, 1 `text_not_in_source`; 52,652 pass (99.32%); 286 instruments
  fully pass. LRITF 156/156. Reports are in `.work/` (git-ignored). The 364 non-passing provisions
  are untriaged: the implementer's sample suggests most are anchoring limits on repeated short
  text such as `(Se deroga)`, plus a few real candidates (`lsh` art. 78 and `scap-dcg-2012`
  art. 17 Bis 22 start with a stray period; `locg` transitory cuarto).
- **P1 parser rule built, corpus not reparsed.** `normalized_blocks` in `diputados.rs` now opens
  a new paragraph when a Roman fraction marker follows page furniture and the open block is a
  fraction or closes a clause; fixture `fixtures/diputados/fraction-after-page-break-sample.txt`.
  LRITF proof (reparsed, inspected, then restored to the committed files): 16 paragraph breaks
  added, as intended.
- **D-7. The committed corpus is older than the parser.** The same LRITF reparse also removed
  three section headings that the committed text carries at the end of arts. 118, 122 and 129
  (`Sección segunda Delitos…`), added `section` to 16 heading contexts, changed the reference
  count from 126 to 127 and reset temporal fields that later stages fill. A reparse therefore
  changes more than the rule under test; each instrument's diff needs review and the temporal
  stages must be re-run. The fidelity check does not see a heading appended to a provision
  (the words are in the source, in order); F2 must add a heading-exclusion check.
- **D-8. Cited fractions are a ready-made linker validator.** 11,039 cited fractions with a
  resolved target: 9,452 found at a paragraph start in the target, 768 found merged mid-line
  (P1 fixes these), 819 not found in the target (candidates for a qualifier attached to the
  wrong article, e.g. `cff` art. 103 citing fractions of art. 92). Lettered sub-items show the
  same merge shape (469 candidates in 152 instruments, not yet checked against page breaks).
- **H1 trial passed, real export blocked.** All 468 instruments exported to a scratch vault with
  no failures; 233,776 wikilinks, none broken apart from the back-link to `Inicio`, which the
  scratch vault lacks. The export into `~/Vaults/Lex-Mex` was stopped by a permission check and
  needs the operator to allow or run it.

**Next action:** operator decisions (corpus-wide reparse, vault export, D-3 wording); then triage
the 364 non-passing provisions and build the L1 linker validator below.

### Added milestone

- **L1. Fraction-qualifier validator in `lex-link`** (D-8): for every edge with a fraction
  qualifier and a resolved target, check the target text has that fraction; report misses as
  evidence first, gate later. After P1 is applied to the corpus, the remaining misses are linker
  attribution defects to repair with fixtures.

## Progress log

- 2026-11-11: baseline, prototype, D-1, D-2; F1 delegated (Sonnet worker); H1 trial started.
- 2026-11-11: F1 delivered and run corpus-wide; P1 rule and fixture; D-7, D-8; H1 trial passed.
- 2026-11-11: L1 built as evidence (`lex-mex audit-qualifiers`, `crates/lex-link/src/qualifier_audit.rs`):
  11,039 cited fractions; 9,451 at a paragraph start, 887 merged mid-paragraph, 701 absent. Of the
  absent, 204 are a qualifier that follows a different citation than the one it was attached to
  (multi-article lists, "de este artículo", a later "del artículo 54 de la Constitución"): the
  linker attribution defect to repair next. 58 are an audit limit (CNBV amendment mark before the
  fraction marker, all `cue-dcg-2003`); about 191 are fractions missing or merged in the target.
- 2026-11-12: committed `415302e69` on branch `upgrade/four-goal-2026-11-12`. Operator decisions
  recorded in `docs/decisions.md` (2026-11-12): D-3 wording rejected in favour of a plain
  "same words, same order" statement; simple in-force status; vault export and corpus-wide
  reparse authorized. F3 (`monitor-sources`) delegated. Search-engine registration steps written
  to the operator's Google Workspace folder.

## Handoff checkpoint 2026-11-12 (session paused at a rate limit; resume from here)

Branch `upgrade/four-goal-2026-11-12`, head `415302e69`. **Uncommitted in the worktree:**

- **F3 built:** `lex-mex monitor-sources` (`crates/lex-cli/src/source_monitor.rs`, wiring in
  `main.rs`, `schemas/source-monitor-report-v1.schema.json`, `fixtures/monitor/`, README section).
  Its own tests passed (18); the full gate below includes it.
- **Reparse adopted for 289 instruments** (8,141 corpus files changed): 2,400+ provisions gain
  paragraph breaks (about 3,300 breaks), 33 defined terms recovered, none lost; 51 instruments
  also gain deterministic in-force determinations (4,486 provisions `unknown` to `effective`);
  LRITF loses the three leaked section headings (arts. 118, 122, 129, confirmed by the operator).
  No audited or lawyer-verified state changed in any instrument. 28 instruments were unchanged.
- **36 held, not adopted** (list: scratchpad `hold.txt`, reproduced here): ccf ccom cff cjm cnpp
  cucb-dcg-2004 cue-dcg-2003 laassp lacp ladua lbm lcnbv lfda lfea lfpccs lfvv lgcs lgdnna lgmime
  lgoaac lic linfonavit lnmasc-penal lnsijpa log lpcinecdi lrsic lsar ltpcpimcp oaac-dcg-2009
  reg-csps reg-lagra-mcdets reg-lfce reg-lftr-mcsre rgic scap-dcg-2012. Most only drop non-legal
  text (signature blocks, headings, running titles, amendment marks) and look like improvements;
  `cjm` gains 2 article ids, `scap-dcg-2012` gains one validation issue, `lic`, `lfpccs`, `lfvv`
  are refused by guards (approved reference, reviewed glossary). `ladua`, `log`, `reg-lftr-mcsre`
  are probably clean (held on a warning line, text unchanged in the flagged article).
- `docs/decisions.md` entry 2026-11-12 and this plan.

**In flight when paused:** a `mech` gate over the adopted corpus (fmt, clippy, workspace tests,
both validators, `verify-fidelity --all`, `audit-qualifiers --all`, and `link --all` in scratch
copy `root3`). Its result was not received. **Do not commit the corpus until that gate is re-run
and passes.**

Method used for the reparse (repeatable): clone `adapters corpus schemas prompts batches` to a
scratch root with `.work` symlinked, run `parse`, `derive-temporal`, `validate`,
`export --format markdown` per instrument with `--root`, compare before/after (ids, tokens,
temporal fields, audited state, validation issues), copy accepted instrument folders back.

**Source monitor run 2026-11-12 (`.work/source-monitor-report.json`):** 465 of 468 unchanged;
changed at source: `cpeum`, `cub-dcg-2005`, `socap-sofipo-dcg-2006` (new files under
`.work/monitor/`). The 113 missing retained PDFs were restored (hash-matching) but have no
extracted text yet, so they are not reparsed or fidelity-checked. Index entries not in corpus:
15 (13 abrogated per the index, EGDF, and `lfiiedb`, which the corpus holds from its DOF URL).
No "Ley de Economía Digital" in the Cámara index.

**Obsidian vault:** partial. 262 new folders written; 206 older folders not refreshed because
their generated files are read-only (exporter gets "Permission denied"). Proposed: make
`Corpus/` writable, export all 468 from the reparsed corpus, restore read-only. Awaiting the
operator's word on touching those permissions. `Inicio.md` still lists 203 instruments.

**Next actions, in order:** (1) re-run the gate, then commit monitor code and the adopted corpus
as separate commits; (2) review and adopt the held 36; (3) extract and reparse the 113 restored
instruments; (4) re-ingest the 3 changed sources; (5) finish the vault export; (6) repair the
linker qualifier attribution (204 cases); (7) operator: in-force rule cases (`lftr`, `reg-lfmn`,
"(Se deroga)" provisions), scheduling `monitor-sources --all --discover --strict` daily.

**Gate result received 2026-11-12 (supersedes "result was not received" above):** over the
adopted corpus, fmt, strict clippy, 508 workspace tests, `validate lritf` (127 references) and
`validate ifpe-dcg-2021` pass. `verify-fidelity --all` totals are unchanged (52,652 of 53,016
pass), as expected for whitespace-only changes. `audit-qualifiers --all`: cited fractions at a
paragraph start 9,451 to 9,618, merged mid-paragraph 887 to 752, absent 700. `link --all` in
scratch copy `root3` exits 0 with exactly the two accepted gaps (`lftr`,
`socap-sofipo-dcg-2006`); it would relink `larefam` (46 references, three files), which is not
applied to the repository. The corpus and monitor code are ready to commit on the operator's word.

## Checkpoint 2026-11-12 (later): committed, held set reviewed

- Committed on the operator's word, after re-running the native gate on the tree (fmt, strict
  clippy, 508 tests, both validators): `bec9467a9` (monitor-sources) and `92e1e1365` (289
  reparsed instruments, decisions entry, this plan, the review brief). Post-commit check of
  `92e1e1365`: no temporal value removed (only `null`/`unknown` replaced); review fields changed
  only `not_analyzed` to `machine_accepted` (4,486). Wording correction: "31 references newly
  resolve" in that commit message counts added `resolution_rule: same_article` lines, not a
  measured count of newly resolved edges.
- **Held 36 re-run** in a fresh scratch root cloned from `92e1e1365` with a binary built from it
  (script: session scratchpad `reparse_held.py`; every token-change span listed, every deletion
  printed with its token count, head and tail, and checked to end in a signature, running title
  or heading). Result:
  - **Adopted 30** (`ccf ccom cff cjm cnpp cucb-dcg-2004 lacp ladua lbm lcnbv lfda lfea lgcs
    lgdnna lgmime lgoaac linfonavit lnmasc-penal lnsijpa log lpcinecdi lrsic lsar ltpcpimcp
    oaac-dcg-2009 reg-csps reg-lagra-mcdets reg-lfce reg-lftr-mcsre rgic`; 27 change on disk,
    `cucb-dcg-2004`, `log`, `oaac-dcg-2009` reparse byte-identical). `cjm` gains 546
    deterministic in-force determinations (`not_analyzed` to `machine_accepted`), as the 51 did in
    `92e1e1365`; no other review or temporal field changes. `reform-temporal-evidence.json`
    gains 315 reform-decree transitory entries in 13 instruments; 13 ids are renamed with their
    text preserved; no evidence text is lost (same check on `92e1e1365`: none lost). Heading
    context: `ccom` arts. 1390 Ter to Ter 3 gain `Capítulo I` and `ccf` art. 1549 Bis moves from
    `Capítulo III` to `Capítulo III Bis`, both as the source prints. Every token change is a
    suffix or running-title deletion of non-legal text: promulgation/signature blocks at the end
    of the last transitory (12 instruments; 10 of them lose exactly one edge, the CPEUM art. 89
    citation inside that block, and none loses another; gained: 12 internal fraction edges in
    `ccom`, `cff`, `ladua`, `reg-lfce`, and `cjm` art. 30 Séptimus to art. 4), a leaked chapter heading now carried in `heading_context` (`ccf`
    art. 1549), running law titles (`lpcinecdi`, `ltpcpimcp`). `reg-csps` annex label gains the
    final period the source prints. Validation issue sets identical before and after; no term
    lost (+4 recovered); no temporal value lost. `verify-fidelity`: pass count equal or higher in
    all 30 except `ccf` art. 1549 Bis (text unchanged; the chapter heading it shares words with
    now sits between articles and the matcher anchors on it, an F1/F2 anchoring limit).
    `link --all` in the scratch root: exit 0, exactly the two accepted gaps, no references file
    changed (the standing `larefam` relink of 46 references is unchanged and still unapplied).
  - **`cjm`: corpus repair.** The committed text lacks arts. 30 Sextus and 30 Séptimus entirely
    (no trace in any provision); the official PDF has them (lines 388 and 414 of the extraction)
    and the parser now recovers them. The adapter's frozen machine-proposed count was taken from
    the defective parse; `adapters/diputados/cjm.json` `expected_articles` 952 → 954 by hand, from
    the source (the `count_baseline` block still records the original proposal). F1 cannot see a
    missing provision; F2 coverage would.
  - **`laassp` held for P2.** The reparse is otherwise a repair (13 leaked Título/Capítulo
    headings leave the text, 120 heading contexts filled), but it appends the reform-decree line
    "Artículo Segundo a Artículo Cuarto.- ……" (source line 3010, between art. 120 and
    Transitorios) to art. 120, the only case in the set where non-provision words are added.
    Six committed instruments already carry the same defect class (`lgec` 49, `lgpdppso` 137,
    `lgpisdme` 43, `lgtaip` 216, `lsat` transitory octavo, `ladua` transitory noveno). Adopt
    after milestone **P2** below.
  - **Still held, 6:** `laassp` (above); `lic`, `lfpccs`, `lfvv` (guards: approved outdated reference, reviewed
    glossary profiles; JRH); `cue-dcg-2003` (amendment marks (101)–(108) leave the text, an
    improvement, but six provisions lose `effective_from` 2003-03-20: arts. 15 Bis 2, 40, 54,
    82 Bis, 84, 84 Bis; needs the reviewer's view); `scap-dcg-2012` (art. 202: the source prints
    "(261)" mid-sentence, "mediante avisos colocados (261) en lugar visible", and the legend
    defines no mark 261; the new parse reads it as an amendment mark and validation fails with
    `unknown_amendment_mark`; source anomaly like D-1, operator decision).

- **Committed** after the native gate passed on the tree (fmt, strict clippy, 508 tests,
  `validate lritf`, `ifpe-dcg-2021`, `cjm`): `a2bd4353b` (`cjm` and its adapter) and `09e4a457f`
  (26 instruments and the entry above). Evidence over the repository after both: `verify-fidelity
  --all` 52,655 of 53,018 provisions pass (was 52,652 of 53,016), 287 instruments fully pass
  (was 286); `audit-qualifiers --all` 11,074 cited fractions: at a paragraph start 9,653 (was
  9,618), merged mid-paragraph 723 (was 752), absent 698 (was 700).

**Next actions, in order:** (1) P2 parser repair, then adopt `laassp`; (2) extract and reparse
the 113 restored instruments; (3) re-ingest the 3 changed sources (`cpeum`, `cub-dcg-2005`,
`socap-sofipo-dcg-2006`); (4) finish the vault export (operator: permissions on `Corpus/`);
(5) linker qualifier attribution (204 cases); (6) operator/JRH: `cue-dcg-2003` dates,
`scap-dcg-2012` mark (261), the `lic`/`lfpccs`/`lfvv` guards, in-force rule cases.

### Added milestone

- **P2. Reform-decree omission lines** ("Artículo(s) Segundo a … .- ……" / "……….." between the
  last article and Transitorios): classify as decree furniture, not provision text; fixture from
  `laassp`; reparse `laassp` and the six instruments above; accept only that deletion, then
  adopt `laassp`.

