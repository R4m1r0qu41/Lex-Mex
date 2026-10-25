---
id: urn:lex-mx:federal:regulation:cub-dcg-2005:article:168-bis-11
instrument_id: urn:lex-mx:federal:regulation:cub-dcg-2005
instrument: DCG-CUB-2005
name: "Disposiciones de carácter general aplicables a las instituciones de crédito"
provision_type: article
number: "168 Bis 11"
aliases: ["DCG-CUB-2005 — Artículo 168 Bis 11"]
generated: true
temporal_status: effective
review_status: machine_accepted
amendment_marks: [260,346]
source_url: https://www.cnbv.gob.mx/Normatividad/Disposiciones%20de%20car%C3%A1cter%20general%20aplicables%20a%20las%20instituciones%20de%20cr%C3%A9dito.pdf
source_sha256: b03eb544f3bc5b71a181bbd71f5a6024209a4873cff0da309b43ff8279cc2836
---

> Título SEGUNDO · Capítulo VI

# Artículo 168 Bis 11

La persona titular de la [Dirección General](articulo-1.md) de la Institución será responsable de la implementación del [Sistema de Control Interno](articulo-1.md) en materia de seguridad de la información que procure su confidencialidad, integridad y disponibilidad. El marco de gestión a que se refiere este artículo deberá asegurar que la [Infraestructura Tecnológica](articulo-1.md), propia o provista por terceros, se apegue a los requerimientos siguientes:

I. Que cada uno de sus componentes realice las funciones para las que fue diseñado, desarrollado o adquirido.

II. Que sus procesos, funcionalidades y configuraciones, incluyendo su metodología de desarrollo o adquisición, así como el registro de sus cambios, actualizaciones y el inventario detallado de cada componente de la Infraestructura Tecnológica, estén documentados.

III. Que se hayan considerado aspectos de seguridad de la información en la definición de proyectos para adquirir o desarrollar cada uno de sus componentes, debiendo incluirlos durante las diversas etapas del ciclo de vida. Este comprenderá la elaboración de requerimientos, diseño, desarrollo o adquisición, pruebas de implementación, pruebas de aceptación por parte de los [Usuarios](articulo-1.md) de la Infraestructura Tecnológica, procesos de liberación incluyendo pruebas de vulnerabilidades y análisis de código previos a su puesta en producción, pruebas periódicas, gestión de cambios, reemplazo y destrucción de información.

Tratándose de componentes de comunicaciones y de cómputo, los aspectos de seguridad deberán incluir, al menos, lo siguiente:

a) Segregación lógica, o lógica y física de las diferentes redes en distintos dominios y sub redes, dependiendo de la función que desarrollen o el tipo de datos que se transmitan, incluyendo segregación de los ambientes productivos de los de desarrollo y pruebas, así como componentes de seguridad perimetral y de redes que aseguren que solamente el tráfico autorizado es permitido. En particular, en aquellos segmentos con enlaces al exterior, tales como Internet, proveedores, autoridades, otras redes de la Institución o matriz y otros terceros, todo ello referido a servicios críticos, ya sean sistemas de pagos, equipos de cifrado, autorizadores de operaciones, entre otros, considerar zonas seguras, incluyendo las denominadas zonas desmilitarizadas (DMZ por sus siglas en inglés).

b) Configuración segura de acuerdo con el tipo de componente, considerando al menos, puertos y servicios, permisos otorgados bajo el principio de mínimo privilegio, uso de medios extraíbles de almacenamiento, listas de acceso, actualizaciones del fabricante y reconfiguración de parámetros de fábrica. Se entenderá como principio de mínimo privilegio a la habilitación del acceso únicamente a la información y recursos necesarios para el desarrollo de las funciones propias de cada [Usuario de la Infraestructura Tecnológica](articulo-1.md).

IV. Que cada uno de sus componentes sea probado antes de ser implementado o modificado, utilizando mecanismos de control de calidad que eviten que en dichas pruebas se utilicen datos reales del ambiente de producción, se revele información confidencial o de seguridad, o se introduzca cualquier funcionalidad no reconocida para dicho componente.

V. Que cuente con las licencias o autorizaciones de uso, en su caso.

