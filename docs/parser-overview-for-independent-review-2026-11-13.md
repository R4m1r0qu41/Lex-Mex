# How the Lex-Mex parser works — overview for independent review (2026-11-13)

Branch `upgrade/four-goal-2026-11-12`, written at commit `6bf0f9615`. This document describes the
**parser only**: how a retained official source becomes canonical provision records. Linking
(citations, defined-term usages), temporal derivation and export are outside it except where they
constrain the parser. It states behaviour the code has today and the decisions and source cases
that made it so. It is a reading guide, not a specification. Where it and the code disagree, the
code and its tests are authoritative.

Companion records: `docs/parser-rule-inventory.md` (every rule group with its pinning tests; line
numbers there refer to an older baseline), `docs/held-source-parser-contracts.md` (contracts for
held and reviewed sources), `docs/ingestion-difficulty-log.md` (cases that were held rather than
fixed), `docs/decisions.md` (dated rationale).

---

## 1. What the parser is responsible for

**Input:** one instrument's retained official source plus its adapter (`adapters/<publisher>/<slug>.json`).

**Output:**
- `provisions.json`: articles, transitories and annexes, each with an id, label, number, heading
  context, exact text and initial temporal fields;
- `reform-temporal-evidence.json`: the transitories of every amending decree;
- enactment evidence, and amendment references for CNBV compilations;
- the extracted defined terms.

Invariants the rest of the system relies on:

1. **Official text is never rewritten.** Normalization removes page furniture and joins wrapped
   lines; it never edits words. Every normalization is deterministic and pinned by a fixture
   (`AGENTS.md`, Source integrity).
2. **Offsets are Unicode character offsets** into the unchanged provision text. Downstream spans
   (references, qualifiers, term usages, reviewer decisions) must validate against them.
3. **Fail closed, do not guess.** When a structure is unrecognized, the parser stops with an error
   or the instrument is held. It does not drop text silently or invent a boundary.
4. **Provenance is recorded:** official URL, retrieval time, source SHA-256, extracted-text SHA-256,
   extractor version, parser and schema versions.
5. **No model is involved.** Parsing is pure Rust over the extracted text. A model may later
   *propose* a temporal reading; it never parses.

---

## 2. Pipeline position

```
fetch → extract → parse → derive-temporal → link → validate → export
                   ^^^^^
```

`parse` (`run_parse` in `crates/lex-cli/src/main.rs`) does the following in order:

- reads the extracted text;
- resolves the publication date;
- checks reviewed source evidence (repeal notices, reviewer provision notes) against the retained
  source;
- dispatches to a source-family parser;
- runs the glossary and citation passes over the new provisions;
- checks that every persisted reviewer decision is still carried by the adapter;
- runs the positional furniture audit;
- writes the canonical files.

Crates:

| Crate | Parser-relevant content |
|---|---|
| `lex-parse` | Extraction (`extraction.rs`, `html.rs`); source families (`diputados.rs`, `itf.rs`, `dcg.rs`, `standard.rs`); glossary extraction (`terms.rs`); furniture audit (`furniture_audit.rs`); fidelity (`fidelity.rs`); reform evidence records (`reform.rs`) |
| `lex-grammar` | Article-label grammar (`labels.rs`), Spanish/DOF dates, character slicing, connector words. It holds no legal classification |
| `lex-analysis` | The initial temporal status set at parse time (`initial`). Validation and temporal derivation also live here but are not parsing |
| `lex-link` | Citations and term usages (consumes provisions; out of scope here) |

The split into these crates (October 2026) moved rule groups whole, with their tests, without
rewriting predicates (`parser-rule-inventory.md` §5–7).

---

## 3. Extraction

