# Parser split A/B review — 2026-10-24

Verdict: **A/B parity verified for the covered inputs.** No new production
regression found. This is an intermediate review, not G4/G5 or full corpus
parse-coverage acceptance. Claude may proceed with the planned G4 DB evidence
while retaining the coverage limitations explicitly.

Reviewed receipt commit: `b3c7e961fcfbc9390d24ae8cd75958bc57243942` in
`/Users/jr/Dev/lex-mex-parser-split`, clean when checked. Production code is
unchanged from tested C3 `1250889ce`; baseline B is `85c41f389`.
Evidence: [machine report](parser-split-ab-codex-review-2026-10-24-evidence.json).

## Independently verified

- All 27 queue steps ended with rc=0; queue state contains 27 completed entries,
  final COMPLETE marker present, no stop marker.
- All 15 raw runs have true unchanged/frozen/committed harness and unchanged
  binary flags, false live-source-change flag, and the same harness SHA-256
  `fa321e4659694f9d6daa78820842d3a6e36f9bf8e5a60369c391d5eca014044e`.
  Frozen binaries and harness snapshot hash to the committed evidence values.
- Independently compared every command row (excluding elapsed time only) and
  saved stage file-hash map in all 12 pairs: B3/B4 and B4/C3, B4/C2, B4/C1,
  each for retained, sweep and supplement. Zero differences. Each comparison
  has complete evidence, no legacy flag, no notes and correct binary/harness
  bindings. Formats, input identities and final root/corpus digests agree.
- Recomputed stage-map digests and hashed all **44,660 raw stdout/stderr files**
  against recorded digests, covering **22,330 command rows**. Rehashed all 1,307
  frozen retained input files and 21 supplementary files; both frozen input-tree
  identities match their manifests and run records.
- Each variant has 1,771 retained rows (8 nonzero), 2,660 sweep rows (0 nonzero),
  and 35 supplementary rows (1 nonzero). Failures reproduce baseline exactly.
- Corpus, schemas, accepted decisions and database fixtures are unchanged from
  B. Production paths are unchanged from C3 through the reviewed receipt.
  `git diff --check 366cef07f b3c7e961f` passes.

No Cargo checks, full replay, harness self-test or DB builds were rerun here.
The previously recorded native/self-test results remain separate evidence.
Historical stage outputs are represented by retained maps/digests; their original
filesystem roots were overwritten by sequential runs. Equality of those saved
maps is verified, not a fresh reconstruction of every historical output file.

## Limits and follow-up

1. **Coverage:** 106 instruments have no retained parse inputs. Two replayed
   inputs (`cub-dcg-2005`, `lfiiedb`) are not provenance-matched. The sweep gives
   downstream parity but does not replace parse replay. Keep these as explicit
   limitations; no blanket full-coverage acceptance or source refetch is implied.
2. **Existing failures:** eight retained and one supplementary command failure
   reproduce identically. Preserve their deferred diagnoses. Zero regression
   does not establish that baseline ingestion is correct.
3. **Date wording (documentation):** the receipt's claim that shared clock/window
   means date dependence cannot hide a difference is too strong. The queue
   crossed UTC midnight, and equality for one execution window does not prove
   equality for all dates. Narrow the statement to the observed exact outputs
   in the tested window; all-date equivalence also relies on the reviewed
   behavior-preserving code moves. This does not invalidate the measured result.
4. **R-001:** the uninterrupted queue completed, so the restart defect did not
   affect these results. Repair separately with an interruption/retry test;
   retain this frozen evidence. A tooling-only repair does not itself require
   repeating this completed production comparison.
5. **Remaining gates:** advisor disposition of coverage/boundaries, G4 pinned
   B/B versus B/C DB builds, independent verify/goldens and logical parity,
   then G5 final audit packet. Keep isolated provenance tightening D-001 visible
   as the deliberate behavior change; it is not a parser-output change.

Main's pre-existing local work and Claude's worktree were preserved. This review
adds only this report and its evidence on main; no code/corpus changes, commits,
pushes, merges or deployment. Exactly one next action: Claude prepares G4 DB
parity evidence against tested C3, with the above limitations carried forward.
