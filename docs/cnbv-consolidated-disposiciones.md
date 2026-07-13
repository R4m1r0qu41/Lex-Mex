# CNBV consolidated disposiciones — amendment provenance, in-force status, corpus currency

Status: **specification, ratified by JRH 2026-07-12; implementation gated.**
Scope: the CNBV *disposiciones de carácter general* (DCGs) — consolidated
regulatory texts the CNBV republishes as a single compiled PDF that folds in
every *Resolución Modificatoria* (RM) issued against it. This is the model the
`itf-dcg` parser generalization must implement. Sections 2 and 4 (in-force
status, corpus currency) generalize to every instrument, not only DCGs, and
are noted as such.

Related: `docs/legal-model.md` (temporal basis, terms, edges),
`docs/decisions.md` (2026-07-12 entries), the `itf-dcg` parser
(`crates/lex-parse/src/itf.rs`, `dcg.rs`), and the scap-dcg-2012 pilot.

---

## 1. Amendment markers and the REFERENCIAS legend

### 1.1 What the markers are

A compiled DCG carries numbered `(N)` superscript **amendment markers**
inline in the text and a **REFERENCIAS** legend at the end of the document.
The legend maps each marker to a one-line provenance statement:

```
N)  Reformado por Resolución publicada en el Diario Oficial de la
    Federación el 3 de diciembre de 2014.
```

Each statement records an **action** — *Reformado* (amended), *Adicionado*
(added), *Derogado* (repealed), *Modificado*, *Fe de erratas*, etc. — and the
**DOF publication date** of the modifying resolution (RM) that made the
change. The RM is version-control provenance: it tells you *when* a unit
changed and *by which* resolution.

### 1.2 The three-layer model

```
  (N)  marker on a structural node
   │  integer key — deterministic, authoritative
   ▼
  REFERENCIAS[N] = { acción, fecha_DOF }
   │  DOF-date string — NOT a unique key (see 1.5)
   ▼
  TRANSITORIOS block of that RM = in-force rules
```

- **Layer 1 → 2 (marker → REFERENCIAS): the authoritative link. Build and
  keep it.** `N` is an integer key into a numbered legend. This is the "keep
  the link in the compiled version to the reference" JRH ratified: the corpus
  records, for each marker, the resolved `{ acción, fecha_DOF }` so a reader
  can inspect what changed, when, and by which RM, and search the transitorio
  themselves. This *replaces* the earlier "keep the mention, no link"
  framing — the reference link is now kept; only the transitorio link is
  withheld.
- **Layer 2 → 3 (REFERENCIAS → the specific RM transitorio): deferred.** The
  modifying resolutions are **not corpus instruments** — only the final
  compiled text is. Ingesting RM texts would balloon the corpus with many
  "useless" superseded texts (e.g. the CUB has hundreds of RMs). We do **not**
  build the marker → transitorio join now, and we do **not** ingest RM texts.
  "Later we might wire them in too" — recorded as a future option, not a
  current requirement.

### 1.3 Markers attach to any structural node — not only articles

The marker can hang off **any** node: the *denominación* (the instrument's
own title), a TÍTULO, an artículo, a párrafo, a fracción, an inciso. OAAC's
compiled title opens with `(18)` — `REFERENCIAS[18]` = "Reformado por
Resolución publicada en el DOF el 3 de diciembre de 2014", the RM that renamed
the instrument.

This reframes the parser's earlier failures. The `itf-dcg` parser was
**provision-centric**: it assumed a marker always attaches to an article or a
transitory, so a marker landing on a denominación, a CONSIDERANDO, or a TÍTULO
heading had nowhere to drain and raised a conservative `discard` error. That
error was the parser being too narrow, **not** the document being corrupt.

**Rule.** Collect every `(N)` wherever it appears. Attach it to its nearest
enclosing structural node:

| Marker context                    | Attachment target                       |
|-----------------------------------|-----------------------------------------|
| denominación (instrument title)   | instrument-level `amendment_marks`      |
| TÍTULO / Capítulo / Sección head  | that heading node                       |
| artículo / párrafo / fracción     | the provision (`Provision.amendment_marks`) |
| per-RM TRANSITORIOS block         | that reform transitory (`TemporalEvidence.amendment_marks`) — already implemented |
| CONSIDERANDO / attribution block  | nearest preceding node, else instrument-level |

