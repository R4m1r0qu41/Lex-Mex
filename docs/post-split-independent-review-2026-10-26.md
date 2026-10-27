# Independent review: post-split corpus repair and public query

**Verdict: hold integration.** The source recovery, structural retirements and
operator-approved temporal execution have substantial independent support, but
one parser-boundary regression and two query-adapter defects need correction.
A fourth, smaller client-error inconsistency is also reproduced. No merge,
push, deployment or branch edits were performed by this review.

Reviewed handoff: `2026-10-25-lex-mex-itf-dcg-separator-candidate-review.md`,
including its consolidated update. Final parser audit tip `6a5121964`, corpus
`bcd17a4e4`; query tip `134e4e381`. Earlier independent gates ran at `e71a2f025`
and query `145bcfca1`. Parser production/manifests/schemas/adapters/fixtures are
identical between e71a2f025 and 6a5121964; the latter adds temporal data and docs.
The query changes after145bcfca1 affect only static term-page budgeting/tests.
The earlier findings therefore remain present at the requested final tips.

## Findings by introducing commit

### R-001 — P1: a genuine transitory section header can become body text

Commit `d6a28e6a7`, `crates/lex-parse/src/diputados.rs:1693–1699`.
**Reproduced against frozen baseline and the compiled candidate.**

`interrupts_sentence` is applied to every transitory section-header spelling,
not just the LRASCAP page-break fragment. With a preceding transitory body that
lacks final punctuation, it suppresses an explicit standalone `TRANSITORIOS`.
The next section's ordinal is then admitted as another ordinary transitory.

Synthetic input, identical for both binaries:

```text
LEY DE PRUEBA

Artículo 1.- Norma principal.

TRANSITORIOS

PRIMERO.- Texto sin punto

TRANSITORIOS

SEGUNDO.- Texto de otra sección.
```

Both parse commands exit0. Baseline emits article1 and transitoryprimero only.
Candidate emits those plus transitorysegundo, and appends `TRANSITORIOS` to
primero's canonical body. Adding a final full stop restores the old boundary.
This changes law/decree ownership on punctuation alone and occurs outside an
omnibus enactment. No claim is made that this synthetic shape is present in a
specific committed instrument; it demonstrates the widened production predicate.

Required repair: constrain continuation handling to the demonstrated singular
fragment/context and preserve explicit section boundaries. Add this negative
case alongside the LRASCAP positive fixture, with punctuated/unpunctuated bodies
and ordinary/omnibus contexts. Keep the approved layer-qualified IDs. Recheck
LRASCAP and the affected parser gates; no new legal ruling is needed to narrow
this accidental scope expansion.

### R-002 — P2: valid reference pages fail when a resolved target has no public route

Commit `145bcfca1`, unchanged at `134e4e381`, `reader/query-sqlite.mjs:88–93`.
**Reproduced on the final caead5ed database.**

`references()` throws `Unresolvable edge target` when a resolved canonical target
has a non-URL-safe ordinal. The adapter deliberately excludes68 such provision
IDs from public lookup, but still needs to present references to them. The
candidate database has five resolved edges to these target shapes.

Concrete public route: `mx/lft/transitory/10/references` returns503/unavailable
because its target includes `lft:transitory:9º`. This is valid stored reference
evidence, not a service failure. It prevents the whole page from being read.

Required repair: retain exact target_id, source span, resolution status and
qualifiers, and represent an unavailable public route explicitly (the existing
nullable target may suffice). Do not normalize canonical IDs, drop the edge,
change its resolution status or invent a route. Add this real database case and
a synthetic mixed page of routable/non-routable references.

### R-003 — P2: the1000-result depth cap is exceeded by normal pagination

Commit `145bcfca1`, `reader/query-sqlite.mjs:56–63`, unchanged at final tip.
**Reproduced without constructing or tampering with a cursor.**

Following every issued cursor for `obligación` with limit7 returns143pages and
1001items, despite MAX_DEPTH1000. `page()` rejects offsets>=1000 but does not
limit the final page to the remaining capacity. A diagnostic cursor at offset999
also returns10items for limit10. This is a small cap overrun, not an unbounded
search or an established security exploit.

Required repair: constrain both fetch/returned page size to the remaining depth,
and end pagination exactly at the declared maximum. Test non-divisor limits
(3,7,9), changing limits across pages, and the last permitted offset. Preserve
query/release binding and stable ordering.

### R-004 — P3: unknown-instrument searches skip cursor validation

Commit `145bcfca1`, `reader/query-sqlite.mjs:68–71`.
**Reproduced on the final database.**

A base64url cursor encoding JSONnull returns400 for a normal search, but200 for
`search?q=ley&instrument=no-such-instrument&cursor=bnVsbA`. The early unknown-slug
return precedes page validation. Validate request/cursor shape consistently
before an empty-result shortcut. This is a client-error contract inconsistency;
R-001–R-003 are the integration blockers.

## Claim-by-claim disposition

“Reproduced” means a new command/probe was run; “inspected” means source/data
comparison without repeating every historical producer action.

