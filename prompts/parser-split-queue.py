#!/usr/bin/env python3
"""Sequential A/B queue for docs/plans/parser-responsibility-split.md.

Runs every required run in a fixed order with the committed harness and stops
at the first non-zero exit of a run (including exit 3, which means the executing
harness or binary changed mid-run) or of a comparison (1 = drift or missing
evidence, 2 = legacy format, never acceptable here). Progress is appended to
.work/parser-split/queue.log and completed steps are recorded in
queue-state.json, so a stopped queue resumes at the failing step.

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
        "--label", label, "--baseline", BASELINE, *extra,
    ]
    out = WORK / f"{label}.out"
    with out.open("w") as handle:
        return subprocess.run(command, stdout=handle, stderr=subprocess.STDOUT).returncode


def main():
    state = json.loads(STATE.read_text()) if STATE.exists() else {"done": []}
    for index, (action, kind, first, second, extra) in enumerate(steps()):
        name = f"{index:02d} {action} {first}" + (f" vs {second}" if action == "compare" else "")
        if name in state["done"]:
            continue
        say(f"START {name}")
        if action == "run":
            code = run_command(kind, first, second, extra)
        else:
            out = WORK / f"compare-{first}-vs-{second}.out"
            with out.open("w") as handle:
                code = subprocess.run(
                    [sys.executable, str(HARNESS), "compare", first, second],
                    stdout=handle, stderr=subprocess.STDOUT,
                ).returncode
        say(f"END   {name} rc={code}")
        if code != 0:
            (WORK / "queue-stopped.json").write_text(json.dumps({"step": name, "rc": code}))
            say(f"STOP at {name}: non-zero exit {code}")
            sys.exit(code)
        state["done"].append(name)
        STATE.write_text(json.dumps(state, indent=1))
    say("QUEUE COMPLETE: every run succeeded and every comparison was exact")


if __name__ == "__main__":
    main()
