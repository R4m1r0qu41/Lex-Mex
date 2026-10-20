# SQLite M2 builder — 2026-10-20

> Historical blocked-build checkpoint. The upstream repairs and current rerun
> are documented in the [integration report](sqlite-m2-upstream-integration-2026-10-20.md).

## Outcome

M2's transactional projection and safe local publication path are implemented
in the DB worktree, uncommitted. An end-to-end synthetic release passes twice
with identical database bytes and manifests. The real corpus build fails closed
on two upstream contract mismatches; **no full-corpus DB artifact exists**.
M2 acceptance remains open. M3 independent verification and M4 full-corpus
reproducibility are not claimed.

Corpus input: accepted `611171e52062c67d0ef000cf42c72b87af4c4ef3` (underlying
corpus unchanged from bfcda816f). The full-input attempt used unreferenced test
builder snapshot `404ce306d4b07dd35f6d305f64654ce09b35d9f3`; no branch moved.
Later synthetic publication tests were added after that snapshot; they did not
relax the preflight gate. Exact diagnostics and all 117 affected term identities
are in [the evidence receipt](sqlite-m2-builder-2026-10-20-evidence.json).

## Implemented behavior

- `lex-mex --root <repo> db project --corpus-commit <full-oid>
  --builder-commit <full-oid> --output-dir derived/db` reads immutable Git objects.
  The input inspector retains each file once, hashing those exact bytes.
- The compiled implementation, local validator modules/schemas, Cargo.lock and
  build inputs must match the claimed builder commit. Rust/SQLite versions,
  compile options, stable source timestamp and sorted input descriptors enter
  the release identity. Uncommitted builder code cannot claim the old HEAD.
- Closed schemas, shared in-memory corpus validation, glossary accounting,
  source ownership/hashes, publication/review/repeal metadata, parent rules and
  resolved target ownership run before building. Existing closure-gap metadata
  stays separate from validation of the stored graph; failed relinks do not
  become invented reference edges. Stored report errors need exact accepted
  error multiplicity. Warnings are preserved in retained inputs.
- The inherited debt ledger's historical hash and accepted-identity subset of
  proposed series-checked rows are now checked. The historical authority bytes
  stay pinned even when the source review predates the inventory policy.
  Original 17 debt rows and human decisions are unchanged.
- Prepared insertions run in one transaction, loading instruments, provisions,
  title/provision anchors, references, ordered qualifiers, independent approvals
  and operator reviews, terms, usages, glossary accounting, enactment evidence,
  source manifests, accepted gaps/debt, raw evidence and standards catalogue.
  Source ordinals/pointers survive; provision row IDs follow sorted input-file
  order and array order. Instrument slugs are unique.
- Coverage reports source manifests/canonical records/stored graphs separately;
  listed closure gaps remain explicit. Analysis completeness is `not_assessed`,
  with per-provision status counts still queryable. No whole-instrument replay
  measurement is inferred; reproducibility rows remain absent and the view
  supplies `not_assessed`. Partial replay evidence remains raw evidence.
- Outputs are restricted to ignored `derived/db/`, rejecting protected paths,
  traversal, aliases and symlink components. SQLite uses foreign keys, DELETE
  journaling and FULL synchronous mode. Owned staging directories roll back
  on handled failures. An interrupted `.building-*` directory or stale lock is
  not a published release; inspect/remove only that owned failed staging/claim
  before a fresh retry. Prior releases are never repaired or overwritten.
- External-snapshot raw/normalized/supporting-table parity, integrity/FK checks,
  FTS rebuild/integrity and a read-only reopen precede fsynced manifest/receipt
  writes and an atomic sibling-directory rename with an exclusive claim.
  The manifest's logical hash covers every SQL table's sorted typed values;
  raw input bytes and complete database bytes have separate checksums.
  This preliminary build gate does not replace M3's independent verifier.

## Real build blockers and remedy

The complete preflight finished in 112.47 seconds with exit 1, before staging a
SQLite artifact. It inventoried the accepted 4,960 inputs and checked the full
federal selection. Shared corpus checks reported no additional errors in this
pass. The native validators do not currently enforce all these JSON schemas,
which explains why their success did not reveal the mismatches.

| Cluster | Observed issue | Recommended upstream repair |
|---|---|---|
| Term identifiers | 117 definitions in 26 instruments and 8 usages in 3 instruments fail `[a-z0-9-]+`; the parser's `terms::slug` retains punctuation. Existing suffixes include parentheses, straight/curly quotes, comma, slash, colon, apostrophe, period and leading `[` characters. | Establish one versioned term-ID contract across producer, schema and consumers. Audit the nine leading-bracket names in LGAPPCEA/LGCS against their exact sources before treating every case as legitimate punctuation. Preserve source text. Prefer retaining existing identities if a reviewed compatibility grammar can describe them honestly; otherwise perform a deliberate canonical ID migration with all usages/import bindings on main. Do not strip punctuation or relabel IDs inside the DB loader. Add fixtures for quoted LAN names, parentheses, slash/colon, bracket handling and dangling usages; verify all canonical term/usage files against the chosen schema. |
| Operational source | `reg-lgs-insumos/source-manifest.json` records `secretaria_de_salud`, also recorded by its adapter; the manifest schema permits only Cámara, CNBV and DOF. | Add the already-used publisher identifier to the shared source contract/schema with an official-source fixture and consistency check, rather than falsely renaming the source to another publisher. No legal determination is needed for this metadata alignment. |

These are technical source-contract repairs. Nine bracket-prefixed terms require
source inspection to distinguish literal names from extraction furniture. No
legal conclusion or canonical correction is inferred from their appearance.
The DB-only plan keeps parser/corpus repairs on main; this implementation leaves
both canonical data and input schemas unchanged rather than accepting invalid
inputs silently. After repair, integrate by **normal merge** preserving
6505baf57 ancestry, revalidate the debt seed and rerun with full new commit IDs.
SCJN remains evidence-only under its separate deferred integration plan.

## Verification and limits

Final mechanical gate: formatting and full-workspace strict Clippy pass;
`cargo test --locked --workspace --quiet` passes **387 tests**, zero failures.
Required locked validators pass: LRITF 145 articles / 11 transitories / 126
references; IFPE 59 / 4 / 115; both zero issues. Corpus diff from HEAD is empty.

Regression coverage includes transaction rollback on duplicate/missing targets,
source-pointer and supporting-table corruption, raw handling, catalogue/FTS
separation, inherited authority, safe output paths, publication collisions and
stale claims. A synthetic pinned Git repository exercises the full CLI library
path: exact compiled-builder binding; two byte-identical releases; dirty source
isolation; read-only write rejection; existing-release refusal; invalid committed
source preflight preserving the previous release; builder-source mismatch.
Temporary synthetic releases are cleaned with their test repositories.

No full DB, `verify-db` command, public deployment, legal-review change, commit
or push is part of this working-tree result. Main's two unrelated untracked
suggestion files are preserved. Next: upstream source-contract repair, then
repeat the full M2 gate before proceeding to M3.
