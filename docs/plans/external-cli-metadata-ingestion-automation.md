# External CLI, metadata portability, and ingestion automation

This is the living execution plan for making Lex-Mex consumable from other
projects and for advancing prepared ingestion batches without continuous
operator prompting. Repository code, committed corpus data, `AGENTS.md`, and
the active cluster-2 ingestion plan outrank this document.

## Purpose and observable outcome

An external project can install the `lex-mex` binary, point it at this
repository, enumerate and search either the whole committed corpus or selected
instruments, and obtain stable paths for use with other command-line tools.
The repository also has a reviewed metadata-evolution direction and a bounded
design for continuing prepared ingestions until a declared milestone, budget,
rate limit, or safety gate is reached.

The complete outcome is:

- a consumer CLI that does not require the caller's working directory to be
  the Lex-Mex checkout;
- a deterministic subset bundle boundary for applications that should not
  receive the entire repository;
- an additive, schema-first metadata evolution that preserves canonical JSON
  and treats Markdown/YAML as presentation;
- resumable ingestion orchestration that never auto-accepts a legal decision
  or freezes an uninspected parser proposal.

## Scope and exclusions

This plan covers read-only corpus discovery and search, future subset
packaging, metadata/schema design, and structural-ingestion orchestration. It
does not authorize temporal model execution, bulk canonical-data migration,
automatic legal-review resolution, publication of unreviewed bundles, or
cross-provider model invocation.

The current implementation milestone does not copy corpus data into consumer
projects. Consumers point the CLI at a Lex-Mex checkout through
`LEX_MEX_ROOT` or `--root`. A copied subset needs a content-addressed bundle
manifest and remains a later milestone.

## Authoritative surfaces

- `crates/lex-cli/src/main.rs`: current CLI and batch orchestration.
- `crates/lex-core/src/lib.rs`: canonical Rust types and controlled values.
- `crates/lex-export/src/lib.rs`: canonical JSON and generated Markdown.
- `schemas/`: external schemas at trusted input/output boundaries.
- `corpus/mx/<slug>/`: committed canonical data and derived publication files.
- `batches/` and `schemas/batch-manifest.schema.json`: executable batch inputs.
- `docs/metadata-portability-and-ingestion-automation.md`: reviewed upgrade
  decisions and automation design.
- `docs/plans/cluster-2-federal-corpus-ingestion.md`: ingestion inventory,
  sequence, provenance, and current `lgbn` checkpoint.

## Initial baseline

The immutable baseline is local `main` at `16312049e` on 2026-07-25. The
worktree was clean and the branch was one commit ahead of `origin/main`.
Cluster 2 had 149 committed instruments, with `lgbn` still the next AD1
instrument. The CLI had ingestion, validation, export, review, batch, and
adapter commands but no consumer-oriented list, path, or search commands.

## Current checkpoint

Implementation is in progress from the baseline above. The consumer CLI
surface has been added without changing canonical data: `LEX_MEX_ROOT`,
`instruments`, `path`, and ripgrep-backed `search` with instrument filtering.
The independent metadata proposal has been compared with current canonical
types and repository trust boundaries. All required code checks and both
baseline corpus validators pass.

## Progress

- [x] (2026-07-25) Verified Git state, the active cluster-2 plan, current CLI,
  canonical JSON, Markdown front matter, Rust types, and exporter boundaries.
- [x] (2026-07-25) Added consumer CLI discovery, path, and search surfaces,
  including external-root and selected-instrument support.
- [x] (2026-07-25) Recorded the metadata proposal review and bounded unattended
  ingestion design.
- [x] (2026-07-25) Passed formatting, warning-denying Clippy, 96 workspace
  tests, LRITF and IFPE baseline validation, diff checks, and CLI smoke tests
  from an external working directory.
- [ ] Implement deterministic subset bundles after their manifest, compatibility
  contract, and validation rules are reviewed.
- [ ] Implement the resumable ingestion state machine and exercise it first on
  `lgbn` with a bounded one-instrument run.

## Decisions and discoveries

- Decision: canonical JSON remains the legal-data authority; Markdown and its
  YAML front matter remain generated presentation.
  Rationale: this is an explicit repository trust boundary, and reversing it
  would duplicate or weaken deterministic Rust ownership.

- Decision: expose a mounted-repository consumer mode before copied bundles.
  Rationale: `LEX_MEX_ROOT`, selected search, and stable paths solve the first
  application need without inventing a package compatibility contract.

- Decision: unattended ingestion is a resumable state machine, not a command
  that blindly combines `batch run` with `--freeze-counts`.
  Rationale: the first parse is a proposal; inspection and anomaly handling
  remain judgment work even when an agent performs them without operator
  babysitting.

- Discovery: much of the proposed metadata already exists in canonical Rust
  types or sidecars. The material gaps are portable external schemas, explicit
  compatibility/version semantics, subset manifests, citation data, and a
  deliberately designed historical-version model.

## Milestones, verification, and stop conditions

### M1 — Consumer CLI

Verify help output, JSON instrument enumeration, paths, whole-corpus search,
selected-instrument search, passthrough ripgrep flags, unit tests, Clippy, and
the two required baseline validators. Recover by reverting only the CLI and
documentation delta; no corpus files are changed.

### M2 — Portable subset bundle

Define a versioned bundle manifest containing selected instrument slugs,
canonical instrument IDs, source hashes, included artifact classes, Lex-Mex
schema/parser versions, generation time, and file digests. A bundle is valid
only if every included corpus validates before and after copying. Stop on an
unknown schema, changed source hash, missing target declared required by the
bundle profile, or any canonical diff outside the selected instruments.

### M3 — Resumable unattended ingestion

Add explicit `prepare`, `provisional`, `inspect`, `freeze`, `close`, and
`checkpoint` states with machine-readable receipts. Start with one instrument
and one known adapter family. Stop on an unexpected dirty worktree, source
acquisition/rate-limit exhaustion, parser anomaly, shared-code change,
validation failure, expected-edge failure, audited-review delta, or configured
instrument/time/commit budget.

### M4 — Bounded acceleration

Allow the harness to start a fresh bounded agent session after each successful
instrument or defect cluster, resume from the task plan and receipt, and
continue until the named batch or operator milestone closes. Commits and pushes
remain separately authorized operational actions.

## Recovery and handoff

Every orchestration receipt must identify the repository commit, task-plan
path and digest, batch manifest and digest, instrument slug, completed state,
validation artifacts, changed paths, and exact next state. Work products and
retry state stay under ignored `.work/`; accepted decisions and milestone
state stay in this plan or the cluster-2 plan. A new session verifies Git,
manifests, adapters, corpus presence, and receipts before resuming.

## Next action

Use the reviewed design to specify the subset bundle manifest before changing
canonical schemas or resuming `lgbn`.

## Outcomes and retrospective

M1 is complete: external consumers can enumerate, locate, and search selected
committed corpora without running inside the Lex-Mex checkout. No canonical
data changed. Portable copied bundles and unattended ingestion remain planned
milestones.