| Source format | Tool | Notes |
|---|---|---|
| PDF | `pdftotext -layout` | `-nopgbrk` for Diputados sources. The two CNBV parsers keep page breaks because they merge paragraphs deterministically across a page boundary. The `pdftotext` version is recorded. On 2026-11-12 it changed from 26.06 to 26.11 for the CPEUM re-ingest. |
| DOC | `textutil` / `antiword` | Version recorded. |
| HTML (DOF notes) | In-crate extractor (`html.rs`) | Block structure, table cells and declared encodings (UTF-8, legacy Latin-1, malformed-UTF-8 fallback). |

Empty extraction fails. The extracted text, not the PDF, is what all offsets address.
`verify-fidelity` later checks, token by token and in order, that every committed provision appears
in the text extracted from the retained official file.

---

## 4. Source families

The adapter's `parser` field selects the family. Counts are the current adapters.

| Family | Adapters | Sources |
|---|---|---|
| `diputados` | 458 | Cámara de Diputados consolidated PDFs of laws, codes and regulations |
| `itf-dcg` | 9 | CNBV compiled DCGs with margin amendment markers. Built for the Fintech DCG (DOF 10-09-2018) and also used, for now, by `cub-dcg-2005`, `socap-sofipo-dcg-2006`, `cue-dcg-2003` and `scap-dcg-2012` |
| `ifpe-dcg` | 1 | The DCG-IFPE-2021 layout (chapter context required) |
| standards (NOM/NMX) | separate `standards` command | Numbered clauses, transitories and post-transitory supplements (`standard.rs`, validated by `standard_validate.rs`) |

JRH (2026-11-12) asked for dedicated parsers for the four DCGs that borrow `itf-dcg`. That work is
parked until the Diputados PDFs are settled.

---

## 5. The Diputados parser in detail

Entry point: `parse_diputados_publication` (`crates/lex-parse/src/diputados.rs`). It generalizes
the original LRITF parser: the committed LRITF corpus is its byte-identity fixture. The adapter
supplies instrument-specific configuration through `DiputadosOptions`:
- running-header lines;
- stop markers;
- annex markers;
- separate-enactment or nested-instrument profiles;
- a reviewed list of separatorless article headings.

### 5.1 Block normalization (`normalized_blocks`)

The extracted text becomes a sequence of logical blocks.

- **Page furniture is removed:**
  - the instrument's running header (default: the uppercased official title, compared without
    accents or a final period);
  - `N de M` page numbers;
  - Cámara/Secretaría letterheads;
  - complete dated DOF edition headers.

  Only a *complete* header line is furniture: a sentence that cites `DIARIO OFICIAL (Primera
  Sección)` stays text (`is_dof_edition_header`).
- **Wrapped lines are joined** into paragraphs. A paragraph break is kept before a new provision
  start, a structural heading, a Roman fraction (`III.`, `IV.-`, `II Bis.`), a numbered paragraph
  or an amendment-note line. Lone `C`, `L`, `D`, `M` are not fractions: at a line start they are
  initials (`C. Presidente`).
- **An enumeration clause closes** at `;`, `:` or `.`, optionally followed by `y`/`o`. This and the
  fraction rule decide where one fraction ends and the next begins.

**Effect of later repairs.** Fractions that older parses had run together on one line are now
separated (P1, 2026-11). The reparse changed only whitespace in many provisions. It also tripped the
guards on reviewer decisions bound to the old text hash (`lic`, `lfpccs`, `lfvv`, `lieps`), which
led to the re-confirmation mechanism (§8).

### 5.2 Enactment partitioning (`partition_enactment`, `partition_nested_instrument`)

Some consolidated PDFs are not a law's own text.

- **Nested law in an enacting decree:** a reform decree titled and enacted as a "Ley que establece,
  reforma, adiciona y deroga…" enacts a new law verbatim inside one of its articles. Examples are
  `lcmopfih` and `lisipl` (1980s omnibus fiscal decrees), and `lfiiedb`, still held.
- **Omnibus enactment:** a decree issues the law and carries its own overall transitories.

The adapter declares the boundaries through `nested_instrument` and `separate_enactment`, which are
mutually exclusive. The parser then splits the law's provisions from the decree's. Missing,
duplicate, reversed or wrong-title boundaries fail closed. The omnibus path is enabled only by the
decree title in the preamble (first 80 blocks).

