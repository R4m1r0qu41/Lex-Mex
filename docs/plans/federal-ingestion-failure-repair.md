# Federal ingestion failure repair

## Purpose and scope

Review the operator-annotated 2026-10-14 campaign report and all 128 failed
identities, diagnose against retained official source bytes, implement narrow
deterministic parser repairs with regression fixtures, and retry affected
sources. Report genuinely ambiguous cases for operator inspection. Review the
14 held identities separately; do not infer legal approval, admit provisional
corpus, run temporal models, commit, push, or deploy.

Deliver the findings in `docs/federal-ingestion-failure-repair-report.md` in
the main repository, not only in a hidden worktree. Preserve operator notes.

## Authority and immutable initial baseline

- Main repository: `/Users/jr/Dev/lex-mex`, branch `main`, HEAD
  `1a37ccde2e113e550d08a82bbead5620c9b6d868`.
- Initial unrelated state: deleted `nota-viabilidad-comites-banca-multiple.docx`
  and `.md`, untracked Word lock file. Preserve all three entries.
- Read `AGENTS.md`, `CODEX.md`, `PLANS.md`, relevant accepted decisions, and
  applicable nested instructions. Existing provider layout and CODEX fallback
  already comply with the on-demand convention; active capsule establishes
  registration but its checkpoint 77 predates current HEAD.
- Annotated input:
  `.delta/worktrees/tf70fk1hzqdh/Lex-Mex/docs/federal-campaign-result-2026-10-14.md`.
  That worktree also holds uncommitted campaign implementation and original
  `.work/campaigns/federal-text-2026-10-14/` evidence. Read as evidence; preserve
  its work. Main lacks this campaign implementation. Integrate only reviewed
  dependencies needed for reproduction, with provenance recorded.
- Source text and receipts establish structural facts; operator annotations
  guide inspection and do not confer new legal-review approval.

## Milestones and verification

1. Inventory all 128 failures and 14 holds from frozen receipts; reconcile
   annotations on CJM (including article 434 definitions for later linking),
   LAAM, LISSFAM, LGPIST, REG-LGEEPA-MOE and LMIGRA. Inspect exact source
   boundaries; distinguish observed symptoms from confirmed mechanisms.
2. GPT-6.1-Sol executes repair: review campaign dependency diff, make minimum
   necessary main-repository integration, reproduce defects with fixtures,
   then repair signature boundaries, multiline furniture, ordinal headings,
   appendix attribution and evidence identity as supported by sources.
   Do not weaken validation or fabricate publication dates/counts. Preserve
   source evidence, legal decisions, and exact text/offset integrity.
3. Selectively retry affected identities using retained source bytes in an
   isolated output root. Inspect manifests, validation, canonical candidate
   diff, review queue and Markdown. Classify every failed identity as repaired
   and verified, still failing with precise blocker, or requiring operator
   judgment. Do not confuse provisional export with canonical admission.
