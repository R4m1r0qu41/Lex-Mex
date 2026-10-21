# SQLite M2 upstream integration — 2026-10-20

## Integration and scope

The operator-supplied upstream repair handoff names main commit
`db48f899d7cfdbc59736291473a335cf434cfcaf`. Its decisions are recorded in
[the October 20 entry](decisions.md).
It was integrated by normal merge at
`6cb2dfe3ddcb8b389e370e4b3ad0c43298aae83e`, parents 611171e52 and db48f899d.
No squash/rebase occurred; seed source 6505baf57 remains an ancestor.

All previously dirty M2 files were backed up locally and verified byte-for-byte
unchanged immediately after merging. The DB corpus equals upstream exactly;
its provisions and references are unchanged by the repair. The October 18
hash-bound ruling remains byte-identical, and the seed validates all 17 accepted
rows, including historical lineage. No new human legal decision was inferred.
Main remains at db48f899d with its two unrelated untracked suggestion documents.

## Compatibility and discovered defect

- Updated the synthetic DB fixture to glossary strategy 4 and included a
  non-empty `invalidated_fractions` array. M2 parity tests now prove the field
  survives projection and detect its deletion.
- The complete synthetic release test uses strategy 4. Its invalid-publisher
  control now uses a fictional unsupported value: `secretaria_de_salud` is a
  valid publisher after the repair.
- The first repaired full run passed source validation, then caught a parity
  scope bug: reference-registry `source-manifest.json` was being checked as a
  law manifest. Insertion had correctly retained it as evidence only. Restricted
  law-manifest parity to `corpus/mx/`, added a reference-manifest regression,
  and included the missing path in diagnostics. That failed owned staging build
  was cleaned; it did not publish an artifact.

The new census is 468 instruments, 68,328 provisions, 35,949 reference edges,
5,727 terms and 102,432 usages. Five closure gaps and 17 accepted debt rows
remain. SCJN registry integration stays deferred; its three files are evidence
inputs only. This differs from the explicitly historical strategy-3 measurements
in the earlier M2 report. For current golden-query expectations, these new
term/usage counts and strategy 4 supersede the October 19 pins; the other
counts and three-outcome debt/coverage lookup semantics are unchanged. Preserve
optional non-empty `invalidated_fractions` independently from unread fractions
and repeal placeholders.

## Native checks

Final mechanical gate: formatting and full-workspace strict Clippy pass;
`cargo test --locked --workspace --quiet`: **392 passed**, zero failures
(42 + 3 + 32 + 2 + 6 + 9 + 6 + 15 + 273 + 4).
Required locked validators pass: LRITF 145 articles / 11 transitories / 126
references; IFPE 59 / 4 / 115; zero issues for both. The post-merge production
seed check validates 17 accepted rows. Corpus diff from merge HEAD is empty.

## Full build checkpoint

The full M2 build **exits 0** in 190.67 seconds. It completed
schema/shared validation, transactional inserts, external-snapshot raw/row/
supporting parity, integrity/FK/FTS checks, read-only reopen and immutable local
publication. All 4,960 selected files are retained: 3,287 normalized, 1,545
evidence-only and 128 explicit standards-body exclusions.

Validation artifact: `derived/db/m2-validation/38d0013e795906c3aec110c753b231607f10ca8ddce73b1112f3e56e327af464/corpus.sqlite`.
Release ID: `38d0013e795906c3aec110c753b231607f10ca8ddce73b1112f3e56e327af464`.
Database SHA-256: `02e08c68d187090ecfeaa2b3b11206aef84d53ea2ee781c374f8d6d5ce424656`.
Size: 473,972,736 bytes.
Logical-content SHA-256: `94aea32d1528b9b82808074bb24abdc22526d97f750fed64209af0e519f4ed80`.
[Machine-readable receipt](sqlite-m2-upstream-integration-2026-10-20-evidence.json).

| Table | Rows |
|---|---:|
| Instruments | 468 |
| Provisions | 68,328 |
| References | 35,949 |
| Defined terms | 5,727 |
| Term usages | 102,432 |
| Glossary coverage | 468 |
| Enactment evidence | 99 |
| Source manifests | 498 |
| Reference qualifiers | 12,360 |
| Operator reviews / outdated-reference approvals | 16 / 17 |
| Accepted debt / closure gaps | 17 / 5 |
| Standards catalogue | 32 |
| Evidence/exclusion artifacts | 1,673 |
| Pipeline coverage observations | 1,872 |

No whole-instrument reproducibility measurement is asserted: its table has zero
rows and the view retains `not_assessed`. Actual per-provision temporal/review
statuses are preserved separately.

The builder is unreferenced test Git snapshot
`bc26e6f00d94567eabf3f2ab9502b725e6b7d8d3`, matching the compiled M2 working sources;
the corpus input is the retained merge commit above. This is a local validation
artifact, not a final production release. After M2 review/commit, repin the
builder to its real commit and rebuild under M3/M4 gates. Full-corpus repeated
build/reproducibility has not been run; only the synthetic release has two-build
byte-identity evidence.

## Separate artifact inspection

A mechanical read-only inspection independently confirms the manifest/receipt
file checksum, all table counts, integrity `ok`, zero FK violations and 68,328
FTS rows. Read-only writes fail. All 468 coverage records use strategy 4.
LGAPPCEA retains invalidated fraction III; LGCS retains I, II, IV, VI, VII,
VIII, X and XII exactly. All three SCJN registry files occur in retained inputs
and evidence artifacts, with zero law-source-manifest rows. No SQLite sidecars
or owned `.building` directories remain. Measured size is 452.016 MiB, above
the historical M1 estimate; this is an observation, not a performance claim.
This inspection does not implement the M3 independent verifier.

## Remaining scope

Preserve the upstream open items: LAN short/title-containing usage matches and
XXV a./b.; LGCS's seven remaining terms/restored pre-2022 definitions. M2 preserves
committed facts and review state, and does not settle these questions.
M3 independent verification/query acceptance and M4 reproducibility remain
separate gates. This report records validation before the M2 implementation commit. The
operator subsequently authorized committing that implementation. The existing
artifact continues to identify its exact test builder snapshot; after the commit,
repin and rebuild under M3/M4. No push or merge into main.
