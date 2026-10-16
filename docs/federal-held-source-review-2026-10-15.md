# Federal original holds — source review and repair routing

Reviewed 2026-10-15 while the structural bulk-admission run proceeds. This is
the complete, frozen set of 14 campaign holds, not a list of new failures.
The campaign's accounting is 14 held identities; the separately flagged LFCE
warning is already present and is not a fifteenth hold.

## Scope and current state

The authority for each preserved reason is the frozen campaign inventory at
`.delta/worktrees/tf70fk1hzqdh/Lex-Mex/.work/campaigns/federal-text-2026-10-14/inventory.json`,
with the original human-facing list in
`.delta/worktrees/tf70fk1hzqdh/Lex-Mex/docs/federal-campaign-result-2026-10-14.md`.
Each item has an attempt receipt under
`.delta/worktrees/tf70fk1hzqdh/Lex-Mex/.work/campaigns/federal-text-2026-10-14/items/<slug>/attempt-1/`.
Those retained attempt directories contain setup/attempt receipts but no
candidate, source manifest, extracted text, or retained official source for
these holds.

At review time, none of the 14 exact slugs has
`corpus/mx/<slug>/instrument.json`. That is a membership check, not a legal
finding. `reg-lfiiedb` is already in the corpus; it is a distinct regulation
and does not clear the held law `lfiiedb`. The locally admitted `laem` and
`lamni` are unrelated to this set.

The recommendations below separate an intake/parser repair from an
operator/legal scope decision. A structural admission can retain
`not_analyzed` review state and `unknown` temporal status; it must still have
an identified official source, a complete deterministic parse, and a safe
instrument identity.

The nested-body counts, boundaries, and ownership descriptions below are
recorded diagnoses from `docs/ingestion-difficulty-log.md` and the frozen
campaign hold record. This review did not freshly verify those assertions
against an official source. They remain the inputs to a future source-backed
fixture and reviewed ownership decision.

## Clusters

| Cluster | Holds | What should happen first |
|---|---|---|
| Official-source acquisition | `consar-cuf-dcg-2018`, `cusf-dcg-2014`, `pld-lic115-dcg-2009`, `reg-lgs-insumos` | Locate the issuer's official compiled text or a formal DOF source, retain bytes and manifest, then run the ordinary parse gate. This is technical provenance work; no legal interpretation is needed to begin. | This are all 'circulares unicas' or 'disposiciones de caracter general' they are therciary level regulation emmitted by the CNBV. The are sourced form https://www.cnbv.gob.mx/paginas/normatividad.aspx [These are part of the Fintech Cluster but in reality we shoudl have a specific parcer for DCG with it's own rules - same for NOMs and NMXs later down the line]
| Nested enactment structure | `lcmopfih`, `lisipl`, `lfiiedb` | Define a reusable containing-instrument/nested-instrument boundary before writing parser support. The containing text may formally be a law while materially operating as a reform decree; the parser model must not force either legal kind. Clause ownership remains a canonical model decision. | confirmed all 3 are laws nested inside a decree, I think we should have a specific parecer for those. lcmopfih. LEY QUE ESTABLECE, REFORMA, ADICIONA Y DEROGA DIVERSAS DISPOSICIONES FISCALES Y QUE REFORMA OTRAS LEYES FEDERALES **ARTICULOS PRIMERO A VIGESIMO QUINTO.- ..........** CAPITULO XII CONTRIBUCION DE MEJORAS **ARTICULO VIGESIMO SEXTO**.- Se establece una contribución de mejoras por obras públicas, de
conformidad con las disposiciones con vigencia propia contenidas en la siguiente: **"LEY DE CONTRIBUCION DE MEJORAS POR OBRAS PUBLICAS FEDERALES DE INFRAESTRUCTURA HIDRAULICA.** Articulo VIGESIMO SEXTO is the one form the decree where the law is nested in. ARTICULOS VIGESIMO SEPTIMO A TRIGESIMO TERCERO.- .......... are the remaining decree articles. Then Transitorios are Law only the tell is **PRIMERO.- La presente Ley** esta ley. 
lispil: Also nested one, however this one is a bit different here the Decree is the Law,  Articulos Primero a Octavo are of the Decree then Articulo noveno is the relevant part where the tax is created it as only 7 articles expresed in ordinal numeral 1o a 7o then there is an Articulo Decimo of the Decree then the Transitorios are of the Decree which are of the law and they are the only ones. That is why the title is called Impuesto and not Ley only then in brackets is called (LEY QUE ESTABLECE, REFORMA Y ADICIONA LAS DISPOSICIONES RELATIVAS A DIVERSOS IMPUESTOS) Which is also the name of the decree but because it is uninutiable form that name the Impuesto which is the toral part is given. 
then lfiiedb: Classic nested, Articulo Primero is of the Decree it emits the Law. Then numeral are the Laws articles Article 1 to 141 the following Transitorio is of the Law Único. **La presente Ley** entrará en vigor el día siguiente al de su publicación en el Diario Oficial de la Federación. Then is the next article of the decree. Articulo Segundo.- Artículo Segundo.- Se reforman los artículos 2, párrafo primero, fracción XXIV Bis; 16, párrafo tercero; 17, párrafos octavo y noveno; 21, párrafo primero, fracción III, inciso a), párrafo segundo; 32, párrafos primero, segundo, quinto, sexto y séptimo; 50, párrafos primero, tercero, cuarto y sexto, y 108, párrafo primero; y se adicionan los artículos 2, párrafo primero, con una fracción XXXIX Bis; 9 Bis; 35 Bis, y 54, con un párrafo quinto, de la Ley Federal de Presupuesto y Responsabilidad Hacendaria, para quedar como sigue:... (this decree only has two articles they are incorported in the law because article segundo of the decree reforms and drogates laws relative to the one emited so the legislators just left is full in the compiltion) after come the Transitorios of the Decree here is the tale Primero. **El presente Decreto**, entrará en vigor el día siguiente al de su publicación en el Diario Oficial de la Federación.
| Current/historical scope or annual lifecycle | `egdf`, `lif-2026`, `pef-2026`, `lftr` | Decide whether the named identity belongs in the corpus and, for annual instruments, record its fiscal-year lifecycle without pretending to decide legal effect. | egdf is superseded by the constitution of Mexico City. lftr is also superseded by the LEY EN MATERIA DE TELECOMUNICACIONES Y RADIODIFUSIÓ The rest  belong because they define goverment income and spenditure. 
| Identity/reference policy | `lga`, `lgag`, `lcnbv` | Repair catalog identity mapping for the two Aguas/Archivos identities; make a reviewed reference-policy choice for the known historical LMV citation. | Go for repair the third is the CNBV organic law. 

