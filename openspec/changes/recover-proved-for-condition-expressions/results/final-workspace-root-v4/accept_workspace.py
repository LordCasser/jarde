#!/usr/bin/env python3
"""Accept one completed for-condition root-v4 local workspace receipt, read-only."""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import sys

sys.dont_write_bytecode = True

ROOT = Path('/Users/lordcasser/workspace/projects/jarde').resolve()
DEFAULT_WORKSPACE = Path('/private/tmp/jarde-for-condition-workspace-root-v4')
DEFAULT_OUT = Path('/private/tmp/jarde-for-condition-workspace-root-v4/local-workspace-acceptance-root-v2.json')
RUNNER = Path('/private/tmp/jarde-for-condition-workspace-root-v4.py')
RUNNER_SHA256 = '70f4b0f974ea934f0bc761b4b56722aef8ceee010fa7e6027ed9c30b88b8fabd'
CONDITIONAL_BASE = ROOT / 'openspec/changes/recover-proved-nested-loop-side-exits/results/final-workspace-root-v3/verify-cf11-own-ci.py'
CONDITIONAL_BASE_SHA256 = '7669fb27ff653b5e8e35b68a2b54ec0723b4d5d63ee5c05fc0d7af5ed639c1f7'
CONDITIONAL_HELPER = ROOT / 'openspec/changes/recover-proved-conditional-switch-fallthrough/results/verify-conditional-ci-root-v5.py'
CONDITIONAL_HELPER_SHA256 = 'bea37c7d4d37182168a7c1cbbde3ee31c94d61f6bc32667bedbdd02ce7e5b0ca'
GUARD = ROOT / 'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
GUARD_SHA256 = '51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
SCANNER = ROOT / 'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
SCANNER_SHA256 = 'd954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
SCHEMA = 'for-condition-full-workspace-batched-root-v1'
SEED = '5350648285461741569'
SOURCE_COUNT = 4718
BATCH_COUNT = 95
TARGET_COUNT = 360
COMMAND_COUNT = 98
FREE_LIMIT = 5 * 1024**3
TARGET_LIMIT = 1024**3
BASE_ENV = {
    'CARGO_BUILD_JOBS': '1',
    'CARGO_INCREMENTAL': '0',
    'CARGO_PROFILE_DEV_DEBUG': '0',
    'CARGO_PROFILE_TEST_DEBUG': '0',
    'RUST_TEST_THREADS': '1',
    'CARGO_TERM_COLOR': 'always',
}
EXPECTED_ENV = {**BASE_ENV, 'PROPTEST_RNG_SEED': SEED, 'CARGO_PROFILE_TEST_STRIP': 'symbols'}
ALLOWED_STRIPPED_ENV = {'JAVA_TOOL_OPTIONS', '_JAVA_OPTIONS', 'JDK_JAVA_OPTIONS', 'CLASSPATH'}
REQUIRED_TESTS = {
    'tests/p3_for_latch.rs': (
        'condition_effect_producers_remain_ordered_and_the_for_update_is_unique',
        'field_array_and_negate_reads_are_for_conditions_but_written_local_bound_stays_while',
        'local_assignment_of_a_shared_condition_call_remains_a_structured_while',
        'long_float_and_double_array_comparisons_keep_their_counted_for_headers',
    ),
    'tests/p3_crossing_array_read_values.rs': (
        'f2_recovers_and_the_unprotected_control_only_projects_its_two_for_headers',
    ),
    'tests/recover_unicode_identifiers.rs': (
        'the_ascii_presentations_only_change_in_the_two_proved_f1_for_headers',
    ),
    'tests/p3_nested_loop_side_exits.rs': (
        'upstream_nested_loops2_keeps_outer_for_around_inner_while',
        'a_for_condition_public_late_stop_reports_dimension_usage_and_location',
    ),
}


