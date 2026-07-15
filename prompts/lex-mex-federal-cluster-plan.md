# Lex-Mex — Federal Corpus Ingestion Cluster Plan

**Purpose.** Batch-feed the Lex-Mex add-and-link workflow with tight, reference-linked clusters of Mexican federal instruments grouped by subject matter. Each batch is sized so that most internal cross-references (`link` stage) resolve *within* the batch, minimizing dangling edges. Cross-batch edges that will remain open are listed explicitly as **bridge edges** so you can sequence ingestion to close them.

**Sources.**
- Cámara de Diputados — *Leyes Federales vigentes* (`ref/` slugs) and *Reglamentos de Leyes Federales vigentes* (`regley/` files). Consolidated texts, canonical for the statute + its reglamentos.
- Comisión Nacional Bancaria y de Valores (CNBV) — *Normatividad*. Use for (a) cleaner copies of the financial laws and (b) the **Disposiciones de carácter general (DCG)** that exist only there. Comisión Nacional de Seguros y Fianzas (CNSF) and Comisión Nacional del Sistema de Ahorro para el Retiro (CONSAR) publish their own DCGs for the insurance and retirement-savings sub-clusters.

**Convention.** Slugs follow your existing pattern (`lritf`, `ifpe-dcg-2021`, `itf-dcg-2018`): `ley-abbrev` for statutes, `reg-<ley>` for reglamentos, `<sector>-dcg-<year>` for general dispositions. `[IN CORPUS]` marks instruments already committed.

> **Verify-before-ingest flags.** Several 2024–2026 reforms renamed or replaced instruments (energy, telecommunications, competition, data protection/INAI, judiciary). These are marked ⚠ and should be confirmed against the current Diario Oficial de la Federación (DOF) title before you fetch — do not ingest on the strength of a remembered title.

---

## How to read the batches

Each domain is broken into batches of ~3–7 instruments. Within a batch, one instrument is the **hub** (highest in-degree — the others cite it). Ingest the hub first, then its reglamentos, then peers, then DCGs. Recommended global order runs financial → commercial/civil → tax → the rest, because the financial and commercial hubs (LGTOC, Código de Comercio, LGSM) are the most-cited targets across the whole corpus.

---

## Domain 1 — Financial system, banking & fintech (extends current corpus)

Richest cross-citation density in the federal corpus; also where Lex-Mex already lives. Ingest first.

### DCG source register (sourced, CNBV / CNSF / CONSAR)

Dates below are the **original DOF publication** of each compiled DCG; each carries many amending resolutions, so ingest the CNBV/regulator compiled text and record the amendment provenance exactly as `itf-dcg-2018` already does (margin marks resolved through the REFERENCIAS legend). Slugs use the DOF year of first publication.

| DCG (official title) | Regulator | Sector | DOF (original) | Slug |
|---|---|---|---|---|
| Disposiciones de carácter general aplicables a las instituciones de crédito (Circular Única de Bancos) | CNBV | Banks | 02/12/2005 | `cub-dcg-2005` |
| Disposiciones de carácter general aplicables a las casas de bolsa (Circular Única de Casas de Bolsa) | CNBV | Broker-dealers | 06/09/2004 | `cucb-dcg-2004` |
| Disposiciones de carácter general aplicables a las emisoras de valores y a otros participantes del mercado de valores (Circular Única de Emisoras) | CNBV | Issuers | 19/03/2003 | `cue-dcg-2003` |
| Disposiciones de carácter general aplicables a los fondos de inversión y a las personas que les prestan servicios | CNBV | Investment funds | 24/11/2014 | `fi-dcg-2014` |
| Disposiciones de carácter general aplicables a las entidades de ahorro y crédito popular, organismos de integración, sociedades financieras comunitarias y organismos de integración financiera rural | CNBV | SOFIPO / popular savings | 18/12/2006 | `socap-sofipo-dcg-2006` |
| Disposiciones de carácter general aplicables a las actividades de las sociedades cooperativas de ahorro y préstamo | CNBV | SOCAP | 04/06/2012 | `scap-dcg-2012` |
| Disposiciones de carácter general aplicables a los almacenes generales de depósito, casas de cambio, uniones de crédito y sociedades financieras de objeto múltiple reguladas | CNBV | Auxiliary credit / SOFOM ER | 19/01/2009 | `oaac-dcg-2009` |
| Disposiciones de carácter general a que se refiere el artículo 115 de la Ley de Instituciones de Crédito (PLD/FT) | CNBV/SHCP | AML/CFT – banks | 20/04/2009 | `pld-lic115-dcg-2009` |
| Disposiciones de carácter general en materia financiera de los Sistemas de Ahorro para el Retiro (Circular Única Financiera) | CONSAR | Retirement savings | 26/01/2018 | `consar-cuf-dcg-2018` |
| Circular Única de Seguros y Fianzas | CNSF | Insurance & bonding | 19/12/2014 | `cusf-dcg-2014` |
| Disposiciones de carácter general aplicables a las ITF | CNBV | Fintech (ITF) | 10/09/2018 | `itf-dcg-2018` **[IN CORPUS]** |
| Disposiciones aplicables a las IFPE | CNBV/Banxico | Fintech (e-money) | 28/01/2021 | `ifpe-dcg-2021` **[IN CORPUS]** |

