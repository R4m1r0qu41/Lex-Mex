# Lex-Mex parser split: why it mattered, how it worked, and where we stand

Review date: 2026-10-24. Code reviewed at HEAD
`e3ad94ee3bf6e5a5ec34274ad77af10ce88bcf1d`.
Intended audience: a project stakeholder who wants to understand the engineering
without reading every Rust module. Published to `/Users/jr/Vaults/Spearhead/20_Repos/Lex-Mex` on 2026-10-25;
the repository retains the source copy and machine evidence.

## Result of the HEAD-versus-plan check

The implemented split meets the plan's central architectural objective. The
accepted parser candidate `1250889ce` and reviewed Reader R7 `0661d5ef0` are
ancestors of HEAD. Production code matches reviewed R7; formatting and locked,
offline workspace metadata checks pass. The workspace now has nine crates,
including the three new responsibility crates.

No new production defect was found by this bounded review. This is a check of
current structure, ancestry and accepted evidence, not a fresh execution of the
multi-hour corpus replay or a proof that every existing legal record is correct.

The main discrepancy is documentary: the split plan, integration plan and early
receipts retain pending/pre-integration language. Later audit, integration,
publication and cleanup records supersede that language. Current-state notes
have been appended to the two plans while preserving their original baselines.
The separate post-promotion P/R audit is still recorded as pending; this report
does not silently declare that entire audit complete.

## 1. Why was the split needed?

Lex-Mex had grown beyond a document parser. The same crate, `lex-parse`, also
contained rules for recognizing labels and dates, resolving citations, matching
defined terms, validating records, classifying initial temporal evidence and
applying certain audited reviews. These tasks are related, but they answer
different questions and change for different reasons.

For example, reading “Artículo 12” from a source document is a parsing concern.
Determining which other provision a reference names is a linking concern.
Checking that its character offsets match the original text is a validation
concern. Deriving a commencement proposal is an analysis concern. Keeping all
of these behind one large parser made ownership harder to see and made the
database depend on a larger surface than it needed.

The goal was to make those boundaries visible in code and enforceable by Rust's
dependency graph. It was also a preparation for safer future repairs: a change
to page-header handling can stay in parsing, while a citation-resolution change
can stay in linking. The split did not replace deterministic rules with a model,
remove regression tests, or attempt to resolve legal ambiguities.

## 2. How was it accomplished?

The work proceeded in stages rather than moving everything in one operation.
Claude Sonnet 5.5 was the designated executor, Opus 5.5 the bounded advisor,
and Codex provided subsequent independent reviews.

| Responsibility | Owner now | Plain-language job |
| --- | --- | --- |
| Shared legal data types | `lex-core` | Defines the records passed between stages |
| Lexical conventions | `lex-grammar` | Recognizes shared label, ordinal and date syntax |
| Document structure | `lex-parse` | Turns supported official source formats into structured records; handles furniture and glossary extraction |
| References and usages | `lex-link` | Extracts/resolves citations and matches uses of defined terms |
| Validation and analysis | `lex-analysis` | Checks records, derives temporal evidence and applies the moved audited-review operations |
| Database projection | `lex-db` | Builds and verifies the SQLite representation of committed corpus inputs |

“Four crates” means the four responsibilities grammar, parse, link and analysis;
it does not mean the entire project now has only four crates.

First, the executor inventoried public symbols, callers and 26 rule groups.
Code was organized into modules inside the existing parser. Grammar and
analysis were extracted next, then linking, and finally first-party consumers
were moved to their responsible crates. Old `lex_parse` public paths were kept
as thin compatibility exports so existing consumers could still compile.

The dependency direction is now explicit: grammar has no project-crate
dependency; analysis and link depend on core and grammar; neither depends on
parse, the CLI, export or the database. Parse retains initial-analysis calls
at their original point and compatibility exports. The database depends on
core, grammar and analysis, not on the parser crate. Some cross-stage regression
tests deliberately remain in parse because their fixtures start with parsing.
Standard recognition-dependent validation also remains there.

