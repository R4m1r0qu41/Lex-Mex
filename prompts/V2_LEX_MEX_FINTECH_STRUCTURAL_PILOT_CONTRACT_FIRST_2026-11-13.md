# Lex-Mex — Fintech Corpus Structural Projection Pilot (v2)

**Status:** Revised agent-orchestration build plan — proposal, not implementation  
**Date:** 2026-11-13  
**Target:** Existing Lex-Mex repository, isolated worktree and a *verified* frozen sample from the Fintech corpus associated with https://www.leg4ll4bs.com/lex-mex  
**Primary deliverable:** Deterministic, source-span-backed **local** JSON subsection projection and local lookup harness over unchanged canonical `Provision.text`  
**Authority:** Existing canonical provisions remain authoritative. No corpus adoption, public deployment, database migration, or API change is authorized by this plan.

> **Coordinator instruction:** Execute the gates in order. Start at Gate 0 and pause for operator approval of the frozen sample and proposed structural contract before modifying production code. Read `AGENTS.md`, `CODEX.md`, active-run/capsule documentation and current git state; do not overwrite another agent's active ingestion task. Keep the project scoped to subsection recognition and retrieval, *not* normative ontology, temporal reasoning, blockchain, or a universal legal AST.

## 1. Basis, attribution and what is verified

This plan revises `LEX_MEX_FINTECH_STRUCTURAL_PILOT_AGENT_BUILD_PLAN_2026-11-13.md` in light of seven independent design comments supplied by the operator. The earlier technical report is `LEX_MEX_PARSER_SYSTEM_INDEPENDENT_AUDIT_2026-11-13_CODEX.md`; **its document states that its reviewer is Codex**. The separate comments were described by the operator as **Astra's review of the plan**. Do not collapse these attributions.

The Codex report established, for its inspected checkout, that:

- Canonical `lex-core::Provision` supplies machine-readable instrument/provision metadata and verbatim provision text; it does **not** expose a general paragraph/fracción/inciso tree.
- Defined terms have specialized locator and Unicode span records, but those spans are not necessarily identical in scope to a structural inciso.
- `lex-parse` recognizes forms such as `VI.-` that `lex-export::paragraph_anchors` can miss.
- SQLite and public HTTP/Model Context Protocol (MCP) read whole provisions, not arbitrary subsection paths.
- `verify-fidelity` aligns emitted provisions to extracted source; **it is not a complete source-to-canonical coverage proof**.
- Other defects (Unicode public routing, folded suppressed articles, orphan CNBV markers, held temporal evidence) exist but are not all prerequisites for the subsection pilot.

The earlier audit was based on checkout `c8495f7b864174eb2be760e6891ab9594840acf1` (as stated in that report). Reconfirm repository state and current behavior: neither this plan nor prior audits prove what has changed since.

**Public surface limitation:** Prior inspection identified a published partial collection and an `llms.txt` entrypoint. Actual current public-release identity and instrument membership must be checked afresh. If the live manifest cannot be verified, select a clearly identified local equivalent and describe it as **local**, not as confirmed deployed content.

## 2. Objective, success boundary, and non-goals

### Specific question the pilot must answer

Can Lex-Mex deterministically locate and return **precisely defined** subsections—such as a fracción or inciso—within its existing canonical text, preserving exact characters, source identity, nesting, and uncertainty, **without** reconstructing the provision's structure with a language model for each query?

### In scope

1. Freeze ~20–30 authentic provisions from a small, verified Fintech corpus inventory.
2. Specify subsection boundaries, parenthood, locator identity, and per-parent structural coverage **before** coding.
3. Implement a small local scanner/projection of **existing** canonical `Provision.text` into a versioned JSON artifact (or reuse a verified existing equivalent rather than duplicating it).
4. Implement a local command-line lookup harness using that projection, with typed results and failures.
5. Compare with existing glossary spans and export-anchor recognition, including `VI.-`.
6. Independently assess exactness, coverage, abstentions, and zero canonical modifications.

