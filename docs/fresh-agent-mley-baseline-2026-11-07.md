Ensayo independiente realizado únicamente con contenido público y solicitudes de red. No consulté repositorio, archivos locales, memoria ni informes previos. **45 intentos de URL: 25 Lex-Mex y 20 mLey**, incluyendo fallos de transporte. No hubo respuestas 429. No realicé cambios ni comunicaciones externas.

Para abreviar las URL exactas de Lex-Mex, uso:

`B = https://www.leg4ll4bs.com/lex-mex/releases/c305db4a3cfb408b76c13431e26beda72d5ce665046c95783fab36507354c969/`

Las rutas siguientes son relativas a esa base, sin omisiones de caracteres.

**Descubrimiento y transporte**

- Lex-Mex: `https://www.leg4ll4bs.com/lex-mex/` falló dos veces con `web.run` (“not accessible via this tool”). Su HTML sí llegó mediante curl con red habilitada: HTTP 200, 7.827 bytes, 1,771 s. La portada enlazó `llms.txt`; este llevó al índice de publicación y a manifiesto. No ejecuté la interfaz JavaScript.
- mLey: `https://mley.mx/` llegó mediante `web.run`, con catálogo y enlaces públicos a `llms.txt` y `leyes.md`. Ambos enlaces fallaron en ese cliente por “Unsupported content-type: text/markdown”; curl sí los recuperó.
- Dos intentos iniciales con curl dentro del sandbox fallaron por DNS, uno por sitio; no son evidencia de indisponibilidad del servidor.
- Todos los Markdown consultados mediante red habilitada respondieron 200. Las mediciones de bytes son cuerpos descargados; los tiempos son los observados por curl o urllib, no mediciones comparables de rendimiento del producto.

**Respuestas por pregunta**

