# mLey public-surface benchmark — 2026-10-27

Scope: homepage HTML, agent-discovery text and eight publicly delivered JavaScript
modules. No backend repository was located; no claim is made about its database,
internal ingestion rules, test suite or deployment costs. No code is copied into
Lex-Mex. Observations are from reading delivered code, not a completed browser
usability or legal-accuracy audit. A requested article page returned429; it was
not repeatedly retried.

## Patterns worth adopting

| Observed pattern | Public evidence | Implication for Lex-Mex |
| --- | --- | --- |
| Useful HTML plus small canonical article routes | [Homepage](https://mley.mx/) publishes a catalogue, article links such as `/CPF/articulo/11-bis/`, source areas and navigation in HTML. | Make an article directly readable without downloading a corpus or executing the entire reader application. |
| Small agent discovery | [llms.txt](https://mley.mx/llms.txt) points to catalogues and per-instrument Markdown indexes, distinguishes legal text from summaries, and explains version links/cache revalidation. | Give agents a short route to relevant text; measure actual tool reading, not just HTTP200. |
| Unresolved references are usable product states | [pendingReferences module](https://mley.mx/static/ley/js/ley/pendingReferences.ab65fb237ea5.js?v=20261016-integration-focus) presents separate missing-article/missing-law messages and can show a related law without claiming it substitutes for the citation. | Keep reading available when a citation has no verified target. Preserve the citation and its limitation rather than forcing a guessed link or blocking publication. |
| Document-level version binding | [pdfReadersModel](https://mley.mx/static/ley/js/ley/pdfReadersModel.795d7c7726ea.js) validates64hex document versions and constructs version-specific law-PDF viewer routes. | Stable article identity can coexist with version-bound source documents. No inference about the backend hash algorithm is made from this alone. |
| Version-specific PDF positioning and local failure | [viewArticlePdf module](https://mley.mx/static/ley/js/ley/viewArticlePdf20260921.ae79d9af44b6.js?v=20261012-toasts) binds mapping to the law/version, checks page bounds, keeps a small mapping cache, and exposes retry/timeouts when mapping/loading fails. | Failure to locate a PDF page should affect that convenience, while exact text and source access remain available. |
| Separate navigation and state | [readerReferences](https://mley.mx/static/ley/js/ley/readerReferences.f55b24482010.js) constructs canonical article/DOF URLs; [homeAllSearch](https://mley.mx/static/ley/js/ley/homeAllSearch.4c1fe3d75ff6.js?v=20261009-stable-search) maintains query/history state and reports partial progress/cancellation. | Keep content addresses independent of floating-panel state; search failure/progress should be explicit and recoverable. |

There are different meanings of “pin”: the homepage Pin feature saves reading
windows; hashed static asset filenames are cache/version identifiers; PDF version
identifiers bind documents. These should not be treated as proof that the whole
site is rebuilt or that every legal statement is verified against a global hash.
The public [judgment page](https://mley.mx/sentencias/6b702211e5b7e0fc823ee40d7c840069d33f632b424fae0f4892399496b9b581/)
explicitly labels a PDF version asSHA-256, but that does not establish the law
backend's complete provenance scheme.

The homepage also distinguishes record-update dates from legal reform dates and
shows an official-check schedule separately from successful check history. We
should maintain those distinctions in our own freshness model. Displaying a
schedule alone is not evidence that source verification succeeded.

## What this benchmark does not justify

Do not copy the number of overlays/features, assume its links are legally correct,
infer its backend technology from asset paths, or treat frontend source as an
open-source license for reuse. We did not compare measured latency, token use,
full-corpus update time or correctness between the two products.

The useful lesson is functional separation: readable text, optional links,
versioned sources and graceful failure can coexist. Visual polish and exhaustive
reference resolution do not need to be prerequisites for a useful legal reader.

## Evidence retained

Bounded download inventory, URL/SHA/size records, homepage, agent document and
public script copies are temporary research inputs under
`/private/tmp/lex-mley-benchmark-20261027/`. The repository retains this observation
report and a hashed research inventory in the delivery-plan evidence, not vendored
third-party source. Source can change after this observation date.
