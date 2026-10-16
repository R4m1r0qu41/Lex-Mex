# Original held-source reprocessing — 2026-10-15

## Result

Ten of the 14 original holds are admitted locally, taking the working corpus
from 457 committed instruments at `0b944572a25db5fbb499f752bb06a3fec8cc9e93`
to 467. The original 71,343 committed corpus files are byte-identical. The
reprocessing adds 1,014 articles, 88 transitories, and one annex; it preserves
425 references: 424 resolved and one exact, approved historical stale
reference.

The 14 dispositions are: 10 locally admitted, three DCGs deferred for their
dedicated parsing and official-source work, and EGDF excluded by the operator. No temporal
model ran, no reverse relinking of existing corpus material was performed, and
no commit or push occurred.

GPT-6.1-Sol independently accepted the corrected actual outputs. Its review
caught LISIPL running-title contamination in Article 2 and containing-act
QUINTO/wrapper evidence. Exact adapter headers and a source-backed regression
corrected it; private and main parse/link/validate/export reruns passed without
changing retained source bytes or source manifests. The final native gate passed
after the correction and batch inventory reconciliation.

## Admitted candidates

All ten candidates completed their local link, validate, and export stages
with exit code zero. `review_status` remains `not_analyzed`; temporal states remain `unknown`
except existing source-marked repealed RIS provisions. No commencement dates
were inferred; structural admission does not resolve legal effect.

| Instrument | Articles | Transitories | References | Enactment evidence | Reform evidence | Markdown |
|---|---:|---:|---:|---|---:|---:|
| `lga` | 123 | 17 | 20 | — | 8 | 141 |
| `lgag` | 45 | 2 | 2 | 1 elided sibling article; 3 transitories | 0 | 49 |
| `lif-2026` | 34 | 30 | 31 | — | 0 | 65 |
| `pef-2026` | 39 | 16 | 39 | — | 0 | 57 |
| `lftr` | 324 | 0 | 106 | 1 elided sibling article; 45 transitories | 88 | 326 |
| `reg-lgs-insumos` | 255 | 6 | 132 | — | 29 | 262 |
| `lcnbv` | 31 | 11 | 50 | — | 41 | 43 |
| `lcmopfih` | 15 | 5 | 11 | 2 containing articles; 8 transitories | 0 | 22 |
| `lisipl` | 7 | 0 | 3 | 2 containing articles; 7 transitories | 0 | 9 |
| `lfiiedb` | 141 | 1 | 31 | 2 containing articles; 8 transitories | 0 | 144 |

LCNBV retains the exact source span for the citation to LMV Article 16 Bis 7.
It is represented as the approved stale historical reference rather than
silently deleted or linked to a fabricated current provision.

PEF has 38 printed annex sections represented by one canonical annex
container. The extracted table interleaves the printed Article 74 citation
with a value of zero; that citation remains unlinked. No Article 0 target or
invented correction was created.

## Deferred and excluded dispositions

| Instrument | Disposition | Reason |
|---|---|---|
| `consar-cuf-dcg-2018` | Deferred | Official CONSAR compiled source still requires separate acquisition and validation. |
| `cusf-dcg-2014` | Deferred | Official CNSF compiled source still requires separate acquisition and validation. |
| `pld-lic115-dcg-2009` | Deferred | The base 2009 official compiled DCG source remains unresolved; later agreements do not substitute for it. |
| `egdf` | Excluded | Operator decision: exclude from corpus. A retrieved historical DOF page is retained as acquisition evidence only; no further processing is authorized. |

## Source-intake caveats preserved

Official acquisition receipts, headers, timestamps, source hashes, extracted
text hashes, and formal-source chains are retained in
`.work/held-reprocess-intake/` and summarized in the accompanying evidence.
No source snapshot was silently substituted. The earlier unmanifested LFIIEDB
copy remains separately retained for comparison with the newly retained formal
DOF enactment source.

RIS (`reg-lgs-insumos`) uses official Secretaría de Salud/OAG compiled bytes
obtained through the official HTTP endpoint after the HTTPS endpoint failed
certificate validation. The original 1998 and 2021 reform are supported by
retained formal DOF full issues. Failed portal/error responses remain discovery
evidence, not source documents. The OAG PDF's 77 physical pages carry footers
numbered 1 through 77 of 78; its final retained page completes the executive
signature, so the report records a publisher-pagination discrepancy without a
missing-content conclusion.

LFTR uses the Cámara abrogated-law PDF with retained formal DOF enactment and
abrogation pages for provenance. This records historical source evidence; it
does not resolve provision-level legal effects.

## Validation

The native gate passed:

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace` — 265 passed, 0 failed
- `cargo run --locked -p lex-cli -- validate lritf` — valid; 145/11/126; 0 issues
- `cargo run --locked -p lex-cli -- validate ifpe-dcg-2021` — valid; 59/4/113; 0 issues

The reprocessing set has no errors. Its remaining warnings are 20 unfrozen
counts, 42 non-numeric article ordering warnings, and the one approved stale
historical-reference warning. The count warnings are not a substitute for
source review or a temporal determination.

The machine-readable record is
`docs/federal-held-source-reprocessing-evidence-2026-10-15.json`. It records
the source manifests, candidate tree hashes, stage receipts, operator
disposition, and implementation hash set. The operator-annotated source report
was not changed.

The water batch manifest is now `batches/env_water_e3.json`: its filename and
ID agree with the lowercase schema. It replaces `env_E3_water.json` and maps
Aguas to `lgag`, preserving `lga` for Archivos. Historical review packets remain
unchanged. The final gate caught the filename/ID mismatch and the stale batch test
count before completion; the manifest inventory now has 227 distinct slugs.

## Handoff

Unrelated note deletions and operator annotations remain untouched. Changes are
unstaged; HEAD remains the baseline commit, with no new commit or push.
Next action: review and authorize a commit of this accepted repair/admission set.
After integration, separately scope reverse linking across the 467 instruments;
the three deferred DCGs still require dedicated source/parser work.
