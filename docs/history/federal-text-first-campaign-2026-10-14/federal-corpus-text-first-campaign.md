> **Historical record, 2026-10-16.** Produced 2026-10-14 against a 262-instrument corpus and an uncommitted `campaign` tool that was never landed. Superseded: the federal laws and regulations were admitted by the held-source route (467 instruments) and the project is in maintenance mode for them. Kept as provenance only; its counts and next steps are not current. Source patch: git-ignored `archive/campaign-2026-10-14-patch/`.

# Federal corpus text-first campaign

## Purpose and observable outcome

Run one resumable ingestion campaign across the explicitly inventoried federal
laws and regulations. Account for every candidate, preserve every failed attempt
with a reason and evidence, group related failures into repair batches, and
prepare structurally sound Markdown without requiring corpus-wide graph closure.
The operator authorized execution on 2026-10-14.

SQLite + `rusqlite` is the selected base for a later derived corpus database,
primarily for model retrieval and evidence-backed use, also for human reading.
Database serving, semantic inference, a future Rust service, reader deployment,
legal-review resolutions, temporal model runs, and historical-version synthesis
are excluded from this campaign.

## Authority and source boundaries

Read `AGENTS.md`, `CODEX.md`, `PLANS.md`, `docs/decisions.md`,
`docs/metadata-portability-and-ingestion-automation.md` and
`docs/ingestion-difficulty-log.md`. Existing Rust parsers/validators, schemas,
canonical records and audited review decisions outrank this plan.

Inputs are official source inventories and existing batch manifests/adapters.
The historical 454-instrument Cámara universe is a starting inventory, not proof
of coverage of every federal regulatory authority. NOM/NMX have a separate
compiler boundary; classify them separately rather than forcing them into laws.
Retain catalog provenance and explicitly identify unprepared or missing scope.
Prepared prompt files are script inputs, never ambient model reading material.

## Immutable initial baseline

- HEAD: `1a37ccde2e113e550d08a82bbead5620c9b6d868`; branch `main`.
- Worktree was clean at startup.
- `pdftotext`, `pdftohtml` and Cargo are available.
- Existing `batch run` isolates item failures but writes its report at the end,
  writes parse artifacts directly into the corpus and closes reverse links.
- Existing publication documentation reports a four-instrument reader release.
  This is historical documentation, not a live website or release audit.
- No active capsule was discovered at startup.

Refresh HEAD and status when resuming; never reset to the initial baseline.

## Current checkpoint

Runner hardened and integrated; the final required Rust gate passed (243 tests,
including 20 campaign tests). Independent review found no further single-process
staging blocker conditional on the adapter fingerprint guard; the parent
implemented that guard and verified its regression. Final real pilot passed the
isolation/failure gates. The full prepared-inventory first pass completed:
91 present, 99 provisional exports, 128 failed, 14 held, zero unattempted and
no global abort. No new canonical admission or reader deployment has occurred.

Durable result and every failed/held identity:
`docs/federal-campaign-result-2026-10-14.md`.
Local final `report.json` SHA-256:
`ac0d7d4fd3969bd8ff08a88e2cba146bfe54ba9c99767cb05139214f04fda3b3`.
The 99 provisional exports contain 10,806 articles, 784 transitories and
11,689 Markdown files. The existing committed corpus remains 262 instruments.

Full campaign command (also the resume command, retaining both frozen inputs):

```bash
target/debug/lex-mex campaign run federal-text-2026-10-14 \
  .work/federal-campaign-inputs-2026-10-14-v3/manifest.json \
  batches/federal_catalog_discoveries_2026_10_14.json
```

The full inventory is 332 identities: 317 runnable rows and 15 held entries.
Successful new exports are provisional. Selective retry adds `--only <slugs>`;
unselected historical accounting is not a fresh whole-campaign verification.

