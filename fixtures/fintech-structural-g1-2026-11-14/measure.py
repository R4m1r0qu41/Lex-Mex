#!/usr/bin/env python3
"""Measurement probe behind the counts quoted in the G1 contract (pilot-v1, draft 2).

Applies the G1.3 classes and the G1.4 tail rule, as written, to eligible non-holdout articles
(lritf, lic, lmv). The eight public holdout provision IDs are excluded before any text is read.
This is a measurement probe, not the projection, and it produced no expectation. It prints
the input commit and the SHA-256 of each corpus file it read, so G3 can bind the counts.
Run from the lex-mex repository root: python3 fixtures/fintech-structural-g1-2026-11-14/measure.py
"""
import collections
import hashlib
import json
import re
import subprocess

HOLDOUT = {
    'urn:lex-mx:federal:statute:lritf:article:4', 'urn:lex-mx:federal:statute:lritf:article:52',
    'urn:lex-mx:federal:statute:lritf:article:58', 'urn:lex-mx:federal:statute:lic:article:16',
    'urn:lex-mx:federal:statute:lic:article:22-bis', 'urn:lex-mx:federal:statute:lic:article:42',
    'urn:lex-mx:federal:statute:lmv:article:35', 'urn:lex-mx:federal:statute:lmv:article:64-bis-1',
}
QUALIFIERS = ['bis', 'ter', 'quater', 'quinquies', 'sexies', 'septies', 'octies', 'nonies', 'decies']
LETTERS = 'abcdefghijklmnñopqrstuvwxyz'
VALUES = {'I': 1, 'V': 5, 'X': 10, 'L': 50, 'C': 100, 'D': 500, 'M': 1000}


def rank(word):
    word = word.lower().replace('á', 'a')
    return QUALIFIERS.index(word) + 1 if word in QUALIFIERS else None


def roman_value(numeral):
    """Value if `numeral` is in canonical Roman form, else None."""
    total = 0
    for index, char in enumerate(numeral):
        value = VALUES[char]
        nxt = VALUES[numeral[index + 1]] if index + 1 < len(numeral) else 0
        total += -value if nxt > value else value
    out, n = '', total
    for value, symbol in [(1000, 'M'), (900, 'CM'), (500, 'D'), (400, 'CD'), (100, 'C'),
                          (90, 'XC'), (50, 'L'), (40, 'XL'), (10, 'X'), (9, 'IX'), (5, 'V'),
                          (4, 'IV'), (1, 'I')]:
        while n >= value:
            out, n = out + symbol, n - value
    return total if out == numeral else None


def fraction_marker(text):
    """Python reading of lex_core::fraction_marker: (numeral, qualifier, end_char) or None."""
    m = re.match(r'[IVXLCDM]+', text)
    if not m:
        return None
    numeral = m.group(0)
    rest = text[len(numeral):]
    spaced = rest[1:] if rest.startswith('.') else rest
    word_start = spaced.lstrip()
    if len(word_start) < len(spaced):
        word = re.match(r'[^\W\d_]*', word_start).group(0)
        if rank(word):
            label_end = len(text) - len(word_start) + len(word)
            tail = text[label_end:]
            sep = re.match(r'\.[-–]?(?=\s)', tail)
            if sep:
                return numeral, word, label_end + len(sep.group(0))
    sep = re.match(r'\.[-–]?(?=\s)', rest)
    if sep:
        return numeral, None, len(numeral) + len(sep.group(0))
    return None


NOTE = re.compile(r'^Artículo (?:adicionado|reformado|derogado)\b.*DOF \d{2}-\d{2}-\d{4}')
UNSUPPORTED = re.compile(r'^(?:[IVXLCDM]+(?![^\W\d_])|[A-Z]\.\s|\d+[.)°ºo]|[ivxlcdm]+[.)]'
                         r'|[a-zñ]{2,}\)|[a-zñ]\.\s|[-–•]\s)')
SAME_LINE_FRACTION_CONTEXT = re.compile(r'[:;.]\s+(?=[IVXLCDM])')
SAME_LINE_INCISO = re.compile(r'[:;.]\s+([a-zñ])\)\s+[^\W\d_]')


def classify(paragraph, previous_base):
    marker = fraction_marker(paragraph)
    if marker:
        numeral, qualifier, end = marker
        value = roman_value(numeral)
        lone_ok = numeral not in ('L', 'C', 'D', 'M') or \
            (numeral == 'L' and previous_base == 49) or (numeral == 'C' and previous_base == 99)
        if value and value <= 100 and lone_ok:
            return 'F', (value, rank(qualifier) if qualifier else 0)
        return 'U', None
    if re.match(r'^[a-zñ]\)\s', paragraph):
        return 'I', LETTERS.index(paragraph[0])
    if NOTE.match(paragraph) and len(paragraph) <= 400:
        return 'N', None
    if UNSUPPORTED.match(paragraph):
        return 'U', None
    return 'C', None


