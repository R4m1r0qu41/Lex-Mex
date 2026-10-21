#!/usr/bin/env python3
"""Find article headings printed inside another provision's text; never edit.

Usage: python3 -I prompts/detect-embedded-article-headings.py REPO COMMIT OUTPUT_JSON

Reads committed `corpus/mx/*/provisions.json` blobs at COMMIT through Git, so a
dirty working tree cannot leak in. Writes machine-detected candidates for the
SQLite `known_structural_debt` seed (docs/plans/sqlite-corpus-database.md, M1).
Every candidate is unreviewed: the detector proposes a locator and a class, the
operator decides whether it is structural debt. Offsets are Unicode code points
with an exclusive end, the same convention as Rust `char` spans.

Classes (a heading must start a line, optionally after `[`):
- embedded_heading: `Artículo N <suffix>.` printed as a heading inside another
  article; its body may be bracketed (invalidated text) — `body_bracketed`.
- bracketed_placeholder: the heading itself is inside brackets, such as
  `[Artículo 11 Bis. Suprimido]`.
A heading in a transitory or annex provision, or a heading equal to the
containing article's own number, is counted in `excluded` and not proposed.
Prose mentions (`antes Artículo 172-A`) do not start a line and never match.
"""
import hashlib
import json
import re
import subprocess
import sys

DETECTOR_VERSION = 'embedded-article-headings-v2'
# Latin qualifiers seen in federal headings. `GRAMMAR_QUALIFIERS` mirrors
# `QUALIFIERS` in crates/lex-parse/src/labels.rs; any other qualifier is
# outside the current label grammar.
GRAMMAR_QUALIFIERS = {
    'bis', 'ter', 'quáter', 'quater', 'quinquies', 'quintus', 'quintes', 'sexies', 'sextus',
    'sexties', 'sexíes', 'septies', 'septimus', 'séptimus', 'octies', 'octavus', 'nonies',
    'novenus', 'decies', 'undecies', 'duodecies', 'terdecies', 'quaterdecies',
    'quinquiesdecies', 'sexiesdecies', 'septdecies', 'octodecies', 'novodecies',
}
OTHER_QUALIFIERS = ['septiesdecies', 'octiesdecies', 'noniesdecies', 'vicies']
QUALIFIER = '|'.join(sorted(GRAMMAR_QUALIFIERS | set(OTHER_QUALIFIERS), key=len, reverse=True))
LETTER = r'(?:CH|LL|RR|[A-ZÑÁÉÍÓÚ])(?![A-Za-zÁÉÍÓÚáéíóúÑñ])'
HEADING = re.compile(
    r'(?mi)^(?P<open>\[)?[ \t]*(?P<heading>(?:artículo|articulo)[ \t]+'
    r'(?P<label>\d+(?:[oº°](?![A-Za-záéíóú]))?'
    r'(?:(?:[ \t]*-[ \t]*|[ \t]+)' + LETTER + r')?'
    r'(?:[ \t]+(?:' + QUALIFIER + r'))?)'
    r'(?:[ \t]*\.[ \t]*[-–]?|[ \t]*[-–](?![A-Za-z0-9])|(?=[ \t]+derogad[oa]\b)))'
)


def git(repo, *args):
    return subprocess.check_output(['git', '-C', repo, *args])


def normalized(label):
    return re.sub(r'[\s\-]+', ' ', label.replace('º', 'o').replace('°', 'o')).strip().lower()


def grammar(label):
    words = label.lower().split()
    if len(words) > 1 and words[-1] not in GRAMMAR_QUALIFIERS and len(words[-1]) > 2:
        return 'qualifier_outside_grammar'
    return 'within_grammar'


def detect(repo, commit):
    commit = git(repo, 'rev-parse', commit).decode().strip()
    paths = sorted(p for p in git(repo, 'ls-tree', '-r', '--name-only', commit, 'corpus/mx').decode().split('\n')
                   if p.endswith('/provisions.json'))
    candidates, excluded = [], {'non_article_provision': 0, 'own_number': 0, 'quoted_after_colon': 0}
    for path in paths:
        raw = git(repo, 'cat-file', 'blob', f'{commit}:{path}')
        blob_oid = git(repo, 'rev-parse', f'{commit}:{path}').decode().strip()
        for ordinal, provision in enumerate(json.loads(raw)):
            text = provision['text']
            for match in HEADING.finditer(text):
                if match.start() == 0:
                    continue
                label = match.group('label').strip()
                preceding = text[:match.start()].rstrip().rsplit('\n', 1)[-1].rstrip()
                if preceding.endswith(':'):
                    excluded['quoted_after_colon'] += 1
                    continue
                if provision['provision_type'] != 'article':
                    excluded['non_article_provision'] += 1
                    continue
                if normalized(label) == normalized(provision['number']):
                    excluded['own_number'] += 1
                    continue
                start = match.start('heading')
                end = match.end('heading')
                tail = text[end:end + 400].split('\n', 1)[0].strip()
                bracketed = match.group('open') is not None
                candidates.append({
                    'instrument_id': provision['instrument_id'],
                    'kind': 'bracketed_placeholder' if bracketed else 'embedded_heading',
                    'printed_label': label,
                    'heading_text': text[start:end],
                    'body_bracketed': (not bracketed) and tail.startswith('['),
                    'label_grammar': grammar(label),
                    'containing_provision_id': provision['id'],
                    'containing_provision_number': provision['number'],
                    'containing_provision_ordinal': ordinal,
                    'start_char': start,
                    'end_char': end,
                    'source_path': path,
                    'source_blob_oid': blob_oid,
                    'source_sha256': hashlib.sha256(raw).hexdigest(),
                    'review_status': 'unreviewed',
                })
    return commit, candidates, excluded


def main():
    repo, commit, output = sys.argv[1:4]
    commit, candidates, excluded = detect(repo, commit)
    by_instrument = {}
    for row in candidates:
        counts = by_instrument.setdefault(row['instrument_id'], {'embedded_heading': 0, 'bracketed_placeholder': 0})
        counts[row['kind']] += 1
    with open(output, 'w', encoding='utf-8') as handle:
        json.dump({
            'detector': DETECTOR_VERSION,
            'corpus_commit': commit,
            'review_status': 'unreviewed',
            'offset_unit': 'unicode_code_point_exclusive_end',
            'candidate_count': len(candidates),
            'excluded_match_counts': excluded,
            'counts_by_instrument': dict(sorted(by_instrument.items())),
            'candidates': candidates,
        }, handle, ensure_ascii=False, indent=2)
        handle.write('\n')
    print(len(candidates), 'candidates;', excluded)


if __name__ == '__main__':
    main()
