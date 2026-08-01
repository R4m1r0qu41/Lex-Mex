# Standard post-transitory supplements

## Purpose and outcome

Model source-ordered, exact-span top-level material following a genuine
standards `TRANSITORIOS` section without parsing its internal structure. The
observable outcome is a required `supplements.json` for every committed
standard, reviewed tail repartition for all affected standards, and verified
ingestion of NOM-019-STPS-2011, NOM-024-STPS-2001, and
NOM-052-SEMARNAT-2005.

This milestone excludes supplements in standards with no transitories,
internal appendix structure, inferred normativity or legal effect, and the
out-of-repository Maximasa refresh.

## Authority and boundaries

Repository `AGENTS.md`, accepted decisions, schemas, Rust types and validators,
committed canonical files, retained source manifests, and current Git state are
authoritative. Official held-out Platiica PDFs may be reacquired only from the
URLs and hashes already recorded in repository preparation artifacts.

Initial baseline: commit `8bbef887e6ea58ddbc5059c4fca506417d42f844`,
branch `codex/post-transitory-supplements`, isolated worktree
`/Users/jr/Dev/lex-mex-post-transitory-supplements`.

## Current checkpoint

- 2026-07-31: trusted boundary implemented and tested; all 29 committed
  standards inventoried and migrated. The inventory found 23 affected records,
  not the anticipated 16; seven additional STPS guides were silently swallowed
  behind previously unrecognized `Dado en…` closings.
- 2026-07-31: fresh Platiica downloads for NOM-019, NOM-024 and NOM-052 matched
  all six recorded source/text hashes; temporary compiles produced the expected
  94/3/1, 87/2/2 and 76/3/8 counts.
- Current next action: run the full trusted-boundary acceptance suite and land
  the atomic migration commit, then add the three verified corpus directories.

## Decisions

- Represent only configured top-level post-transitory Apéndice, Anexo, Guía de
  Referencia, standalone Tabla, and Listado spans. Keep their contents opaque.
- Anchors are exact source strings, ordered, and may span lines. A configured
  anchor must resolve exactly once after the real transitory heading.
- Legal character comes only from explicit source language. Conflicting
  explicit signals invalidate the record; absence yields a warning.
- `standards refresh` refuses any transitory or supplement change by default.
  `--allow-tail-repartition` admits only reviewed final-transitory truncation
  paired with supplement derivation; all guards precede writes.

## Milestones

1. Trusted boundary: types, schemas, shared tail layout, parsers, validation,
   CLI/path/bundle wiring, refresh guards, and regression tests.
2. Corpus migration: bounded inventory for 29 standards; required empty files
   for unaffected records and reviewed anchors/spans for affected records.
3. Held-out ingestion: reacquire hash-matching official PDFs and compile the
   three prepared standards with expected counts and reviewed diffs.
4. Closeout: documentation/totals/receipts/context, full required checks, an
   atomic migration commit and a separate ingestion commit; no push.

At each milestone, stop on source-hash mismatch, clause drift, a change to an
earlier transitory, unreviewed final-transitory loss, invalid exact spans, or
unexpected repository scope. Recovery is from the clean isolated branch and
the last reviewed commit; no destructive operation is authorized.

## Verification and acceptance

Compare reparsed canonical data deeply for every standard. Clauses must remain
byte-identical; earlier transitories must remain byte-identical; a changed final
transitory may lose only closing furniture or represented supplement text.
Inspect affected manifests, supplement spans, validation reports, and full new
NOM diffs. Run repository formatting, clippy, workspace tests, LRITF and IFPE
validation, plus standards validation for all 32 slugs. Acceptance requires
zero swallowed represented material, exact source-span round trips, passing
validation, and expected held-out clause/transitory/supplement counts.

## Outcomes and retrospective

Pending.
