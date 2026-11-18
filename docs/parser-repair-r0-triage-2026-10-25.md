# Post-split repair R0 — triage, acquisition and parser candidate (2026-10-25)

Executes R0 of [post-split corpus and query repair](plans/post-split-corpus-and-query-repair.md)
(steps 2–4, plus the evidence R1 needs) at HEAD `e3ad94ee3`. Separate from the readiness note, which is left
unedited. **Nothing is merged, pushed or applied to the corpus.** The parser
candidate (two narrow edits, below) is committed locally as `dabeb0029` in the linked worktree
`/Users/jr/Dev/lex-mex-parser-repair` (branch `repair/itf-dcg-heading-separator-2026-10-25`,
created from `e3ad94ee3`). Review tier: advisor (Opus, same family) for the plan and the
fix design; no independent review yet. Machine evidence:
[evidence JSON](parser-repair-r0-triage-2026-10-25-evidence.json).

## Disposition of the historical nonzero stages

All replays ran in disposable roots on the frozen binaries B
(`5c4a3f29…`) and C3 (`31e21bc7…`), which agreed on every row. Input identity was checked
against the committed manifest before each run.

| Stage | Input | Replay result | Classification |
| --- | --- | --- | --- |
| `cub-dcg-2005` validate | Retained pair is a **newer edition** (PDF `c41084b3…`, 6,144,432 bytes; manifest pins `f38095eb…`, 4,866,388 bytes). Baseline pair not located. | 15 issues: 9 unresolved references to the 319 Bis family, 2 duplicate legend markers (325, 326), 4 warnings. | Mixed. The 319 Bis family and articles 140 and 175 Bis 4 are lost to a heading-separator parser defect (below). Markers 325/326 are defined twice in the new edition's source text (lines 46310–46318): a source issue, not a parser one. Edition choice stays an operator review. |
| `cucb-dcg-2004` validate | Exact pair | 1 error: reference to `117-bis-5`, which the parse lost | Genuine parser defect (same family) |
| `oaac-dcg-2009` validate | Exact pair | 3 errors: reference to `105-bis`; `12 Bis`, `105 Bis` and `106` also lost | Genuine parser defect (same family) |
| `scap-dcg-2012` validate | Exact pair | 1 error: mark `(261)` not in legend. Committed provision `202` still carries inline `(261)`; the replay strips it. | Held marginal-number drift already recorded in the heading-leak acceptance. Not fixed here. |
| `laero` validate | Exact pair | Frozen 100 articles, found 109. The 9 extras are exactly `73-decies` … `73-octodecies`. | Stale frozen count tied to the 9 active debt rows. Resolves through R1. `expected_articles` untouched. |
| `cjm` validate | Exact pair | Frozen 952, found 954. Extras are `30-sextus` and `30-septimus`; the source has real `Artículo 30 Sextus.-` and `Séptimus.-` headings. | Stale committed corpus and count (the current parser splits the headings). A reparse is an operator decision; no debt rows exist for them. |
| `oga` temporal-request | Exact pair | Parse and validate pass; the request stops: "requires an exact publication day; serialized range is not a day". Adapter has no `publication_date`. | Temporal-request precondition, a metadata gap for R2. Not a parser defect. |
| `lfiiedb` parse | Exact pair, recovered earlier | See [corrected-input replay](lfiiedb-corrected-input-replay-2026-10-25.json) | Resolved wrong-input failure |
| `lftr` validate | Pair located (`.work/federal-held-reprocess/candidates/lftr`), exact | With the successor `lmtr` staged, parse passes; validate fails only on the frozen count (324, found 327). The extras are `180-quintes`, `180-sextus`, `180-septimus` (the source spells the first `Quintes`). | Stale frozen count and committed corpus; the current parser already splits three more merged headings. Not in the debt rows. |

The 106 missing inputs are addressed in the next section.

## First parser candidate: itf-dcg article-heading separator

