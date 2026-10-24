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

REPO = Path(os.environ.get("PARSER_SPLIT_REPO") or Path(__file__).resolve().parent.parent)
FROZEN = os.environ.get("PARSER_SPLIT_FROZEN") == "1"
WORK = REPO / ".work" / "parser-split"
RUN_ROOT = Path(os.environ.get("PARSER_SPLIT_RUN_ROOT", WORK / "run-root"))
INPUTS = WORK / "inputs"
INPUTS_SUPP = WORK / "inputs-supplement"
DEFAULT_SOURCE_WORK = Path("/Users/jr/Dev/lex-mex/.work")

# Offline stages for one instrument that has retained extracted text. The
# pipeline command also fetches, so the stages it chains are replayed here.
REQUIRED_COMMON = (
    "kind",
    "baseline",
    "harness_sha256",
    "harness_unchanged",
    "harness_frozen_snapshot",
    "harness_committed",
    "binary_unchanged",
    "final_corpus_tree_sha256",
    "final_corpus_file_count",
    "final_root_tree_sha256",
    "pristine_corpus_digest",
    "scope",
)
REQUIRED_ROW_KEYS = ("slug", "stage", "argv", "exit", "stdout_sha256", "stderr_sha256", "corpus_tree_sha256")
ROWS_WITH_WORK_TREE = {"parse", "temporal-request", "validate", "derive-temporal", "export-markdown", "extract", "audit-furniture"}
REQUIRED_BY_KIND = {
    "whole-corpus-sweep": ("pristine_head",),
    "retained-replay": ("inputs_tree_sha256",),
}

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


# The harness identity is fixed when the process starts: the exact bytes loaded
# here are hashed and snapshotted, and finish() records whether the file changed
# while the run was in progress.
_HARNESS_BYTES = Path(__file__).resolve().read_bytes()
HARNESS_SHA256_AT_START = sha256_bytes(_HARNESS_BYTES)


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


# The whole-corpus steps that run outside the per-slug stage loops, in order. cmd_sweep
# runs exactly these (head, then standards, per-slug stages, extraction, tail).
SWEEP_HEAD = ("bundle-canonical", "bundle-markdown", "link-all", "instruments", "path", "search",
              "review-list", "review-legacy", "review-packets-generate", "review-packets-list")
SWEEP_TAIL = ("link-lritf", "link-ifpe", "repair-dates-lritf", "repair-dates-ccf", "db-contract-probe")
STANDARDS_STAGES = ("standards-validate", "standards-refresh")


def options_of(args):
    return {
        "supplement": bool(getattr(args, "supplement", False)),
        "only": getattr(args, "only", None),
        "limit": getattr(args, "limit", None),
        "skip_extract": bool(getattr(args, "skip_extract", False)),
    }


def pick(slugs, args):
    if args.only:
        wanted = args.only.split(",")
        slugs = [s for s in slugs if s in wanted]
    if args.limit:
        slugs = slugs[: args.limit]
    return slugs


def replay_scope(args):
    name = "coverage-manifest-supplement.json" if args.supplement else "coverage-manifest.json"
    manifest = json.loads((WORK / name).read_text())
    return {"command": "replay", "options": options_of(args), "slugs": pick(manifest["replay_order"], args),
            "stages": [stage for stage, _ in RETAINED_STAGES]}


def sweep_scope(args):
    adapters = adapter_rows()
    scope = {"command": "sweep", "options": options_of(args),
             "standards": [s for s in committed_slugs() if s not in adapters],
             "slugs": pick([s for s in committed_slugs() if s in adapters], args),
             "stages": [stage for stage, _ in SWEEP_STAGES], "retained": []}
    if not args.skip_extract:
        manifest = json.loads((WORK / "coverage-manifest.json").read_text())
        scope["retained"] = [{"slug": slug, "pdf": (INPUTS / slug / f"{slug}.pdf").exists()}
                             for slug in pick(manifest["replay_order"], args)]
    return scope


def plan_from_scope(scope):
    """Ordered expected rows: (slug, [stages], origin, may_stop_after_failed_parse)."""
    plan = []
    if scope["command"] == "replay":
        plan += [(slug, scope["stages"], "replay", True) for slug in scope["slugs"]]
        return plan
    plan += [("*", [stage], "global", False) for stage in SWEEP_HEAD]
    for slug in scope["standards"]:
        plan += [(slug, [stage], "global", False) for stage in STANDARDS_STAGES]
    plan += [(slug, scope["stages"], "sweep", True) for slug in scope["slugs"]]
    for entry in scope["retained"]:
        plan.append((entry["slug"], ["extract"], "work", False))
        if entry["pdf"]:
            plan.append((entry["slug"], ["audit-furniture"], "work", False))
    plan += [("*", [stage], "global", False) for stage in SWEEP_TAIL]
    return plan


