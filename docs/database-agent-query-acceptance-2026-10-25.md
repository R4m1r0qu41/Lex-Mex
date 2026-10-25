# Database agent query — acceptance record, 2026-10-25

> **Update after the independent review.** The Codex review
> (`database-agent-query-codex-review-2026-10-25.md`, commit `af6de68a8`)
> marked D3 repairs-required. See "D3 repairs (R1, R2)" at the end. The
> sections below record the original local evidence; the review, not the
> earlier wording here, is the authority on D3.

Gates D0–D3 of `plans/database-agent-query-integration.md` are implemented and
verified locally. Independent Codex review is queued and has **not** run. This
is technical evidence, not publication approval or a legal determination. No
corpus, accepted DB, public release or host file changed. Nothing is committed.

Companion evidence: `database-agent-query-acceptance-2026-10-25-evidence.json`
holds hashes, the measured request mix, latency tables and query plans.

## D0 — inputs reconciled

- Reader checkout `f49c07923` (`feat/public-reader-r7`). Its code is identical
  to main `e3ad94ee3` for `crates/`, `schemas/`, `Cargo.toml` and
  `Cargo.lock`. DB checkout `047b421ef`.
- Accepted DB release
  `afdc5acf0b5017d73cd7b8e0b8f8fc5962914b56de26467e42b0c77fd2ef1974`, from
  corpus/builder `5eca03bc5`. Physical SHA-256 `5da6e800…49bf2`, logical
  SHA-256 `8cc37360…60359`, as in the
  [consolidated review](db-reader-lic-consolidated-review-2026-10-22.md).
- Public release
  `c305db4a3cfb408b76c13431e26beda72d5ce665046c95783fab36507354c969`
  (corpus `d2ab64d61`), from the preserved R7 review package. Manifest
  SHA-256 `e3c8b096…dec9`.
- Compatibility: 24 canonical input hashes across 4 instruments, and 870
  provision texts. Each text was compared three ways: DB text, the release
  `text_sha256`, and the release `.txt` artifact. Result: **0 mismatches**.
- The ifpe-dcg-2021 reference count differs, 115 in the release vs 112 when
  joined to DB provisions. The 3 extra edges are anchored to the instrument
  title (`ifpe-dcg-2021:reference:107-109:48`, `:128-130:54`, `:150-152:56`,
  all targeting `lritf`). The adapter serves them.
- Builder-commit verifier: a binary built from an exported `5eca03bc5` tree,
  run with `--root` at this checkout. `db verify` exits 0 in 55.7 s: 4,960
  inputs, 208,744 normalized records, 10,002,069 FTS positions, physical and
  logical SHA-256 match, and 21/21 goldens pass. Its `stale_input_paths` are
  only `schemas/public-instrument-v1.schema.json` and
  `schemas/public-release-v1.schema.json`.
- Pre-existing limitation: the HEAD binary refuses this artifact, with
  `missing pinned input schemas/public-instrument-v1.schema.json` at
  `5eca03bc5`. Verification of an accepted release must use its builder
  commit's binary. Committing the two new schema files will add them to
  `stale_input_paths` in the same way.

## D1 — contract

- Binding (`schemas/public-query-binding-v1.schema.json`) records:
  - DB release, physical and logical SHA-256, manifest SHA-256 and version,
    corpus and builder commits;
  - public release ID, manifest SHA-256, corpus commit and base path;
  - per instrument: canonical input hashes, provision and reference counts,
    a provision digest over `id\ttext_sha256` lines, and annex source digests.
- Responses (`schemas/public-query-response-v1.schema.json`) are a closed
  envelope:
  - fields: schema version, operation, outcome, snapshot, result;
  - result kinds are tied to outcomes;
  - no field can carry an input path, source pointer, reviewer, decision
    reference, blob OID or private rationale. The leakage test checks this.
- The Rust types and handler run every response through the compiled schema
  before emitting it. A violation becomes `contract_violation`.
- Synthetic tests (11, `crates/lex-db/src/public_tests.rs`):
  - Bis, transitory and annex resolution;
  - duplicate printed locators (`3` and `3o.`) return `ambiguous` with both
    candidates;
  - debt, cited-not-in-corpus and plain not-established absences;
  - out-of-scope rows never disclosed;
  - CRLF/accent-exact text with its own annex source;
  - oversized → artifact link verified by hash;
  - `as_of` and old pins refused explicitly;
  - U+0301 / whitespace / punctuation → `empty_query`;
  - phrase never becomes OR;
  - excerpt windows;
  - cursor determinism, overlap, cross-query/page-size/release mismatch, and
    null/scalar/array/non-canonical cursors;
  - JSON `null` cursor means none;
  - title-anchored edges;
  - incoming edges from unpublished instruments withheld;
  - byte-budget page shortening;
  - deadline interrupt;
  - tampered release, DB digest, provision digest and unknown binding fields
    all fail closed. A restored binding opens before the unknown-field case is tested,
  so that case is isolated.

