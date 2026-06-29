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
- Codex CLI authentication when running temporal analysis with `--provider codex`

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
cargo run -p lex-cli -- analyze-temporal lritf --provider codex --model gpt-5.5
cargo run -p lex-cli -- import-temporal lritf response.json --model MODEL_ID
cargo run -p lex-cli -- review list
cargo run -p lex-cli -- review list --all
cargo run -p lex-cli -- review resolve REVIEW_ID \
  --resolution accept-machine-conclusion --reviewer "Reviewer name"
```

Set `LEX_MEX_OBSIDIAN_VAULT` to avoid repeating the vault option:

```bash
export LEX_MEX_OBSIDIAN_VAULT=/Users/jr/Vaults/Lex-Mex
cargo run -p lex-cli -- export lritf --format obsidian
```

The exporter owns only `Corpus/<instrument>/`. Keep human-authored analysis in
the vault's `Notas/` and `Revisiones/` folders.

Without a provider, `analyze-temporal` only creates the schema-bound request
artifact. With `--provider codex`, it runs the model through the authenticated
Codex CLI, requires strict JSON-schema output, validates that every requested
provision appears exactly once and every supporting quotation occurs verbatim
in the source evidence, then records request and response hashes. Materially
unknown transitory effects and confidence below 0.92 are routed to
`corpus/mx/lritf/review-queue.json` and the Obsidian review dashboard.

The complete network-and-model cycle is:

```bash
LEX_MEX_OBSIDIAN_VAULT=/Users/jr/Vaults/Lex-Mex \
  cargo run -p lex-cli -- pipeline lritf \
  --temporal-provider codex --temporal-model gpt-5.5
```

`import-temporal` provides the same deterministic validation and routing for a
model response produced by another provider. `review list` prints the pending
human decisions; model output never resolves those decisions automatically.
`review resolve` requires reviewer identity. A `lawyer-override` additionally
requires `--note` and at least one explicit change: `--temporal-status`, an
applicable effective date, or `--effects-file` containing a JSON array of
corrected structured effects. Resolved items remain in the JSON audit history
but are removed from the pending Obsidian dashboard. No review is resolved
merely by exporting or rerunning `review list`.

Temporal model v2 separates the status of a transitory provision from the
effects it creates. Structured effects cover commencement, deadlines,
adaptation periods, transitional permissions, survival of prior rules for
existing matters, migration, authority assignments, coordination, staged
commencement, sunsets, and repeal. Cohort-based survival may be open-ended by
design without requiring legal review. `external_verification_required` marks
clear rules whose present application depends on checking a later publication
or authority action.

## Development checks

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
