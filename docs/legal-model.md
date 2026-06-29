# Legal model

The Cámara de Diputados consolidated PDF is the operational source for current
federal statutory text. The Diario Oficial de la Federación is the formal
publication source. These roles are represented separately and must not be
collapsed into a single “source of truth” field.

Canonical identities use URNs independent of filenames and presentation:

```text
urn:lex-mx:federal:statute:lritf
urn:lex-mx:federal:statute:lritf:article:4
urn:lex-mx:federal:statute:lritf:transitory:primera
```

Article records preserve complete normalized text at article granularity in the
first vertical slice. Heading context is structural metadata and never inserted
into source text. Reform annotations are retained within the article text until
a later, fixture-driven model can separate them without fidelity loss.

Temporal conclusions distinguish their basis (`source_text`,
`deterministic_rule`, `llm_inference`, or `lawyer_verified`). Unreviewed model
output is never represented as verified legal advice.

