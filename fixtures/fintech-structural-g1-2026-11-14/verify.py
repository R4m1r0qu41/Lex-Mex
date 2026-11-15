#!/usr/bin/env python3
"""Check the G1 fixtures' bindings, span invariants, branch coverage and response links.

Implements no G1 rule. It checks text hashes, exact quotes, that spans sit on paragraph
boundaries, that each row's deciding branch allows its outcome and reason, that the ordering
pairs are witnessed, that every G0 development query is covered with the gold label span, and
that each full response agrees with its row and with the text it binds. Schema validity is
checked separately by schema-check/. It never reads the sealed holdout.
"""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[1]
G0 = REPO / 'fixtures' / 'fintech-structural-g0-2026-11-14'
MODES = ('label', 'intro', 'complete')
OUTCOMES = {'resolved', 'ambiguous', 'not_structured', 'not_found', 'stale_projection',
            'invalid_locator'}
BRANCHES = ['V', 'S', 'N', 'R', 'L0', 'L1', 'L2', 'L3', 'L4', 'L5', 'L6', 'L7']
PATH_ORDER = ['L0', 'L1', 'L2', 'L3', 'L4', 'L5', 'L6', 'L7']
ORDER_PAIRS = {('L1', 'L2'), ('L1', 'L4'), ('L2', 'L3'), ('L2', 'L4'), ('L3', 'L4')}
RELEASE = 'local-c337dae1af9474d0aef246b3de196c49e2527722'


def sha(text):
    return hashlib.sha256(text.encode('utf-8')).hexdigest()


def require(condition, message):
    if not condition:
        raise SystemExit('FAIL: ' + message)


def paragraph_end(text, end):
    return end == len(text) or text[end:end + 2] == '\n\n'


def paragraph_start(text, start):
    return start == 0 or text[start - 2:start] == '\n\n'


def allowed(branch, mode, result):
    """The outcome and reason each deciding branch allows (G1.7 tables)."""
    o, r, a = result['outcome'], result.get('reason'), result.get('ambiguity')
    if branch == 'V':
        return o == 'invalid_locator'
    if branch == 'S':
        return o == 'stale_projection' and r in ('hash_mismatch', 'version_mismatch', 'release_mismatch')
    if branch == 'N':
        return o == 'not_structured' and r in ('unsupported_scope', 'unexpected_layout')
    if branch in ('R', 'L4'):
        if branch == 'R' and mode == 'label':
            return False
        if branch == 'L4' and mode == 'label':
            return o == 'resolved'
        return o == 'resolved' or (o == 'ambiguous' and a == 'span' and r == 'unresolved_boundary')
    if branch == 'L0':
        return r == 'incomplete_path' and (o == 'not_structured' or (o == 'ambiguous' and a == 'path'))
    if branch == 'L1':
        return o == 'ambiguous' and a == 'path' and r == 'ambiguous_parent'
    if branch == 'L2':
        return o == 'ambiguous' and a == 'path' and r in ('same_line_marker', 'unsupported_marker')
    if branch == 'L3':
        return o == 'ambiguous' and a == 'path' and r == 'sequence_break'
    if branch == 'L5':
        return o == 'not_found' and r == 'absent_in_complete_scan'
    if branch == 'L6':
        return o == 'not_structured' and r == 'sequence_gap'
    if branch == 'L7':
        return o == 'not_structured' and r in ('partial_scan', 'sequence_break')
    return False


