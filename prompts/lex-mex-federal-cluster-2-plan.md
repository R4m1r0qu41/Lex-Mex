# Lex-Mex — Federal Corpus Ingestion Cluster Plan 2

**Purpose.** Complete the federal leyes + reglamentos corpus. Cluster 1
(28 batches, 138 instruments, converged 2026-07-10) covered the financial,
commercial/civil, tax-core, penal, admin, labor, environment, health and
transport hubs. This second cluster ingests **every remaining ley and
reglamento vigente** on the official Cámara de Diputados indexes,
**excluding all DCG** (financial regulator dispositions are tracked in the
cluster-1 DCG register).

**Source verification (2026-07-10).**
- `https://www.diputados.gob.mx/LeyesBiblio/index.htm` — 316 leyes vigentes parsed (row numbers 001–316, complete).
- `https://www.diputados.gob.mx/LeyesBiblio/regla.htm` — 138 reglamentos vigentes parsed (numbered rows 1–138 minus the page's own numbering gap at 47, plus the unnumbered Reg. LISSFAM row; the page's 11 «Reglamento Abrogado» rows were excluded).
- Total universe: **454** instruments. In corpus (excl. 10 DCG): **128**. Missing → this cluster: **326** (227 leyes + 99 reglamentos).
- Every title, PDF URL and DOF date below was taken from the live index rows, not from remembered titles.

**Manifests.** One import manifest per batch in `prompts/cluster-2-batches/`
(same schema the vault importer consumes: `slug`, `title`, `type`,
`adapter`, `status`, `ref_page`, `source_pdf`, plus additive
`dof_publication`, `dof_last_reform`, `note`). `status: BLOCKED` entries
are skipped by the importer and require JRH confirmation.

**Slug convention.** Leyes use the official LeyesBiblio `ref/` slug;
reglamentos use `reg-` + the official `regley/` file stem. Both are
lower-cased with hyphens.

**Recommended global order.** As listed below: constitutional and
administrative hubs first (they are the most-cited targets), then fiscal/
financial/economic, justice & security, rights & social, sectoral domains.
Within a batch, ingest the hub (first row) before its reglamentos and peers.

---

## Domain CN — Constitución & Congreso

### Batch CN1 — `cl2_CN1_constitucion_congreso` (6 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| CONSTITUCIÓN Política de los Estados Unidos Mexicanos | constitucion | `cpeum` | NEW |
| LEY Orgánica del Congreso General de los Estados Unidos Mexicanos | ley | `locg` | NEW |
| REGLAMENTO de la Cámara de Diputados | reglamento_legislativo | `reg-diputados` | NEW |
| REGLAMENTO del Senado de la República | reglamento_legislativo | `reg-senado` | NEW |
| REGLAMENTO para el Gobierno Interior del Congreso General de los Estados Unidos Mexicanos | reglamento_legislativo | `rgic` | NEW |
| LEY del Diario Oficial de la Federación y Gacetas Gubernamentales | ley | `ldofgg` | NEW |

### Batch CN2 — `cl2_CN2_leyes_reglamentarias` (10 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY Reglamentaria de las Fracciones I y II del Artículo 105 de la Constitución Política de los Estados Unidos Mexicanos | ley | `lrfiyii-art105` | NEW |
| LEY Reglamentaria del artículo 6o., párrafo primero, de la Constitución Política de los Estados Unidos Mexicanos, en materia del Derecho de Réplica | ley | `lrart6-mdr` | NEW |
| LEY Reglamentaria de la Fracción V del Artículo 76 de la Constitución General de la República | ley | `lrfv-art76` | NEW |
| LEY Reglamentaria de la fracción VI del artículo 76 de la Constitución Política de los Estados Unidos Mexicanos | ley | `lrart76-fracvi` | NEW |
| LEY Reglamentaria de la Fracción XIII Bis del Apartado B, del Artículo 123 de la Constitución Política de los Estados Unidos Mexicanos | ley | `lrfxiiib-art123` | NEW |
| LEY Reglamentaria de la Fracción XVIII del Artículo 73 Constitucional, en lo que se Refiere a la Facultad del Congreso para Dictar Reglas para Determinar el Valor Relativo de la Moneda Extranjera | ley | `lrfxviii-art73` | NEW |
| LEY Federal de Consulta Popular | ley | `lfcpo` | NEW |
| LEY Federal de Revocación de Mandato | ley | `lfrm` | NEW |
| LEY sobre la Celebración de Tratados | ley | `lsct` | NEW |
| LEY Sobre la Aprobación de Tratados Internacionales en Materia Económica | ley | `latime` | NEW |

## Domain AD — Administración pública federal

### Batch AD1 — `cl2_AD1_planeacion_paraestatales` (6 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY de Planeación | ley | `lplan` | NEW |
| LEY Federal de las Entidades Paraestatales | ley | `lfep` | NEW |
| REGLAMENTO de la Ley Federal de las Entidades Paraestatales | reglamento | `reg-lfep` | NEW |
| LEY Federal de Responsabilidad Patrimonial del Estado | ley | `lfrpe` | NEW |
| LEY Federal de Responsabilidades de los Servidores Públicos | ley | `lfrsp` | NEW |
| LEY General de Bienes Nacionales | ley | `lgbn` | NEW |

### Batch AD2 — `cl2_AD2_bienes_obras_servicios` (5 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY del Servicio Postal Mexicano | ley | `lspm` | NEW |
| LEY Federal para la Administración y Enajenación de Bienes del Sector Público | ley | `lfaebsp` | NEW |
| REGLAMENTO de la Ley Federal para la Administración y Enajenación de Bienes del Sector Público | reglamento | `reg-lfaebsp` | NEW |
| REGLAMENTO de la Ley de Obras Públicas y Servicios Relacionados con las Mismas | reglamento | `reg-lopsrm` | NEW |
| LEY Federal de Austeridad Republicana | ley | `lfar` | NEW |