*Optional bridge DCG:* Disposiciones de carácter general aplicables a las casas de bolsa e instituciones de crédito en materia de servicios de inversión (DOF 24/04/2013, `servinv-dcg-2013`) — straddles F2 and F3; ingest once and link from both.

### Batch F1 — Fintech & payments *(closes the current corpus)*
| Instrument | Type | Slug | Role |
|---|---|---|---|
| Ley para Regular las Instituciones de Tecnología Financiera | Ley | `lritf` | hub **[IN CORPUS]** |
| DCG aplicables a las ITF (CNBV, 10/09/2018) | DCG | `itf-dcg-2018` | **[IN CORPUS]** |
| DCG aplicables a las IFPE (CNBV/Banxico, 28/01/2021) | DCG | `ifpe-dcg-2021` | **[IN CORPUS]** |
| Ley General de Títulos y Operaciones de Crédito | Ley | `lgtoc` | cited for e-money / títulos |
| Ley del Banco de México | Ley | `lbm` | Banxico authority over IFPE & payment systems |
| Ley para la Transparencia y Ordenamiento de los Servicios Financieros | Ley | `ltosf` | fees, standardized contracts |

**Bridge edges out:** LRITF → Ley de Instituciones de Crédito (F2), → Ley del Mercado de Valores (F3), → LFPIORPI (F7). Ingesting F2/F3 next closes most of them.

### Batch F2 — Banking (hub: LIC)
| Instrument | Type | Slug |
|---|---|---|
| Ley de Instituciones de Crédito | Ley | `lic` (hub) |
| Ley para Regular las Agrupaciones Financieras | Ley | `lraf` |
| Ley de Protección al Ahorro Bancario | Ley | `lpab` |
| Ley de Protección y Defensa al Usuario de Servicios Financieros | Ley | `lpdusf` |
| Disposiciones de carácter general aplicables a las instituciones de crédito (Circular Única de Bancos, DOF 02/12/2005) | DCG | `cub-dcg-2005` |

**Bridge edges:** LIC ↔ LTOSF (F1), LIC → LGTOC (F1), LIC art. 115 → `pld-lic115-dcg-2009` (F7).

### Batch F3 — Securities & capital markets (hub: LMV)
| Instrument | Type | Slug |
|---|---|---|
| Ley del Mercado de Valores | Ley | `lmv` (hub) |
| Ley de Fondos de Inversión | Ley | `lfi` |
| Disposiciones … aplicables a las emisoras de valores y a otros participantes del mercado de valores (Circular Única de Emisoras, DOF 19/03/2003) | DCG | `cue-dcg-2003` |
| Disposiciones … aplicables a las casas de bolsa (Circular Única de Casas de Bolsa, DOF 06/09/2004) | DCG | `cucb-dcg-2004` |
| Disposiciones … aplicables a los fondos de inversión y a las personas que les prestan servicios (DOF 24/11/2014) | DCG | `fi-dcg-2014` |
| *(optional bridge)* Disposiciones … casas de bolsa e IC en materia de servicios de inversión (DOF 24/04/2013) | DCG | `servinv-dcg-2013` |

