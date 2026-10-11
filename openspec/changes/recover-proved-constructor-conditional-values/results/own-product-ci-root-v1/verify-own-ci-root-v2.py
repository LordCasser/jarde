#!/usr/bin/env python3
"""Verify for-condition product CI against its complete local workspace and CLI receipts."""
from __future__ import annotations

import argparse
import ast
import gzip
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import stat
import sys

sys.dont_write_bytecode = True
ROOT = Path('/Users/lordcasser/workspace/projects/jarde').resolve()
HELPER_PATH = ROOT / 'openspec/changes/recover-proved-conditional-switch-fallthrough/results/verify-conditional-ci-root-v5.py'
HELPER_SHA256 = 'bea37c7d4d37182168a7c1cbbde3ee31c94d61f6bc32667bedbdd02ce7e5b0ca'
CAPTURE_COLLECTOR_PATH = ROOT / 'openspec/changes/recover-proved-conditional-switch-fallthrough/results/capture-conditional-ci-root-v3.py'
CAPTURE_COLLECTOR_SHA256 = 'ed889bf6af08247efcce3c889cd1c790d6829d0ea12e9c0d053bdef801994be3'
WORKSPACE_RUNNER = Path('/private/tmp/jarde-constructor-conditional-workspace-root-v1.py')
WORKSPACE_RUNNER_SHA256 = '81484f984cdf909980b7dab25ee38ac560de0a1fa6028dc912b82ca05e0f0606'
BUILD_RUNNER = Path('/private/tmp/jarde-constructor-conditional-final-lint-build-root-v1.py')
BUILD_RUNNER_SHA256 = '7bf3a61aec009aaa629efad18fe8e177b25d0e3b828de135d52eba498ea5c2c3'
FOCUSED_RECEIPT = Path('/private/tmp/jarde-constructor-conditional-focused-build-root-v6/execution.json')
FOCUSED_RUNNER = Path('/private/tmp/jarde-constructor-conditional-focused-build-root-v6.py')
FOCUSED_RUNNER_SHA256 = '5895c7024985d737ad7fa9b8f8bbd8c55371d87d5bf1c46d02458b6d38df574b'
EXPECTED_WORKSPACE = Path('/private/tmp/jarde-constructor-conditional-workspace-root-v1')
EXPECTED_BUILD_RECORD = Path('/private/tmp/jarde-constructor-conditional-final-lint-build-root-v1/execution.json')
EXPECTED_FINAL_CLI = Path('/private/tmp/jarde-constructor-conditional-final-cli-root-v1')
WORKSPACE_SCHEMA = 'constructor-conditional-full-workspace-batched-root-v1'
BUILD_SCHEMA = 'constructor-conditional-final-lint-build-root-v1'
WORKSPACE_SEED = '5350648285461741569'
CI_SEEDS_RE = re.compile(r'PROPTEST_RNG_SEED:\s*"(\d+)"')
WORKSPACE_CMD = 'cargo test --workspace --all-targets --all-features --locked'
EXPECTED_JOBS = {'stable / test and specification', 'MSRV 1.88.0', 'supply chain', 'fuzz smoke'}
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

def load_ci_helper():
    need(HELPER_PATH.is_file() and sha_file(HELPER_PATH) == HELPER_SHA256,
         'basic CI parser helper path/SHA mismatch')
    return load_module('conditional_ci_basic_parser', HELPER_PATH)


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


def check_recorded_batch_observations(row: dict, observed: list[dict], ci, workspace: Path) -> None:
    """Cross-check normalized runner fields and every raw line against captured stdout."""
    expected_outcomes = [outcome for target in observed for outcome in target['outcomes']]
    raw_stdout = verify_stream(row['streams']['stdout'], workspace)
    cleaned_lines = [ci.clean(line).strip()
                     for line in raw_stdout.decode('utf-8', errors='replace').splitlines()]
    actual_outcome_lines = [line for line in cleaned_lines if ci.OUTCOME_RE.fullmatch(line)]
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
        match = ci.OUTCOME_RE.fullmatch(ci.clean(raw_line).strip())
        need(match is not None and match.group(1) == expected_name and
             ('ignored' if match.group(2).startswith('ignored') else match.group(2)) == expected_simple_state,
             f"batch {row.get('index')} named test raw line differs from its parsed identity/state")

    recorded_outcome_lines = [ci.clean(item['raw_line']).strip() for item in recorded_outcomes]
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
        match = ci.SUMMARY_RE.fullmatch(ci.clean(raw_line).strip())
        need(match is not None and tuple(map(int, match.groups()[1:4])) == counts and
             match.group(1) == summary['status'],
             f"batch {row.get('index')} summary raw line differs from its parsed counts/state")
    recorded_summary_lines = [ci.clean(item['raw_line']).strip() for item in summaries]
    need(recorded_summary_lines == actual_summary_lines,
         f"batch {row.get('index')} stored summaries omit/reorder/change raw stdout lines")
    totals = [sum(target['counts'][i] for target in observed) for i in range(3)]
    need(row.get('summary_test_totals') == totals,
         f"batch {row.get('index')} aggregate totals differ from raw target summaries")


