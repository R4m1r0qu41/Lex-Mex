# SCJN invalidation notes mapped to the registry — 2026-10-08

Companion to `docs/scjn-invalidation-census-2026-10-07.md`. Maps each of the 49 census provisions to a record in `corpus/reference/scjn-invalidity-registry/DeclaracionesInvalidez_lex-mex.md`. **Not legally reviewed; no corpus status was changed.** Matching was by the dates quoted in each provision's note, the law title and the article number; the confidence column says how far each match was checked.

## What the registry says about "which date ends force"

The registry's column *Fecha de surtimiento de efectos* is set by the Court per ruling, so no single rule (notification vs DOF) applies. Triggers found among the matched records:

- **Notification** of the resolutive points to Congress (most records; when two chambers were notified on different days, the record names the later day).
- **DOF publication** (record 339; record 337 takes effect the day after publication).
- **Retroactive** (record 186, LGS 467 Bis, retroactive to 2013-12-05) — needs a human read.
- **Empty** (record 184, LOAPF 27; one of the 19 empty-effect rows).

Recommendation for the parser rule: read the effect date from the registry record, send empty and retroactive records to review, and do not choose a global trigger.

## Mapping

Totals: 38 matched to a record (confidence varies), 7 predate the registry, 4 with no record found.

Confidence terms: *confirmed* — record text names this law and article. *probable* / *date+title only* — dates and law title agree, article not checked. *decree-level* — the record invalidates a whole decree, so whether this article is within it needs a read. Records 840, 882 and 657 each cover several provisions.

