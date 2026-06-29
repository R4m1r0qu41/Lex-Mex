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
