# Pending operator decisions — 2026-10-20

These items need the operator's decision before work can go on. Each has the question, the evidence, the options and a recommendation. To answer, write after the **Answer:** line or reply in the session. Operator answered on 2026-10-20; each answer quotes the operator's words. A recommendation is not a ruling.

State when written:
- The parser batch is on local branch `parser/pending-batch` at `cf8eed626`. It is not merged and not pushed.
- Codex's batch review is drafted but not yet committed, in the DB worktree (`docs/parser-batch-review-2026-10-20.md`).
- The M4 report is not yet committed either.

## A. Legal and reading questions

### A1. lgcs: do the 7 remaining glossary terms stand?

- **Evidence:** The SCJN declared 8 of lgcs's glossary fractions invalid: I, II, IV, VI, VII, VIII, X and XII. They are recorded as `invalidated_fractions`. 7 terms remain, and the glossary's own introduction is also bracketed as invalid.
- **Question:** With the introduction invalid, do the 7 surviving entries still operate as definitions?
- **Options:** (a) they stand, and nothing changes; (b) they do not stand, and all lgcs terms and usages are removed; (c) they stand, but the invalid introduction is recorded as a note on the instrument.
- **Recommendation:** none. This is a legal question.
- **Answer:** "A1. Yes the 7 remaing still stand." The 7 terms stand. This does not rule on the restored pre-2022 definitions, which the source does not print and which stay absent.

### A2. "la presente Ley" inside the law's own title

- **Evidence:** lmigra and lgscmm define `Ley` as "la presente Ley". It names no title, so the title filter now drops `Ley` inside the instrument's own full title: 9 usages in lmigra, 4 in lgscmm. Codex says to keep the current behavior until you decide.
- **Options:** (a) keep them dropped: a word inside a title is the title's name, not a use of the shorthand; (b) bind "la presente Ley" to the instrument's own title, so those 13 come back as usages.
- **Recommendation:** (a).
- **Answer:** "A2. La presente ley keep dorped." (a): the 13 usages stay dropped.

### A3. LIEPS Article 3, I b): `azúcar` in an ingredient list

- **Text:** "…pudiéndose adicionar agua, bióxido de carbono o agua carbonatada, jugo de frutas, extracto de frutas, aceites esenciales, ácido cítrico, **azúcar**, ácido benzoico…" (offset 690). It currently links to the defined term `Azúcares` (fraction XX).
- **Options:** (a) exclude it as a 12th reviewed non-usage, like the other 11; (b) keep it as a usage of the defined term.
- **Recommendation:** (a). It is the same kind of lower-case ingredient mention as the reviewed 11. But the call is yours, because XX defines the sugars the tax applies to.
- **Answer:** "A3. exclude" (a): a 12th reviewed exclusion.

### A4. lan XL: does inciso b. define "Permisos de Descarga"?

- **Text:** "XL. “Permisos”: Para los fines de la presente Ley, existen dos acepciones de permisos: a. “Permisos”: … b. "Permisos de Descarga": Título que otorga el Ejecutivo Federal … para la descarga de aguas residuales…"
- **Now:** XL gives one term, `Permisos`. lan's 4 usages of `Descarga` (XXII) all sit inside "Permiso(s) de Descarga": Article 3 XL b., 92, 93 and 93 Bis. The question was never put to you.
- **Options:**
  - (a) b. defines its own term "Permisos de Descarga", the same shape as LIEPS VIII (a parent term plus incisos). The 4 `Descarga` usages then stop. The singular "Permiso de Descarga" in Articles 92–93 Bis would link to the new term only if the plural/singular form is accepted.
  - (b) Leave it as is, with `Descarga` linking inside the phrase.
  - (c) Keep XL as one term, but drop `Descarga` inside the phrase.
- **Recommendation:** (a). The source quotes the name as a defined name.
- **Answer:** "A4, lan XL b. \"Permisos de Descarga\" in full should link to it same rulling as LIEPS VIII" (a): b. defines "Permisos de Descarga", and the full phrase links to it.

### A5. reg-lgeepa-maaa: which second law does it import from?

