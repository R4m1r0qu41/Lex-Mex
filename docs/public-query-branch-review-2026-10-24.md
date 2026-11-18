# feat/public-query review — 2026-10-24

**Still pending; preserve the branch and keep it out of publication.** Reviewed exact tip `d2170f9fd7ff8ea179512e496f22fffa2c9ddc1c`. This is the preserved pre-R7 experiment from `f5330c3e9`, not the current R7 Reader. No implementation changes were made. Its29 Node tests pass, but targeted probes below fail the intended client-error/text contract.

## Confirmed findings

1. **P1 — HTML exact-text/hash mismatch** (`reader/agent-pages.mjs:74`). Article text is escaped into a bare pre element. Real Chrome parsing strips the first newline and normalizes CRLF. Synthetic input `\námbito\r\nsegunda línea` becomes `ámbito\nsegunda línea`, while the displayed SHA-256 still describes the original bytes. The canonical/raw artifact is unchanged, but an agent reading the advertised exact HTML text receives a different string. Preserve leading newline/CR deterministically and verify parsed DOM text against the canonical string; the HTTP query presentation already has related tests/workarounds.
2. **P2 — null cursor is reported as service failure** (`reader/query-memory.mjs:8`). Base64url of JSON null decodes successfully, then decoded.scope throws outside the decoding catch. Public catalog responds503/unavailable rather than400/bad_request. Validate the decoded object shape before property access and cover null/scalar cursors.
3. **P2 — search can normalize to empty** (`reader/query-http.mjs`, `reader/query-memory.mjs:16–18`). A combining acute accent U+0301 passes raw nonempty validation but normalizes to empty. Probe returns200 with all four synthetic provisions. Reject normalized-empty input rather than ranking an unintended broad query.
4. **Policy mismatch — HTTP official source links accepted by static pages** (`reader/agent-pages.mjs:12`). The synthetic agent page accepts http://example.invalid/source.pdf, while query-contract.checkSource rejects non-HTTPS URLs. Decide/enforce the common public boundary; this review does not infer that every committed official source URL is HTTPS.

## Still outstanding from source inspection

- Static terms create one file per distinct normalized word with no corpus-wide file/output budget. The20-candidate cap limits candidates per term, not the number of term pages. This was a small-corpus experiment; no federal-scale readiness is established.
- The in-memory adapter deliberately serves bounded local datasets, not a federal database. It normalizes full text repeatedly inside sort ranking and builds every matching excerpt before pagination. Benchmark/refactor before treating it as a scalable backend.
- Rust validate_response reparses and recompiles the same schema per response (`crates/lex-core/src/public_query.rs:132–138`); caching is a performance follow-up, not a correctness failure proved here.
- There is no completed database-backed serving/deployment path in this branch. Its retained bridge is a local preview. Earlier planning-era DB counts/status notes must not substitute for the current DB owner's actual acceptance evidence.

## Integration and scope ruling

The old packager always adds agent HTML and replaces llms.txt; the stager accepts agent HTML. Those differ intentionally from the currently approved R7 human publication boundary. Do not build the next public package from this branch or directly import its old Reader UI. Rebase/reimplement only the agreed query work on current main, preserve R7 and exact canonical text, fix the confirmed findings, and independently review before any agent-surface publication. No new federal-scale run or site was authorized here.

The old appendix also alleged that an arbitrary agents/<64hex>/ prefix was not tied to the release. **Do not interpret that as requiring surface ID to equal release ID:** this generator explicitly uses a separate content-derived surface ID. A stronger stager binding audit may be useful, but that wording alone is not a reproduced defect in this review. Likewise no arbitrary new timing threshold is inferred for the schema/performance notes.

Evidence:29/29 existing Node tests via mechanical check, plus parent probes of the HTTP adapter and actual Chrome DOM. Test execution used an exported branch snapshot in /tmp, creating no additional persistent worktree. Synthetic probe inputs are not legal corpus data. No Rust tests, full DB build, branch edits, commits to feat/public-query, merge or deployment were performed.

Exactly one next action: repair/rebase the isolated public-query work on current main with the confirmed contract regressions, then obtain independent acceptance before integration or publication. This is a pending task; the branch was not retired.
