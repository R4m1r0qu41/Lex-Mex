# Lex-Mex audit fixes — final independent re-review, 2026-11-18

## 1. Verdict

**Close. Residual F1 and F2 are repaired at the reviewed heads.** F3 remains closed. R4 remains accepted as deadline classification with cooperative checks, not a hard wall-clock cap. No regression in an earlier closed finding was identified in the scoped diffs and native checks.

The two nonblocking follow-ups are configuration validation for bare delimiters on the reader's base URL, and converting older navigation fixtures to pre-hash construction. Neither changes this closure decision. This is software recognition/transport acceptance, not a determination of any provision's legal status or governing date.

## 2. Identity and scope

- Port: detached `8a74811a8d51c4e0b0c1991bc20ffee7f4ae7a8f`; residual range `74a6242a9..8a74811a8`, cumulative range `117a35504..8a74811a8`. Only `crates/lex-analysis/src/temporal_derive.rs` changes in those ranges.
- Reader: detached `6ba814c9aee0c4e4bb80437c5ac02700288cbe2f`; code head `d2426a630`. F2 range `9500c52d7..d2426a630` changes only `reader/retrieval-check.mjs` and its tests. `d2426a630..6ba814c9a` changes only the closeout note. F3 code remains `aeaf5dbc9`.
- Read the binding prior verdict, original request, both handoffs, reader closeout, both clones' AGENTS/CODEX/CLAUDE instructions, PLANS, and repository next-action sections. The first verdict is absent at the requested port-relative path but was supplied at `../docs/codex-audit-fixes-codex-review-2026-11-17.md`; that copy was read.
- Read the authorized provider-instructions rollout plan. Repository registration is supported by the explicit `lex-mex` project ID. Existing AGENTS/CODEX companion layout and global `project_doc_fallback_filenames = ["CODEX.md"]` satisfy the convention; no adoption edits were necessary.
- Inspected the complete residual diffs, cumulative F1 changes, exact tables, matching/guard paths, URL selection and request/redirect enforcement, unchanged response contract/runtime wiring, and current/historical version references. Earlier findings were checked for regression, not re-audited from scratch.

Port initially had four supplied untracked documents: both handoffs, the binding prior verdict, and `docs/codex-rereview2-brief-2026-11-18.md`. Reader was clean. Those states were preserved. All builds and probes used the supplied throwaway clones or `/private/tmp/lex-rereview3-probes`; neither production worktree was used or modified.

## 3. Findings

### F1 — closed: exact measured vocabulary bounds the tail

Independent calls to `derive_article_temporal_determinations_with_calendar`, publication `2020-01-01`, today `2026-11-17`, empty calendar, reproduce the required behavior:

| Probe | Determinations | Result |
|---|---:|---|
| Prior residual 1: capitalized condition in abrogation title | 0 | `UnrecognizedCommencementPattern` |
| Prior residual 2: capitalized condition as signature | 0 | `UnrecognizedCommencementPattern` |
| Prior residual 3: capitalized condition in motto | 0 | `UnrecognizedCommencementPattern` |
| Prior residual 4: second commencement as signature | 0 | `MultipleCommencementEventsInRecord { count: 2 }` |
| Original witness 1: conditional abrogation | 0 | `UnrecognizedCommencementPattern` |
| Original witness 2: second event in abrogation | 0 | `MultipleCommencementEventsInRecord { count: 2 }` |
| Original witness 3: quoted condition | 0 | `UnrecognizedCommencementPattern` |
| Plain positive control | 1 | exact `2020-01-02`, `day_after_publication` |
| `Salvoconductos` control | 0 | `UnrecognizedCommencementPattern`, not a qualifier |

The tables contain escaped, case-sensitive literal alternations inside the surrounding case-insensitive pattern. No capitalized-word or free-text title/name/motto production remains. A new title, motto or signatory requires an explicit table addition; modifying or extending one with a condition fails the anchored match. Existing generic date, role, whitespace and punctuation shapes still admit combinations of the fixed vocabulary: this is not a whitelist of whole source records.