### Batch AD3 — `cl2_AD3_servicio_publico_laboral` (5 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY del Servicio Profesional de Carrera en la Administración Pública Federal | ley | `lspcapf` | NEW |
| LEY Federal de Remuneraciones de los Servidores Públicos | ley | `lfremsp` | NEW |
| LEY Orgánica del Centro Federal de Conciliación y Registro Laboral | ley | `locfcrl` | NEW |
| REGLAMENTO de los Artículos 121 y 122 de la Ley Federal del Trabajo | reglamento | `reg-art121-122-lft` | NEW |
| REGLAMENTO de la Ley de Ayuda Alimentaria para los Trabajadores | reglamento | `reg-laat` | NEW ⚠ |

- ⚠ `reg-laat`: Pairs with corpus LAAT (Ley de Ayuda Alimentaria para los Trabajadores, batch L1) — NOT Ley de Aeropuertos; the Reglamento de la Ley de Aeropuertos is still absent from the official regley index.

### Batch AD4 — `cl2_AD4_proteccion_civil_misc` (5 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY General de Protección Civil | ley | `lgpc` | NEW |
| REGLAMENTO de la Ley General de Protección Civil | reglamento | `reg-lgpc` | NEW |
| LEY de los Husos Horarios en los Estados Unidos Mexicanos | ley | `lhheum` | NEW |
| LEY para Determinar el Valor de la Unidad de Medida y Actualización | ley | `ldvuma` | NEW |
| LEY para el Diálogo, la Conciliación y la Paz Digna en Chiapas | ley | `ldcpdch` | NEW |

## Domain TX — Fiscal — segunda ola

### Batch TX1 — `cl2_TX1_sat_procedimiento` (5 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY del Servicio de Administración Tributaria | ley | `lsat` | NEW |
| LEY Federal de los Derechos del Contribuyente | ley | `lfdc` | NEW |
| LEY Orgánica de la Procuraduría de la Defensa del Contribuyente | ley | `lopdc` | NEW |
| LEY Federal de Procedimiento Contencioso Administrativo | ley | `lfpca` | NEW |
| REGLAMENTO del Código Fiscal de la Federación | reglamento | `reg-cff` | NEW |

### Batch TX2 — `cl2_TX2_ingresos_presupuesto` (6 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY Federal de Derechos | ley | `lfd` | NEW |
| LEY Federal de Deuda Pública | ley | `lgdp` | NEW |
| REGLAMENTO de la Ley Federal de Presupuesto y Responsabilidad Hacendaria | reglamento | `reg-lfprh` | NEW |
| LEY de Contribución de Mejoras por Obras Públicas Federales de Infraestructura Hidráulica | ley | `lcmopfih` | NEW |
| LEY de Ingresos de la Federación para el Ejercicio Fiscal de 2026 | ley | `lif-2026` | BLOCKED ⚠ |
| PRESUPUESTO de Egresos de la Federación para el Ejercicio Fiscal 2026 | decreto | `pef-2026` | BLOCKED ⚠ |

- ⚠ `lif-2026`: Annual instrument (expires with FY2026); confirm temporal-model treatment before ingesting.
- ⚠ `pef-2026`: Annual decree (expires with FY2026); confirm temporal-model treatment before ingesting.

### Batch TX3 — `cl2_TX3_impuestos_aduanas` (3 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY Federal del Impuesto sobre Automóviles Nuevos | ley | `lfisan` | NEW |
| IMPUESTO sobre Servicios Expresamente Declarados de Interés Público por Ley, en los que Intervengan Empresas Concesionarias de Bienes del Dominio Directo de la Nación (LEY que establece, reforma y adiciona las disposiciones relativas a diversos impuestos) | ley | `lisipl` | NEW |
| REGLAMENTO de la Ley Aduanera | reglamento | `reg-ladua` | NEW |

## Domain FI — Financiero — segunda ola

### Batch FI1 — `cl2_FI1_autoridades_pagos` (5 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY de la Comisión Nacional Bancaria y de Valores | ley | `lcnbv` | NEW |
| LEY de Sistemas de Pagos | ley | `lsp` | NEW |
| LEY Monetaria de los Estados Unidos Mexicanos | ley | `lmeum` | NEW |
| LEY de la Casa de Moneda de México | ley | `lcmm` | NEW |
| LEY de Transparencia y de Fomento a la Competencia en el Crédito Garantizado | ley | `ltfccg` | NEW |

### Batch FI2 — `cl2_FI2_banca_desarrollo` (6 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY Orgánica de Nacional Financiera | ley | `lonf` | NEW |
| LEY Orgánica de Sociedad Hipotecaria Federal | ley | `loshf` | NEW |
| LEY Orgánica del Banco del Bienestar | ley | `lobb` | NEW |
| LEY Orgánica del Banco Nacional de Comercio Exterior | ley | `lobnce` | NEW |
| LEY Orgánica del Banco Nacional de Obras y Servicios Públicos | ley | `lobnosp` | NEW |
| LEY Orgánica del Banco Nacional del Ejército, Fuerza Aérea y Armada | ley | `lobnefaa` | NEW |

### Batch FI3 — `cl2_FI3_seguro_rural_convenios` (8 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY de Fondos de Aseguramiento Agropecuario y Rural | ley | `lfaar` | NEW |
| LEY que Crea el Fondo de Garantía y Fomento para la Agricultura, Ganadería y Avicultura | ley | `lfgfaga` | NEW |
| REGLAMENTO de la Ley que Crea el Fondo de Garantía y Fomento para la Agricultura, Ganadería y Avicultura | reglamento | `reg-lfgfaga` | NEW |
| REGLAMENTO del Artículo 95 de la Ley Federal de Instituciones de Fianzas, para el Cobro de Fianzas Otorgadas a Favor de la Federación, del Distrito Federal, de los Estados y de los Municipios, Distintas de las que Garantizan Obligaciones Fiscales Federales a cargo de Terceros | reglamento | `reg-lfif-art95` | NEW ⚠ |
| REGLAMENTO de la Ley de los Sistemas de Ahorro para el Retiro | reglamento | `reg-lsar` | NEW |
| LEY que Aprueba la Adhesión de México al Convenio Constitutivo del Banco de Desarrollo del Caribe y su Ejecución | ley | `lmccbdc` | NEW |
| LEY que Establece Bases para la Ejecución en México, por el Poder Ejecutivo Federal, del Convenio Constitutivo de la Asociación Internacional de Fomento | ley | `lccaif` | NEW |
| LEY que Establece Bases para la Ejecución en México, por el Poder Ejecutivo Federal, del Convenio Constitutivo del Banco Interamericano de Desarrollo | ley | `lccbid` | NEW |