GLOBAL_KEYS = ("out_tree_sha256", "packets_tree_sha256")


def evidence_problem(result, trees, logs, scope):
    """Why a result's rows, stage trees or logs are incomplete for the given scope, or None."""
    rows = result.get("rows")
    if not isinstance(rows, list) or not rows:
        return "rows missing or empty"
    if not isinstance(trees, dict):
        return "trees file is not a map"
    position = 0
    for slug, stages, origin, may_stop in plan_from_scope(scope):
        taken = 0
        while taken < len(stages) and position < len(rows) and rows[position].get("slug") == slug and rows[position].get("stage") == stages[taken]:
            row = rows[position]
            if not isinstance(row, dict):
                return f"row {position} is not a map"
            needed = list(REQUIRED_ROW_KEYS)
            if origin in ("replay", "work"):
                needed.append("work_tree_sha256")
            if origin == "global":
                needed += GLOBAL_KEYS
            absent = [key for key in needed if key not in row]
            if absent:
                return f"row {slug}/{row.get('stage')} lacks {absent}"
            for stream in ("stdout", "stderr"):
                path = logs / f"{slug}.{row['stage']}.{stream}"
                if not path.is_file():
                    return f"log {path.name} missing"
                if sha256_file(path) != row[f"{stream}_sha256"]:
                    return f"log {path.name} does not match its recorded hash"
            if origin != "global":
                recorded = trees.get(slug, {}).get(row["stage"], {})
                for area, key in (("corpus", "corpus_tree_sha256"), ("work", "work_tree_sha256")):
                    if area == "work" and origin == "sweep":
                        continue
                    saved = recorded.get(area)
                    if not isinstance(saved, dict) or not saved:
                        return f"stage tree {slug}/{row['stage']}/{area} missing or empty"
                    if digest_map(saved) != row[key]:
                        return f"stage tree {slug}/{row['stage']}/{area} does not match its digest"
            position += 1
            taken += 1
            if may_stop and row["stage"] == "parse" and row["exit"] != 0:
                taken = len(stages)  # later stages are intentionally skipped after a failed parse
                break
        if taken < len(stages):
            return f"expected stage {stages[taken]} for {slug} is missing or out of order"
    if position != len(rows):
        return f"{len(rows) - position} unexpected extra rows"
    return None


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


def write_atomic(path, text):
    temporary = path.with_name(path.name + ".tmp")
    temporary.write_text(text)
    os.replace(temporary, path)


def finish(label, binary, kind, baseline, rows, trees, extra):
    final_corpus = tree_map(RUN_ROOT, "corpus/mx")
    result = {
        "label": label,
        "kind": kind,
        "binary": str(binary),
        "binary_sha256": sha256_file(binary),
        "binary_unchanged": extra.get("binary_sha256_at_start") == sha256_file(binary),
        "baseline": baseline,
        "git_head": git("rev-parse", "HEAD"),
        "harness_sha256": HARNESS_SHA256_AT_START,
        "harness_sha256_at_finish": sha256_file(Path(__file__).resolve()),
        "harness_unchanged": sha256_file(Path(__file__).resolve()) == HARNESS_SHA256_AT_START,
        "harness_frozen_snapshot": FROZEN and Path(__file__).resolve().parent.name == "harness-snapshots",
        "harness_committed": harness_committed(_HARNESS_BYTES),
        "harness_live_source_changed_during_run": Path(
            os.environ.get("PARSER_SPLIT_LIVE_PATH") or __file__
        ).read_bytes()
        != _HARNESS_BYTES,
        "final_root_tree_sha256": digest_map(tree_map(RUN_ROOT, exclude=(".git",))),
        "rows": rows,
        "final_corpus_tree_sha256": digest_map(final_corpus),
        "final_corpus_file_count": len(final_corpus),
        **extra,
    }
    # Trees first, result last, each replaced atomically: a result file on disk is
    # always complete, and its presence means the trees file exists.
    write_atomic(WORK / f"trees-{label}.json", json.dumps(trees, sort_keys=True))
    write_atomic(WORK / f"result-{label}.json", json.dumps(result, indent=1, sort_keys=True) + "\n")
    failures = [r for r in rows if r["exit"] != 0]
    print(f"{label}: {len(rows)} commands, {len(failures)} nonzero; corpus digest {result['final_corpus_tree_sha256']}")
    if not (result["harness_unchanged"] and result["binary_unchanged"]):
        print(f"{label}: harness or binary changed while the run was in progress; evidence is not usable", file=sys.stderr)
        sys.exit(3)


