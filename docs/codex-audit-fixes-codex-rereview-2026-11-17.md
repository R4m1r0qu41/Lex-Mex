# Lex-Mex audit fixes — independent re-review, 2026-11-17

## 1. Verdict

**Revise. The audit cannot close at these heads.** F1 repairs the original three witnesses but still accepts capitalized conditions and a second commencement statement as title/motto/signature tokens. F2 repairs the two required navigation regressions but retains the same ambiguity failure for bare query/fragment delimiters. F3 is closed. R4 is accepted with the previously stated cooperative-deadline qualification.

The counterexamples below are synthetic boundary probes, not findings about any instrument’s legal status. The paired 468-instrument census is unchanged. No merge or rollout is approved.

## 2. Identity and scope

- Port clone: `/private/tmp/lex-audit-fixes-rereview2-20261117/port`, detached HEAD `74a6242a96171843bc4b1d07018813c79f9f2486`; reviewed `117a35504..74a6242a9`. Only `crates/lex-analysis/src/temporal_derive.rs` changes in that range.
- Reader clone: `../reader`, detached HEAD `9500c52d71d570de1fcae03b66d2bcbada06da4f`; reviewed code range `921e1bd6c..aeaf5dbc9c15d2a2731d8b4abc3e07cb167e6129`. `aeaf5dbc9..9500c52d7` changes only `docs/codex-audit-f2-f3-closeout-2026-11-17.md`.
- Read both clones’ AGENTS/CODEX/CLAUDE instructions, the rollout convention, prior verdict, original request, F1 handoff and reader closeout note. Existing companion layout and global fallback already satisfy the convention; no instruction edits were needed. Project registration is supported by the repository’s explicit `lex-mex` project ID.
- Inspected complete scoped diffs, anchored derivation paths, response-schema/runtime wiring, skip traversal, deadline handling and version references in repository consumers/docs. Earlier closed findings were checked for regression through the scoped diffs and native tests, rather than re-audited from scratch. C5’s supersession patch at `a283f85ff` was inspected read-only.

At intake the reader was clean. Port had three supplied nonempty untracked review inputs and an empty untracked destination verdict file. This report fills that destination only. Production worktrees were not modified or used for builds.

## 3. Findings

### F1 — remains open: capitalization is not a verified tail boundary

**Original witnesses repaired.** Through `derive_article_temporal_determinations_with_calendar`, publication `2020-01-01`, today `2026-11-17`, empty calendar, each prior witness now yields zero determinations and `UnrecognizedCommencementPattern`. The literal baseline implementation at `117a35504`, compiled alongside the current public API, yields one determination for each. The plain positive control still yields one exact `2020-01-02` determination.

The four legacy paths use the anchored whole clause. The free abrogation and quoted-string patterns are gone. Restricting abrogation to `y abroga la Ley <title>` and the motto to an `Año de ...` quote within a dated México dateline is an acceptable conservative coverage policy. Unsupported shapes may abstain. The genuine measured forms remain covered by eight positive fixtures, all passing, and the unchanged census below.

However, `LEGACY_NAME_WORD` at line 632 accepts **any** capitalized word; the title/motto repetitions at line 669 therefore accept capitalized verbs and condition words as well. `LEGACY_SIGNATURES` at line 646 retains broad capitalized `\\w`/period name tokens. A capitalized sentence can pass as a name; the comment claiming otherwise is false. The enclosing anchors consume that sentence rather than rejecting it.

Independent public-API probes at this head each produce **one exact `2020-01-02` determination**, pattern `day_after_publication`:

```text
La presente Ley entrará en vigor al día siguiente de su publicación y abroga la Ley de Ascensos Siempre Que Se Expida la Norma.

La presente Ley entrará en vigor al día siguiente de su publicación. Siempre Que Se Expida la Norma, D. P.- Rúbricas.

La presente Ley entrará en vigor al día siguiente de su publicación. México, D.F., a 29 de diciembre de 1974.- "Año de la República Siempre Que Se Expida la Norma".- Juan Pérez, D. P.- Rúbricas.

La presente Ley entrará en vigor al día siguiente de su publicación. Esta Ley Entrará En Vigor Inmediatamente, D. P.- Rúbricas.
```

These deliberately capitalized synthetic inputs expose the residual lexical freedom; they are not claimed to occur in the current corpus. The first and third consume a condition in the newly constrained pieces; the fourth hides another commencement event in the retained signature shape. Lowercase variants outside the link-word set correctly abstain, which does not establish that capitalized tokens are names or apparatus. The resolver counts matching transitory records, not commencement events within one record.

**Required revision:** establish a boundary for recognized titles and source apparatus that cannot treat arbitrary capitalized statements as verified metadata. For example, bind permitted tail spans to verified source/title/apparatus evidence, or conservatively admit only positively established measured forms. Independently reject a second commencement event anywhere in the record. Preserve genuine forms with fixtures and add the witnesses above. Adding exception words alone does not establish the boundary. No legal date needs to be selected to fix this recognition defect.

