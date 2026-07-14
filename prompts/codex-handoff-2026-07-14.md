# Handoff — cross-instrument linking, 2026-07-14

Written for a fresh coding agent (Codex or otherwise) picking up this
work with no prior context. Read `AGENTS.md` and `CLAUDE.md` at the repo
root first — if anything here conflicts with them, they win. This file
is a snapshot of state and open items, not a standing document; it will
go stale as work continues, unlike `AGENTS.md`/`CLAUDE.md`.

## Where the work lives

- Worktree: `/Users/jr/Dev/lex-mex-linking`
- Branch: `fable/cross-linking`, stacked on `main`, **NOT pushed** —
  pushing is JRH's call, not the agent's. `main` itself is pushed and
  up to date.
- HEAD as of this handoff: `5552f8a7`.

## Standing constraints (do not relitigate)

- **`corpus/` is committed canonical legal data**, not disposable
  output. Every diff must be reviewed for provenance and legal meaning
  before committing.
- **JRH is the legal reviewer of record.** Never impersonate or infer
  his approval; a pending legal review is only resolved by his explicit
  ruling recorded in the repo.
- **Rust owns canonical normalization, validation, and review-state
  changes.** No hand-editing corpus JSON. A citation-regex or resolver
  fix always goes through the pipeline (`link`/`pipeline` commands),
  never a manual JSON patch.
- **Never represent repository output as official law or legal advice.**
- **Model routing** (see `CLAUDE.md`): mechanical work — running
  `cargo fmt --check`, `clippy -D warnings`, `cargo test --workspace`,
  `validate <slug>`, and all commits — routes to the cheapest capable
  model (Haiku, in this session). Judgment work (parser/resolver fixes,
  fixture design, diagnosing a validation failure) stays on a stronger
  model. Don't burn a strong model's tokens on a pass/fail check-and-commit
  loop.
- **Verify diffs directly, never trust an agent's or your own prose
  summary of what changed.** This session caught a real corpus bug
  (CPEUM self-citations misattributed to LFT) specifically by re-running
  `git diff` and reading the source text, not by trusting a completion
  report. `git diff --stat corpus/ | grep provisions.json` must be empty
  after any relink — `link` only ever touches `references.json`.
- **The Obsidian vault (`/Users/jr/Vaults/Lex-Mex`) is a read-only,
  presentation-only render of this repo's `Corpus/` folders** (as of
  commit `913b51824` in the vault's own history). Never treat it as a
  source of canonical facts; regenerate it via the exporter, don't
  hand-edit it.

## What's done (this session)

1. **#22 — citation-number grammar fix** (commit `3f63c3da`). The
   `number` regex in `crates/lex-parse/src/lib.rs` now: builds its
   qualifier alternation from `labels::QUALIFIERS` (so the full
   Bis..Novodecies/Decies family matches, not just Bis/Ter/Quáter);
   recognizes space-separated single-letter suffixes ("228 h", "304 B")
   with a post-match check that rejects a letter immediately followed by
   whitespace+digit (a list continuation like "133 Y 138", not a
   suffix) and requires lowercase Spanish connectors (a/e/o/u/y) to be
   immediately followed by `.`/`;`/`)` to count as a suffix; fixed a
   missing `\b` on the hyphenated letter branch; and fixed the letter
   branch's separator to same-line whitespace only (`[ \t]+`, not
   `\s+`) so a citation right before a paragraph break can't swallow the
   next fracción's Roman-numeral heading as a phantom suffix (found via
   CPF article 247 during verification). Full 141-instrument relink:
   34 edges retargeted to their correctly-suffixed article, 1 recovered
   edge, 0 removals, `provisions.json` untouched everywhere, LRITF
   byte-identical.
2. **#24 — Decies-family qualifier grammar** (earlier in the session,
   commit series before `82af9ac0`) — folded into the same corpus state
   as #22.
3. **Fable phase-2 runbook** committed: `prompts/cross-linking-runbook.md`
   (`82af9ac0`) — the operating prompt for the autonomous alias-coverage
   loop over the remaining ~80-instrument gap. Read it before starting
   that work; it has its own detailed procedure and is the authority on
   how to run that loop, not this file.
4. **#12 — vault regeneration.** The vault is now a wholesale,
   committed, read-only render of the Rust pipeline (vault commit
   `913b51824`): 141/141 folders exported, 0 writable files, 0 leftover
   Python-tooling artifacts, 628 sampled wikilinks all resolving.
5. **Edge-graph thematic grouping tally** committed:
   `prompts/edge-graph-grouping-2026-07-14.md` (`a1a85895`) — a
   12-cluster map of the 1,734-edge cross-instrument graph as of this
   session, useful context for judging which future alias additions are
   likely to matter most.