def verify_workspace(product: str, workspace: Path, ci):
    workspace = workspace.resolve(strict=True)
    need(workspace.is_dir(), 'workspace receipt directory is invalid')
    execution_path = workspace / 'execution.json'
    metadata_path = workspace / 'metadata.json'
    execution_raw, metadata_raw = execution_path.read_bytes(), metadata_path.read_bytes()
    ex, metadata = json.loads(execution_raw), json.loads(metadata_raw)
    need(ex.get('schema') == WORKSPACE_SCHEMA and ex.get('status') == 'passed',
         'fresh constructor-conditional workspace receipt is incomplete or failed')
    need(ex.get('seed') == WORKSPACE_SEED and ex.get('no_carried_execution') is True,
         'workspace seed differs from the planned first CI seed or execution was carried')
    need(ex.get('rust_min_stack') is None, 'workspace receipt overrides RUST_MIN_STACK')
    runner_required = sorted(required_tests_from_runner())
    need(ex.get('required_named_tests') == runner_required and
         ex.get('required_named_tests_passed') == runner_required,
         'workspace runner did not record the exact complete required-test set as passed')
    need(ex.get('fresh_target_preflight', {}).get('target_absent') is True and
         ex.get('fresh_target_preflight', {}).get('free_bytes', 0) >= FREE_LIMIT,
         'workspace receipt lacks fresh-target and free-space preflight')
    need(ex.get('guard_limits') == {'minimum_free_bytes': FREE_LIMIT,
                                    'maximum_target_bytes': TARGET_LIMIT,
                                    'poll_interval_seconds': 1},
         'workspace resource guard limits differ from the recorded contract')
    expected_env = {**BASE_ENV, 'PROPTEST_RNG_SEED': WORKSPACE_SEED,
                    'CARGO_PROFILE_TEST_STRIP': 'symbols'}
    need(ex.get('environment_overrides') == expected_env,
         'workspace environment differs from the recorded seeded contract')

    runner = ex.get('runner', {})
    need(runner == {'path': str(WORKSPACE_RUNNER),
                    'sha256_before': WORKSPACE_RUNNER_SHA256,
                    'sha256_after': WORKSPACE_RUNNER_SHA256} and
         WORKSPACE_RUNNER.is_file() and sha_file(WORKSPACE_RUNNER) == WORKSPACE_RUNNER_SHA256,
         'workspace runner path or before/after source SHA is invalid')
    # Validate each recorded helper reference as provenance for the run machinery. These
    # references are not treated as product source-closure evidence.
    refs = ex.get('references', [])
    need(isinstance(refs, list) and refs, 'workspace receipt has no runner/helper references')
    for ref in refs:
        path = Path(ref.get('path', ''))
        need(path.is_file() and sha_file(path) == ref.get('sha256'),
             'workspace runner/helper reference path/SHA mismatch: ' + str(path))

    closure = source_paths()
    pins_before, pins_after = ex.get('source_pins_before'), ex.get('source_pins_after')
    need(isinstance(pins_before, dict) and pins_before == pins_after,
         'workspace source pins changed during the full run')
    closure_names = {path.relative_to(ROOT).as_posix() for path in closure}
    need(set(pins_before) == closure_names and len(pins_before) == len(closure),
         'workspace pins do not exactly equal the independently reconstructed source/include/fixture closure')
    need(ex.get('source_closure_count_at_start') == len(closure) == len(pins_before),
         'workspace starting closure count differs from the independent live closure')
    need(pin_sources(closure) == pins_before,
         'workspace pins differ from current live source bytes')
    product_pins = ci.blob_sha256s(product, pins_before)
    need(product_pins == pins_before,
         'workspace source closure differs from the selected product Git commit')
    need('tests/fixtures/corpus-fingerprint.json' in pins_before,
         'workspace closure omits the corpus fingerprint')
    dynamic_paths = set(ex.get('required_dynamic_source_paths', []))
    need({'tests/recover_proved_conditional_constructor_values.rs',
          'tests/fixtures/java8/cf11-conditional-constructor-values/ConditionalConstructionPositive.class',
          'tests/fixtures/java8/cf11-conditional-constructor-values/ConditionalConstructionRefusals.class',
          'tests/fixtures/java8/cf11-conditional-constructor-values/UpstreamSequentialLoops2.class'} <= dynamic_paths <= set(pins_before),
         'workspace required dynamic constructor sources are absent from its independent closure')

    by_id, by_named, inventory = ci.metadata_targets(metadata)
    target_count = len(inventory)
    declared = [tuple(row) for row in ex.get('target_inventory', [])]
    manifest_inventory = [tuple(row) for row in ex.get('static_manifest_target_inventory', [])]
    need(len(declared) == len(set(declared)) and set(declared) == set(inventory),
         'workspace target inventory differs from cargo metadata')
    need(len(manifest_inventory) == len(set(manifest_inventory)) and set(manifest_inventory) == set(inventory),
         'static manifest inventory differs from cargo metadata')
    planned, expected_inventory = expected_workspace_commands(metadata)
    need(len(expected_inventory) == target_count and len(planned) == 95 and target_count == 361,
         'actual cargo metadata does not independently derive 361 targets and 95 batches')
    need(ex.get('metadata', {}).get('target_count') == target_count and
         ex.get('metadata', {}).get('batch_count') == len(planned) and
         ex.get('expected_target_count') == target_count and
         ex.get('expected_batch_count') == len(planned) and
         ex.get('expected_command_count_including_clean') == len(planned) + 3,
         'workspace recorded metadata/batch/command counts differ from derived plan')

    commands = ex.get('commands', [])
    need(len(commands) == len(planned) + 3 and
         [row.get('index') for row in commands] == list(range(len(commands))),
         'workspace receipt does not contain format, metadata, every derived batch, and cleanup in order')
    workspace_env = expected_env
    check_command(commands[0], 0, ['cargo', 'fmt', '--all', '--', '--check'], workspace_env, workspace)
    need(ex.get('format_check') == {'exit_code': 0, 'guard_stop': None},
         'workspace format check did not pass')
    check_command(commands[1], 1,
                   ['cargo', 'metadata', '--no-deps', '--format-version', '1', '--locked'],
                   workspace_env, workspace)
    need(verify_stream(commands[1]['streams']['stdout'], workspace) == metadata_raw,
         'saved cargo metadata differs from its captured stdout')
    observations = []
    for offset, (row, argv) in enumerate(zip(commands[2:-1], planned), start=2):
        check_command(row, offset, argv, workspace_env, workspace)
        observed = ci.local_command_observation(row, by_id, by_named)
        ci.check_workspace_argv(row.get('argv', []), observed, by_named)
        need(len(observed) == len(row.get('observed_counts', [])) == len(row.get('executed_targets', [])),
             f'workspace batch {offset} observed named outcomes differ from its raw record')
        need(row.get('exit_code') == 0 and row.get('guard_stop') is None,
             f'workspace batch {offset} failed or hit a guard')
        check_recorded_batch_observations(row, observed, ci, workspace)
        observations.extend(observed)
    keys = [item['key'] for item in observations]
    need(len(keys) == len(set(keys)) and set(keys) == set(inventory),
         'workspace raw observations do not cover each metadata target exactly once')
    totals = [sum(item['counts'][i] for item in observations) for i in range(3)]
    need(totals[0] > 0 and totals[1] == 0 and ex.get('workspace_summary_count') == target_count and
         ex.get('workspace_totals') == totals,
         'workspace totals/count differ from reconstructed named target outcomes')
    clean_index = len(commands) - 1
    check_command(commands[clean_index], clean_index, ['cargo', 'clean'], workspace_env, workspace)
    cleanup = ex.get('cleanup', {})
    need(cleanup.get('attempted') is True and cleanup.get('argv') == ['cargo', 'clean'] and
         cleanup.get('guarded') is True and cleanup.get('status') == 'passed' and
         cleanup.get('command') == commands[clean_index] and cleanup.get('target_absent_after') is True and
         cleanup.get('pins_unchanged') is True and cleanup.get('source_pins_after') == pins_before and
         not (ROOT / 'target').exists(),
         'workspace guarded cleanup did not remove target or preserve source pins')
    local = {'execution': str(execution_path), 'execution_sha256': sha(execution_raw),
             'metadata': str(metadata_path), 'metadata_sha256': sha(metadata_raw),
             'seed': ex['seed'], 'target_count': target_count, 'batch_count': len(planned),
             'workspace_totals': totals, 'source_pin_count': len(pins_before),
             'source_closure_count_at_start': ex['source_closure_count_at_start'],
             'target_observations': observations}
    return ex, metadata, local


