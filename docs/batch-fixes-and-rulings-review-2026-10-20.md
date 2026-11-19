# Batch fixes and rulings review — 2026-10-20

Reviewed parser/operator-rulings-2026-10-20 at `1647daaa8`, including
`a749c81f0`, `7ae8329ab`, `4fb56e0fe`, `f91ec1de6` and the held-import record.
Read-only against the active Claude branch; report lives in the DB worktree.
No commits, merges, pushes, canonical writes or legal rulings made here.

## Findings before integration

### P1 — Accepted-gap glossary refreshes are absent from batch publication

The ordinary outside-batch path is fixed: `run_batch_closure` runs global
closure and validates/republishes its accepted `relinked_instruments`.
But `write_closure_outputs` has another mutating path:

1. `ClosureDecision::Rejected` with `glossary_refreshed[index]` calls
   `write_refreshed_glossary` for an instrument with an exactly accepted gap.
2. That writes terms, usages and repository Markdown, but never pushes the slug
   into `relinked`; only `ClosureDecision::Accepted` does.
3. The batch extends `closed` only with `corpus.relinked_instruments`, so an
   outside-batch accepted-gap glossary refresh is omitted from the batch report
   and `republish_exports(..., obsidian_vault)`.

Result: repository Markdown can be current while the provided external Obsidian
vault retains stale glossary links. This is established by the production
branches, not a claim that the present corpus already has stale exports.
The added happy-path and unexpected-rejection tests do not cover this branch.

Bounded repair: expose glossary-refreshed slugs separately (or all changed slugs
with explicit outcome kinds). Publish both accepted relinks and accepted-gap
refreshes. Do not simply add gap slugs to the ordinary validation loop: their
known reference errors remain intentional. Require the exact accepted-gap
contract and fresh glossary validation, preserving reference/audit state, then
republish their exports with that qualified status. Add a three-instrument test:
B is admitted; outside-batch A has an accepted gap and glossary changes; A's
repository and temporary external-vault exports both refresh, references stay
identical, the report names A, and repeated closure is idempotent. An additional
unexpected error must still fail completion.

### P2 — ltfccg Article 16 loses reproducible machine provenance

At `7ae8329ab`, Article 16 is `partially_repealed`, `deterministic_rule`,
confidence 1.0, `machine_accepted`. At `4fb56e0fe`, its text and scoped repeal
record are byte/value-identical; only heading context and machine provenance
change. Basis/confidence become null and review becomes `not_analyzed`.

The compiled public-function reproducer confirms `repair_partial_repeals`
changes zero provisions and `derive_article_temporal_determinations` emits no
Article-16 determination. This follows two independent guards:

- `repair_partial_repeals` repairs only already-machine-accepted whole-repealed
  records, so it does not handle a fresh parser-produced partial repeal.
- Derivation Rule A promotes only `Repealed`; Rule B explicitly skips
  `PartiallyRepealed`.

No evidence shows this was an intended new review policy. Treat it as a
re-derivation gap rather than a changed legal classification. Fix before
claiming the reparse preserves temporal provenance, or record an intentional
reset explicitly. Do not manually set machine acceptance just to restore a
count. A narrow rule may promote the deterministic scoped partial-repeal
reading only when its exact opening-note evidence and stored repeal scope agree;
it must leave remaining-text commencement dates unknown and preserve every
human/model-reviewed determination. Test parse→derive→reparse→derive, ambiguous
commencement, changed scope/text, idempotence and human-review protection.

## Findings closed and direct answers

- **Old P2 whitespace lookback: closed.** Counting runs as one unit resolves the
  normalized/raw mismatch. Tests cover 80 spaces, 40 line breaks, tabs and a
  non-ASCII prefix. Whole-word `contains_phrase` also fixes the title-prefix
  exception problem; a lexical occurrence still is not a legal referent ruling.
