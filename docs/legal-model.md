# Legal model

The Cámara de Diputados consolidated PDF is the operational source for current
federal statutory text. The Diario Oficial de la Federación is the formal
publication source. These roles are represented separately and must not be
collapsed into a single “source of truth” field. A regulation may require the
formal DOF publication as a second acquired source when the operational file
is incomplete: the CNBV PDF of the DCG-IFPE-2021 omits its annex bodies, which
exist only in the DOF note. Jointly issued instruments record every issuing
authority explicitly, independent of which authority hosts the operational
file.

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

Validation requires each source span to match the unchanged canonical text and
each resolved target to exist in its instrument's loaded corpus. Markdown and
Obsidian links are injected only during export. Named external-instrument
references and short-form defined-term citations (such as the DCG's `la Ley`)
remain plain text until the referenced instrument or a defined-term layer is
part of the corpus, preventing broken or falsely resolved links.