def check_span_result(text, cid, mode, result, label, is_root):
    if result['outcome'] == 'resolved':
        start, end = result['span']
        empty_root_intro = is_root and mode == 'intro' and start == end == 0
        require(0 <= start < end <= len(text) or empty_root_intro, 'range ' + cid + ' ' + mode)
        require(sha(text[start:end]) == result['substring_sha256'], 'substring ' + cid)
        if mode == 'label':
            require(text[start:end] == result['exact_quote'], 'label quote ' + cid)
            require(paragraph_start(text, start), 'label not at paragraph start ' + cid)
        elif not empty_root_intro:
            require(paragraph_end(text, end), mode + ' end not at paragraph end ' + cid)
        if empty_root_intro:
            require(text.startswith(('I', 'V', 'X', 'L', 'C')) or text[:2].endswith(')'),
                    'empty root intro needs a text that opens with a marker ' + cid)
        if is_root:
            require(start == 0, 'root span start ' + cid)
            if mode == 'complete':
                require(end == len(text), 'root complete must be the whole text ' + cid)
        elif mode == 'intro':
            require(text[label[1]:start].isspace() and not text[start].isspace(), 'intro start ' + cid)
        elif mode == 'complete':
            require(start == label[0], 'complete start ' + cid)
    certain = result.get('certain_span')
    if certain:
        if is_root:
            require(certain[0] == 0, 'root certain start ' + cid)
        elif mode == 'complete':
            require(certain[0] == label[0], 'certain start ' + cid)
        else:
            require(text[label[1]:certain[0]].isspace(), 'certain intro start ' + cid)
        before_same_line = any(
            s['reason'] == 'same_line_marker' and text[certain[1] - 1] in ':;.'
            and text[certain[1]:s['start_char']].isspace()
            for s in result.get('unresolved_segments', []))
        require(paragraph_end(text, certain[1]) or before_same_line, 'certain end ' + cid)
    if result.get('ambiguity') == 'span':
        require('certain_span' in result and result.get('unresolved_segments'), 'span ambiguity fields ' + cid)
        starts = [s['start_char'] for s in result['unresolved_segments']]
        require(starts == sorted(starts), 'segments not in start order ' + cid)
    if result.get('ambiguity') == 'path':
        require(result.get('candidates'), 'path ambiguity without candidates ' + cid)
    for segment in result.get('unresolved_segments', []):
        start, end = segment['start_char'], segment['end_char']
        require(0 <= start < end <= len(text) and paragraph_end(text, end), 'segment ' + cid)
        if segment['reason'] != 'same_line_marker':
            require(paragraph_start(text, start), 'segment start ' + cid)
        if certain:
            require(certain[1] <= start, 'certain part overlaps a segment ' + cid)
    for candidate in result.get('candidates', []):
        start, end = candidate['label_span']
        require(text[start:end] == candidate['exact_quote'], 'candidate quote ' + cid)
    starts = [c['label_span'][0] for c in result.get('candidates', [])]
    require(starts == sorted(starts), 'candidates not in document order ' + cid)


def row_request(row, mode_key):
    if 'request' in row:
        return row['request']
    request = {'provision_id': row.get('provision_id') or row['synthetic_provision_id'],
               'locator': row['locator']}
    if mode_key != 'complete':
        request['mode'] = mode_key
    return request


