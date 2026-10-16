# Federal corpus coverage audit — 2026-10-16

Read-only audit of `corpus/mx/` against the live Cámara de Diputados indexes.
Repository `main` at `25732f3ef58ab19d1965d5015d206e9d3c4fdd40`. No corpus,
adapter, schema or code file was changed. This audit establishes **identity
coverage** only: whether each current Cámara row has a committed instrument
whose retained source points at it. It does not test source currency, text
completeness, links, or legal effect.

## Method

1. Fetched `https://www.diputados.gob.mx/LeyesBiblio/index.htm` and
   `regla.htm` over HTTPS at 2026-10-16T05:15Z (HTTP 200 for both).
   SHA-256 of the retrieved bytes: index
   `a0989ffc0be3b5abce4b5d7d9fb303e4989e033bc0825a339b46197b3553149e`, regla
   `290d5c3d842c6bda231c29bacb3cb05531e6071b82f2c93f9e47c49212b615ce`. The
   pages are not committed.
2. Parsed every table row carrying a consolidated-text link. Rows marked
   `Abrogad*` were set aside and reviewed separately.
3. Matched each row to a corpus slug through the retained
   `source-manifest.json` `official_url` / `reference_url` basename, falling
   back to the documented slug convention (`ref/` slug for laws;
   `reg-` + `regley/` stem for regulations). A row with several PDF links
   (original scan plus consolidated text) matches on any of them.
4. Listed corpus instruments matched by no current row.

## Result

| Measure | Count |
|---|---:|
| Current non-abrogated rows (laws) | 315 |
| Current non-abrogated rows (regulations) | 138 |
| Total current rows | 453 |
| Rows matched to a committed instrument | **452** |
| Rows unmatched | **1** — `egdf` (Estatuto de Gobierno del D.F.), excluded by operator decision |
| Corpus instruments with `instrument.json` | 467 |
| Corpus instruments matched by no current row | 15 |

Every current Cámara law and regulation except EGDF is in the corpus. The
July plan counted 454 (316 laws + 138 regulations); the live law count is now
315 because Cámara marked *Ley Federal de Cinematografía* (`lfc`) abrogated on
2026-05-22. `lfc` is not in the corpus, so nothing needs removal.

## Corpus instruments with no current row (15)

| Slug(s) | Why it is off the list | Action |
|---|---|---|
| `cub-dcg-2005`, `cucb-dcg-2004`, `cue-dcg-2003`, `fi-dcg-2014`, `ifpe-dcg-2021`, `itf-dcg-2018`, `oaac-dcg-2009`, `scap-dcg-2012`, `servinv-dcg-2013`, `socap-sofipo-dcg-2006` (ten CNBV DCGs) | CNBV source; DCGs are outside the Cámara universe by design | none |
| `reg-lgs-insumos` | Secretaría de Salud/OAG source; not a Cámara row | none |
| `reg-csps` | Cámara `.doc` source from the Maximasa federal-gap ingestion; no current row | none, noted |
| `lftr` | Deliberate historical admission of the abrogated telecommunications law | none |
| `cfpc` | Cámara row reads "Abrogado en un plazo que no exceda del 01/04/2027" (SCJN judgment, notified 2025-06-17). In force until the sunset | **confirmed**: operator 2026-10-16, on track for 2027; recheck before 2027-04-01 |
| `reg-lfmn` | Cámara now lists this regulation as *Reglamento Abrogado* (2026-09-04). Its parent LFMN is not in the corpus (replaced by the Ley de Infraestructura de la Calidad, `lic-calidad`, in corpus) | **resolved**: recorded `repealed` 2026-10-16 (see decision record) |

## Decisions and open items

1. **`reg-lfmn` — resolved 2026-10-16.** Operator: abrogate. Recorded as an
   instrument-level `repealed` historical admission, bound to Transitory Segundo
   of `reg-lic-calidad` and its DOF note
   ([decision record](federal-reg-lfmn-abrogation-decision-2026-10-16.md)). No
   effect date or provision state inferred. Corpus now holds two `repealed`
   instruments: `lftr`, `reg-lfmn`.
2. **`cfpc` — confirmed 2026-10-16.** Operator: on track for 2027. No change;
   recheck before 2027-04-01.
3. **EGDF** stays excluded (operator decision 2026-10-15).
4. **DCG track.** `consar-cuf-dcg-2018`, `cusf-dcg-2014` and
   `pld-lic115-dcg-2009` remain deferred and are outside this universe.

## Not covered by this audit

- **Source currency.** Whether each retained PDF is still the latest
  consolidated text, or whether a reform has been published since retrieval,
  was not tested. A later pass could compare each retained
  `source_sha256` / `etag` with a fresh HEAD request.
- **Completeness and legal effect.** Counts for the 2026-10-15 admissions are
  unfrozen observations; all provisions remain `not_analyzed`.
- **Cross-instrument links.** The corpus-wide reverse-linking pass has not
  run and remains the largest quality gap.

Evidence: `docs/federal-coverage-audit-2026-10-16-evidence.json`.
