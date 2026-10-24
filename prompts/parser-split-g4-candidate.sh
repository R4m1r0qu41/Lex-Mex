#!/bin/bash
# G4 candidate DB builds: C3 binary, corpus_commit=B, builder_commit=C (1250889ce), same root as the baseline.
set -u
W=/Users/jr/Dev/lex-mex-parser-split/.work/parser-split
B=85c41f3898d86737766564fa8d3fd8f6d4ae74a3
C=$(git -C $W/g4-root rev-parse 1250889ce)
BIN=$W/bin/C3/lex-mex
cd $W/g4-root || exit 90
echo "C=$C"
for X in a b; do
  START=$(date +%s)
  $BIN db project --corpus-commit $B --builder-commit $C --output-dir derived/db/parser-split-candidate-$X > $W/g4-out/cand-$X.json 2> $W/g4-out/cand-$X.err
  echo "project $X exit=$? seconds=$(( $(date +%s) - START ))"
  DIR=$(python3 -c "import json;print(json.load(open('$W/g4-out/cand-$X.json'))['directory'])") || { echo "no directory for $X"; continue; }
  $BIN db verify --database $DIR/corpus.sqlite --manifest $DIR/manifest.json --corpus-commit $B > $W/g4-out/cand-$X-verify.json 2> $W/g4-out/cand-$X-verify.err
  echo "verify $X exit=$?"
  $BIN db golden-queries --database $DIR/corpus.sqlite > $W/g4-out/cand-$X-golden.json 2> $W/g4-out/cand-$X-golden.err
  echo "golden $X exit=$?"
done
