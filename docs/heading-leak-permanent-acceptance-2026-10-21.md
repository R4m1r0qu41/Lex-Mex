# Heading-leak permanent commit acceptance — 2026-10-21

Accepted: `5eca03bc5085de423ae23b1f1519712604bd28e2`, the permanent repair
commit; both permanent DB builds; the 39-instrument reparse scope and all
recorded holds. This resolves the technical questions in both incoming
October21 heading-leak handoffs. No merge or push is performed or authorized
by this acceptance. The separate workspace/parser consolidation audit remains
an uncommitted proposal, not part of the accepted repair commit.

## Evidence and limits

Mechanical read-only review confirms seven code/seed/schema/fixture bindings
match candidate46be511c; actual permanent databases have identical raw SHA
`5da6e800e4d3b362411c561d9ae67fbfe012abe3164fce30bbb8a21c9e249bf2`
and size482,025,472 bytes. Manifests are identical. Codex reran golden queries:
21/21 pass on each. `queries::golden` compares each complete row set using
`ensure!`; mismatch returns Err, and the CLI uses `?`, so the printed names
are successful checks, not merely enumerated fixture names.

Both Claude independent verifier receipts pin corpus/head5eca03bc5, report
zero stale inputs, and match the independently rehashed files. Full verifier
was not rerun by Codex this turn; complete receipts are preserved in
[acceptance evidence](heading-leak-permanent-acceptance-2026-10-21-evidence.json),
rather than depending on session-scoped scratch logs. Earlier419-test native
checks, source/temporal audit and private validators remain valid evidence;
no code has changed and they were not repeated in this docs-only acceptance.

The candidate/permanent manifest input inventories are identical. Specification
changes are exactly corpus_commit, builder_commit and source_commit_timestamp;
the release stores this provenance in SQLite. Thus the physical file hash
changes, while logical digest remains
`8cc3736018bffae94f303d9cbfa378170c9ff0e2842ce708431f08d528b60359`.
This is expected provenance change, not evidence of changed corpus content.

## Scope and holds verdict

The39 reparses are accepted: retained official source/extracted hashes match,
no removed IDs, twelve intended recovered IDs, and no lawyer-reviewed changes.
The seven commencement dates are accepted as existing deterministic machine
outputs after removal of misattributed heading notes, not new legal rulings.
The original353-source census remains Claude evidence; Codex independently
checked39 source pairs and the three targeted E2E paths, not all353 parses.

LRITF remains untouched pending JRH. cub-dcg-2005, reg-laero and
socap-sofipo-dcg-2006 newer editions need separate acquisition/ingestion review.
lgoaac/oaac/scap no-gain reparses remain held to avoid unrelated parser drift.
Remaining DCG/LGOAAC/LAN/LFPED heading forms require source-backed narrow rules;
none is bulk admitted by this verdict. No further legal call is needed to
accept this technical branch; holds are not legal approvals.

Census:468 instruments,68,341 provisions,35,930 references,5,766 terms,
97,672 usages,14 active debt rows and3 preserved retirements. Standards and
SCJN scope is unchanged. Source ancestry cd6cd2245 must survive a normal merge.

## Documentation and next action

The prior report's candidate validation remains historical evidence. Its
current-state notice and task plan now point here: commit/rebuild are complete,
three stale paths are cleared. These documentation corrections should travel
with the eventual merge; a separate docs commit is appropriate when authorized.
Unrelated operator files and DB-worktree drafts are preserved. No cleanup.

Next: operator-authorized docs commit and reviewed normal merge; push requires
explicit authorization. Do not squash away resolving ancestor cd6cd2245.
