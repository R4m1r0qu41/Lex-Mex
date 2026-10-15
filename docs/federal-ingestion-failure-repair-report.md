# Federal ingestion failure repair — 2026-10-15

Reviewed all 128 frozen failures and 14 held identities. **126 original failures now pass retained-byte extraction, parsing, validation and isolated Markdown export. LMIGRA and OGA remain blocked on precise source-model decisions.** Regression replay of the original 99 provisional successes gives 98 passes and exposes LRCDN's previously omitted unnumbered transitory, which now fails explicitly.

All outputs remain isolated candidates. No canonical corpus admission, temporal-model run, human legal-review resolution, commit, push or deployment occurred. The 14 existing held identities remain held. Main started at `1a37ccde2e113e550d08a82bbead5620c9b6d868`; the two unrelated `nota-viabilidad-comites-banca-multiple` deletions and Word lock file remain untouched.

## Source set and reproducibility

The operator-annotated input is [the preserved 2026-10-14 report](../.delta/worktrees/tf70fk1hzqdh/Lex-Mex/docs/federal-campaign-result-2026-10-14.md). Its CJM, LAAM, LISSFAM, LGPIST, REG-LGEEPA-MOE and LMIGRA annotations were reviewed against the retained extraction and receipts. It remains unchanged in its evidence worktree. That worktree's uncommitted campaign code and documentation were not copied or edited.

The input evidence is `.delta/worktrees/tf70fk1hzqdh/Lex-Mex/.work/campaigns/federal-text-2026-10-14/`: frozen inventory, final report, attempts, official PDFs, extracted text, source manifests, furniture/structure/validation reports and original candidates. [The compact audit](federal-ingestion-failure-repair-evidence.json) records each failed identity, original acquisition hashes, source URL, old/new counts, stage receipts and final disposition; it also records all 14 holds and the 99-case regression summary. Links to publisher PDFs identify the retained acquisition; no later remote PDF was substituted.

[The replay harness](../prompts/replay-federal-ingestion-failures.py) uses the existing CLI `extract`, `parse`, `validate` and isolated Obsidian export. It copies a frozen executable and read-only evidence into separate output roots, checks PDF/text bytes against the original source manifest, and verifies that extraction reproduces the original bytes. It carries acquisition provenance, never old candidate/review files, into each replay. The campaign implementation is unnecessary for this bounded replay; no new campaign dependencies or crates were integrated.

Final local evidence roots are `.work/federal-repair-replay-v5/` and `.work/federal-repair-provisional-v5/`. Each case's `replay_root` in the audit identifies its `extract.log`, `parse.log`, `validate.log`, `export.log`, `candidate-diff.json`, source manifest, furniture audit, canonical candidate and `export/Corpus/<instrument>/` Markdown. `jobs.json` records the exact eight isolated shard commands. Earlier v1–v4 roots are diagnostic history, including corrected harness/export issues; their passes do not override final dispositions.

## Confirmed mechanisms and repairs

| Mechanism | Source-backed change and verification boundary |
|---|---|
| Closing signatures/promulgation | Recognize separate closing blocks by place/signature, exact presidential formula, older congressional role signatures, Palacio/Dado formulas and CJM's `Por tanto, mando…`. Stop after operative text; never classify the signatures as a transitory. Inline formula citations remain text; configured substantive-annex behavior is preserved. |
| Publisher title furniture | Remove up to three contiguous uppercase title lines immediately before the publisher's `CÁMARA…` header, plus adjacent Secretaría header metadata. This covers two-line titles and source/catalog spelling drift; it does not delete matching body headings or arbitrary uppercase text. Positional furniture audit remains enabled. |
| Numeric ordinal glyphs | The parser already recognized `º`/`°`; the strict sequential validator recognized only ASCII `o`. Accept the equivalent source ordinal glyphs without allowing gaps or freezing proposed counts. |
| Military slash ordinals | Recognize exact numeric `/o` suffixes, preserve printed labels, normalize canonical identity and escape filename separators for export. Negative tests preserve arbitrary `1/other` and `parte/o`; an actual Obsidian export test verifies both label and target filename. |
| Thousands labels | Article-base detection used only leading digits, treating `1,000` as `1` and discarding it as a backward quotation. Respect grouped thousands and validate grouping strictly. OGA's complete source set 1–1868 is verified separately below. |
| Wrapper false positives/omissions | A complete `Se deroga.` clause is a real ordinal article, not an amendment wrapper. Explicit `Se aprueba la Ley…` and `Se crea la Ley…` wrappers do not become duplicate article 1. |
| Publication-note variants | Recognize retained publisher forms missing `el`, missing `de` before year, and `en la Diario…`; collapse layout whitespace. Restrict original-publication lookup to text before the reform appendix so a later reform date cannot replace an unsupported original date/range. |
| Judicial appendix attribution | A court heading ends the current legislative-evidence entry and suspends legislative extraction until the next actual act heading. Preserve later reforms, rather than truncating the rest of the source or inventing occurrence-qualified IDs for judicial quotations. Exact court appendix text remains in source bytes and each replay's `judicial-appendix-review.json`; it is not converted into a legal determination. |
| Errata appendix | `FE DE ERRATAS` ends the main regulation before quoted before/after articles; it does not create duplicate transitory 68. The unchanged source retains the correction evidence. |
| Other heading forms | Recognize the exact extracted `. Artículo 40` prefix; accept `TRANSITORIOS.`; recognize original Roman I–V headings only after an established transitory section begins at I. Roman fractions inside an already named transitory remain body text. An unnumbered transitory body now fails explicitly instead of disappearing. |