## D2 — adapter on the real artifacts

- `lex-mex db public-bind` produced
  `derived/public-query/c305db4a-afdc5acf.binding.json` (Git-ignored, SHA-256
  in the evidence JSON) in 1.7 s.
- `crates/lex-db/tests/public_release.rs`, run with `--ignored` against the
  real DB, binding and release:
  - all 870 provisions are field-identical to the release's
    `instrument.json` (15 provision fields, 7 source fields);
  - every provision resolves from its printed number; 0 are ambiguous in the
    real selection;
  - the edge set for every anchor equals the release's, title anchors
    included, on id, source, target, status, span, offsets, form, basis and
    availability;
  - searches stay inside the selection and pages never overlap: `artículo`
    835 hits, `ley` 453, `Comisión Nacional Bancaria` 183, `fracción` 173,
    `autoridad` 70;
  - out-of-scope `ccf`/`laero` IDs get `outside_published_scope` for both
    lookup and incoming references.

## D3 — performance and gates

The adapter was measured on the actual accepted DB, release build, on this
workstation. Startup, which includes hashing 482 MB, takes 1.2 s; RSS is
213 MB.

| Load | p50 | p95 | max |
|---|---|---|---|
| Warm, 1 process, 480 mixed requests | 0.24 ms | 18 ms | 25 ms |
| 4 processes, 640 requests | 0.34 ms | 51 ms | 78 ms |
| 8 processes, 1,280 requests | 1.0 ms | 179 ms | 225 ms |

- Responses are identical across 30 repetitions.
- The largest non-provision response is 11,342 bytes. The largest provision
  response (`itf-dcg-2018:annex:18`) is 212,968 bytes.
- Search plan: FTS5 match → rowid join → allowlist bloom filter → materialized
  (id, score) → temp B-tree sort.
- Bound: each page re-runs the full-DB match for the phrase. For example,
  `ley` matches 23,163 provisions corpus-wide before the scope filter, about
  18 ms. Text and excerpts are read only for the returned page.
- References use the `reference_sources` and `reference_targets` indexes.
- The deadline is 2 s, enforced by a SQLite progress-handler interrupt.
  Concurrency is one request per process; the host caps the pool size.
- Performance ceilings for the deployment environment are not set. The plan
  requires measuring the selected host first (D4).

Native gate after the final edits:

- `cargo fmt --check`
- workspace clippy with `-D warnings`
- `cargo test --workspace`: 441 passed, 0 failed, 1 ignored (the real-data test, run separately: 1 passed)
- `validate lritf`: valid; 145 articles, 11 transitories, 126 references
- `validate ifpe-dcg-2021`: valid; 59 articles, 4 transitories,
  115 references
- current-binary `db golden-queries` on the accepted DB: 21/21
- accepted DB physical SHA-256 unchanged after all runs
- `Cargo.lock` unchanged

Format, clippy, tests and validators were run by the Haiku `mech` runner. Some
runs happened directly in the orchestrating session: intermediate clippy fix
iterations, the first real-data test run, the benchmark and the query-plan
capture. That was a routing miss for mechanical work; the final gate rerun via
`mech` is the reported result.

## Not done

- Adapter-level concurrency limit: the CLI handles one request per process
  and does not cap concurrency itself. The plan asks for concurrency limits at
  the adapter. That limit belongs to the serving shim or process pool chosen
  in D4, and is disclosed here as open.

- Independent review: queued for Codex at Agent Vault
  `provider-handoffs/pending/codex/2026-10-25-lex-mex-db-agent-query-review.md`.
- Hosting, deployment, broader scope, and IFC/IFPE answer-quality evaluation
  (kept separate from retrieval).
- HTML agent pages: not part of this adapter.
- Commit, and its branch placement: operator decision.

## D3 repairs (R1, R2) — 2026-10-25

The Codex repair request is
`provider-handoffs/pending/claude/2026-10-25-lex-mex-db-agent-query-repairs.md`.
Before editing, all nine reviewed source/binding digests matched it: no drift.

**R1, binding completeness.** `public-bind` now:

- validates the release manifest against `public-release-v1` and each
  instrument document against `public-instrument-v1`;
- keeps the byte/hash checks: release ID recomputation, the file list,
  exact-text and Markdown artifacts;
