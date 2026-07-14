# Cross-instrument linking runbook (Fable phase-2)

Status: distilled 2026-07-14 from the hand-run linking pass (stages 1–3:
financial, tax, litigation — commits `6253d5c5`, `293fa14a`, `f1c544b6` on
`fable/cross-linking`). This is the operating prompt for the autonomous
batch loop that finishes alias coverage across the rest of the corpus.
Invoke it from a fresh session in the `/Users/jr/Dev/lex-mex-linking`
worktree (branch `fable/cross-linking`), ideally under `/loop` so the
batch cycle self-paces. Read `AGENTS.md` and `CLAUDE.md` first; if this
file ever conflicts with them, they win.

## What the loop does

The linker infrastructure is DONE and hardened (global alias table,
`regulates` backbone, proximity-bounded resolution, quoted-shorthand and
anaphoric markers, `stale_in_source` disclosure). What remains is
**coverage**: as of 2026-07-14 the corpus has 141 instruments and the
alias table (`adapters/diputados/instrument-aliases.json`) covers 61 —
an 80-instrument gap (44 leyes/códigos + 35 reglamentos + CPEUM).
Citations to an unaliased instrument sit as plain text. Roughly 6–8
domain batches (admin, labor, env, salud, transporte, civil/IP,
penal-special/security), then **CPEUM strictly last** (JRH's bottom-up
ruling; it is cited from everywhere, so its alias entry lands only once
everything below it is covered — and note bare "artículo N
constitucional" citations already have their own citation-form handling,
distinct from the full official title the alias entry adds). Each loop
iteration closes one batch of that gap:

1. **Pick the next batch.** Compute the coverage gap — every
   `corpus/mx/*/instrument.json` whose `short_name` has no alias-table
   key — and group the gap by domain using `batches/*.json` manifests.
   Process ONE domain per iteration, bottom-up within it (reglamentos
   before leyes; DCGs are already fully covered). The loop terminates
   when the gap is empty and a final full-corpus relink runs clean.
2. **Add alias entries** for the batch's instruments. Rules learned the
   hard way:
   - Key MUST equal the adapter's `short_name` byte-for-byte (8 DCG
     entries sat dead for days because of key-order mismatch:
     `CUB-DCG-2005` vs the real `DCG-CUB-2005`).
   - Descriptive titles only; the linker itself filters out bare
     acronyms (no-space phrases are inert). Include the accented title
     and an accent-stripped variant only when accents occur. short_name
     last. Mirror existing entries exactly.
   - Add historical/colloquial titles only with textual evidence (the
     socap pre-2012 title precedent), never speculatively.
3. **Relink.** `cargo run --locked -p lex-cli -- link <slug>` for: each
   batch member, plus every committed instrument whose `provisions.json`
   contains any batch member's official title (grep, case-insensitive,
   accent-stripped variants too). Expect the citing set to be much
   larger than the batch.
4. **Validate everything touched** and classify every anomaly using the
   taxonomy below. Auto-apply what the taxonomy allows; escalate the
   rest. Never leave an instrument invalid: fix it properly or revert
   that instrument to last-committed state and report.
5. **Verify independently** (do not trust your own subagents' prose —
   check the diffs):
   - Aggregate edge diff across all changed `references.json`: count
     added / removed / retargeted by edge id. Expected: purely additive.
     ANY removal or retarget must be individually inspected and either
     traced to a disclosed fix or treated as a regression.
   - Spot-check ≥5 new cross-instrument edges per batch against the
     source provision text (`corpus/mx/<slug>/provisions.json`), source
     span in context.
   - `git diff --stat corpus/ | grep provisions.json` must be empty
     (linking never touches provision text).