- ⚠ `reg-lfif-art95`: Parent Ley Federal de Instituciones de Fianzas abrogated (now LISF); reglamento remains listed vigente.

## Domain EC — Economía, competencia & comercio

### Batch EC1 — `cl2_EC1_competencia_inversion` (6 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY Federal de Competencia Económica | ley | `lfce` | NEW |
| REGLAMENTO de la Ley Federal de Competencia Económica | reglamento | `reg-lfce` | NEW |
| LEY de Inversión Extranjera | ley | `lie` | NEW |
| REGLAMENTO de la Ley de Inversión Extranjera y del Registro Nacional de Inversiones Extranjeras | reglamento | `reg-liernie` | NEW |
| LEY de Protección al Comercio y la Inversión de Normas Extranjeras que Contravengan el Derecho Internacional | ley | `lpcinecdi` | NEW |
| LEY para Regular las Sociedades de Información Crediticia | ley | `lrsic` | NEW |

### Batch EC2 — `cl2_EC2_empresas_mipyme` (9 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY de Cámaras Empresariales y sus Confederaciones | ley | `lcec` | NEW |
| REGLAMENTO de la Ley de Cámaras Empresariales y sus Confederaciones | reglamento | `reg-lcec` | NEW |
| LEY para el Desarrollo de la Competitividad de la Micro, Pequeña y Mediana Empresa | ley | `ldcmpme` | NEW |
| REGLAMENTO de la Ley para el Desarrollo de la Competitividad de la Micro, Pequeña y Mediana Empresa | reglamento | `reg-ldcmpme` | NEW |
| LEY Federal para el Fomento de la Microindustria y la Actividad Artesanal | ley | `lffmaa` | NEW |
| LEY para Impulsar el Incremento Sostenido de la Productividad y la Competitividad de la Economía Nacional | ley | `liispcen` | NEW |
| LEY Nacional para Eliminar Trámites Burocráticos | ley | `lnetb` | NEW |
| LEY de Fomento a la Confianza Ciudadana | ley | `lfcc` | NEW |
| REGLAMENTO de la Ley Federal Sobre Metrología y Normalización | reglamento | `reg-lfmn` | NEW ⚠ |

- ⚠ `reg-lfmn`: Parent Ley Federal sobre Metrología y Normalización abrogated (now Ley de Infraestructura de la Calidad); reglamento remains listed vigente.

### Batch EC3 — `cl2_EC3_comercio_fedatarios_ip` (7 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY Federal de Correduría Pública | ley | `lfcp` | NEW |
| REGLAMENTO de la Ley Federal de Correduría Pública | reglamento | `reg-lfcp` | NEW |
| REGLAMENTO del Código de Comercio en Materia de Prestadores de Servicios de Certificación | reglamento | `reg-ccomer-mpsc` | NEW |
| REGLAMENTO del Artículo 122 de la Ley Federal de Protección al Consumidor | reglamento | `reg-lfpc-art122` | NEW |
| LEY Federal de Juegos y Sorteos | ley | `lfjs` | NEW |
| REGLAMENTO de la Ley Federal de Juegos y Sorteos | reglamento | `reg-lfjs` | NEW |
| REGLAMENTO de la Ley Federal de Protección a la Propiedad Industrial | reglamento | `reg-lfppi` | NEW |

### Batch EC4 — `cl2_EC4_zonas_infraestructura` (5 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY Federal de Zonas Económicas Especiales | ley | `lfzee` | NEW |
| REGLAMENTO de la Ley Federal de Zonas Económicas Especiales | reglamento | `reg-lfzee` | NEW |
| LEY para el Fomento de la Inversión en Infraestructura Estratégica para el Desarrollo con Bienestar | ley | `lfiiedb` | NEW |
| REGLAMENTO de la Ley para el Fomento de la Inversión en Infraestructura Estratégica para el Desarrollo con Bienestar Nuevo Reglamento | reglamento | `reg-lfiiedb` | NEW |
| LEY para la Transparencia, Prevención y Combate de Prácticas Indebidas en Materia de Contratación de Publicidad | ley | `ltpcpimcp` | NEW |

## Domain JU — Justicia federal

### Batch JU1 — `cl2_JU1_poder_judicial_fiscalia` (6 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY Orgánica del Poder Judicial de la Federación | ley | `lopjf` | NEW |
| LEY de Carrera Judicial del Poder Judicial de la Federación | ley | `lcjpjf` | NEW |
| LEY de la Fiscalía General de la República | ley | `lfgr` | NEW |
| REGLAMENTO de la Ley Orgánica de la Procuraduría General de la República | reglamento | `reg-lopgr` | NEW ⚠ |
| LEY Federal de Defensoría Pública | ley | `lfdefp` | NEW |
| LEY General de Mecanismos Alternativos de Solución de Controversias | ley | `lgmasc` | NEW |

- ⚠ `reg-lopgr`: Parent LOPGR abrogated (now Ley de la FGR); reglamento remains on the vigentes index.

### Batch JU2 — `cl2_JU2_justicia_df_extradicion` (7 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY Orgánica de la Procuraduría General de Justicia del Distrito Federal | ley | `lopgjdf` | NEW ⚠ |
| REGLAMENTO de la Ley Orgánica de la Procuraduría General de Justicia del Distrito Federal | reglamento | `reg-lopgjdf` | NEW ⚠ |
| LEY de Extradición Internacional | ley | `lei` | NEW |
| LEY de Amnistía | ley | `lamn` | NEW |
| LEY de Amnistía | ley | `lamni` | NEW |
| LEY Federal para la Protección a Personas que Intervienen en el Procedimiento Penal | ley | `lfppipp` | NEW |
| ESTATUTO de Gobierno del Distrito Federal | estatuto | `egdf` | BLOCKED ⚠ |

- ⚠ `lopgjdf`: Distrito Federal institution superseded by CDMX Fiscalía; still on the federal vigentes index.
- ⚠ `reg-lopgjdf`: Reglamento of the DF Procuraduría; see lopgjdf note.
- ⚠ `egdf`: Estatuto de Gobierno del Distrito Federal: transitional post-CDMX-Constitution status; confirm scope.