**Bridge edges:** `servinv-dcg-2013` also links to LIC (F2); LMV art. 124 → PLD family (F7).

### Batch F4 — Popular savings (hub: LACP)
| Instrument | Type | Slug |
|---|---|---|
| Ley de Ahorro y Crédito Popular | Ley | `lacp` (hub) |
| Ley para Regular las Actividades de las Sociedades Cooperativas de Ahorro y Préstamo | Ley | `lrascap` |
| Disposiciones … aplicables a las entidades de ahorro y crédito popular, organismos de integración, sociedades financieras comunitarias y organismos de integración financiera rural (SOFIPO, DOF 18/12/2006) | DCG | `socap-sofipo-dcg-2006` |
| Disposiciones … aplicables a las actividades de las sociedades cooperativas de ahorro y préstamo (SOCAP, DOF 04/06/2012) | DCG | `scap-dcg-2012` |

**Bridge edges:** LACP / LRASCAP → LIC (F2), → LGTOC (F1).

### Batch F5 — Auxiliary credit organizations & SOFOM (hub: LGOAAC)
| Instrument | Type | Slug |
|---|---|---|
| Ley General de Organizaciones y Actividades Auxiliares del Crédito | Ley | `lgoaac` (hub) |
| Ley de Uniones de Crédito | Ley | `luc` |
| Disposiciones … aplicables a los almacenes generales de depósito, casas de cambio, uniones de crédito y sociedades financieras de objeto múltiple reguladas (DOF 19/01/2009) | DCG | `oaac-dcg-2009` |

**Bridge edges:** SOFOM ER → LIC (F2) and LGTOC (F1); SOFOM ENR PLD obligations → PLD family (F7).

### Batch F6 — Insurance & bonding (regulator: CNSF)
| Instrument | Type | Slug |
|---|---|---|
| Ley de Instituciones de Seguros y de Fianzas | Ley | `lisf` (hub) |
| Ley sobre el Contrato de Seguro | Ley | `lscs` |
| Circular Única de Seguros y Fianzas (CNSF, DOF 19/12/2014) | DCG | `cusf-dcg-2014` |

### Batch F7 — Retirement savings + AML/CFT (bridge-heavy)
| Instrument | Type | Slug |
|---|---|---|
| Ley de los Sistemas de Ahorro para el Retiro | Ley | `lsar` (hub) |
| Disposiciones de carácter general en materia financiera de los SAR (Circular Única Financiera, CONSAR, DOF 26/01/2018) | DCG | `consar-cuf-dcg-2018` |
| Ley Federal para la Prevención e Identificación de Operaciones con Recursos de Procedencia Ilícita | Ley | `lfpiorpi` |
| Reglamento de la LFPIORPI | Reglamento | `reg-lfpiorpi` |
| Disposiciones de carácter general a que se refiere el art. 115 LIC (PLD/FT bancos — anchor of the PLD family) | DCG | `pld-lic115-dcg-2009` |

*PLD note:* each supervised sector has its own art.-115-analogue PLD DCG (art. 124 LMV casas de bolsa, art. 95/95 Bis LGOAAC SOFOM, art. 58 LRITF ITF, art. 226 Bis LFIP, etc.). Model `pld-lic115-dcg-2009` first as the family hub, then add the sectoral PLD DCGs as siblings citing the same LFPIORPI / GAFI framework.

**Bridge edges:** LFPIORPI ↔ Código Penal Federal art. 400 Bis (Domain 7, P3) — the main financial↔penal bridge; LSAR ↔ LSS RCV (Domain 5, L2).

---

## Domain 2 — Tax & public finance

### Batch T1 — Federal tax core (hub: CFF)
| Instrument | Type | Slug |
|---|---|---|
| Código Fiscal de la Federación | Código | `cff` (hub) |
| Ley del Impuesto sobre la Renta | Ley | `lisr` |
| Reglamento de la LISR | Reglamento | `reg-lisr` |
| Ley del Impuesto al Valor Agregado | Ley | `liva` |
| Reglamento de la LIVA | Reglamento | `reg-liva` |
| Ley del Impuesto Especial sobre Producción y Servicios | Ley | `lieps` |
| Reglamento de la LIEPS | Reglamento | `reg-lieps` |

