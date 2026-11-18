# Bounded commencement repair — independent review, 2026-10-28

**B-001 and B-002 are closed for the bounded candidate. The review milestone can close and runtime proof can proceed.** This accepts the supplied candidate behavior, not a complete historical calendar service. No merge, push, deployment or database rebuild was performed or authorized by this review.

Reviewed content: `8fbfd9137b27137b249b6b84801013ef900de33e`, branch `rule/bounded-commencement-2026-10-27`. Head `87df2e98325f06547b38828cac34a97249954e11` adds documentation only. Main baseline `f0f8a868fd78ce1cf0ed939ab82511daf5659725`. Active worktrees were inspected read-only; tests ran in a disposable clone.

## Findings disposition

- **B-001 closed:** the unsupported `2N + 30` business-day bound is removed. Counting excludes publication day and uses the supplied statutory version, weekends, holidays and suspensions. A complete notice-coverage window is required for a result. The shipped calendar has empty notice coverage, so it makes no business-day determination. The prior synthetic suspension counterexample is covered by a passing regression test returning 2020-08-11 with supplied evidence; absent coverage remains insufficient.
- **B-002 closed for the new business-day/month forms:** full subject-to-publication clause anchoring rejects conditions, negation, extra sentences and second events. Independently compiled public-API probes of the original conditional-business, conditional-month and negated-month examples emit zero determinations. The plain business-day probe without calendar evidence also emits zero.
- **One-month result accepted within scope:** the actual `reg-lfpc-art122` clause is “El presente Reglamento entrará en vigor al mes siguiente de su publicación en el Diario Oficial de la Federación.” Publication is 1997-08-27; the conservative two-calendar-month bound is 1997-10-27, long before the evaluation date. No counterexample was found for this complete, unconditional form. The 14 changed articles have no exact commencement date asserted. This review does not establish that the whole current regulation is unaffected by later legal events; normal amendment/repeal evidence rules still apply.

## Deferred limits — not runtime prerequisites

1. **Legacy fragment matching remains unsafe for some inputs.** Independent probes still produce results for a day-after clause ending in `siempre que ...` and for `no  entrará` with doubled whitespace. These use old day-after/calendar-day paths, not the new complete-clause paths. The same fragment matcher/exception-list structure exists in baseline. The receipt must not claim all commencement recognition is now whole-clause. Before broader temporal auto-acceptance, apply whole-clause/structured recognition to those paths and keep these probes as regressions. No new corpus outcome from either synthetic probe is being accepted.
2. **The article-28 calendar is provisional historical metadata, not independently verified formal history.** The committed consolidated provision and its absence of amendment marks do not independently prove the full historical applicability window. No DOF historical editions were checked here. Empty notice coverage correctly prevents this metadata from establishing any date. Before enabling business-day outputs, verify the relevant version and notice interval; do not treat this review as approving arbitrary future coverage entries.
3. **Do not extend the sexennial December holiday mechanically into the present.** The shipped entry is open-ended, while Executive transmission moved to October from 2024. This is disclosed and currently produces no result because coverage is empty. Cap/version the supported calendar before enabling affected periods; this review does not decide that October 1 automatically substitutes under article 28. [Formal constitutional reform](https://sidof.segob.gob.mx/notas/docFuente/5332025).
4. Calendar input loading is permissive: provenance fields are descriptive strings, with no schema/hash validation or overlap rejection. Keep these operator-reviewed inputs; harden validation before treating arbitrary calendar imports as independently verified evidence.

These limits do not require resolving every holiday, rewriting the temporal subsystem or rerunning the national campaign before the frozen runtime proof.

## Canonical diff and legal direction

Only `corpus/mx/reg-lfpc-art122` differs from main: 14 provision records and their 14 Markdown projections. JSON changes are exclusively `temporal_status`, `temporal_basis`, `temporal_confidence`, `review_status`; IDs, source text and exact dates are unchanged. The other 227 candidate changes were reverted. JRH directed article-28 counting; no blanket JRH approval of 241 statuses was inferred.

`lfdc` remains skipped because the full clause includes another sentence; its legacy diagnostic names the unsupported `mes` unit rather than this actual structural reason. This is accurately disclosed and is not evidence of legal ambiguity.

## Validation and integration boundary

Independently run at exact content commit in `/private/tmp/lex-commencement-repair-review-20261028`:

- `cargo fmt --check`: exit 0.
- `cargo test --locked --offline -p lex-analysis`: exit 0; 44 passed, 0 failed, 8 filtered; zero doc-tests.
- `cargo build --locked --offline -p lex-analysis`: exit 0.
- Compiled public-API probes: original three failure cases and missing-calendar case emit zero determinations; two legacy limitations reproduced as above.
- Compared canonical JSON and branch commit ranges directly.

Producer reports 461 workspace tests, two Clippy toolchains and nine instrument validators passing. Those full checks were not independently repeated. No national replay, A/B campaign or DB projection ran in this review.

There is no need to rebuild the accepted four-instrument SQLite to prove runtime: these changed provisions are outside that pilot. A future projection of this branch will also see changed `docs/decisions.md` input bytes; distinguish that release from the frozen accepted database. Respect existing integration gates if choosing to publish a new projection, but do not make that unrelated projection a runtime prerequisite.

Next: accepted-main M0/M1 in [the delivery plan](plans/agent-first-delivery-and-change-tolerant-releases.md), using the frozen 870-provision database for task and local Workers/D1 retrieval-parity proofs. Preserve the calendar and legacy-parser limits as deferred work, not another immediate repair campaign.
