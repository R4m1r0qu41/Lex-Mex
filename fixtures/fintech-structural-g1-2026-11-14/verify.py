#!/usr/bin/env python3
"""Check the G1 expected-behaviour file's bindings and span invariants.

Implements no G1 rule: it checks only text hashes, exact quotes, that spans end at paragraph
boundaries, that each intro starts after its label's whitespace, and that every G0
development query is covered. It never reads the sealed holdout.
"""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[1]
G0 = REPO / 'fixtures' / 'fintech-structural-g0-2026-11-14'
MODES = {'label', 'intro', 'complete'}
OUTCOMES = {'resolved', 'ambiguous', 'not_structured', 'not_found', 'stale_projection',
            'invalid_locator'}


def sha(text):
    return hashlib.sha256(text.encode('utf-8')).hexdigest()


def require(condition, message):
    if not condition:
        raise SystemExit('FAIL: ' + message)


def paragraph_end(text, end):
    return end == len(text) or text[end:end + 2] == '\n\n'


def paragraph_start(text, start):
    return start == 0 or text[start - 2:start] == '\n\n'


def main():
    snapshot = {p['record']['id']: p['record']['text'] for p in
                json.loads((G0 / 'canonical-snapshot.json').read_text(encoding='utf-8'))['provisions']}
    split = {s['provision_id']: s['split'] for s in
             json.loads((G0 / 'sample-manifest.json').read_text(encoding='utf-8'))['samples']}
    doc = json.loads((ROOT / 'expected-behaviour.json').read_text(encoding='utf-8'))
    ids = set()
    counts = {'development': 0, 'synthetic': 0}
    for case in doc['cases']:
        cid = case['case_id']
        require(cid not in ids, 'duplicate case ' + cid)
        ids.add(cid)
        counts[case['source']] += 1
        if 'synthetic_text' in case:
            text = case['synthetic_text']
            require(sha(text) == case['synthetic_text_sha256'], 'synthetic hash ' + cid)
        else:
            pid = case['provision_id']
            require(split.get(pid) == 'development', 'not a development provision: ' + cid)
            text = snapshot[pid]
            require(sha(text) == case['canonical_text_sha256'], 'text binding ' + cid)
        expected = case['expected']
        label = expected.get('label', {})
        for mode, result in expected.items():
            require(mode in MODES or result['outcome'] == 'invalid_locator', 'mode ' + cid)
            require(result['outcome'] in OUTCOMES, 'outcome ' + cid)
            if result['outcome'] == 'resolved':
                start, end = result['span']
                require(0 <= start < end <= len(text), 'range ' + cid + ' ' + mode)
                require(sha(text[start:end]) == result['substring_sha256'], 'substring ' + cid)
                if mode == 'label':
                    require(text[start:end] == result['exact_quote'], 'label quote ' + cid)
                    require(paragraph_start(text, start), 'label not at paragraph start ' + cid)
                else:
                    require(paragraph_end(text, end), mode + ' end not at paragraph end ' + cid)
            if mode == 'intro' and result['outcome'] == 'resolved':
                label_end = label['span'][1]
                start = result['span'][0]
                require(text[label_end:start].isspace() and not text[start].isspace(),
                        'intro start ' + cid)
            if mode == 'complete' and result['outcome'] == 'resolved':
                require(result['span'][0] == label['span'][0], 'complete start ' + cid)
            certain = result.get('certain_span')
            if certain:
                require(certain[0] == label['span'][0] and paragraph_end(text, certain[1]),
                        'certain span ' + cid)
            for segment in result.get('unresolved_segments', []):
                start, end = segment['start_char'], segment['end_char']
                require(0 <= start < end <= len(text) and paragraph_end(text, end),
                        'segment ' + cid)
                if segment['reason'] in ('trailing_attachment', 'unsupported_marker'):
                    require(paragraph_start(text, start), 'segment start ' + cid)
            for candidate in result.get('candidates', []):
                start, end = candidate['label_span']
                require(text[start:end] == candidate['exact_quote'], 'candidate quote ' + cid)
    gold = json.loads((G0 / 'development-gold.json').read_text(encoding='utf-8'))
    queries = {(c['provision_id'], json.dumps({k: v for k, v in a['locator'].items()
                                               if k != 'article'}, sort_keys=True))
               for c in gold['cases'] for a in c['annotations']}
    covered = {(c.get('provision_id'), json.dumps(c['locator'], sort_keys=True))
               for c in doc['cases']}
    require(queries <= covered, 'G0 development queries not covered: '
            + str(sorted(queries - covered)))
    require(counts == {'development': 40, 'synthetic': 28}, 'case counts ' + str(counts))
    print('PASS: ' + str(len(ids)) + ' cases (40 development queries, 28 synthetic query rows); '
          'all 23 G0 development queries covered; hashes, quotes and span boundaries verified.')


if __name__ == '__main__':
    main()