## Domain PE — Penal especial (extiende P1–P3)

### Batch PE1 — `cl2_PE1_delitos_graves` (5 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY General para Prevenir y Sancionar los Delitos en Materia de Secuestro, Reglamentaria de la fracción XXI del artículo 73 de la Constitución Política de los Estados Unidos Mexicanos | ley | `lgpsdms` | NEW |
| Ley General para Prevenir, Investigar y Sancionar los Delitos en Materia de Extorsión, Reglamentaria de la fracción XXI del artículo 73 de la Constitución Política de los Estados Unidos Mexicanos | ley | `lgpisdme` | NEW |
| LEY General para Prevenir, Investigar y Sancionar la Tortura y Otros Tratos o Penas Crueles, Inhumanos o Degradantes | ley | `lgpist` | NEW |
| LEY General en Materia de Desaparición Forzada de Personas, Desaparición Cometida por Particulares y del Sistema Nacional de Búsqueda de Personas | ley | `lgmdfp` | NEW |
| LEY Federal de Declaración Especial de Ausencia para Personas Desaparecidas | ley | `lfdeapd` | NEW |

### Batch PE2 — `cl2_PE2_delitos_materia_especial` (2 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY Federal para Prevenir y Sancionar los Delitos Cometidos en Materia de Hidrocarburos | ley | `lfpsdmh` | NEW |
| LEY Federal para el Control de Sustancias Químicas Susceptibles de Desvío para la Fabricación de Armas Químicas | ley | `lfcsq` | NEW |

## Domain SG — Seguridad pública

### Batch SG1 — `cl2_SG1_seguridad_interior` (6 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY de Seguridad Interior | ley | `lsint` | NEW |
| LEY de la Policía Federal | ley | `lpf` | NEW |
| REGLAMENTO de la Ley de la Policía Federal | reglamento | `reg-lpf` | NEW |
| LEY del Sistema Nacional de Investigación e Inteligencia en Materia de Seguridad Pública | ley | `lsniimsp` | NEW |
| LEY General para la Prevención Social de la Violencia y la Delincuencia | ley | `lgpsvd` | NEW |
| REGLAMENTO de la Ley General para la Prevención Social de la Violencia y la Delincuencia | reglamento | `reg-lgpsvd` | NEW |

### Batch SG2 — `cl2_SG2_seguridad_privada_armas` (6 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY Federal de Seguridad Privada | ley | `lfsp` | NEW |
| REGLAMENTO de la Ley Federal de Seguridad Privada | reglamento | `reg-lfsp` | NEW |
| LEY Federal de Armas de Fuego y Explosivos | ley | `lfafe` | NEW |
| REGLAMENTO de la Ley Federal de Armas de Fuego y Explosivos | reglamento | `reg-lfafe` | NEW |
| LEY del Registro Público Vehicular | ley | `lrpv` | NEW |
| REGLAMENTO de la Ley del Registro Público Vehicular | reglamento | `reg-lrpv` | NEW |

## Domain EL — Electoral

### Batch EL1 — `cl2_EL1_electoral` (5 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY General de Instituciones y Procedimientos Electorales | ley | `lgipe` | NEW |
| LEY General de Partidos Políticos | ley | `lgpp` | NEW |
| LEY General del Sistema de Medios de Impugnación en Materia Electoral | ley | `lgsmime` | NEW |
| LEY General de los Medios de Impugnación en Materia Electoral | ley | `lgmime` | NEW |
| LEY General en Materia de Delitos Electorales | ley | `lgmde` | NEW |

## Domain DH — Derechos humanos

### Batch DH1 — `cl2_DH1_igualdad_mujeres` (7 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY de la Comisión Nacional de los Derechos Humanos | ley | `lcndh` | NEW |
| LEY Federal para Prevenir y Eliminar la Discriminación | ley | `lfped` | NEW |
| LEY General para la Igualdad Sustantiva entre Mujeres y Hombres | ley | `lgimh` | NEW |
| LEY General de Acceso de las Mujeres a una Vida Libre de Violencias | ley | `lgamvlv` | NEW |
| REGLAMENTO de la Ley General de Acceso de las Mujeres a una Vida Libre de Violencia | reglamento | `reg-lgamvlv` | NEW |
| LEY para la Protección de Personas Defensoras de Derechos Humanos y Periodistas | ley | `lppddhp` | NEW |
| REGLAMENTO de la Ley para la Protección de Personas Defensoras de Derechos Humanos y Periodistas | reglamento | `reg-lppddhp` | NEW |

### Batch DH2 — `cl2_DH2_ninez_grupos_vulnerables` (9 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY General de los Derechos de Niñas, Niños y Adolescentes | ley | `lgdnna` | NEW |
| REGLAMENTO de la Ley General de los Derechos de Niñas, Niños y Adolescentes | reglamento | `reg-lgdnna` | NEW |
| LEY de los Derechos de las Personas Adultas Mayores | ley | `ldpam` | NEW |
| LEY General para la Inclusión de las Personas con Discapacidad | ley | `lgipd` | NEW |
| REGLAMENTO de la Ley General para la Inclusión de las Personas con Discapacidad | reglamento | `reg-lgipd` | NEW |
| LEY General para la Atención y Protección a Personas con la Condición del Espectro Autista | ley | `lgappcea` | NEW |
| REGLAMENTO de la Ley General para la Atención y Protección a Personas con la Condición del Espectro Autista | reglamento | `reg-lgappcea` | NEW |
| LEY General de Prestación de Servicios para la Atención, Cuidado y Desarrollo Integral Infantil | ley | `lgpsacdii` | NEW |
| REGLAMENTO de la Ley General de Prestación de Servicios para la Atención, Cuidado y Desarrollo Integral Infantil | reglamento | `reg-lgpsacdii` | NEW |