### Explicitly out of scope for this pilot

- Full-source completeness certification; extractor-to-canonical block accounting (separate bounded companion investigation only).
- Fixing general public Unicode routes, CNBV orphan markers, LGIPE folded suppressed articles, or held LIEPS reform evidence. Record and ticket separately; do not mask them.
- HTTP/MCP integration, production SQLite materialization, public website modifications, automatic deployment or release adoption.
- Rebuilding the canonical parser, changing `Provision` schema, full legal AST, legal obligation/exception interpretation, normative hierarchy/ontology, blockchain attestations.
- Treating an ambiguity as a resolved node to improve headline coverage.

**Non-negotiable:** `Provision.text`, canonical IDs, reviewer approvals, temporal decisions, source manifests, and deployed release artifacts must remain unchanged.

## 3. Selection and independent evaluation design

### Gate G0 — Freeze an evidence-backed sample

**Owner:** Baseline agent + coordinator; quality-assurance agent audits selection.

1. Inspect live `llms.txt`, linked release manifest/index, actual listed instruments and published versions. Pin URLs, retrieval date, release ID and available hashes. If inaccessible, state the limitation and use a pinned local equivalent. Never assume corpus membership from site branding.
2. Inspect local canonical files and relevant `Provision` types, `DefinedTerm` spans, `is_fraction_start`, `paragraph_anchors`, and current parser/reader contracts. Record git HEAD, branch, dirty files, and active worktree ownership.
3. Select approximately **20–30 provisions total** across the smallest set of *verified* instruments capable of testing the rules. Candidate instruments to confirm, not assume: `lritf`, `itf-dcg-2018`, `ifpe-dcg-2021`.
4. Divide the sample before implementation into **development/calibration cases (~12–18)** and **independent holdout cases (~8–12)**. The holdout agent selects/annotates its cases independently and does not give the implementation agent its expected answers before contract/code freeze.
5. Include: ordinary article paragraphs; `I.` / `VI.-`; nested `a)` / `b)`; repeated labels in distinct apartados; a potentially ambiguous closing paragraph; a cited Roman numeral that is **not** a fraction; a definition with an existing `DefinedTerm` span; at least one unsupported/opaque shape; a negative request for an absent label under a genuinely complete scan.
6. Freeze raw canonical JSON, sample selection, approved structural annotations, document hashes and source IDs. Record why every case was selected. Gold expectations must be independently annotated, **not generated by the candidate scanner**.

**G0 deliverables:** `00-baseline.md`, `sample-manifest.json`, `development-gold.json`, independently held `holdout-gold.json`, provenance/diff baseline. **No implementation.**

**G0 acceptance:** Sample membership is substantiated (deployed or explicitly local), test strata are sufficient, two sets are genuinely distinct, and original text/hashes are frozen.

## 4. Gate G1 — Freeze subsection semantics before implementation

**Owner:** Contract agent, independent reviewer and coordinator. The implementation agent may comment but must not silently determine the definitions while coding.

### 4.1 Data model (candidate, not preapproved)

A node describes **where text occurs and its structural role**, not an alternate authoritative copy of legal wording. A node may be an `article_root`, `apartado`, `fraction`, `inciso`, `paragraph`, or a strictly evidenced other kind. Unsupported layouts remain explicit opaque/unresolved regions.

```json
{
  "projection_version": "pilot-v1",
  "source_release_id": "<verified-or-local-baseline>",
  "instrument_id": "<canonical instrument ID>",
  "provision_id": "<canonical provision ID>",
  "canonical_text_sha256": "<hash of exact canonical text>",
  "node_id": "<version-bound derived identity>",
  "locator": {"article": "15", "apartado": null, "fraction": "II", "inciso": "b", "occurrence": 1},
  "kind": "inciso",
  "parent_node_id": "<derived parent ID>",
  "ordinal_in_parent": 2,
  "label_span": {"start_char": 0, "end_char": 2},
  "intro_span": {"start_char": 2, "end_char": 38},
  "complete_span": {"start_char": 0, "end_char": 97},
  "parenthood_status": "resolved",
  "scope": "statute_body"
}
```