Origin: the `nested-law-in-enacting-article` failure class in `ingestion-difficulty-log.md`
(2026-08-01). Validation alone could not see it: `valid: true` described whichever text had been
selected.

### 5.3 The main loop

A single pass over the blocks keeps explicit state: the current provision, the heading context
(libro, título, capítulo, sección, apartado), and whether it is inside statute transitories, a
substantive annex, or the reform appendix. The rules below record the case that caused each.

| Rule | Behaviour | Why (case and decision) |
|---|---|---|
| Article headings | `Artículo 15-D.-`, `ARTICULO 1o.`, `Artículo 4o.-A.-`, `Artículo 2448 A.-` parse to a number plus body (`parse_article_start`, label grammar in `lex-grammar`). | The label grammar covers suffix letters, `Ñ` as a distinct letter, digraphs (`CH`, `LL`, `RR`), Latin qualifiers (Bis … Quintus …), ordinal abbreviations and thousands separators. `Ñ` vs `N`: LFT 353-N and 353-Ñ are different articles (2026-07-12). |
| Suffix after an ordinal mark | `Artículo 1o.-A BIS.-` gives id `1o-A BIS` | Without it the heading collapsed to `1o` and `A BIS.-` was left in the body as a duplicate `article:1` (LIVA; 2026-08-23). |
| Lower-case suffix | `Artículo 228 a.-` and `228 a bis.-` are read only when `.-` closes them at once | LGTOC lost about 17,000 characters (arts. 228 a–v) when these were discarded (2026-10-19). |
| Ordinal-word articles | `Artículo Primero.-` gives `1o` for statutes that number operative articles with words | Decree-wrapper articles with the same shape are dropped only when the body states a promulgation or amendment action (`is_decree_article_wrapper`). |
| Separatorless headings | `Artículo 71 El médico…` is accepted only for labels the adapter lists, and each must open exactly one main-sequence block followed by a capitalized body | `reg-pmimss` (2026-10-18). Accepting the shape generally would read sentences as headings. |
| Split heading | A line with only `Artículo 1` followed by `1. …` stays one article | Reglamento del Senado. Collapsing the lines produced the false identifier `Artículo 1 1` (2026-07-14). |
| Unread heading after a structural heading | Fails the parse | Previously such blocks were discarded with their text, a silent loss (2026-10-19). |
| Cover notes quoting another decree | Before the first article, a block opening a quoted `“Artículo Segundo. …` and the rest of its quotation are preamble | CFPC lost arts. 1–2 and CNPCF art. 1 when a vigencia note was read as articles (2026-10-19). |
| Structural headings | Libro/Título/Capítulo/Sección/Apartado lines update `heading_context` and are not provision text | 711 provisions in 62 instruments once ended with the next chapter heading (CPEUM art. 29 ended `Capítulo II De los Mexicanos`). Reparses and re-fetches removed them (2026-10-19, 2026-10-21). |
| Decree apparatus | Dot-redacted decree articles (`ARTÍCULO SEGUNDO.- ........`) and elided runs (`Artículos Segundo a Décimo.- ……`) are dropped, not appended to the last article | P2 (2026-11-12, `laassp` fixture plus 13 instruments). The body must consist only of ordinals, connectors and redaction punctuation. A substantive body never matches. |
| Statute transitories | Ordinal words (masculine and feminine, through the trigésimo compounds, with accentless variants), numeric `1o.-` (LFJS), and Roman `I.-` inside an established transitory section only | Each form is accepted only where its section makes it unambiguous. `1o.-` requires the strict `.-` so that `1o. de enero…` is not a heading. A lower-case ordinal (`décimo. Asimismo…`) is a sentence carried over a page break, not a heading (CPF 2016). |
| Enactment signatures | A place-and-date block with signatures closes the statute transitories | Closing blocks were mis-read as transitory text (`signature-block-bleed`; 2026-07-27, 2026-08-05). |
| Transitories added after the signatures | A later decree's `SE ESTABLECE UN ARTÍCULO TRANSITORIO ÚNICO DE LA LEY …` reopens the statute transitories; anything else ends the law | LOCG (DOF 15-11-2007). |
| Substantive annexes | After an adapter-configured annex heading, everything is one opaque annex provision until a configured stop | Annexes can quote appendix headings, errata or judgments. Generic heading rules would cut them. |
| Stop markers | The main document ends at the reform appendix (`ARTÍCULOS TRANSITORIOS DE DECRETOS DE REFORMA`), at adapter stop markers (LRITF: `ARTÍCULOS SEGUNDO A DÉCIMO`), at `FE DE ERRATAS`, or at a judicial appendix | The first parser kept appended reform transitories out of the statute's own transitories, because they need amendment-event modeling (2026-06-27). |

