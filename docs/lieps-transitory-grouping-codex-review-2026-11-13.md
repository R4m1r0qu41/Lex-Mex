# Codex review — LIEPS grouping and adoption

Review target and working HEAD: `191f8299db0333bac1cd947a1beda5555c1e36b9`,
branch `fix/lieps-transitory-grouping`; baseline `181f1f62c`.
Scope: `64afb12eb` and `191f8299d` for inclusion in main. Four pre-existing
untracked prompt files are preserved. This review does not merge the branch.

## Lint commit — accept

`64afb12eb` substitutes map_or_default for map followed by unwrap_or_default.
Both Result/Option branches retain their prior values and closure behavior.

## Corpus and trust-boundary checks

Independent comparison of all changed provisions.json files finds 13 changed
provisions, all whitespace-only, with every non-text field unchanged. These
include the four authorized LIEPS reflows and nine other provisions (the tenth
other rejoin is enactment evidence). No canonical legal-status decision was
introduced by these provision changes.

The LIEPS article 3 hashes independently match the old/new reviewed profile
hashes. The 55 changed adapter start offsets align at identical non-whitespace
prefix positions; all other adapter values except the profile hash/evidence
annotation are unchanged. This supports the disclosed 47 term/marker starts
and eight exclusion starts. It is not a second legal approval.

The fixture documents retained-source hashes and bounded line ranges. New tests
cover the erratum, law-named headings, decree-wrapper sections and a title-case
heading following a finished sentence. Section-number renames are acceptable
for this unused machine evidence provided downstream reviewed determinations
remain protected by the existing reapplication guard. A full 468-instrument
replay and an independent official decree-list comparison were not repeated;
the handoff's corresponding figures remain author evidence.

## Parser commit — change requested (P2)

`crates/lex-parse/src/diputados.rs:1147` (`cites_decree_mid_sentence`) decides
that a title-case decree line is a citation solely because the preceding block
ends with a lowercase character or comma. It does not distinguish a genuine
new act carrying its own publication line. A bounded synthetic variant of the
new title-case regression, changing the first transitory's terminal period to
a comma, returns only the second act: the 2010 transitory is lost entirely.
The handoff specifically identifies this false-negative risk for review.

Reproducer input (literal line/blank-line boundaries):

```text
ARTÍCULOS TRANSITORIOS DE DECRETOS DE REFORMA

Decreto por el que se reforma la Ley de Muestra.
Publicado en el Diario Oficial de la Federación el 3 de marzo de 2010

TRANSITORIO

Único. El presente Decreto entrará en vigor al día siguiente,
Decreto por el que se adiciona la Ley de Muestra.
Publicado en el Diario Oficial de la Federación el 4 de abril de 2011

TRANSITORIO

Único. El presente Decreto entrará en vigor el 1 de mayo de 2011.
```

Calling `extract_reform_evidence` with the default empty option lists and sample
instrument identity returns only:

```text
urn:lex-mx:federal:statute:sample:amendment:2011-04-04:transitory:unico El presente Decreto entrará en vigor el 1 de mayo de 2011.
```

This is a synthetic parser-boundary case, not a claim that a committed source
currently has this punctuation. The heading and its independent publication
line identify a separate act despite the preceding punctuation. Narrow the
citation suppression using bounded surrounding context (including independent
publication evidence) and add this negative regression alongside the real LIEPS
citation case. Preserve both act groups; do not weaken the real citation fix.
The wrapper/Primero heuristic has no additional blocking finding in this review;
its dependence on recognized source structure remains a known limitation.

## Validation and disposition

Mechanical delegate ran all with exit 0:
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace` (523 passing tests)
- `cargo run --locked -p lex-cli -- validate lritf`
- `cargo run --locked -p lex-cli -- validate ifpe-dcg-2021`
- `cargo run --locked -p lex-cli -- validate lieps`

Direct-rustc probe compilation failed because existing rlibs were incompatible
with that invocation. The same parent-authored probe then ran successfully via
a temporary lex-parse Cargo example (exit 0), and the delegate removed that
example and its empty directory. No production code/corpus edits remain.
`git diff --check` passed before the final documentation update.

Verdict: accept `64afb12eb`; request the bounded parser fix before accepting
`191f8299d` for main. Review and execution-plan documentation only were added
this session; the four unrelated prompt files were preserved. No commit, push,
merge or deployment. Next action: repair the title-boundary case and re-review.