def required_tests_from_runner() -> set[str]:
    need(WORKSPACE_RUNNER.is_file() and sha_file(WORKSPACE_RUNNER) == WORKSPACE_RUNNER_SHA256,
         'pinned workspace runner unavailable while reading exact required test names')
    tree = ast.parse(WORKSPACE_RUNNER.read_text(encoding='utf-8'), filename=str(WORKSPACE_RUNNER))
    value = None
    for node in tree.body:
        if isinstance(node, ast.Assign) and any(isinstance(t, ast.Name) and t.id == 'REQUIRED_NAMED_TESTS'
                                                for t in node.targets):
            value = ast.literal_eval(node.value)
            break
    need(isinstance(value, set) and len(value) == 12 and all(isinstance(name, str) for name in value),
         'current workspace runner must declare the exact 12 required constructor outcomes')
    return value


def verify_required_tests(local_observations: list[dict], product: str,
                          pins: dict[str, str], metadata: dict, ci) -> dict:
    required = required_tests_from_runner()
    # Current integration outcomes are tied to their direct test target. The seven
    # implementation unit outcomes are emitted by the jarde-java lib test target.
    expected_sources = {}
    for name in required:
        if name.startswith('field::') or name.startswith('init::'):
            expected_sources[name] = 'crates/jarde-java/src/lib.rs'
        else:
            expected_sources[name] = 'tests/recover_proved_conditional_constructor_values.rs'
    source_targets = {}
    for row in local_observations:
        source_targets.setdefault(row['source_path'], []).append(row)
    _, by_named, _ = ci.metadata_targets(metadata)
    passed = []
    for name in sorted(required):
        source = expected_sources[name]
        need(source in pins, 'workspace pins omit the source for required outcome: ' + name)
        product_bytes = ci.git_blob(product, source)
        digest = sha(product_bytes)
        need(digest == pins[source] == sha_file(ROOT / source),
             'required outcome source differs from product Git/live closure: ' + source)
        rows = source_targets.get(source, [])
        need(len(rows) == 1, 'required source does not map to exactly one executed Cargo target: ' + source)
        target = rows[0]
        metadata_target = by_named.get(target['key'])
        need(metadata_target is not None and metadata_target['source_path'] == source,
             'required target metadata source differs from its pinned Rust source')
        matches = [(actual, state) for actual, state in target['outcomes']
                   if actual == name or actual.endswith('::' + name)]
        need(len(matches) == 1 and matches[0][1] == 'ok',
             'required named local outcome did not pass exactly once: ' + source + '::' + name)
        passed.append({'name': name, 'source_path': source,
                       'target': target['key'], 'actual_outcome': matches[0][0], 'status': 'ok'})
    need({item['name'] for item in passed} == required,
         'not all current workspace-runner required constructor tests were checked')
    return {'required_count': len(required), 'tests_from_pinned_workspace_runner': sorted(required),
            'passed_outcomes': passed}


