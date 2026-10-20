# Codex review of Claude's parser-defect sweep — 2026-10-19

## Scope and disposition

Reviewed `b4109cc54..d8ecaeaa009380dd848d66e105d00b0079a1e2e2` on main: qualifier/accounting and glossary-only refresh, LIEPS incisos/usages/export, and the silent-loss/reform-evidence parser changes. Source: Claude's `2026-10-19-lex-mex-parser-defect-sweep.md` handoff. No code, canonical data, adapter, legal-review decision, commit or push changed in this review.

The two independent source replays support the committed LGTOC/CPF repairs. Native checks pass. This is not certification that every parser defect is fixed: the corpus debt and operator reviews remain, and two bounded extraction limitations are described below. Database contract work can start; pinning the first accepted database input should follow refresh of the debt seed and explicit recording of these limitations. Neither synthetic probe establishes an incorrect committed source-backed record that must be repaired before projection.

## Findings

### P2 — Mixed-delimiter extraction can include definition prose in a term

`crates/lex-parse/src/terms.rs`, established glossary extraction (around lines 350–365): the shared delimiter wins whenever found anywhere in the short entry prefix; `own_delimiter_term` is consulted only if it is absent or too long. Thus the newly added fallback does not solve mixed-delimiter entries whose definition later uses the shared delimiter.

Reproduction (synthetic, not a claim about an official instrument):

```text
Para efectos de las presentes disposiciones, se entenderá por:

I. Agencia: La agencia;

II. Auditor Interno, al responsable de las siguientes funciones: revisión y vigilancia;

III. Grupo: El grupo.
```

Actual terms: `Agencia`, `Auditor Interno, al responsable de las siguientes funciones`, `Grupo`. The second term boundary is wrong. This primary-delimiter precedence existed before the sweep; it is an uncovered limitation of the repair, not a demonstrated new regression. A narrow script checked the four affected DCG term sets (socap-sofipo, cub, cucb, oaac) for comma-definition openers inside their term names and found none; that is not a corpus-wide semantic audit.

Suggested repair: consider a recognized entry-specific delimiter before the shared one, with conservative ambiguity handling. Preserve commas inside legitimate names (the existing Programa Integral de Supervisión, Acompañamiento y Evaluación case). Add the reproduction as a regression fixture and inspect any resulting canonical term/usage diff before admission. Do not silently rename terms or infer operator approval.

### P2 compatibility limitation — Lower-case transitory headings now fail

`parse_transitory_start` rejects every lower-case ordinal, rather than only a carried-over ordinal in an existing transitory sentence. A synthetic original-transitory block `primero.- El decreto entrará en vigor mañana.` fails with `unnumbered transitory body requires an explicit source identity`; the same block with `Primero.-` parses. This fails closed rather than silently losing text. The intended CPF continuation case is correctly covered by the new fixture.

No retained official-source regression was established in this review; Claude's 309-source A/B census reports none. Treat capitalization as a supported-input restriction, not a legal syntax invariant. Prefer structural/contextual discrimination if a real lower-case heading arrives; keep both the continuation refusal and lower-case heading cases in its regression set. This does not require reverting the source-backed CPF repair.

## Boundary review

- Glossary-only refresh requires exact accepted-gap errors including multiplicity and refuses an instrument whose terms another adapter imports. Final coverage replay checks terms/usages against the final restored/refreshed sibling corpus before writes. This prevents the reported import-dependency path; no bypass was reproduced by inspection. Existing fixed-point term-target checks remain. A successful unchanged closure cannot alone prove all future changed-state cases.
- Inciso profiles bind literal markers and source hash, require complete marker accounting, and validate referenced definition incisos. Lower-case matching is opt-in. The three LIEPS machine boundary readings and sampled lower-case matches remain operator review items; this review supplies no legal ruling.
- Export anchor lookup uses the actual emitted-anchor registry and falls back to the containing paragraph for inline incisos. Native tests cover these paths; this review did not rebuild an external Obsidian vault.
- Preamble quotation handling is scoped to before the first provision. The suffix repair requires a closing `.-`. Unread numeric headings after structural headings now fail closed. The reform section/title heuristics are broader than one instrument, so the reported 309-source census is useful evidence but is not coverage of sources that were unavailable.
- CPF 209 Quintus exists in the hash-matching retained text and is independently recovered as a committed provision. The old accepted-gap premise was factually wrong. Operator confirmation of the decision correction remains separate.