| Commit or claim | Disposition |
| --- | --- |
| dabeb0029 separator/plural-date repair | Current unit suite and affected source reparses reproduced. Old predicate and negative fixtures inspected; old test suite was not mutation-compiled. Separator widening is source-backed; no separate defect found. |
| 47eabbb21 four structural reparses | Reproduced parse+validate for all4 from hash-matching retained inputs. Exactly14 added IDs, none removed; shared temporal state in the committed diff preserved. Fresh parse structural fields match; a parse alone resets machine derivations, so raw parse output is not falsely called full temporal byte parity. |
| 050660b5a CUCB/OAAC | Reproduced source parse+validate and added IDs:1+3. Retained source/text hashes match; structural fields match. Inspected35/22 amended-article start-date clearances. |
| 730a8b16c retirements | Reproduced seed/input gates. Compared full accepted-row objects: old14active+3retired equals new0active+17retired. Resolving ancestry/pins are enforced by the independent verifier. |
| 600ea9a60 LSCS | Reproduced parse+validate; manually inspected six source appendix sections:3+1+5+2+2+3=16. Both old2012 evidence items remain exactly equal; accepted DB has no incoming reference to those IDs. Article207 change inspected; this is extraction/initial classification, not an applicability ruling. |
| c0b938181/b9f49d9d6/ef28b9619 editions | Acquisition and committed source/text hashes rechecked; all3 parses/validators reproduced. REG-LFDA8 added IDs and LIGIE's lagging publisher-header date inspected. Formal legal effects of every amendment were not independently adjudicated. |
| 7df6203fe duplicate legend validator | Accept the narrow rule: description whitespace is normalized; identical repetition warns, conflicting repetition and missing entries still error. Current regression suite reproduced. Raw source/canonical duplicate evidence remains retained; “read once” describes lookup/validation, not destructive source deduplication. |
| 5399d1762 CUB | Reproduced source parse+validate and7 added IDs; acquisition hashes match. All structural fields match. The producer's454/19 text-change classification was not independently reclassified line by line. Also observed281 effective_from clearances and1 effective→repealed; report these date changes as well as the headline status change. |
| 18f8e50ca goldens | Exactly2 of21 definitions changed; other19 unchanged. G6 checks standalone LFD and zero active LFD debt, G5 expects2/4. All21 goldens reproduced on fresh earlier-tip DBs and final accepted candidates as recorded in machine evidence. Existing operator approval is preserved. |
| d6a28e6a7 omnibus layers | Source-backed15+6LRASCAP layers and approved IDs inspected/reparsed; generic header regression R-001 reproduced. Commit is not accepted as-is. |
| e71a2f025 LRASCAP | Reproduced source parse+validate;21 added transitories, no removed IDs; article132 has423characters. Counted15law-layer and6decree-layer headings in retained source. |
| Four temporal commits |166instrument grouping143/17/3/3 confirmed. Exactly12,211 unknown/not_analyzed→effective/machine_accepted; no ID/text changes or other non-Markdown corpus files. Reproduced all166 via the same repair/derive/apply functions and compared serialized JSON values to committed provisions. Samples from each clause category checked; this is deterministic execution acceptance, not fresh legal review. |
| Final audited state | Accepted versus final DB:16legal operator reviews,17outdated-reference approvals and3protected provision payloads exactly equal. |
|134e4e381 static budget | Code and63-test suite reproduced/inspected. Default5000 is a reasonable configurable output ceiling, not proof of a complete topical index. Frequent-word selection omits rare legal terms; disclose that and retain dynamic search. It bounds output pages, not all indexing RAM/CPU. The45,739/89.1% production measurements were not independently rerun. |

## Independent validation and coverage

At e71a2f025: formatting, strict workspace Clippy,439Rust tests, both locked
validators, seed/input checks, two fresh database projections, independent verify
and21goldens per artifact all passed. Fresh databases and manifests are byte-
identical. `link --all` reports0relinked,466unchanged,2held (LFTR/SOCAP) and an
empty disposable Git diff. These tests do not cover R-001's new negative case.

The102recovered-input comparison was repeated in separate roots with the frozen
B/C3 binaries and the independently built candidate. B=C3 across102; candidate
changes onlyLSCS/LRASCAP. The supplied context harness initially leaves2cases
without sufficient cross-instrument context (LPDUSF approval and an imported
glossary); replaying those2 with complete corpus context succeeds and gives exact
B/C3/candidate file parity. Do not interpret missing-context exits as parser passes.
All12touched source parses and validators independently pass.107acquisition file
pairs were rehashed against producer records:102exact editions,5changed editions;
no mismatches. SOCAP's legend omits91;6parsed provisions carry that mark and the
validation errors correctly keep it held.

Final temporal data was checked separately, since it was added after the first
independent native gate. No production source changed. Final artifact checks and
strict1.99Clippy results are recorded in the companion machine evidence. A broad
CLI re-derivation/export run was stopped after partial sampling to avoid repeating
whole-corpus backlink/Markdown export166times; all166 rule computations were
instead reproduced directly through the same compiled core functions/order,
using2026-10-25 as_of. Canonical serialized JSON values match exactly. Full166
Markdown regeneration is not claimed by that core check.

Query145bcfca1:61/61Node tests on the earlier candidate DB. Accepted older DB:
14/15DB-specific tests pass; only the expected absent standalone LFD heading
fails. Query134e4e381:63/63Node tests on finalcaead5ed pass;4Rust public_query
contract tests also independently pass. Initial loopback EPERM
cleared with permission; it was not a product failure. Adversarial probes still
reproduce R-002–R-004. Browser DOM/mutation campaigns, common-token performance
measurements and federal static-page generation were not independently rerun.

## Scope, evidence and next action

See [machine evidence](post-split-independent-review-2026-10-26-evidence.json) for
full identities, hashes, command receipts and probe results. No active branch,
canonical record, provider config or accepted artifact was edited. Existing main
plan/report/DuckDB drafts are preserved. This report and evidence are new review
artifacts; no commit/push/deployment.

Next: Claude fixes R-001 separately and R-002/R-003 (plus the small R-004) on the
query branch, with failing-before/passing-after regressions. Repeat affected
native/source/DB/Node gates and request bounded re-review. Keep the166temporal
commits and provenance ancestry intact; do not rebase/squash. No full15-hour
parser-split campaign is requested merely for review documentation or tool fixes.
