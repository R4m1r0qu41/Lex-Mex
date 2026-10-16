# Parser and linker repairs — 2026-10-16

All six defects from `docs/parser-linker-audit-2026-10-16.md` are repaired in
the working tree based on `7326c41aa`. No schema or canonical corpus migration
is required. No new commit or push has been performed.

## Resulting behavior

| Defect | Corrected behavior |
|---|---|
| Human decisions overwritten by reparse | Changed or missing human-reviewed evidence stops reparsing before canonical writes. Existing lawyer-verified provisions are checked even if the sidecar is missing or incomplete. Unchanged grounded decisions remain usable. |
| Successor-clause evidence drift | Validate, link, JSON/Markdown/Obsidian export and bundle publication verify the current successor, exact clause, owner, hash and quotation before writing. |
| Missing explicit internal citation disappears | Explicit self-citations remain unresolved and produce a validation error; ambiguous implicit candidates retain earlier suppression. |
| External relative citation becomes internal link | Governing context is checked; unsupported external-relative references stay unlinked. REG-LGN's bare own-Reglamento reference remains valid. |
| Existing target claims wrong instrument | Canonical target ownership is checked before existence can satisfy validation. |
| Quoted reform banner truncates annex | Opaque annex content is preserved until an explicit configured boundary. Its quotations cannot become reform evidence. |

The human-state guard preserves prior records for an explicit audited migration;
it does not reapply an old legal conclusion to changed text. Source text and
character offsets are not rewritten to make links resolve.

## Legal exception preserved

A private LGDNNA relink now exposes Article 135 Ter's literal citation to
Article 157 as an unresolved edge, with exact span `157` at 319..322. Validation
fails as intended. The retained instrument has no Article 157. JRH must decide
how to record this source citation; no replacement or approval was fabricated.
The committed LGDNNA text and graph remain unchanged. This exception must be
resolved before a corpus-wide relink can claim a completely passing result.

## Verification

- Full native gate passed: formatting, clippy with warnings denied, **278 tests**,
  LRITF and IFPE validators (zero issues for both).
- Actual isolated stale-human reparse exits 1 before writes; canonical tree bytes
  remain unchanged. Missing/incomplete sidecars and unchanged reviewed state have
  focused regressions.
- Actual isolated stale-successor validate/link/JSON/Markdown/Obsidian commands
  each exit 1 without changing the predecessor tree. Bundle regression verifies
  rejection before creating output.
- The original private linker/annex reproductions now retain the explicit missing
  citation, reject the false owner, omit the false relative link, and retain the
  entire annex tail.
- Eleven affected parse/link/validate/export replays pass all **44 stages**, with
  identical canonical JSON, source manifests and Markdown. Stored graph integrity
  remains valid across 467 instruments and 27,429 edges.
- Independent GPT-6.1-Sol review accepted CLI/bundle and annex integration. Separate
  isolated Sol implementations supplied linker and annex regressions; parent
  reviewed and integrated their patches.

Final affected-source replay results and file hashes are in
`docs/parser-linker-repair-2026-10-16-evidence.json`.

## Scope

Only implementation, regression fixtures and documentation changed. Canonical
corpus data and human decisions remain unchanged. The audit report/evidence retain
the original pre-fix findings; this report records the subsequent repairs.

## Integration addendum — 2026-10-16 (Claude)

The repairs above were integrated onto the corpus-wide relink (`ab6b7adb5`,
`bfcb4a683`) from the saved patch and landed as the commit after it. The
figures above were measured on `7326c41aa`; this section records what changed
in integration.

- **Merge:** `crates/lex-cli/src/main.rs` conflicted with the relink's closure
  code. Resolved by keeping both sides; `run_link` calls
  `validate_persisted_repeal_evidence` before relinking.
- **Closure path:** `link --all` (`run_corpus_closure`) did not go through
  `run_link`. It now runs the same persisted-evidence check on every instrument
  before proposing graphs, and the successor-drift test asserts the closure
  fails too.
- **Regression found and fixed:** the relative-citation context rule removed 26
  correct "artículo anterior" edges from 26 instruments by letting a law named in
  a later clause govern the citation (for example "…el artículo anterior, se
  estará a lo dispuesto en el Título Sexto de la Ley Federal de Procedimiento…").
  A first fix (cut context at `,` `;` `:`) still removed 6 where the law was
  named without a comma. The rule now treats the context as governing only when a
  genitive is attached to the citation (`de `/`del ` immediately after it), and
  keeps the clause cut. Fixture: `relative_reference_is_not_retargeted_by_a_law_named_in_a_later_clause`.
  Caveat: for an attached citation the clause cut can truncate an instrument
  title that itself contains a comma; no committed instance exists today.
- **Result after integration:** all 467 instruments validate with the release
  binary; `link --all` relinks 0, leaves 446 unchanged and rejects 21 (the 12
  held by the relink, `lgdnna`, and eight whose explicit citation to a missing
  article of their own text is now retained as unresolved). The stored-graph
  count is 30,413 edges, not 27,429. Native gate: fmt, clippy, 287 tests
  (lex-cli 37, parse 206, core 27, export 10, source 4, boundaries 3) green.
- **Open:** LGDNNA Article 157 (JRH). Technical review of both commits by Codex
  is pending.
