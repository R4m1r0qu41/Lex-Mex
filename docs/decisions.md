# Architecture decisions

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