*(If batch feels large, split the three substantive tax laws + reglamentos into T1a and keep CFF as its own anchor — all three cite CFF as procedural hub.)*

### Batch T2 — Fiscal coordination & budgetary discipline
| Instrument | Type | Slug |
|---|---|---|
| Ley de Coordinación Fiscal | Ley | `lcf` |
| Ley Federal de Presupuesto y Responsabilidad Hacendaria | Ley | `lfprh` |
| Ley General de Contabilidad Gubernamental | Ley | `lgcg` |
| Ley de Disciplina Financiera de las Entidades Federativas y los Municipios | Ley | `ldfefm` |
| Ley de Tesorería de la Federación + Reglamento | Ley + Reg | `ltf`, `reg-ltf` |

### Batch T3 — Customs & foreign trade
| Instrument | Type | Slug |
|---|---|---|
| Ley Aduanera | Ley | `ladua` (hub) |
| Ley de Comercio Exterior + Reglamento | Ley + Reg | `lce`, `reg-lce` |
| Ley de los Impuestos Generales de Importación y de Exportación | Ley | `ligie` |

---

## Domain 3 — Environmental & natural resources

### Batch E1 — Environmental framework (hub: LGEEPA)
| Instrument | Type | Slug |
|---|---|---|
| Ley General del Equilibrio Ecológico y la Protección al Ambiente | Ley | `lgeepa` (hub) |
| Reglamentos de la LGEEPA (impacto ambiental; áreas naturales protegidas; prevención y control de la contaminación atmosférica; autorregulación; registro de emisiones y transferencia de contaminantes) | Reglamentos | `reg-lgeepa-<materia>` |
| Ley General de Cambio Climático + Reglamento (Registro Nacional de Emisiones) | Ley + Reg | `lgcc`, `reg-lgcc-rne` |
| Ley Federal de Responsabilidad Ambiental | Ley | `lfra` |

*Note:* LGEEPA has five distinct reglamentos por materia — treat each as its own record with a title-anchored edge back to LGEEPA.

### Batch E2 — Waste, forestry & biodiversity
| Instrument | Type | Slug |
|---|---|---|
| Ley General para la Prevención y Gestión Integral de los Residuos + Reglamento | Ley + Reg | `lgpgir`, `reg-lgpgir` |
| Ley General de Vida Silvestre | Ley | `lgvs` |
| Ley General de Desarrollo Forestal Sustentable + Reglamento | Ley + Reg | `lgdfs`, `reg-lgdfs` |
| Ley de Bioseguridad de Organismos Genéticamente Modificados + Reglamento | Ley + Reg | `lbogm`, `reg-lbogm` |

### Batch E3 — Water ⚠
| Instrument | Type | Slug |
|---|---|---|
| Ley de Aguas Nacionales + Reglamento | Ley + Reg | `lan`, `reg-lan` |
| Ley General de Aguas ⚠ *(confirm whether enacted / in force)* | Ley | `lga` |

**Bridge edges:** water-use fees → Ley Federal de Derechos (tax); atmospheric emissions → LGCC (E1).

---

## Domain 4 — Energy ⚠ (2024–2025 sector reform — verify all titles)

The 2024–2025 constitutional and secondary energy reform renamed the core statutes and restructured the regulators. The Diputados reglamentos index already shows *"Reglamento de la Ley del Sector …"* entries, consistent with the renamed laws. **Confirm each current title in DOF before fetching.**

### Batch EN1 — Sector core ⚠
| Instrument (verify title) | Type | Slug |
|---|---|---|
| Ley del Sector Eléctrico (antes Ley de la Industria Eléctrica) | Ley | `lse` (hub) |
| Ley del Sector Hidrocarburos (antes Ley de Hidrocarburos) | Ley | `lsh` |
| Ley de la Comisión Nacional de Energía / órgano regulador vigente | Ley | `lcne` |
| Ley de la Empresa Pública del Estado — Comisión Federal de Electricidad | Ley | `lcfe` |
| Ley de la Empresa Pública del Estado — Petróleos Mexicanos | Ley | `lpemex` |

