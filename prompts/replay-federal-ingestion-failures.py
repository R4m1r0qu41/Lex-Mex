#!/usr/bin/env python3
"""Replay frozen failures into isolated candidates, never fetch or admit.

Usage: python3 prompts/replay-federal-ingestion-failures.py EVIDENCE_ROOT OUTPUT_ROOT [SLUG ...]
Pass --provisional as the optional selector to replay the original 99 successes.
Run cargo build --locked -p lex-cli first. The input report and receipts are read-only.
Outputs include source hashes, all stage logs, candidate differences, and review-state evidence.
"""
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def replay(evidence, output, only):
    report = json.loads((evidence / 'report.json').read_text())
    output.mkdir(parents=True, exist_ok=True)
    binary = output/'replay-lex-mex'
    if binary.exists():
        raise RuntimeError(f'refusing to replace frozen executable {binary}')
    shutil.copy2(Path('target/debug/lex-mex').resolve(), binary)
    implementation_hashes = {str(p): digest(p) for p in [Path('crates/lex-parse/src/diputados.rs'), Path('crates/lex-parse/src/labels.rs'), Path('crates/lex-parse/src/lib.rs'), Path('crates/lex-export/src/lib.rs'), Path(__file__)]}
    rows = []
    target_status = 'awaiting_inspection' if '--provisional' in only else 'failed'
    only = only - {'--provisional'}
    for slug, attempts in report['attempts'].items():
        attempt = attempts[-1]
        if attempt['status'] != target_status or only and slug not in only:
            continue
        original = Path(attempt['root'])
        root = output / slug
        if root.exists():
            raise RuntimeError(f'refusing to overwrite {root}; choose a fresh replay output')
        root.mkdir()
        shutil.copytree(original / 'adapters', root / 'adapters')
        shutil.copytree(original / '.work' / slug, root / '.work' / slug)
        # Only acquisition provenance is carried forward. No old candidate or
        # review decision can masquerade as a successful new parse.
        corpus = root / 'corpus/mx' / slug
        corpus.mkdir(parents=True)
        source_manifest = original / 'corpus/mx' / slug / 'source-manifest.json'
        shutil.copy2(source_manifest, corpus / 'source-manifest.json')
        sources = [p for p in (root / '.work' / slug).iterdir() if p.suffix in {'.pdf', '.txt'}]
        before = {p.name: digest(p) for p in sources}
        raw = (root/'.work'/slug/(slug+'.txt')).read_text()
        judicial = []
        for match in re.finditer(r'(?m)^ *(?:PUNTOS RESOLUTIVOS de la sentencia|SENTENCIA dictada|VERSIÓN pública de la Sentencia)', raw):
            next_act = re.search(r'(?m)^DECRETO ', raw[match.end():])
            end = match.end() + next_act.start() if next_act else len(raw)
            judicial.append(dict(char_start=match.start(), char_end=end, exact_text=raw[match.start():end]))
        (root/'judicial-appendix-review.json').write_text(json.dumps(judicial, ensure_ascii=False, indent=2)+'\n')
        row = dict(slug=slug, original_code=next((r['code'] for r in reversed(attempt['receipts']) if r['code']), None),
                   original_root=str(original), replay_root=str(root), binary_sha256=digest(binary), implementation_sha256=implementation_hashes,
                   original_status=target_status, source_manifest_sha256=digest(source_manifest), source_hashes=before, stages=[])
        if not (before.get(slug+'.pdf') == json.loads(source_manifest.read_text())['source_sha256'] and before.get(slug+'.txt') == json.loads(source_manifest.read_text())['extracted_text_sha256']):
            raise RuntimeError(f'source bytes do not match frozen manifest for {slug}')
        for stage in ['extract', 'parse', 'validate', 'export']:
            args = [str(binary), '--root', str(root), stage, slug]
            if stage == 'export':
                args += ['--format', 'obsidian', '--obsidian-vault', str(root/'export')]
            result = subprocess.run(args, text=True, capture_output=True)
            (root / (stage + '.log')).write_text(result.stdout + result.stderr)
            row['stages'].append(dict(stage=stage, exit_code=result.returncode))
            if result.returncode:
                row['blocker'] = (result.stdout + result.stderr)[-5000:]
                break
        row['retained_sources_unchanged'] = all(digest(p) == before[p.name] for p in sources)
        provisions_path = corpus / 'provisions.json'
        if provisions_path.exists():
            provisions = json.loads(provisions_path.read_text())
            row['counts'] = {kind: sum(p['provision_type'] == kind for p in provisions) for kind in ['article', 'transitory', 'annex']}
            # Exact gate from frozen campaign, plus explicit apparatus tail audit.
            date = re.compile(r'(?im)^[ \t]*(?:México(?:,[ \t]*D\.[ \t]*F\.)?|Ciudad de México),[ \t]*a[ \t]+\d{1,2}[ \t]+de[ \t]+[a-záéíóúñ]+[ \t]+de[ \t]+\d{4}\.[ \t]*[-–]')
            row['signature_findings'] = [p['id'] for p in provisions if p['provision_type'] == 'transitory' and any(re.search(r'(?i)(?:\bDip\.|\bSen\.|Rúbricas?)', p['text'][m.start():].split('\n\n', 1)[0]) for m in date.finditer(p['text']))]
            row['review_statuses'] = sorted({p['review_status'] for p in provisions})
            row['temporal_statuses'] = sorted({p['temporal_status'] for p in provisions})
            old_path = original / 'corpus/mx' / slug / 'provisions.json'
            old = {p['id']: p for p in json.loads(old_path.read_text())} if old_path.exists() else {}
            new = {p['id']: p for p in provisions}
            diff = dict(added_ids=sorted(new.keys()-old.keys()), removed_ids=sorted(old.keys()-new.keys()),
                        changed=[dict(id=k, before=old[k]['text'], after=new[k]['text']) for k in new.keys() & old.keys() if old[k]['text'] != new[k]['text']])
            (root/'candidate-diff.json').write_text(json.dumps(diff, ensure_ascii=False, indent=2)+'\n')
            row['candidate_diff'] = {k: len(v) for k,v in diff.items()}
        reform = corpus/'reform-temporal-evidence.json'
        new_reform = json.loads(reform.read_text()) if reform.exists() else []
        old_reform_path = original/'corpus/mx'/slug/'reform-temporal-evidence.json'
        old_reform = json.loads(old_reform_path.read_text()) if old_reform_path.exists() else []
        row['reform_evidence_count'] = len(new_reform)
        row['prior_reform_evidence_count'] = len(old_reform)
        old_ids, new_ids = {p['provision_id'] for p in old_reform}, {p['provision_id'] for p in new_reform}
        row['removed_reform_evidence_ids'] = sorted(old_ids-new_ids)
        row['added_reform_evidence_ids'] = sorted(new_ids-old_ids)
        if source_manifest.exists() and (corpus/'source-manifest.json').exists():
            original_manifest = json.loads(source_manifest.read_text())
            replay_manifest = json.loads((corpus/'source-manifest.json').read_text())
            row['acquisition_identity_preserved'] = all(original_manifest.get(k)==replay_manifest.get(k) for k in ['instrument_id','official_url','publisher','retrieved_at','source_sha256','extracted_text_sha256','extraction_tool','schema_version_used'])
        row['judicial_appendices_for_review'] = len(judicial)
        row['review_queue_exists'] = (corpus/'review-queue.json').exists()
        row['markdown_count'] = len(list((root/'export').rglob('*.md')))
        row['status'] = 'replay_passed' if all(s['exit_code']==0 for s in row['stages']) and len(row['stages'])==4 and not row.get('signature_findings') else 'still_failing'
        rows.append(row)
        (output/'results.json').write_text(json.dumps(rows, ensure_ascii=False, indent=2)+'\n')
        print(slug, row['status'], row.get('counts'), flush=True)
    return rows

if __name__ == '__main__':
    replay(Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve(), set(sys.argv[3:]))
