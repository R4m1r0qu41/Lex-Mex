# Held-instrument re-confirmation 2026-11-12

Reviewer: JRH (operator and legal reviewer of record). Decision given in session on 2026-11-12.

The current parser separates fractions that the earlier parse had run together. In `lic`,
`lfpccs`, `lfvv` and `lieps`, every changed provision differs only in whitespace. No words
changed and no provision id was added or removed. The scratch reparse showed the counts below.

| Instrument | Provisions changed | Guard it trips |
|---|---|---|
| `lic` | 39 | stale-citation approval and legal note on art. 194 (LFCE 31 Bis) |
| `lfpccs` | 8 | glossary profile, art. 3 |
| `lfvv` | 5 | glossary profile, art. 2 |
| `lieps` | 4 | glossary profile, art. 3 |

JRH re-confirms the same decisions against the re-spaced text:

- the glossary exception profiles of 2026-10-19, with the same fractions, dispositions and
  terms;
- the `lic` stale citations of 2026-10-16, with the same targets, rationale and notes.

Only `provision_text_sha256` / `source_text_sha256` and the term `start_char` offsets are
refreshed. The offsets are remapped by aligning the non-whitespace characters.

Earlier records: `docs/held-instruments-review-2026-10-16.md` and `docs/decisions.md`
(2026-10-19, glossary exception profiles). Plan: `docs/plans/four-goal-system-upgrade.md`
("JRH decisions 2026-11-12").
