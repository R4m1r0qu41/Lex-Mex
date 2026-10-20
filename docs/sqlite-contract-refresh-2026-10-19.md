# SQLite contract and debt refresh — 2026-10-19

> Historical milestone record. Current M1 review repairs, seed v2, per-file
> provenance and expanded inventory are documented in the
> [review response](sqlite-m1-review-response-2026-10-20.md).

## Outcome and authority

This refresh replaces the current census, debt inventory and query expectations of the October 17 plan. The original M1 physical-layout measurement remains historical evidence; extras-only storage and independent reconstruction remain the contract. No DB, Rust, corpus, parser, adapter or legal-review record is changed here.

- Corpus input baseline: `bfcda816f46b8b0c1d694fc1f2cfb8b4438275b2` on main.
- Reviewed parser sweep: `d8ecaeaa009380dd848d66e105d00b0079a1e2e2`, documented in `docs/parser-sweep-review-2026-10-19.md` at the input commit.
- Follow-up `bfcda816f` repairs the mixed-delimiter finding, changes six terms, adds 28 usages and bumps the strategy to `glossary-extraction-3`. The lower-case-transitory restriction remains deliberately fail-closed. This refresh inspects those contracts; it does not claim a new independent full parser acceptance run for that follow-up.
- DB branch baseline: `04cc039af7d192732489cc5f3d16c4cb6340e385`. Its clean initial state was verified. This refresh does not rebase, merge, rewrite history or move HEAD. Before implementing against the current corpus contracts, integrate main in a separate reviewed step; do not build using the old branch's types while naming the new input. The OID above is the reconciliation baseline, not yet a buildable release commit: the final `--corpus-commit` must also contain the validated production seed, schemas and all required inputs. Verify the locators again at that final commit; do not mix its seed with another commit’s corpus.
- Evidence: [census and reconciliation](sqlite-contract-refresh-2026-10-19.json), [current candidates](sqlite-structural-debt-candidates-2026-10-19.json), [current reviewed seed](sqlite-structural-debt-review-2026-10-19.json).

## Current census

| Record | Count |
|---|---:|
| Federal instruments | 468 |
| Provisions | 68,328 |
| Articles / transitories / annexes | 64,822 / 3,476 / 30 |
| Reference edges | 35,949 |
| Resolved / approved outdated / unavailable historical | 35,928 / 17 / 4 |
| Direct / relative / range expansion | 31,927 / 2,625 / 1,397 |
| Qualifiers: fraction / paragraph / subsection | 8,712 / 2,837 / 811 |
| Official-title source anchors | 15 |
| Terms / usages | 5,734 / 102,204 |
| Approval objects / operator-review objects | 17 / 16 |
| Edges with no recorded resolution rule | 2,606 |
| Glossaries: configured / detected / held / none detected | 18 / 299 / 6 / 145 |
| Accepted closure gaps | 5 |

Counts describe this immutable input, not permanent constraints on future releases. The remaining gaps are cub-dcg-2005, cucb-dcg-2004, lftr, oaac-dcg-2009 and socap-sofipo-dcg-2006. Preserve their exact errors and multiplicity from the pinned allowlist. CPF, LGS and LMV are no longer accepted-gap instruments. CPF 209 Quintus, LIC 112 Quintus and LGTOC 228-B/228-H now exist as separate provisions. The operator confirmation of the corrected LGS premise remains a documentary legal-review follow-up; the database neither reverts canonical data nor invents that confirmation.

## Structural-debt reconciliation

The unchanged v2 detector finds 57 current candidates versus 117 historically. The refresh script maps identities to the old reviewed candidate set; it does not infer a new operator ruling. Every surviving heading must preserve its exact spelling and bracketed-body classification, and its new source locator/hash is checked against pinned Git blobs.

| Status | Rows | Treatment |
|---|---:|---|
| Accepted remaining merged headings | 17 | Eligible for the production seed under the recorded October 18 acceptance |
| Rejected legally removed text | 30 | 27 placeholders plus three LFTR bracketed-body headings; never load as structural debt |
| Deferred DCG headings | 10 | Preserve as deferred; never silently promote |
| Resolved historical debt | 60 | Each now has exactly one separate article; kept in reconciliation history, excluded from active seed |

The 17 remaining accepted rows are laero (9), lfd (3), lfdo (1), lfpa (1), lgs (3). Their source hash/blob, provision ID and exclusive Unicode offsets all validate. No new accepted row or reviewer identity was invented. `reviewer: operator`, `reviewed_at: 2026-10-18` references the recorded operator decision, not this refresh date.

The current reviewed seed remains a documentary artifact until M1 supplies its production schema/type/validator/fixture and working consumer together. At that point promote only these 17 accepted rows to `adapters/shared/_known-structural-debt.json`; preserve/count rejected and deferred dispositions in the input inventory. The builder reads that required seed at its full pinned input commit; a missing seed is a hard failure, never an empty inventory. This refresh does not create an unvalidated production boundary in advance of its consumer.

This is an embedded-article-heading seed, not exhaustive corpus debt. The source decision also records 492 trailing heading leaks in 48 instruments, 40 instruments without retained text for that task, CNBV drift and unread separator forms. These are sourced aggregate observations, not 492 fabricated accepted rows. Inventory the pinned decision document and expose these qualifications separately. Missing article lookup must not treat this seed's absence as proof of completeness. Do not reuse the historical count of 51 false self-links as a current measurement.

## Glossary preservation contract