**Defect.** `parse_main_text` in `crates/lex-parse/src/itf.rs` accepted only a joined `.-`
after the article number. Retained sources also write `Artículo 105 Bis. - `,
`Artículo 106- `, `Artículo 117 Bis 5- ` and, in the new CUB edition, `Artículo 319 Bis. – `.
Those headings were not recognized, so each article's text was appended to the article
before it. The committed corpus has the same loss.

**Change.** Only the separator: `\s*\.-\s*` became `\s*\.?\s*[-–]\s*`. No other form is added.
A census over every retained itf-dcg source found exactly 11 lines newly matched, all of
them real article headings; the A/B below confirms it.

**Tests.** `heading_separators_seen_in_retained_sources_open_an_article` (positive, built
from the exact-pair and new-edition forms) fails on the old regex and passes on the
new one. `body_references_at_a_line_start_do_not_open_an_article` (real line-start body
references: `Artículo 319 Bis, que…`, `…de estas Disposiciones…`, `…a través de…`,
`Artículos 319 Bis a 319 Bis 5…`) passes on both, so the new regex adds no false match.

**Checks run in the worktree for the first edit** (via the mech subagent, raw output in the job log): `cargo fmt --check`
pass; strict workspace/all-target Clippy pass; `cargo test --workspace`: 432 passed (430 plus
the two new tests), 0 failed. The subagent could not capture the script's exit status
under zsh, so the printed results above are the evidence. Not run: the locked LRITF/IFPE
validators, DB verify and goldens. The change is confined to the itf-dcg parser and the
committed corpus is untouched, but that is a reasoned claim here, not a run.

**A/B against C3** (C3 parser sources equal HEAD, so this isolates the edit). Candidate
binary `062d87c8…`, parse and validate on identical disposable roots, all output files compared:

| Instrument | Result |
| --- | --- |
| `cucb-dcg-2004` | `117-bis-5` added; its text removed from `117-bis-4`. Validation now valid. |
| `oaac-dcg-2009` | `12-bis`, `105-bis`, `106` added; text removed from `12` and `105`. Validation now valid. |
| `cub-dcg-2005` (new edition, not baseline) | `140`, `175-bis-4`, `319-bis`, `-1`, `-2`, `-3`, `-5` added; text removed from `319-bis-4`. The gap `139 Bis 7→141` closes; `249→251` stays. The 9 unresolved-reference errors disappear. Still invalid on the 2 duplicate legend markers (a source issue). |
| `scap-dcg-2012`, `cue-dcg-2003`, `fi-dcg-2014`, `itf-dcg-2018` | **Zero changed bytes** in every output file. (`itf-dcg-2018` needed the LRITF corpus staged for its additive glossary.) |

In total, 11 provisions are added and none removed. In each changed article the new text is
an exact prefix of the old text, so the moved span is exactly the tail that begins at the
new heading. Reference counts against the committed corpus are not compared: the disposable
roots do not run `link`.

**Coverage limits.** `servinv-dcg-2013` and `socap-sofipo-dcg-2006` have no retained source;
they are covered only by the unit tests. The `cub` result is on a changed edition and does
not say what the baseline edition would give. The DIPUTADOS and `ifpe-dcg` parsers have their
own heading code and are not touched. Date/legal effect of any added article is not assessed.

## Missing retained inputs: acquisition (R0 step 2)

Each case's pinned official URL was fetched once, sequentially, into a **new** directory
(`.work/parser-repair-20261025/acquisition/<slug>/`, with response headers, status and
fetch time), extracted with the pipeline's own flags (`pdftotext -layout`, plus `-nopgbrk`
except for the CNBV DCG parsers) under pdftotext 26.09.0, and compared with the committed
manifest. Nothing under `corpus/` or the existing `.work/<slug>` trees was written.

| Status | Count | Cases |
| --- | ---: | --- |
| Exact pair recovered (source and text hashes both equal the manifest) | 102 | all remaining missing cases except those below |
| Changed edition (source hash differs) | 5 | `ligie`, `reg-laero`, `reg-lfda`, `socap-sofipo-dcg-2006` and `cub-dcg-2005` (now 6,159,458 bytes, a third distinct hash) |
| Unavailable URL, source only, awaiting extraction reproduction | 0 | |

