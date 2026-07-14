# Lex-Mex Project Status

- **Status date:** 2026-07-14
- **Repository:** <https://github.com/R4m1r0qu41/Lex-Mex>
- **Committed instruments:** 132
- **Active ingestion batch:** `constitutional_CN1_congress`
- **Next checkpoint:** `rgic`
- **Current legal reviewer:** JRH

## 1. Current state

Lex-Mex is a provenance-aware compiler and update engine for Mexican federal
legal materials. Rust owns acquisition metadata, canonical normalization,
validation, reference extraction, review state transitions, and publication.
Model output is a proposal that must pass a schema-gated deterministic boundary
and cannot overwrite an audited human decision.

The repository is now the only ingestion gate for the normalization program.
The committed corpus contains official consolidated texts from Cámara de
Diputados and CNBV sources; the earlier external-vault Python imports are not
canonical. Obsidian remains a presentation target, with generated content
confined to `Corpus/<instrument>/`.

Current committed-corpus totals:

| Artifact | Count |
|---|---:|
| Instruments | 132 |
| Articles | 31,945 |
| Original transitory provisions | 1,165 |
| Annexes | 28 |
| Reference edges | 16,645 |
| Unresolved reference edges | 0 |
| Canonical defined terms | 1,511 |
| Canonical term usages | 31,164 |
| Generated Markdown files | 33,270 |

All 132 committed `validation.json` reports are valid. They contain 187
non-blocking review warnings: 162 suffixed-article ordering notices, 16
unfrozen count baselines in previously admitted instruments, 7 article-gap
notices, and 2 suffix-order notices. These warnings remain explicit; validity
does not imply that temporal analysis or legal review has been performed.

## 2. Implemented system

The Rust workspace has five crates:

- `lex-source`: official-source discovery, acquisition, metadata, and hashing.
- `lex-parse`: text extraction, adapter-driven parsing, canonical provisions,
  references, defined terms, reform evidence, and structural validation.
- `lex-core`: canonical types plus temporal and review state transitions.
- `lex-export`: canonical JSON, Markdown, and bounded Obsidian publication.
- `lex-cli`: instrument commands, batch orchestration, and end-to-end pipeline
  execution.

Each admitted instrument records its official URL, publisher, retrieval time,
source and extracted-text SHA-256 hashes, parser version, schema version,
canonical provisions, reference graph, terms, validation report, and Markdown
export. Parser count baselines are reviewed and frozen per adapter rather than
accepted silently.

The parser supports Cámara and CNBV consolidated-document conventions,
including page boundaries, split headings, suffixed articles, original versus
reform transitories, amendment margin marks, independently sourced CNBV
annexes, internal and cross-instrument references, relative article references,
and exact-span defined terms. Material parser defects require regression
fixtures.

## 3. Temporal and legal-review scope

Structural ingestion and temporal legal analysis are separate programs.
Newly normalized instruments are committed with `review_status: not_analyzed`
unless a distinct temporal run is authorized.

The audited temporal vertical slice currently consists of:

- `lritf`: 145 articles, 11 original transitories, 126 resolved references,
  19 temporal determinations, and 32 structured legal effects.
- `ifpe-dcg-2021`: 59 articles, 4 original transitories, 8 independently
  sourced annexes, 113 resolved references, and 4 temporal determinations.
- `itf-dcg-2018`: 105 articles, 7 original transitories, 20 independently
  sourced annexes, 144 resolved references, and structured provenance for six
  amending resolutions.

JRH remains the designated reviewer for committed LRITF decisions. Two
determinations are lawyer-verified. One review is pending: ITF DCG transitory
SÉPTIMO is marked "Derogado." and awaits confirmation of the formal derogation
boundary. No agent may infer or impersonate that approval.

## 4. Normalization program

Operational batch manifests live in `batches/` and validate against
`schemas/batch-manifest.schema.json`. Prepared source manifests under
`prompts/cluster-2-batches/` are planning inputs only; they must be normalized
and reviewed before execution.

The active CN1 congressional batch is proceeding as one reviewable instrument
per checkpoint:

| Checkpoint | Instrument | Structural result | Commit |
|---:|---|---|---|
| 1 | `locg` | 151 articles / 8 transitories / 31 references / 0 issues | `97fa5cbc` |
| 2 | `reg-diputados` | 323 / 13 / 40 / 0 | `553baa6e` |
| 3 | `reg-senado` | 313 / 4 / 47 / 0 | `488057a5` |
| 4 | `rgic` | Next | — |
| 5 | `ldofgg` | Pending | — |

Checkpoint 4 starts with a provisional source and parser pass, followed by
manual inspection, frozen counts, relinking, validation, Markdown inspection,
the required workspace regression checks, and a dedicated commit. Checkpoint 5
then follows the same sequence. CN1 closes only after reverse relinking and
cross-instrument edge review; a successful forward batch run alone is not
sufficient.

The full execution and recovery procedure is in [`../PLAN.md`](../PLAN.md).

## 5. Verification state

Checkpoint 3 was verified on 2026-07-14 with:

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo run --locked -p lex-cli -- validate lritf`
- `cargo run --locked -p lex-cli -- validate ifpe-dcg-2021`
- `cargo run --locked -p lex-cli -- validate reg-senado`
- the affected CN1 batch end-to-end run and canonical/Markdown inspection

Results were clean for the required regressions. `reg-senado` validated at 313
articles, 4 original transitories, 47 resolved references, and zero issues.
Its 39 reform-transitory evidence records have unique identifiers, and its
Markdown export contains exactly one index, 313 article notes, and 4 original
transitory notes.

## 6. Known gaps and next actions

The immediate action is checkpoint 4, `rgic`. After `ldofgg`, CN1 requires a
reverse-link pass because earlier instruments cannot discover later sibling
targets during their initial parse.

Known engineering and corpus gaps remain:

- batch execution does not yet perform reverse relinking automatically;
- the shared instrument-alias registry is not yet wired into per-adapter
  `external_instruments`, so cross-instrument recall requires explicit review;
- 16 previously admitted instruments still carry unfrozen count warnings;
- no automated Cámara/CNBV/DOF source-change monitor, candidate-version flow,
  or provision-level update diff exists;
- `source-manifest.resulting_git_commit` is not populated automatically;
- full CLI network flows and live model execution remain integration-tested
  manually rather than in hermetic CI;
- temporal analysis and legal review remain deferred for the normalization
  corpus outside the audited three-instrument vertical slice.

The next operational sequence is therefore:

1. ingest, inspect, freeze, validate, and commit `rgic`;
2. repeat for `ldofgg`;
3. reverse-relink and audit CN1 cross-instrument references;
4. select and normalize the next prepared cluster-2 batch without silently
   expanding legal or temporal scope.
