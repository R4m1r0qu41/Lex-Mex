# Glossary coverage and header-reparse review / repairs

Baseline: main `c40378666`, review range `2c14705bd^..c40378666`.
Operator authorized technical fixes and a local commit on 2026-10-19. No push,
new legal decision, glossary exception admission or database build is included.

## Findings repaired

1. **Coverage validation did not check current imports or adapter selection.**
   It could accept a stale imported term count or fabricated placeholder metadata
   as long as the importing corpus's own counts still matched. CLI validation now
   replays term extraction read-only against the current adapter and sibling
   corpora, compares the full generated coverage record, and compares exact terms
   and usages. Regressions cover removed imported terms, same-count renamed terms
   that affect usages, adapter import removal, and fabricated placeholder labels.
   It neither relinks nor rewrites the corpus during validation.
2. **Closure needs final-state coverage preflight.** Rejected instruments may
   retain their old terms after proposals were computed. Before any closure output
   is written, coverage eligible for writing is now checked against that final
   in-memory corpus. A stale import/usage report fails the preflight rather than
   being published with inconsistent counts. This is deliberately fail-closed;
   it does not relax the accepted-gap list or resolve a rejected instrument.
3. **Unknown JSON fields bypassed the closed schema.** Deserializing into the
   coverage structs silently discarded unknown fields before schema validation.
   All four coverage object types now reject unknown fields, matching the existing
   schema's `additionalProperties: false`. A regression checks top-level and
   nested records. No schema or corpus migration is needed.

4. **The DOF header filter could erase substantive citations.** A broad
   substring match discarded prose containing `DIARIO OFICIAL (Primera Sección)`.
   The filter now requires a complete dated page header in either page parity.
   A regression fixture preserves two substantive citation sentences. This was
   reproduced with a fixture; no stored corpus text loss is claimed.

A legacy closure test used `urn:test:` fixture IDs, incompatible with the already
required coverage schema. Those test-only IDs now use `urn:lex-mx:test:`; no
production identity changed. The new preflight is a named helper so the existing
Clippy function-length limit remains enforced.

## Independent reparse review

Retained files were no longer available in the main work directory. Downloaded
all six official PDFs into a temporary review directory; their SHA-256 values and
new `pdftotext -layout -nopgbrk` text hashes match all six committed manifests.
A separate Rust probe replays the current parser from those exact texts.

For RNE, MAAA, MEIA, MPCCA, MRETC and LGS-investigacion respectively, all
37/58/69/59/39/145 provision IDs, order, texts and heading contexts match the
committed corpus. Reform evidence also reproduces exactly for every instrument.
The diff preserves every provision ID/order and changes only text/heading context
within provisions. The colon-redaction and separatorless-heading fixtures remain
in the native test gate; no broader heading grammar was introduced here.

**Evidence-description corrections:** the six instruments have **226 references**,
not 225. Their source spans, targets, resolution rules and qualifier meanings are
unchanged, but header removal shifts some source offsets, qualifier offsets and
therefore edge IDs. “Unchanged references” is semantic/count parity, not byte parity.

The reparse also changed reform evidence beyond the provision-text summary:
MEIA 0→4, MPCCA 4→11, LGS-investigacion 0→1; MAAA and MRETC changed evidence at
unchanged record counts (5 and 3). Independent source replay reproduces these
records. They remain source evidence, not reviewed temporal determinations.
The companion JSON records this accounting; no canonical repair was necessary.

## Answers to the handoff questions

- **Hold codes:** the nine codes are adequate for current failures. Keep them
  stable and add structured source/entry locations when implementing exception
  profiles; do not force profile code to parse English error messages.
- **Placeholder absence:** acceptable for methods that do not measure them.
  Absent means not reported, while an empty list means the wider method measured
  zero. Preserve this distinction in the database. It is not legal approval or a
  universal completeness claim for the established pass.
- **Rejected instruments:** comparing refreshed **terms and usages**, not terms
  alone, is the correct prerequisite for writing their refreshed report. The
  final-state preflight now additionally checks their imports against the corpus
  that will remain on disk. Rejected graphs and audited decisions stay intact.
- **Imports:** yes, validate their current selected term counts and adapter
  bindings. The repair does so by replay and also checks resulting term usages.
  The current coverage format remains an extraction report, not an archival
  fingerprint of every imported definition body; a release must pin its complete
  input corpus as the DB plan already requires.

The LSS 5 A +159 and provision-scoped import choices follow recorded operator
rulings. Source-bound profiles for the remaining holds and the title-span filter
remain separate work. MEIA/MPCCA imports remain deferred as recorded.

## Verification and current state

The private corpus closure completed: 460 unchanged, 0 relinked, 8 rejected,
exactly matching the recorded accepted gaps; 35,950 references and 9,809 resolved
cross-instrument references. All 73,219 canonical files remained byte-identical.
The completion report was recovered after the process monitor failed; no separate
process exit code was recovered. Thirteen targeted CLI validations, including
LRITF and IFPE-DCG-2021, reported zero issues.

After narrowing the DOF filter, all six reparses again matched. PMIMSS also
reproduced all 173 provisions and reform evidence from a newly retrieved PDF
and extracted text whose hashes match its committed manifest.

Final checks passed: `cargo fmt --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`
(**343 tests**), and `cargo build --locked -p lex-cli`. Required locked CLI
validations for LRITF and IFPE-DCG-2021 were repeated against the private snapshot
with the final binary and again reported zero issues.

Mechanical delegation was attempted; the worker hit its usage limit, so the
parent completed the final checks directly.
Main corpus, adapters and operator suggestion documents remain unchanged.
Current census: 468 federal instruments, 68,282 provisions, 35,950 references,
5,310 terms and 94,736 usages. Coverage: 12 configured, 299 detected, 12 held,
145 none detected. Standards remain a separate model.

Next action after this repair commit: the smallest source-bound exception profile
batch for the 12 holds, followed by the separately reviewed title-span usage filter.
