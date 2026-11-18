# R7 integration and work-cache verification — 2026-10-24

R7 production integration is complete. Main at 77aad2960 contains accepted R7
0661d5ef0 as an ancestor. A mechanical scoped diff across crates, reader,
scripts, schemas and Cargo manifests/lockfile is empty. Origin main at inspection
was a07a93f31, which already includes the integration; the later local commit is
handoff housekeeping. No merge, rebase, push or deployment was repeated here.

Rebase is possible for an isolated unpublished branch after preserving its dirty
state, but the permanent reader has already been reconciled onto R7. Its old
history is preserved by tag/backup and the query experiment by feat/public-query.
Rewriting these accepted integration commits would add no benefit. Review the
query branch independently before any future integration or publication.

The host deployment is already version 26. Fresh HTTPS checks with a browser
user agent confirmed current release c305db4a and byte-identical publication.json,
reader.js, reader-core.mjs and reader.css against the tested package retained in
.work/reader-r7-review-20261024/publication. Publication ID is
0d142a9a88b049dcf6623e599783a862fdb839c01e2221a56e62997b39b4e3f0.
The initial Python urllib request returned 403; curl with the browser user agent
succeeded. No browser/mutation campaign was repeated. The host HANDOFF records
its successful deployment and deliberate current-only publication policy.

## Cleanup

The earlier 9.1 GB estimate was stale. The three relevant .work roots occupied
35,024,244 KiB before cleanup. Removed only 12 identified Rust target caches,
19,846,656 KiB (20.32 GB; 18.93 GiB), after verifying directory identity, Rust
cache markers and absence of active Rust build processes. The explicit path list
and retained asset hashes are in work-cache-cleanup-2026-10-24.json.

All twelve caches are absent; 3,102 retained file hashes are unchanged. Remaining
allocation: main 5,268,984 KiB; parser-split 9,175,356 KiB; permanent reader
733,248 KiB. Total 15,177,588 KiB (15.54 GB). Preserved source trees, logs,
frozen binaries, input snapshots, derived databases, tested publication and
rollback assets. Reproduction will rebuild the deleted compiler outputs.
No corpus/code changes or unrelated files were staged or committed.

Next: classify the retained older source/evidence directories against durable
receipts before any further deletion; query/windowed-reader work remains separate.
