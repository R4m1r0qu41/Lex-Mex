# Architecture decisions

## 2026-10-19 — An omnibus decree's section title for this instrument is not a new act

**Why.** LIEPS could not be reparsed from its own unchanged source: the SHA-256 of
its extracted text matches the committed manifest. Since `dd39cbc93` (2026-08-24),
which was found by bisect, any short all-caps `LEY …` line in the reform appendix
starts a new containing act. LIEPS's omnibus fiscal decrees head their LIEPS part
with the law's own title, for example "LEY DEL IMPUESTO ESPECIAL SOBRE PRODUCCION Y
SERVICIOS" after "Ley que establece las reducciones impositivas… Publicada en el
Diario Oficial… 1993". That heading reset the decree's date, so its `PRIMERO`
transitory failed with "without its Diario Oficial publication date". LIEPS was
last parsed before that commit, so the regression stayed hidden.

**What.** The heading is treated as a section of the open decree, not a new act,
when it meets every one of these conditions:
- it equals this instrument's running header, which by default is the uppercased
  official title, compared without accents or a final period;
- a decree is open, with a publication date and no transitories yet;
- it carries no publication date of its own.

Another law's title in the same position is still a new act. Fixture:
`reform-omnibus-own-title-section-sample.txt`, cut from LIEPS.

**Measured.**
- **LIEPS reparse:** reform evidence is byte-identical, and the 84 provision IDs,
  their order and all texts are unchanged except transitory decimocuarto. Its 1980
  signature block is dropped by the existing enactment-signature rule, and with it
  one reference to CPEUM art. 89 from the promulgation formula. References go
  35,950 → 35,949.
- **Other fiscal laws in a probe:** lisr and liva give identical reform evidence. cff
  and lfd fail their reparse identically with and without this fix. cff fails on
  duplicate reform evidence `1995-12-15:transitory:cuarto`, and lfd on validation.
  Both are pre-existing reparse drift on unchanged source text, recorded for later
  work.
- **Tracked file:** the tracked `temporal-analysis-request.json` was restored, not
  regenerated.

## 2026-10-19 — Glossary exception profiles, batch 1

**Why.** Codex review 2026-10-18 asked that small instrument exceptions be
typed, declarative and source-bound, never `allow_gaps`, regex patches,
instrument branches or canonical text edits. Twelve glossaries were still held.

**What.** An `auto_whole` adapter glossary may carry a `profile`
(`lex_core::GlossaryProfile`):
- It is bound to the strategy version and the provision text's SHA-256.
- Every hint names its literal text and character offset.
- Three operations exist:
  - a fraction read as a `definition` with explicit term spans (one or more);
  - a `non_definition` fraction that yields no term;
  - an `ignored_candidates` span that is not a fraction.
- Any drift, unmatched span, unused hint or duplicate fails extraction.
- Every fraction the profile does not name still passes the shared checks, sequence included.
- The coverage record repeats the profile, so an exception is never reported as a clean extraction. `validate` rejects a profile on another method or other text.

**Batch 1.** Each profile was released by hint against committed text that the current parser reproduces.

| Instrument | Hint | Terms / usages |
|---|---|---|
| `lfvv` 2 | VII `Secretaría;` (semicolon) | 9 / 57 |
| `lfpccs` 3 | XXVII `SNICS.` (period) | 29 / 177 |
| `lfca` 2 | ignore `C.` in `Cinematográfica A. C. y` | 21 / 36 |
| `lsct` 2 | quoted names in all eight fractions; V defines `Ratificación`, `adhesión`, `aceptación` | 10 / 3 |
| `lfra` 2 | V defines `cadena causal`; VI and VII non-definitions | 14 / 123 |

**Operator ruling (2026-10-19), not a JRH legal review.**
- **LFRA:** V's "Se entiende por cadena causal…" is a definition, and VI and VII are substantive rules.
- **LIEPS Article 3:**
  - ingest VIII as the parent `Tabacos labrados` plus its four incisos, each linking to its own inciso;
  - ingest IX's incisos (combustibles automotrices, gasolina, diésel…) as the actual definitions.
  - LIEPS needs an inciso locator in `DefinedTerm`, so it is the next batch.

**Measured.**
- `link --all`: 5 relinked, 455 unchanged, 8 rejected (exactly the accepted gaps).
- References are unchanged at 35,950, and no provision changed.
- Terms 5,310 → 5,393; usages 94,736 → 95,132.
- Coverage: 17 configured, 299 detected, 7 held, 145 none detected.

**Known false matches.** About 20 of LFRA's 67 `Ley` usages are other laws' titles (`Ley General del Equilibrio…`). lfvv and lfpccs each have one `Secretaría` that names another ministry. This is the corpus-wide title-prefix defect, which the title-span filter addresses.

**Not in this batch.**
- `ltfccg`: the current parser leaves `Capítulo II.- Información Previa` inside Article 3. That heading would be swallowed into fraction XI, so it needs a parser fix first.
- `fi-dcg-2014`: CNBV has published a new edition. The live source parses with one more article and detects its glossary, so this is re-ingestion work.
- `ladua`: its committed parse is stale.
- `log`, `servinv-dcg-2013`, `reg-lftr-mcsre`: label and boundary cases that need retained-source verification.

## 2026-10-19 — Coverage validation binds current selection and imports

Codex review of `c40378666` found that local coverage counts alone could not
detect stale import counts or changed adapter selection. Validation now replays
extraction read-only and compares the coverage, terms and usages with current
inputs. Corpus closure preflights eligible coverage against the final in-memory
corpus before writing. Coverage objects reject unknown JSON fields as their
existing schema requires. The DOF furniture filter now requires a complete dated
page header, preserving substantive section citations with a regression fixture.
No canonical data or legal decision changed.

The six header reparses independently reproduce from hash-matching PDFs/text,
including reform evidence. Their 226 references retain their meanings/targets;
some IDs and offsets changed with the cleaned text. Reform evidence also changed
(MEIA 0→4, MPCCA 4→11, LGS-investigacion 0→1; MAAA/MRETC at unchanged counts).
See `docs/glossary-coverage-header-review-2026-10-19.md` for exact verification.

## 2026-10-19 — Persisted glossary extraction coverage (`glossary-coverage.json`)

**Why.** Codex review 2026-10-18, P2: a held glossary was reported only on stderr. The persisted empty `terms.json` could not distinguish a held glossary from an instrument with none, and the database must not infer either from `[]`.

**Record.** Every instrument now has a deterministic `corpus/mx/<slug>/glossary-coverage.json`, written wherever `terms.json` is written: `pipeline`, `link <slug>` and `link --all`.
- **Schema.** `schemas/glossary-coverage.schema.json`, Rust type `lex_core::GlossaryCoverage`, strategy version `glossary-extraction-1`.
- **No timestamps:** a second `link --all` is byte-identical.
- **Bound to its inputs.** It records the instrument's source and extracted-text hashes, and the SHA-256 of each glossary or held provision's text (`evidence_sha256`).
- **Status:**
  - `configured`: the adapter names the provisions and styles.
  - `detected`: auto-detection found a complete glossary.
  - `held`: auto-detection found an incomplete glossary, so there are no terms.
  - `none_detected`: auto-detection found no glossary.
- **Per glossary provision:**
  - its method: `established_pass`, `wider_pass`, or the configured style;
  - the printed fraction label of each extracted term;
  - its term count;
  - for the wider-pass grammar (`wider_pass`, `auto_whole`) only, the repealed fractions skipped as whole placeholders.
- **Per hold:** a typed `code` from the completeness check plus its message. The codes are `sequence_break`, `unknown_qualifier`, `noncanonical_numeral`, `no_term_delimiter`, `swallowed_fraction`, `repeal_wording`, `missing_term`, `unplain_term` and `repeated_term`.
- **Per import:** each `additive_to` entry and how many terms it adopted.
- **Totals:** the term and usage counts.
- **Not legal approval.** The record is a technical extraction report. It is never a legal approval and never a finding that a definition is repealed.

**Validation.** `validate <slug>` checks the record against its schema and against the persisted instrument record, provisions, `terms.json` and `term-usages.json`.
- **Error** (`glossary_coverage_inconsistent`): a stale text hash, different counts or fractions, or a status that contradicts the record. For example, `held` with glossaries or terms is an error.
- **Warning** (`glossary_coverage_missing`): no record.
- **Rejected instruments.** In `link --all`, a rejected instrument keeps its persisted terms. Its record is written only when the refresh reproduced those terms exactly. All 8 do today.
- `bundle` copies the file.

**Also aligned.**
- `schemas/defined-term.schema.json` accepted only bare Roman fractions, but 13 instruments' terms already carried `Bis` or `Ter` labels (34 terms). The pattern now allows an optional ` Bis` or ` Ter`. Nothing enforced that schema before, so no data changed.
- `GlossaryDetection.held` is now a typed `GlossaryHold` with provision, code and message.

**Result** (`link --all` in a disposable clone and then in the repository):
- **Files.** 468 new files, and no other file changed: no terms, usages, references, provisions, validation reports or Markdown.
- **Status counts:**
  - `configured`: 12.
  - `detected`: 299 (202 established pass, 97 wider pass).
  - `held`: 12, exactly the known holds (`sequence_break` 6, `missing_term` 4, `unplain_term` 2).
  - `none_detected`: 145.
- **Validation.** Spot `validate` runs found 0 issues on lritf, ifpe-dcg-2021, lss, reg-lgcc-rne, reg-lan, lfca (held), reg-lgeepa-mretc, reg-pmimss and ccom (none). An edited hash fails as `glossary_coverage_inconsistent`.
- **Pipeline.** `pipeline reg-lss-rfar` wrote the same record as `link --all`.

**Not done here.**
- **Source-bound exception profiles.** Boundary, label and delimiter hints for the 12 holds are still open.
- **Title-span usage filter.** Still open. Each is a separate reviewed change.
- **Configured-style placeholders.** The established pass and the configured styles other than `auto_whole` do not report repealed placeholders, so `repeal_placeholders` is absent for them rather than empty.

## 2026-10-19 — `reg-lgcc-rne` adopts LGCC Article 3; meia and mpcca imports wait for the title-span filter

**rne.** Its Article 2 opens "se considerarán las definiciones contenidas en el artículo 3 de la Ley, así como las siguientes:". It now uses `auto_whole` with `additive_to: ["urn:lex-mx:federal:statute:lgcc:article:3"]`. LGCC's 38 terms all come from Article 3.
- **Usages:** 82 → 207. rne's own 82 are unchanged, and LGCC adds 125: `Emisiones` 62, `Secretaría` 38, `Ley` 14, `Mitigación` 11.
- **Sample check:** the sampled `Secretaría` uses are bare "la Secretaría". rne defines no `Secretaría` of its own.
- **Other instruments:** none changes. References stay at 35,950, and `link --all` matched a disposable probe byte for byte.

**meia and mpcca are not configured.** A probe configured both with LGEEPA Article 3. Their own usages were unchanged, but nearly all of the imported ones are false matches:
- **meia:** 21 imported usages.
- **mpcca:** 29 imported usages.
- **The false matches:**
  - LGEEPA's `Ambiente` and `Protección` matched inside other names: "Secretaría de Medio Ambiente y Recursos Naturales", "Agencia Nacional de Seguridad Industrial y de Protección al Medio Ambiente…" and LGEEPA's own title.
  - `Secretaría` matched inside the regulations' own `Secretaría:` definition line.
- **Why they wait.** Admitting them would add known false usages. They wait for the title-span usage filter, which is a separate reviewed change.

**Still unconfigured.**
- `reg-lgeepa-maaa` also adopts the Ley Federal sobre Metrología y Normalización, which is not in the corpus. Precedence between the two laws is an operator question.
- `reg-lgeepa-mretc`'s "otros ordenamientos" is still an operator question.

## 2026-10-19 — Six glossaries held by running headers are released by reparse; one parser fix

The operator asked for the six glossaries held for running headers to be fixed: `reg-lgcc-rne` 2, `reg-lgeepa-maaa` 2, `-meia` 3, `-mpcca` 6, `-mretc` 3 and `reg-lgs-investigacion` 40.

**Cause.** These six were parsed on 2026-07-12. Since 2026-10-15 (`12e0cc752`), the Diputados parser drops up to three uppercase title lines directly above `CÁMARA DE DIPUTADOS…` (`publisher_title_lines`). That rule covers all six headers, including `reg-lgs-investigacion`'s, which prints `INVESTIGACION` without the accent the title has. So no adapter changes: a disposable reparse with the printed lines declared as `header_lines` gave byte-identical provisions to one without them. The same probe showed that `reg-lss-maeba` needed no `header_lines` either (correction recorded in its entry).

**One parser fix** (fixture `colon-dot-redacted-decree-article-sample.txt`, cut from `reg-lgeepa-mretc`):
- mretc prints its redacted decree article with a colon: `ARTÍCULO SEGUNDO: ..........`. The dot-redaction check accepted only `.-`, `.` and `-`, so the current parser appended that line to Article 32.
- `is_dot_redacted_decree_article` now also accepts a colon. It accepts one only there, never as a general ordinal-heading separator, and a test pins that.
- No committed provision contains a paragraph of that shape, so no stored text can change on reparse.

**Reparse** (`pipeline` for each, then `link --all`; identical to a disposable probe apart from retrieval timestamps and the `pdftotext` version string):
- **Same sources.** The PDF and extracted-text hashes match the recorded ones for all six, and the furniture audits report 0 findings.
- **Provisions.** All ids are unchanged. Only `text` and `heading_context` changed, and no changed provision is `lawyer_verified` or `review_required`.
- **Text changes,** each one classified:
  - **Header removal:** 90 provisions; nothing else changed in them.
  - **Headings:** 15 `Capítulo`/`Sección` headings that ended the previous article's text now sit in `heading_context` (rne 7, mretc 7, maaa 1). Before, every rne and mretc article had `chapter: null`.
  - **maaa.** Articles 33–39 Bis move from Sección III to Sección IV.
  - **Signature blocks:** one per instrument, six in all. Each was a closing "Dado en la Residencia… Rúbrica." block (`is_enactment_signature`, since 2026-07-25).
  - **Paragraph breaks:** every new break where a header used to sit follows a colon, semicolon, period or amendment mark. No sentence is split.
- **References.** All 225 are unchanged (rne 43, maaa 49, meia 37, mpcca 17, mretc 13, lgs-investigacion 67). Totals stay at 35,950 references and 9,809 cross-instrument edges, and the rejected set is still the 8 accepted gaps.

**Glossaries** (wider pass, all complete; held glossaries 18 → 12):

| Instrument | Terms | Usages |
|---|---:|---:|
| `reg-lgcc-rne` 2 | 12 (I–XII) | 82 |
| `reg-lgeepa-maaa` 2 | 24 (I, I Bis, I Ter, II–XXII) | 505 |
| `reg-lgeepa-meia` 3 | 19 (I, I Bis, I Ter, II–XVII) | 131 |
| `reg-lgeepa-mpcca` 6 | 15 (I–XV) | 124 |
| `reg-lgeepa-mretc` 3 | 18 (III is a repeal placeholder; VII Bis) | 155 |
| `reg-lgs-investigacion` 40 | 11 (I–XI) | 0 |

- **lgs-investigacion has 0 usages** because its terms are capitalized in the glossary and lowercase in running text (`embarazo`, `feto`), and matching is case-sensitive. This is the lowercase-usage question already open.
- **Imports are not part of this entry.** maaa adopts the Ley Federal sobre Metrología y Normalización, which is not in the corpus, alongside LGEEPA, so precedence between them is a legal question. mretc's "otros ordenamientos" remains an operator question.

## 2026-10-18 — `reg-lss-rfar`: wrapped running header removed by reparse; adopts LSS Article 5 A

The operator asked for the `reg-lss-maeba` treatment to be applied to `reg-lss-rfar`. Its Article 2 glossary already gave 4 complete terms, but its running header sat inside fraction IV's definition.

**Cause and repair:**
- **Cause.** rfar's running header is its official title printed on three lines, followed by a blank line before `CÁMARA DE DIPUTADOS…`. The whole-line title match misses a wrapped title, and the publisher-title rule (`publisher_title_lines`) looks back only three lines, with no blank among them. It leaked into 5 provisions (Articles 2, 6, 8, 13 and 18). A disposable reparse without `header_lines` keeps the leak, so the adapter field is needed here, unlike maeba.
- **Repair.** The adapter declares the three printed lines as `header_lines`, and rfar was reparsed with `pipeline`. The title block and the `CAPÍTULO V` heading wrap differently, so they do not match those lines and are kept.
- **Same source.** The retained PDF and extracted text match the recorded hashes (`4b4529ce…`, `061dd4c0…`). The furniture audit reports 0 findings.

**Reparse diff:**
- **Provisions.** All 24 ids are unchanged, and no field other than `text` changed.
- **Header leaks.** In the 5 provisions, the deleted text is exactly the joined header.
- **Transitory *sexto*.** It loses its "Dado en la residencia del Poder Ejecutivo Federal… Rúbrica." block, which is enactment apparatus (`is_enactment_signature`), as in maeba.
- **References.** 32 → 31. The removed edge came from that signature block: the countersignature's "artículos 18 de la Ley Orgánica de la Administración Pública Federal; 7, fracción XII, y 86 del Reglamento Interior…" was linked to LOAPF 18 with a `fracción XII` qualifier that belongs to the Reglamento Interior's Article 7. No review was bound to it. The other 31 keep their targets, and two shift offsets only.

**Glossary:**
- **Configuration.** Article 2 uses `auto_whole` with `additive_to: ["urn:lex-mx:federal:statute:lss:article:5-a"]`, as for macerf and maeba.
- **Terms.** The same 4 terms, I–IV. Only fraction IV's span shrinks, to exclude the header.
- **Usages.** 79 → 123: its own 79 are unchanged, and 5 A adds 44 (`Ley` 24, `Instituto` 20).

**Result** (`link --all`, byte-identical to a disposable probe apart from the retrieval timestamp):
- No other instrument changes, and the rejected set is still the 8 accepted gaps.
- References are 35,951 → 35,950, and cross-instrument edges 9,810 → 9,809.
- **Open.** As for macerf, whether the 2013 import follows later amendments of 5 A is a legal reading for JRH or the operator.

## 2026-10-18 — `reg-pmimss` admitted (Reglamento de Prestaciones Médicas del IMSS) as a test case

**Operator decision.** Admit `regla/n29.pdf` as a test of ingestion into a more populated and linked corpus. The admission is structural only: provisions are `not_analyzed`, and no legal review is implied.

**Source and batch:**
- **Batch.** `batches/labor_L2_imss_gap_1.json` (one instrument, following `maximasa_federal_gap_1`).
- **Source.** The Cámara other-regulations index (`norma/reglamento.htm`) lists the Reglamento with its DOF publication of 2006-11-30 and no reforms. `regla/n29.pdf` is that publication (sha256 `2031b8a0…`).
- **Slug.** `reg-pmimss`, from the title's initials, as `reg-csps`.
- **Adapter.**
  - parent law LSS (`parent_instrument_id`);
  - `publication_date` 2006-11-30, set by hand;
  - glossary: Article 2, `auto_whole`, `additive_to` LSS Article 5 A, as macerf and maeba.

**Why the publication date is set by hand.** Count freezing would derive 1997-06-30. That date comes from transitory *segundo*, which cites the abrogated regulation's DOF publication. The derived value would have been wrong.

**Two narrow parser additions, each with a fixture:**
- **DOF edition page headers.** The source is a DOF edition, so its running headers carry a page number and change on every page (`Jueves 30 de noviembre de 2006 DIARIO OFICIAL (Séptima Sección) 107`). They leaked into 21 provisions.
  - The Diputados parser now treats a line with both `DIARIO OFICIAL` and a `Sección)` marker as furniture, as `standard.rs` already did.
  - No committed provision contains such a line, inline or alone, so existing text cannot change on reparse.
- **`separatorless_article_headings`.** This adapter field lists exact article labels whose heading the source prints without a separator: `Artículo 71 El médico…`.
  - Only a listed label, in the main article sequence and followed by a capitalized body, opens an article.
  - A listed label that does not open exactly one block fails the parse.
  - There is no global rule, because `artículo 27 Constitucional` would split.

**Result** (`batch run --freeze-counts`, then `link --all`; identical to a disposable probe except for timestamps):
- **Structure.** 171 articles and 2 transitories; validation is clean and the furniture audit reports 0 findings.
- **References.** All 71 resolve: 43 to LSS, 27 internal, 1 to LFT.
- **Corpus.** 468 instruments; references 35,880 → 35,951; cross-instrument edges 9,766 → 9,810. No existing instrument changed, and the rejected set is still the 8 accepted gaps.
- **Glossary.** 20 own terms, I–XX, complete, with no 5 A or 159 name repeated.
- **Usages.** 247 in all: `Instituto` 159 and `Ley` 73 from 5 A, and 15 of its own terms. Its own terms are mostly written in lower case in running text, and matching is case-sensitive.
- **Title-prefix debt.** About 15 `Ley` usages sit inside another title (`Ley General de Salud`, `Ley Federal del Trabajo`, `Ley Federal de Transparencia…`).

**Open:**
- **Temporal analysis.** None was run, because the batch path skips it.
- **Static or dynamic reference.** This 2006 regulation adopts a 5 A amended since. Whether it follows those amendments is a legal reading for JRH or the operator, as for macerf.

## 2026-10-18 — `reg-lss-maeba`: wrapped running header removed by reparse; adopts LSS Article 5 A

The operator asked for the `reg-lss-macerf` treatment to be applied to `reg-lss-maeba`. Its glossary was held because its running header sat inside fraction III.

**Cause and repair:**
- **Cause.** The Diputados parser drops a running header by comparing whole source lines with the uppercased official title. maeba's header wraps onto two lines, so it leaked into 18 provisions, the Article 2 glossary among them.
- **Repair.** The adapter now declares the two printed lines as `header_lines`. That is the existing mechanism, already tested by `configured_wrapped_running_header_never_enters_provision_text`. maeba was then reparsed with `pipeline`.
- **Same source.** The retained PDF and the extracted text match the recorded hashes (`9816ce7f…`, `d8e21cb9…`). No source changed, and the furniture audit reports 0 findings.
- **Correction (2026-10-19).** The cause above is wrong. Since 2026-10-15 (`12e0cc752`), the parser has also dropped up to three uppercase title lines directly above `CÁMARA DE DIPUTADOS…` (`publisher_title_lines`), and that rule already covers maeba's header. A disposable reparse without `header_lines` gives byte-identical provisions. The actual cause was maeba's stale July parse, so the reparse alone was the repair. The `header_lines` entry is redundant but harmless, and it stays. See the 2026-10-19 entry on the six held glossaries.

**Reparse diff:**
- **Provision ids and fields.** All 67 ids are unchanged. No field other than `text` changed.
- **Header leaks.** In 18 provisions the only change is the removed header.
- **Transitory *noveno*.** It no longer ends with the "Dado en la Residencia del Poder Ejecutivo Federal… Rúbrica." block. The current parser has classified that block as enactment apparatus since 2026-07-25 (`is_enactment_signature`), after maeba's July parse. Another 37 regulations still carry such a block from their original parse; that is recorded debt, not changed here.
- **References.** The 17 references and their targets are unchanged. Only their offsets, and the offset-bearing ids, shift by the removed text. No operator review or approval was bound to them.

**Glossary:**
- **Configuration.** Article 2 uses `auto_whole` with `additive_to: ["urn:lex-mx:federal:statute:lss:article:5-a"]`, as for macerf.
- **Terms.** 10 terms, I–X, complete. None repeats a 5 A or 159 name.
- **Usages.** 197 in all: 116 of its own terms and 81 of 5 A terms (`Instituto` 68, `Ley` 8, `Código` 5).

**Result** (`link --all`, byte-identical to a disposable probe apart from the retrieval timestamp):
- No other instrument changes. References stay at 35,880, and the rejected set is still the 8 accepted gaps.
- 18 glossaries remain held.
- The other header-leak holds (`reg-lgcc-rne`, `reg-lgeepa-maaa`, `-meia`, `-mpcca`, `-mretc`, `reg-lgs-investigacion`) are candidates for the same treatment.

## 2026-10-18 — `reg-lss-macerf` adopts LSS Article 5 A only (scoped `additive_to`)

The operator named two targets to check against LSS Article 5 A: `reg-lss-macerf` and `regla/n29.pdf`.

**`reg-lss-macerf` (in the corpus).** Article 2 reads "serán aplicables las definiciones establecidas en el artículo 5 A de la Ley del Seguro Social, así como las siguientes". It adopts 5 A, not Article 159.
- **Scoped entry.** An `additive_to` entry may now be one provision's exact id. It adopts only the terms that provision defines. A whole-instrument entry would also have brought in 159's terms.
- **Fails closed.** The entry must resolve to exactly one corpus instrument, and that instrument must contain the provision. The provision must define at least one term. Otherwise `link` fails. Whole-instrument entries (`ifpe-dcg-2021`, `itf-dcg-2018` → LRITF) behave as before.
- **Configuration.** `reg-lss-macerf` moves from auto-detection to a configured glossary: Article 2, `auto_whole`, `additive_to: ["urn:lex-mx:federal:statute:lss:article:5-a"]`.
- **No conflicts.** Its 10 own terms (II is `(Se deroga)`) repeat no 5 A or 159 name.

**Result** (`link --all`, byte-identical to a disposable probe):
- Its own 10 terms are byte-identical. Usages go from 0 to 449, all of them 5 A terms: `Instituto` 307, `Ley` 133, `Código` 9.
- The links go to `lss/markdown/articulo-5-A.md`.
- No other instrument changes. References stay at 35,880, and the rejected set is still the 8 accepted gaps.
- **Title-prefix debt:** about 7 imported usages sit inside another title (`Ley Federal del…` 5, `Ley del Impuesto…` 1, `Leyes de Instituciones…` 1). The 6 `Ley del Seguro Social` matches name the adopted referent.
- **Own terms still have 0 usages:** running text writes `patrón del campo` and the like in lower case, and the matcher is case-sensitive.

**`n29` (Reglamento de Prestaciones Médicas del IMSS, DOF 30-11-2006).**
- It is not in the corpus or the cluster-2 inventory.
- Its Article 2 adopts "las definiciones establecidas en el Artículo 5 A de la Ley del Seguro Social, así como las siguientes". It defines 20 terms, I–XX, complete, and none of them repeats a 5 A or 159 name.
- Linking it needs a canonical admission: adapter, expected counts and the furniture-audit gate.
- The `regla/` path holds the DOF publication, not a consolidated text. Before admission, a consolidated version should be looked for in the Diputados `regley` index.

**Open:**
- **Static or dynamic reference.** The adopting regulations predate later amendments of 5 A (2009, 2022, 2023, 2024). Linking to the current 5 A treats the reference as dynamic. That is a legal reading for JRH or the operator; it is not decided here.
- **Same wording elsewhere.** `reg-lss-rfar` has the same import wording and is not configured. `reg-lss-maeba` is held, and `extract_instrument_terms` returns before imports when an instrument's own terms are empty. So an import-only or held glossary adopts nothing until that early return changes.

## 2026-10-18 — Glossary completeness check: whole repeal placeholders and qualified fraction order

The operator asked for the completeness-check repair first (Codex review, `docs/glossary-g0-g2-outlier-review-2026-10-18.md`, P1 and the qualified-order P2). Both repairs are in `complete_entries` (`crates/lex-parse/src/terms.rs`), which the wider pass and the `auto_whole` style use.

