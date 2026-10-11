#!/usr/bin/env python3
"""Guarded, reversible one-test same-IR diagnostic for SequentialLoops2."""
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
import sys
import time

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
OUT = Path('/private/tmp/jarde-sequential-loops2-same-ir-diagnostic-root-v1')
PREP = Path('/private/tmp/jarde-sequential-loops2-same-ir-diagnostic-prep-luna-v3')
CANDIDATE = PREP / 'init.rs'
CANDIDATE_PINS = PREP / 'pins.json'
CANDIDATE_PINS_SHA256 = '120ba1ea322dfacb6ad358925fd6a6933a117ae5d741668d51bf6fbd2ca0de85'
CANDIDATE_PATCH = PREP / 'init.rs.patch'
CANDIDATE_PATCH_SHA256 = '460939cd5a11404bba1b8775f7de19a2cb3d193aceae61b7f2f0af0168abebb5'
FINAL_BUILD = Path('/private/tmp/jarde-for-condition-final-lint-build-root-v3/execution.json')
FINAL_BUILD_SHA256 = 'b4b7bba399b44686da9e990293df3ec99eab36b288ab984b2a9e63a711d8477d'
FINAL_BUILD_RUNNER = Path('/private/tmp/jarde-for-condition-final-lint-build-root-v3.py')
FINAL_BUILD_RUNNER_SHA256 = '85e25bb6d87a5993305c0fdeb3c58cd9300886701cf8158267df267ce0b92f1e'
GUARD = ROOT / 'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
GUARD_SHA256 = '51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
SCAN = ROOT / 'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
SCAN_SHA256 = 'd954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
CLASS = Path('/private/tmp/jarde-cf11-sequential-loops2-baseline-root-v1/legs/javac23-original/classes/cf11/UpstreamSequentialLoops2.class')
CLASS_SHA256 = '524b1746bdff10a3b9da021679f9832202df1552c2eb42c02d150284b7dd8664'
INIT = ROOT / 'crates/jarde-java/src/init.rs'
INIT_BASE_SHA256 = 'f1c0c96fed82e86729fefbef55f40d85473526fb86b0389b9253bb6f41f4c4a1'
CANDIDATE_SHA256 = '4506b652e0ded4408bd740cd8f1349065e2f6d5f5fcbdcf7f708e11b32c1b1c2'
TEST_NAME = 'init::tests::cf11_sequential_loops2_constructor_phi_same_ir_diagnostic'
EXPECTED_SOURCE_FILES = 4718
SEED = '5350648285461741569'
INCLUDE_RE = re.compile(r'(?:include_bytes|include_str)!\s*\(\s*"([^"]+)"')
DOCELLIPSIS_EXCEPTIONS = {
    ('../tests/fixtures/…', 'crates/jarde-jvm/src/test_fixtures.rs'),
    ('../tests/fixtures/…', 'crates/jarde-reader/src/test_fixtures.rs'),
}


