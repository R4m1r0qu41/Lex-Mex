# Codex bounded re-review — items 3 and 5

Handoff target: `b640b5441aee2da972d2ecccb3fa21f6ce9d8ff4`.
Reviewed working HEAD: `5a2dc36733ecf10d1779dffb4f65f5c87221360a` on
`upgrade/four-goal-2026-11-12`. The difference is execution-plan documentation only.
Initial working tree clean. No legal decision is made by this review.

## Item 3 — accept the bounded repair

`crates/lex-link/src/references.rs:118` and `:1480` restore self-designations,
exception connectors and inner-part chaining while retaining later-citation checks.
The expanded regression cases in `crates/lex-parse/src/lib.rs:2435` address the
three shapes in the original finding, plus leakage and back-reference cases.
Independent JSON comparison against `ee4f730c3` confirms 13 qualifiers added in
11 instruments, none removed, and no other reference fields changed.
This closes item 3's named repair, not the separate suffixed-article target defects
or a semantic audit of all historical removals. Exception qualifiers still have
no exclusion flag, as disclosed in the handoff.

## Item 5 — change requested (P2)

The explicit evidence-bearing path is a substantial improvement, but two checks
are missing from `check_reconfirmation`:

1. **Preserve the occurrence, not merely the quoted string.** At
   `crates/lex-core/src/lib.rs:657`, only the candidate span is checked. The prior
   offsets are never used. For prior text `art 31 bis y art 31 bis`, an approval
   at character range 4..10 can be migrated to 18..24 in new text
   `art  31 bis y art 31 bis`: that is the second citation, not the first after
   reflow (which would be 5..11). The source provision, target, span and prior
   evidence can all be identical, so `crates/lex-cli/src/main.rs:3220` does not
   distinguish these occurrences either. A decision can consequently be moved
   to a different citation, or one configured candidate can satisfy multiple
   persisted decisions on repeated identical citations. Require prior span
   validation and a whitespace-normalized position mapping; add a repeated-span
   regression and a two-decisions/same-target regression.
2. **Bind the prior text before comparing words.** At
   `crates/lex-core/src/lib.rs:646`, prior_text is compared to new_text but never
   hashed against the prior decision's source_text_sha256. Only the new hash is
   verified. If persisted provisions have drifted from their approved hash, a
   re-confirmation can bless those words as unchanged. Reject when the prior
   hash does not bind to the supplied persisted text; add a stale-prior regression.

These are guard defects, not evidence that committed LIC data was altered
incorrectly. The original LIC data acceptance remains intact. One-level evidence,
unchanged acquisition scope, and absence of a database re-confirmation round trip
remain the disclosed limits; this review does not expand that scope.

## Verification and disposition

A scratch Rust caller under `/tmp/lex-reconfirmation-review` exercises the public
`check_reconfirmation` function with JSON decision fields: first the repeated-span
case above, then the same case with the prior text hash replaced by 64 `b`
characters and that hash copied into reconfirms. Results are recorded below.
No production code or corpus data changed. No commit, push, merge or deployment.
Next action: repair both item 5 checks and re-review their negative regressions.

Native mechanical delegate: `cargo fmt --check`, `cargo clippy --workspace
--all-targets -- -D warnings`, `cargo test --workspace` (517 tests plus doc-tests),
and `cargo run --locked -p lex-cli -- validate` for lritf, ifpe-dcg-2021 and lic
all passed. LRITF/IFPE reported zero issues; LIC reported valid with two issues.

The scratch Cargo invocation failed (101) on a sandboxed Cargo-cache write.
The parent compiled the probe with rustc against the existing native-check rlibs
instead, avoiding that write; initial offset arithmetic was corrected to 18..24.
The corrected probe exited 0 and printed:

```text
moved to second occurrence: Ok(())
unbound prior text: Ok(())
```

Thus both negative cases are accepted by the guard. Parent execution was the
fallback after the delegated Cargo probe could not run in the sandbox.

## Follow-up review — guard fixes at 6bf0f9615

Handoff and working HEAD both bind to
`6bf0f9615defe7dc397c638169473da237e66c69`; initial worktree clean.
The reviewed diff from `548101bb7` changes core/CLI guards, regression tests and
the held-source contract documentation. No corpus changes are included.

- **Prior-text finding addressed:** `crates/lex-core/src/lib.rs:646` verifies
  the persisted text's SHA-256 against the prior decision before comparing
  normalized words. Candidate text binding remains enforced separately.
- **Occurrence finding addressed:** `crates/lex-core/src/lib.rs:665` checks the
  prior cited span, then `:675` maps both offsets through the whitespace reflow.
  Candidate offsets must equal that mapping, so an identical later citation
  cannot inherit the earlier decision. Mapping counts Rust characters, consistent
  with the reference offsets, rather than UTF-8 bytes.
- **Repeated-candidate routing addressed:** `crates/lex-cli/src/main.rs:3229`
  tries the matching candidates until the occurrence check succeeds, preserving
  a refusal if none succeeds. The regression supplies the second occurrence's
  candidate first and verifies that a single candidate cannot satisfy both
  persisted decisions. The core regression also covers valid reflow and misplaced
  prior offsets.

Scope remains unchanged: valid persisted reference records, one level of prior
evidence, no acquisition refresh, and exact cited-span text (internal span
whitespace changes are refused). A database round trip and the full LIC scratch
replay were not independently repeated in this review. These limits do not
reopen the two repaired findings.

**Validation:** the mechanical delegate ran `cargo fmt --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`,
and `cargo run --locked -p lex-cli -- validate` for lritf, ifpe-dcg-2021 and lic;
all passed. LRITF/IFPE have zero issues; LIC remains valid with two issues.
Both new named regressions were separately confirmed (one passed in lex-core,
one in lex-cli). The standalone rustc probe against the current compiled core
exited 0 and returned:

```text
moved to second occurrence: Err("re-confirmation offsets do not point at the same occurrence of the cited span")
unbound prior text: Err("re-confirmation refused: the persisted text does not bind to the prior decision")
```

**Verdict: accept item 5 within the stated scope.** Both findings from the
preceding re-review are closed. Item 3 remains accepted. No new blocking finding
in this bounded diff. This is not approval of the remaining upgrade work or a
legal-status decision. Only review documentation and the execution plan changed
in this session; no code/corpus edits, commits, pushes, merges or deployments.
Next action: resume the existing LIEPS grouping work queue.
