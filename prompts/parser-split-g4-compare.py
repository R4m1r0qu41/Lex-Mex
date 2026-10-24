#!/usr/bin/env python3
"""Row-level comparison of two projected DB releases.

usage: parser-split-g4-compare.py [--identical] DIR_A DIR_B

Prints a JSON report and exits 1 when parity fails. Default (baseline vs
candidate): every table must be equal, except that corpus_release may differ in
its id and in the specification fields builder_commit and cargo_lock_sha256, and
the manifest may differ only in release_id, database_sha256 and those two
fields. --identical (repeat builds of one candidate): nothing may differ.
"""
import hashlib, json, sqlite3, sys
from pathlib import Path

SHADOW = ("provisions_fts_config", "provisions_fts_data", "provisions_fts_docsize", "provisions_fts_idx")

def tables(c):
    return [n for (n,) in c.execute("select name from sqlite_master where type='table' and name not like 'sqlite_%' order by name") if n not in SHADOW]

def table_digest(c, name):
    cols = [r[1] for r in c.execute(f'pragma table_info("{name}")')]
    order = ", ".join(f'"{col}"' for col in cols)
    h = hashlib.sha256(); n = 0
    for row in c.execute(f'select * from "{name}" order by {order}'):
        h.update(json.dumps(row, ensure_ascii=False, default=repr).encode()); h.update(b"\n"); n += 1
    return n, h.hexdigest()

def flat(o, p=""):
    if isinstance(o, dict):
        for k, v in o.items(): yield from flat(v, f"{p}.{k}" if p else k)
    elif isinstance(o, list):
        for i, v in enumerate(o): yield from flat(v, f"{p}[{i}]")
    else:
        yield p, o

ALLOWED_MANIFEST = {"database_sha256", "release_id", "specification.builder_commit", "specification.cargo_lock_sha256"}
ALLOWED_SPEC = {"builder_commit", "cargo_lock_sha256"}


def spec_difference(ca, cb):
    spec = lambda c: json.loads(c.execute("select specification from corpus_release").fetchone()[0])
    fa, fb = dict(flat(spec(ca))), dict(flat(spec(cb)))
    return {k for k in set(fa) | set(fb) if fa.get(k) != fb.get(k)}


def main():
    args = [x for x in sys.argv[1:] if x != "--identical"]
    identical = "--identical" in sys.argv[1:]
    a, b = (Path(x).resolve() for x in args[:2])
    ca = sqlite3.connect(f"file:{a/'corpus.sqlite'}?mode=ro", uri=True)
    cb = sqlite3.connect(f"file:{b/'corpus.sqlite'}?mode=ro", uri=True)
    report = {"a": str(a), "b": str(b), "tables": {}, "table_set_equal": tables(ca) == tables(cb)}
    for name in tables(ca):
        da, db = table_digest(ca, name), table_digest(cb, name)
        report["tables"][name] = {"rows_a": da[0], "rows_b": db[0], "equal": da == db}
    ma, mb = json.load(open(a/"manifest.json")), json.load(open(b/"manifest.json"))
    fa, fb = dict(flat(ma)), dict(flat(mb))
    report["manifest_differences"] = {k: [fa.get(k), fb.get(k)] for k in sorted(set(fa) | set(fb)) if fa.get(k) != fb.get(k)}
    report["unequal_tables"] = [n for n, v in report["tables"].items() if not v["equal"]]
    problems = []
    if not report["table_set_equal"]:
        problems.append("table sets differ")
    allowed_tables = set() if identical else {"corpus_release"}
    for name in report["unequal_tables"]:
        if name not in allowed_tables:
            problems.append(f"table {name} differs")
    if not identical and "corpus_release" in report["unequal_tables"]:
        extra = spec_difference(ca, cb) - ALLOWED_SPEC
        if extra:
            problems.append(f"corpus_release specification differs beyond the allowed fields: {sorted(extra)}")
    allowed_manifest = set() if identical else ALLOWED_MANIFEST
    for key in report["manifest_differences"]:
        if key not in allowed_manifest:
            problems.append(f"manifest field {key} differs")
    report["mode"] = "identical" if identical else "baseline-vs-candidate"
    report["parity"] = not problems
    report["problems"] = problems
    print(json.dumps(report, indent=1))
    if problems:
        print("PARITY FAIL: " + "; ".join(problems), file=sys.stderr)
        sys.exit(1)


main()
