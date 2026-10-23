# Parser split implementation receipt

Living narrative for [the execution plan](plans/parser-responsibility-split.md);
machine evidence is in `parser-split-implementation-evidence.json`. Status:
**IN PROGRESS**. Nothing here is accepted or merged.

## G0 — isolation, reproducible baseline and replay coverage (IN PROGRESS)

- Execution baseline B = `85c41f3898d86737766564fa8d3fd8f6d4ae74a3` (equals the
  plan's planning baseline; no HEAD drift). Worktree
  `/Users/jr/Dev/lex-mex-parser-split`, branch `refactor/parser-split`, created by
  the operator. Local milestone commits are authorised by the goal prompt; no
  push, merge, deployment or corpus change.
- Frozen baseline binary built with `cargo build --locked --release -p lex-cli`;
  sha256 and tool versions are in the evidence file.
- Native baseline at B is green: fmt, clippy `-D warnings`, and 419 workspace
  tests with none ignored; tests left tracked files untouched.
- A/B harness: `prompts/parser-split-ab.py`. It replays retained-source stages
  and a whole-corpus sweep through one frozen binary, rebuilding a disposable
  root from `git archive <B>` at one fixed absolute path with a scrubbed
  environment, so no path or timestamp normalization is needed. Comparison is
  byte-exact on exit code, stdout, stderr and every file written.
- Retained-source coverage: 355 of 500 committed instruments have retained
  extracted text and replay. 113 have none retained in the main checkout's
  ignored `.work`; 32 NOM standards have no adapter and are covered by the sweep
  through `standards validate` and `standards refresh`. Two retained texts
  (`cub-dcg-2005`, `lfiiedb`) do not match the committed manifest hash and are
  recorded as replayed but not provenance-matched.
- Baseline replay `validate` exits non-zero for a few instruments at B (for
  example `cjm`, `laero`). These are deterministic baseline facts tied to
  adapter-frozen counts and the recorded heading-leak holds, not defects
  introduced here; the candidate must reproduce them exactly.
- Self-replay B1 versus B2 is pending. G0 is not PASS until it is exact, the 113
  absent inputs are resolved or listed as blockers, and the advisor accepts the
  coverage matrix.