### 1.4 REFERENCIAS is the validation oracle

Every marker `N` present in the body **must** have a `REFERENCIAS[N]` entry.
This is the anti-silent-loss oracle:

- **N in body but not in REFERENCIAS → hard error.** Genuine corruption or a
  parse that lost the legend. Never drop it silently.
- **N in REFERENCIAS but not in body → warning.** An orphaned legend entry
  (often a marker attached to an annex not reproduced, or lost in extraction).
- A marker in a CONSIDERANDO or attribution block is **valid** — it still
  resolves against REFERENCIAS. Attachment position never gates validity;
  *resolvability against the legend* does.

This oracle is what makes the socap/oaac region-detection fix verifiable:
whatever the parser does with region boundaries, the set of body markers must
equal (⊆, allowing orphaned-legend warnings) the REFERENCIAS key set.

### 1.5 DOF-date collisions — warn, never guess

The DOF date is **not** a unique RM identifier. Two RMs can share a
publication date (observed: the CUIFE listing has *11a* and *12a* both dated
08/01/2015). The compiled PDF's REFERENCIAS legend gives only the **date** —
the RM ordinal ("10a") lives on the CNBV Normatividad web listing, not in the
PDF text. So a marker dated `8 de enero de 2015` cannot be resolved to a
specific RM from the document alone.

Per JRH: a same-date RM pair is an **outlier frontier case** (likely an
upstream fumble), but the model must account for it rather than break on it.

**Rule (applies only if/when the transitorio layer is ever wired in).**
Resolution of a REFERENCIAS date to a specific RM transitorio is best-effort:
exactly one match → link; zero → none (many old RMs' transitorios are not
reproduced in the compiled text, which is fine); **two or more → ambiguous:
attach all candidates, emit a warning, never machine-pick the ordinal.**

**Future disambiguation aid.** Capture a **snapshot of the CNBV Normatividad
page's "Resoluciones Modificatorias" listing** for each DCG (the list carries
the ordinal + DOF date for every RM — see §4, where the same snapshot also
drives currency-checking). A colliding REFERENCIAS date can then be
disambiguated against the ordered list instead of guessed.

---

## 2. In-force status: live vs. staged  *(generalizes to all instruments)*

The genuinely useful signal — more useful than a transitorio link — is
whether a provision is **actually in force today**.

A modifying resolution's TRANSITORIOS block states when its changes enter into
force. The default (`ÚNICO`, ~99% of the time) is *the day after DOF
publication*. But many RMs **stage** entry into force: a change is published
now yet becomes effective months or years later, sometimes tiered across
several provisions (PRIMERA next-day, SEGUNDA a phased set with its own
deadlines). OAAC, for example, carries provisions whose text is compiled and
published today but that **enter into force in 2027** — present in the text,
not yet operative. And staged deadlines can **move**: if a deadline will not
be met, the authority often extends it via a later RM.

These per-RM TRANSITORIOS blocks **are reproduced inside the compiled DCG**
(unlike the RM texts themselves, which are not), and the parser already
captures them as `TemporalEvidence` / reform evidence (scap-dcg-2012 captured
6). So the in-force signal is derivable from data we already hold.

**Proposed model (design call — requires JRH ratification; touches
`TemporalStatus` / effect categories, i.e. the schema-boundary / higher-care
path).** For each provision, expose an **in-force status** derived from the
governing transitorio's effective-date rule:

- `live` — effective date reached (or next-day default with no deferral).
- `staged` — published but effective date is in the future; record the
  `effective_date` and its source transitorio span.
- `staged_extended` — a later RM moved a previously-staged deadline; keep the
  prior date in provenance.
- `unknown` — no determinable effective-date rule (do not assert `live`).

Open questions for JRH before this becomes schema:
1. Is `staged` a new `TemporalStatus` variant, or a computed overlay on top of
   `Effective` (status = what the law says; live/staged = whether *today* is
   past the effective date)? A computed overlay keeps the corpus date-stable
   and avoids re-committing when a date simply passes.
2. Granularity: instrument-level, or per-provision (OAAC needs per-provision —
   different articles stage on different dates)?
3. Do we surface a computed "as-of `today`" liveness in export, or store only
   the effective-date rule and let the reader/UI compute liveness? (Storing
   the rule, computing liveness at render, avoids a corpus that goes stale by
   the calendar.)

---

## 3. Definitional remittance — deferred to the cross-instrument pass