**All positions above are deliberately schematic** and not measurements from a real provision. Fields can change after G1 review. All spans use half-open **Unicode scalar-value** (`Rust char`) indices into exactly the stored canonical `Provision.text`—not UTF-8 byte indices, grapheme indices, normalized text, or PDF page coordinates. JSON must have a defined ordering and deterministic serialization.

### 4.2 Required span distinctions

The contract must explicitly define and test at least:

- **`label_span`:** the literal structural marker (e.g., `II.` or `a)`), excluding text that is not part of that marker.
- **`intro_span`:** text directly introduced by the marker before the first confirmed child (may be empty). Specify treatment of whitespace and intervening newlines.
- **`complete_span`:** the contiguous textual extent of the subsection **including recognized descendants**, ending at a proven sibling/ancestor boundary. It may be unresolved if a trailing paragraph's attachment is uncertain.
- **`direct_content_spans` (if needed):** potentially discontiguous intervals of text directly attributable to the node, excluding child-owned spans. Do not promise a single contiguous `body_span` if it cannot represent the actual structure.
- **`unresolved_segments`:** exact ranges and candidate attachments when parenthood or boundaries cannot safely be determined. Never smuggle them into a successful `complete_span` as certain text.

Define retrieval modes separately: **`label`**, **`intro`**, and **`complete`** (including descendants). A query must specify its intended mode or receive a documented default (`complete` recommended); responses must identify which mode they returned. The existence of a reliable marker is **not** sufficient to assert a reliable full subsection.

### 4.3 Root, identity and locator contract

- Every projection has **one synthetic article root** for the canonical `Provision`; transitories, annexes and standards are out of this pilot unless explicitly selected and typed correctly. The root has the full text span and inherited canonical provision ID; it does not create an additional canonical provision.
- **Human locators** (Article 15 → Apartado A → Fracción II → Inciso b) are distinct from **derived `node_id` values**. Human labels may repeat. IDs must encode/disambiguate occurrence/path and bind to provision, exact text hash and projection version; no promise of stable IDs after amendments or reflow.
- Repeated labels inside separate apartados or nested scopes must resolve by full parent path. When a locator does not uniquely identify a node, return candidates or a typed ambiguity; **never select the first match**.
- `parent_node_id`, child/sibling order, scope and parenthood status are required on every non-root node. Orphans and uncertain parents must be explicit; do not attach them speculatively.
- Distinguish `statute_body`, original transitories, reform appendix, decree text, substantive annex and CNBV compilation. Do not assume they share one grammar.

### 4.4 Per-parent coverage and typed lookup outcomes

Track the scan **per parent and requested child kind**, not merely per article. Define e.g. `scan_status: complete | partial | ambiguous | unsupported` with exact scanned ranges, unresolved ranges and reason codes. A parent can have a complete fraction scan but an incomplete inciso scan.

Lookup outcomes:

| Outcome | Required meaning |
|---|---|
| `resolved` | Unique match with the requested text mode's boundaries verified and hash current. |
| `ambiguous` | More than one plausible matching node/path, unresolved parenthood, or an unsafe requested complete boundary. Return candidate IDs/spans when safe. |
| `not_structured` | Parent or child-kind was unsupported or insufficiently scanned; absence cannot be concluded. |
| `not_found` | Parent is uniquely identified, eligible child-kind scan is **complete**, and the requested label/path is absent. |
| `stale_projection` | Canonical text hash, source-release identity or projection version fails validation. |
| `invalid_locator` | Malformed query or impossible type/path under the declared contract. |

