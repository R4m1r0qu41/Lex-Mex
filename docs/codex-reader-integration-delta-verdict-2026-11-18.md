# Reader integration — independent delta verdict, 2026-11-18

## 1. Verdict

**Close: the five scoped deltas are accepted. No required software revision was found.** The two previously nonblocking retrieval-checker cleanups are complete. The model examples preserve main's existing text; which model they should name remains open for JRH.

This is a post-merge software review. PR #7 is already merged; this verdict creates no merge gate and grants no production rollout approval. F1, F2, F3, R4 and other closed findings were not re-audited. F3 retains `public-query-response-v2` and the frozen v1 schema; R4 retains its accepted cooperative-deadline qualification, without a hard wall-clock guarantee.

## 2. Identity and scope

- Review clone: `/private/tmp/lex-reader-delta-20261118/reader`, detached HEAD `6859a9f6abf6352b715ae8896ba3135de5ef6b95` throughout verification.
- Read the requested handoff first, then `docs/codex-audit-fixes-codex-rereview2-2026-11-18.md`. Read the provider-instructions rollout plan, global/local AGENTS and CODEX instructions, PLANS and repository status. Registration is supported by the local `lex-mex` project ID. The companion-file convention and configured `project_doc_fallback_filenames = ["CODEX.md"]` already exist; no adoption edit was needed.
- Merge `165ba88afdbb073113b0f8d1895d441bee198893`: first parent `0d5145d71d7d185e71b8a29baded026bdbc34ea1`, second parent/main `f530d21cf0bcde9ad86af5f3ad7e5fb5c827aea8`; merge base `d607864d2f4df63d034c9eb30e2c8ec7521613cc`.
- The first parent differs from the previously reviewed `6ba814c9a` only in `docs/project-status.md`.
- Follow-ups reviewed individually and cumulatively in `165ba88af..6859a9f6a`: `b47066033` (base URL), `d6b77debc` and `2a97904e3` (fixtures/assertions), `a95e40af3` (Rust assertion), `6859a9f6a` (model examples). That range changes only `CLAUDE.md`, `README.md`, `crates/lex-cli/src/db.rs`, `reader/retrieval-check.mjs` and its test file.
- Local `origin/main` is `70f1f5afea66ac2a2dbfaef97cef97a49a6a3742`. Its parent `a8abdf82858b4546db347bd0a247724de57b4864` has parents `f530d21cf` and the reviewed HEAD `6859a9f6a`, independently confirming the integration ancestry. Reviewed `crates`, `reader` and `schemas` match that integration commit.

Initial worktree state contained two untracked supplied inputs: `docs/codex-reader-integration-delta-brief.md` and `docs/codex-reader-integration-delta-handoff.md`. Both were preserved. The production worktree was not used.

## 3. Findings

### 1 — Merge resolution: accepted

`sqlite_sidecars` occurs once in the merged library and current HEAD. Its function text is identical across reviewed reader `6ba814c9a`, reviewed port `8a74811a8`, both merge parents, the merge result and HEAD. It appends `-wal`, `-shm` and `-journal` to the complete OS filename, preserving the extension and avoiding a lossy string conversion.

Against the reader parent, the library diff only moves `#[must_use]` relative to its documentation and adds main's sidecar test. Against the main parent, it only adds the reader's `pub mod public;`. The test retains the `.db-wal`, `.sqlite-shm` and `.sqlite-journal` checks. `build.rs` and `verify.rs` have identical blob IDs in both parents and the merge. The duplicate is gone without altering the reviewed R5 behavior.

A whole-tree comparison of path modes/object IDs classified 50,421 paths as agreeing between parents, 24,139 as main-only changes, and 41 as reader-only changes. Every such merge result matches the appropriate parent: **zero unexpected changes**. This used tree metadata, not bulk corpus contents.

Eight paths changed on both sides: `CLAUDE.md`, `Cargo.lock`, `PLANS.md`, `README.md`, the library, `docs/decisions.md`, `docs/project-status.md`, and `reader/README.md`. Combined parent diffs were inspected. Beyond the helper, the lockfile combines main's analysis dependencies with the reader's `url` dependency, consistent with the respective Cargo manifests. README/reader README preserve both branches' additions; CLAUDE combines main's continuity updates with the then-existing reader model example, subsequently restored by `6859a9f6a`. No other overlapping Rust implementation exists in that set.

The three documentation resolutions retain both parents' substantive additions:

- `PLANS.md` keeps main's added active plans and the reader's database-query and retrieval plans. One older reader summary for the four-instrument release is replaced by main's newer publication/handoff summary; the plan entry remains.
- `docs/decisions.md` retains all nonblank lines added by each parent relative to the merge base, including main's decision records and the reader's response-v2 decision.
- `docs/project-status.md` likewise retains both parents' added lines and adds the integration update. Historical open/unmerged wording remains historical context; it is not a new authorization or a reason to reopen closed findings.

No conflict markers remain in those three documents. No follow-up fix is required.

### 2 — Base URL serialized delimiters: accepted

The added `/[?#]/.test(base.href)` is the same serialized-href rule used by `safe()`. WHATWG URL getters cannot distinguish an absent component from a bare delimiter; serialization preserves that distinction. Protocol and credential checks remain in place. Valid encoded delimiters in a pathname (`%3F`, `%23`) do not match the new literal-delimiter regex.

The native regressions reject bare `?`, bare `#`, nonempty query and fragment before any HTTP request, and retain ordinary/trailing-slash-free base acceptance. Independent in-memory calls to the actual exported checker also reject `?x#y`, username-only, username/password and password-only credential forms before any fetch. Ordinary root and nested bases, a nested base without its trailing slash, and an encoded-delimiter path all complete successfully with explicit full-manifest verification. No valid base under the existing no-credentials/no-query/no-fragment contract was rejected by these probes.

### 3 — Pre-hash fixtures and full runs: accepted

All four converted call sites move the same mutation from `change` to `prepare`; the mutation text and original navigation/failure assertions remain intact. `prepare` runs before file byte counts, SHA-256 values and manifest content ID are computed. `change` still runs afterward.

The eight expanded tests preserve their intent:

| Cases | Preserved behavior and improved evidence |
|---|---|
| Four unrelated links: home, DOF, anchor, query | Canonical discovery still returns articles 35 and 39; `report.ok === true` now requires successful integrity verification too. |
| Two same-index variants: fragment, query | Canonical discovery succeeds; variant is never requested; overall success is now asserted. |
| Missing exact article label | Still fails on the exact-article discovery error, now with a correctly hashed fixture. Despite the historical test title, this fixture tests the missing-label case, not an additional duplicate-label case. |
| Foreign LRITF-looking link | Still fails candidate selection and never follows the foreign target, now with correctly hashed index bytes. |

The checker always validates the pinned manifest content ID, retrieved index/provision hashes and exact-text companions before returning a successful report. These fixtures have six manifest files, all covered by that default traversal. Thus the six positive native tests exercise the complete fixture's integrity verification. They do **not** set the optional `verifyManifest` flag: `fullManifestVerified` remains false by default. Independently constructed six-file releases for all six navigation cases were additionally run with `verifyManifest: true`; all verified six files and returned `fullManifestVerified: true`.

Leaving the other mutations post-hash is coherent. Function routes supply dynamic server behavior, including method/redirect or request-host-dependent content, and retain discovery-focused or intentional failure assertions. `llms.txt` is outside the hashed release-file list. Manifest corruption and legal-body tampering deliberately invalidate integrity. Moving these wholesale into pre-hash construction would erase the intended fault or try to hash a handler rather than served bytes. No conversion or assertion change is required.

### 4 — Clippy assertion: accepted

The sole change is `assert!(frames(b"", 8).is_empty())` to `assert_eq!(frames(b"", 8), Vec::<Frame>::new())`. `frames` returns `Vec<Frame>`; `Frame` derives `PartialEq` and `Eq`. A vector equals an empty vector exactly when its length is zero. The expression under test and cap are unchanged, and both forms fail for any emitted frame. Workspace tests and clippy pass. No behavior change or required follow-up.

### 5 — Model example preservation: accepted; owner choice remains open

`6859a9f6a` changes only three example lines in `CLAUDE.md` and `README.md`, replacing `gpt-6.1-sol` with `gpt-5.5`. The patch reverses the model-name substitutions in October `f49c07923`; the resulting example lines exactly match merge parent/main `f530d21cf`. `CLAUDE.md` is byte-identical to that main parent at reviewed HEAD; README additionally retains the reader's public-query documentation. There is no runtime/configuration/model-selection change in this commit. No model was invoked. Which model examples should name is left to JRH.

## 4. Answers to the three handoff questions

1. **Yes.** The retained `sqlite_sidecars` function is textually identical to both reviewed implementations, with main's regression test retained and one definition remaining.
2. **Yes.** The base check uses `safe()`'s literal serialized-href delimiter test. Bare/nonempty components and credentials fail before fetch; ordinary and encoded-path controls succeed.
3. **Yes.** All eight expanded cases retain their original assertions/intent. The six successful navigation cases now require a successful integrity-verified run. The deliberately post-hash function, discovery-file and corruption cases remain coherent.

## 5. Independent checks run