All **24 entries** were independently located in committed transitory text, after the parser's documented whitespace/ordinal-period normalization:

- Three abrogations: `lei:PRIMERO`, `laam:PRIMERO`, `lcacsam:ÚNICO`, including their complete measured date suffixes.
- One motto: `lrcdn:UNICO`.
- Twenty signatories, four each in `lvgc:13`, `loinah:UNICO`, `lrmnyut:UNICO`, `lounam:8°`, and `lrcdn:UNICO`.

The source spans were inspected locally, not inferred from test expectations. These entries record text, not legal conclusions or selected governing dates. `loinah` also has later executive apparatus outside the permitted tail; measuring its names does not imply the entire record is recognized. No new canonical determination is installed by these tables.

A separate **96-assertion public-API sweep** exercises each entry in its permitted position: 24 positive clauses and 72 negative clauses appending a capitalized condition, `Comenzará A Regir`, or `Entrará En Vigor`. All pass. Twelve additional observations after a measured title, signature, or dateline/motto test `Comenzará A Regir`, `Tendrá Vigencia`, `Entra En Vigor`, and a capitalized condition; all emit zero determinations. No table entry itself contains a conditional or commencement statement, so the hypothesized capitalized condition that exactly equals a permitted entry is absent from this finite vocabulary.

The within-record counter is deliberately lexical: folded `entra`, `entran`, `entrará`, `entrarán` followed by `en vigor`. Qualifier checks can take precedence over this reason. It is not a general Spanish event detector. Other verbs still fail the whole-clause match rather than resolving through an unchecked tail. Keeping this counter narrow is acceptable for this bounded recognizer; expanding synonyms is not required for closure. A syntactic dateline date remains acceptable apparatus: it cannot contain an arbitrary statement and is not used to compute the determination. No measured date table is required here.

**Paired census independently reproduced:** literal resolver source from `74a6242a9` compiled alongside the reviewed public API, shared current types/calendar, fixed today and empty calendar. Both sides: **468 instruments; 332 resolved / 1 bounded / 126 skipped / 9 none**. Full Debug-encoded outcomes differ only for `reg-lfppi`, whose reason changes from `BusinessDaysPeriod` to `MultipleCommencementEventsInRecord { count: 2 }`. Its matched text contains two entering-into-force phrases; determinations are identical. All other complete outcome records are identical. This relabel is acceptable and makes no legal decision.

Census artifact SHA-256:

- old: `a3077d7614241a6b6c62fde43f50a824a89a71e1ae0db2e663db75cc9a160561`
- new: `79c3d244303d240bb099e2de7aa468af0ebe2d37626fc43e04782b2f52e07e96`

The `Salvoconductos` test now correctly asserts conservative review. Whole-word `salvo` matching remains unchanged; the independent control confirms the word is not classified as that qualifier. The table policy is an acceptable conservative boundary; unsupported forms remain review work.

### F2 — closed: canonical serialized URL selection and strict followed URLs

The implementation computes `expected = new URL(root + "mx/lritf/index.md", base)` and requires `target.href === expected.href`, retaining explicit origin/path/credential restrictions. The selected URL still passes `safe()`.

Independent hash-valid fixtures containing canonical plus `?`, `#`, `?view=all`, `#toc`, or `?x#y` all complete retrieval of articles 35 and 39 with **full manifest verification enabled**, six files verified. No variant is requested. The new native bare-delimiter fixtures also apply their mutations before manifest hashing and assert `report.ok`, not just discovery.

`safe()` checks origin, credentials, pinned pathname prefix, and serialized query/fragment delimiters on every initial fetch and redirect destination. Independent redirects to targets ending in `?`, `#`, `?x=1`, and `#top` all fail before requesting the destination. Native controls retain rejection/skipping of cross-origin, credentialed and nonempty-component URLs.