def archive_interrupted(label, logs):
    """Move an interrupted attempt aside so the label can run again.

    Nothing is deleted or overwritten: the old logs (and, for an unusable result,
    the result and trees files) are renamed, and the move is recorded in
    attempts-<label>.json. A label whose result is usable evidence is refused:
    the driver reconciles such a result with verify-result, or a fresh label is
    chosen. A result is unusable only when the run itself reported that the
    harness or binary changed while it ran (exit 3).
    """
    result_path = WORK / f"result-{label}.json"
    moved_result = []
    if result_path.exists():
        try:
            existing = json.loads(result_path.read_text())
        except ValueError:
            existing = {}
        if existing.get("harness_unchanged") is True and existing.get("binary_unchanged") is True:
            sys.exit(
                f"refusing: {label} already has a usable finished result; reconcile it "
                "(verify-result) or choose a fresh --label"
            )
        moved_result = [result_path, WORK / f"trees-{label}.json"]
    stamp = time.strftime("%Y%m%dT%H%M%SZ", time.gmtime())
    suffix = f".interrupted-{stamp}"
    counter = 1
    while (WORK / f"{logs.name}{suffix}").exists() or any(
        path.with_name(path.name + suffix).exists() for path in moved_result
    ):
        counter += 1
        suffix = f".interrupted-{stamp}-{counter}"
    files = sum(1 for path in logs.rglob("*") if path.is_file()) if logs.exists() else 0
    target = WORK / f"{logs.name}{suffix}"
    archived = []
    if logs.exists():
        logs.rename(target)
        archived.append(target.name)
    for path in moved_result:
        if path.exists():
            path.rename(path.with_name(path.name + suffix))
            archived.append(path.name + suffix)
    record = WORK / f"attempts-{label}.json"
    history = json.loads(record.read_text()) if record.exists() else []
    history.append({"archived_to": archived, "files": files, "utc": stamp, "had_result": bool(moved_result)})
    write_atomic(record, json.dumps(history, indent=1) + "\n")
    print(f"{label}: archived interrupted attempt ({files} log files) to {', '.join(archived)}", flush=True)


def result_problem(args):
    """Why result-<label>.json is not a complete, clean result of this exact run, or None."""
    result_path, trees_path = WORK / f"result-{args.label}.json", WORK / f"trees-{args.label}.json"
    if not result_path.exists() or not trees_path.exists():
        return "result or trees file missing"
    try:
        result = json.loads(result_path.read_text())
        json.loads(trees_path.read_text())
    except ValueError as error:
        return f"unreadable: {error}"
    expected = "retained-replay" if args.run_kind == "replay" else "whole-corpus-sweep"
    if result.get("kind") != expected:
        return f"kind {result.get('kind')!r}, expected {expected!r}"
    if result.get("label") != args.label:
        return "label does not match"
    for key in list(REQUIRED_COMMON) + list(REQUIRED_BY_KIND[expected]):
        if key not in result:
            return f"required evidence {key} absent"
    for key in ("harness_unchanged", "binary_unchanged", "harness_frozen_snapshot", "harness_committed"):
        if result[key] is not True:
            return f"{key} is not true"
    if result.get("harness_live_source_changed_during_run") is not False:
        return "live harness source changed during the run"
    if result["harness_sha256"] != HARNESS_SHA256_AT_START:
        return "made by a different harness revision"
    if result.get("binary_sha256") != sha256_file(Path(args.binary).resolve()):
        return "made by a different binary"
    if result["baseline"] != args.baseline:
        return "different baseline"
    if expected == "retained-replay":
        name = "coverage-manifest-supplement.json" if args.supplement else "coverage-manifest.json"
        if result["inputs_tree_sha256"] != json.loads((WORK / name).read_text())["inputs_tree_sha256"]:
            return "different frozen inputs"
    logs = WORK / f"logs-{args.label}"
    if not logs.is_dir():
        return "logs directory missing"
    # The run must have covered exactly the scope the queue asked for, not a subset.
    requested = replay_scope(args) if args.run_kind == "replay" else sweep_scope(args)
    if result["scope"] != requested:
        return "recorded scope differs from the requested scope (a subset or different options)"
    return evidence_problem(result, json.loads(trees_path.read_text()), logs, requested)


def cmd_verify_result(args):
    problem = result_problem(args)
    if problem:
        print(f"{args.label}: result not reconcilable: {problem}", file=sys.stderr)
        sys.exit(1)
    print(f"{args.label}: result is complete and clean for this harness revision, binary and inputs")


