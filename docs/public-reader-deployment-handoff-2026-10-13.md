# Lex-Mex producer sign-off and Sites deployment handoff

Reviewed: 2026-10-13 (America/Mexico_City).

Subsequent UI update: the [brand-refresh handoff](public-reader-brand-refresh-2026-10-13.md)
records a new publication ID and updated generated package. The immutable corpus
release and this content-integrity review are unchanged; use the new handoff's
publication pin when deploying the refreshed HTML/CSS.

## Verdict and receiving boundary

**PASS — the exact pinned four-instrument public content package is ready for
the receiving Sites account to package and deploy. No producer blocker remains.**
This is engineering/publication-integrity sign-off. It is not JRH legal approval,
proof of current legal validity, or an upstream freshness determination.

The deployment input is already present at:

`/Users/jr/Documents/4ll4/Webpage/Lex-Mex/public`

Do not deploy this directory as a separate website. Consume its complete contents
through the existing LEG4L L4BS Sites project, mounted at `/lex-mex/`. The receiving
account owns host preparation, deployment and live acceptance. No push, Sites
upload, deployment, DNS change or email change was performed by this review.

## Exact signed-off identity

| Item | Value |
| --- | --- |
| Producer repository/worktree | `/Users/jr/Dev/lex-mex-reader` |
| Producer branch | `feat/public-reader` |
| Committed producer | `4072dd53baecc7b6e30dde540a7935314b9e8a6f` |
| Corpus baseline recorded in publication | `9e990e09995718a77865993b24c5f8c8e7167b42` |
| Main compared during review | `023ed74518f243b89b056963da27169bcdb024e2` |
| Host integration commit | `3b876338d10755755a5afc58865b90d1f95fc373` |
| Release ID | `3c04016651e2fd4169632756e96ec19136136e1f42c4ed666070f23f20b30660` |
| Publication ID | `d84847a5c49e31df359b19369e322d77bf035474000aba45bb18ff728a9172ed` |
| Manifest SHA-256 | `c2b22653d7a0b68edc1f29d2b2554fc45eb4a595f5be91f0b96868f2128aedc7` |
| Publication schema | `reader-publication-v1` |
| Public URL prefix | `/lex-mex/` |
| Files | 1,783 receipt-listed artifacts plus `publication.json`: 1,784 total |

The existing host `lex-mex-release.json` already matches all five pin fields:
schema version, release ID, publication ID, corpus commit and manifest SHA-256.
The host's staged `public/lex-mex` tree also matches the signed-off publication.
Both complete trees were inventoried: no symlinks or unexpected files were found.
Do not upload `.work`, corpus JSON, private review artifacts, source inventories,
tests, producer code or this handoff as public assets.

| Instrument | Provisions | Express references |
| --- | ---: | ---: |
| IFPE-DCG-2021 | 71 | 113 |
| ITF-DCG-2018 | 132 | 144 |
| LIC | 508 | 1,049 |
| LRITF | 156 | 126 |
| Total | 867 | 1,432 |

There are 32 retained, digest-verified PDFs: four main sources and 28 separately
sourced annexes. Every provision has an available verified backing PDF.

## Why the existing pin remains approved

The four selected `corpus/mx/<slug>/` directories and their four adapters have
no differences between producer HEAD and the main checkpoint above. Every
canonical input digest in the release was independently compared with the bytes
at the recorded corpus baseline, producer HEAD and current main. All match.
Later main work adds unrelated corpus/batch coverage; it does not require a new
first-release snapshot or integration of that work into this reader branch.

A fresh end-to-end Rust build from committed producer HEAD passed its structural
and strict public-schema gates. Every release-content file and every instrument
descriptor matched the existing pin byte for byte. Its diagnostic release ID is
`70beb70b64cb648c224ed3a156366f1420314dc9ed1633183208803dd2d00381` because Git
provenance now records the committed producer HEAD rather than the old corpus
baseline. That diagnostic output is **not** the deployment candidate and has not
been staged into the external publication directory.

Repackaging the original retained release with the committed reader assets
produced the exact approved publication ID above. The signed-off publication was
staged into a fresh local destination and staged again successfully. Preserve the
existing pin and immutable URLs; no new pin is necessary for this release.

## Verification performed in this review

