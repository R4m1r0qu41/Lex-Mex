# Post-split bounded independent re-review — 2026-10-27

**Verdict: accept fixes R-001 through R-004. No blocking finding remains in this
bounded scope.** One nonblocking documentation correction remains below. This
acceptance permits operator integration review; it does not authorize or perform
merge, push, deployment, or publication.

Reviewed parser tip `b38d1f33d68cfd432fc2d5b73a28390ef355878a`, query tip
`6155782066d1f660c825a152cb83bacfe352b5f4`. Database corpus/builder identity:
`3cbba683e7cea174600ee197f813d6fa15675083`. Incoming request:
`2026-10-26-lex-mex-post-split-bounded-re-review.md`.

The [previous independent review](post-split-independent-review-2026-10-26.md)
remains the historical corpus/temporal acceptance evidence. This re-review does
not repeat its whole campaign or approve additional legal conclusions.

## Closure of the four findings

| Finding / fix | Independent result |
| --- | --- |
| R-001 / `cf43ae1b4` | Closed. The continuation exception now matches only four explicit mixed-case fragment spellings. A fresh 44-case public-parser matrix covers 11 other header spellings × ordinary/omnibus context × punctuated/unpunctuated preceding body. Every genuine tested header closes the section and stays out of canonical body text. The original regression and LRASCAP positive tests pass in the full suite. |
| R-002 / `b3e562d88` | Closed. The real LFT transitory-10 references route returns 200. Target `urn:lex-mx:federal:statute:lft:transitory:9º` is preserved exactly, with `resolved`, source span/rule, and `target: null`. This means no public route, not unresolved canonical identity. Mixed routable/unroutable pages and unknown-instrument rejection are tested; the Rust contract accepts the representation. |
| R-003 / `b3e562d88` | Closed. The original unmodified cursor walk with limit 7 returns exactly 1,000 results over 143 pages. Page size is bounded by remaining depth; tests cover 3/7/9, changing limits, last offsets, ordering and validated depth options. |
| R-004 / `b3e562d88` | Closed. Unknown-instrument searches now validate pagination before returning empty results. The original JSON-null cursor gives 400. Other scalar/array/mismatched cursors and invalid limits are covered. |

The 44-case matrix deliberately excludes the four permitted mixed-case sentence
fragments from its “must close” set. Their continuation behavior is intentional
and tested positively; the source-backed LRASCAP full reparse below preserves
that behavior. Future expansion of the whitelist still needs source fixtures.

## Corpus and disclosure checks

- `git diff bcd17a4e4 b38d1f33d -- corpus` is empty. The accepted corpus and all
  166 temporal results have not changed in this repair.
- Debt seed and golden definitions also have no diff. Provenance ancestry is
  preserved; no rebase or squash occurred.
- A fresh LRASCAP parse and validation from hash-matching retained PDF/text both
  exit 0; every existing instrument output file remains byte-identical, including
  the layer-qualified IDs and 21 transitories.
- The disclosure now correctly lists 341 amended-article date clearances:
  CUB 281, CUCB 35, OAAC 22, REG-LAERO 3. These are deterministic output changes,
  not legal rulings or overwrites of audited decisions.
- `615578206` discloses that rare words lack static term pages and directs readers
  to search. This addresses the requested disclosure; it does not establish
  federal-scale static-generation performance or change the selected budget.

## Nonblocking R-005 — correct the logical-digest explanation

At `b38d1f33d`, the repair record says the logical digest differs “only because
the builder identity is part of the release provenance.” That causal explanation
is incorrect. `lex-db/src/logical.rs` excludes `corpus_release`; changing builder
identity alone does not change this digest.

I independently reproduced the new digest:
`e3f0d112f2206cbea3d9b144c3ae1a31c3021f44700f6f37eac0371d23a03b3a`.
In memory, replacing **only** the `input_files` row's `docs/decisions.md` bytes
and SHA with the previous `bcd17a4e4` version restores the earlier digest exactly:
`42cfc2e3b8d55c2729c161d45a415954a280029b41250ce08355a0276cb5e1ce`.
No database file was edited.

Thus the logical digest change is fully accounted for by changed retained
**decision-document evidence**, despite unchanged canonical corpus bytes.
Builder/release identity separately explains release/physical-file provenance
changes. Correct this sentence in the record and any repeated explanation;
no parser, corpus, database-schema or digest-policy change is needed. This is
not a new data-integrity blocker.

## Validation actually run

All commands used disposable pinned snapshots under
`/private/tmp/lex-bounded-rereview-20261027/`, not the active writer worktrees.
Command exits were captured directly.

- Repair snapshot: `cargo fmt --check`, strict workspace/all-target Clippy,
  `cargo test --workspace`: **442 passed, 0 failed, 0 ignored**.
- Both locked validators pass: LRITF 145 articles / 11 transitories / 126
  references; IFPE 59 / 4 / 115, each with zero issues.
- Query snapshot: formatting and `cargo test --locked -p lex-core public_query`:
  **5 passed**. Full Node suite on the new database: **71 passed, 0 skipped**.
- Independent original adversarial probes now pass, plus the 44-case parser
  matrix and byte-identical LRASCAP reparse.
- New release `d82f53c073a53960d18f56c323d720ebce84dddcc2f70da949fa0ba25687539d`:
  independent `db verify` and **21 goldens pass**. Both producer database files
  rehash to `17ca152be7300a54f1b11853ee01f6d1cbe6081b558523d75e444b5ea5b6a6bf`;
  both manifests rehash to `482a92c9d4aebaa1176d315909869b43c1d0b56b5f722278a5c1a2ba4d1ecdeb`.
  Verification/goldens ran on one of these identical copies, not redundantly twice.
- The logical-digest counterfactual above was independently computed, including
  reproducing the current manifest digest before restoring the old evidence row.

No fresh database projection was built by this re-review; the producer pair was
independently verified. The 102-instrument campaign, full query workspace test
suite, strict 1.99 pass, browser DOM/mutation campaigns, federal static generation,
and common-token benchmarks were not rerun here. Earlier independent evidence
and this narrow diff bound their relevance; do not claim new passes for them.

The initial mechanical command used an incorrect annotated snapshot path and
was stopped after a read-only fmt invocation on main. Corrected commands used
explicit disposable working directories; no source file was changed.

## Next action and preserved state

Correct R-005 in the documentation, then obtain operator authorization for
normal-merge integration of parser and query branches, preserving retirement
commit ancestry. Validate the actual merged result before publication; this
review does not substitute for an integration gate or a deployed query service.
SOCAP and the separately queued source/temporal work remain held as documented.

Only this report and its [machine evidence](post-split-bounded-re-review-2026-10-27-evidence.json)
are added on main. Existing unrelated plans, reports and DuckDB drafts are
preserved. No commit, merge, push or deployment performed.