def same_line(paragraph):
    fractions = 0
    for m in SAME_LINE_FRACTION_CONTEXT.finditer(paragraph):
        marker = fraction_marker(paragraph[m.end():])
        if marker and re.match(r'\s+[^\W\d_]', paragraph[m.end() + marker[2]:]):
            fractions += 1
    return fractions, len(SAME_LINE_INCISO.findall(paragraph))


def main():
    commit = subprocess.run(['git', 'rev-parse', 'HEAD'], capture_output=True, text=True, check=True).stdout.strip()
    dirty = subprocess.run(['git', 'status', '--porcelain', '--', 'corpus/mx/lritf/provisions.json',
                            'corpus/mx/lic/provisions.json', 'corpus/mx/lmv/provisions.json'],
                           capture_output=True, text=True, check=True).stdout.strip()
    print(f'input commit: {commit}' + (' (corpus files modified in the worktree)' if dirty else ''))
    for slug in ('lritf', 'lic', 'lmv'):
        with open(f'corpus/mx/{slug}/provisions.json', 'rb') as handle:
            print(f'corpus/mx/{slug}/provisions.json sha256 {hashlib.sha256(handle.read()).hexdigest()}')
    counts = collections.Counter()
    development = {c['provision_id'] for c in json.load(open(
        'fixtures/fintech-structural-g0-2026-11-14/development-gold.json'))['cases']}
    dev_hits = []
    for slug in ('lritf', 'lic', 'lmv'):
        for provision in json.load(open(f'corpus/mx/{slug}/provisions.json')):
            if provision['id'] in HOLDOUT or provision.get('provision_type') != 'article':
                continue
            text = provision['text']
            counts['eligible_articles'] += 1
            if re.search(r'(?<!\n)\n(?!\n)', text) or '\n\n\n' in text or text != text.strip():
                counts['unexpected_layout'] += 1
            kinds, base = [], None
            for paragraph in text.split('\n\n'):
                kind, label = classify(paragraph, base)
                if kind == 'F':
                    base = label[0]
                kinds.append((kind, label))
                counts['class_' + kind] += 1
                f, i = same_line(paragraph)
                counts['same_line_fraction'] += f
                counts['same_line_inciso'] += i
                if (f or i) and provision['id'] in development:
                    dev_hits.append(provision['id'])
            for index, (kind, _) in enumerate(kinds):
                if kind == 'N' and any(k != 'N' for k, _ in kinds[index + 1:]):
                    counts['interior_notes'] += 1
            if not any(kind == 'F' for kind, _ in kinds) and any(kind == 'I' for kind, _ in kinds):
                counts['no_fraction_inciso_articles'] += 1
                last_inciso = max(i for i, (kind, _) in enumerate(kinds) if kind == 'I')
                if any(kind != 'N' for kind, _ in kinds[last_inciso + 1:]):
                    counts['no_fraction_inciso_articles_with_tail'] += 1
            fractions = [label for kind, label in kinds if kind == 'F']
            if any(b <= a for a, b in zip(fractions, fractions[1:])):
                counts['fraction_sequence_breaks'] += 1
            bases = [v for v, q in fractions if q == 0]
            if any(b - a > 1 for a, b in zip(bases, bases[1:])):
                counts['articles_with_fraction_gap'] += 1
            # inciso sequences per parent (each fraction region, or the root if no fraction)
            run, broke = [], False
            for kind, label in kinds:
                if kind == 'F':
                    run = []
                elif kind == 'I':
                    if run and label <= run[-1]:
                        broke = True
                    run.append(label)
            counts['inciso_sequence_breaks'] += broke
            if not fractions:
                continue
            counts['fraction_articles'] += 1
            last = max(i for i, (kind, _) in enumerate(kinds) if kind == 'F')
            rest = kinds[last + 1:]
            j = 0
            if rest and rest[0][0] == 'I':
                letters, j = [], 0
                for index, (kind, label) in enumerate(rest):
                    if kind == 'I':
                        if letters and label <= letters[-1]:
                            break
                        letters.append(label)
                        j = index + 1
                    elif kind != 'C':
                        break
            remainder = [kind for kind, _ in rest[j:]]
            if any(kind != 'N' for kind in remainder):
                counts['tail'] += 1
                if 'I' in remainder:
                    counts['tails_with_incisos'] += 1
                tail_end = max(i for i, kind in enumerate(remainder) if kind != 'N')
                counts['interior_notes_inside_tails'] += sum(1 for kind in remainder[:tail_end] if kind == 'N')
            elif remainder:
                counts['ends_on_notes'] += 1
            else:
                counts['ends_on_marker'] += 1
    for key in sorted(counts):
        print(f'{key}: {counts[key]}')
    print('development provisions with same-line candidates:', sorted(set(dev_hits)))


if __name__ == '__main__':
    main()