def workflow_bytes(product: str, ci, pins: dict[str, str]) -> bytes:
    rel = '.github/workflows/ci.yml'
    raw = ci.git_blob(product, rel)
    digest = sha(raw)
    need(digest == pins.get(rel) == sha_file(ROOT / rel),
         'workflow source differs from selected product Git/live source closure')
    return raw


def workflow_clippy_argv(product: str, ci, pins: dict[str, str]) -> list[str]:
    text = workflow_bytes(product, ci, pins).decode('utf-8')
    start = text.index('          cargo clippy')
    end = text.index('\n\n', start)
    return text[start:end].split()


def verify_final_build(product: str, workspace_pins: dict[str, str], build_record: Path,
                       final_cli_path: Path, ci) -> dict:
    need(BUILD_RUNNER.is_file() and sha_file(BUILD_RUNNER) == BUILD_RUNNER_SHA256,
         'final build runner path/SHA mismatch')
    need(FOCUSED_RECEIPT.is_file(), 'accepted focused build receipt is missing')
    focused_raw = FOCUSED_RECEIPT.read_bytes()
    focused = json.loads(focused_raw)
    need(FOCUSED_RUNNER.is_file() and sha_file(FOCUSED_RUNNER) == FOCUSED_RUNNER_SHA256 and
         focused.get('schema') == 'constructor-conditional-focused-build-root-v6' and
         focused.get('status') == 'passed_candidate_build_only' and
         focused.get('target_absent') is True and
         focused.get('runner_sha256_before') == FOCUSED_RUNNER_SHA256 == focused.get('runner_sha256_after'),
         'focused build receipt/runner is not the accepted clean candidate build')
    focused_pins = focused.get('source_pins_before')
    need(focused_pins == focused.get('source_pins_after') == workspace_pins,
         'focused build source closure differs from the fresh workspace receipt')
    focused_cli = focused.get('candidate_cli', {})
    focused_cli_path = Path(focused_cli.get('path', ''))
    need(focused_cli_path.is_file() and not focused_cli_path.is_symlink() and
         focused_cli_path.stat().st_size == focused_cli.get('bytes') and
         sha_file(focused_cli_path) == focused_cli.get('sha256') and
         focused_cli.get('mode') == '0o555' and
         stat.S_IMODE(focused_cli_path.stat().st_mode) == 0o555,
         'focused candidate CLI file does not match its frozen receipt')

    raw = build_record.read_bytes()
    build = json.loads(raw)
    need(build.get('schema') == BUILD_SCHEMA and build.get('status') == 'passed_candidate_build_only',
         'final build receipt is incomplete, wrong-schema, or not passing')
    need(build.get('focused_passed') is True and
         build.get('final_lint_strict_byte_identity_passed') is True and
         build.get('candidate_byte_identity') is True,
         'final build receipt lacks passing focused, lint/strict, and byte-identity gates')
    need(build.get('runner_sha256_before') == BUILD_RUNNER_SHA256 ==
         build.get('runner_sha256_after') and sha_file(BUILD_RUNNER) == BUILD_RUNNER_SHA256,
         'final build runner before/after SHA mismatch')
    need(build.get('source_pins_before') == workspace_pins == build.get('source_pins_after'),
         'final build source pins differ from the full workspace closure')
    need(ci.blob_sha256s(product, workspace_pins) == workspace_pins and
         pin_sources(source_paths()) == workspace_pins,
         'final build source closure differs from product Git blobs or live source bytes')
    need(build.get('target_absent') is True and not (ROOT / 'target').exists(),
         'final build did not leave the Cargo target directory absent')
    expected_env = {**BASE_ENV, 'PROPTEST_RNG_SEED': WORKSPACE_SEED}
    need(build.get('actual_env_overrides') == expected_env and build.get('rust_min_stack') is None,
         'final build environment or default stack setting differs')
    accepted = build.get('accepted_focused_receipt', {})
    need(accepted == {'path': str(FOCUSED_RECEIPT), 'sha256': sha(focused_raw)},
         'final build does not identify the exact accepted focused build receipt')

    guard = ROOT / 'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
    scan = ROOT / 'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
    need(sha_file(guard) == ci.GUARD_SHA and sha_file(scan) == ci.SCAN_SHA,
         'final build guard/scanner source changed')
    expected_argvs = [
        ['cargo', 'fmt', '--all', '--', '--check'],
        workflow_clippy_argv(product, ci, workspace_pins),
        ['openspec', 'validate', '--all', '--strict', '--no-interactive'],
        ['cargo', 'build', '-p', 'jarde-cli', '--bin', 'jarde-cli', '--locked'],
        ['cargo', 'clean'],
    ]
    commands = build.get('commands', [])
    need(len(commands) == len(expected_argvs),
         'final build receipt lacks its format, clippy, strict validation, CLI build and clean commands')
    build_dir = build_record.parent.resolve(strict=True)
    for index, (row, argv) in enumerate(zip(commands, expected_argvs)):
        check_command(row, index, argv, BASE_ENV, build_dir)
    cli_row = build.get('candidate_cli')
    focused_cli = focused.get('candidate_cli', {})
    final_cli_path = final_cli_path.resolve(strict=True)
    need(isinstance(cli_row, dict) and cli_row.get('path') == str(final_cli_path) and
         cli_row.get('sha256') == focused_cli.get('sha256') and cli_row.get('mode') == '0o555',
         'final CLI path/hash/mode do not bind to the focused candidate bytes')
    need(final_cli_path.is_file() and not final_cli_path.is_symlink() and
         final_cli_path.stat().st_size == cli_row.get('bytes') and
         sha_file(final_cli_path) == focused_cli.get('sha256') and
         stat.S_IMODE(final_cli_path.stat().st_mode) == 0o555 and
         focused_cli.get('mode') == '0o555',
         'final CLI file bytes, size, or mode differ from the focused candidate')
    return {'record_path': str(build_record.resolve()), 'record_sha256': sha(raw),
            'runner_path': str(BUILD_RUNNER), 'runner_sha256': BUILD_RUNNER_SHA256,
            'commands': len(commands), 'target_absent_after_clean': True,
            'focused_receipt_path': str(FOCUSED_RECEIPT),
            'focused_receipt_sha256': sha(focused_raw),
            'candidate_cli': {'path': str(final_cli_path), 'bytes': cli_row['bytes'],
                              'sha256': cli_row['sha256'], 'mode': cli_row['mode'],
                              'byte_identical_to_focused_candidate': True}}


