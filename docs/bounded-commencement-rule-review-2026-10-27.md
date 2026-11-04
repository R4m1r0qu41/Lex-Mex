# Bounded commencement rule — independent review, 2026-10-27

**Review complete. Do not merge this optional rule as written. Defer its branch
and proceed with the four-instrument runtime proof on accepted main.** This is
not a reason for another national database campaign or for delaying the reader.
The branch remains preserved; no code, corpus or operator decision was edited.

Reviewed gate tip `bc7e2bad5d141485486149fe336d129d419c15fa`, rule `0a6fa9e97`,
corpus application `2cc5c7a5d`, refactor `ed2abf54f`; later gate record
`f0ea20b04` is documentation. Baseline `dbc88a314`.
Incoming request: `2026-10-27-lex-mex-bounded-commencement-rule-review.md`.

## B-001 — P1: a heuristic buffer is represented as a proven latest date

The new business-day rule computes publication +2N+30 calendar days, calls it
`latest possible`, and automatically emits `Effective`, deterministic basis,
confidence0.9 and `review_required=false` once that date passes.

There is no identified applicable calendar or assumption bounding non-working
periods. “At most five of seven days are business days” gives no minimum supply
of working days and therefore cannot establish this upper bound. Adding30days
is an allowance, not proof of a maximum suspension or holiday interval.

A synthetic calendar makes the issue concrete. Publication2020-03-20,N30 gives
claimed latest2020-06-18. If weekdays2020-03-23 through2020-06-30 are non-working,
the30th subsequent weekday is2020-08-11, even before adding ordinary holidays.
The code declares effectiveness on2020-06-19 in that hypothetical case.
This is a mathematical counterexample to the unsupported calendar assumptions,
**not a claim that this calendar legally governs any of the five instruments**.

Official calendar notices distinguish holidays, vacations and suspension periods:
see [the SEMARNAT notice](https://sidof.segob.gob.mx/notas/docFuente/5608192).
That notice concerns administrative acts/procedures; it does not establish the
vacatio-legis calendar for these statutes. The rule must identify its applicable
legal/calendar basis rather than silently borrowing a generic calendar.

I am not concluding that the five old instruments have not commenced, or that
all241 results are substantively wrong. The generic inference is unsupported
as a guaranteed deterministic bound. If revisited, use an evidenced calendar or
other sufficient, correctly scoped evidence; alternatively retain an explicitly
qualified proposal under an approved inference policy. Never present a convenient
elapsed-time estimate as a mathematical/legal certainty. Do not weaken provenance
or relabel the existing proposal as an audited human determination.

## B-002 — P1: a matching substring can override a condition or negation

Both new regexes recognize a fragment of the clause. The fixed exception-word
list and single-trigger count do not establish that the whole clause is unconditional.
The following synthetic cases were run through the compiled public derivation
function at the reviewed tip:

| Clause suffix/prefix | Observed result |
| --- | --- |
| Business-day form followed by “siempre que el Consejo haya emitido la autorización correspondiente” | Effective, deterministic, no review required |
| One-month form followed by “siempre y cuando se haya expedido el reglamento correspondiente” | Effective, deterministic, no review required |
| “La presente Ley **no** entrará en vigor un mes después de su publicación…” | Effective, deterministic, no review required |

All have one matched commencement phrase. The known LIGIE/SAT wording stays
excluded by its shape, but that is not a complete conditional-language guard.
The month forms previously stayed unsupported and the business-day forms skipped;
this widens automatic acceptance into cases whose premises were not established.

If revisited, recognize the complete approved clause structure and treat unknown
qualifiers/negation as pending interpretation. Avoid an endless outlier-specific
blacklist. Retain positive plain clauses and add these nearest negative cases.
Model-assisted interpretation can propose the event/formula; successful arithmetic
must not erase uncertainty about the legal premise.

## What is acceptable and what was verified

**Effective with no exact start date is a legitimate representation** when there
is sufficient evidence of the status. An exact date is not a prerequisite for
useful reading. The objections above concern how status is inferred, not a demand
to calculate every holiday or resolve every temporal outlier before launch.

The one-month +2calendar-month bound accommodates the two stated ordinary month
readings as an arithmetic bound; no counterexample to that limited arithmetic
was found here. The conditional/negation defect still prevents accepting the
combined new rule as written. No general legal adjudication of month counting
is supplied by this review.

The corpus delta was independently compared:

| Instrument | Changed provisions |
| --- | ---: |
| lbogm |124|
| lfea |31|
| reg-lgeepa-maaa |48|
| lfdc |24|
| reg-lfpc-art122 |14|
| Total |241|

Only temporal_status, temporal_basis, temporal_confidence and review_status
change. No ID, text, effective_from or other non-Markdown corpus file changes.
All changed rows previously had not_analyzed review status and no temporal basis.
No existing audited/imported row is overwritten in this delta. Review/approval
input files have no changes. This verifies scope; it does not validate the premise
behind the241 new deterministic classifications.

The `ed2abf54f` diff extracts the same match into `commencement_verdict`; inspection
found no changed conditions/results from that extraction. It was not independently
mutation-tested against a separately compiled pre-refactor version. When a new
rule eventually lands, its rule-set provenance should clearly identify the new
semantics rather than rely on an unchanged version label alone.

## Checks actually run and deliberately avoided

Disposable snapshot `/private/tmp/lex-bounded-commencement-review-20261027/candidate`:
format check passes; `cargo test --locked --offline -p lex-analysis temporal_derive`
passes25tests; library build passes. Independent compiled-function probes reproduce
both findings. Corpus/commit diffs and the synthetic calendar calculation were
checked. Evidence is [recorded separately](bounded-commencement-rule-review-2026-10-27-evidence.json).

The author's450-test/full-DB/71Node campaign was not independently repeated.
No fresh DB build or full DB verification was run: those checks cannot prove the
unsupported rule premise, and the semantic gate already fails. This proportional
stop is intentional. Existing passing tests show the implementation matches its
fixtures; the negative probes expose assumptions those fixtures do not cover.

## Close this review and move to runtime

Recommended disposition: **reviewed/deferred, not accepted and not merged**.
Keep the experimental branch, source evidence and proposed241-row change intact
for later rule-family work. Do not rebuild or alter main to close this optional
milestone. No rollback is needed because the branch is unmerged.

None of the five affected instruments belongs to the selected LRITF/LIC/ITF/IFPE
pilot. The existing accepted SQLite artifact and870 verified publication texts
remain available. Follow [the agent-first plan](plans/agent-first-delivery-and-change-tolerant-releases.md):
freeze that baseline and prove the actual runtime/reader path. The residual
commencement work is a separate queue, not its delivery gate.

No commit, merge, push, deployment, new legal approval, source mutation or active
branch change. Unrelated main plan/report/DuckDB drafts remain preserved.
