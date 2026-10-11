#!/usr/bin/env python3
"""Read-only validator for one compound full-workspace batch execution record.

Usage: python3 workspace-batches-validator.py EXECUTION_DIR FRESH_VALIDATION_DIR
The output directory must not exist and must be disjoint from EXECUTION_DIR.
"""
from pathlib import Path
import datetime
import hashlib
import json
import re
import sys

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
SUMMARY_RE = re.compile(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;')
TARGET_LIMIT = 1024 ** 3
FREE_LIMIT = 5 * 1024 ** 3


def sha_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha_file(path: Path) -> str:
    return sha_bytes(path.read_bytes())


def inside(path: Path, parent: Path) -> bool:
    try:
        path.relative_to(parent)
        return True
    except ValueError:
        return False


def validate(run_dir: Path) -> dict:
    failures = []
    def check(condition: bool, message: str) -> None:
        if not condition:
            failures.append(message)

    run_dir = run_dir.resolve(strict=True)
    manifest_path = run_dir / 'execution.json'
    check(manifest_path.is_file(), 'execution.json is missing')
    record = json.loads(manifest_path.read_bytes())
    check(record.get('schema') == 'compound-full-workspace-batched-root-v1', 'unexpected execution schema')
    check(record.get('status') == 'passed', f"runner status is {record.get('status')!r}")
    check(not record.get('error'), f"runner recorded error: {record.get('error')}")
    check(record.get('no_carried_execution') is True, 'no_carried_execution is not true')
    check(record.get('guard_limits') == {'free_bytes': FREE_LIMIT, 'target_bytes': TARGET_LIMIT},
          'guard limits differ from required 5 GiB free / 1 GiB target')

    def check_source_map(source_map, label):
        if not isinstance(source_map, dict):
            check(False, f'{label} is missing or malformed')
            return
        for relative, expected in source_map.items():
            rel = Path(relative)
            path = (ROOT / rel).resolve()
            check(not rel.is_absolute() and inside(path, ROOT.resolve()), f'{label} path escapes workspace: {relative}')
            if not path.is_file():
                check(False, f'{label} source is missing: {relative}')
            elif sha_file(path) != expected:
                check(False, f'{label} source pin changed: {relative}')

    check_source_map(record.get('source_pins_before_format'), 'source_pins_before_format')
    check_source_map(record.get('source_pins'), 'source_pins')
    check(record.get('source_pins_before_format') == record.get('source_pins'),
          'workspace source pins changed between initial capture and post-format capture')
    check(record.get('pins_unchanged') is True, 'runner did not confirm unchanged source pins')
    check(not record.get('pin_errors'), f"runner recorded pin read errors: {record.get('pin_errors')}")
    check(not record.get('pin_changes'), f"runner recorded changed pins: {record.get('pin_changes')}")

    for key in ('guard_template', 'target_size_scan_adapter', 'runner'):
        item = record.get(key)
        check(isinstance(item, dict) and bool(item.get('path')) and bool(item.get('sha256')),
              f'{key} pin is missing')
        if isinstance(item, dict) and item.get('path'):
            p = Path(item['path'])
            if p.is_file():
                check(sha_file(p) == item.get('sha256'), f'{key} changed since execution')
            else:
                check(False, f'{key} file is unavailable: {p}')

    metadata_raw_path = run_dir / '1.stdout.raw'
    check(metadata_raw_path.is_file(), 'metadata stdout raw is missing')
    metadata = None
    if metadata_raw_path.is_file():
        try:
            metadata = json.loads(metadata_raw_path.read_bytes())
        except Exception as exc:
            check(False, f'metadata stdout raw is not JSON: {type(exc).__name__}: {exc}')
    metadata_copy = run_dir / 'metadata.json'
    check(metadata_copy.is_file(), 'metadata.json copy is missing')
    if metadata_raw_path.is_file() and metadata_copy.is_file():
        check(metadata_raw_path.read_bytes() == metadata_copy.read_bytes(), 'metadata.json differs from metadata raw')

    expected_batches = []
    expected_targets = []
    expected_ids_by_batch = []
    package_names = {}
    if metadata is not None:
        members = set(metadata.get('workspace_members', []))
        for package in metadata.get('packages', []):
            if package.get('id') not in members:
                continue
            name = package['name']
            package_id = package['id']
            package_names[package_id] = name
            base = ['cargo', 'test', '-p', name, '--all-features', '--locked', '--message-format', 'json']
            targets = package.get('targets', [])
            unit = [t for t in targets if t.get('kind') and t['kind'][0] in ('lib', 'bin', 'example', 'bench')]
            if unit:
                flags = []
                for kind, flag in [('lib', '--lib'), ('bin', '--bins'), ('example', '--examples'), ('bench', '--benches')]:
                    if any(t['kind'][0] == kind for t in unit):
                        flags.append(flag)
                expected_batches.append(base + flags)
                expected_ids_by_batch.append({(package_id, t['name'], t['kind'][0]) for t in unit})
            tests = [t for t in targets if t.get('kind') and t['kind'][0] == 'test']
            for offset in range(0, len(tests), 4):
                batch = tests[offset:offset + 4]
                expected_batches.append(base + sum((['--test', t['name']] for t in batch), []))
                expected_ids_by_batch.append({(package_id, t['name'], 'test') for t in batch})
            expected_targets.extend((name, t['name'], t['kind'][0]) for t in targets)
        check(sorted(map(tuple, record.get('target_inventory', []))) == sorted(expected_targets),
              'recorded target inventory does not match all workspace metadata targets')
        check(len(record.get('target_inventory', [])) == len(set(map(tuple, record.get('target_inventory', [])))),
              'recorded target inventory contains duplicates')
        check(record.get('expected_batch_count') == len(expected_batches), 'expected batch count mismatch')

    format_row = record.get('format_command')
    metadata_row = record.get('metadata_command')
    clean_row = record.get('clean_command')
    batch_rows = record.get('commands', [])
    all_rows = [format_row, metadata_row, *batch_rows, clean_row]
    check(format_row is not None and format_row.get('argv') == ['cargo', 'fmt', '--all', '--', '--check'],
          'format argv mismatch or missing')
    check(metadata_row is not None and metadata_row.get('argv') == ['cargo', 'metadata', '--no-deps', '--format-version', '1', '--locked'],
          'metadata argv mismatch or missing')
    check(len(batch_rows) == len(expected_batches), 'completed batch row count differs from metadata batch count')
    if len(batch_rows) == len(expected_batches):
        for i, (row, expected_argv) in enumerate(zip(batch_rows, expected_batches), start=2):
            check(row.get('index') == i, f'batch row index mismatch at position {i}')
            check(row.get('argv') == expected_argv, f'batch argv mismatch at index {i}')
    check(clean_row is not None and clean_row.get('argv') == ['cargo', 'clean'], 'clean argv mismatch or missing')
    if clean_row is not None:
        check(clean_row.get('index') == len(batch_rows) + 2, 'clean command index mismatch')

    raw_seen = set()
    for row in all_rows:
        if not isinstance(row, dict):
            check(False, 'a command row is missing')
            continue
        check(row.get('exit_code') == 0, f"command {row.get('index')} did not exit 0")
        check(row.get('guard_stop') is None, f"command {row.get('index')} hit the runtime guard")
        check(isinstance(row.get('peak_target_bytes'), int) and row.get('peak_target_bytes') <= TARGET_LIMIT,
              f"command {row.get('index')} exceeds target size guard or lacks measurement")
        check(isinstance(row.get('free_bytes_after'), int) and row.get('free_bytes_after') >= FREE_LIMIT,
              f"command {row.get('index')} violates free-space guard or lacks measurement")
        for stream_name in ('stdout', 'stderr'):
            stream = row.get('streams', {}).get(stream_name)
            check(isinstance(stream, dict), f"command {row.get('index')} lacks {stream_name} stream pin")
            if not isinstance(stream, dict):
                continue
            raw_path = Path(stream.get('path', ''))
            if not raw_path.is_absolute():
                raw_path = (ROOT / raw_path).resolve()
            check(raw_path.name == f"{row.get('index')}.{stream_name}.raw", f"command {row.get('index')} has unexpected {stream_name} raw path")
            check(raw_path.parent.resolve() == run_dir, f"command {row.get('index')} {stream_name} raw escapes execution directory")
            check(raw_path not in raw_seen, f'duplicate raw stream path: {raw_path}')
            raw_seen.add(raw_path)
            if not raw_path.is_file():
                check(False, f'missing raw stream: {raw_path}')
            else:
                content = raw_path.read_bytes()
                check(stream.get('bytes') == len(content), f'raw stream byte count mismatch: {raw_path.name}')
                check(stream.get('sha256') == sha_bytes(content), f'raw stream SHA-256 mismatch: {raw_path.name}')

    expected_raw_names = {f'{row.get("index")}.{stream}.raw'
                          for row in all_rows if isinstance(row, dict)
                          for stream in ('stdout', 'stderr')}
    actual_raw_names = {p.name for p in run_dir.glob('*.raw')}
    check(actual_raw_names == expected_raw_names, 'raw file inventory differs from the complete command stream inventory')

    actual_all = []
    executable_paths = []
    summaries = []
    for batch_index, row in enumerate(batch_rows):
        i = batch_index + 2
        raw_path = run_dir / f'{i}.stdout.raw'
        if not raw_path.is_file():
            continue
        text = raw_path.read_text(errors='replace')
        counts = [tuple(map(int, m)) for m in SUMMARY_RE.findall(text)]
        check(bool(counts), f'batch {i} has no successful Rust test summaries')
        check(all(failed == 0 for _, failed, _ in counts), f'batch {i} has failing test count')
        check(row.get('observed_counts') == [list(c) for c in counts], f'batch {i} summary counts differ from raw')
        summaries.extend(counts)

        emitted = []
        for line in text.splitlines():
            if not line.startswith('{'):
                continue
            try:
                event = json.loads(line)
            except Exception:
                continue
            executable = event.get('executable')
            if event.get('reason') == 'compiler-artifact' and event.get('profile', {}).get('test') and executable:
                target = event.get('target', {})
                kind = target.get('kind', [None])[0]
                emitted.append((event.get('package_id'), target.get('name'), kind))
                executable_path = Path(executable)
                path = (executable_path if executable_path.is_absolute() else ROOT / executable_path).resolve()
                executable_paths.append(str(path))
                check(inside(path, ROOT.resolve() / 'target'), f'batch {i} test executable escapes target: {path}')
        observed = [tuple(x) for x in row.get('executed_targets', [])]
        check(emitted == observed, f'batch {i} recorded executable targets differ from compiler-artifact raw')
        check(len(observed) == len(set(observed)), f'batch {i} executed duplicate test targets')
        if batch_index < len(expected_ids_by_batch):
            check(set(observed) == expected_ids_by_batch[batch_index],
                  f'batch {i} target set differs from its selected metadata targets')
        actual_all.extend(observed)
        summary_check = row.get('test_summary_check')
        if summary_check is not None:
            check(summary_check.get('ok') is True, f'batch {i} guard summary callback failed')

    expected_ids = {(p['id'], t['name'], t['kind'][0])
                    for p in metadata.get('packages', []) if p['id'] in set(metadata.get('workspace_members', []))
                    for t in p.get('targets', [])} if metadata is not None else set()
    check(set(actual_all) == expected_ids, 'aggregate executed target IDs differ from complete workspace inventory')
    check(len(actual_all) == len(set(actual_all)), 'aggregate executed target list contains duplicates')
    totals = [sum(c[j] for c in summaries) for j in range(3)]
    check(record.get('workspace_summary_count') == len(summaries), 'aggregate summary count mismatch')
    check(record.get('workspace_totals') == totals, 'aggregate test totals mismatch')

    removed = record.get('removed_completed_test_executables', [])
    removed_paths = []
    for item in removed:
        p = Path(item.get('path', '')).resolve()
        check(inside(p, ROOT.resolve() / 'target'), f'removed test executable path escapes target: {p}')
        check(isinstance(item.get('bytes'), int) and item['bytes'] > 0, f'invalid removed executable byte count: {p}')
        check(bool(re.fullmatch(r'[0-9a-f]{64}', item.get('sha256', ''))), f'invalid removed executable hash: {p}')
        check(not p.exists(), f'completed test executable still exists: {p}')
        removed_paths.append(str(p))
    check(len(removed_paths) == len(set(removed_paths)), 'removed executable list contains duplicate paths')
    check(sorted(removed_paths) == sorted(executable_paths),
          'removed executable records do not cover exactly the test executables emitted in raw streams')

    check(record.get('target_absent_after_clean') is True, 'runner did not confirm target absent after clean')
    clean_raw_path = run_dir / f"{clean_row.get('index')}.stdout.raw" if clean_row else None
    check(not (ROOT / 'target').exists(), 'workspace target directory exists at validation time')
    validation_ok = not failures
    return {
        'schema': 'compound-full-workspace-batch-validation-v1',
        'utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'status': 'passed' if validation_ok else 'failed',
        'execution_dir': str(run_dir),
        'execution_sha256': sha_file(manifest_path),
        'metadata_target_count': len(expected_targets),
        'expected_batch_count': len(expected_batches),
        'completed_batch_count': len(batch_rows),
        'workspace_summary_count': len(summaries),
        'workspace_totals': totals,
        'raw_stream_count': len(raw_seen),
        'removed_test_executable_count': len(removed),
        'clean_raw_present': bool(clean_raw_path and clean_raw_path.is_file()),
        'failures': failures,
    }


def main() -> int:
    if len(sys.argv) != 3:
        print('usage: validator.py EXECUTION_DIR FRESH_VALIDATION_DIR', file=sys.stderr)
        return 2
    run_dir = Path(sys.argv[1]).resolve()
    output_dir = Path(sys.argv[2]).absolute()
    if output_dir.exists():
        print(f'output directory already exists; refusing to overwrite: {output_dir}', file=sys.stderr)
        return 2
    if inside(output_dir.resolve(), run_dir) or inside(run_dir, output_dir.resolve()):
        print('execution and validation directories must be disjoint', file=sys.stderr)
        return 2
    output_dir.mkdir(parents=True, exist_ok=False)
    try:
        report = validate(run_dir)
    except BaseException as exc:
        report = {
            'schema': 'compound-full-workspace-batch-validation-v1',
            'utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
            'status': 'failed',
            'execution_dir': str(run_dir),
            'failures': [f'validator exception: {type(exc).__name__}: {exc}'],
        }
    (output_dir / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({k: report.get(k) for k in ('status', 'metadata_target_count', 'completed_batch_count', 'workspace_totals', 'failures')}))
    return 0 if report.get('status') == 'passed' else 1


if __name__ == '__main__':
    raise SystemExit(main())
