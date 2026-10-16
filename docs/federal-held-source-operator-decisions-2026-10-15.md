# Original hold decisions — operator review, 2026-10-15

Authority: operator annotations preserved verbatim in
`docs/federal-held-source-review-2026-10-15.md`, SHA-256 `8f323c260a89118cab09e8d04dbe3a98c7fb93d35522a1937b8e9433fb6cabb3`,
and subsequent explicit EGDF answer in this session. This records reviewer
instructions, not machine findings of legal effect or commencement dates.

| Identities | Recorded operator decision | Execution disposition |
| --- | --- | --- |
| `egdf` | Exclude from corpus (explicit follow-up answer) | Excluded; not an ingestion failure |
| `lftr` | Abrogated; current instrument LMTR; keep historical LFTR with an abrogated line | Verify official source and admit historical text with notice; no inferred effect date |
| `lif-2026`, `pef-2026` | Retain both annual government income/expenditure instruments | Source acquisition and structural admission cleared |
| `lga`, `lgag` | Repair distinct Archivos/Aguas identities | Correct stale catalog mapping before ingestion |
| `lcnbv` | Exact stale LMV16Bis7 citation is `historical_superseded_reference_outdated` | Narrow reviewed exception; preserve citation and intended target without a live link |
| `lcmopfih`, `lisipl`, `lfiiedb` | Nested-law model approved, with source-specific ownership explanations | Explicit deterministic boundary profiles; preserve containing-act text separately; do not use “La presente Ley” alone as a delimiter |
| `consar-cuf-dcg-2018`, `cusf-dcg-2014`, `pld-lic115-dcg-2009` | Dedicated DCG parsing should be developed; hold DCG work | Remain deferred; preserve actual CONSAR/CNSF/CNBV publisher provenance |
| `reg-lgs-insumos` | Check official DOF source | Resolve health-regulation source and ingest if structurally valid |

The detailed nested-law notes remain authoritative at the source report. If
source-backed boundaries differ from earlier machine counts, report that
specific discrepancy rather than silently choosing a competing legal reading.
All other missing reference targets remain validation errors. This approval
never authorizes dropping quoted legal text, overwriting human decisions, or
assigning dates from retrieval metadata.
