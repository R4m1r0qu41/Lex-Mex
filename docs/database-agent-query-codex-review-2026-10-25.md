# D3 independent review and D4 migration assessment

Date: 2026-10-25. Verdict: **D3 requires repairs; D4 runtime decision remains open.**
Reviewed uncommitted implementation on Reader HEAD `f49c07923`, with source
digests in the companion evidence JSON. The current accepted artifacts pass
parity checks. The findings below concern input enforcement and the safety of
creating future bindings; they do not establish corruption of those artifacts.

## R1 — P2: binding derivation compares reference IDs, not contents

`crates/lex-db/src/public.rs:345–358` collects public reference IDs and compares
them with DB IDs. Span, offsets, target, resolution and other public edge fields
are not compared. Other public provision/source metadata is also not fully
compared by the binding derivation; the separate real-data acceptance test covers
the present release, not arbitrary future bindings.

Reproduced with the accepted DB read-only and a temporary public release tree:
change LRITF's first reference `source_span` from `98` to
`D3 SYNTHETIC CHANGED SPAN`, update the instrument artifact hash/length and
recompute the release manifest ID. `public-bind` exits 0 and creates a binding.
Thus content-address validation passes while claimed DB/public projection parity
does not. Accepted originals were untouched. This is not an attack that preserves
the original release hash; it demonstrates insufficient compatibility checks
when accepting a newly generated release.

Required repair: compare the complete allowlisted public projection against the
DB-derived projection before producing the binding. Reuse a common deterministic
projection where practical. Include edge source/target/status/span/offset/form/
basis/availability and all provision/source fields served under the release pin.
Keep private fields excluded. Preserve title anchors and out-of-scope target
qualifications. Validate public input schemas and retain byte/hash checks.

Regression acceptance: rehashed, self-consistent public releases with changed
reference span, target or status, provision metadata or official-source metadata
must fail binding; unchanged releases must pass. Exercise title and annex sources.
If the binding schema changes, update types, fixtures and documentation together
and regenerate only derived bindings; never alter accepted DB/publication bytes.

## R2 — P2: unbounded stdin framing precedes the request limit

`crates/lex-cli/src/db.rs:185` uses `BufRead::lines()`. The entire line is allocated
before `PublicReader::handle` checks 8,192 bytes (`public.rs:718`). The SQLite
deadline begins after collection and cannot limit an oversized/unterminated line.

Independent probe: write 16 MiB without a newline to the existing release binary,
flush, wait 2.2 seconds: process remains waiting and has emitted no response.
Append newline: it returns `bad_request`, request exceeds 8,192 bytes. RSS was not
measured; the reported defect is unbounded line collection, not a measured memory
peak. The probe used the accepted DB read-only.

Required repair: enforce the byte cap during stdin framing with bounded reads.
On overflow, reject and terminate, or explicitly drain without unbounded storage
before accepting another frame. Do not allocate the full line first. Keep the
handler limit for direct callers. Document whether LF/CRLF delimiters count.
The future HTTP shim separately needs body limits, ingress timeout and a bounded
process pool; the SQLite progress callback only interrupts SQL execution.

Regression acceptance: oversized input without newline is rejected as soon as
the cap is exceeded; a very large newline-terminated line never creates a large
frame; exact-boundary/multibyte/CRLF/EOF and subsequent-request behavior are
defined and tested at the CLI framing boundary.

## D4 — reconcile with the owner-account migration

The owner's website `CLOUDFLARE-MIGRATION-PLAN.md` at planning baseline
`384034d4eb4e84c2ec4efb02ea9b30c7d1d1f716` selects a customer-owned Cloudflare
Worker with Static Assets on Workers Free. It excludes D1/R2/paid services from
the initial migration. The DB host handoff still describes the existing Sites
Worker and does not bind its options to that decision.

Required handoff correction: identify the target as the new owner-account
Workers Free website, distinguish its migration from DB hosting, and keep the
native adapter local until a separate backend runtime is selected. Do not add
an API route, container, paid plan or new DB engine as an implicit migration step.
The static R7 publication and website migration can be validated independently.
Do not publish the full canonical DB as a public download to bypass deployment
constraints; the adapter's public field and instrument filters must remain.

The current native adapter cannot run unchanged in a Worker isolate. Cloudflare
Containers can host native applications, but are paid and outside the selected
Free migration scope. A later origin/service decision needs its own limits,
credentials, request mapping, concurrency, exact bindings and rollback evidence.

Primary references checked during review:
[Workers limits](https://developers.cloudflare.com/workers/platform/limits/) and
[Containers pricing](https://developers.cloudflare.com/containers/platform/pricing/).
No owner account settings were inspected or changed in this review.

## Checks and limits

- Reviewed source/schema hashes match Claude's acceptance evidence.
- `cargo test --locked -p lex-db --lib public::tests`: 11 passed.
- Ignored real-artifact `public_release` integration test in release mode: 1 passed.
- Accepted DB SHA-256 and binding hash match the recorded values.
- Parent independently reproduced R1; independent Sol reviewer reproduced R2.
- The initial `public_tests` cargo filter matched 0 tests; the corrected module
  command above is the actual targeted-suite evidence.
- No full-workspace rerun or production-host validation in this review.

Exactly one next action: Claude repairs R1/R2, updates D4 for the migration,
runs the regression/native/artifact checks and returns digest-bound evidence for
independent re-review. This review authorizes no implementation commit or deploy.