So of the 106 historical gaps, 102 are closed and 4 are changed editions; with `cub-dcg-2005`
that is 5 changed editions that need separate acquisition/ingestion review and are **not**
admitted. `lfiiedb` was recovered earlier. The recovered pairs are staged for replay only.

## Replay of the recovered 102 on B, C3 and the candidate

Disposable roots, identical HEAD adapters/schemas, parse + validate, every output file hashed.
Where a regulation or glossary needs another instrument, the committed parent (or the full
committed corpus, cloned copy-on-write) was staged.

- **Baseline vs the refactor: B and C3 are byte-identical on all 102** (and on all 38 context
  reruns). The original A/B's missing-input gap (the 106 uncovered instruments) is now
  covered for the 102 recovered ones. Where a run fails, it fails the same way on both.
- **Candidate vs C3: identical on 101 of 102.** The one difference is `lscs`, which fails
  closed at C3 and parses at the candidate (below). No other output file changed.
- Outcomes at baseline: 61 clean in a bare root; 34 more clean once the parent law or
  committed state was staged (35 regulations needed their parent; `lpdusf` and
  `reg-lgeepa-maaa` needed cross-instrument state); 3 parse but fail validation on a stale
  frozen count (`lfd`, `lfdo`, `lfpa`); 2 fail closed in the parser (`lscs`, `lrascap`).

### Second candidate edit: plural decree publication note (`lscs`)

