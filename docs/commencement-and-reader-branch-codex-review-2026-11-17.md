# Commencement and reader branches — Codex audit, 2026-11-17

**Recommendation: port selected commencement changes onto current main; do not
merge either old branch wholesale.** The commencement acceptance remains bounded:
B-001/B-002 are not reopened for the forms actually accepted on October 28, but
legacy fragment matching now blocks broader automatic classification. The reader
findings identify additional limitations of the accepted local D3 adapter. The
claimed transport-version collision is refuted; interface incompatibility is real.

No implementation fix, real-corpus derivation, merge, staging, commit, push or
deployment was performed. This file is the sole change in the source worktrees.
All reproductions used disposable clones or synthetic roots under
`/private/tmp/lex-branch-audit-20261117/`.

## Reviewed identity and earlier acceptance

| Surface | Reviewed HEAD / reference |
|---|---|
| Commencement | `87df2e98325f06547b38828cac34a97249954e11`; content `8fbfd9137` |
| Reader | `6a239b1c57295607f801fb22e09626f6eab48b2c` |
| Main and origin/main | `47a3ffc59457eb4abbe5ec8eda6fee0008ce0128` |
| Main working checkout at intake | `pilot/fintech-structural`, `90448c34f597c369f4c63d1f06018374fb7fe72c` |

All three source worktrees were clean at intake and immediately before this
verdict. The named ranges and heads match the packet. Commencement is 12 commits
ahead / 68 behind origin/main; reader is 12 ahead / 121 behind. This audit did not
perform a merge rehearsal or independently recheck the packet's conflict list.

The October 28 commencement review was read from `main`, since it is absent from
the commencement worktree. It explicitly deferred unsafe legacy fragment matching
(limit 1) and permissive calendar loading (limit 4). The reader's `public.rs`,
`public_tests.rs` and response schema still exactly match the SHA-256 bindings of
the October 25 D3 round-2 acceptance. Thus the new reader findings concern the
accepted code, not an intervening implementation change. The new retrieval
checker is in the later, explicitly unreviewed parking commit.

## C1–C5 dispositions

### C1 — Confirmed, P1 before broader automatic classification

Location: `crates/lex-analysis/src/temporal_derive.rs:540`, fallback through
`try_relative_days` at line 750. The plural-subject bypass produces an exact
calendar-day date without calendar evidence:

```text
Publication: 2020-01-01
Las presentes disposiciones ENTRARAN EN VIGOR A LOS TREINTA DIAS HABILES
siguientes a su publicacion.

Resolved { effective_from: 2020-01-31, pattern: "relative_days", provenance: None }
One article determination: effective_from = 2020-01-31
```

`treinta` resolves to 30. The unsupported plural subject misses the anchored
recognizer, unaccented `HABILES` misses the accented guard, and the legacy regex
absorbs the business-day words as free intervening words. No business calendar
was supplied. This confirms the reported unsafe date interpretation, without
asserting the legally correct business-day date.

**The suggested singular variant is refuted for the supplied complete form.**
`La presente Ley entrara en vigor a los treinta dias habiles siguientes a su
publicacion.` is accepted by the new `h[áa]biles` recognizer and returns
`BusinessDaysCalendarInsufficient`, with zero determinations. An accented plural
control returns `BusinessDaysPeriod`, also with zero determinations.

The specific bypass is newly reproduced; its underlying legacy fragment escape
falls within October 28 deferred limit 1. It predates the branch. The bounded
acceptance never approved arbitrary legacy clauses, and does not cover this
case. Fixing only one accent guard is insufficient for the larger rollout:
unsupported/conditional clauses must not fall back into permissive date fragments.

### C2 — Confirmed, P2; blocking before business-calendar outputs are enabled

Locations: `crates/lex-cli/src/main.rs:4580–4678`, particularly `calendar_items`
and `load_business_day_calendar`. A full CLI reproduction used the same synthetic
calendar, complete coverage and ten-business-day clause, changing only the key
`suspension_periods` to `suspension_period`:

| Input | CLI exit | Exact effective_from written |
|---|---:|---|
| Correct key; suspension 2020-01-02 through 2020-01-03 | 0 | 2020-01-17 |
| Misspelled key; identical suspension entries | 0 | 2020-01-15 |

The typo is silently ignored and the earlier result is saved as a deterministic
classification. This is a concrete additional failure under the permissive-loader
problem already noted in limit 4, not a previously tested suspension-key case.
The shipped empty notice coverage still prevents ordinary business-day outputs;
this reproduction deliberately supplies synthetic complete coverage.

