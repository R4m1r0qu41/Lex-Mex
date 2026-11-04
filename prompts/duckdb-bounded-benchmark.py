#!/usr/bin/env python3
"""Opt-in analytics experiment; no production dependencies or extension loading."""
import argparse
import csv
import hashlib
import json
from pathlib import Path
import platform
import resource
import shutil
import sqlite3
import statistics
import subprocess
import sys
import time

VERSION = '1.4.3'
TABLES = {
    'instruments': ['id'],
    'provisions': ['id', 'instrument_id', 'text'],
    'reference_edges': ['source_provision_id', 'target_instrument_id', 'resolution_status'],
    'defined_terms': ['id', 'instrument_id'],
    'term_usages': ['provision_id', 'term_id'],
}
QUERIES = {
    'provision_volume': '''SELECT instrument_id, COUNT(*), SUM(LENGTH(text))
        FROM provisions GROUP BY instrument_id ORDER BY instrument_id''',
    'citation_destinations': '''SELECT target_instrument_id, resolution_status, COUNT(*)
        FROM reference_edges GROUP BY target_instrument_id, resolution_status
        ORDER BY target_instrument_id, resolution_status''',
    'term_usage_by_owner': '''SELECT d.instrument_id, COUNT(*)
        FROM term_usages u JOIN defined_terms d ON u.term_id=d.id
        GROUP BY d.instrument_id ORDER BY d.instrument_id''',
    'citation_origin_destination': '''SELECT p.instrument_id, e.target_instrument_id, COUNT(*)
        FROM reference_edges e JOIN provisions p ON e.source_provision_id=p.id
        GROUP BY p.instrument_id,e.target_instrument_id
        ORDER BY p.instrument_id,e.target_instrument_id''',
}


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def write_json(path, data):
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n')


def readonly(path):
    connection = sqlite3.connect(path.resolve().as_uri() + '?mode=ro&immutable=1', uri=True)
    connection.execute('PRAGMA query_only=ON')
    return connection


def worker(engine, directory):
    started = time.perf_counter()
    if engine == 'sqlite':
        connection = readonly(directory / 'snapshot.sqlite')
        version = sqlite3.sqlite_version
        import_seconds = 0.0
        table_counts = {t: connection.execute(f'SELECT COUNT(*) FROM {t}').fetchone()[0]
                        for t in TABLES}
    else:
        import duckdb  # Installed only into the opt-in experiment environment.
        if duckdb.__version__ != VERSION:
            raise ValueError(f'Expected duckdb {VERSION}, got {duckdb.__version__}')
        version = duckdb.__version__
        connection = duckdb.connect(str(directory / 'analytics.duckdb'), config={
            'threads': '1', 'memory_limit': '256MB',
            'autoinstall_known_extensions': 'false', 'autoload_known_extensions': 'false',
        })
        connection.execute('SET max_temp_directory_size=\'256MB\'')
        import_start = time.perf_counter()
        table_counts = {}
        for table, columns in TABLES.items():
            # Inputs are CSVs exported from NOT NULL TEXT columns. No inference.
            types = '{' + ','.join(f"'{c}':'VARCHAR'" for c in columns) + '}'
            connection.execute(f'''CREATE TABLE {table} AS SELECT * FROM
                read_csv(?, header=true, columns={types}, nullstr='\\N',
                allow_quoted_nulls=false)''', [str(directory / f'{table}.csv')])
            table_counts[table] = connection.execute(f'SELECT COUNT(*) FROM {table}').fetchone()[0]
        connection.execute('CHECKPOINT')
        import_seconds = time.perf_counter() - import_start
    results = {}
    for name, query in QUERIES.items():
        times = []
        expected = None
        for _ in range(4):  # First process query + three warm repetitions.
            t = time.perf_counter()
            rows = connection.execute(query).fetchall()
            times.append(time.perf_counter() - t)
            encoded = json.dumps(rows, ensure_ascii=False, separators=(',', ':')).encode()
            signature = hashlib.sha256(encoded).hexdigest()
            if expected is not None and signature != expected:
                raise ValueError(f'Nondeterministic query: {name}')
            expected = signature
        results[name] = {'row_count': len(rows), 'result_sha256': expected,
                         'first_in_process_seconds': times[0],
                         'warm_seconds': times[1:], 'warm_median_seconds': statistics.median(times[1:])}
    connection.close()
    peak = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
    write_json(directory / f'{engine}.json', {
        'version': version, 'table_counts': table_counts, 'queries': results,
        'import_seconds': import_seconds, 'worker_wall_seconds': time.perf_counter() - started,
        'process_peak_rss_bytes': peak if sys.platform == 'darwin' else peak * 1024,
    })


