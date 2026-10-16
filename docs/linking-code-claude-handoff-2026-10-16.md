---
artifact_type: provider-task-handoff
status: pending
project: lex-mex
source_harness: codex
target_harness: claude
created: 2026-10-16
---

# Claude handoff — linking-code audit

Repository: `/Users/jr/Dev/lex-mex`; branch `main`.
Verified code baseline: `667e33a2c99d7a3c4a7e927b2efd8c82222f8669`.
Generated: `2026-10-16T18:18:51.835731+00:00`; expiry: `2026-10-17T18:18:51.835731+00:00` or earlier if
HEAD/source hashes drift. Initial worktree was clean; this audit adds only the
report, evidence, plan, handoff and plan-index entry. Verify current Git state
before taking over; preserve any subsequent operator review edits.

## Task and findings

The operator requested a linking-code audit and findings handed to Claude while
reviewing held instruments. Codex performed the audit, not another repair pass.
The six earlier fixes are already committed in `078359f72`. Seven **additional**
findings are documented in `docs/linking-code-audit-2026-10-16.md`:

| ID | Priority | Finding | Evidence status |
|---|---|---|---|
| L1 | P2 | Paragraph numerals become extra article links; LGPP95 has false Article2 edge | Actual stored corpus |
| L2 | P2 | Repeated fractions generate duplicate anchors; LFTR180Bis cannot target CPEUM20/B/I uniquely | Actual Obsidian export |
| L3 | P2 | Inline definitions receive links to anchors that do not exist; LFCSQ Desvío/Polígono | Actual Obsidian export |
| L4 | P2 | Independent semicolon clause retargets earlier direct citation | Synthetic reproduction |
| L5 | P2 | Configured aliases match inside longer alphabetic words | Synthetic reproduction |
| L6 | P2 | Display-acronym collision produces wrong local Markdown destination | Synthetic metadata reproduction; current acronyms unique |
| L7 | P3 | Parenthesized compound qualifiers lose fraction/inciso scope | Synthetic reproduction |

**One next action:** Claude independently reproduce L1–L3 and prepare narrow
corrective patches/regressions; then address the bounded L4–L7 cases. The audit
report and evidence contain precise source locations, controls, actual outcomes
and proposed remedies. Do not mistake metadata inventory candidates for fully
verified rendered failures.

## Acceptance checks for proposed repairs

- L1: LGPP95 produces Article10 with paragraph2/incisoc; paragraph2 is not
  Article2. Numeric/word ordinals and genuine article lists remain supported.
- L2/L3: every emitted fragment link has exactly one actually emitted target.
  Use a shared anchor registry; fall back to whole-article/whole-note links when
  scope or an anchor is unavailable. Do not guess which repeated fraction is
  intended. Section-aware modeling requires coordinated trusted-boundary changes.
- L4/L5: later independent clauses and partial-word aliases cannot retarget a
  citation; preserve article lists, punctuation and longest-title precedence.
- L6: canonical instrument identity decides local versus cross-instrument paths.
- L7: retain supported qualifier hierarchy and exact source character spans.
- Preserve earlier safeguards: no clearing audited human decisions, no stale
  successor-clause publication, no fabricated targets or altered source text.
- For implementation changes use repository-required fmt/clippy/workspace/
  LRITF/IFPE checks and affected source/export stages, plus focused regressions.

## Validation and exclusions

Current targeted baseline: 206 lex-parse and10 lex-export tests passed, zero
failures. Thirteen private extraction cases, actual CPEUM/LFTR/LFCSQ Obsidian
exports and a synthetic Markdown collision reproduced findings. No full native
gate or corpus-wide relink was run during this audit. Code/corpus unchanged;
no staging, commit, push or Claude invocation.

The 21 held instruments and operator legal decisions are a separate workstream.
Do not edit `docs/held-instruments-review-2026-10-16.md`, infer JRH approval,
change source citations, or bulk-regenerate canonical data as part of reproducing
these findings. The pending Codex request about Claude's relative-citation and
closure integration was reviewed; no new concrete regression there was found
in this bounded pass. L4 is the separate direct-citation path.

## Bounded source manifest

Read instructions, report/evidence and plan first, then navigate named code
locations. Digests below bind the exact reviewed files (SHA-256). If they drift,
refresh the handoff rather than treating old findings as current verification.

- `AGENTS.md` — `4209466ec6bdea3e62fa13457af881c09c1804f58106f4e3f4a8290adef0b1c8`
- `CODEX.md` — `b08fe415da04b2b88b5cf2f54e8efb6b56e39434f77c61767cf391fc353e2d34`
- `docs/linking-code-audit-2026-10-16.md` — `aa0a7d427baad85e117b1464b11499479eb80694fd4fba7de56dc77c79bb0a0d`
- `docs/linking-code-audit-2026-10-16-evidence.json` — `621a9c471886eb2cb3884d86ecb25f3cb54b417b51425d519bd4f68e0606c5c7`
- `docs/plans/linker-claude-audit-2026-10-16.md` — `015fe42d27d765de9781e0e53e254026c27df6acb7e46ea3fde3ad87319497a1`
- `crates/lex-parse/src/lib.rs` — `c8aefb457a4337195f866f9badd8645df60ffa1bacd5d261cff372ac6b12a97c`
- `crates/lex-export/src/lib.rs` — `f8361e1138f356fbf7a4e56eb6245695116983c1b616fe3fc1dd6a874979b667`
- `crates/lex-cli/src/main.rs` — `be7fef24658f6e0b8d4bfe8c1fbcd99e38ab095d466fe72fe09f999e18422dd5`
- `crates/lex-core/src/lib.rs` — `7d017c4115e5ea58a8dab0136fa8c6d5b2fa2bb7dd651f727947c1114b4f9016`
- `schemas/reference-edge.schema.json` — `08ec62c532e31613243b0e317bd162921a6281f3230b7a7d53bdf39624926f16`
- `adapters/diputados/_instrument-aliases.json` — `f28068208730f2cf93d7172d95120fe83b8a73feacf457c541c2014f09377074`

Private reproduction caches are under `.work/linker-claude-audit-20261016/`.
Durable inputs/results are embedded in the public evidence JSON; those caches
are conveniences, not the only copy of a finding. No provider-owned config was
edited. This is a written handoff; the operator starts the receiving harness.