def verify_capture_record(capture_dir: Path, product: str, run_id: str, ci):
    need(not capture_dir.is_symlink(), 'CI capture directory must not be a symlink')
    capture_dir = capture_dir.resolve(strict=True)
    path = capture_dir / 'capture-execution-root-v2.json'
    need(not path.is_symlink(), 'CI capture receipt must not be a symlink')
    raw = path.read_bytes()
    doc = json.loads(raw)
    need(doc.get('schema') == 'recover-proved-conditional-switch-fallthrough-ci-capture-root-v2' and
         doc.get('status') == 'captured', 'CI capture receipt is incomplete or has an unexpected schema')
    need(doc.get('product_commit') == product and doc.get('run_id') == int(run_id),
         'captured CI evidence belongs to a different product commit or run')
    collector = doc.get('collector', {})
    collector_path = Path(collector.get('path', ''))
    need(collector_path.resolve() == CAPTURE_COLLECTOR_PATH.resolve() and
         collector_path.is_file() and sha_file(collector_path) == CAPTURE_COLLECTOR_SHA256 == collector.get('sha256') and
         sha(ci.git_blob(product, str(collector_path.relative_to(ROOT)))) == CAPTURE_COLLECTOR_SHA256,
         'CI capture collector does not match its live/product source pin')
    summary = doc.get('validated_run_summary', {})
    need(summary.get('headSha') == product and summary.get('status') == 'completed' and
         summary.get('conclusion') == 'success', 'CI summary head/status/conclusion mismatch')
    run_match = re.search(r'/actions/runs/(\d+)(?:$|[/?#])', summary.get('url', ''))
    need(run_match and int(run_match.group(1)) == int(run_id), 'CI summary URL does not identify the requested run')
    jobs = summary.get('jobs', [])
    need(len(jobs) == len(EXPECTED_JOBS) and {job.get('name') for job in jobs} == EXPECTED_JOBS,
         'CI job inventory does not include stable, MSRV, supply-chain and fuzz jobs exactly once')
    need(all(job.get('status') == 'completed' and job.get('conclusion') == 'success' and
             job.get('steps') and all(step.get('status') == 'completed' and step.get('conclusion') == 'success'
                                      for step in job['steps']) for job in jobs),
         'a CI job or step is failed, skipped, or incomplete')
    step_count = sum(len(job['steps']) for job in jobs)
    need(step_count > 0 and doc.get('jobs') == len(jobs) and doc.get('steps') == step_count,
         'CI capture job/step totals differ from the complete run summary')
    commands = doc.get('commands', [])
    need(len(commands) == 3, 'CI capture must bind summary, stable-job log, and supply-chain log commands')
    ids = {job['name']: str(job['databaseId']) for job in jobs}
    expected_argv = [
        ['gh', 'run', 'view', str(run_id), '--json', 'headSha,status,conclusion,jobs,url'],
        ['gh', 'run', 'view', str(run_id), '--job', ids['stable / test and specification'], '--log'],
        ['gh', 'run', 'view', str(run_id), '--job', ids['supply chain'], '--log'],
    ]
    stable_raw = supply_raw = None
    for index, (row, argv) in enumerate(zip(commands, expected_argv)):
        need(row.get('argv') == argv and row.get('cwd') == str(ROOT) and row.get('exit_code') == 0 and
             not row.get('timed_out') and row.get('spawn_error') is None,
             f'CI capture command {index} argv/cwd/status mismatch')
        env = row.get('environment_override', {})
        need(set(env) == {'NO_PROXY', 'no_proxy'} and
             all(item.get('appended') == ['results-receiver.actions.githubusercontent.com',
                                          'productionresultssa1.blob.core.windows.net'] and
                 re.fullmatch(r'[0-9a-f]{64}', item.get('final_value_sha256', ''))
                 for item in env.values()), 'CI capture proxy record is malformed')
        stdout = verify_stream(row['stdout'], capture_dir)
        verify_stream(row['stderr'], capture_dir)
        if index == 0:
            need(json.loads(stdout) == summary, 'captured summary stdout differs from API summary')
        else:
            filename = 'ci-stable-job-v2.log.gz' if index == 1 else 'ci-supply-job-v2.log.gz'
            packed_path = capture_dir / filename
            need(not packed_path.is_symlink(), 'captured compressed CI log must not be a symlink')
            packed = packed_path.read_bytes()
            rec = row['stdout']
            need(len(packed) == rec.get('bytes') and sha(packed) == rec.get('sha256'),
                 'captured compressed CI log length/SHA mismatch')
            decoded = gzip.decompress(packed)
            need(len(decoded) == rec.get('uncompressed_bytes') and sha(decoded) == rec.get('raw_sha256') and
                 gzip.compress(decoded, mtime=0) == packed,
                 'captured compressed CI log does not reproduce canonical raw bytes')
            raw_ref = row.get('stdout_raw', {})
            raw_path = capture_dir / raw_ref.get('path', '')
            need(not raw_path.is_symlink() and raw_path.resolve().parent == capture_dir and raw_path.read_bytes() == decoded and
                 raw_ref.get('bytes') == len(decoded) and raw_ref.get('sha256') == sha(decoded),
                 'captured decompressed CI log differs from its pinned raw stream')
            if index == 1:
                stable_raw = decoded
            else:
                supply_raw = decoded
    return doc, summary, step_count, stable_raw, supply_raw, sha(raw)


