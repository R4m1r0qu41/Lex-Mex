---
artifact_type: provider-handoff
status: historical
project: lex-mex
date: 2026-10-20
from: claude
to: codex
---

# Lex-Mex: Claude's independent review of SQLite M1 (`6505baf57`)

**Repository:** `/Users/jr/Dev/lex-mex-db`, branch `db/sqlite-projection`, local merge `6505baf57c7aa7051cffe62742f46f10276a834a` (parents `04cc039af` and main `bfcda816f`).
- **Scope:** the M1 schema and provenance boundary. That covers `crates/lex-db/**`, `crates/lex-cli/src/db.rs`, `schemas/database-manifest.schema.json`, `schemas/structural-debt.schema.json`, `adapters/shared/_known-structural-debt.json` and `fixtures/database/adversarial.json`, read against `docs/plans/sqlite-corpus-database.md` and `docs/sqlite-m1-implementation-2026-10-20.md`.
- **Reviewer tier:** Claude Opus 5.5 (a different model family from the author), with an advisor pass. This is not a legal review.
- **Read-only:** the DB worktree was not edited, staged or committed. Builds used a scratchpad `CARGO_TARGET_DIR`, and the worktree was clean afterwards.
- **Recommendation:** this review could be committed to the DB branch as `docs/sqlite-m1-review-2026-10-20.md` by that branch's writer.

## Verdict

M1's executable contract is sound in most respects. Its one real gap is that **the seed validator does not bind the seed to the review decisions**. Settle findings 1 and 2 before the M2 loaders are written. Findings 3 and 4 are scope and documentation fixes that can be made alongside. The committed 17-row seed itself is correct (see the verified items below). The gap is in what the validator would accept.

## Findings

### P1 — `validate_seed` accepts operator acceptance it cannot trace

`crates/lex-db/src/contract.rs` `validate_seed` checks each row's locator: ancestry, blob OID and hash, Unicode span, container ownership and the "not separately parsed" condition. Nothing compares the set of rows against the accepted rows in the machine review evidence. The seed's only decision binding is a whole-file hash of `docs/decisions.md`.

Reproduced at the pinned commit with `bfcda816f…`, in both directions:

| Scratch seed | Change | Result |
|---|---|---|
| `seed-16.json` | `lgs 192 Sextus` row dropped | `validated 16 accepted debt rows` |
| `seed-promoted-deferred.json` | `oaac-dcg-2009 12 Bis` added. Review status is `deferred` ("ruling 3: held DCG candidates wait for the DCG pass"); set to `accepted`/`operator` | `validated 18 accepted debt rows` |
| `seed-promoted-rejected.json` | `lftr 180 Quintes` added. Review status is `rejected` ("LFTR bracketed bodies are legally removed text"); set to `accepted`/`operator` | `validated 18 accepted debt rows` |

Each 18 is the 17 committed rows plus the one promoted row. The promoted rows' locators are copied unchanged from the review JSON, so every locator check passes.

```bash
S=/private/tmp/claude-501/-Users-jr-Dev-lex-mex/6531988a-4d65-412d-be2a-fabd22c36e40/scratchpad
CARGO_TARGET_DIR=$S/target cargo run --locked -q -p lex-cli -- db check-seed \
  --corpus-commit bfcda816f46b8b0c1d694fc1f2cfb8b4438275b2 --seed $S/seed-promoted-rejected.json
```

**Why it matters:**
- A promoted row presents an operator rejection as operator acceptance. That is legal-review state, and the repository never infers operator approval.
- A dropped row turns a known missing article into a plain absence. That contradicts the 2026-10-17 decision that "absent" is never read as "does not exist".
- The `pinned_inputs.rs` test asserts that an empty seed is valid, so this behavior is currently enforced by a test.

**Suggested repair.** You choose the mechanism. Requirements:
- Bind the seed to the review evidence: `docs/sqlite-structural-debt-review-2026-10-19.json` (path and sha256), read at `source_corpus_commit`.
- Require the seed rows to equal that file's `review_status == "accepted"` rows exactly, both by identity and by locator fields (span, container, blob OID, reviewer, decision reference).
- Keep an empty seed valid only when the review evidence has zero accepted rows.
- Add tests for the promotion and the drop.

### P2 — The decision binding reads `docs/decisions.md` at the input commit

`validate_seed` hashes `git.read(&seed.decision_source_path)`, and `GitInput::read` reads at the input commit, not at `source_corpus_commit`.
- **Consequence:** the next main commit that adds any decision entry makes `inspect-inputs` fail with "debt decision source changed" for every later corpus commit. M2's first integration of main will hit this.
- That failure is closed, so it is safe. But it turns re-binding the seed into a routine mechanical step, and a step done routinely tends to stop being reviewed.
- The whole-file hash also does not pin the specific ruling text.
- **Suggestion:** read the decision source at `source_corpus_commit`, and at the input commit check only that the cited section (heading plus body digest) is still present and unchanged. P1's repair makes the review JSON the primary binding anyway.

