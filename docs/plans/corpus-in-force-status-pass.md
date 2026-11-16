# Corpus-wide in-force and SCJN invalidity status pass (plan)

**Branch:** `status/in-force-scjn`, in the linked worktree `/Users/jr/Dev/lex-mex-status-pass`,
created by JRH from `main` at `47a3ffc59`.

**Authority:**
- [JRH decision, 2026-11-16](../jrh-in-force-rule-and-scjn-invalidity-decision-2026-11-16.md)
  (decisions 1–12);
- the [answer sheet](../jrh-lgpp-lgipe-status-questions-2026-11-15.md);
- `docs/decisions.md` entries 2026-10-07 and 2026-11-16.

**Review:** one Codex review at the end. Cases the rules cannot decide go to JRH in one batch.

## What exists today (reused, not replaced)

- **The dating rule.** `lex-analysis/src/temporal_derive.rs` (`temporal-derive-v3`) has two rules:
  - **Rule A** promotes an untouched article whose text opens with a repeal marker.
  - **Rule B** applies one resolvable commencement clause to the untouched, non-repealed articles.

  Every other law is skipped with a named reason. That is why 27,033 articles are `unknown`.
  The rule never touches a provision whose `review_status` is not `not_analyzed`.
- **Partial repeal.** `Provision.repeals` holds `ProvisionRepeal {scope, ordinals, cause:
  legislative | judicial, dof_date, renumbering}` (2026-10-07). No rule sets `cause: judicial`
  yet, and only two corpus records are `partially_repealed`, both legislative.
- **Reviewer notes.** `ProvisionNote` (`schemas/reviewer-provision-note.schema.json`) is a reviewer
  note bound to a provision, kept in the adapter. It binds the text hash, a source quote, the
  reviewer and the decision source, and never changes text or temporal fields. It is used today
  for CPEUM. JRH's note wording (decision 10) goes here.
- **Instrument-level invalidity.** `source_repeal_notice`, as for LGMIME.

## Design

1. **Close the note inventory first.** Before any status changes, every provision whose text
   contains `inválid` must land in a recognised note shape or on a listed residue list. The
   shapes are anchored on the Cámara note grammar:
   - "… declarado inválido y suprimido …";
   - "… declarado inválido y "recupera su vigencia …" …";
   - "… declarado inválido por sentencia …";
   - "Resolutivos … notificados …";
   - "… DOF dd-mm-yyyy (En la porción normativa …)";
   - and so on.

   The residue is never classified automatically.
2. **Map each shape to a status** (decisions 2, 3, 8 and 12):
   - every note on the article restores the earlier text: `effective`;
   - a sub-unit or a "porción normativa" fell: `partially_repealed`, with a `repeals` entry of
     `cause: judicial`;
   - the whole article fell: `repealed`, with `effective_to` set to the ruling's effect date;
   - a plain invalidation revives nothing unless the note says so (decision 12).
3. **Take each ruling's date from the ruling** (decision 11).
   - Retain and hash the Cámara list
     `https://www.diputados.gob.mx/LeyesBiblio/compila/declara.htm`, and each ruling it links
     that the inventory needs.
   - Record each ruling once: identifier, DOF publication, effect date and the quoted effects
     sentence. Notes refer to rulings by their printed DOF and notification dates.
   - A ruling whose effects do not run from notification goes to JRH.
4. **Record the effect date of a sub-unit removal.** `ProvisionRepeal` has only `dof_date`, the
   publication date. Add an optional `effective_on` (the date the removal took effect), updating
   together the Rust type, `public-instrument-v1.schema.json`, the validator, the fixtures and the
   documentation. Whole articles use `effective_to`.
5. **Notes.** Each changed provision gets a `ProvisionNote` with JRH's wording (decision 10), the
   quote of the printed note, reviewer JRH, and this decision as its source. A provision with
   several notes gets one combined note; the 300-character limit is checked.