def need(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def sha(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def sha_file(path: Path) -> str:
    return sha(path.read_bytes())


def load_module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    need(spec is not None and spec.loader is not None, 'cannot load pinned parser: ' + str(path))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def verify_stream(ref: dict, workspace: Path) -> bytes:
    need(isinstance(ref, dict), 'command stream reference is not an object')
    raw_path = Path(ref.get('path', ''))
    path = raw_path if raw_path.is_absolute() else workspace / raw_path
    need(not path.is_symlink(), 'raw stream is a symlink: ' + str(path))
    resolved = path.resolve(strict=True)
    need(resolved.is_relative_to(workspace.resolve(strict=True)),
         'raw stream escapes the workspace evidence directory: ' + str(path))
    data = resolved.read_bytes()
    need(type(ref.get('bytes')) is int and len(data) == ref['bytes'],
         'raw stream byte count mismatch: ' + str(path))
    need(ref.get('sha256') == sha(data), 'raw stream SHA-256 mismatch: ' + str(path))
    return data


def expected_batches(metadata: dict, parser) -> tuple[list[list[str]], list[tuple[str, str, str]]]:
    members = set(metadata.get('workspace_members', []))
    need(len(members) == len(metadata.get('workspace_members', [])), 'duplicate workspace member IDs')
    commands: list[list[str]] = []
    coverage: list[tuple[str, str, str]] = []
    for package in metadata.get('packages', []):
        if package.get('id') not in members:
            continue
        package_name = package['name']
        targets = package.get('targets', [])
        for target in targets:
            need(isinstance(target.get('kind'), list) and len(target['kind']) > 0,
                 'metadata target lacks a kind: ' + repr((package_name, target.get('name'))))
        base = ['cargo', 'test', '-p', package_name, '--all-features', '--locked', '--message-format', 'json']
        unit_targets = [target for target in targets if target['kind'][0] in ('lib', 'bin', 'example', 'bench')]
        if unit_targets:
            flags = []
            for kind, flag in (('lib', '--lib'), ('bin', '--bins'), ('example', '--examples'), ('bench', '--benches')):
                if any(target['kind'][0] == kind for target in unit_targets):
                    flags.append(flag)
            commands.append(base + flags)
            coverage.extend((package_name, target['name'], target['kind'][0]) for target in unit_targets)
        test_targets = [target for target in targets if target['kind'][0] == 'test']
        for start in range(0, len(test_targets), 4):
            group = test_targets[start:start + 4]
            commands.append(base + sum((['--test', target['name']] for target in group), []))
            coverage.extend((package_name, target['name'], 'test') for target in group)
    _, _, inventory = parser.metadata_targets(metadata)
    expected = [tuple(row) for row in inventory]
    need(len(coverage) == len(set(coverage)) and set(coverage) == set(expected),
         'metadata batch plan does not cover its workspace targets exactly once')
    need(len(expected) == TARGET_COUNT, f'Cargo metadata target count is {len(expected)}, expected {TARGET_COUNT}')
    need(len(commands) == BATCH_COUNT, f'Cargo metadata batch count is {len(commands)}, expected {BATCH_COUNT}')
    return commands, expected


def check_command(row: dict, index: int, argv: list[str], workspace: Path, raw_paths: set[Path], verified_streams: dict[str, dict], parser) -> None:
    need(isinstance(row, dict), f'command {index} record is not an object')
    need(row.get('index') == index and row.get('argv') == argv and row.get('cwd') == str(ROOT),
         f'command {index} index/argv/cwd differs from the expected invocation')
    need(type(row.get('exit_code')) is int and row['exit_code'] == 0 and row.get('guard_stop') is None,
         f'command {index} did not exit successfully under the guard')
    need(type(row.get('peak_target_bytes')) is int and row['peak_target_bytes'] <= TARGET_LIMIT,
         f'command {index} exceeded or omitted the 1-GiB target bound')
    need(type(row.get('free_bytes_after')) is int and row['free_bytes_after'] >= FREE_LIMIT,
         f'command {index} exceeded or omitted the 5-GiB free-space bound')
    need(row.get('env_overrides') == EXPECTED_ENV,
         f'command {index} environment overrides differ from the pinned Rust/seed contract')
    streams = row.get('streams', {})
    need(set(streams) == {'stdout', 'stderr'}, f'command {index} lacks both raw streams')
    for ref in streams.values():
        data = verify_stream(ref, workspace)
        path = Path(ref['path'])
        resolved = path.resolve(strict=True) if path.is_absolute() else (workspace / path).resolve(strict=True)
        raw_paths.add(resolved)
        verified_streams[str(resolved)] = {'bytes': len(data), 'sha256': sha(data)}
        # The pinned helper performs its own read while parsing; compare that read against this safe read.
        need(parser.verified_stream(ref) == data, f'conditional parser read differs for command {index}')


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--workspace', type=Path, default=DEFAULT_WORKSPACE)
    ap.add_argument('--out', type=Path, default=DEFAULT_OUT)
    args = ap.parse_args()
    workspace = args.workspace.resolve(strict=True)
    out = args.out.resolve()
    need(workspace.is_dir(), 'workspace path is not a directory')
    need(not out.exists(), 'refusing to overwrite acceptance manifest: ' + str(out))
    for path, expected in ((CONDITIONAL_BASE, CONDITIONAL_BASE_SHA256),
                           (CONDITIONAL_HELPER, CONDITIONAL_HELPER_SHA256),
                           (RUNNER, RUNNER_SHA256), (GUARD, GUARD_SHA256),
                           (SCANNER, SCANNER_SHA256)):
        need(path.is_file() and sha_file(path) == expected,
             'pinned verifier/runner/guard/scanner SHA mismatch: ' + str(path))

    # Import the committed base parser, which imports the separately pinned conditional parser.
    base = load_module('for_condition_committed_workspace_base_v3', CONDITIONAL_BASE)
    parser = base.ci
    need(parser.__file__ == str(CONDITIONAL_HELPER) and sha_file(Path(parser.__file__)) == CONDITIONAL_HELPER_SHA256,
         'base parser did not load the exact pinned conditional outcome parser')

    execution_path = workspace / 'execution.json'
    metadata_path = workspace / 'metadata.json'
    need(not execution_path.is_symlink() and not metadata_path.is_symlink(), 'receipt or metadata is a symlink')
    execution_raw = execution_path.read_bytes()
    metadata_raw = metadata_path.read_bytes()
    execution = json.loads(execution_raw)
    metadata = json.loads(metadata_raw)
    need(execution.get('schema') == SCHEMA and execution.get('status') == 'passed',
         'root-v4 workspace receipt is incomplete or not passing')
    need(execution.get('finished_at') and execution.get('no_carried_execution') is True,
         'workspace lacks a completion time or was carried from another run')
    need(execution.get('seed') == SEED, 'workspace seed differs from the reviewed default Rust seed')
    need(execution.get('rust_min_stack') is None, 'RUST_MIN_STACK is overridden')
    need(execution.get('environment_overrides') == EXPECTED_ENV,
         'workspace default Rust environment differs from the reviewed seed/strip settings')
    stripped = execution.get('stripped_environment_keys')
    need(isinstance(stripped, list) and len(stripped) == len(set(stripped)) and set(stripped) <= ALLOWED_STRIPPED_ENV,
         'stripped inherited environment keys are unexpected')
    need(execution.get('source_closure_count_expected') == SOURCE_COUNT,
         'workspace source-closure contract is not 4,718')
    need(execution.get('fresh_target_preflight', {}).get('target_absent') is True and
         execution['fresh_target_preflight'].get('free_bytes', 0) >= FREE_LIMIT,
         'fresh target/free-space preflight did not meet the reviewed bounds')
    need(execution.get('guard_limits') == {
        'minimum_free_bytes': FREE_LIMIT,
        'maximum_target_bytes': TARGET_LIMIT,
        'poll_interval_seconds': 1,
    }, 'workspace guard limits or one-second polling differ')
    runner_record = execution.get('runner', {})
    need(runner_record.get('path') == str(RUNNER) and
         runner_record.get('sha256_before') == RUNNER_SHA256 == runner_record.get('sha256_after') and
         sha_file(RUNNER) == RUNNER_SHA256,
         'executed root-v4 runner does not match its before/after pinned bytes')
    refs = {entry.get('path'): entry.get('sha256') for entry in execution.get('references', [])}
    need(refs.get(str(GUARD)) == GUARD_SHA256 and refs.get(str(SCANNER)) == SCANNER_SHA256,
         'execution does not pin the approved guard and target-size scanner')

    expected_relpaths = set(base.workspace_input_paths())
    need(len(expected_relpaths) == SOURCE_COUNT, 'committed base parser closure is not exactly 4,718 paths')
    before = execution.get('source_pins_before')
    after = execution.get('source_pins_after')
    need(isinstance(before, dict) and before == after and len(before) == SOURCE_COUNT,
         'workspace before/after source pin maps are incomplete or differ')
    need(set(before) == expected_relpaths, 'workspace source pins differ from committed base parser closure')
    live = {relative: sha_file(ROOT / relative) for relative in sorted(expected_relpaths)}
    need(live == before, 'workspace source pins do not match current live source bytes')
    cleanup = execution.get('cleanup', {})
    need(cleanup.get('pins_unchanged') is True and cleanup.get('source_pins_after') == before,
         'guarded cleanup did not confirm unchanged source pins')

    batches, inventory = expected_batches(metadata, parser)
    need(execution.get('expected_batch_count') == BATCH_COUNT and
         execution.get('expected_target_count') == TARGET_COUNT,
         'execution expected-count fields differ from the reviewed workspace contract')
    metadata_record = execution.get('metadata', {})
    need(metadata_record == {
        'command_index': 1,
        'workspace_member_count': len(set(metadata.get('workspace_members', []))),
        'target_count': TARGET_COUNT,
        'batch_count': BATCH_COUNT,
    }, 'execution metadata summary differs from fresh Cargo metadata')
    declared = [tuple(row) for row in execution.get('target_inventory', [])]
    need(len(declared) == len(set(declared)) and set(declared) == set(inventory),
         'execution target inventory differs from Cargo metadata')

    commands = execution.get('commands')
    need(isinstance(commands, list) and len(commands) == COMMAND_COUNT and
         [row.get('index') for row in commands] == list(range(COMMAND_COUNT)),
         'receipt must contain exactly 98 ordered fmt/metadata/batch/clean command observations')
    fmt = ['cargo', 'fmt', '--all', '--', '--check']
    md = ['cargo', 'metadata', '--no-deps', '--format-version', '1', '--locked']
    clean = ['cargo', 'clean']
    raw_paths: set[Path] = set()
    verified_streams: dict[str, dict] = {}
    check_command(commands[0], 0, fmt, workspace, raw_paths, verified_streams, parser)
    check_command(commands[1], 1, md, workspace, raw_paths, verified_streams, parser)
    need(execution.get('format_check') == {'exit_code': 0, 'guard_stop': None},
         'format-check summary does not record a real successful exit')
    metadata_stdout = verify_stream(commands[1]['streams']['stdout'], workspace)
    need(metadata_stdout == metadata_raw,
         'metadata.json is not byte-identical to command 1 raw stdout')
    need(json.loads(metadata_stdout) == metadata, 'metadata raw stdout parses differently from metadata.json')
    need(metadata_record.get('command_index') == commands[1]['index'], 'metadata command index is not command 1')

    rows = commands[2:97]
    need(len(rows) == BATCH_COUNT, 'workspace does not contain exactly 95 batch commands')
    by_id, by_named, parser_inventory = parser.metadata_targets(metadata)
    need(len(parser_inventory) == TARGET_COUNT and set(map(tuple, parser_inventory)) == set(inventory),
         'committed parser target inventory differs from independently planned metadata inventory')
    observations = []
    per_command = []
    for batch_no, (row, argv) in enumerate(zip(rows, batches), start=0):
        index = batch_no + 2
        check_command(row, index, argv, workspace, raw_paths, verified_streams, parser)
        observed = parser.local_command_observation(row, by_id, by_named)
        parser.check_workspace_argv(row.get('argv', []), observed, by_named)
        need(len(observed) == len(row.get('observed_counts', [])) == len(row.get('executed_targets', [])),
             f'batch {index} raw targets, count records, and executed-target records differ')
        target_totals = [sum(item['counts'][i] for item in observed) for i in range(3)]
        need(row.get('summary_test_totals') == target_totals,
             f'batch {index} aggregate summary differs from named outcomes parsed from raw stdout')
        need(all(item['counts'][1] == 0 for item in observed), f'batch {index} includes a failed named test')
        observations.extend(observed)
        per_command.append({'index': index, 'argv': argv, 'target_count': len(observed),
                            'test_totals': target_totals,
                            'targets': [{'key': list(item['key']), 'counts': list(item['counts']),
                                         'outcomes': [list(outcome) for outcome in item['outcomes']]}
                                        for item in observed]})

    observed_keys = [item['key'] for item in observations]
    need(len(observed_keys) == len(set(observed_keys)) and set(observed_keys) == set(inventory),
         'observed Cargo targets do not cover the 360 metadata targets exactly once')
    totals = [sum(item['counts'][i] for item in observations) for i in range(3)]
    outcome_count = sum(len(item['outcomes']) for item in observations)
    need(totals[0] > 0 and totals[1] == 0 and sum(totals) == outcome_count,
         'named test outcomes are not a complete passing summary')
    need(execution.get('workspace_summary_count') == len(observations) == TARGET_COUNT and
         execution.get('workspace_totals') == totals,
         'workspace-level target summary count/totals differ from independent raw reconstruction')

    required_checks = []
    target_by_key = {item['key']: item for item in observations}
    for source, names in REQUIRED_TESTS.items():
        need(source in before, 'source pins omit required regression file: ' + source)
        target = base.metadata_target_for_source(source, by_named)
        key = base.target_key(target)
        need(target['source_path'] == source and key in target_by_key,
             'required source does not map to exactly one executed metadata target: ' + source)
        source_text = (ROOT / source).read_text(encoding='utf-8')
        source_sha = before[source]
        passed = []
        for name in names:
            pattern = r'(?m)^\s*(?:pub\s+)?fn\s+' + re.escape(name) + r'\s*\('
            need(re.search(pattern, source_text) is not None, 'required test definition missing: ' + source + '::' + name)
            matches = [(actual, state) for actual, state in target_by_key[key]['outcomes']
                       if actual == name or actual.endswith('::' + name)]
            need(len(matches) == 1 and matches[0][1] == 'ok',
                 'required named test did not pass exactly once: ' + source + '::' + name)
            passed.append(matches[0][0])
        required_checks.append({'source_path': source, 'source_sha256': source_sha,
                                'target': list(key), 'passed_test_names': passed})

    need(execution.get('removed_completed_test_executables') is not None,
         'runner omitted completed test executable cleanup receipts')
    clean_row = commands[97]
    check_command(clean_row, 97, clean, workspace, raw_paths, verified_streams, parser)
    need(clean_row.get('guard_stop') is None, 'cargo clean was stopped by the guard')
    need(clean_row.get('test_summary_check') is None, 'cargo clean unexpectedly contains a test summary')
    need(clean_row == cleanup.get('command') and cleanup.get('attempted') is True and
         cleanup.get('argv') == clean and cleanup.get('guarded') is True and cleanup.get('status') == 'passed' and
         cleanup.get('target_absent_after') is True and cleanup.get('pins_unchanged') is True,
         'final guarded cargo clean receipt is incomplete or unsuccessful')
    need(not (ROOT / 'target').exists(), 'workspace target directory still exists after cargo clean')
    referenced_raws = {Path(ref['path']).resolve(strict=True)
                       for row in commands for ref in row['streams'].values()}
    actual_raws = {path.resolve(strict=True) for path in workspace.glob('*.raw') if path.is_file()}
    need(actual_raws == referenced_raws == raw_paths and len(raw_paths) == COMMAND_COUNT * 2,
         'workspace raw stream inventory is not the exact 196 pinned command streams')

    manifest = {
        'schema': 'for-condition-local-workspace-acceptance-root-v2',
        'status': 'accepted_local_workspace_only',
        'evidence': {
            'workspace': str(workspace),
            'execution_sha256': sha(execution_raw),
            'metadata_sha256': sha(metadata_raw),
            'runner_path': str(RUNNER),
            'runner_sha256': RUNNER_SHA256,
            'conditional_base_sha256': CONDITIONAL_BASE_SHA256,
            'conditional_helper_sha256': CONDITIONAL_HELPER_SHA256,
            'guard_sha256': GUARD_SHA256,
            'scanner_sha256': SCANNER_SHA256,
        },
        'contract': {
            'batch_count': len(rows), 'metadata_target_count': len(inventory),
            'command_count': len(commands), 'raw_stream_count': len(raw_paths),
            'source_pin_count': len(before), 'seed': execution['seed'],
            'rust_min_stack': execution['rust_min_stack'],
            'resource_limits': execution['guard_limits'],
            'target_absent_before_and_after': True,
            'source_pins_equal_before_after_and_live': True,
            'named_outcome_parser': 'committed conditional CI helper; exact SHA pinned above',
        },
        'actual_command_receipts': [
            {'index': row['index'], 'argv': row['argv'], 'exit_code': row['exit_code'],
             'guard_stop': row['guard_stop'], 'peak_target_bytes': row['peak_target_bytes'],
             'free_bytes_after': row['free_bytes_after'],
             'streams': {name: {'path': ref['path'], 'bytes': ref['bytes'], 'sha256': ref['sha256']}
                         for name, ref in row['streams'].items()}}
            for row in commands
        ],
        'verified_raw_streams': [{'path': path, **verified_streams[path]} for path in sorted(verified_streams)],
        'actual_workspace_totals': {
            'passed': totals[0], 'failed': totals[1], 'ignored': totals[2],
            'named_test_outcome_count': outcome_count,
            'target_summary_count': len(observations),
        },
        'required_test_outcomes': required_checks,
        'commands': per_command,
    }
    out.parent.mkdir(parents=True, exist_ok=True)
    with out.open('x', encoding='utf-8') as stream:
        stream.write(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({'status': manifest['status'], 'batch_count': len(rows),
                      'target_count': len(inventory), 'command_count': len(commands),
                      'raw_stream_count': len(raw_paths), 'source_pin_count': len(before),
                      'actual_workspace_totals': manifest['actual_workspace_totals'],
                      'acceptance_manifest': str(out)}, ensure_ascii=False))
    return 0


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except Exception as exc:
        print('for-condition local workspace acceptance failed: ' + str(exc), file=sys.stderr)
        raise