Every material rule has regression coverage. [Fixture provenance](../fixtures/diputados/repair-provenance.md) binds 15 bounded source excerpts to official URLs and source/text hashes; additional boundary and negative cases are inline in Rust tests. Schemas, review-state semantics, validation severity and accepted legal decisions were not relaxed.

## Operator annotations and precise findings

**CJM.** The retained PDF has four original numeric transitories `1o`–`4o` (extraction lines 8792–8806, PDF page 134). After `4o` come `Por tanto, mando se imprima, publique, circule…`, the Palacio presidential block signed `A. L. Rodríguez`, the ministerial communication, a 29 August 1933 Gobernación signature, and the literal `Al C....`. These are source apparatus rather than additional transitories. All four clauses survive; the fourth ends at `si fueren mayores los que en éste se concedan.` The operator's future-linking note is preserved: **definitions are in article 434**, not a presumed article 1/4 glossary. No glossary configuration or historical presidential-status finding was inferred. [Retained-source fixture](../fixtures/diputados/repair-cjm-excerpt.txt).

**LAAM.** The operator correctly observed that the document ends with the normal publication formula. The defect was its attribution: the original candidate's `TERCERO` included congressional signatures and the entire presidential formula. The repaired clause ends at `hasta pasar a situación del retiro o causar baja en los términos que establecen las disposiciones aplicables.` Its source signature block begins `México, D.F., a 28 de abril de 2004…`; it belongs outside the clause. Reform evidence remains separately attributed. [Publisher PDF](https://www.diputados.gob.mx/LeyesBiblio/pdf/LAAM.pdf).

**LISSFAM.** The source really ends with `VERSIÓN pública de la Sentencia… Declaratoria General de Inconstitucionalidad 1/2021`, with a 16 December 2025 publication note (extraction line 5013 onward). The parser repair prevents that judicial appendix from entering legislative reform transitories. The replay preserves 231 articles, six original transitories and 26 legislative evidence items; the complete court appendix remains separately reviewable. LAREFAM and LOEFAM have similar court appendices followed by later actual reforms; those later 2025 reforms survive. Notification/publication dates and judicial legal effects require the existing audited review process and formal DOF evidence, not parser approval.

**LGPIST and REG-LGEEPA-MOE.** The two-line-header hypotheses are confirmed. LGPIST's repeated second band is `INHUMANOS O DEGRADANTES`; REG-LGEEPA-MOE's is `EN MATERIA DE ORDENAMIENTO ECOLÓGICO`. Original positional audits show these in multiple provisions. Publisher-adjacent removal clears the contamination while preserving body prose. LSINT additionally has a declaration-of-invalidity note sharing its Secretaría header line; header cleanup does not decide that legal status.

**REG-LFAFE and REG-LSM.** Successful validation initially concealed missing transitories. REG-LFAFE prints `TRANSITORIOS.` followed by `ARTICULO PRIMERO` through `ARTICULO OCTAVO`; the final candidate contains 99 articles and eight original transitories. REG-LSM prints `TRANSITORIOS:` followed by Roman `I.-` through `V.-`; the final candidate contains 255 articles and five original transitories. These are source-recognition repairs, not manufactured count baselines. [LFAFE fixture](../fixtures/diputados/repair-reg-lfafe-transitories.txt), [LSM fixture](../fixtures/diputados/repair-reg-lsm-transitories.txt).

**REG-LEPECFE.** The source has 82 articles and eight original transitories. Its appended 29 December 2025 errata quotes article 68 twice, first with `Petróleos Mexicanos`, then `Comisión Federal de Electricidad`. The main consolidated article 68 already carries the latter wording and its errata note. The parser does not apply a new correction or reinterpret its legal effect; it stops before the quoted correction appendix and retains the unchanged official source evidence.

**LGAPPCEA and the complete removed-identity audit.** Its source has six original law transitories. After those and the enactment signatures, extraction lines 617–625 print `PUNTOS RESOLUTIVOS DE SENTENCIA DE LA SCJN` and `SENTENCIA dictada… Acción de Inconstitucionalidad 33/2015`. Under that court text's `CONSIDERANDO`, line 650 begins `SÉPTIMO. Efectos.` The old parser mislabeled this judicial reasoning as `urn:lex-mx:federal:statute:lgappcea:transitory:septimo`. Removing that false seventh law clause preserves all six actual original transitories; the full judicial source remains in `judicial-appendix-review.json`. The only other removed identity across the original 128 is REG-LEPECFE's false `transitory:68`, represented twice by the errata's before/after article quotations described above. LFDEFP/LGDLPI lose duplicate wrapper records while retaining each real article 1 identity. No other original candidate provision IDs were removed. [LGAPPCEA publisher PDF](https://www.diputados.gob.mx/LeyesBiblio/pdf/LGAPPCEA_270516.pdf).

## Remaining two failed identities: exact source questions

**LMIGRA — two explicit enactment-transitory layers, not a temporal-analysis failure.** The campaign's `CAMPAIGN_NESTED_LAW_STRUCTURE` symptom label is too broad. The immediate failure is the parser's restriction on two separately headed transitory layers, not a finding that a second law was inserted into the law's operative text. The operator's conditional-commencement observation is supported and is a separate issue.

The law's original section begins at extraction line 3056 (PDF pages 48–49):