### Batch DH3 — `cl2_DH3_victimas_pueblos_indigenas` (5 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY General de Víctimas | ley | `lgv` | NEW |
| REGLAMENTO de la Ley General de Víctimas | reglamento | `reg-lgv` | NEW |
| LEY del Instituto Nacional de los Pueblos Indígenas | ley | `linpi` | NEW |
| LEY General de Derechos Lingüísticos de los Pueblos Indígenas | ley | `lgdlpi` | NEW |
| Ley Federal de Protección del Patrimonio Cultural de los Pueblos y Comunidades Indígenas y Afromexicanas | ley | `lfppcpcia` | NEW |

## Domain TR — Transparencia, datos & archivos

### Batch TR1 — `cl2_TR1_transparencia_datos` (5 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY General de Transparencia y Acceso a la Información Pública | ley | `lgtaip` | NEW |
| REGLAMENTO de la Ley Federal de Transparencia y Acceso a la Información Pública Gubernamental | reglamento | `reg-lftaipg` | NEW ⚠ |
| LEY General de Protección de Datos Personales en Posesión de Sujetos Obligados | ley | `lgpdppso` | NEW |
| LEY Federal de Protección de Datos Personales en Posesión de los Particulares | ley | `lfpdppp` | NEW |
| REGLAMENTO de la Ley Federal de Protección de Datos Personales en Posesión de los Particulares | reglamento | `reg-lfpdppp` | NEW |

- ⚠ `reg-lftaipg`: Parent LFTAIPG abrogated (now LGTAIP/LFTAIP regime); reglamento remains listed vigente.

### Batch TR2 — `cl2_TR2_archivos_estadistica` (5 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY General de Archivos | ley | `lga` | NEW |
| REGLAMENTO de la Ley Federal de Archivos | reglamento | `reg-lfa` | NEW ⚠ |
| LEY del Sistema Nacional de Información Estadística y Geográfica | ley | `lsnieg` | NEW |
| REGLAMENTO de la Ley de Información Estadística y Geográfica | reglamento | `reg-lieg` | NEW ⚠ |
| LEY General de Comunicación Social | ley | `lgcs` | NEW |

- ⚠ `reg-lfa`: Parent Ley Federal de Archivos abrogated (now Ley General de Archivos); reglamento remains listed vigente.
- ⚠ `reg-lieg`: Parent Ley de Información Estadística y Geográfica abrogated (now LSNIEG); reglamento remains listed vigente.

## Domain RE — Población, migración & exterior

### Batch RE1 — `cl2_RE1_poblacion_migracion` (8 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY General de Población | ley | `lgp` | NEW |
| REGLAMENTO de la Ley General de Población | reglamento | `reg-lgp` | NEW |
| LEY de Migración | ley | `lmigra` | NEW |
| REGLAMENTO de la Ley de Migración | reglamento | `reg-lmigra` | NEW |
| LEY de Nacionalidad | ley | `lnac` | NEW |
| REGLAMENTO de la Ley de Nacionalidad | reglamento | `reg-lnac` | NEW |
| LEY sobre Refugiados, Protección Complementaria y Asilo Político | ley | `lrpcap` | NEW |
| REGLAMENTO de la Ley sobre Refugiados y Protección Complementaria | reglamento | `reg-lrpc` | NEW ⚠ |

- ⚠ `reg-lrpc`: Parent renamed to Ley sobre Refugiados, Protección Complementaria y Asilo Político (lrpcap, this batch).

### Batch RE2 — `cl2_RE2_exterior_religion_simbolos` (10 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY del Servicio Exterior Mexicano | ley | `lsem` | NEW |
| REGLAMENTO de la Ley del Servicio Exterior Mexicano | reglamento | `reg-lsem` | NEW |
| LEY para Conservar la Neutralidad del País | ley | `lcnp` | NEW |
| LEY de Cooperación Internacional para el Desarrollo | ley | `lcid` | NEW |
| LEY de Asociaciones Religiosas y Culto Público | ley | `larcp` | NEW |
| REGLAMENTO de la Ley de Asociaciones Religiosas y Culto Público | reglamento | `reg-larcp` | NEW |
| LEY sobre el Escudo, la Bandera y el Himno Nacionales | ley | `lebhn` | NEW |
| REGLAMENTO de la Ley sobre el Escudo, la Bandera y el Himno Nacionales | reglamento | `reg-lebhn` | NEW |
| LEY para el uso y protección de la denominación y del emblema de la Cruz Roja | ley | `lupdecr` | NEW |
| REGLAMENTO de la Ley para el Uso y Protección de la Denominación y del Emblema de la Cruz Roja | reglamento | `reg-lupdecr` | NEW |

## Domain SD — Desarrollo social & vivienda

### Batch SD1 — `cl2_SD1_desarrollo_social` (6 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY General de Desarrollo Social | ley | `lgds` | NEW |
| REGLAMENTO de la Ley General de Desarrollo Social | reglamento | `reg-lgds` | NEW |
| LEY de Asistencia Social | ley | `lasoc` | NEW |
| LEY General de la Alimentación Adecuada y Sostenible | ley | `lgaas` | NEW |
| LEY de Vivienda | ley | `lviv` | NEW |
| LEY General de Asentamientos Humanos, Ordenamiento Territorial y Desarrollo Urbano | ley | `lgahotdu` | NEW |

### Batch SD2 — `cl2_SD2_economia_social` (6 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY de la Economía Social y Solidaria | ley | `less` | NEW |
| LEY de Sociedades de Solidaridad Social | ley | `lsss` | NEW |
| LEY General de Sociedades Cooperativas | ley | `lgsc` | NEW |
| LEY de Sociedades de Responsabilidad Limitada de Interés Público | ley | `lsrlip` | NEW |
| LEY Federal de Fomento a las Actividades Realizadas por Organizaciones de la Sociedad Civil | ley | `lffaosc` | NEW |
| REGLAMENTO de la Ley Federal de Fomento a las Actividades Realizadas por Organizaciones de la Sociedad Civil | reglamento | `reg-lffarosc` | NEW |

## Domain ED — Educación, ciencia & deporte

### Batch ED1 — `cl2_ED1_educacion` (6 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY General de Educación | ley | `lge` | NEW |
| LEY General de Educación Superior | ley | `lges` | NEW |
| LEY General del Sistema para la Carrera de las Maestras y los Maestros | ley | `lgscmm` | NEW |
| LEY Reglamentaria del Artículo 3o. de la Constitución Política de los Estados Unidos Mexicanos, en materia de Mejora Continua de la Educación | ley | `lrart3-mmce` | NEW |
| REGLAMENTO de la Ley General de la Infraestructura Física Educativa | reglamento | `reg-lgife` | NEW ⚠ |
| LEY General en materia de Humanidades, Ciencias, Tecnologías e Innovación | ley | `lgmhcti` | NEW |