**Paired census independently reproduced:** 468 committed instruments, 332 resolved / 1 bounded / 126 skipped / 9 none, zero changed full outcome records. Both complete deterministic Debug-encoded API output files have SHA-256 `d899c1eeeab72bcb4cf83d0fe4227b4520261fac88651946d516caf82e68aa67`. This compares the literal pre-fix resolver with the current resolver using shared current types/calendar code, fixed today and an empty calendar. It verifies the handoff’s parity claim for that boundary, including determination contents/counts, without a canonical write or a pipeline replay. It does not establish general tail safety or business-day historical coverage.

### F2 — required regressions repaired; bare delimiters still block closure

At `reader/retrieval-check.mjs:109–115`, origin/path, credential and nonempty search/hash restrictions are applied before counting candidates; `safe()` remains on the selected fetch. The native tests pass for canonical plus `#toc`, canonical plus `?view=all`, credentials, and the cross-origin control.

Independent hash-valid synthetic releases confirm full successful retrieval with canonical plus `#toc`, `?view=all` or `?x#y`; none fetches the variant. But canonical plus either of these fails after four HTTP exchanges, before fetching the instrument index:

```markdown
[Navigation](mx/lritf/index.md?)
[Navigation](mx/lritf/index.md#)
```

Result: `discoveryCompleted: false`, `Expected exactly one LRITF index link`. Empty `URL.search`/`URL.hash` do not distinguish absent components from present empty components. A bare `#` is ordinary navigation; this is a remaining instance of the selection defect, rather than an off-origin escape. **It blocks F2 closure under the requested eligible-full-URL rule.**

**Required predicate:** compare the resolved serialized URL with the exact canonical URL, retaining the origin/path/credential restrictions. For example, compute `expected = new URL(root + 'mx/lritf/index.md', base)` and require `target.href === expected.href`. Serialization retains bare `?`/`#`, so those variants are excluded. Keep strict selected-fetch enforcement; add both duplicate-navigation regressions. Apply the same exact-component policy to `safe()` if it is intended to enforce the no-query/no-fragment rule everywhere. Raw relative href equality is unnecessary and would reject legitimate equivalent relative/absolute spellings.

The supplied extra-navigation fixtures mutate the index after computing manifest hashes and therefore mainly test discovery. The independent fixture applies the mutation before hashing, so its successful controls also pass integrity verification.

### F3 — closed: coherent response v2, frozen v1

- `RESPONSE_VERSION` and emitted `schema_version` are `public-query-response-v2`; `RESPONSE_SCHEMA` includes the v2 schema used by the runtime validator.
- v1 SHA-256 independently matches `636536a8e58f9855fa7812278944fb69c6e6192be0f4bacaf7e9f104d591f559`.
- v2 structurally confines `skip_cursor` to `response_too_large` through `if`/`then`. Independent validation checked all 14 error codes with and without the field, plus its empty-string rejection. Resetting the response discriminator to v1 still rejects `skip_cursor`; removing the field makes that v1 response valid. Thus rejection is not merely a version-discriminator mismatch.
- Native real-response tests confirm oversized and ordinary responses emit v2 and validate against v2. The oversized-item traversal regression passes, maintaining explicit opt-in continuation and bound request/release cursors. Optional `skip_cursor` is acceptable because an oversized empty page has no item to skip.
- `PROJECTION_VERSION` remains `public-query-v1`. Scoped diffs leave corpus, database-builder/projection and public-release production paths/artifacts unchanged. No database rebuild was performed or byte-reproducibility claim inferred from a rebuild.
- Repository version-reference search finds current README/runtime/host handoff coherently on v2; v1 remains in the frozen schema and explicitly historical acceptance evidence and completed D1 milestone row. No stale active consumer/fixture was identified. This is a new wire contract, not compatible v1.

### R4 — accepted qualification, no new requirement

Accepted as deadline classification and cooperative checks. The code and native deadline tests remain unchanged by this review range, including expiry after dispatch. They do not interrupt every long non-SQL computation or establish a hard end-to-end wall-clock cap. The operator has not required such a cap, so this is **not an open audit item**. The prior 325 ms measurement was not rerun or presented as a new measurement here.

## 4. Disposition by earlier finding

| Earlier item | Disposition |
|---|---|
| C1 / October 28 limit 1 | Original business-day and original F1 witnesses repaired; broad whole-clause guarantee still blocked by residual F1 above. |
| C2 / limit 4 | Typed/schema/semantic calendar-input validation remains closed; passing validation does not independently verify provenance. |
| C3 | Word/accent/whitespace guard fix remains closed. |
| C4 | Split save-gate fix remains closed; persistence regression passes. |
| C5 | Supersession notes preserve historical measurements; closed, unchanged. |
| C6 | Shared-calendar date repair remains closed; native regressions pass. |
| R1 | Traversal and explicit skip policy remain accepted; F3 contract integration now closed. |
| R2 | Required nonempty fragment/query regressions fixed; residual F2 remains open. |
| R3 | Original version-name collision claim remains refuted; no new revision. |
| R4 | Accepted with cooperative-deadline qualification above. |
| R5 | Full-filename SQLite sidecars remain closed in both applicable paths. |
| R6 | URL-bound annex selection and digest checks remain closed. |
| write_binding race | Prior create_new no-clobber fix remains closed; no new crash-atomicity claim. |
| Shared number words / limit 3 | No regression found; computational safeguard acceptance and existing legal/evidence qualifications remain unchanged. |

