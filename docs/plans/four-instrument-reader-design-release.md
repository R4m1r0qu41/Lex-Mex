# Four-instrument Reader design release — single-run execution plan

2026-10-22. Owner: Claude Sonnet5.5; Opus5.5 advisor on call. Operator approved
this bounded implementation after Codex's Reader handoff review. This is an
execution plan, not a report of completed implementation.

## Goal and observable finish

Deliver one reproducible, locally reviewable Reader package implementing the
approved Revision02 design for exactly `lritf, ifpe-dcg-2021, itf-dcg-2018, lic`.
Produce source-backed release comparison, implementation, verification receipts,
responsive screenshots, and a final publication/rollback packet. Complete all
local work in one sustained goal run; do not stop after a plan or first passing
test. The finish is **ready for operator review and host deployment**, not live
publication. No host modification/deploy, Git commit/push/merge, corpus legal
resolution or external write is authorized by this plan. Obtain any later
commit/deploy authorization only after the concrete package is reviewable.

Suggested goal text: “Execute docs/plans/four-instrument-reader-design-release.md
to completion: integrate approved Revision02 into a verified four-instrument
Reader package, preserve unrelated work, and deliver the complete local
publication and rollback evidence. Use Sonnet5.5 as owner and Opus5.5 on call.
Stop only at the defined acceptance boundary or a documented external/legal
blocker; do not deploy or commit without separate authorization.”

## Authority, baseline and bounded context

Initial main: `85c41f3898d86737766564fa8d3fd8f6d4ae74a3`, pushed and verified.
Reader: `/Users/jr/Dev/lex-mex-reader`, branch feat/public-reader at
`f5330c3e9ef8c087a1551b9e85c3c7233a4fa595`,43 dirty entries at review.
It is the permanent online presentation workspace; never remove/reset/clean it.
Verify current Git drift and active writers before starting. Canonical data,
accepted decisions and native checks outrank this plan. Read local AGENTS.md,
provider instructions and PLANS.md in every worktree actually used.

Read only these task sources initially, then follow specific implementation links:
- `docs/reader-publish-handoff-review-2026-10-22.md` (main)
- `docs/heading-leak-permanent-acceptance-2026-10-21.md` (main)
- Reader `reader/README.md`, `reader/{index.html,reader.js,reader.css,reader-core.mjs,package.mjs,stage.mjs}`
- Reader `docs/plans/continuous-reader-and-agent-query.md`
- Reader `docs/plans/database-backed-agent-access.md` for exclusions only
- Incoming handoff `2026-10-21-lex-mex-reader-publish-handoff.md`

Accepted design: Figma ar2bES9p6jeUBY0w3E4eDY, approved board160:570
(Revision02). Inspect that board and required components with applicable Figma
skills before implementing; preserve existing designs. Research-tools board
246:438 is not approved for implementation here. Do not import its Cmd-hover
floating articles, session tray or article-reform footer. No Figma write needed.

## Settled scope and routine defaults

- Implement approved Revision02 over the four-instrument public release;
  full-corpus DB integration and new agent/query service are excluded.
- Use accepted main85c41f389 as the canonical comparison/build baseline; do not
  refresh sources or derive/rewrite LRITF or other canonical legal records.
- Exclude experimental static `agents/<id>/` generation and query-preview/service
  from this publication path. Preserve their code/drafts separately. Update
  packager/stager allowlists and llms/navigation consistently so no dead routes
  or experimental discovery promises ship. Keep existing supported canonical
  Markdown/text discovery if its packaged routes verify. This is the proposed
  exclusion accepted as the conservative four-instrument scope; document it.
- Continuous text, active article/index tracking, exact article copying, direct
  provision links and responsive layout are core. Existing source/PDF viewing
  remains. Ship PDF-follow disabled/unavailable unless exact article-to-PDF
  anchors can be verified against retained PDF hashes within this run; never
  infer page positions from article number or label occurrence alone. Clearly
  label unavailable controls. No research-tool footer or unverified reform date.
- Use local retained official sources with verified hashes. No new acquisition
  without a separate provenance review. No claim of current legal validity.
- Do not require a DB server or the internal482MB SQLite release for this
  four-instrument site. Preserve public-data exclusion and partial coverage.

## Work isolation and preservation

