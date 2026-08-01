# Architecture decisions

## 2026-07-31 — Packet-based review assignment landed: `batch_id` is the grouping key

Operator go-ahead. Staged 2026-07-28 (`docs/plans/maximasa-legal-integration.md`
M4 future scope, item 4) with two open questions: a grouping key, and a
reviewer-assignment record kept distinct from `legal_review_status`/
`technical_review_status`. Both decided here, as my own call, stated
explicitly rather than left implicit in the code:

**Grouping key: `batch_id`.** `batches/*.json` already groups the federal
corpus into legally coherent topical clusters for ingestion (`labor_L1_labor`,
`financial_F2_banking`, ...) — reusing it means a reviewer sees the same
grouping the ingestion plan already reasoned about, and no new categorization
had to be invented. A packet's `instruments` list is restricted to what
`corpus/mx` actually holds at generation time, since a batch manifest can
name instruments not yet ingested. Running `review-packets generate` against
the live corpus today produced 30 packets covering 147 of 181 committed
instruments (0 schema violations). **The gap is real, not a bug**: standards
(29) and the CNBV DCG family (`socap-sofipo-dcg-2006` and its siblings) have
no `batches/*.json` manifest at all — a separate ingestion track each — so
this landing does not cover them. Extending the grouping key to a second
source is future work, not silently done here.

