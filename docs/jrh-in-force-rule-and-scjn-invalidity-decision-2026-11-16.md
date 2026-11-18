# JRH decision: the in-force rule and SCJN invalidity (2026-11-16)

JRH answered the [LGPP/LGIPE answer sheet](jrh-lgpp-lgipe-status-questions-2026-11-15.md) on
2026-11-16 and settled four follow-up questions in session the same day. Recorded by Claude, from
the operator's own words; not a signed review record. No temporal or review state is changed by
this entry. The rule is applied later through the deterministic pipeline and the audited review
workflow, with JRH as reviewer. It extends the
[LGMIME decision](federal-lgmime-invalidity-decision-2026-11-12.md) and supersedes nothing.

## Decisions

1. **LGPP and LGIPE are in force as instruments** (Q1). Specific units may be invalid, as the law's
   own notes and the SCJN rulings state.
2. **"Recupera su vigencia" means in force** (Q2): `effective`.
3. **A unit with an invalidated or suppressed part is `partially_repealed`** (Q3). This applies
   when grayed text is gone, or when a whole fracción or párrafo states it is invalidated or
   suppressed.
4. **The 20 LGIPE articles added in 2023 and suppressed are `repealed`** (Q4). They are first
   split out of their host articles, without changing any wording.
5. **The in-force rule, corpus-wide** (Q5 and the session follow-up). Text shown in an in-force
   instrument is in force, unless one of these applies:
   - it shows that it is derogated: `repealed`;
   - the SCJN invalidated or suppressed it, in whole (`repealed`) or in part (`partially_repealed`);
   - it comes into force at a future time, determined or not: not in force yet
     (`future_effective` or `conditional_pending`). Staged entry into force, as in CNPCF, falls
     here.

   JRH: "Go for corpus-wide now, but don't panic either, this is probably a real improvement …
   This will have to be done either way, so might as well get it over with now and then work over
   proper rulings rather than keep doing passes later on." No provision should be left `unknown`
   merely because it was never analysed.
6. **Only a few temporal states.** In JRH's terms: in force, derogated, repealed, and not in force
   yet. Derogated and repealed have the same legal effect: not in force now or in the future.
   Derogation comes from the legislature; repeal by the SCJN comes from the judiciary. Abrogation
   is the legislative repeal of a whole instrument, which normally is not in the corpus, because
   the corpus holds only instruments in force (vigentes).
7. **One status, with the cause in the note** (session). The system keeps a single `repealed`
   status for both legislative derogation and SCJN invalidation. The note states the cause. No
   schema change.
8. **Every SCJN ruling gets the same treatment** (session). This covers each "declarado inválido"
   note that does not restore the earlier text, whatever the ruling, not only AI 71/2023. JRH
   named LGPP art. 72 b) and f) (AI notified 10-09-2014, DOF 13-08-2015). JRH also named LGPP
   art. 23, fourth paragraph of d), where only the "porción normativa" in parentheses is invalid
   (AI notified 11-10-2022, DOF 26-06-2023).
9. **Dates are recorded now** (Q6 and session). A ruling takes effect at its notification. For
   AI 71/2023 that is 23-06-2023, "the official date for the repeal". For other rulings it is the
   printed "notificada" date. Each suppressed or invalidated unit gets that end date now; the
   dates are not deferred as they were for LGMIME.
10. **Note wording** (Q7): "declarado inválido por la SCJN (AI 71/2023, notificada 23-06-2023, DOF
    24-11-2023)", plus "recupera su vigencia con el texto al 2 de marzo de 2023" where that
    applies. JRH dropped "no derogado": "it is confusing and unnecessary, the text of repealment
    is enough." Rulings whose printed note gives no AI number are cited by their printed
    notification and DOF dates.

## How to read the Cámara texts (JRH)

The LGPP and LGIPE texts use a colour code:
- a red parenthesis marks invalid text inside it;
- light-blue square brackets mark text that regained validity, and the text between them is the
  earlier text now in force;
- red parentheses can sit inside blue brackets;
- in general, grayed text between red parentheses is invalidated.

The retained text extraction keeps the brackets and parentheses but not the colours, and the
printed notes name the invalid "porción normativa" where only part of a unit fell.

## Follow-up answers (JRH, 2026-11-16)

11. **Dates come from each ruling.** JRH: "the date should be taken from the rulling which is
    available in this list https://www.diputados.gob.mx/LeyesBiblio/compila/declara.htm". The
    printed note identifies the ruling. Its effective date is read from the ruling, reached
    through the Cámara list of SCJN declarations, and is not taken from the note's text. This
    refines decision 9, as the LGMIME decision required. The retained copy of each ruling is
    hashed and its effects paragraph quoted, as for LGMIME.
