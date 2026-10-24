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
  restart-refused  an existing logs-<label> without --archive-interrupted is
                   refused and left byte-identical
  restart-archive  with --archive-interrupted the interrupted logs are renamed
                   aside intact, the attempt is recorded, and the label runs
  restart-finished a label with a finished result is refused even with the flag
  driver-outputs   the queue driver never reuses (truncates) an existing output
                   file name
  verify-result    verify-result accepts a clean result and rejects one with a
                   different binary, a missing trees file or a changed-run flag
  unusable-result  a result the run marked unusable (exit 3) is archived with its
                   logs, byte-identical, and the label runs again
  result-no-logs   a usable result without a logs directory is refused, not
                   overwritten, even with --archive-interrupted
  completeness     verify-result rejects empty tree maps and log directories,
                   a missing or empty rows list, a missing work-tree digest, a
                   tampered stage tree, a tampered log and a dropped row; it
                   accepts an intentional stop after a failed parse and a result
                   whose recorded scope is exactly the requested one
  subset-as-full   a run restricted with --only is rejected when the full scope
                   was requested, by verify-result and by the driver
  driver-reconcile the driver records a verified existing result as done without
                   rerunning, and rejects an unusable one; its state write is atomic
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
    # A copy outside the repository must be told where the repository is.
    env = ENV if script == HARNESS else {**ENV, "PARSER_SPLIT_REPO": str(REPO)}
    if background:
        return subprocess.Popen(command, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    return subprocess.run(command, env=env, capture_output=True, text=True)


def replay(label, script=HARNESS, background=False, extra=()):
    args = ["replay", "--binary", str(BINARY), "--label", label, "--only", "ccf", *extra]
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

        # same-label restart (R-001)
        interrupted = WORK / "logs-selftest-restart"
        interrupted.mkdir()
        (interrupted / "ccf.parse.stdout").write_text("partial evidence from an interrupted attempt\n")
        before = sorted((p.name, p.read_bytes()) for p in interrupted.iterdir())
        done = replay("selftest-restart")
        check("restart-refused: existing logs without the flag fail", done.returncode != 0 and "--archive-interrupted" in done.stderr + done.stdout)
        check("restart-refused: logs left byte-identical", sorted((p.name, p.read_bytes()) for p in interrupted.iterdir()) == before)
        done = replay("selftest-restart", extra=["--archive-interrupted"])
        check("restart-archive: label runs again", done.returncode == 0, done.stderr[-200:])
        moved = list(WORK.glob("logs-selftest-restart.interrupted-*"))
        check("restart-archive: interrupted logs kept intact", len(moved) == 1 and sorted((p.name, p.read_bytes()) for p in moved[0].iterdir()) == before)
        record = WORK / "attempts-selftest-restart.json"
        check("restart-archive: attempt recorded", record.exists() and len(json.loads(record.read_text())) == 1)
        if done.returncode == 0:
            check("restart-archive: new run is clean evidence", all(load("selftest-restart")[k] for k in ("harness_unchanged", "harness_committed", "binary_unchanged")))
            check("restart-archive: restarted run compares exactly with the control", compare("selftest-a", "selftest-restart") == 0)
        done = replay("selftest-restart", extra=["--archive-interrupted"])
        check("restart-finished: finished label refused even with the flag", done.returncode != 0 and "finished result" in done.stderr + done.stdout)

        # verify-result and result-present recovery (A-002)
        def verify(label, binary=BINARY, only="ccf"):
            tail = ["--only", only] if only else []
            return harness(["verify-result", "--run-kind", "replay", "--binary", str(binary), "--label", label, "--baseline", load(label)["baseline"], *tail]).returncode
        check("verify-result: a clean result is accepted", verify("selftest-restart") == 0)
        check("verify-result: a different binary is rejected", verify("selftest-restart", WORK / "bin" / "C3" / "lex-mex") == 1)
        shutil.copy(WORK / "trees-selftest-restart.json", WORK / "trees-selftest-restart.keep")
        (WORK / "trees-selftest-restart.json").unlink()
        check("verify-result: a missing trees file is rejected", verify("selftest-restart") == 1)
        shutil.move(WORK / "trees-selftest-restart.keep", WORK / "trees-selftest-restart.json")
        marked = load("selftest-restart")
        marked["binary_unchanged"] = False
        (WORK / "result-selftest-unusable.json").write_text(json.dumps(marked, indent=1, sort_keys=True))
        shutil.copy(WORK / "trees-selftest-restart.json", WORK / "trees-selftest-unusable.json")
        (WORK / "logs-selftest-unusable").mkdir()
        (WORK / "logs-selftest-unusable" / "ccf.parse.stdout").write_text("log of a run that exited 3\n")
        before_result = (WORK / "result-selftest-unusable.json").read_bytes()
        done = replay("selftest-unusable")
        check("unusable-result: refused without the flag", done.returncode != 0)
        done = replay("selftest-unusable", extra=["--archive-interrupted"])
        check("unusable-result: label runs again with the flag", done.returncode == 0, done.stderr[-200:])
        kept = list(WORK.glob("result-selftest-unusable.json.interrupted-*"))
        check("unusable-result: old result kept byte-identical", len(kept) == 1 and kept[0].read_bytes() == before_result)
        check("unusable-result: old logs and trees kept", len(list(WORK.glob("logs-selftest-unusable.interrupted-*"))) == 1 and len(list(WORK.glob("trees-selftest-unusable.json.interrupted-*"))) == 1)
        check("unusable-result: attempt records the result", json.loads((WORK / "attempts-selftest-unusable.json").read_text())[0]["had_result"] is True)
        check("unusable-result: new result is clean", load("selftest-unusable")["binary_unchanged"] is True)
        shutil.copy(WORK / "result-selftest-restart.json", WORK / "result-selftest-nologs.json")
        done = replay("selftest-nologs", extra=["--archive-interrupted"])
        check("result-no-logs: usable result refused, not overwritten", done.returncode != 0 and (WORK / "result-selftest-nologs.json").read_bytes() == (WORK / "result-selftest-restart.json").read_bytes())

        # completeness and scope (A-002 follow-up)
        check("subset-as-full: a --only run is rejected when the full scope is requested", verify("selftest-restart", only=None) == 1)

        def clone_run(src, dst, mutate_result=None, mutate_trees=None, mutate_logs=None):
            result = load(src)
            result["label"] = dst
            if mutate_result:
                mutate_result(result)
            trees = json.loads((WORK / f"trees-{src}.json").read_text())
            if mutate_trees:
                trees = mutate_trees(trees)
            (WORK / f"result-{dst}.json").write_text(json.dumps(result, indent=1, sort_keys=True))
            (WORK / f"trees-{dst}.json").write_text(json.dumps(trees, sort_keys=True))
            shutil.copytree(WORK / f"logs-{src}", WORK / f"logs-{dst}")
            if mutate_logs:
                mutate_logs(WORK / f"logs-{dst}")
            return dst

        def empty_logs(logs):
            for path in logs.iterdir():
                path.unlink()

        def tamper_log(logs):
            target = sorted(logs.glob("*.stdout"))[0]
            target.write_text(target.read_text() + "tampered\n")

        def drop_work_key(result):
            result["rows"][0].pop("work_tree_sha256", None)

        def break_tree(trees):
            slug = sorted(trees)[0]
            stage = sorted(trees[slug])[0]
            trees[slug][stage]["corpus"] = {"tampered": "x"}
            return trees

        def drop_row(result):
            result["rows"].pop(1)

        cases = (
            ("empty tree maps and empty log directory", dict(mutate_trees=lambda t: {}, mutate_logs=empty_logs)),
            ("rows key absent", dict(mutate_result=lambda r: r.pop("rows"))),
            ("rows empty", dict(mutate_result=lambda r: r.update(rows=[]))),
            ("work-tree digest absent from a row", dict(mutate_result=drop_work_key)),
            ("a stage tree that does not match its digest", dict(mutate_trees=break_tree)),
            ("a log that does not match its hash", dict(mutate_logs=tamper_log)),
            ("a dropped row (missing stage)", dict(mutate_result=drop_row)),
        )
        for index, (title, mutation) in enumerate(cases):
            name = clone_run("selftest-restart", f"selftest-bad{index}", **mutation)
            check(f"completeness: {title} is rejected", verify(name) == 1)
        check("completeness: the unmutated control with exactly the requested scope is accepted", verify(clone_run("selftest-restart", "selftest-good")) == 0)
        done = harness(["replay", "--binary", str(BINARY), "--label", "selftest-parsefail", "--only", "lfiiedb"])
        check("completeness: the parse-failure run completes", done.returncode == 0, done.stderr[-200:])
        if done.returncode == 0:
            rows = load("selftest-parsefail")["rows"]
            check("completeness: parse failed and later stages were skipped (the baseline fact)", len(rows) == 1 and rows[0]["stage"] == "parse" and rows[0]["exit"] != 0)
            check("completeness: an intentional stop after a failed parse is accepted", verify("selftest-parsefail", only="lfiiedb") == 0)

        # driver never truncates an earlier attempt's output
        import importlib.util
        spec = importlib.util.spec_from_file_location("queue_driver", REPO / "prompts" / "parser-split-queue.py")
        driver = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(driver)
        with tempfile.TemporaryDirectory() as tmp:
            first = Path(tmp) / "label.out"
            check("driver-outputs: fresh name is used as is", driver.unused_path(first) == first)
            first.write_text("attempt one")
            second = driver.unused_path(first)
            second.write_text("attempt two")
            third = driver.unused_path(first)
            check("driver-outputs: later attempts get their own names", second.name == "label.attempt-2.out" and third.name == "label.attempt-3.out")
            check("driver-outputs: earlier output untouched", first.read_text() == "attempt one")

            # driver reconciles a finished-but-unrecorded result
            driver.BASELINE = load("selftest-restart")["baseline"]
            driver.LOG = Path(tmp) / "queue.log"  # keep the real queue log out of the test
            check("driver-reconcile: a verified result is accepted without a rerun", driver.reconcile("retained", "selftest-restart", "B", ["--only", "ccf"]) is True)
            check("subset-as-full: the driver rejects a --only result for the full scope", driver.reconcile("retained", "selftest-restart", "B", []) is False)
            bad = load("selftest-restart")
            bad["harness_unchanged"] = False
            bad["label"] = "selftest-nologs-bad"  # the flag must be the only defect
            (WORK / "result-selftest-nologs-bad.json").write_text(json.dumps(bad, indent=1, sort_keys=True))
            shutil.copy(WORK / "trees-selftest-restart.json", WORK / "trees-selftest-nologs-bad.json")
            (WORK / "logs-selftest-nologs-bad").mkdir()
            check("driver-reconcile: an unusable existing result is rejected", driver.reconcile("retained", "selftest-nologs-bad", "B", ["--only", "ccf"]) is False)
            driver.STATE = Path(tmp) / "queue-state.json"
            driver.write_state({"done": ["x"]})
            check("driver-reconcile: state write is atomic and complete", json.loads(driver.STATE.read_text()) == {"done": ["x"]} and not list(Path(tmp).glob("*.tmp")))
    finally:
        cleanup()
    failed = [name for name, ok in results if not ok]
    print(f"{len(results) - len(failed)}/{len(results)} checks passed")
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
