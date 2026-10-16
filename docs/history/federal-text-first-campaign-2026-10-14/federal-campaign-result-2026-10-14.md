> **Historical record, 2026-10-16.** Produced 2026-10-14 against a 262-instrument corpus and an uncommitted `campaign` tool that was never landed. Superseded: the federal laws and regulations were admitted by the held-source route (467 instruments) and the project is in maintenance mode for them. Kept as provenance only; its counts and next steps are not current. Source patch: git-ignored `archive/campaign-2026-10-14-patch/`.

# Federal text-first campaign result — 2026-10-14

## Outcome

The first continuous prepared-inventory pass completed, without global abort.
It does **not** establish complete nationwide regulatory coverage, canonical
admission of new text, legal validity, or reader publication.

| Execution disposition | Instruments |
|---|---:|
| Existing canonical, structurally accounted `present` | 91 |
| New provisional Markdown, `awaiting_inspection` | 99 |
| Failed structural gate | 128 |
| Held on existing source/identity/legal decision | 14 |
| Unattempted | 0 |
| Total inventoried identities | 332 |

All 227 new unheld candidates were fetched and extracted. Of these, 99 passed
parse, validation and the additional signature inspection gate and exported
11,689 Markdown files: 10,806 articles, 784 transitories and 99 instrument
READMEs. Their 11,590 provisions remain `not_analyzed`; no legal review or
temporal model analysis occurred. The signature gate is narrow: passing it is
not proof of complete source inspection.

The canonical corpus and adapters are unchanged. The existing committed corpus
remains 262 instruments, not 361. A separate local portable Markdown bundle
contains 230 existing non-NOM instruments; it has not been deployed to the reader.
SQLite + `rusqlite` is recorded as the future derived serving base, not installed
as an unused dependency.

## Inputs and reproducibility

Producer baseline: `1a37ccde2e113e550d08a82bbead5620c9b6d868`, `main`, with
the uncommitted campaign implementation/docs described by the task plan.
Executed binary SHA-256:
`2e9fa5e9c0e6bfd3ce2c2f8b9f3d45f78082dbb90cfb038cfe46c5dc02139827`.
Campaign module SHA-256:
`80cd40174782c7455dd889378fd263057532c09fc261f28e45b419664557fe79`.

```bash
target/debug/lex-mex campaign run federal-text-2026-10-14 \
  .work/federal-campaign-inputs-2026-10-14-v3/manifest.json \
  batches/federal_catalog_discoveries_2026_10_14.json
```

The CLI exited 1 with `CAMPAIGN_COMPLETED_WITH_HOLDS_OR_FAILURES`, as expected;
`global_abort` is null. The terminal wrapper's final successful exit is not a
claim that every instrument ingested successfully.

Frozen prepared manifest SHA-256:
`8a89c2f5c1dd949fb21ac63688c806c2d895a90c99158f9676fb37dc15a1fc24`.
It contains 316 runnable prepared rows and 15 carried-forward holds.
The additional manifest inventories `reg-lic-calidad`, yielding 332 identities.
One original hold (`lfce`) is structurally present, but its hold rationale is
still recorded; no reviewer/source hold was automatically legally resolved.

Local evidence root: `.work/campaigns/federal-text-2026-10-14/`.
It contains frozen inventory, state, per-stage/per-attempt receipts, source
PDF/text, manifests, validation/inspection reports, generated Markdown and
generation-scoped retry manifests.

Initial full-pass `report.json` SHA-256:
`0dd686683011b43caef0d193e56a17ea28b004cbd56dc8c6e6d248995fcde43b`.
After the deliberate selective LFSA retry, final `report.json` SHA-256:
`ac0d7d4fd3969bd8ff08a88e2cba146bfe54ba9c99767cb05139214f04fda3b3`.
Both snapshots have the same final disposition counts.

These ignored `.work/` artifacts are local, not committed or replicated by Delta.
This tracked summary preserves all failed/held identities and remedy groups;
exact titles, source URLs, raw errors and hashes remain in the local frozen
inventory/receipts and the tracked prepared inputs. Do not delete the evidence
root before source inspection and admission. Rebuilding from later upstream
sources is a new acquisition, not reproduction of these retained bytes.

