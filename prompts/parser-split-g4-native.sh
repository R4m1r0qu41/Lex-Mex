#!/bin/bash
# G4 native commands at C=1250889ce in a disposable clone; each line: command, exit code, seconds.
W=/Users/jr/Dev/lex-mex-parser-split/.work/parser-split
cd $W/g4-native || exit 90
export CARGO_TARGET_DIR=$W/g4-native/target
run() { local name=$1; shift; local s=$(date +%s); "$@" > $W/g4-out/native-$name.log 2>&1; echo "$name exit=$? seconds=$(( $(date +%s) - s ))"; }
echo "HEAD=$(git rev-parse HEAD)"
run fmt cargo fmt --check
run clippy cargo clippy --workspace --all-targets -- -D warnings
run test cargo test --workspace
run validate-lritf cargo run --locked -p lex-cli -- validate lritf
run validate-ifpe-dcg-2021 cargo run --locked -p lex-cli -- validate ifpe-dcg-2021
echo "status-after:"; git status --short | head -20
