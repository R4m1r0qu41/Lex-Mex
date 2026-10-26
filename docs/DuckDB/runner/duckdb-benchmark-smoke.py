#!/usr/bin/env python3
"""Synthetic cross-engine contract checks, not corpus performance evidence."""
import importlib.util
import json
from pathlib import Path
import sqlite3
import tempfile

spec = importlib.util.spec_from_file_location('benchmark', Path(__file__).with_name('duckdb-bounded-benchmark.py'))
benchmark = importlib.util.module_from_spec(spec)
spec.loader.exec_module(benchmark)

with tempfile.TemporaryDirectory(prefix='lex-duckdb-smoke-') as temporary:
    root = Path(temporary)
    source = root / 'source.sqlite'
    connection = sqlite3.connect(source)
    for table, columns in benchmark.TABLES.items():
        connection.execute(f'CREATE TABLE {table} ({",".join(c+" TEXT NOT NULL" for c in columns)})')
    connection.execute('CREATE TABLE corpus_release (id TEXT, specification TEXT)')
    connection.execute("INSERT INTO corpus_release VALUES ('synthetic', '{}')")
    connection.executemany('INSERT INTO instruments VALUES (?)', [('law-a',), ('law-b',)])
    connection.executemany('INSERT INTO provisions VALUES (?,?,?)', [
        ('p-a', 'law-a', '\nArtículo áé\r\n"exacto", segunda línea'),
        ('p-b', 'law-b', 'ñ 👩🏽‍⚖️'), ('p-c', 'law-a', '')])
    connection.executemany('INSERT INTO reference_edges VALUES (?,?,?)', [
        ('p-a', 'law-b', 'resolved'), ('title-anchor', 'law-b', 'unresolved')])
    connection.execute("INSERT INTO defined_terms VALUES ('t-a','law-a')")
    connection.executemany('INSERT INTO term_usages VALUES (?,?)', [('p-a', 't-a'), ('p-c', 't-a')])
    connection.commit()
    connection.close()
    manifest = root / 'manifest.json'
    data = {'release_id': 'synthetic', 'specification': {},
            'database_sha256': benchmark.digest(source), 'logical_content_sha256': 'synthetic-not-a-corpus'}
    manifest.write_text(json.dumps(data))
    result = benchmark.benchmark(source, manifest, root / 'out', timeout=30)
    assert result['query_parity'] and len(result['engines']['duckdb']['queries']) == 4
    assert result['engines']['duckdb']['table_counts']['provisions'] == 3
    assert benchmark.digest(source) == data['database_sha256']
    print('PASS cross-engine parity: Unicode, CRLF, leading newline, empty text, joins, title anchor; source unchanged')
    try:
        benchmark.benchmark(source, manifest, root / 'out')
        raise AssertionError('Output replacement accepted')
    except ValueError as error:
        assert 'already exists' in str(error)
    print('PASS existing output refused')
    data['database_sha256'] = '0' * 64
    manifest.write_text(json.dumps(data))
    try:
        benchmark.benchmark(source, manifest, root / 'bad-manifest')
        raise AssertionError('Wrong manifest accepted')
    except ValueError as error:
        assert 'SHA' in str(error)
    print('PASS wrong manifest refused')
    data['database_sha256'] = benchmark.digest(source)
    data['release_id'] = 'wrong-release'
    manifest.write_text(json.dumps(data))
    try:
        benchmark.benchmark(source, manifest, root / 'bad-release')
        raise AssertionError('Wrong embedded identity accepted')
    except ValueError as error:
        assert 'identity' in str(error)
    assert json.loads((root / 'bad-release/status.json').read_text())['status'] == 'failed'
    print('PASS embedded identity mismatch fails with failure receipt')

    # DuckDB 1.4.3 sniffs escape as empty after a long quote-free sample, then rejects a
    # later RFC 4180 doubled quote. The loader must pin csv.writer's dialect instead.
    import csv
    import duckdb
    fixture = root / 'sniffer.csv'
    quoted = 'dice "exacto" y\nmas, coma'
    with fixture.open('w', newline='') as stream:
        writer = csv.writer(stream)
        writer.writerow(['id', 'instrument_id', 'text'])
        writer.writerow(['a', 'b', 'c,c,c'])
        writer.writerows(('p', 'i', 'pad, pad') for _ in range(2046))
        writer.writerow(['last', 'i', quoted])
    probe = duckdb.connect(config={'threads': '1'})
    probe.execute("SET memory_limit='256MB'")
    benchmark.load_projection_csv(probe, 'provisions', benchmark.TABLES['provisions'], fixture)
    loaded = probe.execute("SELECT text FROM provisions WHERE id='last'").fetchone()
    assert loaded == (quoted,)
    assert probe.execute('SELECT COUNT(*) FROM provisions').fetchone() == (2048,)
    probe.close()
    print('PASS dialect pin preserves a doubled quote past the sniffer sample')