## Validation actually run

Mechanical worker:

- `cargo fmt --check` — pass.
- `cargo clippy --workspace --all-targets -- -D warnings` — pass.
- `cargo test --workspace --quiet` — 365 passed, zero failed (42 + 3 + 32 + 15 + 269 + 4).
- `cargo run --locked -p lex-cli -- validate lritf` — 145 articles, 11 transitories, 126 references, zero issues.
- `cargo run --locked -p lex-cli -- validate ifpe-dcg-2021` — 59 articles, 4 transitories, 115 references, zero issues.

Independent source replay in `/private/tmp/lex-mex-sweep-review`:

- SHA-256 of both retained PDF and extracted text matches the committed manifest for LGTOC and CPF.
- LGTOC: 466 provisions; IDs/order, exact body text and heading contexts match; reform evidence matches exactly.
- CPF: 559 provisions; same parity checks pass; reform evidence matches exactly; 209 Quintus recovered.
- Synthetic probes reproduce the two limitations above. The final probe runner exits zero; an initial version used unwrap for the expected lower-case rejection and exited 101, then was corrected to report that expected rejection explicitly.

Private closure on a `git archive HEAD` snapshot: 468 instruments; zero relinked, 463 unchanged, five rejected exactly matching the accepted gaps; references 35,949 → 35,949; 9,844 resolved cross-instrument references. The JSON report retains `status: failed` because it records five rejections; the CLI separately recognizes the accepted-gap allowance. All 73,265 canonical files are byte-identical, with zero additions/removals. A worker shell wrapper then failed on zsh’s read-only `status` variable; its exit was 1 and the standalone closure exit was not captured. Normal closure completion and exact accepted-gap matching were observed, but an independently captured exit-zero result is not claimed. Details are recorded in the companion evidence JSON. Log: `/private/tmp/lex-mex-sweep-closure.log`. Canonical source text and human review state in the main checkout were not changed.

Companion evidence: `docs/parser-sweep-review-2026-10-19-evidence.json`.

## Database and four-stage sequencing

Start the existing SQLite workstream after refreshing its contract and debt inventory against the reviewed main commit, including inciso, usage form, unread fractions and glossary coverage. The DB is a rebuildable projection; it must preserve uncertainty, not repair it. Do not silently move an active database writer's worktree or overwrite its changes.

Before the first accepted artifact: pin complete inputs, ingest current accepted gaps and structural-debt evidence, expose independent coverage/unknown states, preserve legal review records, and pass independent row/payload/FTS parity and deterministic rebuild verification. Current census: 468 instruments, 68,328 provisions, 35,949 references, 5,734 terms and 102,176 usages (counted from committed federal JSON in this review). Five accepted gaps must not become a claim that the entire graph is complete. Known source-retention limitations must stay visible.

Do not wait for exhaustive grammar, temporal analysis, all heading repairs or unattended scheduling. Finish resumable jobs/retries, acquired-only access, exact-source retention/freshness and edge quarantine as subsequent pipeline work. Building those first could reduce future refresh churn, but delays a useful read-only index and introduces additional state boundaries before the projection is proven.

Model recommendation: GPT-6.1-Sol for bounded implementation and integration; use an independent higher-judgment review for schema/provenance boundaries and final acceptance, escalating to Astra for an unresolved architectural issue or repeated failures. Official OpenAI documentation describes Sol as near-Astra for complex coding at lower cost: https://developers.openai.com/api/docs/models/gpt-6.1-sol . This is task-based advice, not a measured Lex-Mex model benchmark.

Exactly one next action: update the database contract/debt seed against the reviewed input commit, carrying the two extraction limitations as follow-ups rather than silently claiming exhaustive parser correctness.

Unrelated state preserved: `docs/grok-db-suggestions.md` and `docs/grok_gemini_db_suggestions.md` remain untracked and untouched. This review does not authorize DB execution or a push.