## Per-hold routing

| Hold | Preserved issue | Classification | Suggested remedy | Operator/legal judgment? |
|---|---|---|---|---|
| `consar-cuf-dcg-2018` | “Official CONSAR compiled PDF URL not validated.” | Acquisition/provenance | Resolve a CONSAR-hosted compiled source or formal DOF publication chain; retain the response bytes, URL, retrieval time and SHA-256 before adapter work. Do not use an unofficial mirror as canonical text. | No, for source resolution. |
| `cusf-dcg-2014` | “Official CNSF page timed out/502 during source resolution; no official compiled PDF URL validated.” | Acquisition/provenance | Retry discovery against CNSF's official archive or DOF; once an official compiled source is retained, proceed through normal extraction. A transient 502 is not a parsing diagnosis. | No, for source resolution. |
| `egdf` | “Estatuto de Gobierno del Distrito Federal: transitional post-CDMX-Constitution status; confirm scope.” | Corpus-scope and temporal review | Decide whether the federal corpus includes this potential historical instrument. If yes, acquire and admit it with review state `not_analyzed`; conduct its constitutional-transition analysis separately with formal sources. | **Yes** — inclusion/scope decision. | Superseded by Mexico City Consitution is abrogated.
| `lcmopfih` | Recorded diagnosis: the actual 15-article law and its five closing transitories are quoted inside Article Twenty-Six of a 1990 containing instrument; the containing instrument has a different eight-clause transitory section. The wrapper is formally a law while materially a reform decree, and the current parser captures the wrapper. | Parser/model boundary | Add an explicit, fixture-backed containing-instrument/nested-law boundary model that does not force the containing instrument's legal kind: extract Articles 1–15 and the law-owned five transitories as `lcmopfih`; retain outer Article Twenty-Six and the outer eight transitories only as enactment evidence. Do not infer ownership from the first `TRANSITORIOS` heading. Source diagnosis: `docs/ingestion-difficulty-log.md` (`lcmopfih`, 2026-08-01). | **Yes** — approve canonical ownership/model before implementation. | This is now the 'classic' law nested insed a decree the law
| `lcnbv` | Article 15 really cites LMV Article 16 Bis 7, which is absent from the current LMV source and corpus; validation rejects the unresolved cross-instrument edge. | Reference-model policy | Keep the source citation. Adopt a corpus-wide, reviewed representation for a known historical/unavailable target — for example an explicit `historical_target_unavailable` edge/warning — and add a regression fixture. Do not silently delete the edge. Source diagnosis: `docs/ingestion-difficulty-log.md` (`lcnbv`, 2026-08-02). | **Yes** — reference-policy decision. | Yes the lcnbv is not 'upddated' that citation is stale. it should be `historical_superseded_reference_outdated`
| `lfiiedb` | The 2026 decree contains the 141-article law and law-owned `Transitorio Único`, quoted LFPRH reforms, then separate decree transitories; the parser mistakes quoted LFPRH article numbers for transitories. | Parser/model boundary | Partition the source at the enacted-law boundary: law Articles 1–141 plus `Único`; quoted LFPRH reform and decree transitories remain enactment/reform evidence. Fixture each boundary and relink already-admitted `reg-lfiiedb` only after the law exists. Source diagnosis: `docs/ingestion-difficulty-log.md` (2026-10-13). | **Yes** — approve law-versus-decree ownership model. | Law inside decree model.
| `lftr` | “2025 reform abrogated IFT; confirm current DOF title with reviewer before fetch.” Later planning identifies `lmtr`, Ley en Materia de Telecomunicaciones y Radiodifusión, as the candidate to assess. | Identity and scope verification | Verify the formal DOF basis and current official source before recording any replacement relation between `lftr` and `lmtr`. Keep `lftr` held pending that verification and a scope decision; no parser repair is indicated. | **Yes** — identity/scope decision. | Abrogated Law the current law is lmtr. because it is very new keep with an abrogated line.
| `lga` | Prepared `lga` meant Ley General de Archivos, while `batches/env_E3_water.json` used `lga` for Ley General de Aguas. | Catalog identity collision | Reserve `lga` for Ley General de Archivos and `lgag` for Ley General de Aguas, as the later cluster plan already states; update the stale water manifest and source inventory together, then acquire/parse each separately. | No legal judgment after the slug mapping is confirmed; this is metadata repair. |
| `lgag` | The Aguas hold was recorded under conflicting slug `lga`; catalog presence did not clear it. | Catalog identity collision | Resolve together with `lga`: move the Aguas source record, review notes and any source candidate to `lgag`; retain a migration note so old evidence remains findable. Then source/parse it normally. | No legal judgment for the identity migration; current-law scope should be verified from official source metadata. |
| `lif-2026` | “Annual instrument (expires with FY2026); confirm temporal-model treatment before ingesting.” | Annual lifecycle metadata | Treat annuality as structured provenance/lifecycle metadata, not a prerequisite for preparing official-source acquisition and deterministic parsing. Preserve the hold pending explicit clearance before admission; separately review commencement and end-of-fiscal-year effect. | **Yes** — clear the existing hold for admission, unless a later scope policy makes this automatic. | Keep it is the annual goverment income low it is updated every year but is very relevant.
| `lisipl` | Recorded diagnosis: the seven-article tax is quoted in Article Ninth of a 1968 containing instrument; its sole `TRANSITORIOS` section belongs to the containing instrument, not the tax. The wrapper is formally a law while materially a reform decree, and the parser captures only the wrapper. | Parser/model boundary | Reuse the legal-kind-neutral containing-instrument/nested-instrument solution, with a simpler profile than `lcmopfih`: extract the seven tax articles; preserve the containing instrument's transitories as enactment evidence and do not create tax transitories from them. Source diagnosis: `docs/ingestion-difficulty-log.md` (`lisipl`, 2026-08-01). | **Yes** — approve nested-boundary ownership model. | Nested model. 
| `pef-2026` | “Annual decree (expires with FY2026); confirm temporal-model treatment before ingesting.” | Annual lifecycle metadata | Same routing as `lif-2026`: prepare official source and deterministic structural parse first, while preserving the hold pending explicit clearance before admission; fiscal-year lifecycle and legal effects are separate review work. | **Yes** — clear the existing hold for admission, unless a later scope policy makes this automatic. | Yes it is called Prespuesto but it is a law for all practical purposes it satys. It is anual and it is the counter part to the lif-2026
| `pld-lic115-dcg-2009` | “CNBV Normatividad search exposed later agreements, but not the base compiled DCG PDF.” | Acquisition/provenance | Locate the base 2009 official compiled text through CNBV/SHCP or formal DOF sources; preserve the base-versus-later-agreement chain in the manifest. Do not synthesize a base source from later amendments. | No, for source resolution. | This is  DCG we can hold I think we should have a specific parecer for DCG. 
| `reg-lgs-insumos` | “Not listed in Diputados reglamentos-de-leyes index; needs official source resolution.” | Acquisition/provenance | Use the competent health/DOF publisher as the source-discovery target, retain the official source and manifest, then create a normal adapter. Absence from the Diputados index is not evidence of absence or repeal. | No, for source resolution. | Check offical DOF source.

## Sequencing recommendation

Start the four acquisition-only holds in parallel. They are the lowest-risk
path to enlarging the corpus, because their blocker is the lack of retained
official bytes rather than the parser. Resolve `lga`/`lgag` as one catalog
migration before fetching either source. Treat `lcmopfih`, `lisipl` and
`lfiiedb` as one scoped design task with fixtures, after the operator approves
the ownership model; that generic model must accommodate a containing act
without assigning it a legal kind. Keep `lcnbv`, `egdf`, and any historical `lftr` decision
out of that parser task because each changes corpus policy or legal scope.

Annuality alone need not block preparation of `lif-2026` and `pef-2026` for a
future structural admission. The current authorization covers the 225 passing
candidates only, so it does not clear either original hold: source acquisition
and parsing may be prepared, but admission requires an explicit hold-clearance
decision. Their annual lifecycle must be visible in provenance and later
review, rather than erased.

No source was fetched, no source text or canonical record was changed, and no
hold was cleared by this review.
