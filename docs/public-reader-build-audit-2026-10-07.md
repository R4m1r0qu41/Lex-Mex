# Public reader build audit —2026-10-07

The isolated reader branch implements a public release producer, a functioning
Spanish reader and local LEG4L L4BS integration. It does not change parser rules,
canonical corpus, legal decisions, production deployment or Git history.

## Trust boundary

Rust rereads and validates selected committed corpora before export. The public
schema explicitly allows source metadata, provision text, temporal classifications,
express references and public condition fields. Reviewer identities, rationales,
condition evidence, review queues and transitory effect proposals are excluded.
Exact .txt bodies retain canonical bytes; presentation references exist only in
Markdown and browser views. Hash-addressed retained PDFs must match their source
manifest. Annex source URLs map through adapter annex order and exact manifest URL,
with missing/ambiguous mappings rejected. Each annex exposes its own source.

A release manifest records corpus commit, builder digest, canonical input digests,
source provenance, partial coverage and every artifact digest/size. Deterministic
builds produce the same release identifier. Outputs are promoted from staging and
never overwrite an existing destination. Node packaging validates the manifest ID
and allowlisted file digests before adding the reader assets and publication receipt.

## Reader behavior

Three panes provide provision navigation/search, unchanged source text and the
retained official PDF. Express references open a comparison panel while preserving
the primary selection and PDF; explicitly selecting a target can replace it.
Shared URLs pin release/instrument/provision. .md/.txt links, alternate metadata
and llms.txt expose the same release to independent background model retrieval.
Article navigation within a source preserves PDF position. Selecting an annex
loads its own PDF. No unverified article-to-PDF page mapping is invented.

The source panel separates retrieval date from upstream freshness and labels the
latter unrecorded. Classifications remain qualified. Partial coverage is visible.
No browser tool API, MCP server or automatic freshness monitor exists yet.

## Evidence and limitations

225 Rust tests plus doc-tests, strict Clippy/fmt, all four validators, seven Node
tests and six host rendered tests passed. Release reproducibility and all867 exact
bodies/32 retained source PDFs were checked. Chrome rendered the actual PDF viewer;
background exact text, digest verification, reference comparison, annex switching,
search and390px layout were verified. Local host delivered .md/.txt/PDF with correct
MIME types and preserved Spanish/English routes and legacy redirect. ITF has existing
non-numeric ordering warnings for15-bis and15-bis-1, both disclosed; no errors.
Actual Codex/Claude desktop embedding is not qualified by these Chrome checks.

Publication directory: `/Users/jr/Documents/4ll4/Webpage/Lex-Mex/public`.
Reader source: `/Users/jr/Dev/lex-mex-reader/reader`.
Manifest and publication IDs are recorded in the task plan and publication.json.
Generated site copy is under `LEG4L_L4BS/public/lex-mex`; host source edits consist
of a route, one navigation link, an llms.txt entry and their regression test.
No commit, push or deployment. Main checkout was preserved for concurrent parser
work. Existing source snapshots cannot establish current or historical legal validity.

## Recovery

Keep generated artifact and source changes separate when committing. Rebuild from
the pinned reader checkout and verified source store into a new output directory;
verify publication.json before staging into the website. Restore the prior static
package as a unit if a local candidate fails. Production uses the established site
release workflow; do not deploy this folder independently without host packaging.

## Follow-up staging verification

User authorized local source commits. A verified publication stager now handles
repeat updates without discarding older pinned releases. Four regression tests
cover old-release preservation/idempotence, immutable collision rejection before
advancing the pointer, symlink/tamper rejection, and concurrent stage locking.
All11Node tests pass. Host prebuild verifies1783files against a committed release
pin; all6rendered tests and build pass. Generated files are ignored in Git.
The reviewed package was restaged into the actual host directory successfully.
No new Rust/canonical changes; previous required Rust checks remain applicable.
No push/deployment. Commit identities are recorded in the session receipt.
