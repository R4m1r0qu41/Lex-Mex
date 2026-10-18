# Merged-heading split

## Purpose and outcome

Some article headings are printed inside another article's text instead of
starting their own provision, for example `Artículo 15 Sextus.-` inside
`lfped` 15 Quintus. The operator decided on 2026-10-18 that these headings
are split into their own articles before any change to citation extraction
(`docs/decisions.md`). The observable outcome:

- every accepted merged heading in the federal laws and regulations is its
  own canonical provision;
- the containing article keeps only its own text;
- the false self-link edges disappear;
- validation, the closure gate and Markdown stay consistent.

**In scope:** the 17 law and regulation instruments with accepted rows in
`docs/sqlite-structural-debt-review-2026-10-17.json` (on
`db/sqlite-projection`), plus any further heading the reparse itself finds in
those instruments.

**Out of scope:**
- the bracketed placeholders (`lgipe`, `lgcs`) and the LFTR 180 Quintes–Septimus
  rows, which are legally removed text;
- the held DCGs, which wait for the DCG pass;
- the citation-regex extension, a later step once every split is done;
- temporal analysis and legal review.

## Authority and baseline

- **Authority:** `AGENTS.md`, `docs/decisions.md` (2026-10-17 and 2026-10-18
  entries), Rust parser and validation code, committed corpus.
- **Immutable baseline:** `main` at `7302d5190`. Label grammar ranks Latin
  ordinals beyond `Nonies` (`fe4a53f1e`).
- **Retained source:** `.work/` holds source PDF and extracted text whose
  SHA-256 matches the committed `source_sha256` and `extracted_text_sha256`
  for 13 of the 17 instruments. `lfd`, `lfdo`, `lfpa` and `lgs` have none.

**Probe** (disposable clone, `parse` from retained text, 2026-10-18):

| Group | Instruments | Result |
|---|---|---|
| Clean | `lfped` +7, `lgamvlv` +6, `lgdnna` +1, `lgmdfp` +3, `lgv` +1, `reg-lgcfd` +1, `reg-lsem` +4 | See below. |
| Collateral | `lac`, `laero`, `lfpc`, `lic`, `lgtoc` | Splits found, except `lgtoc`, which needs lowercase-letter headings. Temporal and review fields change on 100–489 provisions each, and so do `heading_context` and the text of unrelated articles. These were parsed in July, so the current parser differs. Not acceptable as a heading split. |
| Fails | `cpf` | `duplicate reform evidence id …cpf:amendment:1965-01-13:section-4:transitory:unico` |
| No source | `lfd`, `lfdo`, `lfpa`, `lgs` | Re-fetching would bring newer consolidated text. |

What "clean" means for the first group:

- the only changes are new provisions, plus truncation of the containing
  article;
- ordering and all other provision fields are unchanged;
- each new article carries its container's fields exactly, except that
  `heading_context` is corrected where the merged text had swallowed a
  chapter heading (three `reg-lsem` articles, one `lgmdfp` article);
- the false self-link edges disappear, one genuine `lfped` edge moves into
  its new article, and `reg-lsem` gains three parent-law edges.

## Milestones

### M1 — Reparse the clean 7

**Verification in a disposable clone, before touching the repository:**

- **Byte accounting.** Each old container text equals the new container text,
  plus each split heading and body, with only separator whitespace between.
- **Every rewritten file is inspected:** terms, term usages, instrument,
  reform evidence and validation.
- **No reviewed or protected state** sits on any affected provision or edge.
- **Count baselines** are re-frozen for these 7 only, citing the operator
  decision.
- **`link --all`** exits 0, and no graph outside the 7 changes.
- **Edges.** The three new `reg-lsem` edges are checked against the text.
- **Markdown** for the 7 is regenerated and inspected.

**Then:** apply the same commands in the repository, run the native gate,
and commit exact paths.