One wording correction to the packet: the shipped JSON **does contain**
`schema_version: "1"`; the loader does not enforce it. Unknown/missing/wrongly
shaped list keys and defaulted provenance remain the issue. Required inputs need
explicit validation, including an intentional-empty policy, before enabling this
path. This audit does not approve any historical calendar metadata.

### C3 — Mixed: already covered P1 false negatives; confirmed P3 false positive

Location: `temporal_derive.rs:476–529`. The earlier doubled-whitespace negation
finding is limit 1 and remains reproducible: `no  entrará` still resolves a
next-day commencement. Equivalent whitespace variants are the same unresolved
class, not a new closure of B-002.

The false positive is independently confirmed:

```text
El Reglamento Interno entrará en vigor al día siguiente de su publicación.
-> ExceptionOrConditionalLanguage { marker: "no entrará en vigor" }

El Reglamento General entrará en vigor al día siguiente de su publicación.
-> day_after_publication, 2020-01-02
```

The suffix of `Interno` is misread as a standalone negation. This creates
unnecessary review rather than a false exact date, so P3 is appropriate for this
half. Whole-clause recognition should distinguish tokens and whitespace; a
negation-regex patch alone would not cure the rest of the legacy fallback paths.

### C4 — Confirmed, P2, pre-existing persistence defect

Location: `crates/lex-cli/src/main.rs:4694–4752`. A synthetic, already
machine-accepted deterministic article was stored as `repealed` with an opening
partial-repeal note and no ordinary transitories. Normal derive printed:

```text
reclassified 1 partially repealed articles ...
... (no ordinary transitories); 0 repealed
exported Markdown
```

Exit was 0, but `provisions.json` still had `repealed` and an empty `repeals` list.
An identical root run with `--repair-dates-only` persisted `partially_repealed` and
the paragraph-1 repeal. Thus `split` is counted correctly in repair-only mode
but omitted from normal derive's save gate. Export success is not evidence that
the canonical repair was saved. This is not a new regression introduced by the
bounded changes.

### C5 — Confirmed stale navigation, P3; preserve historical evidence

`docs/post-split-r2-queue-2026-10-25.md:95` still says five instruments **now**
classify, 241 provisions. The final branch retains only 14 `reg-lfpc-art122`
article changes after `230791871`; the other 227 were reverted. The queue needs
an explicit superseding checkpoint.

The integration-readiness block at line 59 is explicitly a historical gate for
`bc7e2bad5`; 241 is correct for that named old candidate. Do not silently rewrite
its historical measurements to 14. Mark it superseded and point to the final
repair/review instead. These records do not prove 241 current canonical changes.

## R1–R5 dispositions

### R1 — Confirmed pagination limitation, P2 if exhaustive traversal is promised

Location: `crates/lex-db/src/public.rs:1136–1171`. A schema-valid reference row
with 32 qualifiers of 512 characters exceeds the default 16 KiB envelope limit.
With that row first and a small later row, the actual `page` function returned
the following for one-/two-row page batches. This is a page-layer probe, not a
new real-release endpoint replay:

```text
page_size=1 -> response_too_large
page_size=2 -> response_too_large
later row alone -> fits
```

The probe validates the synthetic response shape independently; the first
attempt used over-limit qualifier fields and was corrected before the final
reproducer. No committed release row of this size was demonstrated.

Failing closed is appropriate; silently skipping the row would not be a valid
repair. But an error contains no continuation cursor, and changing page size
cannot move beyond it. D3 tested oversized **provision** responses and multi-row
page shortening; the earlier review did not rule on recoverable traversal past
an oversized list item. Its acceptance does not establish such a guarantee.
Choose and test an explicit typed oversized-item/continuation policy before
promising exhaustive enumeration. This is an availability/completeness issue,
not a false exact-text claim.

### R2 — Confirmed, P2, new unreviewed checker

Locations: `reader/retrieval-check.mjs:28–32,101`. Four local fixture probes added
one legitimate extra link to the release index: site home `/`, an official DOF
URL, `#top`, or `?view=all`. Each aborted discovery after four HTTP exchanges
with `URL escapes allowed origin or pinned release`, before following any
external link. The original 15 checker tests pass and do not cover this case.

The safety condition is suitable for a link actually selected for retrieval,
but not as a throwing predicate over every unrelated index link. Classify/filter
candidate links without aborting discovery, then enforce the strict pin on the
selected path. No live DOF or production website request was made by these probes.

### R3 — Refuted as a version-name collision; incompatible interfaces confirmed

The packet conflates a schema **filename**, a DB projection version and a wire
response version:

| Surface | Identifier |
|---|---|
| Reader native DB projection | `public-query-v1` |
| Reader response schema/version | `public-query-response-v1` |
| Main schema filename | `schemas/public-query-v1.schema.json` |
| Main actual transport constant/schema version | `public-retrieval-v1` |

