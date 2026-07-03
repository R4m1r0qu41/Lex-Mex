---
id: urn:lex-mx:federal:regulation:ifpe-dcg-2021:annex:8
instrument_id: urn:lex-mx:federal:regulation:ifpe-dcg-2021
instrument: DCG-IFPE-2021
provision_type: annex
number: "8"
aliases: ["DCG-IFPE-2021 — Anexo 8"]
generated: true
temporal_status: unknown
review_status: not_analyzed
source_url: https://www.cnbv.gob.mx/Normatividad/Disposiciones%20aplicables%20a%20las%20instituciones%20de%20fondos%20de%20pago%20electr%C3%B3nico%20a%20que%20se%20refieren%20los%20art%C3%ADculos%2048%2C%20segundo%20p%C3%A1rrafo%3B%2054%2C%20primer%20p%C3%A1rrafo%20y%2056%2C%20primer.pdf
source_sha256: 493282f369e52da50db28c4777119591852a52313e5bb1cef82d1bd453899bb0
---

# Anexo 8

Especificaciones del sistema de información desarrollado por un tercero para el cifrado de información compartida con la Comisión Nacional Bancaria y de [Valores](../../lritf/markdown/articulo-4.md) y el Banco de México

Para efectos de este anexo, los términos con inicial mayúscula utilizados en este, en singular o plural, tendrán los mismos significados que los establecidos para dichos términos el Código de Comercio y las Reglas de la Infraestructura Extendida de Seguridad (IES), así como los siguientes:

Certificado Digital Calificado: a aquel Certificado Digital emitido, conforme a las Reglas de la IES, por el Servicio de Administración Tributaria, en su carácter de Agencia Certificadora, también denominado en las disposiciones de este como “e.firma”, que es almacenado en un archivo digital con extensión “.cer” cuando es obtenido ante dicha autoridad de conformidad con las disposiciones que esta establezca para tal efecto, así como aquel otro Certificado Digital que, conforme a las Reglas de la IES, emita un tercero autorizado, en su caso, por el Banco de México, sujeto a la determinación de este último de que dicho Certificado Digital cumple con los mismos requisitos de seguridad y de acreditación de la identidad del interesado que observa el Servicio de Administración Tributaria para su expedición.

Cifrado: al proceso de aplicar los Datos de Verificación de Firma Electrónica Calificados a un Mensaje de Datos para generar uno nuevo que sea ininteligible para cualquier persona, excepto para el Titular del Certificado Digital Calificado del que forman parte los Datos de Verificación de Firma Electrónica Calificados, quien, a su vez, funge como el Destinatario de dicho Mensaje de Datos.

Creación de una Firma Electrónica: al proceso de aplicar los Datos de Creación de Firma Electrónica Calificados a un Mensaje de Datos y generar la Firma Electrónica que se agrega al referido Mensaje de Datos.

Datos de Creación de Firma Electrónica a aquellos Datos de Creación de Firma Electrónica previstos en las Reglas de la IES que el Calificados: Titular genera como parte del proceso de emisión de su respectivo Certificado Digital, que se almacenan en un archivo digital con extensión “.key”.

Datos de Verificación de Firma a aquellos Datos de Verificación de Firma Electrónica a que se refieren las Reglas de la Electrónica Calificados: IES que son parte de la información incluida en el Certificado Digital.

Descifrado: al proceso de aplicar los Datos de Creación de Firma Electrónica Calificados a un Mensaje de Datos que haya sido [Cifrado](articulo-1.md), para que el Titular del respectivo Certificado Digital pueda ver el contenido del Mensaje de Datos original.

Firma Electrónica: al conjunto de datos que se agrega a un Mensaje de Datos, el cual está asociado en forma lógica a éste y es atribuible al Titular una vez utilizado el Sistema de Información Calificado y que cumple con los requisitos de Firma Electrónica Avanzada o Fiable a que se refiere el artículo 89 del Código de Comercio, según sea modificado o sustituido con posterioridad.

Infraestructura Extendida de a aquel a que se refieren las Reglas de la IES. Seguridad (IES):

Reglas de la IES: a las Reglas para Operar como Agencia Registradora y/o Agencia Certificadora en la Infraestructura Extendida de Seguridad, emitidas por el Banco de México mediante la Circular-Telefax 6/2005, según sean modificadas o sustituidas con posterioridad.

Sistema de Información Calificado: a aquel sistema de información que permite, por una parte, la Creación de Firmas Electrónicas, como Dispositivo de Creación de Firma Electrónica en términos de las Reglas de la IES, y, por otra parte, la Verificación de Firmas Electrónicas, como Dispositivo de Verificación de Firma Electrónica en términos de dichas Reglas, así como llevar a cabo el Cifrado y Descifrado de Mensajes de Datos.

Titular: a aquel a que se refiere las Reglas de la IES, que interviene en su carácter de Firmante en términos del artículo 89 del Código de Comercio.

Verificación de una Firma Electrónica: al proceso de aplicar los Datos de Verificación de Firma Electrónica a la Firma Electrónica de un Mensaje de Datos y comprobar, tanto la fiabilidad de dicha Firma Electrónica mediante la verificación de que esta fue creada para ese mismo Mensaje de Datos utilizando los Datos de Creación de la Firma Electrónica que corresponden a los Datos de Verificación de Firma Electrónica, como la integridad del Mensaje de Datos al no sufrir alteración después de generada su Firma Electrónica.

El programa de cómputo desarrollado por un tercero para realizar el Cifrado de información compartida con la [CNBV](../../lritf/markdown/articulo-4.md) y el Banco de México deberá cumplir con las siguientes especificaciones:

I. Tener como función principal la aplicación de algoritmos criptográficos que cumplan con las especificaciones de Firma Electrónica previstas en las Reglas de la IES.

II. Mantener comunicación con una Agencia Registradora de la Infraestructura Extendida de Seguridad para estar en posibilidad de solicitar y verificar la validez de Certificados Digitales Calificados de los Firmantes involucrados en los procesos de Creación y Verificación de Firmas Electrónicas y cifrar y descifrar Mensajes de Datos. Para tal efecto, el programa de cómputo deberá cumplir con el Protocolo de Comunicación con la Infraestructura Extendida de Seguridad que la Dirección General de Sistemas de Pagos e Infraestructuras de Mercados mantenga a disposición de los interesados en la página que el Banco de México tiene en su sitio de internet que se identifica con el dominio www.banxico.org.mx

III. Implementar el estándar RFC 3852 “Cryptographic Message Syntax (CMS)” para la Creación de una Firma Electrónica y cifrar Mensajes de Datos. Dentro del estándar referido, la especificación del archivo resultante de la Creación de una Firma Electrónica que se genera mediante el así denominado Signed-data Content Type sobre la información que se conforma del tipo de datos archivos cuya especificación en su notación ASN.1 se describe más adelante. De igual forma, dentro del estándar referido, la especificación del archivo resultante al cifrar un Mensaje de Datos se genera mediante el así denominado Enveloped-data Content Type sobre la información que se conforma del tipo de datos archivos cuya especificación en su notación ASN.1 se describe más adelante.

La información que se emplea dentro del estándar RFC 3852 es aquella que se dispone de acuerdo con la siguiente descripción en la notación ASN.1:

Archivos ::= SEQUENCE of Archive Archivo ::= SEQUENCE {

nombre OCTET STRING,

contenido OCTET STRING }

donde nombre es el nombre del archivo que contiene la información de interés. Por otro lado, contenido es la información contenida en dicho archivo interpretada como una secuencia de bytes.
