# Heading-leak review and parser/debt repair — 2026-10-21

Current acceptance: commit5eca03bc5 and both permanent builds are accepted;
zero stale inputs remain. See [permanent acceptance](heading-leak-permanent-acceptance-2026-10-21.md).
The candidate snapshot and pending statements below record the earlier stage.

Reviewed incoming branch `parser/heading-leak-refetch-2026-10-21` at
`9a2df2529`, including `cd6cd2245`, against pushed `e718da2e5`.
Operator requests technical fixes; no new branch commit/merge/push authorized.
This report is written in the repository, not a hidden worktree.

## Repairs made by Codex

The upstream LOCG, LFCPQ and LSAR fixtures explain their intended changes.
Two predicates needed narrower source ownership:

- The LSAR wrapper previously discarded any single sentence beginning
  `De la Ley …`, including `De la Ley anterior se derogan los artículos 3 y 4.`
  The existing negative test had a second sentence, so it missed this case.
  Now only `De la <configured full law title>` matches. A source fixture and
  actual parse regression preserve the operative single-sentence example;
  the LSAR positive fixture remains supported.
- A post-signature added-transitory heading now must name the configured law.
  A heading for another law cannot reopen this instrument's transitories.
  Wrong-owner/absent-owner negatives supplement the existing LOCG fixture.

The ACUERDO rule remains narrow (uppercase act label followed by lowercase
connector in reform evidence). Singleton and substantive-annex signature paths
retain their existing guards. No official source text or canonical corpus was
edited by these two repairs. Source/provision hashes and publication provenance
remain distinct from parser behavior.

## Retirement format chosen and implemented

Use **seed version 3 with separate retirement history**, preserving the October
18 decision and October 19 acceptance ledger byte-for-byte. No unvalidated
`resolved` review status is added. No historical acceptance is deleted or
reopened, and no intermediate ledger commit is required.

| Accepted partition | Count |
| --- | ---: |
| Active SQL debt rows | 14 |
| Retired, with original accepted row retained | 3 |
| Original accepted authority set | 17 |

Retire LGS 60 Decies, 192 Quintus and 192 Sextus. Every retirement pins the
resolving ancestor `cd6cd22451437322287b974c23349b6d6ca9a33a`, exact standalone
ID, source blob and SHA. Builder and independent verifier separately check:
original accepted metadata/identity/span, exact active+retired partition,
ancestor relationships, absence at acceptance, presence/owner/type/number/body
at resolution and current input, and no old embedded heading left in the old
container. Duplicate, dropped, promoted and forged records fail closed. A later
missing target invalidates retirement. V2 seeds keep their exact-active-set rule.

Schema, Rust types, builder validator, independent verifier and negative
fixtures change together. Active DB debt excludes retired rows; raw pinned seed
bytes retain all retirement evidence. The SQL layout/manifest remain version 2;
changed seed/schema/builder/input bytes give a new release, never a rewrite of
historical artifacts. `docs/decisions.md` appends this technical decision.

## Source and temporal audit

Independently checked all **39** reparse source PDFs and extracted texts against
unchanged manifests. No provision ID was lost; **12** recovered IDs are exactly
three LGS articles and nine LISSSTE transitories. LRITF and every review-queue
file are unchanged. No lawyer-reviewed provision was changed.

Only seven common provisions change temporal fields, solely `effective_from`
from null; temporal status, basis, confidence and review status are unchanged:

| Provision | Machine-derived date |
| --- | --- |
| LCM 244 | 2000-05-13 |
| LFDA 114 and 156 | 1997-03-24 |
| Reglamento de Diputados 284 | 2011-01-01 |
| Reglamento de Aviación Civil 92 and 97 | 1998-12-08 |
| Reglamento LAPP 15 | 2012-11-06 |

Inspected old/new source tails: each removed amendment annotation belongs to
the next chapter, title or section, not the article. These outcomes are
consistent with the existing deterministic commencement rule after removing
that misattributed amendment evidence. This is no new JRH legal determination
or independent reconstruction of all historical amendments.

The upstream 353-source A/B census is reported by Claude; Codex did not repeat
all 353 parses. Independent retained-source E2E stages on the affected three
instruments and full DB source-parity checks are recorded below.

## Validation and artifacts

Native gate: fmt, strict workspace Clippy, **419 tests**, CLI build pass.
Private parse/derive/validate/export checks pass for LOCG, LFCPQ and LSAR,
with zero canonical/output drift. LSAR is valid with five unchanged warnings
(one suffix ordering and four reviewed historical references); the others
have zero issues. Required LRITF/IFPE validators pass with zero issues.

Two fresh builds, independent verifiers and **21/21 goldens each pass**.
Both database files are byte-identical: 482,025,472 bytes, SHA-256
`f1c8626f23123bfee54d1553245bac4e86f0e4699e22524ecfe2e7691ac151bc`.
Census: **468 instruments, 68,341 provisions, 35,930 reference edges, 5,766
defined terms, 97,672 term usages and 14 active debt rows**. The raw seed
preserves three retirements. Independent verification covers 4,960 inputs,
208,744 normalized records and 10,002,069 FTS token occurrences.
Receipts and source audit: [evidence](heading-leak-refetch-review-and-repair-2026-10-21-evidence.json).

Test snapshot: `46be511cebcd374c5befb561d98ffa4ab4205f2e`, an unreferenced
Git object with parent `9a2df2529`. It includes only the named repair paths,
not the operator's untracked suggestion/prompt files. Source code is frozen at
this snapshot; reports/plan updates afterward are navigation only. The branch
HEAD remains unchanged, so the three changed selected input paths are
reported stale against HEAD; they must clear after an authorized permanent
commit/rebuild. This is not a production-release or committed-builder claim.

## Holds and next action

LRITF remains untouched for JRH. The newer cub-dcg-2005, reg-laero and
socap-sofipo-dcg-2006 editions remain separate re-ingestion work. Remaining
heading forms in the held DCGs/LGOAAC/LAN/LFPED are not silently normalized.
No legal call is required for the technical repairs and debt retirement above.
Any ambiguous new source reading remains for operator review.

Validation is complete. Next: review these repairs, then an operator-authorized
commit and rebuild against the permanent commit to clear the three expected
stale paths (seed, decisions and seed schema).
Preserve resolving/source ancestry through normal merges; do not squash away
the retirement's resolving commit. No merge, push or replacement of existing
published local DB releases is performed here.
