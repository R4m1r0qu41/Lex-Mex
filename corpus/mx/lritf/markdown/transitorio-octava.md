---
id: urn:lex-mx:federal:statute:lritf:transitory:octava
instrument_id: urn:lex-mx:federal:statute:lritf
instrument: LRITF
name: "Ley para Regular las Instituciones de Tecnología Financiera"
provision_type: transitory
number: "OCTAVA"
aliases: ["LRITF — OCTAVA"]
generated: true
temporal_status: effective
review_status: machine_accepted
transitory_effects: ["adaptation_period","transitional_permission","migration","coordination_mandate"]
source_url: https://www.diputados.gob.mx/LeyesBiblio/pdf/LRITF.pdf
source_sha256: d6f645e6a7d3c2eeb46905d4d24ecd8e078907057dc034cda715abf019ce8491
---

# OCTAVA

Las personas que a la entrada en vigor del presente ordenamiento se encuentren realizando las actividades reguladas en esta Ley deberán dar cumplimiento a la obligación de solicitar su autorización ante la Comisión Nacional Bancaria y de [Valores](articulo-4.md) en los términos en que se establezca en las disposiciones de carácter general que para tal efecto se emitan, en un plazo que no exceda de doce meses contado a partir de la entrada en vigor de estas disposiciones. Dichas personas podrán continuar realizando tales actividades hasta en tanto la Comisión Nacional Bancaria y de Valores resuelva su solicitud, pero hasta en tanto no reciban la autorización respectiva deberán publicar en su página de internet o medio que utilice que la autorización para llevar a cabo dicha actividad se encuentra en trámite por lo que no es una actividad supervisada por las autoridades mexicanas. La Comisión Nacional Bancaria y de Valores denegará la autorización cuando las personas respectivas incumplan la obligación de publicación señalada en este párrafo.

En caso de que las personas a que se refiere el párrafo anterior no soliciten su autorización en el plazo de doce meses previsto en dicho párrafo o no la obtengan una vez solicitada, éstas deberán abstenerse de continuar prestando sus servicios para la celebración de nuevas [Operaciones](articulo-4.md) y deberán realizar únicamente los actos tendientes a la conclusión o cesión de las Operaciones existentes reguladas en esta Ley, notificando a sus [Clientes](articulo-4.md) dicha circunstancia y la forma en que se concluirán o cederán las Operaciones.

Las autoridades competentes procurarán que en los sitios de internet de sociedades que no obtengan o no cuenten con la autorización correspondiente se alerte a los Clientes de los riesgos de operar con dichas entidades y buscarán impedir su oferta en territorio nacional, salvo lo dispuesto en el primer párrafo de este artículo.

## Efectos transitorios estructurados

### Efecto 1 — adaptation_period

- **Alcance afectado:** Personas que ya realizaban actividades reguladas: solicitud de autorización
- **Regla de aplicación:** migration_to_new_rule
- **Detonante:** external_event: Entrada en vigor de las disposiciones generales sobre la solicitud
- **Condición de terminación:** relative_period: Plazo máximo de doce meses desde la entrada en vigor de esas disposiciones
- **Autoridades responsables:** Comisión Nacional Bancaria y de Valores
- **Verificación:** external_verification_required

### Efecto 2 — transitional_permission

- **Alcance afectado:** Continuación temporal de las actividades mientras se resuelve la solicitud, con aviso público obligatorio
- **Regla de aplicación:** transitional_permission
- **Detonante:** authority_action: Presentación de la solicitud de autorización
- **Condición de terminación:** authority_action: Resolución de la solicitud por la Comisión Nacional Bancaria y de Valores
- **Autoridades responsables:** Comisión Nacional Bancaria y de Valores
- **Verificación:** external_verification_required

### Efecto 3 — migration

- **Alcance afectado:** Cese de nuevas operaciones y conclusión o cesión de operaciones existentes cuando no se solicite u obtenga autorización
- **Regla de aplicación:** mixed
- **Detonante:** external_event: Vencimiento del plazo sin solicitud o falta de obtención de la autorización
- **Condición de terminación:** cohort_exhaustion: Conclusión o cesión de las operaciones existentes
- **Autoridades responsables:** Comisión Nacional Bancaria y de Valores
- **Verificación:** open_ended_by_design

### Efecto 4 — coordination_mandate

- **Alcance afectado:** Alertas a clientes e impedimento de ofertas de sociedades no autorizadas
- **Regla de aplicación:** not_applicable
- **Detonante:** external_event: Sociedad que no obtiene o no cuenta con autorización
- **Condición de terminación:** No aplica
- **Autoridades responsables:** Autoridades competentes
- **Verificación:** external_verification_required
