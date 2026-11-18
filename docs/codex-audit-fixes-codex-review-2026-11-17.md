# Codex audit fixes — re-review, 2026-11-17

**Verdict: revise both candidates before claiming the audit is closed.** The
original witnesses are repaired and the native gates pass, but the port's new
tail recognizer still admits conditions/second events, and the reader checker
still fails on valid additional navigation links. The oversized-item policy is
reasonable; its response-contract evolution needs a versioned treatment.

No implementation fix or corpus mutation was made. The only source-worktree
write is this uncommitted verdict. Probes and builds ran in throwaway clones.

## Identity and scope

- Port: `fix/codex-audit-2026-11-17`, HEAD
  `117a355049be88359d805676ca4a66bb4052ffc0`, base `47a3ffc59`.
- Reader: `feat/public-reader-r7`, HEAD
  `921e1bd6c26ef643e2cfe35d9473b972dbe5abb8`, reviewed against `6a239b1c5`.
- C5 records: `1ae2a7a90..a283f85ff`.
- Main working checkout at intake: `pilot/fintech-structural`,
  `f80487e867623710453b4a8b1de203f3be209a4b`.

Both candidate worktrees were clean at intake. The port uses current-main code
without the old branch's corpus snapshots; `git diff 47a3ffc59..117a35504 --
corpus` is empty. The schema/type/CLI/calendar changes and the supplied tests
were inspected against the earlier audit, rather than inferring correctness
from the passing test counts.

The operator's updated instructions retire capsules; the older adapters in the
reviewed branches do not authorize starting one. Their AGENTS/CODEX companion
layout and the global fallback setting already satisfy the on-demand convention.
No instruction file was changed.

## Findings requiring revision

### F1 — P1: the optional tail reopens C1/deferred limit 1

Location: port `crates/lex-analysis/src/temporal_derive.rs:634`, `LEGACY_TAIL`.
The initial date forms are anchored, but two tail components consume unparsed
substantive text:

```text
(?:\s+y\s+abroga\s+[^.;:]+)?
...
(?:"[^"]+"\.?-\s*)?
```

The first is not limited to a plain identification of an abrogated law. The
second does not establish that a quoted string is a year motto rather than a
condition. The public derivation API, publication 2020-01-01 and an empty calendar,
produces **one exact 2020-01-02 determination** for every witness below:

```text
La presente Ley entrará en vigor al día siguiente de su publicación y abroga
la norma anterior siempre que se haya expedido el reglamento.

La presente Ley entrará en vigor al día siguiente de su publicación y abroga
el decreto anterior, pero esta Ley entrará en vigor a los treinta días de
su publicación.

