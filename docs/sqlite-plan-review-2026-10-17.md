# SQLite plan and prerequisite review — 2026-10-17

## Outcome

The strengthened database-only plan is suitable to enter M1 after the operator's
execution instruction. M1 must settle the three contract details below before
M2; this review does not authorize or start the build. No new code defect was
identified in the bounded `resolution_rule` review.

Reviewed main HEAD: `cd49b9db4dcc4967bc60afc793484ad1f129ed87`.
Plan: `docs/plans/sqlite-corpus-database.md`, SHA-256
`836a7272dc4fbaaaf39c8620eb4105f8a6eed60033e516fba9c0b51bc8c39806`.
The repository uses `PLANS.md` as its index; there is no new root `PLAN.md`.

## Contract details to settle in M1

1. **Storage representation is contradictory.** Input contract item 4 specifies
   raw file bytes plus columns and extras-only JSON, but the relational table
   descriptions still require full per-row payloads. Pick one physical layout
   after the required sample measurement. Define “complete payload” as a
   reconstructible logical record if extras-only wins. Preserve absent versus
   explicit null, empty arrays, array order and lossless numbers; parity must
   compare with raw JSON independently of insertion code. Do not implement both
   layouts accidentally or use storage savings to drop metadata.
2. **Structural-debt seeds need record-level evidence.** `docs/decisions.md`
   records aggregate counts for roughly 77 merged headings, not a complete
   mapping of every missing label to its containing provision. M1 must create
   a reviewed, commit-pinned seed with source path/hash and record locator for
   each asserted mapping. Do not fabricate rows from the aggregate. The golden
   example is supported: LGTOC `Artículo 228 b.` occurs inside canonical
   `urn:lex-mx:federal:statute:lgtoc:article:228`. Reproducibility rows likewise
   require recorded measurements, not inference from absent local files.
3. **Unknown is distinct from known missing.** The plain summary overstates
   absence as known-missing. Return that qualification only where an accepted
   gap or sourced debt entry supports it; otherwise return “not found in this
   release; coverage not established.” The reproducibility table also needs a
   single contract: explicit `not_assessed` rows for unmeasured instruments, or
   a view that supplies that state over absent measurement rows. The current
   text describes both. Accepted-gap records support source instrument, target
   and multiplicity, but do not identify an exact citing provision/span.

Golden-query precision: LRSIC has four unavailable historical edges in Transitory
Quinto, while article 2 fraction V targets the current LRAF. Pin the historical
fixtures to those four edge IDs rather than treating every LRSIC citation as
historical. The corrected LIC article 194 → LFCE 31 Bis fixture matches the
stored note naming article 3, fracción IX; it must remain unretargeted.

## Prerequisite and incoming handoff

Reviewed `2026-10-17-lex-mex-resolution-rule-and-db-plan.md` in the incoming
Codex handoff queue. Its HEAD and three-local-commit statement match Git.
Independent comparison against `f0380e0fa` confirms 455 changed reference files,
31,829 rule additions and zero other edge-field, count or ordering changes.
Rule counts: named 4,091; own default 4,369; own marker 13,900; relative 2,519;
same article 1,499; parent shorthand 5,451; absent 4,058.

The protected-edge comparison removes only the proposed rule when the old rule
is absent, then compares the complete edge. A recorded rule cannot change or
clear, and other fields remain protected. Existing tests cover initial addition
and changed-rule rejection. `de la misma` and `del presente Capítulo` may be
labelled `own_instrument_default`: the field records the algorithmic branch,
not a legal conclusion that the text lacks any contextual cue. Consumers must
not interpret it as a calibrated confidence score. Parent consistency is
checked in the CLI with adapter context; the DB validator must retain that
check rather than relying only on `lex-parse` validation.

## Continuity and validation

Capsule 112 matches HEAD/status but retained the old plan digest
`828a7696…` and no current SQLite plan source binding. Git-drift agreement did
not establish semantic freshness. Refresh the plan digest and next action with
this review; historical capsule decisions remain historical evidence.

Fresh checks passed: `cargo fmt --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, and
`cargo test --workspace` (309 tests). Mechanical relative-link checks for
`PLANS.md` and the task plan passed, as did `git diff --check`. The plan digest
and unchanged suggestion-document hashes were verified.
No full closure/re-export or full corpus validator rerun is claimed. Code,
canonical corpus and operator suggestion documents were preserved. No database,
worktree, commit or push was created.

Next action: on the operator's go, begin M1 in the dedicated database worktree,
resolving the three contract details before proceeding to M2.