- ⚠ `reg-lgife`: Parent Ley General de la Infraestructura Física Educativa abrogated; reglamento remains listed vigente.

### Batch ED2 — `cl2_ED2_profesiones_deporte_juventud` (6 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY Reglamentaria del Artículo 5o. Constitucional, relativo al ejercicio de las profesiones en la Ciudad de México | ley | `lrart5-prof` | NEW |
| REGLAMENTO de la Ley Reglamentaria del Artículo 5o. Constitucional, Relativo al Ejercicio de las Profesiones en el Distrito Federal | reglamento | `reg-lrart5c` | NEW |
| LEY General de Cultura Física y Deporte | ley | `lgcfd` | NEW |
| REGLAMENTO de la Ley General de Cultura Física y Deporte | reglamento | `reg-lgcfd` | NEW |
| LEY del Instituto Mexicano de la Juventud | ley | `limj` | NEW |
| LEY de Premios, Estímulos y Recompensas Civiles | ley | `lperc` | NEW |

### Batch ED3 — `cl2_ED3_universidades` (5 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY Orgánica de la Universidad Nacional Autónoma de México | ley | `lounam` | NEW |
| LEY Orgánica de la Universidad Autónoma Metropolitana | ley | `louam` | NEW |
| LEY Orgánica de la Universidad Autónoma Agraria Antonio Narro | ley | `louaaan` | NEW |
| LEY que crea la Universidad Autónoma Chapingo | ley | `luach` | NEW |
| LEY Orgánica del Instituto Politécnico Nacional | ley | `loipn` | NEW |

## Domain CU — Cultura & medios

### Batch CU1 — `cl2_CU1_patrimonio_cultura` (8 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY General de Cultura y Derechos Culturales | ley | `lgcdc` | NEW |
| REGLAMENTO de la Ley General de Cultura y Derechos Culturales | reglamento | `reg-lgcdc` | NEW |
| LEY Federal sobre Monumentos y Zonas Arqueológicos, Artísticos e Históricos | ley | `lfmzaah` | NEW |
| REGLAMENTO de la Ley Federal Sobre Monumentos y Zonas Arqueológicas, Artísticos e Históricos | reglamento | `reg-lfmzaah` | NEW |
| LEY Orgánica del Instituto Nacional de Antropología e Historia | ley | `loinah` | NEW |
| REGLAMENTO de la Ley Orgánica del Instituto Nacional de Antropología e Historia | reglamento | `reg-loinah` | NEW |
| LEY que crea el Instituto Nacional de Bellas Artes y Literatura | ley | `linbal` | NEW |
| LEY Orgánica del Seminario de Cultura Mexicana | ley | `loscm` | NEW |

### Batch CU2 — `cl2_CU2_cine_libro_bibliotecas` (7 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY Federal de Cine y el Audiovisual | ley | `lfca` | NEW |
| REGLAMENTO de la Ley Federal de Cinematografía | reglamento | `reg-lfcine` | NEW |
| LEY de Fomento para la Lectura y el Libro | ley | `lfll` | NEW |
| REGLAMENTO de la Ley de Fomento para la Lectura y el Libro | reglamento | `reg-lfll` | NEW |
| LEY General de Bibliotecas | ley | `lgb` | NEW |
| LEY General de Turismo | ley | `lgt` | NEW |
| REGLAMENTO de la Ley General de Turismo | reglamento | `reg-lgt` | NEW |

## Domain SA — Salud (extiende S1)

### Batch SA1 — `cl2_SA1_salud_especial` (6 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY General para el Control del Tabaco | ley | `lgct` | NEW |
| REGLAMENTO de la Ley General para el Control del Tabaco | reglamento | `reg-lgct` | NEW |
| LEY General para la Detección Oportuna del Cáncer en la Infancia y la Adolescencia | ley | `lgdocia` | NEW |
| REGLAMENTO de la Ley General de Salud en Materia de Control Sanitario de la Disposición de Órganos, Tejidos y Cadáveres de Seres Humanos | reglamento | `reg-lgs-mcsotcsh` | NEW |
| REGLAMENTO de la Ley General de Salud en Materia de Control Sanitario para la Producción, Investigación y Uso Medicinal de la Cannabis y sus Derivados Farmacológicos | reglamento | `reg-lgs-mcspiumc` | NEW |
| REGLAMENTO de la Ley General de Salud en Materia de Prestación de Servicios de Atención Médica | reglamento | `reg-lgs-mpsam` | NEW |

## Domain TEL — Telecomunicaciones & radiodifusión

### Batch TEL1 — `cl2_TEL1_telecom_radiodifusion` (4 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY en Materia de Telecomunicaciones y Radiodifusión | ley | `lmtr` | NEW |
| LEY del Sistema Público de Radiodifusión del Estado Mexicano | ley | `lsprem` | NEW |
| REGLAMENTO de la Ley Federal de Telecomunicaciones y Radiodifusión en Materia de Capacidad Satelital como Reserva del Estado | reglamento | `reg-lftr-mcsre` | NEW |
| REGLAMENTO de la Ley Federal de Radio y Televisión, en Materia de Concesiones, Permisos y Contenido de las Transmisiones de Radio y Televisión | reglamento | `reg-lfrt-mcpctrt` | NEW ⚠ |

- ⚠ `reg-lfrt-mcpctrt`: Parent Ley Federal de Radio y Televisión abrogated; reglamento remains listed vigente.

## Domain MIL — Fuerzas armadas

### Batch MIL1 — `cl2_MIL1_organica_justicia` (7 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY Orgánica del Ejército y Fuerza Aérea Mexicanos | ley | `loefam` | NEW |
| LEY Orgánica de la Armada de México | ley | `loam` | NEW |
| ORDENANZA General de la Armada | ordenanza | `oga` | NEW |
| CÓDIGO de Justicia Militar | codigo | `cjm` | NEW |
| CÓDIGO Militar de Procedimientos Penales | codigo | `cmpp` | NEW |
| LEY del Servicio Militar | ley | `lsm` | NEW |
| REGLAMENTO de la Ley del Servicio Militar | reglamento | `reg-lsm` | NEW |