### 5.4 Reform-decree evidence (`extract_reform_evidence`)

The appendix after the statute is split into acts, and each act's transitories become temporal
evidence. These are not provisions.

- **Act headings recognized:** `DECRETO …`; a short all-caps `LEY …` title line (a long block, a
  running header, a DOF-date citation or a sentence beginning `LEY ` is not one); `REGLAMENTO de …`;
  `ACUERDO por el que …` (LFCPQ 2009); `REFORMAS al …` (CPF 1966). An omnibus decree's section
  title naming this instrument is not a new act (2026-10-19).
- **Act date:** taken from the act's `Publicado en el Diario Oficial de la Federación el …` note
  (`decree_publication_date`).
- **Evidence ids:** `<instrument>:amendment:<date>:transitory:<ordinal>`. A second decree on the
  same date adds `decree-N`; a second transitory section in one decree adds `section-N`
  (`flush_reform`). Example: LIC 1993-12-23 has two decrees, so the second reads
  `…:1993-12-23:decree-2:transitory:primero`. Every evidence id is unique.
- **Collisions fail the parse.** Each case below has its own fix (2026-10-19):
  - an omnibus decree's operative `Artículo Cuarto.` read as a repeated transitory (CFF 1995);
  - a resolution title merged into the previous decree's signature across a page break (CFF 2002);
  - the carried-over `décimo. Asimismo…` sentence (CPF 2016).
- **Judicial dispositions** in the appendix are not reform transitories.

What this evidence is *not*: the consolidated PDF often prints only a decree's marks ("Fracción
adicionada DOF 13-08-2009") and not its transitories. That happens, for example, when the decree
issued another law (LIC #39) or is a Ley de Ingresos printed as an elision. The absence is then a
property of the source, not a parse loss.

### 5.5 Initial temporal fields

Every provision starts `unknown` / `not_analyzed`. Only an explicit, unambiguous printed note sets a
status at parse time: a repeal marker alone on its line is a repeal, a sentence that mentions a
repeal is not, and partial-repeal notes are classified only in one unambiguous shape. Derivation of
commencement dates is a later stage (`derive-temporal`). An instrument-level `repealed` status comes
only from a reviewer-approved `source_repeal_notice` (§8).

---

## 6. The CNBV parsers (`itf.rs`, `dcg.rs`)

CNBV compiled documents are the operational source; the resoluciones modificatorias are provenance
and are not extracted individually (2026-07-05, 2026-07-12).

- **Margin amendment markers.** Markers such as `(7)` attach to the following paragraph as
  `amendment_marks`. They resolve through the document's REFERENCIAS legend (number to resolución and
  action). Marks never alter text. The marker regex was widened to three digits after a two-digit
  cap lost markers (2026-07-30).
- **Orphan markers.** A marker with no following content is reported, not dropped. A marker
  orphaned by a structural heading is still discarded silently; this is documented current
  behaviour.
