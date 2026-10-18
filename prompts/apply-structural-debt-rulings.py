#!/usr/bin/env python3
"""Apply recorded operator rulings to structural-debt candidates; never edit text.

Usage: python3 -I prompts/apply-structural-debt-rulings.py CANDIDATES_JSON OUTPUT_JSON

Input is the output of prompts/detect-embedded-article-headings.py. Each
candidate gains a mechanical `series_check` and a review status taken only
from the rulings in docs/decisions.md (2026-10-17, "Latin ordinal qualifiers
and the structural-debt candidate classes"). A row no ruling covers stays
`unreviewed`. Statuses: accepted, rejected, deferred, unreviewed.
"""
import hashlib
import json
import re
import sys

DECISION = 'docs/decisions.md#2026-10-17-latin-ordinal-qualifiers-and-the-structural-debt-candidate-classes'
REVIEWER = 'operator'
REVIEWED_AT = '2026-10-17'
DCG_PASS = {'urn:lex-mx:federal:regulation:oaac-dcg-2009', 'urn:lex-mx:federal:regulation:socap-sofipo-dcg-2006'}
RANK = {
    'bis': 1, 'ter': 2, 'quáter': 3, 'quater': 3, 'quinquies': 4, 'quintus': 4, 'quintes': 4,
    'sexies': 5, 'sextus': 5, 'sexties': 5, 'sexíes': 5, 'septies': 6, 'septimus': 6, 'séptimus': 6,
    'octies': 7, 'octavus': 7, 'nonies': 8, 'novenus': 8, 'decies': 9, 'undecies': 10,
    'duodecies': 11, 'terdecies': 12, 'quaterdecies': 13, 'quinquiesdecies': 14,
    'sexiesdecies': 15, 'septdecies': 16, 'octodecies': 17, 'novodecies': 18,
}
LETTERS = 'abcdefghijklmnñopqrstuvwxyz'


def next_letters(letter):
    """Letters that may follow `letter`; `ñ` is optional (LGTOC runs n, o)."""
    following = LETTERS[LETTERS.index(letter) + 1:]
    return set(following[:2]) if following[:1] == 'ñ' else set(following[:1])


def parts(label):
    """Split `228 a bis`, `69 C Bis`, `78 Decies`, `69-C` into (base, letter, rank)."""
    words = re.sub(r'\s*-\s*', ' ', label.lower()).split()
    base = re.match(r'\d+', words[0]).group()
    letter = next((w for w in words[1:] if len(w) == 1 and w in LETTERS), None)
    rank = next((RANK[w] for w in words[1:] if w in RANK), 0)
    return base, letter, rank


def series_order(rows):
    """True when the headings continue the container's series without a gap."""
    base, letter, rank = parts(rows[0]['containing_provision_number'])
    expected = (letter, rank)
    for row in rows:
        b, l, r = parts(row['printed_label'])
        if b != base:
            return False
        if l == expected[0] and r == expected[1] + 1:
            expected = (l, r)
        elif l is not None and r == 0 and (l == 'a' if expected[0] is None else l in next_letters(expected[0])):
            expected = (l, 0)
        else:
            return False
    return True


def review(row, series_ok):
    if row['kind'] == 'bracketed_placeholder':
        return 'rejected', 'legally_removed_placeholder', 'ruling 2: bracketed placeholders are text that was legally removed'
    if row['instrument_id'] in DCG_PASS:
        return 'deferred', 'merged_heading', 'ruling 3: held DCG candidates wait for the DCG pass'
    if row['body_bracketed']:
        return 'unreviewed', 'merged_heading', 'heading printed with a bracketed body; ruling 2 covers bracketed placeholders, extension to these rows not ruled'
    if series_ok:
        return 'accepted', 'merged_heading', 'ruling 1: heading continues the containing article\'s numbering series (series_check passed)'
    return 'unreviewed', 'merged_heading', 'series_check failed; no ruling applies'


def main():
    source, output = sys.argv[1:3]
    raw = open(source, 'rb').read()
    data = json.loads(raw)
    groups = {}
    for row in data['candidates']:
        if row['kind'] == 'embedded_heading':
            groups.setdefault(row['containing_provision_id'], []).append(row)
    ok = {pid: series_order(rows) for pid, rows in groups.items()}
    rows, counts = [], {}
    for row in data['candidates']:
        series_ok = ok.get(row['containing_provision_id'], False) if row['kind'] == 'embedded_heading' else False
        status, kind, basis = review(row, series_ok)
        reviewed = status in ('accepted', 'rejected', 'deferred')
        rows.append({**row, 'series_check': series_ok, 'kind': kind, 'detected_kind': row['kind'],
                     'review_status': status, 'review_basis': basis,
                     'reviewer': REVIEWER if reviewed else None,
                     'reviewed_at': REVIEWED_AT if reviewed else None,
                     'decision_reference': DECISION if reviewed else None})
        counts[status] = counts.get(status, 0) + 1
    with open(output, 'w', encoding='utf-8') as handle:
        json.dump({
            'candidates_source': source,
            'candidates_sha256': hashlib.sha256(raw).hexdigest(),
            'corpus_commit': data['corpus_commit'],
            'detector': data['detector'],
            'decision_reference': DECISION,
            'status_counts': dict(sorted(counts.items())),
            'rows': rows,
        }, handle, ensure_ascii=False, indent=2)
        handle.write('\n')
    print(dict(sorted(counts.items())))


if __name__ == '__main__':
    main()
