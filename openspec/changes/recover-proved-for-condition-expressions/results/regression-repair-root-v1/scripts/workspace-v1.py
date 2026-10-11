#!/usr/bin/env python3
"""Run the guarded full-workspace test batches for the for-condition candidate."""

from __future__ import annotations

import datetime
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import runpy
import signal
import stat
import subprocess
import sys
import time

sys.dont_write_bytecode = True
ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
OUT = Path('/private/tmp/jarde-for-condition-workspace-root-v1')
TEMPLATE = ROOT / 'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
SCAN_PATH = ROOT / 'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
TEMPLATE_SHA256 = '51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
SCAN_SHA256 = 'd954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
EXPECTED_BATCHES = 95
EXPECTED_TARGETS = 360
EXPECTED_SOURCE_FILES = 4718
FREE_LIMIT = 5 * 1024**3
TARGET_LIMIT = 1024**3
STRIPPED_ENV_KEYS = ('JAVA_TOOL_OPTIONS', '_JAVA_OPTIONS', 'JDK_JAVA_OPTIONS', 'CLASSPATH')
DOCELLIPSIS_EXCEPTIONS = {
    ('../tests/fixtures/…', 'crates/jarde-jvm/src/test_fixtures.rs'),
    ('../tests/fixtures/…', 'crates/jarde-reader/src/test_fixtures.rs'),
}
INCLUDE_RE = re.compile(r'(?:include_bytes|include_str)!\s*\(\s*"([^"]+)"')
SUMMARY_RE = re.compile(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;')


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_paths() -> set[Path]:
    paths = {ROOT / 'Cargo.toml', ROOT / 'Cargo.lock', ROOT / '.github/workflows/ci.yml',
             ROOT / 'tests/fixtures/corpus-fingerprint.json'}
    for base in ('src', 'crates', 'tests'):
        paths.update((ROOT / base).rglob('*.rs'))
    paths.update((ROOT / 'crates').glob('*/Cargo.toml'))
    for base in (ROOT / 'tests/fixtures',
                 ROOT / 'openspec/changes/recover-class-scope-constructor-parameters/evidence/frozen'):
        paths.update(path for path in base.rglob('*') if path.is_file())
    for path in list(paths):
        if path.suffix != '.rs':
            continue
        relative_source = str(path.relative_to(ROOT))
        for relative_include in INCLUDE_RE.findall(path.read_text(encoding='utf-8')):
            included = (path.parent / relative_include).resolve()
            if not included.is_relative_to(ROOT):
                raise RuntimeError(f'include escapes workspace: {path}: {relative_include}')
            if not included.is_file():
                if (relative_include, relative_source) in DOCELLIPSIS_EXCEPTIONS:
                    continue
                raise FileNotFoundError(f'missing include: {path}: {relative_include}')
            paths.add(included)
    for path in paths:
        if not path.is_file():
            raise FileNotFoundError(f'missing source pin: {path}')
    return paths


def pin_sources(paths: set[Path]) -> dict[str, str]:
    return {str(path.relative_to(ROOT)): sha256(path) for path in sorted(paths)}


def load_module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f'cannot load guard module: {path}')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main() -> int:
    global record, guard, env, source_file_set, clean_armed
    if OUT.exists():
        raise SystemExit(f'refusing to overwrite prior run output: {OUT}')
    if not TEMPLATE.is_file() or sha256(TEMPLATE) != TEMPLATE_SHA256:
        raise SystemExit('guard template SHA-256 mismatch')
    if not SCAN_PATH.is_file() or sha256(SCAN_PATH) != SCAN_SHA256:
        raise SystemExit('target-size scanner SHA-256 mismatch')
    if (ROOT / 'target').exists():
        raise SystemExit('fresh-target preflight failed: workspace target already exists')
    free_before = __import__('shutil').disk_usage(ROOT).free
    if free_before < FREE_LIMIT:
        raise SystemExit(f'5 GiB free-space preflight failed: {free_before}')

    source_file_set = source_paths()
    source_pins_before = pin_sources(source_file_set)
    if len(source_pins_before) != EXPECTED_SOURCE_FILES:
        raise SystemExit(f'expected {EXPECTED_SOURCE_FILES} source files, got {len(source_pins_before)}')
    OUT.mkdir()
    clean_armed = True

    guard = load_module('for_condition_guard_v1', TEMPLATE)
    scan = runpy.run_path(str(SCAN_PATH), run_name='for_condition_target_scan')
    guard.ROOT = ROOT
    guard.OUT = OUT
    guard.expected_test_summaries = lambda index, raw: None
    guard.command_stream = lambda path: {
        'path': str(path), 'bytes': path.stat().st_size, 'sha256': sha256(path)
    }
    guard.target_bytes = lambda: scan['target_bytes'](ROOT)
    guard.FREE_LIMIT = FREE_LIMIT
    guard.TARGET_LIMIT = TARGET_LIMIT
    env = os.environ.copy()
    stripped = []
    for key in STRIPPED_ENV_KEYS:
        if key in env:
            env.pop(key)
            stripped.append(key)
    env.pop('RUST_MIN_STACK', None)
    env.update(guard.ENV_VALUES)
    env['PROPTEST_RNG_SEED'] = '5350648285461741569'
    env['CARGO_PROFILE_TEST_STRIP'] = 'symbols'
    guard.ENV_VALUES = {**guard.ENV_VALUES,
                        'PROPTEST_RNG_SEED': '5350648285461741569',
                        'CARGO_PROFILE_TEST_STRIP': 'symbols'}
    if 'RUST_MIN_STACK' in env:
        raise RuntimeError('RUST_MIN_STACK must not be set')

    record = {
        'schema': 'for-condition-full-workspace-batched-root-v1',
        'status': 'running',
        'started_at': datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'seed': '5350648285461741569',
        'no_carried_execution': True,
        'source_closure_count_expected': EXPECTED_SOURCE_FILES,
        'source_pins_before': source_pins_before,
        'source_pins_after': None,
        'stripped_environment_keys': stripped,
        'rust_min_stack': env.get('RUST_MIN_STACK'),
        'environment_overrides': guard.ENV_VALUES,
        'fresh_target_preflight': {'target_absent': True, 'free_bytes': free_before},
        'guard_limits': {'minimum_free_bytes': FREE_LIMIT, 'maximum_target_bytes': TARGET_LIMIT,
                         'poll_interval_seconds': 1},
        'references': [
            {'path': str(TEMPLATE), 'sha256': TEMPLATE_SHA256},
            {'path': str(SCAN_PATH), 'sha256': SCAN_SHA256},
            {'path': '/private/tmp/jarde-run-conditional-workspace-batches-root-v5.py',
             'sha256': '15fb42dd044f2371fa8d0c75010f9b87b3474ad4a20c5f9d1007f15af299c76f'},
            {'path': '/private/tmp/jarde-for-condition-focused-root-v3.py',
             'sha256': 'b65b77d235366e22e8d76453b6c966d624a6c0a1eeb1a397cdf71f7236b59c99'},
        ],
        'docellipsis_exceptions': [
            {'include': include, 'source': source}
            for include, source in sorted(DOCELLIPSIS_EXCEPTIONS)
        ],
        'commands': [],
        'removed_completed_test_executables': [],
        'cleanup': {'armed_after_fresh_target_preflight': True},
        'runner': None,
    }
    runner_path = Path(__file__).resolve()
    record['runner'] = {'path': str(runner_path), 'sha256_before': sha256(runner_path)}

    def save() -> None:
        temporary = OUT / 'execution.json.tmp'
        temporary.write_text(json.dumps(record, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
        temporary.replace(OUT / 'execution.json')

    def assert_sources_unchanged(stage: str) -> None:
        current = pin_sources(source_file_set)
        if current != source_pins_before:
            record['source_change_detected_at'] = stage
            record['source_pins_after'] = current
            raise RuntimeError(f'workspace source closure changed at {stage}')

    def command(index: int, argv: list[str]) -> dict:
        assert_sources_unchanged(f'before-command-{index}')
        row = guard.run_command(index, argv, env)
        record['commands'].append(row)
        save()
        if row['exit_code'] != 0 or row['guard_stop'] is not None:
            raise RuntimeError(f'command {index} failed or hit a guard')
        assert_sources_unchanged(f'after-command-{index}')
        return row

    def make_batches(metadata: dict) -> tuple[list[list[str]], list[tuple[str, str, str]]]:
        members = set(metadata['workspace_members'])
        commands: list[list[str]] = []
        coverage: list[tuple[str, str, str]] = []
        for package in metadata['packages']:
            if package['id'] not in members:
                continue
            base = ['cargo', 'test', '-p', package['name'], '--all-features', '--locked',
                    '--message-format', 'json']
            unit_targets = [target for target in package['targets']
                            if target['kind'][0] in ('lib', 'bin', 'example', 'bench')]
            if unit_targets:
                flags = []
                for kind, flag in (('lib', '--lib'), ('bin', '--bins'),
                                   ('example', '--examples'), ('bench', '--benches')):
                    if any(target['kind'][0] == kind for target in unit_targets):
                        flags.append(flag)
                commands.append(base + flags)
                coverage.extend((package['name'], target['name'], target['kind'][0])
                                for target in unit_targets)
            test_targets = [target for target in package['targets']
                            if target['kind'][0] == 'test']
            for start in range(0, len(test_targets), 4):
                batch = test_targets[start:start + 4]
                commands.append(base + sum((['--test', target['name']] for target in batch), []))
                coverage.extend((package['name'], target['name'], 'test') for target in batch)
        expected = [(package['name'], target['name'], target['kind'][0])
                    for package in metadata['packages'] if package['id'] in members
                    for target in package['targets']]
        if sorted(coverage) != sorted(expected) or len(coverage) != len(set(coverage)):
            raise RuntimeError('metadata target coverage is not exact and unique')
        return commands, expected

    try:
        record['runner']['sha256_after'] = sha256(runner_path)
        if record['runner']['sha256_after'] != record['runner']['sha256_before']:
            raise RuntimeError('runner source changed before execution')
        save()
        fmt = command(0, ['cargo', 'fmt', '--all', '--', '--check'])
        record['format_check'] = {'exit_code': fmt['exit_code'], 'guard_stop': fmt['guard_stop']}
        metadata_row = command(1, ['cargo', 'metadata', '--no-deps', '--format-version', '1', '--locked'])
        metadata_bytes = (OUT / '1.stdout.raw').read_bytes()
        (OUT / 'metadata.json').write_bytes(metadata_bytes)
        metadata = json.loads(metadata_bytes)
        batches, expected_targets = make_batches(metadata)
        if len(batches) != EXPECTED_BATCHES or len(expected_targets) != EXPECTED_TARGETS:
            raise RuntimeError(f'expected {EXPECTED_BATCHES} batches / {EXPECTED_TARGETS} targets; '
                               f'got {len(batches)} / {len(expected_targets)}')
        members = set(metadata['workspace_members'])
        expected_ids = [(package['id'], target['name'], target['kind'][0])
                        for package in metadata['packages'] if package['id'] in members
                        for target in package['targets']]
        record['metadata'] = {'command_index': metadata_row['index'],
                              'workspace_member_count': len(members),
                              'target_count': len(expected_targets),
                              'batch_count': len(batches)}
        record['target_inventory'] = expected_targets
        record['expected_batch_count'] = EXPECTED_BATCHES
        record['expected_target_count'] = EXPECTED_TARGETS
        save()

        all_summaries = []
        all_actual_targets = []
        for batch_index, argv in enumerate(batches, start=2):
            row = command(batch_index, argv)
            raw = (OUT / f'{batch_index}.stdout.raw').read_text(encoding='utf-8', errors='replace')
            summaries = [tuple(map(int, match)) for match in SUMMARY_RE.findall(raw)]
            if not summaries or any(failed != 0 for passed, failed, ignored in summaries):
                raise RuntimeError(f'batch {batch_index} has a missing or failing test summary')
            actual_targets = []
            executables = []
            for line in raw.splitlines():
                if not line.startswith('{'):
                    continue
                artifact = json.loads(line)
                if artifact.get('reason') != 'compiler-artifact':
                    continue
                profile = artifact.get('profile', {})
                executable = artifact.get('executable')
                if profile.get('test') and executable:
                    target = artifact['target']
                    actual_targets.append((artifact['package_id'], target['name'], target['kind'][0]))
                    executable_path = Path(executable)
                    if not executable_path.is_absolute():
                        executable_path = ROOT / executable_path
                    if executable_path.is_symlink():
                        raise RuntimeError(f'refusing to remove symlink test executable: {executable_path}')
                    executables.append(executable_path.resolve(strict=True))
            if len(actual_targets) != len(set(actual_targets)) or len(actual_targets) != len(summaries):
                raise RuntimeError(f'batch {batch_index} summary/artifact cardinality mismatch')
            if len(executables) != len(set(executables)):
                raise RuntimeError(f'batch {batch_index} reported duplicate test executables')
            row['observed_counts'] = [list(item) for item in summaries]
            row['executed_targets'] = [list(item) for item in actual_targets]
            row['summary_test_totals'] = [sum(item[index] for item in summaries) for index in range(3)]
            all_summaries.extend(summaries)
            all_actual_targets.extend(actual_targets)
            for executable in executables:
                deps = (ROOT / 'target/debug/deps').resolve(strict=True)
                if executable.parent != deps or executable.is_symlink():
                    raise RuntimeError(f'refusing to remove out-of-scope test executable: {executable}')
                info = executable.lstat()
                if not stat.S_ISREG(info.st_mode) or not info.st_mode & 0o111:
                    raise RuntimeError(f'refusing to remove non-executable/non-regular artifact: {executable}')
                removed = {'path': str(executable), 'bytes': info.st_size,
                           'sha256': sha256(executable), 'completed_batch': batch_index}
                executable.unlink()
                record['removed_completed_test_executables'].append(removed)
            save()
            print(json.dumps({'completed_batch': batch_index - 1,
                              'total_batches': len(batches),
                              'test_totals': row['summary_test_totals']}), flush=True)

        if sorted(all_actual_targets) != sorted(expected_ids) or len(all_actual_targets) != len(set(all_actual_targets)):
            raise RuntimeError('actual executed target set is not exact and unique')
        if sum(item[0] for item in all_summaries) <= 0:
            raise RuntimeError('workspace had no passing tests')
        record['workspace_summary_count'] = len(all_summaries)
        record['workspace_totals'] = [sum(item[index] for item in all_summaries) for index in range(3)]
        record['status'] = 'passed'
    except BaseException as error:
        record['status'] = 'failed'
        record['error'] = f'{type(error).__name__}: {error}'
    finally:
        if clean_armed:
            cleanup = {'attempted': True, 'argv': ['cargo', 'clean'], 'guarded': True}
            try:
                clean_row = guard.run_command(len(record['commands']), ['cargo', 'clean'], env)
                record['commands'].append(clean_row)
                cleanup['command'] = clean_row
                cleanup['status'] = ('passed' if clean_row['exit_code'] == 0
                                     and clean_row['guard_stop'] is None else 'failed')
                if cleanup['status'] != 'passed':
                    record['status'] = 'failed'
                    record.setdefault('error', 'guarded cargo clean failed')
            except BaseException as error:
                cleanup['status'] = 'failed-before-completion'
                cleanup['error'] = f'{type(error).__name__}: {error}'
                record['status'] = 'failed'
                record.setdefault('error', f'guarded cargo clean: {type(error).__name__}: {error}')
            cleanup['target_absent_after'] = not (ROOT / 'target').exists()
            cleanup['source_pins_after'] = pin_sources(source_file_set)
            cleanup['pins_unchanged'] = cleanup['source_pins_after'] == source_pins_before
            if not cleanup['target_absent_after'] or not cleanup['pins_unchanged']:
                record['status'] = 'failed'
                record.setdefault('error', 'cleanup postcondition failed')
            record['cleanup'] = cleanup
        record['source_pins_after'] = pin_sources(source_file_set)
        if record['source_pins_after'] != source_pins_before:
            record['status'] = 'failed'
            record.setdefault('error', 'workspace source pins changed')
        record['runner']['sha256_after'] = sha256(runner_path)
        if record['runner']['sha256_after'] != record['runner']['sha256_before']:
            record['status'] = 'failed'
            record.setdefault('error', 'runner source changed during execution')
        record['finished_at'] = datetime.datetime.now(datetime.timezone.utc).isoformat()
        temporary = OUT / 'execution.json.tmp'
        temporary.write_text(json.dumps(record, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
        temporary.replace(OUT / 'execution.json')
    print(json.dumps({**{key: record.get(key) for key in ('status', 'error', 'expected_batch_count', 'expected_target_count', 'workspace_summary_count', 'workspace_totals')}, 'cleanup_status': record.get('cleanup', {}).get('status'), 'target_absent': record.get('cleanup', {}).get('target_absent_after')}))
    return 0 if record['status'] == 'passed' and record.get('cleanup', {}).get('status') == 'passed' else 1


if __name__ == '__main__':
    raise SystemExit(main())
