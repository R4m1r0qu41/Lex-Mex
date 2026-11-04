#!/usr/bin/env python3
"""Summarize human-scored runtime trials; validate cited literal spans against a frozen oracle.

This checks supplied evidence, not whether a model really fetched it. Preserve the transcript.
No model APIs, network, database writes or legal auto-grading.
"""
import argparse
import json
import statistics
from pathlib import Path

DIMENSIONS = ('access', 'source_fidelity', 'legal_reading', 'scope', 'usability')
FLAGS = {'wrong_instrument', 'altered_quote', 'invented_norm', 'false_freshness', 'false_coverage'}
LAYERS = {'access', 'discovery', 'retrieval', 'source_data', 'reasoning', 'presentation', 'none'}


def evaluate(run, key):
    expected = {c['id'] for c in key['cases']}
    rows = run['cases']
    if len(rows) != len(expected) or {c['id'] for c in rows} != expected:
        raise ValueError('Require each of the ten question IDs exactly once')
    if run['mode'] not in ('public', 'local'):
        raise ValueError('mode must be public or local; compare like with like')
    for field in ('model', 'model_version', 'started_at', 'entrypoint', 'observed_release', 'transcript_path'):
        value = run.get(field)
        if not isinstance(value, str) or not value.strip() or value.startswith('REQUIRED'):
            raise ValueError(f'Fill run field {field}')
    results = []
    for row in rows:
        status = row['status']
        if status not in ('answered', 'blocked', 'not_run'):
            raise ValueError(f"{row['id']}: invalid status")
        if set(row['scores']) != set(DIMENSIONS):
            raise ValueError(f"{row['id']}: require five scoring dimensions")
        if not set(row['critical_flags']) <= FLAGS:
            raise ValueError(f"{row['id']}: unknown critical flag")
        if status == 'answered':
            if not isinstance(row['answer'], str) or not row['answer'].strip():
                raise ValueError(f"{row['id']}: preserve the answer")
            if any(type(row['scores'][d]) is not int or not 0 <= row['scores'][d] <= 2 for d in DIMENSIONS):
                raise ValueError(f"{row['id']}: answered rows require integer scores 0..2")
            if row['scores']['access'] == 0:
                raise ValueError(f"{row['id']}: no access must be blocked, not an answered trial")
        elif any(v is not None for v in row['scores'].values()):
            raise ValueError(f"{row['id']}: blocked/not_run scores must stay null")
        if status != 'not_run' and row.get('failure_layer') not in LAYERS:
            raise ValueError(f"{row['id']}: identify failure_layer (or none)")
        flags = set(row['critical_flags'])
        checks = []
        for citation in row['evidence']:
            ref = key['evidence'].get(citation.get('id'))
            quote = citation.get('quote')
            if ref is None:
                # The oracle includes selected witnesses only, not every valid citation.
                checks.append({'id': citation.get('id'), 'result': 'manual_review_unlisted'})
            elif not isinstance(quote, str) or not quote:
                checks.append({'id': citation.get('id'), 'result': 'no_literal_span_supplied'})
            elif quote not in ref['text']:
                flags.add('altered_quote')
                checks.append({'id': citation['id'], 'result': 'literal_span_mismatch'})
            else:
                checks.append({'id': citation['id'], 'result': 'literal_span_matches'})
        seconds = row.get('seconds')
        if seconds is not None and (type(seconds) not in (int, float) or seconds < 0):
            raise ValueError(f"{row['id']}: seconds must be nonnegative or null")
        score = sum(row['scores'].values()) if status == 'answered' else None
        results.append({'id': row['id'], 'status': status, 'score_out_of_10': score,
                        'critical_flags': sorted(flags), 'span_checks': checks,
                        'pass': status == 'answered' and score >= 8 and not flags})
    answered = [x for x in results if x['status'] == 'answered']
    times = [r['seconds'] for r in rows if r['status'] == 'answered' and r.get('seconds') is not None]
    return {'model': run['model'], 'mode': run['mode'], 'observed_release': run['observed_release'],
            'answered': len(answered), 'blocked': sum(x['status'] == 'blocked' for x in results),
            'not_run': sum(x['status'] == 'not_run' for x in results),
            'points_on_answered': sum(x['score_out_of_10'] for x in answered),
            'maximum_on_answered': 10 * len(answered),
            'passed': sum(x['pass'] for x in results),
            'median_answer_seconds': statistics.median(times) if times else None,
            'pilot_pass': all(x['pass'] for x in results), 'cases': results,
            'caution': 'Scores are human judgments. Literal checks do not prove retrieval, completeness or legal truth. Blocked trials are access failures, not legal knowledge scores.'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('run', type=Path)
    parser.add_argument('--key', type=Path, default=Path(__file__).resolve().parents[1] / 'docs/runtime-proof-answer-key-2026-10-28.json')
    args = parser.parse_args()
    try:
        result = evaluate(json.loads(args.run.read_text()), json.loads(args.key.read_text()))
    except (ValueError, KeyError, TypeError) as error:
        parser.error(str(error))
    print(json.dumps(result, ensure_ascii=False, indent=2))


if __name__ == '__main__':
    main()