**Recovery:** `git restore` the instrument directories and adapters, because
nothing outside them changes.

**Stop if:** any change appears beyond the splits listed above.

### M2 — The remaining 10 (needs operator approval of the mechanism)

**Proposed mechanism:** a deterministic Rust split of the committed provision
text, using the parser's own heading recognition. A new article inherits its
container's fields, and `heading_context` is recomputed for any chapter
heading inside the split text.

**Acceptance:**
- it reproduces all seven M1 reparses byte for byte;
- `lgtoc` additionally needs lowercase single-letter heading support, with a
  fixture;
- `cpf` needs the duplicate reform-evidence ID understood first, or the
  canonical split avoids reparsing.

**Consequences:** splitting `lic` 112 Quintus and `lgtoc` 228 b/h resolves the
`cpf` and `lmv` closure gaps. The accepted-gaps file and those graphs then
change, as a recorded decision.

### M3 — Citation extraction

Once every split has landed, extend the citation regex to the spellings the
corpus prints (`Sextus`, `Septdecies`, lowercase letters …), so citations
reach the new articles.

**Blocker to raise first:** `lftr` is not split, so its three false
self-links would point at missing targets and reject its graph. This needs
either a heading-line exclusion in extraction or an operator choice.

### M4 — Database seed

Rebase `db/sqlite-projection` onto the result. Regenerate the
structural-debt candidates and review. Rows for split containers no longer
bind, and only the unsplit instruments keep accepted debt.

## Current checkpoint

2026-10-18: M1 landed on `main`. The 7 clean instruments are reparsed and gain
23 articles in total: `lfped` 7, `lgamvlv` 6, `reg-lsem` 4, `lgmdfp` 3, and 1
each for `lgdnna`, `lgv` and `reg-lgcfd`.

- **Recovered text.** Four of these articles had been lost from the committed
  corpus: `lgmdfp` 12 Duodecies and `reg-lsem` 33 Undecies–Terdecies.
- **Edges.** 8 false self-links removed; 3 `reg-lsem` parent-law edges added.
  References go 35,887 → 35,880, resolved cross-instrument 9,763 → 9,766.
- **Baselines.** Count baselines re-frozen for the 7.
- **Closure.** `link --all` exits 0 with 0 relinked and the 8 accepted gaps
  unchanged.
- **Markdown** regenerated for the 7.
- **Probe match.** The repository result is byte-identical to the probe.

M2 is waiting for the operator to approve the mechanism.

## Progress

- 2026-10-18: rulings recorded in `docs/decisions.md`; `lgs` accepted-gap
  reason corrected; probe run.
- 2026-10-18: M1 verification passed in the probe. Byte accounting is exact,
  and the only text between a container and its split articles is the heading
  line, except where text was lost. Instrument, terms, reform evidence and
  validation files are unchanged apart from counts. Term usages keep the same
  (term, span) multiset; only the IDs move. No reviewed state is on an affected
  container, and the held `lgdnna` 135 Ter → 157 edge is unchanged. A
  regression fixture (`article-latin-ordinal-after-chapter-sample.txt`) fails
  under the pre-`fe4a53f1e` grammar with `["33 Nonies", "34"]` and passes now.
  Applied in the repository.

## Discoveries

- **The detector undercounts.** The reparse finds headings that the
  line-start detector missed: `reg-lsem` 33 Undecies–Terdecies and `lgmdfp`
  12 Duodecies.
- **The old parse lost text, not only structure.** In `lgmdfp` and
  `reg-lsem`, everything after a chapter heading that followed an
  unrecognized ordinal heading was missing from the corpus.
- **Chapter headings were merged as well.** Merged text also swallowed
  chapter headings, so the containers carried the wrong `heading_context`
  for the articles that followed.

## Next action

The operator decides the M2 mechanism for the remaining 10. The proposal is
a deterministic Rust split of the committed text, accepted only if it
reproduces the M1 reparses byte for byte.
