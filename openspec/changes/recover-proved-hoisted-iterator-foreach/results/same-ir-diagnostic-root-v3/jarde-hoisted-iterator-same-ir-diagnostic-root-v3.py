#!/usr/bin/env python3
"""Guarded, reversible one-test same-IR diagnostic for TestNestedLoops foreach."""
from __future__ import annotations

import datetime
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import runpy
import shutil
import stat
import sys
import time

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
OUT = Path('/private/tmp/jarde-hoisted-iterator-same-ir-diagnostic-root-v3')
PREP = Path('/private/tmp/jarde-hoisted-iterator-diagnostic-prep-luna-v3')
CANDIDATE_PINS = PREP / 'pins.json'
CANDIDATE_PINS_SHA256 = '7b8d7d63a4cbbebf4bc764c798d7b0fb92c1e2c03935477fcf4bdc6e6f72d9f7'
CANDIDATE_PATCH = PREP / 'candidate.patch'
CANDIDATE_PATCH_SHA256 = 'a8bdef57d2892cadf0165721f38ec3b6e720db315aceea5e96cbe041282c4c45'
FINAL_BUILD = Path('/private/tmp/jarde-constructor-conditional-final-lint-build-root-v1/execution.json')
FINAL_BUILD_SHA256 = '90ecba1b52177d7dfae590679603e4624c4e56a42ae2b58d0616d7078c1f46e2'
FINAL_BUILD_RUNNER = Path('/private/tmp/jarde-constructor-conditional-final-lint-build-root-v1.py')
FINAL_BUILD_RUNNER_SHA256 = '7bf3a61aec009aaa629efad18fe8e177b25d0e3b828de135d52eba498ea5c2c3'
GUARD = ROOT / 'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
GUARD_SHA256 = '51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
SCAN = ROOT / 'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
SCAN_SHA256 = 'd954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
CLASS = Path('/private/tmp/jarde-cf11-testnestedloops-baseline-root-v1/legs/javac23-original/classes/cf11/UpstreamNestedLoops.class')
CLASS_SHA256 = '8ece89e4d38751c20529622b57417afc8601368fc20078656ea2b24c17ed8231'
CLI = Path('/private/tmp/jarde-constructor-conditional-final-cli-root-v1')
CLI_BYTES = 61286928
CLI_SHA256 = 'a71c31e80da7c6550aa572e5368f0ecad12ac862066c5d40535f5818c2263241'
CLI_MODE = 0o555
FINAL_BUILD_SCHEMA = 'constructor-conditional-final-lint-build-root-v1'
FINAL_BUILD_STATUS = 'passed_candidate_build_only'
EXPECTED_SOURCE_FILES = 4727
SEED = '5350648285461741569'
ENV_OPT_IN = 'JARDE_FOREACH_CANDIDATE_DIAGNOSTIC'
TEST_NAME = 'facade::same_class_generic_call_inventory_tests::frozen_upstream_nested_loops_foreach_candidate_diagnostic'
SOURCE_CANDIDATES = {
    'crates/jarde-java/src/build.rs': {
        'candidate': PREP / 'build.rs.candidate',
        'base_sha256': 'f5c0757ec670ec72aa377ea1f84529a576fe595c895a2d3ce1c7c66ee441c5fe',
        'candidate_sha256': 'b7aa470b0721d09867833e4f61d05748c2a4ba0603fa44f0487f9a3d00c4933b',
    },
    'src/facade.rs': {
        'candidate': PREP / 'facade.rs.candidate',
        'base_sha256': '9c55f3a53b0e435ea5c3e2bedac0fe66f32d474aecc9d1237029518f3703f22b',
        'candidate_sha256': '719c8c3253437cbf1d33719b897c90d7a6773e757fe72242be0aa4ae28ac050f',
    },
}
INCLUDE_RE = re.compile(r'(?:include_bytes|include_str)!\s*\(\s*"([^"]+)"')
DOCELLIPSIS_EXCEPTIONS = {
    ('../tests/fixtures/…', 'crates/jarde-jvm/src/test_fixtures.rs'),
    ('../tests/fixtures/…', 'crates/jarde-reader/src/test_fixtures.rs'),
}