The only closure blockers are residual F1 and F2. No other earlier closed finding is reopened by the scoped changes.

## 5. Independent checks actually run

Native checks were delegated to the instructed mechanical role; interpretation, independent probes and this verdict remained with the reviewer. Logs: `/private/tmp/lex-rereview2-native-20261117/`.

Port, from the port clone:

| Exact command | Result |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo test --workspace --locked --offline` | exit 0; **575 passed**, 0 failed |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | exit 0; existing unused workspace `walkdir` warning |
| `cargo run --locked -p lex-cli -- validate lritf` | exit 0; valid, 145 articles / 11 transitories / 127 references, 0 issues |
| `cargo run --locked -p lex-cli -- validate ifpe-dcg-2021` | exit 0; valid, 59 articles / 4 transitories / 115 references, 0 issues |
| `git diff 47a3ffc59..HEAD -- corpus` | exit 0; empty, 0 bytes |

Reader, from the reader clone:

| Exact command | Result |
|---|---|
| `node --test reader/retrieval-check.test.mjs` | initial sandbox run: 23 listen-EPERM failures; rerun with loopback permission: **23 passed**, 0 failed, exit 0 |
| `cargo fmt --all --check` | exit 0 |
| `cargo test --workspace --locked --offline` | exit 0; **451 passed**, 0 failed, 1 ignored |
| `cargo clippy -p lex-db --all-targets --locked --offline -- -D warnings` | exit 0; existing unused workspace `walkdir` warning |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | exit 101; precisely the disclosed `main.rs:1905` map_unwrap_or and `db.rs:318` assert_is_empty errors |

The entire reader `main.rs`, `db.rs` and root `Cargo.toml` are byte-identical to `921e1bd6c`, independently compared using `git show` against on-disk bytes. The two diagnostics and unused `walkdir` declaration therefore predate these commits. No unrelated fix was attempted.

Additional exact commands and evidence, scratch root `/private/tmp/lex-rereview2-probes-20261117`:

- `cargo run --offline --manifest-path /private/tmp/lex-rereview2-probes-20261117/Cargo.toml`: original nine public-API observations plus paired 468-instrument census; `port-probes.log`, `census-old.txt`, `census-new.txt`. At invocation the harness had a single binary. Baseline source is `src/baseline.rs`, captured with `git show 117a35504:crates/lex-analysis/src/temporal_derive.rs`, test module omitted. Initial scratch compilation needed the repository lockfile and missing lex-grammar dependency; final run exited 0.
- `cargo run --offline --manifest-path /private/tmp/lex-rereview2-probes-20261117/Cargo.toml --bin extra-tail`: 11 observations (the original nine plus second-event signature and motto-condition probes); exit 0, `extra-tail.log`. Four residual accepting cases are reproduced above.
- `cargo run --offline --manifest-path /private/tmp/lex-rereview2-probes-20261117/Cargo.toml --bin schema`: **33 independent schema assertions**, exit 0, `schema.log`.
- `node --test /private/tmp/lex-rereview2-probes-20261117/navigation-probes.mjs`: **5 observational tests passed**, exit 0, `navigation-probes.log`. Three successful full-retrieval controls and two asserted discovery failures; the passing observation tests do not accept the defect. Loopback permission was needed after initial EPERM. An initial scratch fixture exposed the supplied post-hash mutation issue and was corrected before the final five-case run.
- `shasum -a 256 ../reader/schemas/public-query-response-v1.schema.json`: frozen hash above. `shasum -a 256` on both census artifacts: equal hash above.
- `git -C ../reader diff aeaf5dbc9..HEAD --name-only`: only closeout note. `git -C ../reader diff 921e1bd6c..HEAD -- corpus`: empty. `git diff --check`: clean before report creation.

The census consumes only instrument/provision JSON as script inputs. No bulk corpus text was loaded into reviewer context. No holdout file was opened, no `--holdout-gold` or forbidden structure lookup ran, and no legal interpretation was made.

## 6. Closeout and next action

Only this verdict file was written in the reviewed source trees. Existing review inputs/handoffs and unrelated local state are preserved. No code, corpus, schema, test, existing document or provider-owned configuration was edited. Nothing was staged, committed, pushed, merged or deployed.

Continuity decision: both clone discovery commands reported `found: false`; no existing capsule required checkpointing, and archived capsules were not restarted. The operator’s one-file write restriction takes precedence over generic plan/receipt/rolling-context administration. This self-contained verdict is the authorized repository record; no external receipt or rolling note was written. The review itself is complete, with implementation work left to the builder.

**Next action:** narrow/verify the residual F1 tail tokens and complete exact-URL selection for F2, then return those bounded changes with the counterexamples as regressions. F3 needs no further revision; retain R4’s accepted qualification. **Audit remains open; merge and rollout are not approved.**