Official catalog reconciliation and historical-source scope:
`docs/federal-catalog-reconciliation-2026-10-14.md`.
Other federal regulation/statute catalogs and regulatory authorities are not
exhausted by this prepared campaign. NOM/NMX use a separate compiler boundary.

## Repair packets: all 128 failed identities

These are **observed symptom families**, not confirmed common root causes.
Do not resolve a legal question, relax a validator, freeze proposed counts, or
admit known malformed text merely to clear a group.

### 1. Signature/promulgation leakage — 87

Code: `CAMPAIGN_SOURCE_BOUNDARY_SIGNATURE`.
Reason: a legislative signature/promulgation span remains inside a parsed
transitory. The private `structure-inspection.json` records exact unchanged
provision text span, byte offsets and provision ID. Export was prevented.

Recommended first repair: inspect representative source boundaries and add a
narrow reusable parser rule with regression fixtures; use exact adapter stop
markers only when an instrument-specific boundary is genuinely required.
Then retry this group and re-inspect labels, counts, reform evidence and tails.

```text
cjm laam laem lagra lamn lamni lasoc lcacsam lcacsefam lcid lcjpjf lcndh
lcpro ldpam ldparm ldrs ldsca lebhn lec lei lemefam len less lfafe lfdeapd
lffaosc lffpmn lfgr lfll lfm lfmped lfmzaah lfpdppp lfped lfpsdmh lfsa
lfsp lfsv lfvv lgaas lgahotdu lgamvlv lgappcea lgb lgcdc lgcfd lgcs lgds
lge lges lgimh lgipd lgipe lgmde lgmhcti lgp lgpas lgpp lgpsacdii lgpsvd
lgsc lgscmm lih limj linpi lissfam lmin log loipn lopgjdf lota louam
lpeam lperc lpo lppddhp lram lrpv lsem lsnieg lsniimsp lsprem luach
luefa lupdecr lviv lvzmm
```
-- CJM:

- Make a note that in this one definitions are in art. 434 for latter linking.
- This is peculiar is old Abelardo L. Rodriguez was a substitute president and in this particular instance numerical ordinals were used 1o, 2o, 3o. also this seems to be some kind of left over. Al C.... should be an outlier. 

-- LAAM:
- Finishes as it should with En cumplimiento de lo dispuesto por la fracción I del Artículo 89 de la Constitución Política de los
Estados Unidos Mexicanos, y para su debida publicación y observancia, expido el presente Decreto en la
Residencia del Poder Ejecutivo Federal, en la Ciudad de México, Distrito Federal, a los veintitrés días del
mes de junio de dos mil cuatro.- Vicente Fox Quesada.- Rúbrica.- El Secretario de Gobernación,
Santiago Creel Miranda.- Rúbrica. **not sure what is the issue there**. 

-- lissfam
- The end of the doc is the ruling of the SCJN that declared the invalidity of certain articles. 

### 2. Recurring page furniture admitted — 21

Code: `CAMPAIGN_FURNITURE_CONTAMINATION`.
Reason: positional PDF audit found recurring header/footer material in at least
three parsed provisions. `furniture-audit.json` preserves each band's evidence.
Compare exact bands against the official title/source layout before adding
narrow header filters; do not strip arbitrary similar legal text.

```text
lansi lfcsq lfppcpcia lgmdfp lgpisdme lgpist lgpsdms lrart3-mmce
lrart5-prof lsint reg-larefagn reg-ldrs-moirsse reg-lemefagn reg-lftaipg
reg-lftr-mcsre reg-lgeepa-moe reg-lgpsacdii reg-lgs-mcsotcsh
reg-lgs-mcspiumc reg-lrart5c reg-lsm
```

-- lgpist
- looked at it it could be that because the name is too long is in two lines ![](image.png) that is the only different thing I could find.

-- reg-lgeepa-moe
- same thing two rows at the top because of the length of the name. 

### 3. Nested-law/decree structure — 1