| Instrument | Provision | Record | Case (resolution date) | Effect trigger | Effect date | Confidence |
|---|---|---|---|---|---|---|
| cff | article:17-f | 889 | A.I. 308/2020 Y ACUMULADA 3/2021 (2023-05-29) | notification | 31/05/2023 | probable |
| cfpc | article:496 | 1107 | DEC. GRAL. INC. 5/2023 (2025-01-07) | notification | 7/02/2025 | confirmed |
| cnpcf | article:610 | 1046 | A.I. 154/2023 (2024-08-13) | notification | 14/08/2024 | confirmed |
| cnpcf | article:638 | 1046 | A.I. 154/2023 (2024-08-13) | notification | 14/08/2024 | confirmed |
| cnpp | article:187 | 840 | A.I. 130/2019 Y SU ACUMULADA 136/2019 (2022-11-24) | notification | 25/11/2022 | decree-level; article not confirmed |
| cnpp | article:192 | 840 | A.I. 130/2019 Y SU ACUMULADA 136/2019 (2022-11-24) | notification | 25/11/2022 | decree-level; article not confirmed |
| cnpp | article:242 | 339 | A.I 10/2014 A.I. 10/2014 y su acumulada 11/2014 (2018-03-22) | DOF publication | 25 de junio de 2018 | confirmed |
| cnpp | article:249 | 339 | A.I 10/2014 A.I. 10/2014 y su acumulada 11/2014 (2018-03-22) | DOF publication | 25 de junio de 2018 | confirmed |
| lcf | article:4-a | — | — | — | — | not in registry (ruling predates 2009-01-20) |
| lcf | article:10-c | — | — | — | — | not in registry (ruling predates 2009-01-20) |
| lcf | article:10-d | — | — | — | — | not in registry (ruling predates 2009-01-20) |
| lcm | article:338 | — | — | — | — | no registry record found |
| lfar | article:24 | 745 | A.I. 139/2019 (2022-04-05) | notification | 6/04/2022 | date+title only |
| lfdo | article:2 | 840 | A.I. 130/2019 Y SU ACUMULADA 136/2019 (2022-11-24) | notification | 25/11/2022 | confirmed |
| lfed | article:1 | 657 | A.I. 100/2019 (2021-06-21) | notification | 22/06/2021 | date+title; article not confirmed |
| lfed | article:2 | 657 | A.I. 100/2019 (2021-06-21) | notification | 22/06/2021 | date+title; article not confirmed |
| lfed | article:5 | 657 | A.I. 100/2019 (2021-06-21) | notification | 22/06/2021 | date+title; article not confirmed |
| lfed | article:7 | 657 | A.I. 100/2019 (2021-06-21) | notification | 22/06/2021 | date+title; article not confirmed |
| lfed | article:9 | 657 | A.I. 100/2019 (2021-06-21) | notification | 22/06/2021 | date+title; article not confirmed |
| lfed | article:11 | 657 | A.I. 100/2019 (2021-06-21) | notification | 22/06/2021 | date+title; article not confirmed |
| lfed | article:15 | 657 | A.I. 100/2019 (2021-06-21) | notification | 22/06/2021 | date+title; article not confirmed |
| lfed | article:126 | 657 | A.I. 100/2019 (2021-06-21) | notification | 22/06/2021 | date+title; article not confirmed |
| lfed | article:173 | 657 | A.I. 100/2019 (2021-06-21) | notification | 22/06/2021 | date+title; article not confirmed |
| lfed | article:190 | 657 | A.I. 100/2019 (2021-06-21) | notification | 22/06/2021 | date+title; article not confirmed |
| lfed | article:214 | 657 | A.I. 100/2019 (2021-06-21) | notification | 22/06/2021 | date+title; article not confirmed |
| lfed | article:228 | 657 | A.I. 100/2019 (2021-06-21) | notification | 22/06/2021 | date+title; article not confirmed |
| lfprh | article:19-ter | — | — | — | — | no registry record found |
| lfprh | article:61 | 745 | A.I. 139/2019 (2022-04-05) | notification | 6/04/2022 | date+title only |
| lfrm | article:32 | 733 | A.I. 151/2021 (2022-02-03) | notification | 8/02/2022 | probable; not read |
| lfrm | article:59 | 733 | A.I. 151/2021 (2022-02-03) | notification | 8/02/2022 | probable; not read |
| lfrm | article:61 | 733 | A.I. 151/2021 (2022-02-03) | notification | 8/02/2022 | probable; not read |
| lgra | article:49 | 882 | A.I. 29/2023 Y SUS ACUMULADAS 30/2023, 31/2023, 37/2023, 38/2023, 43/2023 Y 47/2023 (2023-05-08) | notification | 08/05/2023 | decree-level; article not confirmed |
| lgra | article:73 | 882 | A.I. 29/2023 Y SUS ACUMULADAS 30/2023, 31/2023, 37/2023, 38/2023, 43/2023 Y 47/2023 (2023-05-08) | notification | 08/05/2023 | decree-level; article not confirmed |
| lgs | article:10-bis | 685 | A.I. 54/2018 (2021-09-21) | notification | 22/09/2021 | confirmed |
| lgs | article:464-ter | — | — | — | — | no registry record found |
| lgs | article:467-bis | 186 | A.I 2/2014 (2014-12-01) | retroactive (read) | 02-12-2014 | confirmed; retroactive |
| lieps | article:2-a | — | — | — | — | not in registry (ruling predates 2009-01-20) |
| lieps | article:27 | — | — | — | — | not in registry (ruling predates 2009-01-20) |
| lnep | article:139 | 282 | A.I 61/2016 (2017-04-04) | notification | 05/04/2017 | confirmed |
| lnuf | article:6 | 701 | A.I. 64/2019 (2021-10-26) | notification | 3/11/2021 | probable |
| loapf | article:27 | 184 | A.I 1/2013 (2014-10-23) | empty | — | confirmed; effect date empty |
| loapf | article:29 | — | — | — | — | no registry record found |
| lopdc | article:5 | — | — | — | — | not in registry (ruling predates 2009-01-20) |
| lopdc | article:12 | — | — | — | — | not in registry (ruling predates 2009-01-20) |
| lrart6-mdr | article:3 | 337 | A.I 122/2015 A.I. 122/2015 y sus acumuladas 124/2015 y 125/2015 (2018-02-01) | DOF publication | 24 de abril de 2018 | probable; not read |
| lrart6-mdr | article:10 | 337 | A.I 122/2015 A.I. 122/2015 y sus acumuladas 124/2015 y 125/2015 (2018-02-01) | DOF publication | 24 de abril de 2018 | probable; not read |
| lrart6-mdr | article:19 | 337 | A.I 122/2015 A.I. 122/2015 y sus acumuladas 124/2015 y 125/2015 (2018-02-01) | DOF publication | 24 de abril de 2018 | probable; not read |
| lrart6-mdr | article:25 | 337 | A.I 122/2015 A.I. 122/2015 y sus acumuladas 124/2015 y 125/2015 (2018-02-01) | DOF publication | 24 de abril de 2018 | probable; not read |
| lsn | article:5 | 840 | A.I. 130/2019 Y SU ACUMULADA 136/2019 (2022-11-24) | notification | 25/11/2022 | confirmed |

## Open items

1. The 7 pre-registry provisions (LCF 4-A, 10-C, 10-D; LIEPS 2-A, 27; LOPDC 5, 12) come from 2008 rulings and need another source (DOF or SJF).
2. LGS 464-ter, LOAPF 29, LFPRH 19-ter and LCM 338 should be in the registry by date but no record was found; LCM 338 cites a late-2025 general declaration. Search by hand.
3. Earlier candidate matches that proved false and were dropped: record 816 (electoral finance) for LFPRH 19-ter; record 855 (Tamaulipas) for LRART6-MDR.
4. Read the "date+title only", "probable" and "decree-level" rows against their records before any status change.
5. Operator still decides whether each hit is `repealed` or `partially_repealed` with cause judicial.
