# Sequential canonical-state fold (Scope 2 Stage C's candidate engine)

Planning pass, 2026-07-31. Doctrine originally recorded the same day in
`docs/decisions.md` ("Operator doctrine: sequential canonical-state fold,
generalizing the decree-diff engine for both NOM and SHCP/CNBV cases"),
proposed after the ellipsis-completeness correction and the art. 115 LIC
pilot (`docs/plans/cnbv-art115-lic-consolidation.md`). Operator sign-off on
the doctrine arrived the same session via the directive "go for 1" — this
plan is the fresh planning pass M4's own rule requires before implementation
("do not start without a fresh planning pass and operator sign-off on the
trusted-boundary shape first", `docs/plans/maximasa-legal-integration.md`).

**This landing is deliberately partial, and the boundary matters.** It ships
the plan doc itself and the pieces of the data model that do not depend on
how post-transitorios annexes get represented. It does **not** implement
`apply()`'s real work (parsing a decree's own ellipsis-diff prose), does
**not** add a `standards consolidate` command, does **not** touch NOM-247 or
any other committed record, and does **not** resolve the annex-modeling
question. The operator is taking that question into a separate worktree
because it needs its own guidance — this plan does not anticipate or
constrain that decision.

## Problem

A NOM's retained text is its base publication; a SHCP/CNBV resolución's
retained text is the same shape (`docs/plans/cnbv-art115-lic-consolidation.md`).
Neither publisher issues a compiled current text the way Cámara de Diputados
statutes and CNBV disposiciones do. To hold a *current* text at all, Lex-Mex
has to build it — hashing each input (base plus every decree) and
compiling their combined effect, exactly the way Diputados and CNBV print
marginal reform notes on their own compiled texts.

Two prior findings converge on one mechanism:

- NOM MODIFICACIÓN decrees carry a whole-clause ellipsis diff ("unchanged
  span ... replacement in full") — Scope 2's original `decree-diff` framing,
  2026-07-27.
- The art. 115 LIC pilot found the *same* ellipsis mechanism one level
  deeper: inside a named unit's sub-parts, not only across a whole clause,
  plus a `recorriéndose los demás en su orden` renumbering instruction that
  ellipsis alone cannot represent.

**Why Stage C is not "an easy fix", restated.** NOM-247's second decree
modifies six numerals, adds `5.1.5`, eliminates three numerals, and
eliminates `Apéndice normativo A` — material the corpus does not model at
all and that the still-open `transitory-absorbs-annex` defect currently
swallows into a transitory. A consolidation engine cannot merely substitute
clause text; at least one real case requires it to delete an annex it has no
representation of. Stage C sits on top of that decision; this plan's scope
does not.

## The algorithm (doctrine, unchanged from `docs/decisions.md`)

```
canonical := base publication text
for decree in decrees sorted by DOF date:
    canonical := apply(canonical, decree)
```

`apply()` has exactly three per-unit operations:

- **`replace`** — the decree gives explicit new text for this unit.
- **`keep`** — ellipsis: content and position both unchanged.
- **`shift`** — the decree's own resolving-clause prose renumbers this unit
  ("recorriéndose los demás en su orden"): position moves, content does not.
  Collapsing this into `keep` is the specific way a naive fold silently
  mislabels a provision — what was "tercer párrafo" stays labeled third when
  the decree's own prose says it is now fourth.

Governing rules carried from the doctrine, load-bearing for any future
wiring:

1. Apply each decree against the *current* canonical state, not the original
   base every time — the bug a compiled draft actually made during the
   pilot.
2. A resolving clause's prose description (particularly derogations and
   reordering) is applied even when the decree gives no text restatement for
   that unit.
3. Cross-check a decree's own REFORMAN/DEROGAN/ADICIONAN lists against what
   its replacement body actually contains before applying anything — this is
   what surfaced both a 5-item gap list and 46 ellipsis-affected provisions
   in the pilot.
4. **No deletions, ever.** A repealed unit becomes `derogado` with the
   repealing decree's date/codigo recorded in place — never removed,
   nothing renumbered around it. This directly answers M4's open
   "retained-text strategy for derogation-caused span shifts" question: there
   is no span shift, because nothing is ever removed.
5. The fold is a plain sequential reduction per instrument — no graph
   structure needed. Cross-instrument reference resolution stays the
   separate, already-deferred concern it was.

## Scope of this landing

**In scope.**

- `StandardTextBasis::DerivedConsolidation`, the schema value Stage C's
  output will eventually assert. No committed standard uses it; landed now
  so the schema does not have to change again once wiring starts.
- `CanonicalFoldOperation` (`Replace { text }` / `Keep` / `Shift {
  new_position }`) — the three-operation type the doctrine specifies, with
  no fourth "delete" variant by construction, matching rule 4 above.
- `fold_unit`, a pure function folding one unit's chronological operation
  list into its current `(text, position)`. It is the isolated core of
  `apply()`'s per-unit bookkeeping — ordering, replace/keep/shift semantics —
  proven against a synthetic decree history. It does not read a decree's
  prose, classify ellipsis spans, or resolve REFORMAN/DEROGAN/ADICIONAN
  lists; that classification step is real Stage C work and stays unbuilt.

**Out of scope, explicitly.**

- Parsing a real decree's ellipsis-diff prose into `CanonicalFoldOperation`
  values. That is the actual hard part of Stage C and is not started.
- A `standards consolidate` (or equivalent) CLI command. Nothing wires
  `fold_unit` into `standards compile`, `refresh`, or any parser.
- Any change to NOM-247 or any other committed record. `derived_consolidation`
  exists in the schema; nothing sets it.
- The annex-modeling decision and `transitory-absorbs-annex`. Reserved by
  the operator for separate handling.
- The CNBV art. 115 LIC pilot's own consolidation. That plan
  (`docs/plans/cnbv-art115-lic-consolidation.md`) is the doctrine's other
  proving ground and is not touched by this landing.

## Verification

- `fold_unit` unit tests over a synthetic four-decree sequence exercising
  all three operations in order, asserting: `keep` carries text and position
  forward unchanged; `replace` changes text without touching position;
  `shift` moves position without touching content and is a distinct value
  from `keep` (`PartialEq`, not just behaviorally); a full chronological
  sequence folds left-to-right over the *previous* step's result, not the
  original base, ending in a `derogado`-style replacement.
- `StandardTextBasis` is matched exhaustively at every existing call site
  (`crates/lex-cli/src/main.rs`'s `standard_text_basis_name`) — the compiler
  enforced this: adding the variant broke the build until the new arm was
  written, exactly the guard M4's "layers consistent" rule is for.
- Schema: `derived_consolidation` added to `text_basis`'s enum in
  `schemas/standard-metadata.schema.json`, with a description stating it is
  reserved and unused. All 29 committed standards re-validate at 0
  violations, and a synthetic document asserting `text_basis:
  "derived_consolidation"` validates successfully against the updated
  schema (checked directly, not merely inferred from the enum edit).
- All 29 committed standards re-run through `standards refresh`: zero-byte
  diff corpus-wide, confirming this landing touched no committed data.
- 144 workspace tests pass (4 new), fmt clean, clippy clean.

## Acceptance, measured

| Criterion | Result |
|---|---|
| `CanonicalFoldOperation` has exactly the doctrine's three operations, no delete | 3 variants: `Replace`/`Keep`/`Shift` |
| `shift` is provably distinct from `keep`, not collapsible | Dedicated test asserts `PartialEq` inequality alongside the position-changed behavior |
| Fold is left-to-right over the running result, not the original base | Dedicated test: 4-step synthetic sequence, final state reflects step 4 applied after steps 1–3, not against the initial text |
| `derived_consolidation` is schema-valid and currently unused | 0 violations across 29 committed standards; synthetic document with the new value validates; no committed `standard.json` sets it |
| No corpus data moved | `standards refresh` across all 29 is a zero-byte diff |

## Next action

Full Stage C — parsing a real decree's ellipsis-diff prose into
`CanonicalFoldOperation` values and wiring `fold_unit` into a real command —
is blocked on the annex-modeling decision the operator is taking into a
separate worktree. That decision, plus a second, narrower sign-off on the
wiring step itself (distinct from the doctrine sign-off already given here),
are both prerequisites this plan does not resolve.
