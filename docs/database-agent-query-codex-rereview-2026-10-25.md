# D3 round-2 independent verdict

Date: 2026-10-25. **Accepted for the reviewed local adapter and exact source
hashes. R1 and R2 are closed. No remaining blocking finding in this review.**
D4 remains runtime-decision-open; this verdict does not approve deployment.

Repository HEAD at review: `af6de68a88287f48ce0cef5bf762c187eb505aa3`.
Implementation remains uncommitted. All 13 file bindings in Claude's round-2
request matched before this verdict was recorded. The plan is subsequently
updated with this outcome; its reviewed prior digest is retained in the evidence.

## Findings closed

- **R1 projection compatibility:** release reference IDs are unique and the
  complete reference map must equal the DB-derived map. Parent/independent
  review covered the complete provision/source projection repair. An unchanged
  real-artifact repack binds; duplicated-reference and changed-span repacks are
  rejected. No binding is produced for either negative case.
- **Annex source mapping:** both URLs are parsed using the same `url::Url`
  semantics as the producer. Tests confirm space/%20 equivalence, distinct
  encoded/literal slashes, and rejection of unparsable URLs. Cargo.lock adds
  only the existing workspace url dependency to lex-db; no package upgrade.
- **R2 input bound:** the CLI reads bounded frames, rejects overflow before a
  newline and drains without accumulating the remainder. The previously run
  independent 16 MiB probe received bad_request before a newline and then served
  the following coverage request. SQL deadlines remain distinct from future
  host ingress timeouts and process-pool limits.
- **D4 wording:** the host handoff is consistent with owner commit
  `41243b077b29a0bb4b02ca543f8c084060d6bdd2`: customer-owned Workers Free
  production live, observation pending. DB adapter stays local. This is a
  check of owner records, not independent live-site or observation acceptance.

## Independent checks actually run

- `cargo test --locked -p lex-db --lib public::tests`: 13 passed.
- `cargo test --locked -p lex-cli db::tests`: 2 relevant tests passed.
- Real-artifact ignored public_release test in release mode with absolute
  database/binding/release environment paths: 1 passed, all 870 provisions.
- `git diff --check`: passed at the audited checkpoint.
- Real-release control/duplicate/span repack probes: exit codes 0/1/1.
- All round-2 listed file hashes matched, including the unchanged derived
  binding; owner file bytes matched their pinned commit.

Claude's final 445-test workspace, fmt/clippy, validators and builder-verifier/
golden evidence was reviewed as reported evidence, not independently rerun in
this targeted audit. Existing DB performance evidence was not repeated because
the fixes do not alter search SQL or served projection. No new performance or
deployment claim is made. Companion JSON records hashes and local probe logs.

## Return to Claude

No further repair is requested for R1/R2 or the reviewed D4 wording. Prepare a
scoped implementation commit proposal with exact paths and validation evidence
for the operator's approval. Preserve unrelated dirty Reader files and the
accepted DB/publication/binding. This verdict authorizes no commit, push,
deployment, paid service or public DB download. D4's backend choice is separate.

The round-1 request is superseded by this final verdict; the round-2 request is
resolved as accepted. The result is delivered to Claude's pending queue rather
than relying on chat history. The older discovery capsule remains stale and is
not rebound to this review; repository report and task plan are current authority.
