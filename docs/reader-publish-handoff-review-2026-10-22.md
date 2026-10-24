# Reader publish handoff review — 2026-10-22

Reviewed Claude's `2026-10-21-lex-mex-reader-publish-handoff.md` against main
85c41f389 and bounded Reader source/plan inspection. Reader remains at
f5330c3e9 with43 dirty entries. No Reader, host, design or corpus edits,
publication, merge, push or deployment performed by this review.

## Verdict

Accept the handoff as a navigation and scope packet, not an implementation-ready
or publish-ready release. Four-instrument republish, new design implementation
and full-corpus DB-backed site are separate deliverables. Operator request this
turn is review; it does not choose or authorize these deliverables.

The DB prerequisite is stale: main85c41f389 is merged and remotely verified,
with the accepted permanent5eca release. DB acceptance is no longer a blocker.
Public projection, bounded site queries and privacy exclusion remain unbuilt
integration boundaries; do not serve the internal SQLite release directly.
The permanent reader workspace is the online presentation workspace, not stale
work to discard. Do not merge its dirty branch wholesale into main.

## Findings

1. **Build identity is unresolved.** Four-instrument package publication differs
   from the host pin; whether its UI matches Reader HEAD is unverified. Select
   a source commit and compare public artifacts before treating it as a release.
   Recommend current accepted main85c41f389 as comparison baseline, preserving
   canonical LRITF decisions and requiring explicit review of any public text,
   temporal or review-state changes. Updating presentation does not require
   re-deriving canonical facts.
2. **Deployment strategy conflicts.** package.mjs still imports/generates static
   agent pages while the DB-backed agent plan supersedes that deployment model.
   This is not merely stale prose: the packager's behavior selects what ships.
   Recommend human-reader/design work first, leaving experimental agent routes
   out of the release until their strategy is explicitly settled. This remains
   a recommendation, not an operator keep/drop ruling.
3. **Pending work is not a reproducible build.**43 dirty entries include unrelated
   deletions and overlapping documents. Preserve them; isolate scoped reviewed
   changes into an integration branch/worktree with an accepted-main base. Do
   not reset or clean the permanent workspace. Reader-specific instructions
   apply there; provider-owned configuration is not incidental product work.
4. **Design approval is limited.** Revision02 is reported approved; research-tools
   board246:438 is reported unreviewed. This review did not inspect Figma or
   validate its current approval state. Implementing the approved revision is
   distinct from shipping hover/session/reform tools. PDF-follow feasibility
   and exact source anchors remain prerequisites for that feature.
5. **Query defects remain reported, not independently reproduced here.** Unknown
   slug200, null cursor503, HTTP source URLs and missing CR escaping should be
   repaired with regressions before any route using those paths ships. Keeping
   a preview out of scope does not justify deploying its defective static
   agent-page counterpart. Test the actual packaged surface.
6. **Old checks do not establish current integration.** Reported29Node/228Rust
   tests belong to the older Reader tree, not419-test main. Re-run relevant
   tests after integrating scoped Reader changes with current main, verify
   publication/private-data boundaries and preserve host rollback pin. This
   audit ran no tests and did not inspect the live host.

## Proposed execution order

1. Choose deliverable: four-instrument refresh, approved-design implementation,
   or full-corpus DB site. Confirm static agent-page disposition and whether
   research tools are approved. The handoff currently authorizes packet only.
2. Create isolated integration state from accepted main; inventory only needed
   Reader changes and preserve all unrelated permanent-workspace work.
3. Compare the four-instrument public projection with deployed retained pin,
   including text/source/reference/temporal/review differences. Report before
   legal-content replacement; no inferred JRH approval.
4. Implement selected UI and its actual data path, repair relevant defects,
   run Reader/native/publication checks and produce a local reviewable package.
5. Obtain concrete host-write/deploy authorization at the final publish step,
   then verify live routes/headers and rollback. Do not deploy from this packet.

Next action: operator selects the deliverable and resolves agent-page/research
scope. Recommendation: approved-design implementation over the four-instrument
public release first; design the full-corpus public query integration separately.
This does not require changing the permanent workspace into a standalone clone.
