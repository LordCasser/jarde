#!/usr/bin/env python3
"""Independent acceptance of the focused local-reference-write API run only."""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import re
import sys
from pathlib import Path

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
RUN_DIR = Path('/private/tmp/jarde-local-reference-write-focused-root-v3')
RECEIPT = RUN_DIR / 'execution.json'
RUNNER = Path('/private/tmp/jarde-local-reference-write-focused-runner-root-v3.py')
CLOSURE_HELPER = Path('/private/tmp/jarde-iterator-assignment-full-validation-prep-luna-v2/workspace_runner.py')
GUARD_HELPER = ROOT / 'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
SIZE_HELPER = ROOT / 'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
TEST = ROOT / 'tests/p3_local_reference_write_compatibility.rs'
BUILDER = ROOT / 'crates/jarde-java/src/build.rs'

RECEIPT_SHA = 'f322798aff08a9e510410b1167da847f3072b53f9f7a575e8ab033ab0c47fd30'
RUNNER_SHA = '41d52db53da0462f3e2dc65c410f62ed20ed0aa57c28e1fe4b9b9da498adb327'
CLOSURE_HELPER_SHA = '63e52b55fe5d20d5f2ece98ac80676ccdb2de8dfd33d7199b688e075fb1e9673'
GUARD_HELPER_SHA = '51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
SIZE_HELPER_SHA = 'd954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
BUILDER_SHA = '56664a850c8b8f47d27c66072cfb3552a492a6901e1d29a528e19f809780ce5d'
MIN_FREE = 5 * 1024**3
MAX_TARGET = 1024**3
ENV = {
    'CARGO_BUILD_JOBS': '1', 'CARGO_INCREMENTAL': '0',
    'CARGO_PROFILE_DEV_DEBUG': '0', 'CARGO_PROFILE_TEST_DEBUG': '0',
    'RUST_TEST_THREADS': '1', 'CARGO_TERM_COLOR': 'always',
}
SEED = '5350648285461741569'
TEST_ARGV = ['cargo', 'test', '--test', 'p3_local_reference_write_compatibility',
             '--locked', '--', '--nocapture']
EXPECTED_TESTS = {
    'no_cast_iterator_assignment_preserves_object_target_and_full_source_map',
    'reference_write_controls_keep_complete_public_results_across_evidence_requests',
    'pre_cancel_does_not_publish_partial_class_or_method_artifacts',
}
CLEAN_EXCEPTION = ('user-approved fixed cargo clean only: no minimum-free or maximum-target '
                   'enforcement; actual one-second resource samples and raw streams retained')


def need(ok: bool, message: str) -> None:
    if not ok:
        raise ValueError(message)


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load_module(name: str, path: Path, expected_sha: str):
    need(path.is_file() and sha(path) == expected_sha, f'pinned helper mismatch: {path}')
    spec = importlib.util.spec_from_file_location(name, path)
    need(spec is not None and spec.loader is not None, f'cannot load helper: {path}')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def expected_fixture_matrix(source: str) -> set[str]:
    block = source.split('const FIXTURES:', 1)[1].split('\n];', 1)[0]
    rows = re.findall(r'fixture!\((debug|none), (javac8|javac23), '
                      r'(CF11(?:OuterAssignmentTarget|CastReadControl|CharSequenceControl|'
                      r'ReferenceArrayControl|ParameterControl|SlotReuseControl|HandlerWriteControl))\)',
                      block)
    need(len(rows) == 28 and len(set(rows)) == 28, 'expected exactly 28 unique fixture declarations')
    expected = {(profile, jdk, cls) for profile in ('debug', 'none')
                for jdk in ('javac8', 'javac23') for cls in (
                    'CF11OuterAssignmentTarget', 'CF11CastReadControl',
                    'CF11CharSequenceControl', 'CF11ReferenceArrayControl',
                    'CF11ParameterControl', 'CF11SlotReuseControl', 'CF11HandlerWriteControl')}
    need(set(rows) == expected, 'fixture declarations do not form the pinned 7x2x2 matrix')
    return {f'tests/fixtures/p3-local-reference-write-compatibility/{profile}/{jdk}/{cls}.class'
            for profile, jdk, cls in rows}


