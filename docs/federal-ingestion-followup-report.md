# Federal ingestion follow-up

All 227 frozen cases now pass extraction, parsing, validation and export: 128 original failures and 99 original provisional cases. The prior three structural blockers are repaired. No source bytes changed, and the 14 separate holds remain held.

Implementation follows the operator annotations in the preserved [original report](federal-ingestion-failure-repair-report.md) and the [follow-up contract](plans/federal-ingestion-failure-repair.md#follow-up-operator-annotations-and-implementation-contract). This report records structural ingestion, source ownership and serialization. It does not resolve legal status, commencement conditions, surviving effects or admission to the canonical corpus. The 14 original holds remain held.

The [final audit](federal-ingestion-followup-evidence.json) records every case, source hashes, stages, prior-candidate differences, the 14 unchanged hold dispositions and exact identity migrations. The frozen executable SHA-256 is `da1a7aab3e18113caacec0155a4aaba4ea71f8d046cda28110873a099707185b`.

The reviewed adapter options separate the LMIGRA law's ten original clauses from four enclosing-decree clauses and preserve the elided sibling article. REG-LMIGRA has 262 article records, no own transitory layer, thirteen enclosing-decree clauses and two elided sibling articles. Decree evidence is typed separately, carries decree-qualified identity, printed heading, owner, containing-act title, publication metadata and source/text hashes, and is persisted in `enactment-evidence.json`. Loading and bundling validate the sidecar; exports identify its owner in a separate Spanish section. These records are excluded from own-law counts, the ordinary provision-reference graph and automatic temporal-request inputs. Preserved decree bodies do not imply graph closure or commencement resolution.

LRCDN and OGA retain their exact singular printed headings. Their sole unnumbered clauses have stable internal `transitory:unico` identities with `numbering_basis: "implicit_singleton"`; the parser never inserts ÚNICO into the source label or text. Persisted validation rejects coexistence with any other own transitory.

OGA records `publication_date: null` and explicit `publication_period` metadata: version `1.0.0`, kind `serialized_range`, inclusive dates `1912-01-01` and `1912-01-08`, exact source note and source SHA-256. The reviewed exception requires the retained OGA source identity and note. Neither endpoint nor the 1985 reform date becomes an exact publication day. Every OGA provision carries the same period and no exact day. Date-relative derivation skips the case, date repair makes no change, and temporal-request generation fails before creating a request.

Existing exact-day JSON retains its date strings and omits the new optional metadata. The global `0.1.0` schema version and unrelated standard/model temporal schemas remain unchanged. The new publication-period schema is explicitly discriminated and versioned; exact-day and period records are mutually exclusive. Legacy readers requiring a non-null date reject new period records rather than accepting a fabricated day. CLI indexes, canonical bundles, Markdown and Obsidian carry the period. Workspace Rust callers now handle optional exact days; external Rust or JSON consumers outside this repository were not verified and must add period support or fail closed.

A separate pre-existing LMIGRA Article 3 glossary defect surfaced after structural parsing succeeded. The generic comma style truncated four distinct colon-defined labels to “Niña”. The reviewed adapter uses `mixed_fractions`, preserving comma-containing labels and the source's first comma definition. The page-joined XIX is recognized only after its explicit `(antes fracción XXI)` editorial note. Generic inference remains unchanged, duplicate-identity validation remains active, and a source fixture plus negative cases cover the opt-in behavior.

Remaining future work includes act-level amendment inventory, decree-owned temporal review, cross-instrument repeal evidence, verified commencement conditions, surviving effects for older permits/proceedings and executive-deadline compliance. Passage of time alone does not resolve these questions.

| Source | Own articles | Own original transitories | Separate decree transitories / elided siblings | Publication |
|---|---:|---:|---:|---|
| LMIGRA | 162 | 10 | 4 / 1 | `2011-05-25` |
| REG-LMIGRA | 262 | 0 | 13 / 2 | `2012-09-28` |
| LRCDN | 31 | 1 implicit singleton | 0 / 0 | `1974-12-31` |
| OGA | 1868 | 1 implicit singleton | 0 / 0 | Serialized 1–8 January 1912; no exact day |

Independent anchored source-heading extraction matched every candidate article number: 162/162, 262/262, 31/31 and 1868/1868, respectively; no missing or extra source identities. REG-LMIGRA’s maximum article base is 250; its suffix articles account for the 262 records. [Fixture provenance](../fixtures/diputados/followup-provenance.md) records each retained PDF/text hash and fixture hash. The source boundaries are explicit:

- LMIGRA prints `ARTÍCULO PRIMERO. Se expide la Ley de Migración.`, the law's `TRANSITORIOS` from PRIMERO to DÉCIMO, then `ARTÍCULOS SEGUNDO A NOVENO. ……….` and `TRANSITORIOS DEL DECRETO POR EL QUE EXPIDE LA LEY DE MIGRACIÓN Y SE` followed by the containing act's remaining title. Its first decree clause begins `PRIMERO. El presente Decreto entrará en vigor al día siguiente de su publicación en el Diario Oficial`; this is distinct from the law's `PRIMERO. La Ley de Migración entrará en vigor al día siguiente de su publicación en el Diario Oficial`.
- REG-LMIGRA prints `ARTÍCULO PRIMERO. Se expide el Reglamento de la Ley de Migración, en los siguientes términos:`, then after article 250 `ARTÍCULO SEGUNDO: ……….` and `ARTÍCULO TERCERO. ……….`. Only then does `TRANSITORIOS` introduce PRIMERO through DÉCIMO TERCERO. The new attribution follows this enclosing-decree sequence.
- LRCDN prints `ARTICULO TRANSITORIO:` with the unnumbered body `La presente ley entrará en vigor al día siguiente de su publicación en el "Diario Oficial" de la Federación.` The closing congressional and promulgation material follows separately.
- OGA prints `TRANSITORIO` and `La presente Ordenanza comenzará a surtir sus efectos desde el primero de febrero de mil novecientos doce, quedando derogadas las Leyes y disposiciones que en todo o en parte se opongan a ella.` The publication note is `Nueva Ordenanza publicada en el Diario Oficial de la Federación en forma seriada los días 1º al 8 de enero de 1912`. The printed absolute commencement text is preserved without a new temporal determination.


Compared with the prior v5 outputs, 223 previously passing cases retain identical provision records and legislative reform evidence. REG-LMIGRA is the only changed prior pass: all 262 article IDs survive; article 250 loses exactly `\n\nARTÍCULO SEGUNDO: ……….` because that elided sibling is now separate evidence. Its substantive body is unchanged. The thirteen former own transitories move from `urn:lex-mx:federal:regulation:reg-lmigra:transitory:<suffix>` to `urn:lex-mx:federal:regulation:reg-lmigra:enactment:2012-09-28:decree:transitory:<suffix>`. The suffixes are `primero`, `segundo`, `tercero`, `cuarto`, `quinto`, `sexto`, `septimo`, `octavo`, `noveno`, `decimo`, `decimo-primero`, `decimo-segundo` and `decimo-tercero`. Every moved body is identical; the audit contains all full old/new IDs and body hashes. The second elided sibling is now retained as well. There are no unexplained identity removals or other changed existing provision records.

Later reform evidence is unchanged for all 224 prior passes. Newly supported complete candidates retain 46 LMIGRA, 4 LRCDN and 6 OGA reform-evidence records; REG-LMIGRA retains its 9. Judicial appendix retention and the original report’s signature-gate limitations remain applicable. All 227 PDF/text pairs matched their frozen manifests, acquisition identity stayed unchanged, and all candidates have `not_analyzed` review status. No prior review queue was copied, no temporal model or derive command ran, and no human decision was imported. Existing explicit source repeal-note behavior remains unchanged.

The four affected candidates’ manifests, validation reports, candidate differences, review-queue absence and exported Markdown were inspected. Every decree evidence body appears in its separate Spanish Markdown export. LRCDN exports its printed heading with explicit singleton metadata. OGA exports the period in structured frontmatter and prose, including both inclusive endpoints without an exact-day claim. Tests verify that clearing evidence removes stale decree Markdown/JSON from Markdown and Obsidian output and writes an empty canonical sidecar. Stored-record validation checks owner, ID, matching source identities, body hash and publication metadata; bounded source fixtures and retained-byte replay separately corroborate attribution. Loading a sidecar does not re-extract or re-prove its raw PDF spans.

The final native gate was delegated to `mechanical` and passed in `.work/federal-followup-checks-v7/`:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run --locked -p lex-cli -- validate lritf
cargo run --locked -p lex-cli -- validate ifpe-dcg-2021
```

All 250 workspace tests passed. LRITF: 145 articles, 11 transitories, 126 references, zero issues. IFPE: 59 articles, 4 transitories, 113 references, zero issues. Formatting and `cargo build --locked -p lex-cli` also passed. The final eight-shard replay is `.work/federal-followup-all-v1/`; all eight commands exited zero. A bounded metadata audit checked all 230 committed ordinary instruments: 230 non-null exact days, no publication periods and no provision/instrument publication mismatch. The LRITF serializer fixture also compares every original provision field and the full instrument JSON through the actual on-disk serializer.

Changed paths span the existing five Rust crates, two additive schemas, five exact-source follow-up fixtures and their provenance, the opt-in adapter overrides and frozen-source harness under `prompts/`, this report/audit and the task plan/index. `corpus/` and existing legal reviews are unchanged. The operator-annotated original report SHA-256 remains `dfba75d98b84b47205225538261c2da2292f67f640bac6fd36354f4610a32976`; the original audit remains `4b7f3f7977b34c0a56922a659dc3344a335c78d1f736c7458663a885b26a52e8`. The hidden evidence worktree, unrelated document deletions and Word lock were preserved. No staging, commit, push, deployment or canonical admission occurred.

Next action: integrating review of these artifacts, followed by separately authorized source-inspection/admission or temporal-review work. All 14 original holds and the CJM article 434 future-definition-linking note remain preserved. The integrating parent owns the final continuity receipt/checkpoint.
