# Post-split corpus repair — record (2026-10-25)

Execution record for R0 and R1 of [post-split corpus and query repair](plans/post-split-corpus-and-query-repair.md)
and the five edition re-ingestions, under the operator instruction of 2026-10-25. Local branch
`repair/itf-dcg-heading-separator-2026-10-25`; **nothing is merged or pushed**. Review tier: advisor
(Opus, same family) for the plan and the `lscs` count; **no independent review yet** (Codex handoff
extended). Machine evidence: [post-split-corpus-repair-2026-10-25-evidence.json](post-split-corpus-repair-2026-10-25-evidence.json).
The decision is recorded in `docs/decisions.md` under 2026-10-25.

## Commits (normal history, in order)

| Commit | Content |
| --- | --- |
| `dabeb0029` | Parser: `itf-dcg` article heading separator; plural decree publication note (tests added) |
| `47eabbb21` | Reparse `laero`, `lfd`, `lfdo`, `lfpa`; frozen counts 100/784/56/118 to 109/787/57/119 |
| `050660b5a` | Reparse `cucb-dcg-2004`, `oaac-dcg-2009`; remove their two entries from the closure accepted gaps |
| `730a8b16c` | Retire the 14 merged-heading debt rows, naming `47eabbb21` |
| `600ea9a60` | Reparse `lscs` |
| `c0b938181` | Re-ingest `reg-laero` |
| `b9f49d9d6` | Re-ingest `reg-lfda` (frozen count 188 to 196) |
| `ef28b9619` | Re-ingest `ligie` |
| `223700bf9` | Record the corpus repair (decision, status, this record) |
| `7df6203fe` | Validator: an identical repeated legend entry is a warning (revertible) |
| `5399d1762` | Re-ingest `cub-dcg-2005`; remove its closure accepted-gap record |

Built and run with the binary from `dabeb0029` (SHA-256 `cb575827…`); `crates/` is unchanged since.

## What changed in the corpus

- **R1 four.** 14 provisions added (`laero` 73 Decies to Octodecies, `lfd` 257 Sextus to Octavus,
  `lfdo` 12 Quintus, `lfpa` 69 C Bis); none removed; order kept. Removed text is leaked next-section
  headings, signature blocks and spurious self-citations created by the merged heading itself
  (for example `Artículo 257 Sextus. (Se deroga)` read as a reference to article 257). Reproduced
  byte-for-byte against the earlier clone candidate. Shared provisions keep their temporal
  determinations; no review decision changed.
- **cucb, oaac.** `117 Bis 5` (cucb) and `12 Bis`, `105 Bis`, `106` (oaac) recovered. Same reparse
  also removes stray `(N)` margin numbers from article text, records the amendment legend
  separately, re-derives glossary terms (cucb 27 to 47, oaac 59 to 62) and leaves
  `effective_from` unset for amended articles (35 and 22) under the deterministic rule instead of
  asserting the original commencement date. Both now have no unresolved references.