def check_test_shape(source: str) -> dict:
    fixture_paths = expected_fixture_matrix(source)
    controls = source.split('const CONTROLS:', 1)[1].split('\n    ];', 1)[0]
    control_rows = re.findall(
        r'\(\s*"(CF11[^"]+)"\s*,\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,?\s*\)',
        controls, re.S)
    need(len(control_rows) == 9, 'expected nine control method specifications')
    names = set(re.findall(r'#\[test\]\s*fn\s+(\w+)\s*\(', source))
    need(names == EXPECTED_TESTS, f'named tests differ: {sorted(names)}')
    need('filter(|row| row.2 == "CF11OuterAssignmentTarget")' in source,
         'positive loop is not statically restricted to the outer-assignment fixture')
    need('for (profile, jdk, class, bytes) in FIXTURES {' in source,
         'control/pre-cancel loops do not range over the full fixture matrix')
    for evidence_request in ('RecoveryEvidenceRequest::all()', 'no_rule_details()'):
        need(evidence_request in source, f'missing evidence request variant: {evidence_request}')
    for equality in ('default.text, all.text', 'default.text, plain.text',
                     'default_body.source_map, all_body.source_map',
                     'default_body.source_map, plain_body.source_map'):
        need(equality in source, f'missing cross-request invariance assertion: {equality}')
    return {'fixture_rows': len(fixture_paths), 'positive_fixture_cases': 4,
            'control_fixture_cases': len(control_rows) * 4,
            'pre_cancel_fixture_cases': len(fixture_paths),
            'named_test_count': len(names)}


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument('--receipt', required=True, type=Path)
    ap.add_argument('--receipt-sha256', required=True)
    ap.add_argument('--candidate-sha256', required=True)
    ap.add_argument('--out', required=True, type=Path)
    args = ap.parse_args()
    need(args.out.parent.is_dir() and not args.out.exists(), 'out must be a new file in an existing directory')
    need(args.receipt.resolve() == RECEIPT, 'unexpected receipt path')
    need(args.receipt_sha256 == RECEIPT_SHA and sha(args.receipt) == RECEIPT_SHA,
         'receipt digest differs from frozen focused-v3 receipt')
    need(args.candidate_sha256 == BUILDER_SHA and sha(BUILDER) == BUILDER_SHA,
         'candidate builder digest mismatch')
    need(sha(RUNNER) == RUNNER_SHA, 'focused runner digest mismatch')

    guard = load_module('focused_guard', GUARD_HELPER, GUARD_HELPER_SHA)
    size = load_module('focused_size', SIZE_HELPER, SIZE_HELPER_SHA)
    closure = load_module('focused_closure', CLOSURE_HELPER, CLOSURE_HELPER_SHA)
    need(guard.FREE_LIMIT == MIN_FREE and guard.TARGET_LIMIT == MAX_TARGET,
         'guard helper limits differ from 5 GiB / 1 GiB')
    need(closure.ROOT == ROOT, 'source closure root mismatch')

    receipt = json.loads(args.receipt.read_text(encoding='utf-8'))
    need(receipt.get('schema') == 'local-reference-write-focused-root-v3', 'receipt schema mismatch')
    need(receipt.get('status') == 'passed_focused_api_only', 'focused run did not pass')
    need(receipt.get('runner_sha256') == RUNNER_SHA == receipt.get('runner_sha256_after'),
         'runner pin changed during run')
    need(receipt.get('rust_min_stack') is None, 'Rust test stack override was not default')
    need(receipt.get('actual_env_overrides') == {**ENV, 'PROPTEST_RNG_SEED': SEED},
         'recorded process overrides differ from runner constants')

    before = receipt.get('source_pins_before_format')
    after_fmt = receipt.get('source_pins_after_format')
    after = receipt.get('source_pins_after')
    need(isinstance(before, dict) and len(before) == 4784, 'before-format closure is incomplete')
    need(before == after_fmt == after, 'source closure changed during format/test/cleanup')
    need(receipt.get('formatter_changed_paths') == [], 'formatter changed source files')
    live = closure.pin_sources(closure.source_paths())
    need(live == after, 'current live source closure differs from frozen receipt')
    need(live.get('crates/jarde-java/src/build.rs') == BUILDER_SHA,
         'live build.rs is not the accepted candidate')

    source = TEST.read_text(encoding='utf-8')
    need(sha(TEST) == live.get('tests/p3_local_reference_write_compatibility.rs'),
         'focused test source is not pinned in current closure')
    shape = check_test_shape(source)
    fixture_paths = expected_fixture_matrix(source)
    need(fixture_paths <= live.keys(), 'fixture matrix is not fully included in live source closure')
    fixture_pins = {p: live[p] for p in sorted(fixture_paths)}
    need(len(fixture_pins) == 28, 'fixture pin matrix is incomplete')

    commands = receipt.get('commands')
    need(isinstance(commands, list) and len(commands) == 3, 'expected fmt/test/final-clean commands')
    expected_argv = [['cargo', 'fmt', '--all'], TEST_ARGV, ['cargo', 'clean']]
    peak = 0
    raw_hashes = {}
    for i, (row, argv) in enumerate(zip(commands, expected_argv, strict=True)):
        need(row.get('index') == i and row.get('argv') == argv, f'command {i} argv/index mismatch')
        need(row.get('cwd') == str(ROOT) and row.get('exit_code') == 0 and row.get('guard_stop') is None,
             f'command {i} cwd/exit/guard mismatch')
        need(row.get('env_overrides') == ENV, f'command {i} environment mismatch')
        need(isinstance(row.get('duration_seconds'), (int, float)) and row['duration_seconds'] > 0,
             f'command {i} duration missing')
        streams = row.get('streams')
        need(set(streams or {}) == {'stdout', 'stderr'}, f'command {i} streams malformed')
        for name, data in streams.items():
            raw_path = Path(data['path'])
            need(raw_path.parent == RUN_DIR and raw_path.name == f'{i}.{name}.raw' and raw_path.is_file(),
                 f'command {i} {name} raw path mismatch')
            raw = raw_path.read_bytes()
            digest = hashlib.sha256(raw).hexdigest()
            need(data.get('bytes') == len(raw) and data.get('sha256') == digest,
                 f'command {i} {name} raw digest/length mismatch')
            raw_hashes[f'{i}.{name}'] = {'bytes': len(raw), 'sha256': digest}
        observed_peak = row.get('peak_target_bytes')
        need(isinstance(observed_peak, int) and observed_peak >= 0, f'command {i} peak missing')
        peak = max(peak, observed_peak)
        free = row.get('free_bytes_after')
        need(isinstance(free, int) and free >= 0, f'command {i} free-space sample missing')
        if i < 2:
            need(observed_peak <= MAX_TARGET and free >= MIN_FREE,
                 f'command {i} exceeded guarded target/free limits')
            need('resource_guard_exception' not in row, f'command {i} has unexpected cleanup exception')
        else:
            need(row.get('resource_guard_exception') == CLEAN_EXCEPTION,
                 'final cleanup is not the fixed cargo-clean-only exception')
            need(row['argv'] == ['cargo', 'clean'], 'cleanup exception applies to non-clean command')
        need(row.get('test_summary_check') is None, f'command {i} unexpected summary checker field')

    test_row = commands[1]
    stdout = (RUN_DIR / '1.stdout.raw').read_text(encoding='utf-8')
    parsed_summaries = re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;', stdout)
    parsed_named = re.findall(r'^test (.+) \.\.\. (ok|FAILED|ignored)$', stdout, re.M)
    need(parsed_summaries == [('ok', '3', '0', '0')], 'raw stdout does not show exactly 3/0/0')
    need(len(parsed_named) == 3 and {name for name, _ in parsed_named} == EXPECTED_TESTS
         and all(result == 'ok' for _, result in parsed_named),
         'raw stdout named-test outcomes mismatch')
    need(test_row.get('test_summaries') == [['ok', '3', '0', '0']], 'receipt test summaries mismatch raw stdout')
    need(set(tuple(x) for x in test_row.get('named_outcomes', [])) == set(parsed_named),
         'receipt named outcomes mismatch raw stdout')
    need(receipt.get('target_absent') is True, 'target directory was not recorded absent after cleanup')
    need(not (ROOT / 'target').exists(), 'live target directory exists after run')
    need(receipt.get('free_bytes_after', 0) >= MIN_FREE, 'final free-space sample below 5 GiB')
    need(receipt.get('free_bytes_after') == commands[-1]['free_bytes_after'],
         'final free-space summary differs from cleanup row')

    result = {
        'schema': 'local-reference-write-focused-acceptance-v1',
        'status': 'accepted_focused_api_only',
        'receipt_sha256': RECEIPT_SHA,
        'runner_sha256': RUNNER_SHA,
        'candidate_build_rs_sha256': BUILDER_SHA,
        'source_closure_count': len(live),
        'source_closure_unchanged': True,
        'fixture_class_matrix_count': len(fixture_pins),
        'fixture_class_pins': fixture_pins,
        'static_test_shape': shape,
        'named_test_outcomes': parsed_named,
        'test_summary': parsed_summaries,
        'commands': [{'index': r['index'], 'argv': r['argv'], 'exit_code': r['exit_code'],
                      'peak_target_bytes': r['peak_target_bytes'],
                      'free_bytes_after': r['free_bytes_after']} for r in commands],
        'raw_stream_pins': raw_hashes,
        'target_absent': True,
        'scope': 'Focused API evidence only. Does not establish late-stop behavior, full-workspace '
                 'acceptance, compiler/runtime behavior, or all task-2.2 requirements.',
    }
    args.out.write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'status': result['status'], 'named_tests': len(parsed_named),
                      'fixtures': len(fixture_pins), 'source_pins': len(live),
                      'peak_target_bytes': peak, 'out': str(args.out)}))
    return 0


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except (OSError, ValueError, KeyError, TypeError) as exc:
        print(f'acceptance failed: {exc}', file=sys.stderr)
        raise SystemExit(1)