def sha_bytes(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def sha_file(path: Path) -> str:
    return sha_bytes(path.read_bytes())


def candidate_tool_pins() -> dict[str, str]:
    pins = json.loads(CANDIDATE_PINS.read_text())
    assert pins['schema'] == 'jarde-hoisted-iterator-same-ir-diagnostic-candidate-v3'
    assert pins['status'] == 'private static diagnostic candidate; unapplied, uncompiled, and unrun'
    assert pins['scope']['test_path'] == TEST_NAME and pins['scope']['environment_opt_in'] == ENV_OPT_IN
    assert pins['scope']['class'] == 'cf11/UpstreamNestedLoops' and pins['scope']['method_descriptor'] == '(Ljava/util/List;Ljava/util/List;)V'
    result = {str(CANDIDATE_PINS): CANDIDATE_PINS_SHA256}
    bases = {x['path']:x['sha256'] for x in pins['base_source_pins']}
    candidates = {x['path']:x['sha256'] for x in pins['candidate_files']}
    assert set(candidates) == {str(x['candidate']) for x in SOURCE_CANDIDATES.values()}
    for rel, row in SOURCE_CANDIDATES.items():
        assert bases[str(ROOT/rel)] == row['base_sha256']
        assert candidates[str(row['candidate'])] == row['candidate_sha256']
    assert pins['patch'] == {'path':str(CANDIDATE_PATCH),'sha256':CANDIDATE_PATCH_SHA256}
    assert pins['frozen_input']['path'] == str(CLASS) and pins['frozen_input']['sha256'] == CLASS_SHA256
    assert pins['diagnostic_safety']['force_refusal_before_rewrite_or_caller_removal'] is True
    result.update(bases);result.update(candidates);result[str(CANDIDATE_PATCH)]=CANDIDATE_PATCH_SHA256
    result[str(CLASS)]=CLASS_SHA256
    baseline=pins['frozen_input']['baseline_execution'];result[baseline['path']]=baseline['sha256']
    for entry in pins['diagnostic_delta'].values():
        if isinstance(entry, dict) and 'path' in entry and 'sha256' in entry:
            result[entry['path']] = entry['sha256']
    for entry in pins.get('private_review_files', []):
        result[entry['path']] = entry['sha256']
    for text,digest in result.items():
        q=Path(text)
        assert q.is_file() and not q.is_symlink() and sha_file(q)==digest, text
    return result


def verify_runner_tool_pins() -> dict[str, str]:
    expected = {
        str(FINAL_BUILD): FINAL_BUILD_SHA256,
        str(FINAL_BUILD_RUNNER): FINAL_BUILD_RUNNER_SHA256,
        str(GUARD): GUARD_SHA256,
        str(SCAN): SCAN_SHA256,
        str(CLI): CLI_SHA256,
    }
    for path_text, digest in expected.items():
        path = Path(path_text)
        if not path.is_file() or sha_file(path) != digest:
            raise RuntimeError(f'guard/build/tool pin mismatch: {path}')
    cli_stat = CLI.stat()
    if cli_stat.st_size != CLI_BYTES or stat.S_IMODE(cli_stat.st_mode) != CLI_MODE:
        raise RuntimeError('final-build CLI byte count or mode mismatch')
    build = json.loads(FINAL_BUILD.read_text(encoding='utf-8'))
    cli = build.get('candidate_cli')
    if (build.get('schema') != FINAL_BUILD_SCHEMA or build.get('status') != FINAL_BUILD_STATUS
            or build.get('runner_sha256_before') != FINAL_BUILD_RUNNER_SHA256
            or build.get('runner_sha256_after') != FINAL_BUILD_RUNNER_SHA256
            or not build.get('candidate_byte_identity') or not isinstance(cli, dict)
            or cli.get('path') != str(CLI) or cli.get('bytes') != CLI_BYTES
            or cli.get('sha256') != CLI_SHA256 or cli.get('mode') != '0o555'):
        raise RuntimeError('final-build receipt/CLI identity is not the pinned candidate build')
    expected_env = {
        'CARGO_BUILD_JOBS': '1', 'CARGO_INCREMENTAL': '0',
        'CARGO_PROFILE_DEV_DEBUG': '0', 'CARGO_PROFILE_TEST_DEBUG': '0',
        'RUST_TEST_THREADS': '1', 'CARGO_TERM_COLOR': 'always',
        'PROPTEST_RNG_SEED': SEED,
    }
    if build.get('actual_env_overrides') != expected_env or build.get('rust_min_stack') is not None:
        raise RuntimeError('final-build receipt does not bind the requested default Rust stack/seed')
    return expected


def enumerate_baseline_source_paths() -> set[Path]:
    """Match constructor-conditional-final-build-v1's frozen source closure before the external include is applied."""
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
                raise RuntimeError(f'baseline include escapes workspace: {path}: {relative_include}')
            if not included.is_file():
                if (relative_include, relative_source) in DOCELLIPSIS_EXCEPTIONS:
                    continue
                raise FileNotFoundError(f'missing baseline include: {path}: {relative_include}')
            paths.add(included)
    for path in paths:
        if not path.is_file():
            raise FileNotFoundError(f'missing baseline source pin: {path}')
    return paths


def save(record: dict) -> None:
    temporary = OUT / 'execution.json.tmp'
    temporary.write_text(json.dumps(record, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    temporary.replace(OUT / 'execution.json')


def ensure_raw_pair(index: int) -> tuple[Path, Path]:
    stdout_path, stderr_path = OUT / f'{index}.stdout.raw', OUT / f'{index}.stderr.raw'
    for path in (stdout_path, stderr_path):
        if not path.exists():
            path.write_bytes(b'')
    return stdout_path, stderr_path


def run_guarded(guard, index: int, argv: list[str], env: dict[str, str]) -> dict:
    try:
        return guard.run_command(index, argv, env)
    except BaseException as error:
        stdout_path, stderr_path = ensure_raw_pair(index)
        free = shutil.disk_usage(ROOT).free
        size = guard.target_bytes()
        return {
            'index': index, 'argv': argv, 'cwd': str(ROOT), 'started_at': None,
            'duration_seconds': 0, 'exit_code': None,
            'guard_stop': {'phase': 'pre_spawn_or_guard_exception', 'error': repr(error),
                           'target_bytes': size, 'free_bytes': free},
            'peak_target_bytes': size, 'free_bytes_after': free,
            'env_overrides': guard.ENV_VALUES, 'test_summary_check': None,
            'streams': {
                'stdout': {'path': str(stdout_path), 'bytes': stdout_path.stat().st_size,
                           'sha256': sha_file(stdout_path)},
                'stderr': {'path': str(stderr_path), 'bytes': stderr_path.stat().st_size,
                           'sha256': sha_file(stderr_path)},
            },
        }


def main() -> int:
    if OUT.exists():
        raise SystemExit(f'refusing to overwrite prior output: {OUT}')
    runner_sha256_before = sha_file(Path(__file__))
    candidate_pins_sha = sha_file(CANDIDATE_PINS) if CANDIDATE_PINS.is_file() else None
    if candidate_pins_sha != CANDIDATE_PINS_SHA256:
        raise SystemExit('candidate pins SHA mismatch')
    try:
        candidate_pins = json.loads(CANDIDATE_PINS.read_text(encoding='utf-8'))
        candidate_pins_before = candidate_tool_pins()
        tool_pins = verify_runner_tool_pins()
    except BaseException as error:
        raise SystemExit(f'candidate/final-build/tool pin preflight failed: {error}') from error

    build = json.loads(FINAL_BUILD.read_text(encoding='utf-8'))
    source_pins_before = build.get('source_pins_before')
    if not isinstance(source_pins_before, dict) or len(source_pins_before) != EXPECTED_SOURCE_FILES:
        raise SystemExit('final-build receipt source closure is not exactly 4,727 files')
    if build.get('source_pins_after') != source_pins_before:
        raise SystemExit('final-build receipt before/after source pins differ')
    source_paths = set(source_pins_before)
    for relative, spec in SOURCE_CANDIDATES.items():
        if relative not in source_paths:
            raise SystemExit(f'candidate source is absent from frozen closure: {relative}')
    if build.get('target_absent') is not True:
        raise SystemExit('final-build receipt does not establish an absent target directory')

    if not CLASS.is_file() or CLASS.is_symlink() or sha_file(CLASS) != CLASS_SHA256:
        raise SystemExit('external frozen class SHA mismatch')
    if (ROOT / 'target').exists():
        raise SystemExit('fresh-target preflight failed')
    free_before = shutil.disk_usage(ROOT).free
    if free_before < 5 * 1024**3:
        raise SystemExit(f'5 GiB free-space preflight failed: {free_before}')

    # Enumerate only while all workspace sources have their ordinary in-tree include graph.
    # The patched facade test intentionally references the separately pinned external class.
    enumerated = enumerate_baseline_source_paths()
    enumerated_rel = {str(path.relative_to(ROOT)) for path in enumerated}
    if enumerated_rel != source_paths:
        raise SystemExit('independent source-path enumeration differs from final-build receipt')

    guard = import_module('nested_foreach_guard_v1', GUARD)
    scan = runpy.run_path(str(SCAN), run_name='nested_foreach_target_scan')
    if guard.FREE_LIMIT != 5 * 1024**3 or guard.TARGET_LIMIT != 1024**3:
        raise SystemExit('guard thresholds differ from 5 GiB / 1 GiB requirements')
    guard.ROOT = ROOT
    guard.OUT = OUT
    guard.expected_test_summaries = lambda _index, _raw: None
    guard.command_stream = lambda path: {
        'path': str(path), 'bytes': Path(path).stat().st_size, 'sha256': sha_file(Path(path))
    }
    guard.target_bytes = lambda: scan['target_bytes'](ROOT)
    env = os.environ.copy()
    stripped = []
    for key in (*guard.STRIPPED_ENV_KEYS, 'RUST_MIN_STACK'):
        if key in env:
            env.pop(key)
            stripped.append(key)
    env.update(guard.ENV_VALUES)
    env['PROPTEST_RNG_SEED'] = SEED
    env[ENV_OPT_IN] = '1'
    guard.ENV_VALUES = {**guard.ENV_VALUES, 'PROPTEST_RNG_SEED': SEED, ENV_OPT_IN: '1'}
    if 'RUST_MIN_STACK' in env:
        raise RuntimeError('RUST_MIN_STACK must remain unset for the default Rust stack')
    if env.get('RUST_TEST_THREADS') != '1' or env.get('PROPTEST_RNG_SEED') != SEED:
        raise RuntimeError('test thread/seed environment mismatch')

    current_before = guard.source_pins(source_paths)
    if current_before != source_pins_before:
        raise SystemExit('live source pins differ from constructor-conditional-final-build-v1 baseline')
    originals = {relative: (ROOT / relative).read_bytes() for relative in SOURCE_CANDIDATES}
    for relative, raw in originals.items():
        if sha_bytes(raw) != SOURCE_CANDIDATES[relative]['base_sha256']:
            raise SystemExit(f'live candidate source differs from its exact base: {relative}')
    candidate_bytes = {relative: SOURCE_CANDIDATES[relative]['candidate'].read_bytes()
                       for relative in SOURCE_CANDIDATES}
    OUT.mkdir()
    record = {
        'schema': 'hoisted-iterator-same-ir-diagnostic-root-v3',
        'status': 'running',
        'scope': 'one guarded ignored exact test; temporary build.rs and facade.rs candidate only',
        'started_at': datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'runner_path': str(Path(__file__).resolve()),
        'runner_sha256_before': runner_sha256_before,
        'candidate_prep': str(PREP),
        'candidate_pins': {'path': str(CANDIDATE_PINS), 'sha256': candidate_pins_sha},
        'candidate_patch': {'path': str(CANDIDATE_PATCH), 'sha256': CANDIDATE_PATCH_SHA256},
        'candidate_tool_pins_before': candidate_pins_before,
        'guard_tool_pins_before': tool_pins,
        'final_build_receipt': {'path': str(FINAL_BUILD), 'sha256': FINAL_BUILD_SHA256,
                                'schema': build['schema'], 'status': build['status']},
        'source_closure_count': len(source_pins_before),
        'source_enumeration_matches_final_build': True,
        'source_pins_before': current_before,
        'source_pins_after_restore': None,
        'candidate_sources': {relative: {
            'base_sha256': SOURCE_CANDIDATES[relative]['base_sha256'],
            'candidate_sha256': SOURCE_CANDIDATES[relative]['candidate_sha256'],
            'bytes': len(candidate_bytes[relative]),
        } for relative in SOURCE_CANDIDATES},
        'external_class': {'path': str(CLASS), 'bytes': CLASS.stat().st_size,
                           'sha256': sha_file(CLASS), 'separately_pinned_outside_source_closure': True},
        'test_path': TEST_NAME,
        'environment_overrides': guard.ENV_VALUES,
        'stripped_environment_keys': stripped,
        'rust_min_stack': env.get('RUST_MIN_STACK'),
        'fresh_target_preflight': {'target_absent': True, 'free_bytes': free_before},
        'guard_limits': {'minimum_free_bytes': guard.FREE_LIMIT,
                         'maximum_target_bytes': guard.TARGET_LIMIT,
                         'poll_interval_seconds': 1,
                         'process_group_termination': True},
        'normal_closure_scanner_not_run_with_external_include': True,
        'commands': [],
        'restore': {'attempted': False, 'verified_files': {}, 'verified': False},
        'clean_command': None,
        'target_absent_after': None,
    }
    save(record)
    applied: dict[str, bool] = {relative: False for relative in SOURCE_CANDIDATES}
    command_succeeded = False
    try:
        for relative, raw in candidate_bytes.items():
            path = ROOT / relative
            path.write_bytes(raw)
            applied[relative] = True
            if sha_file(path) != SOURCE_CANDIDATES[relative]['candidate_sha256']:
                raise RuntimeError(f'candidate read-back mismatch: {relative}')
        temporary_pins = guard.source_pins(source_paths)
        expected_temporary = dict(source_pins_before)
        for relative, spec in SOURCE_CANDIDATES.items():
            expected_temporary[relative] = spec['candidate_sha256']
        if temporary_pins != expected_temporary:
            raise RuntimeError('temporary pins differ outside the two candidate source files')
        record['temporary_source_pins_only_two_candidates_changed'] = True
        save(record)

        argv = ['cargo', 'test', '-p', 'jarde', '--lib', '--all-features', '--locked',
                TEST_NAME, '--', '--ignored', '--exact', '--nocapture']
        row = run_guarded(guard, 0, argv, env)
        record['commands'].append(row)
        save(record)
        if row['exit_code'] != 0 or row['guard_stop'] is not None:
            raise RuntimeError('diagnostic test failed or hit a resource guard')
        stdout_path = Path(row['streams']['stdout']['path'])
        stdout_text = stdout_path.read_text(errors='replace')
        summaries = re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;', stdout_text)
        if len(summaries) != 1 or summaries[0] != ('1', '0', '0'):
            raise RuntimeError(f'exact one-test summary missing (requires 1/0/0): {summaries!r}')
        record['diagnostic_test_summary'] = {'passed': 1, 'failed': 0, 'ignored': 0}
        command_succeeded = True
        record['status'] = 'completed_same_ir_diagnostic_only'
        save(record)
    except BaseException as error:
        record['status'] = 'failed'
        record['error'] = f'{type(error).__name__}: {error}'
        save(record)
    finally:
        record['restore']['attempted'] = True
        for relative, was_applied in applied.items():
            path = ROOT / relative
            try:
                if was_applied:
                    current_sha = sha_file(path)
                    candidate_sha = SOURCE_CANDIDATES[relative]['candidate_sha256']
                    base_sha = SOURCE_CANDIDATES[relative]['base_sha256']
                    if current_sha == candidate_sha:
                        path.write_bytes(originals[relative])
                    elif current_sha != base_sha:
                        raise RuntimeError(f'live file changed during diagnostic; refusing overwrite: {relative}')
                record['restore']['verified_files'][relative] = sha_file(path) == SOURCE_CANDIDATES[relative]['base_sha256']
                if not record['restore']['verified_files'][relative]:
                    raise RuntimeError(f'baseline restore SHA mismatch: {relative}')
            except BaseException as error:
                record['restore']['verified_files'][relative] = False
                record['restore'].setdefault('errors', {})[relative] = f'{type(error).__name__}: {error}'
                record['status'] = 'failed'
        record['restore']['verified'] = all(record['restore']['verified_files'].values())

        try:
            clean = run_guarded(guard, len(record['commands']), ['cargo', 'clean'], env)
            record['clean_command'] = clean
            record['commands'].append(clean)
        except BaseException as error:
            record['cleanup_error'] = f'{type(error).__name__}: {error}'
            record['status'] = 'failed'
        try:
            restored_enumerated = enumerate_baseline_source_paths()
            restored_relative = {str(path.relative_to(ROOT)) for path in restored_enumerated}
            record['source_enumeration_after_restore_matches'] = restored_relative == source_paths
            record['source_pins_after_restore'] = guard.source_pins(source_paths)
            record['source_pins_match_before'] = record['source_pins_after_restore'] == source_pins_before
            if not record['source_enumeration_after_restore_matches']:
                raise RuntimeError('restored source-path enumeration differs from constructor-conditional-final-build-v1 closure')
        except BaseException as error:
            record['source_pin_error'] = f'{type(error).__name__}: {error}'
            record['source_pins_match_before'] = False
            record['status'] = 'failed'
        record['target_absent_after'] = not (ROOT / 'target').exists()
        if not record['restore']['verified'] or not record['source_pins_match_before'] or not record['target_absent_after']:
            record['status'] = 'failed'
        if (record['clean_command'] is None or record['clean_command']['exit_code'] != 0
                or record['clean_command']['guard_stop'] is not None):
            record['status'] = 'failed'
        try:
            record['candidate_tool_pins_after'] = candidate_tool_pins()
            record['candidate_tool_pins_match_before'] = record['candidate_tool_pins_after'] == candidate_pins_before
            record['guard_tool_pins_after'] = verify_runner_tool_pins()
            record['guard_tool_pins_match_before'] = record['guard_tool_pins_after'] == tool_pins
        except BaseException as error:
            record['tool_pin_error'] = f'{type(error).__name__}: {error}'
            record['candidate_tool_pins_match_before'] = False
            record['guard_tool_pins_match_before'] = False
        if not record['candidate_tool_pins_match_before'] or not record['guard_tool_pins_match_before']:
            record['status'] = 'failed'
        record['runner_sha256_after'] = sha_file(Path(__file__))
        if record['runner_sha256_after'] != record['runner_sha256_before']:
            record['status'] = 'failed'
        record['finished_at'] = datetime.datetime.now(datetime.timezone.utc).isoformat()
        save(record)

    print(json.dumps({
        'status': record['status'], 'error': record.get('error'),
        'diagnostic_test_summary': record.get('diagnostic_test_summary'),
        'restore_verified': record['restore']['verified'],
        'source_pins_match_before': record.get('source_pins_match_before'),
        'target_absent_after': record['target_absent_after'],
        'commands': len(record['commands']),
    }, ensure_ascii=False))
    return 0 if (command_succeeded and record['status'] == 'completed_same_ir_diagnostic_only'
                 and record['restore']['verified'] and record.get('source_pins_match_before')
                 and record.get('candidate_tool_pins_match_before')
                 and record.get('guard_tool_pins_match_before') and record['target_absent_after']) else 1


def import_module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f'cannot load pinned module: {path}')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


if __name__ == '__main__':
    raise SystemExit(main())
