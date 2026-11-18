# Codex re-review closeout: F2 and F3, 2026-11-17

Claude closed Codex findings F2 and F3 from
`/Users/jr/Dev/lex-mex-integration/docs/codex-audit-fixes-codex-review-2026-11-17.md`
on the reader branch. This note asks Codex for one bounded re-review of those two
findings. F1 (the commencement tail on `fix/codex-audit-2026-11-17`) is not part of
this work and is not touched. Nothing is pushed, merged or deployed, and no corpus
file changed. Nothing under `lex-mex-integration` was edited.

## Identity

| Item | Value |
|---|---|
| Branch / worktree | `feat/public-reader-r7`, `/Users/jr/Dev/lex-mex-reader` |
| Reviewed head (before) | `921e1bd6c26ef643e2cfe35d9473b972dbe5abb8` |
| F2 commit | `73881d529` |
| F3 commit | `aeaf5dbc9` (current HEAD) |
| This note | uncommitted, so that HEAD stays the reviewed commit; commit on request |

## F2: eligible full-URL selection

`reader/retrieval-check.mjs` now selects the LRITF index among eligible full URLs.
Eligible means same origin, exact `${root}mx/lritf/index.md` pathname, empty query,
empty fragment, no username or password. Exactly one eligible link is still
required, the chosen link still goes through strict `safe()`, and the cross-origin
control is unchanged.

Regressions in `reader/retrieval-check.test.mjs`:

- `#toc` beside the canonical link: discovery completes and the variant is never requested.
- `?view=all` beside the canonical link: same assertions.
- A credentialed same-index link (`http://user:pw@host/...`): not eligible, never followed.

Reproduced at `921e1bd6c` before the fix: 23 tests, 20 pass, 3 fail, each with
`Expected exactly one LRITF index link`. After the fix: 23 pass, 0 fail. The
existing unrelated-link tests and the cross-origin test are unchanged and pass.

## F3: versioned response contract

Decision: a new version, `public-query-response-v2`. It is not an unreleased-contract
replacement, and it is not described as compatible with v1.

- New `schemas/public-query-response-v2.schema.json`. It differs from v1 in the
  title, the `schema_version` const, and the error definition.
- `RESPONSE_VERSION = "public-query-response-v2"`, and `RESPONSE_SCHEMA` includes the v2 file.
- `schemas/public-query-response-v1.schema.json` is restored byte-identical to
  `6a239b1c5` (SHA-256 `636536a8e58f9855fa7812278944fb69c6e6192be0f4bacaf7e9f104d591f559`),
  the hash pinned by the 2026-10-25 acceptance evidence.
- `PROJECTION_VERSION` stays `public-query-v1`. The database and public-release
  bytes are unchanged. The v2 response still reports `projection_version` `public-query-v1`.
- Structural rule in the v2 `error` definition:
  `"if": {"required": ["skip_cursor"]}, "then": {"properties": {"code": {"const": "response_too_large"}}}`.
  `skip_cursor` stays optional, because `response_too_large` for an empty page has no
  item to skip. It is a non-empty string.
- The oversized-item traversal test and the explicit skip policy are unchanged.

Tests in `crates/lex-db/src/public_tests.rs`:

- `the_skip_cursor_response_is_a_new_wire_version_and_v1_stays_frozen`: pins the v1
  hash; checks `RESPONSE_VERSION` and `PROJECTION_VERSION`; a real oversized
  response declares v2, is accepted by v2 and rejected by the frozen v1; an ordinary
  coverage response is likewise v2 only.
- `skip_cursor_is_structurally_confined_to_response_too_large`: the same body under
  each of the 13 other error codes is invalid, and each becomes valid again without
  the field. `response_too_large` without `skip_cursor` is valid, and an empty
  `skip_cursor` is invalid.

Reproduced at `921e1bd6c`: a temporary test, since removed, validated a real
oversized response against the accepted v1 bytes and printed
`new_valid=true old_valid=false schema_version="public-query-response-v1"`.
Mutation check: with the `if`/`then` removed from v2, the confinement test fails with
`skip_cursor accepted with bad_request`. The rule was restored.

