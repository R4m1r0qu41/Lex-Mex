# JRH decision: LGMIME declared invalid by the SCJN (2026-11-12)

## Decisions

1. **`lgmime` (Ley General de los Medios de Impugnación en Materia Electoral, DOF 02-03-2023)
   is no longer in force.** JRH, 2026-11-12. The SCJN declared the whole decree that issued it
   invalid. It was not abrogated. The schema has no status for invalidity, so the instrument
   is recorded as `repealed` (JRH: "Yes, repealed + note"). Every note states that it was
   declared invalid, not abrogated.
2. **Every provision gets `effective_to` 2023-06-23.** JRH: "Every time the SCJN invalidates a
   law the specifics of temporal issues must be consulted directly in the ruling that
   invalidates it … from the notification to Congress … So the correct date is indeed
   23-06-2023."
3. **Out of scope for this decision:** `lgpp` and `lgipe`, whose 2023 reforms fell under the
   same ruling. JRH asked for a report first.

## Evidence

- **Ruling.** Acción de Inconstitucionalidad 71/2023 and its joined cases 75, 89, 90, 91, 92
  and 93/2023. It was published in the DOF on 24-11-2023 at
  `https://dof.gob.mx/nota_detalle.php?codigo=5709440&fecha=24/11/2023`. The retained bytes
  (private, `.work/lgmime-invalidity/dof-5709440.html`, retrieved 2026-11-12) have SHA-256
  `12ebece74f64ac0a9380c71657edb1ece1b0508bc95386e031b985d1c5836a82`.
  - Resolutivo SEGUNDO: "Se declara la invalidez del DECRETO … y se expide la Ley General de los
    Medios de Impugnación en Materia Electoral, publicado en el Diario Oficial de la Federación
    el dos de marzo de dos mil veintitrés, la cual surtirá sus efectos a partir de la
    notificación de estos puntos resolutivos al Congreso de la Unión".
  - Paragraph 206: "De conformidad con el artículo 45 de la Ley Reglamentaria, esta declaratoria
    de inconstitucionalidad surtirá sus efectos a partir de la notificación de los puntos
    resolutivos de la presente sentencia al Congreso de la Unión."
  - Efectos: "recuperan vigencia las disposiciones legales que regían antes de la entrada en
    vigor del decreto impugnado". The revived law is `lgsmime` (1996), already in the corpus as
    `in_force`.
- **Notification date.** The ruling's extracted text does not print the date itself. The
  retained Cámara source of `lgmime` prints "Decreto … declarado inválido por sentencia de la
  SCJN a Acción de Inconstitucionalidad notificada para efectos legales el 23-06-2023 y
  publicada DOF 24-11-2023". JRH confirmed 23-06-2023.
- **Source header.** The retained `LGMIME.pdf` (source SHA-256
  `94e9c11b4c7f2b192eb33fb17818bea89d6d0bc5a85c85f6b08754fd6b2f6143`) prints
  "Declaratoria de invalidez del Decreto DOF 02-03-2023 que expidió esta Ley, por Sentencia de
  la SCJN publicada en el Diario Oficial de la Federación el 24 de noviembre de 2023". The
  header-basis repeal notice now accepts this printed form beside "Ley Abrogada DOF …"
  (`header_repeal_note_is_dated`, fixture `fixtures/diputados/lgmime-invalidity-header.txt`).

## Boundaries

- No successor is recorded. `lgsmime` is the revived predecessor, not a successor.
- The retained text, provisions and references of `lgmime` are not altered. Only the
  temporal fields change, through the audited review workflow, with reviewer JRH.
