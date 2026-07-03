# Architecture decisions

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
59 articles, and four transitories, but only lists the eight annexes: their
bodies appear solely in the formal DOF publication (código 5610487). The
pipeline therefore acquires both sources with full provenance. The CNBV PDF
remains the operational source for articles and transitories;
`formal-source-manifest.json` records the DOF acquisition, whose
deterministic HTML text extraction supplies the annex bodies as first-class
`annex` provisions. Annex table rows are preserved as single lines with
` | ` cell separators.

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
