# Disposition of eight untracked files — 2026-10-24

The operator asked for the long-standing untracked files in the main checkout to be
distilled into one note and then deleted. Which facts to keep was decided on the
**advisor tier** (Opus, same model family as the executor, not independent). Codex
reviewed none of this disposition. Each file's SHA-256 is recorded so a copy found
later can be matched. The files were deleted from the working tree after this note
was committed; they were never tracked, so Git cannot restore them.

| Original path | SHA-256 | Kept here |
| --- | --- | --- |
| `docs/reader-r7-final-review-2026-10-24.md` | `813b24710fbe521acaa7edfa4f8e1c84215cec38fde00114c1e24e49679faaf1` | Distilled below |
| `docs/reader-r7-final-review-2026-10-24-evidence.json` | `829a7f04325c0352571b394c70caab1b67de3e2bdc4fc7bfe25fdde2f2e3381b` | Distilled below |
| `docs/four-instrument-reader-continuation-review-2026-10-22.md` | `824eeeda8804871ed1eec1cd85d74c2f9f40e0cb9890ff7bd1659114d2d8853f` | Hash only |
| `docs/four-instrument-reader-continuation-review-2026-10-22-evidence.json` | `353bbb6b825be723e5dfd9a3714951321a1a5f4c755ae9989d30d38832cb927c` | Hash only |
| `docs/claude-reader-handoff-heartbeat.md` | `aeb469933fca0cd9ad31187a028f0d1d918445efdfd9b60130c832ade354f969` | Hash only |
| `docs/grok-db-suggestions.md` | `3ceab7b1cb11280fd8ff62cc25146d0f89eb1b73eea2d1d8d685307eb5e732d8` | One line below |
| `docs/grok_gemini_db_suggestions.md` | `6b0cf9887d3875871388a8b3fa68ccf5ee94728c958d68d56602fb2983845e8f` | One line below |
| `prompts/adr-citation-grammar-and-tickets.md` | `d6012009c801178c4b3f49f830c874f024f1c0f46ef8bca207d3030604d1b592` | Distilled below |

## R7 final review (Codex, 2026-10-24)

Author: Codex, an independent reviewer. The file text below is Codex's, not the
advisor's. Reviewed commit: `0661d5ef0ff61517287c4de0437b90f6bb60ffcc`.
Verdict recorded in the evidence JSON: "Ready for R7 feature-branch push; no blocking
finding". The earlier Libro P3 was closed: Libro precedes Título/Capítulo in reading
and export paths and separates repeated title/chapter index groups in different books.
The two lint-fix changes are equivalent empty-vector assertions in tests.

Independently run by Codex at that commit in a disposable clone:

- `cargo fmt --check` passed.
- Strict workspace/all-target Clippy passed under Rust 1.95 and 1.99.
- `cargo test --workspace`: 430 passed, 0 failed, 0 ignored.
- LRITF and IFPE validators: valid, zero issues (145/11/126 and 59/4/115).
- `node --test reader/`: 25 passed.
- Full preferred browser harness on the rebuilt package: 60 passed, 0 failed.
- Package descriptor id, all 1,790 listed hashes and sizes, and A/B equality verified.
  All 1,782 public-release content rows match the accepted release. Manifest
  differences are only the commit-derived release id, corpus commit and timestamp.

Reviewed preferred publication `0d142a9a88b049dcf6623e599783a862fdb839c01e2221a56e62997b39b4e3f0`,
release `c305db4a3cfb408b76c13431e26beda72d5ce665046c95783fab36507354c969`. The exact
package and hash-bound logs are kept in the ignored directory
`.work/reader-r7-review-20261024/` in this checkout.

Stated limits: only the preferred package got the new full browser run. No new
mutation campaign, no Safari, Firefox or real-device run. Raw modified-click text
charset headers remain a host follow-up. No source date was newly corroborated or
legally reviewed.