def prepare(args, git_init=False):
    binary = Path(args.binary).resolve()
    baseline = args.baseline or json.loads((WORK / "coverage-manifest.json").read_text())["baseline"]
    logs = WORK / f"logs-{args.label}"
    if logs.exists() or (WORK / f"result-{args.label}.json").exists():
        if not args.archive_interrupted:
            sys.exit(
                f"refusing to overwrite {logs} or its result; choose a fresh --label, or pass "
                "--archive-interrupted to move an interrupted attempt aside"
            )
        archive_interrupted(args.label, logs)
    logs.mkdir(parents=True)
    snapshots = WORK / "harness-snapshots"
    snapshots.mkdir(exist_ok=True)
    snapshot = snapshots / f"{HARNESS_SHA256_AT_START}.py"
    if not snapshot.exists():
        snapshot.write_bytes(_HARNESS_BYTES)
    build_pristine_root(baseline, git_init)
    pristine = tree_map(RUN_ROOT, "corpus")
    tree_id = git("rev-parse", f"{baseline}:corpus")
    return binary, baseline, logs, {
        "pristine_corpus_digest": digest_map(pristine),
        "baseline_corpus_git_tree": tree_id,
        "binary_sha256_at_start": sha256_file(binary),
    }


def cmd_replay(args):
    binary, baseline, logs, extra = prepare(args)
    inputs = INPUTS_SUPP if args.supplement else INPUTS
    manifest_name = "coverage-manifest-supplement.json" if args.supplement else "coverage-manifest.json"
    manifest = json.loads((WORK / manifest_name).read_text())
    scope = replay_scope(args)
    slugs = scope["slugs"]
    extra["scope"] = scope
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
    scope = sweep_scope(args)
    extra["scope"] = scope
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
    for slug in scope["standards"]:
        for stage, argv in (("standards-validate", ["standards", "validate", slug]), ("standards-refresh", ["standards", "refresh", slug])):
            global_step(binary, logs, rows, slug, stage, argv)
    rows.extend(run_stages(binary, SWEEP_STAGES, scope["slugs"], logs, trees, with_work=False))
    # Re-extraction from retained sources runs last: it rewrites manifests.
    if not args.skip_extract:
        manifest = json.loads((WORK / "coverage-manifest.json").read_text())
        retained = [entry["slug"] for entry in scope["retained"]]
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
    # Evidence completeness is part of the comparison: a run missing a required
    # field, in a different evidence format, or made by a different harness
    # revision is not like-for-like, so by default it fails. Legacy results can
    # be compared only with --allow-legacy-format, which reports the evidence as
    # incomplete and never exits 0.
    required = list(REQUIRED_COMMON)
    notes = []
    for name, result in ((args.a, a), (args.b, b)):
        for key in required + list(REQUIRED_BY_KIND.get(result.get("kind"), ())):
            if key not in result:
                notes.append({"run": name, "field": key, "why": "required evidence absent"})
        for key in ("harness_unchanged", "binary_unchanged", "harness_frozen_snapshot", "harness_committed"):
            if result.get(key) is False:
                drift.append({"kind": "run-mutated", "run": name, "field": key})
    # Every row must carry the full evidence set; a key missing from both rows of
    # a pair would otherwise never be compared.
    for name, result in ((args.a, a), (args.b, b)):
        for index, row in enumerate(result["rows"]):
            needed = list(REQUIRED_ROW_KEYS)
            replay_row = result.get("kind") == "retained-replay" and row.get("stage") in ROWS_WITH_WORK_TREE
            if replay_row or row.get("stage") in ("extract", "audit-furniture"):
                needed.append("work_tree_sha256")
            absent = [key for key in needed if key not in row]
            if absent:
                notes.append({"run": name, "row": index, "slug": row.get("slug"), "stage": row.get("stage"), "why": "row evidence absent", "fields": absent})
    if a.get("kind") != b.get("kind"):
        drift.append({"kind": "run-kind", "a": a.get("kind"), "b": b.get("kind")})
    if "scope" in a and "scope" in b and a["scope"] != b["scope"]:
        drift.append({"kind": "run-scope", "a": a["scope"].get("options"), "b": b["scope"].get("options")})
    for key in sorted(set(a) ^ set(b)):
        notes.append({"field": key, "why": "evidence format differs", "present_in": args.a if key in a else args.b})
    if a.get("harness_sha256") and b.get("harness_sha256") and a["harness_sha256"] != b["harness_sha256"]:
        notes.append({"field": "harness_sha256", "why": "runs made by different harness revisions"})
    for key in (
        "baseline",
        "pristine_head",
        "final_corpus_tree_sha256",
        "final_corpus_file_count",
        "final_root_tree_sha256",
        "pristine_corpus_digest",
        "inputs_tree_sha256",
    ):
        if key in a and key in b and a[key] != b[key]:
            drift.append({"kind": "run-field", "field": key, "a": a.get(key), "b": b.get(key)})
    evidence_complete = not notes
    if notes and not args.allow_legacy_format:
        drift.extend({"kind": "incomplete-evidence", **note} for note in notes)
    summary = {
        "a": args.a,
        "b": args.b,
        "a_binary_sha256": a["binary_sha256"],
        "b_binary_sha256": b["binary_sha256"],
        "comparator_harness_sha256": HARNESS_SHA256_AT_START,
        "a_harness_sha256": a.get("harness_sha256"),
        "b_harness_sha256": b.get("harness_sha256"),
        "rows_compared": len(a["rows"]),
        "evidence_complete": evidence_complete,
        "legacy_format_allowed": bool(args.allow_legacy_format),
        "notes": notes,
        "drift_count": len(drift),
        "drift": drift[:200],
    }
    (WORK / f"compare-{args.a}-vs-{args.b}.json").write_text(json.dumps(summary, indent=1) + "\n")
    print(json.dumps({k: v for k, v in summary.items() if k != "drift"}))
    for entry in drift[:20]:
        print("DRIFT", json.dumps(entry))
    if drift:
        sys.exit(1)
    if not evidence_complete:
        print("NO DRIFT BUT EVIDENCE INCOMPLETE (legacy format); not a pass", file=sys.stderr)
        sys.exit(2)
    sys.exit(0)