- **Page breaks are kept in extraction** so that paragraph merging across a page is deterministic.
- **Heading forms.** The `itf-dcg` heading accepts `Artículo 105 Bis. -`, `117 Bis 5-` and
  `319 Bis. –` (period and spaces optional, hyphen or en dash; 2026-10-25).
- **Old compilation format.** Older DCGs (2003–2015) use an older compilation format (2026-07-12).
- **Open.** `socap-sofipo-dcg-2006` (legend omits entry 91) and `scap-dcg-2012` (mark 261) carry
  marks that point to modifying decrees absent from the legend (JRH, 2026-11-12).

---

## 7. Glossary extraction (`terms.rs`)

A glossary provision ("Para los efectos de esta Ley se entenderá por: I. …") becomes defined terms
with exact spans. Detection runs in passes:
- the established style;
- a wider fallback;
- an adapter-reviewed exception profile (`glossary.profile`) bound to the provision text hash and
  term offsets.

A glossary that cannot be read completely is **held, not partially extracted**
(`GlossaryHoldCode`: `SequenceBreak`, `UnknownQualifier`, `NoncanonicalNumeral`, `NoTermDelimiter`,
`SwallowedFraction`, `RepealWording`, `MissingTerm`, `UnplainTerm`, `RepeatedTerm`). Glossary
coverage is persisted per instrument (`glossary-coverage.json`).

Decisions that shaped it:
- every fraction qualifier spelling and repeal placeholder is accounted for (2026-10-18/19);
- an entry's printed name is its definition site, so no term matches inside it (2026-10-20);
- SCJN-invalidated entries are handled as printed (2026-10-20);
- a profile fails closed on any drift or unused hint.

The profile's text-hash binding is why a whitespace-only reparse needs the reviewer to re-confirm
(2026-11-12: `lfpccs`, `lfvv` adopted; `lieps` held).

---

## 8. Guard rails around the parser

Mechanisms that check or constrain parser output, so that a parse cannot silently change reviewed
facts:

- **Persisted-decision guard** (`preserve_intake_decisions`). A reparse refuses to replace or clear:
  - a persisted repeal notice;
  - a reviewer provision note;
  - a stale-citation approval;
  - a legal-operator review.

  The only audited exception is a **re-confirmation** after a reflow (`reconfirms`;
  `lex_core::check_reconfirmation`). It accepts a record only if:
  - every non-evidence field is unchanged;
  - the persisted text hashes to the prior decision;
  - the words are identical after whitespace normalization;
  - the new hash binds to the new text;
  - the prior offsets cover the cited span, and the new offsets are those offsets mapped through the
    reflow (the same occurrence).

  Added 2026-11-13 in response to Codex review item 5 and its re-review. It replaces the manual
  removal of `references.json` used once for `lic`.
- **Source-backed instrument notices.** `source_repeal_notice` must quote the retained source's
  dated header: `Ley Abrogada DOF dd-mm-yyyy`, or `Declaratoria de invalidez del Decreto DOF
  dd-mm-yyyy …` for an SCJN invalidation (lgmime, 2026-11-12). Codex's note applies: the header
  predicate checks neither the header's location nor the instrument's identity, so reviewer approval
  remains the authority. `provision_notes` (reviewer notes such as CPEUM's 2028 application) must
  quote the retained source and never enter provision text.
- **Positional furniture audit** (`furniture_audit.rs`). Detects running headers by Y-coordinate
  repetition across pages (threshold 0.95, never from a single page). It is an automatic ingestion
  gate (2026-08-24) because string filters cannot catch every publisher header.
- **Validation** (`lex-analysis`). Checks counts against the adapter's baseline, article order (gap
  tolerance only when the adapter allows it), duplicate ids, span validity and amendment-mark
  integrity. It reports issues and never repairs.
- **Fidelity** (`verify-fidelity`). Every provision must appear in the extracted official text in
  order.
- **Hold-out policy** (2026-07-28). A structural difficulty without an obviously correct fix keeps
  the instrument out of `corpus/` entirely, with an entry in `ingestion-difficulty-log.md`. A
  known defect never enters canonical data.

