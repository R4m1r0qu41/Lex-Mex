# Consulta Lex-Mex: Q01 a Q10

**Fecha de consulta:** 29 de octubre de 2026
**Colección:** Lex-Mex (leg4ll4bs.com/lex-mex), *release* fijada `c305db4a3cfb408b76c13431e26beda72d5ce665046c95783fab36507354c969`
**Base de rutas:** `https://www.leg4ll4bs.com/lex-mex/releases/c305db4a3cfb408b76c13431e26beda72d5ce665046c95783fab36507354c969/`

## Alcance de la consulta

No hubo bloqueos de acceso. Leí únicamente los índices, `coverage.json`, una parte de `manifest.json` (la respuesta llegó truncada) y los artículos citados abajo. No descargué ningún `instrument.json` completo ni el corpus entero.

En cada pregunta separo dos cosas:

- **Cita literal:** texto tal como aparece en el corpus.
- **Explicación:** interpretación propia.

---

## Q01. Instrumentos disponibles y qué queda fuera

**Rutas consultadas:** `index.md`, `coverage.json`, `/lex-mex/llms.txt`

**Explicación.** El índice de la *release* lista cuatro instrumentos:

- DCG-IFPE-2021
- DCG-ITF-2018
- LIC
- LRITF

`coverage.json` confirma esos cuatro con `"scope": "selected_partial"`. Para cada uno marca `"historical_validity": "not_established"` y advierte que algunas remisiones apuntan a destinos no resueltos (`"references": "express_with_disclosed_unresolved_targets"`).

Cualquier otra norma queda fuera: otras leyes financieras, circulares de Banco de México, disposiciones de prevención de lavado de dinero (PLD) para Instituciones de Tecnología Financiera (ITF), etcétera. También quedan fuera las versiones históricas.

---

## Q02. Quién autoriza a una ITF y si el Gobierno garantiza el dinero de sus clientes

**Instrumento y artículo:** LRITF, artículo 11
**Ruta:** `mx/lritf/provision-295a570221f35ae7155d988fe6cd44333289ef8e9f63466561ea1e249aa78d35.md`

**Cita literal (párrafo primero):**

> "Para organizarse y operar como ITF se requiere obtener una autorización que será otorgada por la CNBV, previo acuerdo del Comité Interinstitucional, en términos del Capítulo I del Título III de la presente Ley."

**Cita literal (párrafo tercero, primera oración):**

> "Ni el Gobierno Federal ni las entidades de la administración pública paraestatal podrán responsabilizarse o garantizar los recursos de los Clientes que sean utilizados en las Operaciones que celebren con las ITF o frente a otros, así como tampoco asumir alguna responsabilidad por las obligaciones contraídas por las ITF o por algún Cliente frente a otro, en virtud de las Operaciones que celebren."

**Explicación.** La autorización la otorga la Comisión Nacional Bancaria y de Valores (CNBV), previo acuerdo del Comité Interinstitucional. El Gobierno no garantiza los recursos de los clientes. Además, el mismo párrafo obliga a las ITF a divulgar esa ausencia de garantía en su página de internet, sus aplicaciones, su publicidad y sus contratos.

---

## Q03. Primer párrafo del artículo 2 de las DCG IFPE 2021 y consentimiento exigido

**Instrumento y artículo:** DCG IFPE 2021, artículo 2
**Ruta:** `mx/ifpe-dcg-2021/provision-37403519a99ed630f31f9ed419457894d4ab5248b59de84c556340a7bf0d024f.md`

**Cita literal:**

> "Las instituciones de fondos de pago electrónico, al pactar la celebración de Operaciones y la prestación de servicios a través de Canales de Instrucción, deberán requerir el consentimiento expreso de sus Clientes para dichos efectos, el cual se podrá obtener a través del proceso de Autenticación referido en el artículo 7 de las presentes Disposiciones. Adicionalmente, las instituciones de fondos de pago electrónico deberán:"

**Explicación.** Se exige consentimiento **expreso** del Cliente para operar por Canales de Instrucción. La norma permite ("se podrá") obtenerlo mediante la Autenticación del artículo 7, pero no lo impone como única vía.

---

## Q04. Dónde exige la LRITF separar el dinero de los clientes de los recursos propios

**Rutas consultadas:**

