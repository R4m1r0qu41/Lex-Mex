#!/usr/bin/env python3
"""Sequential A/B queue for docs/plans/parser-responsibility-split.md.

Runs every required run in a fixed order with the committed harness and stops
at the first non-zero exit of a run (including exit 3, which means the executing
harness or binary changed mid-run) or of a comparison (1 = drift or missing
evidence, 2 = legacy format, never acceptable here). Progress is appended to
.work/parser-split/queue.log and completed steps are recorded in
queue-state.json, so a stopped queue resumes at the failing step. A resumed step
runs again under its own label: the harness moves the interrupted attempt's logs
aside (--archive-interrupted), and every attempt's stdout goes to its own file
(<label>.out, then <label>.attempt-2.out, ...), so no earlier record is lost.
If the driver stopped after a run finished but before recording it, the resume
asks the harness to verify the result (verify-result: complete, clean, same
harness revision, binary and inputs) and records it as done without rerunning;
a result the run itself marked unusable (exit 3) is archived with its logs and
the step runs again; any other result already on disk stops the queue for an
explicit decision. State and the stop marker are written atomically.

Order: the new baseline pair B3/B4 (retained, sweep, supplement), each pair
compared strictly; then the candidates, C3 first because it is the final code,
then C2 and C1 as bisection tools, each compared with B4. Do not edit the
harness or this driver while the queue runs: any run made from a different
harness revision fails the strict comparison.
"""
import json
import os
from pathlib import Path
import subprocess
import sys
import time

REPO = Path(__file__).resolve().parent.parent
WORK = REPO / ".work" / "parser-split"
HARNESS = REPO / "prompts" / "parser-split-ab.py"
BASELINE = "85c41f3898d86737766564fa8d3fd8f6d4ae74a3"
STATE = WORK / "queue-state.json"
LOG = WORK / "queue.log"

# (kind, label-suffix, extra harness arguments)
KINDS = (("retained", []), ("sweep", []), ("supp", ["--supplement"]))


def steps():
    plan = []
    for kind, extra in KINDS:
        plan.append(("run", kind, f"{kind}-B3", "B", extra))
        plan.append(("run", kind, f"{kind}-B4", "B", extra))
        plan.append(("compare", kind, f"{kind}-B3", f"{kind}-B4", []))
    for binary in ("C3", "C2", "C1"):
        for kind, extra in KINDS:
            plan.append(("run", kind, f"{kind}-{binary}", binary, extra))
            plan.append(("compare", kind, f"{kind}-B4", f"{kind}-{binary}", []))
    return plan


def unused_path(path):
    """First of path, <stem>.attempt-2<suffix>, ... that does not exist yet."""
    candidate, attempt = path, 1
    while candidate.exists():
        attempt += 1
        candidate = path.with_name(f"{path.stem}.attempt-{attempt}{path.suffix}")
    return candidate


def write_state(state):
    temporary = STATE.with_name(STATE.name + ".tmp")
    temporary.write_text(json.dumps(state, indent=1))
    os.replace(temporary, STATE)


def harness_args(kind, label, binary, extra):
    harness_kind = {"retained": "replay", "sweep": "sweep", "supp": "replay"}[kind]
    return harness_kind, ["--binary", str(WORK / "bin" / binary / "lex-mex"), "--label", label, "--baseline", BASELINE, *extra]


def reconcile(kind, label, binary, extra):
    """True when a finished result already on disk is verified clean for this run."""
    if not (WORK / f"result-{label}.json").exists():
        return False
    harness_kind, tail = harness_args(kind, label, binary, extra)
    done = subprocess.run(
        [sys.executable, str(HARNESS), "verify-result", "--run-kind", harness_kind, *tail],
        capture_output=True, text=True,
    )
    say(f"RECONCILE {label}: rc={done.returncode} {(done.stdout + done.stderr).strip()}")
    return done.returncode == 0


def say(text):
    line = f"{time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime())} {text}"
    print(line, flush=True)
    with LOG.open("a") as handle:
        handle.write(line + "\n")


def run_command(kind, label, binary, extra):
    harness_kind, tail = harness_args(kind, label, binary, extra)
    command = [sys.executable, str(HARNESS), harness_kind, *tail, "--archive-interrupted"]
    out = unused_path(WORK / f"{label}.out")
    with out.open("w") as handle:
        return subprocess.run(command, stdout=handle, stderr=subprocess.STDOUT).returncode


def main():
    state = json.loads(STATE.read_text()) if STATE.exists() else {"done": []}
    for index, (action, kind, first, second, extra) in enumerate(steps()):
        name = f"{index:02d} {action} {first}" + (f" vs {second}" if action == "compare" else "")
        if name in state["done"]:
            continue
        attempts = state.setdefault("attempts", {})
        attempts[name] = attempts.get(name, 0) + 1
        write_state(state)
        say(f"START {name} (attempt {attempts[name]})")
        if action == "run" and reconcile(kind, first, second, extra):
            say(f"END   {name} rc=0 (reconciled an existing verified result; not rerun)")
            state["done"].append(name)
            write_state(state)
            continue
        if action == "run":
            code = run_command(kind, first, second, extra)
        else:
            out = unused_path(WORK / f"compare-{first}-vs-{second}.out")
            with out.open("w") as handle:
                code = subprocess.run(
                    [sys.executable, str(HARNESS), "compare", first, second],
                    stdout=handle, stderr=subprocess.STDOUT,
                ).returncode
        say(f"END   {name} rc={code}")
        if code != 0:
            stopped = unused_path(WORK / "queue-stopped.json")
            stopped.with_name(stopped.name + ".tmp").write_text(json.dumps({"step": name, "rc": code}))
            os.replace(stopped.with_name(stopped.name + ".tmp"), stopped)
            say(f"STOP at {name}: non-zero exit {code}")
            sys.exit(code)
        state["done"].append(name)
        write_state(state)
    say("QUEUE COMPLETE: every run succeeded and every comparison was exact")


if __name__ == "__main__":
    main()