This is separation by responsibility, not separation by arbitrary file length.
The rules and regression cases remain; they are easier to locate through
[the rule inventory](parser-rule-inventory.md).

## 3. What stayed unchanged, and what intentionally changed?

The acceptance requirement was preservation of source text, IDs and ordering,
reference targets and spans, glossary results, validation messages, temporal
outputs, audited decisions, exports and logical database rows.

One independent defect, D-001, was found in database provenance checking:
a claimed builder could add a Rust/JSON source file in a pinned directory
without rejection. The accepted fix tightens that check. It changes which
builder claims are accepted, not the corpus content or interpretation of law.

Later Reader integration added public-release schemas and UI/export work outside
the split's original scope. Comparing the split candidate with HEAD shows only
one change within the parser/database crate set: a database test assertion was
restated to satisfy strict linting. The two added public schemas belong to the
Reader. Corpus, adapters and accepted-decision bytes have no differences in
that scoped comparison.

## 4. Steps and tests: why did verification take so long?

A passing unit suite cannot establish that moving code preserved a whole legal
corpus. The project therefore used several different kinds of evidence.

| Gate | What it established | Recorded result |
| --- | --- | --- |
| G0: baseline and frozen inputs | The comparison process can reproduce unchanged software before judging a candidate | Strict B3/B4 self-replay exact; unavailable sources explicitly listed |
| G1: module organization | Responsibility moves inside parse do not alter results | Candidate C1 equals baseline |
| G2: grammar/analysis extraction | New crate boundaries preserve behavior and old paths | Candidate C2 equals baseline |
| G3: linking and consumers | Direct ownership, compatibility and builder-source pinning work | Candidate C3 equals baseline; provenance negatives reject mismatches |
| G4: final acceptance | Native checks, whole-corpus stages and database equivalence | 425 Rust tests; both validators; three DB verifications and 21 golden queries each |
| G5 and follow-ups | Independent review challenges both code and the evidence machinery | Production accepted; provenance fix accepted; tooling failures repaired and reviewed |

The final strict queue ran two baseline executions and three candidate versions,
each over retained-source replay, the whole-corpus sweep and recovered-source
supplement. That made 15 runs and 12 strict comparisons, grouped into 27 queue
steps. Independent review covered 22,330 executed command rows and 44,660
stdout/stderr files. All comparisons reported zero drift. Outputs were compared
as content, including tree hashes and structured records, rather than merely
checking that commands returned success.

There were 355 retained-source instruments, seven recovered inputs and 32
standards exercised through the standards sweep. The whole-corpus sweep
covered committed records independently of whether their original parse input
was retained. These sets must not be added as though they were disjoint counts
of admitted instruments.

The operator confirmed that 15 hours was a rough estimate. The measured
strict queue duration is **10h 53m**. The preserved queue log runs from
2026-10-23 23:42:30 UTC to 2026-10-24 10:35:33 UTC, **10h 53m 03s**. Earlier
legacy runs, compilation, database builds, independent checks and repairs were
additional work; no precise cumulative duration is claimed for those activities. Each whole-corpus sweep alone took roughly 86 minutes.

The evidence machinery itself needed corrections. Missing comparison evidence
became a failure; the executing harness was frozen at run start. Wrapper scripts
were repaired to propagate failed commands. Restart recovery became atomic and
checks that a completed result contains the full requested scope, rows, logs
and saved trees before marking it done. Recorded follow-up checks reached
54 harness tests, 16 wrapper tests and acceptance of all 15 real run plans.

An executor mutation test overwrote three original native logs. The incident
was disclosed, the native results regenerated, and replacement hashes recorded;
the databases and strict A/B artifacts were unaffected. Replacement logs must
not be presented as the original bytes. This distinction is part of the audit
trail, not a reason to erase the incident.

Reader integration subsequently passed 430 Rust tests. The accepted R7 review
records 25 Node tests and 60 successful preferred-browser cases. Those are later
Reader/integration evidence, not the original split's 425-test gate. This turn
reran formatting and dependency/ancestry checks, not those complete campaigns.