All commands ran against the producer worktree unless an absolute host path is
shown. No producer source or canonical data fixes were needed.

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run --locked -p lex-cli -- validate lritf
cargo run --locked -p lex-cli -- validate ifpe-dcg-2021
cargo run --locked -p lex-cli -- validate itf-dcg-2018
cargo run --locked -p lex-cli -- validate lic
node --test reader/reader.test.mjs reader/package.test.mjs reader/stage.test.mjs
```

Results: fmt and strict Clippy passed; 219 Rust tests passed; all four structural
validators returned valid; 11 Node tests passed. The first Node attempt was
blocked by sandbox loopback binding (`EPERM`); the authorized local-loopback retry
passed. ITF retains two pre-existing `non_numeric_article` ordering warnings for
`article:15-bis` and `article:15-bis-1`; there are no validation errors. This review
does not alter those records or infer a legal-review resolution.

The fresh end-to-end build used:

```sh
cargo run --locked -p lex-cli -- public-release build \
  --instrument lritf,ifpe-dcg-2021,itf-dcg-2018,lic \
  --source-store .work/reader-sources \
  --output .work/deployment-review-20261013/rebuilt-release \
  --base-path /lex-mex/
node reader/package.mjs --release .work/public-release-v2 \
  --output .work/deployment-review-20261013/repacked-publication
```

An independent scripted audit checked all receipt hashes and sizes, receipt and
manifest content IDs, builder digest, complete file inventories, current reader
assets, canonical-input digests, all 867 exact `.txt` bodies, canonical temporal
and review classifications, Markdown text/source provenance, all 32 backing PDF
bytes, and annex source identities. Newly schema-validated instrument output
equals the existing pinned output, including its strict public allowlist.
Reviewer identities, rationales, condition evidence, private queues and model
effect proposals are excluded by that boundary.

Installed Google Chrome 155.0.8059.40 passed 11 behavioral/HTTP checks: pinned
deep link and exact rendering; background text without changing the visible
selection; full PDF download/hash and byte range; in-browser PDF verification;
same-source navigation/back; reference comparison retaining the primary view;
explicit target selection; separately sourced annex PDF switching; suffix search;
390px layout without horizontal overflow; no-JavaScript discovery/static indexes.
There were no page errors or failed HTTP responses. Desktop/mobile screenshots
were inspected. The headless run checks PDF responses, iframe URL and digest; it
does not qualify native PDF viewer interaction or Codex/Claude desktop embedding.

Read-only host checks also passed:

```sh
node /Users/jr/Documents/4ll4/Webpage/LEG4L_L4BS/scripts/verify-lex-mex.mjs
node --test /Users/jr/Documents/4ll4/Webpage/LEG4L_L4BS/tests/rendered-html.test.mjs
```

The verifier checked 1,783 artifacts; all six rendered tests passed against the
existing host distribution. **A fresh host build was not run in this producer
workspace.** It remains a receiving-account release check, not evidence supplied
by those existing-distribution tests.

Temporary probes, JSON evidence and screenshots are in ignored
`.work/deployment-review-20261013/`. Durable conclusions and exact identities are
recorded here; the receiver does not need that working directory to deploy.

## Receiving-account release sequence

1. Use the registered `jrhernaiz` account owning the existing Sites project. Open
   `/Users/jr/Documents/4ll4/Webpage/LEG4L_L4BS` and read its `AGENTS.md`, current
   `HANDOFF.md`, `README.md`, `LAUNCH-CHECKLIST.md` and applicable Sites skill.
   Inspect current Git state and preserve unrelated parent-repository changes.
   The current local handoff records Sites version 18 as the prior production
   release; verify the actual project version before using it for rollback.
2. Recheck the producer and host pins above. Stage the entire publication through
   the producer stager, rather than copying selected PDFs or committing ignored
   generated files:

   ```sh
   cd /Users/jr/Dev/lex-mex-reader
   node reader/stage.mjs \
     --publication /Users/jr/Documents/4ll4/Webpage/Lex-Mex/public \
     --destination /Users/jr/Documents/4ll4/Webpage/LEG4L_L4BS/public/lex-mex
   cd /Users/jr/Documents/4ll4/Webpage/LEG4L_L4BS
   node scripts/verify-lex-mex.mjs
   ```

3. Complete the host discovery detail: add
   `https://www.leg4ll4bs.com/lex-mex/` to the existing site `public/sitemap.xml`,
   preserving its Spanish/English entries. The current sitemap contains only
   `/` and `/en`; reader routing, navigation and root `llms.txt` integration already
   exist. Run the required fresh `npm test` after that site edit. Do not claim the
   producer sign-off covers a subsequently changed host build.