A main transport fixture passes main's actual `checkResponse`; a synthetic valid
D3 response is rejected by it, as expected for a different envelope. Both being
version 1 is not itself a collision and does not require renumbering one to v2.
They cannot be substituted at the same interface without an explicit translation
or contract decision. No P1/P2 naming defect is established by this finding.

### R4 — Deadline scope gap confirmed; P2 before a hard whole-request deadline claim

Locations: `public.rs:921–927,1452,1652`. SQLite's progress handler cannot interrupt
Rust excerpt tokenization, and the deadline is disabled after dispatch without a
final elapsed-time check. A synthetic 800,007-byte provision with a tiny FTS hit
set returned `outcome: ok` after the configured deadline:

| Configured deadline | Observed debug-build elapsed |
|---:|---:|
| 5 ms | 142 ms |
| 20 ms | 144 ms |
| 100 ms | 145 ms |

This confirms the mechanism. It does **not** establish a production, optimized
2-second overrun on the largest committed provisions; that benchmark was not run.
The old D3 records describe a SQLite deadline and distinguish host timeouts;
they did not independently test this excerpt-stage case. Preserve that scope
qualification rather than treating the earlier stdin-framing repair as an
end-to-end CPU bound. A future hard deadline needs bounded/cooperative non-SQL
work or host isolation, not only a SQL progress callback.

### R5 — Confirmed, P2 for arbitrary-path readers/verifiers; lower impact for builder

Locations: branch `public.rs:149–155`; main `verify.rs:530–535` and
`build.rs:331–336`. Real SQLite WAL probes, with the writer kept open, show:

```text
lex.db:     lex.db-wal exists; branch open_readonly succeeds and reads its row
lex.sqlite: lex.sqlite-wal exists; branch open_readonly rejects it
```

The builder names its artifact `corpus.sqlite`, so its current fixed-name path
avoids the naming defect. However, public reader/open and `db verify --database`
accept arbitrary `Path`/`PathBuf` values with no `.sqlite` restriction. Renamed
`.db` inputs are not excluded by those interfaces. There is no supported basis
for assuming every accepted caller path has the builder's extension.

Fix the filename-independent suffix construction on main even if D3 is retired;
reuse a tested helper in verify/build where appropriate. The builder's copied
loop merits consistency hardening despite being unreachable with its current
fixed name. This probe establishes a missed immutability guard, not that the
entire main verifier accepts an altered database: its other digest/content
checks were not bypassed or exercised here.

## Two additional material findings

**C6 — Configured-calendar derivation and date repair disagree (P2 before enabling
business-day dates).** `temporal_derive.rs:381–394` repairs dates using an empty
calendar. CLI repair-only returns before loading the configured calendar. After
the synthetic valid-calendar run set an article date to 2020-01-17, adding a
recognized reform note and invoking repair-only left that original-enactment
date intact. The existing repair contract would clear an amended article's date
when the original commencement is resolved, but its calendar input is unavailable
on this path. The normal derive caller also invokes repair without that context.
No current shipped business-day result is claimed; this becomes relevant when
calendar evidence is enabled. Pass the same evidenced calendar through both
operations and pin the repair behavior before then.

**R6 — A successful annex binding can fail to open (P2 availability).**
`public.rs:446–495` binds an annex by its unique configured URL and digest;
`load_scope` at `653–690` later selects retained manifests by digest alone. A
synthetic retained manifest list with distinct URLs sharing identical content
passes binding and writes a binding file, then opening it fails:

```text
bind=ok
open_error=annex 1 source is ambiguous
```

Shared content hashes do not imply identical source metadata. The binder/open
identity rules must agree. This does not show wrong source data being returned;
it is a fail-closed availability defect beyond the earlier URL-parsing parity
repair. Preserve a valid-bind-implies-open regression with shared-digest cases.

## Answers and recommended direction

1. **Corpus exposure for C1:** a read-only scan of ordinary committed transitories
   at both commencement `87df2e983` and current main `47a3ffc59` found **zero**
   unaccented `habil`/`habiles` clauses containing the commencement trigger. A
   broader branch scan found 13 business-word/trigger records, all accented.
   The accented guard blocks the legacy bypass even for unsupported plural
   subjects. This audit found no committed transitory reaching the erroneous
   C1 relative-days path and establishes no already-committed wrong date from
   it. This is a bounded exposure result, not a general temporal-date audit.
