> **Historical record, 2026-10-16.** Produced 2026-10-14 against a 262-instrument corpus and an uncommitted `campaign` tool that was never landed. Superseded: the federal laws and regulations were admitted by the held-source route (467 instruments) and the project is in maintenance mode for them. Kept as provenance only; its counts and next steps are not current. Source patch: git-ignored `archive/campaign-2026-10-14-patch/`.

# Federal catalog reconciliation — 2026-10-14

This is discovery evidence, not a determination of legal validity or a claim of
complete federal regulatory coverage. Prepared inventory and exact-URL comparison
are reproduced by `prompts/build-federal-campaign.py`.

## Baseline and scope

Baseline `1a37ccde2e113e550d08a82bbead5620c9b6d868` contains 262 instruments:
230 non-NOM records, including ten DCGs, and 32 NOMs. The historical July inventory
of 454 Cámara laws/regulations is not a current denominator. The 53 prepared
manifests contain 323 unique rows: 91 present and 232 absent. Historical `NEW`
flags are not current-state evidence.

The fresh four-catalog audit retained response bytes, headers and SHA-256 under
`.work/federal-campaign-inputs-2026-10-14-v3/`; these ignored local artifacts are
not replicated or committed. The tracked normalizer and official URLs allow a
fresh audit. Current response hashes:

| Catalog | SHA-256 | Discovery boundary |
|---|---|---|
| `https://www.diputados.gob.mx/LeyesBiblio/index.htm` | `586896bd83c53a2a1bd520fed15a0bb197123e55a2269c6d42c29489642158cf` | Primary `/pdf/` links; mobile duplicates and archives are not counted as new current instruments |
| `https://www.diputados.gob.mx/LeyesBiblio/regla.htm` | `290d5c3d842c6bda231c29bacb3cb05531e6071b82f2c93f9e47c49212b615ce` | Includes consolidated, historical, reform and errata PDFs |
| `https://www.diputados.gob.mx/LeyesBiblio/norma/reglamento.htm` | `469a6a2a6860ce395986c8a119a6ef5fbc7fb6f8a89ddae861824eac7925cba1` | 59 unmatched PDF links; source identities unprepared |
| `https://www.diputados.gob.mx/LeyesBiblio/norma/estatuto.htm` | `07f83f00016c770fdbcd1fb4c962392e52a0cf93f39cca531aa6e779705d04ad` | 54 unmatched PDF links; source identities unprepared |

The 139 unmatched exact URLs are **not 139 missing instruments**. A bounded
read-only row-level reconciliation covered the one primary-law URL and 25
law-regulation URLs; it did not download their PDFs or resolve legal questions.

## Primary-law and law-regulation reconciliation

URL suffixes below are relative to
`https://www.diputados.gob.mx/LeyesBiblio/`. Catalog repeal notices and dates
require formal DOF evidence before a legal determination.