6. **CPEUM self-reference marker fix** (commit `5552f8a7`, diagnosed and
   fixed this session): `self_reference_markers()` in
   `crates/lex-cli/src/main.rs` had no arm for `instrument_type ==
   "constitution"`, so CPEUM fell into the ley-shaped default ("de esta
   ley") and had no internal candidate in the citation-resolver's
   same-sentence race — any correctly configured external law named
   later in the same run-on sentence won by default. Concretely: CPEUM
   art. 3's "...artículo 123 de esta Constitución, en los términos... que
   establezca la Ley Federal del Trabajo..." was silently retargeting the
   Constitution's own article 123 to LFT on a fresh relink, even though
   the *committed* corpus happened to already have the correct edge (an
   accident of when it was originally generated, not a validated
   invariant). Added the missing arm ("de esta constitución"/"de la
   presente constitución"/"esta constitución"); a relink of `cpeum` in
   isolation now reproduces the committed `references.json`
   byte-for-byte, so no corpus commit was needed — only the code change.
   **This class of bug is systemic, not a one-off**: 207 occurrences of
   "esta/la presente Constitución" exist across CPEUM's own text, and
   every one of them was exposed to the same landmine before this fix —
   only one happened to collide with a configured external-law marker
   under the alias table's current size. Expect more of these to surface
   as phase-2 grows the alias table (a newly-aliased law whose name now
   appears in the same sentence as one of those 207 phrases can flip a
   previously-fine edge). There is no automated check for this class of
   defect today — see Open Items below.

## Open items

### #23 — Capítulo V Bis heading recognition (cosmetic, low priority)
LGTOC's "Capítulo V Bis" heading is glued to adjacent text and produces
a wrong `heading_context` on articles 228-a through 228-v. Cosmetic only
— does not affect citation resolution or validation. Not investigated
this session.

### #25 — remaining relink drift in 7 instruments
Same symptom class as the CPEUM bug (a fresh `link` run diverges from
the committed corpus with **unmodified** code) but **not yet
root-caused**: `itf-dcg-2018`, `ldfefm`, `linfonacot`, `lvgc`,
`reg-linfonacot`, `reg-lss-rfar`, `socap-sofipo-dcg-2006`.

Method that worked for CPEUM, reuse it per-instrument:
```
cd /Users/jr/Dev/lex-mex-linking
git status -s                      # confirm clean before touching corpus/
cargo build --locked -p lex-cli
./target/debug/lex-mex link <slug>
git diff --stat -- corpus/mx/<slug>   # see what moved
git diff -- corpus/mx/<slug>/references.json   # read the exact edge(s)
```
Then find the `source_provision_id`/`start_char` in the diff, print the
surrounding source text from `corpus/mx/<slug>/provisions.json`, and read
what marker or law name is actually nearby. Do **not** assume the
CPEUM-style missing-self-marker pattern generalizes — check each
instrument's `instrument_type` (in `adapters/**/<slug>.json`) and its
actual self-reference phrasing before proposing a fix. It's plausible
some of these are a different bug entirely (e.g. a DCG whose own
"de las presentes disposiciones" phrasing has a variant not in the
`"regulation"` arm, or an alias-table collision unrelated to
self-reference at all).

**After each per-instrument diagnosis and fix**: `git checkout --
corpus/mx/<slug>` to discard the scratch relink you used to reproduce
the bug, apply the code fix, rebuild, relink *that instrument only*
again, confirm the diff either matches the committed state
byte-for-byte (no corpus commit needed, like CPEUM) or is an intentional
correction (then it needs its own reviewed corpus commit, edge-diffed
by id same as #22's method). Run the full gate suite
(`fmt --check`, `clippy -D warnings`, `test --workspace`, `validate
lritf`, `validate ifpe-dcg-2021`, `validate <slug>`) before any commit.

Consider, once all 7 are root-caused: is a **regression test or
lint worth adding** for "every self-reference-shaped phrase in an
instrument's own text should have a corresponding entry in
`self_reference_markers()` for its type" — this bug class had zero
automated coverage and was only found by hand-diagnosis. Not designed
yet; flag it to JRH before building anything, since it's a
resolver-correctness design call, not a mechanical fix.

### Fable phase-2 launch (the big remaining item)
Not started this session. Per `prompts/cross-linking-runbook.md`: fresh
session in this worktree, ideally under `/loop`, working through the
~80-instrument alias-coverage gap domain-by-domain, CPEUM's own alias
entry added strictly last (JRH's bottom-up ruling — it's cited from
everywhere). The runbook has its own escalation taxonomy (auto-apply
additive alias entries; escalate anything that requires a matcher/regex
change to a human or a stronger model) — follow it rather than
improvising.

### Loose ends
- `/archive/` line in the **main-repo** (not this worktree's)
  `.gitignore` is an uncommitted working-tree edit sitting on `main` —
  needs an eventual commit, unrelated to this branch's work.
- `fable/cross-linking` (15 commits as of this handoff) remains
  unpushed. Do not push without an explicit JRH/user go-ahead.

## Environment gotchas hit this session

- **macOS has no `timeout`/`gtimeout`.** A loop that pipes through
  `timeout N cmd` will silently no-op on this machine with no error —
  check actual output file counts, don't trust "the loop finished."
- **A 141-instrument relink loop exceeds a 2-minute default shell
  timeout.** Either raise the timeout explicitly or chunk the loop and
  resume from where the log directory left off (`ls <logdir> | wc -l`
  against the expected 141).
- A long-running agent can die mid-task from a session/rate limit with
  **zero code changes made** — always `git status -s` first before
  assuming any work needs to be reverted or resumed; it may simply need
  relaunching from scratch.
