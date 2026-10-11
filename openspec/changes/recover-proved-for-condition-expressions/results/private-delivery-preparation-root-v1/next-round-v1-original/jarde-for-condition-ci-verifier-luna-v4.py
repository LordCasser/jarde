#!/usr/bin/env python3
"""Verify for-condition product CI against its complete local workspace and CLI receipts."""
from __future__ import annotations

import argparse
import datetime
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import stat
import sys

sys.dont_write_bytecode = True
ROOT = Path('/Users/lordcasser/workspace/projects/jarde').resolve()
BASE_PATH = ROOT / 'openspec/changes/recover-proved-nested-loop-side-exits/results/final-workspace-root-v3/verify-cf11-own-ci.py'
BASE_SHA256 = '7669fb27ff653b5e8e35b68a2b54ec0723b4d5d63ee5c05fc0d7af5ed639c1f7'
HELPER_PATH = ROOT / 'openspec/changes/recover-proved-conditional-switch-fallthrough/results/verify-conditional-ci-root-v5.py'
HELPER_SHA256 = 'bea37c7d4d37182168a7c1cbbde3ee31c94d61f6bc32667bedbdd02ce7e5b0ca'
CAPTURE_PATH = ROOT / 'openspec/changes/recover-proved-conditional-switch-fallthrough/results/capture-conditional-ci-root-v3.py'
CAPTURE_SHA256 = 'ed889bf6af08247efcce3c889cd1c790d6829d0ea12e9c0d053bdef801994be3'
WORKSPACE_RUNNER = Path('/private/tmp/jarde-for-condition-workspace-root-v4.py')
WORKSPACE_RUNNER_SHA256 = '70f4b0f974ea934f0bc761b4b56722aef8ceee010fa7e6027ed9c30b88b8fabd'
CONDITIONAL_WORKSPACE_RUNNER = Path('/private/tmp/jarde-run-conditional-workspace-batches-root-v5.py')
CONDITIONAL_WORKSPACE_RUNNER_SHA256 = '15fb42dd044f2371fa8d0c75010f9b87b3474ad4a20c5f9d1007f15af299c76f'
FOCUSED_RUNNER = Path('/private/tmp/jarde-for-condition-focused-root-v3.py')
FOCUSED_RUNNER_SHA256 = 'b65b77d235366e22e8d76453b6c966d624a6c0a1eeb1a397cdf71f7236b59c99'
BUILD_RUNNER = Path('/private/tmp/jarde-for-condition-final-lint-build-root-v3.py')
BUILD_RUNNER_SHA256 = '85e25bb6d87a5993305c0fdeb3c58cd9300886701cf8158267df267ce0b92f1e'
ACCEPTED_CANDIDATE_V3 = Path('/private/tmp/jarde-for-condition-candidate-cli-root-v3')
ACCEPTED_CANDIDATE_V3_SHA256 = '51c9a1326031fd10444f7e18a055c6daecef233423c90478257becf24d1e0bd4'
EXPECTED_WORKSPACE = Path('/private/tmp/jarde-for-condition-workspace-root-v4')
EXPECTED_BUILD_RECORD = Path('/private/tmp/jarde-for-condition-final-lint-build-root-v3/execution.json')
EXPECTED_FINAL_CLI = Path('/private/tmp/jarde-for-condition-final-cli-root-v3')
WORKSPACE_SCHEMA = 'for-condition-full-workspace-batched-root-v1'
BUILD_SCHEMA = 'for-condition-final-lint-build-root-v1'
WORKSPACE_SEED = '5350648285461741569'
EXPECTED_BATCHES = 95
EXPECTED_TARGETS = 360
EXPECTED_SOURCE_FILES = 4718
WORKFLOW_SHA256 = 'b545890a55cc4fda31d70b332889c111d789afbf0949581e96bf2b94ea6cd18a'
BASE_ENV = {
    'CARGO_BUILD_JOBS': '1', 'CARGO_INCREMENTAL': '0',
    'CARGO_PROFILE_DEV_DEBUG': '0', 'CARGO_PROFILE_TEST_DEBUG': '0',
    'RUST_TEST_THREADS': '1', 'CARGO_TERM_COLOR': 'always',
}
FREE_LIMIT = 5 * 1024**3
TARGET_LIMIT = 1024**3
DOCELLIPSIS_EXCEPTIONS = {
    ('../tests/fixtures/…', 'crates/jarde-jvm/src/test_fixtures.rs'),
    ('../tests/fixtures/…', 'crates/jarde-reader/src/test_fixtures.rs'),
}
INCLUDE_RE = re.compile(r'(?:include_bytes|include_str)!\s*\(\s*"([^"]+)"')
REQUIRED_TESTS = {
    'tests/p3_for_latch.rs': (
        'condition_effect_producers_remain_ordered_and_the_for_update_is_unique',
        'field_array_and_negate_reads_are_for_conditions_but_written_local_bound_stays_while',
        'local_assignment_of_a_shared_condition_call_remains_a_structured_while',
        'long_float_and_double_array_comparisons_keep_their_counted_for_headers',
    ),
    'tests/p3_nested_loop_side_exits.rs': (
        'upstream_nested_loops2_keeps_outer_for_around_inner_while',
        'a_for_condition_public_late_stop_reports_dimension_usage_and_location',
    ),
    'tests/p3_crossing_array_read_values.rs': (
        'f2_recovers_and_the_unprotected_control_only_projects_its_two_for_headers',
    ),
    'tests/recover_unicode_identifiers.rs': (
        'the_ascii_presentations_only_change_in_the_two_proved_f1_for_headers',
    ),
}