Inventory Reader dirty paths, hashes and owners before edits. Preserve unrelated
tracked deletions (`nota-viabilidad-comites-banca-multiple.*`), provider files,
assessment/prompts and query experiments. Do not merge the Reader branch wholesale.
Use an operator-created isolated integration worktree based on accepted main
per local concurrent-writer rules. If none exists, prepare its exact requested
path/branch and request creation; meanwhile perform bounded read-only inventory
and design review. Never silently share another writer's mutable checkout.
An already isolated operator-provided workspace is sufficient; do not ask again.

Transfer only reviewed Reader implementation/support files into integration,
with a path manifest and source hashes. Preserve attribution and local rules;
no broad recursive copy. Select an integration branch name and private
CARGO_TARGET_DIR/output roots so active DB/Reader jobs cannot collide.
All new package directories must be unique, ignored and outside their input
release. Existing release/publication files remain immutable.

## Execution milestones

### M0 — Baseline, design and source inventory

Record Git commit/status, chosen writer/worktree, design board, source/package
locations and tool versions. Read current host pin and retained release from
known files read-only; the handoff's pin may have drifted. No host enumeration.
Known retained release: Reader `.work/public-release-v2`, release3c04016651e2fd4169632756e96ec19136136e1f42c4ed666070f23f20b30660.
Known host pin is under `/Users/jr/Documents/4ll4/Webpage/LEG4L_L4BS/lex-mex-release.json`;
read host-local instructions before later host work, which is excluded here.
Identify only required retained source PDFs from the release/source store.
Do not preload corpus or unrelated vault contents.

### M1 — Public release parity and content disposition

Build the explicit four-instrument public release from the accepted baseline
using its actual current CLI help; README example is:
`cargo run --locked -p lex-cli -- public-release build --instrument lritf,ifpe-dcg-2021,itf-dcg-2018,lic --source-store <verified-store> --output <new-release-dir> --base-path /lex-mex/`.
Review the exporter/version compatibility before copying older Reader tooling.
Compare old/new public provision IDs, exact text/source digests, references,
terms if exported, temporal classifications and exposed review qualifications.
Classify all changes; no aggregate “unchanged” claim based on counts alone.

If public text, temporal or review facts differ, finish the comparison and
continue UI development on the retained approved release. Do not block all
implementation waiting for a legal-content ruling. Report the baseline update
as a separate held publication option for the operator. Default deliverable in
that case is a new UI publication over the retained release, with its original
corpus identity. Never infer JRH approval or silently replace legal content.
If no such changes require review, the newly verified baseline may be used;
record release identity and exact content disposition explicitly.

### M2 — Implement approved design and packaging boundary

Implement continuous instrument reading with one logical text scroll, stable
article anchors, selection/index tracking, direct-link/history behavior, exact
body copy and responsive source/PDF panes. Manual PDF scrolling must not be
hijacked; unavailable follow must remain honest. Preserve display of out-of-
release citations as unavailable, and source/release identity on relevant views.
Measure LIC (largest selected instrument) render/navigation; avoid eager DOM
work that blocks interaction. Do not expand the instrument set to test scale.

Remove experimental-agent output from this publication path without deleting
its unrelated work. Repair package/stage discovery references and add meaningful
regressions proving excluded/private files cannot ship. Query-preview defects
need not be repaired because that surface is excluded; any code path actually
reused in the release must be reviewed and repaired before shipping.
Use semantic controls, keyboard navigation, readable focus and empty/loading/
error states. Avoid invented dates or legal summaries. No new design invention
when approved components provide the answer.

### M3 — Verification and reproducible review package

Delegate deterministic checks to the local mechanical tier where required;
Sonnet owns diagnosis, scope and fixes. Native code/canonical-boundary changes
require fmt, strict workspace Clippy, locked workspace tests, LRITF and IFPE
validators. Run write-capable validators only in isolated disposable snapshots;
inspect canonical/queue/export diffs and expect no unintended changes.
Run `node --test reader/` and meaningful added behavior/privacy tests. Use
current main's test baseline; do not claim the old228-test Reader gate covers
integrated419-test main. Record exact counts, commands, exits and limitations.

Build publication twice from the same chosen input/UI; compare immutable
publication IDs and content digests. Use package/stage verification in an
isolated local staging directory, never the host. Tampered digest, forbidden
private artifact and invalid publication must fail closed. Verify complete
routes/relative assets, canonical discovery and direct links under `/lex-mex/`.

