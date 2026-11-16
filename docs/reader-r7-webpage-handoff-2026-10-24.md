# R7 readiness and webpage handoff — 2026-10-24

**Technical preflight passes; no new blocking code issue found.** The previously reported Libro omission is fixed at `a21459733`, including separate book index groups and regression coverage. No additional source repair was needed in this check. Source authority is the clean permanent Reader workspace, `/Users/jr/Dev/lex-mex-reader`, branch `feat/public-reader-r7`, reviewed HEAD `0baaefec9e01c34202647c1495f5299d15af54e7`.

## Actual integration status

R7 includes the accepted parser sync, Libro fix and newer-Clippy test assertion fix. Production-source trees (`reader`, `crates`, `schemas`, `adapters`, `corpus`) at permanent HEAD are byte-identical to reviewed R7 `0661d5ef0ff61517287c4de0437b90f6bb60ffcc` and the main merge rehearsal `976040a2f`. Permanent HEAD adds preserved documents only.

**R7 is now merged into local main at `976040a2f0c54cc22b4e3c58ed600dc101529460`.** On the operator's instruction, main was fast-forwarded from64ac9a6e9 to the reviewed normal merge (parents64ac9a6e9 andd607864d2). Production-source parity against accepted0661 was rechecked after promotion. The older published-reader branch remainsf11beebce; deployment now uses reviewed main/source and the exact package below. Eight colliding untracked main files were preserved under `.work/r7-webpage-readiness/main-untracked-preserved` in the permanent Reader; all16 initial untracked hashes are recorded there. No files were discarded. Do not build from the separate unreviewed `feat/public-query` branch.

## Verification and fixes to readiness records

- Fresh Node suite on the permanent workspace: **25/25 pass**; diff check passes. Initial sandbox loopback EPERM cleared on the exact permitted rerun.
- Reverified accepted package descriptor ID, all1790 listed sizes/hashes, source/UI parity and clean production-source diff against reviewed0661. Rebuilt with current Reader source and the exact reviewed immutable release input; publication ID reproduces exactly.
- Fresh comparison against the webpage's existing F1 release: all1782 listed immutable content rows match. The release manifest changes only corpus_commit, release_id and committed_at. No legal text, PDF, review decision or canonical data repair is being requested.
- Existing exact-source accepted checks are retained and inspected: fmt, strict Clippy,430 Rust tests, both validators0issues and full preferred browser60/0. These native/browser suites were **not rerun in this status check** because production bytes are identical. The fresh Node25 result is separate.
- Local staging rehearsal combines historical3c04, current F1 release9626 and newR7 releasec305. Re-staging F1 and then R7 succeeds while keeping all three releases. This is a non-destructive pointer/UI rollback rehearsal, not a deletion-based exact restoration.
- Fresh isolated webpage preflight copied only tracked host source at `38960ce` into `/tmp/r7-webpage-preflight-20261024`, used the existing node_modules without installation, staged R7+retained releases and generated the proposed pin. Verifier1790 artifacts, `npm test` **7/7**, and separate `npm run build` all pass;32 PDFs prepared. Actual webpage source, pin and generated public tree were not changed. Logs are embedded in companion evidence.

The old discovery-experiment capsule attached to this permanent path is stale after the workspace reconciliation (wrong branch/HEAD and missing experimental source bindings). It is navigation for an older task, not authority for R7. It was preserved rather than silently rebased or closed; use this handoff, current task-plan checkpoint and Git. Do not restore archived experiment files into this publication to satisfy that capsule.

## Exact receiver inputs

Reviewed package: `/Users/jr/Dev/lex-mex-reader/.work/r7-webpage-readiness/publication`.
Proposed pin: `/Users/jr/Dev/lex-mex-reader/.work/r7-webpage-readiness/proposed-pin.json`.

```json
{
  "schema_version": "reader-publication-v1",
  "release_id": "c305db4a3cfb408b76c13431e26beda72d5ce665046c95783fab36507354c969",
  "publication_id": "0d142a9a88b049dcf6623e599783a862fdb839c01e2221a56e62997b39b4e3f0",
  "corpus_commit": "d2ab64d61ecfe103cc2a2980c5d04ed63c20bb7a",
  "manifest_sha256": "e3c8b096b41e5cdb231172def8cc6cd3f7d2b9a84613456d06933b28b969dec9"
}
```

Preserved external review package also exists at `/Users/jr/Dev/lex-mex/.work/reader-r7-review-20261024/publication`. Never re-export merely to rename its corpus_commit to a documentation-only HEAD: that would create another release identity requiring a new comparison.

## Webpage receiver sequence

1. Read the actual webpage `AGENTS.md`, `HANDOFF.md`, pin and Git state in `/Users/jr/Documents/4ll4/Webpage/LEG4L_L4BS`. Its current tracked pin remains F1 publication2df70a8a/release9626. This R7 task did not alter or requery production.
2. When instructed to publish R7, stage the exact package through `reader/stage.mjs` into `/Users/jr/Documents/4ll4/Webpage/Lex-Mex/public` and then the webpage `public/lex-mex`; preserve both3c04 and9626. Apply the exact proposed pin above. All four identity fields change from F1, not just publication_id.
3. Re-run host verifier, `npm test` and `npm run build` in that actual updated checkout. The isolated preflight proves compatibility, not that the actual host is already staged. Review the pin diff and commit only authorized host paths.
4. Use the registered owner account and existing Sites project `appgprj_6a7e46b3af708191a764192048995087`; preserve its audience/project. Follow the current Sites skill/source helper to open, push, build/archive and save/deploy the exact source commit. Do not create a replacement on access failure. The operator authorized this local main merge and the handoff for deployment from the other account; no canonical GitHub push was performed here.
5. Check live receipt/publication0d142a9a, releasec305, canonical Reader navigation/copy,112 Quintus, PDF full/range behavior, and pinned links to both9626 and3c04. Check current favicon equality with the main site. Keep prior version24 for rollback.

Raw .txt/.md UTF-8 charset handling remains the disclosed host follow-up; R7's plain-click blob path is already independently tested. No new claim about raw live headers, Safari/Firefox, real devices or legal effect dates is made here.

Exactly one next action: the owner-account webpage session stages the exact R7 package, updates the pin, verifies/builds, and publishes the existing Site using the sequence above. Main integration is complete locally. No actual host staging, pin edits, deployment or canonical push was performed here; the handoff update is committed separately on main.
