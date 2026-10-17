# Claude code review — 2026-10-17

Verdict: changes requested. Reviewed the pending Claude parent-law handoff at
`69af0e75ec545af24cf103347860c7d20b2936ea`, including the advisor narrowing and
relevant earlier protection/export fixes. Six P2 findings remain. Reproductions
use synthetic text or deliberately modified private corpus copies; this review
has not demonstrated that the new parent-rule cases corrupt stored edges.
No code, adapter, corpus, operator note or legal decision was changed.

## Findings

### R1 — Explicit configured law title loses to the default parent (P2)

`crates/lex-parse/src/lib.rs:854`, `:892`, `:923`, `:946`.
With REG-LAN's mapped LAN parent and the configured exact LMTR title,
`Se aplicará el artículo 1 de la Ley en Materia de Telecomunicaciones y Radiodifusión.`
resolves to LAN article 1 instead of LMTR article 1. Both targets exist.
The `en` continuation is accepted as bare shorthand; the parent marker starts
before the full title and wins earliest-marker selection.

Make a full explicit title overlapping the same governing phrase outrank its
bare-parent prefix. Preserve genuine parent references followed by a separate
later law citation; global full-title precedence would introduce another error.
Add both regressions and retain REG-LAN's accepted quoted-parent example.

### R2 — Quoting Ley bypasses the dated/anaphoric exclusions (P2)

`crates/lex-parse/src/lib.rs:786`–`:814` (`bare_parent_marker`).
`quoted || bare_word_follows(...)` skips every continuation check when the
marker ends in a quote. Bare `Ley vigente hasta el 31 de diciembre de 2001`
and `Ley en cita` correctly produce no parent edge, while the equivalent
`“Ley” vigente hasta ...` and `“Ley” en cita` incorrectly produce LAN article 10.
Straight quotes reproduce the dated case too.

Apply continuation/context exclusions after closing quotes as well. Cover both
quote styles, dated/anaphoric negatives, and genuine quoted-parent positives.

### R3 — Unsupported anaphoric Ley/Código becomes the parent (P2)

`crates/lex-parse/src/lib.rs:837`–`:854` (`bare_word_follows`).
After a sentence explicitly identifies LFT, `artículo 1 de la Ley en comento`
and `de la Ley que se cita` resolve to LAN. The equivalent Código example,
with CCF as the stated antecedent and CFF as parent, resolves to CFF.
The `en` and `que` allowlists admit these unsupported antecedent references.

Leave unsupported anaphora unlinked for review rather than infer the parent.
Add Ley/Código regressions and preserve ordinary bare-parent controls.
This does not require implementing cross-sentence antecedent resolution.

### R4 — Equivalent historical wording bypasses the dated-law guard (P2)

`crates/lex-parse/src/lib.rs:837`–`:855`.
`Ley en vigor al 31 de diciembre de 1990` and
`Ley que estaba vigente al 31 de diciembre de 1990` resolve to current LAN,
whose recorded publication is 1992-12-01. The existing `Ley vigente hasta ...`
control correctly emits no edge. These are explicit dated readings, not bare
parent shorthand.

Reject these equivalent temporal continuations for automatic parent selection;
do not invent a historical target. Add positive/negative controls independently
of the quotation fix.

### R5 — Single-instrument linking bypasses the bulk historical-edge guard (P2)

`crates/lex-cli/src/main.rs:2663` (`run_link`), `:2929`, `:3004`.
The single-instrument path writes its new graph without `protected_edge_drift`,
which bulk closure uses to refuse changes to protected historical edges.
This confirms the open question already raised by Claude, rather than a newly
discovered production corruption. A private closure run refused the change without mutating LRSIC; subsequent
private single-link succeeded and retargeted the historical edge. Exact
reproduction is in the evidence.

Use one pre-write guard for single and bulk operations. Recorded operator
migrations such as LRSIC/LGMIME need an explicit audited migration path; preserve
their accepted decisions. Require refusal before any canonical file changes in
an unapproved migration. Do not conflate this with silent removal of attached
`legal_operator_review`: relinking separately checks that payload and binding.

### R6 — Persisted legal-review binding is not checked by validate (P2)

`crates/lex-core/src/lib.rs:1011` (`LegalOperatorReview::matches`),
`crates/lex-parse/src/lib.rs:2313` (`validate_reference`) and the
reference validation path used by `crates/lex-cli/src/main.rs:3047`.
In a private LGMIME copy, changing an attached review's `source_text_sha256` to
64 zeros leaves `validate lgmime` successful. The source text and reference edge
are unchanged. A structurally valid hash therefore appears to attest an exact
review even though its source binding is false.

Validate every persisted annotation against its current edge and unchanged
source text, including schema/metadata checks. Report a validation error on
mismatch; retain the annotation for diagnosis. Keep the existing relink-time
adapter/payload/exact-match protections. Add a valid control and tampered hash,
span, target, and metadata cases where they exercise distinct checks.

## Verification and boundaries

The mechanical native gate passed on this HEAD:

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`: 301 passed, zero failed/ignored.
- `cargo run --locked -p lex-cli -- validate lritf`: 145 articles,
  11 transitories, 126 references, zero issues.
- `cargo run --locked -p lex-cli -- validate ifpe-dcg-2021`: 59 articles,
  4 transitories, 115 references, zero issues.

The successful gate does not cover the new negative cases above. The companion
[evidence JSON](claude-code-review-2026-10-17-evidence.json) preserves inputs,
actual/expected results, source hashes, private commands and review receipts.
The independent parent review checked the 123-entry mapping baseline but did not
legally approve every mapping; no concrete mapping identity defect was established.
Prior L1–L7 fixes were inspected within this scope, not reopened as outstanding.
Fresh three-instrument export acceptance was not completed to a reviewed
acceptance result; no passing export remeasurement is claimed. The private
closure contrast supports R5, not general corpus closure acceptance.

The eight accepted closure gaps (CPF, LGS, LMV, LFTR and four DCGs) remain accepted
and unchanged. Suppressing a shorthand edge whose parent article is absent is
also accepted policy. Neither is a finding. No full corpus closure or bulk reparse
ran on the main worktree. Advisor parser-reproducibility observations remain a
separate bounded source/fixture investigation; do not bulk-replace canonical text.

## Repair order and acceptance

1. Fix R1–R4 together in parent-marker recognition, with separate regression
   cases and the existing mixed-citation/quoted-parent controls.
2. Close R5 through a common pre-write guard and an explicit audited override
   for operator-approved migrations; demonstrate rejection leaves all files intact.
3. Close R6 in persisted reference validation without altering review decisions.
4. Run the required native gate and private affected-stage replays; compare
   graph changes, review state and exported Markdown. Then inspect only affected
   committed edges before any authorized relink. Keep structural parser debt separate.

A written response is queued for Claude; no Claude CLI/model was invoked.
Review documents are uncommitted; no code/data commit or push was made.
