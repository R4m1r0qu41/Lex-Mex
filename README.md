# Lex-Mex

[![CI](https://github.com/R4m1r0qu41/Lex-Mex/actions/workflows/ci.yml/badge.svg)](https://github.com/R4m1r0qu41/Lex-Mex/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

Lex-Mex is an experimental, provenance-aware compiler and temporal-analysis
pipeline for Mexican federal legal texts. It acquires official sources,
preserves their provenance, produces stable canonical records, validates model
output, routes material legal uncertainty to a named reviewer, and publishes
lawyer-readable Markdown and Obsidian views.

The committed corpus currently contains **133 federal instruments** acquired
through the Rust ingestion gate: 32,159 articles, 1,167 original transitory
provisions, 28 annexes, and 16,675 resolved reference edges. The normalization
program is expanding the original LRITF/Fintech vertical slice across official
Cámara de Diputados and CNBV consolidated sources while preserving source
hashes, parser baselines, canonical JSON, validation reports, and generated
Markdown for every instrument.

Temporal analysis remains intentionally narrower than structural ingestion.
The audited temporal corpus currently covers LRITF, `ifpe-dcg-2021`, and
`itf-dcg-2018`; one legal review remains pending for the ITF DCG's derogated
transitory SÉPTIMO. Newly normalized instruments stay `not_analyzed` until a
separate schema-gated temporal pass is authorized.

The active congressional batch (`constitutional_CN1_congress`) is complete
through `locg`, `reg-diputados`, `reg-senado`, and `rgic`. Its final ingestion
checkpoint is `ldofgg`, followed by a CN1 reverse-link validation pass.

> [!IMPORTANT]
> Lex-Mex is not an official publication, is not affiliated with the Mexican
> government, and does not provide legal advice. Always verify legal text and
> conclusions against the cited official sources.

## How it works

```text
official source -> fetch + hash -> extract -> parse -> canonical JSON
                                                     |-> validation report
                                                     |-> reference graph
                                                     |-> temporal analysis
                                                     |-> human review queue
                                                     `-> Markdown / Obsidian
```

Rust owns source integrity, canonical normalization, schema validation, review
state transitions, and publication. A model may propose temporal
classifications, but its output cannot enter the corpus until deterministic
checks pass. Model output never resolves a legal review automatically.

The workspace is divided into five crates:

| Crate | Responsibility |
|---|---|
| `lex-source` | Source discovery, acquisition, metadata, and hashing |
| `lex-parse` | Text extraction orchestration and canonical parsing |
| `lex-core` | Canonical types, temporal effects, and review state |
| `lex-export` | JSON, Markdown, and Obsidian publication |
| `lex-cli` | Commands and end-to-end pipeline orchestration |

## Prerequisites

- Stable Rust with `rustfmt` and Clippy (the checked-in toolchain file installs
  these components)
- `pdftotext` from [Poppler](https://poppler.freedesktop.org/)
- Network access to official source sites when fetching a source
- Authenticated Codex CLI only when using `--temporal-provider codex`
- Obsidian and its CLI only if you want the optional vault workflow

On macOS, Poppler can be installed with Homebrew:

```bash
brew install poppler
```

## Quick start

Clone the repository and run the deterministic pipeline:

```bash
git clone https://github.com/R4m1r0qu41/Lex-Mex.git
cd Lex-Mex
cargo run --locked -p lex-cli -- pipeline lritf
```

This downloads the official PDF, records its response metadata and SHA-256,
extracts and hashes the text, parses the canonical provisions, validates the
corpus and its internal reference graph, and exports linked Markdown under
`corpus/mx/lritf/markdown/`. The downloaded PDF is temporary and is removed
after successful validation unless `--keep-work` is supplied.

To publish generated notes to an external Obsidian vault:

```bash
mkdir -p "$HOME/Vaults/Lex-Mex"
export LEX_MEX_OBSIDIAN_VAULT="$HOME/Vaults/Lex-Mex"
cargo run --locked -p lex-cli -- pipeline lritf
```

Open that directory as a vault in Obsidian. The exporter owns only
`Corpus/<instrument>/`; keep human-authored work in `Notas/`, `Revisiones/`, or
another directory outside that generated boundary.

## Temporal analysis and review

Without a provider, temporal analysis creates a schema-bound request artifact:

```bash
cargo run --locked -p lex-cli -- analyze-temporal lritf
```

With Codex, it executes the model, validates strict JSON-schema output,
requires exactly one determination for every requested item, verifies every
supporting quotation against the source evidence, and records request and
response hashes:

```bash
cargo run --locked -p lex-cli -- analyze-temporal lritf \
  --provider codex --model gpt-5.5
```

The complete network-and-model cycle is:

```bash
LEX_MEX_OBSIDIAN_VAULT="$HOME/Vaults/Lex-Mex" \
  cargo run --locked -p lex-cli -- pipeline lritf \
  --temporal-provider codex --temporal-model gpt-5.5
```

Provider-neutral responses can enter through the same deterministic boundary:

```bash
cargo run --locked -p lex-cli -- \
  import-temporal lritf response.json --model MODEL_ID
```

Materially unknown effects and confidence below the configured threshold are
routed to the instrument's `review-queue.json` and the generated Obsidian
review dashboard. The designated reviewer can also open a review on a
machine-accepted determination to correct or enrich it. Review resolution
requires a reviewer identity:

```bash
cargo run --locked -p lex-cli -- review list
cargo run --locked -p lex-cli -- review list --all
cargo run --locked -p lex-cli -- review --instrument ifpe-dcg-2021 \
  open PROVISION_ID --reason "Why the accepted conclusion needs review"
cargo run --locked -p lex-cli -- review resolve REVIEW_ID \
  --resolution accept-machine-conclusion --reviewer "Reviewer name"
```

A `lawyer-override` additionally requires a note and at least one explicit
change. Resolved decisions remain in the audit history and survive later model
reruns.

Temporal model v2 separates a transitory provision's own status from the legal
effects it creates. Effects include commencement, deadlines, adaptation,
transitional permissions, survival of prior rules, migration, authority
assignments, coordination, staged commencement, sunsets, and repeal.
Open-ended cohort exhaustion is represented explicitly rather than treated as
uncertainty. A clear rule that depends on checking a later official act is
marked `external_verification_required`, separately from legal ambiguity.

## Individual pipeline stages

Each command takes the slug of a configured instrument, for example `lritf`,
`reg-senado`, or `itf-dcg-2018`:

```bash
cargo run --locked -p lex-cli -- discover diputados
cargo run --locked -p lex-cli -- discover cnbv
cargo run --locked -p lex-cli -- fetch lritf
cargo run --locked -p lex-cli -- extract lritf
cargo run --locked -p lex-cli -- parse lritf
cargo run --locked -p lex-cli -- link lritf
cargo run --locked -p lex-cli -- validate lritf
cargo run --locked -p lex-cli -- export lritf --format json
cargo run --locked -p lex-cli -- export lritf --format markdown
cargo run --locked -p lex-cli -- export lritf --format obsidian
cargo run --locked -p lex-cli -- pipeline ifpe-dcg-2021
```

For `ifpe-dcg-2021`, fetch and extract also acquire the eight annex PDFs CNBV
publishes alongside the main document (recorded in
`annex-source-manifests.json`), plus the formal DOF publication for
promulgation-date provenance (`formal-source-manifest.json`).

## Repository layout

```text
adapters/   source-specific acquisition and parsing configuration
corpus/     committed canonical records, analysis, validation, and Markdown
crates/     Rust implementation
docs/       architecture, legal model, decisions, and current project status
fixtures/   parser regression inputs
prompts/    versioned temporal-analysis prompts
schemas/    versioned JSON Schemas for trusted boundaries
```

See [`docs/project-status.md`](docs/project-status.md) for the current corpus
inventory, verification state, known gaps, and active checkpoint. The detailed
normalization execution and recovery sequence is maintained in
[`PLAN.md`](PLAN.md).

## Development

Run the same checks as CI before committing:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run --locked -p lex-cli -- validate lritf
cargo run --locked -p lex-cli -- validate ifpe-dcg-2021
```

Read [`CONTRIBUTING.md`](CONTRIBUTING.md) before changing canonical data,
parsing, schemas, or review state. Repository-specific instructions for coding
agents are in [`AGENTS.md`](AGENTS.md).

## License

Original Lex-Mex software and documentation are available under either the
[MIT License](LICENSE-MIT) or the [Apache License 2.0](LICENSE-APACHE), at your
option. This dual license combines a simple permissive grant with Apache 2.0's
explicit patent terms.

Official Mexican legal texts retain their public-law status and are not
relicensed by this repository. Provenance, attribution, and legal-source terms
are explained in [`NOTICE.md`](NOTICE.md).