def sha_bytes(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def sha_file(path: Path) -> str:
    return sha_bytes(path.read_bytes())


def verify_candidate_tool_pins(candidate_pins: dict) -> dict[str, str]:
    entries = [candidate_pins.get('candidate'), candidate_pins.get('patch'),
               candidate_pins.get('input_class'), candidate_pins.get('base')]
    for section in ('read_only_dependency_pins', 'frozen_evidence'):
        entries.extend(candidate_pins.get(section, {}).values())
    result = {}
    for entry in entries:
        if not isinstance(entry, dict) or not isinstance(entry.get('path'), str):
            raise RuntimeError('malformed candidate tool pin entry')
        path = Path(entry['path'])
        expected = entry.get('sha256')
        if not isinstance(expected, str) or not path.is_file() or sha_file(path) != expected:
            raise RuntimeError(f'candidate tool/input pin mismatch: {path}')
        result[str(path)] = expected
    fixed = {
        str(CANDIDATE_PINS): CANDIDATE_PINS_SHA256,
        str(CANDIDATE_PATCH): CANDIDATE_PATCH_SHA256,
        str(FINAL_BUILD): FINAL_BUILD_SHA256,
        str(FINAL_BUILD_RUNNER): FINAL_BUILD_RUNNER_SHA256,
        str(GUARD): GUARD_SHA256,
        str(SCAN): SCAN_SHA256,
    }
    for path_text, expected in fixed.items():
        path = Path(path_text)
        if not path.is_file() or sha_file(path) != expected:
            raise RuntimeError(f'execution tool pin mismatch: {path}')
        result[path_text] = expected
    return result


def load_module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f'cannot load pinned module: {path}')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def enumerate_baseline_source_paths() -> set[Path]:
    """Mirror final-build-v3 enumeration before the external-include candidate is applied."""
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
        # Preserve an attempted command even if the shared guard rejects it before spawn.
        stdout_path, stderr_path = ensure_raw_pair(index)
        free = shutil.disk_usage(ROOT).free
        target = guard.target_bytes()
        return {
            'index': index, 'argv': argv, 'cwd': str(ROOT), 'started_at': None,
            'duration_seconds': 0, 'exit_code': None,
            'guard_stop': {'phase': 'pre_spawn_or_guard_exception', 'error': repr(error),
                           'target_bytes': target, 'free_bytes': free},
            'peak_target_bytes': target, 'free_bytes_after': free,
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
    if not FINAL_BUILD.is_file() or sha_file(FINAL_BUILD) != FINAL_BUILD_SHA256:
        raise SystemExit('final-build-v3 receipt SHA mismatch')
    if not FINAL_BUILD_RUNNER.is_file() or sha_file(FINAL_BUILD_RUNNER) != FINAL_BUILD_RUNNER_SHA256:
        raise SystemExit('final-build-v3 runner SHA mismatch')
    if not GUARD.is_file() or sha_file(GUARD) != GUARD_SHA256:
        raise SystemExit('guard-template SHA mismatch')
    if not SCAN.is_file() or sha_file(SCAN) != SCAN_SHA256:
        raise SystemExit('target-size scanner SHA mismatch')
    if not CANDIDATE.is_file() or sha_file(CANDIDATE) != CANDIDATE_SHA256:
        raise SystemExit('candidate init.rs SHA mismatch')
    if not CANDIDATE_PINS.is_file() or sha_file(CANDIDATE_PINS) != CANDIDATE_PINS_SHA256:
        raise SystemExit('candidate pins missing or SHA mismatch')
    if not CANDIDATE_PATCH.is_file() or sha_file(CANDIDATE_PATCH) != CANDIDATE_PATCH_SHA256:
        raise SystemExit('candidate patch SHA mismatch')
    candidate_pins = json.loads(CANDIDATE_PINS.read_text(encoding='utf-8'))
    if candidate_pins.get('candidate', {}).get('sha256') != CANDIDATE_SHA256:
        raise SystemExit('candidate pin does not bind requested init.rs')
    if candidate_pins.get('base', {}).get('sha256') != INIT_BASE_SHA256:
        raise SystemExit('candidate base pin mismatch')
    if not CLASS.is_file() or CLASS.is_symlink() or sha_file(CLASS) != CLASS_SHA256:
        raise SystemExit('external class pin mismatch')
    candidate_tool_pins_before = verify_candidate_tool_pins(candidate_pins)
    if not INIT.is_file() or sha_file(INIT) != INIT_BASE_SHA256:
        raise SystemExit('live init.rs is not the pinned pre-candidate source')
    if (ROOT / 'target').exists():
        raise SystemExit('fresh-target preflight failed')
    free_before = shutil.disk_usage(ROOT).free
    if free_before < 5 * 1024**3:
        raise SystemExit(f'5 GiB free-space preflight failed: {free_before}')

    # The accepted final-build receipt is the authoritative path set; this independent
    # enumeration confirms it while the ordinary, in-repository include graph is intact.
    build = json.loads(FINAL_BUILD.read_text(encoding='utf-8'))
    if build.get('schema') != 'for-condition-final-lint-build-root-v1' or build.get('status') != 'passed_candidate_build_only':
        raise SystemExit('final-build receipt is not an accepted candidate-build receipt')
    source_pins_before = build.get('source_pins_before')
    if not isinstance(source_pins_before, dict) or len(source_pins_before) != EXPECTED_SOURCE_FILES:
        raise SystemExit('final-build receipt source closure is not the expected 4,718 files')
    if build.get('source_pins_after') != source_pins_before:
        raise SystemExit('final-build receipt before/after source pins differ')
    enumerated = enumerate_baseline_source_paths()
    enumerated_rel = {str(path.relative_to(ROOT)) for path in enumerated}
    if enumerated_rel != set(source_pins_before):
        raise SystemExit('independent source-path enumeration differs from final-build receipt')

    guard = load_module('sequential_loops2_guard_v1', GUARD)
    scan = runpy.run_path(str(SCAN), run_name='sequential_loops2_target_scan')
    guard.ROOT = ROOT
    guard.OUT = OUT
    guard.expected_test_summaries = lambda _index, _raw: None
    guard.command_stream = lambda path: {
        'path': str(path), 'bytes': Path(path).stat().st_size, 'sha256': sha_file(Path(path))
    }
    guard.target_bytes = lambda: scan['target_bytes'](ROOT)
    env = os.environ.copy()
    stripped = []
    for key in guard.STRIPPED_ENV_KEYS:
        if key in env:
            env.pop(key)
            stripped.append(key)
    if 'RUST_MIN_STACK' in env:
        env.pop('RUST_MIN_STACK')
        stripped.append('RUST_MIN_STACK')
    env.update(guard.ENV_VALUES)
    env['PROPTEST_RNG_SEED'] = SEED
    guard.ENV_VALUES = {**guard.ENV_VALUES, 'PROPTEST_RNG_SEED': SEED}
    if 'RUST_MIN_STACK' in env:
        raise RuntimeError('RUST_MIN_STACK must remain unset for the default Rust stack')

    source_paths = set(source_pins_before)
    current_before = guard.source_pins(source_paths)
    if current_before != source_pins_before:
        raise SystemExit('live source pins differ from final-build-v3 baseline')
    original_init = INIT.read_bytes()
    candidate_bytes = CANDIDATE.read_bytes()
    OUT.mkdir()
    record = {
        'schema': 'cf11-sequential-loops2-same-ir-root-v1',
        'status': 'running',
        'scope': 'one temporary test; SSA/CFG/outside_readers/sites observation only',
        'started_at': datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'runner_path': str(Path(__file__).resolve()),
        'runner_sha256_before': sha_file(Path(__file__)),
        'final_build_receipt': {'path': str(FINAL_BUILD), 'sha256': FINAL_BUILD_SHA256,
                                'schema': build['schema'], 'status': build['status']},
        'source_closure_count': len(source_pins_before),
        'source_enumeration_matches_final_build': True,
        'source_pins_before': current_before,
        'source_pins_after_restore': None,
        'candidate': {'path': str(CANDIDATE), 'bytes': len(candidate_bytes),
                      'sha256': sha_bytes(candidate_bytes)},
        'candidate_tool_pins_before': candidate_tool_pins_before,
        'candidate_tool_pins_after': None,
        'external_class': {'path': str(CLASS), 'bytes': CLASS.stat().st_size,
                           'sha256': sha_file(CLASS), 'separately_pinned_outside_source_closure': True},
        'environment_overrides': guard.ENV_VALUES,
        'stripped_environment_keys': stripped,
        'rust_min_stack': env.get('RUST_MIN_STACK'),
        'fresh_target_preflight': {'target_absent': True, 'free_bytes': free_before},
        'guard_limits': {'minimum_free_bytes': guard.FREE_LIMIT,
                         'maximum_target_bytes': guard.TARGET_LIMIT,
                         'poll_interval_seconds': 1},
        'normal_closure_scanner_not_run_with_candidate': True,
        'commands': [],
        'restore': {'attempted': False, 'verified': False},
        'clean_command': None,
        'target_absent_after': None,
    }
    save(record)
    applied = False
    try:
        # No closure enumeration occurs with the absolute include in place. We hash only
        # the fixed 4,718 receipt paths and require init.rs to be the sole changed pin.
        INIT.write_bytes(candidate_bytes)
        applied = True
        if sha_file(INIT) != CANDIDATE_SHA256:
            raise RuntimeError('candidate bytes failed read-back verification')
        temporary_pins = guard.source_pins(source_paths)
        expected_temporary = dict(source_pins_before)
        expected_temporary['crates/jarde-java/src/init.rs'] = CANDIDATE_SHA256
        if temporary_pins != expected_temporary:
            raise RuntimeError('temporary source pins differ outside the init.rs candidate')
        record['temporary_source_pins_only_init_changed'] = True
        save(record)

        argv = ['cargo', 'test', '-p', 'jarde-java', '--lib', '--all-features', '--locked',
                TEST_NAME, '--', '--exact', '--nocapture']
        row = run_guarded(guard, 0, argv, env)
        record['commands'].append(row)
        save(record)
        if row['exit_code'] != 0 or row['guard_stop'] is not None:
            raise RuntimeError('diagnostic test command failed or hit resource guard')
        stdout_path = Path(row['streams']['stdout']['path'])
        stdout_text = stdout_path.read_text(errors='replace')
        summaries = re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;', stdout_text)
        if len(summaries) != 1 or summaries[0] != ('1', '0', '0'):
            raise RuntimeError(f'exact one-test summary missing: {summaries!r}')
        record['diagnostic_test_summary'] = {'passed': 1, 'failed': 0, 'ignored': 0}
        record['status'] = 'completed_same_ir_diagnostic_only'
        save(record)
    except BaseException as error:
        record['status'] = 'failed'
        record['error'] = f'{type(error).__name__}: {error}'
        save(record)
    finally:
        if applied:
            record['restore']['attempted'] = True
            try:
                if sha_file(INIT) != CANDIDATE_SHA256:
                    raise RuntimeError('live init.rs changed during diagnostic; refusing to overwrite it')
                INIT.write_bytes(original_init)
                record['restore']['verified'] = sha_file(INIT) == INIT_BASE_SHA256
                if not record['restore']['verified']:
                    raise RuntimeError('restored init.rs does not match pinned baseline')
            except BaseException as error:
                record['restore']['error'] = f'{type(error).__name__}: {error}'
                record['status'] = 'failed'
                record.setdefault('error', record['restore']['error'])
        try:
            clean = run_guarded(guard, len(record['commands']), ['cargo', 'clean'], env)
            record['clean_command'] = clean
            record['commands'].append(clean)
        except BaseException as error:
            record['cleanup_error'] = f'{type(error).__name__}: {error}'
            record['status'] = 'failed'
        try:
            record['source_pins_after_restore'] = guard.source_pins(source_paths)
            record['source_pins_match_before'] = record['source_pins_after_restore'] == source_pins_before
        except BaseException as error:
            record['source_pin_error'] = f'{type(error).__name__}: {error}'
            record['source_pins_match_before'] = False
            record['status'] = 'failed'
        record['target_absent_after'] = not (ROOT / 'target').exists()
        if not record['source_pins_match_before'] or not record['target_absent_after']:
            record['status'] = 'failed'
        if record['clean_command'] is None or record['clean_command']['exit_code'] != 0 or record['clean_command']['guard_stop'] is not None:
            record['status'] = 'failed'
        try:
            record['candidate_tool_pins_after'] = verify_candidate_tool_pins(candidate_pins)
            record['candidate_tool_pins_match_before'] = (
                record['candidate_tool_pins_after'] == record['candidate_tool_pins_before'])
        except BaseException as error:
            record['candidate_tool_pins_error'] = f'{type(error).__name__}: {error}'
            record['candidate_tool_pins_match_before'] = False
        if not record['candidate_tool_pins_match_before']:
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
    return 0 if record['status'] == 'completed_same_ir_diagnostic_only' and record['restore']['verified'] and record['source_pins_match_before'] and record['candidate_tool_pins_match_before'] and record['target_absent_after'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