Frozen normalized input:
`.work/federal-campaign-inputs-2026-10-14-v3/manifest.json`,
SHA-256 `8a89c2f5c1dd949fb21ac63688c806c2d895a90c99158f9676fb37dc15a1fc24`.
Its adjacent `inventory-audit.json` retains all input digests, original prepared
metadata/holds and official catalog response hashes. These local ignored artifacts
are not replicated or committed; tracked scripts reproduce them.

- 53 prepared manifests: 323 unique candidates, 91 already canonical, 232 absent.
- Conservative normalization: 316 runnable rows and 15 held rows; operational
  holds outside the prepared set are retained, hence 331 accounted identities.
- Six absent prepared candidates remain held: `lcnbv`, `lcmopfih`, `lisipl`,
  `lfiiedb`, `lga` (Archivos collision), and `lgag` (Aguas hold).
- A bounded follow-up reconciled all 26 unmatched primary law/law-regulation
  links. Most are historical, reform or errata source alternatives. One new
  current regulation is explicitly inventoried as `reg-lic-calidad` in
  `batches/federal_catalog_discoveries_2026_10_14.json`; include that manifest
  alongside the frozen prepared input in the full campaign. Evidence and
  historical-scope holds: `docs/federal-catalog-reconciliation-2026-10-14.md`.
- The fresh four-catalog audit found 139 unmatched primary/discovery PDF links,
  not 139 verified missing instruments. Mobile law duplicates and archive links
  are retained as HTML evidence but excluded from current-candidate discovery.
  Other federal regulations/statutes remain unprepared pending source identity.
- Canonical-Markdown bundle:
  `.work/federal-reader-bundle-2026-10-14/bundle-manifest.json`,
  SHA-256 `35b5b8e4cb44a884434fba6d7f8ce27e1214f7db934c8d7ab8fab24004ed735b`,
  230 instruments, 44,054 files. This is a portable producer bundle, not a
  completed reader release or legal-validity certification.
- Including all 32 NOMs in a Markdown bundle failed because
  `nom-001-semarnat-2021` lacks generated Markdown; no output was created by
  that failed attempt. NOMs remain outside this laws/regulations first pass.

## Decisions

1. Extend the Rust CLI instead of introducing an independent ingestion pipeline.
2. Stage new work under `.work/campaigns/<id>/`; never overwrite canonical
   artifacts or audited legal state during first-pass acquisition.
3. Preserve strict validation. Passing machine checks alone do not establish
   complete extraction. Do not blindly freeze counts from the parser's output.
4. A new passing candidate awaits official-source/structural inspection before
   admission. Failures and pre-existing holds retain evidence and do not block
   unrelated items.
5. Defer global reverse-link closure, not source integrity or existing resolved
   edge validity. Track text, graph and publication readiness separately.
6. Keep attempts durable at stage boundaries, including interruption recovery.
   Resume requires matching inputs/implementation and intact output artifacts.
7. Group symptoms and suspected mechanisms separately from confirmed causes.
8. Existing corpus publication can advance independently, but site edits require
   the actual reader repository to be attached; deployment is not authorized.
9. Pilot source inspection found LFSA's last transitory includes legislative
   signatures and the promulgation formula despite a passing validation report.
   This is a known source-boundary defect, not eligible text. Add a conservative
   detection gate/regression fixture and retain the candidate for stop-marker
   repair; a passing validator must never be treated as inspection approval.

## Milestones, verification and recovery

### M1 — Inventory and baseline

Reconcile prepared manifests, adapters, canonical corpus, known holds and
official catalog scope using scripts. Identify duplicated/conflicting identities,
remaining candidates and source inventory dates. Freeze the campaign inputs with
digests and report unprepared scope. Existing legal holds remain explicit.

Verification: reconcile every inventory item into a mutually exclusive execution
disposition; do not sum overlapping text/link/publication readiness counts.
Recovery: no acquisition occurs before inventory/preflight succeeds.