Browser-check desktop and narrow/mobile layouts on the local served package:
all four instruments, long LIC navigation, back/forward/deep links, full article
copy boundaries, reference navigation, unavailable external targets, source PDF
and disabled follow behavior. Record representative screenshots and measured
render/navigation observations, without claiming formal performance budgets
unless measured. Private-data denylist checks must supplement, not replace,
explicit public-field allowlisting. Confirm no fixtures silently replace real
release data. Source digests and retained PDF identity must verify.

### M4 — Reviewable publication packet and handoff

Write implementation receipt, baseline comparison (machine-readable plus human
summary), selected release/publication IDs and hashes, validation evidence,
screenshots, exact changed-file manifest, unresolved holds and rollback paths
under repository docs. Keep large generated packages/screenshots in ignored
artifact roots with reproducible commands; do not commit PDFs/DBs or private
vault material. Update this task plan, project receipt, rolling context and
active-run capsule explicitly. Bind capsule to plan path/digest. Do not mark
success until the final package is locally reviewable and all required gates
pass, or clearly name the external/legal blocker and completed independent work.

Provide the exact host staging commands, pin delta and rollback procedure as a
proposal. Do not run them. If publication differs legally, make the retained-
content UI package the ready option and the new-content version a held option.
Present a concrete commit/deploy request only after this packet exists.

## Opus advisor and stop rules

Sonnet5.5 owns execution. Consult Opus5.5 read-only at: an unclear source/UI
boundary, a proposed canonical/export schema change, ambiguous legal/public
content diff, architectural compatibility conflict, repeated substantive test
failure, or final independent readiness review. Supply a bounded question,
relevant diff and evidence; Sonnet integrates the advice. Do not invoke another
provider from Codex. Within Claude use its authorized advisor mechanism.

Do not escalate routine styling, path selection, tests or formatter work.
Advisor cannot grant operator/JRH approval. Stop dependent work for missing
approved design access, absent verified sources, active-writer conflict,
private-data leakage or unclassified source change; continue other safe local
milestones. Never hide failure by weakening validation or dropping fixtures.

Recovery: preserve original immutable package and host pin; abandon only the
new isolated output on failure. Revert only owned scoped changes if necessary;
never reset the permanent Reader workspace. No cleanup without specific authority.

## Current checkpoint — implementation reviewed, 2026-10-22

M0–M4 executed by Sonnet in the operator-created isolated worktree
`/Users/jr/Dev/lex-mex-reader-four`, branch `feat/four-instrument-reader-design`
at main85c41f389 plus uncommitted implementation. Do not restart from M0 or
copy the permanent Reader again. Preserve all existing implementation and ignored
outputs. Verify drift and sole writer before editing. The original packet is
`docs/four-instrument-reader-implementation-receipt-2026-10-22.md` in that worktree.

Read the main-workspace review `docs/db-reader-lic-consolidated-review-2026-10-22.md`,
`docs/four-instrument-reader-codex-review-2026-10-22.md` and
`docs/lic-publication-source-audit-2026-10-22-evidence.json`. They are currently
uncommitted in `/Users/jr/Dev/lex-mex`; copy only these named review inputs and
this updated plan into the integration worktree as needed. Preserve their hashes.
This continuation supersedes the earlier M1 blanket LIC hold and pending-M0 next
action. Full-corpus/experimental-agent/research-feature exclusions still apply.

### R0 — Verify review baseline and preserve release artifacts

Confirm main85c41f389 and integrated worktree identity; hash current reader.js,
reader-core.mjs and publication receipts. Preserve ship-a/b, baseline-unmodified-ui
and release-main-a/b. Old ready publication0b797aa6 is review baseline, not final
approved deploy. No cleanup. Read explicit source-audit evidence; verify the
511 LIC IDs/bodies match accepted main and the accepted DB if using its candidate.
No new source fetch, corpus parse, temporal derivation or legal-review write.

### R1 — Fix event-driven active selection

First reproduce the reviewer scenario in a real local browser on current shipped
bytes: two consecutive articles remain in the observer's upper-half root while
the next heading crosses barHeight()+24. Inspect active label/sidebar, exact copy
body and selected share URL; they must track the article at that line. If the
finding does not reproduce, supply event trace/geometry and explain the code path
that forces an update before dismissing it. Do not weaken the requirement.

