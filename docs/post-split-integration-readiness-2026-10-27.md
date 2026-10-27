# Post-split integration readiness — 2026-10-27

**Ready for an operator-authorized normal merge. Nothing here is merged, pushed or published, and Codex's acceptance is not a
merge or deployment authorization.** Codex's bounded re-review
([report](post-split-bounded-re-review-2026-10-27.md), [evidence](post-split-bounded-re-review-2026-10-27-evidence.json))
accepts the R-001 to R-004 fixes with no blocking finding; the one non-blocking note (R-005, a wrong explanation of the logical-digest
change) is corrected in a docs-only commit.

## What is accepted, and the exact identities

| Item | Branch | Tip | Tested source |
| --- | --- | --- | --- |
| Parser, corpus, retirements, goldens, temporal derivation | `repair/itf-dcg-heading-separator-2026-10-25` | `773c53f2d` | `b38d1f33d` (the tip adds only the docs-only R-005 correction) |
| Public-query contract and SQLite adapter | `repair/public-query-contract-2026-10-25` | `615578206` | `615578206` |
| Records, Codex reviews | `main` | `f8ec6788f` | docs only |

Independent reviews: `docs/post-split-independent-review-2026-10-26.md` (hold, four repairs) and
`docs/post-split-bounded-re-review-2026-10-27.md` (fixes accepted). Review tier of the fixes: Codex.

## Merge plan (for the operator to authorize)

1. Use an operator-created integration worktree from `main` (one reviewed integration session; main's own working tree has another
   writer's uncommitted plan edits that must stay untouched).
2. **Parser branch first, then the query branch,** each as a normal merge commit (`git merge --no-ff`). No rebase, no squash, no
   fast-forward: the 17 retirement records pin commit SHAs (`47eabbb21`, ancestors `bcd17a4e4`, `6505baf57` are preserved).
3. Dry runs done read-only with `git merge-tree --write-tree`: **both merges are conflict-free**, and the two branches touch **disjoint
   paths** (parser branch 13,869 files, query branch 20, no overlap), so the second merge cannot conflict with the first. Merge base for
   both is `e3ad94ee3`.

## Checks of the actual merged result, before any publication

The tested identities above are not the merge commits, so repeat on the merged result (the parser change alters pinned builder sources,
so builder and corpus identity are the merge commit for the database build):

- fmt, strict clippy on 1.95 and 1.99.0, the workspace tests (expect 442 + the query branch's Rust tests), both locked validators
  (LRITF 145/11/126, IFPE 59/4/115), all 12 touched instruments, `link --all` on a clone (expect exit 0, 0 relinked, only `lftr` and
  `socap-sofipo-dcg-2006` rejected).
- Two fresh `db project` runs from the merge commit: byte-identical, `db verify`, all 21 goldens. Census to compare against:
  68,395 provisions, 36,077 edges, 102,282 usages, 0 active and 17 retired debt rows. The logical digest will differ from `42cfc2e3…`
  only if the stored `docs/decisions.md` bytes differ (the R-005 explanation).
- `node --test reader/` with `LEX_MEX_QUERY_DB` set to that fresh database (expect 71 of 71), and without it (53 pass, 18 skip).
- Provenance: every retirement record still resolves on the merged history.

## Not authorized, not resolved by this acceptance

- **Merge, push, serving or publication.** None happens without the operator.
- `socap-sofipo-dcg-2006` stays held (its legend omits entry 91); the 74 instruments `derive-temporal` skips and the 6 with no
  ordinary transitories stay queued in the R2 document; the common-token search-cost cap remains a product decision (measured worst
  case about 0.2 s cold); the serving and deployment task is separate.
- No audited legal state changed (16 reviews, 17 approvals and the 3 protected provisions are identical across the accepted and final
  databases); the 12,211 temporal transitions and the 341 start-date clearances are rule outputs, not legal rulings.