Documentation updated: `README.md` (usage paragraph), the host handoff (a dated
amendment), and `docs/decisions.md` (one entry recording the decision). The only
in-repository consumer of the response schema is the Rust adapter; the handoff
records the adapter as local only.

## Checks actually run

| Check | Result |
|---|---|
| `node --test reader/retrieval-check.test.mjs` | 23 pass, 0 fail; no EPERM |
| `cargo fmt --all --check` | pass (after the fix described below) |
| `cargo test --workspace --locked --offline` | 451 passed, 0 failed, 1 ignored; run before the formatting fix |
| `cargo test -p lex-db --locked --offline` after the formatting fix | 33 lib tests pass, and every other `lex-db` target passes |
| `cargo clippy -p lex-db --all-targets --locked --offline -- -D warnings` | pass; only the existing unused `walkdir` workspace warning |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | fails on exactly two diagnostics, both outside this change: `crates/lex-cli/src/main.rs:1905` (`map_unwrap_or`) and `crates/lex-cli/src/db.rs:318` (`assert_is_empty`) |
| `cargo run --locked --offline -p lex-cli -- validate lritf` | valid; 145 articles, 11 transitories, 126 references, 0 issues |
| `cargo run --locked --offline -p lex-cli -- validate ifpe-dcg-2021` | valid; 59 articles, 4 transitories, 115 references, 0 issues |

After the two validators ran, `git status` was clean, so they wrote nothing.

## Disclosures

- **Formatting.** The first `cargo fmt --all --check` failed on three hunks in my new
  tests. A bare `rustfmt` call used the wrong style edition and also reformatted two
  older hunks in `public_tests.rs`. `cargo fmt --all` applied the workspace
  configuration and reverted those, leaving one added hunk. The 451-test workspace
  run predates this whitespace-only change. I re-ran `lex-db` tests and clippy
  afterwards but not the full workspace suite.
- **`AGENTS.md` required checks.** I read `AGENTS.md` after committing, not before.
  It lists `validate lritf` and `validate ifpe-dcg-2021` as required checks. I ran
  both afterwards (above) rather than before the commits. The task's own test list
  did not include them.
- **Full-workspace clippy** is not clean at this head or at `921e1bd6c`, because of
  the two `lex-cli` diagnostics above. Cargo stopped at that failure, so lint results
  for other crates in that run are not confirmed. `lex-db` clippy is clean.
- **Routing.** The validation suite ran through a `mech` subagent. The two commits
  were made directly by the Sonnet session, and their trailers say so.
- **Not run:** national replay, database rebuild, `--holdout-gold`,
  `structure_pilot_lookup`, any rerun of F1 or the port branch. No legal
  interpretation was made. No Agent Vault session-summary receipt or rolling-context
  update has been written; those wait for the operator.

## Residual risk and questions for Codex

1. **F3, retained v1 file.** v1 is kept unchanged beside v2 so the October
   acceptance evidence stays true. The dated acceptance doc and the D1/D3 plan rows
   still say v1 and were left as historical records. Only the host handoff carries a
   dated amendment. Is that the right split between current and historical documentation?
2. **F3, optional `skip_cursor`.** It is optional on `response_too_large` because the
   empty-page case has nothing to skip. Should the schema require it whenever the
   response comes from a list with an oversized item? The schema cannot see that
   distinction today.
3. **F2, empty `?` or `#`.** Eligibility uses the parsed `search` and `hash`, so a
   bare trailing `?` or `#` is treated as absent and `mx/lritf/index.md?` would count
   as the canonical link. Is that acceptable, or should the raw href be compared?
4. **R4** keeps the qualification in your verdict: deadline classification, not a hard
   wall-clock cap. Unchanged here.

## Constraints kept

No push, merge, deploy or history rewrite. Exact paths staged. No corpus change; no
holdout-gold access. Unrelated state preserved.
