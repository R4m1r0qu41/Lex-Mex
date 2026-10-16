# Bounded federal source inspection — batch 01, 2026-10-15

## Result

| Candidate | Frozen source | Parsed records | Disposition |
| --- | --- | --- | --- |
| LAEM | Cámara, 9 pages, publication header 30 July 2010 | 17 articles; 5 transitories | Admitted locally; law ownership confirmed by operator |
| LAMNI | Cámara, 2 pages, publication header 22 January 1994 | 4 articles; 2 transitories | Admitted locally; temporal review unresolved |

LAEM and LAMNI are admitted to the local canonical working tree under the
operator's “admit” instruction. Corpus count is now 232 locally; the committed
baseline remains 230 until a commit is authorized and made. Of the 227 replay
candidates, two are admitted locally and 225 remain pending this inspection
sequence. The original 14 campaign holds remain unchanged. No temporal model or
legal-effect decision was made; the operator's LAEM ownership decision is
preserved below. Earlier inspection findings below describe the frozen replay,
with admission differences recorded in the admission section.

## Evidence inspected

For both frozen roots listed in the companion plan: retained extracted source,
manifest, instrument/provisions, candidate diff, validation, reference graph,
judicial appendix review and selected Markdown (first article and final
transitory; also LAMNI article 3). All 28 candidate provision bodies match their
source after collapsing whitespace and removing explicitly enumerated page
headers/footers from the comparison copy. All four source-byte hashes match
manifests. Every candidate provision ID has a Markdown file. LAMNI's sole
resolved reference has the exact source span and a local target that exists;
LAEM has no resolved reference records. This does not establish exhaustive
cross-instrument reference discovery.

Both validations are valid, with two `counts_not_frozen` warnings each. The
source counts corroborate 17/5 and 4/2, but adapters have not been modified to
freeze those baselines. Both judicial appendix review artifacts are empty;
no prior review queue is copied. All 28 provision states are `not_analyzed`
and `unknown`, with no derived effective dates. Instrument-level `in_force`
is existing machine metadata, not a reviewer finding in this inspection.

Candidate diffs are against earlier campaign candidates, not the committed
corpus: each removes only congressional/executive signature material after its
last transitory. Neither adds/removes a provision identity. Both candidate slugs
are absent from current `corpus/`. Manifest `resulting_git_commit` still records
the acquisition-era commit `1a37ccde2…`, while replay repair baseline is
`12e0cc752…`; do not describe that field as the replay implementation revision.
Exact hashes and artifact locations are in
`federal-source-inspection-batch-01-evidence.json`.

## LAEM attribution finding

The retained source opens with an issuing decree and `ARTÍCULO ÚNICO. Se
expide la Ley que crea la Agencia Espacial Mexicana.` After law article 17,
its five transitories include `El presente Decreto` in Primero, `presente
Decreto` in Segundo and `este Decreto` in Cuarto. Current candidates store all
five under ordinary LAEM transitory IDs and have no separate enactment sidecar.

### Operator resolution — 2026-10-15

JRH inspected the PDF and confirmed that all five transitories belong to the
law; the issuing decree has one artículo único and no transitories of its own.
This supersedes the inspection's provisional ownership hold and proposed
migration to decree evidence. Retain all five ordinary LAEM transitory IDs and
exact bodies. No ownership migration or parser repair is required for LAEM on
this finding. Both candidates are structurally ready for the admission gate;
this attribution decision does not resolve temporal effects or commencement.

## LAMNI identity and readiness

The PDF URL ends in `19.pdf`; identity comes from its title, 22 January 1994
publication header and Chiapas-specific article 1, not the filename. It is the
1994 law, distinct from the 2020 amnesty candidate LAMN. Its transitories say
`Esta Ley`, and the source shows no expiding single-article wrapper between
the issuing formula and the law title. The final transitory body ends before
signatures; the exported final transitory retains that boundary. Article 3's
commission-dependent effects remain exact source text and unresolved.

Recommend structural admission only after freezing independently inspected
counts, rerunning the native/affected admission gate and reviewing the canonical
diff. Formal DOF evidence is still required for any later determination of
publication-dependent legal effects; this inspection makes none. No live-source
freshness check or PDF visual/layout audit was performed: conclusions concern
the retained campaign extraction and supporting furniture audit.

## Admission execution — 2026-10-15

Added frozen adapters `adapters/diputados/laem.json` (17/5) and
`adapters/diputados/lamni.json` (4/2). Reused retained PDFs and acquisition
manifests, then ran `extract`, `parse`, `link`, `validate` and Markdown `export`
with the current Rust CLI for each instrument. No network fetch was needed.
Both validations have zero issues. LAEM has one resolved reference to committed
CPEUM article 123; LAMNI has one local reference to article 1. Exact character
spans and target existence were checked. LAEM's additional edge is explained by
linking against the complete current corpus instead of its isolated replay root.

All instrument/provision records, source manifests and reform evidence are
identical to their inspected replay JSON. All 28 provisions remain
`not_analyzed` / `unknown`, with null effective dates; no review queue or model
result was created. No existing instrument was rewritten. Markdown inspected:
LAEM's final transitory and LAMNI's linked article 2 retain their source bodies.
Generated publication output is under each new corpus directory's `markdown/`.
Exact stage output and canonical artifact digests are in the companion evidence.

Native validation results are recorded in the plan after completion. Existing
corpus reverse relinking remains deferred to the broader ingestion sequence.
No commit or push performed; unrelated working-tree changes remain intact.

## Next action

Inspect the next bounded cluster among the remaining 225 replay candidates.

## Original operator annotation (preserved verbatim)

All five Transitorios belong to the Law the decree was an articulo unico and in the PDF has no transitorios
