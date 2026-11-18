# Fintech structural pilot — Codex G2 code review

Date: 2026-11-16. **Verdict: revise; do not freeze `b68cf68e5` for G3.**
One reproduced classifier defect permits false absence claims outside the
hand-derived fixture cases. The existing fixture and workspace gates pass.

## Reviewed identity and scope

Freeze candidate: `b68cf68e5`. Checkout: `pilot/fintech-structural`, HEAD
`a6a790fd4c5648feaffdeffecf1e51bbec4c3cdb`, clean at intake. The four pilot
source/example/test files are byte-identical at HEAD and the candidate; the
later commit only corrects checkpoint wording. Review covers implementation
`1075a6ce7` and the test expansion at `b68cf68e5`, not only the latter's diff.

Sources: the G2 provider handoff, approved G1 contract and prior dispositions,
`structure_pilot.rs`, its fixture-driven integration tests and local example,
and targeted public synthetic inputs. This is the continuing receiving Codex
review context, not an independent fresh review of earlier Codex verdicts.
No private answers or holdout provisions were used in this code review, and
G3 was not run.

## P2 — Unsupported-marker recognition is narrower than G1.3

Location: `crates/lex-core/src/structure_pilot.rs`, `is_unsupported`, specifically
the ASCII-uppercase test and the lowercase-only multi-letter run.

G1.3's unsupported forms include “a capital letter, a period and a space”
and “two or more letters followed by `)`”. The implementation recognises
only ASCII capitals for the former and only the lowercase inciso alphabet
for the latter. It therefore classifies both of these public synthetic
articles as ordinary content:

```text
Texto:

AA) Apartado.
```

```text
Texto:

Á. Apartado.
```

Reproduction: load these as eligible synthetic articles through
`structure_pilot_lookup`, request `{fraction: "I"}` in label mode. Both actual
responses are `not_found` / `absent_in_complete_scan`, with `scan_status`
`complete` and no reasons. Under G1.3/G1.6, these starts must be unsupported,
making the root fraction scan partial. Neither carries a requested Roman
label, so the expected result is L7: `not_structured` / `partial_scan`.

This is material because the pilot promises not to claim absence after a
scan that may have missed a structural form. It is not a legal classification.
The measurement probe shares the lowercase/ASCII restrictions, so matching
its counters does not independently exclude this common semantic blind spot.

**Required repair:** recognise the contracted letter forms, including uppercase
and mixed-case multi-letter starts and accented capitals, without silently
changing G1 or canonical text. Add public synthetic regression fixtures/tests
to the implementation test suite; keep frozen G1 artifacts unchanged. Exercise
the three modes and scan status/reasons for these controls. Re-run the declared
gates and inspect any non-holdout counter changes rather than automatically
re-baselining the probe to match the implementation.

## Dispositions of the six disclosed implementation choices

1. **Owner N for unsupported markers — accepted.** In note interruption, use
   m, the first candidate, so m and its ancestors retain partial scans and
   conservative cuts. In post-child attachment, use F, the certain owner;
   the inciso's attachment uncertainty is already represented separately.
   This is an engineering completion of the ownership rules, not a legal
   attachment decision. Pin these choices with public synthetic tests.
2. **Non-root intro cuts — accepted by name.** Use the G1.5 cutoff table:
   a candidate cuts a non-root intro only when it cuts that node and lies
   inside the intro. An inciso candidate in its holder fraction's marker
   paragraph therefore leaves the fraction's intro resolved. D8 defines
   intro as the rest of that paragraph, not the pre-child chapeau. This
   interpretation is specific to pilot-v1; it does not assert certain
   child attachment. The inciso lookup remains a candidate ambiguity and
   the scan partial. Pin this in a synthetic case so it cannot drift.
3. **Fraction label fields — accepted.** Separator-inclusive `printed_label`,
   separator-free normalised label, 1-based sibling ordinal and null ordinal
   for ambiguous-parent tail incisos are consistent with this pilot's shape.
   Canonical qualifier names still govern paths; source spelling is retained
   in the printed label.
4. **Optional marker fields — accepted.** Omission preserves the exact
   approved projection/response fixtures; no schema change is required.
5. **Noncanonical same-line numeral and unsupported span — accepted.** An
   unmatchable noncanonical numeral still makes the fraction scan partial;
   it must never enable `not_found`. A Roman unsupported candidate's label
   span may be its leading numeral. Pin the unmatchable-candidate behaviour
   with a public synthetic case.
6. **No loaded projection — accepted only for this local loaded-set scope.**
   `unknown_provision` means unavailable in the lookup context, not absent
   from the complete repository or from Mexican law. The example builds
   current text and projections together, so that meaning is adequate here.
   `set_current_text` is documented as replacing an existing entry; callers
   must not use it to add known canonical text without a projection and then
   interpret `unknown_provision` as an authoritative absence. Document this
   precondition/loaded-universe limitation and pin it; no broader serving or
   missing-projection API is accepted by this review.

These rulings supply review authority for the disclosed choices without
editing the frozen contract. They do not waive the classifier repair above.

## Holdout and evaluation boundaries

The builder disclosed a pre-freeze smoke run that built in-memory projections
for all LRITF articles, including holdout members, while issuing/outputting
only a development query. That departs from the blanket pre-freeze exclusion
procedure and must remain recorded. It is not, on the disclosed facts, evidence
that the builder saw private answers or tuned on holdout results; do not claim
the smoke input excluded the holdout or silently erase the deviation. Further
builder runs must filter before projection construction. No G3 score or claim
of complete evaluator blindness is accepted here.

Original gold remains immutable; its known compatibility issue requires the
independent evaluator's versioned private disposition after code/contract
freeze and before G3 scoring. The verified local custody receipt remains
separate evidence. This code review neither changes gold nor runs the example
against any holdout provision.

## Validation actually run

A mechanical delegate checked code identity and ran at HEAD with the candidate
pilot code unchanged:

- `cargo fmt --check`: pass.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: exit 0;
  Cargo still reports the existing unused `walkdir` workspace-dependency warning.
- `cargo test --workspace --locked`: all reported suites pass, including the
  nine structure-pilot integration tests. See the mechanical run for the
  suite-level counts; this review does not infer readiness from green tests.
- `cargo run --locked -p lex-cli -- validate lritf`: 0 issues.
- `cargo run --locked -p lex-cli -- validate ifpe-dcg-2021`: 0 issues.
- G0 `verify.py` without a holdout flag and G1 `verify.py`: pass.

Reviewer independently ran the existing local example on the two synthetic
articles above, reproducing the false `not_found` responses. Inputs reside
only in temporary reviewer scratch; no corpus file, frozen fixture, parser
rule, schema or private evaluator artifact was changed. The corpus sweeps
check projected-node self-consistency and schema validity; they do not supply
independent structural truth for unknown forms. No new all-instrument validation
or fidelity audit is claimed; those remain attributed builder evidence.

**Next action:** builder fixes the unsupported-marker defect and pins the
reviewed open-choice interpretations, then submits the corrected code commit
for freeze acceptance. No G3 run yet. This review writes its verdict and plan
checkpoint only; no staging, commit, push, merge or deployment was performed
by this session. At closeout a concurrent writer advanced HEAD to
`cab26ba78f12fda0e68402246b4ec6840ff081e5` through two unrelated legal-decision
documentation commits. A read-only comparison confirmed all four candidate
pilot files remain identical; those unrelated commits are not reviewed here.
The external provider handoff remains pending; no external closeout is claimed.
