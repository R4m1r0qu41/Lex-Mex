# Agent instructions

- Keep Rust responsible for canonical normalization, validation, state changes,
  and publication.
- Never silently alter source text. Any normalization must be deterministic and
  covered by a fixture.
- Preserve the official URL and source/extracted-text SHA-256 hashes.
- Distinguish deterministic facts from model inference in types and exports.
- A temporal model response must validate against
  `schemas/temporal-model-output-v2.schema.json` before it can enter the corpus.
- Add a regression fixture for every material parser defect.
- Do not add a new crate or directory without code or data that uses it now.
- Run formatting, Clippy, tests, and corpus validation before committing.