Native checks were delegated to the repository-required `mechanical` role in this clone. These are fresh executions, not the handoff's claimed results.

| Exact command | Independent result |
|---|---|
| `cargo fmt --all --check` | Exit 0. |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Exit 0. Existing Cargo warning: unused workspace dependency `walkdir`. |
| `cargo test --workspace --locked` | Exit 0; 616 passed, 0 failed, 1 ignored (`public_release`); doc tests pass. |
| `node --test reader/retrieval-check.test.mjs` | Initial sandbox execution exit 1: all 34 fail at loopback bind (`listen EPERM`). Same command rerun with approved loopback escalation: exit 0, 34 passed, 0 failed. |
| `cargo run --locked -p lex-cli -- validate lritf` | Exit 0; valid, 145 articles / 11 transitories / 127 references; 0 issues. |
| `cargo run --locked -p lex-cli -- validate ifpe-dcg-2021` | Exit 0; valid, 59 articles / 4 transitories / 115 references; 0 issues. |
| `shasum -a 256 schemas/public-query-response-v1.schema.json` | Exit 0; `636536a8e58f9855fa7812278944fb69c6e6192be0f4bacaf7e9f104d591f559`. |
| `git diff origin/main...HEAD -- corpus` | Exit 0; empty. |
| `git diff f530d21cf...HEAD -- corpus` | Exit 0; empty against the integration's actual main parent too. |
| `git diff 6ba814c9a HEAD -- crates/lex-db/src/public.rs crates/lex-db/src/public_tests.rs` | Exit 0; empty. |
| `git diff --quiet a8abdf828 HEAD -- crates reader schemas` | Exit 0; reviewed implementation matches the integration commit. |
| `git diff --check` | Exit 0 before and after report creation. |

Additional parent/revision comparisons actually run: `git show` on every delta commit and `f49c07923`; `git diff-tree --cc --name-only 165ba88af`; combined patches via `git diff-tree -c -p 165ba88af -- CLAUDE.md Cargo.lock README.md reader/README.md` and `git diff-tree --cc -p 165ba88af -- PLANS.md docs/decisions.md docs/project-status.md`; pairwise parent diffs for the library, build and verifier; and manifest comparisons for `crates/lex-db/Cargo.toml` and `crates/lex-analysis/Cargo.toml`. In-memory `python3 - <<'PY'` checks used `git ls-tree -r -z`, `git show`, `git rev-parse`, and `difflib.SequenceMatcher` for the whole-tree inheritance, helper equality, doc added-line retention and example comparisons described above. All completed successfully.

Two `node --input-type=module <<'JS'` probe programs ran without persistent probe files or network access. They imported the actual checker, constructed SHA-256/content-ID-valid in-memory releases, and replaced `globalThis.fetch` with a route-backed Response adapter, restoring it afterward:

- First invocation: **20 scenarios passed** — 8 pre-fetch configuration rejections; 4 successful valid-base controls; 6 successful navigation cases with explicit full-manifest verification; 2 intended discovery failures. Its foreign-only negative used an index replacement after hashing, which cannot establish hash-valid fixture construction for that negative.
- Second invocation independently constructed and hashed the foreign-only index before execution: intended candidate-selection failure, exactly four local GET/HEAD exchanges, no foreign fetch. This removes that probe limitation for the cross-origin case.

The mock probes supplement the passing native loopback suite; they are not live-host compatibility evidence. No pipeline replay, census rerun, production deployment test, legal determination, sealed holdout access, forbidden `structure_pilot_lookup`, or `--holdout-gold` invocation was performed.

## 6. Closeout and next action

**Delta accepted; no required software follow-up.** The example-model choice remains an owner decision for JRH. The handoff's existing operational limitation that CI does not run the Node suite remains; it is not a newly discovered defect or a gate on the already merged PR. Production rollout remains unapproved by this review.

Only `docs/codex-reader-integration-delta-verdict-2026-11-18.md` was added as a deliberate output. Existing source, tests, schemas, corpus, instructions, handoffs and verdicts were not edited; the two pre-existing untracked inputs remain. Native checks produced their ordinary build/cache artifacts. No staging, commit, push, merge, deployment or history rewrite was performed.

Continuity decision: discovery returned `found: false`, and local instructions retire Lex-Mex capsules; no capsule is required or restarted. The explicit one-file write constraint takes precedence over generic receipt, plan, status and Vault rolling-context administration. This verdict is the authorized completion record; no separate receipt or external continuity artifact was written. The bounded review is complete, with no unfinished review blocker.

Next action: JRH reviews this post-merge acceptance record and decides separately whether to change the example model names. Any later fix belongs on a new branch; this review adds no merge gate.
