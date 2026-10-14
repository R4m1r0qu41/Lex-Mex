# Reader design and EC2 recovery — 2026-10-08

## Delivered design

[Separate Lex-Mex Figma file](https://www.figma.com/design/ar2bES9p6jeUBY0w3E4eDY)
contains editable desktop reader (`2:41`), mobile reader (`2:121`), mobile source
view (`2:148`), consultation/error states (`2:166`), reusable components (`2:25`),
and implementation notes (`2:183`). Mobile source/back navigation is wired.

The design contains 7 main components, 25 instances, 16 scoped variables and
6 text styles. A structural readback found no image-filled UI nodes. Desktop,
mobile, source and error compositions were visually inspected; mobile wordmark
and instance-label wrapping were corrected. Fonts were explicitly read back as
SF Pro and Source Serif 4. Georgia is unavailable in this Figma environment;
Source Serif 4 is a documented design substitution, not a production CSS edit.

This is an editable design proposal based on the current reader. It adds a
dedicated mobile source view and a compact desktop source access panel; it is
not a pixel-identical import of the embedded browser PDF viewer. The LRITF
Article 1 text is verbatim from the committed corpus. No PDF page mapping,
upstream freshness, reviewed legal status or expanded publication is invented.
The LEG4L L4BS design file and reader source were preserved.

## Intake outcome

Main remains at `e3dfcbd89` with 243 instruments. All 243 stored validation
reports are valid; this was a report inventory, not 243 fresh validator runs.
Live LRITF and IFPE validations passed. Reader tests passed 4/4 after allowing
the preview test to bind its local socket.

Eight new EC2 candidates are in `/private/tmp/lex-mex-intake-20261008` on
`feat/intake-ec2-20261008`, based on `e3dfcbd89`:

| Slug | Articles | Original transitories | References |
|---|---:|---:|---:|
| reg-lcec | 50 | 2 | 4 |
| ldcmpme | 26 | 6 | 6 |
| reg-ldcmpme | 37 | 2 | 9 |
| lffmaa | 45 | 3 | 5 |
| liispcen | 21 | 7 | 20 |
| lnetb | 114 | 17 | 8 |
| lfcc | 17 | 3 | 5 |
| reg-lfmn | 118 | 4 | 16 |
| **Total** | **428** | **44** | **73** |

The initial sandboxed fetch failed DNS; the network-enabled retry acquired the
official PDFs. Seven passed initially. LFFMAA failed ordering because its first
nine labels use `1o.` through `9o.`. All 45 source headings matched the parsed
sequence. The existing reviewed ordinal-label convention was applied through
`allow_article_gaps: true` in that new adapter; no parser code changed.

All eight subsequently passed `link`, `validate` and Markdown export. Retained
PDF and extracted-text hashes were recomputed and matched all eight manifests.
Independent source-heading counts and numeric sequences matched each candidate,
including the two Bis articles in reg-lfmn. Initial identity, terminal article
text and original transitory labels were inspected. Details and hashes are in
[`ec2-intake-audit-2026-10-08.json`](ec2-intake-audit-2026-10-08.json).

Counts remain unfrozen: 16 count-baseline warnings plus two reg-lfmn suffix
warnings remain. All 472 provision temporal statuses are `unknown`; no temporal
analysis ran and no review queue was created. These candidates are not legally
reviewed, integrated, committed or published. Complete original-transitory and
PDF boundary review before freezing counts. Reg-lfmn's historical parent-law
context still requires separate legal review; acquisition does not settle it.
The old batch report retains the initial LFFMAA failure; the later per-instrument
validation reports and this audit record the successful narrow recovery.

## Commands and checks actually run

- `CARGO_TARGET_DIR=/private/tmp/lex-mex-target-20261008 cargo build --locked -p lex-cli`
  passed. The binary is named `lex-mex`, not `lex-cli`; an initial invocation of
  the latter path failed before any validation and was corrected.
- `lex-mex --root /Users/jr/Dev/lex-mex validate lritf` and
  `validate ifpe-dcg-2021` passed.
- `node --test reader/reader.test.mjs` in the reader worktree passed 4/4.
- `lex-mex --root /private/tmp/lex-mex-intake-20261008 batch run
  /private/tmp/lex-mex-intake-20261008/batches/economy_EC2_empresas_mipyme.json
  --only reg-lcec,ldcmpme,reg-ldcmpme,lffmaa,liispcen,lnetb,lfcc,reg-lfmn --keep-work`
  performed acquisition through export; initial LFFMAA failure recovered as above.
- For each of those eight slugs: `link <slug>`, `validate <slug>` and
  `export <slug> --format markdown` with the same `--root` passed. External
  Obsidian export was disabled by unsetting `LEX_MEX_OBSIDIAN_VAULT`.
- `cargo fmt --check` and `git diff --check` passed. Full Clippy/workspace tests
  were not run; no code changed or commit was made.

## Independent work and next action

[`dots-independent-intake-workflow.md`](dots-independent-intake-workflow.md)
provides complete role prompts, job cards, isolated writer rules, source gates,
review packets and cutoff recovery. It is a draft, not an activated schedule.

Next: independently review the eight EC2 candidates' remaining source boundaries
and warnings, then freeze reviewed counts and prepare exact integration changes.
EC3 source research can proceed independently from its prepared inventory.