| Pregunta | Lex-Mex | mLey |
|---|---|---|
| **1. Instrumentos y alcance** | `B/index.md` enumera **DCG-IFPE-2021, DCG-ITF-2018, LIC y LRITF**. Declara cobertura parcial; no representa todo el derecho financiero ni toda la legislación mexicana. | Portada: catálogo de **320 leyes**, además de reglamentos, NOM/NMX, tesis, sentencias y DOF. Consulté `https://mley.mx/leyes.md` y `https://mley.mx/reglamentos.md`; el primero contiene LRITF, LIC y LRSIC. Su alcance es claramente más amplio. |
| **2. Autorización ITF y garantía estatal** | Art. 11: autorización de **CNBV, previo acuerdo del Comité Interinstitucional**. Gobierno Federal y entidades paraestatales no pueden garantizar los recursos de clientes ni asumir las obligaciones descritas. Texto completo recuperado en **L11**, abajo. | Misma respuesta, sustentada directamente en `https://mley.mx/LRITF/articulo/11/index.html.md`. Texto completo disponible, separado de comentarios autogenerados. |
| **3. Primer párrafo exacto IFPE art. 2 y consentimiento** | Texto completo recuperado en **F2** y versión `.txt`; transcripción debajo. Exige consentimiento **expreso**, que puede obtenerse mediante autenticación del art. 7. | No localicé estas disposiciones en los índices públicos consultados. No puedo transcribirlo ni responderlo desde mLey sin usar otra fuente. Esto no prueba ausencia en todo el sitio, particularmente en su cobertura DOF. |
| **4. Separación de recursos propios/clientes** | **LRITF art. 46, primer párrafo**, recuperado en **L46**. Exige recursos propios segregados y recursos de clientes identificados por cada cliente. | Misma respuesta desde `https://mley.mx/LRITF/articulo/46/index.html.md`. |
| **5. Cuenta bancaria individual o recursos agrupados** | Arts. 46–47, **L46/L47**: cuentas a nombre de la ITF distintas de sus recursos propios; recursos identificados por cliente y registro de titulares/saldos. El art. 46 contempla para IFPE “una o más cuentas”. **Estos textos no exigen una cuenta bancaria distinta por cliente ni permiten mezclar fondos propios y ajenos.** Una estructura agrupada debe conservar separación e identificación; no queda demostrada aquí la conformidad de una implementación concreta ni todas sus reglas complementarias. | Misma conclusión sustentada en `https://mley.mx/LRITF/articulo/46/index.html.md` y `https://mley.mx/LRITF/articulo/47/index.html.md`. Texto completo de ambos artículos recuperado. |
| **6. Seguir art. 2 → art. 7 y conservar evidencia** | El enlace expreso de **F2** lleva a **F7**. Debe recabarse y validarse identificador de cliente y, al menos, un factor de autenticación; identificador único asociado a sus operaciones. Se conserva evidencia conforme al art. 29 IV, cuyo enlace seguí a **F29**: fecha/hora, cuentas/operaciones, canal y factores utilizados, IP/teléfono u otros datos del canal; conservación segura mínima de **180 días naturales**, protección contra alteración y controles de acceso/disponibilidad. | No localicé la norma de origen; no pude recorrer esta cadena usando mLey. No completé la respuesta con el texto obtenido en Lex-Mex. |
| **7. Diferencia de objeto ITF2018/IFPE2021** | **T1** dice que ITF2018 regula la organización de las ITF y la operación de las instituciones de financiamiento colectivo. IFPE2021 se identifica por su título como disposiciones para IFPE vinculadas a arts. 48 segundo párrafo, 54 primero y 56 primero/segundo; **F2**, **F7** y LRITF48 muestran canales/autenticación y seguridad. **Resultado parcial:** no recuperé una cláusula general de objeto IFPE2021; no sostengo una comparación exhaustiva solo con su título. | No localicé ninguna de las dos DCG en los índices examinados; comparación no respondida desde este sitio. |
| **8. LIC art. 2: tipos y definición** | **C2**: banca múltiple y banca de desarrollo. Servicio: captación de recursos del público en mercado nacional para colocación en el público mediante actos que causan pasivo directo o contingente, debiendo el intermediario cubrir principal y, en su caso, accesorios financieros. Texto íntegro recuperado. | Misma respuesta desde `https://mley.mx/LIC/articulo/2o/index.html.md`; texto íntegro recuperado. |
| **9. Fuente/versión LRITF11; revisión hasta hoy** | **L11** identifica PDF de Cámara y SHA-256 `d6f645e6a7d3c2eeb46905d4d24ecd8e078907057dc034cda715abf019ce8491`. `B/manifest.json`: recuperación **2026-07-12T06:57:26.609663Z**, publicación identificada por el digest de B, `last_successful_upstream_check: null`, validez histórica no establecida. **No prueba revisión de reformas hasta 7 noviembre 2026.** | `https://mley.mx/LRITF/llms.txt`: actualización del registro **2026-10-12**, reforma DOF detectada **14-11-2025**. Art. 11 enlaza ficha, Word y PDF oficiales. La portada aclara que fechas de registros no son fechas de reforma y muestra ausencia de registros recientes de revisión oficial. **Tampoco prueba revisión hasta hoy.** |
| **10. LRSIC art. 27** | No aparece LRSIC en el catálogo de cuatro instrumentos. No recuperable dentro de la colección publicada examinada; es una limitación de cobertura, no un fallo del transporte de artículos. | Recuperado completo en `https://mley.mx/LRSIC/articulo/27/index.html.md`: las sociedades deben guardar secreto sobre identidad de acreedores al proporcionar información crediticia, salvo supuesto del art. 39, cuando informan directamente al cliente los nombres correspondientes. |

**Transcripción solicitada, IFPE2021 art. 2**

URL exacta:

`B/mx/ifpe-dcg-2021/provision-37403519a99ed630f31f9ed419457894d4ab5248b59de84c556340a7bf0d024f.txt`

> Las instituciones de fondos de pago electrónico, al pactar la celebración de Operaciones y la prestación de servicios a través de Canales de Instrucción, deberán requerir el consentimiento expreso de sus Clientes para dichos efectos, el cual se podrá obtener a través del proceso de Autenticación referido en el artículo 7 de las presentes Disposiciones. Adicionalmente, las instituciones de fondos de pago electrónico deberán:

Es transcripción del texto publicado en Lex-Mex; no cotejé visualmente el PDF oficial.

**Mapa exacto de artículos Lex-Mex consultados**