**P1, repeal placeholders:**
- **A fraction counts as repealed only if the whole entry is a placeholder.** That means `Derogada.`, `(Se deroga);` or `Se deroga.`, optionally followed by amendment notes (`Fracción adicionada DOF … . Derogada DOF …`) and the `y` that introduces an inline glossary's last entry. Before, the test was only that the entry started that way.
- **The last entry is checked on its first paragraph only,** because it runs on to the article's closing paragraphs.
- **The 13 placeholder shapes found in committed glossary provisions all pass.** A table test (`wider_pass_skips_every_surveyed_repeal_placeholder_shape`) runs each one through the wider pass. Most of them occur in glossaries the established pass finds, and that pass does not use this check.
- **An entry that starts with repeal wording but goes on holds the glossary.** Example: `Derogados: Los permisos retirados`. It is neither dropped nor read as a term, and no legal finding is made about it.
- **The check for a hidden fraction heading now also runs on placeholders.** So `II. Derogada. ENCABEZADO III. Secretaría: …` holds instead of losing III.

**P2, qualified fractions:**
- **Each fraction is checked against a key:** its value plus its qualifier (none, `Bis`, `Ter`). An unqualified fraction must follow the previous number. A qualified one must repeat the number with the next qualifier.
- **So these hold:** `I, I Bis, I Bis`; `I Ter` straight after `I`; and a numeral not in standard spelling (`IIII`).
- **The printed label stays as the entry's `fraction`.**

**Result:**
- A disposable `link --all` probe changes no corpus file. References stay at 35,880, and the rejected set is still the 8 accepted gaps.
- The same 19 glossaries are held, for the same reasons.
- So none of the 94 admissions, and neither `lss` glossary, relied on either defect. The established pass is unchanged.

**Still open from the same review:**
- a persisted extraction-coverage report, so that a held glossary is distinguishable from an empty `[]`;
- source-bound per-instrument extraction profiles;
- the title-span usage filter.

## 2026-10-18 — LSS defines two term sets: Article 5 A and Article 159

**Operator ruling (2026-10-18, after the Codex outlier review).** LSS Article 5 A is the glossary. Article 159 adds to it, the way a further `se entenderá por:` does: it defines a different set of terms. Both are linked, and they must not conflict. This is an operator ruling on glossary scope, not a JRH legal approval. It covers `lss` only. The other instruments listed under "Which glossary governs" in the entry below are still open.

**Mechanism:**
- **More than one glossary provision.** An adapter's `glossary` may list `additional_provisions` (`crates/lex-source/src/lib.rs`). Their terms add to the first provision's (`extract_glossaries`). A term identifier defined in two of them fails `link`; it is not resolved by precedence.
- **Two new styles apply an auto-detection grammar to one named provision:**
  - `auto_established`: the established `se entenderá por:` pass, exactly as auto-detection applies it.
  - `auto_whole`: the wider pass, accepted only whole. An incomplete glossary fails `link` instead of being held with no terms.
- **`lss`:** 5 A uses `auto_whole`, and 159 uses `auto_established`, so 159's terms stay byte for byte.
- **Ordering against the Codex review.** Codex put the repeal-placeholder and qualified-fraction guard repairs before any new admission. 5 A has no repealed fraction and no `Bis`/`Ter`, so neither defect can reach it.

**Result** (`link --all`, byte-identical to a disposable probe):
- `lss` goes from 6 terms to 30, and from 5 usages to 946. Its 6 Article 159 records are unchanged, and no term identifier repeats.
- No other instrument changes. References stay at 35,880, and the rejected set is still exactly the 8 accepted gaps.
- **Usages by term:** `Instituto` 500, `Ley` 407, `Código` 28, `Pensión` 5 (159), `Buzón IMSS` 4, `Trabajador Independiente o por cuenta propia` 2. The other 24 terms have none, for two reasons:
  - 13 of 5 A's names are alternations (`Trabajadores o trabajador`, `Patrones, patrón o persona empleadora`), and the bare matcher needs that exact string;
  - the matcher is case-sensitive, and running text writes `beneficiarios` or `cuenta individual` in lower case.

  Aliases for alternation names are a separate schema and semantic decision.
- **Title-prefix debt in `lss`:** a scratch count finds about 72 usages inside another title:
  - 64 are `Ley` followed by another law's title, including 4 of `Ley del Seguro Social que se deroga` (the repealed 1973 law);
  - 8 are `Instituto` inside the INFONAVIT or ISSSTE names.

  The title-span filter in the Codex review remains the fix.

**Open:**
- **Article 159 is incomplete under the established pass.** It yields 6 of 8 terms:
  - VII (`Monto constitutivo es la cantidad…`) has no comma delimiter;
  - VIII (`Suma asegurada es…`) follows VII inline after a period.

  This is the copula class of `ltfccg` in the Codex review. Its fix is a per-entry term-span profile, not a general split on `es`.
- **Imports are not configured.** The `reg-lss-*` imports of 5 A (G3) remain unconfigured, and `reg-lss-maeba` is still held for a header leak.

## 2026-10-18 — Glossary auto-detection gains a wider fallback pass (glossary-detection G0–G1)

The operator approved G0 and G1 of `docs/plans/glossary-detection.md`.

**What changed** (`crates/lex-parse/src/terms.rs`, `detect_glossary`):
- **Fallback only.** The established pass (`se entenderá por:`) is unchanged and runs first. The wider pass runs only when it finds nothing, so every glossary that already had terms stays byte for byte.
- **Introductions the wider pass reads:**
  - `se entiende por:`, including inserted phrases (`se entiende, en singular o plural, por:`, `se entenderá, por:`);
  - `se debe(n) entender por:` and `se entenderá como:`;
  - imported definitions (`…las definiciones contenidas en el artículo 3 de la Ley, así como las siguientes:`, `… y en las siguientes definiciones:`, `… así como a las siguientes:`);
  - `se aplican` or `se observarán las siguientes definiciones:` and `se considera:`.
- **Skipped:** chapter-, title- and article-scoped introductions (`cpf` 401, `ccom` 1416, `ladua` 137 bis 2), and enumerations that are not glossaries (`cff` 16, `lisr` 142, `lasoc` 12).
- **Where a fraction may start:** after `;`, `:`, `.`, a line break, the conjunction `y` (the last inline entry), or an amendment note (`DOF dd-mm-yyyy`). Numerals may be written `I.-`.
- **A glossary is accepted only whole.** If it fails any of these checks it yields 0 terms and is reported as "held" with a reason, because a partial glossary would be wrong silently:
  - fractions run I, II, III… without a gap (`Bis`/`Ter` may follow their base);
  - every fraction that is not a repeal placeholder yields a plain term (no `:`, `;`, `.`, line break or quotes in it);
  - no fraction swallows another fraction heading;
  - no term repeats.

**Result** (`link --all`, byte-identical to a disposable probe):
- 94 instruments that had 0 terms gain 1,619 terms and 28,245 usages.
- No instrument that already had terms changes.
- References stay at 35,880, and the rejected set is still exactly the 8 accepted gaps.
- 19 glossaries are held with a reason:
  - running page headers or `Página 1 de 9` inside the glossary: `reg-lgcc-rne`, `reg-lgeepa-maaa`, `-meia`, `-mpcca` and `-mretc`, `reg-lgs-investigacion`, `reg-lss-maeba`, `log`;
  - gaps or typos in the source numbering: `fi-dcg-2014`, `servinv-dcg-2013`, `reg-lftr-mcsre`, and `ladua` (`lI.`);
  - a spurious `C.` from `S. A. de C. V.`: `lfca`;
  - entries without a term delimiter: `lfpccs`, `lfra`, `lfvv`, `ltfccg`;
  - a nested list: `lieps`;
  - quoted names that need the quoted style: `lsct`.

**G0 cross-check** (bare `Ley`/`Reglamento` used at least 3 times with an express definition):
- **`Ley`:** 33 of 47 targets now have the term, and 6 are held.
- **`Reglamento`:** 12 of 17 now have the term, and 3 are held.
- **The rest:**
  - the established pass already chose another glossary: `lss` art. 159 rather than 5 A, `reg-lgp`, `scap-dcg-2012`, `reg-lgs-mcsotcsh`, `lrart3-mmce`;
  - a held instrument: `socap-sofipo-dcg-2006`;
  - a quoted definition that needs the quoted style: `reg-lfmn`, `reg-lgs-msi`;
  - no Roman fractions: `reg-senado`.

**Open for the operator:**
- **Which glossary governs** where the established pass picked a later or narrower one: `lss` (ruled 2026-10-18: both 5 A and 159, see the entry above), `lgs`, `lrart3-mmce`, `reg-lgs-mpsam`, `reg-lgp`, `scap-dcg-2012`, `reg-lgs-mcsotcsh`.
- **Title-prefix usages.** The bare matcher adds at most 1,045 usages where `Ley`, `Reglamento` or `Código` is followed by a title word (`de`, `Federal`, `General`…). Some of these are the defined law's own title, so this is an upper bound. A title-aware filter remains a separate change.
- **Imports are not configured yet** (`additive_to`, G3).

## 2026-10-18 — `reg-lan` Article 1 defines its four vocables

Operator ruling: `reg-lan` Article 1 establishes the definitions for the whole regulation. It reads: Cuando en el mismo se expresen los vocablos "Ley", "Reglamento", "La Comisión" y "Registro", se entenderá que se refiere a la Ley de Aguas Nacionales, al presente Reglamento, a la Comisión Nacional del Agua y al Registro Público de Derechos de Agua, respectivamente. The committed text matches the operator's quotation.

**Implementation.** A new adapter glossary style, `quoted_respectively`, is configured only for `reg-lan`, on `:article:1` (`crates/lex-parse/src/terms.rs`):
- Each quoted vocable becomes a term, kept exactly as printed. The regulation writes `La Comisión` with a capital L 338 times and `la Comisión` 3 times, so the printed form matches the text.
- Every term's definition span is the whole defining sentence. The referents inside it are therefore not counted as usages.
- Extraction fails rather than guessing when the number of vocables differs from the number of referents.
- Auto-detection is unchanged. A census found no other instrument with this glossary form: `lic` 71, `luc` 60, `lgtoc` 80 and `reg-lgcc-rne` 2 are equivalences or other forms, not glossaries of this kind.

**Result.** `link --all` relinked only `reg-lan`. The 8 rejected instruments are exactly the accepted gaps, and references stay at 35,880. `reg-lan` gains:
- 4 terms;
- 753 usages: `La Comisión` 337, `Ley` 260, `Reglamento` 119 and `Registro` 37;
- Markdown links from those usages to Article 1. Article 1 has a single paragraph, so the links target the whole note.

Its 126 `parent_shorthand` edges to `lan` are unchanged.

**Known imprecision, not fixed here.** The shared usage matcher also matches a term that is the first word of another title. A pattern scan finds at least 40 such usages: `Ley` in another law's title, such as `Ley Federal de Derechos` or `Ley General del Equilibrio Ecológico…` (35); `Reglamento` in another regulation's title, such as `Reglamento para la Prevención…` (4); and `Registro` in `Registro de Obligaciones y Empréstitos…` (1). The scan also hit 2 correct usages, which name the defined referents in full: `Ley de Aguas Nacionales` and `Registro Público de Derechos de Agua`. The existing `Ley` terms in 94 instruments behave the same way (`reg-lfaebsp`: 5 of 56). A title-aware filter would change usages corpus-wide, so it needs its own decision.

**Superseded for `reg-lan` the same day (operator ruling: option b, reg-lan only).**
- **The rule.** A usage counts only when the vocable is written exactly as Article 1 expresses it:
  - between a matched pair of straight (`"Ley"`) or curly (`“Ley”`) quotes;
  - with its capital letter;
  - with no singular or plural variant.
- **Implementation.** It is the adapter option `glossary.usage_form: "quoted"` (`extract_quoted_term_usages` in `crates/lex-parse/src/terms.rs`). The recorded span is the word inside the quotes, so Markdown reads `"[Ley](articulo-1o.md)"`. The quotes are the condition for a match, not part of the span.
- **Result.** `reg-lan` now has 706 usages: `La Comisión` 337, `Ley` 223, `Reglamento` 112 and `Registro` 34. Every usage is quoted, no mixed quote pair occurs, and terms and references are unchanged.
- **What was dropped.** 47 bare usages:
  - 41 were the first word of another title, which is the imprecision above;
  - 2 were genuine bare uses that are no longer linked as terms: `la Ley` in Article 120 and `este Reglamento` in Article 135. The Article 120 citation still links to `lan` through its `parent_shorthand` edge;
  - 4 were other text: two `su Reglamento` naming other laws' regulations, a quoted `"El Registro"`, and a chapter heading stored inside Article 53 (`Capítulo IV Registro Público de Derechos de Agua`).
- **Elsewhere.** The other 93 `Ley` terms keep the bare matcher and remain debt.

## 2026-10-18 — Merged headings are split into their own articles; letter series accepted by equivalence

Operator rulings, continuing the 2026-10-17 entry below:

1. **Letter series are accepted by equivalence with the ordinals.** The `lgtoc` 228 a–v headings are the same per-article numbering device as the Latin ordinals. They are accepted under the same gap-free series rule, where a, b, … are the first, second, … suffix. With the ordinal rows, all 77 series-checked rows are accepted as `merged_heading` debt. The equivalence governs review only. Letter and ordinal suffixes remain distinct identifiers, because 14 instruments print both on the same base number (for example CFF `29-A` and `29 Bis`).
2. **The LFTR 180 Quintes–Septimus bodies are legally removed text,** like the bracketed placeholders. They are not structural debt.
3. **`lgs` reason corrected.** In `_closure-accepted-gaps.json`, the `lgs` gap now records that CPF 209 Quintus does not exist and that the reference is stale or wrong. The recorded error is unchanged, and the closure gate compares errors only.
4. **Split the merged headings into their own articles first,** before any citation-extraction change. Execution plan: `docs/plans/merged-heading-split.md`.

**M1 applied.** Seven instruments with retained, hash-matching source were reparsed: `lfped`, `lgamvlv`, `lgdnna`, `lgmdfp`, `lgv`, `reg-lgcfd` and `reg-lsem`. They gain 23 articles. Four of those had been lost from the corpus, not just merged: `lgmdfp` 12 Duodecies and `reg-lsem` 33 Undecies–Terdecies. Their machine-proposed count baselines are re-frozen at the new counts under this decision. The other 10 instruments wait for an operator-approved mechanism.

## 2026-10-17 — Latin ordinal qualifiers and the structural-debt candidate classes

Operator rulings on the SQLite M1 structural-debt candidates (`docs/sqlite-m1-contract-decisions-2026-10-17.md` on `db/sqlite-projection`):

1. **Latin ordinals are one qualifier series.** Words such as `Decies`, `Sextus` and `Séptimus` are the same numbering device as `Bis`, `Ter` and so on, continuing the same article number. Checked against the committed text: in all 28 groups of embedded ordinal headings, the heading has the containing article's base number and continues its series without a gap (`78 Nonies` → `Decies` … `Novodecies`; `15 Quintus` → `Sextus` … `Novenus`; `180 Quáter` → `Quintes`, `Sextus`, `Septimus`). The label grammar (`crates/lex-parse/src/labels.rs`) now ranks the 18 spellings the corpus prints beyond `Nonies`, with equivalent variants sharing a rank (`Quintes` = `Quintus`, `Sexties`/`Sexíes`/`Sextus` = `Sexies`, `Septimus`/`Séptimus` = `Septies`, `Octavus` = `Octies`, `Novenus` = `Nonies`). No stored provision number uses these words (a scan of every `provisions.json` at `fe4a53f1e` found none), so stored validation does not change. A future reparse that separates these headings now parses them, and the count re-freeze recorded above still applies.
2. **Bracketed placeholders are text that was legally removed.** Lines such as `[Artículo 11 Bis. Suprimido]` (`lgipe` 20, `lgcs` 7) are not missing structure, and they are not loaded as structural debt.
3. **Held DCG candidates wait for the DCG pass** (`oaac-dcg-2009`, `socap-sofipo-dcg-2006`).
4. **Held laws and regulations cite stale or wrong references**, pointing at articles that do not exist. The committed text confirms this for `lgs` → CPF 209 Quintus and `lftr` → LGTAIP 180 Bis. It does not confirm it for `cpf` → LIC 112 Quintus or `lmv` → LGTOC 228-B/228-H. Those headings are printed inside LIC 112 Quáter and LGTOC 228. This contradiction is open, and `_closure-accepted-gaps.json` is unchanged.
5. **Golden query G1.** Every bare `la Ley` in `reg-lan` means the Ley de Aguas Nacionales, through the definition in `reg-lan` article 1 (`"Ley"` … "se entenderá que se refiere a la Ley de Aguas Nacionales"). All 126 `parent_shorthand` edges in `reg-lan` target `lan`. `reg-lan` has no `terms.json` entry for that definition. (Superseded 2026-10-18: the definition is now extracted; see "`reg-lan` Article 1 defines its four vocables".)

Not changed: reference extraction. 51 stored edges in 12 instruments are false self-links. In each, an embedded heading's own text is read as a citation to its bare base article (for example `lfped` 15 Quintus → `lfped` article 15, and all 23 `lgtoc` 228 a–v headings → article 228). They remain as committed pending an operator decision.

## 2026-10-17 — Reference edges record their resolution rule

Every reference edge now carries an optional `resolution_rule` recording how the linker chose its target: `own_instrument_default` (no instrument named), `own_instrument_marker` (`de esta Ley`, `del presente Reglamento`), `named_instrument` (a configured name, alias or official title), `parent_shorthand` (a bare `la Ley` resolved to the adapter parent), `same_article` (a fraction cited inside its own article) and `relative_position` (`artículo anterior/siguiente`). It is provenance only: no target, status or span changed. Validation rejects a rule that does not fit its edge (an own-instrument, same-article or relative rule on a cross-instrument edge, a relative rule without the relative form, a parent-shorthand edge whose target is not the adapter parent). The relink guard lets a protected historical or approved edge gain the field but never change a recorded rule. Relink result: 455 instruments rewritten, 31,829 edges gained a rule (13,900 own marker, 4,369 own default, 4,091 named, 5,451 parent shorthand, 1,499 same article, 2,519 relative), zero edges added, removed or otherwise changed, no validation report changed; the 8 accepted-gap instruments keep their previous graphs and edges without the field (`NULL`, "not recorded"), and 4 instruments have no edges. Six stale Markdown pages in `lgmime` and `lrsic`, not re-exported after their single-instrument migrations, now match their committed edges.

## 2026-10-17 — SQLite projection plan adopted with changes

The database plan (`docs/plans/sqlite-corpus-database.md`) is adopted with these operator-approved changes: the build runs in a dedicated worktree and branch and merges into `main` after acceptance; releases live in the Git-ignored `derived/db/` folder inside the repository and are never committed, laid out as immutable content-addressed directories so they can be published online later; the 32 standards are catalogue-only; a logical content digest is the reproducibility gate and byte identity a reported observation; known structural debt, reproducibility status and accepted-gap targets are stored so "absent" is never read as "does not exist". Prerequisite: the linker records each edge's `resolution_rule` before the database build.

## 2026-10-17 — Codex review fixes (R1–R6) and the validation-warning decisions

Codex's review of the parent-law work asked for six repairs (R1–R6); they were made in `feeabc23e` and Codex verified them there. The follow-on punctuation fix (`7dc6d649c`) and the three warning changes below have not yet been reviewed by Codex. R1–R6: a configured full law title outranks its bare-parent prefix; a quoted `“Ley”` no longer bypasses the dated or anaphoric exclusions; unsupported anaphora (`en comento`, `que se cita`) and equivalent historical wording (`en vigor al…`, `que estaba vigente al…`) stay unlinked, as does a dated modifier behind an attached comma or parenthesis; `link <slug>` applies the same protected-edge guard as the corpus closure and refuses before writing, with the audited override `link <slug> --migrate-protected` (it prints every changed edge and cannot be combined with `--all`; use it only for a recorded operator migration such as LRSIC/LGMIME); and `validate` rechecks every persisted `legal_operator_review` against its edge and source text (`legal_operator_review_invalid`).

Warning decisions (operator, 2026-10-17, approving the advisor re-pass proposals 1–3):

1. **Count baselines frozen.** 248 adapters (244 Diputados and the four non-held CNBV DCGs `cue-dcg-2003`, `fi-dcg-2014`, `scap-dcg-2012`, `servinv-dcg-2013`) now carry `expected_articles`, `expected_transitories` and a `count_baseline` with provenance `machine-proposed`, taken from the committed provisions (they matched each validation report). This is a drift gate, not an audit: the counts include the roughly 77 article headings still embedded in a previous article (14 instruments: `lgtoc` 22, `lgipe` 16, `lac` 10, `lgcs` 6, `lfped` 5, `lftr` 3, `cpf` 3, `lgs` 3, `lfd` 2, `lgmdfp` 2, `lic` 2, `reg-lgcfd`, `lgdnna`, `lfdo` 1 each), so a future parse that separates them will fail the count and need a deliberate re-freeze. The four held DCGs (`cub-dcg-2005`, `cucb-dcg-2004`, `oaac-dcg-2009`, `socap-sofipo-dcg-2006`) stay unfrozen for the DCG pass.
2. **`non_numeric_article` replaced.** A suffixed article is ordered by its sort key whether or not `allow_article_gaps` is set: a same-base order slip is a `suffix_order` warning and a base that runs backwards is an `article_order` error. The 919 old notices carried no information and are gone; no new issue appeared in any validation report. One behaviour change: in an instrument without `allow_article_gaps`, a suffixed article number that does not parse under the label grammar is now an error (`unparseable_article_number`) rather than a warning, so a future reparse that separates the missing Latin-ordinal headings (`Sextus`, `Septimus`, `Octavus`, `Novenus`…) fails validation until the grammar is extended.
3. **Closure accepted-gap list.** `adapters/diputados/_closure-accepted-gaps.json` records the 8 instruments `link --all` is expected to reject (`cpf`, `lgs`, `lmv`, `lftr` and four DCGs) with their exact errors. A closure whose rejections equal the list exits 0; a new, changed or resolved rejection prints the instrument and exits 1, and a clean closure with entries still listed fails as stale. Errors are compared with multiplicity, so a new citation to an already-accepted missing target also fails. Edit the list only with a recorded decision; resolving a gap requires removing its entry.

Pre-database recheck (Codex, same day) found two more code gaps, fixed before any database work: whitespace before a comma (`de la “Ley” , vigente hasta…`) no longer bypasses the dated-law exclusion, and strict ordering now compares a plain article against a preceding suffixed one (`[1, 2-A, 2]` warns `suffix_order`; `[1, 3-A, 2]` is an `article_order` error). No stored instrument is affected: all 467 validate, with 42 federal warnings (87 with the 45 on the NOM standards, a separate model not freshly validated), and `link --all` is unchanged. The database must include or exclude the 32 standards explicitly and keep the 8 accepted gaps, machine-proposed counts and merged-heading debt distinguishable from audited facts.

Not done: reparsing the 14 merged-article instruments and the reproducibility of the roughly one-third of the corpus with no retained text (deferred by the operator); moving the 21 expected approved/historical warnings to info; the NOM standards validation.

## 2026-10-16 — Parent-law linking for regulations

A regulation's bare `de la Ley` (or the quoted shorthand `de la “Ley”` / `de la "Ley"`) now resolves to its parent law. The parent is recorded per regulation in the adapter field `parent_instrument_id` (validated in `lex-source`; at link time it must be a committed corpus instrument and not the regulation itself), never inferred at link time. The mapping was generated by script from the committed official titles — 92 by exact title, 28 by title prefix ("… en Materia de …", "… para …"), 3 by manual override after checking each against its cited articles (`reg-lfmzaah`, `reg-lgamvlv`, `reg-lmin`, where the regulation title carries an older or pluralised law name) — and cross-checked against each regulation's own definitions entry "Ley: …" where one exists (39 agree, none disagree, 81 have none). The estimated hit rate of citations against the parent's articles was 99.8% (5,292 of 5,301), and no regulation fell below the 80% threshold. Twenty-one regulations are excluded and listed in [the mapping note](parent-law-mapping-2026-10-16.md): parent not in the corpus (renamed or replaced laws), repealed parent (`reg-lftr-mcsre`), congressional rules, and free-standing regulations.

Linker rule: a bare `Ley` governs only when no title follows it (`de la Ley de Pesca` stays unlinked), only when the citation window has no internal marker (`de este Reglamento`; a mixed citation is skipped, not guessed), and never for `dicha Ley`. Configured aliases and official titles still win over it, so a later-named law keeps its own citation. A parent-shorthand edge is emitted only when its target article exists: a miss (a renumbered or repealed parent article) is neither an edge nor a rejection, by deliberate policy, so the closure gate does not turn parent drift into held instruments. Relative references (`artículo anterior de la Ley`) and transitory citations through a bare `Ley` stay unlinked. This closes the earlier `reg-lan` Article 122 case: it now resolves to LAN Article 122.

Advisor re-pass (same day): reading every added edge whose text after `Ley` began with `vigente`, `que`, `en`, or a date found 6 false links among 5,457, and the rule now also refuses a bare `Ley` followed by `vigente hasta|desde|a partir|en|durante|al` (a dated reading of the law: `reg-lisr` articles 10, 22, 23), by `que` plus a title verb (`Ley que Regula el Uso de la Fuerza…`: `reg-lopgjdf` article 7), and by `en cita` (an earlier-cited law: `reg-lfppi` articles 12 and 173). Those 6 edges were removed and nothing was added; plain relative clauses (`de la Ley que será presentada…`) and plain `vigente` still link. Result of the first relink: 118 regulations relinked, 5,457 edges added (all cross-instrument to the parent), 0 edges removed, 0 retargeted, 0 qualifier changes, validation warnings unchanged (1,502), `link --all` still rejects only the 8 accepted-gap instruments. Thirty added edges sampled across regulations were all correct. Not claimed: that a bare `Ley` can never mean a different law than the mapped parent.

## 2026-10-16 — Linker and exporter audit fixes (L1–L7); accepted gaps

Seven findings from the Codex linking-code audit are fixed with regression tests. Parser: a number after `párrafo`/`numeral` is a qualifier number, not another article (`artículo 10, párrafo 2, inciso c)` no longer links Article 2; `párrafo N` is now a paragraph qualifier) (L1); a semicolon clause that is not a continuation of the citation list no longer governs a direct citation (L4); configured aliases and internal markers match on lexical boundaries, so `Ley del Seguro Socialista` is not `Ley del Seguro Social` (L5); a parenthesized `inciso a)` after a fraction keeps both scopes (L7). Exporter: a single registry of the block anchors a note actually emits; a repeated fraction numeral (CPEUM article 20 under several apartados) emits no anchor, so links fall back to the whole article (L2); a definition link uses an anchor only if the registry has it, otherwise the whole note (L3); local versus cross-instrument Markdown paths follow canonical instrument identity, not the display short name (L6). Corpus effect of the relink: 17 false edges removed (paragraph numbers read as articles, including 3 false LGIPE links from `lgmime`), 71 citations gained fraction/inciso/paragraph qualifiers, none added; the `lgmime` approved edge gained a qualifier and was refreshed with `link lgmime`, outside the closure guard, as one recorded instrument migration.

Accepted gaps, by operator decision: `cpf`, `lgs`, `lmv` and `lftr` each have a citation to a target article that is not parsed in the committed target text (`Quintus`/lowercase-suffix headings, or an embedded invalidated article). Fixing them requires reparsing committed instruments, which would change canonical text and reset derived temporal fields (a reparse of `lic`, `lgtoc` and `lfpc` rewrites 24–31 provision texts each; `cpf` currently fails to reparse), so they stay on their previous graphs, unlinked. The four DCGs wait for a DCG pass. These are known, bounded gaps, not an open task.

## 2026-10-16 — LRSIC fraction V resolves to the current LRAF

