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
- **Correction (supersedes "in-force determinations" above, in the handoff section and in the
  messages of `92e1e1365` and `a2bd4353b`).** Classified by script: of the 4,486 provisions in
  `92e1e1365` that gained `temporal_basis: deterministic_rule`, 3,963 moved `unknown` to
  `effective` (1,766 with an `effective_from` date, 2,197 without one) and 523 were already
  `repealed` (522) or `partially_repealed` (1) and only gained the basis and
  `machine_accepted`. All 546 in `cjm` (`a2bd4353b`) are of that second kind: "(Se deroga)"
  articles already `repealed`, none made `effective`. The rule is the existing `derive-temporal`
  repeal classification, not a new one, but it bears on the open "(Se deroga)" point in the
  2026-11-12 entry of `docs/decisions.md` (machine-accepted repeal of a marker provision inside an
  in-force instrument), which still needs the reviewer.

**Next actions, in order:** (1) ~~P2 parser repair, then adopt `laassp`~~ done, see below; (2) ~~extract and reparse
the 113 restored instruments~~ 110 adopted in `b9f4d319f`, see "batch of 112"; (3) re-ingest the 3 changed sources (`cpeum`, `cub-dcg-2005`,
`socap-sofipo-dcg-2006`); (4) finish the vault export (operator: permissions on `Corpus/`);
(5) linker qualifier attribution (204 cases); (6) operator/JRH: `cue-dcg-2003` dates,
`scap-dcg-2012` mark (261), the `lic`/`lfpccs`/`lfvv` guards, in-force rule cases.

## Checkpoint 2026-11-12 (P2 and baseline)

