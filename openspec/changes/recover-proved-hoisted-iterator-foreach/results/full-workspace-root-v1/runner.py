#!/usr/bin/env python3
"""Run guarded full-workspace test batches for constructor-conditional evidence."""

from __future__ import annotations

import argparse
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
import tomllib

sys.dont_write_bytecode = True
ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
OUT = Path('/private/tmp/jarde-hoisted-iterator-full-validation-prep-luna-v1/workspace-run')
TEMPLATE = ROOT / 'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
SCAN_PATH = ROOT / 'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
TEMPLATE_SHA256 = '51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
SCAN_SHA256 = 'd954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
FREE_LIMIT = 5 * 1024**3
TARGET_LIMIT = 1024**3
RUNTIME_CLI = Path('/private/tmp/jarde-hoisted-iterator-candidate-cli-root-v2')
RUNTIME_CLI_SHA256 = '8bdb868ab58e4c9196970438c71de529996c66ba826c84fe216794d328fdfc9a'
STRIPPED_ENV_KEYS = ('JAVA_TOOL_OPTIONS', '_JAVA_OPTIONS', 'JDK_JAVA_OPTIONS', 'CLASSPATH')
DOCELLIPSIS_EXCEPTIONS = {
    ('../tests/fixtures/…', 'crates/jarde-jvm/src/test_fixtures.rs'),
    ('../tests/fixtures/…', 'crates/jarde-reader/src/test_fixtures.rs'),
}
INCLUDE_RE = re.compile(r'(?:include_bytes|include_str)!\s*\(\s*"([^"]+)"')
SUMMARY_RE = re.compile(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;')
OUTCOME_RE = re.compile(r'^test (.+) \.\.\. (ok|ignored|FAILED)(?:, .*)?$', re.MULTILINE)
REQUIRED_NAMED_TESTS = {
    'conditional_constructor_public_reports_agree_across_evidence_selections',
    'conditional_constructor_positive_all_evidence_reports_four_exact_methods',
    'conditional_constructor_range_excluding_all_new_bcis_keeps_summary_and_text',
    'conditional_constructor_public_late_stop_and_precancel_publish_atomically',
    'conditional_sites_are_not_presented_when_the_final_body_is_explanation_only',
    'field::tests::conditional_new_receipt_counts_only_one_method_local_direct_new',
    'field::tests::conditional_new_receipt_rejects_derived_primary_and_derived_only_candidate_anchors',
    'init::tests::conditional_constructor_output_has_one_live_stack_phi_and_one_join_consumer',
    'init::tests::actual_local_conditional_new_has_its_local_store_as_the_single_consumer',
    'init::tests::actual_shared_field_return_keeps_dup_outside_the_new_expression',
    'init::tests::actual_nested_conditional_constructor_arms_do_not_become_one_two_way_site',
    'init::tests::actual_independent_branch_effect_stays_on_the_existing_statement_path',
    'nested_inner_iterator_projects_without_changing_outer_loop_or_binding',
    'nested_foreach_late_stop_and_precancel_leave_no_partial_method_artifact',
    'build::tests::iterator_declaration_gate_uses_real_local_owner_and_refuses_incomplete_facts',
    'build::tests::iterator_declaration_gate_rejects_real_split_slot_identity_changes',
    'build::tests::iterator_phi_gate_counts_real_duplicate_inputs_and_ignores_unrelated_phi',
    'build::tests::mixed_iterator_real_phi_is_live_and_fails_the_production_use_gate',
    'build::tests::alias_branch_phi_does_not_expand_the_exact_value_reader_allowlist',
}
REQUIRED_DYNAMIC_SOURCE_PATHS = {
    'tests/recover_proved_conditional_constructor_values.rs',
    'tests/fixtures/java8/cf11-conditional-constructor-values/UpstreamSequentialLoops2.class',
    'tests/fixtures/java8/cf11-conditional-constructor-values/UpstreamSequentialLoops2.java',
    'tests/fixtures/java8/cf11-conditional-constructor-values/ConditionalConstructionPositive.class',
    'tests/fixtures/java8/cf11-conditional-constructor-values/ConditionalConstructionPositive.java',
    'tests/fixtures/java8/cf11-conditional-constructor-values/ConditionalConstructionRefusals.class',
    'tests/fixtures/java8/cf11-conditional-constructor-values/ConditionalConstructionRefusals.java',
    'tests/fixtures/java8/cf11-conditional-constructor-values/README.md',
    'tests/fixtures/java8/cf11-conditional-constructor-values/pins.json',
    'tests/p3_hoisted_iterator_foreach.rs',
    'tests/fixtures/p3-hoisted-iterator/HoistedIteratorPhiControls.java',
    'tests/fixtures/p3-hoisted-iterator/HoistedIteratorPhiControls.class',
    'tests/fixtures/p3-hoisted-iterator/README.md',
}


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def static_target_inventory() -> list[tuple[str, str, str]]:
    """Discover current Cargo targets from workspace manifests and conventional source paths."""
    workspace = tomllib.loads((ROOT / 'Cargo.toml').read_text(encoding='utf-8'))
    members = workspace.get('workspace', {}).get('members', [])
    if not members:
        raise RuntimeError('workspace member list is empty')
    rows: set[tuple[str, str, str]] = set()

    def add(package: str, kind: str, name: str, path: Path) -> None:
        if path.is_file():
            rows.add((package, name, kind))

    for member in members:
        manifest = ROOT / member / 'Cargo.toml' if member != '.' else ROOT / 'Cargo.toml'
        if not manifest.is_file():
            raise FileNotFoundError(f'workspace member manifest missing: {manifest}')
        data = tomllib.loads(manifest.read_text(encoding='utf-8'))
        package = data['package']['name']
        package_data = data['package']
        directory = manifest.parent
        lib = data.get('lib')
        if package_data.get('autolib', True):
            if lib is not None:
                lib_path = directory / lib.get('path', 'src/lib.rs')
                lib_name = lib.get('name', package.replace('-', '_'))
                add(package, 'lib', lib_name, lib_path)
            else:
                add(package, 'lib', package.replace('-', '_'), directory / 'src/lib.rs')

        explicit_bins = data.get('bin', [])
        for spec in explicit_bins:
            path = directory / spec.get('path', f"src/bin/{spec.get('name', package)}/main.rs")
            name = spec.get('name', package if path == directory / 'src/main.rs' else path.stem)
            add(package, 'bin', name, path)
        if package_data.get('autobins', True):
            if not explicit_bins:
                add(package, 'bin', package, directory / 'src/main.rs')
            bindir = directory / 'src/bin'
            if bindir.is_dir():
                for path in bindir.glob('*.rs'):
                    add(package, 'bin', path.stem, path)
                for path in bindir.glob('*/main.rs'):
                    add(package, 'bin', path.parent.name, path)

        for kind, folder, auto_key in (
            ('example', 'examples', 'autoexamples'),
            ('test', 'tests', 'autotests'),
            ('bench', 'benches', 'autobenches'),
        ):
            for spec in data.get(kind, []):
                path = directory / spec['path']
                add(package, kind, spec.get('name', path.stem), path)
            if package_data.get(auto_key, True):
                base = directory / folder
                if base.is_dir():
                    for path in base.glob('*.rs'):
                        add(package, kind, path.stem, path)
                    for path in base.glob('*/main.rs'):
                        add(package, kind, path.parent.name, path)

        build_path = package_data.get('build', 'build.rs')
        if build_path and (directory / build_path).is_file():
            rows.add((package, 'build-script-build', 'custom-build'))

    return sorted(rows)


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
    global record, guard, env, source_file_set, clean_armed, OUT
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--regression-receipt', type=Path, required=True)
    ap.add_argument('--regression-receipt-sha256', required=True)
    ap.add_argument('--candidate-cli', type=Path, default=Path('/private/tmp/jarde-hoisted-iterator-candidate-cli-root-v3'))
    ap.add_argument('--candidate-cli-sha256', default=RUNTIME_CLI_SHA256)
    ap.add_argument('--out', type=Path, default=OUT)
    args = ap.parse_args()
    OUT = args.out.resolve()
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
    regression_path = args.regression_receipt.resolve(strict=True)
    if regression_path.is_symlink() or sha256(regression_path) != args.regression_receipt_sha256:
        raise SystemExit('regression receipt path/hash mismatch')
    regression = json.loads(regression_path.read_text(encoding='utf-8'))
    if (regression.get('schema') != 'hoisted-iterator-regressions-root-v3'
            or regression.get('status') != 'passed_regressions_candidate_only'
            or regression.get('target_absent') is not True
            or regression.get('runner_sha256_before') != regression.get('runner_sha256_after')
            or regression.get('source_pins_before') != regression.get('source_pins_after')
            or regression.get('source_pins_before') != source_pins_before):
        raise SystemExit('completed regression receipt is not bound to this clean source closure')
    if regression.get('same_cli_as_runtime_candidate_v2') is not True:
        raise SystemExit('regression candidate CLI was not byte-equal to runtime candidate v2')
    candidate_cli = args.candidate_cli.resolve(strict=True)
    if (candidate_cli.is_symlink() or candidate_cli.stat().st_size != 61314560
            or sha256(candidate_cli) != args.candidate_cli_sha256
            or args.candidate_cli_sha256 != RUNTIME_CLI_SHA256
            or candidate_cli.read_bytes() != RUNTIME_CLI.read_bytes()
            or (candidate_cli.stat().st_mode & 0o777) != 0o555):
        raise SystemExit('candidate CLI does not exactly match the frozen runtime-tested CLI')
    cli_pin = {'path': str(candidate_cli), 'bytes': candidate_cli.stat().st_size,
               'sha256': sha256(candidate_cli), 'mode': oct(candidate_cli.stat().st_mode & 0o777)}
    if regression.get('candidate_cli') != cli_pin:
        raise SystemExit('regression receipt CLI identity differs from frozen candidate CLI')
    regression_builds = [row for row in regression.get('commands', [])
                         if row.get('argv') == ['cargo', 'build', '-p', 'jarde-cli', '--bin', 'jarde-cli', '--locked']]
    if len(regression_builds) != 1 or regression_builds[0].get('index') != 6 or regression_builds[0].get('exit_code') != 0 or regression_builds[0].get('guard_stop') is not None:
        raise SystemExit('regression receipt does not contain successful candidate build at command index 6')
    if set(regression_builds[0].get('streams', {})) != {'stdout', 'stderr'}:
        raise SystemExit('regression candidate build lacks raw stdout/stderr streams')
    for stream, stream_pin in regression_builds[0].get('streams', {}).items():
        raw_path = Path(stream_pin.get('path', ''))
        if raw_path.is_symlink() or not raw_path.is_file():
            raise SystemExit('regression candidate-build raw stream missing or symlink: ' + stream)
        raw = raw_path.read_bytes()
        if (type(stream_pin.get('bytes')) is not int or len(raw) != stream_pin['bytes']
                or hashlib.sha256(raw).hexdigest() != stream_pin.get('sha256')):
            raise SystemExit('regression candidate-build raw stream pin differs: ' + stream)
    relative_sources = set(source_pins_before)
    missing_required_sources = REQUIRED_DYNAMIC_SOURCE_PATHS - relative_sources
    if missing_required_sources:
        raise SystemExit(f'required constructor evidence inputs are outside source closure: {sorted(missing_required_sources)}')
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
        'schema': 'hoisted-iterator-full-workspace-batched-root-v2',
        'status': 'running',
        'started_at': datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'seed': '5350648285461741569',
        'no_carried_execution': True,
        'regression_input_receipt': {'path': str(regression_path), 'sha256': args.regression_receipt_sha256, 'schema': regression['schema'], 'status': regression['status']},
        'candidate_cli': cli_pin,
        'source_closure_count_at_start': len(source_pins_before),
        'required_dynamic_source_paths': sorted(REQUIRED_DYNAMIC_SOURCE_PATHS),
        'source_pins_before': source_pins_before,
        'source_pins_after': None,
        'stripped_environment_keys': stripped,
        'rust_min_stack': env.get('RUST_MIN_STACK'),
        'environment_overrides': guard.ENV_VALUES,
        'fresh_target_preflight': {'target_absent': True, 'free_bytes': free_before},
        'guard_limits': {'minimum_free_bytes': FREE_LIMIT, 'maximum_target_bytes': TARGET_LIMIT,
                         'poll_interval_seconds': 1},
        'references': [
            {'path': '/private/tmp/jarde-hoisted-iterator-full-validation-prep-luna-v1/workspace_runner.py',
             'sha256': sha256(Path(__file__).resolve())},
            {'path': str(TEMPLATE), 'sha256': TEMPLATE_SHA256},
            {'path': str(SCAN_PATH), 'sha256': SCAN_SHA256},
            {'path': str(regression_path),
             'sha256': args.regression_receipt_sha256},
            {'path': str(candidate_cli), 'sha256': args.candidate_cli_sha256},
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

    def command(index: int, argv: list[str], *, allow_nonzero: bool = False) -> dict:
        assert_sources_unchanged(f'before-command-{index}')
        row = guard.run_command(index, argv, env)
        record['commands'].append(row)
        save()
        if row['guard_stop'] is not None or (row['exit_code'] != 0 and not allow_nonzero):
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
        static_targets = static_target_inventory()
        if sorted(expected_targets) != static_targets:
            missing = sorted(set(static_targets) - set(expected_targets))
            unexpected = sorted(set(expected_targets) - set(static_targets))
            raise RuntimeError(f'Cargo metadata target inventory differs from manifests/files; missing={missing}, unexpected={unexpected}')
        expected_batches_from_inventory = sum(
            1 for package in metadata['packages'] if package['id'] in set(metadata['workspace_members'])
            and any(target['kind'][0] in ('lib', 'bin', 'example', 'bench') for target in package['targets'])
        ) + sum(
            (sum(1 for target in package['targets'] if target['kind'][0] == 'test') + 3) // 4
            for package in metadata['packages'] if package['id'] in set(metadata['workspace_members'])
        )
        if len(batches) != expected_batches_from_inventory:
            raise RuntimeError(f'computed batch count mismatch: {len(batches)} != {expected_batches_from_inventory}')
        members = set(metadata['workspace_members'])
        expected_ids = [(package['id'], target['name'], target['kind'][0])
                        for package in metadata['packages'] if package['id'] in members
                        for target in package['targets']]
        record['metadata'] = {'command_index': metadata_row['index'],
                              'workspace_member_count': len(members),
                              'target_count': len(expected_targets),
                              'batch_count': len(batches)}
        record['target_inventory'] = expected_targets
        record['expected_batch_count'] = len(batches)
        record['expected_target_count'] = len(expected_targets)
        record['expected_command_count_including_clean'] = 2 + len(batches) + 1
        record['static_manifest_target_inventory'] = static_targets
        save()

        all_summaries = []
        all_actual_targets = []
        for batch_index, argv in enumerate(batches, start=2):
            row = command(batch_index, argv, allow_nonzero=True)
            raw = (OUT / f'{batch_index}.stdout.raw').read_text(encoding='utf-8', errors='replace')
            summary_rows = [{'status': match.group(1), 'passed': int(match.group(2)),
                             'failed': int(match.group(3)), 'ignored': int(match.group(4)),
                             'raw_line': raw[match.start():].splitlines()[0]}
                            for match in SUMMARY_RE.finditer(raw)]
            summaries = [(row['passed'], row['failed'], row['ignored']) for row in summary_rows]
            outcomes = [{'name': match.group(1), 'status': match.group(2), 'raw_line': match.group(0)}
                        for match in OUTCOME_RE.finditer(raw)]
            expected_outcome_count = sum(passed + failed + ignored for passed, failed, ignored in summaries)
            row['observed_summaries'] = summary_rows
            row['named_test_outcomes'] = outcomes
            row['named_test_outcome_count'] = len(outcomes)
            row['summary_test_totals'] = [sum(item[index] for item in summaries) for index in range(3)]
            save()
            if len(outcomes) != expected_outcome_count:
                raise RuntimeError(f'batch {batch_index} named outcome/summary mismatch: '
                                   f'{len(outcomes)} outcomes != {expected_outcome_count}')
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
                    executables.append((target['kind'][0], executable_path.resolve(strict=True)))
            if len(actual_targets) != len(set(actual_targets)) or len(actual_targets) != len(summaries):
                raise RuntimeError(f'batch {batch_index} summary/artifact cardinality mismatch')
            if len(executables) != len(set(executables)):
                raise RuntimeError(f'batch {batch_index} reported duplicate test executables')
            row['observed_counts'] = [list(item) for item in summaries]
            row['executed_targets'] = [list(item) for item in actual_targets]
            all_summaries.extend(summaries)
            all_actual_targets.extend(actual_targets)
            for kind, executable in executables:
                deps = (ROOT / 'target/debug/deps').resolve(strict=True)
                expected_parent = (ROOT / 'target/debug/examples').resolve(strict=True) if kind == 'example' else deps
                if executable.parent != expected_parent or executable.is_symlink():
                    raise RuntimeError(f'refusing to remove out-of-scope test executable: {executable}')
                info = executable.lstat()
                if not stat.S_ISREG(info.st_mode) or not info.st_mode & 0o111:
                    raise RuntimeError(f'refusing to remove non-executable/non-regular artifact: {executable}')
                removed = {'path': str(executable), 'bytes': info.st_size,
                           'sha256': sha256(executable), 'completed_batch': batch_index}
                executable.unlink()
                record['removed_completed_test_executables'].append(removed)
            save()
            if (row['exit_code'] != 0 or row['guard_stop'] is not None
                    or not summary_rows or any(item['status'] != 'ok' or item['failed'] != 0
                                               for item in summary_rows)
                    or any(item['status'] == 'FAILED' for item in outcomes)):
                raise RuntimeError(f'batch {batch_index} failed or hit a guard after preserving raw observations')
            print(json.dumps({'completed_batch': batch_index - 1,
                              'total_batches': len(batches),
                              'test_totals': row['summary_test_totals']}), flush=True)

        if sorted(all_actual_targets) != sorted(expected_ids) or len(all_actual_targets) != len(set(all_actual_targets)):
            raise RuntimeError('actual executed target set is not exact and unique')
        if sum(item[0] for item in all_summaries) <= 0:
            raise RuntimeError('workspace had no passing tests')
        passing_names = {item['name'] for row in record['commands']
                         for item in row.get('named_test_outcomes', []) if item['status'] == 'ok'}
        missing_required_tests = REQUIRED_NAMED_TESTS - passing_names
        if missing_required_tests:
            raise RuntimeError(f'required constructor tests did not pass by exact name: {sorted(missing_required_tests)}')
        record['required_named_tests'] = sorted(REQUIRED_NAMED_TESTS)
        record['required_named_tests_passed'] = sorted(REQUIRED_NAMED_TESTS & passing_names)
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
        record['observed_command_count_including_clean'] = len(record['commands'])
        if (record.get('status') == 'passed'
                and record.get('observed_command_count_including_clean')
                != record.get('expected_command_count_including_clean')):
            record['status'] = 'failed'
            record.setdefault('error', 'observed command count differs from computed format/metadata/batch/clean plan')
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