**Record: `ReviewPacket` (`lex-core`), one file per packet under
`review-packets/`, new `schemas/review-packet.schema.json`.** `status`
(`unassigned`/`assigned`/`in_review`/`reviewed`), `reviewer`, `assigned_at`,
`notes` — workflow only. It carries no verdict and does not write to any
instrument's own review-status field; assigning a packet does not review
anything, it only names who is responsible. `lex-mex review-packets
generate` never overwrites an existing packet file (an already-assigned
packet's state survives a re-run); `assign <packet-id> --reviewer <name>`
refuses on any packet not currently `unassigned`, so reassignment stays a
deliberate, separate act rather than a silent overwrite.

**Deferred, per the plan's own text**: a way for a reviewer to flag a missing
backlink on the fly, and any link from a completed packet review back into an
instrument's `legal_review_status`. Neither is scoped here.

140 workspace tests pass (4 new), fmt clean, clippy clean.

## 2026-07-31 — Scope 2 Stage B landed: per-decree source hash on the existing `modifications[]` list

Operator sign-off. The 2026-07-29 decomposition note described Stage B as
"`source_url`/`source_sha256` become a list covering the base and every
decree" — read narrowly, that would add a second top-level array duplicating
`modifications[].official_url`. **The per-decree list already exists**: the
`modification` schema def has required `official_url` and `modifications` has
been an array since before Stage A. What was missing was a hash, not a
container. Stage B adds one field, `source_sha256: Option<String>`, to each
`modifications[]` entry (`StandardModificationSource`, `lex-core`), pinning
that decree's own source bytes with the same hash semantics as the base
publication's top-level `source_sha256` — not its extracted text, which has
no per-decree equivalent yet. Left unset, not `false`/`null`-equivalent,
whenever `included_in_source: true`: a decree already folded into the
retained base text (`nom-051-scfi-ssa1-2010`, `nom-187-ssa1-scfi-2002` today)
has no separate document to pin independently of the base hash.

**Additive by construction, verified, not asserted.** `#[serde(default,
skip_serializing_if = "Option::is_none")]` means omitting the field
serializes with no key. Confirmed two ways: a new lex-cli test round-trips a
modification with the field set and asserts an unset one has no
`source_sha256` key at all in the raw JSON; and all 29 committed standards
were re-run through `standards refresh` — `git status` on `corpus/` came back
empty, i.e. zero bytes changed anywhere, which is what an additive schema
change to a `skip_serializing_if` field is supposed to produce. Both schemas
re-validated with the Python `jsonschema` venv pass at 0 violations. No Rust
validator rule was added beyond the schema's existing `$defs/sha256` pattern
match — a hand-written check would only restate what the schema already
enforces, which M4's own "layers consistent, not redundant" expectation rules
out.

**Deliberately not built in this pass**: nothing fetches or hashes a decree's
source PDF. `standards compile` is untouched. The same acquisition-drift risk
argued against re-deriving the four CNBV instruments below applies verbatim
here — no queued compile needs a real decree hash yet, so none was computed.
Backfilling `source_sha256` for the five Maximasa NOMs' existing modifications
is future work, not required by this schema addition.

## 2026-07-31 — Marker-cap regex widened to three digits; the four affected CNBV instruments are NOT re-derived this pass

Operator go-ahead on the marker-cap defect found 2026-07-30 (`amendment_marker_regex`
`dcg.rs:176`, `legend_entry_re` `itf.rs:179`, both `\d{1,2}`, silently dropping
or misparsing any three-digit-or-higher margin marker). Both regexes, plus the
inline annex-heading marker sub-capture (`dcg.rs:294`, the landscape `ANEXO 14
(2)` form), are now `\d{1,3}`. Two new `dcg.rs` unit tests exercise a
standalone three-digit marker and a three-digit inline heading marker
directly against `parse_annex_document`; one new `itf.rs` test extends the
shared `parser-sample.txt` fixture with a genuine `150)` legend entry and
asserts it becomes its own `AmendmentReference` rather than being appended as
trailing text onto entry `9` — the exact swallow bug confirmed in
`cub-dcg-2005`'s committed legend. fmt clean, clippy clean, 135 workspace
tests pass.

**The four committed instruments the 2026-07-30 entry found affected
(`cucb-dcg-2004`, `cub-dcg-2005`, `oaac-dcg-2009`, `cue-dcg-2003`) are
deliberately NOT re-derived in this pass — their `provisions.json` still
carries the pre-fix data.** Re-deriving them the way `standards refresh`
re-derives a NOM requires the original retained extracted text; the DCG/ITF
instrument family has no such committed artifact (unlike a standard's
`standard.json` + retained text), and none survived locally from the original
ingestion (`.work/<slug>` is cleaned up after a successful `pipeline` run
unless `--keep-work` was passed, and it wasn't). The only way to re-derive
today is a live re-fetch through `pipeline`, and a direct check
(`curl -I` against `cub-dcg-2005`'s `official_url`) found CNBV has already
republished a newer compiled PDF since the 2026-07-13 ingestion:
content-length 4,866,388 → 6,144,432 bytes, ETag revision `,153` → `,156`,
`last-modified` 2026-07-03 → 2026-07-15. Re-fetching now would silently fold
a genuine legal update into a diff meant to be a scoped parser bug fix — the
exact conflation the still-unbuilt "Corpus currency" mechanism (`docs/decisions.md`
2026-07-12, "Amendment markers on CNBV reform transitorios") exists to
prevent by surfacing source drift as its own reviewed report rather than an
implicit side effect. Re-deriving these four stays pending that mechanism (or
an explicit, separately-authorized re-ingestion of each) — not done here.

**New known gap, worth stating plainly: the DCG/ITF instrument family has no
retention parity with standards.** A NOM's `standards refresh` can prove a
parser fix is safe against the exact bytes that produced the committed data,
hash-checked, with zero re-acquisition risk. No equivalent exists for CNBV
DCG/ITF instruments — a parser-only fix to this family can currently only be
verified by unit fixture, never mechanically replayed against its own
committed corpus entry. Not scoped to build here; flagged for whoever picks
up the corpus-currency mechanism, since the same retained-text pattern likely
solves both.

## 2026-07-31 — Review fixes on Stage A, same day: ten findings, all closed

A ten-finding code review ran against the Stage A landing commit
(`4ab432e89`); every finding was fixed the same day. The corpus effect is
message-precision only: warning counts are byte-identical corpus-wide (checked
old-vs-new across all 29 standards), no clause or transitory span moved, and
the only committed diffs are the 13 `validation.json` files whose
trailing-material warnings now name the actual heading instead of a byte
count. The durable decisions:

**One `LINE_LEAD` fragment feeds every line-anchored regex in `standard.rs`.**
The page-break (`\x0c`) admission from 2026-07-29 had reached only two of the
five line-anchored patterns. The three it missed made the fix
self-undermining: a form-fed ordinal line made the heading finder reject the
genuine TRANSITORIOS section (re-admitting phantom clauses *and* silencing
every downstream warning at once), and a form-fed TRANSITORIOS/APÉNDICE was
visible to the heading finder but invisible to span bounding, so the last
clause absorbed the whole transitorios section — or the last transitory
swallowed the annex and harvested its dates as `asserted_dates`. **The rule:
a character-class fix to line anchoring is applied through one shared
fragment or not at all** — partial application converts a visible defect into
a quieter one. Three regression fixtures.

**Run selection sees the full match list; the TRANSITORIOS boundary applies
to the winner afterwards.** Truncating candidates first shortens only
body-side runs (the índice sits before the boundary and never loses a row),
so a body with partially regex-invisible headings could lose the length
comparison to its own índice — the NOM-052 failure mode returning through
selection. A run must still *start* before the boundary, so post-transitorios
numbering that restarts at 1 cannot compete. Fixture:
`index-outnumbers-body-sample.txt`.

**`refresh` now has four guards, all before any write**: clause count,
transitory count (new — transitories move neither the clause count nor the
marks, so a transitory-parser regression previously rewrote committed data
with exit code 0 and left the corpus self-consistently wrong), amendment
marks (`--allow-mark-change`), and report validity (new — the old order wrote
the three files and bailed after, leaving invalid canonical data behind a
non-zero exit that a batch loop can miss). A refused refresh is now
byte-for-byte a no-op, and both new guards have tests asserting exactly that.

**Trailing-material detection is structural first, byte-count last.** The
flat 2000-byte threshold under-reported compact normative annexes (the exact
silent omission the warning exists for) and over-reported long signature
blocks. The transitorios section ending at an APÉNDICE/ANEXO heading is
itself the evidence — warned unconditionally, naming the heading; a
signature-closed tail is searched for a later annex-like heading (GUÍA,
LISTADO, TABLA included — in the *warning search only*; extending
`SECTION_END` itself would move committed transitory spans, which is the
held-out `transitory-absorbs-annex` defect, still gated); only a headingless
remainder falls back to the byte heuristic, measured after the last
"Rúbrica". All 13 committed warnings survived and now name their heading.

**Title parsing: date phrases are excluded before numeral matching, and annex
identifiers must start with a genuine capital or digit.** "del diverso
publicado el 30 de junio de 2011" could mint a false `amended_by` mark on an
unrelated clause numbered 30 or 2011 — a false *presence* mark, strictly
worse than any missed mark in this design — and a global `(?i)` case-folded
`[A-Z0-9]` so "Anexo de la ..." prose produced the bogus target "Anexo de".
Neither ever fired on the three recorded titles. Also: the verb-segmenting
regex and the verb→action classifier are now generated from one
`VERB_FAMILIES` table, making the "regex sees a family the classifier
doesn't" drift structurally impossible rather than merely tested against.

**Consumer surface**: `lex-mex instruments --json` now carries
`published_designation` (a reader is never shown a designation appearing
nowhere in the record's own retained text without the discrepancy marked) and
`amendment_marked_clauses`. Per-clause rendering is recorded in the Stage A
plan as *deferred* on the nonexistent standards Markdown profile, closing the
review's "neither delivered nor deferred" gap.

Also: constant regexes hoisted to `LazyLock` statics, the modification-target
derivation computed once per validation pass, the third `collapse_whitespace`
copy replaced with the crate-level helper, and `as_str()` added to
`StandardModificationAction` so prose messages reuse the serde token.

## 2026-07-31 — Landed Scope 2 Stage A: clause-level amendment marks derived from decree titles

Operator sign-off arrived as the directive "go for M4"; Stage A was the only
M4 item blocked on sign-off rather than on unscoped design, so it is what
shipped. Plan and full acceptance table: `docs/plans/standards-amendment-marks.md`.
**Stage A applies no text.** The sequential-fold doctrine recorded below is
Stage C and did not leak into this.

**What the change is.** A modifying decree names its targets in its own DOF
title. `StandardModificationSource` now carries that title verbatim as pure
input (`title: Option<String>`), and the parser derives from it: `amended_by`
on each matching `StandardClause` (the modification's index plus the decree's
own verb — `modified` / `added` / `eliminated`), and validation warnings for
every named unit matching no committed clause. A 252-clause instrument-level
"this standard has an unincorporated modification" warning becomes 17 marked
clauses and three named unresolved targets.

**Where derived data was allowed to live, and why it matters.** The plan put
`affects` on the modification, inside `standard.json`. That file is pure
passthrough — `validate` re-derives clauses and transitories and bails on
drift, but nothing re-derives metadata. Derived data there would go stale on a
parser change with no check firing: the same failure shape as the `\d{1,2}`
marker cap found on 2026-07-30. So the derived half was placed where
determinism checks already run — `amended_by` in `clauses.json` (reparse-and-
compare), unresolved targets in `validation.json` (report-compare) — and no new
canonical file was introduced. **The general rule this instance establishes:
derived output goes only where a re-derive-and-compare check already exists, or
that check is built in the same change.**

**Two things deliberately not done.**

- *No nearest-ancestor resolution.* A target matches a committed clause number
  exactly or it does not resolve. Attaching an unmatched target's mark to a
  parent clause would claim a decree addressed text it never named, and the
  unmatched case carries real information.
- *No action-erasing index.* `amended_by` is `Vec<StandardClauseAmendment>`,
  not the planned `Vec<usize>`, because a bare index cannot distinguish an
  eliminated clause from a modified one and the plan's own rendering example
  would print "Numeral modificado" for a repealed numeral. **A clause marked
  `eliminated` still carries its full live text** — the mark records the
  repeal, nothing applies it. That is the sharpest legal-meaning trap in this
  change.

**A finding that narrows the premise: the cheap lever is SSA1-shaped, not
universal.** SSA1 publishes "Modificación de los numerales 3.2, 3.10, ... de la
Norma Oficial Mexicana NOM-247-SSA1-2008"; STPS publishes "ACUERDO de
Modificación a la Norma Oficial Mexicana NOM-020-STPS-2011, ..." with no
numeral in the title at all. Title parsing therefore reports three distinct
states, never conflating them: targets found; a title recorded that names
nothing (`standard_modification_scope_unknown`, NOM-020 — scope *unknown*, not
*empty*); and no title recorded at all (`standard_modification_title_absent`).
Guessing in the second case would have been worse than the instrument-level
warning it replaced.

**A finding that strengthens an existing open question.** NOM-247's 2012 decree
modifies `5.2.7.ii.1)`, and the committed base text has no `5.2.5`, `5.2.6`, or
`5.2.7` at all — the `5.2` family stops at `5.2.4`. A *modificación* of a
numeral absent from the base is not the benign unresolved case (that is
`5.1.5`, an *adición* of new material). It is independent evidence for the
plan's standing open question that NOM-247 has at least four modifications, not
the two recorded — its 2011 CONSIDERANDO cites decrees of 2010-01-22 and
2010-07-19 that `standard.json` does not carry. Not resolved here.

**A defect found in review, before it reached the corpus.** The action-verb
regex was asymmetric: `reforma[sn]?` matched the conjugated `reforman`, but
`deroga(?:ci[oó]n(?:es)?|se)` did not match `derogan`. Because title segments
are cut *at* verb matches, an unseen family is not skipped — its targets are
absorbed by the preceding family. "Se reforman los numerales 3.2 y 3.4 y se
derogan los numerales 5.1 y 5.2" therefore parsed as a single *modified*
segment and recorded two repeals as modifications: mislabelling, which is
strictly worse than the "report scope unknown rather than guess" behaviour this
change is built around, and precisely the failure `action` exists to prevent.
Fixed by covering nominal and conjugated forms symmetrically for every family,
with the mixed-verb title as a regression test. None of the three recorded
titles used a conjugated form, so **no committed corpus data was ever wrong** —
all 29 standards revalidate byte-identically after the fix. **The general
lesson: in a parser that segments at keyword matches, uneven keyword coverage
does not lose data, it misattributes it.**

**New tooling, and a guard worth stating.** `lex-mex standards refresh <slug>`
re-derives a committed standard's parsed files from its retained text — the
counterpart to `validate`, and the mechanism that made this backfillable across
29 records without re-acquiring original PDFs. It aborts on a clause-count
change. It also aborts on *any* amendment-mark change unless
`--allow-mark-change` is passed: marks never move the clause count, so a
title-parser regression that drops or misattributes them would otherwise pass
every mechanical guard and be written straight into committed canonical data.
A printed count is not a check.

**Blast radius.** 26 of 29 committed standards refresh to a zero-byte diff.
Only `nom-247-ssa1-2008` (`standard.json`, `clauses.json`, `validation.json`)
and `nom-020-stps-2011` (`standard.json`, `validation.json`) changed. NOM-247's
`clauses.json` diff is `amended_by` insertions only — no clause text, id,
number, or span moved. **NOM-247 is one of the five NOMs in the Maximasa
bundle, so that out-of-repo bundle lock is now stale** and needs a mechanical
regeneration pass; not done here.

## 2026-07-31 — Operator doctrine: sequential canonical-state fold, generalizing the decree-diff engine for both NOM and SHCP/CNBV cases

Operator framing, proposed after seeing the ellipsis-completeness correction
above and the art. 115 LIC pilot (`docs/plans/cnbv-art115-lic-
consolidation.md`) as a whole. **Design proposal, awaiting sign-off — not
implemented, no schema/parser change made.** Recorded now so it isn't lost
before the formal M4 pass; folded into `docs/plans/maximasa-legal-
integration.md`'s M4 scope text alongside this entry.

**The algorithm.** For any instrument with no publisher-compiled text
(NOMs, and now confirmed also SHCP/CNBV resoluciones like art. 115 LIC),
build the canonical text as a **strict left fold over the decree history in
chronological order**, not by taking each unit's single "latest full
restatement":

```
canonical := base publication text
for decree in decrees sorted by DOF date:
    canonical := apply(canonical, decree)
```

1. Start from the original (base) publication text.
2. Apply each modifying decree **against the current canonical state**, not
   against the original base each time — verified today's compiled draft
   got this wrong (it took each key's last full restatement in isolation,
   which is only correct if that restatement's own ellipsis gaps happen to
   already be filled, which they are not).
3. When a decree's own `ARTÍCULO ÚNICO`/resolving clause carries a prose
   description (not a text restatement) — particularly derogations or
   reordering — apply that description as an operation in its own right.
   Confirmed necessary: `64ª`'s 2014-12-31 edit exists *only* as a preamble
   instruction, never restated.
4. Verify the único's REFORMAN/DEROGAN/ADICIONAN lists against what the
   decree's replacement body actually contains before applying anything —
   this cross-check is what surfaced both the 5 known gaps and the 46
   ellipsis-affected keys in the pilot; it is not optional bookkeeping.
5. **Ellipsis inside a decree's replacement text means the canonical text
   for that sub-part is unchanged — splice in only the explicitly given
   new text, leave everything under an ellipsis exactly as canonical
   already had it.** Confirmed directly against DOF's own source markup
   (`<span>...</span>`), not inferred.
6. **No deletions, ever. A repealed unit becomes `derogado` with the
   repealing decree's date/codigo recorded in place — it is never removed
   from the structure and nothing is renumbered around it.** This matches
   how DOF's own compiled texts actually render repeals ("Artículo N.-
   (Derogado, D.O.F. [fecha])") and directly answers the open question
   already sitting in M4's scope text ("a retained-text strategy for
   derogation-caused span shifts") — the answer is: there is no span
   shift, because nothing is ever removed or renumbered.

**The wrinkle found running this against the pilot, not yet in the five
points above when first proposed:** step 5's "ellipsis = unchanged" is
necessary but not sufficient. When a decree inserts a new paragraph and its
own prose states the later ones are "*recorriéndose los demás en su
orden*" (shifted down), an ellipsis-covered later paragraph's **content**
is unchanged but its **ordinal position moves**. A fold that treats
ellipsis as pure copy-forward-by-position would silently mislabel it —
e.g. what was "tercer párrafo" silently staying labeled third when the
único's own prose says it is now fourth. The único's prose (step 3/4) is
what carries this instruction; it cannot be inferred from the ellipsis
alone. **`apply()` therefore needs three per-unit operations, not two:
`replace` (explicit new text given), `keep` (ellipsis, content and
position both unchanged), and `shift` (position changes per the único's
own renumbering instruction, content unchanged) — collapsing `shift` into
`keep` is the specific way a naive implementation of this model breaks.**

**Generalizes, does not replace, the existing NOM-side scoping** (M4 /
`docs/plans/maximasa-legal-integration.md`, `docs/ingestion-difficulty-log.md`
`decree-diff`): a NOM MODIFICACIÓN's whole-clause ellipsis span
("unchanged span ... replacement text", `docs/decisions.md` 2026-07-26) and
this SHCP/CNBV sub-part ellipsis (found today) are **the same mechanism at
different nesting depth** — clause-level for NOMs, sub-part-of-a-named-unit
level for SHCP/CNBV resoluciones. One fold model, one `apply()` with the
three operations above, covers both; the earlier framing of them as
distinct engines ("simpler than M4" vs. "needs the ellipsis engine") is
superseded by this doctrine.

**Loop, not graph, for the per-instrument fold.** The fold above is a plain
sequential reduction over one instrument's own decree history — no graph
structure is needed for it. Cross-instrument reference resolution
(`docs/cnbv-consolidated-disposiciones.md` §3) is a separate, already-
deferred concern and should stay separate rather than being merged into
this fold.

## 2026-07-31 (correction, same day) — "para quedar como sigue" is a nested ellipsis-diff, not a full-text replacement; the entry below is wrong on its central claim

Operator caught this directly in the compiled output, not in the source
reading: disposición 4ª's "current text" (from RM 2024-08-28, codigo
5737473) reads `... ... I. ... a) ... i. Primer apellido... ii. ... iii.
...` — riddled with literal `...`. Checked against the source RM's own
REFORMAN clause for 4ª: it names only specific numerales/incisos/fracciones
("la 4ª, segundo párrafo, fracciones I, incisos a), numeral i. y b),
numerales i. primer y segundo párrafos y iii. segundo párrafo, II, incisos
a), numeral x., IV,..." — dozens of pinpointed sub-parts, e.g. fracción III
of 4ª is conspicuously absent). The `...` in the replacement body **is DOF's
own convention for "this sub-part is untouched, carries over unchanged from
the immediately prior version"** — confirmed literally present in the raw
DOF HTML (`<span>...</span>`), not an extraction artifact.

**This retracts the entry below's central claim.** Quoted from it, struck,
not silently edited:

> ~~This SHCP/CNBV format instead names the unit by number and gives its
> entire new text, so applying it is a keyed full-text replacement
> (`disposición_id → new_text`), not a splice.~~
>
> ~~named-unit full-text replacement needs no ellipsis-splicing logic at
> all, just chronological last-writer-wins per disposición number~~

**What's actually true:** "para quedar como sigue" restates the numbered
unit, but only fills in the parts the RM actually touches — everything else
inside that same unit is elided with `...` and must be spliced in from the
nearest prior version that gives real text for that specific sub-part
(walking back through the key's own history, recursively, since an
intermediate version may *also* have elided the same sub-part if it wasn't
touched there either). This is the NOM ellipsis-diff mechanism after all —
just operating one level of nesting deeper (inside a named unit) instead of
across the whole document. The "simpler than M4, no splicing needed" framing
is wrong; **this instrument needs real ellipsis-splicing, scoped to
sub-parts of a named unit rather than the whole document.**

**Scope, measured against the pilot compiled draft (2026-07-31):** grepped
`compiled_draft.json`'s 96 `current_text` values for literal `...` — **46 of
96 keys are affected**, not a handful. Worst cases: `4a` (135 ellipses,
source 5737473), `64a` (35), `14a` (28), `74a` (16). None of these 46 are
corrected in the compiled draft or its Obsidian copy as of this entry — they
carry a blanket warning, not a per-provision fix, pending the real
splice-reconstruction work below.

**What survives from the entry below, unretracted:** the extraction
mechanics (locate `ARTÍCULO ÚNICO`, parse REFORMAN/DEROGAN/ADICIONAN lists,
split "para quedar como sigue" by disposición-number headers) are still
correct and reusable — they just produce an *elided* text per touched unit,
not a complete one. The wholly-derogated-numeral finding (`7ª-1`) and the
prose-only-structural-edit finding (`64a`'s 2014-12-31 edit) both still
hold. The DEROGAN/REFORMAN-list-vs-body cross-check discipline from that
entry's caveat is exactly the right instinct, generalized further by this
correction: **never trust a replacement body's completeness without
checking it against what the RM's own preamble says it touched.**

**Next, not done this session:** build the recursive splice — for each `...`
span in a key's current text, identify which specific sub-part (fracción/
inciso/numeral, by the ordinal immediately preceding the `...`) it stands
for, and walk that key's history backward for the nearest version that gives
real text for that same sub-part. This is materially more work than the
keyed-replacement model assumed and should be scoped as its own pass, not
rushed.

## 2026-07-31 — A second, simpler decree-diff subclass: SHCP/CNBV "ARTÍCULO ÚNICO" named-unit replacement

Found while inventorying DOF resoluciones modificatorias for *Disposiciones
de carácter general a que se refiere el artículo 115 de la Ley de
Instituciones de Crédito* (no consolidated text exists — see
`docs/plans/cnbv-art115-lic-consolidation.md` for the full inventory and
research trail). This instrument has no publisher-compiled text (like a
NOM) but its modifying decrees are **not** shaped like NOM decrees.

**The format.** Every RM read so far (2 of 16, both confirmed) has a single
resolving clause of this exact shape:

> ARTÍCULO ÚNICO.- Se REFORMAN las disposiciones `<N, N, N...>`; se
> DEROGA(N) `<N, N...>` (sometimes only a named párrafo/fracción inside a
> disposición that is *also* being reformed); y se ADICIONAN `<N, N...>`;
> todas de las Disposiciones..., **para quedar como sigue:** `<the complete
> new text of every disposición named in the REFORMAN/ADICIONAN lists, in
> full, numbered>`.

**Why this differs from the NOM case** (`docs/decisions.md` 2026-07-26,
`docs/ingestion-difficulty-log.md` `decree-diff`): a NOM
MODIFICACIÓN/ACUERDO states its diff as an **ellipsis span** — "unchanged
text ... replacement text" — which requires splicing the replacement into
the retained base at the right offset. This SHCP/CNBV format instead names
the unit by number and gives its **entire new text**, so applying it is a
keyed full-text replacement (`disposición_id → new_text`), not a splice. A
disposición touched by both a reform and a partial derogation (e.g. one
párrafo removed) still appears exactly once, in full, under "para quedar
como sigue" — the derogated párrafo is simply absent, so partial-
derogation-within-a-reformed-unit needs no special handling. A **wholly**
derogated disposición (repealed outright, not reformed) would be named only
in a "se DEROGA" clause with no replacement text and must be dropped from
the compiled output — not yet observed directly, needs confirming against a
real instance.

**Consequence for Scope 2 / M4.** `docs/plans/maximasa-legal-integration.md`
scoped the `decree-diff` engine around the NOM ellipsis case. This SHCP/CNBV
format is a **distinct, mechanically simpler** subclass — named-unit
full-text replacement needs no ellipsis-splicing logic at all, just
chronological last-writer-wins per disposición number with full history
retained as provenance. Do not assume every `decree-diff` instrument needs
the heavier ellipsis engine; check which shape a given publisher uses
first. Worth generalizing once more instruments are seen: this is plausibly
how *most* SHCP/CNBV resoluciones modificatorias are written (it matches
Mexican regulatory drafting convention — "para quedar como sigue" is the
standard formula), so it may cover more of the CNBV-adjacent, no-
consolidated-text corpus than the ellipsis case does. This methodology
should be reused directly the next time a corpus instrument is updated
against a new DOF reforma once the current corpus is digested — the
extraction pattern (locate `ARTÍCULO ÚNICO`/multi-artículo resolving
clause, parse the REFORMAN/DEROGAN/ADICIONAN lists, split "para quedar como
sigue" by disposición-number headers) is the reusable part, independent of
which instrument it's applied to.

**Caveat found running this against all 16 RMs of the pilot instrument
(2026-07-31, `docs/plans/cnbv-art115-lic-consolidation.md`): "always full
text" is not universal.** Two real counterexamples:

- A **purely structural edit** (paragraph renumbering, a single-paragraph
  derogation with no other substantive change) can be stated as an
  instruction entirely inside the preamble, with **no** "para quedar como
  sigue" restatement anywhere — the numeral is named in the DEROGAN/REFORMAN
  lists but never appears in the replacement body. Mechanical extraction
  keyed only on replacement-body headers silently misses this; it must be
  caught by cross-checking every DEROGAN/REFORMAN-list numeral against the
  set that actually appears in the replacement body, and flagging the
  difference rather than assuming absence means "not actually changed."
- A **wholly derogated numeral** (repealed outright, not reformed) is named
  only in a DEROGA clause with no replacement text, confirmed for real:
  disposición "7ª-1" — note the hyphenated sub-numeral format, which a
  numeral-key normalizer built only for "Nª [Bis/Ter/...]" will silently
  fail to recognize at all, losing the provision's entire lifecycle rather
  than just its latest state.

Net: the named-unit full-text-replacement shape is still the dominant case
and still simpler than the NOM ellipsis-diff, but a correct extraction must
diff the DEROGAN/REFORMAN/ADICIONAN *lists* against the replacement *body*
per RM, not just parse the body — the gap between those two sets is where
both counterexamples were found.

## 2026-07-29 — Lex-Mex as compiler for NOMs; Scope 2 decomposed; a dangling amendment legend

Operator framing, and it identifies a real asymmetry: for Cámara de Diputados
statutes and CNBV disposiciones the publisher issues a consolidated text, so
provenance is "hash the one file they publish." NOMs have no such file. To
hold a current version at all, **Lex-Mex would have to do the compilation
itself** — hashing each input document (base publication plus each
modificatorio) rather than one source, keeping only current text, and
annotating each modified clause with the decree that changed it, exactly the
way Diputados and CNBV print marginal reform notes. Superseded versions would
not be retained, only pointed at.

Scope 2 is therefore decomposed into three stages with different dependencies:

- **Stage A — mark, don't modify.** Parse the numerals a decree names in its
  own title, attach them to the clauses they target with the decree's date and
  URL. Turns `standard_unconsolidated_modification` from an instrument-level
  warning into a clause-level one. Requires no text transformation, no new
  text basis, and nothing from the unresolved annex defect. Planned in
  `docs/plans/standards-amendment-marks.md`; **not started, awaiting sign-off**
  per the M4 rule.
- **Stage B — multi-source provenance.** `source_url`/`source_sha256` become a
  list covering the base and every decree. A schema question, separable from
  any text change.
- **Stage C — actual consolidation**, asserting a `derived_consolidation` text
  basis.

**Why Stage C is not "an easy fix", stated with the counterexample.**
NOM-247's second decree performs four distinct operations: modifies six
numerals, *adds* 5.1.5, *eliminates* three numerals, and *eliminates Apéndice
normativo A*. That last deletes post-transitorios material the corpus does not
model at all and that `transitory-absorbs-annex` currently swallows into a
transitory. A consolidation engine cannot merely substitute clause text; it
must delete an annex it has no representation of. Stage C therefore sits on
top of the annex decision, while Stage A does not.

**The verification story changes, and must be asserted deliberately.** For a
statute, correct means "matches the file Diputados published." For a
compilation Lex-Mex produces there is no such file, so the guarantee becomes
hashed inputs plus a deterministic transform yielding byte-reproducible
output, gated on a named human review. That is defensible but is a different
claim than the corpus makes anywhere today, and `text_basis` is where it would
be made. It must not arrive implicitly as a side effect of Stage C.

**Correction, 2026-07-30: the paragraph below is wrong and superseded.** The
claim that the amendment legend is empty everywhere was measured against the
wrong field and never checked against the actual `amendment-references.json`
sibling files. It is retracted, not merely refined — see the 2026-07-30 "CNBV
legend re-pass" entry below for the corrected per-instrument counts (every
legend is populated, every current mark resolves) and the real, different
defect the re-pass found (a marker-regex digit cap that silently drops some
markers ≥100). The provision-marked counts in the paragraph below (507, 264,
260, 245, 204, 152, 96, 88, 28) are the only part that still stands; the
legend-entry-count claim ("not one instrument has a single legend entry") does
not and should be disregarded.

> Defect found while surveying the precedent: the amendment legend is empty
> everywhere it is used. `amendment_marks` on a provision resolves through the
> instrument's `amendment_references` legend, and is rendered into Markdown
> frontmatter. Across nine committed CNBV instruments, 1,844 provisions carry
> marks and not one instrument has a single legend entry
> (`socap-sofipo-dcg-2006` 507, `cucb-dcg-2004` 264, `cub-dcg-2005` 260,
> `oaac-dcg-2009` 245, `scap-dcg-2012` 204, `fi-dcg-2014` 152, `cue-dcg-2003`
> 96, `itf-dcg-2018` 88, `servinv-dcg-2013` 28).

## 2026-07-30 — CNBV legend re-pass: the 2026-07-29 "empty everywhere" claim was wrong; real defect is a 2-digit marker cap

Operator authorized a re-pass on the CNBV `amendment_references` legend
question before Stage A sign-off. Re-measuring against the actual
`corpus/mx/<id>/amendment-references.json` sibling file (not whatever field
the 2026-07-29 count read):

| Instrument | Provisions marked | Unique marks | Legend entries | Unresolved marks |
|---|---:|---:|---:|---:|
| `socap-sofipo-dcg-2006` | 507 | 62 | 97 | 0 |
| `cucb-dcg-2004` | 264 | 49 | 99 (capped)† | 0 |
| `cub-dcg-2005` | 260 | 40 | 99 (capped)† | 0 |
| `oaac-dcg-2009` | 245 | 47 | 99 (capped)† | 0 |
| `scap-dcg-2012` | 204 | 24 | 42 | 0 |
| `fi-dcg-2014` | 152 | 28 | 47 | 0 |
| `cue-dcg-2003` | 96 | 64 | 99 (capped)† | 0 |
| `itf-dcg-2018` | 88 | 14 | 18 | 0 |
| `servinv-dcg-2013` | 28 | 5 | 8 | 0 |
| `ifpe-dcg-2021` | 0 | — | n/a — no `amendment-references.json`; uses `formal-source-manifest.json` / `annex-source-manifests.json` instead | — |

`adapters/cnbv/` lists ten instruments, not nine; `ifpe-dcg-2021` is the
tenth. It carries zero `amendment_marks` and no legend file at all — it
appears to use a more evolved multi-source provenance shape already
(relevant to Stage B, not investigated further here). † = see the capped-
legend defect below; "99" undercounts the real legend length for these four.

Every mark currently on a committed provision resolves to a legend entry —
the marks⊆legend validator is not vacuously passing here, it is genuinely
satisfied for every mark the parser currently produces. The extraction lives
in `crates/lex-parse/src/itf.rs` (`flush_legend`, sharing
`amendment_marker_regex` with `crates/lex-parse/src/dcg.rs`) — I had read
`dcg.rs` alone and concluded, wrongly, that no REFERENCIAS extraction
existed at all.

**A real, more serious defect than "legend gap": silent loss of live
markers, confirmed in committed data.** Both `amendment_marker_regex`
(`dcg.rs:176`, `^\((\d{1,2})\)$`) and `legend_entry_re` (`itf.rs:179`) match
only one or two digits, capped at 99. This is not just a legend-formatting
issue: the *same* regex is used to recognize a margin marker in the body
text, so a `(100)`-or-higher marker on a provision fails to parse as a
marker at all and falls through as ordinary content. Checked directly —
`grep -oE '\(1[0-9]{2}\)' corpus/mx/<id>/provisions.json` for the four
capped instruments finds three-digit parenthesized tokens sitting inside
committed provision text today: `cucb-dcg-2004` (`(100)`, `(101)`, `(106)`,
`(108)`–`(111)`, `(113)`, `(114)`, …), `cub-dcg-2005` (`(104)`, `(105)`,
`(108)`, `(112)`–`(115)`, up through `(148)` and beyond), `oaac-dcg-2009`
(`(102)`–`(115)`), `cue-dcg-2003` (`(101)`–`(108)`). These are real
provisions whose amendment attribution is currently **dropped**, not merely
undocumented — the exact "silent loss" the 2026-07-12 entry's
marks⊆legend invariant was built to catch, except that invariant checks the
markers the parser *did* extract, so it cannot see the ones the regex threw
away before they became a mark. On the legend side, the same cap folds
`100)  …` lines into legend entry 99's `description` as trailing text
instead of a new entry — `cub-dcg-2005` alone has ~299 such embedded
`\d{1,3}\)` fragments inside entry 99, with dates running to 2026-07-03.
Not fixed in this pass — widening both regexes to `\d{1,3}` (or unbounded),
re-splitting the four affected legends, and re-extracting the swallowed
in-body markers is a scoped follow-up, distinct from Stage A, and higher
priority than its "legend gap" framing implied.

**Conclusion for Stage A:** the "populate the CNBV legend" fork of the
Stage A sign-off question is moot — the legend mechanism works correctly for
everything the parser currently recognizes as a marker. The open sign-off
item remains the `StandardModificationTarget` / `amended_by` shape itself;
the marker-cap defect above is new, separate work.

## 2026-07-30 — Trusted-compiled-source-first correctness; self-compilation is the fallback

Operator correction to the 2026-07-29 "verification story changes" framing
above. Restated in the operator's own terms: compiling everything ourselves
is more difficult and riskier than pointing at a compiled version that
already exists and is trusted to stay correct. So:

- **If a trusted compiled source exists for an instrument, "correct" means
  matching that source's file.** Self-compilation (hashed inputs + a
  deterministic transform, byte-reproducible, gated on named human review —
  the 2026-07-29 framing) is the *fallback*, used only when no trusted
  compiled source exists.
- **Trust is manually triggered, per source, never inferred.** Trusted today:
  **Diputados** and **CNBV**. **Not trusted (known uncompiled)**: the source
  referred to as "pagiina" in the operator's message — name unconfirmed
  against the adapters directory this session; needs to be pinned down
  before it is used anywhere that assumes trust.
- **Multi-source is not universal.** A single published text with no
  modificatorios can legitimately rest on one file — normally the DOF
  version when no Diputados version exists. The never-modified, single-
  source, not-in-Diputados instrument (a new NOM with no amending decree
  yet) is the outlier case the model must keep representable, not an error
  state.

**Reconciling against existing fields.** `SourceManifest` and `Instrument`
(statute/DCG side) already carry `operational_source` /
`formal_publication_source` (e.g. `cub-dcg-2005`: `operational_source:
"cnbv"`, `formal_publication_source: "dof"`) — structural support for
"which source is the one we point at" already exists there, but nothing
marks *that* source as manually-triggered-trusted versus merely preferred.
`StandardMetadata` (NOM side) has no equivalent field at all — only
`source_url` / `official_dof_url` / `official_registry_url` /
`text_basis: AsPublished | OfficialCompilation`. Neither side has an
explicit trusted-source boolean/enum today.

**Not implemented.** This entry records the doctrine; it does not add a
field. The natural next step is a `trusted_compiled_source: bool` (or an
enum naming which source) alongside `operational_source`, extended to
`StandardMetadata`, plus a validator rule that `text_basis:
OfficialCompilation` requires a trusted source and `AsPublished` does not.
That is implementation work and needs its own pass, not a byproduct of this
correction.

## 2026-07-29 — Reviewer data corrections; `published_designation` added

Three corrections supplied by the same review, kept out of the parser commit
so a regression in one could not hide behind the others.

**NOM-020-STPS-2011's recorded modification was wrong.** It cited an
alternative-procedure acuerdo of 2014-12-09, which the reviewer identified as
not the modification at all. The real one is the *ACUERDO de Modificación*
signed 15 September 2015 and published in the DOF on **2015-10-19**
(`codigo=5411988`), whose entire operative content is `ÚNICO. Se elimina el
inciso j) del numeral 13.2`. The modification record now carries that date and
URL. The distinction matters generally: platiica lists "procedimiento
alternativo autorizado" records alongside real modifications, and only the
latter make retained clause text stale.

**NOM-025-STPS-2008's effective date is 2009-03-01.** Its text sets entry into
force two months after a 2008-12-30 publication, landing on a nonexistent 30
February; the registry's own field reads an impossible `2009-02-29`, which is
why it was left `null` on ingestion. The reviewer supplied the governing rule:
an obligation falling on a non-existent or non-working day moves to the next
available working day. Recorded as a real date rather than an absence.

**`published_designation` added to `StandardMetadata`.** The reviewer granted
authority to apply a designation rename wherever the registry shows one, and
to leave it alone where it does not — noting the asymmetry: ECOL became
SEMARNAT, but SCFI persists in NOM-051 and NOM-187 even though the Secretaría
de Comercio y Fomento Industrial became the Secretaría de Economía. So the
prefix follows the *registry's* redesignation, not the authority's rename.

Verified before applying rather than assumed: across all committed standards,
every `designation` appears in its own retained text. NOM-002-SEMARNAT-1996
would have been the first exception, since its text is titled
NOM-002-ECOL-1996. Applying the rename silently would have broken a
corpus-wide invariant with nothing recording it, so the optional
`published_designation` field carries the published form and raises a
`standard_redesignated` warning; `validate_metadata` errors if it is recorded
when the two designations are equal, so it cannot become a general
"former name" field. Schema, `lex-core` type, validator, test and
`docs/standards-module.md` landed together, per the trusted-boundary rule.

NOM-002-SEMARNAT-1996 is now ingested (73 clauses, 3 transitorios), closing
the last of the six batch-2 flags that did not depend on the unresolved
`transitory-absorbs-annex` defect.

## 2026-07-29 — A standard's normative body ends at TRANSITORIOS

Reviewer-supplied domain rule, answering the batch-2 flag report: **a NOM's
normative numbered body ends at TRANSITORIOS.** What follows may be
apéndices, anexos, tablas or listados — sometimes normative — or an
explicitly non-binding "Guía de Referencia", but it is never
clause-structured, and it needs different extraction rules than the body.

Two parser changes follow from that rule. Both are bounded to
`parse_standard_clauses`; the data corrections the same review supplied
(NOM-020's modification date, NOM-025's effective date, the ECOL→SEMARNAT
designation pattern) are deliberately *not* in this change, so a regression
in one cannot hide behind four others.

1. **The clause run is bounded at the real TRANSITORIOS heading.** The
   índice-disambiguation Scope 1 built for transitorios is extracted as
   `real_transitorios_heading` and now serves both paths, so the clause and
   transitory parsers can never disagree about where a body stops. This
   closes `annex-continues-numbering` and `annex-form-numbering`. Note the
   2026-07-28 log entry explicitly *rejected* "cut at the first TRANSITORIOS
   marker" — correctly, since NOM-019's índice lists TRANSITORIOS before its
   body. Cutting at the **real** occurrence is the same idea done right, and
   the entry predicted exactly that.

2. **Form feed joins the leading-whitespace class.** `pdftotext` emits
   `\x0c` immediately before a page's first line with no intervening
   newline, so any heading landing on a page boundary never matched the
   line-start anchor. This — not a length contest — is why
   NOM-052-SEMARNAT-2005 compiled its índice: its real body was *invisible*,
   leaving the índice as the only candidate run. Verified safe before
   landing: DOF running headers ("6 (Edición Vespertina) DIARIO OFICIAL")
   also follow a form feed, but carry no ordinal period and open with `(`,
   so `plausible_top_level` already rejects them. Simulated across all 26
   committed standards, admitting the form feed changed zero clauses.

**Regression verification was deep equality, not counts.**
`standards validate` reparses committed text and bails if the result
differs from committed `clauses.json`/`transitories.json` by any field.
All 26 previously-committed standards passed unchanged, so every clause
ID, label, text and span is byte-identical.

**New check: `standard_trailing_material`.** Bounding the body at
TRANSITORIOS is correct but drops content that is sometimes normative —
NOM-052's Listados carry the hazardous-waste classifications the standard
exists to establish; NOM-010's Apéndice I carries its exposure limits.
Without a signal, a compiled standard would present a complete-looking
clause body while omitting operative content. `validate_standard` now warns
when substantial text follows the transitorios section. It fires on 13
standards and deliberately does not distinguish normative from non-binding
trailing material — making that call requires reading it, which is the
point of surfacing it. Regenerating 11 committed `validation.json` files
for the new warning changed nothing else: clauses, transitorios, retained
text and metadata all compared byte-identical.

**Newly found, deliberately not fixed here: `transitory-absorbs-annex`.**
`section_end_marker` ends the transitorios section only at a signature
marker, `APÉNDICE`, or `ANEXO`. Trailing material introduced any other way
— usually `Guía de Referencia I`, sometimes bare `Tabla N.` — is absorbed
by the *last transitory*. Pre-existing, not caused by this change, and
affecting committed standards (NOM-027's TERCERO is 22,370 chars;
NOM-085's QUINTO 27,938; NOM-020's QUINTO 15,327). It is why
NOM-019-STPS-2011, NOM-024-STPS-2001 and NOM-052-SEMARNAT-2005 stay held
out even though their clause defects are fixed: ingesting them would mean
committing transitorios known to contain tens of thousands of characters
of guide and table text. Not fixed in the same pass because adding `Tabla`
as a section-end marker could truncate legitimate transitory text that
references a table inline, and because the reviewer has indicated
post-transitorios annexes need their own extraction approach — so *how* to
model them is an open design question, not a marker list.

Ingested under the fix: NOM-010-STPS-2014 (206 clauses, was 950 phantom)
and NOM-035-STPS-2018 (111 clauses, was 124 with questionnaire rows), both
carrying `standard_trailing_material`.

**Reviewer finding on NOM-247-SSA1-2008, recorded for Scope 2.** Its two
modifying decrees name in their own titles exactly which numerals they
touch: the first modifies `1.4, 2, 3.2, 3.10, 3.12, 3.17, 3.18, 3.19,
3.36, 3.44 y 8`; the second modifies `3.2, 3.10, 3.33, 4, 5.1.1,
5.2.7.ii.1)`, adds `5.1.5`, and eliminates `5.2.2.8, 5.2.3.4, 5.2.4.5` and
`el Apéndice normativo A`. That makes a cheap intermediate capability
possible well before the full decree-diff engine: parse the affected
numerals from a decree's title and attach them to the modification record,
turning `standard_unconsolidated_modification` from an
instrument-level warning into a clause-level one — a reader could then see
that NOM-247's clause 3.2 specifically is stale, rather than that the
standard as a whole is. Not implemented; recorded as the natural first
increment of Scope 2.

## 2026-07-28 — Batch 2 complete: validation cannot detect wrong-run selection

Batch 2 ran to completion across all 27 candidates: 21 ingested, 6 held
out and flagged (`docs/plans/nom-standards-batch-2.md`,
`docs/ingestion-difficulty-log.md`). The corpus went from 160 to 177
instruments and from 1,487 to 3,238 standard clauses.

**The load-bearing finding: every one of the five parser failures in this
batch reported `valid; 0 issues`.** `validate_standard` checks that the
clause run it is handed is internally consistent — ascending numbering,
non-overlapping spans, offsets inside the text — and never asks whether
`numbered_body_run` selected the correct run. A standard can therefore
compile "clean" while carrying 744 phantom clauses absorbed from an
appendix table (NOM-010-STPS-2014), or while missing its entire body
because the índice was compiled instead (NOM-052-SEMARNAT-2005, 1.1%
clause-span coverage). Absent the hold-out-and-flag policy adopted
earlier the same day, all five would now be committed canonical data,
hashed into Maximasa's bundle lock, indistinguishable from correct
records by any automated check this repository has.

Three failure classes were newly identified and named:
`annex-continues-numbering` (the run does not stop at the terminal
heading; a following annex/table/questionnaire continuing the same
numeric sequence is absorbed — NOM-010, NOM-035, NOM-024),
`indice-selected-as-body` (the table of contents is itself a complete
consecutive run and wins selection outright — NOM-052), and
`metadata-ambiguity` applied to a legal-identity rather than parser
question (NOM-002-SEMARNAT-1996, below). These join
`annex-form-numbering` from tranche 1 (NOM-019-STPS-2011, where an annex
form's independently *restarted* numbering out-competed the real body on
raw length — a distinct mechanism from the three above, not a variant).

Two discriminators caught every case and are deliberately *not* yet
implemented as checks, because doing so is a validator change that should
be reviewed on its own terms rather than folded into an ingestion batch:
clause-span coverage as a fraction of document length (the índice failure
scored 0.011; every correct instrument scored ≥0.31), and whether the
selected run terminates at a Bibliografía/Concordancia heading. Promoting
both into `validate` is the recommended next code change; three of the
four parser classes are plausibly one shared fix in `numbered_body_run`.

**NOM-002-SEMARNAT-1996 is a decision, not a defect.** Its text parses
cleanly, but the retained official text is titled NOM-002-**ECOL**-1996
and names SEMARNAP as issuer, while the registry indexes it as
NOM-002-**SEMARNAT**-1996 under SEMARNAT. Committing it would assert a
`designation` appearing nowhere in its own source text, and
`StandardMetadata` has no field for "published as X, now indexed as Y."
Not resolved by convention here; held for reviewer direction. This will
recur for every pre-2000 ECOL-era environmental NOM, so the decision sets
a pattern rather than settling one record.

Recorded conservatively rather than assumed harmless during this batch:
NOM-020-STPS-2011's *ACUERDO de modificación* (2014-12-09) is recorded as
an unconsolidated modification, giving it a standing warning that its
clause text is not current until that decree is checked — the same
treatment NOM-247 carries. "Procedimiento alternativo autorizado" records
on other standards were *not* treated as modifications: they authorize
compliance methods, not text changes. NOM-025-STPS-2008's
`effective_date` is `null` because its text sets entry into force two
months after a 2008-12-30 publication (a nonexistent 30 February) and the
registry's own field reads an impossible `2009-02-29`; an invented date
would have been worse than an absent one.

A flag report for reviewer triage was written to Spearhead at
`20_Repos/Lex-Mex/NOM Batch 2 — Flagged Instruments for Review.md`.

## 2026-07-28 — Batch NOM ingestion: hold-out-and-flag policy, review deferred to packets

Decision, operator-directed: treat NOM ingestion as a general batch process
rather than one-at-a-time, applying the existing federal cluster-2 "Batch
operating loop" discipline (`docs/project-status.md`) with two changes.

1. **Hold out, don't force through.** When an instrument hits a structural
   difficulty that isn't a quick, obviously-correct fix (a new
   signature-block variant, a decree-diff case, an ordinal restart, an
   unresolved acquisition question), it is held out of `corpus/` entirely
   and flagged in the new `docs/ingestion-difficulty-log.md`, rather than
   compiled with a known defect. Considered and rejected: compiling anyway
   with the defect marked on the record. Rejected because `corpus/` is
   committed canonical data that Maximasa's bundle lock hashes and
   consumes directly — a known-wrong structure riding along, even flagged,
   is a defect in canonical data rather than an absence from it. Operator
   confirmed this explicitly when asked.