- LRITF artículo 11: `mx/lritf/provision-295a5702….md`
- LRITF artículo 22: `mx/lritf/provision-4716a7f96f3961aac91981a23d57c2176d57b537102107a6ae7d89bd3a9433bb.md`
- LRITF artículo 48: `mx/lritf/provision-dde55a8670e157ea1cc6f73e571e34601335cbb82540bd072b544855b4c3bc3c.md`

**Resultado: no localicé ese pasaje en los artículos consultados.**

**Explicación.** El artículo 48 menciona "segregación de funciones", que es una materia que la CNBV regula mediante disposiciones generales. Eso no es una obligación de separar el dinero de los clientes del patrimonio de la ITF, y no debe leerse así.

No afirmo que la LRITF carezca de esa regla: solo que no la encontré en los artículos leídos. Para responder con certeza habría que buscar en el texto completo de `mx/lritf/instrument.json`, lo que implica descargar el instrumento entero.

---

## Q05. ¿Una IFPE debe abrir una cuenta bancaria por cliente o puede concentrar recursos?

**Rutas consultadas:**

- LRITF artículo 22: `mx/lritf/provision-4716a7f9….md`
- DCG IFPE 2021 artículo 1: `mx/ifpe-dcg-2021/provision-e0d33f4df2b9d49a3f491bd8a751b8ff69c2adbc03b26a82b63f0dd6db2b62cf.md`

**Cita literal (LRITF, artículo 22, fracción I):**

> "Abrir y llevar una o más cuentas de fondos de pago electrónico por cada Cliente…"

**Cita literal (DCG IFPE 2021, artículo 1, definición de "Cuenta", inicio):**

> "Cuenta: a aquel registro contable en el que la institución de fondos de pago electrónico realiza, entre otros, las anotaciones de…"

**Qué permite concluir el corpus.** Lo que se exige "por cada Cliente" es una cuenta de fondos de pago electrónico, es decir, un registro en los libros de la Institución de Fondos de Pago Electrónico (IFPE). No es una cuenta bancaria.

**Qué no permite concluir.** En lo consultado no hay norma que exija, permita o prohíba concentrar el dinero de los clientes en una o varias cuentas bancarias. Este punto depende de lo que resulte en Q04 y de normas que podrían estar fuera de la colección, como la regulación de Banco de México.

---

## Q06. Remisión del artículo 2 al 7: qué debe recabarse y qué evidencia conservarse

**Rutas consultadas:**

- DCG IFPE 2021 artículo 7: `mx/ifpe-dcg-2021/provision-2795597b21312a9f5752bf7b1bbb7ab717dd6961d40efbb5b9799ca0c714a2a5.md`
- DCG IFPE 2021 artículo 29: `mx/ifpe-dcg-2021/provision-7c3f21270877d548fef41325114a0566a1d51165334f31e3c4d046958f326c38.md`

**Cita literal (artículo 7):**

> "…deberán recabar y validar, al menos, lo siguiente: I. El Identificador de Cliente El Identificador de Cliente y II. Un Factor de Autenticación."

La repetición "El Identificador de Cliente El Identificador de Cliente" aparece así en la copia del corpus. Parece un artefacto de extracción del PDF y no la corregí.

> "El Identificador de Cliente deberá ser único para cada Cliente y deberá asociarse a todas las Operaciones realizadas por este último."

> "Asimismo, las instituciones de fondos de pago electrónico deberán guardar evidencia de la Autenticación, conforme a lo establecido en el artículo 29, fracción IV de las presentes Disposiciones."

**Explicación de la remisión al artículo 29, fracción IV.** Esa fracción exige registros de los accesos y actividades de los Clientes que incluyan, al menos, cuatro datos:

1. La fecha, hora, minuto y segundo de la actividad.
2. Las Cuentas involucradas: la del ordenante y, en su caso, las de los receptores.
3. El Canal de Instrucción utilizado **y los Factores de Autenticación** empleados.
4. Las direcciones IP, el número de línea telefónica o datos equivalentes.

Esos registros deben conservarse de forma segura por **al menos 180 días naturales** a partir de su generación, con mecanismos que impidan su alteración. Si el Cliente lo solicita, se le deben entregar en un plazo máximo de 10 días hábiles, siempre que se trate de operaciones en sus propias Cuentas durante los 180 días previos a la solicitud.

---