4. Build/package the whole host through its existing Sites workflow, including
   the generated `public/lex-mex` artifacts in the release output. Git source
   uploads alone omit those ignored assets. Confirm build output contains the
   pinned manifest, all declared artifacts, receipt and reader assets. Follow the
   site's actual root hosting-manifest/build staging convention; do not create
   an alternative hosting provider or deploy this folder independently.
   The current Sites skill uses its source workflow helper for opening and
   packaging the existing project, and `build-site.mjs` for framework builds.
   Preserve the existing project ID and public audience; use a fresh credential
   through hidden stdin only. Pass the helper's verified project ID, commit SHA
   and unchanged archive to `save_site_version`, then deploy that saved version
   with `deploy_site_version`. Never invoke the helper on the broad dirty `4ll4`
   parent as though all of it were website source; follow its selected Sites
   checkout and the existing sanitized host/root staging boundary. Read the
   [Sites skill](/Users/jr/.codex-accounts/pop/plugins/cache/openai-curated-remote/sites/1.0.1/skills/sites/SKILL.md)
   in the receiving account and use its installed helper paths.
5. Deploy the prepared host release under the operator's authorization. This
   handoff supplies producer sign-off; it does not perform the external action.
6. Perform the live acceptance checks below before declaring the reader live.

For an independent local verification of the publication before host staging,
use `reader/stage.mjs` with a fresh temporary destination. It checks the public
allowlist, symlinks, hashes, sizes, manifest and receipt identity before writing.
Compare its reported IDs with this handoff; stop on any mismatch. Existing pinned
release directories must remain available if a later publication is staged.

## Live acceptance and recovery

Check the site's canonical apex/www behavior using the existing domain policy;
no DNS change is expected. Verify home `/`, English `/en`, legacy `/es` redirect,
the reader navigation link and sitemap retain their intended behavior.

Verify `/lex-mex/` reaches the reader and that its redirect to
`/lex-mex/index.html` preserves `release`, `instrument` and `provision` query
parameters. Load this exact first-article link:

```text
https://www.leg4ll4bs.com/lex-mex/?release=3c04016651e2fd4169632756e96ec19136136e1f42c4ed666070f23f20b30660&instrument=lritf&provision=urn%3Alex-mx%3Afederal%3Astatute%3Alritf%3Aarticle%3A1
```

Check `/lex-mex/latest.json`, `/lex-mex/publication.json`, `/lex-mex/llms.txt`
and the pinned release's `manifest.json`, `index.md`, `coverage.json` and one
instrument index. Confirm deployed release/publication IDs and manifest hash
match this handoff. Fetch an article `.txt` and compare its bytes; follow a
reference and an annex; open a retained PDF and use its hash-verification button.

The host must serve `.mjs`/`.js` as JavaScript, `.json` as JSON, `.md` as Markdown,
`.txt` as plain text and `.pdf` as PDF, without treating static files as HTML
fallback routes. Verify PDF download and a valid byte-range request (`206` and
correct `Content-Range`). Verify the inline PDF visually in a normal browser and
the reader at 390px. Root pointers should revalidate; immutable release paths
should retain their bytes and may use long-lived immutable caching. Neither
source retrieval dates nor cache headers prove upstream freshness.

If packaging, pins, hashes, routes or MIME handling fail, stop promotion and keep
the prior production version active. If live acceptance fails after publication,
restore the verified prior complete Sites release. Preserve this local candidate
and its immutable release paths for diagnosis; do not patch generated article
text or PDFs to repair host problems. Record the deployed Sites version and live
test results in the website handoff after successful acceptance.

## Explicit limits carried into publication

Coverage is the named four-instrument snapshot, not the entire repository.
Historical validity is not established by structural validation. The upstream
freshness ledger is unrecorded, PDF page mappings are unset, and SQLite/MCP,
WebMCP and desktop-app qualification are later work. These limitations remain
visible in the package and do not block this static source-backed first release.