- requires exact equality between the release and the DB-derived projection
  for:
  - instrument metadata: title, short name, publication and latest-reform
    dates, coverage;
  - main source metadata on the document and the manifest item;
  - all 15 provision fields and both artifact paths per provision;
  - the source of each provision. An annex's expected source comes from the
    DB's own copy of the adapter's `annex_pdf_urls`, matched after
    percent-decoding (the producer compares parsed URLs); every match must be
    unique;
  - every edge, compared as a complete object: id, source, span, offsets,
    target instrument and provision, status, form, basis, and
    `target_published` over the whole selection. Title-anchored edges are
    included.
- `get_provision`, `references` and `bind` now use the same projection
  helpers. The binding format is unchanged.

R1 regression tests:

- `binding_rejects_rehashed_releases_with_changed_public_projection`: 15
  self-consistent rehashed mutations covering span, target, status, published
  flag, title-anchor offset, dropped edge, label, temporal status, heading,
  text+hash, official URL, annex publisher, annex given main source,
  instrument title, and an unknown field. Each must fail for its intended
  reason, and the unchanged release must bind.
- Real-artifact probes, using temporary repacked copies of `c305db4a…` against
  the accepted DB:
  - Codex's span mutation: `lritf: release and database reference edges differ`;
  - a publisher change: `release instrument metadata differs`;
  - a temporal-status change: `differ in temporal_status`;
  - none of these writes a binding;
  - a control repack with no change reproduces the same release ID and binds
    identically.
- Re-binding the real accepted artifacts with the stricter checks gives a
  binding byte-identical to the original (`d2babd17…`), written to a new
  derived path.

**R2, bounded CLI framing.** `db public-query` reads stdin through a framer
that buffers at most 8,192 + 1 bytes:

- frames end at LF; one CR before the LF is part of the delimiter, and
  delimiters do not count toward the limit;
- an over-limit frame gets a `bad_request` response as soon as the limit is
  exceeded, and its remainder is discarded unstored through the next LF;
- later frames are served normally;
- invalid UTF-8 gets `bad_request`;
- a final unterminated frame is accepted at EOF;
- the handler's own 8,192-byte check remains for direct callers.

R2 tests:

- `db::tests::exact_limit_crlf_multibyte_and_eof_are_defined` covers:
  - the exact boundary and limit + 1;
  - CRLF, and a CR followed by a non-LF byte;
  - multibyte UTF-8 at and over the limit;
  - invalid UTF-8;
  - EOF tail and empty frames;
  - the frame following a rejected one.
- `oversized_unterminated_input_is_rejected_before_more_is_read`: 16 MiB is
  delivered lazily; rejection is observed on the first chunk read, and the
  next request is served.
- Process probe with the release binary and the accepted DB: 16 MiB written
  with no newline. The `bad_request` response arrived 0.011 s later, before
  any newline. The following request returned `ok`, and max RSS was 211 MB,
  the same as the startup baseline.
- A host shim still needs its own body limit, ingress timeout and process
  cap. The SQLite progress handler interrupts SQL only.

**Evidence after repairs, by category.** The `mech` runner ran the native
gate. Parent-session probes are labelled as such.