2. **Review no longer gates ingestion, and is deferred to packets.**
   Standards' `legal_review_status`/`technical_review_status` already
   supported an unreviewed state independent of successful compilation —
   nothing mechanical changes here. What changes is the *operating*
   assumption: JRH (or another reviewer) reviewing every ingested
   instrument one at a time is not workable at batch scale. Once enough
   instruments are ingested, they will be grouped into review packets
   (e.g. "industrial food processing pack") and handed to assigned
   reviewers — plural, not only JRH — as a batch. This is a change to
   *when and by whom* review happens, not to the reviewer-of-record rule
   itself; JRH's role for material already reviewed under the old flow is
   unaffected. Packet grouping and reviewer assignment are staged, not
   built — see `docs/plans/nom-standards-batch-2.md` "Packets."

The genuinely new artifact is `docs/ingestion-difficulty-log.md`: a
durable, failure-class-tagged record distinct from any single plan's own
`Progress`/`Surprises and discoveries` section, so a recurring pattern is
visible across unrelated batches (NOM standards now; state/municipal
corpora later) instead of buried in per-plan prose. Ordinary parser
defects that get fixed immediately still just get a regression fixture,
per the existing loop — this log is only for difficulties deliberately
left open.

Staged the first concrete batch under this policy:
`docs/plans/nom-standards-batch-2.md`, 27 candidates sourced from
Maximasa's `nom-register.md` (Tables 1, 3, and 4 minus the five already
canonical). Corrected in the same pass: the batch is 27, not the round 25
first estimated conversationally — Table 2 (SSA1/SCFI) is fully done
(4/4), leaving 22 STPS + 3 SEMARNAT + 2 gap-analysis instruments. Also
corrected: standards acquisition has no adapter yet
(`docs/standards-module.md`), so locating each official source remains
real manual work — the "easy ones ingest quickly" framing only ever
applied to parsing/validation, not sourcing.