`extract_reform_evidence` (`crates/lex-parse/src/diputados.rs`) read only `Publicado/Publicada`
after a decree heading. LSCS heads its 1966 decree `REFORMAS a la Ley … Publicadas en el
Diario Oficial de la Federación el 5 de enero de 1966`, so its transitories had no date and
the whole parse failed. The reform-evidence regex now also accepts `Publicados/Publicadas`
(the separate header-note regex at the top of the file is unchanged). With the full corpus
staged, `lscs` parses and validates (210 articles, 0 issues); provision IDs equal the
committed set, but the new run isolates **16** reform transitories where the committed
`reform-temporal-evidence.json` holds 2, and 5 provision texts, one reference and the
evidence file differ. Those differences are temporal-evidence data, left for R2 review;
the committed record is untouched. Tests: a positive test and a test that a plural
citation inside an operative article does not redate the decree. Both fail on the old
regex. (The second fails on the old regex because the decree then has no date at all, so
it is a weaker negative than the first edit's.) Plural `publicados/publicadas en el Diario
Oficial` also occurs in body-text citations in 125 retained files; the 101-of-102 identical
result is the evidence that the wider regex does not redate anything else.

### `lrascap`: a committed-corpus structure loss; fix needs an ID-scheme decision

The pinned source fails closed identically on B, C3 and the candidate. Accepting the exact header
`TRANSITORIOS DEL DECRETO` only moves the parse to a second guard, so I reverted that edit. The
committed record explains why: `lrascap` has 144 articles and **0 transitories**, and the law's
own transitory block (headed `TRANSITORIOS DEL ARTÍCULO PRIMERO`, PRIMERO through DÉCIMO
QUINTO) sits inside article `132`, whose text is 39,289 characters. The overall decree then has
a second layer (`TRANSITORIOS DEL DECRETO`) that repeats the ordinals PRIMERO to QUINTO, which
the current provision-ID scheme cannot hold without a layer qualifier. Recovering the
transitories therefore needs a decision on how enactment-layer transitories are identified.
Not fixed; recorded for the operator.

## R1 technical candidate: the four clusters reparsed (nothing committed)

The baseline parser already splits all 14 debt headings, so no heading rule is needed. I reparsed
the four instruments from their exact recovered sources in a copy-on-write clone of the committed
corpus (`.work/parser-repair-20261025/r1-clone`), with the four frozen `expected_articles`
counts updated in the worktree (patch `r1-adapter-counts.patch`: laero 100→109, lfd 784→787,
lfdo 56→57, lfpa 118→119), and ran parse, link, derive-temporal and validate. All four validate
(`lfd` keeps its one `suffix_order` warning). The committed corpus is untouched.

| Instrument | Provisions | Added | Removed | Order | Other changes |
| --- | --- | --- | --- | --- | --- |
| `laero` | 107→116 | `73-decies` … `73-octodecies` (9) | 0 | kept | 5 whitespace-only, 3 trailing-tail removals (leaked section headings, signature block), 1 mixed (`article:18`); `heading_context` changes on 28 provisions; references 36→35 |
| `lfd` | 793→796 | `257-sextus`, `-septimus`, `-octavus` | 0 | kept | 2 tail removals (a leaked heading, the signature block); references 377→373 |
| `lfdo` | 57→58 | `12-quintus` | 0 | kept | 2 whitespace-only, 2 tail removals; references 65→64 (22 ids replaced by 21 as spans moved) |
| `lfpa` | 122→123 | `69-c-bis` | 0 | kept | 1 whitespace-only, 5 tail removals; `heading_context` changes on 17; references 18→16 |

- **Dropped references are spurious.** The four removed `lfd` edges and the `lfpa` `69-c` edge
  were self-citations created by the merged heading text itself (`Artículo 257 Sextus. (Se
  deroga)` read as a reference to article 257); the others lie in removed signature blocks
  (a citation of CPEUM article 89). No reference gained a new target.
- **Temporal state:** shared provisions keep their determinations (the status counts differ
  only by the added provisions). New provisions get `deterministic_rule` states: effective
  at 0.9 for `73-decies`, `12-quintus` and `69-c-bis`; repealed at 1.0 for `257-sextus`.
  No legal-review or audited decision changed.
- **No other instrument changed.** `link` ran per instrument, not `--all`, so backlinks from
  other instruments into the added articles are not recomputed here.
- **Not done:** the retirement seeds for the 14 rows. They record a resolving corpus commit,
  which exists only after the reparsed files are committed; the plan requires your scoped
  authorization for that. `cjm` (+2) and `lftr` (+3) are the same situation without debt rows.

## Decisions for the operator

1. **The committed corpus is missing real articles.** Exact-pair evidence: `cucb` 117 Bis 5;
   `oaac` 12 Bis, 105 Bis and 106. Inferred only, because the baseline pair is missing:
   `cub` 140, 175 Bis 4 and the 319 Bis family. None is in `_known-structural-debt.json`.
   The earlier decision to hold the `oaac` and `scap` reparses cited "no gain"; that reason
   no longer holds for `oaac`. Options: reparse `cucb` and `oaac` with the repaired parser
   (including the marginal-number drift the same reparse brings), or register the losses as
   new structural debt. Neither is done.
2. **`cub-dcg-2005`:** the pinned baseline edition cannot be re-derived from local files.
   Acquiring the pinned edition or admitting the newer one is a separate acquisition and
   re-ingestion review.
3. **R1:** the reparsed candidate and its diff are ready for review. Authorize committing the
   reparsed `laero`, `lfd`, `lfdo`, `lfpa` (with the count updates) and then the 14 retirement
   seeds? `cjm` and `lftr` would follow the same path.
   `lrascap`: decide how enactment-layer transitories are identified (see above).
4. **Changed editions** (`ligie`, `reg-laero`, `reg-lfda`, `socap-sofipo-dcg-2006`,
   `cub-dcg-2005`): which, if any, to admit as new editions. Not done.
5. **`lscs`:** the candidate parse gives 16 reform transitories against 2 committed; whether to
   adopt that is an R2 temporal-evidence decision.
6. Whether to commit the parser candidate, which this plan does not imply.

## Next

Independent review of the candidate (a Codex handoff is queued), then the operator's
choices above. R1 needs your authorization to reparse and retire; R2 follows R1; R3 (public
query) is not started.