### Batch MIL2 — `cl2_MIL2_carrera_educacion` (10 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY de Ascensos de la Armada de México | ley | `laam` | NEW |
| REGLAMENTO de la Ley de Ascensos de la Armada de México | reglamento | `reg-laam` | NEW |
| LEY de Ascensos y Recompensas del Ejército, Fuerza Aérea y Guardia Nacional | ley | `larefam` | NEW |
| REGLAMENTO de la Ley de Ascensos y Recompensas del Ejército, Fuerza Aérea y Guardia Nacional (Antes | reglamento | `reg-larefagn` | NEW |
| LEY de Disciplina del Ejército, Fuerza Aérea y Guardia Nacional | ley | `ldefam` | NEW |
| LEY de Disciplina para el Personal de la Armada de México | ley | `ldparm` | NEW |
| LEY de Educación Militar del Ejército, Fuerza Aérea y Guardia Nacional | ley | `lemefam` | NEW |
| REGLAMENTO de la Ley de Educación Militar del Ejército, Fuerza Aérea y Guardia Nacional (Antes | reglamento | `reg-lemefagn` | NEW |
| LEY de Educación Naval | ley | `len` | NEW |
| REGLAMENTO de la Ley de Educación Naval | reglamento | `reg-len` | NEW |

### Batch MIL3 — `cl2_MIL3_servicios_militares` (6 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY de Recompensas de la Armada de México | ley | `lram` | NEW |
| LEY para la Comprobación, Ajuste y Cómputo de Servicios de la Armada de México | ley | `lcacsam` | NEW |
| LEY para la Comprobación, Ajuste y Cómputo de Servicios en el Ejército y Fuerza Aérea Mexicanos | ley | `lcacsefam` | NEW |
| LEY del Instituto de Seguridad Social para las Fuerzas Armadas Mexicanas | ley | `lissfam` | NEW |
| REGLAMENTO de la Ley del Instituto de Seguridad Social para las Fuerzas Armadas Mexicanas | reglamento | `reg-lissfam` | NEW |
| LEY que Crea la Universidad del Ejército y Fuerza Aérea | ley | `luefa` | NEW |

## Domain AG — Agrario & rural

### Batch AG1 — `cl2_AG1_agrario` (6 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY Agraria | ley | `lagra` | NEW |
| REGLAMENTO de la Ley Agraria en Materia de Certificación de Derechos Ejidales y Titulación de Solares | reglamento | `reg-lagra-mcdets` | NEW |
| REGLAMENTO de la Ley Agraria en Materia de Ordenamiento de la Propiedad Rural | reglamento | `reg-lagra-mopr` | NEW |
| REGLAMENTO de la Ley Agraria para Fomentar la Organización y Desarrollo de la Mujer Campesina | reglamento | `reg-lagra-fodmc` | NEW |
| LEY Orgánica de los Tribunales Agrarios | ley | `lota` | NEW |
| LEY de Expropiación | ley | `lexp` | NEW |

### Batch AG2 — `cl2_AG2_desarrollo_rural` (7 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY de Desarrollo Rural Sustentable | ley | `ldrs` | NEW |
| REGLAMENTO de la Ley de Desarrollo Rural Sustentable en Materia de Organismos, Instancias de Representación, Sistemas y Servicios Especializados | reglamento | `reg-ldrs-moirsse` | NEW |
| LEY de Capitalización del Procampo | ley | `lcpro` | NEW |
| LEY de Energía para el Campo | ley | `lec` | NEW |
| REGLAMENTO de la Ley de Energía para el Campo | reglamento | `reg-lecampo` | NEW |
| LEY de Desarrollo Sustentable de la Cafeticultura | ley | `ldsc` | NEW |
| LEY de Desarrollo Sustentable de la Caña de Azúcar | ley | `ldsca` | NEW |

### Batch AG3 — `cl2_AG3_organizaciones_productos` (7 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY de Organizaciones Ganaderas | ley | `log` | NEW |
| REGLAMENTO de la Ley de Organizaciones Ganaderas | reglamento | `reg-logan` | NEW |
| LEY de Productos Orgánicos | ley | `lpo` | NEW |
| REGLAMENTO de la Ley de Productos Orgánicos | reglamento | `reg-lpo` | NEW |
| LEY sobre Cámaras Agrícolas, que en lo sucesivo se denominarán Asociaciones Agrícolas | ley | `lcaaa` | NEW |
| LEY Federal para el Fomento y Protección del Maíz Nativo | ley | `lffpmn` | NEW |
| LEY de Fomento a la Industria Vitivinícola | ley | `lfiv` | NEW |

### Batch AG4 — `cl2_AG4_sanidad_agroalimentaria` (8 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY Federal de Sanidad Animal | ley | `lfsa` | NEW |
| REGLAMENTO de la Ley Federal de Sanidad Animal | reglamento | `reg-lfsa` | NEW |
| LEY Federal de Sanidad Vegetal | ley | `lfsv` | NEW |
| REGLAMENTO de la Ley Federal de Sanidad Vegetal | reglamento | `reg-lfsv` | NEW |
| LEY Federal de Producción, Certificación y Comercio de Semillas | ley | `lfpccs` | NEW |
| REGLAMENTO de la Ley Federal de Producción, Certificación y Comercio de Semillas | reglamento | `reg-lfpccs` | NEW |
| LEY Federal de Variedades Vegetales | ley | `lfvv` | NEW |
| REGLAMENTO de la Ley Federal de Variedades Vegetales | reglamento | `reg-lfvv` | NEW |

## Domain EV — Ambiente (extiende E1–E3)

### Batch EV1 — `cl2_EV1_aguas_vida_silvestre` (4 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY General de Aguas | ley | `lgag` | NEW |
| REGLAMENTO de la Ley General de Vida Silvestre | reglamento | `reg-lgvs` | NEW |
| REGLAMENTO de la Ley General del Equilibrio Ecológico y la Protección al Ambiente en Materia de Ordenamiento Ecológico | reglamento | `reg-lgeepa-moe` | NEW |
| LEY General de Economía Circular | ley | `lgec` | NEW |

### Batch EV2 — `cl2_EV2_mar_pesca` (4 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY General de Pesca y Acuacultura Sustentables | ley | `lgpas` | NEW |
| REGLAMENTO de la Ley de Pesca | reglamento | `reg-lpesca` | NEW ⚠ |
| LEY Federal del Mar | ley | `lfm` | NEW |
| LEY de Vertimientos en las Zonas Marinas Mexicanas | ley | `lvzmm` | NEW |

- ⚠ `reg-lpesca`: Parent Ley de Pesca abrogated (now LGPAS); reglamento remains listed vigente.

## Domain EN — Energía (marco 2025)

### Batch EN1 — `cl2_EN1_electrico_planeacion` (7 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY del Sector Eléctrico | ley | `lse` | NEW |
| REGLAMENTO de la Ley del Sector Eléctrico | reglamento | `reg-lse` | NEW |
| LEY de Planeación y Transición Energética | ley | `lpte` | NEW |
| REGLAMENTO de la Ley de Planeación y Transición Energética | reglamento | `reg-lpte` | NEW |
| LEY de la Comisión Nacional de Energía | ley | `lcne` | NEW |
| LEY de la Empresa Pública del Estado, Comisión Federal de Electricidad | ley | `lepecfe` | NEW |
| REGLAMENTO de la Ley de la Empresa Pública del Estado, Comisión Federal de Electricidad | reglamento | `reg-lepecfe` | NEW |

### Batch EN2 — `cl2_EN2_hidrocarburos` (8 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY del Sector Hidrocarburos | ley | `lsh` | NEW |
| REGLAMENTO de la Ley del Sector Hidrocarburos | reglamento | `reg-lsh` | NEW |
| LEY de la Empresa Pública del Estado, Petróleos Mexicanos | ley | `lepepm` | NEW |
| REGLAMENTO de la Ley de la Empresa Pública del Estado, Petróleos Mexicanos | reglamento | `reg-lepepm` | NEW |
| LEY de Ingresos sobre Hidrocarburos | ley | `lih` | NEW |
| REGLAMENTO de la Ley de Ingresos sobre Hidrocarburos | reglamento | `reg-lih` | NEW |
| LEY del Fondo Mexicano del Petróleo para la Estabilización y el Desarrollo | ley | `lfmped` | NEW |
| LEY de la Agencia Nacional de Seguridad Industrial y de Protección al Medio Ambiente del Sector Hidrocarburos | ley | `lansi` | NEW |

### Batch EN3 — `cl2_EN3_renovables_nuclear` (7 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY de Biocombustibles | ley | `lbio` | NEW |
| REGLAMENTO de la Ley de Biocombustibles | reglamento | `reg-lbio` | NEW |
| LEY de Geotermia | ley | `lgeo` | NEW |
| REGLAMENTO de la Ley de Geotermia | reglamento | `reg-lgeo` | NEW |
| LEY Reglamentaria del Artículo 27 Constitucional en Materia Nuclear | ley | `lrart27-mn` | NEW |
| LEY de Responsabilidad Civil por Daños Nucleares | ley | `lrcdn` | NEW |
| REGLAMENTO de la Ley del Servicio Público de Energía Eléctrica, en Materia de Aportaciones | reglamento | `reg-lspee-ma` | NEW ⚠ |

- ⚠ `reg-lspee-ma`: Parent LSPEE abrogated (2025 electric framework); reglamento remains listed vigente.

## Domain MN — Minería

### Batch MN1 — `cl2_MN1_mineria` (4 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY de Minería | ley | `lmin` | NEW |
| REGLAMENTO de la Ley Minera | reglamento | `reg-lmin` | NEW |
| LEY que Declara Reservas Mineras Nacionales los Yacimientos de Uranio, Torio y las demás Substancias de las cuales se Obtengan Isótopos Hendibles que puedan Producir Energía Nuclear | ley | `lrmnyut` | NEW |
| REGLAMENTO de la Ley que Declara Reservas Mineras Nacionales los Yacimientos de Uranio, Torio y las demás Substancias de las cuales se Obtengan Isótopos Hendibles que puedan Producir Energía Nuclear | reglamento | `reg-lrmineras` | NEW |

## Domain TP — Transporte (extiende MI1–MI2)

### Batch TP1 — `cl2_TP1_ferroviario_aeroespacial` (4 instruments)

| Instrument | Type | Slug | Status |
|---|---|---|---|
| LEY Reglamentaria del Servicio Ferroviario | ley | `lrsf` | NEW |
| LEY de Protección del Espacio Aéreo Mexicano | ley | `lpeam` | NEW |
| REGLAMENTO de la Ley de Protección del Espacio Aéreo Mexicano | reglamento | `reg-lpeam` | NEW |
| LEY que crea la Agencia Espacial Mexicana | ley | `laem` | NEW |

---

**Total: 326 instruments in 53 batches.**

## Known gaps & carried-over blockers

- The **Reglamento de la Ley de Aeropuertos** is still absent from the
  official regley index (the index's `Reg_LAAT.pdf` is the Ley de Ayuda
  Alimentaria reglamento). Parent ley `LAERO` is in corpus; reglamento
  remains source-blocked.
- `reg-lgs-insumos` (Reglamento de Insumos para la Salud) is still absent
  from the regley index; remains source-blocked.
- The three financial DCGs flagged in cluster 1 (CUSF, CONSAR CUF,
  PLD art. 115 LIC) stay in the cluster-1 DCG register — DCGs are out of
  scope for this cluster.
- Cluster-1 blockers now resolved by this plan from the live indexes:
  `lga` → Ley General de Archivos (TR2) and `lgag` → Ley General de Aguas
  (EV1) are distinct instruments, both vigentes; `lftr` was replaced by the
  Ley en Materia de Telecomunicaciones y Radiodifusión (`lmtr`, TEL1);
  `lfce` is vigente (EC1); the 2025 energy framework is fully enumerated in
  EN1–EN3 with its reglamentos; transparency/data is enumerated in TR1–TR2.