1. `glossary-coverage.json` is a first-class canonical input per instrument, with one reconstructible coverage record. The builder requires a supported `schema_version` and strategy; current strategy is `glossary-extraction-3`. Older strategies need an explicit supported mapping, never a silent upgrade.
2. Keep all keys and exact values: status, strategy, provision locators, method, fractions, imports and their counts, hold code/message, counts, profile, usage form, repeal placeholders and unread fractions. Preserve list order and unknown fields through raw input and extras according to the M1 rules. Coverage/profile structures with closed schemas still validate against the schema at the input commit.
3. Preserve `DefinedTerm.inciso` together with `fraction`. Their optional presence remains in extras; queryable generated columns may expose them. Do not confuse a definition's inciso with the serialized reference qualifier type `subsection`.
4. Preserve source-bound profile text hashes, fraction dispositions, term spans, ignored candidates, inciso marker spans and cross-reference locators. Preserve adapter `usage_form` (`quoted` or `with_lowercase`) and the mirrored coverage value. Matching a lower-case usage does not let the DB infer another legal definition or scope.
5. Absence means unmeasured/unrecorded; an empty array means measured zero; explicit null is distinct from absence. In particular, absent `unread_fractions` does not mean complete extraction. `none_detected` is a detector outcome; `held` is not no glossary. `detected` with unread fractions must remain visibly incomplete. Use presence projections (`json_type`/equivalent) as well as `json_extract` when a query needs that distinction.
6. Enforce term/usage/coverage parity against pinned input. Validate all live term targets and exact usage spans. Glossary-only refresh on an accepted-gap instrument does not make its frozen graph current; retain its independent closure state. The DB does not rerun extraction, link or resolve holds.
7. Do not materialize full duplicate per-row payloads. Raw bytes plus mapped columns/extras/ordered children reconstruct the complete logical record. A coverage view can project nested data without discarding any fields.

## Updated golden-query expectations

- G1: 126 reg-lan parent-shorthand edges still target LAN. Article 1 now has four defined terms (`Ley`, `Reglamento`, `La Comisión`, `Registro`); the historical missing-definition claim is superseded. Compare exact stored term/usage rows, including quoted usage form.
- G2–G4: LIC's approved historical LFCE 31 Bis citation, its independent operator note and the four unavailable LRSIC historical edges remain protected. Read exact IDs and review payloads from pinned inputs, not a guessed replacement.
- G5: use LFTR → LGTAIP 180 Bis, multiplicity 1, for `cited_but_missing`. CPF → LIC 112 Quintus must no longer appear in that view.
- G6a: LGTOC 228-B now returns found at `urn:lex-mx:federal:statute:lgtoc:article:228-b`; no active debt qualification for it.
- G6b: LFD 257 Sextus returns not separately parsed, with `urn:lex-mx:federal:statute:lfd:article:257-quintus`, heading chars [100, 121), source hash and October 18 decision.
- G7/G8: retain accent-equivalent lexical search and exact Terdecies lookup. BM25 never substitutes for identity lookup.
- Add glossary boundary fixtures: an inciso-defined term, `with_lowercase` and `quoted` usage forms, a held glossary, a detected glossary with unread fractions, absent versus empty accounting arrays, an imported term and a rejected graph with current glossary coverage. Mutation must prove dropped optional metadata or changed order fails parity.

## Four-stage boundary and reproducibility

Project acquired/structured/linked/analyzed facts independently. Structural inclusion is supported by committed provisions/validation; source manifests preserve acquisition provenance but do not establish durable PDF retention or current upstream freshness. Accepted-gap membership qualifies graph refresh. Temporal/review states remain as stored, including `not_analyzed`. No derived overall 'complete' or 'effective' flag is permitted.

The October 17 full-payload prototype size (333.7 MB extras-only) is historical and must be remeasured against this larger input. The October 19 review has two bounded source-replay measurements at d8ecaeaa0, not a complete reproducibility assessment of bfcda816f. Do not promote those to whole-instrument reproducibility rows without a scope-aware evidence contract. Retain `not_assessed` for unmeasured scopes, preserve the review evidence as a documentary input, and do not infer measurements from local source availability.

## Verification and recovery

Regenerate from pinned blobs using:

```bash
python3 -I prompts/refresh-sqlite-debt.py /path/to/lex-mex bfcda816f46b8b0c1d694fc1f2cfb8b4438275b2 /private/tmp/sqlite-refresh-verification
```

Compare the three output JSON files byte for byte with this worktree's documents. Independently check every current row's source blob/hash/span and every resolved row's current provision, status counts, current gaps, census and query pins. Failure stops the refresh; do not patch canonical text or relabel a row to make it pass. Preserve the October 17 files as historical evidence. No parser rerun, relink, temporal model, DB build or migration runs in this task.

Verification completed by a separate mechanical worker: the final helper regenerated all three JSON artifacts byte-identically; all 57 rows passed independent blob OID/SHA-256/Unicode-span checks; all 60 resolved rows map to exactly one standalone article; the five gap slugs match; relative Markdown links resolve; `git diff --check` passes. No bytecode cache was produced and no crates/corpus/adapters/Cargo path changed. Rust tests were not rerun for this documentary refresh; the helper and input-binding checks are the applicable validation. The three generated documents are worktree files, not yet committed artifacts.

Exactly one next action: implement M1 against integrated current main and this refreshed contract, promoting the 17 accepted seed rows with schema/type/validator/fixture and the working consumer in the same change. Integration and implementation follow a separate execution instruction; this refresh does not rewrite the database branch.