- **Evidence:** maaa adopts LGEEPA Article 3 and also the Ley Federal sobre Metrología y Normalización. That law is not in the corpus. My understanding, still to be verified, is that it was abrogated by the Ley de Infraestructura de la Calidad, which is committed as `lic-calidad`.
- **Options:** (a) import LGEEPA only, and record the LFMN reference as unresolved; (b) also import `lic-calidad`'s definitions as the successor law; (c) leave maaa unconfigured.
- **Recommendation:** (a). Option (b) treats the successor as the referent, which is a legal reading.
- **Answer:** "A5. Link to the new Ley de Infraestructura de la calidad, in terms of Transitorio NOVENO" (b). The operator quoted LIC Transitorio NOVENO: references to the LFMN on normalización, evaluación de la conformidad and metrología "se entenderán a los referidos en la Ley de Infraestructura de la Calidad". maaa imports LGEEPA Article 3 and `lic-calidad` Article 4.

### A6. reg-lgeepa-mretc: "otros ordenamientos"

- **Text:** "Sin perjuicio de las definiciones que establezcan otros ordenamientos jurídicos, para los efectos del presente Reglamento, se considerarán las definiciones contenidas en la Ley General del Equilibrio Ecológico y la Protección al Ambiente, así como las siguientes:"
- **Options:** (a) import LGEEPA Article 3 only, treating "otros ordenamientos" as an unspecified reservation that imports nothing; (b) keep mretc unconfigured.
- **Recommendation:** (a).
- **Answer:** "A6. link to LGEEPA only" (a).

### A7. reg-lgeo and the multi-law energy glossaries: precedence between originals

- **Evidence:** These regulations import definitions from more than one law. When two originals define the same word, which one governs is a legal choice.
- **Next:** I will prepare a per-instrument table of the conflicting terms before asking for a ruling. Nothing to decide yet; listed so it is not lost.

## B. Authorizations

### B1. ltfccg: re-fetch its source?

- **Evidence:** ltfccg's glossary is held by a chapter-heading leak at fraction IX, which the batch's parser now reads. A reparse needs the source matching its committed `source_sha256`, and none is retained.
- **Options:** (a) fetch the current official text. If it differs, this becomes re-ingestion of a new edition, with new provenance; (b) leave it held.
- **Recommendation:** (a), as a separate reviewed step.
- **Answer:** "B1 yes".

### B2. fi-dcg-2014: re-ingest CNBV's new edition?

- **Evidence:** CNBV has published a new edition. The live source parses with one more article and detects its glossary. The committed parse is held at a fraction IV sequence break.
- **Options:** (a) re-ingest the new edition; (b) leave it held.
- **Recommendation:** (a).
- **Answer:** "b2, yes".

### B3. Codex's two technical findings: fix before the batch merges?

Codex found two defects. Neither changes the current corpus.
- **P1:** a batch admission relinks only its own instruments, but usages now depend on every title. The repair is to run the corpus-wide closure inside batch completion.
- **P2:** a title printed with a very long run of spaces escapes the filter's lookback window.

Codex's sequence: merge the DB branch first, then fix both on the parser branch with regression fixtures, then merge the batch.
- **Recommendation:** follow it. Say if you would rather merge first and fix after.
- **Answer:** "b3, yes". Fixed in `a749c81f0` on `parser/pending-batch`.

### B4. Push

Everything is local: the DB merge, the batch and the fixes. Pushing main needs your go-ahead once Codex has reviewed the integrated result.
- **Answer:** "b4 yes": push main once Codex has reviewed the integrated result. main was already 41 commits ahead of origin before this batch.

## D. New questions from carrying out the answers (2026-10-20)

### D1. Names of authorities: what counts as a usage inside them?

- **Why this came up:** A5 and A6 are configured but **not committed**, because the imports add only false matches.
  - **maaa (+19):**
    - 12 are LGEEPA's `Ambiente` or `Protección` inside "Secretaría de Medio Ambiente y Recursos Naturales", "Procuraduría Federal de Protección al Ambiente", the ASEA name, or LGEEPA's title as printed ("las Leyes General del Equilibrio Ecológico y la Protección al Ambiente").
    - 7 are the LIC's `Verificación`. The LIC defines it as authorities checking compliance with NOMs; maaa uses it for checking an environmental-audit action plan ("Verificación de cumplimiento del Plan de Acción").
  - **mretc (+16):**
    - 13 are `Ambiente` or `Protección` inside the same names;
    - 2 are `Contaminante` in "Registro de Emisiones y Transferencia de Contaminantes";
    - 1 is `Residuo` inside an old regulation's title.