def bind_ci_name(raw_name: str, status: str, target: dict) -> str:
    if '***' not in raw_name:
        return raw_name
    pieces = raw_name.split('***')
    pattern = '^' + '[^\\n]*'.join(re.escape(part) for part in pieces) + '$'
    choices = [name for name, actual_status in target['outcomes']
               if actual_status == status and re.fullmatch(pattern, name)]
    need(len(choices) == 1,
         'masked CI test label does not bind to exactly one same-status local outcome: ' + repr(raw_name))
    return choices[0]


def parse_ci_workspace(group: str, seed: str, targets: list[dict], ci) -> dict:
    need(seed in group, 'CI workspace group does not contain its workflow seed')
    headers = []
    for line in group.splitlines():
        if 'Running ' in ci.clean(line):
            item = ci.parse_ci_header(line, targets)
            if item is not None:
                headers.append(item)
    blocks = ci.stdout_blocks_cargo_json(group.encode())
    need(len(headers) == len(targets) == len(blocks),
         'CI group does not report one header and JSON result block per local target')
    keys = [row['key'] for row in headers]
    need(len(keys) == len(set(keys)) and set(keys) == {row['key'] for row in targets},
         'CI target headers omit, duplicate, or introduce a workspace target')
    local = {row['key']: row for row in targets}
    observed = []
    redactions = []
    for header, block in zip(headers, blocks):
        expected = local[header['key']]
        bound = []
        for raw_name, status in block['outcomes']:
            name = bind_ci_name(raw_name, status, expected)
            bound.append((name, status))
            if name != raw_name:
                redactions.append({'target': list(expected['key']), 'raw_label': raw_name,
                                   'bound_name': name, 'status': status})
        need(len(set(name for name, _ in bound)) == len(bound),
             'CI label binding introduces duplicate test names')
        need(block['counts'] == expected['counts'] and tuple(bound) == expected['outcomes'],
             'CI named outcomes and counts differ from local target: ' + repr(expected['key']))
        observed.append({'key': list(expected['key']), 'counts': list(block['counts']),
                         'outcomes': [{'name': name, 'status': status} for name, status in bound]})
    totals = [sum(row['counts'][i] for row in observed) for i in range(3)]
    return {'seed': seed, 'target_count': len(observed), 'workspace_totals': totals,
            'target_observations': observed, 'masked_public_label_mappings': redactions}


