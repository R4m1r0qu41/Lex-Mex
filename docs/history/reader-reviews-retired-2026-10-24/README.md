# Retired Reader review snapshots — 2026-10-24

**Historical evidence, not current release holds.** The operator requested retirement after a dedicated windowed-reader worktree exists. Created `/Users/jr/Dev/lex-mex-reader-windowed` on `spike/windowed-reader`, baseline main `a07a93f31`, and verified it clean BEFORE retiring either older worktree. No spike implementation was started.

| Retired checkout | Preserved branch / tip | Review disposition |
|---|---|---|
| lex-mex-reader-four | feat/four-instrument-reader-design / f11beebce | R5 cap fixed256d1b410; F1 loading gate fixedb9b8be012; later reviewed/merged R7 preserves the fix. Account-switch packet and owner handoff are historical. |
| lex-mex-reader-five | feat/reader-tweaks-r7 / 0661d5ef0 | R7 review accepted; sole Libro finding fixeda21459733, independently accepted in final0661 review and merged main. |

All10 previously untracked review/evidence files are copied **verbatim** beneath reader-four/docs and reader-five/docs. Their hash inventories and the tracked plan diff are retained here. The diff is stored as tracked-changes.patch.json with base64 bytes and SHA-256, preserving original whitespace without treating patch context as document whitespace; the off-repo archive retains the original .patch file. Do not rewrite old verdicts to fabricate what was known earlier. The R6 code-review appendix's public-query findings were transferred to `../../public-query-branch-review-2026-10-24.md`; those are still pending.

Full local preservation: `/Users/jr/Dev/.work-backups/lex-mex-reader-retired-2026-10-24/`. Each checkout has original untracked reviews, the dirty tracked plan where applicable, binary patch, HEAD/branch/status manifest, and all previously ignored roots. The ignored .work/target directories were moved intact on the same filesystem rather than discarded. This retires active checkouts; it does not claim disk space recovery. Both branch tips are ancestors of current main and branch refs remain. The older hold capsule for reader-four was copied to the archive, then closed using the specifically staged retirement receipt in Agent Vault; no Vault commit was made.

Worktree removal was performed only after verifying the spike worktree, unchanged branch tips, empty remaining untracked/ignored inventories, matching dirty patch and preserved document hashes. Git worktree registration removed for four/five. Main, permanent Reader, DB/parser and unrelated experimental worktrees were not removed.

The separate pending `2026-10-24-lex-mex-post-integration-independent-review.md` is not resolved by retiring these historical UI snapshots. It remains an administrative P/R evidence-packet review queue. No other provider handoff was silently closed.

Current workspaces: permanent `/Users/jr/Dev/lex-mex-reader` remains unchanged with its pre-existing handoff documentation edits; windowed spike has its own clean worktree; `feat/public-query` remains preserved without an added worktree and has a new review report/evidence on main. No source edits, branch deletion, push or deployment in this retirement task.


## Later disk cleanup (operator requested, same day)

The initially moved bulk ignored .work/target roots have now been compacted and deleted; they are no longer at the old archive paths. The10 review documents, dirty-plan originals/encoded patch and status manifests remain. Unique raw evidence is hash-addressed and compressed under main/.work/retained-reader-parser-evidence-2026-10-24. See ../../reader-parser-worktree-cleanup-2026-10-24.md for exact deletions, recovery mapping and the removal of the unused windowed/parser worktrees.
