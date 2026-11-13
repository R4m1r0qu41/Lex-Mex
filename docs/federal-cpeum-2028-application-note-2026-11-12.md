# JRH decision: CPEUM arts. 82, 116 and 122 carry a 2028 application note (2026-11-12)

## Decision

JRH, 2026-11-12: arts. 82, 116 and 122 carry the note "Artículo vigente a partir del proceso
electoral 2028", shown small like the SCJN notes ("CPEUM is a temporal issue. It should have a
small note …"). The note is a reviewer note (`provision_notes` in `adapters/diputados/cpeum.json`).
It is never part of the provision text and changes no temporal field.

## Evidence

- Retained Cámara source `CPEUM.pdf`, retrieved 2026-11-13T04:54:35Z, SHA-256
  `cefb525ae8dfc97a7da40597d2ee918fe254d4bfce88301ff3c9959ced9e6e65` (the same hash the source
  monitor saw on 2026-11-12). It prints the "DECRETO por el que se reforman y adicionan los
  artículos 82, 116 y 122 … en materia de nacionalidad única …", published DOF 07-10-2026.
- Transitorio Segundo: "Lo dispuesto en este Decreto será aplicable a partir del proceso
  electoral correspondiente al año 2028." Primero: in force the day after publication.
  Tercero: not applicable to current officeholders.
- The decree's Artículo Único names only arts. 82, 116 and 122.

## Boundaries and caveats

- The decree reforms only some paragraphs of each article (82 first paragraph and fraction I;
  116 fraction I, fifth and sixth paragraphs; 122 Apartado A, fraction III). The note, worded by
  JRH, is shown on the whole article.
- Temporal fields stay as derived. No 2028 date is invented. The text shown is the reformed
  text, which applies only from the 2028 electoral process.
- Re-ingest: the new source also reflows paragraph breaks in 19 other articles (no word
  changes), and reference offsets shift with them. No provision id is added or removed.