1. Native gate (`mech`), all exit 0:
   - `cargo fmt --check`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo test --locked -p lex-db --lib public::tests`: 12/12
   - `cargo test --locked -p lex-cli db::tests`: 2/2
   - `cargo test --workspace`: 444 passed, 0 failed, 1 ignored
   - `cargo run --locked -p lex-cli -- validate lritf`: valid, 145/11/126, 0
     issues
   - `cargo run --locked -p lex-cli -- validate ifpe-dcg-2021`: valid,
     59/4/115, 0 issues
2. Artifact parity and re-bind (`mech`):
   - `LEX_PUBLIC_DATABASE=/Users/jr/Dev/lex-mex/derived/db/permanent-5eca-a/afdc5acf0b5017d73cd7b8e0b8f8fc5962914b56de26467e42b0c77fd2ef1974/corpus.sqlite LEX_PUBLIC_BINDING=derived/public-query/c305db4a-afdc5acf.binding.json LEX_PUBLIC_RELEASE=/Users/jr/Dev/lex-mex/.work/reader-r7-review-20261024/publication/releases/c305db4a3cfb408b76c13431e26beda72d5ce665046c95783fab36507354c969 cargo test --locked --release -p lex-db --test public_release -- --ignored`:
     1/1
   - `target/release/lex-mex db public-bind --database-dir /Users/jr/Dev/lex-mex/derived/db/permanent-5eca-a/afdc5acf0b5017d73cd7b8e0b8f8fc5962914b56de26467e42b0c77fd2ef1974 --release-dir /Users/jr/Dev/lex-mex/.work/reader-r7-review-20261024/publication/releases/c305db4a3cfb408b76c13431e26beda72d5ce665046c95783fab36507354c969 --output <scratch>/rebind-final.json`
     then `cmp` against the original binding: identical.
3. Accepted-builder verifier and goldens (`mech`):
   - the binary built from an exported `5eca03bc5` tree:
     `lex-mex --root /Users/jr/Dev/lex-mex-reader db verify --database /Users/jr/Dev/lex-mex/derived/db/permanent-5eca-a/afdc5acf0b5017d73cd7b8e0b8f8fc5962914b56de26467e42b0c77fd2ef1974/corpus.sqlite --manifest /Users/jr/Dev/lex-mex/derived/db/permanent-5eca-a/afdc5acf0b5017d73cd7b8e0b8f8fc5962914b56de26467e42b0c77fd2ef1974/manifest.json --corpus-commit 5eca03bc5085de423ae23b1f1519712604bd28e2`:
     exit 0 at head `af6de68a8`, with 4,960 inputs, 208,744 records,
     10,002,069 FTS positions, and physical and logical SHA-256 matching;
   - known qualification: its `stale_input_paths` are only
     `schemas/public-instrument-v1.schema.json` and
     `schemas/public-release-v1.schema.json`;
   - `db golden-queries` with that binary: 21/21; with the current binary:
     21/21;
   - `shasum -a 256 /Users/jr/Dev/lex-mex/derived/db/permanent-5eca-a/afdc5acf0b5017d73cd7b8e0b8f8fc5962914b56de26467e42b0c77fd2ef1974/corpus.sqlite`: `5da6e800…49bf2`, unchanged;
     `Cargo.lock` unchanged.
4. Mutation probes (parent session; scratch scripts not preserved in the
   repository; SHA-256 in the evidence JSON):
   - `python3 -I r1probe.py /Users/jr/Dev/lex-mex/.work/reader-r7-review-20261024/publication/releases/c305db4a3cfb408b76c13431e26beda72d5ce665046c95783fab36507354c969 <scratch>/<mode> <mode>` for modes `span`,
     `source`, `status` and the control `none`, each followed by
     `db public-bind`;
   - `python3 -I r2probe.py <release lex-mex> /Users/jr/Dev/lex-mex/derived/db/permanent-5eca-a/afdc5acf0b5017d73cd7b8e0b8f8fc5962914b56de26467e42b0c77fd2ef1974/corpus.sqlite <binding>`.
5. Performance: not repeated (see below).

The repairs do not change query SQL or the served projection; the real-data
parity rerun confirms the projection is unchanged. The earlier latency and
concurrency measurements therefore still apply and were not repeated.

## Re-review round 1 repairs — 2026-10-25

The operator relayed Codex's verdict: R2 fixed; R1 still blocking.

- **Duplicate references.** The release's references now become a map keyed
  by edge ID; a repeated ID is rejected (`release repeats reference …`), and
  the whole map must equal the DB map.
  - Synthetic regression: the mutation "duplicate edge in place of another"
    copies edge 0 over edge 1.
  - Real-artifact probe (`r1probe.py` mode `dup`, against `c305db4a…`):
    `lritf: release repeats reference urn:lex-mx:federal:statute:lritf:article:102:reference:392-394:98:direct`,
    exit 1, no binding written. The span probe still fails, and the
    unchanged control still binds.
- **Annex URL semantics.** `same_url` parses both sides with `url::Url`, as
  the producer's `Url`-typed config and manifest fields do.
  - Test `annex_urls_compare_as_parsed_urls`: a raw space equals `%20`,
    `a%2Fb.pdf` differs from `a/b.pdf`, and an unparsable URL matches nothing.
  - `crates/lex-db/Cargo.toml` adds `url.workspace = true`. `Cargo.lock` gains
    one dependency line (`"url"` under `lex-db`) and no new package.
- Re-binding the accepted artifacts is still byte-identical to `d2babd17…`;
  the ifpe and itf annex mappings are unchanged.
- **D4 status.** The host handoff now reports owner production live on
  Workers Free with observation pending, pinned to owner commit `41243b07`.
  The adapter stays local.

Gate after these repairs (`mech`): fmt and clippy clean. `public::tests` 13/13. `db::tests` 2/2. Workspace:
445 passed, 0 failed, 1 ignored. Both validators valid. Real-artifact
`public_release` 1/1. Builder-commit verifier exit 0 at head `af6de68a8`,
with the same two known stale schema paths; goldens 21/21 with that binary
and 21/21 with the current one. Accepted DB SHA-256 `5da6e800…49bf2`
unchanged.

Probes run directly by the parent session: the duplicate, span and control
repacks, and the accepted re-bind (byte-identical). Performance was not
repeated, because query SQL and the served projection are unchanged.