## 5. How does this interact with the database?

The canonical corpus is the committed structured data. SQLite is a reproducible,
queryable projection of that data, with normalized tables, provenance and FTS
search. Moving Rust code into new crates does not require reinterpreting the law
or rebuilding an otherwise accepted database just to attach today's HEAD.

The builder embeds/pins the relevant source files and manifests. The split
updated both compilation triggers and runtime provenance checks to include the
new crates. A future source addition must trigger rebuilding and must not be
accepted under a false builder identity. Parse remains in the pinned source
inventory conservatively, even though `lex-db` no longer links it.

For acceptance, one baseline-built DB and two candidate-built DBs used the same
corpus commit. Independent checking compared all ordinary logical rows and
68,341 FTS content rows. Schema, legal-review payloads and logical content
matched. Only explicitly enumerated builder/release provenance changed between
baseline and candidate. The two candidate DBs and manifests were byte-identical.
Their shared logical content digest begins `8cc37360`.

The accepted permanent release remains `afdc5acf…`, corpus/builder `5eca03bc5`.
The split acceptance DBs name builder `1250889ce`. Those are different truthful
provenance identities for equivalent logical content; neither should be relabeled
to HEAD. Fresh read-only SQL counts on the permanent artifact confirm:

| Contents | Count |
| --- | ---: |
| Instruments | 468 |
| Provisions | 68,341 |
| Reference edges | 35,930 |
| Defined terms | 5,766 |
| Term usages | 97,672 |
| Active structural-debt rows | 14 |
| Catalogue standards | 32 |

The accepted records also preserve three debt retirements. SCJN invalidity
registry evidence remains an input, not searchable provisions; its fuller
integration has a separate plan. The database exists and has reproducibility,
verification and golden-query evidence. A federal-scale deployed query service
is a separate deliverable, not an automatic consequence of building SQLite.

## 6. Overall project state

The deterministic parser split is integrated. The database foundation is built
and accepted within its documented scope. R7 is integrated and the exact tested
four-instrument publication is live, as independently checked earlier today.
The public Reader's four-instrument scope is smaller than the 468-instrument
local database; those two counts describe different products.

The remaining limits are concrete:

- 106 instruments lack retained parse inputs. Whole-corpus downstream checks
  cover their committed records, but fresh parse equivalence is unproved.
- Known baseline failures and two provenance-mismatched replay sources were
  disclosed. Exact reproduction of an existing failure is evidence of parity,
  not evidence that the underlying problem is fixed.
- Fourteen structural-debt rows, source-specific holds and temporal/legal review
  work remain. The refactor did not silently resolve them.
- The preserved public-query experiment is not accepted for publication. Its
  review found exact-text HTML, cursor validation and normalized-empty search
  defects, plus scaling and source-URL policy work. It needs isolated repair on
  current main and independent review before integration.
- The newly supplied `lexmex-retest-2026-10-24.md` reports agent-discovery gaps.
  Its unchanged-output observation is consistent with the refactor's purpose;
  it does not alone prove a deployment failure. Its legal-source claims and
  external tool limitations have not been independently verified by this audit.

Historical source paths changed during authorized cleanup. Necessary comparison
artifacts are recoverable through the SHA-addressed compressed manifest in
`.work/retained-reader-parser-evidence-2026-10-24/`; old absolute parser-worktree
paths should no longer be treated as existing directories. Redundant compiler
outputs were removed. The three permanent project checkouts remain the working
layout, with unrelated OCR experiments outside that cleanup scope.

The useful next implementation task is to repair and review the agreed agent
query/discovery surface on current main. It should use the now-separated parser,
linking and analysis responsibilities and the accepted database, while preserving
R7 and exact legal text. It should not trigger another full split replay merely
because documentation or unrelated UI code changes.

## Evidence and verification boundary

This turn: ancestry checks, zero production diff against reviewed R7, scoped
candidate-to-HEAD diff, formatting, locked offline Cargo metadata, read-only DB
census and recovery/hash verification of the archived queue log. No corpus
mutation, source refetch, full test replay, database rebuild or deployment.
Unrelated `docs/lexmex-retest-2026-10-24.md` was preserved.

