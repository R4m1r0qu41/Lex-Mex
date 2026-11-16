# LGPP and LGIPE after the SCJN invalidity ruling: questions for JRH (2026-11-15)

**What this is.** The one legal question still open in the repository, written as an answer
sheet. Write your answer after each **Answer:** line. Nothing changes in the corpus until you
answer; the agents then apply your answers through the audited review workflow, with you as
reviewer.

**Why it may feel already decided.** On 2026-11-12 you ruled on **LGMIME**, which fell under the
same SCJN judgment ([decision](federal-lgmime-invalidity-decision-2026-11-12.md)). That decision
explicitly left LGPP and LGIPE out: "JRH asked for a report first." The report is the
[article-by-article table](lgpp-lgipe-invalidity-review-2026-11-12.md), delivered 2026-11-12 and
cleaned up on 2026-11-13. No answer to it has been recorded since. If you remember answering,
the answer never reached the repository. Please answer again here.

## Background, in brief

- **The ruling.** SCJN, Acción de Inconstitucionalidad 71/2023 and joined cases. It was notified
  to Congress on 23-06-2023 and published in the DOF on 24-11-2023. It invalidated the 2023
  electoral reform decree (DOF 02-03-2023).
- **LGMIME vs. LGPP/LGIPE.** For LGMIME the whole law came from the invalid decree, so the whole
  law fell. LGPP and LGIPE are older laws that the decree *amended*. Only those 2023 amendments
  fell, so the laws themselves remain. Both are recorded today as `in_force`.
- **What the printed texts say.** The Cámara de Diputados texts print a note at each affected
  unit. There are two kinds:
  - **"recupera su vigencia"**: the 2023 change was invalidated and the text as of 2 March 2023
    is back in force. This includes an invalidated *derogación* or *recorrido*, which brings the
    earlier text back.
  - **"suprimido"**: the unit (a numeral, inciso, fracción, párrafo or whole article) was
    *added* by the 2023 decree, so it simply disappears.
- **How many units are affected:**

  | | LGPP | LGIPE |
  |---|---|---|
  | Articles with at least one note | 53 | 238 |
  | Only "recupera" notes (proposed `effective`) | 42 | 203 |
  | At least one "suprimido" sub-unit (proposed `partially_repealed`) | 11 | 35 |
  | Whole articles added in 2023 and suppressed (proposed `repealed`) | 0 | 20 |
  | Current status of those articles in the corpus | `effective` | `unknown` |

  The partially repealed articles are:
  - **LGPP:** arts. 3, 5, 9, 18, 23, 25, 34, 44, 58, 59 and 68.
  - **LGIPE:** arts. 3, 5, 6, 7, 11, 31, 35, 42, 43, 44, 48, 54, 55, 56, 57, 58, 72, 84, 85,
    98, 99, 102, 117, 141, 216, 224, 254, 264, 269, 273, 288, 331, 343, 449 and 459.

  The 20 suppressed LGIPE articles are 11 Bis, 60 Bis, 80 Bis to 80 Quinquies, 125 Bis to 125
  Septies, 250 Bis, 272 Bis to 272 Sexies, and 284 Bis and Ter. The full table, with the first
  printed note of each article, is in the [review table](lgpp-lgipe-invalidity-review-2026-11-12.md).

## Questions

### Q1. The two laws as instruments

Proposed: **LGPP and LGIPE stay `in_force`.** The ruling invalidated the amending decree, not the
laws.

**Answer:**

### Q2. Articles where every note says "recupera su vigencia"

Proposed: **`effective`**, because the 2 March 2023 text is back in force. That is 42 LGPP and
203 LGIPE articles. For LGPP this confirms the current status. For LGIPE it would replace
`unknown`, which is there only because those articles were never analysed in time (see Q5).

**Answer:**

### Q3. Articles where a sub-unit added in 2023 was "suprimido"

Proposed: **`partially_repealed`**. The article stands, but a numeral, inciso, fracción or
párrafo added in 2023 no longer exists. That is 11 LGPP and 35 LGIPE articles, listed above.
Some of these articles also have "recupera" notes; the proposal is still `partially_repealed`,
because one part is gone.

**Answer:**

### Q4. The 20 whole LGIPE articles that were added in 2023 and suppressed

Proposed: **`repealed`**, with a note that they were declared invalid by the SCJN, not
derogated, as for LGMIME.

An engineering fact you don't need to decide: the parser currently folds these 20 articles into
the article before them (for example, 80 Bis to 80 Quinquies sit inside article 80). The agents
will split them into their own records first, without changing any wording, and then apply your
answer. The host articles (80, 125, 272 and so on) are themselves in force.

**Answer:**

### Q5. LGIPE articles currently `unknown`

Most LGIPE articles (561 of 567) have never been temporally analysed, which is why the table
shows `unknown`. Two ways to proceed:
- **(a)** set the 238 noted articles to the status your answers to Q2–Q4 give, based on the
  ruling's notes alone, and leave the other LGIPE articles `unknown`;
- **(b)** leave all LGIPE articles `unknown` until LGIPE gets a full temporal analysis, and
  record the ruling's effects only as notes for now.

Proposed: **(a)**. The ruling's effect on these articles is printed and unambiguous; other
amendments since 2023 would be handled by the later analysis.

**Answer:**

### Q6. Dates

For LGMIME you confirmed that the ruling took effect at notification, 23-06-2023. You later
deferred per-provision end dates, because the instrument's not-in-force status was what
mattered. Proposed for LGPP/LGIPE, by
the same rule:
- the effect date is **23-06-2023** wherever a date is recorded;
- per-provision end dates for the suppressed sub-units and articles are **deferred**, as for
  LGMIME, so they don't block the status change.

**Answer:**

### Q7. Wording of the note

Proposed, as for LGMIME: each changed status carries a note saying the unit was "declarado
inválido por la SCJN (AI 71/2023, notificada 23-06-2023, DOF 24-11-2023), no derogado", plus
"recupera su vigencia con el texto al 2 de marzo de 2023" where that applies.

**Answer:**

## Anything else

If any article in the table should be treated differently, name it here. One line each is
enough.

**Answer:**
