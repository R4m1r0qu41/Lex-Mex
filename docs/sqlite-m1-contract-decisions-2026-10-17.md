# SQLite M1 contract decisions — 2026-10-17

## Outcome

The three contract details the Codex review required before M2
([review](sqlite-plan-review-2026-10-17.md)) are settled below as contracts.
One input is still open: the structural-debt seed exists only as
**unreviewed machine-detected candidates**. The operator has to review the
candidate classes before M2 loads any seed row. No Rust code, dependency,
database file or canonical record changed.

- Corpus commit measured: `4582e47fb08027c2cfd2e97f34cbae24d902a163`, on
  branch `db/sqlite-projection` in the `../lex-mex-db` worktree.
- Plan: [`plans/sqlite-corpus-database.md`](plans/sqlite-corpus-database.md),
  patched in the same change as this record.
- Candidate seed:
  [`sqlite-structural-debt-candidates-2026-10-17.json`](sqlite-structural-debt-candidates-2026-10-17.json)
  (SHA-256 `0b2bccc0…c965f`), produced by
  `prompts/detect-embedded-article-headings.py` (SHA-256 `6ffe6309…22ec`).

## 1. Physical payload layout: extras-only

**Measurement.** A throwaway prototype (Python stdlib `sqlite3`, SQLite 3.51.0)
built the full corpus at the commit above twice. Both builds used the same
query columns, four indexes and an external-content FTS5 index over label and
text, then ran `VACUUM`. The only difference was the per-record JSON. Sizes
come from `dbstat`. This was a prototype for measurement, not the builder.

| Object | A: full payload | B: extras only |
|---|---:|---:|
| File total | 497.1 MB | 333.7 MB |
| `input_files` (exact raw JSON) | 167.5 MB | 167.5 MB |
| `provisions` | 213.5 MB | 93.0 MB |
| `term_usages` | 33.1 MB | 13.5 MB |
| `reference_edges` | 32.6 MB | 13.1 MB |
| FTS5 data | 32.9 MB | 32.9 MB |

**Decision: layout B.** The raw bytes are the byte-level reconstruction source.
The relational rows carry query columns plus an `extras` object. Exactly one
layout is implemented. No table carries a second full copy of a record. The
one exception is the 33 approval and operator-review objects, which are
small and audit-critical; they keep their complete object as `payload`. The
projected release size is about 335 MB plus the `heading_title` FTS column and
the extra indexes the query fixtures need. M4 records the actual size; the
plan's earlier 400–500 MB estimate is withdrawn.

**Contract.**

1. **Column mapping.** A column maps a canonical key only when the Rust
   contract always serializes that key, meaning a field without
   `skip_serializing_if`. A column `NULL` then means an explicit JSON `null`.
   If a mapped key is absent, the builder fails with the path and record ID.
2. **Keys that can be absent stay in `extras`.** This covers keys the Rust
   contract can omit, for example `numbering_basis`, `publication_period`,
   `amendment_marks`, `repeals`, `commencement_condition`, `resolution_rule`,
   `heading_context.libro/section/apartado` inside its object, `terms.fraction`
   and `section_heading`. It also covers every key the builder does not know,
   and those keys are kept verbatim.
3. **Queryable projections of those keys** are `VIRTUAL` generated columns
   over `extras`, for example `resolution_rule`. Presence is therefore recorded
   once, in `extras`.
4. **Projection-derived columns are excluded from reconstruction:** ordinal,
   `heading_title`, derived live target, source instrument and the
   projection-derived text hash.
5. **Child rows.** `qualifiers` become ordered `reference_qualifiers` rows,
   keyed `(edge_id, ordinal)`. The approval and review objects become their
   own one-to-one rows.
6. **Complete payload.** A record's *complete payload* is a logical record:
   mapped columns, plus `extras`, plus ordered children, plus attached review
   objects. It must equal the raw record as a JSON value under these rules:
   - object key order is not significant
   - array order is significant, and so are empty arrays
   - `null` is distinct from an absent key
   - integers are distinct from floats
   - strings and numeric values must match exactly
7. **Numbers.** Values are read from `serde_json::Value` of the raw bytes, never
   through `lex-core` structs. `Provision` and `ReferenceEdge` do not deny
   unknown fields, so a typed round trip would silently drop a future key.
   Both confidence fields are `f32`, and widening an `f32` 0.94 gives
   0.9399999976…. `REAL` stores the `f64` parsed from the JSON literal. The
   verifier also checks that each stored value re-serializes to the raw
   literal. The current literal set is 0.84–1.0.
