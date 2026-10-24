#!/usr/bin/env python3
"""Row-level comparison of two projected DB releases. usage: g4_compare.py DIR_A DIR_B"""
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

def main():
    a, b = (Path(x).resolve() for x in sys.argv[1:3])
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
    print(json.dumps(report, indent=1))

main()