VI. Que cuente con medidas de seguridad para su protección, así como para el acceso y uso de la información que sea recibida, generada, transmitida, almacenada y procesada en la propia Infraestructura Tecnológica, contando al menos con lo siguiente:

a) Mecanismos de identificación y [Autenticación](articulo-1.md) de todos y cada uno de los Usuarios de la Infraestructura Tecnológica, que permitan reconocerlos de forma inequívoca y aseguren el acceso únicamente a las personas autorizadas expresamente para ello, bajo el principio de mínimo privilegio.

Para lo anterior, se deberán incluir controles pertinentes para aquellos Usuarios de la Infraestructura Tecnológica con mayores privilegios, derivados de sus funciones, tales como, la de administración de bases de datos y de sistemas operativos.

Asimismo, se deberán prever políticas y procedimientos para las autorizaciones de accesos por excepción, tales como usuarios de ambientes de desarrollo con acceso a ambientes de producción y con acceso por eventos de contingencia, entre otros. Dichas políticas y procedimientos deberán ser aprobados por el oficial en jefe de seguridad de la información.

b) Cifrado de la información conforme al grado de sensibilidad o clasificación que la Institución determine y establezca en sus políticas, cuando dicha información sea transmitida, intercambiada y comunicada entre componentes, o almacenada en la Infraestructura Tecnológica o se acceda de forma remota.

c) Claves de acceso con características de composición que eviten accesos no autorizados, considerando procesos que aseguren que solo el Usuario de la Infraestructura Tecnológica sea quien las conozca, así como medidas de seguridad, cifrado en su almacenamiento y mecanismos para cambiar las claves de acceso cada 90 días o menos. En el caso de los Usuarios de la Infraestructura Tecnológica asignados a aplicativos o componentes para autenticarse entre ellos, el cambio a que alude este inciso deberá realizarse al menos una vez al año. En el evento de que algún Usuario de la Infraestructura Tecnológica tenga conocimiento de las claves de acceso y deje de prestar sus servicios a la Institución, estas deberán modificarse de manera inmediata.

d) Controles para terminar automáticamente sesiones no atendidas, así como para evitar sesiones simultáneas no permitidas con un mismo identificador de Usuario de la Infraestructura Tecnológica.

e) Mecanismos de seguridad, tanto de acceso físico, como de controles ambientales y de energía eléctrica, que protejan la Infraestructura Tecnológica y permitan la operación conforme a las especificaciones del proveedor, fabricante o desarrollador.

f) Medidas de validación para garantizar la autenticidad de las transacciones ejecutadas por los diferentes componentes de la Infraestructura Tecnológica, considerando, al menos lo siguiente:

1. La veracidad e integridad de la información.

2. La Autenticación entre componentes de la Infraestructura Tecnológica, que aseguren que se ejecutan solo las solicitudes de servicio legítimas desde su origen y hasta su ejecución y registro.

3. Los protocolos de mensajería, comunicaciones y cifrado, los cuales deben procurar la integridad y confidencialidad de la información.

4. La identificación de transacciones atípicas, previendo que las aplicaciones cuenten con medidas de alerta automática para su atención de las áreas operativas correspondientes.

5. La actualización y mantenimiento de certificados digitales y componentes proporcionados por proveedores de servicios que estén integrados al proceso de ejecución de transacciones.

Las medidas a que alude este inciso deberán establecerse acorde con el grado de riesgo que las Instituciones definan para cada tipo de transacción.

VII. Que cuente con mecanismos de respaldo y procedimientos de recuperación de la información que mitiguen el riesgo de interrupción de la operación, en concordancia con lo dispuesto en el Artículo [164 Bis](articulo-164-Bis.md) de las presentes disposiciones.

VIII. Que mantenga registros de auditoría íntegros, incluyendo la información detallada de los accesos o intentos de acceso y la operación o actividad efectuada por los Usuarios de la Infraestructura Tecnológica, lo anterior, con independencia del nivel de privilegios con el que estos cuenten para el acceso, generación o modificación de la información que reciban, generen, almacenen o transmitan en cada componente de la Infraestructura Tecnológica, incluyendo actividad de procesos automatizados, así como los procedimientos para la revisión periódica de dichos registros.