### M2 — Durable staged campaign runner

Add typed CLI options, ledger/receipts, isolated stages, reports and repair
manifests. Reuse existing fetch/extract/parse/validate/export functions. Preserve
existing batch behavior. Avoid external Obsidian writes in campaign execution.

Verification: focused tests for dedup/conflicts, blocked-item handling, interruption,
fingerprint changes, corrupt artifacts, failed stages and grouped retry selection.
Verify no candidate writes into live canonical paths.
Recovery: durable attempt receipts precede continuation; unsafe filesystem/global
invariants stop the campaign, item-local failures continue.

### M3 — Representative pilot and code gate

Run the real CLI on normal law/regulation candidates, an existing canonical
candidate, a known hold and controlled failures. Inspect official source hashes,
extracted text, labels/counts/order, validation, source boundaries, generated
Markdown and unchanged canonical/review state.

Run:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run --locked -p lex-cli -- validate lritf
cargo run --locked -p lex-cli -- validate ifpe-dcg-2021
```

Validate every newly admitted instrument separately. The two named validators
are regression checks, not proof of full campaign validity.

Recovery: do not start the full campaign until staging/resume invariants pass.
Add regression fixtures for material parser defects; never solve legal questions
or weaken gates just to increase success counts.

### M4 — Full inventoried campaign

Build once per implementation revision. Process remaining eligible sources with
bounded retries and source-host rate limits. Persist evidence and reports
continuously. Inspect new provisional artifacts in bounded clusters, admitting
only source-reviewed candidates. Track anything awaiting inspection honestly.

Verification: final report accounts for all frozen inventory items, includes
held/unattempted/failed candidates, and emits actionable grouped retry inputs.
Exercise selective retry and no-op resume with the exact CLI implementation.
Recovery: stop safely on disk/permission/global corruption; preserve completed
attempts and one exact resume command. Never claim all federal regulations were
ingested if official inventory reconciliation is incomplete.

### M5 — Admission, release candidate and repair rounds

Promote source-inspected passing candidates without replacing unrelated
canonical state. Review canonical diffs and review queues; validate exports.
Prepare a broader pinned reader release through the existing supported interface
if available in attached worktrees. Otherwise report the producer/reader boundary
and exact blocking attachment. No push or deployment.

After a shared defect is fixed, test and retry all affected group members, not
the whole corpus. Keep resolved attempt history. Linking is a later bounded
phase against stable canonical identities.

## Stop conditions

Stop the affected item for source/structural ambiguity, unknown official identity,
missing formal evidence required for a decision, or a legal modeling question.
Stop the campaign for unsafe concurrent canonical edits, broken global identity
invariants, storage failure or corrupt campaign state. Do not overwrite an
audited human decision, admit known malformed text or invent missing metadata.

## Timestamped progress

- 2026-10-14: Verified clean startup baseline and tooling; recorded approved
  SQLite + rusqlite boundary. Started independent inventory audit and isolated
  campaign implementation. No model temporal calls or external publication.
- 2026-10-14: Completed metadata inventory audit and conservative normalization;
  four Python inventory tests passed. Baseline fmt, Clippy, 223 workspace tests,
  LRITF and IFPE validations passed. Validated all 230 non-NOM corpus records
  (audit resumed after terminal time limits); no canonical diff resulted.
  Built the 230-instrument portable Markdown bundle without site edits.
- 2026-10-14: Real CLI pilot attempted `lfsa`, `reg-lfsa`, `lfzee` and `lfiiedb`
  using the frozen consolidated inventory with `--only`. Acquisition/extraction,
  parse, validation and Markdown export succeeded provisionally for the first
  pair; existing LFZEE accounted present and LFIIEDB held. No global abort or live
  canonical/adapter diff. Source manifest, validation, first/last provisions and
  Markdown counts inspected: LFSA 177 articles/10 transitories; REG-LFSA
  373/8. LFSA signature leakage was discovered and queued for a new gate before
  full run. Source SHA-256:
  LFSA `199baa54ad422398c57a8c173ef25c3a75ed0d6532128d244fcd46246e6f4e08`;
  REG-LFSA `a3435ac18a1631617fea83ae6a81bad84f456e3e8fd44b69bb6b7ea39425df9c`.
  Initial pilot export is provisional evidence only and must not be published.
- 2026-10-14: Independent scratch-only review reproduced unsafe pre-existing
  temporary-file hardlinks, symlink campaign ancestry, concurrent-run races
  and polluted pending dependency snapshots in the initial runner. No actual
  corpus was altered. Full acquisition remains gated on fixes/regression tests
  for these path-isolation and dependency-integrity defects.
- 2026-10-14: Integrated fixes for exclusive atomic files/attempt/snapshot
  directories, non-link output ancestry, OS-backed lock, exact dependency maps,
  directory sync, prerequisite preflight, bounded fetch retries and enriched
  reports. Parent added captured-map fingerprinting and staged adapter-copy
  comparison to stop concurrent parser-option changes before acquisition.
  Final fmt/Clippy/workspace 243 tests/LRITF/IFPE gates passed. The manifest
  fixture census was updated to 44 batches/227 rows for the one new discovery.
- 2026-10-14: Final real pilot (`federal-text-final-pilot-2026-10-14`) accounted
  332 identities with `--only lfsa,reg-lfsa,lfzee,lfiiedb,reg-lic-calidad`.
  LFSA failed `CAMPAIGN_SOURCE_BOUNDARY_SIGNATURE` with exact-span/provision
  evidence and zero Markdown; REG-LFSA produced 382 provisional Markdown files;
  REG-LIC-CALIDAD produced 265; LFZEE present; LFIIEDB held. No global abort,
  canonical/adapter diff or legal-review resolution.
- 2026-10-14: Executed the full campaign across both frozen manifests. All 227
  new unheld candidates fetched/extracted; 99 exported provisionally, 128 failed
  safely outside canonical corpus. Failure packets: 87 signature boundaries,
  21 page furniture, 1 nested-law structure, 8 parse/evidence and 11 validation.
  CLI completion-with-holds/failures exit was expected; no global abort.
- 2026-10-14: Independently verified raw source/extracted-text hashes, passing
  reports and exported READMEs for all 99 successes. Actual selected-success
  resume appended no attempts/repeated no source stages. Selective unchanged-
  source LFSA retry appended only attempt 2, failed the same boundary gate and
  left the other 331 attempt counts unchanged. Canonical corpus/adapters remained
  unchanged. Wrote the shared result and cross-batch difficulty summary.

## Exactly one current next action

Repair the 87-member signature-boundary family with representative source-backed
fixtures and a narrow coherent rule, then selectively rerun that group.

## Outcomes and retrospective

M1–M4 first-pass acquisition and reporting are complete for the prepared inventory
plus one explicit discovery. No canonical admission or reader deployment is done.
The 99 provisional successes require bounded source/structural inspection before
admission; 128 failures and 14 held cases remain explicit. Broader federal-source
identity reconciliation and graph/database serving remain separate milestones.

The pilot exposed why validation alone is insufficient: LFSA passed the old
structural report while swallowing signatures. Independent review also exposed
filesystem isolation defects before full acquisition; regression-backed fixes
preceded the run. Retain the staging boundary and do not equate successful
acquisition, source completeness, legal validity and publication.

Continuity: the active-run capsule is checkpointed explicitly by the prescribed
helper. This plan and `docs/federal-campaign-result-2026-10-14.md` are the shared
repository handoff. The external Agent Vault Git checkout is not attached to
this Delta thread, so its tracked daily receipt and rolling context were not
manually edited. Attach that checkout before completing those external writes.
The active run remains open because source inspection/admission and repairs are
unfinished.
