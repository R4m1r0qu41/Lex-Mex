# Four-goal upgrade — independent Codex review

Reviewed handoff target `fbf46774e76260702a4dc42b366cfd750f993a39` from working HEAD
`ee4f730c3af0e5315994a4086ab09ca54d5f0c32`; the intervening commit changes only the execution plan.
Initial worktree clean. This is technical review, not JRH legal approval or publication approval.

## Verdicts

1. **Accept — P2 omission runs.** `crates/lex-parse/src/diputados.rs:440` restricts removal to ordinal/article/connector words and redaction punctuation; substantive bodies and numeric articles do not match. Fixtures cover article and transitory contexts and negative cases. Acceptance concerns this rule; the full historical reparse was not repeated.
2. **Accept — monitor-sources.** `crates/lex-cli/src/source_monitor.rs:366` separates changed downloads from retained originals; `:406` uses exclusive creation and hard-link installation without replacing existing targets. `:554` distinguishes validator agreement from hash verification and bypasses HEAD shortcuts for retention. Mock integration coverage exercises unchanged, changed, unavailable and retention paths. No live network monitor was run. The lexical path check assumes the repository's ordinary directory layout; it is not a symlink security boundary.
3. **Change requested — qualifier attribution (P2).** `crates/lex-link/src/references.rs:1430` drops qualifiers after the connector chain unless they point back explicitly. The plan itself records lost valid shapes such as `de esta Ley, fracciones V y VII` and `excepto fracción I`. These lose available citation precision in the Obsidian projection. Add positive regression cases for the documented losses and preserve their qualifiers without reintroducing the later-citation leakage. Independent structural comparison of `05d15bcd0` confirms 190 reference files, exactly 1,192 removals, zero additions, and no other edge-field changes; it does not establish semantic correctness of every removal.
4. **Accept with bounded assurance — SCJN header.** `crates/lex-core/src/lib.rs:653` requires the exact invalidity prefix and a parseable decree date; `:696` additionally validates the notice, source hash and source-text quotation. Undated and differently worded notices fail. A foreign quotation absent from the retained source fails. This does not prove that any matching quotation anywhere in a consolidated source describes the current instrument: the predicate checks neither header location nor instrument identity, and accepts arbitrary trailing text after the date. Keep reviewer approval as the authority; do not describe this as automatic legal identity verification.
5. **Accept recorded LIC data; change requested for migration mechanism (P2).** `crates/lex-cli/src/main.rs:3140` checks approvals only if references.json exists, and requires exact prior approval membership. Comparing `136fb83bf^` to current data confirms article 187's approval is identical; article 194 differs only in source_text_sha256, reviewer, captured_at, decision_source and decision_source_sha256. The specific authorized adoption is supported. Removing the file to permit migration is not a reusable audited migration path. Add an explicit re-confirmation operation that retains prior evidence, checks authorized field changes and validates the candidate before replacement; test both unauthorized refusal and authorized migration. This is an existing workflow gap, not a new regression in that commit.
6. **Accept — provision_notes boundary, with coverage follow-up.** `crates/lex-core/src/lib.rs:584`, `:1649`, `crates/lex-cli/src/main.rs:2117`, `:3132`, and `crates/lex-export/src/lib.rs:192` separate annotations from source text, bind source and normalized provision hashes, check quotations during parse and refuse removal/replacement of persisted notes. Existing tests cover hash normalization, changed words/source hashes and attributed export. Add integration tests for persisted-note removal and stale/missing target rejection when extending this boundary. Validation without retained source caches checks hash binding, not the source quotation anew. No claim is made that the entire CPEUM re-ingest or 465-instrument replay was independently reproduced.

## Disposition

Request changes on items 3 and 5 before treating the upgrade mechanisms as fully accepted. The reviewed LIC data does not need rollback. The header-location limitation in item 4 should remain explicit. JRH's deferred temporal decisions and held instruments remain out of scope.

Native validation results are recorded below. No code or canonical data was edited; no commit, push, merge or deployment was performed. Next action: bounded qualifier regressions and an audited re-confirmation design, followed by re-review.

## Validation run at ee4f730c3

Mechanical delegate ran successfully (exit 0):
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace` (all suites passed)
- `cargo run --locked -p lex-cli -- validate lritf` (0 issues)
- `cargo run --locked -p lex-cli -- validate ifpe-dcg-2021` (0 issues)
- `cargo run --locked -p lex-cli -- validate cpeum` (0 issues)
