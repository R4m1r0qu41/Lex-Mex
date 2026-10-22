# Database, Reader and LIC consolidated review — 2026-10-22

Main baseline85c41f389, accepted permanent DB corpus/builder5eca03bc5.
Reader integration: `/Users/jr/Dev/lex-mex-reader-four`, branch
`feat/four-instrument-reader-design`, main85c41f389 plus uncommitted work.
This is a technical review and execution recommendation, not publication
approval or a new legal determination. No implementation, corpus, host or
Reader files were changed by Codex; reports/plan only.

## Database verdict

The local database implementation through M4 is complete and normally merged
and pushed to main. Two accepted immutable releases at5eca03bc5 are reproducible:
release `afdc5acf0b5017d73cd7b8e0b8f8fc5962914b56de26467e42b0c77fd2ef1974`,
physical SHA `5da6e800e4d3b362411c561d9ae67fbfe012abe3164fce30bbb8a21c9e249bf2`,
logical SHA `8cc3736018bffae94f303d9cbfa378170c9ff0e2842ce708431f08d528b60359`.
Census468 instruments,68,341 provisions,35,930 references,5,766 terms,
97,672 usages,14 active debt rows; three retired rows retain accepted history.
The accepted DB already contains511 LIC provisions and all three recovered
112 Quintus/Sextus/Septimus articles. It does not need a LIC ingestion repair.

Prior evidence:419-test main native gate; two full verifier receipts with no
stale inputs;21/21 goldens independently rerun on each. This review re-read
manifests and queried LIC read-only; it did not repeat a full DB build/verifier.
See [permanent acceptance](heading-leak-permanent-acceptance-2026-10-21.md).
Standards remain catalogue-only and SCJN evidence-only. Temporal completeness
and current legal validity are not established by DB inclusion.

The public Reader still consumes a separate allowlisted release. Four selected
instruments do not require Worker/D1, an API, or serving the internal SQLite.
Full-corpus delivery remains a separate public-projection/paged-query project.
Do not make completion of that project a prerequisite for this Reader release.
Existing accepted DB artifacts remain valid pinned releases. Do not rebuild or
replace them merely because UI bytes change. If later integration changes DB
selected schemas/builder dependencies, any new DB release needs its own pinned
manifest/input review; a different raw-input logical digest is not automatically
a corpus regression. Do not claim a new merge-pinned DB without building it.

## Reader verdict

The retained-content design package passes independent integrity checks:
22/22 Node tests; every publication file hash/size; byte-identical ship-a/b;
1,777 unchanged retained release files; only four changed UI files;15/15
source-transfer bindings. Prior424Rust tests and isolated validators are Claude
receipts, not rerun by Codex. The main.rs extraction preserves existing validation
computation; no missing repeal/parent/glossary checks found in focused review.

One source-level blocker needs reproduction/repair: active tracking uses only
IntersectionObserver callbacks, although reading-line crossing need not change
intersection membership. That can leave copy/share/label on the wrong article.
See [Reader review](four-instrument-reader-codex-review-2026-10-22.md) for code
locations, geometry and required regression. Browser tests must exercise the
actual event sequence, not only the pure pickActive function. Codex did not
run a fresh browser session or independently re-evaluate Figma visual fidelity.

## LIC verdict and correction to the blanket hold

The earlier receipt held all LIC public text differences pending review. That
was appropriate while unexplained. The retained-source audit now identifies
all26 differences and verifies the three recovered bodies. **The source-fidelity
technical hold is cleared for these exact bytes.** No new JRH legal ruling is
needed for deterministic structural/formatting corrections already in accepted
main. The user still approves the concrete public release/pin and deployment.
Do not mark a legal-review row approved or assign a commencement date from this
finding. Any further discrepancy reopens the technical review for that item.

The retained PDF and extracted text exactly match committed source hashes:
PDF `98225a423eaa3dc47c9ec4ddb49ef8b188ed7f8e7e9ef17323a1c33e47e4284d`;
text `6ec122ec76d92e48e959d56937117da5c216ae06f420ef74e307cf0db5fe98b5`.
This is a comparison with the already retained official source, not a new
source acquisition or assertion of today's authoritative consolidation.

| Change | Independent finding | Disposition |
| --- | --- | --- |
|15 paragraph/line changes | Whitespace-normalized bodies exactly equal | Accept technical normalization |
|8 leaked chapter/apartado headings | Exact removed text exists in retained source; article body retained | Accept document-boundary correction |
|112 Quáter split | All three new bodies were already inside its old text; separate printed headings exist in source | Accept112 Quintus/Sextus/Septimus as separate provisions |
|108 Bis editorial tail | Missing `Artículo Décimo Primero del citado Decreto.` exists at source lines7478–7481; other change is whitespace | Accept source restoration, not a new interpretation of the2014 decree |
|Vigésimo signature | Substantive body unchanged; removed signature/promulgation block exists after it in retained source | Accept provision-boundary correction; source PDF remains available |
|248 heading contexts |124 chapter changes and124 apartado additions, matching recorded reparse classification | Retain as structural navigation; do not treat as article text |

All shared provision source, temporal basis/status/dates, review status, repeals
and commencement conditions are unchanged. Zero old IDs removed.511 public IDs
and bodies exactly match current committed corpus and the accepted DB. The three
new articles retain machine_accepted/effective classifications with unknown
`effective_from`; show existing qualifications and do not promote to verified.
The original repair landed in9441b7976 (Oct19 sixteen-instrument batch), not the
later39-instrument refetch. The implementation receipt should distinguish them.

Full source-audit evidence, exact changed IDs and removed/restored strings:
[LIC evidence](lic-publication-source-audit-2026-10-22-evidence.json).
Retained-source lines8493 onward print the three article headings/bodies;
lines14450 onward distinguish Vigésimo and the signatures. These source records
are sufficient to assess these textual repairs; no formal promulgation or
commencement ruling is being made here.

References also change: LIC1049→1137, IFPE113→115, ITF144→149; LRITF126 unchanged.
Their current-main validation is already accepted, but Claude must independently
validate public edge spans/targets against the new selected release and expose
outside-selection targets honestly. No silent rewrite to make the old release
appear current. Publication IDs must be rebuilt, never edited manually.

## Recommended implementation outcome

Prepare a preferred new-content package over the verified current-main four-
instrument release (870 provisions: LIC511, IFPE71, ITF132, LRITF156), plus the
retained867-provision package as rollback/fallback. Existing candidate release
9626bfe45c4ebc95b47351fd20c1de86f9b4f54856aee1de6d408e5d8389991f may be reused
only after its exact bytes and compatibility verify. UI repair changes the
publication ID; do not invent a replacement ID in advance.

The active plan is [four-instrument-reader-design-release](plans/four-instrument-reader-design-release.md),
now with review-driven R0–R4 continuation. Sonnet5.5 implements in the existing
isolated worktree; Opus5.5 advises from a fresh bounded review packet. No new
worktree, database milestone or canonical reparse is needed. Final result is
reviewable package(s), corrected evidence and concrete commit/publish proposal.
