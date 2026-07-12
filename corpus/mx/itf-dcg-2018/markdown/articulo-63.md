---
id: urn:lex-mx:federal:regulation:itf-dcg-2018:article:63
instrument_id: urn:lex-mx:federal:regulation:itf-dcg-2018
instrument: DCG-ITF-2018
name: "Disposiciones de carácter general aplicables a las instituciones de tecnología financiera"
provision_type: article
number: "63"
aliases: ["DCG-ITF-2018 — Artículo 63"]
generated: true
temporal_status: effective
review_status: not_analyzed
amendment_marks: [2]
source_url: https://www.cnbv.gob.mx/Normatividad/Disposiciones%20de%20car%C3%A1cter%20general%20aplicables%20a%20las%20instituciones%20de%20tecnolog%C3%ADa%20financiera.pdf
source_sha256: 71787505c1b264e4e329816c009c4c1cf210f4e8889119412f83f6b1853998a2
---

> Título TERCERO · Capítulo VI

# Artículo 63

El director general o, en su caso, el administrador único de la institución de financiamiento colectivo, será responsable de la implementación de los controles internos en materia de seguridad de la información que procure su confidencialidad, integridad y disponibilidad. El marco de gestión a que se refiere este párrafo, deberá asegurar que la [Infraestructura Tecnológica](../../lritf/markdown/articulo-4.md) de dicha institución, ya sea propia o provista por terceros, se apegue a los requerimientos siguientes:

I. Que cada uno de sus componentes realice las funciones para las que fue diseñado, desarrollado o adquirido.

II. Que sus procesos, funcionalidades y configuraciones, incluyendo su metodología de desarrollo o adquisición, así como el registro de sus cambios, actualizaciones y el inventario detallado de cada componente de la Infraestructura Tecnológica, estén documentados.

III. Que se hayan considerado aspectos de seguridad de la información en la definición de proyectos para adquirir o desarrollar cada uno de sus componentes, debiendo incluirlos durante las diversas etapas del ciclo de vida. Este comprenderá la elaboración de requerimientos, diseño, desarrollo o adquisición, pruebas de implementación, pruebas de aceptación por parte de los Usuarios de la Infraestructura Tecnológica, procesos de liberación incluyendo pruebas de vulnerabilidades y análisis de código previos a su puesta en producción, pruebas periódicas, gestión de cambios, reemplazo y destrucción de información.

Tratándose de componentes de comunicaciones y de cómputo, los aspectos de seguridad deberán incluir, al menos, lo siguiente:

a) Segregación lógica, o lógica y física de las diferentes redes en distintos dominios y subredes, dependiendo de la función que desarrollen o el tipo de datos que se transmitan, incluyendo segregación de los ambientes productivos de los de desarrollo y pruebas, así como componentes de seguridad perimetral y de redes que aseguren que solamente el tráfico autorizado es permitido. En particular, en aquellos segmentos con enlaces al exterior, tales como Internet, proveedores, autoridades, otras redes de la institución de financiamiento colectivo o matriz y otros terceros, todo ello referido a aquellos servicios definidos como críticos por la propia institución, ya sean sistemas de pagos, equipos de [Cifrado](articulo-2.md), autorizadores de [Operaciones](../../lritf/markdown/articulo-4.md), entre otros, deberán considerar zonas seguras, incluyendo las denominadas zonas desmilitarizadas (DMZ por sus siglas en inglés).

b) Configuración segura de acuerdo con el tipo de componente, considerando al menos, puertos y servicios, permisos otorgados bajo el principio de mínimo privilegio, uso de medios extraíbles de almacenamiento, listas de acceso, actualizaciones del fabricante y reconfiguración de parámetros de fábrica. Se entenderá como principio de mínimo privilegio a la habilitación del acceso únicamente a la información y recursos necesarios para el desarrollo de las funciones propias de cada [Usuario de la Infraestructura Tecnológica](articulo-2.md).

c) Mecanismos de seguridad en las aplicaciones que procuren que, durante su ejecución se protejan de ataques o intrusiones, tales como inyección de código, manipulación de la sesión, fuga de información, alteración de privilegios de acceso, entre otros. Dichos mecanismos deberán de ser implementados tanto para las aplicaciones proporcionadas por terceros como para las aplicaciones desarrolladas, implementadas y mantenidas por la propia institución de financiamiento colectivo.

IV. Que cada uno de sus componentes sea probado antes de ser implementado o modificado, utilizando mecanismos de control de calidad que eviten que en dichas pruebas se utilicen datos reales del ambiente de producción, se revele información confidencial o de seguridad o se introduzca cualquier funcionalidad no reconocida para dicho componente.

V. Que cuente con las licencias o autorizaciones de uso, en su caso.