Sources: [execution plan](plans/parser-responsibility-split.md),
[implementation receipt](parser-split-implementation-receipt.md),
[independent audit](parser-split-independent-audit-2026-10-24.md),
[tooling follow-up](parser-split-tooling-followup-review-2026-10-24.md),
[integration receipt](parser-reader-integration-receipt.md),
[R7/live verification](r7-integration-and-work-cleanup-2026-10-24.md),
[permanent DB acceptance](heading-leak-permanent-acceptance-2026-10-21.md),
[query review](public-query-branch-review-2026-10-24.md),
[compact evidence retention](reader-parser-worktree-cleanup-2026-10-24.md),
and [this review's machine evidence](parser-split-head-review-2026-10-24-evidence.json).

## Annex A — The 106 missing retained parse inputs

This is the historical coverage gap at the frozen split baseline, not a claim
that these instruments are absent from the corpus/database or legally held.
Their committed records participated in downstream whole-corpus checks; their
original parse behavior could not be replayed from retained inputs. No new
filesystem-wide source search or acquisition was performed for this annex.

Derivation: the hash-verified original coverage manifest has 113 entries marked
“retained extracted text absent from source .work”. The hash-verified recovery
search and supplementary replay identify the same seven recovered instruments:
`lcmopfih`, `lftr`, `lga`, `lgag`, `lif-2026`, `pef-2026`, `reg-lgs-insumos`.
Removing those seven gives exactly 106 unique instruments below. The separate
32 standards without adapter configurations are not part of this list.
Titles and parser families are read from adapters at baseline `85c41f3898`.
All rows have the same blocker: no retained extracted-text input available for
that replay after the recorded recovery search. This does not establish whether
each original PDF is also absent; do not infer a source-specific parser defect.

Suggested remediation: recover the exact text/source pair matching committed
manifest hashes from archives first. If unavailable, acquire an official edition
through a separate provenance-reviewed task; a changed edition is a new ingestion
candidate, not a substitute for historical parity evidence. Recovering an input
clears the coverage gap only after its identity and replay result are verified.

| # | Instrument slug | Official title | Parser family |
| ---: | --- | --- | --- |
| 1 | `lbogm` | Ley de Bioseguridad de Organismos Genéticamente Modificados | `diputados` |
| 2 | `lce` | Ley de Comercio Exterior | `diputados` |
| 3 | `lcf` | Ley de Coordinación Fiscal | `diputados` |
| 4 | `lcmm` | Ley de la Casa de Moneda de México | `diputados` |
| 5 | `lcpaf` | Ley de Caminos, Puentes y Autotransporte Federal | `diputados` |
| 6 | `ldcmpme` | Ley para el Desarrollo de la Competitividad de la Micro, Pequeña y Mediana Empresa | `diputados` |
| 7 | `ldcpdch` | Ley para el Diálogo, la Conciliación y la Paz Digna en Chiapas | `diputados` |
| 8 | `ldfefm` | Ley de Disciplina Financiera de las Entidades Federativas y los Municipios | `diputados` |
| 9 | `ldvuma` | Ley para Determinar el Valor de la Unidad de Medida y Actualización | `diputados` |
| 10 | `lfaebsp` | Ley Federal para la Administración y Enajenación de Bienes del Sector Público | `diputados` |
| 11 | `lfar` | Ley Federal de Austeridad Republicana | `diputados` |
| 12 | `lfcc` | Ley de Fomento a la Confianza Ciudadana | `diputados` |
| 13 | `lfcp` | Ley Federal de Correduría Pública | `diputados` |
| 14 | `lfd` | Ley Federal de Derechos | `diputados` |
| 15 | `lfdc` | Ley Federal de los Derechos del Contribuyente | `diputados` |
| 16 | `lfdo` | Ley Federal contra la Delincuencia Organizada | `diputados` |
| 17 | `lfed` | Ley Nacional de Extinción de Dominio | `diputados` |
| 18 | `lffmaa` | Ley Federal para el Fomento de la Microindustria y la Actividad Artesanal | `diputados` |
| 19 | `lfisan` | Ley Federal del Impuesto sobre Automóviles Nuevos | `diputados` |
| 20 | `lfjs` | Ley Federal de Juegos y Sorteos | `diputados` |
| 21 | `lfpa` | Ley Federal de Procedimiento Administrativo | `diputados` |
| 22 | `lfpca` | Ley Federal de Procedimiento Contencioso Administrativo | `diputados` |
| 23 | `lfprh` | Ley Federal de Presupuesto y Responsabilidad Hacendaria | `diputados` |
| 24 | `lfra` | Ley Federal de Responsabilidad Ambiental | `diputados` |
| 25 | `lfremsp` | Ley Federal de Remuneraciones de los Servidores Públicos | `diputados` |
| 26 | `lftse` | Ley Federal de los Trabajadores al Servicio del Estado, Reglamentaria del Apartado B) del Artículo 123 Constitucional | `diputados` |
| 27 | `lgcc` | Ley General de Cambio Climático | `diputados` |
| 28 | `lgcg` | Ley General de Contabilidad Gubernamental | `diputados` |
| 29 | `lgdp` | Ley Federal de Deuda Pública | `diputados` |
| 30 | `lgeepa` | Ley General del Equilibrio Ecológico y la Protección al Ambiente | `diputados` |
| 31 | `lgn` | Ley de la Guardia Nacional | `diputados` |
| 32 | `lgpc` | Ley General de Protección Civil | `diputados` |
| 33 | `lgpgir` | Ley General para la Prevención y Gestión Integral de los Residuos | `diputados` |
| 34 | `lgsm` | Ley General de Sociedades Mercantiles | `diputados` |
| 35 | `lgsnsp` | Ley General del Sistema Nacional de Seguridad Pública | `diputados` |
| 36 | `lgtrata` | Ley General para Prevenir, Sancionar y Erradicar los Delitos en Materia de Trata de Personas y para la Protección y Asistencia a las Víctimas de estos Delitos | `diputados` |
| 37 | `lgvs` | Ley General de Vida Silvestre | `diputados` |
| 38 | `lhheum` | Ley de los Husos Horarios en los Estados Unidos Mexicanos | `diputados` |
| 39 | `lieps` | Ley del Impuesto Especial sobre Producción y Servicios | `diputados` |
| 40 | `ligie` | Ley de los Impuestos Generales de Importación y de Exportación | `diputados` |
| 41 | `liispcen` | Ley para Impulsar el Incremento Sostenido de la Productividad y la Competitividad de la Economía Nacional | `diputados` |
| 42 | `lisf` | Ley de Instituciones de Seguros y de Fianzas | `diputados` |
| 43 | `lisr` | Ley del Impuesto sobre la Renta | `diputados` |
| 44 | `liva` | Ley del Impuesto al Valor Agregado | `diputados` |
| 45 | `lmeum` | Ley Monetaria de los Estados Unidos Mexicanos | `diputados` |
| 46 | `lncm` | Ley de Navegación y Comercio Marítimos | `diputados` |
| 47 | `lnetb` | Ley Nacional para Eliminar Trámites Burocráticos | `diputados` |
| 48 | `loapf` | Ley Orgánica de la Administración Pública Federal | `diputados` |
| 49 | `locfcrl` | Ley Orgánica del Centro Federal de Conciliación y Registro Laboral | `diputados` |
| 50 | `lopdc` | Ley Orgánica de la Procuraduría de la Defensa del Contribuyente | `diputados` |
| 51 | `lopsrm` | Ley de Obras Públicas y Servicios Relacionados con las Mismas | `diputados` |
| 52 | `lpab` | Ley de Protección al Ahorro Bancario | `diputados` |
| 53 | `lpdusf` | Ley de Protección y Defensa al Usuario de Servicios Financieros | `diputados` |
| 54 | `lpue` | Ley de Puertos | `diputados` |
| 55 | `lrascap` | Ley para Regular las Actividades de las Sociedades Cooperativas de Ahorro y Préstamo | `diputados` |
| 56 | `lsat` | Ley del Servicio de Administración Tributaria | `diputados` |
| 57 | `lscs` | Ley sobre el Contrato de Seguro | `diputados` |
| 58 | `lsn` | Ley de Seguridad Nacional | `diputados` |
| 59 | `lsp` | Ley de Sistemas de Pagos | `diputados` |
| 60 | `lspcapf` | Ley del Servicio Profesional de Carrera en la Administración Pública Federal | `diputados` |
| 61 | `lspm` | Ley del Servicio Postal Mexicano | `diputados` |
| 62 | `lss` | Ley del Seguro Social | `diputados` |
| 63 | `luc` | Ley de Uniones de Crédito | `diputados` |
| 64 | `lvgc` | Ley de Vías Generales de Comunicación | `diputados` |
| 65 | `reg-art121-122-lft` | Reglamento de los Artículos 121 y 122 de la Ley Federal del Trabajo | `diputados` |
| 66 | `reg-ccomer-mpsc` | Reglamento del Código de Comercio en Materia de Prestadores de Servicios de Certificación | `diputados` |
| 67 | `reg-cff` | Reglamento del Código Fiscal de la Federación | `diputados` |
| 68 | `reg-laassp` | Reglamento de la Ley de Adquisiciones, Arrendamientos y Servicios del Sector Público | `diputados` |
| 69 | `reg-laat` | Reglamento de la Ley de Ayuda Alimentaria para los Trabajadores | `diputados` |
| 70 | `reg-ladua` | Reglamento de la Ley Aduanera | `diputados` |
| 71 | `reg-laero` | Reglamento de la Ley de Aeropuertos | `diputados` |
| 72 | `reg-lce` | Reglamento de la Ley de Comercio Exterior | `diputados` |
| 73 | `reg-lcec` | Reglamento de la Ley de Cámaras Empresariales y sus Confederaciones | `diputados` |
| 74 | `reg-ldcmpme` | Reglamento de la Ley para el Desarrollo de la Competitividad de la Micro, Pequeña y Mediana Empresa | `diputados` |
| 75 | `reg-lfaebsp` | Reglamento de la Ley Federal para la Administración y Enajenación de Bienes del Sector Público | `diputados` |
| 76 | `reg-lfcp` | Reglamento de la Ley Federal de Correduría Pública | `diputados` |
| 77 | `reg-lfcpq` | Reglamento de la Ley Federal para el Control de Precursores Químicos, Productos Químicos Esenciales y Máquinas para Elaborar Cápsulas, Tabletas y/o Comprimidos | `diputados` |
| 78 | `reg-lfda` | Reglamento de la Ley Federal del Derecho de Autor | `diputados` |
| 79 | `reg-lfjs` | Reglamento de la Ley Federal de Juegos y Sorteos | `diputados` |
| 80 | `reg-lfpc-art122` | Reglamento del Artículo 122 de la Ley Federal de Protección al Consumidor | `diputados` |
| 81 | `reg-lfpiorpi` | Reglamento de la Ley Federal para la Prevención e Identificación de Operaciones con Recursos de Procedencia Ilícita | `diputados` |
| 82 | `reg-lfppi` | Reglamento de la Ley Federal de Protección a la Propiedad Industrial | `diputados` |
| 83 | `reg-lgcc-rne` | Reglamento de la Ley General de Cambio Climático en Materia del Registro Nacional de Emisiones | `diputados` |
| 84 | `reg-lgeepa-anp` | Reglamento de la Ley General del Equilibrio Ecológico y la Protección al Ambiente en Materia de Áreas Naturales Protegidas | `diputados` |
| 85 | `reg-lgeepa-maaa` | Reglamento de la Ley General del Equilibrio Ecológico y la Protección al Ambiente en Materia de Autorregulación y Auditorías Ambientales | `diputados` |
| 86 | `reg-lgeepa-meia` | Reglamento de la Ley General del Equilibrio Ecológico y la Protección al Ambiente en Materia de Evaluación del Impacto Ambiental | `diputados` |
| 87 | `reg-lgeepa-mpcca` | Reglamento de la Ley General del Equilibrio Ecológico y la Protección al Ambiente en Materia de Prevención y Control de la Contaminación de la Atmósfera | `diputados` |
| 88 | `reg-lgeepa-mretc` | Reglamento de la Ley General del Equilibrio Ecológico y la Protección al Ambiente en Materia de Registro de Emisiones y Transferencia de Contaminantes | `diputados` |
| 89 | `reg-lgpc` | Reglamento de la Ley General de Protección Civil | `diputados` |
| 90 | `reg-lgs-investigacion` | Reglamento de la Ley General de Salud en Materia de Investigación para la Salud | `diputados` |
| 91 | `reg-lgs-msi` | Reglamento de la Ley General de Salud en Materia de Sanidad Internacional | `diputados` |
| 92 | `reg-lgtrata` | Reglamento de la Ley General para Prevenir, Sancionar y Erradicar los Delitos en Materia de Trata de Personas y para la Protección y Asistencia a las Víctimas de estos Delitos | `diputados` |
| 93 | `reg-lieps` | Reglamento de la Ley del Impuesto Especial sobre Producción y Servicios | `diputados` |
| 94 | `reg-linfonacot` | Reglamento de la Ley del Instituto del Fondo Nacional para el Consumo de los Trabajadores | `diputados` |
| 95 | `reg-linfonavit-mtai` | Reglamento de la Ley del Instituto del Fondo Nacional de la Vivienda para los Trabajadores en Materia de Transparencia y Acceso a la Información | `diputados` |
| 96 | `reg-lisr` | Reglamento de la Ley del Impuesto sobre la Renta | `diputados` |
| 97 | `reg-liva` | Reglamento de la Ley del Impuesto al Valor Agregado | `diputados` |
| 98 | `reg-lopsrm` | Reglamento de la Ley de Obras Públicas y Servicios Relacionados con las Mismas | `diputados` |
| 99 | `reg-lpue` | Reglamento de la Ley de Puertos | `diputados` |
| 100 | `reg-lss-macerf` | Reglamento de la Ley del Seguro Social en Materia de Afiliación, Clasificación de Empresas, Recaudación y Fiscalización | `diputados` |
| 101 | `reg-lss-maeba` | Reglamento de la Ley del Seguro Social, en materia de administración y enajenación de bienes adjudicados con motivo de la aplicación del procedimiento administrativo de ejecución | `diputados` |
| 102 | `reg-lss-rfar` | Reglamento de la Ley del Seguro Social para la constitución, inversión y uso para la operación de las Reservas Financieras y Actuariales y la Reserva General Financiera y Actuarial, así como para la determinación de los costos financieros que deberán reintegrarse a la Reserva de Operación para Contingencias y Financiamiento | `diputados` |
| 103 | `reg-pmimss` | Reglamento de Prestaciones Médicas del Instituto Mexicano del Seguro Social | `diputados` |
| 104 | `reg-senado` | Reglamento del Senado de la República | `diputados` |
| 105 | `servinv-dcg-2013` | Disposiciones de carácter general aplicables a las casas de bolsa e instituciones de crédito en materia de servicios de inversión | `itf-dcg` |
| 106 | `socap-sofipo-dcg-2006` | Disposiciones de carácter general aplicables a las entidades de ahorro y crédito popular, organismos de integración, sociedades financieras comunitarias y organismos de integración financiera rural | `itf-dcg` |

Coverage source SHA-256 values (recovered from the retained compressed archive):

- `coverage-manifest.json`: `8893e33b20a90ba10b0da2f684158d951b72153acf21f1757fc35eb8dbd608d3`
- `absent-search.json`: `0638f4bb89e6ea60a4e0b445dc5fa11e759a806f043fc61b232c058f18815383`
- `coverage-manifest-supplement.json`: `efe68af8c7f9028ceb630bc20915c63dd8c0982b914631ad2bf4d1ba2e0a5971`