A missing label in a partially scanned article is **not** `not_found`. If `complete_span` is unresolved, a `label` lookup may succeed, but a `complete` lookup must not claim success. Never substitute a whole-article response as an exact subsection match.

### 4.5 Glossary-span comparison

Existing `DefinedTerm` spans are **comparison evidence**, not automatically the same concept as structural `inciso` spans. Compare canonical substring, labels, child scopes and offset conventions; record whether differences reflect a real error or legitimately different scope (e.g., continuation paragraphs inside a definition). Keep both representations intact until independently adjudicated. Do not overwrite existing glossary records.

**G1 deliverables:** `01-span-parenthood-contract.md`, `projection-schema.json` or Rust types with an equivalent machine-checkable specification, `lookup-contract.md`, and hand-reviewed expected behavior for each development/negative example.

**G1 acceptance:** Independent reviewer signs off on boundary semantics, per-parent completeness, ambiguous trailing paragraphs, repeated labels, locator distinction, and exact typed return modes. **Do not code until contract acceptance.**

## 5. Gate G2 — Implement the smallest local projection

**Owner:** Projection engineer, one writer per source module; QA produces independent tests.

1. Use existing `lex-grammar` helpers where appropriate; inspect dependency direction before choosing a crate. A few source-family configuration flags are acceptable if grounded in actual samples; no giant universal grammar.
2. Read **already canonical** `Provision.text` only. Do not re-fetch, re-extract, reparse, or rewrite the source to improve pilot results.
3. Emit article-root, recognized nodes and explicit unresolved/opaque intervals as an **optional derived artifact** in scratch/test paths only. Preserve source sequence and complete parent/child linkage.
4. Enforce span bounds; label/intro/complete semantics; ancestor containment; proven sibling order and non-overlap where appropriate; exact substring recovery; per-parent/kind scan coverage; identity uniqueness; scope boundaries.
5. Reject stale projection when exact input text hash or version changes. Ensure same frozen input + rules produces byte-identical output in two independent runs.
6. Do not turn recognized markers into successfully resolved complete subtrees when trailing text attachment is uncertain.
7. Include negative controls for inline citations, quotative headings, repeated Roman markers, ambiguous final paragraphs, and nested incisos.

**G2 deliverables:** minimal Rust implementation (if the repo architecture supports it), versioned pilot JSON artifacts, unit/property/golden tests and a reproducible invocation. **No canonical JSON or release modifications.**

**G2 acceptance:** The scanner correctly captures reviewed development examples; resolved spans exactly reconstruct original characters; ambiguous spans are flagged; canonical Git/hash diff is zero; deterministic output is verified. Reviewer can reproduce locally.

## 6. Gate G3 — Local lookup harness and independent holdout

**Owner:** Local-harness engineer + independent QA/reviewer.

1. Provide a **local command-line lookup** (or similarly isolated test harness) accepting release/local snapshot, provision ID, full typed structural locator, and requested mode (`label | intro | complete`). No new public API route or MCP tool.
2. Return canonical provision/source metadata, node/path identity, requested exact substring and spans, projection/hash/version, per-parent coverage status, and a typed outcome. For failure cases, no false exact text claim.
3. Run the frozen **independent holdout** only after G1/G2 interface and implementation are fixed. Report all failures; do not silently tune against holdout and then call it independent. If iterating, define and freeze a *new* holdout set.
4. Compare (without forcing equality) same-case glossary definition spans and Obsidian anchor behavior. Include `VI.-` and document parser/export divergence. A narrow parity patch may be proposed as a **separate follow-up change**, but it is not required for the projection's local acceptance.
5. Preserve current `get_provision` behavior and all HTTP/MCP interfaces. Produce a human-readable demonstration for one nested inciso, one `VI.-`, one unresolved trailing paragraph, one repeated label, and one verified `not_found`.

**G3 deliverables:** `local-lookup` harness, fixtures, `02-holdout-results.md`, and sample request/response outputs.