6. **In-force rule (decisions 5–6), last.** A new rule C, in the dating rule, applies to an
   `in_force` instrument whose commencement was skipped:
   - **Plainly past:** when every reading of the skipped clause is past, untouched non-repealed
     articles become `effective`, with no `effective_from`. Examples: a day, business-day, month
     or year period counted from an old publication; a number word; several clauses all in the
     past.
   - **Not decidable by rule:** an exception, a condition, a declaratoria or a future date
     stays `unknown` and goes to the JRH batch. Mapping such cases to `future_effective` or
     `conditional_pending` is Claude's engineering reading, labelled as such.
   - **Basis:** `deterministic_rule`, citing this decision; never `lawyer_verified`.

## Two paths, by review state (JRH, 2026-11-16)

- **Not yet analysed** (`unknown`, `not_analyzed`; LGIPE's noted articles are this kind): the
  deterministic rules set the status, with basis `deterministic_rule`.
- **Already accepted** (`effective`, `machine_accepted`; LGPP's noted articles, and others): the
  dating rule never overwrites these. They change only through the audited reviewer path,
  `review open` then `review resolve`, which records them as lawyer-verified. Claude builds the
  complete per-article list from the notes: status, unit removed, ruling and date. JRH confirms
  it in one answer sheet, and Claude then records each resolution citing that confirmation. JRH
  chose this over letting a newer rule replace an accepted status.

### Narrowed later the same day (JRH, 2026-11-16)

JRH: "most should be reviewable without my input … I can take whatever is unclear after that".
This narrows the path above. JRH no longer confirms clear cases one by one. Instead:
- **Clear cases go by rule.** A case is clear when:
  - its note parses whole (head, unit, ordinal and kind);
  - its ruling is on the Cámara list, with effects from notification;
  - the unit can be addressed.

  These are applied by a deterministic rule with basis `deterministic_rule`, never
  `lawyer_verified`. The rule may replace an earlier `deterministic_rule` result. It never
  replaces a model or lawyer result, or a provision with a review item. Measured: all 133
  accepted provisions that carry notes are `deterministic_rule` (132 `effective`, 1 `repealed`).
- **Unclear cases go to JRH.** They appear on JRH's sheet, each with a recommended default and the
  decision it rests on, so JRH answers by exception. Only those go through `review resolve`.
- **The list becomes an audit report** for Codex and JRH, emitted by the Rust classifier with the
  Python inventory as a second oracle.

## Order

1. **LGPP end to end:** 106 provisions, with both note kinds, the 2014 and 2022 rulings and the
   nested art. 23 porción.
2. **LGIPE.** First split the 20 suppressed articles the parser folds into their hosts, with a
   regression fixture and no wording change; then apply the statuses.
3. **Other instruments with SCJN notes,** once the inventory closes.
4. **Rule C** across the corpus.

## Checks

- **Inventory closure:** a recognised shape or a listed residue for every `inválid` provision.
- **Diff invariant.** Outside the LGIPE split, only temporal fields, `repeals` and adapter notes
  change; text, IDs and counts stay identical. Nothing leaves `repealed`. An old-to-new status
  matrix is reported.
- **Gates:**
  - `validate` for all 468 instruments, and the AGENTS.md checks;
  - `verify-fidelity --all` equal to the baseline;
  - the `link --all` closure.

## Kept for JRH (left `unknown` meanwhile)

- transitorios and annexes;
- provisions of instruments recorded `repealed` (`lftr`, `reg-lfmn`);
- commencement the rule cannot decide;
- the one "Recorrido del inciso declarado inválido" (decision 12 needs an explicit revival);
- any ruling whose effects do not run from notification.

## Known conflict

This pass changes `lic`, `lmv` and probably `lritf` provision files. Those are the files the
fintech pilot's G0 sample pins, so `fixtures/fintech-structural-g0-2026-11-14/verify.py` will
report drift if the pilot branch is ever merged after this. The pilot is adopted for local use
at `991b5dc85`; re-pinning is a separate, later task.
