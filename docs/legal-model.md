# Legal model

The Cámara de Diputados consolidated PDF is the operational source for current
federal statutory text. The Diario Oficial de la Federación is the formal
publication source. These roles are represented separately and must not be
collapsed into a single “source of truth” field. An operational source is not
necessarily a single file: the CNBV publishes the DCG-IFPE-2021's main body
and its eight annexes as separate PDFs from the same official host, linked
from the instrument's own row on the Normatividad page rather than embedded
in one document. All of them are the operational source; the formal DOF
publication is additionally acquired for promulgation-date provenance, not
because it is the only place the annex text exists. Jointly issued
instruments record every issuing authority explicitly, independent of which
authority hosts the operational file.

Canonical identities use URNs independent of filenames and presentation:

```text
urn:lex-mx:federal:statute:lritf
urn:lex-mx:federal:statute:lritf:article:4
urn:lex-mx:federal:statute:lritf:transitory:primera
urn:lex-mx:federal:regulation:ifpe-dcg-2021
urn:lex-mx:federal:regulation:ifpe-dcg-2021:article:17
urn:lex-mx:federal:regulation:ifpe-dcg-2021:transitory:cuarto
urn:lex-mx:federal:regulation:ifpe-dcg-2021:annex:8
```

Article records preserve complete normalized text at article granularity in the
first vertical slice. Annexes are first-class canonical provisions with their
complete content. Heading context is structural metadata — title, chapter, and,
where an instrument uses them, section and apartado — and is never inserted
into source text. Reform annotations are retained within the article text until
a later, fixture-driven model can separate them without fidelity loss.

Temporal conclusions distinguish their basis (`source_text`,
`deterministic_rule`, `llm_inference`, or `lawyer_verified`). Unreviewed model
output is never represented as verified legal advice.

Express article references are canonical graph edges stored separately from
provision text. Each edge records the source provision, exact source span and
Unicode character offsets, target instrument and provision, qualifiers,
resolution status, confidence, and `express_cross_reference` basis. Range
expansions may create non-rendered graph edges for intermediate targets. An
edge may target another instrument in the corpus when the source cites it by
its configured official name; citations of an instrument's official title
itself (for example, the statutory-basis articles named in the DCG's title)
anchor to the instrument identifier with spans validated against the official
title. Edges are directed; reverse navigation exists only as a presentation
feature.

Expressly defined terms are canonical records anchored to the exact span of
their definition entry in the instrument's glossary provision (LRITF Article
4, DCG Article 1). A glossary may be expressly additive to another
instrument's — the DCG defines its terms "además de los términos utilizados
en la Ley…" — so a usage resolves against the instrument's own glossary
first, then the glossaries it is additive to. Every exact usage occurrence
(including deterministic singular/plural variants) is a canonical
`term_usage` fact with its exact span; export renders the first usage per
provision as a link to the definition's block anchor.

Validation requires each source span to match the unchanged canonical text and
each resolved target to exist in its instrument's loaded corpus. Markdown and
Obsidian links are injected only during export. Named external-instrument
references and citation-style uses of `la Ley` as a bare shorthand remain
plain text until the referenced instrument is in the corpus or the shorthand
is expressly defined, preventing broken or falsely resolved links.
