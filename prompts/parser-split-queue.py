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

Order: the new baseline pair B3/B4 (retained, sweep, supplement), each pair
compared strictly; then the candidates, C3 first because it is the final code,
then C2 and C1 as bisection tools, each compared with B4. Do not edit the
harness or this driver while the queue runs: any run made from a different
harness revision fails the strict comparison.
"""
import json
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


def say(text):
    line = f"{time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime())} {text}"
    print(line, flush=True)
    with LOG.open("a") as handle:
        handle.write(line + "\n")


def run_command(kind, label, binary, extra):
    harness_kind = {"retained": "replay", "sweep": "sweep", "supp": "replay"}[kind]
    command = [
        sys.executable, str(HARNESS), harness_kind,
        "--binary", str(WORK / "bin" / binary / "lex-mex"),
        "--label", label, "--baseline", BASELINE, "--archive-interrupted", *extra,
    ]
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
        STATE.write_text(json.dumps(state, indent=1))
        say(f"START {name} (attempt {attempts[name]})")
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
            unused_path(WORK / "queue-stopped.json").write_text(json.dumps({"step": name, "rc": code}))
            say(f"STOP at {name}: non-zero exit {code}")
            sys.exit(code)
        state["done"].append(name)
        STATE.write_text(json.dumps(state, indent=1))
    say("QUEUE COMPLETE: every run succeeded and every comparison was exact")


if __name__ == "__main__":
    main()