8. **Parity is independent of insertion code.** The verifier reads the
   rows back through its own reconstruction code. It compares them with
   values parsed from the raw bytes, and it checks the raw bytes against the
   Git blobs at the corpus commit as a separate test.
9. **Evidence-only files** are stored only in `input_files`, with their
   handling policy, and query views use only fields those files actually
   contain. These are reform evidence, temporal requests and results, review
   queues, amendment legends, validation reports and the standards files.

**Prototype evidence.** Layout B was reconstructed for every record:

- 68,086 provisions
- 35,887 edges, with 12,302 qualifiers and 33 review objects
- 3,534 terms
- 62,785 usages

There were zero mismatches. Mutation checks were then run on copies:

- **Dropped optional key:** 2 files fail.
- **Null versus value on a mapped column, plus an injected key:** 1 file fails.
- **Integer `1` stored for a `1.0` confidence:** 1 file fails.
- **Swapped qualifier ordinals:** 1 file fails.

The M3 verifier must reproduce these checks in Rust; this prototype is not
acceptance evidence.

## 2. Structural-debt and reproducibility seeds

**Finding.** The repository has no record-level list of merged headings.
`docs/decisions.md` records only an aggregate: about 77 headings in 14
instruments. No seed row is derived from that aggregate. A deterministic,
read-only detector over committed `provisions.json` blobs proposes 117
candidates in 22 instruments, and none of them is reviewed. The detector
matches line-initial `Artículo N [letter] [qualifier]` headings, optionally
inside brackets. It skips four hits that follow a line ending in `:`: these
are CCF quotations of a former article 54, introduced by "a la letra
señalaba:". Prose mentions such as LFD "antes Artículo 172-A" and
line-wrapped citations such as LADUA "Artículo 144-A de esta Ley" do not
match.

| Instrument | decisions.md | Embedded heading | Bracketed placeholder | Qualifier outside label grammar |
|---|---:|---:|---:|---:|
| `lgtoc` | 22 | 23 | 0 | 0 |
| `lgipe` | 16 | 0 | 20 | 0 |
| `lac` | 10 | 10 | 0 | 10 |
| `laero` | — | 9 | 0 | 9 |
| `socap-sofipo-dcg-2006` | — | 8 | 0 | 0 |
| `lfped` | 5 | 7 | 0 | 7 |
| `lgcs` | 6 | 0 | 7 | 0 |
| `lgamvlv` | — | 6 | 0 | 6 |
| `cpf` | 3 | 3 | 0 | 2 |
| `lfd` | 2 | 3 | 0 | 3 |
| `lftr` | 3 | 3 | 0 | 3 |
| `lgs` | 3 | 3 | 0 | 2 |
| `lic` | 2 | 3 | 0 | 2 |
| `lfpc` | — | 2 | 0 | 0 |
| `lgmdfp` | 2 | 2 | 0 | 2 |
| `oaac-dcg-2009` | — | 2 | 0 | 0 |
| `lfdo`, `lgdnna`, `reg-lgcfd` | 1 each | 1 each | 0 | 0 / 1 / 1 |
| `lfpa`, `lgv`, `reg-lsem` | — | 1 each | 0 | 0 / 1 / 1 |
| **Total** | **77** | **90** | **27** | **50** |

"Outside label grammar" means the qualifier is not one of the ten in
`crates/lex-parse/src/labels.rs`. Examples are `Decies`, `Sextus`,
`Séptimus`, `Novenus` and `Quintes`. Even a corrected heading parser would
then fail `unparseable_article_number`, as `docs/decisions.md` anticipates.
`lgtoc` has 23 rows rather than 22 because `228 a bis` is printed in addition
to `228 a`–`228 v`.

**Operator decisions needed before M2 loads the seed.** These are not decided
here:

1. **Embedded headings (90).** Accept as `merged_heading` debt, per row or per
   class. In the 50 rows outside the grammar, the label grammar is also a gap.
2. **Bracketed placeholders (27).** These are LGIPE and LGCS lines such as
   `[Artículo 11 Bis. Suprimido]`. Decide whether they are missing structure
   or text that was legally removed and is printed as a placeholder.
3. **Bracketed bodies (3).** These are the LFTR 180 Quintes–Septimus
   headings, whose bodies are bracketed text. They are the invalidated
   articles behind the `lftr` accepted gap.
4. **Instruments the aggregate omitted.** `laero`, `lgamvlv`, `lfpc`, `lfpa`,
   `lgv`, `reg-lsem` and the two held DCGs. Either confirm them, or record why
   the aggregate excluded them.