Las Instituciones deberán conservar los registros de auditoría a que se refiere esta fracción por un periodo de tres años cuando dichos registros se refieran a actividades realizadas sobre componentes que procesen o almacenen información considerada como crítica de conformidad con la clasificación señalada en el Artículo [86](articulo-86.md), fracción III, inciso b), numeral 3 de las presentes disposiciones. En caso contrario, el periodo de conservación de los registros será mínimo de seis meses.

IX. Que para la atención de los Incidentes de Seguridad de la Información se cuente con procesos de gestión que aseguren la detección, clasificación, atención y contención, investigación y, en su caso, análisis forense digital, diagnóstico, reporte a niveles jerárquicos competentes, solución, seguimiento y comunicación a autoridades, clientes y contrapartes de dichos incidentes.

Para la detección y respuesta de Incidentes de Seguridad de la Información a que hace referencia el párrafo anterior, el director general deberá designar un equipo que incorpore a personal de las diferentes áreas de la Institución para participar en cada actividad del proceso de gestión antes señalado del que, en todo caso, deberá formar parte el oficial en jefe de seguridad de la información de conformidad con la fracción VII del Artículo [168 Bis 14](articulo-168-Bis-14.md) de estas disposiciones.

En caso de que se detecte la existencia de vulnerabilidades y deficiencias en la Infraestructura Tecnológica, deberán tomarse las acciones correctivas o controles compensatorios de acuerdo al nivel de riesgo de que se trate, previniendo que los Usuarios de la Infraestructura Tecnológica o la Institución puedan verse afectados.

X. Que sea sometida a la realización de ejercicios de planeación y revisión anuales que permitan medir su capacidad para soportar su operación, garantizando que se atiendan oportunamente las necesidades de incremento de capacidad detectadas como resultado de dichos ejercicios.

Asimismo, la Institución deberá evaluar la obsolescencia de los componentes de la Infraestructura Tecnológica, debiendo contar con un plan para su actualización.

XI. Que cuente con controles automatizados o, en ausencia de estos, que se realicen controles compensatorios, tales como doble verificación, que previo o posteriormente a la realización de la operación de que se trate, minimicen el riesgo de eliminación, exposición, alteración o modificación de información, que se deriven de procesos manuales o semi-automatizados realizados por el personal de la Institución, con el objetivo de prevenir errores, omisiones, sustracción o manipulación de información.

XII. Que tenga controles que permitan detectar la alteración o falsificación de libros, registros y documentos digitales relativos a las operaciones activas, pasivas y de servicios de la Institución.

XIII. Que cuente con procesos para medir y asegurar los niveles de disponibilidad y tiempos de respuesta, que garanticen la ejecución de las operaciones y servicios realizados; lo anterior incluyendo los supuestos en que las Instituciones contraten la prestación de servicios por parte de proveedores externos para el procesamiento y almacenamiento de información.

XIV. Que cuente con dispositivos o mecanismos automatizados para detectar y prevenir eventos e Incidentes de Seguridad de la Información, así como para evitar conexiones y flujos de datos entrantes o salientes no autorizados y fuga de información, considerando entre otros, medios de almacenamiento removibles.

Las Instituciones deberán correlacionar los datos obtenidos de los dispositivos o mecanismos automatizados a que alude el párrafo anterior con los datos de otras fuentes, tales como registros de actividad o de Incidentes de Seguridad de la Información.

Adicionalmente, a lo señalado en el párrafo anterior, las Instituciones deberán mantener controles que eviten la filtración de la información correspondiente a la configuración de la Infraestructura Tecnológica, tales como direcciones IP, reglas de los cortafuegos, así como versiones de hardware y software.

XV. Que para la prestación de servicios de tecnologías de información a los Usuarios de la Infraestructura Tecnológica, en sus fases de estrategia, diseño, transición, operación y mejora continua se proteja la integridad de la Infraestructura Tecnológica así como la integridad, confidencialidad y disponibilidad de la información recibida, generada, procesada, almacenada y transmitida por esta.

XVI. Definir en los respectivos manuales de políticas y procedimientos, los roles y responsabilidades del personal de la Infraestructura Tecnológica en materia de seguridad de la información de la Institución

El director general será responsable de documentar en políticas y procedimientos lo previsto en este artículo.