def verify_product_ci(capture_dir: Path, product: str, run_id: str,
                      local_observations: list[dict], product_pins: dict[str, str], ci) -> dict:
    capture, summary, steps, stable_raw, supply_raw, capture_sha = verify_capture_record(
        capture_dir, product, run_id, ci)
    workflow = workflow_bytes(product, ci, product_pins).decode('utf-8')
    seeds = ci.CI_SEEDS_RE.findall(workflow)
    need(len(seeds) == 2 and workflow.count('run: ' + ci.WORKSPACE_CMD) == 2,
         'current product workflow does not define exactly two workspace seed runs')
    lines = ci.strip_log(stable_raw).splitlines()
    groups = ci.command_groups(lines, ci.WORKSPACE_CMD)
    need(len(groups) == 2, 'stable job log does not contain both complete workspace command groups')
    seed_runs = []
    unused = list(groups)
    for seed in seeds:
        matches = [group for group in unused if seed in group]
        need(len(matches) == 1, 'cannot uniquely bind a CI workspace group to seed ' + seed)
        group = matches[0]
        unused.remove(group)
        seed_runs.append(parse_ci_workspace(group, seed, local_observations, ci))
    need(not unused, 'stable job contains an unbound workspace command group')
    local_totals = [sum(row['counts'][i] for row in local_observations) for i in range(3)]
    for result in seed_runs:
        need(result['workspace_totals'] == local_totals and result['target_count'] == len(local_observations),
             'CI seed aggregate differs from actual local workspace totals/target count')
    need(seed_runs[0]['target_observations'] == seed_runs[1]['target_observations'],
         'the two CI seed runs differ in target outcomes')
    # The supply-chain job must have a complete successful job log in the captured evidence.
    need(supply_raw is not None and len(supply_raw) > 0,
         'captured supply-chain job log is absent')
    return {'capture_sha256': capture_sha, 'summary': summary, 'jobs': len(summary['jobs']),
            'steps': steps, 'seed_runs': seed_runs, 'local_workspace_totals': local_totals,
            'masked_public_label_mappings': [mapping for run in seed_runs
                                             for mapping in run['masked_public_label_mappings']]}