**Stale statements in the original, corrected as of this note.** The review said no
main merge had happened and that R7 was pushed to `origin/feat/reader-tweaks-r7`.
Current state, checked from Git on 2026-10-24: R7 `0661d5ef0` is in `main` through
merge `976040a2f`. `origin/feat/reader-tweaks-r7` and the local branch were deleted
today at the operator's instruction (the branch was an ancestor of `origin/main`).
The operator stated that the site was deployed. I did not verify the deployment or
the pin change; the tracked handoff `docs/reader-r7-webpage-handoff-2026-10-24.md`
records the proposed pin and staging inputs.

Referenced by name from another branch: the reconciliation doc
`docs/reader-workspace-reconciliation-2026-10-24.md` (on `f49c07923`) cites
`reader-r7-final-review-2026-10-24.md` as its step-1 source. This note is the
surviving record of that review.

## Superseded interim review, heartbeat prompt

The 2026-10-22 continuation review was an interim snapshot, superseded by the tracked
[R0–R4 review](four-instrument-reader-r0-r4-codex-review-2026-10-22.md). The IFPE
Article 46 retarget (span 2108..2109, now pointing at ITF Article 9) is recorded in
that tracked review at line 84, so nothing further is kept. The heartbeat file was a
ready-to-use prompt for a scheduled monitor that was never scheduled; it is not
evidence.

## Grok and Grok/Gemini database notes (2026-10-17)

Operator-owned suggestion inputs to the SQLite plan. Their census figures (467
instruments, 68,086 provisions, 35,887 edges) are stale against later reparses. The
plan `docs/plans/sqlite-corpus-database.md` on `f49c07923` records the SHA-256 of
`grok_gemini_db_suggestions.md`; the current hash above equals it. The plan records no
hash for `grok-db-suggestions.md`.

## ADR: citation grammar and tickets (held 2026-10-20)

`docs/decisions.md` does not mention this ADR; this section is the only tracked
record of the hold.

- **Status.** Held as of 2026-10-20 and not the next plan. The phases (0 inspect,
  1 tickets only, 2a contract, 2b shadow, 3 cut over one family, 4 heading grammar)
  stay as a record of a reviewed design and **none is authorized**. Baseline when
  written: `main` at `d8ecaeaa0` (current at hold time `db48f899d`).
- **What to do instead.**
  1. Finish the open workstreams (the database branch merging `main`; the reader
     audited against the October 6 source gates). Neither waits on the linker.
  2. Fix a wrong citation by class, as the glossary was fixed: a fixture for the
     sentence, a rule that names the modifier (dated, anaphor, title, quote), and a
     count of the edges that move. No shadow grammar beside the scanner and no
     second writer.
  3. Record a span the rule refuses on purpose in the `unread_fractions` shape. A
     refused span is disclosed, does not reject the instrument, and invents no target.
  4. The structural debt actually open is 492 trailing heading leaks in 48
     instruments (40 without retained source), the DCG parser, and unread glossary
     fractions.
- **Decision as drafted.** Right link or no link; an unlinked citation is allowed.
  A deterministic grammar would decide only the citation phrase, with alias tables,
  `parent_instrument_id`, historical dating, protected-edge guards and
  `_closure-accepted-gaps.json` staying the resolver and gates. After any cutover the
  grammar is the only writer and the marker-window decision is deleted. Tickets are
  the work queue for "add this link" and "correct this link"; they are not
  quarantine, and `link` and `validate` never call a model or create tickets.
- **Never in scope.** No NLP library, parser generator or model on the write path; no
  reparse or edit of committed instruments; no linking of `Ley de Amnistía`
  (`lamn`/`lamni`); no invented provision; no parser loss read as absence (CPF
  `209 Quintus` and the others recovered 2026-10-19 were in the retained source); no
  inferred parent where `parent_instrument_id` is absent; no heading grammar in the
  same change.
- **Stop conditions.** HEAD moved so the ADR no longer matches the linker; an
  unclassifiable shadow diff; an edge that would retarget a `legal_operator_review`
  or protected historical edge; a grammar wanting to link `Ley de Amnistía`, invent a
  provision or infer a parent; a phase 3 that cannot delete the scanner decision
  without a second behavior change.
- **Cost the ADR named.** Until phase 3's sunset the tree carries two decision
  procedures, and that interval is the risk.

The preservation inventory `docs/parser-reader-integration-evidence.json` already
lists this ADR path as preserved; that listing now points at this note.
