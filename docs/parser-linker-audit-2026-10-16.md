# Parser and linker audit — 2026-10-16

The six findings below were subsequently repaired and verified. See
`docs/parser-linker-repair-2026-10-16.md` for results; this audit retains the
original pre-fix reproductions.

Audit baseline: clean `main`, `7326c41aa`. The earlier ten-source admission and
later successor-clause changes were already committed when this check began.
This audit does not authorize another commit or push.

The current corpus passes graph integrity and the eleven affected instrument
replays are identical. Independent reviews nevertheless reproduced defects not
covered by the existing tests. Passing structural validation does not clear them.

## Findings and remedies

1. **Reparse can overwrite applied human legal decisions (high priority).**
   `crates/lex-cli/src/main.rs:2152` only warns when persisted evidence becomes
   stale; `run_parse` then writes newly parsed provisions (`:2092`). A private
   LRITF reconstruction changes reviewed Séptima text and returns exit 0 while
   changing its canonical state from `lawyer_verified`/`effective`, with an
   effective date and two effects, to `not_analyzed`/`unknown`, null date and no
   effects. The historical temporal-result sidecar remains intact; this finding
   concerns the applied canonical decision, not deletion of its audit history.
   The reconstruction also omits reform evidence, so eight machine decisions
   become stale; the independently altered Séptima demonstrates the human case.
   Remedy: stop before any canonical write when an ordinary rerun would clear
   a lawyer-verified decision. Preserve prior records for an explicit audited
   migration. Never force the old legal conclusion onto changed source text.

2. **Repeal evidence is not rechecked by validate/export (high priority).**
   `crates/lex-cli/src/main.rs:3095` and `:3138` load metadata but do not validate
   the current successor clause. Exact successor matching occurs only during
   parse (`validate_repeal_config`, `:1973`). In an isolated copy of `reg-lfmn`,
   replacing the successor's abrogating provision entirely still produces
   `validation: valid` with exit 0. The notice's recorded clause hash and actual
   successor text then disagree. The actual committed clause remains intact.
   Remedy: resolve and verify cross-corpus repeal evidence on validation and all
   export paths, with failure before publishing/writing. Test missing, changed,
   wrong-owner, and unchanged successor clauses. Preserve the operator decision;
   surface stale evidence for review rather than changing instrument status.

3. **Explicit missing internal citations disappear (medium priority).**
   `crates/lex-parse/src/lib.rs:443` removes every unresolved internal edge,
   including an explicit `artículo 999 de esta Ley`. The reproduction returns no
   edge and valid validation. This exceeds the documented suppression of
   ambiguous external context. An actual example is LGDNNA Article 135 Ter:
   `artículo 157 de esta Ley`; the instrument's own articles end at 154 and no
   edge records that citation. Remedy: distinguish explicit self-context from an
   unclassified citation. Preserve an unresolved edge or a blocking diagnostic
   for explicit missing self-targets. Keep uncertain external-context suppression
   separate; do not indiscriminately turn all previously dropped candidates into
   errors. **JRH review:** decide how to record LGDNNA's apparent stale source
   citation. Do not invent a target or silently substitute another article.

4. **External relative citations can link to an internal neighbor (medium).**
   `crates/lex-parse/src/lib.rs:515` ignores governing instrument context for
   relative references. In a three-article fixture, Article 2's `artículo
   anterior de la Constitución Política de los Estados Unidos Mexicanos` links
   to the fixture's own Article 1, even with the CPEUM alias configured.
   Remedy: apply instrument-context classification to relative references and
   leave unsupported external-relative cases unlinked with a diagnostic. Keep
   ordinary internal previous/next references. No such incorrect external edge
   was established among the 2,580 committed relative references inspected by
   the bounded suffix probe.

5. **Target existence does not prove target ownership (medium).**
   `crates/lex-parse/src/lib.rs:2121` checks membership in a flat external target
   set. A synthetic edge naming CPEUM as `target_instrument_id` but an existing
   LGA provision as `target_provision_id` passes validation. Remedy: validate
   against an owner-aware target index or verify the canonical target's owner
   independently. Add mismatched-owner and correct-owner regressions. The full
   committed graph contains no such mismatch.

6. **An annex can terminate on a quoted reform-banner phrase (medium).**
   `crates/lex-parse/src/diputados.rs:1308` tests global stop markers before the
   active annex branch (`:1320`); `is_stop_marker` (`:480`) uses substring
   matching. A legal annex paragraph containing `ARTÍCULOS TRANSITORIOS DE
   DECRETOS DE REFORMA` is treated as document termination. Parsing succeeds
   while discarding that paragraph and all following annex text. Remedy: require
   a structural appendix boundary; preserve an already-entered substantive annex
   when the phrase occurs in ordinary prose. Add a source-shaped regression for
   both a genuine appendix heading and an inline quotation. The current PEF
   replay is unchanged; this is a reproduced failure shape, not demonstrated
   current PEF text loss.

## Checks performed

- `cargo fmt --check`, clippy with warnings denied, and `cargo test --workspace`:
  passed, **266 tests**, zero failures.
- LRITF validation: 145 articles, 11 transitories, 126 references, zero issues.
- IFPE validation: 59 articles, 4 transitories, 113 references, zero issues.
- Scripted graph check: 467 ordinary instruments, 68,086 provisions, 27,429
  references (27,423 resolved, five historical unavailable, one reviewed stale).
  Exact source/qualifier spans, source ownership, existing targets and target
  ownership all pass. The separate 32 NOM standards were outside this audit.
- Isolated parse/link/validate/export replay for `lga`, `lgag`, `lif-2026`,
  `pef-2026`, `lftr`, `reg-lgs-insumos`, `lcnbv`, `lcmopfih`, `lisipl`, `lfiiedb`
  and `reg-lfmn`: all 44 stages exit 0. Instrument/provisions/references/source
  manifest/enactment/reform JSON and all compared Markdown bytes are unchanged.
- Independent GPT-6.1-Sol linker and parser reviews used private reproductions.
  No main source code, canonical text, graph, legal decision or export was changed.

## Recommended next action

First protect audited human decisions and successor-clause evidence. Repair
the remaining reproduced validation/extraction defects with narrow regressions,
then rerun the native gate and affected isolated replays before a new code/data
commit. Keep the LGDNNA citation as a separate operator decision. Current
admissions need no rollback based on these replay and integrity results.

Machine-readable evidence and reproductions:
`docs/parser-linker-audit-2026-10-16-evidence.json`.
