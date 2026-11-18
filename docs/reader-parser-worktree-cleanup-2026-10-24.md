# Reader/parser worktree and stale artifact cleanup — 2026-10-24

Completed the operator's requested three-checkout layout: `/Users/jr/Dev/lex-mex`, `/Users/jr/Dev/lex-mex-db`, and `/Users/jr/Dev/lex-mex-reader`. Existing unrelated OCR experiment worktrees under Dev/experiments were not removed.

## Why the extra worktrees existed

- `lex-mex-reader-windowed` was a clean reservation on `spike/windowed-reader` at a07a93f31. It was created to satisfy the earlier request to give the future spike its own worktree before retiring four/five. No spike implementation, unique commit or local source edit existed. Removed; branch remains for future use.
- `lex-mex-parser-split` was the temporary parser/R7 integration workspace at0661d5ef0. Its production code and history are merged into main. There were no tracked/untracked source edits. Removed; integration branch remains. It is not required for running the current parser.

The earlier retirement moved all ignored artifacts intact, which preserved about9.1GiB of stale work as well as another2GiB target cache. That was preservation, not disk cleanup. This task replaces those bulk copies with compact evidence and removes the originals.

## Deleted stale data

- `/Users/jr/Dev/.work-backups/lex-mex-reader-retired-2026-10-24/reader-four/ignored/.work`: 9542220 KiB allocated.
- `/Users/jr/Dev/.work-backups/lex-mex-reader-retired-2026-10-24/reader-four/ignored/target`: 2074340 KiB allocated.
- `/Users/jr/Dev/.work-backups/lex-mex-reader-retired-2026-10-24/reader-five/ignored/.work`: 213412 KiB allocated.
- `/Users/jr/Dev/lex-mex-parser-split/.work`: 9175528 KiB allocated.

Total stale work removed: **21,005,500 KiB (~21.51 GB /20.03GiB)**, plus the two obsolete checkout trees. This is removed directory allocation, not a claim about snapshot-aware filesystem free-space accounting.

## Kept only evidence that still matters

The separate P/R independent audit is still pending. Preserve official retained parse inputs, raw logs, result/tree manifests, harness snapshots/tools, frozen comparison binaries, historical package receipts/UI variants and comparison DBs. Those were selected, hashed, deduplicated and gzip-compressed into:

`/Users/jr/Dev/lex-mex/.work/retained-reader-parser-evidence-2026-10-24/`

55,914 original paths map to3,946 unique objects, **551,190,901 compressed bytes (~551MB)** plus the manifest. Every compressed object was decompressed and rehashed before deletion. Duplicate databases and repeated receipts are stored once. The manifest records each original absolute path, exact byte length, SHA-256 and object path. To recover a file, locate its original_path in manifest.json and decompress its referenced objects/<sha256>.gz into a separate temporary location. Do not imply the old absolute paths still exist.

One clean integration clone had a commit not in the canonical object store; imported it locally before removal as `refs/archive/retired-parser-cache/reader-integration-2026-10-24` (f890942001f45ce65017653911903443fe2228b2). Other clone commits were already present. The g4 validation root contained only staged deletions of tracked files, no unique additions/modifications; its needed derived DBs and metadata were retained. Redundant Git clones, generated publication/content copies and Rust targets were discarded.

All10 historical review/evidence documents and the dirty-plan preservation archive remain intact; source branches remain. The active publication package, canonical corpus, main/DB derived databases, actual website files/pin and current Reader package were outside deletion scopes. No code, canonical data, DB runtime artifacts or publication was changed. No tests or deployment were necessary for this filesystem cleanup; validation was clean Git state/ancestry and artifact hash recovery.

Exactly one next action: use the three permanent checkouts; open the preserved spike branch or create a new temporary worktree only when its implementation is actually assigned. Public-query and the separate P/R audit remain pending in their recorded scopes.