Separate observer candidate membership from reading-line position. Recompute on
scroll through one requestAnimationFrame scheduler; schedule a trailing update
when settle/loading suppression ends, without racing explicit navigation. Keep
passive history as replace, deliberate navigation as push, and PDF auto-follow
disabled. Test movement without any IO entry/exit, short adjacent articles, long
article tail, resize, deep-link settle, interrupted settle and quick instrument
switches. A helper-only test is insufficient: include an event-level browser
regression and final package browser evidence. Use the smallest reliable repair.

### R2 — Advance the public LIC candidate safely

Source fidelity is technically cleared for the exact current-main differences:
15 whitespace-only,8 misplaced headings,112 Quáter split into3 source-printed
articles,108 Bis source-note completion and Vigésimo signature exclusion. These
were accepted canonical repairs from9441b7976. Do not solicit another JRH ruling
for these same deterministic facts or infer approval of legal validity.

Build/verify the preferred four-instrument current-main release, total870:
LIC511, IFPE71, ITF132, LRITF156. Existing release-main-a/b can be reused only
with file/digest/input parity. All shared temporal/review/source fields must
remain exactly as in the audited candidate. New112 articles retain their
recorded machine classifications and unknown effective dates. No invented
commencement, reform date, or reviewed status.

Validate all selected public reference spans and resolved targets; outside-
selection targets must remain qualified. Verify direct URLs to112 Quintus,
Sextus and Septimus find those articles, not the first article fallback. Verify
112 Quáter no longer duplicates their bodies,108 Bis includes the exact source
note tail, and Vigésimo excludes signatures from article copy. Confirm LRITF
public record bytes remain identical to the retained release. Keep a retained-
content package as fallback; never overwrite baseline releases. Update the
baseline comparison and publication proposal to reflect technical clearance.

If any difference falls outside the evidence list, hold only that content update
and continue R1/UI over the retained release. Document the new discrepancy
precisely; do not invent legal approval to meet the goal.

### R3 — Final package and integrated verification

Build the preferred package twice from the fixed UI and verified current-main
release. Verify all bytes, release/publication IDs, public allowlists and source
PDF hashes. Also create or retain a verified retained-content fallback using
fixed UI if practical; original deployed publication remains rollback target.
No expected publication ID should be hard-coded from the obsolete ship-a.

Run node --test reader/, updated event/browser regression, package/stage tamper
and private-field exclusions, local staging/rollback rehearsal and final browser
checks. Test each new LIC direct link, exact article copy, index/read-line identity,
references, release query, back/forward, search, narrow layouts and PDF offer.
Measure representative LIC loads; report slow outliers without declaring them
solved by clean-load averages. Investigate a consistent foreground stall; do
not make a flaky timer assertion the only acceptance gate.

Native Rust code at the original receipt passed424tests; Codex did not rerun it.
If Rust/schema/Cargo inputs remain byte-identical to that tested state, preserve
those receipts and explicitly bind hashes instead of claiming a new run. If any
change, run required fmt/strictClippy/lockedworkspace tests and isolated LRITF/
IFPE validators again; record real counts. No canonical/queue/output drift.
Public schemas/Cargo changes can affect a future DB input/build identity: run
workspace DB tests as part of changed native gates, but do not rebuild accepted
internal DB releases solely to publish this static four-instrument UI. Any new
DB release claim requires a separate actual pinned build/verify/golden gate.

### R4 — Fresh review and corrected publication packet

Consult Opus5.5 in a fresh bounded context using diff, acceptance criteria,
source audit and actual final receipts, not only the builder's transcript.
Ask it to challenge R1 event delivery, copy/URL correctness, LIC/source parity,
public data exclusion, actual shipped-byte tests and rollback. Sonnet integrates
findings and reruns affected checks. Request Codex final re-review from this
new packet; neither advisor can grant operator publication authority.

Update implementation/evidence documents, plan and continuity. Include an exact
source change manifest excluding unrelated work. The old proposal changes only
publication_id; that is no longer correct for the preferred870-provision release.
Compute the complete pin delta from actual final files: publication_id,
release_id, corpus_commit, manifest_sha256 and any other differing host fields.
For retained-content fallback only publication_id may change if verified.

Keep the backed-up deployed pin in the non-served ignored backup directory along
with both host destination backups. Rollback must restore the original complete
pin and deployed package, including latest.json; do not only revert publication_id
when the selected release changed. Rehearse locally. Host writes/deploy, commits,
merge and push still require separate operator authorization after the concrete
packet is ready. No publication occurs just because the technical hold is cleared.

## R5 — Codex final review follow-up, 2026-10-22