12. **No revival unless stated.** JRH: "in case it is invalidate 'plainly' no text comes back
    in-force unless explicitly stated". A unit declared invalid with no statement that earlier
    text "recupera su vigencia" has no text in force. The unit is `repealed`, or the article is
    `partially_repealed` when only part fell.
13. **Effect rule from the ruling, effect date from the note's notification date.** Reading the
    rulings showed they state "surtirá sus efectos a partir de la notificación de los puntos
    resolutivos al Congreso…" but never print that notification date. Notification happens after
    the ruling is signed: DGI 1/2024 and AI 71/2023 were read in full. Recommended default: take the
    rule from the ruling's effects sentence and the date from the notification date the Cámara
    note prints, recording both sources. JRH: "take the rule from the ruling's effects sentence,
    and the date from the notification date the Cámara note prints. yes this is the correct
    instanse". This refines decision 11 for notification-based effects. Where no note prints a
    notification date, the effect date stays empty and only the DOF publication date is kept.
    Any ruling whose effects do not run from notification still goes to JRH (plan, design 3).
14. **LGCS: the text shown is the text in force** (JRH, 2026-11-17). Asked about the Ley General
    de Comunicación Social units whose note reads "recupera su vigencia con el texto que tenía al
    27 de diciembre de 2022", including the eight glossary fractions of art. 4 that the corpus
    currently records as invalidated, JRH: "On the LGCS they text shown is the one in force".
    Every LGCS unit carrying that revival note is in force with the text printed. This reverses
    the 2026-10 treatment that dropped the art. 4 definitions (`glossary-coverage.json`
    `invalidated_fractions`; the test comment in `crates/lex-parse/src/terms.rs`). Not yet
    applied. Units with a suppression note and no revival statement, such as the folded
    "Artículo 3 Bis.- Suprimido", stay under decision 12; LGCS has 7 such folded placeholders, to
    be split like the 20 LGIPE ones.
15. **LFPA art. 28: 1 October every six years is a non-business day** (JRH, 2026-11-18). Art. 28
    does not list the day of the transmission of the Executive Power since 2024; the Calendario
    Oficial decree published in the DOF on 12-07-2024 (nota 5733285) does, as a mandatory rest day
    for federal servants. Asked directly, JRH confirmed it counts as a non-business day for LFPA
    art. 28 counts in 2024 and every six years after (1 October 2030, and so on). Not yet applied:
    adding it to the business-day calendar changes the commencement census and needs its own
    branch and Codex review. Full text in `docs/decisions.md` (2026-11-18).

## Corrections to earlier records

- The [2026-11-13 table](lgpp-lgipe-invalidity-review-2026-11-12.md) covered only AI 71/2023.
  It proposed `effective` for LGPP art. 72, which carries invalidity notes from the 2014 ruling.
  Under decision 8, art. 72 is `partially_repealed`, and the table undercounts in the same way
  wherever older rulings appear.
- The 2026-11-12 open point on provisions whose own text is a repeal marker ("(Se deroga)") is
  answered by decision 5: they are `repealed`. The other open point, the two instruments recorded
  as `repealed` (`lftr`, `reg-lfmn`), is not addressed here and stays open.

## Scale, for the implementation

Measured 2026-11-16 on `8949c8420`:
- 68,397 provisions: 32,334 `effective`, 30,510 `unknown` (in 467 instruments), 5,551
  `repealed`, 2 `partially_repealed`;
- a loose search ("declarad… inválid…") flags about 480 provisions in 42 instruments. This is
  not an inventory. The notification-date pattern matched only about 20 of those instruments.
  Checked samples show other shapes:
  - `lgcs` prints "Artículo declarado inválido por Resolutivos de la SCJN a Acción de
    Inconstitucionalidad notificados para efectos legales el 08-05-2023";
  - `lgappcea` prints only a DOF date ("… Acción de Inconstitucionalidad DOF 27-05-2016"), with no
    notification date.

  Some hits may be the statute's own wording (`lrfiyii-art105` is likely one, not yet checked).
  The implementation must first list the actual note shapes, anchored on the Cámara note grammar.
  AI 71/2023 accounts for 291 matched provisions, in LGPP and LGIPE.
- Not every note prints a notification date. Under decision 11 the date always comes from the
  ruling, through the Cámara list of declarations, so a missing date in the note does not block.
- **Correction (same day): `lgcs` arts. 1–3 do revive.** Earlier wording here said that no
  earlier text revives for them. That was wrong; the earlier record was written before the whole
  note had been read. The note continues: "Artículo que 'recupera su vigencia con el texto que
  tenía al 27 de diciembre de 2022' por Engrose de sentencia de la SCJN … notificado el
  31-05-2023 y publicado DOF 24-11-2023". That is an explicit statement of revival, so decision 12
  does not apply and the articles are in force with that text. Classification must read every
  note sentence of a unit, not only the first.