- **Not only these two.** The same kind of match already exists elsewhere: 61 usages inside those three agency names (lgeepa 49, mostly "Procuraduría Federal de Protección al Ambiente"; ldrs 7; lbogm 5).
- **Options:**
  - (a) **Curated authority names as non-usage spans**, like the title filter: a reviewed list of agency names (SEMARNAT, PROFEPA, ASEA, …), applied corpus-wide. A usage is kept only when the term's definition names that authority, so `Secretaría` defined as SEMARNAT stays. This also removes the existing 61.
  - (b) **Pinned exclusions** for maaa and mretc only (about 32 entries), leaving the existing 61 as they are.
  - (c) Admit the imports with the false matches.
- **Also decide:**
  - **`Verificación` in maaa:** is it the LIC's defined Verificación? My reading is that it is not.
  - **The 2 `Contaminante` in the RETC name:** are they genuine? My reading is that they are, since the registry is of LGEEPA contaminants.
- **Recommendation:** (a). Exclude maaa's `Verificación`; keep the RETC `Contaminante`.
- **Answer:** "the false Ambiente and Proteccion should't be that hard to fix … when in doubt use the full exact name". Applied as institution full names taken from the glossaries; the remaining cases were decided from context (`docs/decisions.md`).

### D2. lan XVI: confirm the same reading as XL

- **Text:** "XVI. "Cuenca Hidrológica": … Para los fines de esta Ley, se considera como: a. “Región hidrológica”: … b. "Región Hidrológico - Administrativa": …"
- **What happens:** the A4 rule (parent plus quoted incisos, as LIEPS VIII) also reads these two names as terms. No usage changes, because lan never prints either name as it is defined.
- **Options:** (a) confirm; (b) limit the rule to XL.
- **Recommendation:** (a).
- **Answer:** "LAN … just with it" (a).

### D3. ltfccg IX: read "Valuador profesional" as a term?

- **Text:** "IX. Valuador profesional es la persona que cuenta con cédula profesional de postgrado en valuación…"
- **Now:** ltfccg's glossary uses "Term. Definition"; IX alone writes "Term es …". The whole glossary is held for it, so ltfccg has 0 terms. The reparse removed the heading leaks, but they were not the cause of this hold.
- **Options:**
  - (a) A reviewed profile reads IX's term as "Valuador profesional", and the rest by the shared rule. This releases ltfccg's terms (10, as read).
  - (b) Leave it held.
- **Recommendation:** (a).
- **Answer:** "valudor profesionale es is s definition. read IX a one-off profile" (a).

## C. Technical backlog (no decision needed; listed for completeness)

- **Label and boundary holds:** `log` (II, after its introduction), `servinv-dcg-2013` (IV after II), `reg-lftr-mcsre` (IX after VII) and `ladua` (IV after I; reparse first).
  - These need retained-source checks.
  - A source typo may turn into a reading question for you.
- **12 unread fractions in 11 instruments:**
  - lfped III Bis
  - lgdnna XVII–XVIII
  - lgmasc III
  - lgpsacdii VIII
  - lissste XII
  - lss 159 VII
  - reg-laassp IX
  - reg-ldcmpme II
  - reg-lgn VI
  - reg-lgvs XV Bis
  - socap-sofipo-dcg-2006 XXXVIII

  Each one needs a profile, or a ruling if the source is ambiguous.
- **meia and mpcca imports.** The title filter removes matches inside LGEEPA's own title. But "Secretaría de Medio Ambiente y Recursos Naturales" and the ASEA agency name are not instrument titles, so the `Ambiente` and `Protección` false matches may remain. They need a re-probe before configuration.
- **Reparse blockers:** cff (a duplicate evidence id) and lfd (validation), both from unchanged source.


**Standing instruction (2026-10-20):** "when you encounter an issue, look for the surrounding context to see if it can be fixed, without my input." Only legal questions come here from now on.
