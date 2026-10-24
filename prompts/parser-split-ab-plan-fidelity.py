"""Offline check: the harness's expected-plan and completeness logic accepts all 15 real gate results.

Read-only: imports the harness from this directory and reads result-*, trees-* and
logs-* for the retained, sweep and supp runs (B3, B4, C3, C2, C1) under
.work/parser-split/. Exit status 0 means every result is complete for the scope the
queue requested. Guards against drift between cmd_sweep/run_stages and the plan.
"""
import importlib.util, json, sys, types
from pathlib import Path
spec = importlib.util.spec_from_file_location("ab", Path(__file__).resolve().parent / "parser-split-ab.py")
ab = importlib.util.module_from_spec(spec); spec.loader.exec_module(ab)
W = ab.WORK
def ns(**kw): return types.SimpleNamespace(supplement=False, only=None, limit=None, skip_extract=False, **kw)
bad = 0
for kind in ("retained", "sweep", "supp"):
    run_kind = "sweep" if kind == "sweep" else "replay"
    args = ns(supplement=(kind == "supp")) if False else types.SimpleNamespace(supplement=(kind == "supp"), only=None, limit=None, skip_extract=False)
    scope = ab.sweep_scope(args) if run_kind == "sweep" else ab.replay_scope(args)
    for label in ("B3", "B4", "C3", "C2", "C1"):
        name = f"{kind}-{label}"
        result = json.loads((W / f"result-{name}.json").read_text())
        trees = json.loads((W / f"trees-{name}.json").read_text())
        problem = ab.evidence_problem(result, trees, W / f"logs-{name}", scope)
        print(name, "OK" if problem is None else f"PROBLEM: {problem}", "| rows", len(result["rows"]))
        bad += problem is not None
print("scope sizes: retained slugs", len(ab.replay_scope(types.SimpleNamespace(supplement=False, only=None, limit=None, skip_extract=False))["slugs"]), "supp", len(ab.replay_scope(types.SimpleNamespace(supplement=True, only=None, limit=None, skip_extract=False))["slugs"]))
sw = ab.sweep_scope(types.SimpleNamespace(supplement=False, only=None, limit=None, skip_extract=False))
print("sweep: standards", len(sw["standards"]), "slugs", len(sw["slugs"]), "retained", len(sw["retained"]))
sys.exit(1 if bad else 0)