## 2026-07-27 — Scope 2 staged, not started; Maximasa NOM slice closed for now

Decision: land Scope 1 (below), then explicitly stage Scope 2 (the
decree-diff engine) as recorded future scope under M4 in
`docs/plans/maximasa-legal-integration.md` rather than beginning it in the
same session. Operator instruction: finish the current Maximasa NOM
ingestion/processing slice first; plan Scope 2 separately later.

Closing this slice required two things beyond Scope 1's own code:

1. `docs/project-status.md` and `docs/standards-module.md` gaps a review
   pass caught before Scope 1 could be called done: the new
   `transitories.json` artifact class wasn't reflected in the corpus
   totals table, and two facts needed verification rather than assumption
   — that standards' lack of a Markdown export profile genuinely leaves
   `Generated Markdown files` unaffected, and that NOM-002-STPS's
   `2000-09-08` asserted date is a verified predecessor-publication
   citation in the transitorio's own text, not another signature-block
   leak like the one just fixed in NOM-051.
2. A second mechanical bundle refresh in Maximasa: `transitories.json`
   becoming a required `bundle create` output file changed the five-NOM
   bundle from 4 files/standard (20 total) to 5 (25 total). Regenerated
   `nom-bundle-manifest.json` at Lex-Mex `a3a48296f`, reran
   `build_demo_data.py`, and updated Maximasa's hardcoded bundle-lock test
   expectations (`selected_files_checked` 20→25, new sha256) — all 14
   Maximasa tests pass with every locked file verified against the live
   corpus. No NOM was rechecked against its official source; this is the
   same standing mechanical-only refresh authorized 2026-07-26 ("update
   HEAD and hashes, but do not recheck the NOMs"), triggered by a new
   required file rather than new corpus content. Also added the now-
   machine-visible NOM-051 transitorio-staleness fact (see Scope 1's
   finding below) to `nom-candidate-package.yaml` and `OPEN_QUESTIONS.md`
   O-7, since it was flagged pending and this refresh was the natural
   point to fold it in.

## 2026-07-27 — Standards transitorio inspection (Scope 1)

Operator-scoped, bounded implementation: two sub-scopes were carved out of
the transitorio/decree-diff idea flagged 2026-07-26 (see below) — "Scope 1"
(transitorio inspection, lightweight, this entry) and "Scope 2" (the
decree-diff engine, not started). Explicitly *not* a full structural parse
of transitorio content: some standards' or statutes' transitorios are long
and complex (heavily amended instruments especially), and the goal was
addressability plus date-scanning, not understanding everything a
transitorio says.

`StandardTransitory` (`lex-core`), `parse_standard_transitories`
(`lex-parse/standard.rs`), and `transitories.json` (new committed file per
standard, required by `bundle create`, addressable via
`lex-mex path --kind transitories`) landed together with
`schemas/standard-transitory.schema.json`. The ordinal recognizer
(`transitory_ordinals`, `parse_transitory_start`) is not new code: it
already existed in `diputados.rs` for statute transitories and was made
`pub(crate)` for reuse rather than reimplemented — the "always serialized"
peculiarity the operator named was already solved there.

Backfilled `transitories.json` for all five already-committed NOMs by
recompiling from their already-retained source/text (no new official-source
research); all five reparsed with clause counts and issue counts
byte-identical to their committed state before this landed.

Three parser defects surfaced compiling this against NOM-051's real
retained text (fixtures added for each, see `docs/standards-module.md`
"Standards transitorio inspection" for detail): the índice's repeated
TRANSITORIOS heading matched first instead of the real section (same
class of bug as the Bibliografía heading fix, 2026-07-26); an untrimmed
line with leading indentation was passed to the ordinal recognizer, which
requires the ordinal at position 0; and the signature-block marker didn't
recognize the post-2016 "Ciudad de México, a ..." dateline (only the
pre-CDMX-renaming "México, D.F., a ..." form), letting a decree's closing
signature and its own sign-off date bleed into the last transitorio's
text and asserted dates.

**Finding:** NOM-051's `transitory:segundo` still asserts the original
2020 decree's `2025-10-01` phase-three date. Two 2025 ACUERDOs (see the
2026-07-26 entry) pushed the real date to `2028-01-01`; neither is part of
the retained source, and nothing before this landed could represent or
surface that staleness at all. This doesn't close the gap — reading the
current true date still requires knowing about both ACUERDOs — but it
makes the staleness checkable against the retained text for the first
time, and is the addressable prerequisite Scope 2 needs ("replace
transitorio SEGUNDO's date" requires SEGUNDO to be an addressable object).

## 2026-07-26 — How to read a platiica/DOF NOM record, for every future ingestion

Operator-supplied methodology (verified against NOM-051-SCFI/SSA1-2010's
registry record and DOF/SIDOF primary sources), recorded here because it
governs every future NOM/NMX ingestion, not just this one.

**A platiica registry record's `Historial Documental` mixes several
distinct document kinds under one list; each needs different treatment:**

- The NOM itself (the base PDF, which may be stale — NOM-051's stayed at
  its 2014 state until this session's 2020 re-source).
- `Procedimientos para la evaluación de la conformidad`: a separate,
  complementary instrument (a compliance checklist), not part of the NOM's
  own text. Not a modification.
- `PROYECTO de Modificación`: never normative. Always excluded. Reliably
  identified by starting with the literal word `PROYECTO` and by carrying
  no vigencia/in-force date on the DOF page (shows as blank/`XXX`).
- `MODIFICACIÓN`: an actual normative change to the standard's text.
- `ACUERDO`: see below — do not assume scope from the type name.

**`ACUERDO` is not a fixed category of change; it is the generic official
name for any binding decision a competent authority publishes in DOF
within its jurisdiction.** It can be substantive and close to NOM-level
content (e.g. an ACUERDO determining permitted food additives — normative
because that authority chose ACUERDO as the default publication vehicle
rather than a full NOM modification), or it can touch only a modification's
transitorio dates (as NOM-051's two 2025 ACUERDOs do), or it can itself
read "ACUERDO por el que se modifican, adicionan y derogan ...". The name
never tells you the scope — read what it actually says every time.

**How to read a `MODIFICACIÓN`'s (or an ACUERDO's) own diff against a base
text:** these decrees state their own substitution instructions verbatim,
in the decree's own text, not as an editorial summary. A run of untouched
numerals or phase labels followed by `...` means that span is unchanged;
whatever numeral or label follows next is given in full, and that full
text is the verbatim replacement (or addition, or derogation) for that
specific numeral. Example, confirmed directly against DOF's page for the
2025-07-31 ACUERDO (`https://diariooficial.gob.mx/nota_detalle.php?codigo=5764197&fecha=31/07/2025`):
"PRIMERA FASE. ... SEGUNDA FASE. Del 1 de octubre de 2023 al 31 de
diciembre de 2027 ..." — `PRIMERA FASE` is untouched; only `SEGUNDA FASE`
(and, further down, `TERCERA FASE`) received new text. Applying this is
mechanical substitution against the decree's own explicit instructions,
not Lex-Mex performing its own legal consolidation — the distinction that
kept NOM-247 out of scope for a self-consolidation still holds (no
official consolidated *republication* exists for its decrees), but the
per-numeral substitution mechanism itself is a legitimate, deterministic
operation Lex-Mex can perform once a decree gives it explicitly.

**`Bibliografía` splits into two kinds of entries, only one of which is a
traceability chain:** cited Leyes, Reglamentos, and Acuerdos form a real
parent-authority hierarchy (a NOM is issued under a Reglamento under a Ley,
occasionally under a further Acuerdo) worth backlinking once the target is
in the Lex-Mex corpus. ISO guides and academic citations are not normative
and should be excluded from that hierarchy.

**Finding that reopens NOM-247:** the earlier conclusion (2026-07-26,
`maximasa-legal-integration.md`) that incorporating NOM-247's two 2011/2012
modifications would require Lex-Mex to perform its own legal consolidation
no longer holds as stated. Both decrees use the same
unchanged-span-then-explicit-replacement convention described above. If
that pattern is mechanically parseable the same way for NOM-247's finer,
numeral-level scope, incorporating them is applying the decrees' own
stated instructions, not synthesizing text Lex-Mex invented. Not yet
implemented; see the standards-module note this same date for the
concrete blocker found while attempting a NOM-051 pass under this
methodology (transitorios have no structured representation at all in the
current standards schema).
## 2026-07-16 — Batch completion closes its bounded graph

`batch run` now has a deterministic closure phase after every successfully
processed selected instrument has entered the corpus. It reverse-relinks each
such instrument against the complete committed sibling set, validates it, and
regenerates Markdown (and an explicitly requested Obsidian target). A batch
cannot report success if this bounded closure fails.

`expected_edges` is now a recall oracle rather than an unused planning note.
Concrete entries use `SOURCE -> TARGET` or `SOURCE articulo N -> TARGET`,
where each name is a committed corpus slug or short name. The batch report
records every check as `satisfied`, `missing`, `deferred`, or `invalid`.
Missing and malformed concrete expectations fail closure; a target absent from
the committed corpus, or a source not processed in this run (including
`--only`), remains explicitly deferred rather than producing an invented edge
or a false pass. This remains a bounded batch check, not a substitute for the
deferred corpus-wide relink and review program.

## 2026-07-16 — Unanalyzed temporal status is unknown

A consolidated current source establishes the wording the publisher presents;
it does not, by itself, establish that every printed provision is legally
effective. Consolidations may retain provisions affected by judicial
invalidity, delayed commencement, or another temporal condition. The prior
parser default introduced at `9429d2bb` therefore made an unsupported legal
inference by assigning `effective` before temporal analysis.

Freshly parsed ordinary provisions now start `unknown` while
`review_status` remains `not_analyzed`. Only an explicit repeal note at the
start of the source text (`Se deroga`, `Derogado`, and the existing narrow
variants) starts `repealed`; that is a deterministic transcription of the
publisher's express notation, not a model or reviewer conclusion. Persisted
machine-accepted, review-required, and lawyer-verified temporal
determinations continue to override the parser's initial state and retain
their evidence hashes, bases, dates, confidence, and review history.

Validation enforces this boundary: a `not_analyzed` provision must carry the
deterministic initial status implied by its exact text, so a future parser,
import, or hand edit cannot silently restore `effective` as an unanalyzed
default. The one-time canonical migration changed 30,124 unanalyzed ordinary
provisions across 144 corpora from `effective` to `unknown`, with matching
generated Markdown. It left 3,592 explicit repeal notes, one pending reviewed
repeal, and all 21 accepted or lawyer-verified effective determinations
unchanged. This migration records no JRH legal-review decision and does not
alter official source text, reference edges, terms, provenance, or temporal
evidence.

## 2026-07-14 — Diputados split headings and reform-appendix identity

Ingesting the Reglamento del Senado de la República exposed two independent
layout boundaries in Cámara de Diputados consolidated PDFs that validation
counts alone did not catch:

- **A bare article heading followed by a numbered paragraph stays one
  article.** The PDF prints `Artículo 1` on one line and `1. ...` on the next.
  Collapsing both lines before parsing produced the false compound identifier
  `Artículo 1 1`. When a line contains only a valid article heading and the
  next line starts a dot-delimited paragraph numeral, the parser now supplies
  the omitted heading/body separator while preserving that numeral in the
  canonical text. Genuine compound headings such as `Artículo 15 Bis 1`
  remain unchanged.
- **A true decree title is a hard reform-appendix boundary.** Page furniture
  can otherwise join the preceding signature or errata page to the next
  decree. An uppercase `DECRETO` title (plus the documented older title-case
  forms) opens that boundary; a wrapped legal sentence beginning `Decreto de
  ...` does not. Likewise, a DOF publication phrase changes the containing
  decree date only before its transitory section begins. The same phrase
  inside a transitory remains canonical evidence instead of silently
  re-dating that and all following transitories. Singular/plural `ARTÍCULO(S)
  TRANSITORIO(S)` headings, colon-ended ordinals, signature blocks, and `Fe de
  erratas` pages are normalized explicitly. Operative `ARTÍCULO ÚNICO` text
  remains outside temporal evidence; only the decree's transitories enter
  `reform-temporal-evidence.json`.
- **Numbered reform transitories are evidence headings.** Inside an explicit
  transitory section, older `ARTICULO 1o.-` / `ARTICULO 2o.-` forms are parsed
  as transitories, not discarded as operative decree articles. This preserves
  the RGIC decree of October 21, 1966 alongside its ordinal-form peers.
- **Same-day decrees receive distinct temporal-evidence identities.** The
  first decree published on a date retains the established
  `:amendment:YYYY-MM-DD:transitory:<ordinal>` ID. A later decree on that same
  date uses
  `:amendment:YYYY-MM-DD:decree-N:transitory:<ordinal>`, where `N` is its
  one-based source order for that date. This keeps existing non-colliding IDs
  stable while preventing several same-day `ÚNICO` provisions from collapsing
  onto one identity. Publication-date extraction still reads the first ten
  characters after `:amendment:`.

The Senate regulation now yields 313 articles, 4 original transitories, 47
resolved canonical references, and 39 uniquely identified reform
transitories. Temporal analysis remains deferred; this structural ingest
creates no machine conclusion and no legal-review resolution.

The RGIC exercises the combined rules: 214 articles, 2 original transitories,
30 resolved references, and 23 reform transitories attributed to their actual
DOF dates. Its 359 canonical paragraphs match the official extracted text
after removing only configured running-page furniture. Temporal analysis is
likewise deferred.

## 2026-07-12 — Old CNBV compilation format (2003–2015 DCGs)

Ingesting the six older CNBV disposiciones (cue-2003, cucb-2004, cub-2005,
socap-sofipo-2006, oaac-2009, fi-2014) generalized the `itf-dcg` parser,
which had been tuned to the 2018 vintage. The format diverges in several
ways at once; each is handled additively so the committed itf-dcg-2018,
scap-dcg-2012, and ifpe corpora stay byte-identical (verified by the itf
fixture tests and a scap re-parse):

- **Preamble/índice is skipped until the first article.** These documents
  open with a table of contents that echoes a `TRANSITORIOS` heading and the
  annex list — each with its own `(N)` markers — before Artículo 1. Region
  transitions and marker accumulation are gated on `body_started`, set at
  the first article heading; otherwise the índice `TRANSITORIOS` echo flips
  the scanner into the transitorios region and every body marker strands
  (the SOCAP/OAAC failure, ~1,600–3,200 stranded marks). Índice markers are
  redundant with the same marker on the provision they annotate — still
  recorded in the REFERENCIAS legend — so preamble markers are dropped.
- **Ordinal article abbreviations** (`Artículo 1o.-` … `9o.-`, also `º`/`°`)
  are accepted and normalized to the plain number (`1o` → `1`, as `8 ≡ 8o`).
- **Feminine and singular transitorios** — the section heading in
  `TRANSITORIO`/`TRANSITORIA`/`TRANSITORIAS` as well as `TRANSITORIOS`, and
  feminine ordinals (`ÚNICA`, `PRIMERA`…) — because a "Disposición
  Transitoria" is feminine.
- **Attribution dates are accumulated across line wraps** that can split the
  date itself (`… el 12 de enero de` / `2015)`); the date before the closing
  paren resolves the section (its markers otherwise strand).
- **A marker at the foot of a section attaches to that section's last
  provision** before the section is flushed at a `TRANSITORIOS` heading;
  a true remainder is heading-level marginalia and is dropped. Markers in a
  trailing CONSIDERANDO or the REFERENCIAS legend are likewise marginalia,
  dropped rather than errored. A structural mis-parse now surfaces through
  the article-count/gap and legend-presence checks rather than a stranded
  marker.
- **Parenthesized legend numbers** (`(N)  text`) are accepted alongside the
  bare `N)  text` form.
- **`allow_article_gaps: true`** on each adapter: these compilations renumber
  away derogated articles (e.g. cue has 15 Bis with no bare 15), a legitimate
  gap, so the sequential-order check yields warnings, not errors.

All six validate with zero errors, counts unfrozen (matching the
scap/servinv precedent). Result: cue 114 arts, fi 232, cucb 337, oaac 295,
socap 548, cub 705.

## 2026-07-12 — CNBV amendment resolution, in-force status, corpus currency

Ratified with JRH after the CNBV DCG batch surfaced how the `(N)` markers
actually work. Full spec: `docs/cnbv-consolidated-disposiciones.md`. This
extends the earlier "amendment markers on reform transitorios" entry below;
it does not contradict it.

- **Markers attach to *any* structural node**, not only articles/transitorios
  — the denominación, a TÍTULO, a párrafo, a fracción. OAAC's compiled title
  opens with `(18)`. The `itf-dcg` parser's `discard` errors on
  cue/cub/cucb/fi were the parser being **provision-centric**, not the
  documents being corrupt: a marker in a CONSIDERANDO or attribution block is
  valid. Fix is attach-to-nearest-node, keeping the true error only for a body
  marker with no legend entry.
- **REFERENCIAS is the validation oracle.** Every body marker `N` must resolve
  to a `REFERENCIAS[N]` legend entry (`{ acción, fecha_DOF }`); an unresolved
  `N` is a hard error (anti-silent-loss), an orphaned legend entry a warning.
  The body-marker set ⊆ legend key set is the invariant that verifies the
  socap/oaac region-detection fix.
- **Keep the marker → REFERENCIAS link; defer the marker → transitorio link.**
  The authoritative, deterministic layer (integer key into a numbered legend)
  is built and kept, so a reader sees what changed, when, and by which RM.
  This refines the prior "keep the mention, no link" to "keep the *reference*
  link." The modifying resolutions (RMs) are **not** corpus instruments — only
  the final compiled text is; ingesting RM texts would balloon the corpus
  (the CUB alone has hundreds of RMs). Wiring the transitorio link is a
  future option, not a current requirement.
- **DOF date is not a unique RM key.** Two RMs can share a DOF date (CUIFE
  *11a* and *12a* both 08/01/2015) — an outlier, but the model accounts for
  it: if the transitorio layer is ever wired in, a colliding date yields
  attach-all-candidates + warning, never a machine-picked ordinal. Future aid:
  a snapshot of the CNBV Normatividad "Resoluciones Modificatorias" listing
  (carries ordinals + dates) disambiguates, and doubles as an update signal.
- **In-force status: live vs. staged (design proposal, generalizes to all
  instruments).** The useful signal is whether a provision is operative today.
  Per-RM TRANSITORIOS blocks (already captured as `TemporalEvidence`) state
  entry-into-force: default next-day, but OAAC stages provisions into 2027,
  and deadlines get extended. Proposed statuses `live` / `staged` /
  `staged_extended` / `unknown`, likely a **computed overlay** on `Effective`
  (status = what the law says; liveness = whether *today* is past the
  effective date) so the corpus stays date-stable. Touches `TemporalStatus` /
  effect categories → schema-boundary path; shape awaits JRH sign-off.
- **Corpus currency (new requirement).** The CNBV refreshes compiled PDFs on
  new RMs, with the page lagging days (ITF-DCG-2018 refreshed a Thursday,
  reflected later). A scheduled mechanism must re-acquire source hashes,
  snapshot the RM listing, cross-check the latest REFERENCIAS date, and emit a
  currency report to review — never auto-committing changed law. Subsumes the
  ITF-DCG-2018 reform-re-ingest TODO as its first flagged case.
- **Definitional remittance deferred to the cross-instrument pass.** A bare
  glossary remittance ("Valores: a los considerados como tales por la Ley del
  Mercado de Valores") resolves transitively to the target instrument's
  glossary entry (LMV art. 2 fr. XXIV) by **lemma-join** — deterministic only
  once LMV is in the corpus and the headword maps 1:1. Runs once after full
  federal ingestion (near complete), then incrementally. Not built in the DCG
  parser.

## 2026-07-12 — Amendment markers on CNBV reform transitorios

CNBV consolidated disposiciones (DCGs) carry numbered `(N)` superscript
amendment markers that reference a REFERENCIAS legend — version-control
provenance recording *when* a provision was amended and *by which*
modifying resolution. The modifying resolutions are **not corpus
instruments** (only the final compiled text is), so the marker is kept as
a mention with no outbound link (JRH, reviewer of record for CNBV).
Markers on articles and original transitorios were already captured as
`Provision.amendment_marks`; but these texts also **re-amend their own
reform transitorios**, so a marker can land inside a per-resolution
TRANSITORIOS section. The `itf-dcg` parser previously errored there
(a reform transitory becomes `TemporalEvidence`, which had no marks
field) rather than silently drop provenance.

`TemporalEvidence` now has an optional `amendment_marks: Vec<u32>`
(`skip_serializing_if` empty, so the committed IFPE/ITF reform evidence is
unchanged — their reform transitorios carry no marks). A marker preceding
a reform-transitory ordinal, or on its continuation lines, attaches to
that transitory exactly as it would to an article. Only a marker with no
open transitory to receive it (inside the parenthesized attribution
block) is still surfaced as an error. First exercised ingesting
`scap-dcg-2012` (parser `itf-dcg`): 382 articles, 204 provisions carrying
marks, 6 reform transitorios carrying marks (e.g. SEGUNDO/2018-01-23 →
[39]); text stays clean of the raw `(N)` glyphs.

## 2026-07-12 — `Ñ` is a distinct letter in canonical article identifiers

LFT article 353 runs a letter-suffix series (`353-A` … `353-U`) that
includes both `353-N` and `353-Ñ` — two distinct articles. The label
grammar and the retired Python tooling both folded `Ñ`→`N` (accent
stripping / NFD + drop-combining-marks), collapsing them onto one
canonical id `…:article:353-n`; the vault only ever held a single folded
`articulo-353.md`, a defect this normalization corrects.

`Ñ` is a distinct letter of the Spanish alphabet, not an accented `n`.
The canonical slug therefore **preserves `ñ`** (lowercased UTF-8): `353-Ñ`
→ `…:article:353-ñ`, file `articulo-353-ñ.md`, distinct from `353-n`.
Only article-label slugs (`labels.rs`) preserve it; defined-term slugs
(`terms.rs::slug`) still fold `ñ`→`n` and remain ASCII, because the
term-id schema constrains ids to `[a-z0-9-]+`. No committed provision or
reference schema constrains the id charset beyond the `urn:lex-mx:`
prefix, so non-ASCII article ids validate. For ordering, `Ñ` sorts
between `N` and `O` (Spanish collation), matching how the law sequences
353-N, 353-Ñ, 353-O. Only LFT carries an `ñ` article label, so the three
committed corpora and every earlier bulk instrument reparse
byte-identically.

## 2026-07-12 — Reference-graph rules for bulk código ingestion

Ingesting the foundational codes (CCom, CPF, CNPP, CFPC, LAmp, LBM,
LGTOC, LTOSF) surfaced reference-resolution and structural cases the
single-statute slice never hit. The rules settled here:

- **A citation classified as internal that resolves to no existing
  provision is dropped, not committed.** A dangling internal edge is a
  broken link, almost always a still-external citation this pass could
  not name — for example CNPP article 167's offense catalog, whose
  "Código Penal Federal" context is declared once at the top and
  resolves only through the named-offense authority table (wiring
  deferred to the penal batch). Dropping keeps the graph free of broken
  links; a genuinely missing article surfaces through the frozen count
  baseline instead. Cross-instrument edges (target is another
  instrument) are still emitted when unresolved, so a configured
  external target that does not exist still fails validation. The three
  committed instruments have no unresolved internal edges, so they are
  byte-identical.
- **Reference citations recognize compound identifiers (`95 Bis 3`),
  hyphenated qualifiers (`156-Bis`), and the adjectival Constitution
  reference (`el artículo 134 constitucional`).**
- **A backward "preceding-law" context scan was tried and rejected:** it
  fixed the "De la Ley X, los artículos N; N Bis; …" list pattern but
  mis-attached a prior citation's law name to a following citation
  (`artículo 20 de la Ley y … el artículo 11`), perturbing the audited
  DCG graphs. The drop rule above reaches the same corpus outcome (no
  edge) without that risk.
- **Reform-decree transitorios are kept out of the instrument by two
  guards:** a second transitory-section header, and a repeated ordinal,
  each end the statute's transitory section, since a statute has one
  section with unique ordinals (LAmp interleaves several decrees'
  `PRIMERO…` sets before the reform-appendix marker).

## 2026-07-11 — The repository is the only ingestion and processing gate

Between 2026-07-08 and 2026-07-10, a Python tool suite living inside the
Obsidian vault (`Herramientas/`) bulk-imported 135 additional instruments
directly from Cámara de Diputados consolidated PDFs, with its own parsing,
linking, and audit rules, no version control, and no schema gate. That
created two divergent rule sets and made the vault the only holder of
canonical facts for those instruments — exactly what this repository's
architecture forbids.

Decision (protocol designer, 2026-07-11): the repository's Rust pipeline is
the sole ingestion and processing gate. The 135 vault-only instruments will
be re-ingested through it (structural ingestion first; temporal analysis
deferred and run later per batch by legal priority). The vault returns to a
visualization/interaction layer only; the Python tooling is frozen
immediately and retires at parity.

What folds into the repository from the vault tooling:

- **`batches/*.json`** — 26 batch-ingestion manifests (25 converted from
  `Herramientas/import_batches/` with the F2/F3/F4 variant schema
  normalized, plus `legacy_core_pre_manifest` reconstructing the ten
  instruments imported before manifests existed). `blocked` entries and
  their reasons are preserved verbatim; blocked sources stay blocked until
  a reviewer clears them. Schema: `schemas/batch-manifest.schema.json`;
  Rust boundary type: `BatchManifest` in `lex-source`.
- **`adapters/diputados/_instrument-aliases.json`** — the hand-curated
  citation-alias table (official titles, accent-stripped variants,
  colloquial names such as "Circular Única de Bancos").
- **`adapters/shared/_named-offenses.json`** — the hand-transcribed CNPP
  art. 167 → CPF named-offense authority table (21 offenses), wiring
  deferred to the penal batch.

Known vault-side defects (Obsidian-invisible mid-block term anchors,
letter-suffixed articles folded into parent files, embedded page running
headers) are not repaired in place; re-ingestion supersedes them.

Count expectations for bulk instruments are parser-proposed frozen
baselines: the first successful parse proposes counts, they are written
into the adapter marked machine-proposed (distinct from the hand-audited
counts of the three original instruments), and subsequent runs enforce
them as drift detection.

## 2026-07-06 — Second-pass code review fixes on the ITF DCG ingestion

An external review of the amendment-marker and relative-reference work found
eight issues, six of them real correctness bugs. All eight are fixed here.

**Main document extraction lost its page-break markers.** `run_extract`
gated `keep_page_breaks` on `parser == "ifpe-dcg"` only; the newer `itf-dcg`
parser was never added, so its compiled main document was extracted with
`pdftotext -nopgbrk` even though `itf.rs` explicitly scans for `\u{c}` to
decide whether a paragraph legitimately continues across a page boundary.
The page-break-aware merge logic was silently dead code for the whole
~105-article main document (annexes were unaffected — they hardcode the
flag separately). Fixing this and reparsing corrected 24 provisions where a
page break had incorrectly glued two paragraphs together — most visibly,
fraction III of article 54 had been silently merged into fraction II's
text, invisible to fraction-anchor linking. Word-level fidelity re-verified
across all 2,132 canonical paragraphs; temporal evidence text is untouched
(the reform-transitory scanner never used page-break state), so all 17
persisted determinations, including the pending SÉPTIMO review, re-applied
unchanged.

**A shared "pending marker" mechanism replaces two independent, drifted
copies.** `dcg.rs`'s `parse_annex_document` and `itf.rs`'s main-document
scanner had each grown their own hand-rolled version of "hold a marker,
swallow the blank line right after it, drain onto whichever provision
comes next" — and the two copies had already diverged into different bugs:

- In `parse_annex_document`, a page-number footer between a marker and the
  following blank line didn't reset the "swallow the next blank" flag, so
  the footer let that swallow-intent leak across itself onto a blank line
  the marker was never actually adjacent to, incorrectly merging two
  paragraphs. Fixed by making every non-blank, non-marker line — including
  a footer — sever that adjacency, matching how an ordinary content line
  already did.
- In `itf.rs`, a marker appearing inside a per-resolution TRANSITORIOS
  section was queued but never drained anywhere, since a per-resolution
  transitory becomes `TemporalEvidence`, which has no `amendment_marks`
  field to receive it — and the CONSIDERANDO/REFERENCIAS transitions
  didn't clear the queue either, unlike the TRANSITORIOS transition and
  the four structural-heading transitions, which did. The marker simply
  vanished with no trace.

Both are now the same shared `PendingMarks` type (in `dcg.rs`, used by
both parsers): `push`/`drain_onto` for the normal case, and `discard`,
which **errors** instead of silently dropping a marker at a boundary with
no receiver — a per-resolution transitory, a considerando, or the legend.
Discovering a real document exercises one of those cases needs a human
look, not a silent loss of provenance.

That strict rule has one evidenced, deliberate exception:
`discard_from_heading` clears silently at a Título/Capítulo/Sección/
Apartado boundary (and at a TRANSITORIOS/REFERENCIAS transition reached
directly from Body), because the real document repeals an entire Apartado
with no article of its own — a heading followed by a lone `(Derogado)`
note, itself marked. `HeadingContext` has no field to receive a mark, but
the fact is always redundant with the same marker already recorded
directly on the individual provisions the heading covers, so nothing is
lost by discarding it there.

**`orphan_paren_re` narrowed to a self-verifying retry.** The regex
repairing article 21's glyph-splitting artifact (`) Artículo 21.- …`) was
applied to every line of the whole document up front, with nothing but the
literal text "Artículo" constraining it. It now only runs as a fallback at
the exact point of trying to match an article heading, and is accepted
only when stripping the leading `) ` actually turns the line into a real
`article_re` match — so it can never alter a line that merely happens to
start the same way without being a mis-rendered heading.

**Reform-evidence ID/label construction is now one shared function**
(`reform_evidence_item` in `lib.rs`), called by both LRITF's
`ReformEvidenceBuilder` and the ITF DCG's `flush_reform` — closing a
literal duplication of the `{instrument_id}:amendment:{date}:transitory:
{ordinal}` convention. Each caller still assembles its own `text` before
calling it (LRITF's decree appendix is block-scanned and paragraph-joined;
the ITF DCG's resolution sections are line-scanned and space-joined) — the
two are not forced into a shared join strategy, since doing so would have
altered persisted, already-hashed evidence text for one or the other.

**Reform-evidence file write-gate restored to its original invariant, and
correctly extended.** A prior fix changed the write condition from
`parser == "lritf"` to `!reform_evidence.is_empty()`, so a future reparse
producing zero reform evidence would leave a stale non-empty file on disk
rather than overwriting it to `[]`. Restored to writing unconditionally —
even when empty — gated on `matches!(parser, "lritf" | "itf-dcg")`,
extending the original LRITF invariant to the new parser instead of
narrowing it for both.

**The shared annex marker-stripping logic added for the ITF DCG was
verified against the real IFPE DCG-2021 corpus, not just asserted safe.**
The margin-marker regex in `parse_annex_document` is shared unconditionally
across both DCG parsers, but only 2 of IFPE's 8 real annexes have fixture
coverage. Refetched and re-extracted all 8 real annex PDFs (byte-identical
to what's committed), confirmed zero standalone marker-shaped lines exist
in any of them, and reparsed the full instrument: zero annexes gained an
`amendment_marks` entry, and `provisions.json`/`references.json` are
byte-identical to what was already committed.

## 2026-07-05 — Compiled-document amendment markers as structured provenance

The compiled CNBV document for the general Fintech DCG (DOF 10/09/2018,
six resoluciones modificatorias through 09/09/2025) prints a numbered
margin marker (`(7)`) beside every amended block, and closes with a
REFERENCIAS legend mapping each number to its amending resolution and
action (Reformado / Adicionado / Derogado / Sustituido). Following the
standing rule that compiled documents are the operational source and
resoluciones are provenance references — never individually extracted —
the markers are treated as structured marginalia, not prose:

- Markers are removed from canonical provision text (they are typography,
  like page-number footers) and recorded per provision as
  `amendment_marks`, deduplicated and sorted.
- The legend is parsed into corpus-level `amendment_references`
  (`amendment-references.json`), keeping the verbatim legend text.
- Marker placement is spatial: the layout extraction emits each marker at
  the vertical position of the text it annotates, which can be just
  before a provision's heading line or between its body lines. Markers
  are therefore held pending and attached to whichever provision the next
  content line belongs to; structural headings (títulos, capítulos,
  secciones, apartados) clear them, since a chapter-title mark is not
  provision provenance.
- A blank line immediately after a marker line is part of the marker's
  own line box: paragraphs flow across markers unbroken.
- Inline parenthesized numbers in prose (`un (1) reporte`) are untouched —
  only whole-line markers count.
- One glyph-splitting artifact exists in the source PDF (article 21's
  marker renders its closing parenthesis at the start of the heading
  line); the orphan parenthesis is removed deterministically and the case
  is fixture-covered.

Word-level fidelity holds: all 2,104 canonical paragraphs of the ITF DCG
are exact substrings of the extracted sources after removing exactly the
markers, page numbers, and the one orphan parenthesis.

Each of the six per-resolution TRANSITORIOS sections after the original
one is attributed to its resolution by the parenthesized block following
the heading, and its articles become reform temporal evidence
(`…:amendment:<dof-date>:transitory:<ordinal>`), mirroring the LRITF
reform-decree appendix. Only the original 2018 transitories are canonical
provisions. `latest_reform_date` derives from the maximum attributed
resolution date. The instrument deliberately has no formal DOF source
acquisition: the compiled document consolidates seven DOF publications,
and per-resolution provenance lives in the legend and the adapter's
`relevant_reform_transitories`; the original DOF nota can be attached
later if a decision comes to depend on it.

## 2026-07-05 — Relative article references

`artículo anterior` / `artículo siguiente` are express citations whose
target is inferred from position rather than named, so they carry the new
distinguishable `reference_form: relative` instead of masquerading as
direct numeric citations. Resolution walks the source provision's
same-type sequence in document order: a transitory's `anterior` is the
previous transitory, never the last numbered article, and the instrument
title (which has no position) can never carry one. A phrase with no
neighbor in its direction — `artículo anterior` inside the first article —
produces no edge.

Deliberate exclusions, each deterministic:

- The plural `los artículos anteriores` names an open-ended set with no
  single target; it stays unlinked (three LRITF occurrences).
- Bare self-references (`este artículo`, `el presente artículo`, 174
  occurrences) are not extracted: the reader is already inside the target,
  and the useful fraction-scoped form (`fracción N del presente artículo`)
  is already handled by the same-article path.
- `del citado artículo anterior` still resolves, but the intervening word
  keeps the pre-number qualifier from attaching — the qualifier machinery
  requires exact adjacency (`del`/`de los` ending at the header) and does
  not guess across words.

The pre-number qualifier pattern also gained the noun-first paragraph form
(`párrafos segundo y tercero del artículo N`) and the `penúltimo` ordinal,
both fixture-tested; `penúltimo párrafo` appears on two LRITF article 138
relative edges today, the noun-first form has no numeric-target occurrence
yet in either instrument.

## 2026-07-03 — Fraction-level references and previews

A fraction never exists in isolation — `fracción XI` only means something
relative to its article — so fraction precision is layered onto article
edges rather than modeled as standalone targets. Three additions:

1. **Pre-number qualifiers.** Phrases written before the article number
   (`las fracciones II, III, IV y V del artículo 22`, `el séptimo párrafo
   del artículo 29`) are captured when they end exactly at the `artículo`
   header, connected by `del`/`de los`, and attach to every article in the
   cited list. Previously only post-number qualifiers were captured.
2. **Anchored qualifier spans.** `ReferenceQualifier` gains optional
   Unicode character offsets, validated against the unchanged canonical
   text like edge spans. Offsets are backward compatible: existing
   qualifiers without offsets remain valid.
3. **Same-article fraction citations.** `fracción N del presente artículo`
   / `de este artículo` produces one edge per numeral, targeting the
   containing provision, spanning exactly the numeral, and only when the
   provision actually has that fraction as a paragraph.

Presentation uses a dual affordance because a native Obsidian hover can
preview either a whole note or a single block, not a composed
article-header-plus-fraction view: the article number keeps its whole-note
link, and each fraction numeral in an anchored qualifier links to the
target's `^f-<n>` block — `fracción [[articulo-36#^f-xi|XI]] del artículo
[[articulo-36|36]]`. Same-article numerals link to the provision's own
fraction block. A numeral links only if the target note actually has the
fraction anchor; otherwise it stays plain text. Anchor links are
Obsidian-only (standard Markdown has no block anchors). Generating a
per-fraction note to get the composed preview remains a possible later
presentation add-on.

Enabling same-article extraction grew the audited graphs deliberately:
LRITF 95 → 115 edges (the original 95 unchanged plus 20 self-targeting
fraction edges), DCG 98 → 111.

## 2026-07-03 — Defined-term glossary layer

Mexican financial instruments commonly define their working vocabulary in a
glossary provision within the opening articles — LRITF Article 4
(fraction-style, `I. Término, a …`), DCG-IFPE-2021 Article 1 (colon-style,
`Término: a …`) — though not always, so the glossary is adapter
configuration, not a parser assumption. Terms are extracted as canonical
`DefinedTerm` records (`terms.json`) with the exact span of each definition
entry, including continuation paragraphs such as incisos. The DCG's Article
1 expressly defines its terms "además de los términos utilizados en la
Ley…": that additive relationship is configured (`glossary.additive_to`),
so a DCG usage resolves against the DCG glossary first and falls back to
LRITF Article 4 — `Cliente`, `Operaciones`, and `Infraestructura
Tecnológica` in the DCG resolve to the statute's definitions.

Usages (`term-usages.json`) are deterministic exact matches at word
boundaries, longest match first, case-sensitive because capitalization is
what distinguishes the defined `Control` from the ordinary word `control`.
Glossaries state that terms apply "en singular o plural", so one
singular/plural variant is generated per word with deterministic rules
(`-ón` ↔ `-ones`, vowel ↔ `+s`, consonant ↔ `+es`): `Operación` matches the
defined `Operaciones`, `Comisión Supervisora` matches `Comisiones
Supervisoras`. At a sentence, list-item, or table-cell start the capital is
positional and carries no signal, so a term whose only capital is its
initial letter does not match there — `I. Controles de acceso…` is not the
defined `Control` — while acronyms and multi-word terms match anywhere.
A term never matches inside its own definition entry. Validation covers
term identity, definition spans, exact usage spans, cross-instrument
resolvability, and non-overlapping usages; both files are schema-bound
(`defined-term.schema.json`, `term-usage.schema.json`).

Presentation: generated Obsidian notes carry block anchors on every
fraction paragraph (`^f-xi`) and on each colon-style definition entry
(`^t-<slug>`). A term links to its definition's block —
`[[Corpus/LRITF/articulo-4#^f-ii|Clientes]]` — so hovering shows only the
definition, not the whole glossary article. To keep notes readable, only
the first usage of each term per provision is rendered as a link, and term
links never overlap reference links; all usages remain canonical facts.
The audited LRITF canonical core (provisions, references, temporal result,
review queue) is unchanged by this layer; the fraction anchors also lay the
groundwork for fraction-level reference previews.

## 2026-06-27 — PDF extraction boundary

The LRITF operational source is a text-based PDF. `lex-parse` invokes
`pdftotext -layout -nopgbrk` for extraction, records the extractor version, and
then performs all canonical normalization in Rust. This keeps the source
adapter reproducible without adding an immature PDF parser to the canonical
core.

## 2026-06-27 — Article-level first slice

The first parser emits ordinary articles and the statute's own transitory
provisions. It deliberately excludes appended full reform-decree transitories
from the statute provision list. Those require amendment-event modeling and
must not be conflated with the statute's own transitories.

## 2026-06-27 — No hidden LLM call

Temporal analysis produces a versioned, schema-bound request artifact. Model
execution and response import are explicit boundaries so deterministic runs do
not depend on credentials or silently change canonical data.

## 2026-06-28 — External Obsidian vault boundary

The Obsidian vault is not nested inside canonical corpus storage. The CLI
publishes to an explicit vault root supplied with `--obsidian-vault` or
`LEX_MEX_OBSIDIAN_VAULT`, and the exporter owns only
`Corpus/<instrument-short-name>/` below that root. Human-authored `Notas/`,
`Revisiones/`, attachments, and `.obsidian/` settings remain outside the
exporter's write boundary.

## 2026-06-29 — Explicit temporal execution and deterministic routing

Temporal execution remains opt-in. The default command emits only a request;
`--provider codex` invokes the locally authenticated Codex CLI with the
versioned prompt and a strict output schema. The importer is provider-neutral
and rejects missing, duplicate, or unknown provision identifiers, invalid date
ranges and confidence values, and supporting quotations that are not exact
source substrings. Request and response hashes preserve the execution boundary.

The source adapter explicitly selects reform-decree transitories relevant to
LRITF. This prevents transitories for other statutes bundled into an omnibus
decree from entering LRITF temporal analysis.

## 2026-06-29 — Temporal review policy

Machine conclusions are accepted only at confidence 0.92 or above. A
determination enters legal review when the provision status, effect type,
application rule, or a material boundary remains unknown. Express survival,
adaptation, and conditional rules do not enter legal review merely because they
are transitory. The exporter publishes the queue to Obsidian, but only a human
review workflow may resolve it.

## 2026-06-29 — Audited human review resolution

Review resolution is an explicit canonical state transition. It requires a
reviewer identity; lawyer overrides also require a reason and an explicit
temporal status. The verified determination is labeled `lawyer_verified`, while
the original model proposal, reviewer, resolution, note, and timestamp remain
in the review record. Resolved records stay in the JSON queue for audit but are
excluded from the default CLI listing and pending Obsidian dashboard.
Subsequent model imports reconcile against this history and preserve resolved
human decisions instead of reopening or replacing them.

## 2026-06-29 — Formal-source review context

The LRITF adapter maps each analyzed publication date to an official DOF
publication URL. Review imports attach that formal source alongside the Cámara
de Diputados operational source. Where the one-law slice cannot yet provide an
affected-provision diff, the queue states that limitation explicitly instead
of leaving the reviewer to infer whether the field was omitted accidentally.

## 2026-06-29 — Transitory provision status versus legal effect

Following legal-review guidance from JRH, the temporal model treats a
transitory's own status separately from the effects it creates. An effective
transitory may preserve prior rules for an existing cohort, grant an adaptation
period, mandate regulation, allocate authority, or stage application without
itself being conditional or temporary. Each material effect records its scope,
application rule, trigger, end condition, responsible authorities, and
verification status.

Completion of every proceeding in a protected cohort is modeled as
`cohort_exhaustion` with `open_ended_by_design`; the unknowable global end date
does not itself require legal review. A clear rule dependent on a later
publication or authority action uses `external_verification_required` rather
than being mislabeled as legal ambiguity. Until changed, JRH is the designated
legal reviewer for actual lawyer-verified resolutions.

External facts confirmed during review use `externally_verified` and must carry
an official source URL, event date, and note. JRH verified that SÉPTIMA's
twelve-month clock began with LRITF's entry into force on 10 March 2018 and
that the referenced joint provisions were published on 28 January 2021. The
separate Article 71 coordination agreement remains factually unverified.

## 2026-07-03 — DCG-IFPE-2021 dual official sources

The January 28, 2021 disposiciones for instituciones de fondos de pago
electrónico (`ifpe-dcg-2021`) are jointly issued by the Comisión Nacional
Bancaria y de Valores and Banco de México; the instrument records both
issuing authorities explicitly, independent of which site hosts the file.
The operational CNBV PDF contains the índice, considerandos, seven chapters,
59 articles, and four transitories, but only lists the eight annexes by
title; it does not contain their bodies.

An initial implementation treated the formal DOF publication (código
5610487) as the only available source for annex bodies. JRH pointed out that
the CNBV Normatividad page's "Ver más" panel — visible per row, alongside
`Descargar` and any `Resoluciones Modificatorias` — links each annex as its
own PDF hosted directly on `www.cnbv.gob.mx`. That panel is populated by
`GET /_vti_bin/Cnbv.Webpart.Normatividad/NormatividadAjax.svc/ResolucionesYAnexos?normaId=1036`
(the instrument's row ID), which returns a JSON array of annex descriptions,
URLs, and order; the same response's empty `Resoluciones` array confirms no
amending resolution has been issued for this instrument since 2021-01-28.
These per-annex PDFs are the correct operational annex source: they are
hosted by the same operational publisher as the main PDF, they are the
mechanism CNBV itself uses to publish annexes from that page, and a
word-level fidelity comparison confirms their content is identical to the
DOF note's. The pipeline now fetches, hashes, and extracts each of the eight
annex PDFs as part of the operational acquisition (`annex-source-manifests.json`,
one manifest per annex, ordered) and parses each into its own `annex`
provision using the same paragraph and page-break rules as an article. The
formal DOF publication is still fetched and hashed for promulgation-date
provenance and cross-verification, per the standing rule to attach a formal
source when a decision depends on a later official act, but its text is no
longer parsed for canonical content.

Both official hosts (www.cnbv.gob.mx and www.dof.gob.mx) serve incomplete
TLS certificate chains. The adapter ships the missing public intermediate CA
certificates (GlobalSign RSA OV SSL CA 2018 and Go Daddy Secure Certificate
Authority G2), each of which chains to a standard trusted root; they are
added as additional trust anchors only for adapter fetches.

## 2026-07-03 — DCG parsing and heading model

The CNBV PDF has no page headers or footers, and page breaks fall
mid-sentence. Extraction keeps the form-feed page markers, and a paragraph
merges across a page break unless the previous line ends in `.`, `:`, or
`;`. Article 1's two-column definition layout is reconstructed
deterministically: lines indented past the definition column continue the
current definition; other lines split on their first run of three or more
spaces into term and definition fragments, and term fragments accumulate
until one ends with `:`. The adapter names definition-layout articles
explicitly. Heading context gains optional `section` and `apartado` levels
for Chapter II; heading subject lines remain structural context and are not
inserted into provision text, matching the LRITF chapter model.

Each annex PDF is parsed independently: its first non-blank, non-page-number
line must be its own "ANEXO N" / "Anexo N" heading (cross-checked against
the annex number implied by its position in the adapter's `annex_pdf_urls`),
and everything after it — including the subtitle — accumulates into body
paragraphs using the identical article rules. A bare 1-3 digit line is
treated as a page-number footer and dropped without affecting paragraph
boundaries. This is deliberately the same prose-oriented normalization used
for articles, not a bespoke table-cell reconstruction: Annexo 1's dense
multi-column risk-indicator matrix therefore renders as long, harder-to-scan
paragraphs rather than a gridded table, since a source-position-aware table
reconstruction would be exactly the "immature PDF parser" the project
already avoids for the main text. No content is lost — a word-level
comparison against the extracted PDF text found zero missing or added
words across all eight annexes — only the visual row/column structure of
that one dense table.

## 2026-07-03 — Cross-instrument references and title citations

Reference extraction now resolves targets against every instrument loaded
under `corpus/mx/`. The audited LRITF graph keeps its original whole-group
context policy and stays byte-identical. Multi-instrument extraction uses a
sentence-scoped policy: within the citation sentence, the earliest marker
decides among the instrument's own internal markers (configured per
adapter), configured external instrument names (for example, the LRITF's
full official name), and generic external-law context. Generic markers match
at word boundaries so `de la Ley,` counts as external. Citations of the
DCG's defined term `la Ley` without the full statute name remain unlinked —
resolving them requires the out-of-scope defined-term layer — as do named
laws not yet in the corpus, such as the Código de Comercio.

The DCG's statutory basis — LRITF Articles 48, 54, and 56 — is cited only in
the instrument's official title, not in any provision body. These citations
are canonical edges anchored to the instrument ID itself, with spans
validated against `official_title` and paragraph qualifiers preserved.
`disposición ORDINAL Transitoria` citations become transitory reference
edges; CUARTO resolves to LRITF's OCTAVA transitoria. A canonical reference
remains directed; reverse navigation is provided only by Obsidian backlinks
at presentation time.

## 2026-07-03 — Reviewer-initiated review of accepted determinations

A machine-accepted determination previously could not be corrected: only
items routed to review at import time were resolvable, and hand-editing the
temporal result would bypass the audit trail. `review open` (with
`review --instrument <slug>`) now lets the designated reviewer open a
pending item for any existing determination, preserving the machine
conclusion verbatim as the proposal; resolution then follows the normal
audited lawyer-override path. An existing item — pending or resolved — is
never replaced, so resolved reviews remain immutable. Opening also flags
the determination itself (`review_required` with the reviewer's reason)
and the canonical provision (`review_status: review_required`), so the
corpus and dashboards reflect the pending review instead of continuing to
report machine acceptance.

Reparsing re-applies the persisted temporal result to the fresh provisions
instead of resetting them: a default `pipeline` rerun therefore never
erases applied temporal state, including lawyer-verified decisions. Two
follow-on defects surfaced this and were corrected:

- **Reparse re-application originally accepted a bare substring match** of
  each supporting quotation against the new text. A materially changed
  provision could retain the quoted fragment nearby and incorrectly
  inherit a stale determination. `TemporalDetermination` now carries
  `evidence_sha256`, the hash of the exact evidence text the determination
  was made against; reapplication requires an exact hash match, not a
  quotation surviving somewhere in different text. A determination
  recorded before this field existed (empty hash) is grandfathered in once
  via the substring check it replaces, then has its hash backfilled so
  every later reapply is strict.
- **The evidence used for reapplication must be built the same way the
  temporal-analysis request itself is built** — ordinary transitory text
  plus the reform evidence the adapter marks relevant — not by scanning
  canonical provisions alone. An amendment-event determination's provision
  ID (`…:amendment:DATE:transitory:ordinal`) never appears among canonical
  provisions; it exists only in reform evidence. Reapplication reuses the
  shared evidence builder and runs with the freshly reparsed reform
  evidence (not a stale copy on disk), so amendment-event determinations
  reapply correctly instead of being uniformly flagged stale.

**Preserving review history previously kept only *resolved* items across a
model rerun.** A pending review — whether the model itself routed it there
or a reviewer opened it — could be silently cleared if a rerun happened to
produce a confident, clean result for that evidence, contradicting the
rule that review cannot be resolved by model confidence alone.
`preserve_temporal_review_history` now forward-carries every previous
item, pending or resolved — but only *restores it onto the corpus* when
`evidence_sha256` on the previous determination matches the freshly
routed current determination's own hash (already computed by this same
rerun against current evidence, before being overwritten). A hash
mismatch means the evidence changed since that review was made, so the
old determination is never applied: the freshly routed determination
stands.

The old review item itself is never dropped, though — an earlier version
of this fix did exactly that, silently deleting a reviewer's identity,
rationale, timestamp, and prior machine proposal from `review-queue.json`
on the very next hash mismatch, contrary to `AGENTS.md`'s requirement to
preserve those for every legal-review resolution regardless of what
happens to the underlying evidence afterward. The item is archived
verbatim under a version-qualified ID scoped to the evidence it concerns
(`review:temporal:<provision_id>:evidence:<hash>`, or `:evidence:legacy`
for a record with no hash at all), so it cannot collide with — or be
mistaken for — a fresh review opened under the canonical ID for the
current evidence. The CLI warns the operator by provision ID when this
happens, since it means a review is needed of the new text.

That archival step itself had a second-order bug: it reprocessed every
previous item on every call, including ones it had already archived. An
already-archived item's ID already carries an `:evidence:<hash>` suffix,
so archiving it again appended a second suffix
(`…:evidence:hash1:evidence:hash2`) instead of leaving the historical
record untouched, and the same provision could be reported superseded
more than once from a single rerun. An already-archived item is now
recognized by its ID and carried forward into `review_items` verbatim,
never re-compared against a determination or re-archived: only the one
live item under a provision's canonical ID is ever evaluated for
restoration or archival. Verified across two successive evidence changes
for the same provision — the archived ID and its contents stay identical
after the second rerun, and no second warning fires.

**Reparse reapplication's legacy fallback was itself unsafe.** A
determination predating evidence hashing (empty `evidence_sha256`) was
grandfathered in via the same one-time substring check it was meant to
replace, and its hash silently backfilled. That is exactly the weak check
the hash exists to replace: it is not run at all. A legacy record is now
unconditionally marked stale, forcing a fresh temporal-analysis run
instead of trusting an unverifiable match.

**`schemas/temporal-analysis.schema.json`, which documents the canonical
`TemporalDetermination` shape, was not updated for `evidence_sha256`.**
With `additionalProperties: false`, every determination written after
that field was added violated the schema. The field is now declared
(required, empty string or 64 lowercase hex characters) so committed
determinations validate.

**`review open` did not regenerate Markdown or the Obsidian dashboard**,
unlike `review resolve`; a newly opened review was invisible in published
output until an unrelated command happened to re-export. `review open` now
regenerates both, matching `resolve`.

First use: JRH corrected DCG transitory CUARTO's empty
`responsible_authorities`. The authorization that starts CUARTO's six-month
clock is granted by the CNBV previo acuerdo del Comité Interinstitucional
(LRITF art. 35, first paragraph), whose members represent the SHCP, Banco
de México, and the CNBV (art. 35, second paragraph) — verified against the
committed LRITF corpus text. The determination is now `lawyer_verified`
with the original machine proposal retained in the review record.

## 2026-07-03 — Multi-instrument vault indexes

With two instruments publishing notes with identical stems (for example,
`articulo-1`), generated Obsidian index links now use the full
`Corpus/<instrument>/<note>` path so wikilinks cannot resolve to the wrong
instrument. The pending-review dashboard aggregates review queues across all
committed instruments.

## 2026-07-02 — Canonical reference graph and presentation-only links

Express LRITF article citations are stored in `references.json`, separately
from canonical provision text. Edges use Unicode character offsets and exact
source spans, retain paragraph/fraction/inciso qualifiers, and distinguish
direct citations from range-expansion targets. Internal references must resolve
to a canonical provision before validation passes.

Standard Markdown and Obsidian wikilinks are injected only during export.
Named external-law citations are deliberately left unlinked until their target
instrument is in the corpus. The standalone `link` stage can regenerate the
graph from an already reviewed corpus without reparsing source text or changing
temporal decisions.