VI. Que cuente con medidas de seguridad para su protección, así como para el acceso y uso de la información que sea recibida, generada, transmitida, almacenada y procesada en la propia Infraestructura Tecnológica contando, al menos, con lo siguiente:

a) Mecanismos de identificación y [Autenticación](articulo-2.md) de todos y cada uno de los Usuarios de la Infraestructura Tecnológica, que permitan reconocerlos de forma inequívoca y aseguren el acceso únicamente a las personas autorizadas expresamente para ello, bajo el principio de mínimo privilegio.

Para lo anterior, se deberán incluir controles pertinentes para aquellos Usuarios de la Infraestructura Tecnológica con mayores privilegios, derivados de sus funciones, tales como la de administración de bases de datos, sistemas operativos y aplicativos.

Asimismo, se deberán prever en manuales las políticas y procedimientos para las autorizaciones de accesos por excepción, tales como usuarios de ambientes de desarrollo con acceso a ambientes de producción y con accesos por eventos de contingencia, entre otros. Dichas políticas y procedimientos deberán ser aprobados por el oficial en jefe de seguridad de la información.

b) Cifrado de la información conforme al grado de sensibilidad o clasificación de la información que la institución de financiamiento colectivo determine y establezca en sus políticas, cuando dicha información sea transmitida, intercambiada y comunicada entre componentes o almacenada en la Infraestructura Tecnológica o se acceda de forma remota.

Las instituciones de financiamiento colectivo deberán cifrar al menos, la información que hayan clasificado como crítica en términos de estas disposiciones.

c) Claves de acceso con características de composición que eviten accesos no autorizados, considerando procesos que aseguren que solo el Usuario de la Infraestructura Tecnológica sea quien las conozca, así como medidas de seguridad, Cifrado en su almacenamiento y mecanismos para solicitar el cambio de claves de acceso cada noventa días o menos. Tratándose de [Clientes](../../lritf/markdown/articulo-4.md), el plazo referido será el definido por las propias instituciones de financiamiento colectivo en los manuales a que alude el último párrafo de este artículo. En el caso de los Usuarios de la Infraestructura Tecnológica asignados a aplicativos o componentes para autenticarse entre ellos, el cambio a que alude este inciso deberá realizarse, al menos, una vez al año. En el evento de que algún Usuario de la Infraestructura Tecnológica tenga conocimiento de las claves de acceso y deje de prestar sus servicios a la institución de financiamiento colectivo, estas deberán inhabilitarse de manera inmediata.

d) Controles para terminar automáticamente sesiones no atendidas, así como para evitar sesiones simultáneas no permitidas con un mismo identificador de Usuario de la Infraestructura Tecnológica.

e) Mecanismos de seguridad, tanto de acceso físico, como de controles ambientales y de energía eléctrica, que protejan la Infraestructura Tecnológica y permitan la operación conforme a las especificaciones del proveedor, fabricante o desarrollador.

f) Medidas de validación para garantizar la autenticidad de las transacciones ejecutadas por los diferentes componentes de la Infraestructura Tecnológica considerando, al menos, lo siguiente:

1. La veracidad e integridad de la información.

2. La Autenticación entre componentes de la Infraestructura Tecnológica, que aseguren que se ejecutan solo las solicitudes de servicio legítimas desde su origen y hasta su ejecución y registro.

3. Los protocolos de mensajería, comunicaciones y Cifrado, los cuales deben procurar la integridad y confidencialidad de la información.

4. La identificación de transacciones atípicas, previendo que se cuenten con herramientas de monitoreo o medidas de alerta automática para su atención por las áreas operativas correspondientes.

5. La actualización y mantenimiento de certificados digitales y componentes proporcionados por proveedores de servicios que estén integrados al proceso de ejecución de transacciones.

Las medidas a que alude este inciso deberán establecerse acorde con el grado de riesgo que las instituciones de financiamiento colectivo definan para cada tipo de transacción.

Las instituciones de financiamiento colectivo, en la clasificación de la información a que alude el inciso b) de esta fracción, deberán considerar al menos una categoría referente a la información crítica. En dicha categoría deberán incluir como mínimo la [Información Sensible](articulo-2.md) y las imágenes de identificaciones oficiales e información biométrica de los Clientes, así como cualquier otra que determinen de acuerdo con sus políticas.

VII. Que cuente con mecanismos de respaldo y procedimientos de recuperación de la información que mitiguen el riesgo de interrupción de la operación, en concordancia con lo estipulado en su [Plan de Continuidad de Negocio](articulo-2.md) a que alude el Capítulo V del Título Tercero de las presentes disposiciones.