def need(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def sha(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def sha_file(path: Path) -> str:
    return sha(Path(path).read_bytes())


def load_json(path: Path):
    return json.loads(Path(path).read_text(encoding='utf-8'))


def load_module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    need(spec is not None and spec.loader is not None, 'cannot load verifier module: ' + str(path))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


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
        source = str(path.relative_to(ROOT))
        for relative in INCLUDE_RE.findall(path.read_text(encoding='utf-8')):
            included = (path.parent / relative).resolve()
            need(included.is_relative_to(ROOT), 'literal include escapes workspace: ' + str(included))
            if not included.is_file():
                need((relative, source) in DOCELLIPSIS_EXCEPTIONS,
                     f'unrecognized missing literal include: {source}: {relative}')
                continue
            paths.add(included)
    for path in paths:
        need(path.is_file(), 'source closure input is missing: ' + str(path))
    return paths


def pin_sources(paths: set[Path]) -> dict[str, str]:
    return {path.relative_to(ROOT).as_posix(): sha_file(path) for path in sorted(paths)}


def verify_stream(ref: dict, evidence_root: Path) -> bytes:
    path = Path(ref.get('path', ''))
    if not path.is_absolute():
        path = evidence_root / path
    need(not path.is_symlink(), 'raw command stream must not be a symlink: ' + str(path))
    resolved = path.resolve(strict=True)
    need(resolved.is_relative_to(evidence_root.resolve(strict=True)),
         'raw command stream escapes its evidence directory: ' + str(path))
    raw = resolved.read_bytes()
    need(len(raw) == ref.get('bytes') and sha(raw) == ref.get('sha256'),
         'raw command stream bytes/SHA mismatch: ' + str(path))
    return raw


def check_command(row: dict, index: int, argv: list[str], env_expected: dict[str, str], evidence_root: Path) -> None:
    need(row.get('index') == index and row.get('argv') == argv and row.get('cwd') == str(ROOT),
         f'command {index} index/argv/cwd mismatch')
    need(row.get('exit_code') == 0 and row.get('guard_stop') is None,
         f'command {index} failed or hit its resource guard')
    need(row.get('peak_target_bytes', TARGET_LIMIT + 1) <= TARGET_LIMIT and
         row.get('free_bytes_after', 0) >= FREE_LIMIT,
         f'command {index} exceeded the resource guard')
    need(row.get('env_overrides') == env_expected, f'command {index} environment differs from the receipt contract')
    streams = row.get('streams', {})
    need(set(streams) == {'stdout', 'stderr'}, f'command {index} lacks complete raw streams')
    verify_stream(streams['stdout'], evidence_root)
    verify_stream(streams['stderr'], evidence_root)


def expected_workspace_commands(metadata: dict) -> tuple[list[list[str]], list[tuple[str, str, str]]]:
    members = set(metadata['workspace_members'])
    commands: list[list[str]] = []
    coverage: list[tuple[str, str, str]] = []
    for package in metadata['packages']:
        if package['id'] not in members:
            continue
        base = ['cargo', 'test', '-p', package['name'], '--all-features', '--locked', '--message-format', 'json']
        units = [target for target in package['targets']
                 if target['kind'][0] in ('lib', 'bin', 'example', 'bench')]
        if units:
            flags = []
            for kind, flag in (('lib', '--lib'), ('bin', '--bins'), ('example', '--examples'), ('bench', '--benches')):
                if any(target['kind'][0] == kind for target in units):
                    flags.append(flag)
            commands.append(base + flags)
            coverage.extend((package['name'], target['name'], target['kind'][0]) for target in units)
        tests = [target for target in package['targets'] if target['kind'][0] == 'test']
        for start in range(0, len(tests), 4):
            batch = tests[start:start + 4]
            commands.append(base + sum((['--test', target['name']] for target in batch), []))
            coverage.extend((package['name'], target['name'], 'test') for target in batch)
    expected = [(package['name'], target['name'], target['kind'][0])
                for package in metadata['packages'] if package['id'] in members
                for target in package['targets']]
    need(len(coverage) == len(set(coverage)) and set(coverage) == set(expected),
         'metadata batch planner did not cover all workspace targets exactly once')
    return commands, expected


def verify_workspace(product: str, workspace: Path, base, ci):
    workspace = workspace.resolve(strict=True)
    need(workspace.is_dir(), 'workspace receipt directory is invalid')
    execution_path = workspace / 'execution.json'
    metadata_path = workspace / 'metadata.json'
    execution_raw = execution_path.read_bytes()
    metadata_raw = metadata_path.read_bytes()
    ex = json.loads(execution_raw)
    metadata = json.loads(metadata_raw)
    recorded_commands = ex.get('commands')
    need(isinstance(recorded_commands, list), 'workspace receipt has no command-record list')
    failed_rows = []
    for row in recorded_commands:
        if not isinstance(row, dict):
            failed_rows.append({'invalid_row': repr(row)})
        elif row.get('exit_code') != 0 or row.get('guard_stop') is not None:
            failed_rows.append((row.get('index'), row.get('argv'),
                                row.get('exit_code'), row.get('guard_stop')))
    need(not failed_rows, 'workspace command failure/guard cannot be hidden by runner status: ' + repr(failed_rows))
    need(ex.get('schema') == WORKSPACE_SCHEMA and ex.get('status') == 'passed',
         'workspace run is incomplete, wrong schema, or not passing')
    need(ex.get('seed') == WORKSPACE_SEED and ex.get('no_carried_execution') is True,
         'workspace seed is not the first CI seed or execution was carried')
    need(ex.get('source_closure_count_expected') == EXPECTED_SOURCE_FILES,
         'workspace source closure count contract differs')
    fresh_preflight = ex.get('fresh_target_preflight', {})
    need(fresh_preflight.get('target_absent') is True and
         fresh_preflight.get('free_bytes', 0) >= FREE_LIMIT,
         'workspace receipt lacks a fresh absent-target/5-GiB preflight')
    need(ex.get('guard_limits') == {'minimum_free_bytes': FREE_LIMIT,
                                    'maximum_target_bytes': TARGET_LIMIT,
                                    'poll_interval_seconds': 1},
         'workspace receipt resource guard differs from the reviewed limits')
    need(ex.get('rust_min_stack') is None, 'workspace receipt overrides RUST_MIN_STACK')
    expected_workspace_env = {**BASE_ENV, 'PROPTEST_RNG_SEED': WORKSPACE_SEED,
                              'CARGO_PROFILE_TEST_STRIP': 'symbols'}
    need(ex.get('environment_overrides') == expected_workspace_env,
         'workspace receipt environment overrides differ from the expected seed/guard')
    runner = ex.get('runner', {})
    need(runner.get('path') == str(WORKSPACE_RUNNER) and
         runner.get('sha256_before') == WORKSPACE_RUNNER_SHA256 == runner.get('sha256_after') and
         WORKSPACE_RUNNER.is_file() and sha_file(WORKSPACE_RUNNER) == WORKSPACE_RUNNER_SHA256,
         'workspace runner path or before/after/source SHA is invalid')
    references = {row.get('path'): row.get('sha256') for row in ex.get('references', [])}
    expected_references = {
        str(ci.GUARD): ci.GUARD_SHA,
        str(ci.SCAN): ci.SCAN_SHA,
        str(CONDITIONAL_WORKSPACE_RUNNER): CONDITIONAL_WORKSPACE_RUNNER_SHA256,
        str(FOCUSED_RUNNER): FOCUSED_RUNNER_SHA256,
    }
    need(all(references.get(path) == digest for path, digest in expected_references.items()),
         'workspace receipt guard/scanner/source-runner references do not match reviewed pins')
    need(all(path.is_file() and sha_file(path) == digest for path, digest in (
             (ci.GUARD, ci.GUARD_SHA), (ci.SCAN, ci.SCAN_SHA),
             (CONDITIONAL_WORKSPACE_RUNNER, CONDITIONAL_WORKSPACE_RUNNER_SHA256),
             (FOCUSED_RUNNER, FOCUSED_RUNNER_SHA256))),
         'current workspace guard/scanner/source runners differ from their pinned SHAs')

    closure = source_paths()
    pins_before = ex.get('source_pins_before')
    pins_after = ex.get('source_pins_after')
    need(isinstance(pins_before, dict) and pins_before == pins_after,
         'workspace source pins changed before/after the workspace run')
    need(len(pins_before) == EXPECTED_SOURCE_FILES and set(pins_before) ==
         {path.relative_to(ROOT).as_posix() for path in closure},
         'workspace source pins do not exactly match the complete 4,718-file closure')
    live = pin_sources(closure)
    need(live == pins_before, 'workspace source pins differ from current live source bytes')
    git_pins = ci.blob_sha256s(product, pins_before)
    need(git_pins == pins_before, 'workspace source pins differ from product commit Git blobs')
    need('tests/fixtures/corpus-fingerprint.json' in pins_before,
         'workspace source closure omits the runtime-read corpus fingerprint')

    by_id, by_named, inventory = ci.metadata_targets(metadata)
    target_count = len(inventory)
    need(target_count == EXPECTED_TARGETS, 'fresh Cargo metadata does not describe exactly 360 targets')
    declared = [tuple(row) for row in ex.get('target_inventory', [])]
    need(len(declared) == len(set(declared)) and set(declared) == set(inventory),
         'workspace target inventory does not exactly match Cargo metadata')
    commands = ex.get('commands', [])
    need(len(commands) == 98 and [row.get('index') for row in commands] == list(range(98)),
         'workspace receipt does not contain fmt, metadata, 95 batches, and clean in order')
    fmt_argv = ['cargo', 'fmt', '--all', '--', '--check']
    metadata_argv = ['cargo', 'metadata', '--no-deps', '--format-version', '1', '--locked']
    clean_argv = ['cargo', 'clean']
    workspace_env = expected_workspace_env
    check_command(commands[0], 0, fmt_argv, workspace_env, workspace)
    need(ex.get('format_check') == {'exit_code': 0, 'guard_stop': None},
         'workspace fmt --check summary is not passing')
    check_command(commands[1], 1, metadata_argv, workspace_env, workspace)
    saved_metadata = verify_stream(commands[1]['streams']['stdout'], workspace)
    need(saved_metadata == metadata_raw and sha(metadata_raw) == sha(saved_metadata),
         'metadata.json differs from the exact cargo metadata raw stdout')
    need(ex.get('metadata', {}).get('command_index') == 1 and
         ex['metadata'].get('target_count') == target_count and
         ex['metadata'].get('batch_count') == EXPECTED_BATCHES,
         'workspace metadata record does not bind 360 targets and 95 batches')
    planned, expected_inventory = expected_workspace_commands(metadata)
    need(len(planned) == EXPECTED_BATCHES and len(expected_inventory) == EXPECTED_TARGETS,
         'metadata does not derive the required 95 batches over 360 targets')
    need(ex.get('expected_batch_count') == EXPECTED_BATCHES and
         ex.get('expected_target_count') == EXPECTED_TARGETS,
         'workspace record expected batch/target counts differ')

    observations = []
    for batch_number, (row, argv) in enumerate(zip(commands[2:97], planned), start=0):
        index = batch_number + 2
        check_command(row, index, argv, workspace_env, workspace)
        observed = ci.local_command_observation(row, by_id, by_named)
        ci.check_workspace_argv(row.get('argv', []), observed, by_named)
        need(len(observed) == len(row.get('observed_counts', [])) ==
             len(row.get('executed_targets', [])),
             f'workspace batch {index} raw/test artifact cardinality mismatch')
        batch_totals = [sum(item['counts'][i] for item in observed) for i in range(3)]
        need(row.get('summary_test_totals') == batch_totals,
             f'workspace batch {index} reported summary totals differ from its named raw outcomes')
        observations.extend(observed)
    keys = [row['key'] for row in observations]
    need(len(keys) == len(set(keys)) and set(keys) == set(inventory),
         'actual workspace target observations do not cover every target exactly once')
    totals = [sum(row['counts'][i] for row in observations) for i in range(3)]
    need(totals[0] > 0 and totals[1] == 0,
         'full workspace must have at least one pass and no failed test outcomes')
    need(ex.get('workspace_summary_count') == len(observations) == target_count and
         ex.get('workspace_totals') == totals,
         'workspace summary count/totals differ from reconstructed target outcomes')

    check_command(commands[97], 97, clean_argv, workspace_env, workspace)
    cleanup = ex.get('cleanup', {})
    need(cleanup.get('attempted') is True and cleanup.get('argv') == clean_argv and
         cleanup.get('guarded') is True and cleanup.get('status') == 'passed' and
         cleanup.get('command') == commands[97],
         'workspace finally cleanup does not record the successful guarded cargo clean command')
    need(cleanup.get('target_absent_after') is True and not (ROOT / 'target').exists() and
         cleanup.get('pins_unchanged') is True and cleanup.get('source_pins_after') == pins_before,
         'workspace cleanup did not remove target or preserve the complete source closure')

    local = {'execution': str(execution_path), 'execution_sha256': sha(execution_raw),
             'metadata': str(metadata_path), 'metadata_sha256': sha(metadata_raw),
             'seed': ex['seed'], 'target_count': target_count, 'batch_count': len(planned),
             'workspace_totals': totals, 'source_pin_count': len(pins_before),
             'target_observations': observations}
    return ex, metadata, local


def verify_required_tests(local_observations: list[dict], product: str,
                          pins: dict[str, str], metadata: dict, ci) -> dict:
    by_source = {source: [target for target in local_observations if target['source_path'] == source]
                 for source in REQUIRED_TESTS}
    _, by_named, _ = ci.metadata_targets(metadata)
    checked = []
    source_text = {}
    for source, names in REQUIRED_TESTS.items():
        need(source in pins, 'workspace pins omit new required test source: ' + source)
        blob = ci.git_blob(product, source)
        digest = hashlib.sha256(blob).hexdigest()
        need(digest == pins[source] == sha_file(ROOT / source),
             'required test source is not bound to product Git and live bytes: ' + source)
        text = blob.decode('utf-8')
        for name in names:
            pattern = r'(?m)^\s*(?:pub\s+)?fn\s+' + re.escape(name) + r'\s*\('
            need(re.search(pattern, text) is not None,
                 f'required test definition missing from product-pinned {source}: {name}')
        source_text[source] = {'sha256': digest, 'test_names': list(names)}
        target_rows = by_source[source]
        need(len(target_rows) == 1, 'required test source does not map to exactly one executed Cargo target: ' + source)
        target = target_rows[0]
        target_key = target['key']
        metadata_target = by_named.get(target_key)
        need(metadata_target is not None and metadata_target['source_path'] == source,
             'required test target metadata source mismatch: ' + source)
        outcomes = target['outcomes']
        passed = []
        for name in names:
            matches = [(actual, status) for actual, status in outcomes
                       if actual == name or actual.endswith('::' + name)]
            need(len(matches) == 1 and matches[0][1] == 'ok',
                 f'new required named local outcome did not pass exactly once: {source}::{name}')
            passed.append(matches[0][0])
        checked.append({'source_path': source, 'target': target_key,
                        'required_test_names': list(names), 'passed_outcomes': passed})
    return {'new_tests': checked, 'source_text_checks': source_text,
            'prior_outcomes_guarded_by_full_named_comparison': True}


def workflow_clippy_argv(product: str, ci) -> list[str]:
    raw = ci.git_blob(product, '.github/workflows/ci.yml')
    need(hashlib.sha256(raw).hexdigest() == WORKFLOW_SHA256 and
         sha_file(ROOT / '.github/workflows/ci.yml') == WORKFLOW_SHA256,
         'product workflow differs from the CI verifier pinned workflow')
    text = raw.decode('utf-8')
    start = text.index('          cargo clippy')
    end = text.index('\n\n', start)
    return text[start:end].split()


def verify_final_build(product: str, workspace_pins: dict[str, str], build_record: Path,
                       accepted_candidate: Path, ci) -> dict:
    need(BUILD_RUNNER.is_file() and sha_file(BUILD_RUNNER) == BUILD_RUNNER_SHA256,
         'final lint/build runner path or SHA mismatch')
    need(accepted_candidate == ACCEPTED_CANDIDATE_V3 and accepted_candidate.is_file() and
         not accepted_candidate.is_symlink() and
         sha_file(accepted_candidate) == ACCEPTED_CANDIDATE_V3_SHA256 and
         stat.S_IMODE(accepted_candidate.stat().st_mode) == 0o555,
         'accepted v3 candidate CLI path/hash/mode mismatch')
    raw = build_record.read_bytes()
    build = json.loads(raw)
    need(build.get('schema') == BUILD_SCHEMA and build.get('status') == 'passed_candidate_build_only',
         'final build receipt is incomplete, wrong-schema, or not passing')
    need(build.get('focused_passed') is True and
         build.get('final_lint_strict_byte_identity_passed') is True and
         build.get('candidate_byte_identity') is True,
         'final build receipt does not report successful lint/strict/accepted-candidate identity')
    need(build.get('runner_sha256_before') == BUILD_RUNNER_SHA256 ==
         build.get('runner_sha256_after') and sha_file(BUILD_RUNNER) == BUILD_RUNNER_SHA256,
         'final build runner before/after SHA mismatch')
    need(build.get('source_pins_before') == workspace_pins == build.get('source_pins_after'),
         'final build source pins differ from workspace before/after closure')
    expected_candidate_changes = sorted([
        'tests/p3_nested_loop_side_exits.rs',
        'tests/fixtures/java8/cf11-for-condition/README.md',
        'tests/p3_crossing_array_read_values.rs',
        'tests/recover_unicode_identifiers.rs',
    ])
    need(build.get('changes_since_accepted_candidate') == expected_candidate_changes,
         'final build source-difference receipt does not match the focused v3 candidate delta')
    need(len(workspace_pins) == EXPECTED_SOURCE_FILES and
         ci.blob_sha256s(product, workspace_pins) == workspace_pins and
         {path: sha_file(ROOT / path) for path in workspace_pins} == workspace_pins,
         'final build source closure differs from current product Git blobs or live bytes')
    need(build.get('target_absent') is True and not (ROOT / 'target').exists(),
         'final build receipt did not leave target absent after cargo clean')
    expected_env = {**BASE_ENV, 'PROPTEST_RNG_SEED': WORKSPACE_SEED}
    need(build.get('actual_env_overrides') == expected_env and
         build.get('rust_min_stack') is None,
         'final build environment or default RUST_MIN_STACK setting differs')

    clippy = workflow_clippy_argv(product, ci)
    expected_argvs = [
        ['cargo', 'fmt', '--all', '--', '--check'],
        clippy,
        ['openspec', 'validate', '--all', '--strict', '--no-interactive'],
        ['cargo', 'build', '-p', 'jarde-cli', '--bin', 'jarde-cli', '--locked'],
        ['cargo', 'clean'],
    ]
    commands = build.get('commands', [])
    need(len(commands) == 5, 'final build receipt must contain exactly five commands including cargo clean')
    build_output = build_record.parent.resolve(strict=True)
    for index, (row, argv) in enumerate(zip(commands, expected_argvs)):
        # The receipt's top-level actual_env_overrides includes PROPTEST_RNG_SEED;
        # guard rows record the template's six CARGO/RUST overrides only.
        check_command(row, index, argv, BASE_ENV, build_output)

    cli = build.get('candidate_cli')
    need(isinstance(cli, dict) and cli.get('path') == str(EXPECTED_FINAL_CLI) and
         cli.get('sha256') == ACCEPTED_CANDIDATE_V3_SHA256 and cli.get('mode') == '0o555',
         'final build CLI identity differs from the accepted candidate v3')
    final_cli = Path(cli['path'])
    need(final_cli.is_file() and not final_cli.is_symlink() and
         final_cli.stat().st_size == cli.get('bytes') and
         sha_file(final_cli) == ACCEPTED_CANDIDATE_V3_SHA256 and
         stat.S_IMODE(final_cli.stat().st_mode) == 0o555,
         'final CLI bytes/hash/mode differ from the accepted candidate v3')
    need(final_cli.stat().st_size == accepted_candidate.stat().st_size,
         'final CLI byte length differs from accepted candidate v3')
    return {'record_path': str(build_record.resolve()), 'record_sha256': sha(raw),
            'runner_path': str(BUILD_RUNNER), 'runner_sha256': BUILD_RUNNER_SHA256,
            'commands': len(commands), 'target_absent_after_clean': True,
            'candidate_cli': {'path': str(final_cli), 'bytes': final_cli.stat().st_size,
                              'sha256': ACCEPTED_CANDIDATE_V3_SHA256, 'mode': '0o555',
                              'byte_identical_to_accepted_v3': True}}


def verify_references(base, ci, product: str) -> dict:
    paths = [
        (BASE_PATH, BASE_SHA256),
        (HELPER_PATH, HELPER_SHA256),
        (CAPTURE_PATH, CAPTURE_SHA256),
        (Path(base.PUBLIC_LABEL_RULE), base.PUBLIC_LABEL_RULE_SHA),
        (Path(base.WORKSPACE_RUNNER), base.WORKSPACE_RUNNER_SHA),
        (Path(base.COMMITTED_BASE), base.COMMITTED_BASE_SHA),
        (WORKSPACE_RUNNER, WORKSPACE_RUNNER_SHA256),
        (CONDITIONAL_WORKSPACE_RUNNER, CONDITIONAL_WORKSPACE_RUNNER_SHA256),
        (FOCUSED_RUNNER, FOCUSED_RUNNER_SHA256),
        (BUILD_RUNNER, BUILD_RUNNER_SHA256),
        (ci.GUARD, ci.GUARD_SHA),
        (ci.SCAN, ci.SCAN_SHA),
    ]
    for path, expected in paths:
        need(path.is_file() and sha_file(path) == expected, 'verifier reference file/SHA mismatch: ' + str(path))
    for path in (BASE_PATH, HELPER_PATH, CAPTURE_PATH, ci.GUARD, ci.SCAN,
                 Path(base.COMMITTED_BASE)):
        rel = path.resolve().relative_to(ROOT).as_posix()
        need(hashlib.sha256(ci.git_blob(product, rel)).hexdigest() == sha_file(path),
             'reference does not match product Git blob: ' + rel)
    need(base.HELPER_SHA == HELPER_SHA256 and base.CAPTURE_SHA == CAPTURE_SHA256 and
         base.WORKSPACE_CMD == 'cargo test --workspace --all-targets --all-features --locked' and
         tuple(base.SEEDS) == ('5350648285461741569', '5350648285461741570'),
         'committed verifier/helper CI constants changed')
    need(base.WORKFLOW_SHA == WORKFLOW_SHA256,
         'committed verifier workflow pin changed')
    return {'committed_verifier': {'path': str(BASE_PATH), 'sha256': BASE_SHA256},
            'ci_helper': {'path': str(HELPER_PATH), 'sha256': HELPER_SHA256},
            'capture_collector': {'path': str(CAPTURE_PATH), 'sha256': CAPTURE_SHA256},
            'workspace_runner': {'path': str(WORKSPACE_RUNNER), 'sha256': WORKSPACE_RUNNER_SHA256},
            'conditional_workspace_runner': {'path': str(CONDITIONAL_WORKSPACE_RUNNER), 'sha256': CONDITIONAL_WORKSPACE_RUNNER_SHA256},
            'focused_runner': {'path': str(FOCUSED_RUNNER), 'sha256': FOCUSED_RUNNER_SHA256},
            'final_build_runner': {'path': str(BUILD_RUNNER), 'sha256': BUILD_RUNNER_SHA256}}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--product-commit', required=True)
    parser.add_argument('--run-id', required=True)
    parser.add_argument('--capture', type=Path, required=True)
    parser.add_argument('--workspace', type=Path, default=EXPECTED_WORKSPACE)
    parser.add_argument('--final-build', type=Path, default=EXPECTED_BUILD_RECORD)
    parser.add_argument('--candidate-cli', type=Path, default=ACCEPTED_CANDIDATE_V3)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    need(re.fullmatch(r'[0-9a-f]{40}', args.product_commit) is not None,
         'product commit must be lowercase 40-hex')
    need(args.run_id.isdecimal(), 'CI run id must be decimal')
    out = args.out.resolve()
    need(not out.exists(), 'refusing to overwrite CI acceptance output: ' + str(out))
    need(BASE_PATH.is_file() and sha_file(BASE_PATH) == BASE_SHA256,
         'committed verifier file/SHA mismatch before import')
    need(HELPER_PATH.is_file() and sha_file(HELPER_PATH) == HELPER_SHA256,
         'CI helper file/SHA mismatch before import')
    base = load_module('committed_cf11_own_ci_verifier', BASE_PATH)
    ci = base.ci
    need(callable(base.verify_ci_capture) and callable(base.parse_ci_group),
         'committed verifier lacks its pinned capture/group parsers')
    refs = verify_references(base, ci, args.product_commit)

    ex, metadata, local = verify_workspace(args.product_commit, args.workspace, base, ci)
    build = verify_final_build(args.product_commit, ex['source_pins_before'],
                               args.final_build.resolve(strict=True), args.candidate_cli.resolve(strict=True), ci)
    required = verify_required_tests(local['target_observations'], args.product_commit,
                                     ex['source_pins_before'], metadata, ci)
    ci_result = base.verify_ci_capture(args.capture.resolve(strict=True), args.product_commit,
                                       args.run_id, local['target_observations'])
    acceptance = {
        'schema': 'for-condition-own-ci-current-product-acceptance-luna-v4',
        'status': 'accepted',
        'product_commit': args.product_commit,
        'run_id': int(args.run_id),
        'ci': {'jobs': 4, 'steps': ci_result['steps'], 'seed_runs': ci_result['seed_runs'],
               'run_summary': ci_result['summary'], 'aggregate_counts_predeclared': False,
               'masked_public_label_mappings': ci_result['masked_public_label_mappings'],
               'public_label_rule': ci_result['public_label_rule']},
        'fresh_local_workspace': {key: value for key, value in local.items()
                                  if key != 'target_observations'},
        'workspace_runner_contract': {'path': ex['runner']['path'],
                                      'sha256': ex['runner']['sha256_before'],
                                      'complete_batch_count': local['batch_count'],
                                      'target_count': local['target_count']},
        'final_cli_build': build,
        'required_tests': required,
        'workspace_target_count': local['target_count'],
        'all_local_and_ci_named_outcomes_compared': True,
        'references': refs,
        'evidence': {'capture': str(args.capture.resolve()),
                     'capture_execution_sha256': sha_file(args.capture.resolve() / 'capture-execution-root-v2.json'),
                     'workspace_execution': local['execution'],
                     'workspace_execution_sha256': local['execution_sha256'],
                     'workspace_metadata': local['metadata'],
                     'workspace_metadata_sha256': local['metadata_sha256'],
                     'final_build_record': build['record_path'],
                     'final_build_record_sha256': build['record_sha256'],
                     'accepted_candidate_v3_path': str(ACCEPTED_CANDIDATE_V3),
                     'accepted_candidate_v3_sha256': ACCEPTED_CANDIDATE_V3_SHA256},
        'acceptance_scope': ('Exact parameterized product Git commit and CI run. Both configured CI seeds are parsed per target and every named outcome is compared with the fresh local workspace receipt; masked labels bind only to a unique same-status local outcome. Workspace and final build source closures are the complete 4,718-file include/fixture closure and are checked against product Git blobs and live bytes. The final CLI is byte-identical to accepted candidate v3.'),
    }
    out.parent.mkdir(parents=True, exist_ok=True)
    with out.open('x', encoding='utf-8') as stream:
        stream.write(json.dumps(acceptance, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({'status': 'accepted', 'product_commit': args.product_commit,
                      'run_id': int(args.run_id), 'jobs': 4, 'steps': ci_result['steps'],
                      'target_count': local['target_count'],
                      'workspace_totals': [row['workspace_totals'] for row in ci_result['seed_runs']],
                      'masked_public_labels': len(ci_result['masked_public_label_mappings'])},
                     ensure_ascii=False))
    return 0


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except Exception as error:
        print('for-condition own-CI verification failed: ' + str(error), file=sys.stderr)
        raise
