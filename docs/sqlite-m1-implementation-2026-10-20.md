# SQLite M1 implementation — 2026-10-20

> Historical milestone record. Current M1 review repairs, seed v2, per-file
> provenance and expanded inventory are documented in the
> [review response](sqlite-m1-review-response-2026-10-20.md).

## Outcome

M1's executable contract foundation is implemented on `db/sqlite-projection`. Current main `bfcda816f46b8b0c1d694fc1f2cfb8b4438275b2` was merged into the database worktree without rewriting either branch. The two documentation conflicts were resolved with the preserved refreshed DB plan and current main's corpus status. The local integration commit follows the final checks; no push or full-corpus database build is part of this milestone.

The previously refreshed input/seed evidence remains intact. No corpus file differs from bfcda816f, no source is reparsed, and no legal-review record is resolved or migrated.

## Implemented boundary

- `crates/lex-db`: working fixture builder and independent read-back verification, consumed by the CLI; not an empty crate. `lex-core` has no SQLite dependency.
- `rusqlite = 0.40.2` with bundled SQLite, locked in Cargo.lock. Selection checked against the [official release](https://github.com/rusqlite/rusqlite/releases/tag/v0.40.2) and [crate documentation](https://docs.rs/rusqlite/0.40.2/rusqlite/). The tested bundled runtime is 3.53.2.
- Closed version-1 manifest and structural-debt schemas, matching Rust types and validators. Release identity binds a serialized specification; the specification has sorted unique input paths, content hashes, corpus/builder commits, Cargo.lock digest, toolchain/SQLite settings, inclusion policy and stable source timestamp. Database and logical-content checksums are sidecar manifest fields, not self-checksums inside the DB.
- The production structural-debt seed promotes exactly the 17 operator-accepted rows. Deferred/rejected/repaired rows remain documentary evidence. Validation checks ancestry, decision hash, blob OID/hash, Unicode source span, container ownership, duplicate identity and whether the target is now separately parsed. Any drift fails closed.
- A read-only Git input inspector requires full commit IDs, classifies every selected canonical JSON file and rejects unknown canonical layouts/names. Required files and the production seed must exist at that commit. It never reads dirty corpus files as committed input.
- SQL DDL for the planned record groups, source anchors, ordered children, reviews, coverage, debt, evidence/catalogue, reproducibility and FTS. M1's insertion/reconstruction covers instruments, provisions, edges, ordered qualifiers, separate approvals/operator reviews, definitions, usages and glossary coverage. The remaining table loaders belong to M2; their DDL is not evidence of a populated full projection.
- Raw fixture bytes are retained independently. Relational records use mapped columns plus extras; arbitrary optional metadata survives, including explicit null, absent keys, arrays and number kinds. Reviews remain independent one-to-one objects. Fixture verification reconstructs rows without the insertion transformation and compares directly to the raw JSON values.

The fixture contains title-anchored references, unavailable and approved historical targets, Unicode spans, annex/transitory records, inciso terms, lower-case usages, unknown future metadata and independent review objects. Additional tests exercise cross-instrument usages, glossary accounting presence, missing/empty seeds and dirty Git snapshots. Its texts and reviewer identities are explicitly synthetic and do not represent legal decisions.

## Input handling matrix

| Input | Policy |
|---|---|
| instrument, provisions, references, terms, usages, glossary coverage | Normalized records plus exact raw input |
| operational/formal/annex source manifests; enactment evidence | Normalized policy; remaining insertion/read-back loaders are M2 |
| validation, reform evidence, amendment legends, temporal requests/results, review queue | Preserved evidence-only inputs; not inferred law provisions |
| adapter JSON, schemas, accepted gaps, production debt seed, pinned decision/review evidence | Evidence/contract inputs, inventoried and hashed |
| standard metadata and validation | Catalogue evidence only; no claim of fresh standards validation |
| standard clauses, transitories and supplements | Explicit declared body exclusion: digests/inventory only; no provision or FTS rows |
| generated markdown directories, including their JSON sidecars | Outside canonical selection; never duplicate canonical evidence |
| unknown JSON inside the canonical selection | Fail with path diagnostic |

The real input probe initially stopped on `corpus/mx/lcmopfih/markdown/enactment-evidence.json`. The inspector now explicitly excludes generated presentation directories. A pinned-Git regression keeps an invalid-JSON sidecar there and proves it cannot enter canonical inputs. No canonical file was removed or changed.

## Commands and scope

```bash
cargo run --locked -p lex-cli -- db contract-probe
cargo run --locked -p lex-cli -- db check-seed --corpus-commit bfcda816f46b8b0c1d694fc1f2cfb8b4438275b2 --seed adapters/shared/_known-structural-debt.json
cargo run --locked -p lex-cli -- db inspect-inputs --corpus-commit FULL_COMMIT_CONTAINING_SEED
```

`contract-probe` builds only an in-memory adversarial fixture. `check-seed` explicitly checks a candidate seed file against pinned source blobs; it is not a mixed-input release build. `inspect-inputs` requires a seed committed at the selected full commit, and inventories/parses inputs; it does not claim full-corpus semantic verification. M2 must invoke the existing corpus validation contracts before publishing a candidate. M3 supplies independent release verification and query acceptance. There is no `build-db` publication command yet.

The full input inspection used an unreferenced private Git commit object based on bfcda816f plus the seed and two new schemas. It moved no branch and is not a release or builder-commit pin. It exercised 4,917 input files (181,905,055 bytes inventoried): 3,287 normalized, 1,534 evidence-only, 96 explicitly excluded standards-body files. It identified 468 federal instruments, 32 standards and 17 accepted debt rows. Metadata and full inspection digest are in the companion evidence file.

## Validation actually run

- `cargo fmt --check` — pass on final source.
- `cargo clippy --workspace --all-targets -- -D warnings` — pass on final source.
- `cargo test --workspace --quiet` — 373 tests pass, including seven new M1 tests.
- Required locked LRITF and IFPE validations — pass, zero issues (145/11/126 and 59/4/115 articles/transitories/references respectively). Canonical corpus unchanged by subsequent inventory-only changes.
- Locked CLI feature/fixture probe — pass: SQLite 3.53.2, FTS5, JSON and STRICT enforced, 12 logical records independently reconstructed.
- Production candidate seed — 17 accepted rows validate against bfcda816f.
- Full real-input inspection — exit 0 on the private candidate object. The missing-seed and unknown-input controls fail as expected in tests.
- Mutation checks reject lost optional metadata, numeric-kind changes, missing independent review, changed usage spans and qualifier ordinal gaps. Duplicate IDs, missing live targets, unsupported versions and stale debt bindings fail; fixture transactions roll back on invalid input.
- `git diff bfcda816f -- corpus` — empty. Main worktree's two untracked suggestion files preserved.

Initial development failures were fixture enum/review-field mistakes, a mutation that collided with a primary key before reaching the verifier, and strict lint limits; all were corrected before the final gate. They were not source/parser defects. A successful M1 fixture does not certify full-corpus DB parity.

## Remaining milestones

M2: implement the transactional full pinned-input loader, complete normalized record loaders, safe immutable artifact output and failure/recovery gates. M3: independent release read-back verification, FTS/query fixtures and corruption checks. M4: full selection builds, reproducibility/size evidence and acceptance. The 17 remaining structural-debt rows, five accepted graph gaps, six held glossaries, source-retention limits and pending legal reviews remain qualifications; M1 does not repair them.

Exactly one next action: review M1's schema/provenance boundary, then implement M2 against the committed integration input; do not start serving, MCP or temporal inference.
