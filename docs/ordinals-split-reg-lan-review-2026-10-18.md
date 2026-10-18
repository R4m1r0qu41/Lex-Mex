# Ordinal grammar, heading split and REG-LAN review

Reviewed 2026-10-18, `4582e47fb..adadd9c9da40a58d3a63af10ce4013024ff4d54a`.
Incoming handoff: `2026-10-18-lex-mex-ordinals-split-and-reg-lan-terms.md`.

## Outcome

The seven committed reparses pass independent source/body checks. The reviewed
REG-LAN sentence produces the intended four terms. No corruption of that stored
result was found. Two glossary parser hardening defects and an impossible M2
acceptance requirement need correction before relying on the general mechanism.
This is a review, not authorization to execute M2 or alter legal decisions.

## Findings

### P2 — Referent counting can accept an unpaired list

`crates/lex-parse/src/terms.rs`, `quoted_respectively_entries`, counts commas and
adds one whenever the last chunk contains ` y `. A conjunction inside one
referent is therefore mistaken for a list separator. Direct execution of the
current public extraction function accepts three terms for this two-referent
sentence:

> Los vocablos "A", "B" y "C", se entenderá que se refiere al Registro, a la Secretaría de Educación y Cultura, respectivamente.

This contradicts the claimed fail-closed pairing check. The current REG-LAN
sentence is correctly handled; the failure is synthetic. Remedy: use a narrow,
explicit referent-list grammar with parallel introduction markers, refusing
ambiguous internal conjunctions, or bind this opt-in adapter style to reviewed
vocables/referents and their source spans. Add acceptance and rejection fixtures
for internal conjunctions and separator whitespace. Do not infer legal pairings.

### P2 — A second definition sentence is silently ignored

The same function uses `captures`, not an explicit exactly-one-match check.
Two consecutive valid sentences defining A/B and then C/D return only A/B.
For this deliberately single-sentence style, collect matches and reject multiple
matches (or explicitly support all of them with separate spans and duplicate
checks). Add a regression proving that no definition disappears silently.

A related span defect is reproducible with `Primera oración.\nCuando ...`:
`rfind(". ")` fails at a newline, so the returned definition span includes the
preceding sentence. Recognize the supported sentence boundary whitespace or
require a source-bound sentence prefix. The current committed sentence uses the
supported boundary and is unaffected. Add a newline boundary regression.

### P2 — M2 cannot reproduce source-recovered text from committed text alone

`docs/plans/merged-heading-split.md` requires a committed-text splitter to
reproduce all seven M1 reparses byte for byte. Four new articles were absent
from that old committed text: LGMDFP 12 Duodecies and REG-LSEM 33
Undecies–Terdecies. A splitter cannot reconstruct their bodies or missing chapter
context without another input.

Revise the gate: require exact parity for the 19 articles recoverable by splitting
old committed containers; separately test the four source-recovery cases using
hash-bound source evidence. A committed-text-only run must report unavailable
content/context rather than invent it. Keep operator approval of M2 pending.

### P3 — Edge accounting in the handoff and split plan is inaccurate

The diff contains 12 removed edge IDs and 5 added IDs, with no modified common
IDs. Two citations moved to new provisions: LFPED → article 5 and REG-LSEM →
LSEM article 18. Thus **10 false heading-derived self-links disappeared**, and
three genuinely new parent-law edges appeared. Net change: **−7**, agreeing with
35,887 → 35,880. The handoff/plan say eight removals and omit the second move.
Correct that evidence description; it is not a corpus-data repair.

## Answers and independent verification

- **Equivalent ranks:** accepted as ordering equivalence under the recorded
  operator decision. Rank equality does not collapse canonical identifiers;
  spellings retain their slugs except existing accent normalization. A
  metadata-only census found no stored article pair using distinct equivalent
  spellings on the same base in one instrument. This does not prove no future
  source can print both; preserve identities and report such a collision for
  review rather than deduplicating by rank. No new legal interpretation made.
- **Byte accounting:** all nine changed old containers partition exactly into
  the remaining body plus the 19 promoted bodies after removing only their
  heading lines. No existing provision is removed; other fields of those old
  containers are unchanged. The four source-recovered bodies are additional.
- **Source replay:** PDF and extracted-text SHA-256 values match the committed
  manifests for all seven. A separate temporary Rust probe called the current
  parser on each retained text; every provision ID, order, text and heading
  context matches the committed result (117/115/195/212/219/155/196 provisions
  respectively for LFPED/LGAMVLV/LGDNNA/LGMDFP/LGV/REG-LGCFD/REG-LSEM).
- **Glossary:** four stored terms are present. Existing tests verify the exact
  reviewed sentence and its shared definition span. The public extractor probes
  reproduce the synthetic failures above. At least 40 title-prefix false usages
  remain acknowledged debt from the handoff; this review does not claim a fresh
  count of them. A title-aware usage filter should be a separate bounded change
  with true-referent controls and a reviewed corpus usage diff, not bundled into
  the ordinal or heading splitter.
- **Native gate:** fresh `cargo fmt --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`, and
  `cargo test --workspace` pass (313 tests). `git diff --check` passes.
  No fresh full closure, CLI validation/re-export or database build was run.

Evidence: `docs/ordinals-split-reg-lan-review-2026-10-18-evidence.json` records
source-hash checks and exact body partitions. Probe runs used an isolated
`/private/tmp` crate and repository-locked dependency versions; they did not
edit code or canonical files. An initial offline probe selected an uncached
version and failed; using the repository lockfile completed successfully.

## Continuity and next action

Capsule 114 was stale: it still described the earlier database handoff at
4582e47fb and four local commits. This review baseline is adadd9c9d; the incoming
handoff records eleven local commits. The separate database worktree was not
inspected or changed. No commit/push, parser fix or canonical mutation here;
operator suggestion files remain untouched.

Next action: repair the glossary guards and correct the split plan's M2 gate
and edge accounting, then re-review before the remaining heading-split work.