---

## 9. Working principles that shaped the code

1. **The repository is the only ingestion gate** (2026-07-11). A vault tool suite once imported 135
   instruments with its own rules. Everything was re-ingested through this parser.
2. **A regression fixture for every material parser defect.** There are 86 Diputados fixtures under
   `fixtures/diputados/`, plus `lritf/`, `ifpe-dcg-2021/`, `itf-dcg-2018/`, `standards/`, `fidelity/`
   and `database/`. Fixtures are excerpts of retained source text.
3. **Adapter-declared exceptions over general heuristics.** Separatorless headings, stop and annex
   markers, nested boundaries and glossary profiles are reviewed per instrument. A heuristic broad
   enough to cover them would misread other sources.
4. **Decide from context** (operator rule, 2026-10-20; repository rule `AGENTS.md` "Decision
   routing", 2026-11-13). Non-legal parsing questions are settled from the surrounding source text
   (the page or two around the problem, the decree a note cites) and recorded. Only strict legal
   questions go to JRH.
5. **Reparse in scratch, diff, then adopt.** Corpus-wide changes are replayed in a scratch root and
   compared per instrument (ids, tokens, temporal fields, reviewed state, validation) before adoption.
   The last full replay (465 instruments, 2026-11-12) differed only in the held instruments.

---

## 10. Known limitations and open defects (current)

All are non-legal and in the agents' queue. Each has a recorded source case.

| Area | Defect | Case |
|---|---|---|
| Reform evidence | Yearly SHCP quota *acuerdos* were filed under the preceding decree in the committed parse. The held reparse groups them correctly but drops the 1984-12-31, 1988-12-31 and `1º de enero de 2002` decree transitories; the `1º` date form is the likely cause. The Ley de Ingresos 2017 transitory is dated 2013-12-20. | `lieps` (held); decree-list comparison against the Diputados reform page, `.work/ref-pages/` |
| Segmentation | Suppressed articles printed `[Artículo 80 Bis. Suprimido]` are folded into the preceding article. | 20 in `lgipe` (arts. 80 Bis–Quinquies, 272 Bis–Sexies, 284 Bis–Ter, …) |
| Fractions | Fractions printed `VI.-` are not recognized by the Obsidian fraction anchors; some fractions remain merged mid-line. | `lcf` art. 25; `lacp` art. 10 fraction IX |
| Nested law | Nested law still held. | `lfiiedb` |
| Invalidated text | Invalidated articles are embedded in the text (accepted closure gap). | `lftr` |
| CNBV marks | Marks with no legend entry. | `socap-sofipo-dcg-2006` (91), `scap-dcg-2012` (261) |
| DCG family | Four DCGs use the Fintech `itf-dcg` parser. | Dedicated parsers parked |

Out of parser scope but often confused with it: linker target defects (ordinal-suffixed citations
`2o.-A` resolving to the bare article, CMPP art. 164's citations of another code) and qualifier
attribution (`crates/lex-link/src/references.rs`, accepted by Codex 2026-11-13).

---

## 11. How to verify

```bash
cargo test -p lex-parse -p lex-grammar -p lex-analysis      # parser, grammar and initial-status tests
cargo run --locked -p lex-cli -- parse <slug> && \
  cargo run --locked -p lex-cli -- validate <slug>         # one instrument
cargo run --locked -p lex-cli -- verify-fidelity --all      # provisions against extracted sources
```

The full repository gate is `cargo fmt --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` (519 tests at
`6bf0f9615`), and `validate lritf` / `validate ifpe-dcg-2021`. LRITF is the byte-identity fixture of
the Diputados parser: reparsing it must reproduce the committed corpus exactly.

Suggested review focus: §5.1 (block joining and fraction rules, the source of most whitespace-only
churn), §5.3 (fail-closed versus silent-loss paths), §5.4 (act recognition and dating, where `lieps`
shows real defects), and §8 (whether every guard binds to the right evidence).