4. Run `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
   `cargo test --workspace`, `cargo run --locked -p lex-cli -- validate lritf`,
   and `cargo run --locked -p lex-cli -- validate ifpe-dcg-2021`; delegate named
   deterministic verification to the mechanical role. Record actual results.
5. Final main-repository report gives source-backed findings, all dispositions,
   changes, tests, and precise questions/source locations for unresolved cases.
   Update this plan and continuity artifacts with exact paths and next action.

## Recovery and stop conditions

Retain original source bytes, campaign state and worktree changes. Retry into
separate outputs; never overwrite audited canonical data. Stop an individual
repair on uncertain source partition, missing evidence, or legal modeling
choice; write the report and continue independent cases. Stop implementation
when supported fixes and verification finish and every remaining failure has
an actionable explanation. No commit is authorized in this session.

## Current checkpoint and progress

- 2026-10-15: Read repository instructions and rollout plan, inspected main and
  evidence-worktree state, located and read annotated report. Existing report
  counts: 87 signature, 21 furniture, 1 nested structure, 8 parse/evidence,
  11 validation failures; 14 separate holds.
- Initial investigation hypotheses: signature markers may omit older formats;
  long titles may split page furniture; numeric ordinals may explain 1–9 gaps.
  These are hypotheses pending inspection, not accepted repairs.

- 2026-10-15 source-backed implementation milestone: all 128 frozen failures
  replayed diagnostically in `.work/federal-repair-replay-v1/`. Confirmed
  signature/promulgation boundaries, publisher-adjacent multiline titles,
  ordinal glyph validation, military `/o` headings, thousands-number sequence
  regression, wrapper false positives, source publication-note variants,
  judicial appendix misattribution, errata and stray-dot heading boundaries.
  Fixtures preserve bounded exact excerpts. The CLI campaign implementation
  is not required: `prompts/replay-federal-ingestion-failures.py` reuses existing
  extract/parse/validate/export stages with isolated copies of retained bytes.
  No campaign source-worktree edits or main corpus admission.
- Verification milestone: `.work/federal-repair-checks-v2/` full five-command
  gate passed before secondary LOEFAM/OGA/LSINT repairs. A new full gate and
  final frozen replay are pending; v1 export configuration errors are harness
  errors and must not be counted as parser blockers.

- Final review checkpoint: corrected replay v2 passed 126/128 failures and all
  99 original provisional successes. Remaining LMIGRA restriction is the
  explicit law/decree transitory-layer boundary, separate from its conditional
  commencement. LOEFAM export exposed a raw slash-in-filename defect after
  `/o` headings became parseable; a filename-only repair and real Obsidian
  export regression now cover it. The 99-case diff review found 12 instruments
  with 26 text changes, consisting of header/signature apparatus removal, no
  provision-ID changes and no reform-evidence ID removals. Exact counts will be
  bound to the final frozen executable replay below.
- Source sequence audit: OGA contains all 1–1868 source article headings;
  diagnostic old parser emitted 1000 articles ending at 1290. Thousands labels
  were treated as regressing below article999, losing 868 separate articles.
  Final replay must match the complete source ID set, not merely validate.

- Source-completeness checkpoint after v3 (127/128 stage passes plus 99/99):
  zero-transitory inspection uncovered 8 REG-LFAFE clauses under the literal
  `TRANSITORIOS.` heading and 5 REG-LSM Roman-numeral clauses. Both need narrow
  grammar repairs; pass/fail validation alone did not establish completeness.
  OGA's first note is a serialized 1º–8 January 1912 publication range, but the
  old date lookup falsely selected a 1985 reform date; its only original
  transitory is unnumbered. Fail closed on these unsupported identities.
  OGA's 1868 recovered article bodies are diagnostic only, not an admissible
  dated candidate. These new material findings are in the authorized scope.

- Final checkpoint, 2026-10-15: all supported repairs and 15 bounded source
  fixtures are in main. Final five-command gate passed in
  `.work/federal-repair-checks-v8/`: 238 workspace tests, LRITF 145/11/126 with
  zero issues, IFPE 59/4/113 with zero issues. Formatting/build also passed.
- Final frozen replay generation v5 uses one executable
  `b65a6806c02d8e897d7a99c0471c9137ed278d99aa3e48e7219622a92278473e`.
  Original 128 failures: 126 pass extraction/parse/validation/export; LMIGRA
  remains blocked on two transitory layers, OGA on serialized original
  publication metadata and an unnumbered clause. All 14 prior holds retained.
- Regression 99: 98 pass; LRCDN now fails explicitly on its previously omitted
  unnumbered original transitory. 16 instruments/30 existing text changes were
  inspected; LPF gains 5 original clauses and LSSS gains 1. No prior provisional
  IDs or legislative evidence IDs/text were lost. REG-LFSV 193/11 preserved;
  its lowercase wrapped `transitorio.` negative fixture prevents truncation.
- Across 227 source replays, PDF/text hashes and acquisition identity remained
  unchanged. Main corpus/schemas/legal decisions and source evidence worktree
  remain unchanged. No stage/commit/push/deploy/admission occurred.
- Final visible outputs: `docs/federal-ingestion-failure-repair-report.md`
  (all 128+14 individual dispositions and exact unresolved source excerpts),
  `docs/federal-ingestion-failure-repair-evidence.json`, replay harness and
  fixture provenance. Parent/integrating session owns continuity receipt and
  active-capsule checkpoint.

**Next action:** integrating review of the completed follow-up implementation,
report and audit below; no canonical admission or temporal-review decision.

## Outcomes and retrospective

Execution complete within the authorized stop condition: deterministic defects
were repaired and verified; uncertain source identities were reported rather
than fabricated. The final report supersedes diagnostic pass counts. Validation
alone initially missed omitted clauses and a false seventh LGAPPCEA transitory;
source-count and text-diff inspection were necessary to catch these. A new
period-heading rule briefly truncated REG-LFSV at wrapped lowercase prose;
its exact-source regression and corrected final replay verify preservation.
No canonical publication or commit was authorized or performed.

## Follow-up: operator annotations and implementation contract

### Baseline and authority

The operator annotated `docs/federal-ingestion-failure-repair-report.md` after
checkpoint 80 and requested diagnosis and a plan for GPT-6.1-Sol coding.
Preserve that annotated report verbatim; write follow-up findings to
`docs/federal-ingestion-followup-report.md`. Prior repair code, fixtures and
v5/v8 evidence remain the baseline, uncommitted at `1a37ccde2`. Preserve all
unrelated changes and the original hidden evidence worktree. Existing 14
holds are not cleared by this request.

Operator clarifications accepted for structural implementation:

- LMIGRA is issued within an enclosing decree. The law owns its ten original
  transitories; the enclosing decree owns its four distinct transitories.
- LRCDN has one original transitory despite omission of the word UNICO.
  Use a stable internal identity without inserting UNICO into official text.
- OGA's serialized publication is an explicit exceptional source form.
  Represent the stated 1–8 January 1912 period faithfully, never substitute
  the 1985 reform date or arbitrarily call either endpoint the exact date.
- Transitories can establish commencement, deadlines, surviving effects and
  rules for older permits/proceedings. Passage of time, a budget year, or an
  executive deadline is not evidence that every effect ended or that an
  implementing regulation was timely published. No automatic stale/repealed
  classification or reviewer resolution belongs to this parser repair.

### Diagnosis and design decisions

1. **Source-owned transitory layers.** Replace the limited flattening path
   with explicit structural states. Keep law-owned clauses in provisions.
   Add a typed, separately persisted enactment-decree evidence collection,
   distinct from law provisions and later reform evidence. Include stable
   decree-qualified IDs, owner/scope, exact source heading/ordinal/body and
   publication/source provenance. Recognize layers only from corroborated
   enacting wrappers and sibling-article/outer-heading boundaries; ordinal
   restart alone is insufficient. Preserve elided sibling articles as elided
   evidence, never reconstruct their missing text.
2. **Sole unnumbered clause.** Preserve the source heading (`TRANSITORIO` or
   `ARTICULO TRANSITORIO`) and body. Assign a deterministic nonprinted
   identity such as `transitory:unnumbered-1`, explicitly distinguished from
   a source ordinal. Accept only a singular section with one unnumbered
   clause and a verified closing boundary; ambiguous mixtures still fail.
   Apply to LRCDN and OGA, without changing previously numbered identities.
3. **OGA publication period.** Add an explicit adapter exception backed by
   its exact retained source note. Exact-day publication fields become
   optional where needed, with validated period metadata recording inclusive
   start/end and source evidence. Exact-day records retain their JSON values
   and semantics. OGA must have no asserted exact publication day, including
   its provisions. Consumers must represent the period or reject unsupported
   records explicitly; no coercion to an endpoint. Update types, schemas,
   validators, serialization, CLI index/export and relevant consumers together.
   Document version compatibility. Leave unrelated standard metadata alone.
   Date-relative derivation and model request creation must fail closed where
   an exact day is required; no legal commencement review is performed here.
4. **Presentation and portability.** Export decree-owned clauses with an
   explicit separate heading/ownership, include their evidence in portable
   outputs where supported, and never count them as own-law transitories.
   Preserve review-state isolation and distinguish date ranges in Markdown
   and structured output. Persisted sidecars must be loaded and validated,
   not merely written and forgotten.

### Execution milestones and acceptance

1. GPT-6.1-Sol implements source-layer and sole-clause changes with bounded
   exact-source and negative fixtures. Parent reviews semantics. LMIGRA:
   preserve all source article IDs, ten law clauses, four enclosing-decree
   clauses and subsequent reform evidence without ordinal collision.
   REG-LMIGRA is an explicitly related attribution regression: retained
   source has article base 250 (262 article records including suffixes),
   then sibling decree articles II/III and thirteen decree clauses. Verify
   this source shape and preserve all article IDs; do not invent a regulation
   transitory layer merely to retain old counts. LRCDN: 31 articles and one
   source-labeled unnumbered clause.
2. Implement the bounded OGA exception and complete consumer/schema support.
   Verify 1868 distinct source articles and one unnumbered clause; exact date
   absent and period 1912-01-01 through 1912-01-08 retained. Include negative
   tests for reversed ranges, conflicts with exact dates, wrong source note,
   reform-date fallback, and temporal consumers guessing an endpoint.
3. Replay the four affected sources against retained bytes in fresh isolated
   outputs. Inspect manifests, source/evidence attribution, validation,
   candidate diffs, review-state behavior and Markdown. Extend regression
   replay to the prior 227-source set because core date/types and shared
   parsing change. Count intentional ownership moves separately from loss;
   prove preserved bodies/IDs or explicitly document any namespace migration.
   Retain original v5 audit unchanged as historical evidence.
4. Delegate the repository gate to `mechanical`: `cargo fmt --check`,
   `cargo clippy --workspace --all-targets -- -D warnings`,
   `cargo test --workspace`,
   `cargo run --locked -p lex-cli -- validate lritf`, and
   `cargo run --locked -p lex-cli -- validate ifpe-dcg-2021`.
   Verify backward compatibility with existing committed corpus and unchanged
   audited reviews. No canonical admission, commit, push or deployment.
5. Write the follow-up report with exact outcomes, source-backed decisions,
   validation, remaining gaps and future work. Update this plan at milestones.
   Stop only after supported implementation and checks finish, or document a
   genuine unresolved attribution/model conflict while completing other work.

### Source investigation and deferred temporal work

The linked [Reglamento PDF](https://www.diputados.gob.mx/LeyesBiblio/regley/Reg_LMigra.pdf)
confirms its enclosing-decree structure. Its source publication note identifies
28 September 2012; publication and actual entry into force are separate facts.
Use retained bytes for replay; no automatic resolution of LMIGRA conditions.

The [LEYFED download page](https://www.diputados.gob.mx/LeyesBiblio/legiscd.htm)
offers a ZIP with a PDF index, as well as a Windows installer. On inspection
it labels the bundle updated 3 March 2026 and includes law reform decrees from
September 2003 onward. Potential future use: bounded completeness comparison
and decree-source discovery. No bulk download or executable execution is
needed for this work; bundle coverage cannot replace per-source provenance.

Record as future work: separate act-level amendment inventory, decree-owned
transitory tracking, cross-instrument repeal evidence (including the 2000
Naval regulation noted by the operator), verified publication/commencement
conditions, surviving effects on older permits/proceedings, and executive
deadline compliance. Do not infer global legal status from these examples.

### Follow-up checkpoint

- 2026-10-15: Read all operator annotations and current dirty state; discovered
  capsule 80. Read-only GPT-6.1-Sol diagnosis confirmed mandatory single-day
  types and temporal consumers require coordinated changes for OGA. Parent
  accepted explicit date-period representation and separate decree evidence.
  Official linked pages inspected; original annotated report remains intact.
- 2026-10-15 implementation milestone: added separate typed decree evidence,
  persisted sidecar loading and hash/ownership validation, source-faithful
  implicit singleton metadata, optional exact-day publication plus versioned
  serialized-period metadata, guarded temporal consumers, portable evidence
  and separate Spanish Markdown presentation. Full regression is pending.
- Source replay exposed a pre-existing LMIGRA Article 3 glossary collision:
  legacy comma extraction truncated four distinct colon labels beginning
  “Niña, niño o adolescente”. Parent approved an explicit adapter-only
  `mixed_fractions` style, including the source's page-joined XIX after its
  exact `(antes fracción XXI)` editorial note. Generic style inference and
  validation remain unchanged. Exact-source and negative fixtures added.
- 2026-10-15 final follow-up checkpoint: all 227 frozen-source E2E cases pass
  with binary `da1a7aab3e18113caacec0155a4aaba4ea71f8d046cda28110873a099707185b`;
  all source bytes/acquisition identities unchanged. LMIGRA 162/10 plus 4
  decree clauses/1 elided sibling; REG-LMIGRA 262/0 plus 13 decree clauses/2
  elided siblings; LRCDN 31/1 and OGA 1868/1. Independent source-heading sets
  match all four complete article ID sets. OGA exact day is null and its
  1912-01-01..1912-01-08 period remains explicit in all records/exports.
- Regression comparison: 223 prior passes retain identical provision records
  and reform evidence. REG-LMIGRA preserves all 262 article IDs and moves 13
  unchanged bodies to decree-qualified identities. Article 250 loses only the
  elided sibling wrapper, retained in separate evidence; no unexplained loss.
  All 224 prior passes retain identical later reform evidence. All 14 holds
  remain. Original annotated report/audit hashes verified unchanged.
- Final delegated gate `.work/federal-followup-checks-v7/` passes all five
  required commands, 250 tests, LRITF 145/11/126 and IFPE 59/4/113, zero issues;
  formatter/build pass too. All 230 existing ordinary corpora have consistent
  exact-day metadata; no corpus/review bytes changed. No admission or commit.
- Deliverables: `docs/federal-ingestion-followup-report.md`,
  `docs/federal-ingestion-followup-evidence.json`, five source fixtures with
  provenance, additive schemas and the separate follow-up replay harness.
  Implementation and verification complete. Integrating parent owns final
  continuity receipt/checkpoint and the single next action above.