### Batch EN2 — Energy fees, transition & rural
| Instrument | Type | Slug |
|---|---|---|
| Ley de Ingresos sobre Hidrocarburos + Reglamento | Ley + Reg | `lih`, `reg-lih` |
| Ley de Transición Energética | Ley | `lte` |
| Ley de Energía para el Campo + Reglamento | Ley + Reg | `lec`, `reg-lec` |
| Ley de Biocombustibles *(nueva, DOF 18/03/2025)* | Ley | `lbio` |

**Bridge edges:** LIH → CFF / Ley Federal de Derechos (tax); sector laws → LGEEPA impacto ambiental (E1).

---

## Domain 5 — Labor & social security

### Batch L1 — Labor (hub: LFT)
| Instrument | Type | Slug |
|---|---|---|
| Ley Federal del Trabajo | Ley | `lft` (hub) |
| Ley Federal de los Trabajadores al Servicio del Estado (apartado B) | Ley | `lftse` |
| Ley del Instituto del Fondo Nacional de la Vivienda para los Trabajadores + Reglamento | Ley + Reg | `linfonavit`, `reg-linfonavit` |
| Ley de Ayuda Alimentaria para los Trabajadores + Reglamento | Ley + Reg | `laat`, `reg-laat` |

### Batch L2 — Social security
| Instrument | Type | Slug |
|---|---|---|
| Ley del Seguro Social + Reglamentos (afiliación; RCV/reservas; enajenación de bienes) | Ley + Regs | `lss`, `reg-lss-<materia>` |
| Ley del ISSSTE | Ley | `lissste` |
| Ley del Instituto del Fondo Nacional para el Consumo de los Trabajadores + Reglamento | Ley + Reg | `linfonacot`, `reg-linfonacot` |

**Bridge edges:** LSS RCV → LSAR (F7); INFONAVIT/INFONACOT → LFT (L1).

---

## Domain 6 — Administrative organization & public integrity

### Batch A1 — Public administration & procurement
| Instrument | Type | Slug |
|---|---|---|
| Ley Orgánica de la Administración Pública Federal | Ley | `loapf` (hub) |
| Ley Federal de Procedimiento Administrativo | Ley | `lfpa` |
| Ley de Adquisiciones, Arrendamientos y Servicios del Sector Público + Reglamento *(nueva, DOF 16/04/2025)* | Ley + Reg | `laassp`, `reg-laassp` |
| Ley de Obras Públicas y Servicios Relacionados con las Mismas | Ley | `lopsrm` |
| Ley de Asociaciones Público Privadas + Reglamento | Ley + Reg | `lapp`, `reg-lapp` |

### Batch A2 — Anti-corruption & accountability (SNA)
| Instrument | Type | Slug |
|---|---|---|
| Ley General del Sistema Nacional Anticorrupción | Ley | `lgsna` (hub) |
| Ley General de Responsabilidades Administrativas | Ley | `lgra` |
| Ley de Fiscalización y Rendición de Cuentas de la Federación | Ley | `lfrcf` |
| Ley Orgánica del Tribunal Federal de Justicia Administrativa | Ley | `lotfja` |
| Reglamento de la Ley del Servicio Profesional de Carrera en la APF | Reglamento | `reg-lspcapf` |

### Batch A3 — Transparency, data protection & archives ⚠
| Instrument (verify post-INAI reform) | Type | Slug |
|---|---|---|
| Ley General de Transparencia y Acceso a la Información Pública ⚠ | Ley | `lgtaip` |
| Ley Federal de Transparencia y Acceso a la Información Pública + Reglamento ⚠ | Ley + Reg | `lftaip`, `reg-lftaip` |
| Ley General de Protección de Datos Personales en Posesión de Sujetos Obligados ⚠ | Ley | `lgpdppso` |
| Ley Federal de Protección de Datos Personales en Posesión de los Particulares + Reglamento ⚠ *(reformada 2025)* | Ley + Reg | `lfpdppp`, `reg-lfpdppp` |
| Ley General de Archivos | Ley | `lga-archivos` |