**Seed contract.**

- **Location.** After review, the accepted rows become one committed builder
  input, `adapters/shared/_known-structural-debt.json`. It sits beside the
  other operator-curated tool inputs, `_closure-accepted-gaps.json` and
  `_named-offenses.json`. Its schema, Rust type, validator and fixture land in
  the same change as the `lex-db` code that consumes it. The candidate file
  stays an evidence artifact. It is not a builder input.
- **Row fields:**
  - `instrument_id` and `kind`
  - `printed_label` and the exact `heading_text`
  - `containing_provision_id`
  - `start_char` and `end_char`, as Unicode code points with an exclusive end
  - `label_grammar`
  - `source_path`, `source_blob_oid` and `source_sha256`
  - `detected_at_commit` and the detector version
  - `review_status`: `accepted`, `rejected` or `unreviewed`
  - `reviewer`, `reviewed_at` and the decision reference
- **Loading rule.** Only `accepted` rows load into `known_structural_debt`.
  The manifest counts `unreviewed` and `rejected` rows. The builder never
  infers debt.
- **Binding.** At the corpus commit, the containing provision must exist and
  its text at `[start_char, end_char)` must equal `heading_text` exactly.
  Otherwise the build fails. If the blob changed since detection but the
  locator still verifies, the verification receipt reports it.
- **Reproducibility.** No per-instrument measurement is recorded anywhere in
  the repository, so `instrument_reproducibility` holds zero rows at this
  baseline. A row requires the instrument, status, measuring commit, method or
  command, and an evidence path with its SHA-256.

## 3. Unknown versus known-missing

1. **A lookup has one of three outcomes:**
   - **Found.**
   - **Not found, with sourced qualifications.** There can be one or more:
     - `not_separately_parsed`, from an accepted debt row
     - `cited_not_in_corpus`, from an accepted-gap error record
     - `excluded`, from a coverage exclusion, such as standard bodies or
       `egdf`
   - **Not found in this release; coverage not established.**

   No outcome says that a provision does not exist. Qualifications can
   combine. LIC 112 Quintus already has a debt candidate (printed inside
   `lic` article 112 Quáter at 688–710) and is also cited by the `cpf`
   accepted gap.
2. **`cited_but_missing` is instrument-level.** Accepted-gap records give the
   source instrument, the target URN, the multiplicity and the reason. Held
   graphs do not store the rejected edge, so no citing provision or span is
   claimed.
3. **Reproducibility uses one contract.** The table holds only recorded
   measurements. A view over `instruments` supplies `not_assessed` wherever no
   measurement row exists. There are no stored `not_assessed` rows.
4. **An absent `resolution_rule` stays `NULL`.** This applies to the 4,058
   edges from held graphs and means "rule not recorded".

## Golden-query fixture pins

- **G1.** No stored edge comes from `reg-lan` article 122. The
  `parent_shorthand` edges to LAN article 122 come from article 153
  (`urn:lex-mx:federal:regulation:reg-lan:article:153:reference:557-560:122:direct`)
  and article 188 (`…:reg-lan:article:188:reference:53-56:122:direct`). The
  fixture is pinned to those two edge IDs. **The operator should confirm this
  is the case they reviewed.**
- **G2.** `urn:lex-mx:federal:statute:lic:article:194:reference:4292-4298:31-bis:direct`.
  It is `historical_superseded_reference_outdated` with rule `named_instrument`,
  and it carries both an approval and an operator note. It is never retargeted.
- **G3.** Four LRSIC Transitory Quinto edges, all `historical_target_unavailable`
  and pointing at `lraf-1990`. Their offsets are 25-27, 29-33, 36-40 and
  213-215. LRSIC article 2 fraction V targets the current LRAF and stays live.
- **G5.** Instrument-level: `cpf` → `urn:lex-mx:federal:statute:lic:article:112-quintus`,
  multiplicity 1.
- **G6.** Requires the accepted `lgtoc` row: `Artículo 228 b.-` inside
  `urn:lex-mx:federal:statute:lgtoc:article:228`, chars 2213–2229.

## Validation and preservation

Commands actually run:

- the census
- both prototype builds
- the reconstruction comparison and mutation checks
- the detector

All of them read Git blobs at the commit above. No Rust code changed, so the
native Rust gate was not rerun for this documentation and evidence change.
The operator's two untracked suggestion files in the main worktree were not
touched.

Next action: the operator reviews the four candidate classes. M1
implementation then continues: manifest and seed types and schemas, the DDL
under this contract, an adversarial fixture, and feature probes.