def benchmark(source, manifest_path, output, timeout=120):
    if output.exists():
        raise ValueError('Output already exists; use a fresh directory')
    manifest = json.loads(manifest_path.read_text())
    if source.stat().st_size > 1024 ** 3:
        raise ValueError('MVP input limit is 1 GiB')
    for suffix in ['-wal', '-journal']:
        sidecar = Path(str(source) + suffix)
        if sidecar.exists() and sidecar.stat().st_size:
            raise ValueError(f'Not an immutable release: {sidecar}')
    before = digest(source)
    if manifest['database_sha256'] != before:
        raise ValueError('Database SHA does not match manifest')
    output.mkdir(parents=True)
    write_json(output / 'status.json', {'status': 'running'})
    try:
        started = time.perf_counter()
        snapshot = output / 'snapshot.sqlite'
        shutil.copyfile(source, snapshot)
        if digest(snapshot) != before:
            raise ValueError('Snapshot differs from accepted source')
        connection = readonly(snapshot)
        release_rows = connection.execute('SELECT id,specification FROM corpus_release').fetchall()
        if len(release_rows) != 1 or release_rows[0][0] != manifest['release_id']:
            raise ValueError('Embedded release identity differs from manifest')
        if json.loads(release_rows[0][1]) != manifest['specification']:
            raise ValueError('Embedded release specification differs from manifest')
        write_json(output / 'release-row.json', release_rows)
        counts = {}
        export_start = time.perf_counter()
        for table, columns in TABLES.items():
            counts[table] = connection.execute(f'SELECT COUNT(*) FROM {table}').fetchone()[0]
            if counts[table] > 500_000:
                raise ValueError(f'{table}: exceeds 500,000-row MVP bound')
            with (output / f'{table}.csv').open('w', newline='') as stream:
                writer = csv.writer(stream)
                writer.writerow(columns)
                for row in connection.execute(f'SELECT {",".join(columns)} FROM {table}'):
                    if any(value is None or not isinstance(value, str) or value == '\\N' for value in row):
                        raise ValueError('Unexpected nullable/non-text/sentinel value; stop, do not coerce')
                    writer.writerow(row)
        connection.close()
        export_seconds = time.perf_counter() - export_start
        for engine in ['sqlite', 'duckdb']:
            with (output / f'{engine}.stdout').open('wb') as out, (output / f'{engine}.stderr').open('wb') as err:
                subprocess.run([sys.executable, str(Path(__file__).resolve()), '--worker', engine,
                                '--output', str(output)], check=True, timeout=timeout,
                               stdout=out, stderr=err)
        engines = {engine: json.loads((output / f'{engine}.json').read_text())
                   for engine in ['sqlite', 'duckdb']}
        if any(engine['table_counts'] != counts for engine in engines.values()):
            raise ValueError('Imported table counts differ')
        parity = all(engines['sqlite']['queries'][q]['result_sha256'] ==
                     engines['duckdb']['queries'][q]['result_sha256'] for q in QUERIES)
        if not parity:
            raise ValueError('Query result parity failed; do not interpret timings')
        if digest(source) != before or digest(snapshot) != before:
            raise ValueError('Source or snapshot modified')
        result = {'status': 'pass', 'query_parity': parity,
                  'source': str(source), 'manifest_sha256': digest(manifest_path),
                  'database_sha256': before, 'release_id': manifest['release_id'],
                  'logical_content_sha256': manifest['logical_content_sha256'],
                  'python': sys.version, 'platform': platform.platform(),
                  'driver_sha256': digest(Path(__file__)),
                  'dependency_pin_sha256': digest(Path(__file__).with_name('duckdb-benchmark-requirements.txt')),
                  'queries': QUERIES,
                  'export_seconds': export_seconds, 'engines': engines,
                  'sqlite_bytes': snapshot.stat().st_size,
                  'duckdb_bytes': (output / 'analytics.duckdb').stat().st_size,
                  'wall_seconds': time.perf_counter() - started,
                  'limitations': ['First query is not OS-cold; source copy/export warms filesystem cache',
                                 'SQLite uses existing indexes; DuckDB stores only five selected table projections',
                                 'Footprints are different projections, not compression ratios',
                                 'RSS includes engine/import;256MB is DuckDB buffer limit, not total process cap',
                                 'No FTS, legal inference, temporal derivation or serving benchmark']}
        write_json(output / 'report.json', result)
        write_json(output / 'status.json', {'status': 'pass'})
        return result
    except BaseException as exc:
        write_json(output / 'status.json', {'status': 'failed', 'error': str(exc),
                   'source_unchanged': digest(source) == before})
        raise


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path)
    parser.add_argument('--manifest', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--timeout', type=int, default=120, choices=range(1, 301))
    parser.add_argument('--worker', choices=['sqlite', 'duckdb'], help=argparse.SUPPRESS)
    args = parser.parse_args()
    if args.worker:
        worker(args.worker, args.output.resolve())
        return
    worktree = Path(subprocess.check_output(['git', 'rev-parse', '--show-toplevel'], text=True).strip()).resolve()
    if '.delta' not in worktree.parts or not (worktree / '.git').is_file():
        parser.error('Run from an operator-created isolated .delta worktree')
    if not args.source or not args.manifest:
        parser.error('--source and --manifest are required')
    output = args.output.resolve()
    if not output.is_relative_to(worktree / '.work'):
        parser.error('--output must be inside this worktree .work directory')
    if not subprocess.check_output(['git', 'rev-parse', '--git-common-dir'], text=True).strip():
        parser.error('Expected linked Git worktree')
    result = benchmark(args.source.resolve(), args.manifest.resolve(), output, args.timeout)
    result['worktree_head'] = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
    write_json(output / 'report.json', result)
    print(json.dumps({'status': result['status'], 'report': str(output / 'report.json')}))


if __name__ == '__main__':
    main()
