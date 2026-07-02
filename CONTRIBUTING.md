# Contributing to Lex-Mex

Lex-Mex welcomes focused bug reports, source adapters, parser fixtures,
validation improvements, and documentation corrections. Legal-data changes
require the same review discipline as code changes.

## Before opening a change

1. Open an issue for a new instrument, schema change, or architectural change.
2. Keep each change narrow enough that its source, canonical-data, and review
   effects can be audited together.
3. Do not include credentials, local vault contents, downloaded work files, or
   personal data.

## Local checks

Install stable Rust and Poppler, then run:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run --locked -p lex-cli -- validate lritf
```

CI runs the same checks. `Cargo.lock` is committed and must remain current.

## Source and corpus changes

- Use an official source and record its publisher, URL, retrieval metadata, and
  hashes.
- Never hand-edit official source text to make a parser pass. Fix the parser
  deterministically and add the smallest useful regression fixture.
- Explain canonical JSON changes in the pull request and inspect the resulting
  Markdown.
- Keep legal inference, factual external verification, and deterministic
  extraction visibly separate.
- Do not resolve a review without an authorized reviewer's actual identity and
  decision. A contributor or coding agent must never manufacture legal review.

## Commit and pull-request expectations

- Use imperative, descriptive commit subjects.
- State the official sources consulted and the checks run.
- Call out schema migrations, canonical-data changes, and reviewer decisions.
- Avoid unrelated formatting or generated-data churn.

By contributing original work, you agree that it may be distributed under the
repository's `MIT OR Apache-2.0` license. Official legal source text is handled
as described in [`NOTICE.md`](NOTICE.md).
