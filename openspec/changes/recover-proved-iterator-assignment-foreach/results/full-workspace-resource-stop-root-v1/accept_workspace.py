#!/usr/bin/env python3
"""Accept one completed iterator-assignment root-v1 local workspace receipt, read-only."""
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
DEFAULT_WORKSPACE = Path('/private/tmp/jarde-iterator-assignment-full-validation-prep-luna-v1/workspace-run')
DEFAULT_OUT = Path('/private/tmp/jarde-iterator-assignment-full-validation-prep-luna-v1/workspace-acceptance.json')
RUNNER = Path('/private/tmp/jarde-iterator-assignment-full-validation-prep-luna-v1/workspace_runner.py')
RUNNER_SHA256 = '7843192827b11ee0ffc6a4cdf2305190fabda2d841c2d5526c667811f4dd9554'
REGRESSION_DEFAULT = Path('/private/tmp/jarde-iterator-assignment-regressions-root-v1/execution.json')
CONDITIONAL_BASE = ROOT / 'openspec/changes/recover-proved-nested-loop-side-exits/results/final-workspace-root-v3/verify-cf11-own-ci.py'
CONDITIONAL_BASE_SHA256 = '7669fb27ff653b5e8e35b68a2b54ec0723b4d5d63ee5c05fc0d7af5ed639c1f7'
CONDITIONAL_HELPER = ROOT / 'openspec/changes/recover-proved-conditional-switch-fallthrough/results/verify-conditional-ci-root-v5.py'
CONDITIONAL_HELPER_SHA256 = 'bea37c7d4d37182168a7c1cbbde3ee31c94d61f6bc32667bedbdd02ce7e5b0ca'
GUARD = ROOT / 'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
GUARD_SHA256 = '51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
SCANNER = ROOT / 'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
SCANNER_SHA256 = 'd954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
SCHEMA = 'iterator-assignment-full-workspace-root-v1'
SEED = '5350648285461741569'
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
    'tests/recover_proved_conditional_constructor_values.rs': (
        'conditional_constructor_public_reports_agree_across_evidence_selections',
        'conditional_constructor_positive_all_evidence_reports_four_exact_methods',
        'conditional_constructor_range_excluding_all_new_bcis_keeps_summary_and_text',
        'conditional_constructor_public_late_stop_and_precancel_publish_atomically',
        'conditional_sites_are_not_presented_when_the_final_body_is_explanation_only',
    ),
    'crates/jarde-java/src/field.rs': (
        'field::tests::conditional_new_receipt_counts_only_one_method_local_direct_new',
        'field::tests::conditional_new_receipt_rejects_derived_primary_and_derived_only_candidate_anchors',
    ),
    'crates/jarde-java/src/build.rs': (
        'build::tests::iterator_declaration_gate_uses_real_local_owner_and_refuses_incomplete_facts',
        'build::tests::iterator_declaration_gate_rejects_real_split_slot_identity_changes',
        'build::tests::iterator_phi_gate_counts_real_duplicate_inputs_and_ignores_unrelated_phi',
        'build::tests::mixed_iterator_real_phi_is_live_and_fails_the_production_use_gate',
        'build::tests::alias_branch_phi_does_not_expand_the_exact_value_reader_allowlist',
    ),
    'tests/p3_hoisted_iterator_foreach.rs': (
        'nested_iterators_project_and_preserve_the_outer_assignment_and_cast',
        'outer_assignment_foreach_public_api_and_real_controls_stay_conservative',
        'nested_foreach_late_stop_and_precancel_leave_no_partial_method_artifact',
    ),
    'crates/jarde-java/src/init.rs': (
        'init::tests::conditional_constructor_output_has_one_live_stack_phi_and_one_join_consumer',
        'init::tests::actual_local_conditional_new_has_its_local_store_as_the_single_consumer',
        'init::tests::actual_shared_field_return_keeps_dup_outside_the_new_expression',
        'init::tests::actual_nested_conditional_constructor_arms_do_not_become_one_two_way_site',
        'init::tests::actual_independent_branch_effect_stays_on_the_existing_statement_path',
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


def check_recorded_batch_observations(row: dict, observed: list[dict], parser) -> None:
    """Cross-check runner convenience fields against CI-helper parsing of the raw streams."""
    expected_outcomes = [outcome for target in observed for outcome in target['outcomes']]
    raw_stdout = parser.verified_stream(row['streams']['stdout'])
    cleaned_lines = [parser.clean(line).strip()
                     for line in raw_stdout.decode('utf-8', errors='replace').splitlines()]
    actual_outcome_lines = [line for line in cleaned_lines if parser.OUTCOME_RE.fullmatch(line)]
    actual_summary_lines = [line for line in cleaned_lines
                            if line.startswith('test result: ok.') or line.startswith('test result: FAILED.')]
    recorded_outcomes = row.get('named_test_outcomes')
    need(isinstance(recorded_outcomes, list) and len(recorded_outcomes) == len(expected_outcomes),
         f"batch {row.get('index')} named outcome record count differs from raw test blocks")
    need(row.get('named_test_outcome_count') == len(expected_outcomes),
         f"batch {row.get('index')} named outcome count field differs from raw test blocks")
    for actual, (expected_name, expected_state) in zip(recorded_outcomes, expected_outcomes):
        need(isinstance(actual, dict) and actual.get('name') == expected_name,
             f"batch {row.get('index')} named test differs from raw test blocks")
        state = actual.get('status')
        need(state in ('ok', 'ignored', 'FAILED'),
             f"batch {row.get('index')} has an unknown named test state")
        expected_simple_state = 'ignored' if expected_state.startswith('ignored') else expected_state
        need(state == expected_simple_state,
             f"batch {row.get('index')} named test status differs from raw test blocks")
        raw_line = actual.get('raw_line')
        need(isinstance(raw_line, str), f"batch {row.get('index')} named test omits its raw line")
        match = parser.OUTCOME_RE.fullmatch(parser.clean(raw_line).strip())
        need(match is not None and match.group(1) == expected_name and
             ('ignored' if match.group(2).startswith('ignored') else match.group(2)) == expected_simple_state,
             f"batch {row.get('index')} named test raw line differs from its parsed identity/state")

    recorded_outcome_lines = [parser.clean(item['raw_line']).strip() for item in recorded_outcomes]
    need(recorded_outcome_lines == actual_outcome_lines,
         f"batch {row.get('index')} stored named outcomes omit/reorder/change raw stdout lines")

    summaries = row.get('observed_summaries')
    need(isinstance(summaries, list) and len(summaries) == len(observed),
         f"batch {row.get('index')} summary rows do not match executed target count")
    for summary, target in zip(summaries, observed):
        counts = target['counts']
        need(summary.get('status') == ('ok' if counts[1] == 0 else 'FAILED') and
             [summary.get('passed'), summary.get('failed'), summary.get('ignored')] == list(counts),
             f"batch {row.get('index')} stored summary counts differ from raw target summary")
        raw_line = summary.get('raw_line')
        need(isinstance(raw_line, str), f"batch {row.get('index')} summary omits its raw line")
        match = parser.SUMMARY_RE.fullmatch(parser.clean(raw_line).strip())
        need(match is not None and tuple(map(int, match.groups()[1:4])) == counts and
             match.group(1) == summary['status'],
             f"batch {row.get('index')} summary raw line differs from its parsed counts/state")
    recorded_summary_lines = [parser.clean(item['raw_line']).strip() for item in summaries]
    need(recorded_summary_lines == actual_summary_lines,
         f"batch {row.get('index')} stored summaries omit/reorder/change raw stdout lines")
    totals = [sum(target['counts'][i] for target in observed) for i in range(3)]
    need(row.get('summary_test_totals') == totals,
         f"batch {row.get('index')} aggregate totals differ from raw target summaries")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--workspace', type=Path, default=DEFAULT_WORKSPACE)
    ap.add_argument('--out', type=Path, default=DEFAULT_OUT)
    ap.add_argument('--regression-receipt', type=Path, default=REGRESSION_DEFAULT)
    ap.add_argument('--regression-receipt-sha256', required=True)
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

    regression_path = args.regression_receipt.resolve(strict=True)
    need(not regression_path.is_symlink() and sha_file(regression_path) == args.regression_receipt_sha256,
         'regression receipt path/hash differs')
    regression = json.loads(regression_path.read_text(encoding='utf-8'))
    need(regression.get('schema') == 'iterator-assignment-regressions-root-v1' and
         regression.get('status') == 'passed_candidate_build_only' and
         regression.get('target_absent') is True and
         regression.get('runner_sha256_before') == regression.get('runner_sha256_after') and
         regression.get('source_pins_before') == regression.get('source_pins_after'),
         'input regression receipt is incomplete or not clean')
    runtime_cli = Path('/private/tmp/jarde-iterator-assignment-candidate-cli-root-v1')
    runtime_cli_sha = '526e2e5a6db8a798a92d491123264bbb4d7105c0c1c3c2c9495c1b316eb89dd5'
    cli = regression.get('candidate_cli', {})
    candidate_cli = Path(cli.get('path', ''))
    need(regression.get('candidate_differs_from_old_product') is True and
         candidate_cli == runtime_cli and candidate_cli.is_file() and not candidate_cli.is_symlink() and
         candidate_cli.stat().st_size == 61314560 and sha_file(candidate_cli) == runtime_cli_sha and
         candidate_cli.stat().st_mode & 0o777 == 0o555 and
         cli == {'path': str(candidate_cli), 'bytes': 61314560, 'sha256': runtime_cli_sha, 'mode': '0o555'},
         'regression candidate CLI is not the exact focused-build CLI')
    build_rows = [row for row in regression.get('commands', [])
                  if row.get('argv') == ['cargo','build','-p','jarde-cli','--bin','jarde-cli','--locked']]
    need(len(build_rows) == 1 and build_rows[0].get('index') == 6 and
         build_rows[0].get('exit_code') == 0 and build_rows[0].get('guard_stop') is None and
         set(build_rows[0].get('streams', {})) == {'stdout','stderr'},
         'regression receipt does not bind a successful build at index 6 with both raw streams')
    for stream, ref in build_rows[0]['streams'].items():
        raw_path = Path(ref.get('path', ''))
        need(not raw_path.is_symlink() and raw_path.is_file(),
             'regression CLI build raw stream is missing or symlink: ' + stream)
        raw = raw_path.read_bytes()
        need(type(ref.get('bytes')) is int and len(raw) == ref['bytes'] and sha(raw) == ref.get('sha256'),
             'regression CLI build raw stream differs: '+stream)
    execution_path = workspace / 'execution.json'
    metadata_path = workspace / 'metadata.json'
    need(not execution_path.is_symlink() and not metadata_path.is_symlink(), 'receipt or metadata is a symlink')
    execution_raw = execution_path.read_bytes()
    metadata_raw = metadata_path.read_bytes()
    execution = json.loads(execution_raw)
    metadata = json.loads(metadata_raw)
    need(execution.get('schema') == SCHEMA and execution.get('status') == 'passed',
         'iterator-assignment workspace receipt is incomplete or not passing')
    need(execution.get('finished_at') and execution.get('no_carried_execution') is True,
         'workspace lacks a completion time or was carried from another run')
    need(execution.get('regression_input_receipt') == {'path': str(regression_path), 'sha256': args.regression_receipt_sha256, 'schema': regression['schema'], 'status': regression['status']}, 'workspace regression input binding differs')
    need(execution.get('candidate_cli') == regression.get('candidate_cli') and regression.get('candidate_differs_from_old_product') is True, 'workspace CLI identity differs from focused-built candidate')
    need(execution.get('seed') == SEED, 'workspace seed differs from the reviewed default Rust seed')
    need(execution.get('rust_min_stack') is None, 'RUST_MIN_STACK is overridden')
    need(execution.get('environment_overrides') == EXPECTED_ENV,
         'workspace default Rust environment differs from the reviewed seed/strip settings')
    stripped = execution.get('stripped_environment_keys')
    need(isinstance(stripped, list) and len(stripped) == len(set(stripped)) and set(stripped) <= ALLOWED_STRIPPED_ENV,
         'stripped inherited environment keys are unexpected')
    source_count = len(regression.get('source_pins_before', {}))
    need(source_count > 0 and execution.get('source_closure_count_at_start') == source_count,
         'workspace dynamic source closure differs from the completed regression receipt')
    need(execution.get('source_pins_before') == execution.get('source_pins_after'),
         'workspace top-level source pin maps differ')
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
         'executed iterator-assignment root-v1 runner does not match its before/after pinned bytes')
    refs = {entry.get('path'): entry.get('sha256') for entry in execution.get('references', [])}
    need(refs.get(str(GUARD)) == GUARD_SHA256 and refs.get(str(SCANNER)) == SCANNER_SHA256,
         'execution does not pin the approved guard and target-size scanner')

    expected_relpaths = set(base.workspace_input_paths())
    expected_relpaths.update(str(path.relative_to(ROOT)) for path in (ROOT / 'examples').glob('*.rs'))
    expected_relpaths.update(str(path.relative_to(ROOT)) for path in (ROOT / 'tests/fixtures/p3-iterator-assignment').glob('*') if path.is_file())
    need(len(expected_relpaths) == source_count, 'committed CI helper closure differs from regression receipt source count')
    before = execution.get('source_pins_before')
    after = execution.get('source_pins_after')
    need(isinstance(before, dict) and before == after and len(before) == source_count,
         'workspace before/after source pin maps are incomplete or differ')
    need(set(before) == expected_relpaths, 'workspace source pins differ from committed closure plus current examples/iterator fixture')
    live = {relative: sha_file(ROOT / relative) for relative in sorted(expected_relpaths)}
    need(live == before == regression['source_pins_before'], 'workspace source pins do not match regression/live source bytes')
    cleanup = execution.get('cleanup', {})
    need(cleanup.get('pins_unchanged') is True and cleanup.get('source_pins_after') == before and regression['source_pins_after'] == before,
         'guarded cleanup did not confirm unchanged source pins')

    batches, inventory = expected_batches(metadata, parser)
    target_count = len(inventory)
    batch_count = len(batches)
    command_count = 2 + batch_count + 1
    need(execution.get('expected_batch_count') == batch_count and
         execution.get('expected_target_count') == target_count and
         execution.get('expected_command_count_including_clean') == command_count and
         execution.get('observed_command_count_including_clean') == command_count,
         'execution expected/observed counts differ from fresh metadata-derived plan')
    metadata_record = execution.get('metadata', {})
    need(metadata_record == {
        'command_index': 1,
        'workspace_member_count': len(set(metadata.get('workspace_members', []))),
        'target_count': target_count,
        'batch_count': batch_count,
    }, 'execution metadata summary differs from fresh Cargo metadata')
    declared = [tuple(row) for row in execution.get('target_inventory', [])]
    need(len(declared) == len(set(declared)) and set(declared) == set(inventory),
         'execution target inventory differs from Cargo metadata')
    static_declared = [tuple(row) for row in execution.get('static_manifest_target_inventory', [])]
    need(len(static_declared) == len(set(static_declared)) and set(static_declared) == set(inventory),
         'execution manifest/file target inventory differs from Cargo metadata')

    commands = execution.get('commands')
    need(isinstance(commands, list) and len(commands) == command_count and
         [row.get('index') for row in commands] == list(range(command_count)),
         'receipt must contain exactly the metadata-derived fmt/metadata/batch/clean command sequence')
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

    rows = commands[2:-1]
    need(len(rows) == batch_count, 'workspace batch commands differ from fresh metadata')
    by_id, by_named, parser_inventory = parser.metadata_targets(metadata)
    need(len(parser_inventory) == target_count and set(map(tuple, parser_inventory)) == set(inventory),
         'committed parser target inventory differs from independently planned metadata inventory')
    observations = []
    per_command = []
    for batch_no, (row, argv) in enumerate(zip(rows, batches), start=0):
        index = batch_no + 2
        check_command(row, index, argv, workspace, raw_paths, verified_streams, parser)
        observed = parser.local_command_observation(row, by_id, by_named)
        parser.check_workspace_argv(row.get('argv', []), observed, by_named)
        check_recorded_batch_observations(row, observed, parser)
        need(len(observed) == len(row.get('observed_counts', [])) == len(row.get('executed_targets', [])),
             f'batch {index} raw targets, count records, and executed-target records differ')
        target_totals = [sum(item['counts'][i] for item in observed) for i in range(3)]
        need(row.get('summary_test_totals') == target_totals,
             f'batch {index} aggregate summary differs from named outcomes parsed from raw stdout')
        need(all(item['counts'][1] == 0 for item in observed), f'batch {index} includes a failed named test')
        observations.extend(observed)
        per_command.append({'index': index, 'argv': argv, 'target_count': len(observed),
                            'test_totals': target_totals,
                            'targets': [{'key': list(item['key']), 'source_path': item['source_path'], 'counts': list(item['counts']),
                                         'outcomes': [list(outcome) for outcome in item['outcomes']]}
                                        for item in observed]})

    observed_keys = [item['key'] for item in observations]
    need(len(observed_keys) == len(set(observed_keys)) and set(observed_keys) == set(inventory),
         'observed Cargo targets do not cover metadata targets exactly once')
    totals = [sum(item['counts'][i] for item in observations) for i in range(3)]
    outcome_count = sum(len(item['outcomes']) for item in observations)
    need(totals[0] > 0 and totals[1] == 0 and sum(totals) == outcome_count,
         'named test outcomes are not a complete passing summary')
    need(execution.get('workspace_summary_count') == len(observations) == target_count and
         execution.get('workspace_totals') == totals,
         'workspace-level target summary count/totals differ from independent raw reconstruction')

    required_checks = []
    required_names = {name for names in REQUIRED_TESTS.values() for name in names}
    need(len(required_names) == 20, 'local acceptor must bind the retained 18 tests and two iterator-assignment tests')
    need(execution.get('required_named_tests') == sorted(required_names) and
         execution.get('required_named_tests_passed') == sorted(required_names),
         'runner required named-test inventory or passing receipt differs from the reviewed 20-name set')
    target_by_key = {item['key']: item for item in observations}
    for source, names in REQUIRED_TESTS.items():
        need(source in before, 'source pins omit required regression file: ' + source)
        target = base.metadata_target_for_source(source, by_named)
        key = base.target_key(target)
        if source.startswith('tests/'):
            need(target['source_path'] == source,
                 'required integration source does not map to its exact Cargo target: ' + source)
        else:
            need(target['package'] == 'jarde-java' and target['kind'] == 'lib' and
                 target['source_path'] == 'crates/jarde-java/src/lib.rs',
                 'required unit-test module does not map to the jarde-java library target: ' + source)
        need(key in target_by_key,
             'required source does not map to an executed metadata target: ' + source)
        source_text = (ROOT / source).read_text(encoding='utf-8')
        source_sha = before[source]
        if source.startswith('crates/jarde-java/src/'):
            module_name = Path(source).stem
            lib_source = 'crates/jarde-java/src/lib.rs'
            need(lib_source in before and re.search(
                r'(?m)^\s*(?:(?:pub(?:\([^)]*\))?)\s+)?mod\s+' + re.escape(module_name) + r'\s*;',
                (ROOT / lib_source).read_text(encoding='utf-8')) is not None,
                'required Java unit-test module is not declared by the pinned crate library: ' + module_name)
        passed = []
        for name in names:
            function_name = name.rsplit('::', 1)[-1]
            pattern = r'(?m)^\s*(?:pub\s+)?fn\s+' + re.escape(function_name) + r'\s*\('
            need(len(re.findall(pattern, source_text)) == 1, 'required test definition missing or duplicated: ' + source + '::' + name)
            matches = [(actual, state) for actual, state in target_by_key[key]['outcomes']
                       if actual == name or actual.endswith('::' + name)]
            need(len(matches) == 1 and matches[0][1] == 'ok',
                 'required named test did not pass exactly once: ' + source + '::' + name)
            passed.append(matches[0][0])
        required_checks.append({'source_path': source, 'source_sha256': source_sha,
                                'target': list(key), 'passed_test_names': passed})

    removals = execution.get('removed_completed_test_executables')
    need(isinstance(removals, list) and len(removals) == target_count,
         'runner must record one completed test executable removal per metadata target')
    removed_paths = []
    for item in removals:
        need(isinstance(item, dict), 'completed executable removal receipt is not an object')
        path = Path(item.get('path', ''))
        need(path.is_absolute() and path.is_relative_to(ROOT / 'target') and
             path.parent in ((ROOT / 'target/debug/deps'), (ROOT / 'target/debug/examples')),
             'completed executable removal path is outside target/debug test output directories')
        need(type(item.get('bytes')) is int and item['bytes'] > 0 and
             isinstance(item.get('sha256'), str) and re.fullmatch(r'[0-9a-f]{64}', item['sha256']) and
             type(item.get('completed_batch')) is int and 2 <= item['completed_batch'] < command_count - 1,
             'completed executable removal receipt has malformed size/hash/batch data')
        removed_paths.append(str(path))
    need(len(set(removed_paths)) == target_count, 'completed executable path was recorded more than once')
    clean_row = commands[-1]
    check_command(clean_row, command_count - 1, clean, workspace, raw_paths, verified_streams, parser)
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
    need(actual_raws == referenced_raws == raw_paths and len(raw_paths) == command_count * 2,
         'workspace raw stream inventory is not the exact two-stream-per-command set')

    manifest = {
        'schema': 'iterator-assignment-full-workspace-local-acceptance-v1',
        'status': 'accepted_local_workspace_only',
        'not_ci_or_runtime_acceptance': True,
        'evidence': {
            'workspace': str(workspace),
            'regression_receipt': {'path': str(regression_path), 'sha256': args.regression_receipt_sha256, 'schema': regression['schema'], 'status': regression['status']},
            'execution_schema': SCHEMA,
            'execution_sha256': sha(execution_raw),
            'metadata_sha256': sha(metadata_raw),
            'runner_path': str(RUNNER),
            'runner_sha256': RUNNER_SHA256,
            'workspace_runner_sha256': RUNNER_SHA256,
            'conditional_base_sha256': CONDITIONAL_BASE_SHA256,
            'conditional_helper_sha256': CONDITIONAL_HELPER_SHA256,
            'guard_sha256': GUARD_SHA256,
            'scanner_sha256': SCANNER_SHA256,
        },
        'contract': {
            'batch_count': len(rows), 'metadata_target_count': len(inventory),
            'removed_completed_test_executable_count': len(removals),
            'command_count': len(commands), 'raw_stream_count': len(raw_paths),
            'source_pin_count': len(before), 'seed': execution['seed'],
            'rust_min_stack': execution['rust_min_stack'],
            'resource_limits': execution['guard_limits'],
            'target_absent_before_and_after': True,
            'source_pins_equal_before_after_and_live': True,
            'named_outcome_parser': 'committed conditional CI helper; exact SHA pinned above',
            'totals_source': 'independently parsed from this run’s raw Cargo stdout; no prior total is used',
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
            'per_target_named_outcomes_present': True,
        },
        'required_test_outcomes': required_checks,
        'all_target_outcomes': [
            {'key': list(item['key']), 'source_path': item['source_path'],
             'counts': list(item['counts']),
             'outcomes': [{'name': name, 'status': state} for name, state in item['outcomes']]}
            for item in observations
        ],
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
        print('iterator-assignment local workspace acceptance failed: ' + str(exc), file=sys.stderr)
        raise
