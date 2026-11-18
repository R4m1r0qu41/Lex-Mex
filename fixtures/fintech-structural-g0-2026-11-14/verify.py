#!/usr/bin/env python3
"""Verify the frozen G0 submission. No candidate recognizer or projection code."""
import argparse
import hashlib
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[1]


def digest(data):
    return hashlib.sha256(data).hexdigest()


def load(path):
    return json.loads(path.read_text(encoding='utf-8'))


def require(condition, message):
    if not condition:
        raise SystemExit('FAIL: ' + message)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--holdout-gold', type=Path,
                        help='Reviewer-only: check the sealed answer file commitment; never print it.')
    args = parser.parse_args()
    manifest = load(ROOT / 'sample-manifest.json')
    base = manifest['baseline_commit']
    for name, expected in manifest['artifact_sha256'].items():
        require(digest((ROOT / name).read_bytes()) == expected, 'artifact hash: ' + name)
    sources = load(ROOT / 'source-snapshots.json')
    committed = {}
    for name, expected in sources['source_file_sha256'].items():
        raw = subprocess.check_output(['git', 'show', base + ':' + name], cwd=REPO)
        require(digest(raw) == expected, 'pinned source hash: ' + name)
        require((REPO / name).read_bytes() == raw, 'current source drift: ' + name)
        committed[name] = json.loads(raw)
    samples = manifest['samples']
    require(len(samples) == 24, 'sample size')
    require(len({s['provision_id'] for s in samples}) == 24, 'sample IDs overlap')
    require(sum(s['split'] == 'development' for s in samples) == 16, 'development size')
    require(sum(s['split'] == 'holdout' for s in samples) == 8, 'holdout size')
    snapshots = load(ROOT / 'canonical-snapshot.json')['provisions']
    require(len(snapshots) == 24, 'snapshot size')
    by_id = {s['record']['id']: s for s in snapshots}
    require(len(by_id) == 24, 'duplicate snapshot IDs')
    for sample in samples:
        record = by_id[sample['provision_id']]['record']
        path = 'corpus/mx/' + sample['slug'] + '/provisions.json'
        original = next(p for p in committed[path] if p['id'] == record['id'])
        require(original == record, 'canonical snapshot drift')
        require(digest(record['text'].encode('utf-8')) == sample['canonical_text_sha256'],
                'sample text binding')
    count = 0
    for case in load(ROOT / 'development-gold.json')['cases']:
        text = by_id[case['provision_id']]['record']['text']
        require(digest(text.encode('utf-8')) == case['canonical_text_sha256'], 'gold text binding')
        for query in case['annotations']:
            count += 1
            if query['expected_outcome'] == 'resolved':
                span = query['label_span']
                start, end = span['start_char'], span['end_char']
                require(0 <= start < end <= len(text), 'development label range')
                require(text[start:end] == query['exact_quote'], 'development label quote')
            else:
                require(query['expected_outcome'] in ['not_found', 'not_structured'], 'outcome')
        for observed in case.get('non_marker_observations', []):
            require(text[observed['start_char']:observed['end_char']] == observed['exact_quote'],
                    'non-marker observation quote')
        for witness in case['existing_definition_witnesses']:
            records = committed['corpus/mx/' + by_id[case['provision_id']]['slug'] + '/terms.json']
            require(witness in records, 'definition witness drift')
            require(0 <= witness['start_char'] < witness['end_char'] <= len(text), 'definition range')
    development = load(ROOT / 'development-selection.json')
    for case in development:
        edges = committed['corpus/mx/' + case['slug'] + '/references.json']
        seed = case['query_seed']
        edge = next(e for e in edges if e['id'] == seed['reference_edge_id'])
        for key in ['source_provision_id', 'target_provision_id', 'source_span']:
            require(seed[key] == edge[key], 'reference seed drift')
        require(seed['qualifiers'] == [q for q in edge['qualifiers']
                                       if q['qualifier_type'] == 'fraction'], 'qualifier seed drift')
    if args.holdout_gold:
        require(digest(args.holdout_gold.read_bytes()) == manifest['sealed_holdout']['sha256'],
                'sealed holdout commitment')
    print('PASS: 24 exact pinned records; 16 development / 8 disjoint holdout; '
          + str(count) + ' development queries; source/seed/definition/artifact hashes verified.'
          + (' Sealed commitment verified.' if args.holdout_gold else ' Holdout answers not opened.'))


if __name__ == '__main__':
    main()
