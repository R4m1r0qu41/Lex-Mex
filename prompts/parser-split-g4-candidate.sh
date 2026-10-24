#!/bin/bash
# G4 candidate DB builds: C3 binary, corpus_commit=B, builder_commit=C (1250889ce),
# same root as the baseline. Runs project, verify and golden-queries for builds a
# and b, keeps every output, and exits non-zero if any step failed (the first
# failing code). A failed project or a missing directory skips that build's verify
# and golden steps and counts as a failure.
# G4_WORK, G4_BIN and G4_BUILDER_COMMIT override the defaults (used by the self-test).
W=${G4_WORK:-/Users/jr/Dev/lex-mex-parser-split/.work/parser-split}
B=85c41f3898d86737766564fa8d3fd8f6d4ae74a3
C=${G4_BUILDER_COMMIT:-$(git -C "$W/g4-root" rev-parse 1250889ce)}
BIN=${G4_BIN:-$W/bin/C3/lex-mex}
FAILED=0
note() { if [ "$1" -ne 0 ] && [ "$FAILED" -eq 0 ]; then FAILED=$1; fi; }
cd "$W/g4-root" || exit 90
echo "C=$C"
for X in a b; do
  START=$(date +%s)
  "$BIN" db project --corpus-commit "$B" --builder-commit "$C" --output-dir "derived/db/parser-split-candidate-$X" > "$W/g4-out/cand-$X.json" 2> "$W/g4-out/cand-$X.err"
  CODE=$?
  echo "project $X exit=$CODE seconds=$(( $(date +%s) - START ))"
  note $CODE
  if [ "$CODE" -ne 0 ]; then continue; fi
  DIR=$(python3 -c "import json,sys;print(json.load(open(sys.argv[1]))['directory'])" "$W/g4-out/cand-$X.json")
  CODE=$?
  if [ "$CODE" -ne 0 ]; then echo "no directory for $X"; note $CODE; continue; fi
  "$BIN" db verify --database "$DIR/corpus.sqlite" --manifest "$DIR/manifest.json" --corpus-commit "$B" > "$W/g4-out/cand-$X-verify.json" 2> "$W/g4-out/cand-$X-verify.err"
  CODE=$?
  echo "verify $X exit=$CODE"
  note $CODE
  "$BIN" db golden-queries --database "$DIR/corpus.sqlite" > "$W/g4-out/cand-$X-golden.json" 2> "$W/g4-out/cand-$X-golden.err"
  CODE=$?
  echo "golden $X exit=$CODE"
  note $CODE
done
exit "$FAILED"
