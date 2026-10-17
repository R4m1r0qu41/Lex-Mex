# Claude fix verification — 2026-10-17

Verdict: the original R1–R6 reproductions pass, with one remaining P2 boundary
case in dated parent-law recognition. Reviewed repair `feeabc23e` at clean
baseline `bcaaf24e823d7697b4031469d10215526f35f6f2`. Do not treat this as complete
parent-law acceptance or authorization for a bulk corpus rewrite.

## Remaining finding — punctuation bypasses the dated-law exclusion (P2)

`crates/lex-parse/src/lib.rs:817`–`:829`, `bare_word_follows`.
The function returns true for punctuation before checking the following words.
In private REG-LAN-based fixtures, each of these resolves to current LAN article 1:

- `Se aplicará el artículo 1 de la “Ley”, vigente hasta el 31 de diciembre de 1990.`
- `Se aplicará el artículo 1 de la “Ley” (vigente hasta el 31 de diciembre de 1990).`
- The unquoted `Ley, vigente hasta ...` equivalent.

LAN's recorded publication is 1992-12-01. These explicit historical readings
should remain unlinked rather than infer that current parent. The corresponding
unpunctuated forms now correctly produce no edge. This is a closely related
remaining boundary of R2/R4, demonstrated with synthetic input; no committed
bad edge or exhaustive corpus occurrence search is claimed.

Apply the dated/anaphoric exclusions to an immediately attached comma or
parenthesized modifier before accepting punctuation as a bare-parent boundary.
Preserve ordinary end punctuation, genuine parent citations followed by a
separate clause/law citation, and existing quote-style positives. Add fixtures
for quoted/unquoted comma and parenthesis cases; avoid a broad punctuation strip
that would make a later independent sentence govern the earlier citation.

## Fixes verified

| Finding | Independent result |
|---|---|
| R1 full-title precedence | Original exact LMTR title now resolves to LMTR; genuine parent followed by a separate law remains covered. |
| R2 quoted continuation guards | Original curly/straight quoted dated and anaphoric cases now remain unlinked. Punctuation boundary above remains. |
| R3 anaphora | Original Ley and Código antecedent cases remain unlinked. |
| R4 dated wording | Original `en vigor al` and `que estaba vigente` cases remain unlinked. |
| R5 single-link protection | Private historical retarget refuses with exit 1; all canonical file hashes remain identical. Explicit `--migrate-protected` succeeds and logs affected IDs. |
| R6 persisted review binding | Valid LGMIME review validates; changing only its source hash produces `legal_operator_review_invalid` and exit 1. |

The migration flag is explicit and conflicts with `--all`; it does not bypass
exact review reapplication. The operator migration rationale/receipt remains
part of the surrounding documented workflow; console output is not a durable
legal-review record.

## Validation and scope

The required native gate passes:

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`: 304 passed, zero failed/ignored.
- `cargo run --locked -p lex-cli -- validate lritf`: 145 articles,
  11 transitories, 126 references, zero issues.
- `cargo run --locked -p lex-cli -- validate ifpe-dcg-2021`: 59 articles,
  4 transitories, 115 references, zero issues.

The repair commit changes only parser/CLI code and LGMIME's validation summary
reference count (39 → 34). No source provisions, references, operator approvals
or legal notes changed in that commit. This verification changes only review
and plan documents. Eight accepted closure gaps and missing-parent-article
suppression remain accepted. No main corpus closure, reparse or relink ran.

A private REG-LAN link → validate → Markdown export replay also passes:
202 articles, 14 transitories, 198 references (126 parent edges), zero issues.
Every copied canonical JSON file, source manifest and Markdown file is
byte-identical after the replay; there is no REG-LAN review queue to preserve.
No parser stage or source download was needed for these linker/validator fixes.

Original reproduction inputs, private outputs and hashes are preserved in the
[companion evidence](claude-fix-verification-2026-10-17-evidence.json).

Next action: close the punctuation boundary with narrow regression fixtures,
then rerun the affected parent-law cases and required checks before broader
relinking. No code change, commit or push was made by this verification.