⚠ The 2024–2025 reform abolishing the Instituto Nacional de Transparencia (INAI) reassigned these functions and prompted new/reformed statutes. Confirm current titles and in-force dates before ingest.

**Bridge edges:** LFPDPPP → LFPA (A1); data protection ↔ LFTR (Domain 9).

---

## Domain 7 — Penal & criminal justice

### Batch P1 — Criminal core (hubs: CPF + CNPP)
| Instrument | Type | Slug |
|---|---|---|
| Código Penal Federal | Código | `cpf` (substantive hub) |
| Código Nacional de Procedimientos Penales | Código | `cnpp` (procedural hub) |
| Ley Nacional de Ejecución Penal | Ley | `lnep` |
| Ley Nacional del Sistema Integral de Justicia Penal para Adolescentes | Ley | `lnsijpa` |
| Ley Nacional de Mecanismos Alternativos de Solución de Controversias en Materia Penal | Ley | `lnmasc-penal` |

### Batch P2 — Security & organized crime
| Instrument | Type | Slug |
|---|---|---|
| Ley General del Sistema Nacional de Seguridad Pública | Ley | `lgsnsp` (hub) |
| Ley Federal contra la Delincuencia Organizada | Ley | `lfdo` |
| Ley de Seguridad Nacional | Ley | `lsn` |
| Ley de la Guardia Nacional + Reglamento | Ley + Reg | `lgn`, `reg-lgn` |
| Ley Nacional del Registro de Detenciones / Ley Nacional sobre el Uso de la Fuerza | Ley | `lnrd`, `lnuf` |

### Batch P3 — Special penal regimes (bridge-heavy)
| Instrument | Type | Slug |
|---|---|---|
| Ley General para Prevenir, Sancionar y Erradicar los Delitos en Materia de Trata de Personas + Reglamento | Ley + Reg | `lgtrata`, `reg-lgtrata` |
| Ley Federal de Extinción de Dominio | Ley | `lfed` |
| Ley Federal para el Control de Precursores Químicos + Reglamento | Ley + Reg | `lfcpq`, `reg-lfcpq` |
| (bridge) Ley Federal para la Prevención e Identificación de Operaciones con Recursos de Procedencia Ilícita | Ley | `lfpiorpi` → F7 |

**Bridge edges:** CPF art. 400 Bis ↔ LFPIORPI (F7); extinción de dominio ↔ CNPP (P1). Ingest P3 *after* F7 to close the money-laundering bridge in one direction.

---

## Domain 8 — Civil, commercial & private law

### Batch C1 — Commercial (hubs: Código de Comercio, LGSM, LGTOC)
| Instrument | Type | Slug |
|---|---|---|
| Código de Comercio | Código | `ccom` (hub) |
| Ley General de Sociedades Mercantiles | Ley | `lgsm` |
| Ley General de Títulos y Operaciones de Crédito | Ley | `lgtoc` *(shared with F1)* |
| Ley de Concursos Mercantiles | Ley | `lcm` |
| Ley Federal de Protección al Consumidor + Reglamento | Ley + Reg | `lfpc`, `reg-lfpc` |

*Ingest LGTOC once (in F1) and let C1 reference the same record — do not duplicate.*

### Batch C2 — Civil & intellectual property
| Instrument | Type | Slug |
|---|---|---|
| Código Civil Federal | Código | `ccf` (hub) |
| Código Nacional de Procedimientos Civiles y Familiares | Código | `cnpcf` |
| Ley Federal del Derecho de Autor + Reglamento | Ley + Reg | `lfda`, `reg-lfda` |
| Ley Federal de Protección a la Propiedad Industrial | Ley | `lfppi` |

---

## Domain 9 — Telecom, competition & digital trust ⚠