A glossary entry may define a term by **remitting to another instrument**:
LRITF-style, the ITF-DCG's `Valores` (art. 1, fr. LVI) reads "*a los
considerados como tales por la Ley del Mercado de Valores*". The locally
resolved link (usage → art. 1 fr. LVI) is only *half* the information; the
operative definition is LMV art. 2, fr. XXIV (the enumerated one). The useful
resolution is transitive: usage → local glossary entry → **LMV glossary
entry** → the real definition.

This is **deferred to the cross-instrument reference-resolution pass** (the
CURRENT SCOPE "followed by cross-instrument reference resolution"), which runs
**once, after the full federal corpus is ingested** (close to complete), then
is maintained incrementally as instruments are added.

Determinism notes for that pass:
- The remittance here is **bare** — it names the law (LMV) but **not** the
  article. Resolution is therefore a **lemma-join**: match headword `Valores`
  in LMV's glossary. Deterministic only when (a) LMV is in the corpus and
  (b) the headword maps 1:1; a missing or multi-sense headword must surface,
  not silently resolve.
- Where a remittance *does* name an article ("en los términos del artículo N
  de la Ley Y"), it resolves as an ordinary express cross-reference edge.
- Reuses the existing `instrument-aliases.json` for instrument-name
  resolution and the additive-glossary rule already in `legal-model.md`.

Nothing here is built in the DCG parser generalization; it is recorded so the
second pass inherits the rule.

---

## 4. Corpus currency — upstream update detection  *(generalizes to all instruments)*

The CNBV republishes a DCG's compiled PDF whenever a new RM lands, and the
Normatividad page can lag the actual update by a couple of days (observed:
ITF-DCG-2018's live PDF was refreshed on a Thursday; the page reflected it
days later). A committed corpus therefore silently goes stale against its
official source.

**Requirement (JRH): wire in a mechanism that checks for upstream updates on a
schedule and keeps the corpus current** — surfacing drift for review rather
than auto-committing changed law.

Design outline (to be specified in its own implementation note):

- Periodically re-acquire each instrument's operational source **hash** and
  compare against the committed source hash (`lex-source` already hashes on
  fetch). A changed hash → **staleness signal**, routed to review, never
  auto-ingested.
- For CNBV DCGs, additionally capture the **Normatividad "Resoluciones
  Modificatorias" listing snapshot** (§1.5): a new RM row (new ordinal / new
  DOF date) is an early, page-level update signal even before the compiled PDF
  is refreshed — and the same snapshot disambiguates colliding REFERENCIAS
  dates.
- Cross-check the compiled text's **latest REFERENCIAS date** and any
  `latest_reform` metadata against the RM listing: if the listing shows an RM
  newer than the corpus's latest marker, the corpus is behind.
- Output is a **currency report** (per instrument: current / behind / source
  unreachable) feeding the review queue. Re-ingestion of a changed source
  stays a deliberate, reviewed act (`corpus/` is committed canonical data).

This subsumes the standing "ITF-DCG-2018 reform re-ingest" TODO: it becomes
the first case the currency mechanism flags.

---

## 5. Implementation checklist (when unblocked)

1. **Marker generalization** (`itf.rs`/`dcg.rs`): markers attach to any node;
   REFERENCIAS as validation oracle (§1.4); replace the false `discard` errors
   on cue/cub/cucb/fi with attach-to-nearest-node; keep the true error for a
   body marker with no REFERENCIAS entry. Fixture per context (denominación
   marker, CONSIDERANDO marker, título marker).
2. **socap/oaac region detection**: the real structural fix, verified against
   the REFERENCIAS oracle (body-marker set ⊆ legend key set).
3. **servinv ÚNICO dedup**: multiple reform ÚNICO transitorios must get
   distinct ids (per-RM disambiguation, e.g. by DOF date), not collide.
4. **In-force status** (§2): only after JRH ratifies the model shape (schema /
   effect-category decision).
5. **Currency mechanism** (§4): separate implementation note; acquisition-layer.
6. **Remittance** (§3): cross-instrument pass, after full ingestion — not now.

Marker → REFERENCIAS resolution, the "keep the reference link, defer the
transitorio link" policy, and the date-collision warn-don't-guess rule are
**ratified** and may proceed. In-force status and the currency mechanism are
ratified in intent; their schema/interface shapes await JRH sign-off.
