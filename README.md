# Lex-Mex

Lex-Mex is a provenance-aware compiler for Mexican federal legal texts. The
initial vertical slice ingests the current consolidated **Ley para Regular las
Instituciones de Tecnología Financiera (LRITF)** published by the Cámara de
Diputados, hashes and extracts the source, parses article-level canonical
records, validates them, and exports JSON and lawyer-readable Markdown.

## Prerequisites

- stable Rust
- `pdftotext` from Poppler
- network access to `https://www.diputados.gob.mx`

## Run the vertical slice

```bash
cargo run -p lex-cli -- \
  --obsidian-vault /Users/jr/Vaults/Lex-Mex \
  pipeline lritf
```

The pipeline writes version-controlled canonical records and standard Markdown
under `corpus/mx/lritf/`. When `--obsidian-vault` is supplied, it publishes
Obsidian notes beneath `Corpus/LRITF/` in that vault. The downloaded PDF is
temporary and is deleted after successful validation. Its URL, response
metadata, byte hash, extracted-text hash, and tool versions remain in
`source-manifest.json`.

Individual stages are also available:

```bash
cargo run -p lex-cli -- discover diputados
cargo run -p lex-cli -- fetch lritf
cargo run -p lex-cli -- extract lritf
cargo run -p lex-cli -- parse lritf
cargo run -p lex-cli -- validate lritf
cargo run -p lex-cli -- export lritf --format markdown
cargo run -p lex-cli -- \
  --obsidian-vault /Users/jr/Vaults/Lex-Mex \
  export lritf --format obsidian
cargo run -p lex-cli -- analyze-temporal lritf
```

Set `LEX_MEX_OBSIDIAN_VAULT` to avoid repeating the vault option:

```bash
export LEX_MEX_OBSIDIAN_VAULT=/Users/jr/Vaults/Lex-Mex
cargo run -p lex-cli -- export lritf --format obsidian
```

The exporter owns only `Corpus/<instrument>/`. Keep human-authored analysis in
the vault's `Notas/` and `Revisiones/` folders.

`analyze-temporal` creates a schema-bound request artifact. It does not claim a
legal conclusion without a model response; importing model output is kept
separate from deterministic publication.

## Development checks

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
