#!/bin/bash
# G4 native commands at C=1250889ce in a disposable clone. Prints one line per
# command (name, exit code, seconds), keeps every command's full output in
# g4-out/native-<name>.log, runs all commands even after a failure, and exits
# non-zero if any command failed (the first failing code).
# G4_WORK overrides the work directory (used by the self-test).
W=${G4_WORK:-/Users/jr/Dev/lex-mex-parser-split/.work/parser-split}
cd "$W/g4-native" || exit 90
export CARGO_TARGET_DIR=$W/g4-native/target
FAILED=0
run() {
  local name=$1; shift
  local start=$(date +%s)
  "$@" > "$W/g4-out/native-$name.log" 2>&1
  local code=$?
  echo "$name exit=$code seconds=$(( $(date +%s) - start ))"
  if [ "$code" -ne 0 ] && [ "$FAILED" -eq 0 ]; then FAILED=$code; fi
}
echo "HEAD=$(git rev-parse HEAD)"
run fmt cargo fmt --check
run clippy cargo clippy --workspace --all-targets -- -D warnings
run test cargo test --workspace
run validate-lritf cargo run --locked -p lex-cli -- validate lritf
run validate-ifpe-dcg-2021 cargo run --locked -p lex-cli -- validate ifpe-dcg-2021
echo "status-after:"
git status --short > "$W/g4-out/native-status-after.txt"
STATUS=$?
head -20 "$W/g4-out/native-status-after.txt"
if [ "$STATUS" -ne 0 ] && [ "$FAILED" -eq 0 ]; then FAILED=$STATUS; fi
exit "$FAILED"
