#!/usr/bin/env python3
"""Refresh documentary SQLite debt evidence from pinned Git blobs, never corpus text.
Usage: refresh-sqlite-debt.py REPO FULL_COMMIT OUTPUT_DIRECTORY
Requires the sibling detector and historical review artifact in this worktree.
This does not create the production builder seed or accept a new legal ruling.
"""
import collections
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import subprocess
import sys
import unicodedata

sys.dont_write_bytecode = True


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def norm(label):
    return re.sub(r'[\s-]+', ' ', ''.join(c for c in unicodedata.normalize('NFD', label.lower()) if not unicodedata.combining(c))).strip()


def main():
    repo, commit, output = sys.argv[1:]
    output = Path(output)
    output.mkdir(parents=True, exist_ok=True)
    here = Path(__file__).resolve().parent
    def git(*args):
        return subprocess.check_output(['git', '-C', repo, *args])
    full = git('rev-parse', commit).decode().strip()
    if full != commit:
        raise ValueError('pass the full immutable commit OID')
    cache = {}
    def raw(path):
        if path not in cache:
            cache[path] = git('show', f'{commit}:{path}')
        return cache[path]
    def data(path):
        return json.loads(raw(path))
    decisions = raw('docs/decisions.md').decode()
    anchor = 'docs/decisions.md#2026-10-18-merged-headings-are-split-into-their-own-articles-letter-series-accepted-by-equivalence'
    assert 'all 77 series-checked rows are accepted' in decisions
    assert 'The LFTR 180 Quintes–Septimus bodies are legally removed text' in decisions
    spec = importlib.util.spec_from_file_location('detector', here/'detect-embedded-article-headings.py')
    detector = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(detector)
    _, candidates, excluded = detector.detect(repo, commit)
    old_path = here.parent/'docs/sqlite-structural-debt-review-2026-10-17.json'
    old_raw = old_path.read_bytes()
    old = json.loads(old_raw)
    def key(row):
        return row['instrument_id'], row['containing_provision_id'], norm(row['printed_label'])
    previous = {key(row): row for row in old['rows']}
    assert len(previous) == len(old['rows'])
    rows = []
    for candidate in candidates:
        prior = previous.get(key(candidate))
        status, basis, ref, date = 'unreviewed', 'new candidate; no ruling assigned', None, None
        if prior:
            assert candidate['heading_text'] == prior['heading_text']
            assert candidate['body_bracketed'] == prior['body_bracketed']
            status, basis, ref, date = (prior['review_status'], prior['review_basis'], prior['decision_reference'], prior['reviewed_at'])
            if status == 'proposed':
                status, basis, ref, date = 'accepted', '2026-10-18 operator acceptance of the original 77 series-checked rows; locator rebound mechanically', anchor, '2026-10-18'
            elif prior['instrument_id'].endswith(':lftr') and prior['body_bracketed']:
                status, basis, ref, date = 'rejected', '2026-10-18 ruling: LFTR bracketed bodies are legally removed text', anchor, '2026-10-18'
        rows.append({**candidate, 'kind': 'legally_removed_placeholder' if status == 'rejected' else 'merged_heading', 'detected_kind': candidate['kind'], 'detected_at_commit': commit, 'detector_version': detector.DETECTOR_VERSION, 'series_check': prior.get('series_check') if prior else None, 'review_status': status, 'review_basis': basis, 'reviewer': 'operator' if status in ('accepted', 'rejected', 'deferred') else None, 'reviewed_at': date, 'decision_reference': ref})
    active = {key(row) for row in rows}
    resolved = []
    for row in old['rows']:
        if key(row) in active:
            continue
        assert row['review_status'] == 'proposed', 'unclassified old row disappeared'
        provisions = data(row['source_path'])
        found = [p for p in provisions if norm(p['number']) == norm(row['printed_label']) and p['provision_type'] == 'article']
        assert len(found) == 1, f'old debt is not separately parsed: {key(row)}'
        resolved.append({'instrument_id': row['instrument_id'], 'printed_label': row['printed_label'], 'old_containing_provision_id': row['containing_provision_id'], 'current_provision_id': found[0]['id'], 'status': 'separately_parsed', 'verified_at_commit': commit, 'source_path': row['source_path'], 'source_blob_oid': git('rev-parse', f'{commit}:{row["source_path"]}').decode().strip(), 'source_sha256': sha(raw(row['source_path']))})
    names = git('ls-tree', '-r', '--name-only', commit, 'corpus/mx').decode().splitlines()
    instruments = sorted(p for p in names if p.endswith('/instrument.json'))
    census = collections.Counter(instruments=len(instruments))
    coverage = collections.Counter()
    statuses = collections.Counter()
    forms = collections.Counter()
    qualifier_types = collections.Counter()
    golden = {}
    for instrument_path in instruments:
        base = str(Path(instrument_path).parent)
        slug = Path(base).name
        for filename, countkey in [('provisions.json', 'provisions'), ('references.json', 'references'), ('terms.json', 'terms'), ('term-usages.json', 'term_usages')]:
            records = data(f'{base}/{filename}')
            census[countkey] += len(records)
            if filename == 'provisions.json':
                for p in records:
                    census[f'provision_{p["provision_type"]}'] += 1
            elif filename == 'references.json':
                for e in records:
                    statuses[e['resolution_status']] += 1
                    forms[e['reference_form']] += 1
                    census['title_anchors'] += e['source_provision_id'] == data(instrument_path)['id']
                    census['qualifiers'] += len(e['qualifiers'])
                    qualifier_types.update(q['qualifier_type'] for q in e['qualifiers'])
                    census['approvals'] += e.get('outdated_reference_approval') is not None
                    census['operator_reviews'] += e.get('legal_operator_review') is not None
                    census['edges_without_resolution_rule'] += 'resolution_rule' not in e
                if slug == 'reg-lan':
                    edges = [e for e in records if e.get('resolution_rule') == 'parent_shorthand']
                    assert all(e['target_instrument_id'].endswith(':lan') for e in edges)
                    golden['G1_parent_shorthand_count'] = len(edges)
                if slug == 'lic':
                    golden['G2_lic_194_lfce_31_bis'] = [e for e in records if e['source_provision_id'].endswith(':article:194') and e['target_provision_id'].endswith(':lfce:article:31-bis')]
        c = data(f'{base}/glossary-coverage.json')
        coverage[c['status']] += 1
        if slug == 'reg-lan':
            golden['G1_reg_lan_terms'] = [t['term'] for t in data(f'{base}/terms.json')]
    gaps = data('adapters/diputados/_closure-accepted-gaps.json')
    assert len(gaps['accepted']) == 5
    assert len(rows) == 57 and len(resolved) == 60
    counts = dict(sorted(collections.Counter(row['review_status'] for row in rows).items()))
    assert counts == {'accepted': 17, 'deferred': 10, 'rejected': 30}
    for row in rows:
        src = raw(row['source_path'])
        assert sha(src) == row['source_sha256']
        p = next(p for p in json.loads(src) if p['id'] == row['containing_provision_id'])
        assert p['text'][row['start_char']:row['end_char']] == row['heading_text']
    def write(name, value):
        (output/name).write_text(json.dumps(value, ensure_ascii=False, indent=2)+'\n')
    candidate_name = 'sqlite-structural-debt-candidates-2026-10-19.json'
    write(candidate_name, {'detector': detector.DETECTOR_VERSION, 'corpus_commit': commit, 'review_status': 'unreviewed', 'offset_unit': 'unicode_code_point_exclusive_end', 'candidate_count': len(candidates), 'excluded_match_counts': excluded, 'candidates': candidates})
    write('sqlite-structural-debt-review-2026-10-19.json', {'candidates_source': f'docs/{candidate_name}', 'candidates_sha256': sha((output/candidate_name).read_bytes()), 'corpus_commit': commit, 'detector': detector.DETECTOR_VERSION, 'offset_unit': 'unicode_code_point_exclusive_end', 'historical_review_sha256': sha(old_raw), 'decision_reference': anchor, 'status_counts': counts, 'rows': rows})
    write('sqlite-contract-refresh-2026-10-19.json', {'corpus_commit': commit, 'prior_review_sha256': sha(old_raw), 'detector_script_sha256': sha((here/'detect-embedded-article-headings.py').read_bytes()), 'refresh_script_sha256': sha(Path(__file__).read_bytes()), 'decision_source_sha256': sha(raw('docs/decisions.md')), 'census': dict(census), 'reference_status_counts': dict(statuses), 'reference_form_counts': dict(forms), 'qualifier_type_counts': dict(qualifier_types), 'glossary_coverage_counts': dict(coverage), 'accepted_gaps': gaps['accepted'], 'current_debt_counts': counts, 'resolved_old_debt': resolved, 'golden_query_pins': golden})
    print(json.dumps({'census': dict(census), 'debt': counts, 'resolved_old_debt': len(resolved), 'coverage': dict(coverage)}, sort_keys=True))


if __name__ == '__main__':
    main()