**G3 acceptance:** Exact-mode semantics work; `not_found` is coverage-justified; `ambiguous` and `not_structured` cannot be confused with missing; no first-match behavior; reproducible frozen holdout results.

## 7. Quantitative reporting: no misleading 100% claim

Report by **instrument, node kind, shape and requested retrieval mode**, always with numerator, denominator, and case IDs:

| Metric | Required report |
|---|---|
| Resolved exactness | Number of resolved cases with correct label, span, parent and exact substring / all attempted resolved cases; no wrong `resolved` allowed on acceptance cases. |
| Structural coverage | Number of eligible parent/kind combinations with `scan_status=complete` / total eligible combinations. |
| Abstention rate | Cases returned `ambiguous`, `not_structured`, or other non-success outcomes / total attempted, with reasons. |
| Unsupported shapes | Count, categories, affected source ranges; do not hide them in an accuracy percentage. |
| False absence | Count of `not_found` results where actual target exists or scan was incomplete: target **zero**. |
| Canonical change | Changed canonical files, IDs, text bytes, reviewer decisions, source manifests: **zero**. |
| Text-span integrity | All `resolved` outputs round-trip as exact substrings using Unicode scalar offsets; target **100%** on tested resolved nodes. |
| Determinism | Hash of repeated JSON projection and local responses identical for same inputs/config/version. |
| Regressions | Exact commands, exit codes, before/after relevant tests; list pre-existing failures separately. |

**Do not report “100% gold accuracy” as the project success measure if most structurally difficult cases abstain.** Present exactness *and* coverage/abstention together. Publish both development and untouched holdout results. A high abstention rate may yield **iterate**, not **adopt**, even with zero false positives.

## 8. Companion investigation — official-source completeness (separate track)

This is **not a gate** for G1–G3 and must not be represented as something a scanner over `Provision.text` can solve.

A scoped read-only companion can propose an **extracted-source-to-canonical disposition ledger** for one or two instruments. It must first demonstrate how source extraction offsets/blocks map to canonical text and intentional exclusions. Potential categories: provision content, structural heading, page furniture, signature, decree apparatus, reform appendix, opaque or unresolved. Record losses and limitations, including that normalized text offsets do not reconstruct original PDF visual positions without retained mapping.

Deliverable: `companion-source-coverage-feasibility.md` with specific required mappings, estimated engineering cost, and a **separate authorization recommendation**. No claim of full official-source completeness from the subsection projection; no changes to canonical ingestion in this pilot.

## 9. Separate engineering backlog (not pilot blockers)

- **`VI.-` export anchor parity:** Include discrepancy in regression evidence; propose narrow standalone fix using agreed recognition semantics after the local contract is reviewed.
- **Unicode public routing (`353-N` versus `353-Ñ`):** Separate public reader/routing compatibility task with complete route/URL tests.
- **CNBV orphan amendment markers:** Separate parser/provenance issue; never imply an already-discarded marker can be recovered by scanning committed provision text.
- **LGIPE suppressed article boundaries and held LIEPS decree evidence:** Separate source-backed reparsing/review tasks, not Fintech structural pilot scope.
- **Normative ontology, Tarquin/Orlog/Arrow-inspired typed legal rules and blockchain:** Architectural ideas for later; not part of this build.

## 10. Agents, isolation and decision rights

| Role | Assignment | Write authority |
|---|---|---|
| Coordinator | Active-run discovery, worktree boundaries, gates, sample freeze, owner handoffs | Plan, status, gate reports only |
| Baseline/sampling agent | Verify live/local corpus, select development set, pin hashes and evidence | Pilot inventory/fixtures only |
| Contract agent | Define spans, identity, parenthood, error and coverage rules | Contract/spec/test expectations only |
| Projection agent | Implement derived scanner after G1 | Assigned limited Rust modules, scratch outputs/tests |
| Independent QA/holdout agent | Independently select and seal holdout, challenge implementation, run adversarial tests | Holdout fixtures and audit outputs; no production rewrites |
| Independent reviewer | Verify evidence/claims/commands and recommend adopt/iterate/abandon | Review document only |