Code: `CAMPAIGN_NESTED_LAW_STRUCTURE`; instrument `lmigra`.
The parser detected an unsupported nested/containing-act structural boundary.
Inspect its own provisions/transitories versus enclosing decree evidence before
choosing a model or parser repair. Do not force all quoted articles into the law.
Related pre-existing held cases are listed separately below.

-- This is not a law nested in a larger decree, it is however one whose in-force action of some parts depend on a future act (Transitorio Segundo, in force until de Reglamento is published, which it is now)

### 4. Other parse/evidence failures — 8

Code: `CAMPAIGN_PARSE_FAILED`. The raw diagnostics support these smaller
triage groups; root mechanisms remain to be verified.

| Instruments | Recorded reason | Required next investigation |
|---|---|---|
| `larefam`, `loefam` | Duplicate reform evidence ID ending `amendment:2022-09-09:transitory:segundo` | Same-day containing acts and evidence-ID disambiguation; preserve each source act |
| `lcaaa`, `lrmnyut`, `oga` | Reform transitory without its containing act heading | Source appendix/heading partitioning and containing-act attribution |
| `lgdnna` | Reform transitory `Primero` lacks Diario Oficial date after a decree heading beginning `DECRETO por el que se reforma el artículo 13, fracción XX, se adiciona el Capítulo` | Verify source date/heading layout; do not invent the date |
| `louaaan`, `reg-lfafe` | No configured publication date and no recognized DOF note | Acquire/verify official publication evidence and extraction layout before supplying metadata |

### 5. Invalid canonical validation — 11

Code: `CAMPAIGN_VALIDATION_INVALID`. Every validation report is retained; these
are the error-level reasons, not the non-blocking unfrozen-count warnings.

| Instrument | Error-level diagnostic |
|---|---|
| `lcnp` | Article 2 expected, article 3 found |
| `lfdefp` | Duplicate article 1 ID and order restart |
| `lgdlpi` | Duplicate article 1 ID and order restart |
| `linbal` | Article 1 expected; parsed sequence starts at 10 |
| `lounam` | Article 1 expected; parsed sequence starts at 10 |
| `lsm` | Article 1 expected; parsed sequence starts at 10 |
| `lsrlip` | Article 1 expected; parsed sequence starts at 10 |
| `reg-lagra-mcdets` | Article 1 expected; parsed sequence starts at 10 |
| `reg-lepecfe` | Duplicate transitory ID `transitory:68` |
| `reg-lfcine` | Article 40 expected, article 41 found |
| `reg-lrmineras` | Article 1 expected; parsed sequence starts at 10 |

Investigate ordinal heading recognition, source boundaries, genuine source gaps
and restarts as separate mechanisms. Apparent omission of 1–9 is not permission
to allow article gaps. Each material defect needs a reproducing fixture.

## All 14 held identities

These require explicit source/identity/operator/legal action, not blind retries.

| Instrument | Preserved reason |
|---|---|
| `consar-cuf-dcg-2018` | Official CONSAR compiled PDF URL not validated |
| `cusf-dcg-2014` | Official CNSF source resolution timed out/502; compiled PDF not validated |
| `egdf` | Transitional post-CDMX-Constitution status/scope requires confirmation |
| `lcmopfih` | Law nested in a containing 1990 act; own versus outer transitories not safely partitioned |
| `lcnbv` | Article 15 cites unavailable current LMV article 16 Bis 7; do not silently drop/resolve it |
| `lfiiedb` | Nested law, quoted LFPRH reform and separate decree transitories require a modeling decision |
| `lftr` | Post-2025 regulator/current official-title verification remains held |
| `lga` | Archivos/Aguas slug-title collision requires identity reconciliation |
| `lgag` | Aguas hold is recorded under conflicting historical `lga`; catalog presence cannot clear it |
| `lif-2026` | Annual-instrument treatment requires explicit confirmation |
| `lisipl` | Nested tax inside Article NOVENO of a 1968 act; outer transitories cannot be attributed blindly |
| `pef-2026` | Annual-budget-decree treatment requires explicit confirmation |
| `pld-lic115-dcg-2009` | Official base compiled DCG source remains unresolved |
| `reg-lgs-insumos` | Official source resolution remains held |

## All 99 provisional successes awaiting source inspection

