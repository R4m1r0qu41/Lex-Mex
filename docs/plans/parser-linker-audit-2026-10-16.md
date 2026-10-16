# Parser and linker audit — 2026-10-16

## Scope and baseline

Operator requests a parser/linker check before the next commit. Initial main
HEAD is `7326c41aa`; worktree is clean. Earlier admission and subsequent
successor-clause changes are already committed. Audit current repository truth,
not the older active-run capsule. The operator subsequently authorized fixing the six defects. No new commit or
push is part of that repair instruction.

Review current extraction, target resolution, source partitions, persisted human
decisions, and export behavior. Script corpus JSON as inputs; inspect individual
source spans only for a concrete finding. Preserve canonical data and audited
decisions. Do not infer legal effects or commencement dates.

## Execution and verification

1. Independent read-only Sol linker and parser reviews; parent owns integration.
2. Mechanical native gate: fmt, clippy, workspace tests, LRITF and IFPE validators.
3. Script graph span/target/ownership checks across committed ordinary instruments.
4. Isolated replay of affected sources where retained bytes are available; compare
   text, boundaries, graph and legal-review metadata without writing main corpus.
5. Publish reproducible findings and remaining limits in
   `docs/parser-linker-audit-2026-10-16.md` and companion evidence JSON. Material
   defects require a narrow regression and validation before any proposed fix.

Recovery: stop on changed HEAD or another writer. Preserve existing canonical
data and put uncertain ownership/legal questions in the report for JRH.

## Current checkpoint

Audit completed at unchanged `7326c41aa`: native gate passes 266 tests; graph
integrity passes for 467 instruments and 27,429 references; all 44 stages across
11 isolated source replays pass with identical canonical JSON and Markdown.
Independent Sol reviews reproduced six defects, including two high-priority
trust-boundary failures (human-decision preservation and successor-clause drift).
LGDNNA 135 Ter's missing self-target 157 is a separate legal-review exception.
Only audit documentation changed in main; no code/data repair, stage or commit.

Exactly one next action: repair the documented defects with focused regressions
and fail-before-write checks, preserving the LGDNNA pointer for JRH review.

## Repair checkpoint — 2026-10-16

User authorized fixes after the audit. Parent integrated three linker fixes and
an annex fix prepared by GPT-6.1-Sol in separate archive snapshots; parent added
human-decision and successor-clause guards across CLI and bundle publication.
No schema or canonical corpus migration is required. Explicit missing self
citations now remain visible to validation; the LGDNNA pointer still needs JRH.

Final native gate passes 278 tests, formatting, clippy, LRITF and IFPE. All 44
stages in eleven isolated source replays pass with identical canonical JSON,
source manifests and Markdown. Original adversarial cases now reject stale
human/successor evidence before writes, reject wrong target ownership, retain
explicit missing self-citations, omit unsupported external relatives and preserve
opaque annex text. Independent Sol reviews accept integration. Final evidence:
`docs/parser-linker-repair-2026-10-16-evidence.json`.

Code and fixture repairs are complete; no schema or corpus migration, staging,
commit or push. The corpus remains at 467 instruments with unchanged bytes.
Exactly one next action: obtain JRH's disposition of LGDNNA 135 Ter's citation to
missing Article 157 before a corpus-wide relink. A new code commit is separate
from that legal disposition and has not been performed.
