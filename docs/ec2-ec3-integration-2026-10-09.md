# EC2 and EC3 integration — 2026-10-09

The reviewed fourteen-instrument packet has been applied to main's working tree
at `e3dfcbd89`: 940 new articles and 68 original transitories. At the pre-commit integration checkpoint, the working corpus had 257 instruments
and the committed baseline had 243. The commit record below supersedes that
checkpoint; all 257 instruments are now committed locally.

## Applied changes

The packet's 1,149 adapter, corpus, Markdown and operational-manifest files were
copied only after verifying both checkout HEADs, every source SHA-256, each
destination's absence and the path allowlist. The eleven pre-existing dirty
documentation/instruction files survived the copy byte-for-byte.

Fourteen exact official-title aliases were added to
`adapters/diputados/_instrument-aliases.json`. The six runnable EC3 entries advance
the manifest inventory regression expectation from 41 manifests/215 unique slugs
to 42/221. LFJS remains blocked and absent from main's corpus and adapters.

## Bounded reverse linking

Relinking and validation/export ran for the fourteen new instruments and seven
existing instruments whose provisions mention the new official titles:
`lie`, `lplan`, `lfpc`, `reg-lfpc`, `lic-calidad`, `lan`, `reg-lopsrm`.
All 63 commands succeeded.

Two newly resolved references were verified against their exact canonical spans
and existing targets:

- REG-LDCMPME Article 3 → LDCMPME Article 3.
- LIISPCEN Article 4 → LDCMPME Article 3, fraction III qualifier retained.

The new-instrument reference total is 194, up from the packet's 192. Only six
packet files differ after this derived graph update: the references, validation
report and affected article Markdown for those two source instruments. New
provisions, manifests, temporal state and review data match the reviewed packet.

The seven existing instruments produced no changes in provisions that cite a
new alias. Their relink regenerated unrelated prior graph results and removed
two trailing JSON newlines. Those changes were compared and restored to the
exact pre-link state; no tracked existing corpus path is changed. The unrelated
regeneration differences include references to promulgation Article 89 and older
internal/reference rules, so they require a separate source-reviewed task.

Title-only mentions without a numbered provision are outside the current
reference extractor's edge model. This integration does not claim new graph
edges for every named instrument or corpus-wide graph closure.

## Verification

The following passed on the final integrated main state, using
`CARGO_TARGET_DIR=/private/tmp/lex-mex-target-20261008`:

```text
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run --locked -p lex-cli -- validate lritf
cargo run --locked -p lex-cli -- validate ifpe-dcg-2021
```

Twenty-one bounded validations and Markdown exports passed during the reverse
link check. The fourteen additions retain 36 reviewed ordinal/suffix warnings.
All 257 stored validation reports are valid; this inventory is not a fresh run
of all 257 validators. Existing canonical text, provenance and legal-review
records remain unchanged. The reader checkout is clean; Figma and SCJN work were
not modified.

Exact application and graph evidence is in
[`ec2-ec3-integration-result-2026-10-09.json`](ec2-ec3-integration-result-2026-10-09.json).
The original packet and source-review reports remain historical evidence of the
reviewed input.

## Remaining work

LFJS is retained under the intake worktree's `.work/held/lfjs/corpus/`. Its five
numeric original transitories need a fixture-backed parser repair before
admission. No legal review or temporal model analysis was performed here.
Address the LFJS numeric-transitory defect in a separate bounded parser task.
Publication remains governed by the separate reader release plan.

## Local commit record

The operator authorized local commits on 2026-10-09. The integration was split
into one support commit, fourteen instrument commits and a Codex-instruction
commit. A final documentation commit records this history. No push/deployment.

- Support and EC3 inventory: `1bac29446`.
- Codex companion instructions: `023ed7451`.
- Canonical corpus checkpoint: `ebe688df2`, 257 committed instruments.

| Instrument | Commit |
|---|---|
| reg-lcec | `7fdcf1384` |
| ldcmpme | `67e3d6e84` |
| reg-ldcmpme | `fb9f5f302` |
| lffmaa | `04fdf4be0` |
| liispcen | `6279519e1` |
| lnetb | `3c52143d7` |
| lfcc | `afe49e4f6` |
| reg-lfmn | `f30fb5b25` |
| lfcp | `2eccf90c0` |
| reg-lfcp | `8ea276ce2` |
| reg-ccomer-mpsc | `2929154f8` |
| reg-lfpc-art122 | `de02ab79a` |
| reg-lfjs | `b121a4639` |
| reg-lfppi | `ebe688df2` |

The staged bytes were verified against the reviewed allowlist; staged diff checks
passed. An extra blank line at CODEX.md EOF was removed during staging without
changing instructions. No code or canonical changes occurred after the passing
required gate, so that gate was not repeated just for commits. The original
integration packet/result JSONs preserve the pre-commit application evidence.

## Supersession note — 2026-10-13

The LFJS hold above was resolved after this report. A fixture-backed parser
repair and an adapter stop marker admitted `lfjs` (17 articles, 5 original
transitories) at `bcf67151a`, with the parser change at `eeec7e94e`. The
corpus now holds 262 instruments, all committed. The statements above are
preserved as the record of the 2026-10-09 integration. See
`docs/ingestion-difficulty-log.md`.
