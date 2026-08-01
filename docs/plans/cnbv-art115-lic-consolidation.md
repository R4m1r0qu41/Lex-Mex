# CNBV/SHCP art. 115 LIC disposiciones — consolidation (research pass)

Status: **inventory closed for the base instrument (2026-07-31), pending
final gap-year sweep. Nothing fetched into `.work/` or `corpus/`.**

Instrument (in scope): *DISPOSICIONES de carácter general a que se refiere
el artículo 115 de la Ley de Instituciones de Crédito* — the PLD/AML rules
for **banks** (identification, KYC, reporting obligations under LIC art.
115). Issued by **SHCP**. Title pattern: singular *"el artículo 115"*, no
"87-D" / "95-Bis" / "Sociedades Financieras de Objeto Múltiple" in the
title.

## Sourcing determination — self-compilation confirmed necessary

Verified directly: fetched `www.cnbv.gob.mx/Paginas/Normatividad.aspx` (the
same registry `adapters/cnbv/*.json` compiled sources are drawn from —
cross-checked against `itf-dcg-2018`'s table row) and grepped its full
~190KB table for `artículo 115`. Two hits, both `Acuerdo` rows (format-
notice documents about specific avisos, not the base disposiciones):
`04/03/2024` and `01/04/2015`. **No row exists for the base instrument
itself** — operator independently confirmed the same.

Per the trusted-source-first doctrine (`docs/decisions.md`, capsule
501c9c5c8): no CNBV-compiled trusted source exists, so **self-compilation
from DOF (base + every Resolución Modificatoria) is the correct path**, not
a fallback taken by default. This mirrors the NOM situation, not the DCG
situation — `docs/cnbv-consolidated-disposiciones.md`'s REFERENCIAS-legend
model does **not** apply (there is no compiled PDF to carry markers).

## Scoping correction — a sister instrument was contaminating the inventory

**Found and corrected during this pass.** SOFOMs (Sociedades Financieras de
Objeto Múltiple) have their **own separate** PLD disposiciones, issued under
LGOAAC arts. 87-D/95-Bis, whose titles also happen to contain "artículo
115" (plural *"los artículos 115... en relación con el 87-D..."*) because
they incorporate the bank rules by cross-reference. This is a **different
instrument**, not an amendment to the bank disposiciones. Confirmed by
reading full text (not just title) of codigo 5475644 (2017-03-09): its
legal basis cites only LGOAAC arts. 87-B/95-Bis, its CONSIDERANDOs discuss
only *sociedades financieras de objeto múltiple*, zero mention of bancos/
instituciones de crédito.

Sister instrument's own base publication: **2011-03-17, codigo 5182183**
("Jueves 17 de marzo de 2011... PODER EJECUTIVO"). Its own amendment chain
includes at least 5554779 (2019-03-21), 5584217 (signed 2020-01-08),
5629270 (2021-09-09) — all excluded from the table below. **Not pursued
further this session** — record here only so a future SOFOM-instrument
ingestion doesn't re-discover this from zero.

This also explains the earlier false alarm: the operator's original
"no reforms in 2012/2016/2018/2021/2023/2025" read was checked against a
`site:` search that plausibly *did* surface the SOFOM-sister 2021 event
under the same "artículo 115" query, muddying which instrument it belonged
to. Re-checked with instrument-scoped terms (see Gap years below) — 2021
is now confirmed empty **for the bank instrument specifically**, so the
operator's original conclusion holds for this instrument once correctly
scoped; the sister instrument's 2021 event is real but out of scope.

## Acquisition mechanics — confirmed working

- `dof.gob.mx` and `www.cnbv.gob.mx` both serve incomplete TLS chains
  (`crates/lex-source/src/lib.rs:170-178`, existing doctrine). The CA PEMs
  already in `adapters/cnbv/` (`globalsign-rsa-ov-ssl-ca-2018.pem`,
  `godaddy-secure-ca-g2.pem`) fix this — every fetch below used
  `curl --cacert <(cat both pems)` against `nota_detalle.php?codigo=...&fecha=DD/MM/YYYY`.
  Reuse the same PEMs for a new `art115-lic` adapter.
- `sidof.segob.gob.mx/notas/docFuente/<codigo>` mirrors the same notes,
  fetches cleanly with no CA workaround (valid chain) — useful corroboration
  or fallback if `dof.gob.mx` rate-limits.
- The user's DOF full-text search UI (`busqueda_detalle.php#gsc.tab=0`) is a
  Google CSE embed and will not render under a non-JS fetch. It was used
  manually to seed leads; every lead was then confirmed against
  `nota_detalle.php` directly before being trusted.

## RM inventory — bank instrument, all rows directly fetched and title-confirmed

Every row below was fetched via `curl --cacert` against `dof.gob.mx` this
session and its title read from the live page (not a search snippet).

| # | DOF date | codigo | Action (as titled, confirmed) |
|---|----------|--------|-------------------------------|
| 0 | 2009-04-20 | 5087613 | (base publication) — SHCP, signed Carstens |
| 1 | 2010-06-16 | 5146921 | reforma y adiciona |
| 2 | 2010-09-09 | 5158510 | reforma y adiciona |
| 3 | 2010-12-20 | 5171759 | reforma |
| 4 | 2011-08-12 | 5204615 | reforma, deroga y adiciona |
| 5 | 2013-03-13 | 5292050 | reforma |
| 6 | 2014-04-25 | 5342249 | reforma, adiciona y deroga |
| 7 | 2014-09-12 | 5360165 | reforma, adiciona y deroga |
| 8 | 2014-12-31 | 5377890 | reforma, adiciona y deroga |
| 9 | 2015-09-10 | 5408023 | reforma y adiciona |
| 10 | 2017-02-24 | 5472995 | reforma, adiciona y deroga diversas |
| 11 | 2017-12-27 | 5509088 | reforma |
| 12 | 2019-03-22 | 5554909 | reforma, adiciona y deroga diversas |
| 13 | 2020-06-09 | 5594646 | reforma y adiciona diversas |
| 14 | 2020-07-14 | 5596648 | reforma y adiciona |
| 15 | 2022-03-03 | 5644451 | reforma y adiciona (recognizes prepaid cards for foreign visitors) |
| 16 | 2024-08-28 | 5737473 | reforma, adiciona y deroga diversas |

**1 base + 16 RMs, 17 documents total, all confirmed by direct fetch.**

### Cross-check source (corroboration only, not trusted)

A compiled-as-of-2014-12-31 PDF exists at
`gob.mx/cms/uploads/attachment/file/207270/...pdf` ("D.O.F. 31 Dic 14 (DCG
Bancos)"). Not a CNBV Normatividad-registry source, so it cannot be the
operational source per the trusted-source-first doctrine — but it's a
useful independent cross-check for the pre-2015 state (rows 0–8) since it
was compiled by a third party at a point in time, before self-compilation
is trusted.

## Gap years — re-checked with instrument-scoped terms

2012, 2016, 2018, 2021, 2023, 2025: no Instrument-A row found. 2021 was
specifically re-checked (it's the year the scoping correction above
explains) and the only 2021 hits are the SOFOM-sister resolution (5629270,
out of scope) and an unrelated Banco de México circular (5618161, Circular
2/2021, a different regulator's different instrument). Treat the remaining
gap years as **best-effort empty, not exhaustively proven** — DOF has no
queryable index for this instrument and CNBV's registry doesn't carry it,
so absence-from-search is the only signal available short of sequentially
scanning DOF codigo ranges.

## Abrogation check

Operator confirms (2026-07-31): no RM abrogated/replaced the instrument
wholesale — all 16 are partial `reforma`/`adiciona`/`deroga` of individual
numerales. Compilation base stays 2009-04-20 (row 0).

## Compiled-output modeling rule (operator, 2026-07-31)

Where the same article/numeral was touched by more than one RM across
2009–2024, **the compiled output carries only the latest in-force version**
of that provision. Earlier versions are not discarded — they're retained as
**provenance/historical record** (which RM, which date, superseded — not
currently in force), same shape as the amendment-marks/REFERENCIAS
provenance model used elsewhere in this repo, just without a publisher-
supplied legend to anchor it. This needs an explicit per-article
"latest-wins, history-retained" resolution pass once numeral-level diffs
are extracted (Next steps, below) — it is not automatic from having the 17
documents.

## CORRECTION (2026-07-31, same day) — the "always full text" claim below is wrong; 46 of 96 provisions are affected

Operator caught this by reading the compiled output directly (disposición
4ª's text was full of literal `...`). Verified against source: `...` inside
a "para quedar como sigue" restatement is **DOF's own convention for "this
sub-part is untouched, retains its prior text"** — confirmed literally
present in the raw DOF HTML markup, not an extraction artifact. This means
the "structural finding" section below and the `docs/decisions.md`
2026-07-31 entry it's based on are **wrong on their central claim** (a full
retraction is recorded in `docs/decisions.md`, quoted-and-struck, at the top
of that file — read it before trusting anything below about "no
ellipsis-splicing needed").

**Real finding:** this format IS an ellipsis-diff, like the NOM case — just
nested one level deeper (elided sub-parts *inside* a named, numbered unit,
rather than across the whole document). A correct compiled text requires
walking each affected key's history backward to splice in the last real
text for every elided sub-part. **Not done.** Measured scope: **46 of the 96
keys in `compiled_draft.json` contain literal `...`** (list in
`.work/art115-lic/ellipsis_affected_keys.json`) and are flagged in both the
JSON and the Obsidian copy — this supersedes the original "5 known gaps"
framing; the real defect surface is an order of magnitude larger. The 5
gaps below are still real and still separate (they're about missing
restatements entirely, not elided sub-parts within a restatement that
exists) — do not conflate the two.

## Extraction pass — all 16 RMs (2026-07-31, time-boxed per operator direction)

Ran the `ARTÍCULO ÚNICO` extraction (methodology in `docs/decisions.md`
2026-07-31) against all 16 cached RM texts plus the base. Script, cached
DOF HTML, and output all live in `.work/art115-lic/` (gitignored — hold-out
work area, matches the existing NOM convention of compiling to `.work/`
only, never `corpus/`).

- `.work/art115-lic/extract2.py` — per-RM extraction (preamble +
  REFORMAN/DEROGAN/ADICIONAN lists + per-disposición full replacement
  text), handles the two-`para quedar como sigue`-blocks case (a later RM
  amending an earlier RM's own transitorio, flagged
  `is_transitorio_amendment`).
- `.work/art115-lic/base.json` — the 2009-04-20 base split into 68
  disposiciones.
- `.work/art115-lic/all_rms.json` — all 16 RMs' extracted blocks.
- `.work/art115-lic/merge.py` — builds chronological per-disposición
  history, latest-full-text-wins compiled view.
- `.work/art115-lic/compiled_draft.json` + `INDEX.md` — the result: **96
  distinct disposición keys** (base's 68 plus new numerals added by later
  RMs), 78 touched more than once, 17 never reformed since 2009. **Research
  draft only — do not treat as correct without the manual checks below.**

### Known gaps — exact documents to compare (updated 2026-07-31, operator is doing the manual check)

The "always full-text replacement" methodology finding holds for the large
majority of touches, but is **not universal**. Found by cross-checking each
RM's DEROGA clause against its own replacement-text numerals. Each gap below
now names precisely which DOF documents to pull and what to compare — this
supersedes the earlier, more general gap descriptions. Fetch any codigo at
`https://www.dof.gob.mx/nota_detalle.php?codigo=<codigo>&fecha=<DD/MM/YYYY>`
or the SIDOF mirror `https://sidof.segob.gob.mx/notas/docFuente/<codigo>`.

1. **`68a`** — COMPARE the base text (2009-04-20, codigo **5087613** — the
   only text on record for this key; touch_count=1, nothing since 2009)
   AGAINST the derogation instruction in RM 2014-04-25 (codigo **5342249**)
   preamble: "se DEROGA...el tercer párrafo de la disposición 68ª". ACTION:
   remove 68ª's tercer párrafo per that instruction — the only outstanding
   edit, confirmed no other RM in the 16-RM set touches 68ª.
2. **`64a`** — COMPARE, in order: (1) RM 2014-12-31 (codigo **5377890**)
   preamble — prose-only instruction, no restatement anywhere: "el primer
   párrafo, así como tercer y cuarto, para quedar como segundo y quinto
   párrafos de la 64ª... se DEROGA el segundo párrafo de la 64ª" — AGAINST
   (2) RM 2017-02-24 (codigo **5472995**)'s full restatement of 64ª, the
   next full text given after that edit. ACTION: verify 5472995 already
   correctly reflects the 2014-12-31 renumbering/derogation. If yes, the
   text currently shown (2024-08-28, codigo **5737473**, which restates
   64ª again downstream of 5472995) is safe as-is. If 5472995 got it
   wrong, the error carries forward and needs correcting there too.
3. **`14a_Bis`** — COMPARE the current text (2022-03-03, codigo
   **5644451**, the latest full restatement) AGAINST RM 2024-08-28 (codigo
   **5737473**) preamble, which derogates "la 14ª Bis, fracción II, cuarto
   párrafo" in prose only — 14ª Bis is never restated by 5737473 (only
   plain 14ª is, a different key). ACTION: remove "fracción II, cuarto
   párrafo" from the 2022-03-03 text to get the true post-2024-08-28 text.
4. **`16a_Bis`** — COMPARE the current text (2019-03-22, codigo
   **5554909**, the only time this key was ever given full text — added
   by this RM) AGAINST RM 2024-08-28 (codigo **5737473**)'s exact DEROGA
   clause: "...la 14ª Bis, fracción II, cuarto párrafo; la 16ª Bis; todas
   ellas..." — note 16ª Bis has **no** paragraph/fracción qualifier,
   unlike 14ª Bis listed right before it in the same sentence, which
   suggests **whole** repeal rather than partial. ACTION: read the exact
   DEROGA clause off codigo 5737473 (DOF 28/08/2024) directly to confirm;
   if whole, mark 16ª Bis repealed as of 2024-08-28, not live text.
5. **`7a_1`** — lifecycle reconstruction, three documents in order: (1)
   **ADDED** by RM 2017-02-24 (codigo **5472995**) — preamble lists "la
   7ª-1" under se ADICIONAN; its added text should sit under a "7ª-1.-"
   header in that RM's own replacement body, which this extraction's key
   normalizer doesn't recognize (hyphenated format) — pull it directly
   from codigo 5472995. (2) **AMENDED** by RM 2017-12-27 (codigo
   **5509088**) — preamble reforms "la 7ª-1, primer párrafo" along with
   three other items (16ª, 21ª-3, 62ª Quáter), but this extraction's
   body-split only recovered "16ª" from that RM — check codigo 5509088
   directly for whether it restates 7ª-1's primer párrafo in full or only
   in prose. (3) **REPEALED** by RM 2019-03-22 (codigo **5554909**) —
   DEROGA clause names "7ª-1" with no replacement text (confirmed).
   ACTION: for the current compiled draft, 7ª-1 no longer exists as of
   2019-03-22 and should not appear as a live provision; steps (1)-(2) are
   only needed if 7ª-1's historical text itself is wanted.

None of these are corrected in `compiled_draft.json` — each carries the
same reference text above as its `known_gap` note. The same references are
now also in `.work/art115-lic/INDEX.md` and in the compiled-draft copy at
`/Users/jr/vaults/Obsidian Mac M5/Leg4114bs/Art115-LIC-disposiciones-compiled-draft.md`
(operator's working copy, generated 2026-07-31 for immediate use outside
this repo — not corpus-grade, not legal advice, carries the same warnings).

## What this session did NOT do

- No `art115-lic` adapter was written (no Rust code touched at all).
- Nothing entered `corpus/` — the compiled draft lives in `.work/`, which
  is gitignored, per the same hold-out convention used for NOMs with
  unresolved defects.
- The 5 known-gap keys (`68a`, `64a`, `14a_Bis`, `16a_Bis`, `7a_1`) are not
  resolved — flagged, not fixed.
- No legal review of the compiled text's substance — this is a mechanical
  extraction, not a reviewed corpus record.

## Structural finding — decree format is simpler than NOM's, not the M4 case

Read two RMs' full operative clauses (5146921/2010-06-16, 5204615/2011-08-12)
to answer the M4-learning question directly. **This instrument's RMs do not
use the NOM-style verbatim ellipsis-diff** ("unchanged span ... replacement
text"). Instead, each RM has a single `ARTÍCULO ÚNICO.-` clause with this
exact shape:

> Se REFORMAN las disposiciones `<N, N, N...>`; se DEROGA(N) `<N, N...>`
> (sometimes only a named paragraph/fracción within a disposición); y se
> ADICIONAN `<N, N...>`; todas de las Disposiciones..., **para quedar como
> sigue:** `<full replacement text of every disposición named in the
> REFORMAN/ADICIONAN lists, numbered, in full>`

Consequences for compilation:
- **Reformed/added numerals**: the RM gives the *complete* new text of the
  numbered unit — a straight replace-by-key operation, not a diff to
  splice. A disposición that had one paragraph derogated and others
  reformed still appears once, in full, under "para quedar como sigue" —
  the derogated paragraph is simply absent from that text, so partial
  derogation-within-a-reformed-unit needs no special handling.
- **Wholly derogated numerals** (a numeral repealed outright, not reformed)
  are named only in a "se DEROGA(N)" clause with no replacement text and
  must be dropped from the compiled output — **confirmed** in the full
  16-RM pass: 5554909 (2019-03-22) wholly derogates disposición "7ª-1"
  this way (see `7a_1` in Known gaps below).
- **Caveat found in the full pass, narrows the "always full text" claim:**
  not every reform gets a "para quedar como sigue" restatement. A purely
  *structural* edit (paragraph renumbering, single-paragraph derogation
  with no other substantive change) can be described as an instruction
  entirely within the preamble, with no replacement text anywhere — see
  `64a` in Known gaps below. The full-text-replacement convention is the
  norm for substantive changes, not a universal guarantee; an extraction
  must still cross-check the DEROGA/REFORMA lists against what actually
  appears in the replacement body, the way this pass did.
- This is **simpler than what `decree-diff`/M4 was scoped for** (M4 assumes
  ellipsis-style diffs per `docs/ingestion-difficulty-log.md`). A
  purpose-built parser for "named-unit full-text replacement, keyed by
  disposición number" may suffice here without the heavier ellipsis-
  splicing engine — worth flagging back to `docs/plans/maximasa-legal-
  integration.md` as a *second, easier* decree-diff subclass rather than
  assuming every `decree-diff` case needs the full M4 machinery.
- Also surfaced in 5204615's CONSIDERANDO (provenance-only, doesn't change
  scope): the 2009-04-20 base itself "abrogaron a sus similares publicadas
  en diciembre de 2006" — i.e. a pre-2009 predecessor instrument was fully
  abrogated by the 2009 base. Confirms 2009-04-20 is the correct
  compilation base (nothing before it is legally live), just noting the
  lineage exists further back if ever needed.

**All 16 RMs now extracted** (see "Extraction pass" above) — this
supersedes the "not yet read" state. Result: `.work/art115-lic/`, gitignored,
not `corpus/`.

## Next steps

1. **Resolve the 5 known gaps** (`68a`, `64a`, `14a_Bis`, `16a_Bis`,
   `7a_1`) by hand — each needs a targeted read of its specific RM
   preamble against the base/prior text, not a re-run of the mechanical
   extraction (which structurally can't see these by design: no
   replacement text exists for the mechanical pass to find).
2. Spot-check a sample of the 78 "touched more than once" keys against
   their source RM text for correctness (the mechanical split has not been
   verified beyond the two RMs read in full during the structural-finding
   pass, plus the gap-hunting above) before trusting the draft for
   anything beyond orientation.
3. Decide, with the operator, whether this compiled draft is useful enough
   as-is for whatever "some work I'm doing" (the original ask) needs, or
   whether it must wait for a reviewed, corpus-grade pass — this stays a
   `.work/`-only research draft either way per the `decree-diff` gate,
   unless the operator explicitly signs off on treating it as sufficient
   for the immediate external need without a full corpus ingestion.
4. If this instrument later becomes a real corpus target: write the
   `art115-lic` adapter, port `extract2.py`/`merge.py`'s logic (or
   equivalent) into `lex-parse` as the "named-unit full-text replacement"
   decree-diff subclass, add fixtures for the known-gap cases (prose-only
   structural edit, wholly-derogated hyphenated numeral), and run it
   through the normal validation/review pipeline — none of that happened
   this session.
5. No `abrog` grep needed (operator-confirmed at instrument level); the
   2009-04-20 base's own predecessor-abrogation (Dec. 2006) is provenance
   context only, doesn't change the base.