### Batch D1
| Instrument (verify) | Type | Slug |
|---|---|---|
| Ley Federal de Telecomunicaciones y Radiodifusión ⚠ *(nueva 2025, abrogó IFT)* + Reglamento(s) | Ley + Reg | `lftr`, `reg-lftr-<materia>` |
| Ley Federal de Competencia Económica ⚠ *(verify post-reform regulador)* + Reglamento | Ley + Reg | `lfce`, `reg-lfce` |
| Ley de Firma Electrónica Avanzada + Reglamento | Ley + Reg | `lfea`, `reg-lfea` |
| Ley de Infraestructura de la Calidad *(sustituyó a la Ley Federal sobre Metrología y Normalización)* | Ley | `lic-calidad` |

**Bridge edges:** LFEA ↔ LGTOC / Código de Comercio (e-commerce, mensajes de datos); LFCE ↔ sector laws (telecom, energy, financial).

---

## Domain 10 — Health

### Batch S1 (hub: LGS)
| Instrument | Type | Slug |
|---|---|---|
| Ley General de Salud | Ley | `lgs` (hub) |
| Reglamentos de la LGS por materia (insumos para la salud; investigación; publicidad; sanidad internacional; trasplantes; control sanitario de productos y servicios) | Reglamentos | `reg-lgs-<materia>` |
| Ley de los Institutos Nacionales de Salud | Ley | `lins` |

*LGS carries the largest reglamento fan-out in the corpus (six+ reglamentos por materia). Treat the batch as LGS + one record per reglamento; do not merge.*

---

## Domain 11 — Mobility, transport & infrastructure

### Batch MI1 — Land & air
| Instrument | Type | Slug |
|---|---|---|
| Ley de Caminos, Puentes y Autotransporte Federal | Ley | `lcpaf` |
| Ley de Aviación Civil + Reglamento | Ley + Reg | `lac`, `reg-lac` |
| Ley de Aeropuertos + Reglamento | Ley + Reg | `laero`, `reg-laero` |
| Ley General de Movilidad y Seguridad Vial | Ley | `lgmsv` |

### Batch MI2 — Maritime & general communications
| Instrument | Type | Slug |
|---|---|---|
| Ley de Navegación y Comercio Marítimos + Reglamento | Ley + Reg | `lncm`, `reg-lncm` |
| Ley de Puertos + Reglamento | Ley + Reg | `lpue`, `reg-lpue` |
| Ley de Vías Generales de Comunicación | Ley | `lvgc` |

---

## Suggested global ingestion order

The order below front-loads the most-cited hubs so later batches resolve against records already committed.

1. **F1 → F2 → F3 → F4 → F5 → F6 → F7** (finance; F1 also closes the current corpus)
2. **C1 → C2** (commercial/civil hubs LGTOC, Código de Comercio, LGSM, Código Civil — heavily cited by finance, tax, penal)
3. **T1 → T2 → T3** (tax; CFF cited by finance & energy)
4. **P1 → P2 → P3** (penal; P3 after F7 closes the AML bridge)
5. **A1 → A2 → A3** (administrative; A3 gated on INAI-reform verification)
6. **L1 → L2** (labor/social; L2 after F7 for the LSAR bridge)
7. **E1 → E2 → E3**, **EN1 → EN2** (environment/energy; gated on reform verification)
8. **D1, S1, MI1 → MI2** (telecom/digital, health, transport)

## Notes for the pipeline

- **Shared records, not duplicates.** LGTOC (F1/C1), LFPIORPI (F7/P3), LSAR (F7/L2) each belong to two domains. Ingest once; let the second batch link to the existing canonical record. This is exactly the cross-instrument edge case the linker is built for.
- **Reglamentos por materia.** LGEEPA, LGS and LSS each split into multiple reglamentos. Model each as a separate canonical record with a title-anchored edge to its parent statute, mirroring how the IFPE per-annex PDFs are handled.
- **DCG provenance.** Pull CNBV/CNSF/CONSAR DCGs from the regulator, not Diputados, and capture the compiled-vs-original distinction (as done for `itf-dcg-2018`'s six amending resolutions). Attach the formal DOF publication where a determination depends on promulgation or commencement.
- **⚠ instruments** (energy, telecom, competition, transparency/data, water) sit on 2024–2026 reforms. Route the title/in-force confirmation to the reviewer of record before fetch; do not let a remembered title enter the source manifest.