| URL suffix | Catalog title or family | Disposition |
|---|---|---|
| `pdf/LFC.pdf` | Ley Federal de Cinematografía | Catalog says Ley Abrogada, DOF 22/05/2026; historical-scope/formal-source review required |
| `regley/Reg_LAASSP_2010.pdf` | Reglamento de la Ley de Adquisiciones, Arrendamientos y Servicios del Sector Público | Historical predecessor, not interchangeable with committed 2025 `reg-laassp`; catalog says Abrogado |
| `regley/Reg_LCFE_090215.pdf` | Reglamento de la Ley de la Comisión Federal de Electricidad | Catalog says Abrogado; hold for historical identity/formal-act review |
| `regley/Reg_LEG.pdf` | Reglamento de la Ley de Energía Geotérmica | Catalog says Abrogado; hold for historical identity/formal-act review |
| `regley/Reg_LFGFAGA_fe_16may55.pdf` | Reglamento de la Ley que Crea el Fondo de Garantía y Fomento para la Agricultura, Ganadería y Avicultura | Errata lead for `reg-lfgfaga`, not a new consolidated instrument |
| `regley/Reg_LGS_MCSAEPS_orig_18ene88_ima.pdf` | Salud: Control Sanitario de Actividades, Establecimientos, Productos y Servicios | Original-source lead for `reg-lgs-mcsaeps`, not the current snapshot |
| `regley/Reg_LGS_MCSOTCSH_orig_20feb85.pdf` | Salud: Disposición de Órganos, Tejidos y Cadáveres | Original-source lead for prepared `reg-lgs-mcsotcsh` |
| `regley/Reg_LGS_MCSOTCSH_fe_09jul85.pdf` | Same | Errata-source lead for prepared `reg-lgs-mcsotcsh` |
| `regley/Reg_LGS_MCSOTCSH_ref01_26nov87.pdf` | Same | Reform-source lead for prepared `reg-lgs-mcsotcsh`, not complete consolidation |
| `regley/Reg_LGS_MIS_orig_06ene87.pdf` | Salud: Investigación para la Salud | Historical original; stable-title match and later reform evidence needed |
| `regley/Reg_LGS_MPSS_171214.pdf` | Salud: Protección Social en Salud | Catalog says Abrogado; hold for historical-scope/formal-source review |
| `regley/Reg_LHidro.pdf` | Reglamento de la Ley de Hidrocarburos | Catalog says Abrogado; hold for historical identity/formal-act review |
| `regley/Reg_LICal.pdf` | Reglamento de la Ley de Infraestructura de la Calidad | New current catalog discovery; added as provisional `reg-lic-calidad` in an explicit batch |
| `regley/Reg_LIE.pdf` | Reglamento de la Ley de la Industria Eléctrica | Catalog says Abrogado; hold for historical identity/formal-act review |
| `regley/Reg_LPDB.pdf` | Reglamento de la Ley de Promoción y Desarrollo de los Bioenergéticos | Catalog says Abrogado; hold for historical identity/formal-act review |
| `regley/Reg_LPI_161216.pdf` | Reglamento de la Ley de la Propiedad Industrial | Catalog says Abrogado; not the replacement `reg-lfppi` regulation |
| `regley/Reg_LPM_090215.pdf` | Reglamento de la Ley de Petróleos Mexicanos | Catalog says Abrogado; hold for historical identity/formal-act review |
| `regley/Reg_LRArt5C_orig_01oct45.pdf` | Profesiones, Artículo 5o. Constitucional | Original-source lead for `reg-lrart5c` |
| `regley/Reg_LRArt5C_ref01_04jul51.pdf` | Same | Reform-source lead for `reg-lrart5c` |
| `regley/Reg_LRArt5C_ref02_08may75.pdf` | Same | Reform-source lead for `reg-lrart5c`, not complete consolidation |
| `regley/Reg_LSM_orig_10nov42_ima.pdf` | Reglamento de la Ley del Servicio Militar | Original-source lead for prepared `reg-lsm` |
| `regley/Reg_LSM_ref01_12jul43_ima.pdf` | Same | Reform-source lead for prepared `reg-lsm` |
| `regley/Reg_LSM_ref02_17oct44_ima.pdf` | Same | Reform-source lead for prepared `reg-lsm` |
| `regley/Reg_LSM_ref03_06jun47_ima.pdf` | Same | Reform-source lead for prepared `reg-lsm` |
| `regley/Reg_LTE_040517.pdf` | Reglamento de la Ley de Transición Energética | Catalog says Abrogado; hold for historical identity/formal-act review |
| `regley/Reg_T3_LHidro.pdf` | Actividades del Título Tercero de la Ley de Hidrocarburos | Catalog says Abrogado; hold for historical identity/formal-act review |

## Known identity and scope holds

Prepared `lga` means Archivos; operational `batches/env_E3_water.json` uses `lga`
for Aguas. Both that identity and prepared Aguas (`lgag`) are quarantined until
the conflict and existing reviewer/source hold are explicitly reconciled.
Passing validation or presence on an official catalog cannot clear a legal hold.

The other-federal-regulation and statute catalogs require their own row-level
identity reconciliation. Other regulatory authorities are not exhausted by
Cámara catalogs. NOM/NMX use the separate standards boundary. This campaign must
not be described as complete nationwide regulatory coverage.