Source and extracted-text hashes, passing reports and exported README existence
were independently checked for all 99 after the run. That verifies artifact
integrity, not legal/source completeness or admission approval.

```text
cmpp larcp lbio lcne ldefam ldsc lepecfe lepepm lexp lfca lfiv lfpccs
lfppipp lgct lgdocia lgec lgeo lgmasc lgmime lgpdppso lgsmime lgt lgtaip
lgv lmtr lnac loam loinah lopjf loscm lpf lpte lrart27-mn lrcdn lrpcap
lrsf lse lsh lsss reg-laam reg-lagra-fodmc reg-lagra-mopr reg-larcp
reg-lbio reg-lebhn reg-lecampo reg-len reg-lepepm reg-lfa reg-lffarosc
reg-lfll reg-lfmzaah reg-lfpccs reg-lfpdppp reg-lfrt-mcpctrt reg-lfsa
reg-lfsp reg-lfsv reg-lfvv reg-lgamvlv reg-lgappcea reg-lgcdc reg-lgcfd
reg-lgct reg-lgdnna reg-lgds reg-lgeo reg-lgife reg-lgipd reg-lgp
reg-lgpsvd reg-lgs-mpsam reg-lgt reg-lgv reg-lgvs reg-lic-calidad
reg-lieg reg-lih reg-lissfam reg-lmigra reg-lmin reg-lnac reg-logan
reg-loinah reg-lopgjdf reg-lopgr reg-lpeam reg-lpesca reg-lpf reg-lpo
reg-lppddhp reg-lpte reg-lrpc reg-lrpv reg-lse reg-lsem reg-lsh
reg-lspee-ma reg-lupdecr
```

Inspect official-source identity, provision counts/labels/order, source tails,
reform evidence, validation, references, review state and Markdown before
admission. Keep source review separate from JRH's legal-review authority.

## Retry and resume evidence

`retry-index.json` points to the current generation:
`a40ac89a13030efe2de14be46972b036d97953a1a4b0bd488b1b02f57de6c084`.
Each failure code above has a separate private retry manifest. Do not overwrite
the original frozen inventory using a repair subset.

Preferred same-campaign selective retry after a verified repair:

```bash
target/debug/lex-mex campaign run federal-text-2026-10-14 \
  .work/federal-campaign-inputs-2026-10-14-v3/manifest.json \
  batches/federal_catalog_discoveries_2026_10_14.json \
  --only lfsa
```

The actual selected-success resume for `reg-lfsa,reg-lic-calidad,lfzee` appended
no attempts and repeated no source stages. A deliberate unchanged-source LFSA
retry appended only LFSA attempt 2 and failed the same signature gate; the other
331 attempt counts were unchanged. This tests retry isolation, not a successful
repair. Both invocations returned the expected completion-with-holds/failures
diagnostic. Unselected results remain historical accounting.

Successful reuse requires unchanged input/implementation fingerprints and intact
own artifacts. A new executable/parser/adapter/dependency snapshot can invalidate
reuse. Run only the repaired group when changes are bounded; do not blindly
rerun all 128 deterministic failures.

## Verification and remaining work

Passed the final repository gate:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run --locked -p lex-cli -- validate lritf
cargo run --locked -p lex-cli -- validate ifpe-dcg-2021
```

243 workspace tests passed, including 20 campaign tests; five Python inventory
tests passed. All 230 existing non-NOM instruments were separately validated.
Independent scratch review and regression tests cover hardlink/symlink output
redirection, concurrent ownership and stale pending snapshot pollution.
The parent regression binds staged adapter options to the captured fingerprint.
The process lock coordinates cooperating processes, not hostile filesystem
renames. Non-Unix directory flushing/hardlink checks are qualified.

No new canonical admission, full graph closure, database implementation, temporal
model call, legal-review resolution, reader build/deployment, commit or push.
Reader integration needs its separate repository attached; the producer bundle
is not an implemented release-builder contract.

**Next action:** repair the 87-member signature-boundary family using source-backed
fixtures and a narrow coherent rule, then selectively rerun it. Source-inspect
and admit the provisional-success queue in bounded clusters independently.
