# LIEPS reform-appendix grouping fixture

`lieps-reform-appendix-grouping-sample.txt` is a bounded excerpt of the retained LIEPS
extracted text, made for the transitory-grouping repair (`lieps` held since 2026-11-12). The
lines are verbatim, page furniture included. Some operative and transitory passages are left
out, for example 2016 transitories Segundo, Tercero and Quinto to Vigésimo Segundo, and
fraction I of 2016 Único. No line is altered or reconstructed. The fixture is not admitted
to the canonical corpus.

| Field | Value |
|---|---|
| Source slug | `lieps` |
| Source PDF SHA-256 | `32639ede8448f91e6ddde556d4b8bceaf349b904bc21631f06e1c8a7a3d416f4` |
| Extracted-text SHA-256 | `c4b6dac008c1dfa3cf9fd92a232ac948ad31e158d1a1304e4e81ca33832c8736` |
| Fixture SHA-256 | `b8fdc3247363ba0c5906a1bfa5baefeed0889dfd7f11fafa117d0e343573ab83` |
| Source lines | 3172 (banner); 3908–3913, 3988–4022 (DOF 31-12-1988 and its erratum); 5338–5367, 5664–5667 (DOF 01-01-2002); 7280–7316, 7578–7605 (DOF 15-11-2016) |
| Decree list compared | `ref/lieps.htm`, SHA-256 `84e10255fd7476ef21d86f51ec60d9bae77fb92bd65376caf14aa9af28b91062` |

Each shape matches one defect found by comparing the parse with the Diputados decree list:

1. An erratum title printed after a page break merged into the 1988 `ARTICULO UNICO`, so the
   transitory was dropped as a new act.
2. The heading forms `Disposiciones Transitorias de la Ley …` and `Transitorios de la Ley …` were
   not read as section headers, so the 2002 and 2016 transitories were never isolated.
3. A wrapped line beginning `Decreto por el que` inside transitory Cuarto was read as a new
   decree title. The date it cites (20-12-2013) then replaced the real one (15-11-2016).