- **Baseline.** A fresh root and release binary from `3b5dabeb4` ran parse, derive-temporal,
  validate and export over all 353 checkable instruments: the output equals the committed corpus
  except the held instruments (`laassp`, `cue-dcg-2003`, `scap-dcg-2012` differ; `lic`, `lfpccs`,
  `lfvv` stop at their guards; `scap-dcg-2012` fails validation as recorded) and two committed
  files not reproducible from HEAD: `larefam` (glossary coverage, term usages, art. 14 Markdown:
  the committed corpus links "Militar" and "Ejército" inside the names "Dirección General de
  Educación Militar" and "Universidad del Ejército"; HEAD drops those two false usages; the
  committed state came from the prior session's scratch root) and `lfrsp`
  (`reform-temporal-evidence.json`: HEAD labels the 1995-05-26 entries "Acto DOF" instead of
  "Sección transitoria 2, Decreto"; the source is the Ley Orgánica del Poder Judicial de la
  Federación, a law, so HEAD's label is the accurate one; 14 ids renamed, no text lost). The
  prior session's "28 unchanged" compared provisions and references only, which is why `lfrsp`
  was not caught. Both are refreshed to HEAD's output.
- **P2 built.** `is_elided_decree_article_run` in `diputados.rs`: a block whose words are only an
  article prefix, ordinal words and the connectors `a`/`y`, and whose body is only redaction
  dots (three or more) or ellipsis, is decree apparatus; dropped in the article loop (with the
  existing single-ordinal rule) and skipped at the statute-transitory fallthrough. Fixtures
  `elided-decree-article-range-sample.txt` (laassp shape) and
  `elided-decree-articles-after-transitories-sample.txt` (ladua shape), plus a predicate test with
  negative cases. Source scope: 100 such lines in the retained texts; the committed corpus
  already omitted 87; 13 were in a provision, all in hash-matching instruments.
- **P2 result** (all 353 reparsed against the baseline): exactly 14 instruments change (the 13
  plus `laassp`), each by removing that one line as a suffix of one provision and its Markdown:
  `laassp` art. 120, `lgec` 49, `lgpdppso` 137, `lgpisdme` 43, `lgtaip` 216; last transitory of
  `ladua`, `lbio`, `lcne`, `lepecfe`, `lepepm`, `lgeo`, `lpte`, `lse`, `lsh`. Issue sets, terms,
  references and `verify-fidelity` counts unchanged in all 14. `laassp` is now adopted with its
  earlier reviewed changes (13 leaked headings removed, 120 heading contexts filled, one internal
  fraction edge gained).
- **Extraction probe for the 113** (isolated scratch root, real `.work/` untouched): `extract`
  with pdftotext 26.11.0 reproduces the manifest `extracted_text_sha256` exactly for 112; the
  113th, `socap-sofipo-dcg-2006`, has no retained source (its official file changed at source).
  So item (2) of the next actions is mechanical for 112.
- **Held now 5:** `lic`, `lfpccs`, `lfvv`, `cue-dcg-2003`, `scap-dcg-2012`.
- **Committed:** `24d4fc083` (P2 rule, fixtures, 14 instruments) and `473755e3f` (`larefam`,
  `lfrsp`, this plan), after the native gate (fmt, strict clippy, 511 tests, both validators).
  Afterwards on the repository: `link --all` exit 0, relinked 0, exactly the two accepted gaps,
  no file changed (the standing `larefam` relink is gone); `verify-fidelity --all` 52,655 of
  53,018. Remaining elided-run lines in committed provisions: 2 (`lsat` transitory octavo,
  `lfisan`), both in the 113 without retained text, so P2 covered every instrument it could.
- **Batch of 112, prepared, not started.** `extract` rewrites `source-manifest.json`
  (`extraction_tool` 26.06.0 → 26.11.0), so it must not run in the repository; the probe's
  hash-matching texts are copied into `.work/` instead. None of the 112 carries human or model
  temporal state (5,134 `not_analyzed`, 8,825 `machine_accepted`/`deterministic_rule`).
  Method: "before" `verify-fidelity` on the committed 112; reparse in a fresh root; classify by
  whole-folder diff; adopt without reading only whitespace-only text changes with identical ids,
  issues, terms and evidence text; read the source for every id added or removed and every count
  failure; then rerun the full baseline and `link --all`.

## Checkpoint 2026-11-12 (batch of 112)

- Texts copied into `.work/` after re-checking each SHA-256 against the manifest (112 of 112).
  First-ever fidelity check of the committed 112 ("before"): 13,837 of 13,959 provisions pass,
  93 instruments fully pass.
- Reparse in a fresh root from `0e70a0510`, classified by whole-folder diff (script
  `classify.py` in the session scratchpad). **Held 2:** `lieps` (parse stops: glossary profile
  reviewed against different text; JRH) and `lftr` (recorded `repealed`, an accepted closure
  gap; the reparse adds 3 ids and fails validation; operator). **Adopted 110:** 107 change,
  3 reparse byte-identical (`reg-lieps`, `reg-lss-rfar`, `servinv-dcg-2013`). Every token change
  (340 spans in 56 instruments) is a deletion of non-legal text, by class: running law titles in
  capitals (232; 186 mid-provision, 46 at the end), leaked Título/Capítulo/Sección headings at
  the end of a provision (56, the heading now in `heading_context`; 294 heading contexts filled,
  none replaced), promulgation and signature blocks (53, all suffixes), elided decree-article
  runs (2: `lsat`, `lfisan`, closing the P2 remainder). No id added or removed, no validation
  issue added, no term lost, no reform-evidence text lost. 44 edges lost, all inside deleted
  text (38 the CPEUM art. 89 citation in promulgation blocks, the rest in running titles); 37
  gained. Temporal: 98 provisions gain `deterministic_rule`/`machine_accepted` (81 `unknown` to
  `effective`, 17 repeal markers already `repealed`); no value lost. `verify-fidelity` on the
  110: 13,430 → 13,433 of 13,551 pass, 92 → 94 instruments fully pass, none worse.
- **Committed** `b9f4d319f` (the 110 plus this plan) after the native gate (fmt, strict clippy,
  511 tests, both validators), commit made by the `mech` runner. **Full baseline from
  `b9f4d319f`** (fresh root, release binary, parse/derive-temporal/validate/export over all
  465 with text): output equals the committed corpus for every instrument except the held
  ones — `cue-dcg-2003` (47 files), `lftr` (59) and `scap-dcg-2012` (5) differ; `lic`,
  `lfpccs`, `lfvv`, `lieps` stop at their parse guards; `lftr` and `scap-dcg-2012` fail
  validation as recorded. On the repository afterwards: `link --all` exit 0, relinked 0,
  unchanged 466, rejected exactly the 2 accepted gaps, references 36,105 → 36,105, no file
  changed. `verify-fidelity --all` (now covers 465 inputs; 1 missing, `socap-sofipo-dcg-2006`;
  2 retained-text hash mismatches, `cub-dcg-2005` and `lfiiedb`, both known since the initial
  baseline): 66,495 of 66,977
  provisions pass (54,458 exact, 12,037 modulo classified gaps; 364 unclassified gap, 84 not
  located, 32 diverged, 2 text not in source), 382 of 468 instruments fully pass.
  `audit-qualifiers --all`: 11,068 labels, 9,769 at paragraph start, 620 mid-paragraph, 679
  absent, 20 qualifiers without labels. Corpus totals by script at `b9f4d319f`: 64,861 articles
  / 3,506 transitories / 30 annexes (unchanged), 36,105 references (−6), temporal status
  `effective` 32,334 (+81), `unknown` 30,510 (−81), `repealed` 5,551, `partially_repealed` 2.
- **Correction to the batch entry above** (recount from `b112-class.json` restricted to the 110
  adopted; the entry's figures mixed in the 2 held). Token changes: 341 spans in 57
  instruments, every one a deletion: running titles 232, leaked headings 56, promulgation and
  signature blocks 51, elided decree-article runs 2. Edges: 38 lost, all the CPEUM art. 89
  citation inside deleted promulgation blocks; 31 gained, all `same_article` fraction
  citations ("fracción I de este artículo", "… del presente artículo") in provisions whose
  text did not change, so produced by the current reference extractor, not by a deletion.
  Net −7, which with `laassp`'s +1 at P2 accounts for 36,111 → 36,105.
  `verify-fidelity` cannot detect an over-deletion (a shorter text still passes the
  subsequence test), so for these spans the class tally is the only check.
- **Held now 7:** `lic`, `lfpccs`, `lfvv`, `lieps` (parse guards, JRH); `cue-dcg-2003`,
  `scap-dcg-2012` (operator); `lftr` (accepted closure gap, operator).
- **Next actions, in order:** (1) re-ingest the 3 changed sources (`cpeum`, `cub-dcg-2005`,
  `socap-sofipo-dcg-2006`) — this is a fetch of new official files and so a decision about
  which version the corpus records; (2) finish the vault export (operator: permissions on
  `Corpus/`); (3) linker qualifier attribution (204 cases); (4) operator/JRH: the seven held
  instruments above and the in-force rule cases.

## Re-ingest preview 2026-11-12 (scratch only; nothing written to the repository)

The three official files that `monitor-sources` found changed (`.work/monitor/<slug>/<sha>.pdf`,
SHA-256 re-checked) were run through extract, parse, derive-temporal, validate and export in
an isolated scratch root with its own `.work/`, then classified against the committed folders.
None of the three carries human or model review state (only `not_analyzed` or
`machine_accepted`/`deterministic_rule`). Adopting any of them changes which official version
the corpus records, so it is the operator's decision; each also has an open question:

- **`cpeum`** (Diputados, Last-Modified 2026-10-09): validates; no id added or removed; 15
  spans in arts. 82, 116, 122 only, the reform DOF 07-10-2026 (gender-inclusive wording; the
  President, governors and the Head of Government of Mexico City must not hold or acquire
  another nationality; "Párrafo reformado DOF 07-10-2026" marks); `latest_reform_date`
  null → 2026-10-07; no edge or term change; `reform-temporal-evidence.json` gains the decree's
  six transitories (`cpeum:amendment:2026-10-07:transitory:primero` … `sexto`). The three
  articles stay `unknown` / `not_analyzed`. The decree enters into force the day after
  publication, but applies from the 2028 electoral process and not to current officeholders
  (Transitorios Segundo, Tercero), so how those three provisions are dated is a temporal
  question for JRH, not something the deterministic rule should settle.
- **`cub-dcg-2005`** (CNBV, reform 2026-11-06): validates; no id change; 169 spans in 22
  provisions (18 articles, chiefly 91 Bis 1, 172 Bis 37–39, 68, 92; transitories quinto to
  décimo); 4 edges lost, 9 gained; terms unchanged. The transitory diffs move footnote text
  ("A continuación se transcribe la disposición …") between transitories, which looks like a
  page-layout shift; read against the source before adopting.
- **`socap-sofipo-dcg-2006`** (CNBV): fails validation. The new official file's reform legend
  goes from 90 to 92 with no entry (91), yet marks arts. 44, 65, 100, 149, 205 Bis 13 and
  209 Bis 9 with "(91)"; the committed version (fetched 2026-07-13) defined (91). A source
  defect, like `scap-dcg-2012`'s "(261)". Otherwise: 7 articles added (47 Bis, 48 Bis, 66 Bis,
  69 Bis, 101 Bis, 103 Bis, 152 Bis), 63 spans in 26 provisions, 1 term lost (95 → 94).
  It remains an accepted closure gap either way.

Scratch root and classification: session scratchpad `reingest/`, `reingest-class.json`. That
root is a preview only and must not be adopted: its manifest `source_sha256` was set by hand
and `retrieved_at` is stale. An adoption goes through `fetch`, then the usual reparse review.

## Linker qualifier attribution 2026-11-12 (local side branch, closure red)

On branch `upgrade/qualifier-attribution-2026-11-12` (from `9e98455e3`, local): `b68957bbb`
(rule and test) and `05d15bcd0` (190 instruments, `references.json` only). Not merged into
`upgrade/four-goal-2026-11-12`, because `link --all` now rejects a third instrument, which the
closure rule (decisions.md, P1) treats as a failure.

- **Defect.** The last article number of a citation list took every qualifier phrase up to the
  sentence end (each citation group ends at the next `artículo` header, so a phrase written
  before a later header fell inside the earlier group). Examples: CCF "artículo 2980 y … la
  fracción V del artículo 2935" gave 2980 "fracción V"; CFF 92 gave "fracciones XI … XVIII de
  este artículo" to art. 92; CCOM 18 got "su fracción XX" of art. 21.
- **Rule** (`last_number_qualifiers` in `crates/lex-link/src/references.rs`): a qualifier
  belongs to the last number when connector text alone separates them or the previous chained
  qualifier (punctuation, `y`/`o`/`e`, `su`, ordinal words, `numeral 5`, `Apartado B`, `Base V`,
  a bare numeral with `Bis`/`Ter`, an inciso letter; after an ordinal mark `2o.`), unless the
  phrase is followed by an article header or `de este artículo`; after the chain, only a
  back-reference (`de dicho / del mismo / del citado artículo`, `su segundo párrafo`) with no
  number, `;` or relative article in between. Non-last numbers and pre-number qualifiers are
  unchanged. Test `a_later_qualifier_is_not_attached_to_the_previous_citation` (lex-parse) with
  the source shapes. Gate: fmt, strict clippy, 512 tests, both validators.
- **Effect, by diff of a scratch `link --all` against the corpus:** 190 instruments,
  1,192 qualifiers removed, none added; edge ids and every other field identical; references
  36,105; a second `link --all` relinks 0. Removed: 306 re-attributed (the same span is another
  edge's pre-qualifier), 48 re-attributed to a `same_article` edge, 42 in amendment notes, 796
  orphans. The orphans were sampled, not fully reviewed: 60 read at random plus all 68 with a
  gap under 20 characters. Most are correct removals (the phrase belongs to a later citation,
  an annex, "este artículo" or "esta fracción"); about 38 sit on edges whose target is already
  wrong (below); a few percent are correct qualifiers now lost ("excepto fracción I", "de esta
  Ley, fracciones V y VII", "salvo el de la fracción IV"), the conservative side of the rule.
  Repository Markdown is byte-identical (it links whole articles; fraction anchors are written
  only for the Obsidian vault). `audit-qualifiers --all`: labels 11,068 → 10,237, absent
  679 → 357 (the rest: fraction missing in the target, and the `cue-dcg-2003` audit limit);
  found at paragraph start 9,769 → 9,305, the difference being wrong attachments that happened
  to hit an existing fraction. These figures supersede the "204 cases" estimate.
- **`lgmime` held (operator decision).** Its edge to LGIPE art. 11 Bis
  (`transitory:vigesimo-octavo:reference:41-47:11-bis:direct`, outdated-reference approval
  2026-10-16) carries "párrafo 2", which belongs to "44, párrafo 2, de la Ley General de
  Partidos Políticos"; the rule removes it, and the guard forbids any change to an approved
  edge. Options: (a) `link lgmime --migrate-protected`, the audited override for a recorded
  operator migration, with the approval re-recorded; (b) narrow the guard to the fields the
  approval records (span, target, text hashes; qualifiers are not among them); (c) accept
  `lgmime` as a third closure gap. JRH chose (a) on 2026-11-12 (below).
- **Parse path finding.** `parse lgmime` regenerates that edge without the guard (drops
  "párrafo 2", keeps the approval fields) and still validates; parse and link agree for
  `lrsic`, `ccf`, `cff`, `ifpe-dcg-2021`, `lgpp`. So the approved-edge guard lives only in
  `link`; a full reparse can change an approved edge silently. Pre-existing; not changed here.
  The full 465 baseline was not rerun on the side branch.
- **Separate defects found, not fixed:** (1) truncated lists: a list item the number pattern
  does not accept ("90, penúltimo párrafo, 110, …", "315, últimos dos supuestos, 316 …") ends
  the list, so the later articles get no edge (LGPP art. 44 in `lgmime` has none); (2)
  suffixed numbers read short: "2o.-A", "1o.-C", "14- B" give edges to arts. 2, 1, 14; (3)
  compound ordinals ("décimo primer párrafo") are read as "primer párrafo".

## JRH replies 2026-11-12 (operator and legal reviewer are the same person)

JRH's statements, verbatim except for typos fixed:

1. "lgmime and lgpp were both declared invalid by the Supreme Court in 2023 (the court invalidated
   the decree), which in turn returned the vigencia to the previously abrogated law … In this
   case it is worth looking for that ruling in the DOF and checking it"
   (DOF 24-11-2023, nota 5709440).
2. "CPEUM is a temporal issue. It should have a small note (like the ones from SCJN) saying
   artículo vigente a partir del proceso electoral 2028."
3. "cub-dcg-2005 and socap-sofipo-dcg-2006 were included in the Fintech package but should have
   their own parsers for DCG, same for cue-dcg-2003, scap-dcg-2012."
4. "Happy to merge the side branch." The `lgmime` approved-edge option (a/b/c) is not yet chosen.

**Ruling check (read-only).** The source is the DOF page for nota 5709440. The copy is kept in
the session scratchpad at `dof-5709440/nota.html`, sha256
`12ebece74f64ac0a9380c71657edb1ece1b0508bc95386e031b985d1c5836a82`.

- The case is AI 71/2023 and its joined cases (acumuladas) 75, 89, 90, 91, 92 and 93/2023.
- Resolutivo SEGUNDO invalidates the whole decree of DOF 02-03-2023, which reformed the LGIPE,
  the LGPP and the LOPJF and issued the LGMIME. The invalidity takes effect when the resolutivos
  are notified to Congress; the source text gives that date as 23-06-2023.
- Efectos: "recuperan vigencia las disposiciones legales que regían antes de la entrada en
  vigor del decreto impugnado."

The corpus is inconsistent with the ruling on three points. None was changed; each is a JRH
temporal decision.

- `lgmime` is the invalidated 2023 law, not the revived one. It is recorded `status: in_force`.
  All 70 articles are `effective` from 2023-03-03 by `deterministic_rule`, with
  `machine_accepted` and no `effective_to`. Its own source header already prints the
  invalidity declaration.
- `lgsmime` (1996) is the law that regains force. It is recorded `in_force`, with its latest
  reform on 2025-11-14, which is consistent.
- `lgpp` has `latest_reform_date` 2023-03-02, an invalidated reform. Of its 97 articles, 53
  print the "declarado inválido y recupera su vigencia" notes and are `effective`. Whether the
  dates reflect the restored text still has to be checked.
- `lgipe` carries 242 `unknown` provisions that mention invalidity. Those were left as they are.

The `lgmime` edge whose qualifier the repair removes sits in a law that is no longer in force.
That may change how JRH weighs options (a), (b) and (c).

**Held four: what the guards protect.** The four were reparsed in a scratch copy with the
guards removed (scratchpad `held4/`).

- In all four, every text change is whitespace only: lic 39 provisions, lfpccs 8, lfvv 5,
  lieps 4. No provision id was added or removed.
- The change is that the new parse separates a fraction that had been run into the previous
  one. Examples: lfpccs art. 3 "…semillas; XIII. Normas Mexicanas" becomes two lines; lieps
  art. 3 "…se entiende por: I. Bebidas…" likewise.
- This moves the hash of the glossary article that JRH reviewed (`lfpccs` 3, `lfvv` 2,
  `lieps` 3). In `lic` it changes the text hash of art. 194, which holds the approved stale
  citation of LFCE 31 Bis; the span and offsets (4292–4298) are unchanged. Art. 187, LRAF
  28 Bis, is untouched.
- Adopting them means JRH re-confirms the same definitions and the same two stale citations
  against the re-spaced text, and the recorded hashes are refreshed.

**JRH decisions 2026-11-12 (answers in session):**

1. **`lgmime` approved edge:** migrate it with `link lgmime --migrate-protected`, recorded as a
   JRH migration. The same edge was refreshed the same way on 2026-10-16.
2. **`lgmime` temporal:** the instrument is invalid from 2023-06-23. Every provision gets
   `effective_to` 2023-06-23 and the instrument is not in force. Basis: SCJN AI 71/2023,
   DOF 24-11-2023.
3. **Held four (`lic`, `lfpccs`, `lfvv`, `lieps`):** JRH re-confirms the definitions and the
   stale citations against the re-spaced text. Refresh the recorded hashes and adopt.
4. **`cpeum` arts. 82, 116 and 122:** they carry the note "Artículo vigente a partir del proceso
   electoral 2028", shown like the SCJN notes.

**`lgmime` invalidity, as applied 2026-11-12.** Record:
`docs/federal-lgmime-invalidity-decision-2026-11-12.md`.

- **Done at the instrument level.** `lgmime` now carries a header-basis `source_repeal_notice`
  quoting the printed "Declaratoria de invalidez del Decreto DOF 02-03-2023 …". The ruling is
  the formal source (DOF nota 5709440, retained in `.work/lgmime-invalidity/`), the reviewer
  is JRH, and no successor is recorded. Status is `repealed`.
- **Code.** `header_repeal_note_is_dated` (lex-core) accepts the printed invalidity header
  beside "Ley Abrogada DOF …"; test
  `source_repeal_notice_accepts_a_printed_scjn_invalidity_header`, fixture
  `fixtures/diputados/lgmime-invalidity-header.txt`. The exporter banner reads "Ley declarada
  inválida por la SCJN (no abrogada) — texto histórico" instead of "Ley abrogada".
- **Not done: `effective_to` 2023-06-23 on the 100 provisions.** Three things block it:
  - No provision in the corpus has an `effective_to`.
  - `parse` resets temporal fields, and `derive-temporal` re-derives the 70 articles as
    `effective` from 2023-03-03.
  - `review open` / `resolve --resolution lawyer-override` needs a stored
    `temporal-analysis-result.json`, which only the model-analysis path writes, and that path
    runs through Codex. So JRH's date has no durable deterministic home.
  - The proposed design is an optional, reviewer-supplied effects date on the repeal notice
    (for an invalidity, the date taken from the ruling). `derive-temporal` would then set
    `temporal_status: repealed` and `effective_to` on every provision of a noticed instrument.
    That is a boundary change (schema, types, derive rule, fixture, docs), made as its own
    milestone. The `reg-lfmn` notice deliberately carries no dates, so the field must stay
    optional.
- **Precedent kept:** like `reg-lfmn`, the instrument is `repealed` while its articles still
  read `effective`.

**Held-four outcome.** `lic`, `lfpccs` and `lfvv` were adopted. `lieps` stays held. Its provisions are
whitespace-only, but the reparse also re-segments its reform-decree evidence
(`reform-temporal-evidence.json`): 107 entries become 101, 19 ids are removed, 13 are added
(for example the yearly `Acto DOF 2014-12-22` … `2025-12-22` unico entries) and 2 have their
text rewritten. That is outside the whitespace re-confirmation and needs its own review.

**`lic` adoption note.** The parse guard refuses to replace a persisted stale-reference
decision whose text hash changed, even after the adapter carries the re-confirmed hash. For
the adoption, `corpus/mx/lic/references.json` was removed before the parse and regenerated.
The regenerated file was then compared with the old one: art. 187's approval is identical and
art. 194's differs only in the five refreshed fields (`source_text_sha256`, reviewer,
`captured_at`, `decision_source`, its sha). A guard path for re-confirmed hashes is still owed.

**465 baseline after `ea9558b36`.** Every slug reparsed, derived, validated and exported in a
scratch copy. Only the expected instruments differ: `lieps` (held; parse refuses on the stale
glossary profile), `cue-dcg-2003` (47 files), `scap-dcg-2012` (5; validate fails) and `lftr`
(59; validate fails, accepted gap). `lgmime` and the adopted `lic`, `lfpccs`, `lfvv` reproduce
the corpus exactly.

### Added milestone

- **P2. Reform-decree omission lines** ("Artículo(s) Segundo a … .- ……" / "……….." between the
  last article and Transitorios): classify as decree furniture, not provision text; fixture from
  `laassp`; reparse `laassp` and the six instruments above; accept only that deletion, then
  adopt `laassp`.


## JRH decisions 2026-11-12 (second round)

- **`lgmime` provision end dates: deferred.** JRH: the law is no longer in force, and its
  repeal takes effect 23-06-2023, the date the SCJN set, which is also when the previous law
  regained force. Recording that it is not in force (instrument `repealed`, invalidity note) is
  enough for now. Provision `effective_to` is not applied and must not hold the project. The
  notice effects-date field is parked, not owed.
- **DCG parsers: parked** until the Diputados PDFs are settled. The marks (91) in
  `socap-sofipo-dcg-2006` and (261) in `scap-dcg-2012` point to modifying decrees.
- **LGPP / LGIPE:** they need an article-by-article review to establish which articles are
  repealed. Each keeps its note as printed in the PDF, the same way as a last-reform note.
- **Vault `Corpus/`:** read and write authorized. Writes go only through the repository's own
  export, respecting the parser ingest.
- **CPEUM:** go (see milestone C1).

### Milestone H1: one hold must not hold the corpus

JRH: a single failure or hold cannot block the full corpus. These all-or-nothing gates were
found:
1. `link --all` rejects an instrument's whole new graph when one edge targets a missing
   article.
2. The closure gate fails on any unaccepted gap.
3. The persisted-decision guard in `parse` refuses the whole instrument, and a whitespace-only
   change to the text is enough to trip it (`lic`, `lieps`).

Target: per-instrument (and where possible per-edge) isolation, so the remainder lands and
the holds are listed. No guard is changed yet. This is a design item for review.

### Applied 2026-11-12 (second round)

- **C1 CPEUM** (`a461a41c0`): re-fetched (SHA-256 `cefb525a…`, the hash the monitor saw). New
  boundary `provision_notes` (schema, type, guards, export). Arts. 82, 116 and 122 carry JRH's
  note. Only those three change words; 19 articles reflow paragraph breaks only. Temporal fields
  unchanged. The prior retained PDF (`ca63a23a…`) was overwritten by `fetch` and is not kept locally;
  its hash stays in git history. `pdftotext` moved from 26.06 to 26.11.
- **LGPP/LGIPE table** (`fbf46774e`): `docs/lgpp-lgipe-invalidity-review-2026-11-12.md`, for JRH. No
  status changed. LGIPE `unknown` is not analysis-related to the ruling: 561 of 567 never analyzed.
- **Vault export**: all 468 instruments exported to `~/Vaults/Lex-Mex` through `export --format
  obsidian`. The 40,572 read-only files were made writable for the run and set back to
  read-only. The 32 NOM standards have no instrument adapter and were not exported by this path.
  The vault's own git now shows 12,940 changed paths, not committed (operator's vault).
- **Codex handoff**: `AI/60_Evaluations/provider-handoffs/pending/codex/2026-11-12-lex-mex-four-goal-upgrade-review.md`
  (Agent Vault `157a575`).

### Codex review delivered 2026-11-12

At `ee4f730c3`, independent review is recorded in
`docs/four-goal-upgrade-codex-review-2026-11-12.md`. Native fmt, strict clippy,
workspace tests and LRITF/IFPE/CPEUM validators passed. LIC approval differences
and the 190-file qualifier-only migration were independently checked.
Change requests: preserve documented valid qualifier shapes and provide an
audited re-confirmation path instead of removing persisted references.
Next action: bounded fixes/design for these two findings, then re-review.
JRH temporal decisions and held instruments remain unchanged.

## 2026-11-13: Codex change requests, decision routing, MCP tests

**Decision routing (JRH).** Recorded in `AGENTS.md` ("Decision routing"): only strict legal
questions go to JRH; everything else is resolved by the agents from the surrounding context and
recorded. Applied here: the LGPP/LGIPE caveats were settled from the text; only the status
classification stays with JRH.

**Codex item 5, audited re-confirmation.** `reconfirms` on `outdated_reference_approval` and
`legal_operator_review` (schema `reference-edge.schema.json`, `lex_core::ReconfirmedEvidence`,
`check_reconfirmation`, guard `preserve_intake_decisions`). Replay of the real case: `lic` as of
`136fb83bf^` with `references.json` present and `reconfirms` in the adapter parses and reproduces
the committed references exactly apart from the two new blocks; without `reconfirms` it refuses.
The public release cannot carry it (reference allowlist); the database stores the record whole.
The committed `lic` data is not rewritten (Codex: no rollback needed).

**Codex item 3, qualifier shapes.** Every removal in `05d15bcd0` was listed with the text
between the number and the phrase (1,132 anchored; the 246 with a gap under 40 characters were
sorted by shape, the other 886 were not read). Restored, with tests beside the leakage
tests: the self-designation `de esta Ley,` (and `del presente Código`, …) between number and
qualifier, and exception connectors (`excepto`, `salvo el de la`, `con excepción de la/del`),
consistent with the accepted `salvo … su segundo párrafo`. An excepted part followed by its own
`que` clause is not attached (LFPC art. 66 leakage found in review). Relink: 10 instruments, 10
qualifiers added, none removed, no other field changed, Markdown identical, closure unchanged.
Second pass (advisor): `de la` / `del` between two qualifiers continues the chain (`inciso a) de la
fracción IV`, `párrafo segundo de la fracción I`), but never opens it (`3o. y de la fracción XXIX-F
del artículo 73`, LISR 88/151), and an opening `de los` still reaches the back-reference test
(REG-LMIN 33). Relink: 3 more qualifiers (reg-laero 9, cff 142, cub-dcg-2005 324), none removed; a
second `link --all` relinks 0. Known losses among the sorted shapes: qualifiers after `2o.-A` / `32-A` (the edge targets art. 2 or 32, not the
suffixed article; target defect, below), and qualifiers that scope a later citation.
Limitation: an exception qualifier reads the same as a positive one; there is no exclusion flag.
D-8 of the 10: all fractions found in their target except `cmpp` (edge already on the wrong
target, below) and `lacp` art. 10 fraction IX (merged mid-line in the corpus text).

**Follow-ups found (non-legal, agents' queue).**
1. Ordinal-suffixed articles `2o.-A`, `32-A` resolve to the bare number (about 46 citations).
2. `cmpp` art. 164 cites arts. 362–367 of another code ("deberes especiales de marinos") and is
   resolved to CMPP's own articles.
3. Fractions printed `VI.-` are not recognized by the Obsidian fraction anchors
   (`fraction_labels`); `lacp` art. 10 fraction IX is merged mid-line.
4. LGIPE: 20 suppressed articles (`[Artículo 80 Bis. Suprimido]` …) are folded into the
   preceding article; split them as their own provisions (parser fix with fixture).
5. MCP findings in `docs/mcp-agent-tests-2026-11-13.md` (instrument discovery, `release_id`
   contract, reviewer notes absent from the release).

**Decree-list pilot (Diputados `ref/<slug>.htm`, read-only).** Retained `.work/ref-pages/lic.htm`
(SHA-256 `68f53f62…`, ISO-8859-1) and `lieps.htm` (`84e10255…`). Each page lists every reform
decree (number, title, DOF date) and, for LIEPS, the SHCP quota *acuerdos*. Compared with the
decree groups in `reform-temporal-evidence.json` (date plus `decree-N` for same-day decrees):
- `lic`: 49 of 50 decrees. Its 11 `section-N` entries (1999-01-19, 2014-01-10) were read against the
  source; the ones checked belong to their own decree (per-article transitory blocks). Contents of
  the other groups were not checked. The missing #39 (DOF 13-08-2009) issued the Ley para Regular las
  Actividades de las Sociedades Cooperativas de Ahorro y Préstamo; the LIC PDF prints only its
  "Fracción adicionada" marks, not its transitories, so the gap is expected.
- `lieps` (held): the committed evidence files the yearly quota *acuerdos* under the preceding
  decree (2013-12-20 holds the 2017–2019 acuerdos; 2021-11-12 those of 2022–2025). The held
  reparse gives each acuerdo its own group under its own DOF date, matching the page; that is the
  correct segmentation. But it also drops the transitories of the 1984-12-31 and 1988-12-31
  decrees, which the page lists. Neither version is right; `lieps` stays held (agents' queue,
  fixtures from the retained text). Not a JRH question.
- `lieps`, full comparison (`.work/ref-pages/compare.py`, decree section of the page; the 13
  acuerdo dates all appear elsewhere on the page). Page: 46 decrees on 44 dates.
  - Both parses date one group `2013-12-20`, a date the page does not list. Its transitory ("El
    presente Decreto entrará en vigor conforme a lo siguiente…") sits under the header "Publicado en
    el Diario Oficial de la Federación el 15 de noviembre de 2016" (Ley de Ingresos 2017, page #43).
    Dating defect; #43 is therefore not an elision gap.
  - #27 (DOF 30-12-2002): printed with its Artículo Único; no transitories isolated. Unexplained.
  - #26: the PDF heads it "Publicada en el Diario Oficial de la Federación el 1º de enero de 2002"
    (ordinal sign `º`) and prints its transitories (fractions XIII, XIV …); neither parse has them.
    Likely the `1º` date form.
  - 1984-12-31 and 1988-12-31: both decrees are printed; the held parse drops their transitories
    (searched as substrings in every held entry: not folded elsewhere).
  Adoption criterion for `lieps`: every page decree whose transitories the PDF prints has a group
  under its own DOF date, acuerdos under their own dates, no group on a date the page lacks.
- A decree-list comparison script exists only in scratch; making it a read-only command
  (`check-decrees <slug>`) is the next step if more instruments are piloted.

**`lieps` grouping repair (2026-11-13, branch `fix/lieps-transitory-grouping`).** JRH set the
order: LIEPS first, then the structural pilot. Tracing the appendix against the retained text found
three mechanisms, all in `crates/lex-parse/src/diputados.rs`, plus one consequence:
1. *Erratum title after a page break* (1984-12-31, 1988-12-31; a regression from the
   title-with-note rule). `FE de erratas a la Ley…` merged into the preceding `ARTICULO UNICO`, and
   the block was then read as a new act, which dropped the transitory. Now an erratum title opening
   a page after a finished sentence starts its own block. A margin mark that carries its DOF date
   (`Fe de erratas al artículo DOF 21-12-1928`) is not a title.
2. *Law-named transitory headings* (2002-01-01 #26, 2002-12-30 #27). `Disposiciones Transitorias de
   la Ley …` and `Transitorios de la Ley …` were not section headers. They are now accepted in the
   reform appendix, as a short line with no closing punctuation. The ordinal `1º` was not the
   cause.
3. *Cited decree read as a title* (2013-12-11 #41, 2016-11-15 #43). A wrapped line beginning
   `Decreto por el que …` after a mid-sentence break (`…a que se refiere el`) was split off as a
   decree heading, and the date it cites replaced the real one. That is the source of the
   `2013-12-20` group. A title-case decree name now counts as a citation when the open block
   ends in a lowercase letter or a comma; uppercase `DECRETO ` is unchanged.
4. *Wrapping decree article* (consequence of 2, found in `lfd` and `lfpca`). A law-named section
   can open with the decree article that introduces a nested series (`ARTÍCULO SEXTO.- En relación
   con las modificaciones … se estará a lo siguiente:` then `Primero.` … `Sexto.`). An `Artículo`
   start followed at once by a bare `Primero` opens a new section, so the ordinals cannot collide.

Fixture `fixtures/diputados/lieps-reform-appendix-grouping-sample.txt` (provenance alongside) and
four tests. **Result for `lieps`:** all 46 page decrees have a group under their own date, and the
13 quota acuerdos each sit under their own date, all of which the page lists. No group falls on a
date the page lacks, so the adoption criterion is met. Reform evidence goes from 107 to 142 entries
(32 ids removed, 67 added, 5 texts changed). The committed 2015-11-18 and 2019-12-09 section-2
entries had held the following acuerdo's text, and the 1984/1988 transitories no longer carry the
erratum text. Provisions: the same 4 whitespace-only changes, as re-confirmed by JRH on 2026-11-12
(decision 3). The glossary profile hash, its 47 term offsets and the 8 usage-exclusion offsets in
art. 3 were refreshed by non-whitespace alignment, with the prior text bound to the reviewed hash.
The output keeps the same 58 terms and the same 315 usages by term, provision and text.

**Corpus-wide effect** (baseline and fixed parse of all 468 slugs in a scratch root; the same 3
failures as the baseline: `lfiiedb`, `socap-sofipo-dcg-2006`, and `lieps` before the refresh). 43
other instruments change:
- *Reform evidence, 34:* 217 entries added, 49 renamed (an omnibus decree's overall transitories
  become `section-N` once its law-named section is read), 4 removed, 27 texts changed. Examples:
  - `lfda` 2014-07-14 regains transitories Décimo Primero to Cuadragésimo Quinto, which had been
    cut at a cited decree;
  - `lagra`/`lota` 1993-07-09 and `cpf`/`lfpsdmh` 2018-06-01 transitories had stopped mid-sentence
    at `…del` and now run to their end;
  - new groups `ccf` 1985-02-07 and `lgeepa` 2018-06-05 come from their printed headers.
- *Provisions, 10 (`ldfefm`, `ldsca`, `lepepm`, `lih`, `liispcen`, `lmigra`, `locg`, `lpab`,
  `reg-lce`; `reg-lmigra` enactment evidence):* whitespace only. A false paragraph break before a
  cited `Decreto por el que` is rejoined. References and term usages shift offsets accordingly.
- After `derive-temporal`, `validate` and `export`, no temporal field and no validation report
  changes in any of the 44 instruments. No evidence id is referenced outside its own file.

The parse prints 142 transitories for `lieps` and these entries are inputs to temporal analysis.
No temporal model was run.

**Codex items 4 and 6 (accepted with follow-ups).** Item 4: the SCJN header predicate checks
neither header location nor instrument identity and accepts trailing text; reviewer approval stays
the authority, and the limit is recorded here (no change). Item 6: integration tests for removing a
persisted note and for a stale or missing note target are queued with the next `provision_notes`
change.

### Codex bounded re-review 2026-11-13

Report: `docs/four-goal-upgrade-codex-re-review-2026-11-13.md`, reviewed at
`5a2dc3673`. Item 3 accepted for the named qualifier shapes (independently
confirmed +13 qualifiers, 11 instruments, no other edge changes). Item 5 remains
change-requested: re-confirmation accepts relocation to a repeated identical
citation and does not bind persisted prior text to its approved hash. Both
reproduced against the compiled core. Native gate: 517 tests, strict clippy,
fmt and LRITF/IFPE/LIC validators passed. Next review action: repair both item 5
checks with negative regressions, then bounded re-review. Other work queues
and JRH decisions remain unchanged.

### Item 5 guard fixes — Codex follow-up 2026-11-13

At `6bf0f9615`, the prior-hash and repeated-occurrence fixes were independently
reviewed. The original compiled-core probes now refuse both cases with the
expected errors. Native fmt, strict clippy, workspace tests and LRITF/IFPE/LIC
validators pass (LIC retains two warnings). Evidence and final verdict:
`docs/four-goal-upgrade-codex-re-review-2026-11-13.md`, follow-up section.
The reviewed scope remains whitespace-only re-confirmation with exact citation
text and unchanged acquisition. Next action after review closure: resume the
existing LIEPS grouping work queue; legal decisions and other holds are unchanged.

**Parser overview for independent review (2026-11-13):** `docs/parser-overview-for-independent-review-2026-11-13.md`.