Answers to the handoff questions:

1. **Yes, enforce the same component policy on every followed URL and redirect.** This completes the existing no-query/no-fragment contract. Selection may skip unrelated navigation; an actually followed URL must pass the strict policy. This does not require every path to equal the LRITF index: other followed paths remain constrained by the release pin.
2. **Track the base-URL blind spot separately, nonblocking.** Independent `baseUrl` values ending in bare `?` and `#` are accepted and complete full manifest verification. Every actual request URL is component-free because relative resolution clears the base component and `safe()` checks the resulting URL. This is an inconsistent configuration acceptance rule, not a remaining candidate-selection ambiguity or demonstrated pin bypass. Follow-up: reject serialized bare components during initial base validation and add configuration regressions. The current implementation must not be described as rejecting them at configuration time.
3. **Move the older same-index tests to pre-hash construction as test maintenance.** Their current post-hash mutation only supports discovery-focused assertions. This is not a closure blocker: the new bare-delimiter tests and five independent pre-hash full-verification probes establish the material behavior, including the older nonempty cases.

### F3 — remains closed

`RESPONSE_VERSION` and emitted `schema_version` remain `public-query-response-v2`; the runtime validator includes v2. `PROJECTION_VERSION` remains `public-query-v1`. The v2 error schema structurally confines nonempty `skip_cursor` to `response_too_large`; it remains optional for the accepted empty-page case.

The frozen v1 file independently hashes to `636536a8e58f9855fa7812278944fb69c6e6192be0f4bacaf7e9f104d591f559`, matching the accepted 2026-10-25 evidence. Native real-response/version and structural-confinement regressions pass in the reader suite. `git diff aeaf5dbc9..HEAD -- crates/lex-db schemas` is empty: F2 did not change F3 code. Current README/runtime/host amendment point to v2; remaining v1 references are the frozen schema, historical acceptance evidence/completed milestone, and explicit compatibility tests. No stale active consumer was identified.

### R4 — accepted qualification unchanged

Deadline classification and cooperative checks remain accepted. The response adapter and its deadline tests are unchanged in the F2 range and pass the native suite. No hard end-to-end wall-clock cap is claimed or required. No new timing measurement was performed.

## 4. Disposition by earlier finding

| Earlier item | Final disposition |
|---|---|
| C1 / October 28 limit 1 | Closed for the bounded whole-clause recognizer, including residual F1. |
| C2 / limit 4 | Remains closed for typed/schema/semantic input validation; provenance is not independently established by schema validity. |
| C3 | Remains closed; word/accent/whitespace behavior preserved. |
| C4 | Remains closed; split-save persistence regression passes. |
| C5 | Remains closed; historical supersession records unchanged. |
| C6 | Remains closed; shared-calendar/repair regressions pass. |
| R1 | Traversal and explicit skip policy remain accepted; F3 remains closed. |
| R2 | Closed, including residual F2. |
| R3 | Original version-name collision claim remains refuted; no revision. |
| R4 | Accepted with cooperative-deadline qualification. |
| R5 | Full-filename SQLite sidecar fixes remain closed. |
| R6 | URL-bound annex selection/digest checks remain closed. |
| `write_binding` race | No-clobber fix remains closed; no crash-atomicity claim added. |
| Shared number words / limit 3 | No regression identified; prior computational acceptance and legal/evidence qualifications unchanged. |

No earlier closed finding is reopened. The historical legal/evidence qualifications are preserved, not resolved by this review.

## 5. Independent checks actually run

Native commands were delegated to the repository-required mechanical role. All commands below exited 0 except the explicitly identified full-reader-workspace clippy check. Native outputs were captured in tool sessions; no persistent native log directory was created.

Port:

| Exact command | Result |
|---|---|
| `cargo fmt --all --check` | pass |
| `cargo test --workspace --locked --offline` | **576 passed**, 0 failed, 0 ignored |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | pass |
| `cargo run --locked -p lex-cli -- validate lritf` | valid; 145 articles / 11 transitories / 127 references; 0 issues |
| `cargo run --locked -p lex-cli -- validate ifpe-dcg-2021` | valid; 59 articles / 4 transitories / 115 references; 0 issues |
| `git diff 47a3ffc59..HEAD -- corpus` | empty |

Reader:

| Exact command | Result |
|---|---|
| `node --test reader/retrieval-check.test.mjs` | **29 passed**, 0 failed, after approved loopback escalation; initial sandbox attempt had 29 listen-EPERM failures |
| `cargo fmt --all --check` | pass |
| `cargo test --workspace --locked --offline` | **451 passed**, 0 failed, 1 ignored |
| `cargo clippy -p lex-db --all-targets --locked --offline -- -D warnings` | pass |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | exit 101: exactly `main.rs:1905` map-unwrap-or and `db.rs:318` assert-is-empty |

The two CLI diagnostics and unused workspace `walkdir` warning are pre-existing. `git diff 921e1bd6c..HEAD -- crates/lex-cli/src/main.rs crates/lex-cli/src/db.rs Cargo.toml` is empty. No unrelated warning cleanup was attempted.

Additional reviewer-run commands/evidence, scratch root `/private/tmp/lex-rereview3-probes`:

- `cargo run --offline --manifest-path /private/tmp/lex-rereview3-probes/Cargo.toml`: exit 0, at the time the scratch package had one binary; 21 public-API observations plus paired 468-instrument census. `port.log`, `census-old.txt`, `census-new.txt`; literal old source in `src/baseline.rs` (test module omitted).
- `cargo run --offline --manifest-path /private/tmp/lex-rereview3-probes/Cargo.toml --bin table-boundaries`: exit 0, **96 assertions passed**, `table-boundaries.log`.
- `node --test /private/tmp/lex-rereview3-probes/navigation-probes.mjs`: exit 0, **11 passed**, after approved loopback escalation; initial sandbox run had 11 listen-EPERM failures. Five full-manifest navigation successes, four rejected redirect targets, two observational base-delimiter acceptances. `navigation.log`.
- Python source-occurrence scan restricted to instrument `provisions.json` transitories, followed by inspection of the eight matching source records: all 24 table entries found; `table-occurrences.txt`. Corpus files were script inputs, not bulk reviewer context.
- `shasum -a 256` on v1 schema and paired census artifacts: hashes above.
- Scoped Git identity/diff checks above; reader `git diff 921e1bd6c..HEAD -- corpus` empty; both clones' `git diff --check` clean before report creation.

These checks do not constitute a national canonical derivation, pipeline replay, database rebuild, live-site deployment test, or independent legal-source applicability review. No sealed holdout was opened; no forbidden lookup or holdout flag was used.

## 6. Closeout and next action

**The audit is closed for these reviewed code heads. The scoped fixes are approved for integration on the evidence above; production rollout is not approved by this review.** No merge, commit, push, deployment or history rewrite was performed. Integration remains an operator action; this review does not authorize applying prospective classifications over canonical human decisions.

Only `docs/codex-rereview-verdict2-2026-11-18.md` was added to the source trees. Existing docs, verdicts, handoffs, code, tests, schemas, corpus and unrelated untracked inputs remain untouched. Reader remains clean.

Continuity decision: discovery in both clones returned `found: false`. No existing capsule requires checkpointing; no archived capsule was restarted. The explicit one-file write constraint takes precedence over generic plan/receipt/rolling-context administration. This verdict is the authorized completion record; no external receipt or rolling context was written. The review is complete, with no unfinished audit blocker.

**Next action:** operator reviews this verdict and integrates the reviewed fixes, recording the two nonblocking reader cleanups separately. Preserve F3's version boundary and R4's accepted qualification.