La presente Ley entrará en vigor al día siguiente de su publicación.
"Siempre que se haya expedido el reglamento".- Juan Pérez, D. P.- Rúbricas.
```

The API reports `day_after_publication`, not review. In the second witness the
single transitory contains two commencement events; counting matching transitory
records does not reject that case. This is a structural-recognition defect;
this review does not choose which legal date governs any such clause.

**Required revision:** constrain the abrogation tail and distinguish verified
source apparatus from arbitrary quoted text. Reject these witnesses without
changing their canonical text or selecting a date. Preserve the plain positive
control and measured genuine signature forms with explicit fixtures. Merely
adding one more exception word would leave the underlying wildcard problem.
Until then, the original C1 witness is closed but the whole-clause guarantee
and October 28 limit 1 are not.

### F2 — P2: R2 remains incomplete for navigation to the selected index

Location: reader `reader/retrieval-check.mjs:104–108`. The revised predicate
compares origin and pathname, but does not exclude search/hash variants before
counting candidates. A valid release index containing the canonical LRITF link
and either of these additional links fails discovery:

```markdown
[Extra navigation](mx/lritf/index.md#toc)
[Extra navigation](mx/lritf/index.md?view=all)
```

Both local probes returned `ok: false`, `Expected exactly one LRITF index link`,
after four exchanges. Neither followed an off-pin link. Original home/DOF/
release-root fragment examples are fixed, but a normal table-of-contents or
alternate-view link to the same article index still prevents retrieval.

**Required revision:** select among eligible full URLs, including the no-query,
no-fragment and credential restrictions, rather than counting every URL with
that pathname. Keep strict `safe()` enforcement on the chosen fetch, and retain
the cross-origin control. Add the two same-index navigation cases as regressions.

### F3 — P2 contract revision: `skip_cursor` changes the same closed wire version

Locations: reader `public.rs:24,940–951,1180–1194` and
`schemas/public-query-response-v1.schema.json:452–455`.

The new explicit skip policy is approved in principle: an oversized item is
reported, its cursor is bound to the same request/release, and the caller must
opt in to continuing after that item. The traversal regression confirms progress
and termination; no silent omission is needed.

However, this is not backward-compatible with the previous closed response
contract. An actual new oversized-item response validates against the new schema
and fails the old schema, while still identifying itself as
`public-query-response-v1`:

```text
new_valid=true old_valid=false schema_version="public-query-response-v1"
```

This differs from the refuted R3 naming claim: here the **same actual wire
identifier** is being reused for a response the prior closed schema rejects.

**Required integration treatment:** version the changed response contract and
update its runtime discriminator, schema, consumers/fixtures and documentation
coherently, or explicitly establish a coordinated unreleased-contract replacement
with all pinned consumers updated. Do not represent this as compatibility with
the previously accepted closed v1. A response-version change need not relabel the
unchanged database projection or canonical/public-release bytes. Also express
in the schema, rather than only its description, that `skip_cursor` belongs to
`response_too_large`; it is currently permitted on any error code.

## Disposition by earlier finding

| Item | Disposition at these heads |
|---|---|
| C1 | Original plural/unaccented business-day witness now emits no determination; broader closure blocked by F1. |
| C2 | Closed for missing/misspelled/extra keys and malformed calendar shape. CLI validation occurs before any mutation; semantics reject reversed ranges, overlapping statutory versions and invalid month/day pairs. |
| C3 | Word-boundary/accent/whitespace guard defect closed. `Interno` is no longer classified as negation. Its original named-subject example can still be conservatively unsupported by the separate subject whitelist; that is not the old substring defect. |
| C4 | Closed: normal derive's save gate counts `split`, with an end-to-end persistence regression. |
| C5 | Closed: both old records explicitly say superseded and preserve the historical 241-provision measurements. |
| C6 | Closed: one loaded calendar reaches both repair and derivation, including repair-only, with the amended-article date-clear regression. |
| R1 | Traversal mechanism repaired and explicit policy approved; wire-contract integration requires F3. |
| R2 | Original extra-link cases fixed; F2 still blocks full closure. |
| R3 | No change required; the original version-name collision claim remains refuted. |
| R4 | Late successful dispatch is now converted to `deadline_exceeded`; the SQL/non-SQL resource-bound qualification below remains. |
| R5 | Closed in both port/main-code paths and reader: sidecars append to the complete OS filename. |
| R6 | Closed: opening reuses the binder's URL-based annex selection and checks the bound digest, preserving distinct same-content sources. |
| `write_binding` race | No-clobber mechanism closed by `create_new`; `sync_all` is used after writing. This is not a new crash-atomic publication guarantee. |
| Shared number words | Acceptable scoped extension; the additional prospective `ldsc` result is disclosed, with no corpus write. |

### R4's remaining qualification

The added checks detect expiry between hits and after dispatch, but do not
interrupt a single long excerpt computation. The independent 800,007-byte
synthetic provision probe with a **20 ms** configured deadline returned
`unavailable / deadline_exceeded` after **325 ms**. This is an improvement over
the earlier late `ok`; it is not a hard wall-clock cap.

Accept this as deadline classification/cooperative checks for the local adapter,
not enforcement of a strict end-to-end execution budget. If the intended promise
is a hard whole-request deadline, R4 remains open until non-SQL work is bounded
or interruptible, or isolated by a suitable host timeout. The supplied zero-timeout
unit test alone does not establish that property. No production two-second
performance or hosting claim was tested or accepted here.

## Answers on the port, tail and prior limits

**Subject strictness:** the conservative whitelist is acceptable for automatic
recognition; unsupported complex titles should go to review rather than a
fragment fallback. The measured loss of seven prior automatic outcomes is an
explicit coverage change. It does not establish that those provisions are not
in force, and no committed status was changed.

**Signature/page apparatus:** allowing measured extraction residue can be useful,
but the current free-quoted-string/abrogation tail is too loose (F1). A capitalized
name test does not validate the semantic contents of the preceding quote. Narrow
and pin recognized source-apparatus shapes or separate them with provenance
before matching the substantive clause. Unmatched apparatus can remain review;
there is no need to invent a legal interpretation to keep every current match.

**October 28 limits:**

- Limit 1: not closed, because F1 still admits conditional/second-event text.
- Limit 3: the computational safeguard is accepted. The shipped December entry
  ends at 2018, and unsupported later occurrence years return insufficient
  rather than being treated as ordinary years. This does not resolve JRH's
  pending legal question or independently approve historical applicability.
- Limit 4: typed/schema/semantic input validation closes the permissive-loader
  defect. Provenance remains operator-reviewed data: nonempty descriptive
  strings are not independent verification of DOF sources, hashes or historical
  coverage. Arbitrary imports must not be represented as independently verified
  evidence merely because they pass this schema. Earlier historical-evidence
  limit 2 remains outside this code acceptance; shipped empty notice coverage
  still prevents business-day determinations.

**Sound base for the corpus-wide in-force rule:** yes as the current-main
integration base and selected code port; no as an approved automatic classifier
at this candidate. Keep this base, fix F1, and return bounded positive/negative
regressions before applying it across canonical records. Do not restart from the
old branch or import its corpus snapshots. Preserve the distinction between the
operator's in-force rule, an article's exact current-wording dates, and audited
legal decisions. No legal holiday/ruling interpretation was made here.

## Independent checks actually run

All builds/probes used detached throwaway clones under
`/private/tmp/lex-audit-fixes-rereview-20261117/`; production worktrees were not
built or modified by the probes.

Port at `117a35504`:

- `cargo fmt --all --check`: pass.
- `cargo test --workspace --offline --locked`: **573 passed**, zero failed.
- `cargo clippy --workspace --all-targets --offline --locked -- -D warnings`:
  pass; existing unused `walkdir` workspace-dependency warning remains.
- Locked/offline `validate lritf` and `validate ifpe-dcg-2021`: valid, zero issues.
- Independent public-API tail probes: one observational test passed, printing
  the three unsupported tail witnesses incorrectly resolved; original business/
  negation controls abstain and plain positive control resolves.
- Paired read-only census: **468 instruments; eight changed; zero date changes
  among still-resolving instruments**. Seven move to review (`lcmm`, `lfdefp`,
  `lfgr`, `lgmdfp`, `lnsijpa`, `ltfccg`, `reg-senado`); `ldsc` newly resolves to
  2026-06-17. The reviewer compiled the literal pre-fix `f55e9584f` resolver
  (test module omitted) alongside the fixed resolver with shared current record
  types and an empty calendar. This is prospective in-memory comparison, not a
  national canonical derivation or replay.

Reader at `921e1bd6c`:

- `cargo fmt --all --check`: pass.
- `cargo test --workspace --offline --locked`: **449 passed**, zero failed;
  one ignored accepted-release test.
- `cargo clippy --offline --locked -p lex-db --all-targets -- -D warnings`:
  pass. Full-reader-workspace clippy was not rerun; the request discloses two
  unrelated existing CLI diagnostics.
- `node --test reader/retrieval-check.test.mjs`: **20 passed** after permitting
  its synthetic loopback server. Initial sandbox execution was blocked by EPERM.
- Two independent Rust probes: old/new closed-schema behavior and the delayed
  deadline-error result above.
- Two local extra-link probes reproduce F2 with hash-valid synthetic releases;
  no live external site was fetched.

No LIC/other-validator rerun, original G3/not-found gold access, `--holdout-gold`,
structure-pilot lookup, database rebuild, corpus write, merge, commit, push or
deployment was performed. LIC's two recorded stale-citation warnings remain
attributed builder evidence; the corpus diff is empty.

Probe snippets and logs are retained in the scratch directory, including
`tail-probes.rs`, `tail-probes.log`, `reader-review-probes.rs`,
`reader-review-probes.log`, `checker-fragment-probes.mjs`,
`checker-fragment-probes.log`, `baseline_temporal.rs`, `census-compare.rs`,
`census-compare.log`, and the native check logs. Observation tests assert the
reported behavior; their passing status is not an acceptance of the reproduced
failures. Earlier verdicts and their evidence were not overwritten.

## Closeout

No plan, project-status file, provider configuration, source fixture or prior
verdict was edited. The source-write exception authorizes only this new report;
external receipt/rolling-note/handoff administration was not performed under
that read-only constraint. Capsules are retired and none was created.

**Next action:** builder narrows the commencement tail, completes full-URL link
selection, and versions/coherently revises the skip-cursor response contract;
then submits the corrected heads with these counterexamples as regressions.
Leave this verdict uncommitted as requested. No merge or rollout is approved.

Scratch artifact-manifest SHA-256: `fb511317de9abd6d3dab472c8d6a14003f7303fad0997b1c8b0c9e1555b74ab5`.
