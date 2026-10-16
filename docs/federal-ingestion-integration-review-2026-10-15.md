# Federal ingestion integration review — 2026-10-15

## Outcome

The follow-up replay artifacts are internally consistent on the structural
gate: 227/227 frozen cases passed extraction, parsing, validation and export
using the same executable hash. The retained PDF/text pairs and acquisition
identities are unchanged; the audit records no unexplained removed provision
IDs. This establishes parser and serialization readiness only. No candidate
has been admitted to `corpus/`, all candidate review states remain
`not_analyzed`, and the source pass does not establish legal completeness or
resolve temporal questions.

## Reconciled campaign counts

The frozen 332-identity campaign accounting reconciles to 91 already present,
99 originally awaiting inspection, 128 original parser/validation failures,
and 14 held identities. Follow-up replay covers the latter 227 unheld cases:
the 99 provisional cases plus all 128 original failures. The current committed
repository contains 230 ordinary instrument records under `corpus/`; these
counts describe different sets and must not be added together. This repair
made no corpus changes.

The raw campaign inventory has a warning attached to `lfce`, although the
campaign accounting treats it as already present. Counting inventory entries
by warning alone therefore yields 15 warning-bearing rows and 90 present
rows; it does not change the campaign's reconciled 91/99/128/14 dispositions.
The separate `lfiiedb` modeling hold and other intake work remain outside the
227-case replay result.

## Audit review and correction

The follow-up evidence reports 227 unique cases, all passed, all source pairs
preserved, all review states `not_analyzed`, no temporal-model run, and no
canonical admission. Its eight replay shards all exited zero. The previous
`original_report_and_audit_preserved` hash for
`docs/federal-ingestion-failure-repair-report.md` was stale: the committed
report hashes to
`2a541efa961f9ed4b65436f109253b73e1fd12f51b6895ab2fa3a26dd6a80694`.
That field is corrected in the follow-up evidence. The companion repair audit
hash remains
`4b7f3f7977b34c0a56922a659dc3344a335c78d1f736c7458663a885b26a52e8`.
Separately, the annotated campaign input's recorded SHA-256 matches its
preserved file in the evidence worktree.

The audit documents 13 intentional transitory identity moves for REG-LMIGRA
and retention of two elided sibling articles as separate decree evidence.
REG-LMIGRA's 262 article identities remain present. The only changed existing
article body is article 250, which loses only the elided sibling wrapper; its
article body otherwise remains unchanged. No unexplained removals are
recorded. The prior review queue was not copied, and no human review decision
was changed.

## Next work

Proceed in a separate, bounded source-inspection/admission plan. Start from the
227 replay candidates, group them by source and existing-corpus dependencies,
and inspect the official-source manifest, retained bytes, candidate diff,
validation, review state, and exported Markdown for each proposed instrument.
Keep the 14 held campaign identities out. Stop each instrument when source
identity, completeness, attribution, or formal-publication evidence is
uncertain; preserve those cases for operator judgment. Do not run temporal
models or infer legal outcomes during structural admission. This review itself
does not admit candidates or alter canonical corpus data.