- **lscs.** 16 reform transitories, checked by hand against the appendix of the source:
  1966 (3), 2002 (1), 2006 (5), 2009 (2), 2012 (2), 2013 (3). The 1935 and 1946 errata notices
  and the 1946 decree contain none. The two committed 2012 items are byte-identical in the new
  record and nothing else in the corpus referenced them. 16 counts extracted evidence, not
  applicability: the 2006 decree amends three laws and the 2013 decree issues the LISF, so which
  transitories govern LSCS is temporal review (R2). Article 207 ("Se deroga el título VII … del
  Código de Comercio") moves from `repealed` (deterministic, 1.0) to `unknown`: it repeals other
  law, not itself.
- **reg-laero.** 191 articles, 9 transitories (unchanged counts); 13 substantive text edits (24 more
  are whitespace) from the 2025-08-29 and 2026-06-09 reforms; evidence for five decree dates it
  lacked. **reg-lfda.** Adds `37 Bis` to `37 Nonies`; 196 articles; latest reform 2026-09-24.
  **ligie.** Same 9 provisions; evidence 11 to 16 decree dates up to 2026-10-13, while the
  publisher header still says Última Reforma 29-12-2025, so `latest_reform_date` trails the newest
  decree. None of the three had review or audited state.

## cub-dcg-2005 and the held edition

**cub-dcg-2005** was first held because its current edition (`b03eb544…`, 6,159,458 bytes, equal to the acquisition hash)
prints legend markers 325 and 326 twice with identical text. Following your instruction to re-ingest all five, `7df6203fe`
adds a narrow rule: an identical repeat is read once and disclosed as `duplicate_amendment_marker_identical`; a differing
repeat stays an error and a missing entry stays an error. It is a validator change at a trusted boundary in its own commit;
revert it to restore the strict rule. Under it, `5399d1762` re-ingests `cub`: 722 provisions (712 articles, 10 transitories);
recovers `140`, `175 Bis 4` and the `319 Bis` family; 474 text changes, of which 454 are margin-number removals, 1 whitespace
and 19 substantive updates of the resolutions of 2026-07-14 and 2026-09-01 (latest reform 2026-07-03 to 2026-09-01); one
provision moves `effective` to `repealed`; no review state existed; two disclosed warnings; the gaps `139 Bis 7→141` close
and `249→251` stays.

**socap-sofipo-dcg-2006** stays **held** (`784be7c5…`, 3,670,327 bytes): its legend jumps from 90) to 92) while the text uses
mark (91) six times. A missing legend entry cannot be invented, so `unknown_amendment_mark` stays an error. The attempt is kept
under `.work/parser-repair-20261025/socap-attempt-1`; the tree was restored to the committed state.

## Checks run

- Locked `validate lritf` and `validate ifpe-dcg-2021`: valid, 0 issues (145/11/126 and 59/4/115).
  First time run on this branch.
- fmt, clippy (1.95 default and 1.99.0), 436 workspace tests (434 plus two validator tests): pass.
- All 11 touched instruments validate (warnings only: `lfd` 1, `cucb` 2, `oaac` 3, `cub` 5 including the two disclosed repeats).
- `link --all` on a copy-on-write clone: exit 0, 0 instruments relinked, 36,073 edges unchanged,
  the 2 rejections (`lftr`, `socap-sofipo-dcg-2006`) equal the recorded accepted gaps. The per-instrument link did not move any
  other instrument's edge.
- Two fresh `db project` runs from `5399d1762`: byte-identical database and manifest (release
  `3c67a66b…`, database `2417cca3…`, logical `673379d4…`); both pass `db verify`.
  Census: 68,374 provisions (+33 over the accepted 68,341: 14 splits, 1 cucb, 3 oaac, 8 reg-lfda, 7 cub), 36,073 edges,
  5,985 terms, 102,278 usages, 0 active and 17 retired debt rows.

## Golden queries: 19 of 21 pass; two need a decision

Not edited, per the plan. Both differences are exactly explained:

| Golden | Now | Proposed replacement |
| --- | --- | --- |
| `G6 active LFD structural debt` | the row is retired, so the query returns no rows | a count of 1 for `lfd:article:257-sextus` in `provisions`, like `G6 LGTOC separate article` |
| `G5 complete cited-but-missing inventory` | `[[2, 4]]`, was `[[6, 17]]` | expect `[[2, 4]]` (the cucb, oaac and cub references resolved; only `lftr` and `socap` remain) |

`db golden-queries` fails until these are decided.

## Not done

`socap-sofipo-dcg-2006` is held. `cjm` (+2) and `lftr` (+3) were out of this instruction; R2 (temporal queue), the R3 remainder
(database-backed serving, output bounds, benchmark) and independent review are open. DuckDB is
being tested elsewhere and is not part of this record.
