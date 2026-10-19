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

### G0 — Usage-driven cross-check (operator idea, 2026-10-18)

A capitalized `Ley` or `Reglamento` used as a noun is a sign that the
instrument defines it somewhere. Examples are `la Ley`, `dicha Ley` and
`conforme al Reglamento`, where no title follows. Scanning for these uses
finds glossaries that the introduction grammar misses, and it checks G1
independently of that grammar.

**Census (2026-10-18, metadata scan, at least 3 definite or anaphoric uses):**

| | `Ley` | `Reglamento` |
|---|---:|---:|
| Instruments with that many uses | 220 | 110 |
| Already a term | 63 | 16 |
| No term, express definition found | 47 (43 list entries, 4 quoted) | 17 (list entries) |
| No term, no express definition | 110 | 77 |

**Findings:**
- **The found definitions are precise.** All 10 sampled were genuine glossary
  entries, for example `VII. Ley: la Ley Minera` (`reg-lmin` 1) and `I. Ley:
  la Ley del Seguro Social` (`lss` 5 A).
- **One inline form was missed:** `se entiende por Ley, a la Ley Federal de
  Protección al Consumidor` (`reg-lfpc` 2).
- **Most of the "no definition" group defines nothing expressly.** These are
  mostly regulations whose glossary opens by importing the parent's
  definitions: `además de las definiciones previstas en el artículo 4 de la
  Ley` (`reg-lfiiedb` 2, `reg-lmigra` 3), or `Las definiciones contenidas en la
  Ley serán aplicables` (`reg-lsem` 2). There, `la Ley` means the parent by
  context: the title and the wording before `así como las siguientes`.
  Citations through it already resolve (`parent_shorthand`). Making it a term
  needs a decision on an implicit definition: where its span is, and which
  `Basis` applies. The type has no `implicit` basis today.
- **In statutes, `el Reglamento` usually means the law's own regulation.**
  An article-level citation to it is rare: only one `artículo N del
  Reglamento` occurs in a statute with exactly one child regulation (`lsh`).

**Use in G1:** an instrument with this signal and no term is a G1 target.
When G1 finishes, each such instrument either has the term or is on a
reviewed list of implicit or absent definitions. The census script is a
disposable scratch probe. Reimplementing it as a Rust audit is deferred: it was
not built in G1, and a later milestone or the database's coverage checks pick it
up.

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

2026-10-18 (G3, first import): `reg-lss-macerf` adopts LSS Article 5 A only,
through a provision-scoped `additive_to` entry. That gives 449 usages; its own
terms are unchanged. `n29` (Prestaciones Médicas) is not in the corpus.
`reg-lss-rfar` (same wording) and `reg-lss-maeba` (held) are not configured.

2026-10-18 (completeness-check repair): Codex P1 (whole-entry repeal
placeholders) and P2 (qualified fraction order and spelling) are fixed in
`complete_entries`. A probe `link --all` changes no corpus file, and the 19 holds
are unchanged. Persisted coverage is still open.

2026-10-18 (operator ruling on LSS): Article 5 A is the glossary, and 159 adds a
different set of terms. Both are linked through `additional_provisions` and the
new `auto_whole` and `auto_established` styles. `lss` goes from 6 terms to 30 and
from 5 usages to 946, with the 159 records unchanged. 159 still lacks VII and VIII.
Details are in `docs/decisions.md` (2026-10-18, "LSS defines two term sets").

2026-10-18 (Codex review at dbc3ab17d): G1 totals and all 19 holds independently
confirmed; 324 tests pass. G2 remains partial (LSS scope unresolved). Reproduced
repeal-placeholder completeness bypass and duplicate qualified-fraction acceptance.
Before expanding admission, follow the [outlier and title-usage review](../glossary-g0-g2-outlier-review-2026-10-18.md):
strict guard repairs, source-bound adapter profiles, persisted coverage, and a
separate title-span usage filter. No production changes in this review.

2026-10-18: G0 and G1 landed (operator "go"). The wider fallback pass adds
1,619 terms and 28,245 usages to 94 instruments that had none, and holds 19
incomplete glossaries with a reason. No existing glossary changed. Details and
the open rulings are in `docs/decisions.md` (2026-10-18, "Glossary
auto-detection gains a wider fallback pass").

**Original next step (superseded by the review checkpoint):** G2 and G3.
- G2 is mostly done by G1: LSH 5, LGEEPA 3, LSE 3, LPTE 3 and LGEO 2 now have
  terms.
- LSS needs a ruling: its art. 159 glossary is the one detected, not 5 A.
- The held list needs the header-leak parser repair.

## Discoveries

- **Inline glossaries end with `, y XXX. Término:`,** and entries often end
  with a period or an amendment note on the same line as the next fraction.
  The first probe held these as incomplete. Allowing those starts admitted 28
  more instruments (66 → 94) without any partial glossary.

- **The wording before the list names the originals.** `así como las
  siguientes` follows a clause that names them (`las definiciones contenidas
  en el artículo 3 de la Ley`). That preceding clause is the evidence for
  `additive_to` in G3 (operator clarification, 2026-10-18).

- Header leaks inside glossary text (a running page title between two
  fractions) would merge two entries silently. The completeness probe has to
  catch this.

## Next action

Repair the reproduced completeness guards and define persisted extraction
coverage, then add the smallest source-bound exception batch from the review.
Operator rulings on glossary scope remain pending except `lss` (both 5 A and
159, ruled 2026-10-18); G3 imports follow accepted originals and scope decisions.
