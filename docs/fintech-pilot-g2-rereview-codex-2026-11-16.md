# Fintech structural pilot — Codex G2 re-review

Date: 2026-11-16. **Verdict: accept `991b5dc85` for pilot-v1 code freeze.**
P2 is closed and the requested implementation-choice pins are present. This
acceptance permits the independent evaluator's G3 step under the existing
private-gold versioning prerequisite; it is not a G3 result or production approval.

## Identity and bounded scope

Intake: clean `pilot/fintech-structural`, HEAD
`ac2a3b4f40684280f0d4378bbb50173540e1a0b5`. Candidate: `991b5dc85`.
The candidate implementation, integration tests and example are unchanged at
intake HEAD. The repair commit changes only `structure_pilot.rs` and
`structure_pilot_fixtures.rs`. Review covers the P2 repair, requested pins,
and the disclosed contract gap, against the prior G2 verdict. This is a bounded
follow-up, not a fresh independent audit of the whole implementation.

Sources read: the 2026-11-16 G2 re-review provider handoff; applicable repository
instructions and PLANS.md; prior G2 review; relevant checkpoint, G1.3 and
compatibility-record sections of the pilot plan/G1 draft-3 review; repair diff;
classifier, LookupContext and integration-test loading/filtering code. No
holdout answer or private gold was opened; no example or G3 run was performed.

## Findings and dispositions

- **P2 closed.** Unicode uppercase/lowercase predicates and an alphabetic
  multi-letter run recognise the contracted letter forms. Classification still
  gives supported fraction/inciso forms precedence. Public controls `AA)`,
  `Ab)`, `Á.`, `á.` and the em dash assert partial scans, `partial_scan` in all
  three modes, and conservative root-intro cuts. Plain words remain content.
  The explicit dash/bullet table is broader than before; it is a finite
  character repertoire, not a claim to cover every Unicode symbol.
- **Choice 1 pinned.** Interior-note tests assert m/root ownership, partial
  inciso scans, exact cuts and segments, with an unaffected sibling control.
  Post-child tests distinguish the inciso attachment segment from the
  fraction/root unsupported segment.
- **Choice 2 pinned.** The holder's intro remains resolved while the same-line
  inciso is L2 ambiguous and its scan partial.
- **Choice 5 pinned.** Unmatchable `IIII.` never enables `not_found` for IV, I
  or C; the test also checks the root cut and candidate segment.
- **Choice 6 pinned and documented.** `set_current_text` cannot insert an
  unloaded ID; its false return and the loaded-set `unknown_provision`
  meaning are explicit. No broader missing-projection API is approved.

The builder's mutation-check claims were inspected through the submitted tests,
not independently rerun. Passing probe counters establish unchanged aggregate
classification/structure on the eligible non-holdout corpus; the probe's
narrower regexes still do not independently establish semantics for new forms.

**Contract-gap ruling:** defer single-capital `A)` / accented-lowercase `á)`
recognition to pilot-v2. G1.3 explicitly enumerates the unsupported forms and
assigns everything else to content. These starts are outside that enumeration
(except independent Roman-start rules). Treating them as content follows the
frozen pilot contract, so this is not an implementation repair blocker. The
reported LIC 45-R apartado layout remains a known structural limitation;
root-child projections do not establish legal hierarchy. Do not amend fixtures,
contract, gold or code to accommodate it during G3. Any later expansion requires
an explicit versioned contract and new public controls. No corpus text or legal
judgment is changed by this engineering disposition.

## Freeze and evaluation boundaries

The frozen G1 section still hashes to
`ad607ca7d65983c05b5cf47e5350d9ed1aaab5591ed12745df8062e9a264ebf6`.
Freeze the reviewed code at `991b5dc85`; later documentation-only checkpoint
commits do not change the implementation identity. Original gold remains
immutable. The independent evaluator must version and document its private
compatibility disposition under `g1-draft3-compatibility-v1` before scoring.
Keep that work outside builder-readable artifacts until the evaluation rules
permit disclosure. The prior smoke-run deviation remains recorded in the first
G2 review; acceptance does not erase it or imply complete evaluator blindness.

## Validation actually run

A mechanical delegate ran at intake HEAD, with candidate code unchanged:

- `cargo fmt --check`: pass.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: pass;
  existing unused `walkdir` workspace-dependency warning remains.
- `cargo test --workspace --locked`: all suites pass, including all 15
  structure-pilot integration tests.
- `cargo run --locked -p lex-cli -- validate lritf`: valid, 0 issues.
- `cargo run --locked -p lex-cli -- validate ifpe-dcg-2021`: valid, 0 issues.
- `python3 fixtures/fintech-structural-g0-2026-11-14/verify.py` (no flags): pass,
  24 exact pinned records; holdout answers unopened.
- `python3 fixtures/fintech-structural-g1-2026-11-14/verify.py`: pass, 123 rows
  and integrity checks.
- `cargo run --offline --locked --manifest-path fixtures/fintech-structural-g1-2026-11-14/schema-check/Cargo.toml`:
  pass, 28 valid responses, 32 must-fail rejections, 346 valid row requests.

Reviewer independently checked the repair's exact changed paths, candidate-file
identity at HEAD and the frozen contract SHA-256. `git diff --check` passes.
No new all-instrument fidelity audit is claimed.

## Recorded outcome

This session changes this review and a checkpoint above the frozen contract
heading only. Instruction adoption was already correct, including the Codex
fallback setting; no instruction files needed editing. No source, canonical
data, schema, frozen fixture or private artifact was changed. No staging,
commit, push, merge or deployment is authorized or performed by this review.

**Next action:** independent evaluator versions the private compatibility
disposition and runs G3 against frozen `991b5dc85`, preserving the disclosed
pilot-v1 limitations and original gold.

Continuity closeout: appended the project-day receipt at
`AI/60_Evaluations/session-summaries/lex-mex/2026-11-16.md` in Agent Vault.
The required guarded rolling-context writer refused because writes are not
enabled for `lex-mex`; its pre-existing dirty note was preserved. Checkpointing
the existing capsule was required and attempted, but its validator rejected the
update because `source_refs` exceeds the 20-source bound. Capsule checkpoint
250 remains unchanged; no automatic checkpoint or external handoff resolution
is claimed. The repository review and plan are the current outcome. External
records remain uncommitted without current-session commit authorization.