def main():
    snapshot = {p['record']['id']: p['record']['text'] for p in
                json.loads((G0 / 'canonical-snapshot.json').read_text(encoding='utf-8'))['provisions']}
    split = {s['provision_id']: s['split'] for s in
             json.loads((G0 / 'sample-manifest.json').read_text(encoding='utf-8'))['samples']}
    doc = json.loads((ROOT / 'expected-behaviour.json').read_text(encoding='utf-8'))
    rows, texts, counts = {}, {}, {'development': 0, 'synthetic': 0}
    branches_seen, pairs_seen = set(), set()
    for case in doc['cases']:
        cid = case['case_id']
        require(cid not in rows, 'duplicate case ' + cid)
        rows[cid] = case
        counts[case['source']] += 1
        if 'synthetic_text' in case:
            text = case['synthetic_text']
            require(sha(text) == case['synthetic_text_sha256'], 'synthetic hash ' + cid)
            require(case['synthetic_provision_id'].startswith('urn:lex-mx:synthetic:'), 'synthetic id ' + cid)
        else:
            pid = case['provision_id']
            require(split.get(pid) == 'development', 'not a development provision: ' + cid)
            text = snapshot[pid]
            require(sha(text) == case['canonical_text_sha256'], 'text binding ' + cid)
        texts[cid] = text
        decision = case['decision']
        branch = decision['branch']
        require(branch in BRANCHES, 'branch ' + cid)
        branches_seen.add(branch)
        for loser in decision['also_applies']:
            require(branch in PATH_ORDER and loser in PATH_ORDER
                    and PATH_ORDER.index(loser) > PATH_ORDER.index(branch),
                    'also_applies must list later path branches ' + cid)
            pairs_seen.add((branch, loser))
        is_root = isinstance(case.get('locator'), dict) and case['locator'].get('article_root') is True \
            and branch == 'R'
        expected = case['expected']
        label = expected.get('label', {}).get('span')
        for mode, result in expected.items():
            require(mode in MODES or (mode == 'request' and 'request' in case), 'mode ' + cid)
            require(result['outcome'] in OUTCOMES, 'outcome ' + cid)
            require(result['outcome'] == 'resolved' or result.get('reason'), 'reason missing ' + cid)
            require(allowed(branch, mode, result), 'branch ' + branch + ' does not allow '
                    + result['outcome'] + '/' + str(result.get('reason')) + ' in ' + cid + ' ' + mode)
            check_span_result(text, cid, mode, result, label, is_root)
    require(branches_seen == set(BRANCHES), 'branches not covered: ' + str(set(BRANCHES) - branches_seen))
    require(ORDER_PAIRS <= pairs_seen, 'ordering pairs not witnessed: ' + str(ORDER_PAIRS - pairs_seen))

    gold = json.loads((G0 / 'development-gold.json').read_text(encoding='utf-8'))
    queries, gold_spans = set(), {}
    for c in gold['cases']:
        for a in c['annotations']:
            key = (c['provision_id'], json.dumps({k: v for k, v in a['locator'].items() if k != 'article'},
                                                 sort_keys=True))
            queries.add(key)
            if a['expected_outcome'] == 'resolved':
                gold_spans[key] = [a['label_span']['start_char'], a['label_span']['end_char']]
    covered = {(c.get('provision_id'), json.dumps(c.get('locator'), sort_keys=True)): c for c in doc['cases']
               if c['source'] == 'development'}
    require(queries <= set(covered), 'G0 development queries not covered: ' + str(sorted(queries - set(covered))))
    for key, span in gold_spans.items():
        require(covered[key]['expected']['label']['span'] == span, 'G0 label span ' + covered[key]['case_id'])
    require(len(queries) == 23 and len(gold_spans) == 20, 'G0 query or resolved label count')

    responses = json.loads((ROOT / 'responses.json').read_text(encoding='utf-8'))['responses']
    seen = set()
    for entry in responses:
        cid, mode_key, resp = entry['case_id'], entry['mode_key'], entry['response']
        row = rows[cid]
        result = row['expected'][mode_key]
        text = texts[cid]
        tag = cid + '/' + mode_key
        require(resp['outcome'] == result['outcome'] and resp.get('reason') == result.get('reason'),
                'response outcome or reason differs from its row ' + tag)
        require(resp['request'] == row_request(row, mode_key), 'request echo ' + tag)
        request = resp['request']
        raw_mode = request.get('mode', 'complete') if isinstance(request, dict) else None
        require(resp['mode'] == (raw_mode if raw_mode in MODES else None), 'mode rule ' + tag)
        pid = row.get('synthetic_provision_id') or row['provision_id']
        prefix = pid + '#pilot-v1:' + sha(text)[:12] + ':'
        if resp['outcome'] == 'stale_projection':
            require(resp['projection_binding']['canonical_text_sha256'] == sha(text), 'projected binding ' + tag)
            require(resp['current_binding']['canonical_text_sha256'] == sha(row['current_text']), 'current binding ' + tag)
            require(resp['projection_binding']['canonical_text_sha256']
                    != resp['current_binding']['canonical_text_sha256'], 'stale without a hash difference ' + tag)
        elif resp['outcome'] != 'invalid_locator':
            require(resp['provision_id'] == pid and resp['canonical_text_sha256'] == sha(text)
                    and resp['source_release_id'] == RELEASE, 'binding ' + tag)
        if 'node_id' in resp:
            require(resp['node_id'].startswith(prefix), 'node_id prefix ' + tag)
        if resp['outcome'] == 'resolved':
            span = [resp['span']['start_char'], resp['span']['end_char']]
            require(span == result['span'] and resp['text'] == text[span[0]:span[1]], 'resolved span or text ' + tag)
        if resp.get('ambiguity') == 'span':
            certain = resp['certain_span']
            require((certain and [certain['start_char'], certain['end_char']]) == result['certain_span'],
                    'certain span ' + tag)
            require([(s['start_char'], s['end_char'], s['reason'], s['candidate_owner_node_ids'])
                     for s in resp['unresolved_segments']]
                    == [(s['start_char'], s['end_char'], s['reason'], [prefix + p for p in s['candidate_owner_paths']])
                        for s in result['unresolved_segments']], 'segments ' + tag)
        if resp.get('ambiguity') == 'path':
            require([(c['path'], [c['label_span']['start_char'], c['label_span']['end_char']],
                      c['node_id']) for c in resp['candidates']]
                    == [(c['path'], c['label_span'], prefix + c['path'] if c['node_id'] else None)
                        for c in result['candidates']], 'candidates ' + tag)
        seen.add((resp['outcome'], resp.get('ambiguity'), resp.get('reason') if resp['outcome'] in
                  ('invalid_locator', 'not_structured') else None))
        if resp['outcome'] == 'resolved':
            seen.add(('resolved-path', resp['node_id'][len(prefix):].split('.')[0]))
            if resp['span']['start_char'] == resp['span']['end_char']:
                seen.add(('resolved-empty-root-intro',))
    for needed in [('resolved', None, None), ('ambiguous', 'path', None), ('ambiguous', 'span', None),
                   ('not_found', None, None), ('not_structured', None, 'unsupported_scope'),
                   ('not_structured', None, 'unexpected_layout'),
                   ('not_structured', None, 'partial_scan'), ('not_structured', None, 'sequence_gap'),
                   ('not_structured', None, 'sequence_break'), ('stale_projection', None, None),
                   ('invalid_locator', None, 'unknown_mode'), ('invalid_locator', None, 'unknown_key'),
                   ('invalid_locator', None, 'unsupported_request_form'),
                   ('invalid_locator', None, 'malformed_request'), ('resolved-path', 'root'),
                   ('resolved-path', 'i'), ('resolved-empty-root-intro',)]:
        require(needed in seen, 'responses do not cover ' + str(needed))

    must_fail = json.loads((ROOT / 'responses-must-fail.json').read_text(encoding='utf-8'))['cases']
    keys = {e['case_id'] + '/' + e['mode_key'] for e in responses}
    for entry in must_fail:
        require(entry['derived_from'] in keys and entry['why'], 'must-fail entry ' + str(entry.get('why')))

    example = json.loads((ROOT / 'projection-example.json').read_text(encoding='utf-8'))
    projection = example['projection']
    base = [c for c in doc['cases'] if c['case_id'].startswith(example['case_ref'] + '-q')]
    require(base and projection['canonical_text_sha256'] == base[0]['synthetic_text_sha256']
            and projection['provision_id'] == base[0]['synthetic_provision_id'], 'projection example binding')
    text = base[0]['synthetic_text']
    for item in projection['nodes']:
        require(item['node_id'] == projection['provision_id'] + '#pilot-v1:' + sha(text)[:12] + ':' + item['path'],
                'projection node_id ' + item['path'])
        if item['label_span']:
            require(text[item['label_span']['start_char']:item['label_span']['end_char']] == item['printed_label'],
                    'projection label ' + item['path'])

    print('PASS: ' + str(len(rows)) + ' rows (' + str(counts['development']) + ' development, '
          + str(counts['synthetic']) + ' synthetic); all 23 G0 development queries covered and all 20 G0 '
          'label spans equal; every branch and ordering pair witnessed; ' + str(len(responses))
          + ' full responses agree with their rows; ' + str(len(must_fail)) + ' must-fail entries well formed; '
          'projection example bound; hashes, quotes and span boundaries verified.')


if __name__ == '__main__':
    main()