- **Old P1 ordinary closure: fixed, accepted-gap publication incomplete above.**
  A rejected batch-owned instrument must not silently pass. Failing at the gap
  check or final validation is appropriate; exact expected-gap handling does not
  turn that instrument into a fully valid one. `relinked` including outside-batch
  instruments is intentional; document its expanded meaning for report consumers.
- **Alias test:** the separately requested alias-catalog-change regression is
  still missing. The code rereads aliases and rebuilds the title index when its
  phrases differ; this is a useful test gap, not independently reproduced stale
  behavior. Add it alongside the accepted-gap publication regression.
- **Quoted senses:** requiring all incisos to start with a quoted name and colon,
  consecutive from a, with at least two, is a reasonable narrow syntactic guard.
  The reviewed source census changes terms only in LAN. It does not establish
  that every future quoted list is a legally operative definition. Add negative
  fixtures for missing/out-of-order/mixed markers and a quoted non-definition
  list when expanding this mechanism.
- **Strategy 6: appropriate.** The general established-pass interpretation and
  term output change; bumping its strategy and rebinding profiles/schema makes
  that change observable. A profile route unable to read LAN's existing quoted
  names is not a substitute for the implemented parser repair.

## Operator rulings and held work

`docs/operator-decisions-2026-10-20.md` records the operator's actual answers:
A1 keeps LGCS's seven terms; A2 keeps the 13 self-title usages dropped; A3 pins
the twelfth LIEPS exclusion; A4 admits LAN XL b's full term/usage. Preserve these
as human decisions. No self-reference binding should be added against A2.

D1 authority-name handling, D2 LAN XVI's two additional readings and D3 ltfccg IX
remain unanswered in the reviewed commit. The two LAN XVI terms are deterministic
parser additions, not recorded operator-approved readings. Keep that distinction
explicit and obtain D2 before describing those readings as ratified. Do not infer
answers from A4 or make fresh legal decisions in this review.

A5/A6 are accepted import rulings but implementation is technically held after
false-match probes; they are not rescinded legal decisions. Preserve the blocks
and avoid admitting known false agency-name/Verificación matches pending D1.
D3 is independent of the partial-repeal provenance defect. B4 authorizes a future
push only after review of the integrated result; this unmerged review does not
satisfy that condition.

## Validation and source checks

`cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --locked --workspace --quiet`: **378 tests pass**, zero failures.
Isolated target `/private/tmp/lex-parser-review-target`; no shared binary writes.
The Article-16 reproducer links the actual compiled lex-parse/lex-core APIs.
No CLI validate/link/derive/export command ran on the shared canonical tree.

Retained PDF and extracted text fingerprints match the manifests for both
ltfccg and fi-dcg-2014; instrument hashes agree too. FI adds only Article 6 Bis
as a new ID, removes no IDs, retains all 43 repealed classifications and yields
25 terms/278 usages. LAN alone changes terms at the quoted-senses commit.
Maaa/mretc import adapters remain unchanged.

Pinned final census: **5,756 terms**, **99,516 usages**, **35,950 references**,
strategy 6 across **468 instruments**. Cross-instrument usages remain **2,545**;
LRITF `autoridad-financiera` remains **88**. Other goldens must still be rerun,
not presumed unchanged. See [evidence](batch-fixes-and-rulings-review-2026-10-20-evidence.json)
for source bindings, fingerprints and counts. This is a pipeline/provenance
review, not fresh legal verification of amendment commencement. The stale
fi-dcg-2014 temporal-analysis request must be regenerated before model use;
its presence as historical input is not a new analysis result.

## Next action

Designated integration owner repairs the two findings and reruns the relevant
regressions. Then commit accepted DB reports and follow DB-first normal merges
(DB → pending batch → operator rulings), preserving seed commit `6505baf57`.
Reconcile strategy-6 fixtures and all changed golden expectations from pinned
source; run integrated native checks and fresh M4-style builds. Keep historical
release `9c444091…` immutable. Only then review the integrated result for the
operator-authorized push. No merge or push was performed here.