def reexec_from_snapshot():
    """Run the harness from a read-only, hash-named snapshot of itself.

    The live file can then be edited freely without changing what executes. If a
    snapshot for this hash already exists its content must match, otherwise the
    run is refused.
    """
    if FROZEN:
        return
    snapshots = WORK / "harness-snapshots"
    snapshots.mkdir(parents=True, exist_ok=True)
    snapshot = snapshots / f"{HARNESS_SHA256_AT_START}.py"
    if snapshot.exists():
        if sha256_file(snapshot) != HARNESS_SHA256_AT_START:
            sys.exit(f"refusing to run: snapshot {snapshot} does not match its name")
    else:
        snapshot.write_bytes(_HARNESS_BYTES)
        snapshot.chmod(0o444)
    env = dict(os.environ, PARSER_SPLIT_FROZEN="1", PARSER_SPLIT_REPO=str(REPO))
    env["PARSER_SPLIT_LIVE_SHA256"] = HARNESS_SHA256_AT_START
    env["PARSER_SPLIT_LIVE_PATH"] = str(Path(__file__).resolve())
    os.execve(sys.executable, [sys.executable, str(snapshot), *sys.argv[1:]], env)


def harness_committed(data):
    """Whether these exact bytes are the harness blob committed at HEAD."""
    try:
        head = git("rev-parse", "HEAD:prompts/parser-split-ab.py")
        blob = subprocess.run(
            ["git", "-C", str(REPO), "hash-object", "--stdin"], input=data, capture_output=True, check=True
        ).stdout.decode().strip()
    except (subprocess.CalledProcessError, OSError):
        return False
    return head == blob


def main():
    reexec_from_snapshot()
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
        run.add_argument(
            "--archive-interrupted",
            action="store_true",
            help="if logs-<label> exists from an interrupted run, rename it aside and run again; never touches a finished label",
        )
        run.set_defaults(func=func)
    verify = sub.add_parser("verify-result", help="exit 0 only if result-<label>.json is complete and clean for this harness, binary and inputs")
    verify.add_argument("--run-kind", choices=("replay", "sweep"), required=True)
    verify.add_argument("--binary", required=True)
    verify.add_argument("--label", required=True)
    verify.add_argument("--baseline", required=True)
    verify.add_argument("--supplement", action="store_true")
    verify.add_argument("--only")
    verify.add_argument("--limit", type=int)
    verify.add_argument("--skip-extract", action="store_true")
    verify.set_defaults(func=cmd_verify_result)
    supp = sub.add_parser("freeze-supplement")
    supp.add_argument("--search", required=True)
    supp.set_defaults(func=cmd_freeze_supplement)
    compare = sub.add_parser("compare")
    compare.add_argument("a")
    compare.add_argument("b")
    compare.add_argument(
        "--allow-legacy-format",
        action="store_true",
        help="compare results from before the evidence fields were required; exits 2, never 0",
    )
    compare.set_defaults(func=cmd_compare)
    args = parser.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