R0–R4 are committed on Reader branch d87db82c0. This supersedes the resume-R0
next action. Codex independently verified final packages,22Node tests, preferred
28browser checks and content/pin parity. Fallback failed the same narrow390px
LIC193 deep link twice: it landed on191. See
`docs/four-instrument-reader-r0-r4-codex-review-2026-10-22.md` and evidence in
the main workspace. The same UI serves legacy3c04 pinned URLs after preferred
staging, so the backward-compatible landing must work. Database/LIC source
fidelity and IFPE46→ITF9 correction remain accepted.

1. Preserve/freeze current source and final packages; reproduce390px LIC193
   fallback failure. Capture requested ID, selection/URL, target offset, loading/
   settle/lock state and font readiness across navigation. Default anchoring
   runs failed twice; do not dismiss as the disclosed no-anchoring limitation.
2. Diagnose and repair explicit navigation stability around deferred layout and
   font changes. External font swap is a hypothesis, not established cause.
   Keep an explicit target stable until genuine user navigation/scroll intent;
   preserve manual scrolling and passive no-push history. Do not simply weaken
   the assertion or add a long test sleep. Add targeted event-level coverage
   and retain evidence that the old final bytes fail. Do not change canonical
   LIC text/temporal state or internal DB to fix UI geometry.
3. Build both preferred870 and fallback867 package pairs with fixed UI; run
  22Node tests and the full28/25browser scenarios on final bytes. Include a
   new narrow193 trace and legacy-release query test under a preferred-staged
   local directory containing both releases. Verify copy/share/index/label and
   requested target, not merely internal agreement on the wrong article. Rerun
   affected mutation proof if scheduler/lock behavior changes; broader mutation
   repetition is unnecessary without an affected assumption.
4. Reverify publication/manifest hashes, exact pairs, sourceUI bindings, public
   content/source/reference parity, full pin delta and local rollback. Keep all
   old failing receipts and immutable output roots. Native424 receipts may be
   reused only with exact hash binding if native/schema/Cargo inputs unchanged.
5. Fresh bounded advisor/code review and updated receipt/evidence/plan. State
   new preferred/fallback IDs from actual files; supersede f5a1937a/709b09c4 as
   appropriate. No commits/merge/push/host staging/deploy without authorization
   current to that action; verify earlier local-commit authorization scope.

Exactly one current next action: Claude diagnoses/fixes R5 in the existing
reader-four workspace and returns a frozen package/evidence for re-review.
Do not restart the DB, LIC ingestion or original design implementation.

### R5 outcome — 2026-10-22 (Claude Opus 5.5; routing deviation from Sonnet recorded)

The 390px LIC 193 fallback defect reproduced. A trace (`.work/evidence/r5/trace-fallback-r4.json`) shows two causes:
deferred-layout oscillation with a bounded settle, and a late font swap. Fonts are one cause, not the only one. The
repair is in `reader/reader.js` only: an explicit navigation holds its target until reader scroll intent or the next
navigation, with bounded realigning. Harness scenarios 16–26 cover target stability, intent, Copiar enlace, index,
PageDown, touch, Volver, resize, first-provision entry, the resize storm, and legacy 3c04 links on a staged directory
that holds both releases.

Final packages:
- preferred `fbcb4e98b64f5dd00c82a387ce04a35c93681f9d1b4d52ce1181ca55d22e7ea3` over 9626 (870 provisions);
- fallback `4a3b2e2fa13871f7dbe0903e317b091cfc52091ad376c62f01f3b9b630dc742c` over 3c04.

They supersede f5a1937a/709b09c4. Only reader.js and publication.json differ from R4.

Results on the final harness:

| Run | Result |
| --- | --- |
| Preferred | 56/0, twice |
| Fallback | 53/0 plus 3 N/A, twice |
| Legacy staged | 71/0 |
| No anchoring | 56/0 |
| Old R4 fallback | fails 25 |

Package verification, fail-closed, pin delta and rehearsal all pass. Native inputs are unchanged. Six bounded Opus
review rounds ran (R5–R10). See `docs/four-instrument-reader-r5-receipt-2026-10-22.md` and its evidence JSON.
Nothing is committed for R5, staged to a host, deployed, merged or pushed.

Exactly one current next action: Codex re-reviews the R5 packet. Operator decisions stay separate: local commit
authorization for R5, the choice of option, host staging, the pin change, deployment, and merge/push.
