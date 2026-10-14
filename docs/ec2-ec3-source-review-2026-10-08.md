# EC2 closure and EC3 intake source review

Project day: 2026-10-08 (America/Mexico_City). Worktree:
`/private/tmp/lex-mex-intake-20261008`, base `e3dfcbd89`.

## Outcome

Fourteen new instruments are structurally reviewed with audited count baselines:
940 articles, 68 original transitories and 192 references. All fourteen pass
validation and Markdown export. No canonical changes have been copied to main,
committed or published. Main's existing 243 instruments remain untouched.

The exact integration allowlist contains 1,149 files with SHA-256 digests:
[`ec2-ec3-integration-packet-2026-10-08.json`](ec2-ec3-integration-packet-2026-10-08.json).
It includes only the fourteen adapters/corpora and the operational EC3 manifest.
It excludes downloads, ignored work products and LFJS. Before applying, verify
both HEADs, all source digests and absence of destination files. Preserve existing
dirty documentation. Recompute affected reverse links separately after admission;
this report does not claim corpus-wide graph closure.

## EC2: reviewed and frozen

| Slug | Articles | Original transitories | References | Warnings |
|---|---:|---:|---:|---|
| reg-lcec | 50 | 2 | 4 | None |
| ldcmpme | 26 | 6 | 5 | None |
| reg-ldcmpme | 37 | 2 | 9 | None |
| lffmaa | 45 | 3 | 4 | None |
| liispcen | 21 | 7 | 19 | None |
| lnetb | 114 | 17 | 7 | None |
| lfcc | 17 | 3 | 4 | None |
| reg-lfmn | 118 | 4 | 16 | 50 Bis and 82 Bis ordering notices; source sequence verified |

Passing structural validation initially concealed source-boundary contamination.
The final original transitories of LDCMPME, LFFMAA, LIISPCEN, LNETB and LFCC
contained enactment signatures and promulgation apparatus. Exact dated signature
stop markers, using the repository's existing adapter mechanism, now exclude that
apparatus. LIISPCEN also needed its two exact wrapped running-header lines in
`header_lines`; eight earlier provisions and the final transitory were affected.

Comparison with the retained pre-repair candidates established that provision
counts, identifiers and every non-text field remained unchanged. Text differences
consisted only of those exact headers and signature tails. Five Article 89
references generated from the signatures disappeared: EC2 has 68 references,
down from the provisional 73. No Rust parser rule or source PDF/text file changed.
Existing stop-marker and wrapped-header regression tests passed in the full suite.

Every one of the 44 original transitory bodies matched an independently split
source-text section after exact page-furniture removal and whitespace collapsing.
Article-heading counts/sequences matched all eight instruments, including Bis
labels. Publication metadata matched the source's initial publication line.
PDF pages 30 of LNETB and 12 of LIISPCEN were visually inspected at the signature
and wrapped-header boundaries. All EC2 temporal statuses remain unknown.

## EC3: six reviewed, one held

| Slug | Articles | Original transitories | References | Disposition |
|---|---:|---:|---:|---|
| lfcp | 23 | 5 | 20 | Reviewed; ordinal adapter exception and exact signature boundary |
| reg-lfcp | 86 | 4 | 13 | Reviewed; ordinal adapter exception, including 56 Bis |
| reg-ccomer-mpsc | 29 | 1 | 15 | Reviewed; 29 ordinal-label notices match 1o.–29o. source sequence |
| reg-lfpc-art122 | 14 | 2 | 7 | Reviewed; publisher's incorrect running header excluded |
| reg-lfjs | 158 | 7 | 29 | Reviewed; five Bis/Ter/Quáter ordering notices match source sequence |
| reg-lfppi | 202 | 5 | 40 | Reviewed; source publication 2026-04-28 |
| lfjs | 17 parsed | **0 parsed / 5 in source** | Excluded | Held for numeric-transitory parser defect |

All seven prepared official URLs downloaded successfully. LFCP and its regulation
initially failed the known ordinal-label ordering gate. Their source-heading
sequences were checked before applying the existing adapter exception. LFCP's
last transitory also required an exact enactment-signature stop marker.

The official Article 122 PDF's running header says it is a seeds regulation.
Its displayed title, promulgation text and Article 1 identify the consumer-law
arbitration regulation. PDF page 1 was visually inspected to confirm the publisher
error. The exact erroneous running-header line was excluded through the adapter;
the original PDF, extracted text and hashes remain unchanged. The header had
contaminated Articles 4, 8 and 11. No substantive provision text was rewritten.

All 24 original transitory bodies of the six admitted EC3 candidates matched the
independent source-text comparison. Their article heading sequences matched,
including the regulation suffixes. References and exports were regenerated.
Ten EC3 provisions retain deterministic source-text repeal classifications;
the others are unknown. No temporal model or legal-review resolution ran.

### LFJS hold and recovery

The retained three-page source has five original transitories labeled `1o.`
through `5o.`. The parser's word-ordinal transitory recognizer does not admit
those bare numeric labels, so it emits none. Enabling the article-order exception
alone would hide the ordering errors without recovering the missing transitories;
that shortcut was not taken. LFJS was moved from the operational manifest's
runnable entries to its blocked list, while the prepared inventory was preserved.

The invalid candidate was moved intact to
`.work/held/lfjs/corpus/` so it cannot masquerade as an admitted sibling. Its
source PDF/text and adapter remain available for diagnosis. None of the fourteen
accepted candidates' resolved references targets LFJS. A future repair needs a
fixture for the five numeric transitories, exact text/identity checks and an
enactment-signature boundary; it must not silently affect unrelated corpora.

## Verification actually performed

- `cargo fmt --check` passed.
- `cargo clippy --workspace --all-targets -- -D warnings` passed.
- `cargo test --workspace` passed: 220 tests across the reported suites,
  plus passing doc-tests.
- `cargo run --locked -p lex-cli -- validate lritf` and
  `cargo run --locked -p lex-cli -- validate ifpe-dcg-2021` passed.
- For all fourteen accepted slugs, `link`, `validate` and
  `export --format markdown` passed with the isolated root and external Obsidian
  export disabled. The initial batch failure reports remain as historical evidence;
  final per-slug validation reports capture recovery.
- Recomputed all 28 retained PDF/text hashes; they match the manifests.
- All 68 original transitory bodies and all article-heading sequences matched
  independent source comparisons. Checked changed terminal text and Markdown.
- Every resolved reference target exists in the candidate/main-baseline snapshot;
  LFJS is excluded. Thirty-six remaining warnings are reviewed ordinal/suffix
  notices, not unfrozen counts.
- Main and intake `git diff --check` passed. Reviewed candidate data, provenance
  and file digests are recorded in
  [`ec2-ec3-reviewed-intakes-2026-10-08.json`](ec2-ec3-reviewed-intakes-2026-10-08.json).

The tests cover the unchanged Rust code at the intake base. Run the required
gate again on the eventual integrated checkout before committing. Figma,
reader source, previous legal reviews and SCJN work were not modified this turn.

## Next action

Apply the verified integration packet to the designated integration checkout,
then check affected reverse links and the required gate. Keep LFJS in its own
fixture-backed repair task. No commit, push or release authorization is inferred.