Operator decision: the citation of "artículo 12 de la Ley para Regular las Agrupaciones Financieras" in LRSIC Article 2 fraction V refers to the current (2014) LRAF, not the historical 1990 law. This is the result the existing dating rule gives for a fraction amended after the successor (DOF 2018), and it supersedes the 2026-08-24 graph. The persisted historical edge was replaced by running `link lrsic`, a recorded one-instrument migration outside the closure guard; the guard is unchanged. The four Transitory Quinto citations (articles 33, 33-A, 33-B) remain `historical_target_unavailable`, and the adapter's historical override stays for them. The reviewer is `Operator`.

## 2026-10-16 — Legal operator review: a note on one exact citation

The stale-citation approval (`outdated_reference_approvals`) is unchanged: it records that one exact citation is outdated and lets the edge keep its cited target with status `historical_superseded_reference_outdated`. Operator review of the held instruments also named, for some citations, the article the reference should now point to. Rather than add a retargeting mechanism (new resolution status, new target semantics, wider validation), the repo adds a separate optional record, `legal_operator_review` (adapter field `legal_operator_reviews`, edge field `legal_operator_review`), that carries the reviewer's note on one exact citation: provision id, span, offsets, provision text hash, cited target, reviewer, capture time, note, decision source and its hash. It never changes the edge's target or status, must match exactly one extracted edge (changed text, moved span or a missing citation fails the run closed), cannot overwrite a previous review, and a persisted review cannot be cleared by a reparse. The reviewer is recorded as `Operator`; JRH approval is not inferred. Legal effect is not decided: a note records what the reviewer found, and no graph edge is retargeted by it.

Applied 2026-10-16 to `lgdnna`, `lgmime`, `lsar`, `lgcs`, `reg-lfce`, `lic`, `reg-lisr`, `reg-lagra-mcdets` and `lpdusf` (17 citations, each with a stale-citation approval and a note citing `docs/held-instruments-review-2026-10-16.md`). Operator corrections the same day: the `lic` note on the LFCE citation names article 3, fracción IX (Información Confidencial), not article 1; and the `reg-lan` citation "artículo 122 de la “Ley” y en la Ley Federal de Procedimiento Administrativo" is article 122 of the Ley de Aguas Nacionales, the regulation's own parent law, so it is not stale: its approval and note were removed and the linker no longer lets a law named after a quoted "Ley" take the citation. A regulation's bare or quoted "Ley" stays unlinked (the parent law is not resolved from it); linking regulation citations to their parent law is a separate open decision.

## 2026-10-16 — Official titles participate in corpus-wide closure

The curated alias table remains the only source of accent-stripped and colloquial citation names. Each committed instrument's exact official title is also a cross-instrument marker when the title contains a space and no other committed instrument uses it. `Ley de Amnistía` is shared by `lamn` and `lamni`, so that title stays unlinked. A curated alias and an official title that assign one marker to different instruments is an error. Adapter-scoped historical markers still override the global marker.

`link --all` reloads the committed corpus once, relinks every instrument, and writes a new reference graph, term graph, validation report, and Markdown only when that graph validates. Instruments that would fail validation stay as they were. This closure is canonical data. The reader database indexes it later and is not an input to it.

Seven in-corpus DCG alias keys were renamed onto the committed `short_name` so their already-curated colloquial names resolve. Deferred DCG keys with no committed instrument were not changed.

## 2026-10-16 — Federal laws and regulations enter maintenance mode; SQLite + `rusqlite` serving base

The operator selected **SQLite + `rusqlite`** on 2026-10-14 as the initial derived corpus database base, recorded here because the campaign branch that first held this record was never landed. Canonical JSON, Rust validation, source provenance and audited legal decisions remain authoritative. The database is a rebuildable, release-pinned projection, not a replacement for the corpus and not a place to infer legal approval. Its primary use is model retrieval and evidence-backed use of the corpus, alongside human reading, so stable canonical IDs, typed references, source spans, coverage and review qualifications must survive the projection. An async Rust service, hosted database, semantic retrieval and model-assisted reasoning are later capabilities. No database dependency, hosting, inference pipeline or deployment is introduced by this entry.

On 2026-10-16 the operator confirmed that, with the federal laws and regulations ingested (452 of 453 current Cámara rows plus the held-source admissions), that scope moves to **maintenance mode**: source-currency checks, reform tracking and repair, not repeat bulk ingestion. DCG ingestion remains pending. The 2026-10-14 text-first `campaign` command (1,890 lines) is not landed; it is preserved as a patch in git-ignored `archive/` and its reports as historical records under `docs/history/federal-text-first-campaign-2026-10-14/`. It can be revived if a later bulk ingestion needs staged, resumable campaigns.

## 2026-10-13 — LFJS admitted; EC4 admitted with `lfiiedb` held

The LFJS hold of 2026-10-09 is resolved. The parser now reads bare numeric
original-transitory headings (`1o.-`) only inside an established transitory
section, and the strict article-order check reads `1o`-`9o` as numbers; both
are fixture-covered. Revalidation removed spurious `non_numeric_article`
warnings from three committed instruments' `validation.json` only.

EC4 admitted `lfzee`, `reg-lfzee`, `reg-lfiiedb` and `ltpcpimcp` with audited
count baselines. Exact adapter stop markers exclude enactment signatures,
promulgation formulas and, for `ltpcpimcp`, an appended SCJN judgment.
`lfiiedb` is held out: its enacting decree nests the law beside a reform of
another law and a separate decree transitory section, and which transitories
belong to the law is an open modeling decision. `ltpcpimcp` carries a Cámara
note that SCJN declared it invalid; that is left to legal review, and its
provisions stay `not_analyzed`. No temporal model run, legal-review resolution
or reverse relinking is part of this change. Evidence:
`docs/ingestion-difficulty-log.md`.

## 2026-10-09 — EC2/EC3 source-reviewed structural integration

Fourteen reviewed instruments were admitted into main's working tree with audited
count baselines. Existing adapter options exclude exact enactment signatures and
wrapped headers; official PDF/text hashes and legal-review boundaries are preserved.
The Article 122 PDF's erroneous seeds-regulation running header is retained in the
source but excluded from provisions after identity/body verification.

Added only verified official-title phrases for the fourteen new instruments to
the shared alias table. Bounded linking adds REG-LDCMPME article 3 and LIISPCEN article 4
citations to LDCMPME article 3. Title-only mentions do not become numbered-provision
edges. Relink-induced unrelated changes in existing corpora were restored; they
require separate source review. LFJS stays held outside corpus because its five
numeric original transitories were omitted. No parser/schema change, temporal
model run or legal-review resolution is part of this integration. Evidence:
`docs/ec2-ec3-integration-2026-10-09.md` (archived 2026-10-16 to the git-ignored `archive/`; recover from git history).

## 2026-10-07 — Article dates: original commencement for unamended articles, reform commencement otherwise

An article's start date is exactly one of two knowable values: the original
instrument commencement, when the article was never amended, or the
commencement of the reform that last touched it. The 2026-10-06 repair cleared
both kinds because it could not tell them apart, which discarded correct
dates. `temporal-derive-v3` separates them by amendment evidence: an article
with no `DOF dd-mm-yyyy` note and no footnote mark takes the resolved original
commencement; an amended article stays unset, because its date is its
reform's commencement and needs that reform's own transitories.

`derive-temporal --repair-dates-only` restored 10,208 dates in 130
instruments (receipt: `docs/temporal-date-restoration-2026-10-07.json`).
Structured comparison against the previous commit showed only `effective_from`
changed, only from null to the original commencement, and no amended article
was touched. Before restoring, every instrument was checked for consistency
between its recorded reform date and its amendment evidence: 130 were
consistent and none recorded a reform with no evidence in its text.

An unmarked article is not proof of an unamended article if the parser missed
a note. Cross-checking against the publisher's decree-by-decree reform list is
the audit for that risk and is not automated. A commencement that depends on a
future uncertain act, and partial repeals, are not modelled yet; see
`docs/plans/partial-repeal-and-conditional-commencement.md`.

## 2026-10-06 — Enforce temporal model boundaries and remove unsupported article dates

Raw temporal model responses must pass the embedded model-output v2 JSON Schema
before canonical deserialization. The model cannot assert external verification
or supply reviewer-only fields. Import also checks that the request's instrument
metadata and complete evidence ID/text-hash set match the current corpus.
Review resolution requires matching current source evidence and a matching live
determination; archived evidence-version IDs cannot mutate current conclusions.
When evidence disappears from a rerun, pending and resolved review records are
archived intact instead of dropped. These repairs enforce existing contracts;
they do not change the model schema or infer reviewer approval.

Original instrument commencement is not the effective date of every article's
current wording. `temporal-derive-v2` retains the separate commencement result
but leaves article `effective_from` unknown. Existing deterministic status
classification is unchanged; this is not a historical-version reconstruction.
The Rust `derive-temporal --repair-dates-only` migration clears only dates equal
to resolved original commencement on machine-accepted, deterministic articles
with effective/future-effective status, no end date, and no transitory effects.
Model and lawyer review states are excluded.

Against baseline `8031ea11`, the migration removed 15,974 dates in 132 instruments.
The per-instrument receipt is `docs/temporal-date-repair-2026-10-06.json`.
Structured comparison confirmed that only those date fields changed; source
text, manifests, hashes, references, and review records were preserved. The
canonical serializer also removes some pre-existing trailing newlines.
Re-export corrected one stale LPUE transitory Markdown status from `repealed`
to its already-canonical `unknown`; no new legal determination was made.

Regression coverage includes schema rejection, stale/archived resolution,
omitted pending/resolved history, and later-added article dates. Isolated CLI
checks reject invalid model claims and stale requests without corpus mutations,
accept a valid response into the review queue, and reject archived resolution.

## 2026-08-24 — Historical same-title targets and nested omnibus transitories remain distinct