One writer per shared file; no concurrent edits to parser/export/reader modules without explicit file ownership. Preserve active agent worktrees and dirty-state snapshots. Legal status or substantive meaning requires operator review; obvious mechanical boundaries can be validated against source evidence by engineers. Any canonical adoption, consumer integration, DB migration or deployment requires a **new approval**.

### Gates and decisions

- **G0** — Sample and provenance frozen; approval to specify contract.
- **G1** — Span/identity/coverage contract accepted; approval to implement isolated prototype.
- **G2** — Projection passes development validation; approval to evaluate untouched holdout and local harness.
- **G3** — Independent holdout, exactness and coverage metrics reviewed; recommendation `adopt local`, `iterate`, or `abandon`.
- **G4 (future, not authorized here)** — Decide separately whether to integrate with SQLite, Obsidian, HTTP/MCP or public release after evidence supports doing so.

**Stop immediately** for unauthorized canonical/reviewer/source modification, evidence loss, any hidden ambiguity in a `resolved` result, non-unique locator silently resolved to first match, or verified false `not_found`.

## 11. Mandatory output files

Choose a repository-conventional pilot directory; avoid overwriting existing artifacts:

1. `00-baseline.md` + `sample-manifest.json` — published/local membership, git and source hashes, chosen development and independent holdout sample.
2. `01-span-parenthood-contract.md` + `projection-schema.json` + `lookup-contract.md` — exact span semantics, locators/identity, per-parent coverage, statuses, reviewer signoff.
3. `02-implementation.md` + projection JSON, minimal scanner and local harness — source paths, commits, reproducible invocation, zero canonical diff.
4. `03-development-and-holdout-results.md` — all sample case IDs and exactness/coverage/abstention metrics; differences from glossary/anchors.
5. `04-independent-review.md` — adverse cases, reviewer evidence, commands and observed exit codes, unresolved risks.
6. `05-decision-record.md` — adopt/iterate/abandon local projection; separate future integration and source-completeness decisions.

**Standard handoff from every agent:**

```text
Task / owner / worktree / base HEAD:
Files and exact hashes inspected:
Facts demonstrated (path + symbol + real example):
Assumptions / open questions:
Changes made (files/commit; or none):
Tests actually run (commands, exit codes):
Resolved exactness / coverage / abstention / false absence:
Canonical data + approval/source diffs:
Blocked cases / next owner:
Gate recommendation:
```

## 12. Ready-to-dispatch instruction to the coordinator

> **Start Lex-Mex Fintech Structural Projection Pilot v2 at G0 only.** Read project AGENTS/CODEX rules and active run state; preserve concurrent work. Verify the actual published Fintech inventory or label the baseline local-only. Freeze ~20–30 source-backed canonical provisions divided into a development set and an independently maintained holdout, including `VI.-`, nested incisos, repeated labels, glossary-span comparison, an ambiguous closing paragraph, unsupported shapes and a genuine negative locator. Return the sample manifest, provenance, current code/schema evidence and proposed exact span/parenthood/identity/coverage contract. **Do not implement before G1 acceptance.** After approval, build only a derived, hash-bound JSON subsection projection and local lookup harness. Preserve canonical text, reviewer decisions, existing endpoints and public release. Measure exactness, scan coverage and abstention separately; require complete parent/kind coverage before returning `not_found`. Flag independent Unicode routing, CNBV markers, LGIPE and source-completeness problems as separate work. Request operator approval before any public or database integration.

## Final boundary

**First prove what the existing canonical article text contains structurally, with exact spans, known parenthood and honest abstentions.** Only after that local proof should agents consider exposing the new representation to the rest of Lex-Mex. Normative ontology and executable legal rules remain future, independent layers.