2. **Commencement base:** implement the corpus-wide workflow on current main.
   Port the useful bounded recognizers, evidenced-calendar counting and their
   tests selectively; do not merge the old branch's corpus/doc snapshots as the
   new truth. A rewrite of all sound arithmetic is unnecessary. Close C1 plus
   deferred limit 1 across every legacy date path, C2/limit 4 and C6 before
   calendar-enabled dates, and C4's save gate. C3's false positive and C5 are
   lower-priority cleanup. Earlier limits 2/3 on historical calendar evidence
   and applicability remain gates before enabling affected dates; fixing
   C1–C4 does not approve those inputs. Keep JRH's recorded in-force rule
   distinct from invention of exact current-wording dates and from audited
   legal decisions. No legal calendar interpretation was made here.
3. **Reader disposition:** main's Rust `public_query.rs` is a DTO validator,
   **not** a replacement implementation of the D3 database server. Main now
   has the `public-retrieval-v1` JS transport, SQLite/in-memory/projection
   adapters, and HTTP/MCP/Worker integration. These serve the current direction,
   with semantics that are not automatically D3-equivalent. Do not merge a
   second old runtime wholesale. Keep the agent-test evidence and
   discoverability plan; also salvage the retrieval checker after R2 is fixed.
   Retain D3's source and tests in history/reference, especially exact binding,
   private-field allowlisting, annex/source parity, keyset pagination and SQL
   semantics tests. Port those guarantees/tests if needed by current main,
   rather than declaring them replaced by the DTO file or deleting their
   evidence. Reuse the native adapter only for a separately chosen native
   backend, after R1/R4/R5/R6 and transport mapping are settled. Prior
   exact-artifact D3 evidence remains valid for the artifacts actually tested;
   it is not a waiver of the newly reproduced limitations.
4. **Independent main repair:** R5 belongs on main regardless of reader-branch
   retirement. No scope here authorizes that implementation.

The remaining capped observations were not promoted to confirmed findings:
the no-clobber `write_binding` check/write race was not independently raced;
the repeated number-word tables are maintenance debt; foreign-provider routing
configuration was not altered. This is a bounded audit, not an exhaustive new
security, corpus-fidelity, historical-calendar or production-performance review.

## Exact verification and artifact boundary

Mechanical delegates ran the following in detached disposable clones:

- `cargo test --offline --locked -p lex-analysis`: 44 passed.
- `cargo build --offline --locked -p lex-cli`: passed for both branches.
- `cargo test --offline --locked -p lex-db --lib public::tests`: 13 passed.
- `node --test reader/retrieval-check.test.mjs`: 15 passed after allowing its
  synthetic loopback server; the first sandboxed attempt failed with `EPERM`.
- `cargo test --offline --locked -p lex-analysis audit_20261117 -- --nocapture`:
  one public-API probe test passed, including C1/C3 controls.
- `cargo test --offline --locked -p lex-db --lib audit_20261117 -- --nocapture`:
  final consolidated five reader probes passed.
- `node --test --test-name-pattern=audit_20261117 reader/retrieval-check.test.mjs`:
  four extra-link probes passed.
- `node .../check-contracts.mjs`: main fixture accepted; D3 envelope rejected;
  distinct actual wire versions printed.
- Full CLI `derive-temporal` on synthetic good/typo calendar roots, normal/repair
  partial-repeal roots and amended-date repair root: all exited 0; emitted
  outcomes/persistence differences above were inspected.
- `python3 .../scan-unaccented.py`: zero C1 candidates at each pinned corpus.

Probe source and logs are retained under
`/private/tmp/lex-branch-audit-20261117/`, notably `commencement-probes.rs`,
`prepare-cli-fixtures.py`, `reader-probes-final.log`, `c1-c3-probe.log`,
`calendar-good.log`, `calendar-typo.log`, `partial-normal.log`,
`partial-repair-only.log`, `calendar-amended-repair.log`,
`reader-r1-probe.log`, `reader-extra.log`, `retrieval-extra-links.log`,
`contract-versions.log`, and `corpus-c1-scan.json`. Probe-only test additions
remain exclusively in the throwaway clones. No original branch test was edited.

No full workspace fmt/clippy/test rerun, real-data public-release replay, national
status pass, original holdout access/hash/copy, or pilot lookup was performed.
The supplied review scope authorizes only this verdict as a source-worktree
write. No plan, provider configuration, Vault receipt or rolling context was
modified. Active-run discovery from the main, commencement and reader worktrees returned
`found: false` for each;
this completed read-only audit leaves no unfinished work requiring a new capsule.

**Next action:** JRH selects the fix/port order using these dispositions. Claude
can then implement on a current-main branch and return bounded regression
results. Leave this verdict uncommitted until the operator agrees, as requested.

Audit scratch-manifest SHA-256: `fad4eae26bb5f63b7284d2a7de4b37f0be8f9c99ae0346ed275213c5d47ad002`.