VIII. Que mantenga registros de auditoría íntegros, incluyendo la información detallada de los accesos o intentos de acceso y la operación o actividad efectuada por los Usuarios de la Infraestructura Tecnológica, lo anterior con independencia del nivel de privilegios con el que estos cuenten para el acceso, generación o modificación de la información que reciban, generen, almacenen o transmitan en cada componente de la Infraestructura Tecnológica, incluyendo actividad de procesos automatizados, así como los procedimientos para la revisión periódica de dichos registros.

Las instituciones de financiamiento colectivo deberán conservar los registros de auditoría a que se refiere esta fracción, por un periodo de tres años cuando dichos registros se refieran a actividades realizadas sobre componentes que procesen o almacenen información considerada como crítica de conformidad con la clasificación que determine la institución de financiamiento colectivo. En caso contrario, el periodo de conservación de los registros será mínimo de seis meses.

IX. Que para la atención de los Eventos de Seguridad de la Información e Incidentes de Seguridad de la Información se cuente con procesos de gestión que aseguren la detección, clasificación, atención y contención, investigación y, en su caso, análisis forense digital, diagnóstico, reporte a áreas competentes, solución, seguimiento y comunicación a autoridades, Clientes y contrapartes.

Para la detección y respuesta de Incidentes de Seguridad de la Información a que hace referencia el párrafo anterior, el director general o, en su caso, el administrador único deberá designar un equipo que incorpore al personal de las diferentes áreas de la institución de financiamiento colectivo para participar en cada actividad del proceso de gestión antes señalado del que, en todo caso, deberá formar parte el oficial en jefe de seguridad de la información de conformidad con el artículo [66](articulo-66.md) de las presentes disposiciones.

En caso de que se detecte la existencia de vulnerabilidades y deficiencias en la Infraestructura Tecnológica, deberán tomarse las acciones correctivas o controles compensatorios de acuerdo con el nivel de riesgo de que se trate, previniendo que los Usuarios de la Infraestructura Tecnológica o la institución de financiamiento colectivo puedan verse afectados.

X. Que sea sometida a la realización de ejercicios de planeación y revisión anuales que permitan medir su capacidad para soportar su operación, garantizando que se atiendan oportunamente las necesidades de incremento de capacidad detectadas como resultado de dichos ejercicios.

Asimismo, la institución de financiamiento colectivo deberá evaluar la obsolescencia de los componentes de la Infraestructura Tecnológica, debiendo contar con un plan para su actualización.

XI. Que cuente con controles automatizados o, en ausencia de estos, que se realicen controles compensatorios, tales como doble verificación y conciliación que, previo o posteriormente a la realización de la operación de que se trate, minimicen el riesgo de eliminación, exposición, alteración o modificación de información, que se deriven de procesos manuales o semiautomatizados realizados por el personal de la institución de financiamiento colectivo, con el objetivo de prevenir errores, omisiones, sustracción o manipulación de información.

XII. Que tenga controles que permitan detectar la alteración o falsificación de libros, registros y documentos digitales relativos a las Operaciones.

XIII. Que cuente con procesos para medir y asegurar los niveles de disponibilidad y tiempos de respuesta, que garanticen la ejecución de las Operaciones realizadas; lo anterior, incluyendo los supuestos en que las instituciones de financiamiento colectivo contraten la prestación de servicios por parte de terceros para el procesamiento y almacenamiento de información.

XIV. Que cuente con dispositivos o mecanismos automatizados para detectar y prevenir Eventos de Seguridad de la Información e Incidentes de Seguridad de la Información, así como para evitar conexiones y flujos de datos entrantes o salientes no autorizados y fuga de información considerando, entre otros, medios de almacenamiento removibles.

Las instituciones de financiamiento colectivo deberán correlacionar los datos obtenidos de los dispositivos o mecanismos automatizados a que alude el párrafo anterior con los datos de otras fuentes, tales como registros de actividad de Eventos de Seguridad de la Información o de Incidentes de Seguridad de la Información.

Adicionalmente, las instituciones de financiamiento colectivo deberán mantener controles que eviten la filtración de la información correspondiente a la configuración de la Infraestructura Tecnológica, tales como direcciones IP, reglas de los cortafuegos, así como versiones de hardware y software.

XV. Que para la prestación de servicios de tecnologías de información a los Usuarios de la Infraestructura Tecnológica, en sus fases de estrategia, diseño, transición, operación y mejora continua, se proteja la integridad de la Infraestructura Tecnológica, así como la integridad, confidencialidad y disponibilidad de la información recibida, generada, procesada, almacenada y transmitida por esta. El director general o, en su caso, el administrador único de la institución de financiamiento colectivo será responsable de documentar en manuales las políticas y procedimientos previstas en este artículo.

El director general o, en su caso, el administrador único de la institución de financiamiento colectivo será responsable de documentar en manuales las políticas y procedimientos previstas en este artículo.