## Q07. ¿Las DCG ITF 2018 y las DCG IFPE 2021 tienen el mismo objeto?

**Respuesta: no.**

**Rutas consultadas:**

- DCG ITF 2018 artículo 1: `mx/itf-dcg-2018/provision-18ff25a9e1c8d89f05ab4dd1d4c2521282b4d55a75716ff38251fb7c9c12c2ff.md`
- DCG IFPE 2021 artículo 1: `mx/ifpe-dcg-2021/provision-e0d33f4d….md`
- Índice de las DCG IFPE 2021, para el título del instrumento: `mx/ifpe-dcg-2021/index.md`

**Cita literal (DCG ITF 2018, artículo 1):**

> "Las presentes disposiciones tienen por objeto establecer el marco normativo aplicable a la organización de las ITF y la operación de las instituciones de financiamiento colectivo."

**Explicación.** El artículo 1 de las DCG IFPE 2021 no declara un objeto: es un artículo de definiciones. Su alcance se infiere del título del instrumento, que remite a los artículos 48, segundo párrafo; 54 y 56 de la LRITF. El artículo 48, segundo párrafo, de la LRITF se refiere a seguridad de la información, registro de cuentas y continuidad operativa de las IFPE. Lo que leí del instrumento es consistente con eso: Canales de Instrucción, Autenticación e Infraestructura Tecnológica.

**Diferencia concreta.** Las DCG ITF 2018 regulan la *operación* solo de las instituciones de financiamiento colectivo. Las DCG IFPE 2021 se dirigen exclusivamente a las IFPE.

---

## Q08. LIC, artículo 2: tipos de instituciones de crédito y servicio de banca y crédito

**Instrumento y artículo:** LIC, artículo 2o
**Ruta:** `mx/lic/provision-e359d342262a572e36568f32f5ad9341cbb4fb854bb64d3940a89ab2c2fc4a7c.md`

**Cita literal:**

> "El servicio de banca y crédito sólo podrá prestarse por instituciones de crédito, que podrán ser: I. Instituciones de banca múltiple, y II. Instituciones de banca de desarrollo."

> "Para efectos de lo dispuesto en la presente Ley, se considera servicio de banca y crédito la captación de recursos del público en el mercado nacional para su colocación en el público, mediante actos causantes de pasivo directo o contingente, quedando el intermediario obligado a cubrir el principal y, en su caso, los accesorios financieros de los recursos captados."

---

## Q09. Fuente y versión del artículo 11 de la LRITF

**Rutas consultadas:**

- `mx/lritf/provision-295a5702….md`
- `coverage.json`
- `manifest.json`
- `/lex-mex/llms.txt`

**Respaldo de la cita:**

| Campo | Valor |
|---|---|
| PDF fuente | `https://www.diputados.gob.mx/LeyesBiblio/pdf/LRITF.pdf` |
| Huella SHA-256 del PDF | `d6f645e6a7d3c2eeb46905d4d24ecd8e078907057dc034cda715abf019ce8491` |
| `retrieved_at` | 2026-07-12T06:57:26Z |
| `pdf_verified` | true |
| *Release* | `c305db4a…`, `committed_at` 2026-10-24 |
| Estado del artículo | `"effective"`, calificado como "qualified repository classification, not a guarantee of legal validity" |
| `last_successful_upstream_check` | null |
| `historical_validity` | `not_established` |

**¿Prueba que se verificaron reformas hasta hoy? No.**

El `llms.txt` dice textualmente:

> "No last-successful-upstream-check ledger or historical validity guarantee is available."

**Explicación.** Lo único que se demuestra es que el texto corresponde al PDF descargado el 12 de julio de 2026. Cualquier reforma posterior tendría que verificarse en el Diario Oficial de la Federación (DOF), lo cual queda fuera de esta colección.

---

## Q10. Ley para Regular las Sociedades de Información Crediticia, artículo 27

**Resultado: no está en la colección.**

**Rutas consultadas:** `index.md`, `coverage.json`

**Explicación.** Ambos archivos listan solo los cuatro instrumentos de Q01, así que no hay ruta que consultar para esta ley.

Lo único que se puede concluir es que esta colección no permite responder la pregunta. La ausencia no dice nada sobre el contenido ni sobre la vigencia de ese artículo, y no lo completé con memoria ni con otras fuentes.