6. **Tally unlinked phrases** (`Ley …` / `Código …` titles in text that
   resolved nowhere), split "ingested-but-unaliased" (feed the next
   iteration) from "not ingested" (report only — ingestion is NOT this
   loop's job).
7. **Gate, report, commit.**
   - Gate: `cargo fmt --check`, `cargo clippy --workspace --all-targets
     -- -D warnings`, `cargo test --workspace`, `validate lritf`,
     `validate ifpe-dcg-2021` — all green, every iteration.
   - Report per batch: alias entries added, edge-count delta table,
     sample edges, anomalies + dispositions, unlinked tally.
   - Commit via a **Haiku** subagent (mechanical; give it the explicit
     worktree path and the exact message; commits and check-runs always
     route to Haiku per CLAUDE.md). Never push.

## Anomaly taxonomy — auto-apply vs escalate

The dividing line (JRH-ratified): **additive alias growth is
auto-applied; anything that changes matcher/grammar/resolver logic or
provision text is escalated.**

AUTO-APPLY (do it, disclose in the batch report):
- New descriptive-title alias entries for ingested instruments.
- Fixing a dead alias key (key ≠ adapter short_name).
- `known_stale_citations` adapter entries — ONLY when the source text
  itself or the decree history proves the citation was stale in the
  officially published law (precedents: lic 187 → LRAF "28 Bis" never
  existed post-2014 rewrite; lsar 68 → 1975-vintage LMV "16 Bis" series;
  reg-lisr transitorio sexto explicitly says "vigente hasta 2013").
  Requires a precise, lawyer-readable `note`. If the evidence requires
  guessing, escalate instead.
- Reverting a single instrument to last-committed state when it cannot
  be made valid (the lic/lmv stage-1 precedent) — with a written reason.

ESCALATE (stop the batch, write up the case, wait for review):
- Any change to `crates/lex-parse` or `crates/lex-cli` — resolver logic,
  citation regexes, label grammar, marker lists. Known open members of
  this family: lowercase suffixes in `ReferencePatterns::number`
  (task #22), heading `V Bis` recognition (#23). New members will look
  like: phantom targets from truncated words, citations silently
  resolving to a coincidentally-numbered wrong article, whole article
  ranges glued into one provision.
- Anything touching provision text, count baselines, or `regulates`
  semantics (one-reglamento-one-law rule; a genuine two-law case goes to
  JRH — the oaac-dcg-2009 precedent).
- A `regulates` inference not proven by the instrument's own text
  (title, Article-1 glossary, "en adición a…" clause). Title silence +
  family resemblance needed a JRH ruling once (itf-dcg-2018); assume it
  will again.
- Same-date / same-ordinal collisions or anything temporal-model shaped.

## Known sharp edges (all hit during stages 1–3)

- **Proximity bound**: markers only count within 90 chars after the last
  cited number (`MARKER_LOOKAHEAD_CHARS`). Calibrated corpus-wide: real
  misattributions started at 96, the farthest legitimate marker was 86.
  If a legitimate citation ever needs >90, that's an escalation, not a
  constant bump.
- **Quoted shorthand**: `de la "Ley"` → the citing regulation's
  `regulates` parent; `este "Reglamento"` → self. Straight and curly
  quotes both. A regulation WITHOUT `regulates` set gets no parent
  resolution — flag such instruments instead of guessing.
- **Anaphoric markers**: both the "de/del" and "en" preposition families
  are covered ("en dicho Código…"). A citation whose only nearby marker
  is anaphoric is deliberately SKIPPED, not linked.
- **Grammar families that keep paying**: lowercase letter suffixes
  (LGTOC 228 a–v), pre-1994 digraphs (26-LL), qualifier-after-letter
  (1o.-A BIS), Decies-family qualifiers (LAC 78 Decies–Novodecies —
  task #24). When a validation error smells like "article doesn't
  exist", first check whether the SIBLING corpus is missing articles
  before assuming the citation is wrong — and check the source PDF
  before assuming the parser is wrong.
- **The `\b` lesson**: a "removed edge" can be the system getting more
  correct (phantom 78-Ter/78-Quater edges came from truncated glued
  headings). Removed ≠ regression until traced — but it must ALWAYS be
  traced.
- **Session limits**: a long relink loop can die mid-run. State on
  disk = code complete, corpus partially relinked. Finish the relink in
  place; do not restart from scratch or `git checkout` blindly — run
  `git status` first, always.

## Standing constraints (non-negotiable)

- `corpus/` is committed canonical data; every diff is reviewed, none is
  disposable. JRH is the legal reviewer of record; nothing impersonates
  or infers his approval. Never represent output as official law.
- Rust owns canonical normalization; no hand-editing of corpus JSON,
  ever — the pipeline must reproduce every fact deterministically.
- Obsidian vault is presentation-only. Export after batches when asked;
  the wholesale regen is a separate tracked task (#12).
- Push is JRH's call. The loop commits locally and never pushes.
- Model routing: this loop's judgment work runs on the session model;
  check-runs and commits go to Haiku subagents; a genuinely new
  legal-temporal modeling question goes to JRH, not to a bigger model.

## Loop pacing (when run under /loop)

One batch per iteration, fully gated and committed before the next.
After each batch, schedule the next wakeup at a long interval (1200s+)
if running unattended, or continue immediately if interactive. STOP the
loop (don't reschedule) when: the coverage gap is empty and the final
full-corpus relink + validation sweep is clean; or an escalation is
pending review; or two consecutive iterations fail the same gate.
