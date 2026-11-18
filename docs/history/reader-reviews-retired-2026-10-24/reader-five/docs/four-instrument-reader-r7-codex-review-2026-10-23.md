# Reader R7 — focused Codex review, 2026-10-23

**Verdict: no blocking finding for the four-instrument R7 packages at commit `162128517576b043a389f753578c5cabb4540898`. One nonblocking heading-coverage issue below.** Worktree `lex-mex-reader-five` was clean at review entry. The receipt's uncommitted wording is preparation history; the source is now committed. This review changes no source, pin, host staging, deployment, push or merge.

## P3 — headingPath omits the supported Libro level

`reader/reader-core.mjs:22` lists title/chapter/section/apartado but omits `libro`, which `HeadingContext` supports at `crates/lex-core/src/lib.rs:715`. The previous middle-column rendering used all stored heading values, so a record with Libro loses that context after R7; the new exports also omit it. A direct example with Libro Segundo, Título I and Capítulo I returns only the latter two levels. Repeated title/chapter paths under different books can consequently lose their distinction.

All four preferred-package instruments have zero nonempty libro fields, so this is **nonblocking for the reviewed packages**. Include libro before title in the complete heading path, with a fixture, before extending this Reader to codes that use books. Index title/chapter-only grouping is an explicitly chosen separate policy and is not challenged here.

## Verification performed

- Reviewed the committed UI, helper, CSS, HTML and browser-harness diff against parent F1 `b9b8be012`. The F1 tracking repair remains present. No Rust/schema/corpus or immutable release changes.
- Node tests **24/24 pass**; diff check passes. All seven receipt source SHA-256 bindings match.
- Preferred `85b4ce98757da6c75b859de2d1eaeded369630adbbbe2734fa74caedc42501b1` and fallback `309f8d5f5bccfb097894c1746a91510f2fa031ebdfcf109a8171d7dd3802237e`: descriptor IDs and every listed hash/size valid, A/B pairs identical. Preferred1790 listed/1791 on disk; fallback1784/1785. Only index.html, reader.css, reader.js, reader-core.mjs and publication.json differ from corresponding F1 packages.
- All four recorded browser-report SHA-256 bindings and result totals match: preferred60/0, fallback57/0+3 expected N/A, legacy75/0, no-anchoring60/0. These are verified **implementer reports, not newly independent full-suite runs**.
- Independent rerun of scenarios28 and29 against preferred package: **2/2 pass**, covering citation/export/panel behavior and size toggle/reload at390px. No page errors.
- Additional independent Chrome probes: **3/3 pass**. (1) Actual browser blob tab, restoring native window.open rather than using the harness stub: document content type text/plain, charset UTF-8, body exactly equals canonical193 and opener is null; opened using a CDP user-gesture evaluation. (2) Real fillAll paused at its first zero-delay yield: size toggle keeps193 physically visible/selected/copy-exact during pending fill and on completion; scrolling selects190, then another size toggle keeps190. (3) localStorage getter throws SecurityError: page, size/name toggles and tracking still work without page errors.
- ITF latest_reform_date remains parser-derived: source uses the maximum parsed reform date; R7 does not change or legally verify it. This is not new corroboration against the PDF.

## Decisions and limits

Keep0.65 as the current small-text choice; no correctness rule requires0.5, which would make text still smaller. If the operator intended an exact50% requirement, that remains a presentation preference to resolve explicitly. Per-article Markdown carrying heading path and article label is acceptable for a presentation export; exact Copy and the new text tab still return the unchanged canonical body. Raw committed release Markdown remains separate and unchanged.

The blob workaround fixes the plain-click human path. Raw .txt/.md and modified-click encoding still depend on the host charset headers; this review does not claim a host-side fix or test current production headers. A browser refusing popups can prevent the new tab; fallback behavior was not added in this scope. The10-minute revoke and modified-click conditions were inspected in source, not elapsed-time tested.

No full independent repetition of all four suites, Safari/Firefox, real-device, screenshot or native Rust checks in this UI-only review. The supplied F1 control abort before scenario28 is an accurately disclosed missing-control error, not proof of a user-visible regression; no mutation claim is made from it. The read-only probes leave shipped package bytes unchanged.

Exactly one next action: operator considers the reviewed R7 packages for publication, with host charset follow-up if desired. Any R7 restaging, pin change, host tests, publication or integration requires its own instruction; prior F1 publication actions do not publish R7 automatically. Complete evidence and reproducible independent probe scripts are in the companion `-evidence.json`.
