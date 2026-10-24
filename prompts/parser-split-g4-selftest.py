#!/usr/bin/env python3
"""Failure-propagation self-test for the G4 helper scripts.

Runs prompts/parser-split-g4-native.sh, prompts/parser-split-g4-candidate.sh and
prompts/parser-split-g4-compare.py against stand-ins (a fake cargo, a fake
lex-mex binary, small SQLite databases) in a temporary directory, and checks that
an injected failure in any single command makes the script exit non-zero while
the command output is still kept, and that an unchanged control exits 0.
Nothing outside the temporary directory is touched. Exit status 0 means every
case behaved as required.
"""
import json
import os
from pathlib import Path
import sqlite3
import subprocess
import sys
import tempfile

PROMPTS = Path(__file__).resolve().parent
results = []


def check(name, ok, detail=""):
    results.append((name, ok))
    print(f"{'PASS' if ok else 'FAIL'}  {name}  {detail}", flush=True)


def run(command, env):
    return subprocess.run(command, env={**os.environ, **env}, capture_output=True, text=True)


def native_cases(tmp):
    work = tmp / "work"
    (work / "g4-out").mkdir(parents=True)
    clone = work / "g4-native"
    clone.mkdir()
    for step in (["init", "-q"], ["config", "user.email", "t@example.invalid"], ["config", "user.name", "t"], ["commit", "-q", "--allow-empty", "-m", "x"]):
        subprocess.run(["git", "-C", str(clone), *step], check=True, capture_output=True)
    bin_dir = tmp / "fakebin"
    bin_dir.mkdir()
    fake = bin_dir / "cargo"
    # fails when FAKE_FAIL appears among the arguments (fmt, clippy, test, lritf, ifpe-dcg-2021)
    fake.write_text('#!/bin/bash\necho "fake cargo $*"\ncase " $* " in *" ${FAKE_FAIL:-NONE} "*) exit 42;; esac\nexit 0\n')
    fake.chmod(0o755)
    env = {"G4_WORK": str(work), "PATH": f"{bin_dir}:{os.environ['PATH']}"}
    script = str(PROMPTS / "parser-split-g4-native.sh")
    control = run(["bash", script], env)
    check("native control: all commands pass, exit 0", control.returncode == 0, control.stdout[-120:])
    for marker, log in (("--check", "fmt"), ("-D", "clippy"), ("--workspace", "test"), ("lritf", "validate-lritf"), ("ifpe-dcg-2021", "validate-ifpe-dcg-2021")):
        done = run(["bash", script], {**env, "FAKE_FAIL": marker})
        kept = (work / "g4-out" / f"native-{log}.log").exists() and "exit=42" in done.stdout
        later = "validate-ifpe-dcg-2021 exit=" in done.stdout
        check(f"native failure injected at {log}: exit non-zero, output kept, later commands still run", done.returncode != 0 and kept and later, f"rc={done.returncode}")


def candidate_cases(tmp):
    work = tmp / "cwork"
    (work / "g4-out").mkdir(parents=True)
    (work / "g4-root").mkdir()
    fake = tmp / "fake-lex-mex"
    fake.write_text(
        '#!/bin/bash\n'
        'case "$2" in\n'
        '  project) echo "{\\"directory\\": \\"/nonexistent\\"}";;\n'
        '  *) echo "[]";;\nesac\n'
        'if [ "$2" = "${FAKE_FAIL:-NONE}" ]; then exit 42; fi\n'
        'exit 0\n'
    )
    fake.chmod(0o755)
    env = {"G4_WORK": str(work), "G4_BIN": str(fake), "G4_BUILDER_COMMIT": "c" * 40}
    script = str(PROMPTS / "parser-split-g4-candidate.sh")
    control = run(["bash", script], env)
    check("candidate control: every step passes, exit 0", control.returncode == 0, control.stdout[-120:])
    for step in ("project", "verify", "golden-queries"):
        done = run(["bash", script], {**env, "FAKE_FAIL": step})
        check(f"candidate failure injected at {step}: exit non-zero", done.returncode != 0, f"rc={done.returncode}")
    done = run(["bash", script], {**env, "FAKE_FAIL": "project"})
    check("candidate failure: a failed project is reported", "project a exit=42" in done.stdout and "project b exit=42" in done.stdout)


def make_release(path, specification, rows):
    path.mkdir(parents=True)
    db = sqlite3.connect(path / "corpus.sqlite")
    db.execute("create table corpus_release (id text, specification text)")
    db.execute("insert into corpus_release values (?, ?)", (specification["builder_commit"], json.dumps(specification)))
    db.execute("create table instruments (id text)")
    db.executemany("insert into instruments values (?)", [(r,) for r in rows])
    db.commit()
    db.close()
    (path / "manifest.json").write_text(json.dumps({"release_id": specification["builder_commit"], "database_sha256": "x" + specification["builder_commit"], "specification": specification}))


def compare_cases(tmp):
    tool = str(PROMPTS / "parser-split-g4-compare.py")
    spec = {"builder_commit": "b", "cargo_lock_sha256": "l1", "corpus_commit": "k"}
    make_release(tmp / "base", spec, ["a", "b"])
    make_release(tmp / "cand", {**spec, "builder_commit": "c", "cargo_lock_sha256": "l2"}, ["a", "b"])
    make_release(tmp / "candtwin", {**spec, "builder_commit": "c", "cargo_lock_sha256": "l2"}, ["a", "b"])
    make_release(tmp / "rowdiff", {**spec, "builder_commit": "c", "cargo_lock_sha256": "l2"}, ["a", "z"])
    make_release(tmp / "specdiff", {**spec, "builder_commit": "c", "cargo_lock_sha256": "l2", "corpus_commit": "other"}, ["a", "b"])
    check("compare control: allowed provenance differences pass", run([sys.executable, tool, str(tmp / "base"), str(tmp / "cand")], {}).returncode == 0)
    check("compare control: identical repeat builds pass --identical", run([sys.executable, tool, "--identical", str(tmp / "cand"), str(tmp / "candtwin")], {}).returncode == 0)
    check("compare: a differing row fails", run([sys.executable, tool, str(tmp / "base"), str(tmp / "rowdiff")], {}).returncode == 1)
    check("compare: a specification difference beyond the allowed fields fails", run([sys.executable, tool, str(tmp / "base"), str(tmp / "specdiff")], {}).returncode == 1)
    check("compare: allowed differences fail under --identical", run([sys.executable, tool, "--identical", str(tmp / "base"), str(tmp / "cand")], {}).returncode == 1)


def main():
    with tempfile.TemporaryDirectory() as raw:
        tmp = Path(raw)
        native_cases(tmp)
        candidate_cases(tmp)
        compare_cases(tmp)
    failed = [name for name, ok in results if not ok]
    print(f"{len(results) - len(failed)}/{len(results)} checks passed")
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
