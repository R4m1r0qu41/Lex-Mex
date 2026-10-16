# Operator decision — REG-LFMN abrogation and CFPC sunset — 2026-10-16

## Decisions

1. **`reg-lfmn` is recorded as an abrogated, historical instrument.** Operator
   instruction, 2026-10-16: "abrogate reg-lfmn". The instrument stays in the
   corpus with instrument status `repealed`, in the same way as `lftr`.
   Reviewer of record for this decision: Operator. It is not a JRH legal-review
   resolution.
2. **`cfpc` is unchanged.** Operator instruction, 2026-10-16: "cfpc is on track
   for 2027". It stays `in_force`. The Cámara index reports a statutory
   abrogation deadline of 2027-04-01; no corpus change is made. Recheck before
   that date.

## Evidence for decision 1

- **Operative text.** The retained text of `reg-lic-calidad` (Reglamento de la
  Ley de Infraestructura de la Calidad) has Transitory *Segundo*: "Con la
  entrada en vigor del presente Reglamento, se abroga al Reglamento de la Ley
  Federal sobre Metrología y Normalización, publicado en el Diario Oficial de
  la Federación el 14 de enero de 1999, y sus reformas…". Provision
  `urn:lex-mx:federal:regulation:reg-lic-calidad:transitory:segundo`, text
  SHA-256 `22fdaccffa218443b44b70a3728007dfba60074c714672dd9bda233b8ab8dc1d`.
- **Entry into force.** Transitory *Primero*: the Reglamento enters into force
  the business day after its DOF publication. The Cámara index gives a DOF
  publication date of 2026-09-04.
- **Formal publication.** DOF note
  `https://dof.gob.mx/nota_detalle.php?codigo=5797937&fecha=04/09/2026`,
  retrieved 2026-10-16; retained bytes SHA-256
  `ac60a35707dfc34fd71d48ba5807748afb4885958e4eaffcae21da0f8f9995d4`
  (private, `.work/reg-lfmn-abrogation/`). The same Segundo text appears in it.
- **Cámara index.** `regla.htm` retrieved 2026-10-16T05:15Z (SHA-256
  `290d5c3d842c6bda231c29bacb3cb05531e6071b82f2c93f9e47c49212b615ce`) lists
  the 1999 regulation as "Reglamento Abrogado", 04/09/2026.
- **Retained source unchanged.** The retained `Reg_LFMN.pdf`
  (SHA-256 `3acf06eaf8b838cdfa68553efe0f007fcc3e9eee10b0ed42d693c99b0ae59bf3`)
  is byte-identical to the live Cámara file, which still prints "TEXTO
  VIGENTE" and carries no abrogation header. The existing header-based
  `source_repeal_notice` basis therefore cannot be used; a successor-clause
  basis was added (see `docs/held-source-parser-contracts.md`).

## Boundaries

- Only the instrument-level status is set. No provision-level temporal state,
  effective date or end date is assigned or inferred; the entry-into-force day
  is not computed.
- Successor identity recorded: `urn:lex-mx:federal:regulation:reg-lic-calidad`.
- The retained instrument text, provisions and references of `reg-lfmn` are not
  altered.