**Decision: a citation to a superseded law is not a citation to a later law
that reused its title.** LRSIC was published on 15 January 2002. Its original
Transitory QUINTO repeals Articles 33, 33-A, and 33-B, plus the general rules
for credit-information societies, of the LRAF then in force. The official
[abrogated-law record](https://www.diputados.gob.mx/LeyesBiblio/abro/lraf.htm)
identifies that LRAF as the law published 18 July 1990 and abrogated 10 January
2014. The [current LRAF](https://www.diputados.gob.mx/LeyesBiblio/pdf/LRAF.pdf)
is a new law issued by Article Fifty-First of the 2014 omnibus decree; Article
Fifty-Second abrogates the 1990 law. Publication order therefore controls the
target identity.

The canonical reference status `historical_target_unavailable` preserves the
source span and the stable historical instrument identity
`urn:lex-mx:federal:statute:lraf-1990` without fabricating a live link to the
2014 LRAF. An adapter-scoped mapping overrides the global same-title alias for
LRSIC. Validation records a non-blocking warning while the historical target
is absent and becomes an error if that target is later added but the edge is
not relinked. Markdown and bundles continue to link only `resolved` edges.
This additive enum value is backward-compatible within schema 0.1.0; no
existing record changes meaning.

**Parser consequence: preserve both enactment layers of an omnibus decree.**
The current LRAF's own transitional regime is Article Fifty-Second. Ellipsized
Articles Fifty-Third and Fifty-Fourth are sibling decree articles, after which
the overall omnibus decree has its Sole Transitory. The Diputados parser now
recognizes ordinal decree articles through the fifties and permits that second
transitory heading only after an ellipsized plural decree-article wrapper. A
focused fixture prevents the exception from swallowing later reform-decree
transitories. The refreshed LRAF corpus contains 193 articles and these two
original transitories; its reform appendix separately retains the 2018 LRITF,
2024 administrative-procedure, and 2025 CNPCF decree transitories as temporal
evidence.

Finally, a clause that *creates* a repeal is not thereby repealed itself.
Initial status detection now accepts only standalone source markers such as
`(Se deroga).` or `Derogado.`; operative `Se deroga …` and `Se derogan …`
clauses start `unknown`. The same deterministic correction was applied to 61
previously parsed original transitories and their generated Markdown. No
lawyer-verified decision was overwritten.

## 2026-08-24 — Positional PDF furniture audit is an automatic structural-ingestion gate

**Decision: run the positional PDF audit after every PDF-backed parse, not
as a manual operator command.** The audit comes from the previously evaluated
REPO-019 / AnyDoc ecosystem, but uses only its underlying
pdf-inspector = "=0.1.7" positional reader as a second opinion. It does
not replace Poppler extraction, alter official source bytes, or rewrite
canonical text.

The parsing stage now audits every main and annex PDF consumed by a parser. It
compares header/footer bands that recur on at least 95% of pages against the
candidate provisions the parser just emitted. A positional band alone is
evidence, not a defect; a band whose text has survived into a provision is a
potential contamination or parser-boundary failure. An isolated match is
reported but does not stop the run because a law's running title can also be
a legitimate citation. The run stops before writing the candidate corpus only
when the same header/footer text reaches three distinct provisions. Repeating
body-zone text is reported separately and never treated as furniture
automatically.

This makes the audit run for ordinary pipeline, parse, and every batch-run
instrument. The standalone audit-furniture command remains a read-only
diagnostic for investigating a retained PDF, while the batch report will
carry the failed instrument and its audit error.

**Retained-source baseline, 2026-08-24.** A read-only sweep of all 100
retained PDFs (7,134 pages) found recurring positional furniture in 77
documents: 411 bands (342 headers, 69 footers; 330 static and 81 dynamic).
Seven documents also have body-zone repeats, which remain report-only because
forms and tables can legitimately repeat there. Four one-page PDFs produce no
recurrence finding: a recurrence needs at least two pages. The sweep did not
reparse or mutate corpus data and found no confirmed gate failure. A future
parse is blocked only by the documented three-distinct-provision rule; do not
add broad string filters from this inventory. The reviewer-facing finding and
targeted refinement queue live in the dedicated Spearhead audit workspace.

**EC1 refinement, 2026-08-24.** LFCE demonstrated that a recurring running
title can also be cited substantively in several original transitories. The
admission comparison therefore requires the normalized positional band to
survive as its own double-newline-delimited parser block, rather than merely
as a substring of provision prose. The focused fixture covers both a leaked
standalone block and an embedded legal citation. This narrows the gate's
evidence without changing extraction, source text, or the three-provision
blocking threshold.

## 2026-08-24 — Deterministic temporal-status triage, three-tier provenance, and a recorded public-review-submission design

**A vault re-export on 2026-08-23 revealed that 30,153 provision notes
carried `temporal_status: effective` with `review_status: not_analyzed`** —
the July Python-era render had asserted provisions were in force wholesale,
never having run any temporal analysis. Corrected to `unknown`
(`0854cd2fe`), which raised the operator's real question: how does a
38,312-article corpus get an honest in-force status without either manual
review of every provision (not tractable) or trusting an unverified blanket
assertion again (the mistake being corrected)?

**Decision: distinguish three tiers of provenance on every provision's
temporal status, and never blur them.** This was not a new mechanism to
build — `lex_core::Basis` (`SourceText`, `DeterministicRule`, `LlmInference`,
`LawyerVerified`, …) and `ReviewStatus` (`NotAnalyzed`, `MachineAccepted`,
`ReviewRequired`, `LawyerVerified`) already modeled exactly this, already
schema-validated, already exercised by the LLM temporal-analysis pipeline.
What was missing was a second *producer* alongside the LLM path — one that
classifies from plain deterministic code, for the specific, narrow question
of whether an article is currently in force, and records that provenance
honestly rather than defaulting everything to `Unknown`. The three tiers, as
the operator specified them:

1. **Machine deterministic analysed** — `basis: deterministic_rule`,
   `review_status: machine_accepted`. New in this entry.
2. **Human reviewed** — `review_status: lawyer_verified`. Pre-existing;
   unaffected. JRH remains the reviewer of record.
3. **Unknown pending human review** — `review_status: not_analyzed`
   (untouched) or `review_required` (an LLM-flagged transitory item already
   in a reviewer's queue). Everything this pass could not classify
   confidently stays here, honestly, rather than being guessed at.

**Implementation: `lex_parse::temporal_derive`
(`crates/lex-parse/src/temporal_derive.rs`).** Two rules, both requiring
unambiguous evidence, both restricted to provisions still at
`not_analyzed` — this is a hard invariant, checked in code and by a
dedicated test (`already_analyzed_articles_are_never_touched`): the pass can
only ever advance a provision out of the untouched state, never revise a
prior human or model determination.

- *Repeal* promotes the provenance of a provision `initial_temporal_status`
  already classified `Repealed` at parse time — the classification is
  unchanged, only its `basis`/`review_status` advance.
- *Commencement* resolves an instrument's own entry-into-force clause from
  its ordinary transitorios, requiring exactly one unqualified match against
  four known-safe shapes (day-after-publication, same-day, literal date,
  calendar-day count), and applies it to every remaining non-repealed
  article as of-today status. Deliberately does not track per-article
  amendment provenance — for a corpus holding only already-enacted,
  currently consolidated text, any article present today has necessarily
  taken effect by now regardless of which reform added it, so asserting
  `Effective` without a precise `effective_from` is honest, where asserting
  a specific `effective_from` without amendment tracking would not be.

Wired into `pipeline` (after `validate`, before export) so every future
ingestion gets this automatically, and available standalone as
`lex-mex derive-temporal <instrument>` for backfill. Full skip taxonomy,
category-by-category slug lists, and the two regex gaps found and fixed
while reviewing this session's own backfill output are recorded in
`docs/ingestion-difficulty-log.md`'s `temporal-derive` entry — that log is
the intended home for addressing outlier categories one at a time, per the
operator's own stated workflow ("check outliers as they come up, add a
fixture").

**2026-08-24 backfill, all 204 committed federal instruments** (NOMs
excluded; separate standards boundary, no `Provision` model): 132 resolved
commencement, 67 skipped (six named categories), 5 with no ordinary
transitories. 4,230 articles promoted `repealed` provenance; 15,974 promoted
`effective`/`future_effective`. All 204 re-validated clean.

**Recorded, not built: public review submissions.** The operator noted this
repository is public and raised, for the record rather than for this pass,
that external readers should eventually be able to submit a review — a
proposed correction or confirmation of a temporal or legal determination —
without being able to write it into the corpus themselves. The intended
shape, sketched here so a future session does not start from nothing:

- A submission is a structured file (schema stub added:
  `schemas/external-review-submission.schema.json`), not free-form
  commentary — it names the provision, the proposed `ReviewResolution`, and
  supporting citation text, the same shape a reviewer's own resolution
  already takes.
- It arrives as a pull request against a `review-submissions/` directory,
  never as a direct write to `corpus/` or `review-queue.json` — a submission
  is data, not yet a decision, exactly the distinction `Basis`/`ReviewStatus`
  already draw between a machine proposal and a verified determination.
- **JRH (or the recorded reviewer of record) must explicitly accept a
  submission before it can affect any provision's status** — the existing
  `review resolve` audited workflow, with the submission as its input
  instead of an LLM determination. No submission is ever auto-applied; this
  is a vandalism/misreading safeguard, not an optimization to remove later.
- Out of scope for this entry: hosting, notification, and identity/authorship
  verification for submitters — those need their own decision once there is
  a concrete channel (GitHub Issues/PRs are the natural first channel, since
  the repository is already there).

## 2026-08-23 — Vault-reconciliation recovery: ten orphaned instruments, a letter-suffix-plus-qualifier parser fix, and a new hold-out class

A reconciliation between this repository and the Obsidian vault found ten
instruments present only as pre-2026-07-11 Python-era vault renders, with no
committed corpus record and no entry in the cluster-2 prepared inventory:
`lisr`, `reg-lisr`, `liva`, `reg-liva`, `lieps`, `reg-lieps`, `lacp`,
`lgoaac`, `lrascap`, `luc`. Four of them (`lisr`, `lieps`, `liva`, `lgoaac`)
were named in the vault's own cluster-2 ingestion note among the 18
instruments whose Python renders folded letter-suffixed articles into their
parents, so where they existed at all they were known-defective. **All ten
admitted clean.**

**The recovery was recorded in the owning cluster-1 manifests, not a new
batch.** A first pass created a separate `recovery_RC1_...` manifest, which
`all_committed_batch_manifests_deserialize` correctly rejected: a slug may
appear in exactly one batch manifest, and `tax_T1_core`,
`financial_F4_popular_savings` and `financial_F5_credit_auxiliaries` already
own these instruments — the same manifests that own committed instruments
like `cff`. The ten had simply never been run through the Rust gate. `lieps` moved to `tax_T1_core`'s
`blocked` list and back once the digraph rule landed, leaving `lex-source`'s
frozen unique-slug baseline at 200.

**`lacp`, `lgoaac`, `lrascap` and `luc` re-pointed to Cámara de Diputados.**
All four are federal laws (`type: ley`) but were recorded against CNBV's
Normatividad mirror with `adapter: "cnbv"`, a cluster-1 grouping artifact
from listing them beside their DCGs; `adapter scaffold` refuses to generate a
cnbv config, which surfaced it. Every one of the other committed federal laws
uses the Diputados consolidated text. Operator decision: re-point all four at
`diputados.gob.mx/LeyesBiblio` (all four pdf and ref URLs returned HTTP 200 on
2026-08-23) and use the `diputados` adapter.

**Parser fix — `article-letter-suffix-qualifier-heading`.** LIVA writes
`Artículo 1o.-A BIS.-`. `heading_letter_suffix` accepted a suffix letter only
when the next character was `.` or `-`; a space follows it here, so the suffix
was rejected, the heading collapsed to `1o`, and `A BIS.-` was stranded in the
body — a duplicate `article:1` that cascaded into 87 validation issues. The
grammar already handled the hyphen form (`32-B Bis`) and qualifier matching
was already case-insensitive, so `BIS` vs `Bis` was never the discriminator;
the untested shape was ordinal mark, `.-`, letter suffix, *then* qualifier.
The fix is additive — every previously-matching input still matches — and an
audit of all 226 then-committed instruments for a stranded letter-plus-qualifier
found zero hits, so no committed instrument carried the defect. Fixture
`fixtures/diputados/article-letter-suffix-qualifier-sample.txt`, test
`letter_suffix_heading_keeps_its_qualifier`. The same branch recovered LIVA's
`18-H BIS`, `18-H TER`, `18-H QUÁTER` and `18-H QUINTUS`.

**Article suffixes are tokens, not characters — the traditional digraphs are
single letters.** LIEPS numbers its article-26 series through the traditional
Spanish alphabet, in which `LL` is its own letter between `L` and `M`
(`26-L`, `26-LL`, `26-M`, `26-N`, `26-Ñ`). `Ñ` was already handled; `LL` could
not be while a suffix was an `Option<char>` ranked by position in a
character string.

Held out on first pass, then admitted the same day on the operator's call to
**encode the rule rather than the instrument**, on the reasoning that the shape
will recur in other older statutes. `SUFFIX_LETTERS` becomes `SUFFIX_TOKENS`,
an ordered table carrying `CH`, `LL` and `RR` at their traditional collation
positions; `Component.letter` becomes `Option<String>`; `letter_rank` takes
`&str`; and a shared `labels::match_suffix_token_at` matches longest-token-first
and case-insensitively, so `LL`/`Ll`/`ll` are one suffix, `26-LLA` still yields
no suffix, and both the grammar and the Diputados heading path use the same
matcher.

Blast radius measured before the change: zero committed instruments carried a
digraph-numbered article or the stranded-digraph body signature. Inserting the
digraphs shifts absolute letter ranks but preserves relative order, and slugs
derive from the raw label rather than the rank, so no committed slug or
ordering changes — confirmed empirically by re-parsing `liva` and `reg-lisr`
to byte-identical output.

**Ordinal-mark cases.** `liva`, `lacp` and `lgoaac` needed the reviewed
`allow_article_gaps: true` adapter setting — the familiar case where
`1o`–`9o` fail `parse::<u32>()`, never advance `expected_number`, and every
later plain numeral then collides with a stuck expectation of 1. No parser
change. `lisr`, `reg-lisr`, `reg-liva`, `reg-lieps`, `lrascap` and `luc`
admitted without it.

Totals move to 236 instruments (204 federal + 32 NOMs), 38,312 articles,
1,543 original transitories, 20,792 reference edges, 0 unresolved; 236/236
`validation.json` reports valid with 376 non-blocking warnings.

## 2026-08-06 — FI3 admitted; preserve a legacy regulation identity and recognize the narrow `LEY del ...` reform heading

FI3 (`batches/financial_FI3_seguro_rural_convenios.json`, normalized from
`prompts/cluster-2-batches/lex-mex-cl2-batch-FI3.json`) completes the prepared
Domain FI sequence and adds all eight entries: `lfaar`, `lfgfaga`,
`reg-lfgfaga`, `reg-lfif-art95`, `reg-lsar`, `lmccbdc`, `lccaif`, and
`lccbid`.

**Preserve `reg-lfif-art95` as the official source identifies it.** The
Cámara regulations index still lists the regulation as vigente, but its title
and text expressly regulate Article 95 of the abrogated Ley Federal de
Instituciones de Fianzas, which is absent from the committed corpus. The
adapter and canonical identity therefore remain tied to LFIF. This ingestion
does not substitute the current LISF, manufacture a parent-law reference, or
infer whether the regulation continues to produce legal effects; those are
legal-temporal conclusions outside structural admission.

**Recognize only the confirmed bare-law-title form.** LFGFAGA's reform
appendix places `LEY del Banco de México.` after a FE DE ERRATAS entry and
before the Banco de México transitories. The former parser recognized only
`LEY que ...`/`Ley que ...` reform-act headings, so it retained the preceding
act identity and failed before canonical output. `is_reform_ley_heading` now
also recognizes the exact publisher form `LEY del `, backed by a focused
fixture that requires the 1993-12-23 Ley identity for PRIMERO and DECIMO
OCTAVO. Broader bare titles such as `LEY de ...` and `LEY General ...` remain
outside the rule; the earlier `ccom`/`lac` hold-out analysis is not silently
broadened.

The shared-parser blast radius was checked in a disposable detached worktree
against every already-available extracted source containing the new prefix.
LIC's provisions, references, terms, and counts remain byte-identical; only
its reform evidence changes, correctly assigning the Banco de México
transitories to the 1993-12-23 Ley and disambiguating the following same-day
decree. LIC was not re-derived or recommitted in FI3; that evidence-only
migration requires its own reviewed reparse. CCF continues to fail with its
pre-existing duplicate 1975-12-30 evidence id before and after the change.

**Adapter boundaries remain source-specific.** Exact wrapped-header lines
remove Cámara page furniture from `reg-lfif-art95`, `lmccbdc`, `lccaif`, and
`lccbid`; exact enactment-signature markers remove only post-provision
apparatus and the associated spurious CPEUM Article 89 edges. The familiar
`allow_article_gaps: true` setting is used only where the official labels
switch from `1o.`/`1º` ordinals to cardinal numbering or include reviewed
suffixes. Seventeen retained warnings are all expected ordinal-label notices
(`reg-lfif-art95`: 7; `lmccbdc`: 10).

**Verification.** The eight-instrument closure passed and adds 337 articles,
23 original transitories, 110 resolved references, 49 defined terms with
1,280 usages, and 52 reform-transitory evidence records. The frozen
operational inventory advances from 38 to 39 manifests and from 192 to 200
unique instrument slugs. Formatting, clippy with warnings denied, all 154
workspace tests, and both audited baseline validators (`lritf`,
`ifpe-dcg-2021`) passed.

## 2026-08-05 — FI2 admitted; five more ordinal-numbering hits and six reviewed enactment-signature boundaries

FI2 (`batches/financial_FI2_banca_desarrollo.json`, normalized from
`prompts/cluster-2-batches/lex-mex-cl2-batch-FI2.json`) continues Domain FI
and adds all six prepared development-bank statutes: `lonf`, `loshf`, `lobb`,
`lobnce`, `lobnosp`, and `lobnefaa`.

**Sixteenth through twentieth confirmed hits of the `1o.`–`9o.` ordinal
case.** `lonf`, `loshf`, `lobnce`, `lobnosp`, and `lobnefaa` use the familiar
single-digit ordinal abbreviations before switching to cardinal numbering at
Article 10. Each received the same reviewed `allow_article_gaps: true`
adapter setting; no parser code or default changed. `lobb` stayed on the
strict default (`false`): its only non-numeric labels are Articles 8 Bis, 25
Bis, and 37 Bis, each correctly positioned between consecutive cardinal
articles, so its three `non_numeric_article` warnings are non-blocking and do
not interrupt the strict numeric run.

**All six sources needed an exact original-enactment boundary.** Each older
Cámara consolidation appends the congressional signatures and presidential
promulgation formula directly after its final original transitory. Without an
adapter-scoped `main_document_stop_markers` entry, the parser correctly
preserved that source text but attached the furniture to the final transitory
and emitted a spurious CPEUM Article 89 reference. The six exact legislative
date markers remove only that non-canonical apparatus. This is the same
reviewed source-specific correction used on earlier Cámara instruments, not a
new shared parser rule; each rerun reduced its reference count by exactly one
and left the substantive final transitory intact.

**Verification.** FI2's bounded closure relinked, validated, and republished
all six instruments. Together they add 249 articles, 38 original
transitories, 144 resolved references, four defined terms with 241 usages,
and 124 reform-transitory evidence records. Validation has zero errors and
three explained warnings, all `lobb` suffix notices. The frozen operational
inventory advances from 37 to 38 manifests and from 186 to 192 unique
instrument slugs.

The full required gate passed: formatting clean, clippy clean with warnings
denied, 152 workspace tests passed, and both audited baseline validators
(`lritf`, `ifpe-dcg-2021`) reported zero issues.

## 2026-08-02 — FI1 admitted, opening Domain FI; a new hold-out class (stale cross-reference to a repealed provision)

TX1–TX3 closed Domain TX; FI1 (`batches/financial_FI1_autoridades_pagos.json`,
normalized from `prompts/cluster-2-batches/lex-mex-cl2-batch-FI1.json`)
opens Domain FI (financiero, segunda ola), adding `lsp`, `lmeum`, `lcmm`,
`ltfccg` — four of the five prepared entries.

**Twelfth through fifteenth confirmed hits of the `1o.`/`1º` ordinal
case**, all four admitted instruments. Same reviewed
`allow_article_gaps: true` fix, no parser change — this is now the single
most common review gate hit across the whole AD/TX/FI program (15 of the
program's total instrument-level review cases).

**`lcnbv` held out — a new failure class, not seen before this batch.**
`lcnbv`'s article 15 cites "artículo 16 Bis 7 de la Ley del Mercado de
Valores" (LMV) — a real citation in the source text, not a parser
artifact. LMV is already committed to the corpus, but has no `16 Bis`
series in either its committed `provisions.json` or a freshly, independently
refetched `LMV.pdf` (current source runs `Artículo 16.-` straight to
`Artículo 17.-`). So this is neither a wiring gap (the target instrument
exists) nor a parsing gap in either instrument (both extract their real,
current source text correctly) — `lcnbv` cites a provision of LMV that
has since been repealed or renumbered, and the citing text was never
updated. `validate_reference_target` fires `unresolved_internal_reference`
exactly as the 2026-07-12 reference-graph rule intends (cross-instrument
unresolved edges are emitted, not silently dropped, so a genuinely
missing wiring target still fails validation) — but that rule was written
for the wiring-gap case, not a verified dead citation to an already-
ingested instrument's repealed provision. Resolving it by dropping the
edge would extend the same-instrument "dangling link is dropped" rule to
cross-instrument edges on a case-by-case, unreviewed basis; held out
instead pending a reviewed disposition. New failure class
`stale-cross-reference-to-repealed-provision` logged; full report
`docs/ingestion-difficulty-log.md`.

**Verification.** All four admitted FI1 instruments validate clean and
reverse-link with 0 unresolved references (83 new edges; 108 new
articles, 26 new original transitories). `all_committed_batch_manifests_deserialize`'s
frozen counts updated again (36→37 manifests, 182→186 unique instrument
slugs — `lsp`/`lmeum`/`lcmm`/`ltfccg` admitted, `lcnbv` in `blocked` not
`instruments`). Full workspace gate: fmt clean, clippy clean, 152 tests
(no new tests this batch — no parser change, an adapter-setting fix plus
a hold-out).

## 2026-08-01 — TX3 admitted; eleventh `1o.` hit, one genuine article-gap, a second nested-law hold-out

Continuing Domain TX after TX2. TX3
(`batches/tax_TX3_impuestos_aduanas.json`, normalized from
`prompts/cluster-2-batches/lex-mex-cl2-batch-TX3.json`) added `lfisan`
and `reg-ladua` — two of the three prepared entries.

**Eleventh confirmed hit of the `1o.`–`9o.` ordinal case.** `lfisan`;
same reviewed `allow_article_gaps: true` fix, no parser change.

**A genuine, isolated article-number gap, not the ordinal case.**
`reg-ladua` failed with "expected article 88, found 89" — article 88's
numeral does not appear anywhere in the source text at all (confirmed
directly: `grep` for "Artículo 88" against the extracted text returns
nothing, while 87 and 89 are both present), not even a repealed-fraction
placeholder. This is exactly the case `allow_article_gaps: true` and the
label-aware ordering path exist for; same reviewed fix, no parser change.
Recorded separately from the ordinal class because the mechanism differs
(a genuinely missing numeral, not a non-numeric-suffix counter freeze) —
both route through the same adapter setting, but conflating them would
obscure that this repo has now seen at least two distinct causes of the
same review gate firing.

**`lisipl` held out — second confirmed instance of
`nested-law-in-enacting-article` the same day.** Same class as TX2's
`lcmopfih`, found independently in the very next batch: `lisipl`'s source
PDF is `ARTICULO NOVENO` of an unrelated 1968 omnibus decree, with the
decree's other articles elided and its own numbering resuming
(interleaved with the nested law's own transitorios) after the nested
law's content ends. Structurally messier than `lcmopfih` — the outer
decree doesn't just wrap the nested law, it continues after it — so held
out under the same policy rather than attempted as a one-off patch.
Full report: `docs/ingestion-difficulty-log.md`.

**Verification.** Both admitted TX3 instruments validate clean and
reverse-link with 0 unresolved references (54 new edges; 295 new
articles, 13 new original transitories). `all_committed_batch_manifests_deserialize`'s
frozen counts updated again (35→36 manifests, 180→182 unique instrument
slugs — `lfisan`+`reg-ladua` admitted, `lisipl` in `blocked` not
`instruments`). Full workspace gate: fmt clean, clippy clean, 152 tests
(no new tests this batch — no parser change).

## 2026-08-01 — TX2 admitted; two genuine parser gaps (reform-act heading recognition, repealed-fraction glossary collision), one hold-out

Opening Domain TX after AD1–AD4. TX2
(`batches/tax_TX2_ingresos_presupuesto.json`, normalized from
`prompts/cluster-2-batches/lex-mex-cl2-batch-TX2.json`) added `lfd`,
`lgdp`, `reg-lfprh` — three of the four prepared NEW entries. `lif-2026`
and `pef-2026` stay explicitly blocked (unchanged, prior deferral).
`lcmopfih` is newly held out (see below).

**Fifth through tenth confirmed hits of the `1o.`–`9o.` ordinal case.**
`lsat`, `lfdc`, `lfpca` (TX1, same-day) and `lgdp`, `lfd` (TX2) all hit the
same review gate as the eight prior instances; same reviewed
`allow_article_gaps: true` fix each time, no parser change. Ten
independent statutes now.

**Two genuine parser gaps, found while landing `lfd` and `lgdp`.**

1. `lgdp`'s first reform decree is titled "Decreto de reformas y adiciones
   a diversos artículos..." — a form `is_decree_heading` deliberately did
   not match (its own docstring warns the bare `Decreto de ` prefix is
   common inside ordinary prose and risks false positives). Widened to
   also accept the narrower, safe `Decreto de reformas ` prefix.
2. `lfd`'s reform appendix records some reforms as a *Ley* rather than a
   *Decreto* or *Reglamento* ("LEY que establece, reforma, adiciona y
   deroga..."), a form `reform_act_heading_kind` had no case for at all.
   Added `is_reform_ley_heading` (`Ley que `/`LEY que `). Recognizing the
   heading text alone wasn't sufficient — `lfd`'s actual failure was the
   heading getting silently merged into an unrelated preceding correction
   note across a page break, leaving `act_kind` unset and crashing on the
   first transitory that followed. The first attempt fixed this by adding
   the new heading to `is_immediate_structural`'s unconditional flush list,
   the same place `is_decree_heading`/`is_reform_regulation_heading`
   already live — but "Ley que" is ordinary Spanish legal prose ("esta Ley
   que hayan quedado firmes...", "en términos de la Ley que la rige..."),
   unlike the much rarer "Decreto que"/"REGLAMENTO de" forms, and matching
   it unconditionally corrupted body text (a spurious paragraph break)
   wherever a PDF line wrap happened to start with it — caught before
   commit by the regression check below (`lic`, `lmv`, `ltosf` all showed
   silent mid-sentence splits with no error). Corrected to follow the
   `amendment_mark_end` precedent already in `normalized_blocks`: scoped to
   `crossed_page_furniture` only, so the extra split fires exactly where
   `lfd`'s real failure occurred and nowhere else. Two regression tests
   (`a_decreto_de_reformas_heading_is_recognized_as_a_decree_heading`,
   `a_ley_heading_split_from_prior_content_by_page_furniture_is_recognized`)
   plus fixtures reproducing both real failures directly.

**A third, deeper parser gap, found only by regression-testing the first
two fixes against the existing corpus, not by TX2 itself.** `reg-lfprh`
crashed on `duplicate defined-term identifier` — its glossary article has
two fractions independently repealed years apart (DOF 2009, DOF 2025),
both retaining the standard placeholder text "Se deroga." rather than a
real definition; `detect_glossary_terms` extracted that placeholder as if
it were the term, so both fractions collided under the same id. Skipping
placeholder fractions (`Se deroga`/`Derogada`/`Derogado`, case/paren-
insensitive) stopped the crash but exposed a second, worse effect:
`glossary_delimiter` derives the term/definition separator from
`starts[0]` unconditionally, and a period-terminated placeholder in
fraction 1 poisoned that choice for the *entire* glossary — every other,
properly colon-delimited fraction silently found no delimiter and was
dropped. `reg-lfprh` went from crashing to committing with 0 defined terms
extracted, not 26. Fixed by choosing the representative delimiter from the
first entry that is *not* a repealed placeholder, not unconditionally from
the first entry found. New test
`repealed_fraction_placeholders_are_skipped_not_collided` asserts both the
collision and the delimiter-poisoning are gone.

**`lcmopfih` held out.** Its source PDF is not the named law's own primary
text: it is one article of an unrelated 1990 omnibus decree that enacts
the real law verbatim as that article's body, with the decree's other
articles elided. The parser correctly extracts the wrapper article but has
no notion of a law nested inside a single enacting article, so the nested
law's real 15 articles are absorbed as body text rather than split into
provisions. New failure class `nested-law-in-enacting-article`,
full report in `docs/ingestion-difficulty-log.md`. Per the 2026-07-28
hold-out-and-flag policy: not compiled with the defect present, moved to
`blocked` in the batch manifest.

**Regression-checked, not assumed, against the rest of the committed
corpus — and this is what caught the false positive above.** Both
`is_reform_ley_heading` (block-splitting affects every `diputados`-parsed
instrument, not just the reform appendix) and the glossary-delimiter fix
could plausibly move already-committed data. Isolated the effect from
unrelated pre-existing corpus/parser drift (a real, separate condition —
re-parsing already-committed `cpeum` today, with *zero* of this session's
changes applied, already differs from its committed data) by comparing
fresh `fetch`+`extract`+`parse` output before/after each version of this
session's parser changes, in a disposable git worktree, for every
committed instrument whose text contains `Ley que`/`LEY que` or a
`se entenderá por:`-style glossary (137 candidates). The first-attempt fix
changed 5 of 126 comparable instruments (`ladua`, `lic`, `linfonavit`,
`lmv`, `ltosf`) — `ladua`/`linfonavit` corrected reform-evidence
attribution as intended, but `lic`/`lmv`/`ltosf` had body-text
mid-sentence corruption, the false positive that forced the
`crossed_page_furniture`-scoped correction. Rerun after the correction: 2
of 128 comparable instruments changed (`ladua`, `linfonavit` — same count
of evidence entries before and after, `provisions.json`/`references.json`
untouched in both), each a transitory that a prior, unrecognized "Ley
que ..." decree heading had been misattributing to a nearby *different*
decree under a synthetic `section-N` qualifier; now correctly dated and
labeled under its own Ley heading. `lic`/`lmv`/`ltosf` came back clean.
Also surfaced nine already-committed instruments (`ccf`, `ccom`,
`cff`, `cpf`, `lac`, `lamp`, `lcf`, `lfcpq`, `lins`) that already fail a
fresh re-parse on unmodified `main`, independent of this session — a
pre-existing latent-defect population, not a regression introduced here.
Two — `ccom`, `lac` — hit a third, broader Ley-heading variant this
session did not fix: a bare law title with no "que reforma..." phrasing
at all (`LEY de Aeropuertos.`, `LEY General de Títulos y Operaciones de
Crédito.`), which would need much wider, riskier prefix matching to
recognize safely and was deliberately left alone rather than attempted as
a same-day patch. One (`lcf`) started passing as an unplanned side effect
of the `Decreto de reformas `/Ley fixes. None of the nine were re-derived
or re-committed as part of this pass.

**Verification.** All three admitted TX2 instruments validate clean,
reverse-link with 0 unresolved references (338 lfd + 8 lgdp + 241
reg-lfprh reference edges). `all_committed_batch_manifests_deserialize`'s
frozen counts updated for TX1+TX2 together (33→35 manifests, 172→180
unique instrument slugs — TX1's 5 plus TX2's 3, `lcmopfih` excluded since
it lives in `blocked`, not `instruments`). Full workspace gate: fmt clean,
clippy clean, 152 tests (3 new).

## 2026-08-01 — TX1 admitted, opening Domain TX

`batches/tax_TX1_sat_procedimiento.json` (normalized from
`prompts/cluster-2-batches/lex-mex-cl2-batch-TX1.json`) added `lsat`,
`lfdc`, `lopdc`, `lfpca`, `reg-cff` — all five. `lsat`, `lfdc`, `lfpca`
hit the familiar `1o.`–`9o.` ordinal-numbering review case (matching the
eight prior AD1–AD3 instances); reviewed `allow_article_gaps: true`, no
parser change. `lopdc` and `reg-cff` validated clean at the default
`allow_article_gaps: false`. All five validate, reverse-link with 0
unresolved references (181 edges total). `lfpca` carries one non-blocking
`suffix_order` warning (`58-1` does not sort after `58-S`), the same
already-documented benign class as elsewhere in the corpus.

## 2026-08-01 — AD4 admitted, closing Domain AD; the first AD batch with no review case and no parser fix

Continuing the cluster-2 admission sequence after AD3. AD4
(`batches/administration_ad4_proteccion_civil_misc.json`, normalized from
`prompts/cluster-2-batches/lex-mex-cl2-batch-AD4.json`) added `lgpc`,
`reg-lgpc`, `lhheum`, `ldvuma`, `ldcpdch` — all five, closing AD4 and, with
it, Domain AD (AD1–AD4).

**Nothing to review this time.** All five instruments ran through the
batch loop clean on the first pass at the scaffold default
`allow_article_gaps: false` — none of the five use the `1o.`–`9o.`
ordinal-abbreviation article numbering that forced a reviewed adapter
setting on four of the ten AD2/AD3 instruments (`lfrsp`, `lspm`, `lfaebsp`,
`lspcapf`, `reg-art121-122-lft`), and none hit the reform-decree
publication-date parser gap that AD3's `lspcapf` surfaced. This is worth
recording precisely because it is a negative result: the pattern is
statute-specific (older, hand-drafted numbering conventions), not universal
to the AD domain, so provisional review still has to check each new
instrument rather than assume either outcome.

**Verification.** All five AD4 instruments validate clean, reverse-link
with 0 unresolved references (50 new edges; 233 new articles, 29 new
original transitories). `all_committed_batch_manifests_deserialize`'s
frozen counts updated again for the reviewed addition (32→33 manifests,
167→172 unique instrument slugs). 149 workspace tests pass, fmt clean,
clippy clean. `review-packets generate` picked up the new batch as its own
packet (33rd) without touching the existing 32.

## 2026-08-01 — AD3 admitted; a fourth `1o.` hit plus a genuine parser fix for `1º`-style day-of-month dates

Continuing the same-day "test with a new batch processing" pass, after
pushing AD2 to `origin/main`. AD3
(`batches/administration_ad3_servicio_publico_laboral.json`, normalized from
`prompts/cluster-2-batches/lex-mex-cl2-batch-AD3.json`) added `lspcapf`,
`lfremsp`, `locfcrl`, `reg-art121-122-lft`, `reg-laat` — all five, closing
AD3. `reg-laat` also resolves the entry `batches/labor_L1_labor.json` left
`blocked` ("Not listed in Diputados reglamentos index"): the official regley
URL was since located, and it pairs with the already-committed `laat`, not
the Ley de Aeropuertos.

**Fourth confirmed hit of the `1o.`–`9o.` ordinal case.** `lspcapf` and
`reg-art121-122-lft` both failed the strict article-order check the same way
`lfrsp` (2026-07-18), `lspm`, and `lfaebsp` (both AD2, same day) already had.
Same fix, same reasoning as the AD2 entry above: reviewed
`allow_article_gaps: true`, no parser change, no default change. Four
independent statutes hitting the identical convention now firmly establishes
it as an expected pattern on pre-1990s (and evidently some later) Cámara
statutes, not an edge case.

**A genuine parser gap, this time — not a review-gate near-miss.** `lspcapf`
hit a second, unrelated failure: a hard parse error, not a validation
warning. Its reform-decree appendix (`ARTÍCULOS TRANSITORIOS DE DECRETOS DE
REFORMA`) cites a decree "Publicado en el Diario Oficial de la Federación el
1º de septiembre de 2005" — the day-of-month ordinal mark (`º`), used
specifically for the first of a month, the same grammatical device as the
article-numbering `1o.` case but on a *date*, in a *different* regex.
`extract_reform_evidence`'s local `publication_re`
(`crates/lex-parse/src/diputados.rs`) required a bare `\d{1,2}` immediately
before " de", so the date was never recognized, `publication_date` stayed
`None`, and the decree's first transitory ("Primero") hit a hard
"found reform transitory ... without its Diario Oficial publication date"
error — no adapter setting could have fixed this, since the failure is
inside parsing itself. `extract_dof_publication` (the *other* function that
reads the consolidated document's own top-of-document publication note) had
already solved this exact case with `(\d{1,2})[oº]?` — the fix here is
widening `extract_reform_evidence`'s regex to match, closing a real
inconsistency between two near-duplicate date patterns in the same file.
New fixture `fixtures/diputados/reform-ordinal-first-day-publication-sample.txt`,
built directly from the failing real document's structure (decree heading,
`Artículo Unico`, a nested `TRANSITORIOS` heading, then `Primero`/`Segundo`).

**Verification.** All five AD3 instruments validate clean, reverse-link with
0 unresolved references (49 new edges). `all_committed_batch_manifests_deserialize`'s
frozen counts updated again for the reviewed addition (31→32 manifests,
162→167 unique instrument slugs). 149 workspace tests pass (1 new), fmt
clean, clippy clean.

**`labor_L1_labor.json`'s stale `blocked` entry removed.** The machine-readable
manifest still listed `reg-laat` under `blocked` ("source not located on
official index") after it was admitted via AD3 — the prose above documented
the resolution but the manifest itself still contradicted it. Removed the
entry, added an explanatory note pointing at AD3 and `corpus/mx/reg-laat`.
`all_committed_batch_manifests_deserialize` counts `instruments[]`, not
`blocked[]`, so this doesn't change its frozen totals; reran it to confirm.

**Aside, not fixed here.** Checking `labor_L1_labor.json` against
`schemas/batch-manifest.schema.json` surfaced that its `batch_id`
(`labor_L1_labor`) fails the schema's own `^[a-z0-9_]+$` pattern (uppercase
`L1`) — one of 26 pre-existing manifests with this same mismatch, evidently
never enforced by the Rust deserializer. Not introduced or fixed this pass;
noted so a future schema sweep doesn't mistake it for new drift.

## 2026-08-01 — AD2 admitted; the provisional-review gate caught the same `1o`-style ordinal case it was built for

Testing the merged post-transitory-supplements fix against a genuinely new
batch (per the operator's "let's test with a new batch processing"). AD2
(`batches/administration_ad2_bienes_obras_servicios.json`, normalized from
`prompts/cluster-2-batches/lex-mex-cl2-batch-AD2.json`) added `lspm`,
`lfaebsp`, `reg-lfaebsp`, `reg-lopsrm`, `lfar` — all five, closing AD2.

**What happened, and why it is not a new finding.** Provisional processing of
`lspm` (the plan's designated first instrument) failed validation: 9
`non_numeric_article` warnings for articles 1–9 (`ARTICULO 1o.-` … `9o.-`,
the standard Spanish ordinal-abbreviation convention for single-digit
articles, switching to plain cardinal numbering — `ARTICULO 10.-` — from
article 10 onward), then 63 `article_order` errors for every article from 10
onward, because the strict-order validator's `expected_number` never advances
past a `non_numeric_article` warning. `lfaebsp` hit the identical shape
independently (84 errors), and separately carries genuine
`6-bis`/`6-ter`/`6-quater` suffixed articles. **This is exactly the
`allow_article_gaps` review gate doing its job, not a newly discovered
defect**: `docs/plans/cluster-2-federal-corpus-ingestion.md`'s 2026-07-18
entry already records the scaffold defaulting `allow_article_gaps` to
`false` specifically so every new instrument gets a reviewed opt-in decision
rather than a silent guess, and `lfrsp` hit this identical `1o`-style pattern
that same day. `crates/lex-parse/src/labels.rs` already canonicalizes
`1o`/`2º`/`3°` ordinal marks for sort-key purposes; setting
`allow_article_gaps: true` on `lspm` and `lfaebsp` — reviewed adapter
configuration, the intended fix path — routed both through the already-built
label-aware ordering path, resolving both the ordinal convention and the
`bis`/`ter` suffixes with zero code change. No default should change: the
strict path staying the default is what forces this review to happen for
every new instrument, and it has now worked as designed three times
(`lfrsp` 2026-07-18, `lspm` and `lfaebsp` today) — three independent
statutes hitting the identical `1o.`…`9o.`-prefix convention is a pattern
worth expecting on any pre-1990s Cámara statute during provisional review,
not a coincidence to rediscover as a 74-error failure each time.

**Verification.** All five AD2 instruments validate clean, reverse-link with
0 unresolved references (184 new edges), and `all_committed_batch_manifests_deserialize`'s
frozen counts were updated for the reviewed addition (30→31 manifests,
157→162 unique instrument slugs) — that test's own guard is what caught the
new manifest needing acknowledgment, not silent drift. 148 workspace tests
pass, fmt clean, clippy clean; no Rust code changed, only the two adapter
configs and the frozen-count test.

**Checked, not assumed: `reg-lfaebsp` and `reg-lopsrm` kept `allow_article_gaps:
false` and still validate clean despite 6 and 3 `non_numeric_article`
warnings each.** `valid: true` alone doesn't prove the order check kept
covering the rest of the document — it proves no error fired, which the same
counter-freeze bug could also produce by accident. Checked directly: both
documents' non-numeric articles are `bis`/`ter`/`quater` suffixes
(`14 bis`, `32 bis`, `32 ter`, `15 Bis`, `15 Ter`, `15 Quater`, …), each sitting
between two consecutive plain numbers (`14`, `14 bis`, `15`). The strict
path's counter is already primed to the next plain number by the article
immediately before the suffix and the suffix never touches it, so the
numeric run stays intact end to end — confirmed by printing both documents'
full article-number order, not inferred from the absence of an error. This
is the opposite shape from `lspm`/`lfaebsp`, where the non-numeric run came
*first*, before anything had primed the counter at all.

## 2026-07-31 — Sequential-fold planning pass landed; the engine itself stays unbuilt

Operator sign-off on the fold doctrine (below, same date) arrived via the
directive "go for 1." M4's own rule requires a fresh planning pass and
sign-off on the trusted-boundary shape before implementation
(`docs/plans/maximasa-legal-integration.md`) — that planning pass is
`docs/plans/sequential-canonical-fold.md`, written now, mirroring how
`standards-amendment-marks.md` preceded Stage A.

**Deliberately partial, and the boundary is the point.** This lands the plan
doc plus the pieces of the data model that do not depend on the
annex-modeling question the operator is taking into a separate worktree:
`StandardTextBasis::DerivedConsolidation` (schema value Stage C's output
will eventually assert; no committed standard uses it), `CanonicalFoldOperation`
(`Replace`/`Keep`/`Shift`, the doctrine's three per-unit operations and no
fourth "delete", matching the doctrine's own no-deletion rule), and
`fold_unit`, a pure function folding one unit's chronological operation list
into `(text, position)` — the isolated bookkeeping core of `apply()`, proven
against a synthetic decree history. **It does not parse a real decree's
ellipsis-diff prose, does not add a `standards consolidate` command, and does
not touch NOM-247 or any other committed record.** That is the actual hard
part of Stage C and stays unbuilt, gated on the annex decision as before.

Verified the same way Stage A and Stage B were: `StandardTextBasis` is
matched exhaustively at its one call site, so the compiler itself refused to
build until the new variant was handled — not asserted, forced. All 29
committed standards re-run through `standards refresh` at zero-byte diff, and
both schemas re-validate at 0 violations, including a synthetic document
that sets `text_basis: "derived_consolidation"` and validates successfully.
144 workspace tests pass (4 new), fmt clean, clippy clean.

## 2026-07-31 — Packet-based review assignment landed: `batch_id` is the grouping key

Operator go-ahead. Staged 2026-07-28 (`docs/plans/maximasa-legal-integration.md`
M4 future scope, item 4) with two open questions: a grouping key, and a
reviewer-assignment record kept distinct from `legal_review_status`/
`technical_review_status`. Both decided here, as my own call, stated
explicitly rather than left implicit in the code:

**Grouping key: `batch_id`.** `batches/*.json` already groups the federal
corpus into legally coherent topical clusters for ingestion (`labor_L1_labor`,
`financial_F2_banking`, ...) — reusing it means a reviewer sees the same
grouping the ingestion plan already reasoned about, and no new categorization
had to be invented. A packet's `instruments` list is restricted to what
`corpus/mx` actually holds at generation time, since a batch manifest can
name instruments not yet ingested. Running `review-packets generate` against
the live corpus today produced 30 packets covering 147 of 181 committed
instruments (0 schema violations). **The gap is real, not a bug**: standards
(29) and the CNBV DCG family (`socap-sofipo-dcg-2006` and its siblings) have
no `batches/*.json` manifest at all — a separate ingestion track each — so
this landing does not cover them. Extending the grouping key to a second
source is future work, not silently done here.

**Record: `ReviewPacket` (`lex-core`), one file per packet under
`review-packets/`, new `schemas/review-packet.schema.json`.** `status`
(`unassigned`/`assigned`/`in_review`/`reviewed`), `reviewer`, `assigned_at`,
`notes` — workflow only. It carries no verdict and does not write to any
instrument's own review-status field; assigning a packet does not review
anything, it only names who is responsible. `lex-mex review-packets
generate` never overwrites an existing packet file (an already-assigned
packet's state survives a re-run); `assign <packet-id> --reviewer <name>`
refuses on any packet not currently `unassigned`, so reassignment stays a
deliberate, separate act rather than a silent overwrite.

**A known staleness gap, stated plainly rather than discovered later as a
bug**: `generate`'s refuse-to-overwrite guarantee protects an assignment, but
it means a packet's `instruments` list is frozen at generation time. If a
batch later gains a newly-committed instrument, its existing packet does not
grow to include it, and nothing detects the drift — there is no
re-derive-and-compare check here the way `standards refresh` gives the
standards module. An `unassigned` packet is harmless to regenerate (delete
and re-run `generate`); an `assigned` or further-along packet is not, since
that would discard real reviewer state. Reconciling a packet's membership
after the fact — adding newly-committed instruments to an already-assigned
packet without disturbing its status — is not built and is the natural
follow-up once this gap is hit for real.

**Deferred, per the plan's own text**: a way for a reviewer to flag a missing
backlink on the fly, and any link from a completed packet review back into an
instrument's `legal_review_status`. Neither is scoped here.

140 workspace tests pass (4 new), fmt clean, clippy clean.

## 2026-07-31 — Scope 2 Stage B landed: per-decree source hash on the existing `modifications[]` list

Operator sign-off. The 2026-07-29 decomposition note described Stage B as
"`source_url`/`source_sha256` become a list covering the base and every
decree" — read narrowly, that would add a second top-level array duplicating
`modifications[].official_url`. **The per-decree list already exists**: the
`modification` schema def has required `official_url` and `modifications` has
been an array since before Stage A. What was missing was a hash, not a
container. Stage B adds one field, `source_sha256: Option<String>`, to each
`modifications[]` entry (`StandardModificationSource`, `lex-core`), pinning
that decree's own source bytes with the same hash semantics as the base
publication's top-level `source_sha256` — not its extracted text, which has
no per-decree equivalent yet. Left unset, not `false`/`null`-equivalent,
whenever `included_in_source: true`: a decree already folded into the
retained base text (`nom-051-scfi-ssa1-2010`, `nom-187-ssa1-scfi-2002` today)
has no separate document to pin independently of the base hash.

**Additive by construction, verified, not asserted.** `#[serde(default,
skip_serializing_if = "Option::is_none")]` means omitting the field
serializes with no key. Confirmed two ways: a new lex-cli test round-trips a
modification with the field set and asserts an unset one has no
`source_sha256` key at all in the raw JSON; and all 29 committed standards
were re-run through `standards refresh` — `git status` on `corpus/` came back
empty, i.e. zero bytes changed anywhere, which is what an additive schema
change to a `skip_serializing_if` field is supposed to produce. Both schemas
re-validated with the Python `jsonschema` venv pass at 0 violations. No Rust
validator rule was added beyond the schema's existing `$defs/sha256` pattern
match — a hand-written check would only restate what the schema already
enforces, which M4's own "layers consistent, not redundant" expectation rules
out.

**Deliberately not built in this pass**: nothing fetches or hashes a decree's
source PDF. `standards compile` is untouched. The same acquisition-drift risk
argued against re-deriving the four CNBV instruments below applies verbatim
here — no queued compile needs a real decree hash yet, so none was computed.
Backfilling `source_sha256` for the five Maximasa NOMs' existing modifications
is future work, not required by this schema addition.

## 2026-07-31 — Marker-cap regex widened to three digits; the four affected CNBV instruments are NOT re-derived this pass

Operator go-ahead on the marker-cap defect found 2026-07-30 (`amendment_marker_regex`
`dcg.rs:176`, `legend_entry_re` `itf.rs:179`, both `\d{1,2}`, silently dropping
or misparsing any three-digit-or-higher margin marker). Both regexes, plus the
inline annex-heading marker sub-capture (`dcg.rs:294`, the landscape `ANEXO 14
(2)` form), are now `\d{1,3}`. Two new `dcg.rs` unit tests exercise a
standalone three-digit marker and a three-digit inline heading marker
directly against `parse_annex_document`; one new `itf.rs` test extends the
shared `parser-sample.txt` fixture with a genuine `150)` legend entry and
asserts it becomes its own `AmendmentReference` rather than being appended as
trailing text onto entry `9` — the exact swallow bug confirmed in
`cub-dcg-2005`'s committed legend. fmt clean, clippy clean, 135 workspace
tests pass.

**The four committed instruments the 2026-07-30 entry found affected
(`cucb-dcg-2004`, `cub-dcg-2005`, `oaac-dcg-2009`, `cue-dcg-2003`) are
deliberately NOT re-derived in this pass — their `provisions.json` still
carries the pre-fix data.** Re-deriving them the way `standards refresh`
re-derives a NOM requires the original retained extracted text; the DCG/ITF
instrument family has no such committed artifact (unlike a standard's
`standard.json` + retained text), and none survived locally from the original
ingestion (`.work/<slug>` is cleaned up after a successful `pipeline` run
unless `--keep-work` was passed, and it wasn't). The only way to re-derive
today is a live re-fetch through `pipeline`, and a direct check
(`curl -I` against `cub-dcg-2005`'s `official_url`) found CNBV has already
republished a newer compiled PDF since the 2026-07-13 ingestion:
content-length 4,866,388 → 6,144,432 bytes, ETag revision `,153` → `,156`,
`last-modified` 2026-07-03 → 2026-07-15. Re-fetching now would silently fold
a genuine legal update into a diff meant to be a scoped parser bug fix — the
exact conflation the still-unbuilt "Corpus currency" mechanism (`docs/decisions.md`
2026-07-12, "Amendment markers on CNBV reform transitorios") exists to
prevent by surfacing source drift as its own reviewed report rather than an
implicit side effect. Re-deriving these four stays pending that mechanism (or
an explicit, separately-authorized re-ingestion of each) — not done here.

**New known gap, worth stating plainly: the DCG/ITF instrument family has no
retention parity with standards.** A NOM's `standards refresh` can prove a
parser fix is safe against the exact bytes that produced the committed data,
hash-checked, with zero re-acquisition risk. No equivalent exists for CNBV
DCG/ITF instruments — a parser-only fix to this family can currently only be
verified by unit fixture, never mechanically replayed against its own
committed corpus entry. Not scoped to build here; flagged for whoever picks
up the corpus-currency mechanism, since the same retained-text pattern likely
solves both.
## 2026-07-31 — Post-transitory standard supplements are exact-span opaque records

Material following a genuine standards TRANSITORIOS section is now a required
canonical boundary rather than either discarded trailing text or part of the
last transitory. `StandardMetadata.supplement_starts` supplies reviewed,
source-ordered exact anchors and one of five kinds: appendix, annex, reference
guide, standalone table, or list. The derived `supplements.json` gives each
top-level item a one-based identity, collapsed heading, exact text/span, and
explicit-source legal character. Internal rows, forms, figures and subsections
remain opaque; section kind never implies normativity.

One shared tail-layout calculation locates the real TRANSITORIOS section,
resolves every anchor exactly once and in order, and supplies both transitory
and supplement boundaries. Closing forms now cover `Dado en…`, `Se expide
en…`, `Provéase la publicación…`, and historical/current Mexico City
datelines. No unconfigured inline Tabla/Guía mention is a boundary. A legal
character is `explicitly_normative` or `explicitly_non_normative` only where
the source says so; conflict is an error and silence is a warning.

The all-29 inventory corrected an assumption in the implementation plan. The
13 existing trailing-material warnings plus NOM-020, NOM-027 and NOM-085 were
not the complete affected set: seven more STPS texts had an unrecognized
`Dado en…` signature followed by guides, so the old parser swallowed both
closing furniture and the guides without emitting its trailing warning.
Twenty-three standards therefore have configured supplements; six correctly
carry an empty required file. Eleven final transitories changed. All earlier
transitories compare deeply equal and every changed final is a strict prefix
truncation; all 29 `clauses.json` files are byte-identical to baseline.

`standards refresh` now compares transitories deeply and refuses any tail
change by default. `--allow-tail-repartition` permits only a reviewed final-
transitory truncation paired with supplements, with every guard before writes.
Compile/validate/path/bundle surfaces all include the required canonical file.
The out-of-repository Maximasa five-NOM bundle is consequently stale and is a
mechanical follow-up, not part of this repository change.

Fresh reacquisition then closed the three held-out records. The Platiica PDFs
and new `pdftotext -layout` outputs matched all six recorded SHA-256 values
before compilation. NOM-019 compiled at 94 clauses / 3 transitories / 1
explicitly non-binding guide; NOM-024 at 87 / 2 / 2 explicitly non-binding
guides; NOM-052 at 76 / 3 / 8 (Tables 1-2, Listados 1-5 and Anexo 1), each
legal character `unspecified` because its retained source states none. The
earlier reviewer characterization is not substituted for an explicit source
declaration.
## 2026-07-31 — Review fixes on Stage A, same day: ten findings, all closed

A ten-finding code review ran against the Stage A landing commit
(`4ab432e89`); every finding was fixed the same day. The corpus effect is
message-precision only: warning counts are byte-identical corpus-wide (checked
old-vs-new across all 29 standards), no clause or transitory span moved, and
the only committed diffs are the 13 `validation.json` files whose
trailing-material warnings now name the actual heading instead of a byte
count. The durable decisions:

**One `LINE_LEAD` fragment feeds every line-anchored regex in `standard.rs`.**
The page-break (`\x0c`) admission from 2026-07-29 had reached only two of the
five line-anchored patterns. The three it missed made the fix
self-undermining: a form-fed ordinal line made the heading finder reject the
genuine TRANSITORIOS section (re-admitting phantom clauses *and* silencing
every downstream warning at once), and a form-fed TRANSITORIOS/APÉNDICE was
visible to the heading finder but invisible to span bounding, so the last
clause absorbed the whole transitorios section — or the last transitory
swallowed the annex and harvested its dates as `asserted_dates`. **The rule:
a character-class fix to line anchoring is applied through one shared
fragment or not at all** — partial application converts a visible defect into
a quieter one. Three regression fixtures.

**Run selection sees the full match list; the TRANSITORIOS boundary applies
to the winner afterwards.** Truncating candidates first shortens only
body-side runs (the índice sits before the boundary and never loses a row),
so a body with partially regex-invisible headings could lose the length
comparison to its own índice — the NOM-052 failure mode returning through
selection. A run must still *start* before the boundary, so post-transitorios
numbering that restarts at 1 cannot compete. Fixture:
`index-outnumbers-body-sample.txt`.

**`refresh` now has four guards, all before any write**: clause count,
transitory count (new — transitories move neither the clause count nor the
marks, so a transitory-parser regression previously rewrote committed data
with exit code 0 and left the corpus self-consistently wrong), amendment
marks (`--allow-mark-change`), and report validity (new — the old order wrote
the three files and bailed after, leaving invalid canonical data behind a
non-zero exit that a batch loop can miss). A refused refresh is now
byte-for-byte a no-op, and both new guards have tests asserting exactly that.

**Trailing-material detection is structural first, byte-count last.** The
flat 2000-byte threshold under-reported compact normative annexes (the exact
silent omission the warning exists for) and over-reported long signature
blocks. The transitorios section ending at an APÉNDICE/ANEXO heading is
itself the evidence — warned unconditionally, naming the heading; a
signature-closed tail is searched for a later annex-like heading (GUÍA,
LISTADO, TABLA included — in the *warning search only*; extending
`SECTION_END` itself would move committed transitory spans, which is the
held-out `transitory-absorbs-annex` defect, still gated); only a headingless
remainder falls back to the byte heuristic, measured after the last
"Rúbrica". All 13 committed warnings survived and now name their heading.

**Title parsing: date phrases are excluded before numeral matching, and annex
identifiers must start with a genuine capital or digit.** "del diverso
publicado el 30 de junio de 2011" could mint a false `amended_by` mark on an
unrelated clause numbered 30 or 2011 — a false *presence* mark, strictly
worse than any missed mark in this design — and a global `(?i)` case-folded
`[A-Z0-9]` so "Anexo de la ..." prose produced the bogus target "Anexo de".
Neither ever fired on the three recorded titles. Also: the verb-segmenting
regex and the verb→action classifier are now generated from one
`VERB_FAMILIES` table, making the "regex sees a family the classifier
doesn't" drift structurally impossible rather than merely tested against.

**Consumer surface**: `lex-mex instruments --json` now carries
`published_designation` (a reader is never shown a designation appearing
nowhere in the record's own retained text without the discrepancy marked) and
`amendment_marked_clauses`. Per-clause rendering is recorded in the Stage A
plan as *deferred* on the nonexistent standards Markdown profile, closing the
review's "neither delivered nor deferred" gap.

Also: constant regexes hoisted to `LazyLock` statics, the modification-target
derivation computed once per validation pass, the third `collapse_whitespace`
copy replaced with the crate-level helper, and `as_str()` added to
`StandardModificationAction` so prose messages reuse the serde token.

## 2026-07-31 — Landed Scope 2 Stage A: clause-level amendment marks derived from decree titles

Operator sign-off arrived as the directive "go for M4"; Stage A was the only
M4 item blocked on sign-off rather than on unscoped design, so it is what
shipped. Plan and full acceptance table: `docs/plans/standards-amendment-marks.md`.
**Stage A applies no text.** The sequential-fold doctrine recorded below is
Stage C and did not leak into this.

**What the change is.** A modifying decree names its targets in its own DOF
title. `StandardModificationSource` now carries that title verbatim as pure
input (`title: Option<String>`), and the parser derives from it: `amended_by`
on each matching `StandardClause` (the modification's index plus the decree's
own verb — `modified` / `added` / `eliminated`), and validation warnings for
every named unit matching no committed clause. A 252-clause instrument-level
"this standard has an unincorporated modification" warning becomes 17 marked
clauses and three named unresolved targets.

**Where derived data was allowed to live, and why it matters.** The plan put
`affects` on the modification, inside `standard.json`. That file is pure
passthrough — `validate` re-derives clauses and transitories and bails on
drift, but nothing re-derives metadata. Derived data there would go stale on a
parser change with no check firing: the same failure shape as the `\d{1,2}`
marker cap found on 2026-07-30. So the derived half was placed where
determinism checks already run — `amended_by` in `clauses.json` (reparse-and-
compare), unresolved targets in `validation.json` (report-compare) — and no new
canonical file was introduced. **The general rule this instance establishes:
derived output goes only where a re-derive-and-compare check already exists, or
that check is built in the same change.**

**Two things deliberately not done.**

- *No nearest-ancestor resolution.* A target matches a committed clause number
  exactly or it does not resolve. Attaching an unmatched target's mark to a
  parent clause would claim a decree addressed text it never named, and the
  unmatched case carries real information.
- *No action-erasing index.* `amended_by` is `Vec<StandardClauseAmendment>`,
  not the planned `Vec<usize>`, because a bare index cannot distinguish an
  eliminated clause from a modified one and the plan's own rendering example
  would print "Numeral modificado" for a repealed numeral. **A clause marked
  `eliminated` still carries its full live text** — the mark records the
  repeal, nothing applies it. That is the sharpest legal-meaning trap in this
  change.

**A finding that narrows the premise: the cheap lever is SSA1-shaped, not
universal.** SSA1 publishes "Modificación de los numerales 3.2, 3.10, ... de la
Norma Oficial Mexicana NOM-247-SSA1-2008"; STPS publishes "ACUERDO de
Modificación a la Norma Oficial Mexicana NOM-020-STPS-2011, ..." with no
numeral in the title at all. Title parsing therefore reports three distinct
states, never conflating them: targets found; a title recorded that names
nothing (`standard_modification_scope_unknown`, NOM-020 — scope *unknown*, not
*empty*); and no title recorded at all (`standard_modification_title_absent`).
Guessing in the second case would have been worse than the instrument-level
warning it replaced.

**A finding that strengthens an existing open question.** NOM-247's 2012 decree
modifies `5.2.7.ii.1)`, and the committed base text has no `5.2.5`, `5.2.6`, or
`5.2.7` at all — the `5.2` family stops at `5.2.4`. A *modificación* of a
numeral absent from the base is not the benign unresolved case (that is
`5.1.5`, an *adición* of new material). It is independent evidence for the
plan's standing open question that NOM-247 has at least four modifications, not
the two recorded — its 2011 CONSIDERANDO cites decrees of 2010-01-22 and
2010-07-19 that `standard.json` does not carry. Not resolved here.

**A defect found in review, before it reached the corpus.** The action-verb
regex was asymmetric: `reforma[sn]?` matched the conjugated `reforman`, but
`deroga(?:ci[oó]n(?:es)?|se)` did not match `derogan`. Because title segments
are cut *at* verb matches, an unseen family is not skipped — its targets are
absorbed by the preceding family. "Se reforman los numerales 3.2 y 3.4 y se
derogan los numerales 5.1 y 5.2" therefore parsed as a single *modified*
segment and recorded two repeals as modifications: mislabelling, which is
strictly worse than the "report scope unknown rather than guess" behaviour this
change is built around, and precisely the failure `action` exists to prevent.
Fixed by covering nominal and conjugated forms symmetrically for every family,
with the mixed-verb title as a regression test. None of the three recorded
titles used a conjugated form, so **no committed corpus data was ever wrong** —
all 29 standards revalidate byte-identically after the fix. **The general
lesson: in a parser that segments at keyword matches, uneven keyword coverage
does not lose data, it misattributes it.**

**New tooling, and a guard worth stating.** `lex-mex standards refresh <slug>`
re-derives a committed standard's parsed files from its retained text — the
counterpart to `validate`, and the mechanism that made this backfillable across
29 records without re-acquiring original PDFs. It aborts on a clause-count
change. It also aborts on *any* amendment-mark change unless
`--allow-mark-change` is passed: marks never move the clause count, so a
title-parser regression that drops or misattributes them would otherwise pass
every mechanical guard and be written straight into committed canonical data.
A printed count is not a check.

**Blast radius.** 26 of 29 committed standards refresh to a zero-byte diff.
Only `nom-247-ssa1-2008` (`standard.json`, `clauses.json`, `validation.json`)
and `nom-020-stps-2011` (`standard.json`, `validation.json`) changed. NOM-247's
`clauses.json` diff is `amended_by` insertions only — no clause text, id,
number, or span moved. **NOM-247 is one of the five NOMs in the Maximasa
bundle, so that out-of-repo bundle lock is now stale** and needs a mechanical
regeneration pass; not done here.

## 2026-07-31 — Operator doctrine: sequential canonical-state fold, generalizing the decree-diff engine for both NOM and SHCP/CNBV cases

Operator framing, proposed after seeing the ellipsis-completeness correction
above and the art. 115 LIC pilot (`docs/plans/cnbv-art115-lic-
consolidation.md`) as a whole. **Design proposal, awaiting sign-off — not
implemented, no schema/parser change made.** Recorded now so it isn't lost
before the formal M4 pass; folded into `docs/plans/maximasa-legal-
integration.md`'s M4 scope text alongside this entry.

**The algorithm.** For any instrument with no publisher-compiled text
(NOMs, and now confirmed also SHCP/CNBV resoluciones like art. 115 LIC),
build the canonical text as a **strict left fold over the decree history in
chronological order**, not by taking each unit's single "latest full
restatement":

```
canonical := base publication text
for decree in decrees sorted by DOF date:
    canonical := apply(canonical, decree)
```

1. Start from the original (base) publication text.
2. Apply each modifying decree **against the current canonical state**, not
   against the original base each time — verified today's compiled draft
   got this wrong (it took each key's last full restatement in isolation,
   which is only correct if that restatement's own ellipsis gaps happen to
   already be filled, which they are not).
3. When a decree's own `ARTÍCULO ÚNICO`/resolving clause carries a prose
   description (not a text restatement) — particularly derogations or
   reordering — apply that description as an operation in its own right.
   Confirmed necessary: `64ª`'s 2014-12-31 edit exists *only* as a preamble
   instruction, never restated.
4. Verify the único's REFORMAN/DEROGAN/ADICIONAN lists against what the
   decree's replacement body actually contains before applying anything —
   this cross-check is what surfaced both the 5 known gaps and the 46
   ellipsis-affected keys in the pilot; it is not optional bookkeeping.
5. **Ellipsis inside a decree's replacement text means the canonical text
   for that sub-part is unchanged — splice in only the explicitly given
   new text, leave everything under an ellipsis exactly as canonical
   already had it.** Confirmed directly against DOF's own source markup
   (`<span>...</span>`), not inferred.
6. **No deletions, ever. A repealed unit becomes `derogado` with the
   repealing decree's date/codigo recorded in place — it is never removed
   from the structure and nothing is renumbered around it.** This matches
   how DOF's own compiled texts actually render repeals ("Artículo N.-
   (Derogado, D.O.F. [fecha])") and directly answers the open question
   already sitting in M4's scope text ("a retained-text strategy for
   derogation-caused span shifts") — the answer is: there is no span
   shift, because nothing is ever removed or renumbered.

**The wrinkle found running this against the pilot, not yet in the five
points above when first proposed:** step 5's "ellipsis = unchanged" is
necessary but not sufficient. When a decree inserts a new paragraph and its
own prose states the later ones are "*recorriéndose los demás en su
orden*" (shifted down), an ellipsis-covered later paragraph's **content**
is unchanged but its **ordinal position moves**. A fold that treats
ellipsis as pure copy-forward-by-position would silently mislabel it —
e.g. what was "tercer párrafo" silently staying labeled third when the
único's own prose says it is now fourth. The único's prose (step 3/4) is
what carries this instruction; it cannot be inferred from the ellipsis
alone. **`apply()` therefore needs three per-unit operations, not two:
`replace` (explicit new text given), `keep` (ellipsis, content and
position both unchanged), and `shift` (position changes per the único's
own renumbering instruction, content unchanged) — collapsing `shift` into
`keep` is the specific way a naive implementation of this model breaks.**

**Generalizes, does not replace, the existing NOM-side scoping** (M4 /
`docs/plans/maximasa-legal-integration.md`, `docs/ingestion-difficulty-log.md`
`decree-diff`): a NOM MODIFICACIÓN's whole-clause ellipsis span
("unchanged span ... replacement text", `docs/decisions.md` 2026-07-26) and
this SHCP/CNBV sub-part ellipsis (found today) are **the same mechanism at
different nesting depth** — clause-level for NOMs, sub-part-of-a-named-unit
level for SHCP/CNBV resoluciones. One fold model, one `apply()` with the
three operations above, covers both; the earlier framing of them as
distinct engines ("simpler than M4" vs. "needs the ellipsis engine") is
superseded by this doctrine.

**Loop, not graph, for the per-instrument fold.** The fold above is a plain
sequential reduction over one instrument's own decree history — no graph
structure is needed for it. Cross-instrument reference resolution
(`docs/cnbv-consolidated-disposiciones.md` §3) is a separate, already-
deferred concern and should stay separate rather than being merged into
this fold.

## 2026-07-31 (correction, same day) — "para quedar como sigue" is a nested ellipsis-diff, not a full-text replacement; the entry below is wrong on its central claim

Operator caught this directly in the compiled output, not in the source
reading: disposición 4ª's "current text" (from RM 2024-08-28, codigo
5737473) reads `... ... I. ... a) ... i. Primer apellido... ii. ... iii.
...` — riddled with literal `...`. Checked against the source RM's own
REFORMAN clause for 4ª: it names only specific numerales/incisos/fracciones
("la 4ª, segundo párrafo, fracciones I, incisos a), numeral i. y b),
numerales i. primer y segundo párrafos y iii. segundo párrafo, II, incisos
a), numeral x., IV,..." — dozens of pinpointed sub-parts, e.g. fracción III
of 4ª is conspicuously absent). The `...` in the replacement body **is DOF's
own convention for "this sub-part is untouched, carries over unchanged from
the immediately prior version"** — confirmed literally present in the raw
DOF HTML (`<span>...</span>`), not an extraction artifact.

**This retracts the entry below's central claim.** Quoted from it, struck,
not silently edited:

> ~~This SHCP/CNBV format instead names the unit by number and gives its
> entire new text, so applying it is a keyed full-text replacement
> (`disposición_id → new_text`), not a splice.~~
>
> ~~named-unit full-text replacement needs no ellipsis-splicing logic at
> all, just chronological last-writer-wins per disposición number~~

**What's actually true:** "para quedar como sigue" restates the numbered
unit, but only fills in the parts the RM actually touches — everything else
inside that same unit is elided with `...` and must be spliced in from the
nearest prior version that gives real text for that specific sub-part
(walking back through the key's own history, recursively, since an
intermediate version may *also* have elided the same sub-part if it wasn't
touched there either). This is the NOM ellipsis-diff mechanism after all —
just operating one level of nesting deeper (inside a named unit) instead of
across the whole document. The "simpler than M4, no splicing needed" framing
is wrong; **this instrument needs real ellipsis-splicing, scoped to
sub-parts of a named unit rather than the whole document.**

**Scope, measured against the pilot compiled draft (2026-07-31):** grepped
`compiled_draft.json`'s 96 `current_text` values for literal `...` — **46 of
96 keys are affected**, not a handful. Worst cases: `4a` (135 ellipses,
source 5737473), `64a` (35), `14a` (28), `74a` (16). None of these 46 are
corrected in the compiled draft or its Obsidian copy as of this entry — they
carry a blanket warning, not a per-provision fix, pending the real
splice-reconstruction work below.

**What survives from the entry below, unretracted:** the extraction
mechanics (locate `ARTÍCULO ÚNICO`, parse REFORMAN/DEROGAN/ADICIONAN lists,
split "para quedar como sigue" by disposición-number headers) are still
correct and reusable — they just produce an *elided* text per touched unit,
not a complete one. The wholly-derogated-numeral finding (`7ª-1`) and the
prose-only-structural-edit finding (`64a`'s 2014-12-31 edit) both still
hold. The DEROGAN/REFORMAN-list-vs-body cross-check discipline from that
entry's caveat is exactly the right instinct, generalized further by this
correction: **never trust a replacement body's completeness without
checking it against what the RM's own preamble says it touched.**

**Next, not done this session:** build the recursive splice — for each `...`
span in a key's current text, identify which specific sub-part (fracción/
inciso/numeral, by the ordinal immediately preceding the `...`) it stands
for, and walk that key's history backward for the nearest version that gives
real text for that same sub-part. This is materially more work than the
keyed-replacement model assumed and should be scoped as its own pass, not
rushed.

## 2026-07-31 — A second, simpler decree-diff subclass: SHCP/CNBV "ARTÍCULO ÚNICO" named-unit replacement

Found while inventorying DOF resoluciones modificatorias for *Disposiciones
de carácter general a que se refiere el artículo 115 de la Ley de
Instituciones de Crédito* (no consolidated text exists — see
`docs/plans/cnbv-art115-lic-consolidation.md` for the full inventory and
research trail). This instrument has no publisher-compiled text (like a
NOM) but its modifying decrees are **not** shaped like NOM decrees.

**The format.** Every RM read so far (2 of 16, both confirmed) has a single
resolving clause of this exact shape:

> ARTÍCULO ÚNICO.- Se REFORMAN las disposiciones `<N, N, N...>`; se
> DEROGA(N) `<N, N...>` (sometimes only a named párrafo/fracción inside a
> disposición that is *also* being reformed); y se ADICIONAN `<N, N...>`;
> todas de las Disposiciones..., **para quedar como sigue:** `<the complete
> new text of every disposición named in the REFORMAN/ADICIONAN lists, in
> full, numbered>`.

**Why this differs from the NOM case** (`docs/decisions.md` 2026-07-26,
`docs/ingestion-difficulty-log.md` `decree-diff`): a NOM
MODIFICACIÓN/ACUERDO states its diff as an **ellipsis span** — "unchanged
text ... replacement text" — which requires splicing the replacement into
the retained base at the right offset. This SHCP/CNBV format instead names
the unit by number and gives its **entire new text**, so applying it is a
keyed full-text replacement (`disposición_id → new_text`), not a splice. A
disposición touched by both a reform and a partial derogation (e.g. one
párrafo removed) still appears exactly once, in full, under "para quedar
como sigue" — the derogated párrafo is simply absent, so partial-
derogation-within-a-reformed-unit needs no special handling. A **wholly**
derogated disposición (repealed outright, not reformed) would be named only
in a "se DEROGA" clause with no replacement text and must be dropped from
the compiled output — not yet observed directly, needs confirming against a
real instance.

**Consequence for Scope 2 / M4.** `docs/plans/maximasa-legal-integration.md`
scoped the `decree-diff` engine around the NOM ellipsis case. This SHCP/CNBV
format is a **distinct, mechanically simpler** subclass — named-unit
full-text replacement needs no ellipsis-splicing logic at all, just
chronological last-writer-wins per disposición number with full history
retained as provenance. Do not assume every `decree-diff` instrument needs
the heavier ellipsis engine; check which shape a given publisher uses
first. Worth generalizing once more instruments are seen: this is plausibly
how *most* SHCP/CNBV resoluciones modificatorias are written (it matches
Mexican regulatory drafting convention — "para quedar como sigue" is the
standard formula), so it may cover more of the CNBV-adjacent, no-
consolidated-text corpus than the ellipsis case does. This methodology
should be reused directly the next time a corpus instrument is updated
against a new DOF reforma once the current corpus is digested — the
extraction pattern (locate `ARTÍCULO ÚNICO`/multi-artículo resolving
clause, parse the REFORMAN/DEROGAN/ADICIONAN lists, split "para quedar como
sigue" by disposición-number headers) is the reusable part, independent of
which instrument it's applied to.

**Caveat found running this against all 16 RMs of the pilot instrument
(2026-07-31, `docs/plans/cnbv-art115-lic-consolidation.md`): "always full
text" is not universal.** Two real counterexamples:

- A **purely structural edit** (paragraph renumbering, a single-paragraph
  derogation with no other substantive change) can be stated as an
  instruction entirely inside the preamble, with **no** "para quedar como
  sigue" restatement anywhere — the numeral is named in the DEROGAN/REFORMAN
  lists but never appears in the replacement body. Mechanical extraction
  keyed only on replacement-body headers silently misses this; it must be
  caught by cross-checking every DEROGAN/REFORMAN-list numeral against the
  set that actually appears in the replacement body, and flagging the
  difference rather than assuming absence means "not actually changed."
- A **wholly derogated numeral** (repealed outright, not reformed) is named
  only in a DEROGA clause with no replacement text, confirmed for real:
  disposición "7ª-1" — note the hyphenated sub-numeral format, which a
  numeral-key normalizer built only for "Nª [Bis/Ter/...]" will silently
  fail to recognize at all, losing the provision's entire lifecycle rather
  than just its latest state.

Net: the named-unit full-text-replacement shape is still the dominant case
and still simpler than the NOM ellipsis-diff, but a correct extraction must
diff the DEROGAN/REFORMAN/ADICIONAN *lists* against the replacement *body*
per RM, not just parse the body — the gap between those two sets is where
both counterexamples were found.

## 2026-07-29 — Lex-Mex as compiler for NOMs; Scope 2 decomposed; a dangling amendment legend

Operator framing, and it identifies a real asymmetry: for Cámara de Diputados
statutes and CNBV disposiciones the publisher issues a consolidated text, so
provenance is "hash the one file they publish." NOMs have no such file. To
hold a current version at all, **Lex-Mex would have to do the compilation
itself** — hashing each input document (base publication plus each
modificatorio) rather than one source, keeping only current text, and
annotating each modified clause with the decree that changed it, exactly the
way Diputados and CNBV print marginal reform notes. Superseded versions would
not be retained, only pointed at.

Scope 2 is therefore decomposed into three stages with different dependencies:

- **Stage A — mark, don't modify.** Parse the numerals a decree names in its
  own title, attach them to the clauses they target with the decree's date and
  URL. Turns `standard_unconsolidated_modification` from an instrument-level
  warning into a clause-level one. Requires no text transformation, no new
  text basis, and nothing from the unresolved annex defect. Planned in
  `docs/plans/standards-amendment-marks.md`; **not started, awaiting sign-off**
  per the M4 rule.
- **Stage B — multi-source provenance.** `source_url`/`source_sha256` become a
  list covering the base and every decree. A schema question, separable from
  any text change.
- **Stage C — actual consolidation**, asserting a `derived_consolidation` text
  basis.

**Why Stage C is not "an easy fix", stated with the counterexample.**
NOM-247's second decree performs four distinct operations: modifies six
numerals, *adds* 5.1.5, *eliminates* three numerals, and *eliminates Apéndice
normativo A*. That last deletes post-transitorios material the corpus does not
model at all and that `transitory-absorbs-annex` currently swallows into a
transitory. A consolidation engine cannot merely substitute clause text; it
must delete an annex it has no representation of. Stage C therefore sits on
top of the annex decision, while Stage A does not.

**The verification story changes, and must be asserted deliberately.** For a
statute, correct means "matches the file Diputados published." For a
compilation Lex-Mex produces there is no such file, so the guarantee becomes
hashed inputs plus a deterministic transform yielding byte-reproducible
output, gated on a named human review. That is defensible but is a different
claim than the corpus makes anywhere today, and `text_basis` is where it would
be made. It must not arrive implicitly as a side effect of Stage C.

**Correction, 2026-07-30: the paragraph below is wrong and superseded.** The
claim that the amendment legend is empty everywhere was measured against the
wrong field and never checked against the actual `amendment-references.json`
sibling files. It is retracted, not merely refined — see the 2026-07-30 "CNBV
legend re-pass" entry below for the corrected per-instrument counts (every
legend is populated, every current mark resolves) and the real, different
defect the re-pass found (a marker-regex digit cap that silently drops some
markers ≥100). The provision-marked counts in the paragraph below (507, 264,
260, 245, 204, 152, 96, 88, 28) are the only part that still stands; the
legend-entry-count claim ("not one instrument has a single legend entry") does
not and should be disregarded.

> Defect found while surveying the precedent: the amendment legend is empty
> everywhere it is used. `amendment_marks` on a provision resolves through the
> instrument's `amendment_references` legend, and is rendered into Markdown
> frontmatter. Across nine committed CNBV instruments, 1,844 provisions carry
> marks and not one instrument has a single legend entry
> (`socap-sofipo-dcg-2006` 507, `cucb-dcg-2004` 264, `cub-dcg-2005` 260,
> `oaac-dcg-2009` 245, `scap-dcg-2012` 204, `fi-dcg-2014` 152, `cue-dcg-2003`
> 96, `itf-dcg-2018` 88, `servinv-dcg-2013` 28).

## 2026-07-30 — CNBV legend re-pass: the 2026-07-29 "empty everywhere" claim was wrong; real defect is a 2-digit marker cap

Operator authorized a re-pass on the CNBV `amendment_references` legend
question before Stage A sign-off. Re-measuring against the actual
`corpus/mx/<id>/amendment-references.json` sibling file (not whatever field
the 2026-07-29 count read):

| Instrument | Provisions marked | Unique marks | Legend entries | Unresolved marks |
|---|---:|---:|---:|---:|
| `socap-sofipo-dcg-2006` | 507 | 62 | 97 | 0 |
| `cucb-dcg-2004` | 264 | 49 | 99 (capped)† | 0 |
| `cub-dcg-2005` | 260 | 40 | 99 (capped)† | 0 |
| `oaac-dcg-2009` | 245 | 47 | 99 (capped)† | 0 |
| `scap-dcg-2012` | 204 | 24 | 42 | 0 |
| `fi-dcg-2014` | 152 | 28 | 47 | 0 |
| `cue-dcg-2003` | 96 | 64 | 99 (capped)† | 0 |
| `itf-dcg-2018` | 88 | 14 | 18 | 0 |
| `servinv-dcg-2013` | 28 | 5 | 8 | 0 |
| `ifpe-dcg-2021` | 0 | — | n/a — no `amendment-references.json`; uses `formal-source-manifest.json` / `annex-source-manifests.json` instead | — |

`adapters/cnbv/` lists ten instruments, not nine; `ifpe-dcg-2021` is the
tenth. It carries zero `amendment_marks` and no legend file at all — it
appears to use a more evolved multi-source provenance shape already
(relevant to Stage B, not investigated further here). † = see the capped-
legend defect below; "99" undercounts the real legend length for these four.

Every mark currently on a committed provision resolves to a legend entry —
the marks⊆legend validator is not vacuously passing here, it is genuinely
satisfied for every mark the parser currently produces. The extraction lives
in `crates/lex-parse/src/itf.rs` (`flush_legend`, sharing
`amendment_marker_regex` with `crates/lex-parse/src/dcg.rs`) — I had read
`dcg.rs` alone and concluded, wrongly, that no REFERENCIAS extraction
existed at all.

**A real, more serious defect than "legend gap": silent loss of live
markers, confirmed in committed data.** Both `amendment_marker_regex`
(`dcg.rs:176`, `^\((\d{1,2})\)$`) and `legend_entry_re` (`itf.rs:179`) match
only one or two digits, capped at 99. This is not just a legend-formatting
issue: the *same* regex is used to recognize a margin marker in the body
text, so a `(100)`-or-higher marker on a provision fails to parse as a
marker at all and falls through as ordinary content. Checked directly —
`grep -oE '\(1[0-9]{2}\)' corpus/mx/<id>/provisions.json` for the four
capped instruments finds three-digit parenthesized tokens sitting inside
committed provision text today: `cucb-dcg-2004` (`(100)`, `(101)`, `(106)`,
`(108)`–`(111)`, `(113)`, `(114)`, …), `cub-dcg-2005` (`(104)`, `(105)`,
`(108)`, `(112)`–`(115)`, up through `(148)` and beyond), `oaac-dcg-2009`
(`(102)`–`(115)`), `cue-dcg-2003` (`(101)`–`(108)`). These are real
provisions whose amendment attribution is currently **dropped**, not merely
undocumented — the exact "silent loss" the 2026-07-12 entry's
marks⊆legend invariant was built to catch, except that invariant checks the
markers the parser *did* extract, so it cannot see the ones the regex threw
away before they became a mark. On the legend side, the same cap folds
`100)  …` lines into legend entry 99's `description` as trailing text
instead of a new entry — `cub-dcg-2005` alone has ~299 such embedded
`\d{1,3}\)` fragments inside entry 99, with dates running to 2026-07-03.
Not fixed in this pass — widening both regexes to `\d{1,3}` (or unbounded),
re-splitting the four affected legends, and re-extracting the swallowed
in-body markers is a scoped follow-up, distinct from Stage A, and higher
priority than its "legend gap" framing implied.

**Conclusion for Stage A:** the "populate the CNBV legend" fork of the
Stage A sign-off question is moot — the legend mechanism works correctly for
everything the parser currently recognizes as a marker. The open sign-off
item remains the `StandardModificationTarget` / `amended_by` shape itself;
the marker-cap defect above is new, separate work.

## 2026-07-30 — Trusted-compiled-source-first correctness; self-compilation is the fallback

Operator correction to the 2026-07-29 "verification story changes" framing
above. Restated in the operator's own terms: compiling everything ourselves
is more difficult and riskier than pointing at a compiled version that
already exists and is trusted to stay correct. So:

- **If a trusted compiled source exists for an instrument, "correct" means
  matching that source's file.** Self-compilation (hashed inputs + a
  deterministic transform, byte-reproducible, gated on named human review —
  the 2026-07-29 framing) is the *fallback*, used only when no trusted
  compiled source exists.
- **Trust is manually triggered, per source, never inferred.** Trusted today:
  **Diputados** and **CNBV**. **Not trusted (known uncompiled)**: the source
  referred to as "pagiina" in the operator's message — name unconfirmed
  against the adapters directory this session; needs to be pinned down
  before it is used anywhere that assumes trust.
- **Multi-source is not universal.** A single published text with no
  modificatorios can legitimately rest on one file — normally the DOF
  version when no Diputados version exists. The never-modified, single-
  source, not-in-Diputados instrument (a new NOM with no amending decree
  yet) is the outlier case the model must keep representable, not an error
  state.

**Reconciling against existing fields.** `SourceManifest` and `Instrument`
(statute/DCG side) already carry `operational_source` /
`formal_publication_source` (e.g. `cub-dcg-2005`: `operational_source:
"cnbv"`, `formal_publication_source: "dof"`) — structural support for
"which source is the one we point at" already exists there, but nothing
marks *that* source as manually-triggered-trusted versus merely preferred.
`StandardMetadata` (NOM side) has no equivalent field at all — only
`source_url` / `official_dof_url` / `official_registry_url` /
`text_basis: AsPublished | OfficialCompilation`. Neither side has an
explicit trusted-source boolean/enum today.

**Not implemented.** This entry records the doctrine; it does not add a
field. The natural next step is a `trusted_compiled_source: bool` (or an
enum naming which source) alongside `operational_source`, extended to
`StandardMetadata`, plus a validator rule that `text_basis:
OfficialCompilation` requires a trusted source and `AsPublished` does not.
That is implementation work and needs its own pass, not a byproduct of this
correction.

## 2026-07-29 — Reviewer data corrections; `published_designation` added

Three corrections supplied by the same review, kept out of the parser commit
so a regression in one could not hide behind the others.

**NOM-020-STPS-2011's recorded modification was wrong.** It cited an
alternative-procedure acuerdo of 2014-12-09, which the reviewer identified as
not the modification at all. The real one is the *ACUERDO de Modificación*
signed 15 September 2015 and published in the DOF on **2015-10-19**
(`codigo=5411988`), whose entire operative content is `ÚNICO. Se elimina el
inciso j) del numeral 13.2`. The modification record now carries that date and
URL. The distinction matters generally: platiica lists "procedimiento
alternativo autorizado" records alongside real modifications, and only the
latter make retained clause text stale.

**NOM-025-STPS-2008's effective date is 2009-03-01.** Its text sets entry into
force two months after a 2008-12-30 publication, landing on a nonexistent 30
February; the registry's own field reads an impossible `2009-02-29`, which is
why it was left `null` on ingestion. The reviewer supplied the governing rule:
an obligation falling on a non-existent or non-working day moves to the next
available working day. Recorded as a real date rather than an absence.

**`published_designation` added to `StandardMetadata`.** The reviewer granted
authority to apply a designation rename wherever the registry shows one, and
to leave it alone where it does not — noting the asymmetry: ECOL became
SEMARNAT, but SCFI persists in NOM-051 and NOM-187 even though the Secretaría
de Comercio y Fomento Industrial became the Secretaría de Economía. So the
prefix follows the *registry's* redesignation, not the authority's rename.

Verified before applying rather than assumed: across all committed standards,
every `designation` appears in its own retained text. NOM-002-SEMARNAT-1996
would have been the first exception, since its text is titled
NOM-002-ECOL-1996. Applying the rename silently would have broken a
corpus-wide invariant with nothing recording it, so the optional
`published_designation` field carries the published form and raises a
`standard_redesignated` warning; `validate_metadata` errors if it is recorded
when the two designations are equal, so it cannot become a general
"former name" field. Schema, `lex-core` type, validator, test and
`docs/standards-module.md` landed together, per the trusted-boundary rule.

NOM-002-SEMARNAT-1996 is now ingested (73 clauses, 3 transitorios), closing
the last of the six batch-2 flags that did not depend on the unresolved
`transitory-absorbs-annex` defect.

## 2026-07-29 — A standard's normative body ends at TRANSITORIOS

Reviewer-supplied domain rule, answering the batch-2 flag report: **a NOM's
normative numbered body ends at TRANSITORIOS.** What follows may be
apéndices, anexos, tablas or listados — sometimes normative — or an
explicitly non-binding "Guía de Referencia", but it is never
clause-structured, and it needs different extraction rules than the body.

Two parser changes follow from that rule. Both are bounded to
`parse_standard_clauses`; the data corrections the same review supplied
(NOM-020's modification date, NOM-025's effective date, the ECOL→SEMARNAT
designation pattern) are deliberately *not* in this change, so a regression
in one cannot hide behind four others.

1. **The clause run is bounded at the real TRANSITORIOS heading.** The
   índice-disambiguation Scope 1 built for transitorios is extracted as
   `real_transitorios_heading` and now serves both paths, so the clause and
   transitory parsers can never disagree about where a body stops. This
   closes `annex-continues-numbering` and `annex-form-numbering`. Note the
   2026-07-28 log entry explicitly *rejected* "cut at the first TRANSITORIOS
   marker" — correctly, since NOM-019's índice lists TRANSITORIOS before its
   body. Cutting at the **real** occurrence is the same idea done right, and
   the entry predicted exactly that.

2. **Form feed joins the leading-whitespace class.** `pdftotext` emits
   `\x0c` immediately before a page's first line with no intervening
   newline, so any heading landing on a page boundary never matched the
   line-start anchor. This — not a length contest — is why
   NOM-052-SEMARNAT-2005 compiled its índice: its real body was *invisible*,
   leaving the índice as the only candidate run. Verified safe before
   landing: DOF running headers ("6 (Edición Vespertina) DIARIO OFICIAL")
   also follow a form feed, but carry no ordinal period and open with `(`,
   so `plausible_top_level` already rejects them. Simulated across all 26
   committed standards, admitting the form feed changed zero clauses.

**Regression verification was deep equality, not counts.**
`standards validate` reparses committed text and bails if the result
differs from committed `clauses.json`/`transitories.json` by any field.
All 26 previously-committed standards passed unchanged, so every clause
ID, label, text and span is byte-identical.

**New check: `standard_trailing_material`.** Bounding the body at
TRANSITORIOS is correct but drops content that is sometimes normative —
NOM-052's Listados carry the hazardous-waste classifications the standard
exists to establish; NOM-010's Apéndice I carries its exposure limits.
Without a signal, a compiled standard would present a complete-looking
clause body while omitting operative content. `validate_standard` now warns
when substantial text follows the transitorios section. It fires on 13
standards and deliberately does not distinguish normative from non-binding
trailing material — making that call requires reading it, which is the
point of surfacing it. Regenerating 11 committed `validation.json` files
for the new warning changed nothing else: clauses, transitorios, retained
text and metadata all compared byte-identical.

**Newly found, deliberately not fixed here: `transitory-absorbs-annex`.**
`section_end_marker` ends the transitorios section only at a signature
marker, `APÉNDICE`, or `ANEXO`. Trailing material introduced any other way
— usually `Guía de Referencia I`, sometimes bare `Tabla N.` — is absorbed
by the *last transitory*. Pre-existing, not caused by this change, and
affecting committed standards (NOM-027's TERCERO is 22,370 chars;
NOM-085's QUINTO 27,938; NOM-020's QUINTO 15,327). It is why
NOM-019-STPS-2011, NOM-024-STPS-2001 and NOM-052-SEMARNAT-2005 stay held
out even though their clause defects are fixed: ingesting them would mean
committing transitorios known to contain tens of thousands of characters
of guide and table text. Not fixed in the same pass because adding `Tabla`
as a section-end marker could truncate legitimate transitory text that
references a table inline, and because the reviewer has indicated
post-transitorios annexes need their own extraction approach — so *how* to
model them is an open design question, not a marker list.

Ingested under the fix: NOM-010-STPS-2014 (206 clauses, was 950 phantom)
and NOM-035-STPS-2018 (111 clauses, was 124 with questionnaire rows), both
carrying `standard_trailing_material`.

**Reviewer finding on NOM-247-SSA1-2008, recorded for Scope 2.** Its two
modifying decrees name in their own titles exactly which numerals they
touch: the first modifies `1.4, 2, 3.2, 3.10, 3.12, 3.17, 3.18, 3.19,
3.36, 3.44 y 8`; the second modifies `3.2, 3.10, 3.33, 4, 5.1.1,
5.2.7.ii.1)`, adds `5.1.5`, and eliminates `5.2.2.8, 5.2.3.4, 5.2.4.5` and
`el Apéndice normativo A`. That makes a cheap intermediate capability
possible well before the full decree-diff engine: parse the affected
numerals from a decree's title and attach them to the modification record,
turning `standard_unconsolidated_modification` from an
instrument-level warning into a clause-level one — a reader could then see
that NOM-247's clause 3.2 specifically is stale, rather than that the
standard as a whole is. Not implemented; recorded as the natural first
increment of Scope 2.

## 2026-07-28 — Batch 2 complete: validation cannot detect wrong-run selection

Batch 2 ran to completion across all 27 candidates: 21 ingested, 6 held
out and flagged (`docs/plans/nom-standards-batch-2.md`,
`docs/ingestion-difficulty-log.md`). The corpus went from 160 to 177
instruments and from 1,487 to 3,238 standard clauses.

**The load-bearing finding: every one of the five parser failures in this
batch reported `valid; 0 issues`.** `validate_standard` checks that the
clause run it is handed is internally consistent — ascending numbering,
non-overlapping spans, offsets inside the text — and never asks whether
`numbered_body_run` selected the correct run. A standard can therefore
compile "clean" while carrying 744 phantom clauses absorbed from an
appendix table (NOM-010-STPS-2014), or while missing its entire body
because the índice was compiled instead (NOM-052-SEMARNAT-2005, 1.1%
clause-span coverage). Absent the hold-out-and-flag policy adopted
earlier the same day, all five would now be committed canonical data,
hashed into Maximasa's bundle lock, indistinguishable from correct
records by any automated check this repository has.

Three failure classes were newly identified and named:
`annex-continues-numbering` (the run does not stop at the terminal
heading; a following annex/table/questionnaire continuing the same
numeric sequence is absorbed — NOM-010, NOM-035, NOM-024),
`indice-selected-as-body` (the table of contents is itself a complete
consecutive run and wins selection outright — NOM-052), and
`metadata-ambiguity` applied to a legal-identity rather than parser
question (NOM-002-SEMARNAT-1996, below). These join
`annex-form-numbering` from tranche 1 (NOM-019-STPS-2011, where an annex
form's independently *restarted* numbering out-competed the real body on
raw length — a distinct mechanism from the three above, not a variant).

Two discriminators caught every case and are deliberately *not* yet
implemented as checks, because doing so is a validator change that should
be reviewed on its own terms rather than folded into an ingestion batch:
clause-span coverage as a fraction of document length (the índice failure
scored 0.011; every correct instrument scored ≥0.31), and whether the
selected run terminates at a Bibliografía/Concordancia heading. Promoting
both into `validate` is the recommended next code change; three of the
four parser classes are plausibly one shared fix in `numbered_body_run`.

**NOM-002-SEMARNAT-1996 is a decision, not a defect.** Its text parses
cleanly, but the retained official text is titled NOM-002-**ECOL**-1996
and names SEMARNAP as issuer, while the registry indexes it as
NOM-002-**SEMARNAT**-1996 under SEMARNAT. Committing it would assert a
`designation` appearing nowhere in its own source text, and
`StandardMetadata` has no field for "published as X, now indexed as Y."
Not resolved by convention here; held for reviewer direction. This will
recur for every pre-2000 ECOL-era environmental NOM, so the decision sets
a pattern rather than settling one record.

Recorded conservatively rather than assumed harmless during this batch:
NOM-020-STPS-2011's *ACUERDO de modificación* (2014-12-09) is recorded as
an unconsolidated modification, giving it a standing warning that its
clause text is not current until that decree is checked — the same
treatment NOM-247 carries. "Procedimiento alternativo autorizado" records
on other standards were *not* treated as modifications: they authorize
compliance methods, not text changes. NOM-025-STPS-2008's
`effective_date` is `null` because its text sets entry into force two
months after a 2008-12-30 publication (a nonexistent 30 February) and the
registry's own field reads an impossible `2009-02-29`; an invented date
would have been worse than an absent one.

A flag report for reviewer triage was written to Spearhead at
`20_Repos/Lex-Mex/NOM Batch 2 — Flagged Instruments for Review.md`.

## 2026-07-28 — Batch NOM ingestion: hold-out-and-flag policy, review deferred to packets

Decision, operator-directed: treat NOM ingestion as a general batch process
rather than one-at-a-time, applying the existing federal cluster-2 "Batch
operating loop" discipline (`docs/project-status.md`) with two changes.

1. **Hold out, don't force through.** When an instrument hits a structural
   difficulty that isn't a quick, obviously-correct fix (a new
   signature-block variant, a decree-diff case, an ordinal restart, an
   unresolved acquisition question), it is held out of `corpus/` entirely
   and flagged in the new `docs/ingestion-difficulty-log.md`, rather than
   compiled with a known defect. Considered and rejected: compiling anyway
   with the defect marked on the record. Rejected because `corpus/` is
   committed canonical data that Maximasa's bundle lock hashes and
   consumes directly — a known-wrong structure riding along, even flagged,
   is a defect in canonical data rather than an absence from it. Operator
   confirmed this explicitly when asked.
2. **Review no longer gates ingestion, and is deferred to packets.**
   Standards' `legal_review_status`/`technical_review_status` already
   supported an unreviewed state independent of successful compilation —
   nothing mechanical changes here. What changes is the *operating*
   assumption: JRH (or another reviewer) reviewing every ingested
   instrument one at a time is not workable at batch scale. Once enough
   instruments are ingested, they will be grouped into review packets
   (e.g. "industrial food processing pack") and handed to assigned
   reviewers — plural, not only JRH — as a batch. This is a change to
   *when and by whom* review happens, not to the reviewer-of-record rule
   itself; JRH's role for material already reviewed under the old flow is
   unaffected. Packet grouping and reviewer assignment are staged, not
   built — see `docs/plans/nom-standards-batch-2.md` "Packets."

The genuinely new artifact is `docs/ingestion-difficulty-log.md`: a
durable, failure-class-tagged record distinct from any single plan's own
`Progress`/`Surprises and discoveries` section, so a recurring pattern is
visible across unrelated batches (NOM standards now; state/municipal
corpora later) instead of buried in per-plan prose. Ordinary parser
defects that get fixed immediately still just get a regression fixture,
per the existing loop — this log is only for difficulties deliberately
left open.

Staged the first concrete batch under this policy:
`docs/plans/nom-standards-batch-2.md`, 27 candidates sourced from
Maximasa's `nom-register.md` (Tables 1, 3, and 4 minus the five already
canonical). Corrected in the same pass: the batch is 27, not the round 25
first estimated conversationally — Table 2 (SSA1/SCFI) is fully done
(4/4), leaving 22 STPS + 3 SEMARNAT + 2 gap-analysis instruments. Also
corrected: standards acquisition has no adapter yet
(`docs/standards-module.md`), so locating each official source remains
real manual work — the "easy ones ingest quickly" framing only ever
applied to parsing/validation, not sourcing.

## 2026-07-27 — Scope 2 staged, not started; Maximasa NOM slice closed for now

Decision: land Scope 1 (below), then explicitly stage Scope 2 (the
decree-diff engine) as recorded future scope under M4 in
`docs/plans/maximasa-legal-integration.md` rather than beginning it in the
same session. Operator instruction: finish the current Maximasa NOM
ingestion/processing slice first; plan Scope 2 separately later.

Closing this slice required two things beyond Scope 1's own code:

1. `docs/project-status.md` and `docs/standards-module.md` gaps a review
   pass caught before Scope 1 could be called done: the new
   `transitories.json` artifact class wasn't reflected in the corpus
   totals table, and two facts needed verification rather than assumption
   — that standards' lack of a Markdown export profile genuinely leaves
   `Generated Markdown files` unaffected, and that NOM-002-STPS's
   `2000-09-08` asserted date is a verified predecessor-publication
   citation in the transitorio's own text, not another signature-block
   leak like the one just fixed in NOM-051.
2. A second mechanical bundle refresh in Maximasa: `transitories.json`
   becoming a required `bundle create` output file changed the five-NOM
   bundle from 4 files/standard (20 total) to 5 (25 total). Regenerated
   `nom-bundle-manifest.json` at Lex-Mex `a3a48296f`, reran
   `build_demo_data.py`, and updated Maximasa's hardcoded bundle-lock test
   expectations (`selected_files_checked` 20→25, new sha256) — all 14
   Maximasa tests pass with every locked file verified against the live
   corpus. No NOM was rechecked against its official source; this is the
   same standing mechanical-only refresh authorized 2026-07-26 ("update
   HEAD and hashes, but do not recheck the NOMs"), triggered by a new
   required file rather than new corpus content. Also added the now-
   machine-visible NOM-051 transitorio-staleness fact (see Scope 1's
   finding below) to `nom-candidate-package.yaml` and `OPEN_QUESTIONS.md`
   O-7, since it was flagged pending and this refresh was the natural
   point to fold it in.

## 2026-07-27 — Standards transitorio inspection (Scope 1)

Operator-scoped, bounded implementation: two sub-scopes were carved out of
the transitorio/decree-diff idea flagged 2026-07-26 (see below) — "Scope 1"
(transitorio inspection, lightweight, this entry) and "Scope 2" (the
decree-diff engine, not started). Explicitly *not* a full structural parse
of transitorio content: some standards' or statutes' transitorios are long
and complex (heavily amended instruments especially), and the goal was
addressability plus date-scanning, not understanding everything a
transitorio says.

`StandardTransitory` (`lex-core`), `parse_standard_transitories`
(`lex-parse/standard.rs`), and `transitories.json` (new committed file per
standard, required by `bundle create`, addressable via
`lex-mex path --kind transitories`) landed together with
`schemas/standard-transitory.schema.json`. The ordinal recognizer
(`transitory_ordinals`, `parse_transitory_start`) is not new code: it
already existed in `diputados.rs` for statute transitories and was made
`pub(crate)` for reuse rather than reimplemented — the "always serialized"
peculiarity the operator named was already solved there.

Backfilled `transitories.json` for all five already-committed NOMs by
recompiling from their already-retained source/text (no new official-source
research); all five reparsed with clause counts and issue counts
byte-identical to their committed state before this landed.

Three parser defects surfaced compiling this against NOM-051's real
retained text (fixtures added for each, see `docs/standards-module.md`
"Standards transitorio inspection" for detail): the índice's repeated
TRANSITORIOS heading matched first instead of the real section (same
class of bug as the Bibliografía heading fix, 2026-07-26); an untrimmed
line with leading indentation was passed to the ordinal recognizer, which
requires the ordinal at position 0; and the signature-block marker didn't
recognize the post-2016 "Ciudad de México, a ..." dateline (only the
pre-CDMX-renaming "México, D.F., a ..." form), letting a decree's closing
signature and its own sign-off date bleed into the last transitorio's
text and asserted dates.

**Finding:** NOM-051's `transitory:segundo` still asserts the original
2020 decree's `2025-10-01` phase-three date. Two 2025 ACUERDOs (see the
2026-07-26 entry) pushed the real date to `2028-01-01`; neither is part of
the retained source, and nothing before this landed could represent or
surface that staleness at all. This doesn't close the gap — reading the
current true date still requires knowing about both ACUERDOs — but it
makes the staleness checkable against the retained text for the first
time, and is the addressable prerequisite Scope 2 needs ("replace
transitorio SEGUNDO's date" requires SEGUNDO to be an addressable object).

## 2026-07-26 — How to read a platiica/DOF NOM record, for every future ingestion

Operator-supplied methodology (verified against NOM-051-SCFI/SSA1-2010's
registry record and DOF/SIDOF primary sources), recorded here because it
governs every future NOM/NMX ingestion, not just this one.

**A platiica registry record's `Historial Documental` mixes several
distinct document kinds under one list; each needs different treatment:**

- The NOM itself (the base PDF, which may be stale — NOM-051's stayed at
  its 2014 state until this session's 2020 re-source).
- `Procedimientos para la evaluación de la conformidad`: a separate,
  complementary instrument (a compliance checklist), not part of the NOM's
  own text. Not a modification.
- `PROYECTO de Modificación`: never normative. Always excluded. Reliably
  identified by starting with the literal word `PROYECTO` and by carrying
  no vigencia/in-force date on the DOF page (shows as blank/`XXX`).
- `MODIFICACIÓN`: an actual normative change to the standard's text.
- `ACUERDO`: see below — do not assume scope from the type name.

**`ACUERDO` is not a fixed category of change; it is the generic official
name for any binding decision a competent authority publishes in DOF
within its jurisdiction.** It can be substantive and close to NOM-level
content (e.g. an ACUERDO determining permitted food additives — normative
because that authority chose ACUERDO as the default publication vehicle
rather than a full NOM modification), or it can touch only a modification's
transitorio dates (as NOM-051's two 2025 ACUERDOs do), or it can itself
read "ACUERDO por el que se modifican, adicionan y derogan ...". The name
never tells you the scope — read what it actually says every time.

**How to read a `MODIFICACIÓN`'s (or an ACUERDO's) own diff against a base
text:** these decrees state their own substitution instructions verbatim,
in the decree's own text, not as an editorial summary. A run of untouched
numerals or phase labels followed by `...` means that span is unchanged;
whatever numeral or label follows next is given in full, and that full
text is the verbatim replacement (or addition, or derogation) for that
specific numeral. Example, confirmed directly against DOF's page for the
2025-07-31 ACUERDO (`https://diariooficial.gob.mx/nota_detalle.php?codigo=5764197&fecha=31/07/2025`):
"PRIMERA FASE. ... SEGUNDA FASE. Del 1 de octubre de 2023 al 31 de
diciembre de 2027 ..." — `PRIMERA FASE` is untouched; only `SEGUNDA FASE`
(and, further down, `TERCERA FASE`) received new text. Applying this is
mechanical substitution against the decree's own explicit instructions,
not Lex-Mex performing its own legal consolidation — the distinction that
kept NOM-247 out of scope for a self-consolidation still holds (no
official consolidated *republication* exists for its decrees), but the
per-numeral substitution mechanism itself is a legitimate, deterministic
operation Lex-Mex can perform once a decree gives it explicitly.

**`Bibliografía` splits into two kinds of entries, only one of which is a
traceability chain:** cited Leyes, Reglamentos, and Acuerdos form a real
parent-authority hierarchy (a NOM is issued under a Reglamento under a Ley,
occasionally under a further Acuerdo) worth backlinking once the target is
in the Lex-Mex corpus. ISO guides and academic citations are not normative
and should be excluded from that hierarchy.

**Finding that reopens NOM-247:** the earlier conclusion (2026-07-26,
`maximasa-legal-integration.md`) that incorporating NOM-247's two 2011/2012
modifications would require Lex-Mex to perform its own legal consolidation
no longer holds as stated. Both decrees use the same
unchanged-span-then-explicit-replacement convention described above. If
that pattern is mechanically parseable the same way for NOM-247's finer,
numeral-level scope, incorporating them is applying the decrees' own
stated instructions, not synthesizing text Lex-Mex invented. Not yet
implemented; see the standards-module note this same date for the
concrete blocker found while attempting a NOM-051 pass under this
methodology (transitorios have no structured representation at all in the
current standards schema).
## 2026-07-16 — Batch completion closes its bounded graph

`batch run` now has a deterministic closure phase after every successfully
processed selected instrument has entered the corpus. It reverse-relinks each
such instrument against the complete committed sibling set, validates it, and
regenerates Markdown (and an explicitly requested Obsidian target). A batch
cannot report success if this bounded closure fails.

`expected_edges` is now a recall oracle rather than an unused planning note.
Concrete entries use `SOURCE -> TARGET` or `SOURCE articulo N -> TARGET`,
where each name is a committed corpus slug or short name. The batch report
records every check as `satisfied`, `missing`, `deferred`, or `invalid`.
Missing and malformed concrete expectations fail closure; a target absent from
the committed corpus, or a source not processed in this run (including
`--only`), remains explicitly deferred rather than producing an invented edge
or a false pass. This remains a bounded batch check, not a substitute for the
deferred corpus-wide relink and review program.

## 2026-07-16 — Unanalyzed temporal status is unknown

A consolidated current source establishes the wording the publisher presents;
it does not, by itself, establish that every printed provision is legally
effective. Consolidations may retain provisions affected by judicial
invalidity, delayed commencement, or another temporal condition. The prior
parser default introduced at `9429d2bb` therefore made an unsupported legal
inference by assigning `effective` before temporal analysis.

Freshly parsed ordinary provisions now start `unknown` while
`review_status` remains `not_analyzed`. Only an explicit repeal note at the
start of the source text (`Se deroga`, `Derogado`, and the existing narrow
variants) starts `repealed`; that is a deterministic transcription of the
publisher's express notation, not a model or reviewer conclusion. Persisted
machine-accepted, review-required, and lawyer-verified temporal
determinations continue to override the parser's initial state and retain
their evidence hashes, bases, dates, confidence, and review history.

Validation enforces this boundary: a `not_analyzed` provision must carry the
deterministic initial status implied by its exact text, so a future parser,
import, or hand edit cannot silently restore `effective` as an unanalyzed
default. The one-time canonical migration changed 30,124 unanalyzed ordinary
provisions across 144 corpora from `effective` to `unknown`, with matching
generated Markdown. It left 3,592 explicit repeal notes, one pending reviewed
repeal, and all 21 accepted or lawyer-verified effective determinations
unchanged. This migration records no JRH legal-review decision and does not
alter official source text, reference edges, terms, provenance, or temporal
evidence.

## 2026-07-14 — Diputados split headings and reform-appendix identity

Ingesting the Reglamento del Senado de la República exposed two independent
layout boundaries in Cámara de Diputados consolidated PDFs that validation
counts alone did not catch:

- **A bare article heading followed by a numbered paragraph stays one
  article.** The PDF prints `Artículo 1` on one line and `1. ...` on the next.
  Collapsing both lines before parsing produced the false compound identifier
  `Artículo 1 1`. When a line contains only a valid article heading and the
  next line starts a dot-delimited paragraph numeral, the parser now supplies
  the omitted heading/body separator while preserving that numeral in the
  canonical text. Genuine compound headings such as `Artículo 15 Bis 1`
  remain unchanged.
- **A true decree title is a hard reform-appendix boundary.** Page furniture
  can otherwise join the preceding signature or errata page to the next
  decree. An uppercase `DECRETO` title (plus the documented older title-case
  forms) opens that boundary; a wrapped legal sentence beginning `Decreto de
  ...` does not. Likewise, a DOF publication phrase changes the containing
  decree date only before its transitory section begins. The same phrase
  inside a transitory remains canonical evidence instead of silently
  re-dating that and all following transitories. Singular/plural `ARTÍCULO(S)
  TRANSITORIO(S)` headings, colon-ended ordinals, signature blocks, and `Fe de
  erratas` pages are normalized explicitly. Operative `ARTÍCULO ÚNICO` text
  remains outside temporal evidence; only the decree's transitories enter
  `reform-temporal-evidence.json`.
- **Numbered reform transitories are evidence headings.** Inside an explicit
  transitory section, older `ARTICULO 1o.-` / `ARTICULO 2o.-` forms are parsed
  as transitories, not discarded as operative decree articles. This preserves
  the RGIC decree of October 21, 1966 alongside its ordinal-form peers.
- **Same-day decrees receive distinct temporal-evidence identities.** The
  first decree published on a date retains the established
  `:amendment:YYYY-MM-DD:transitory:<ordinal>` ID. A later decree on that same
  date uses
  `:amendment:YYYY-MM-DD:decree-N:transitory:<ordinal>`, where `N` is its
  one-based source order for that date. This keeps existing non-colliding IDs
  stable while preventing several same-day `ÚNICO` provisions from collapsing
  onto one identity. Publication-date extraction still reads the first ten
  characters after `:amendment:`.

The Senate regulation now yields 313 articles, 4 original transitories, 47
resolved canonical references, and 39 uniquely identified reform
transitories. Temporal analysis remains deferred; this structural ingest
creates no machine conclusion and no legal-review resolution.

The RGIC exercises the combined rules: 214 articles, 2 original transitories,
30 resolved references, and 23 reform transitories attributed to their actual
DOF dates. Its 359 canonical paragraphs match the official extracted text
after removing only configured running-page furniture. Temporal analysis is
likewise deferred.

## 2026-07-12 — Old CNBV compilation format (2003–2015 DCGs)

Ingesting the six older CNBV disposiciones (cue-2003, cucb-2004, cub-2005,
socap-sofipo-2006, oaac-2009, fi-2014) generalized the `itf-dcg` parser,
which had been tuned to the 2018 vintage. The format diverges in several
ways at once; each is handled additively so the committed itf-dcg-2018,
scap-dcg-2012, and ifpe corpora stay byte-identical (verified by the itf
fixture tests and a scap re-parse):

- **Preamble/índice is skipped until the first article.** These documents
  open with a table of contents that echoes a `TRANSITORIOS` heading and the
  annex list — each with its own `(N)` markers — before Artículo 1. Region
  transitions and marker accumulation are gated on `body_started`, set at
  the first article heading; otherwise the índice `TRANSITORIOS` echo flips
  the scanner into the transitorios region and every body marker strands
  (the SOCAP/OAAC failure, ~1,600–3,200 stranded marks). Índice markers are
  redundant with the same marker on the provision they annotate — still
  recorded in the REFERENCIAS legend — so preamble markers are dropped.
- **Ordinal article abbreviations** (`Artículo 1o.-` … `9o.-`, also `º`/`°`)
  are accepted and normalized to the plain number (`1o` → `1`, as `8 ≡ 8o`).
- **Feminine and singular transitorios** — the section heading in
  `TRANSITORIO`/`TRANSITORIA`/`TRANSITORIAS` as well as `TRANSITORIOS`, and
  feminine ordinals (`ÚNICA`, `PRIMERA`…) — because a "Disposición
  Transitoria" is feminine.
- **Attribution dates are accumulated across line wraps** that can split the
  date itself (`… el 12 de enero de` / `2015)`); the date before the closing
  paren resolves the section (its markers otherwise strand).
- **A marker at the foot of a section attaches to that section's last
  provision** before the section is flushed at a `TRANSITORIOS` heading;
  a true remainder is heading-level marginalia and is dropped. Markers in a
  trailing CONSIDERANDO or the REFERENCIAS legend are likewise marginalia,
  dropped rather than errored. A structural mis-parse now surfaces through
  the article-count/gap and legend-presence checks rather than a stranded
  marker.
- **Parenthesized legend numbers** (`(N)  text`) are accepted alongside the
  bare `N)  text` form.
- **`allow_article_gaps: true`** on each adapter: these compilations renumber
  away derogated articles (e.g. cue has 15 Bis with no bare 15), a legitimate
  gap, so the sequential-order check yields warnings, not errors.

All six validate with zero errors, counts unfrozen (matching the
scap/servinv precedent). Result: cue 114 arts, fi 232, cucb 337, oaac 295,
socap 548, cub 705.

## 2026-07-12 — CNBV amendment resolution, in-force status, corpus currency

Ratified with JRH after the CNBV DCG batch surfaced how the `(N)` markers
actually work. Full spec: `docs/cnbv-consolidated-disposiciones.md`. This
extends the earlier "amendment markers on reform transitorios" entry below;
it does not contradict it.

- **Markers attach to *any* structural node**, not only articles/transitorios
  — the denominación, a TÍTULO, a párrafo, a fracción. OAAC's compiled title
  opens with `(18)`. The `itf-dcg` parser's `discard` errors on
  cue/cub/cucb/fi were the parser being **provision-centric**, not the
  documents being corrupt: a marker in a CONSIDERANDO or attribution block is
  valid. Fix is attach-to-nearest-node, keeping the true error only for a body
  marker with no legend entry.
- **REFERENCIAS is the validation oracle.** Every body marker `N` must resolve
  to a `REFERENCIAS[N]` legend entry (`{ acción, fecha_DOF }`); an unresolved
  `N` is a hard error (anti-silent-loss), an orphaned legend entry a warning.
  The body-marker set ⊆ legend key set is the invariant that verifies the
  socap/oaac region-detection fix.
- **Keep the marker → REFERENCIAS link; defer the marker → transitorio link.**
  The authoritative, deterministic layer (integer key into a numbered legend)
  is built and kept, so a reader sees what changed, when, and by which RM.
  This refines the prior "keep the mention, no link" to "keep the *reference*
  link." The modifying resolutions (RMs) are **not** corpus instruments — only
  the final compiled text is; ingesting RM texts would balloon the corpus
  (the CUB alone has hundreds of RMs). Wiring the transitorio link is a
  future option, not a current requirement.
- **DOF date is not a unique RM key.** Two RMs can share a DOF date (CUIFE
  *11a* and *12a* both 08/01/2015) — an outlier, but the model accounts for
  it: if the transitorio layer is ever wired in, a colliding date yields
  attach-all-candidates + warning, never a machine-picked ordinal. Future aid:
  a snapshot of the CNBV Normatividad "Resoluciones Modificatorias" listing
  (carries ordinals + dates) disambiguates, and doubles as an update signal.
- **In-force status: live vs. staged (design proposal, generalizes to all
  instruments).** The useful signal is whether a provision is operative today.
  Per-RM TRANSITORIOS blocks (already captured as `TemporalEvidence`) state
  entry-into-force: default next-day, but OAAC stages provisions into 2027,
  and deadlines get extended. Proposed statuses `live` / `staged` /
  `staged_extended` / `unknown`, likely a **computed overlay** on `Effective`
  (status = what the law says; liveness = whether *today* is past the
  effective date) so the corpus stays date-stable. Touches `TemporalStatus` /
  effect categories → schema-boundary path; shape awaits JRH sign-off.
- **Corpus currency (new requirement).** The CNBV refreshes compiled PDFs on
  new RMs, with the page lagging days (ITF-DCG-2018 refreshed a Thursday,
  reflected later). A scheduled mechanism must re-acquire source hashes,
  snapshot the RM listing, cross-check the latest REFERENCIAS date, and emit a
  currency report to review — never auto-committing changed law. Subsumes the
  ITF-DCG-2018 reform-re-ingest TODO as its first flagged case.
- **Definitional remittance deferred to the cross-instrument pass.** A bare
  glossary remittance ("Valores: a los considerados como tales por la Ley del
  Mercado de Valores") resolves transitively to the target instrument's
  glossary entry (LMV art. 2 fr. XXIV) by **lemma-join** — deterministic only
  once LMV is in the corpus and the headword maps 1:1. Runs once after full
  federal ingestion (near complete), then incrementally. Not built in the DCG
  parser.

## 2026-07-12 — Amendment markers on CNBV reform transitorios

CNBV consolidated disposiciones (DCGs) carry numbered `(N)` superscript
amendment markers that reference a REFERENCIAS legend — version-control
provenance recording *when* a provision was amended and *by which*
modifying resolution. The modifying resolutions are **not corpus
instruments** (only the final compiled text is), so the marker is kept as
a mention with no outbound link (JRH, reviewer of record for CNBV).
Markers on articles and original transitorios were already captured as
`Provision.amendment_marks`; but these texts also **re-amend their own
reform transitorios**, so a marker can land inside a per-resolution
TRANSITORIOS section. The `itf-dcg` parser previously errored there
(a reform transitory becomes `TemporalEvidence`, which had no marks
field) rather than silently drop provenance.

`TemporalEvidence` now has an optional `amendment_marks: Vec<u32>`
(`skip_serializing_if` empty, so the committed IFPE/ITF reform evidence is
unchanged — their reform transitorios carry no marks). A marker preceding
a reform-transitory ordinal, or on its continuation lines, attaches to
that transitory exactly as it would to an article. Only a marker with no
open transitory to receive it (inside the parenthesized attribution
block) is still surfaced as an error. First exercised ingesting
`scap-dcg-2012` (parser `itf-dcg`): 382 articles, 204 provisions carrying
marks, 6 reform transitorios carrying marks (e.g. SEGUNDO/2018-01-23 →
[39]); text stays clean of the raw `(N)` glyphs.

## 2026-07-12 — `Ñ` is a distinct letter in canonical article identifiers

LFT article 353 runs a letter-suffix series (`353-A` … `353-U`) that
includes both `353-N` and `353-Ñ` — two distinct articles. The label
grammar and the retired Python tooling both folded `Ñ`→`N` (accent
stripping / NFD + drop-combining-marks), collapsing them onto one
canonical id `…:article:353-n`; the vault only ever held a single folded
`articulo-353.md`, a defect this normalization corrects.

`Ñ` is a distinct letter of the Spanish alphabet, not an accented `n`.
The canonical slug therefore **preserves `ñ`** (lowercased UTF-8): `353-Ñ`
→ `…:article:353-ñ`, file `articulo-353-ñ.md`, distinct from `353-n`.
Only article-label slugs (`labels.rs`) preserve it; defined-term slugs
(`terms.rs::slug`) still fold `ñ`→`n` and remain ASCII, because the
term-id schema constrains ids to `[a-z0-9-]+`. No committed provision or
reference schema constrains the id charset beyond the `urn:lex-mx:`
prefix, so non-ASCII article ids validate. For ordering, `Ñ` sorts
between `N` and `O` (Spanish collation), matching how the law sequences
353-N, 353-Ñ, 353-O. Only LFT carries an `ñ` article label, so the three
committed corpora and every earlier bulk instrument reparse
byte-identically.

## 2026-07-12 — Reference-graph rules for bulk código ingestion

Ingesting the foundational codes (CCom, CPF, CNPP, CFPC, LAmp, LBM,
LGTOC, LTOSF) surfaced reference-resolution and structural cases the
single-statute slice never hit. The rules settled here:

- **A citation classified as internal that resolves to no existing
  provision is dropped, not committed.** A dangling internal edge is a
  broken link, almost always a still-external citation this pass could
  not name — for example CNPP article 167's offense catalog, whose
  "Código Penal Federal" context is declared once at the top and
  resolves only through the named-offense authority table (wiring
  deferred to the penal batch). Dropping keeps the graph free of broken
  links; a genuinely missing article surfaces through the frozen count
  baseline instead. Cross-instrument edges (target is another
  instrument) are still emitted when unresolved, so a configured
  external target that does not exist still fails validation. The three
  committed instruments have no unresolved internal edges, so they are
  byte-identical.
- **Reference citations recognize compound identifiers (`95 Bis 3`),
  hyphenated qualifiers (`156-Bis`), and the adjectival Constitution
  reference (`el artículo 134 constitucional`).**
- **A backward "preceding-law" context scan was tried and rejected:** it
  fixed the "De la Ley X, los artículos N; N Bis; …" list pattern but
  mis-attached a prior citation's law name to a following citation
  (`artículo 20 de la Ley y … el artículo 11`), perturbing the audited
  DCG graphs. The drop rule above reaches the same corpus outcome (no
  edge) without that risk.
- **Reform-decree transitorios are kept out of the instrument by two
  guards:** a second transitory-section header, and a repeated ordinal,
  each end the statute's transitory section, since a statute has one
  section with unique ordinals (LAmp interleaves several decrees'
  `PRIMERO…` sets before the reform-appendix marker).

## 2026-07-11 — The repository is the only ingestion and processing gate

Between 2026-07-08 and 2026-07-10, a Python tool suite living inside the
Obsidian vault (`Herramientas/`) bulk-imported 135 additional instruments
directly from Cámara de Diputados consolidated PDFs, with its own parsing,
linking, and audit rules, no version control, and no schema gate. That
created two divergent rule sets and made the vault the only holder of
canonical facts for those instruments — exactly what this repository's
architecture forbids.

Decision (protocol designer, 2026-07-11): the repository's Rust pipeline is
the sole ingestion and processing gate. The 135 vault-only instruments will
be re-ingested through it (structural ingestion first; temporal analysis
deferred and run later per batch by legal priority). The vault returns to a
visualization/interaction layer only; the Python tooling is frozen
immediately and retires at parity.

What folds into the repository from the vault tooling:

- **`batches/*.json`** — 26 batch-ingestion manifests (25 converted from
  `Herramientas/import_batches/` with the F2/F3/F4 variant schema
  normalized, plus `legacy_core_pre_manifest` reconstructing the ten
  instruments imported before manifests existed). `blocked` entries and
  their reasons are preserved verbatim; blocked sources stay blocked until
  a reviewer clears them. Schema: `schemas/batch-manifest.schema.json`;
  Rust boundary type: `BatchManifest` in `lex-source`.
- **`adapters/diputados/_instrument-aliases.json`** — the hand-curated
  citation-alias table (official titles, accent-stripped variants,
  colloquial names such as "Circular Única de Bancos").
- **`adapters/shared/_named-offenses.json`** — the hand-transcribed CNPP
  art. 167 → CPF named-offense authority table (21 offenses), wiring
  deferred to the penal batch.

Known vault-side defects (Obsidian-invisible mid-block term anchors,
letter-suffixed articles folded into parent files, embedded page running
headers) are not repaired in place; re-ingestion supersedes them.

Count expectations for bulk instruments are parser-proposed frozen
baselines: the first successful parse proposes counts, they are written
into the adapter marked machine-proposed (distinct from the hand-audited
counts of the three original instruments), and subsequent runs enforce
them as drift detection.

## 2026-07-06 — Second-pass code review fixes on the ITF DCG ingestion

An external review of the amendment-marker and relative-reference work found
eight issues, six of them real correctness bugs. All eight are fixed here.

**Main document extraction lost its page-break markers.** `run_extract`
gated `keep_page_breaks` on `parser == "ifpe-dcg"` only; the newer `itf-dcg`
parser was never added, so its compiled main document was extracted with
`pdftotext -nopgbrk` even though `itf.rs` explicitly scans for `\u{c}` to
decide whether a paragraph legitimately continues across a page boundary.
The page-break-aware merge logic was silently dead code for the whole
~105-article main document (annexes were unaffected — they hardcode the
flag separately). Fixing this and reparsing corrected 24 provisions where a
page break had incorrectly glued two paragraphs together — most visibly,
fraction III of article 54 had been silently merged into fraction II's
text, invisible to fraction-anchor linking. Word-level fidelity re-verified
across all 2,132 canonical paragraphs; temporal evidence text is untouched
(the reform-transitory scanner never used page-break state), so all 17
persisted determinations, including the pending SÉPTIMO review, re-applied
unchanged.

**A shared "pending marker" mechanism replaces two independent, drifted
copies.** `dcg.rs`'s `parse_annex_document` and `itf.rs`'s main-document
scanner had each grown their own hand-rolled version of "hold a marker,
swallow the blank line right after it, drain onto whichever provision
comes next" — and the two copies had already diverged into different bugs:

- In `parse_annex_document`, a page-number footer between a marker and the
  following blank line didn't reset the "swallow the next blank" flag, so
  the footer let that swallow-intent leak across itself onto a blank line
  the marker was never actually adjacent to, incorrectly merging two
  paragraphs. Fixed by making every non-blank, non-marker line — including
  a footer — sever that adjacency, matching how an ordinary content line
  already did.
- In `itf.rs`, a marker appearing inside a per-resolution TRANSITORIOS
  section was queued but never drained anywhere, since a per-resolution
  transitory becomes `TemporalEvidence`, which has no `amendment_marks`
  field to receive it — and the CONSIDERANDO/REFERENCIAS transitions
  didn't clear the queue either, unlike the TRANSITORIOS transition and
  the four structural-heading transitions, which did. The marker simply
  vanished with no trace.

Both are now the same shared `PendingMarks` type (in `dcg.rs`, used by
both parsers): `push`/`drain_onto` for the normal case, and `discard`,
which **errors** instead of silently dropping a marker at a boundary with
no receiver — a per-resolution transitory, a considerando, or the legend.
Discovering a real document exercises one of those cases needs a human
look, not a silent loss of provenance.

That strict rule has one evidenced, deliberate exception:
`discard_from_heading` clears silently at a Título/Capítulo/Sección/
Apartado boundary (and at a TRANSITORIOS/REFERENCIAS transition reached
directly from Body), because the real document repeals an entire Apartado
with no article of its own — a heading followed by a lone `(Derogado)`
note, itself marked. `HeadingContext` has no field to receive a mark, but
the fact is always redundant with the same marker already recorded
directly on the individual provisions the heading covers, so nothing is
lost by discarding it there.

**`orphan_paren_re` narrowed to a self-verifying retry.** The regex
repairing article 21's glyph-splitting artifact (`) Artículo 21.- …`) was
applied to every line of the whole document up front, with nothing but the
literal text "Artículo" constraining it. It now only runs as a fallback at
the exact point of trying to match an article heading, and is accepted
only when stripping the leading `) ` actually turns the line into a real
`article_re` match — so it can never alter a line that merely happens to
start the same way without being a mis-rendered heading.

**Reform-evidence ID/label construction is now one shared function**
(`reform_evidence_item` in `lib.rs`), called by both LRITF's
`ReformEvidenceBuilder` and the ITF DCG's `flush_reform` — closing a
literal duplication of the `{instrument_id}:amendment:{date}:transitory:
{ordinal}` convention. Each caller still assembles its own `text` before
calling it (LRITF's decree appendix is block-scanned and paragraph-joined;
the ITF DCG's resolution sections are line-scanned and space-joined) — the
two are not forced into a shared join strategy, since doing so would have
altered persisted, already-hashed evidence text for one or the other.

**Reform-evidence file write-gate restored to its original invariant, and
correctly extended.** A prior fix changed the write condition from
`parser == "lritf"` to `!reform_evidence.is_empty()`, so a future reparse
producing zero reform evidence would leave a stale non-empty file on disk
rather than overwriting it to `[]`. Restored to writing unconditionally —
even when empty — gated on `matches!(parser, "lritf" | "itf-dcg")`,
extending the original LRITF invariant to the new parser instead of
narrowing it for both.

**The shared annex marker-stripping logic added for the ITF DCG was
verified against the real IFPE DCG-2021 corpus, not just asserted safe.**
The margin-marker regex in `parse_annex_document` is shared unconditionally
across both DCG parsers, but only 2 of IFPE's 8 real annexes have fixture
coverage. Refetched and re-extracted all 8 real annex PDFs (byte-identical
to what's committed), confirmed zero standalone marker-shaped lines exist
in any of them, and reparsed the full instrument: zero annexes gained an
`amendment_marks` entry, and `provisions.json`/`references.json` are
byte-identical to what was already committed.

## 2026-07-05 — Compiled-document amendment markers as structured provenance

The compiled CNBV document for the general Fintech DCG (DOF 10/09/2018,
six resoluciones modificatorias through 09/09/2025) prints a numbered
margin marker (`(7)`) beside every amended block, and closes with a
REFERENCIAS legend mapping each number to its amending resolution and
action (Reformado / Adicionado / Derogado / Sustituido). Following the
standing rule that compiled documents are the operational source and
resoluciones are provenance references — never individually extracted —
the markers are treated as structured marginalia, not prose:

- Markers are removed from canonical provision text (they are typography,
  like page-number footers) and recorded per provision as
  `amendment_marks`, deduplicated and sorted.
- The legend is parsed into corpus-level `amendment_references`
  (`amendment-references.json`), keeping the verbatim legend text.
- Marker placement is spatial: the layout extraction emits each marker at
  the vertical position of the text it annotates, which can be just
  before a provision's heading line or between its body lines. Markers
  are therefore held pending and attached to whichever provision the next
  content line belongs to; structural headings (títulos, capítulos,
  secciones, apartados) clear them, since a chapter-title mark is not
  provision provenance.
- A blank line immediately after a marker line is part of the marker's
  own line box: paragraphs flow across markers unbroken.
- Inline parenthesized numbers in prose (`un (1) reporte`) are untouched —
  only whole-line markers count.
- One glyph-splitting artifact exists in the source PDF (article 21's
  marker renders its closing parenthesis at the start of the heading
  line); the orphan parenthesis is removed deterministically and the case
  is fixture-covered.

Word-level fidelity holds: all 2,104 canonical paragraphs of the ITF DCG
are exact substrings of the extracted sources after removing exactly the
markers, page numbers, and the one orphan parenthesis.

Each of the six per-resolution TRANSITORIOS sections after the original
one is attributed to its resolution by the parenthesized block following
the heading, and its articles become reform temporal evidence
(`…:amendment:<dof-date>:transitory:<ordinal>`), mirroring the LRITF
reform-decree appendix. Only the original 2018 transitories are canonical
provisions. `latest_reform_date` derives from the maximum attributed
resolution date. The instrument deliberately has no formal DOF source
acquisition: the compiled document consolidates seven DOF publications,
and per-resolution provenance lives in the legend and the adapter's
`relevant_reform_transitories`; the original DOF nota can be attached
later if a decision comes to depend on it.

## 2026-07-05 — Relative article references

`artículo anterior` / `artículo siguiente` are express citations whose
target is inferred from position rather than named, so they carry the new
distinguishable `reference_form: relative` instead of masquerading as
direct numeric citations. Resolution walks the source provision's
same-type sequence in document order: a transitory's `anterior` is the
previous transitory, never the last numbered article, and the instrument
title (which has no position) can never carry one. A phrase with no
neighbor in its direction — `artículo anterior` inside the first article —
produces no edge.

Deliberate exclusions, each deterministic:

- The plural `los artículos anteriores` names an open-ended set with no
  single target; it stays unlinked (three LRITF occurrences).
- Bare self-references (`este artículo`, `el presente artículo`, 174
  occurrences) are not extracted: the reader is already inside the target,
  and the useful fraction-scoped form (`fracción N del presente artículo`)
  is already handled by the same-article path.
- `del citado artículo anterior` still resolves, but the intervening word
  keeps the pre-number qualifier from attaching — the qualifier machinery
  requires exact adjacency (`del`/`de los` ending at the header) and does
  not guess across words.

The pre-number qualifier pattern also gained the noun-first paragraph form
(`párrafos segundo y tercero del artículo N`) and the `penúltimo` ordinal,
both fixture-tested; `penúltimo párrafo` appears on two LRITF article 138
relative edges today, the noun-first form has no numeric-target occurrence
yet in either instrument.

## 2026-07-03 — Fraction-level references and previews

A fraction never exists in isolation — `fracción XI` only means something
relative to its article — so fraction precision is layered onto article
edges rather than modeled as standalone targets. Three additions:

1. **Pre-number qualifiers.** Phrases written before the article number
   (`las fracciones II, III, IV y V del artículo 22`, `el séptimo párrafo
   del artículo 29`) are captured when they end exactly at the `artículo`
   header, connected by `del`/`de los`, and attach to every article in the
   cited list. Previously only post-number qualifiers were captured.
2. **Anchored qualifier spans.** `ReferenceQualifier` gains optional
   Unicode character offsets, validated against the unchanged canonical
   text like edge spans. Offsets are backward compatible: existing
   qualifiers without offsets remain valid.
3. **Same-article fraction citations.** `fracción N del presente artículo`
   / `de este artículo` produces one edge per numeral, targeting the
   containing provision, spanning exactly the numeral, and only when the
   provision actually has that fraction as a paragraph.

Presentation uses a dual affordance because a native Obsidian hover can
preview either a whole note or a single block, not a composed
article-header-plus-fraction view: the article number keeps its whole-note
link, and each fraction numeral in an anchored qualifier links to the
target's `^f-<n>` block — `fracción [[articulo-36#^f-xi|XI]] del artículo
[[articulo-36|36]]`. Same-article numerals link to the provision's own
fraction block. A numeral links only if the target note actually has the
fraction anchor; otherwise it stays plain text. Anchor links are
Obsidian-only (standard Markdown has no block anchors). Generating a
per-fraction note to get the composed preview remains a possible later
presentation add-on.

Enabling same-article extraction grew the audited graphs deliberately:
LRITF 95 → 115 edges (the original 95 unchanged plus 20 self-targeting
fraction edges), DCG 98 → 111.

## 2026-07-03 — Defined-term glossary layer

Mexican financial instruments commonly define their working vocabulary in a
glossary provision within the opening articles — LRITF Article 4
(fraction-style, `I. Término, a …`), DCG-IFPE-2021 Article 1 (colon-style,
`Término: a …`) — though not always, so the glossary is adapter
configuration, not a parser assumption. Terms are extracted as canonical
`DefinedTerm` records (`terms.json`) with the exact span of each definition
entry, including continuation paragraphs such as incisos. The DCG's Article
1 expressly defines its terms "además de los términos utilizados en la
Ley…": that additive relationship is configured (`glossary.additive_to`),
so a DCG usage resolves against the DCG glossary first and falls back to
LRITF Article 4 — `Cliente`, `Operaciones`, and `Infraestructura
Tecnológica` in the DCG resolve to the statute's definitions.

Usages (`term-usages.json`) are deterministic exact matches at word
boundaries, longest match first, case-sensitive because capitalization is
what distinguishes the defined `Control` from the ordinary word `control`.
Glossaries state that terms apply "en singular o plural", so one
singular/plural variant is generated per word with deterministic rules
(`-ón` ↔ `-ones`, vowel ↔ `+s`, consonant ↔ `+es`): `Operación` matches the
defined `Operaciones`, `Comisión Supervisora` matches `Comisiones
Supervisoras`. At a sentence, list-item, or table-cell start the capital is
positional and carries no signal, so a term whose only capital is its
initial letter does not match there — `I. Controles de acceso…` is not the
defined `Control` — while acronyms and multi-word terms match anywhere.
A term never matches inside its own definition entry. Validation covers
term identity, definition spans, exact usage spans, cross-instrument
resolvability, and non-overlapping usages; both files are schema-bound
(`defined-term.schema.json`, `term-usage.schema.json`).

Presentation: generated Obsidian notes carry block anchors on every
fraction paragraph (`^f-xi`) and on each colon-style definition entry
(`^t-<slug>`). A term links to its definition's block —
`[[Corpus/LRITF/articulo-4#^f-ii|Clientes]]` — so hovering shows only the
definition, not the whole glossary article. To keep notes readable, only
the first usage of each term per provision is rendered as a link, and term
links never overlap reference links; all usages remain canonical facts.
The audited LRITF canonical core (provisions, references, temporal result,
review queue) is unchanged by this layer; the fraction anchors also lay the
groundwork for fraction-level reference previews.

## 2026-06-27 — PDF extraction boundary

The LRITF operational source is a text-based PDF. `lex-parse` invokes
`pdftotext -layout -nopgbrk` for extraction, records the extractor version, and
then performs all canonical normalization in Rust. This keeps the source
adapter reproducible without adding an immature PDF parser to the canonical
core.

## 2026-06-27 — Article-level first slice

The first parser emits ordinary articles and the statute's own transitory
provisions. It deliberately excludes appended full reform-decree transitories
from the statute provision list. Those require amendment-event modeling and
must not be conflated with the statute's own transitories.

## 2026-06-27 — No hidden LLM call

Temporal analysis produces a versioned, schema-bound request artifact. Model
execution and response import are explicit boundaries so deterministic runs do
not depend on credentials or silently change canonical data.

## 2026-06-28 — External Obsidian vault boundary

The Obsidian vault is not nested inside canonical corpus storage. The CLI
publishes to an explicit vault root supplied with `--obsidian-vault` or
`LEX_MEX_OBSIDIAN_VAULT`, and the exporter owns only
`Corpus/<instrument-short-name>/` below that root. Human-authored `Notas/`,
`Revisiones/`, attachments, and `.obsidian/` settings remain outside the
exporter's write boundary.

## 2026-06-29 — Explicit temporal execution and deterministic routing

Temporal execution remains opt-in. The default command emits only a request;
`--provider codex` invokes the locally authenticated Codex CLI with the
versioned prompt and a strict output schema. The importer is provider-neutral
and rejects missing, duplicate, or unknown provision identifiers, invalid date
ranges and confidence values, and supporting quotations that are not exact
source substrings. Request and response hashes preserve the execution boundary.

The source adapter explicitly selects reform-decree transitories relevant to
LRITF. This prevents transitories for other statutes bundled into an omnibus
decree from entering LRITF temporal analysis.

## 2026-06-29 — Temporal review policy

Machine conclusions are accepted only at confidence 0.92 or above. A
determination enters legal review when the provision status, effect type,
application rule, or a material boundary remains unknown. Express survival,
adaptation, and conditional rules do not enter legal review merely because they
are transitory. The exporter publishes the queue to Obsidian, but only a human
review workflow may resolve it.

## 2026-06-29 — Audited human review resolution

Review resolution is an explicit canonical state transition. It requires a
reviewer identity; lawyer overrides also require a reason and an explicit
temporal status. The verified determination is labeled `lawyer_verified`, while
the original model proposal, reviewer, resolution, note, and timestamp remain
in the review record. Resolved records stay in the JSON queue for audit but are
excluded from the default CLI listing and pending Obsidian dashboard.
Subsequent model imports reconcile against this history and preserve resolved
human decisions instead of reopening or replacing them.

## 2026-06-29 — Formal-source review context

The LRITF adapter maps each analyzed publication date to an official DOF
publication URL. Review imports attach that formal source alongside the Cámara
de Diputados operational source. Where the one-law slice cannot yet provide an
affected-provision diff, the queue states that limitation explicitly instead
of leaving the reviewer to infer whether the field was omitted accidentally.

## 2026-06-29 — Transitory provision status versus legal effect

Following legal-review guidance from JRH, the temporal model treats a
transitory's own status separately from the effects it creates. An effective
transitory may preserve prior rules for an existing cohort, grant an adaptation
period, mandate regulation, allocate authority, or stage application without
itself being conditional or temporary. Each material effect records its scope,
application rule, trigger, end condition, responsible authorities, and
verification status.

Completion of every proceeding in a protected cohort is modeled as
`cohort_exhaustion` with `open_ended_by_design`; the unknowable global end date
does not itself require legal review. A clear rule dependent on a later
publication or authority action uses `external_verification_required` rather
than being mislabeled as legal ambiguity. Until changed, JRH is the designated
legal reviewer for actual lawyer-verified resolutions.

External facts confirmed during review use `externally_verified` and must carry
an official source URL, event date, and note. JRH verified that SÉPTIMA's
twelve-month clock began with LRITF's entry into force on 10 March 2018 and
that the referenced joint provisions were published on 28 January 2021. The
separate Article 71 coordination agreement remains factually unverified.

## 2026-07-03 — DCG-IFPE-2021 dual official sources

The January 28, 2021 disposiciones for instituciones de fondos de pago
electrónico (`ifpe-dcg-2021`) are jointly issued by the Comisión Nacional
Bancaria y de Valores and Banco de México; the instrument records both
issuing authorities explicitly, independent of which site hosts the file.
The operational CNBV PDF contains the índice, considerandos, seven chapters,
59 articles, and four transitories, but only lists the eight annexes by
title; it does not contain their bodies.

An initial implementation treated the formal DOF publication (código
5610487) as the only available source for annex bodies. JRH pointed out that
the CNBV Normatividad page's "Ver más" panel — visible per row, alongside
`Descargar` and any `Resoluciones Modificatorias` — links each annex as its
own PDF hosted directly on `www.cnbv.gob.mx`. That panel is populated by
`GET /_vti_bin/Cnbv.Webpart.Normatividad/NormatividadAjax.svc/ResolucionesYAnexos?normaId=1036`
(the instrument's row ID), which returns a JSON array of annex descriptions,
URLs, and order; the same response's empty `Resoluciones` array confirms no
amending resolution has been issued for this instrument since 2021-01-28.
These per-annex PDFs are the correct operational annex source: they are
hosted by the same operational publisher as the main PDF, they are the
mechanism CNBV itself uses to publish annexes from that page, and a
word-level fidelity comparison confirms their content is identical to the
DOF note's. The pipeline now fetches, hashes, and extracts each of the eight
annex PDFs as part of the operational acquisition (`annex-source-manifests.json`,
one manifest per annex, ordered) and parses each into its own `annex`
provision using the same paragraph and page-break rules as an article. The
formal DOF publication is still fetched and hashed for promulgation-date
provenance and cross-verification, per the standing rule to attach a formal
source when a decision depends on a later official act, but its text is no
longer parsed for canonical content.

Both official hosts (www.cnbv.gob.mx and www.dof.gob.mx) serve incomplete
TLS certificate chains. The adapter ships the missing public intermediate CA
certificates (GlobalSign RSA OV SSL CA 2018 and Go Daddy Secure Certificate
Authority G2), each of which chains to a standard trusted root; they are
added as additional trust anchors only for adapter fetches.

## 2026-07-03 — DCG parsing and heading model

The CNBV PDF has no page headers or footers, and page breaks fall
mid-sentence. Extraction keeps the form-feed page markers, and a paragraph
merges across a page break unless the previous line ends in `.`, `:`, or
`;`. Article 1's two-column definition layout is reconstructed
deterministically: lines indented past the definition column continue the
current definition; other lines split on their first run of three or more
spaces into term and definition fragments, and term fragments accumulate
until one ends with `:`. The adapter names definition-layout articles
explicitly. Heading context gains optional `section` and `apartado` levels
for Chapter II; heading subject lines remain structural context and are not
inserted into provision text, matching the LRITF chapter model.

Each annex PDF is parsed independently: its first non-blank, non-page-number
line must be its own "ANEXO N" / "Anexo N" heading (cross-checked against
the annex number implied by its position in the adapter's `annex_pdf_urls`),
and everything after it — including the subtitle — accumulates into body
paragraphs using the identical article rules. A bare 1-3 digit line is
treated as a page-number footer and dropped without affecting paragraph
boundaries. This is deliberately the same prose-oriented normalization used
for articles, not a bespoke table-cell reconstruction: Annexo 1's dense
multi-column risk-indicator matrix therefore renders as long, harder-to-scan
paragraphs rather than a gridded table, since a source-position-aware table
reconstruction would be exactly the "immature PDF parser" the project
already avoids for the main text. No content is lost — a word-level
comparison against the extracted PDF text found zero missing or added
words across all eight annexes — only the visual row/column structure of
that one dense table.

## 2026-07-03 — Cross-instrument references and title citations

Reference extraction now resolves targets against every instrument loaded
under `corpus/mx/`. The audited LRITF graph keeps its original whole-group
context policy and stays byte-identical. Multi-instrument extraction uses a
sentence-scoped policy: within the citation sentence, the earliest marker
decides among the instrument's own internal markers (configured per
adapter), configured external instrument names (for example, the LRITF's
full official name), and generic external-law context. Generic markers match
at word boundaries so `de la Ley,` counts as external. Citations of the
DCG's defined term `la Ley` without the full statute name remain unlinked —
resolving them requires the out-of-scope defined-term layer — as do named
laws not yet in the corpus, such as the Código de Comercio.

The DCG's statutory basis — LRITF Articles 48, 54, and 56 — is cited only in
the instrument's official title, not in any provision body. These citations
are canonical edges anchored to the instrument ID itself, with spans
validated against `official_title` and paragraph qualifiers preserved.
`disposición ORDINAL Transitoria` citations become transitory reference
edges; CUARTO resolves to LRITF's OCTAVA transitoria. A canonical reference
remains directed; reverse navigation is provided only by Obsidian backlinks
at presentation time.

## 2026-07-03 — Reviewer-initiated review of accepted determinations

A machine-accepted determination previously could not be corrected: only
items routed to review at import time were resolvable, and hand-editing the
temporal result would bypass the audit trail. `review open` (with
`review --instrument <slug>`) now lets the designated reviewer open a
pending item for any existing determination, preserving the machine
conclusion verbatim as the proposal; resolution then follows the normal
audited lawyer-override path. An existing item — pending or resolved — is
never replaced, so resolved reviews remain immutable. Opening also flags
the determination itself (`review_required` with the reviewer's reason)
and the canonical provision (`review_status: review_required`), so the
corpus and dashboards reflect the pending review instead of continuing to
report machine acceptance.

Reparsing re-applies the persisted temporal result to the fresh provisions
instead of resetting them: a default `pipeline` rerun therefore never
erases applied temporal state, including lawyer-verified decisions. Two
follow-on defects surfaced this and were corrected:

- **Reparse re-application originally accepted a bare substring match** of
  each supporting quotation against the new text. A materially changed
  provision could retain the quoted fragment nearby and incorrectly
  inherit a stale determination. `TemporalDetermination` now carries
  `evidence_sha256`, the hash of the exact evidence text the determination
  was made against; reapplication requires an exact hash match, not a
  quotation surviving somewhere in different text. A determination
  recorded before this field existed (empty hash) is grandfathered in once
  via the substring check it replaces, then has its hash backfilled so
  every later reapply is strict.
- **The evidence used for reapplication must be built the same way the
  temporal-analysis request itself is built** — ordinary transitory text
  plus the reform evidence the adapter marks relevant — not by scanning
  canonical provisions alone. An amendment-event determination's provision
  ID (`…:amendment:DATE:transitory:ordinal`) never appears among canonical
  provisions; it exists only in reform evidence. Reapplication reuses the
  shared evidence builder and runs with the freshly reparsed reform
  evidence (not a stale copy on disk), so amendment-event determinations
  reapply correctly instead of being uniformly flagged stale.

**Preserving review history previously kept only *resolved* items across a
model rerun.** A pending review — whether the model itself routed it there
or a reviewer opened it — could be silently cleared if a rerun happened to
produce a confident, clean result for that evidence, contradicting the
rule that review cannot be resolved by model confidence alone.
`preserve_temporal_review_history` now forward-carries every previous
item, pending or resolved — but only *restores it onto the corpus* when
`evidence_sha256` on the previous determination matches the freshly
routed current determination's own hash (already computed by this same
rerun against current evidence, before being overwritten). A hash
mismatch means the evidence changed since that review was made, so the
old determination is never applied: the freshly routed determination
stands.

The old review item itself is never dropped, though — an earlier version
of this fix did exactly that, silently deleting a reviewer's identity,
rationale, timestamp, and prior machine proposal from `review-queue.json`
on the very next hash mismatch, contrary to `AGENTS.md`'s requirement to
preserve those for every legal-review resolution regardless of what
happens to the underlying evidence afterward. The item is archived
verbatim under a version-qualified ID scoped to the evidence it concerns
(`review:temporal:<provision_id>:evidence:<hash>`, or `:evidence:legacy`
for a record with no hash at all), so it cannot collide with — or be
mistaken for — a fresh review opened under the canonical ID for the
current evidence. The CLI warns the operator by provision ID when this
happens, since it means a review is needed of the new text.

That archival step itself had a second-order bug: it reprocessed every
previous item on every call, including ones it had already archived. An
already-archived item's ID already carries an `:evidence:<hash>` suffix,
so archiving it again appended a second suffix
(`…:evidence:hash1:evidence:hash2`) instead of leaving the historical
record untouched, and the same provision could be reported superseded
more than once from a single rerun. An already-archived item is now
recognized by its ID and carried forward into `review_items` verbatim,
never re-compared against a determination or re-archived: only the one
live item under a provision's canonical ID is ever evaluated for
restoration or archival. Verified across two successive evidence changes
for the same provision — the archived ID and its contents stay identical
after the second rerun, and no second warning fires.

**Reparse reapplication's legacy fallback was itself unsafe.** A
determination predating evidence hashing (empty `evidence_sha256`) was
grandfathered in via the same one-time substring check it was meant to
replace, and its hash silently backfilled. That is exactly the weak check
the hash exists to replace: it is not run at all. A legacy record is now
unconditionally marked stale, forcing a fresh temporal-analysis run
instead of trusting an unverifiable match.

**`schemas/temporal-analysis.schema.json`, which documents the canonical
`TemporalDetermination` shape, was not updated for `evidence_sha256`.**
With `additionalProperties: false`, every determination written after
that field was added violated the schema. The field is now declared
(required, empty string or 64 lowercase hex characters) so committed
determinations validate.

**`review open` did not regenerate Markdown or the Obsidian dashboard**,
unlike `review resolve`; a newly opened review was invisible in published
output until an unrelated command happened to re-export. `review open` now
regenerates both, matching `resolve`.

First use: JRH corrected DCG transitory CUARTO's empty
`responsible_authorities`. The authorization that starts CUARTO's six-month
clock is granted by the CNBV previo acuerdo del Comité Interinstitucional
(LRITF art. 35, first paragraph), whose members represent the SHCP, Banco
de México, and the CNBV (art. 35, second paragraph) — verified against the
committed LRITF corpus text. The determination is now `lawyer_verified`
with the original machine proposal retained in the review record.

## 2026-07-03 — Multi-instrument vault indexes

With two instruments publishing notes with identical stems (for example,
`articulo-1`), generated Obsidian index links now use the full
`Corpus/<instrument>/<note>` path so wikilinks cannot resolve to the wrong
instrument. The pending-review dashboard aggregates review queues across all
committed instruments.

## 2026-07-02 — Canonical reference graph and presentation-only links

Express LRITF article citations are stored in `references.json`, separately
from canonical provision text. Edges use Unicode character offsets and exact
source spans, retain paragraph/fraction/inciso qualifiers, and distinguish
direct citations from range-expansion targets. Internal references must resolve
to a canonical provision before validation passes.

Standard Markdown and Obsidian wikilinks are injected only during export.
Named external-law citations are deliberately left unlinked until their target
instrument is in the corpus. The standalone `link` stage can regenerate the
graph from an already reviewed corpus without reparsing source text or changing
temporal decisions.

## 2026-10-07 — Partial repeal, judicial invalidation and conditional commencement

Decision (operator): model partial repeal explicitly and treat an SCJN
invalidation of an article, fraction or paragraph with the same shape, marked
by a `cause`. Commencement that waits on an unverifiable future act is
`conditional_pending` with an `unverified` condition, filled in by audited
review. Implemented as `partially_repealed` plus `repeals`, and
`conditional_pending` plus `commencement_condition`; models cannot set either.
The parser classifies only the unambiguous parenthesised note; anything else
stays for review. Open: the date that ends force for an SCJN ruling (notice or
DOF publication), the review-queue entry for unverified conditions, and a
versioned model-output schema for the condition record.

## 2026-10-16 — Instrument abrogation by successor clause; REG-LFMN; CFPC

Decision (operator): record `reg-lfmn` as an abrogated historical instrument,
and leave `cfpc` in force (its statutory abrogation deadline is 2027-04-01,
to be rechecked). The retained Cámara text of `reg-lfmn` is unchanged and prints
no abrogation header, so the header-based `source_repeal_notice` could not be
used. `SourceRepealNotice` gained an optional `successor_clause` basis: the
exact abrogating provision of an in-corpus successor instrument, bound by
canonical text hash, with the notice's `source_note` quoted verbatim from it.
Header-basis notices are unchanged byte-for-byte (`lftr`). The notice sets only
instrument status `repealed`; no effect date and no provision-level state is
inferred. Reviewer is the operator, not JRH. The validator change is flagged for
Codex technical review. See
`docs/federal-reg-lfmn-abrogation-decision-2026-10-16.md`.

## 2026-10-16 — Corpus-wide relink: persisted historical edges are protected

Decision (operator-approved plan): `lex-mex link --all` relinks the committed
corpus with unique official titles as markers, but never changes or drops a
persisted `historical_target_unavailable` or approved-outdated edge; an
instrument whose new graph would is held at its previous graph, like an
instrument that cites an article absent from the target corpus. Constitutions
cite themselves as "esta Constitución" or by the adjective "constitucional",
and decree or agreement title openings ("del decreto por el que…") are not
citations of a law named later in the title. `lrsic` is held pending an
operator/JRH decision: the existing dating rule resolves its Article 2 citation
(fraction V, reformed DOF 09-03-2018) to the 2014 LRAF, while the stored graph
and the 2026-08-24 entry keep it on `lraf-1990`. See
`docs/plans/corpus-wide-reverse-linking.md` and the
`corpus-wide-reverse-linking` entry in `docs/ingestion-difficulty-log.md`.