> TRANSITORIOS
>
> PRIMERO. La Ley de Migración entrará en vigor al día siguiente de su publicación en el Diario Oficial de la Federación, excepto las disposiciones que están sujetas a la vacancia prevista en el artículo Segundo Transitorio.

Its `SEGUNDO` enumerates provisions that enter force `hasta que se encuentre vigente el Reglamento de la misma Ley.` After the law's tenth clause, extraction lines 3136–3158 (PDF pages 49–50) print:

> ARTÍCULOS SEGUNDO A NOVENO. ……….
>
> TRANSITORIOS DEL DECRETO POR EL QUE EXPIDE LA LEY DE MIGRACIÓN Y SE REFORMAN, DEROGAN Y ADICIONAN DIVERSAS DISPOSICIONES…

That second section has its own `PRIMERO`–`CUARTO`. Its `SEGUNDO` concerns other laws' amendments, including their dependency on the Reglamento. Flattening it into the law would duplicate ordinals and attribution. **Required decision:** preserve the law's ten original clauses and represent the four enclosing-decree clauses in a separate evidence namespace/type, or approve another explicit attribution model. Independently, resolving the Reglamento commencement condition requires its formal official act and audited legal review; the parser did not resolve it. [Publisher PDF](https://www.diputados.gob.mx/LeyesBiblio/pdf/LMigra.pdf).

-- This is the case of law nested inside a decree. ARTICULO PRIMERO is of that decree. Se expide la Ley de Migración. Then Articulo 1 is the first article of the Law. Then the Transitorios after Articulo 162 are the  Law Transitorios, Primero in-force the next day, SEGUNDO.- Certain parts in force until the regulation is published (it has been https://www.diputados.gob.mx/LeyesBiblio/regley/Reg_LMigra.pdf - same structure of nested law inside a decree) TERCERO gives the executive branch a term to publish the regulation (this is a common transitorio, HOWEVER that does not mean the executive complied with the deadline, may times the executive doesn't there is or used to be a page of the Senate or Diputados with a list the non-compliant regulations the executive hasn't published yet, these are regulation that order in transitorios and pending publication) it goes on until DECIMO these are all from the law. then ARTICULOS SEGUNDO A NOVENO are from the decree the are not visible in the PDF but they certainly contain REFORMAN, DEROGAN Y ADICIONAN DIVERSAS DISPOSICIONES

Also this surfaces 2 things the parsing of the law itself with the transitorios which I have established are pere decree and per law. so in this case the transitorios of the law. A few precision that might help. All transitorios are time-bound they are meant to address the vactio legis issue, meaning not having a period of time when no law applies. so they state when the law or reform goes into force if it is dergoating anything and from where and some times it applies a conditional for it to go into force like the publishing of a reglamento. But once enacted the transitorios goes "stale" in the sense that other than determining when things go into force, they are no longer applicable because they are now old and have been applied, this is a bit tricky because there is no universal rule but a good example is TRANSITORIO SEXTO of Law describes equivalences in migratory permits, they apply to permits issued under the older law, they will apply insofar as those permits exists, however it is reasonable to think there are might be very few or none older permits since tha law was published in 2011 and they usually have a 10 year period, but if there are any the equivalence still applys. Another is TRANSITORIO QUINTO. This is completely stale is is about the budget needed to enact the law, budgets are annual an it is just a mention whatever needs to be enacted must fit within the annual federal budget. Articulo NOVENO is another, is states that the filings initiated before the entering into force of this law will be resolved using the laws in force at the time of the filing so anything pre 2011 the older law is applicable.

**OGA — serialized publication and an unnumbered original transitory.** Its first source note says:

> Nueva Ordenanza publicada en el Diario Oficial de la Federación en forma seriada los días 1º al 8 de enero de 1912

The generic first-match date lookup previously skipped that unsupported range and selected 14 January 1985 from the *reform* appendix. The new original-date boundary fails closed; no 1912 day or 1985 original date is fabricated. [Range regression fixture](../fixtures/diputados/repair-oga-publication-range.txt).

At extraction lines 13224–13227 (PDF page 208), the source has one unnumbered clause:

> TRANSITORIO
>
> La presente Ordenanza comenzará a surtir sus efectos desde el primero de febrero de mil novecientos doce, quedando derogadas las Leyes y disposiciones que en todo o en parte se opongan a ella.

**Required decisions:** represent the serialized original publication without forcing a false single-day metadata claim; choose a source-faithful internal identity for the unlabeled clause. Current date-only metadata and ordinal-based identities do not express these safely. A regression also makes an unnumbered clause an explicit error rather than silent loss.

The separate thousands fix is supported: a direct source-heading audit finds every article 1–1868 exactly once. An intermediate diagnostic candidate recovered all 1868 article IDs, compared with the prior 1000-article output ending at 1290 (**868 separately parsed articles restored**). That diagnostic used the old wrong date and is **not an admissible candidate**. Final OGA replay fails on missing original-publication metadata; its recovered article count is reported only as parser evidence. [Thousands fixture](../fixtures/diputados/repair-oga-excerpt.txt), [publisher PDF](https://www.diputados.gob.mx/LeyesBiblio/pdf/218.pdf).

-- This is an outlier, the Ordenanza was originaly published in a series in 1912 very old, but it is clearly still in force last reform made in 2000 governs military rankings. my gues is this is the only doc that has that serialization I would make an explicit exception for it. It also has a useful example the REGLAMENTO de honores, banderas y luces de la Armada de México.
Publicado en el Diario Oficial de la Federación el 19 de octubre de 2000 was a regulation that derogated some articles in this ordenanza, this is a common issue and one that is hard to track, hopefully lex-mex will eventually be able to deterministically track them.

## Additional provisional source-identity blocker

**LRCDN** previously passed while omitting its unnumbered original clause. Extraction lines 311–314 (PDF page 5) print:

> ARTICULO TRANSITORIO:
>
> La presente ley entrará en vigor al día siguiente de su publicación en el "Diario Oficial" de la Federación.

The new guard rejects silent omission. This is a newly exposed source-identity question from the 99-case regression set, separate from the original 128 failures and 14 carried holds. Preserve its 31 operative articles and choose an explicit internal identity for this source-labeled, unnumbered clause before re-export/admission. No legal commencement decision was inferred. [Publisher PDF](https://www.diputados.gob.mx/LeyesBiblio/pdf/LRCDN.pdf).

-- This one should have said UNICO in the original transitorios it only state ARTICULO TRANSITORIO in singular, it was an error in legislative technique.


## Every original failure

The counts below are observed candidate counts, never approved/frozen baselines. `A/T` means articles/original transitories. `—` means the original failed parse left no candidate. `Pass` means retained-byte extraction, parse, enabled furniture audit, canonical validation, original signature-span inspection and isolated Markdown export passed; it does not mean legal or source-completeness approval. The two `Judgment` cases remain blocked as explained above. Per-case logs, source hashes and differences are in the linked compact audit and local replay roots.

| Identity | Original gate | A/T before → final | Disposition and mechanism |
|---|---|---:|---|
| `cjm` | `SOURCE_BOUNDARY_SIGNATURE` | 952/4 → 952/4 | Pass — old promulgation boundary; all 4 numeric clauses retained; article 434 linking note preserved |
| `laam` | `SOURCE_BOUNDARY_SIGNATURE` | 57/3 → 57/3 | Pass — closing signature/promulgation apparatus excluded |
| `laem` | `SOURCE_BOUNDARY_SIGNATURE` | 17/5 → 17/5 | Pass — closing signature/promulgation apparatus excluded |
| `lagra` | `SOURCE_BOUNDARY_SIGNATURE` | 201/8 → 201/8 | Pass — closing signature/promulgation apparatus excluded |
| `lamn` | `SOURCE_BOUNDARY_SIGNATURE` | 9/5 → 9/5 | Pass — closing signature/promulgation apparatus excluded |
| `lamni` | `SOURCE_BOUNDARY_SIGNATURE` | 4/2 → 4/2 | Pass — closing signature/promulgation apparatus excluded |
| `lansi` | `FURNITURE_CONTAMINATION` | — → 38/9 | Pass — publisher-adjacent title bands removed |
| `larefam` | `PARSE_FAILED` | — → 78/4 | Pass — judicial quotations separated; later 2025 reforms retained |
| `lasoc` | `SOURCE_BOUNDARY_SIGNATURE` | 68/3 → 68/3 | Pass — closing signature/promulgation apparatus excluded |
| `lcaaa` | `PARSE_FAILED` | — → 19/2 | Pass — appendix law-title attribution |
| `lcacsam` | `SOURCE_BOUNDARY_SIGNATURE` | 25/1 → 25/1 | Pass — closing signature/promulgation apparatus excluded |
| `lcacsefam` | `SOURCE_BOUNDARY_SIGNATURE` | 37/4 → 37/4 | Pass — closing signature/promulgation apparatus excluded |
| `lcid` | `SOURCE_BOUNDARY_SIGNATURE` | 42/9 → 42/9 | Pass — closing signature/promulgation apparatus excluded |
| `lcjpjf` | `SOURCE_BOUNDARY_SIGNATURE` | 74/5 → 74/5 | Pass — closing signature/promulgation apparatus excluded |
| `lcndh` | `SOURCE_BOUNDARY_SIGNATURE` | 86/8 → 86/8 | Pass — closing signature/promulgation apparatus excluded |
| `lcnp` | `VALIDATION_INVALID` | 3/0 → 4/0 | Pass — complete Se deroga. article 2 retained; closing apparatus excluded |
| `lcpro` | `SOURCE_BOUNDARY_SIGNATURE` | 22/4 → 22/4 | Pass — closing signature/promulgation apparatus excluded |
| `ldpam` | `SOURCE_BOUNDARY_SIGNATURE` | 52/6 → 52/6 | Pass — closing signature/promulgation apparatus excluded |
| `ldparm` | `SOURCE_BOUNDARY_SIGNATURE` | 80/2 → 80/2 | Pass — closing signature/promulgation apparatus excluded |
| `ldrs` | `SOURCE_BOUNDARY_SIGNATURE` | 193/10 → 193/10 | Pass — closing signature/promulgation apparatus excluded |
| `ldsca` | `SOURCE_BOUNDARY_SIGNATURE` | 150/8 → 150/8 | Pass — closing signature/promulgation apparatus excluded |
| `lebhn` | `SOURCE_BOUNDARY_SIGNATURE` | 70/3 → 70/3 | Pass — closing signature/promulgation apparatus excluded |
| `lec` | `SOURCE_BOUNDARY_SIGNATURE` | 14/3 → 14/3 | Pass — closing signature/promulgation apparatus excluded |
| `lei` | `SOURCE_BOUNDARY_SIGNATURE` | 38/2 → 38/2 | Pass — closing signature/promulgation apparatus excluded |
| `lemefam` | `SOURCE_BOUNDARY_SIGNATURE` | 52/2 → 52/2 | Pass — closing signature/promulgation apparatus excluded |
| `len` | `SOURCE_BOUNDARY_SIGNATURE` | 24/4 → 24/4 | Pass — closing signature/promulgation apparatus excluded |
| `less` | `SOURCE_BOUNDARY_SIGNATURE` | 58/6 → 58/6 | Pass — closing signature/promulgation apparatus excluded |
| `lfafe` | `SOURCE_BOUNDARY_SIGNATURE` | 125/8 → 125/8 | Pass — closing signature/promulgation apparatus excluded |
| `lfcsq` | `FURNITURE_CONTAMINATION` | — → 75/11 | Pass — publisher-adjacent title bands removed |
| `lfdeapd` | `SOURCE_BOUNDARY_SIGNATURE` | 33/3 → 33/3 | Pass — closing signature/promulgation apparatus excluded |
| `lfdefp` | `VALIDATION_INVALID` | 43/7 → 42/7 | Pass — Se aprueba la Ley wrapper excluded |
| `lffaosc` | `SOURCE_BOUNDARY_SIGNATURE` | 32/6 → 32/6 | Pass — closing signature/promulgation apparatus excluded |
| `lffpmn` | `SOURCE_BOUNDARY_SIGNATURE` | 13/6 → 13/6 | Pass — closing signature/promulgation apparatus excluded |
| `lfgr` | `SOURCE_BOUNDARY_SIGNATURE` | 101/16 → 101/16 | Pass — closing signature/promulgation apparatus excluded |
| `lfll` | `SOURCE_BOUNDARY_SIGNATURE` | 28/5 → 28/5 | Pass — closing signature/promulgation apparatus excluded |
| `lfm` | `SOURCE_BOUNDARY_SIGNATURE` | 65/4 → 65/4 | Pass — closing signature/promulgation apparatus excluded |
| `lfmped` | `SOURCE_BOUNDARY_SIGNATURE` | 25/2 → 25/2 | Pass — closing signature/promulgation apparatus excluded |
| `lfmzaah` | `SOURCE_BOUNDARY_SIGNATURE` | 62/4 → 62/4 | Pass — closing signature/promulgation apparatus excluded |
| `lfpdppp` | `SOURCE_BOUNDARY_SIGNATURE` | 64/20 → 64/20 | Pass — closing signature/promulgation apparatus excluded |
| `lfped` | `SOURCE_BOUNDARY_SIGNATURE` | 105/5 → 105/5 | Pass — closing signature/promulgation apparatus excluded |
| `lfppcpcia` | `FURNITURE_CONTAMINATION` | — → 75/5 | Pass — publisher-adjacent title bands removed |
| `lfpsdmh` | `SOURCE_BOUNDARY_SIGNATURE` | 24/5 → 24/5 | Pass — closing signature/promulgation apparatus excluded |
| `lfsa` | `SOURCE_BOUNDARY_SIGNATURE` | 177/10 → 177/10 | Pass — closing signature/promulgation apparatus excluded |
| `lfsp` | `SOURCE_BOUNDARY_SIGNATURE` | 50/8 → 50/8 | Pass — closing signature/promulgation apparatus excluded |
| `lfsv` | `SOURCE_BOUNDARY_SIGNATURE` | 97/5 → 97/5 | Pass — closing signature/promulgation apparatus excluded |
| `lfvv` | `SOURCE_BOUNDARY_SIGNATURE` | 48/6 → 48/6 | Pass — closing signature/promulgation apparatus excluded |
| `lgaas` | `SOURCE_BOUNDARY_SIGNATURE` | 111/7 → 111/7 | Pass — closing signature/promulgation apparatus excluded |
| `lgahotdu` | `SOURCE_BOUNDARY_SIGNATURE` | 119/15 → 119/15 | Pass — closing signature/promulgation apparatus excluded |
| `lgamvlv` | `SOURCE_BOUNDARY_SIGNATURE` | 101/8 → 101/8 | Pass — closing signature/promulgation apparatus excluded |
| `lgappcea` | `SOURCE_BOUNDARY_SIGNATURE` | 18/7 → 18/6 | Pass — six original clauses retained; false SCJN considerando SÉPTIMO excluded |
| `lgb` | `SOURCE_BOUNDARY_SIGNATURE` | 45/4 → 45/4 | Pass — closing signature/promulgation apparatus excluded |
| `lgcdc` | `SOURCE_BOUNDARY_SIGNATURE` | 42/6 → 42/6 | Pass — closing signature/promulgation apparatus excluded |
| `lgcfd` | `SOURCE_BOUNDARY_SIGNATURE` | 157/11 → 157/11 | Pass — closing signature/promulgation apparatus excluded |
| `lgcs` | `SOURCE_BOUNDARY_SIGNATURE` | 45/8 → 45/8 | Pass — closing signature/promulgation apparatus excluded |
| `lgdlpi` | `VALIDATION_INVALID` | 26/8 → 25/8 | Pass — Se crea la Ley wrapper excluded |
| `lgdnna` | `PARSE_FAILED` | — → 181/13 | Pass — actual publication notes without el recognized |
| `lgds` | `SOURCE_BOUNDARY_SIGNATURE` | 87/6 → 87/6 | Pass — closing signature/promulgation apparatus excluded |
| `lge` | `SOURCE_BOUNDARY_SIGNATURE` | 183/17 → 183/17 | Pass — closing signature/promulgation apparatus excluded |
| `lges` | `SOURCE_BOUNDARY_SIGNATURE` | 77/21 → 77/21 | Pass — closing signature/promulgation apparatus excluded |
| `lgimh` | `SOURCE_BOUNDARY_SIGNATURE` | 52/2 → 52/2 | Pass — closing signature/promulgation apparatus excluded |
| `lgipd` | `SOURCE_BOUNDARY_SIGNATURE` | 60/7 → 60/7 | Pass — closing signature/promulgation apparatus excluded |
| `lgipe` | `SOURCE_BOUNDARY_SIGNATURE` | 542/25 → 542/25 | Pass — closing signature/promulgation apparatus excluded |
| `lgmde` | `SOURCE_BOUNDARY_SIGNATURE` | 29/7 → 29/7 | Pass — closing signature/promulgation apparatus excluded |
| `lgmdfp` | `FURNITURE_CONTAMINATION` | — → 188/21 | Pass — publisher-adjacent title bands removed |
| `lgmhcti` | `SOURCE_BOUNDARY_SIGNATURE` | 100/22 → 100/22 | Pass — closing signature/promulgation apparatus excluded |
| `lgp` | `SOURCE_BOUNDARY_SIGNATURE` | 165/4 → 165/4 | Pass — closing signature/promulgation apparatus excluded |
| `lgpas` | `SOURCE_BOUNDARY_SIGNATURE` | 167/10 → 167/10 | Pass — closing signature/promulgation apparatus excluded |
| `lgpisdme` | `FURNITURE_CONTAMINATION` | — → 43/10 | Pass — publisher-adjacent title bands removed |
| `lgpist` | `FURNITURE_CONTAMINATION` | — → 96/16 | Pass — publisher-adjacent title bands removed |
| `lgpp` | `SOURCE_BOUNDARY_SIGNATURE` | 97/9 → 97/9 | Pass — closing signature/promulgation apparatus excluded |
| `lgpsacdii` | `SOURCE_BOUNDARY_SIGNATURE` | 76/8 → 76/8 | Pass — closing signature/promulgation apparatus excluded |
| `lgpsdms` | `FURNITURE_CONTAMINATION` | — → 48/12 | Pass — publisher-adjacent title bands removed |
| `lgpsvd` | `SOURCE_BOUNDARY_SIGNATURE` | 30/4 → 30/4 | Pass — closing signature/promulgation apparatus excluded |
| `lgsc` | `SOURCE_BOUNDARY_SIGNATURE` | 123/4 → 123/4 | Pass — closing signature/promulgation apparatus excluded |
| `lgscmm` | `SOURCE_BOUNDARY_SIGNATURE` | 107/24 → 107/24 | Pass — closing signature/promulgation apparatus excluded |
| `lih` | `SOURCE_BOUNDARY_SIGNATURE` | 64/2 → 64/2 | Pass — closing signature/promulgation apparatus excluded |
| `limj` | `SOURCE_BOUNDARY_SIGNATURE` | 20/4 → 20/4 | Pass — closing signature/promulgation apparatus excluded |
| `linbal` | `VALIDATION_INVALID` | 16/3 → 16/3 | Pass — equivalent source ordinal glyphs validated sequentially |
| `linpi` | `SOURCE_BOUNDARY_SIGNATURE` | 29/9 → 29/9 | Pass — closing signature/promulgation apparatus excluded |
| `lissfam` | `SOURCE_BOUNDARY_SIGNATURE` | 231/6 → 231/6 | Pass — closing apparatus removed; court appendix separately retained for legal review |
| `lmigra` | `NESTED_LAW_STRUCTURE` | — → — | Judgment — explicit law/decree transitory layers; no flattening or temporal decision |
| `lmin` | `SOURCE_BOUNDARY_SIGNATURE` | 79/12 → 79/12 | Pass — closing signature/promulgation apparatus excluded |
| `loefam` | `PARSE_FAILED` | — → 233/4 | Pass — judicial separation, military /o headings and safe filenames |
| `log` | `SOURCE_BOUNDARY_SIGNATURE` | 26/6 → 26/6 | Pass — closing signature/promulgation apparatus excluded |
| `loipn` | `SOURCE_BOUNDARY_SIGNATURE` | 34/4 → 34/4 | Pass — closing signature/promulgation apparatus excluded |
| `lopgjdf` | `SOURCE_BOUNDARY_SIGNATURE` | 84/6 → 84/6 | Pass — closing signature/promulgation apparatus excluded |
| `lota` | `SOURCE_BOUNDARY_SIGNATURE` | 30/5 → 30/5 | Pass — closing signature/promulgation apparatus excluded |
| `louaaan` | `PARSE_FAILED` | — → 26/4 | Pass — original publication note missing de before year recognized |
| `louam` | `SOURCE_BOUNDARY_SIGNATURE` | 37/6 → 37/6 | Pass — closing signature/promulgation apparatus excluded |
| `lounam` | `VALIDATION_INVALID` | 18/8 → 18/8 | Pass — equivalent source ordinal glyphs validated sequentially |
| `lpeam` | `SOURCE_BOUNDARY_SIGNATURE` | 26/4 → 26/4 | Pass — closing signature/promulgation apparatus excluded |
| `lperc` | `SOURCE_BOUNDARY_SIGNATURE` | 145/3 → 145/3 | Pass — closing signature/promulgation apparatus excluded |
| `lpo` | `SOURCE_BOUNDARY_SIGNATURE` | 50/5 → 50/5 | Pass — closing signature/promulgation apparatus excluded |
| `lppddhp` | `SOURCE_BOUNDARY_SIGNATURE` | 67/14 → 67/14 | Pass — closing signature/promulgation apparatus excluded |
| `lram` | `SOURCE_BOUNDARY_SIGNATURE` | 71/3 → 71/3 | Pass — closing signature/promulgation apparatus excluded |
| `lrart3-mmce` | `FURNITURE_CONTAMINATION` | — → 68/14 | Pass — publisher-adjacent title bands removed |
| `lrart5-prof` | `FURNITURE_CONTAMINATION` | — → 73/22 | Pass — publisher-adjacent title bands removed |
| `lrmnyut` | `PARSE_FAILED` | — → 16/1 | Pass — appendix law-title attribution |
| `lrpv` | `SOURCE_BOUNDARY_SIGNATURE` | 27/4 → 27/4 | Pass — closing signature/promulgation apparatus excluded |
| `lsem` | `SOURCE_BOUNDARY_SIGNATURE` | 86/6 → 86/6 | Pass — closing signature/promulgation apparatus excluded |
| `lsint` | `FURNITURE_CONTAMINATION` | — → 34/5 | Pass — publisher-adjacent invalidity note removed as furniture; no legal status inferred |
| `lsm` | `VALIDATION_INVALID` | 65/7 → 65/7 | Pass — equivalent source ordinal glyphs validated sequentially |
| `lsnieg` | `SOURCE_BOUNDARY_SIGNATURE` | 134/17 → 134/17 | Pass — closing signature/promulgation apparatus excluded |
| `lsniimsp` | `SOURCE_BOUNDARY_SIGNATURE` | 51/6 → 51/6 | Pass — closing signature/promulgation apparatus excluded |
| `lsprem` | `SOURCE_BOUNDARY_SIGNATURE` | 32/45 → 32/45 | Pass — closing signature/promulgation apparatus excluded |
| `lsrlip` | `VALIDATION_INVALID` | 13/0 → 13/0 | Pass — equivalent source ordinal glyphs validated sequentially |
| `luach` | `SOURCE_BOUNDARY_SIGNATURE` | 16/5 → 16/5 | Pass — closing signature/promulgation apparatus excluded |
| `luefa` | `SOURCE_BOUNDARY_SIGNATURE` | 7/3 → 7/3 | Pass — closing signature/promulgation apparatus excluded |
| `lupdecr` | `SOURCE_BOUNDARY_SIGNATURE` | 20/2 → 20/2 | Pass — closing signature/promulgation apparatus excluded |
| `lviv` | `SOURCE_BOUNDARY_SIGNATURE` | 97/5 → 97/5 | Pass — closing signature/promulgation apparatus excluded |
| `lvzmm` | `SOURCE_BOUNDARY_SIGNATURE` | 68/4 → 68/4 | Pass — closing signature/promulgation apparatus excluded |
| `oga` | `PARSE_FAILED` | — → — | Judgment — serialized original publication unsupported; unnumbered original clause; 1868 articles recovered diagnostically only |
| `reg-lagra-mcdets` | `VALIDATION_INVALID` | 68/2 → 68/2 | Pass — equivalent source ordinal glyphs validated sequentially |
| `reg-larefagn` | `FURNITURE_CONTAMINATION` | — → 153/3 | Pass — publisher-adjacent title bands removed |
| `reg-ldrs-moirsse` | `FURNITURE_CONTAMINATION` | — → 46/5 | Pass — publisher-adjacent title bands removed |
| `reg-lemefagn` | `FURNITURE_CONTAMINATION` | — → 180/2 | Pass — publisher-adjacent title bands removed |
| `reg-lepecfe` | `VALIDATION_INVALID` | 82/10 → 82/8 | Pass — errata quotations kept outside 82 articles/8 original clauses |
| `reg-lfafe` | `PARSE_FAILED` | — → 99/8 | Pass — source en la Diario note and TRANSITORIOS. recognized; 8 original clauses recovered |
| `reg-lfcine` | `VALIDATION_INVALID` | 71/6 → 72/6 | Pass — exact stray-dot prefix before article 40 recognized |
| `reg-lftaipg` | `FURNITURE_CONTAMINATION` | — → 95/11 | Pass — publisher-adjacent title bands removed |
| `reg-lftr-mcsre` | `FURNITURE_CONTAMINATION` | — → 28/3 | Pass — publisher-adjacent title bands removed |
| `reg-lgeepa-moe` | `FURNITURE_CONTAMINATION` | — → 79/4 | Pass — publisher-adjacent title bands removed |
| `reg-lgpsacdii` | `FURNITURE_CONTAMINATION` | — → 66/6 | Pass — publisher-adjacent title bands removed |
| `reg-lgs-mcsotcsh` | `FURNITURE_CONTAMINATION` | — → 136/3 | Pass — publisher-adjacent title bands removed |
| `reg-lgs-mcspiumc` | `FURNITURE_CONTAMINATION` | — → 80/4 | Pass — publisher-adjacent title bands removed |
| `reg-lrart5c` | `FURNITURE_CONTAMINATION` | — → 102/14 | Pass — publisher-adjacent title bands removed |
| `reg-lrmineras` | `VALIDATION_INVALID` | 27/1 → 27/1 | Pass — equivalent source ordinal glyphs validated sequentially |
| `reg-lsm` | `FURNITURE_CONTAMINATION` | — → 255/5 | Pass — source title furniture and 5 original Roman clauses recovered |

## All 14 existing holds

These are source/identity/model/reviewer holds carried forward from frozen receipts. No blind retry, legal approval or source re-resolution was inferred.

| Identity | Preserved hold | Required next action |
|---|---|---|
| `consar-cuf-dcg-2018` | Compiled official PDF not validated | Resolve and retain official CONSAR compiled source before retry. |
| `cusf-dcg-2014` | Official source resolution timed out/502 | Resolve and retain official CNSF compiled source; catalog presence is insufficient. |
| `egdf` | Post-CDMX-Constitution transitional scope | Operator/reviewer confirms historical/current scope before ingestion. |
| `lcmopfih` | Own law versus containing 1990 act transitories | Decide separate law/containing-act attribution; preserve the 15 law articles and both source layers. |
| `lcnbv` | Current LMV lacks cited article 16 Bis 7 | Reviewer chooses historical-reference treatment or provides official target evidence; never drop/resolve silently. |
| `lfiiedb` | Nested law, quoted LFPRH reforms and enclosing decree | Approve an explicit partition/identity model; keep quoted LFPRH material outside the law’s own provisions. |
| `lftr` | Post-2025 regulator/title verification | Confirm current official identity/title and legal scope from formal sources. |
| `lga` | Archivos/Aguas slug-title collision | Reconcile stable Archivos identity and quarantined Aguas mapping before retry. |
| `lgag` | Aguas historical hold under conflicting lga identity | Reconcile jointly with lga; do not treat separate catalog entry as clearance. |
| `lif-2026` | Annual instrument scope | Operator confirms annual-instrument identity/lifecycle treatment. |
| `lisipl` | Tax nested in Article NOVENO of 1968 act | Decide own versus outer transitory attribution; do not import the containing act blindly. |
| `pef-2026` | Annual budget-decree scope | Operator confirms annual-budget identity/lifecycle treatment. |
| `pld-lic115-dcg-2009` | Official base compiled DCG unresolved | Resolve and retain official base compiled text before adapter work/retry. |
| `reg-lgs-insumos` | Official source resolution unresolved | Resolve and validate official source bytes before retry. |

## Regression inspection and validation

The final frozen executable SHA-256 is `b65a6806c02d8e897d7a99c0471c9137ed278d99aa3e48e7219622a92278473e` across all 227 cases. All 227 PDF/text pairs matched their original acquisition manifests, all regenerated source bytes matched the frozen text, and all acquisition identity checks passed. No network fetch occurred.

The original 99 provisional cases replayed as 98 passes plus the newly exposed LRCDN unnumbered-clause blocker. Of the 98 successful candidates, 16 instruments have 30 changed existing provision texts. Their deltas were inspected: publisher-band/closing-apparatus removal, plus correctly moving five LPF and one LSSS original transitories out of the last article into separate clauses. No prior provision IDs or legislative reform-evidence IDs were lost. REG-LFSV retains all 193 articles and 11 clauses; its lowercase wrapped `transitorio.` is body prose. A regression fixture prevents the initially discovered false section boundary. LRCDN's failed candidate is not exported or represented as a clean regression pass.

Exact final implementation checks, delegated to `mechanical`, all passed in `.work/federal-repair-checks-v8/`:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run --locked -p lex-cli -- validate lritf
cargo run --locked -p lex-cli -- validate ifpe-dcg-2021
```

Workspace tests: 238 passed, 0 failed. LRITF: 145 articles, 11 transitories, 126 references, 0 issues. IFPE: 59 articles, 4 transitories, 113 references, 0 issues. `cargo fmt` and `cargo build --locked -p lex-cli` also passed. Earlier logged failures were diagnosed and corrected; v8 is the final gate. The Python harness was syntax-checked and exercised by the 227 retained-byte E2E cases.


All replayed provisions retain `not_analyzed` review status. The existing parser's explicit source repeal notes may produce deterministic `repealed` status; no temporal model, derive command, manual legal decision or review import ran. No prior review queue was copied into candidate roots; queue absence was inspected and recorded rather than described as an empty approved queue. Existing committed LRITF and IFPE validations remain clean. Cross-instrument admission/graph closure and comprehensive source inspection of the original 99 remain separate work.

Passing the original campaign signature gate has limited coverage. The 99-case review still finds some older signatures joined to body paragraphs in forms that gate never detected. No completeness/admission claim is made for those provisional outputs. This repair review checks the requested failure mechanisms, candidate deltas, source boundaries and retained evidence; it does not convert a structural pass into legal authority.

## Completion and next action

Changed implementation: `crates/lex-parse/src/diputados.rs`, `labels.rs`, `lib.rs` and `crates/lex-export/src/lib.rs`; supporting 15 fixtures/provenance, replay harness, report/audit, task plan and PLANS index. Canonical `corpus/`, schemas, accepted review decisions and evidence-worktree implementation are unchanged. No staging, commit or push occurred. The unrelated document deletions and Word lock remain preserved.

Next action: resolve the explicit LMIGRA, OGA and LRCDN source-identity questions and the 14 carried holds, then source-inspect repaired provisional candidates in bounded admission clusters. Continuity artifacts are maintained by the integrating session; their exact final paths are reported at completion.

A final note to consider not necessarily for this ingestion, but for later linking and Transitorio tracking. the diputados page has for each law the full list of ammending decress (each with their own transitorios which are then incorporated into the law compilation) Also I found that the full corpus can be downloaded in PDF here https://www.diputados.gob.mx/LeyesBiblio/legiscd.htm I don't see the immediate benefit of doings so but if it could help in any way (appears to have a min app or an executable we could consider it)