- **L11:** `B/mx/lritf/provision-295a570221f35ae7155d988fe6cd44333289ef8e9f63466561ea1e249aa78d35.md`
- **L46:** `B/mx/lritf/provision-b07e8a5d8b111b9a70552e2a84abb47220cc0b46c474e88522aa0cf57f9c3796.md`
- **L47:** `B/mx/lritf/provision-a770785655a8fef13357826f759079c3dbd9c2cc8f7d048aa138223fb2b57c41.md`
- **L48:** `B/mx/lritf/provision-dde55a8670e157ea1cc6f73e571e34601335cbb82540bd072b544855b4c3bc3c.md`
- **F1:** `B/mx/ifpe-dcg-2021/provision-e0d33f4df2b9d49a3f491bd8a751b8ff69c2adbc03b26a82b63f0dd6db2b62cf.md`
- **F2:** `B/mx/ifpe-dcg-2021/provision-37403519a99ed630f31f9ed419457894d4ab5248b59de84c556340a7bf0d024f.md`
- **F7:** `B/mx/ifpe-dcg-2021/provision-2795597b21312a9f5752bf7b1bbb7ab717dd6961d40efbb5b9799ca0c714a2a5.md`
- **F29:** `B/mx/ifpe-dcg-2021/provision-7c3f21270877d548fef41325114a0566a1d51165334f31e3c4d046958f326c38.md`
- **T1:** `B/mx/itf-dcg-2018/provision-18ff25a9e1c8d89f05ab4dd1d4c2521282b4d55a75716ff38251fb7c9c12c2ff.md`
- **C2:** `B/mx/lic/provision-e359d342262a572e36568f32f5ad9341cbb4fb854bb64d3940a89ab2c2fc4a7c.md`

**Fidelidad y límites observados**

Lex-Mex distingue texto fuente, referencias y clasificación temporal cualificada. La publicación permite citas reproducibles por digest e identifica huellas de fuentes. Sin embargo:

- **F7 contiene literalmente:** `I. El Identificador de Cliente El Identificador de Cliente y`. Es una duplicación visible cuya procedencia no pude verificar.
- L46 une en un mismo párrafo publicado dos bloques que mLey separa; no altera las palabras observadas, pero la estructura difiere.
- Intenté consultar el PDF retenido en `B/sources/493282f369e52da50db28c4777119591852a52313e5bb1cef82d1bd453899bb0.pdf`; respondió **403**. No hubo cotejo visual ni comprobación propia de su hash. El `pdf_verified: true` del manifiesto es una afirmación publicada por el sitio, no una verificación realizada en este ensayo.
- La ruta PDF anterior se resolvió desde el `pdf_path` del manifiesto; el 403 podría reflejar una resolución de ruta/base incorrecta, además de una restricción de acceso. No lo presento como ausencia definitiva de copia pública.

mLey presenta texto legal completo en bloques claramente separados, con enlaces a fuentes oficiales. Sus resúmenes heurísticos introducen ruido: LIC2 y LRSIC27 se clasifican como “redes, interconexión e infraestructura”, y LRITF11 como “concesiones y uso de recursos públicos”. Respondí desde **Texto del artículo**, sin usar esas clasificaciones como evidencia jurídica. No cotejé sus documentos oficiales.

**Mediciones observadas**

| Recurso | Bytes | Tiempo |
|---|---:|---:|
| Lex portada, curl | 7.827 | 1,771 s |
| Lex llms.txt | 935 | 0,424 s |
| Lex índice publicación | 260 | 0,422 s |
| Lex F2 Markdown / texto | 3.197 / 2.175 | 0,468 / 0,562 s |
| Lex F7 / F29 | 1.761 / 5.340 | 0,593 / 0,408 s |
| Lex manifiesto | 401.877 | 1,025 s, primera lectura |
| mLey llms.txt | 4.264 | 0,538 s |
| mLey leyes.md | 140.947 | 0,673 s, lectura filtrada |
| mLey reglamentos.md | 81.674 | 0,532 s, lectura filtrada |
| mLey LRITF11 | 5.906 | 0,385 s |
| mLey LRITF46 / 47 | 7.153 / 5.600 | 0,380 / 0,425 s |
| mLey LIC2 / LRSIC27 | 5.771 / 4.808 | 0,495 / 0,387 s |

Hubo lecturas redundantes de índices y manifiesto para recuperar salida truncada; están incluidas en los 45 intentos. No calculé bytes totales de transferencia porque algunos resultados iniciales quedaron truncados y `web.run` no expuso cuerpos/tiempos de red comparables.

**Conclusión comparativa**

Para estas preguntas financieras, Lex-Mex ofreció mejor acceso a la cadena IFPE2→7→29 y citas ligadas a una publicación identificada; mLey respondió las preguntas legales comunes y añadió LRSIC27 gracias a su mayor cobertura. Ambos permitieron recuperar textos completos mediante un cliente de red compatible. Los fallos del cliente web y el Markdown no admitido deben conservarse como fricción real, sin confundirlos con falta de contenido.

Ninguno acreditó comprobación de reformas hasta la fecha del ensayo. Este resultado corresponde a **un ensayo de un agente visitante**, no a aceptación multiproveedor, navegación orgánica o validación de producción.
