# Glossary detection and imported definitions

## Purpose and outcome

Many committed instruments have a glossary article, but `terms.json` holds
0 terms for them, so their defined terms have no usages and no Markdown
links. The operator ruled on 2026-10-18 that an introduction such as "se
considerarán las definiciones contenidas en el artículo 3 de la Ley, así
como las siguientes:" is written that way because the parent law holds the
original definitions. The regulation's glossary therefore adds to the parent's.

**Observable outcome:**
- every reviewed glossary yields exactly one term for each fraction that has
  not been repealed;
- an importing glossary resolves usages against its own terms first, then
  against the originals it names (`glossary.additive_to`);
- instruments outside the reviewed list keep their terms and usages byte for
  byte.

**In scope:** law and regulation glossaries introduced by "se entiende por:",
"se entenderá(n), en singular o plural, por:", or an import of another
instrument's definitions followed by "así como las siguientes:" or "y las
siguientes:".

**Out of scope:**
- chapter-level glossaries ("Para los efectos de este Capítulo…"), which need
  a scope model;
- enumerations that only resemble glossaries (`cff` 16 "Se entenderá por
  actividades empresariales las siguientes:", `lisr` 142);
- stripping running page headers, which is a parser repair that needs
  retained source;
- the shared title-prefix usage imprecision (`docs/decisions.md`, 2026-10-18).

## Authority and baseline

- **Authority:** `AGENTS.md`, `docs/decisions.md` (2026-10-18 entries),
  `crates/lex-parse/src/terms.rs` and `crates/lex-cli/src/main.rs`
  (`extract_instrument_terms`).
- **Baseline:** `main` after the reg-lan quoted-usage commit (2026-10-18).
- **Cause:** `detect_glossary_terms` recognizes only
  `se entender[áa] (por|lo siguiente):`. It misses "se entiende por:", inserted
  phrases such as "se entiende, en singular o plural, por:", and import
  introductions.
- **Census (2026-10-18, metadata scan):** 109 unconfigured instruments with
  0 terms have a broad glossary-like introduction followed by Roman
  fractions. This count includes known false positives, so it is an upper
  bound, not the work list.

### Importing glossaries and their originals

| Instrument | Imports | Original in corpus |
|---|---|---|
| `reg-lftr-mcsre` 3 | LFTR art. 3 | 71 terms from art. 3 |
| `reg-lgcc-rne` 2 | LGCC art. 3 | 38 terms from art. 3; the stored text has a header leak before fraction VI |
| `reg-lemefagn` 3 | LEMEFAM art. 2 | 2 terms, from art. 2 |
| `reg-lss-macerf`, `reg-lss-maeba`, `reg-lss-rfar` 2 | LSS art. 5 A | 0 from 5 A (LSS's 6 terms come from art. 159); header leaks in maeba and rfar |
| `lih` 3 | LSH art. 5 | 0 terms (LSH 5 uses "se entiende, en singular o plural, por:") |
| `reg-lgeepa-mretc` 3o | LGEEPA, plus "otros ordenamientos" | 0 terms; header leak |
| `reg-lgeo` 3 | LGEO, LPTE, LSE and their regulations | 0 terms each |
| `lcne` 6, `lbio` 2, `lgeo` 2, `lpte` 3 | several energy laws | 0 terms |

## Milestones

### G1 — Detector and completeness probe

1. Widen the introduction grammar in `detect_glossary_terms`, with fixtures
   for each accepted introduction and for each rejected near-miss (`cff` 16,
   `lisr` 142, chapter-scoped introductions).
2. A completeness probe compares each extracted glossary with its Roman
   numerals. Every fraction that has not been repealed must give one term,
   and a repealed placeholder must give none.

**Verify** in a disposable clone with `link --all`:
- only instruments on the reviewed list change;
- every other instrument's `terms.json` and `term-usages.json` are byte
  identical.

**Stop if** an instrument that already has terms would change.

### G2 — Originals first

Make the originals' glossaries complete (LSH 5, LSS 5 A, LGEEPA 3, LSE,
LPTE, LGEO), using adapter configuration where auto-detection is ambiguous.
LSS already has terms from article 159, so its glossary choice needs an
operator ruling.

### G3 — Imports

Configure `additive_to` for each importing glossary whose single named
original passed G2. Held for operator rulings:
- `reg-lgeo` and the multi-law energy glossaries, where precedence between
  originals is a legal choice;
- `reg-lgeepa-mretc`'s "otros ordenamientos";
- instruments with header leaks, which wait for a parser repair from
  retained source.

### G4 — Records

Record a decision entry, regenerate Markdown for changed instruments, and
send a Codex review packet.

**Recovery:** `git restore` the changed instrument directories and adapters.

## Current checkpoint

2026-10-18: plan drafted from the census, and nothing implemented. The
reg-lan quoted-usage rule landed separately.

## Discoveries

- Header leaks inside glossary text (a running page title between two
  fractions) would merge two entries silently. The completeness probe has to
  catch this.

## Next action

The operator approves G1, or narrows the scope (for example, to the
importing regulations and their originals only).
