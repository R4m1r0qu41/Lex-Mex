#!/usr/bin/env python3
"""Deterministic A/B replay harness for docs/plans/parser-responsibility-split.md.

Subcommands (all outputs land in the ignored .work/parser-split/ directory):

  inventory   public lex-parse symbols and their first-party callers
  freeze      clone retained extracted-text inputs (APFS clone, read-only source)
              and write the hash-bound coverage manifest
  replay      run retained-source stages through ONE frozen binary
  sweep       run whole-corpus stages on the pristine committed corpus
  compare     byte-compare two replay/sweep result files

Every run rebuilds its disposable root from `git archive <baseline>` at the same
absolute path, with a scrubbed environment, so any path a binary embeds is
identical across runs and no normalization is needed. Comparison is byte-exact
on exit code, stdout, stderr and every file written; there is no scrubber.
Nothing here touches the network or the authoritative corpus.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time

REPO = Path(__file__).resolve().parent.parent
WORK = REPO / ".work" / "parser-split"
RUN_ROOT = Path(os.environ.get("PARSER_SPLIT_RUN_ROOT", WORK / "run-root"))
INPUTS = WORK / "inputs"
INPUTS_SUPP = WORK / "inputs-supplement"
DEFAULT_SOURCE_WORK = Path("/Users/jr/Dev/lex-mex/.work")

# Offline stages for one instrument that has retained extracted text. The
# pipeline command also fetches, so the stages it chains are replayed here.
RETAINED_STAGES = [
    ("parse", ["parse", "{slug}"]),
    ("temporal-request", ["analyze-temporal", "{slug}", "--provider", "none"]),
    ("validate", ["validate", "{slug}"]),
    ("derive-temporal", ["derive-temporal", "{slug}"]),
    ("export-markdown", ["export", "{slug}", "--format", "markdown"]),
]
SWEEP_STAGES = [
    ("derive-temporal", ["derive-temporal", "{slug}"]),
    ("validate", ["validate", "{slug}"]),
    ("export-json", ["export", "{slug}", "--format", "json"]),
    ("export-markdown", ["export", "{slug}", "--format", "markdown"]),
]


def sha256_bytes(data):
    return hashlib.sha256(data).hexdigest()


def sha256_file(path):
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def tree_map(base, sub="", exclude=()):
    """path -> sha256 for every file under base/sub, sorted, relative to base."""
    start = base / sub if sub else base
    rows = {}
    if not start.exists():
        return rows
    for path in sorted(start.rglob("*")):
        if exclude and path.relative_to(base).parts[0] in exclude:
            continue
        if path.is_file() and not path.is_symlink():
            rows[str(path.relative_to(base))] = sha256_file(path)
        elif path.is_symlink():
            rows[str(path.relative_to(base))] = "symlink:" + os.readlink(path)
    return rows


def digest_map(rows):
    return sha256_bytes(json.dumps(rows, sort_keys=True).encode())


def git(*args):
    return subprocess.run(
        ["git", "-C", str(REPO), *args], check=True, capture_output=True, text=True
    ).stdout.strip()


def scrubbed_env():
    # Nothing from the operator shell leaks in: no LEX_MEX_ROOT, no vault.
    return {
        "PATH": "/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin",
        "HOME": os.environ["HOME"],
        "TZ": "UTC",
        "LANG": "en_US.UTF-8",
        "LC_ALL": "en_US.UTF-8",
    }


def adapter_rows():
    rows = {}
    for path in sorted((REPO / "adapters").rglob("*.json")):
        if path.name.startswith("_"):
            continue
        try:
            data = json.loads(path.read_text())
        except json.JSONDecodeError:
            continue
        if isinstance(data, dict) and "slug" in data:
            rows[data["slug"]] = {
                "adapter": str(path.relative_to(REPO)),
                "parser": data.get("parser"),
                "source_format": data.get("source_format"),
            }
    return rows


def committed_slugs():
    base = REPO / "corpus" / "mx"
    return sorted(p.name for p in base.iterdir() if p.is_dir())


# --------------------------------------------------------------------------
# inventory
# --------------------------------------------------------------------------
ITEM = re.compile(r"^pub (?:(?:async|const|unsafe) )*(fn|struct|enum|const|type|trait|static|mod|use)\b\s*(.*)")


def cmd_inventory(args):
    src = REPO / "crates" / "lex-parse" / "src"
    items = []
    for path in sorted(src.glob("*.rs")):
        module = "" if path.stem == "lib" else path.stem
        for number, line in enumerate(path.read_text().splitlines(), 1):
            match = ITEM.match(line)
            if not match:
                continue
            kind, rest = match.groups()
            if kind == "use":
                name = rest.rstrip(";")
            else:
                name = re.split(r"[\s(<:;{=]", rest, 1)[0]
            items.append(
                {"module": module, "kind": kind, "name": name, "file": str(path.relative_to(REPO)), "line": number}
            )
    consumers = [
        p
        for crate in ("lex-cli", "lex-db", "lex-export")
        for p in sorted((REPO / "crates" / crate).rglob("*.rs"))
        if "lex_parse" in p.read_text()
    ]
    texts = {p: p.read_text() for p in consumers}
    for item in items:
        if item["kind"] in {"use", "mod"}:
            item["callers"] = []
            continue
        pattern = re.compile(r"\b" + re.escape(item["name"]) + r"\b")
        item["callers"] = [
            f"{p.relative_to(REPO)}:{sum(1 for _ in pattern.finditer(t))}"
            for p, t in texts.items()
            if pattern.search(t)
        ]
    out = {
        "baseline": git("rev-parse", "HEAD"),
        "item_count": len(items),
        "consumer_files": [str(p.relative_to(REPO)) for p in consumers],
        "items": items,
    }
    WORK.mkdir(parents=True, exist_ok=True)
    target = WORK / "api-inventory.json"
    target.write_text(json.dumps(out, indent=1, sort_keys=True) + "\n")
    print(f"{len(items)} public items; {len(consumers)} consumer files -> {target}")


# --------------------------------------------------------------------------
# freeze
# --------------------------------------------------------------------------
def cmd_freeze(args):
    source_work = Path(args.source_work)
    adapters = adapter_rows()
    committed = committed_slugs()
    if INPUTS.exists():
        sys.exit(f"refusing to replace frozen inputs at {INPUTS}")
    INPUTS.mkdir(parents=True)
    coverage = []
    for slug in sorted(set(adapters) | set(committed)):
        row = {"slug": slug, **adapters.get(slug, {"adapter": None, "parser": None, "source_format": None})}
        row["committed"] = slug in committed
        manifest_path = REPO / "corpus" / "mx" / slug / "source-manifest.json"
        manifest_hash = None
        if manifest_path.exists():
            manifest_hash = json.loads(manifest_path.read_text()).get("extracted_text_sha256")
        text = source_work / slug / f"{slug}.txt"
        if slug not in adapters:
            row.update(replayed=False, reason="no adapter configuration under adapters/")
        elif not row["committed"]:
            row.update(replayed=False, reason="not a committed instrument")
        elif not text.exists():
            row.update(replayed=False, reason="retained extracted text absent from source .work")
        else:
            subprocess.run(["cp", "-cR", str(source_work / slug), str(INPUTS / slug)], check=True)
            frozen_text = INPUTS / slug / f"{slug}.txt"
            row.update(
                replayed=True,
                reason=None,
                retained_text_sha256=sha256_file(frozen_text),
                manifest_extracted_text_sha256=manifest_hash,
                text_matches_manifest=sha256_file(frozen_text) == manifest_hash,
                input_files=len(tree_map(INPUTS, slug)),
            )
        coverage.append(row)
    inputs_tree = tree_map(INPUTS)
    manifest = {
        "baseline": git("rev-parse", "HEAD"),
        "source_work": str(source_work),
        "replay_order": [r["slug"] for r in coverage if r["replayed"]],
        "inputs_tree_sha256": digest_map(inputs_tree),
        "input_file_count": len(inputs_tree),
        "coverage": coverage,
    }
    (WORK / "coverage-manifest.json").write_text(json.dumps(manifest, indent=1, sort_keys=True) + "\n")
    replayed = sum(1 for r in coverage if r["replayed"])
    print(f"froze {replayed}/{len(coverage)} slugs; {len(inputs_tree)} input files; digest {manifest['inputs_tree_sha256']}")


def cmd_freeze_supplement(args):
    """Freeze inputs recovered by hash from other local work trees (a recorded search)."""
    found = json.loads(Path(args.search).read_text())["recovered"]
    if INPUTS_SUPP.exists():
        sys.exit(f"refusing to replace frozen inputs at {INPUTS_SUPP}")
    INPUTS_SUPP.mkdir(parents=True)
    rows = []
    for slug, hits in sorted(found.items()):
        text = next((Path(path) for kind, path in hits if kind == "text"), None)
        if text is None:
            rows.append({"slug": slug, "replayed": False, "reason": "only the source file matched by hash"})
            continue
        subprocess.run(["cp", "-cR", str(text.parent), str(INPUTS_SUPP / slug)], check=True)
        manifest = json.loads((REPO / "corpus" / "mx" / slug / "source-manifest.json").read_text())
        frozen = INPUTS_SUPP / slug / f"{slug}.txt"
        rows.append(
            {
                "slug": slug,
                "replayed": True,
                "recovered_from": str(text.parent),
                "retained_text_sha256": sha256_file(frozen),
                "manifest_extracted_text_sha256": manifest.get("extracted_text_sha256"),
                "text_matches_manifest": sha256_file(frozen) == manifest.get("extracted_text_sha256"),
            }
        )
    tree = tree_map(INPUTS_SUPP)
    out = {
        "baseline": json.loads((WORK / "coverage-manifest.json").read_text())["baseline"],
        "search": str(args.search),
        "replay_order": [r["slug"] for r in rows if r["replayed"]],
        "inputs_tree_sha256": digest_map(tree),
        "input_file_count": len(tree),
        "coverage": rows,
    }
    (WORK / "coverage-manifest-supplement.json").write_text(json.dumps(out, indent=1, sort_keys=True) + "\n")
    print(f"froze {len(out['replay_order'])} supplementary slugs; digest {out['inputs_tree_sha256']}")


# --------------------------------------------------------------------------
# replay / sweep
# --------------------------------------------------------------------------
def build_pristine_root(baseline, git_init=False):
    if RUN_ROOT.exists():
        shutil.rmtree(RUN_ROOT)
    RUN_ROOT.mkdir(parents=True)
    archive = subprocess.Popen(["git", "-C", str(REPO), "archive", baseline], stdout=subprocess.PIPE)
    subprocess.run(["tar", "-x", "-C", str(RUN_ROOT)], stdin=archive.stdout, check=True)
    archive.stdout.close()
    if archive.wait() != 0:
        sys.exit("git archive failed")
    if git_init:
        # `bundle` reads HEAD and requires its corpora to be committed, so the
        # sweep root is its own repository with a fixed, reproducible commit.
        fixed = {
            **scrubbed_env(),
            "GIT_AUTHOR_NAME": "ab",
            "GIT_AUTHOR_EMAIL": "ab@example.invalid",
            "GIT_COMMITTER_NAME": "ab",
            "GIT_COMMITTER_EMAIL": "ab@example.invalid",
            "GIT_AUTHOR_DATE": "2026-01-01T00:00:00+0000",
            "GIT_COMMITTER_DATE": "2026-01-01T00:00:00+0000",
        }
        for argv in (["init", "-q"], ["add", "-A"], ["commit", "-q", "-m", "pristine"]):
            subprocess.run(["git", *argv], cwd=RUN_ROOT, env=fixed, check=True, capture_output=True)


def run_command(binary, argv, logs, tag):
    started = time.monotonic()
    result = subprocess.run(
        [str(binary), "--root", str(RUN_ROOT), *argv],
        cwd=RUN_ROOT,
        env=scrubbed_env(),
        capture_output=True,
    )
    elapsed = time.monotonic() - started
    (logs / f"{tag}.stdout").write_bytes(result.stdout)
    (logs / f"{tag}.stderr").write_bytes(result.stderr)
    return {
        "argv": argv,
        "exit": result.returncode,
        "stdout_sha256": sha256_bytes(result.stdout),
        "stderr_sha256": sha256_bytes(result.stderr),
        "elapsed_s": round(elapsed, 3),
    }


def pick(slugs, args):
    if args.only:
        wanted = args.only.split(",")
        slugs = [s for s in slugs if s in wanted]
    if args.limit:
        slugs = slugs[: args.limit]
    return slugs


def run_stages(binary, stages, slugs, logs, trees, with_work):
    rows = []
    for slug in slugs:
        for name, template in stages:
            argv = [part.format(slug=slug) for part in template]
            row = run_command(binary, argv, logs, f"{slug}.{name}")
            corpus = tree_map(RUN_ROOT, f"corpus/mx/{slug}")
            row.update(slug=slug, stage=name, corpus_tree_sha256=digest_map(corpus))
            if with_work:
                work = tree_map(RUN_ROOT, f".work/{slug}")
                row["work_tree_sha256"] = digest_map(work)
                trees.setdefault(slug, {}).setdefault(name, {})["work"] = work
            trees.setdefault(slug, {}).setdefault(name, {})["corpus"] = corpus
            rows.append(row)
            print(f"{slug} {name} exit={row['exit']} {row['elapsed_s']}s", flush=True)
            if name == "parse" and row["exit"] != 0:
                break
    return rows


def finish(label, binary, kind, baseline, rows, trees, extra):
    final_corpus = tree_map(RUN_ROOT, "corpus/mx")
    result = {
        "label": label,
        "kind": kind,
        "binary": str(binary),
        "binary_sha256": sha256_file(binary),
        "baseline": baseline,
        "git_head": git("rev-parse", "HEAD"),
        "harness_sha256": sha256_file(Path(__file__)),
        "final_root_tree_sha256": digest_map(tree_map(RUN_ROOT, exclude=(".git",))),
        "rows": rows,
        "final_corpus_tree_sha256": digest_map(final_corpus),
        "final_corpus_file_count": len(final_corpus),
        **extra,
    }
    (WORK / f"result-{label}.json").write_text(json.dumps(result, indent=1, sort_keys=True) + "\n")
    (WORK / f"trees-{label}.json").write_text(json.dumps(trees, sort_keys=True))
    failures = [r for r in rows if r["exit"] != 0]
    print(f"{label}: {len(rows)} commands, {len(failures)} nonzero; corpus digest {result['final_corpus_tree_sha256']}")


def prepare(args, git_init=False):
    binary = Path(args.binary).resolve()
    baseline = args.baseline or json.loads((WORK / "coverage-manifest.json").read_text())["baseline"]
    logs = WORK / f"logs-{args.label}"
    if logs.exists():
        sys.exit(f"refusing to overwrite {logs}; choose a fresh --label")
    logs.mkdir(parents=True)
    build_pristine_root(baseline, git_init)
    pristine = tree_map(RUN_ROOT, "corpus")
    tree_id = git("rev-parse", f"{baseline}:corpus")
    return binary, baseline, logs, {"pristine_corpus_digest": digest_map(pristine), "baseline_corpus_git_tree": tree_id}


def cmd_replay(args):
    binary, baseline, logs, extra = prepare(args)
    inputs = INPUTS_SUPP if args.supplement else INPUTS
    manifest_name = "coverage-manifest-supplement.json" if args.supplement else "coverage-manifest.json"
    manifest = json.loads((WORK / manifest_name).read_text())
    slugs = pick(manifest["replay_order"], args)
    (RUN_ROOT / ".work").mkdir(exist_ok=True)
    for slug in slugs:
        subprocess.run(["cp", "-cR", str(inputs / slug), str(RUN_ROOT / ".work" / slug)], check=True)
    extra["inputs_tree_sha256"] = manifest["inputs_tree_sha256"]
    trees = {}
    rows = run_stages(binary, RETAINED_STAGES, slugs, logs, trees, with_work=True)
    finish(args.label, binary, "retained-replay", baseline, rows, trees, extra)


def global_step(binary, logs, rows, slug, stage, argv, watch=None):
    row = {"slug": slug, "stage": stage, **run_command(binary, argv, logs, f"{slug}.{stage}")}
    row["corpus_tree_sha256"] = digest_map(tree_map(RUN_ROOT, "corpus/mx"))
    row["out_tree_sha256"] = digest_map(tree_map(RUN_ROOT, ".out"))
    row["packets_tree_sha256"] = digest_map(tree_map(RUN_ROOT, "review-packets"))
    rows.append(row)
    print(f"{stage} exit={row['exit']} {row['elapsed_s']}s", flush=True)


def cmd_sweep(args):
    binary, baseline, logs, extra = prepare(args, git_init=True)
    extra["pristine_head"] = subprocess.run(
        ["git", "rev-parse", "HEAD"], cwd=RUN_ROOT, capture_output=True, text=True
    ).stdout.strip()
    extra["started_utc"] = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
    out = RUN_ROOT / ".out"
    out.mkdir()
    trees = {}
    rows = []
    g = lambda stage, argv: global_step(binary, logs, rows, "*", stage, argv)
    # Bundles first: they require an unmodified, committed corpus.
    g("bundle-canonical", ["bundle", "create", "--instrument", "lritf,ifpe-dcg-2021", "--profile", "canonical", "--output", str(out / "bundle-a")])
    g("bundle-markdown", ["bundle", "create", "--instrument", "ccf,cff", "--profile", "canonical-markdown", "--output", str(out / "bundle-b")])
    g("link-all", ["link", "--all"])
    g("instruments", ["instruments", "--json"])
    g("path", ["path"])
    g("search", ["search", "artículo", "--scope", "canonical", "--", "--sort", "path", "-c"])
    g("review-list", ["review", "list", "--all"])
    g("review-legacy", ["review", "legacy", "--all-instruments"])
    g("review-packets-generate", ["review-packets", "generate"])
    g("review-packets-list", ["review-packets", "list"])
    adapters = adapter_rows()
    standards = [s for s in committed_slugs() if s not in adapters]
    for slug in standards:
        for stage, argv in (("standards-validate", ["standards", "validate", slug]), ("standards-refresh", ["standards", "refresh", slug])):
            global_step(binary, logs, rows, slug, stage, argv)
    slugs = pick([s for s in committed_slugs() if s in adapters], args)
    rows.extend(run_stages(binary, SWEEP_STAGES, slugs, logs, trees, with_work=False))
    # Re-extraction from retained sources runs last: it rewrites manifests.
    if not args.skip_extract:
        manifest = json.loads((WORK / "coverage-manifest.json").read_text())
        retained = pick(manifest["replay_order"], args)
        (RUN_ROOT / ".work").mkdir(exist_ok=True)
        for slug in retained:
            subprocess.run(["cp", "-cR", str(INPUTS / slug), str(RUN_ROOT / ".work" / slug)], check=True)
        extra["inputs_tree_sha256"] = manifest["inputs_tree_sha256"]
        for slug in retained:
            pdf = RUN_ROOT / ".work" / slug / f"{slug}.pdf"
            stages = [("extract", ["extract", slug])]
            if pdf.exists():
                stages.append(("audit-furniture", ["audit-furniture", f".work/{slug}/{slug}.pdf", "--json"]))
            for name, argv in stages:
                row = run_command(binary, argv, logs, f"{slug}.{name}")
                work = tree_map(RUN_ROOT, f".work/{slug}")
                corpus = tree_map(RUN_ROOT, f"corpus/mx/{slug}")
                row.update(slug=slug, stage=name, work_tree_sha256=digest_map(work), corpus_tree_sha256=digest_map(corpus))
                trees.setdefault(slug, {}).setdefault(name, {}).update(work=work, corpus=corpus)
                rows.append(row)
                print(f"{slug} {name} exit={row['exit']} {row['elapsed_s']}s", flush=True)
    # Final mutating steps for command paths the stages above do not reach.
    # They run last so the main sweep stays on the pristine committed corpus.
    g("link-lritf", ["link", "lritf"])
    g("link-ifpe", ["link", "ifpe-dcg-2021"])
    g("repair-dates-lritf", ["derive-temporal", "lritf", "--repair-dates-only"])
    g("repair-dates-ccf", ["derive-temporal", "ccf", "--repair-dates-only"])
    g("db-contract-probe", ["db", "contract-probe"])
    extra["finished_utc"] = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
    finish(args.label, binary, "whole-corpus-sweep", baseline, rows, trees, extra)


# --------------------------------------------------------------------------
# compare
# --------------------------------------------------------------------------
def cmd_compare(args):
    a = json.loads((WORK / f"result-{args.a}.json").read_text())
    b = json.loads((WORK / f"result-{args.b}.json").read_text())
    ta = json.loads((WORK / f"trees-{args.a}.json").read_text())
    tb = json.loads((WORK / f"trees-{args.b}.json").read_text())
    ignore = {"elapsed_s"}
    drift = []
    if len(a["rows"]) != len(b["rows"]):
        drift.append({"kind": "row-count", "a": len(a["rows"]), "b": len(b["rows"])})
    for ra, rb in zip(a["rows"], b["rows"]):
        for key in sorted((set(ra) | set(rb)) - ignore):
            if ra.get(key) != rb.get(key):
                entry = {"slug": ra.get("slug"), "stage": ra.get("stage"), "field": key}
                if key.endswith("_tree_sha256"):
                    area = key.split("_")[0]
                    pa = ta.get(ra["slug"], {}).get(ra["stage"], {}).get(area, {})
                    pb = tb.get(rb["slug"], {}).get(rb["stage"], {}).get(area, {})
                    entry["files"] = sorted(p for p in set(pa) | set(pb) if pa.get(p) != pb.get(p))
                drift.append(entry)
    notes = []
    for key in (
        "baseline",
        "pristine_head",
        "final_corpus_tree_sha256",
        "final_corpus_file_count",
        "final_root_tree_sha256",
        "pristine_corpus_digest",
        "inputs_tree_sha256",
    ):
        if (key in a) != (key in b):
            notes.append({"field": key, "present_in": args.a if key in a else args.b})
        elif a.get(key) != b.get(key):
            drift.append({"kind": "run-field", "field": key, "a": a.get(key), "b": b.get(key)})
    summary = {
        "a": args.a,
        "b": args.b,
        "a_binary_sha256": a["binary_sha256"],
        "b_binary_sha256": b["binary_sha256"],
        "a_harness_sha256": a.get("harness_sha256"),
        "b_harness_sha256": b.get("harness_sha256"),
        "rows_compared": len(a["rows"]),
        "notes": notes,
        "drift_count": len(drift),
        "drift": drift[:200],
    }
    (WORK / f"compare-{args.a}-vs-{args.b}.json").write_text(json.dumps(summary, indent=1) + "\n")
    print(json.dumps({k: v for k, v in summary.items() if k != "drift"}))
    for entry in drift[:20]:
        print("DRIFT", json.dumps(entry))
    sys.exit(1 if drift else 0)


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="command", required=True)
    sub.add_parser("inventory").set_defaults(func=cmd_inventory)
    freeze = sub.add_parser("freeze")
    freeze.add_argument("--source-work", default=str(DEFAULT_SOURCE_WORK))
    freeze.set_defaults(func=cmd_freeze)
    for name, func in (("replay", cmd_replay), ("sweep", cmd_sweep)):
        run = sub.add_parser(name)
        run.add_argument("--binary", required=True)
        run.add_argument("--label", required=True)
        run.add_argument("--baseline", help="full SHA whose archive seeds the pristine root (default HEAD)")
        run.add_argument("--only", help="comma-separated slugs")
        run.add_argument("--limit", type=int)
        run.add_argument("--skip-extract", action="store_true", help="sweep only: skip retained re-extraction")
        run.add_argument("--supplement", action="store_true", help="replay only: use the hash-recovered supplementary inputs")
        run.set_defaults(func=func)
    supp = sub.add_parser("freeze-supplement")
    supp.add_argument("--search", required=True)
    supp.set_defaults(func=cmd_freeze_supplement)
    compare = sub.add_parser("compare")
    compare.add_argument("a")
    compare.add_argument("b")
    compare.set_defaults(func=cmd_compare)
    args = parser.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
