#!/usr/bin/env python3
"""Acceptance self-test for the evidence rules of prompts/parser-split-ab.py.

Run after the harness is committed: python3 prompts/parser-split-ab-selftest.py
Uses the frozen baseline binary on one small instrument in its own disposable
root, writes only selftest-* artifacts under .work/parser-split/, and removes
them. Exit status 0 means every case behaved as required.

Cases:
  control          two runs of one unchanged harness compare exactly (rc 0)
  missing-both     required evidence absent from BOTH results fails (rc 1)
  missing-one      required evidence absent from ONE result fails (rc 1)
  legacy-flag      the same missing evidence under --allow-legacy-format is rc 2
  missing-row-key  a row evidence key absent from both results fails (rc 1)
  live-edit        editing the live harness file mid-run does not change what
                   executes; the run still passes and compares exactly
  edited-run       a later run launched from the edited copy carries a different
                   harness revision and is not committed, so it fails a strict
                   comparison with the control (rc 1)
  snapshot-tamper  changing the executing snapshot mid-run fails the run (rc 3)
"""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time

REPO = Path(__file__).resolve().parent.parent
HARNESS = REPO / "prompts" / "parser-split-ab.py"
WORK = REPO / ".work" / "parser-split"
BINARY = WORK / "bin" / "B" / "lex-mex"
ROOT = WORK / "selftest-root"
ENV = {k: v for k, v in os.environ.items() if k not in ("PARSER_SPLIT_FROZEN", "PARSER_SPLIT_REPO")}
ENV["PARSER_SPLIT_RUN_ROOT"] = str(ROOT)
results = []