def verify_references(product: str, ci) -> dict:
    need(sha_file(HELPER_PATH) == HELPER_SHA256 and sha_file(CAPTURE_COLLECTOR_PATH) == CAPTURE_COLLECTOR_SHA256,
         'basic CI parser or capture collector source SHA mismatch')
    need(sha_file(WORKSPACE_RUNNER) == WORKSPACE_RUNNER_SHA256 and
         sha_file(BUILD_RUNNER) == BUILD_RUNNER_SHA256,
         'current workspace/final-build runner source SHA mismatch')
    git_bound = {}
    for path, expected in ((HELPER_PATH, HELPER_SHA256),
                           (CAPTURE_COLLECTOR_PATH, CAPTURE_COLLECTOR_SHA256),
                           (ci.GUARD, ci.GUARD_SHA), (ci.SCAN, ci.SCAN_SHA)):
        rel = path.resolve().relative_to(ROOT).as_posix()
        live = sha_file(path)
        need(live == expected and sha(ci.git_blob(product, rel)) == live,
             'repository helper differs from selected product Git/live bytes: ' + rel)
        git_bound[rel] = live
    for p in (WORKSPACE_RUNNER, BUILD_RUNNER, FOCUSED_RUNNER, FOCUSED_RECEIPT):
        need(p.is_file(), 'private run machinery/receipt is missing: ' + str(p))
    return {'basic_ci_parser': {'path': str(HELPER_PATH), 'sha256': HELPER_SHA256},
            'capture_collector': {'path': str(CAPTURE_COLLECTOR_PATH), 'sha256': CAPTURE_COLLECTOR_SHA256},
            'workspace_runner': {'path': str(WORKSPACE_RUNNER), 'sha256': WORKSPACE_RUNNER_SHA256},
            'final_build_runner': {'path': str(BUILD_RUNNER), 'sha256': BUILD_RUNNER_SHA256},
            'focused_build_runner': {'path': str(FOCUSED_RUNNER), 'sha256': FOCUSED_RUNNER_SHA256},
            'product_git_live_bound_helpers': git_bound,
            'focused_receipt': {'path': str(FOCUSED_RECEIPT), 'sha256': sha_file(FOCUSED_RECEIPT)}}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--product-commit', required=True)
    parser.add_argument('--run-id', required=True)
    parser.add_argument('--capture', type=Path, required=True)
    parser.add_argument('--workspace', type=Path, default=EXPECTED_WORKSPACE)
    parser.add_argument('--final-build', type=Path, default=EXPECTED_BUILD_RECORD)
    parser.add_argument('--final-cli', type=Path, default=EXPECTED_FINAL_CLI)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    need(re.fullmatch(r'[0-9a-f]{40}', args.product_commit) is not None,
         'product commit must be a lowercase 40-hex SHA')
    need(args.run_id.isdecimal(), 'CI run id must be decimal')
    out = args.out.resolve()
    need(not out.exists(), 'refusing to overwrite a previous acceptance result: ' + str(out))
    ci = load_ci_helper()
    refs = verify_references(args.product_commit, ci)
    ex, metadata, local = verify_workspace(args.product_commit, args.workspace, ci)
    required = verify_required_tests(local['target_observations'], args.product_commit,
                                     ex['source_pins_before'], metadata, ci)
    build = verify_final_build(args.product_commit, ex['source_pins_before'],
                               args.final_build.resolve(strict=True), args.final_cli, ci)
    ci_result = verify_product_ci(args.capture.resolve(strict=True), args.product_commit,
                                  args.run_id, local['target_observations'],
                                  ex['source_pins_before'], ci)
    acceptance = {
        'schema': 'recover-proved-conditional-constructor-values-own-ci-acceptance-root-v1',
        'status': 'accepted', 'product_commit': args.product_commit, 'run_id': int(args.run_id),
        'ci': {'jobs': ci_result['jobs'], 'steps': ci_result['steps'],
               'seed_runs': ci_result['seed_runs'], 'run_summary': ci_result['summary'],
               'aggregate_counts_predeclared': False,
               'masked_public_label_mappings': ci_result['masked_public_label_mappings']},
        'fresh_local_workspace': {key: value for key, value in local.items()
                                  if key != 'target_observations'},
        'workspace_runner_contract': {'path': ex['runner']['path'],
                                      'sha256': ex['runner']['sha256_before'],
                                      'batch_count_from_metadata': local['batch_count'],
                                      'target_count_from_metadata': local['target_count'],
                                      'source_closure_count_at_start': local['source_closure_count_at_start']},
        'required_tests': required, 'final_cli_build': build,
        'all_local_and_both_ci_seed_named_outcomes_compared': True,
        'references': refs,
        'evidence': {'capture': str(args.capture.resolve()),
                     'capture_execution_sha256': ci_result['capture_sha256'],
                     'workspace_execution': local['execution'],
                     'workspace_execution_sha256': local['execution_sha256'],
                     'workspace_metadata': local['metadata'],
                     'workspace_metadata_sha256': local['metadata_sha256'],
                     'final_build_record': build['record_path'],
                     'final_build_record_sha256': build['record_sha256'],
                     'final_cli': build['candidate_cli']},
        'acceptance_scope': ('Exact selected product commit and CI run. The independently reconstructed source/include/fixture closure is compared with the workspace starting count, before/after pins, product Git blobs, and live bytes. Cargo metadata independently derives the full target and batch inventory. All 12 named constructor outcomes from the pinned current workspace runner must pass, and every named outcome and count in each of both CI workspace seed runs is compared with the local observed target results. All four CI jobs and every recorded step must complete successfully. The final CLI is byte-identical to the focused candidate build and bound by the clean final format/clippy/strict-validation/build receipt.')
    }
    out.parent.mkdir(parents=True, exist_ok=True)
    with out.open('x', encoding='utf-8') as stream:
        stream.write(json.dumps(acceptance, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({'status': 'accepted', 'product_commit': args.product_commit,
                      'run_id': int(args.run_id), 'jobs': ci_result['jobs'],
                      'steps': ci_result['steps'], 'target_count': local['target_count'],
                      'batch_count': local['batch_count'],
                      'source_closure_count_at_start': local['source_closure_count_at_start'],
                      'workspace_totals': [run['workspace_totals'] for run in ci_result['seed_runs']],
                      'required_tests': required['required_count'],
                      'masked_public_labels': len(ci_result['masked_public_label_mappings'])},
                     ensure_ascii=False))
    return 0


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except Exception as error:
        print('constructor-conditional own-CI verification failed: ' + str(error), file=sys.stderr)
        raise