### P2 (settle before the M2 loaders) — Ordinal scope and row provenance in the DDL

`crates/lex-db/schema.sql` has two kinds of uniqueness:
- `reference_edges`, `defined_terms`, `term_usages` and `instruments` are unique on `ordinal` across the whole table;
- `provisions` is unique on `(instrument_id, ordinal)`.

The fixture hides the difference because each group is one array. If M2 loads file by file, these ordinals restart at 0 and collide.
- No column records which input file and which array position a row came from.
- Per-file parity against `input_files` needs that, and cross-instrument usages make ownership hard to infer.

**Choose one before writing loaders:**
1. a per-file ordinal, with a foreign key to the owning `input_files.path`, unique on `(path, ordinal)`; or
2. a global ordinal with documented semantics, plus a separate source-path column.

### P3 — Inventory scope differs from the plan and the M1 report

`inspect-inputs` at `6505baf57` exits 0 with 4,917 files. The output is saved at `$S/inspect-6505.json`.
- **Review evidence is missing.** The only `docs/` paths are `decisions.md` and the two parser-sweep files. `docs/sqlite-structural-debt-review-2026-10-19.json`, the candidates file and the contract-refresh JSON are not inventoried. The report's matrix says "pinned decision/review evidence" is inventoried and hashed.
- **`corpus/reference/scjn-invalidity-registry/` is neither selected nor rejected.** It contains `source-manifest.json`, a PDF and an MD file. The selector only reads `corpus/mx/`, so this is a silent drop, which the plan's "no silent drop" rule forbids. Include it or exclude it explicitly: that is your call or the operator's. I make no recommendation.
- **Non-JSON files under `corpus/mx/` are skipped silently.** These are the 32 standards' `extracted-text.txt` files. That is harmless under catalogue-only scope, but the matrix should declare it.
- **The dated document paths are hard-coded** (`parser-sweep-review-2026-10-19.*`) in `input.rs`. A rule or a declared list would age better.

### Notes (one line each, none blocking)

- **`unresolved` is missing from the `resolution_status` CHECK** in `reference_edges`, although `lex_core::ReferenceResolutionStatus::Unresolved` exists. There are 0 such edges at `bfcda816f`, so this fails closed. Map it, or document the refusal and give it a diagnostic.
- **`verify_fixture` trusts the database's own `input_files` row** for the raw bytes and hash. M3 must verify against the sidecar manifest and the Git blobs instead.
- **`instruments` lacks the `slug UNIQUE`** that the plan's table specifies. Fine for M1; it is part of M2's work.
- **`release_id` hashes `serde_json::to_vec(BuildSpecification)`**, whose key order follows the Rust struct's field order. Reordering fields changes the identity under the same `schema_version`. Either pin canonical JSON or note it in the struct.

## Verified (passes)

- The 17 seed rows are all among the 77 rows that were `proposed` and series-checked in `sqlite-structural-debt-review-2026-10-17.json`. The operator accepted those 77 in the 2026-10-18 decision ("all 77 series-checked rows are accepted as `merged_heading` debt"). So `reviewer: operator` on the committed rows is traceable.
- **The foreign key on the generated `live_target` column is enforced.** A `resolved` edge to a missing provision is rejected with `FOREIGN KEY constraint failed` (Python sqlite 3.51.0, the same DDL without FTS).
- **The `lcmopfih/markdown/enactment-evidence.json` exclusion loses nothing.** The file is byte-identical to the canonical `corpus/mx/lcmopfih/enactment-evidence.json`.
- **`resolution_rule` is absent only where expected.** The 2,606 edges without it are all in `cub-dcg-2005`, `socap-sofipo-dcg-2006`, `cucb-dcg-2004`, `oaac-dcg-2009` and `lftr`. These are exactly the five slugs in `_closure-accepted-gaps.json`, matching the plan's NULL ("rule not recorded") rule.
- **The inventory reads Git blobs, not the worktree.** It uses `cat-file blob` and `ls-tree -z`, path components are restricted to normal ones, and the full 40-character OID is required.
- **Validation run:** `cargo test --locked -p lex-db` passes (the contracts and pinned-inputs suites). `db check-seed` on the committed seed is not separately rerun. Its scratch variants are above. `db inspect-inputs` at `6505baf57` exits 0, with 4,917 files.

## Acceptance for the response

1. Fix or rebut P1, with tests for both the promotion and the drop.
2. Choose the ordinal and provenance shape before the M2 loaders.
3. Correct the matrix and scope in the M1 report, or the code.

There is no legal decision, push or main-branch change in this review.


Repository note: this is the independent review of baseline `6505baf57`, retained as historical evidence. The response and current disposition are in [the M1 review response](sqlite-m1-review-response-2026-10-20.md).
