# Pre-database recheck — 2026-10-17

Baseline: clean `624b6306f8399efaf65c29bef73544b3e2ef939e`, including punctuation
fix `7dc6d649c` and the operator's count/order/accepted-gap changes.
The previously reported reproductions pass. Two closely related P2 code gaps
remain. Recommendation: close these narrow gaps before beginning the database
implementation. No database was built or initialized by this recheck.

## Findings

### D1 — Whitespace before a comma bypasses the historical-law check (P2)

`crates/lex-parse/src/lib.rs:833`–`:837`, `bare_word_follows`.
An attached comma and a parenthesis now correctly trigger the exclusion, but
`Se aplicará el artículo 1 de la “Ley” , vigente hasta el 31 de diciembre de 1990.`
still emits a resolved LAN article 1 edge. The function trims whitespace, sees
a non-alphabetic comma and returns true without inspecting its modifier.
LAN's recorded publication is 1992-12-01; this explicit historical reading
should remain unlinked. This is synthetic input, not a demonstrated stored edge.

Apply the same narrow continuation check to attached punctuation and punctuation
separated by whitespace. Preserve ordinary terminal punctuation and independent
later clauses. Add quoted/unquoted whitespace-comma controls alongside the
already-fixed attached-comma and parenthesis cases.

### D2 — Strict ordering forgets a preceding suffix before a plain article (P2)

`crates/lex-parse/src/lib.rs:2091`–`:2097`, `validate_provisions`.
With `allow_article_gaps: false`, article sequences `[1, 2-A, 2]` and
`[1, 3-A, 2]` both validate with no issues. The successful plain-number branch
resets `previous_article` without comparing its sort key to the preceding suffix.
The gaps-allowed path correctly warns `suffix_order` for the first sequence and
errors `article_order` for the second. Strict mode therefore misses the very
backwards ordering the new rule is meant to detect.

Compare each article, including a plain one, against the prior label before
updating state; retain the separate strict plain-number continuity requirement.
Add both regressions plus ordered `[1, 1-A, 2]` and ordinary consecutive controls.
Do not change the operator's warning-versus-error policy.

Neither finding establishes bad stored corpus content. Do not reparse or mutate
canonical text merely to repair these code paths.

## Checks and corpus input

The native gate passes at this HEAD:

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`: 307 passed, zero failed/ignored.
- `cargo run --locked -p lex-cli -- validate lritf`: 145 articles,
  11 transitories, 126 references, zero issues.
- `cargo run --locked -p lex-cli -- validate ifpe-dcg-2021`: 59 articles,
  4 transitories, 115 references, zero issues.

Programmatic inspection of the federal instrument JSON finds 467 instruments,
68,086 provisions and 35,887 reference edges: 35,866 resolved, including 9,763
cross-instrument edges; 21 retained historical/outdated edges remain qualified.
Every resolved target exists. Stored federal validation reports contain zero
errors and 42 warnings. All 248 newly frozen count baselines match committed
article/transitory counts and are labeled machine-proposed. Reviewed changes
leave provisions, reference graphs, source provenance and legal decisions intact;
corpus changes are validation summaries only.

The isolated full `link --all` passes with exit 0: exactly the eight accepted
rejections, 459 unchanged instruments, zero relinked, 35,887 → 35,887 references
and 9,763 resolved cross-instrument edges. Every copied canonical JSON file and
reference graph is byte-identical before/after. The report retains its rejection
details even though the CLI accepts the recorded gap set.

The eight accepted closure gaps remain operator decisions, not new findings.
The closure comparison checks error multiplicity as well as instrument/error
identity. Count-baseline metadata remains free-form strings in the source type;
the new records are consistent, so stricter metadata validation is a hardening
follow-up rather than a demonstrated input blocker.

There are also 32 standards in the separate `standard.json`/clauses model, with
45 stored warnings. They were not freshly validated in this federal-linker pass.
Total stored warnings across both models are 87, not the earlier narrative 95.
A database selection must explicitly include or exclude this separate model;
467 is the federal instrument count, not a claim of complete standards coverage.

## Database boundary

SQLite + `rusqlite` remains the recorded choice. Build it as a reproducible,
commit-pinned projection of canonical JSON, carrying stable IDs, exact text,
source digests, typed reference statuses/qualifiers, review bindings and coverage.
Accepted missing links and machine-proposed counts must remain distinguishable
from audited facts; the database must not resolve them. Keep the existing eight
gaps and merged-heading/reproducibility debt visible. Schema/ingestion/query
implementation and database integrity checks are subsequent work, not verified
by the current corpus tests.

Evidence, exact private commands and expected/actual cases are in the
[companion JSON](pre-database-recheck-2026-10-17-evidence.json).

Next action: repair D1/D2 and return the bounded regressions/native gate before
initiating the database. This review changes reports/plans only; no code/corpus
change, commit or push.