def harness(args, script=HARNESS, background=False):
    command = [sys.executable, str(script), *args]
    if background:
        return subprocess.Popen(command, env=ENV, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    return subprocess.run(command, env=ENV, capture_output=True, text=True)


def replay(label, script=HARNESS, background=False):
    args = ["replay", "--binary", str(BINARY), "--label", label, "--only", "ccf"]
    return harness(args, script, background)


def compare(a, b, *flags):
    return harness(["compare", a, b, *flags]).returncode


def load(label):
    return json.loads((WORK / f"result-{label}.json").read_text())


def derive(src, dst, drop):
    result = load(src)
    for key in drop:
        result.pop(key, None)
    (WORK / f"result-{dst}.json").write_text(json.dumps(result, indent=1, sort_keys=True))
    shutil.copy(WORK / f"trees-{src}.json", WORK / f"trees-{dst}.json")


def derive_rows(src, dst, key):
    result = load(src)
    for row in result["rows"]:
        row.pop(key, None)
    (WORK / f"result-{dst}.json").write_text(json.dumps(result, indent=1, sort_keys=True))
    shutil.copy(WORK / f"trees-{src}.json", WORK / f"trees-{dst}.json")


def check(name, ok, detail=""):
    results.append((name, ok))
    print(f"{'PASS' if ok else 'FAIL'}  {name}  {detail}", flush=True)


VARIANT_SNAPSHOTS = []


def cleanup():
    for digest in VARIANT_SNAPSHOTS:
        (WORK / "harness-snapshots" / f"{digest}.py").unlink(missing_ok=True)
    for path in list(WORK.glob("*selftest-*")):
        shutil.rmtree(path) if path.is_dir() else path.unlink()
    shutil.rmtree(ROOT, ignore_errors=True)


def main():
    cleanup()
    try:
        # control
        for label in ("selftest-a", "selftest-b"):
            done = replay(label)
            assert done.returncode == 0, done.stderr[-400:]
        a = load("selftest-a")
        flags = [a.get(k) for k in ("harness_unchanged", "harness_frozen_snapshot", "harness_committed", "binary_unchanged")]
        check("control: evidence flags all true", all(flags), str(flags))
        check("control: two unchanged runs compare exactly", compare("selftest-a", "selftest-b") == 0)

        # missing evidence
        both = ["final_root_tree_sha256", "harness_committed"]
        derive("selftest-a", "selftest-m1", both)
        derive("selftest-b", "selftest-m2", both)
        check("missing-both: fails", compare("selftest-m1", "selftest-m2") == 1)
        check("legacy-flag: rc 2, never 0", compare("selftest-m1", "selftest-m2", "--allow-legacy-format") == 2)
        derive("selftest-b", "selftest-m3", ["final_root_tree_sha256"])
        check("missing-one: fails", compare("selftest-a", "selftest-m3") == 1)
        check("missing-one with the legacy flag: rc 2 (explicit, never a pass)", compare("selftest-a", "selftest-m3", "--allow-legacy-format") == 2)

        derive_rows("selftest-a", "selftest-r1", "stdout_sha256")
        derive_rows("selftest-b", "selftest-r2", "stdout_sha256")
        check("missing-row-key: absent from both fails", compare("selftest-r1", "selftest-r2") == 1)

        with tempfile.TemporaryDirectory() as tmp:
            # live edit: same bytes as the committed harness, edited while it runs
            live = Path(tmp) / "ab_live.py"
            live.write_bytes(HARNESS.read_bytes())
            proc = replay("selftest-live", live, background=True)
            time.sleep(8)
            live.write_text(live.read_text() + "\n# edited while the run was in progress\n")
            out, err = proc.communicate()
            check("live-edit: run still succeeds", proc.returncode == 0, err[-200:])
            if proc.returncode == 0:
                r = load("selftest-live")
                check("live-edit: result names the frozen hash", r["harness_sha256"] == a["harness_sha256"])
                check("live-edit: change is recorded", r["harness_live_source_changed_during_run"] is True)
                check("live-edit: compares exactly with the control", compare("selftest-a", "selftest-live") == 0)

            # a later run that picks up the edited file is a different revision
            edited = Path(tmp) / "ab_edited.py"
            edited.write_bytes(live.read_bytes())
            import hashlib as _h
            VARIANT_SNAPSHOTS.append(_h.sha256(edited.read_bytes()).hexdigest())
            done = replay("selftest-edited", edited)
            check("edited-run: the run itself completes", done.returncode == 0, done.stderr[-200:])
            if done.returncode == 0:
                e = load("selftest-edited")
                check("edited-run: marked not committed", e["harness_committed"] is False)
                check("edited-run: differs from the control's harness", e["harness_sha256"] != a["harness_sha256"])
                check("edited-run: strict comparison with the control fails", compare("selftest-a", "selftest-edited") == 1)

            # snapshot tamper: a variant harness, so the shared snapshot is untouched
            variant = Path(tmp) / "ab_variant.py"
            variant.write_bytes(HARNESS.read_bytes() + b"\n# selftest variant\n")
            import hashlib
            digest = hashlib.sha256(variant.read_bytes()).hexdigest()
            VARIANT_SNAPSHOTS.append(digest)
            snapshot = WORK / "harness-snapshots" / f"{digest}.py"
            proc = replay("selftest-tamper", variant, background=True)
            deadline = time.time() + 30
            while not snapshot.exists() and time.time() < deadline:
                time.sleep(0.2)
            time.sleep(6)
            snapshot.chmod(0o644)
            snapshot.write_text(snapshot.read_text() + "\n# tampered while the run was in progress\n")
            out, err = proc.communicate()
            check("snapshot-tamper: run fails with rc 3", proc.returncode == 3, f"rc={proc.returncode}")
            if (WORK / "result-selftest-tamper.json").exists():
                check("snapshot-tamper: result marks the harness changed", load("selftest-tamper")["harness_unchanged"] is False)
            snapshot.unlink(missing_ok=True)
    finally:
        cleanup()
    failed = [name for name, ok in results if not ok]
    print(f"{len(results) - len(failed)}/{len(results)} checks passed")
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
