#!/usr/bin/env python3
"""Guarded, minimal validation runner for project-integer-constant-names-with-refused-member-family.

This runner is intentionally not self-starting: root supplies the settled source base,
private evidence directory, frozen CLI destination, and exact new Rust test fullname.
"""
from __future__ import annotations
import argparse
import datetime
import hashlib
import importlib.util
import json
import os
import re
import shutil
import signal
import stat
import subprocess
import sys
import time
from pathlib import Path

sys.dont_write_bytecode = True

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
CONDITIONAL_RUNNER = ROOT / 'openspec/changes/recover-proved-conditional-switch-fallthrough/results/run-validation-build-root-v2.py'
CONDITIONAL_RUNNER_SHA256 = '0a09602cebea718296c1ac55140c6ef6c9d85503de5dead1910f18abc4999587'
GUARD_TEMPLATE = ROOT / 'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
GUARD_TEMPLATE_SHA256 = '51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
SIZE_SCAN = ROOT / 'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
SIZE_SCAN_SHA256 = 'd954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
CI_PATH = '.github/workflows/ci.yml'

HARNESS_EXECUTION = ROOT / 'openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1/harness-v3-java/execution.json'
HARNESS_EXECUTION_SHA256 = '5ffa56c72b2df664d21cb7e9afa30c819cf4a5071cb009c9839e3ae17724f281'
OBSERVATION_INVENTORY = ROOT / 'openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1/observation-input-inventory-root-v1.json'
OBSERVATION_INVENTORY_SHA256 = 'd46f23c9f2bef614fee0d9f77efcfca561ebd5d986ad3b85d5ba2a70a609789e'
FIXTURES = {
    'openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1/harness-v3-java/capture/TestSwitchLabels.test/input/TestSwitchLabels$TestCls.class': {
        'bytes': 753, 'sha256': '4997ca33261d0585a1008dd860131c5ed3f2438206209bb7acb82c90eb743d22',
    },
    'openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1/harness-v3-java/capture/TestSwitchLabels.test/input/TestSwitchLabels$TestCls$Inner.class': {
        'bytes': 744, 'sha256': '2d6221bb75ba81cba428c32e3139038982c4078c80a1af99532c3a45acd54a99',
    },
}

# Pin only the facade, source-map path, CLI entry points, package metadata and exercised tests.
# Literal include_bytes!/include_str! inputs are enumerated dynamically below.
PRODUCT_PATHS = {
    'Cargo.toml', 'Cargo.lock', 'crates/jarde-cli/Cargo.toml',
    'crates/jarde-cli/src/main.rs', 'crates/jarde-cli/src/task.rs', 'crates/jarde-cli/src/export.rs',
    'src/facade.rs', 'src/class_source.rs', 'src/lib.rs', CI_PATH,
}
TEST_PATHS = {
    'tests/member_family_identity.rs',
    'tests/class_source.rs',
    'tests/p3_nested_annotation_source.rs',
    'tests/inner_class_static_mixed_folding.rs',
    'tests/member_class_static_folding.rs',
}
INCLUDE_RE = re.compile(r'(?:include_bytes|include_str)!\s*\(\s*"([^"]+)"')
SUMMARY_RE = re.compile(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored')
CORE_TEST_RE = re.compile(r'(?:[A-Za-z_][A-Za-z0-9_]*::)*integer_constant_name_tests::[A-Za-z_][A-Za-z0-9_]*\Z')

# Indices are assigned below: 0 source-base read, 1 fmt, 2 CI Clippy,
# 3 whole jarde library tests, 4 exact new core test, 5..9 focused API suites,
# 10 candidate CLI build, 11 target cleanup after the private CLI has been frozen.
COMMANDS: list[list[str]] = []
TEST_COMMANDS: set[int] = set(range(3, 10))
CORE_TEST_INDEX = 4


def sha_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def load_validation_modules():
    if sha_file(CONDITIONAL_RUNNER) != CONDITIONAL_RUNNER_SHA256:
        raise RuntimeError('pinned conditional validation runner SHA-256 mismatch')
    spec = importlib.util.spec_from_file_location('conditional_validation_v2', CONDITIONAL_RUNNER)
    if spec is None or spec.loader is None:
        raise RuntimeError('cannot load the committed conditional validation runner')
    conditional = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = conditional
    spec.loader.exec_module(conditional)
    if len(conditional.CI_CLIPPY_ALLOW_LINTS) != 29:
        raise RuntimeError('the imported exact CI Clippy allowance list is no longer 29 entries')
    guard = conditional.load_guard()
    if sha_file(GUARD_TEMPLATE) != GUARD_TEMPLATE_SHA256 or sha_file(SIZE_SCAN) != SIZE_SCAN_SHA256:
        raise RuntimeError('v9 guard or single-stat size scanner pin mismatch')
    return conditional, guard


def canonical_include_paths(paths: set[str]) -> set[str]:
    found: set[str] = set()
    pending = list(sorted(paths))
    scanned: set[str] = set()
    while pending:
        rel = pending.pop()
        if rel in scanned or not rel.endswith('.rs'):
            continue
        scanned.add(rel)
        text = (ROOT / rel).read_text(encoding='utf-8')
        for literal in INCLUDE_RE.findall(text):
            resolved = (ROOT / rel).parent.joinpath(literal).resolve()
            if not resolved.is_file() or not resolved.is_relative_to(ROOT.resolve()):
                raise RuntimeError(f'missing/out-of-root Rust literal include: {rel} -> {literal}')
            child_rel = resolved.relative_to(ROOT).as_posix()
            if child_rel not in found:
                found.add(child_rel)
                pending.append(child_rel)
    missing = set(FIXTURES) - found
    if missing:
        raise RuntimeError('both frozen root/Inner classes must be in the Rust literal include closure: ' + repr(sorted(missing)))
    return found


def verify_fixture_manifest() -> dict:
    if sha_file(HARNESS_EXECUTION) != HARNESS_EXECUTION_SHA256:
        raise RuntimeError('frozen upstream harness-v3 manifest SHA-256 mismatch')
    if sha_file(OBSERVATION_INVENTORY) != OBSERVATION_INVENTORY_SHA256:
        raise RuntimeError('frozen CF12 observation-input inventory SHA-256 mismatch')
    inventory = json.loads(OBSERVATION_INVENTORY.read_text(encoding='utf-8'))
    if inventory.get('schema') != 'cf12-closed-input-inventory-root-v1':
        raise RuntimeError('frozen observation inventory schema mismatch')
    rows = {row.get('path'): row for row in inventory.get('files', [])}
    fixture_rows = {}
    for rel, expected in FIXTURES.items():
        path = ROOT / rel
        if not path.is_file() or path.stat().st_size != expected['bytes'] or sha_file(path) != expected['sha256']:
            raise RuntimeError('frozen TestSwitchLabels fixture identity mismatch: ' + rel)
        evidence_rel = rel.split('/cf12-upstream-java-root-v1/', 1)[1]
        if rows.get(evidence_rel) != {'path': evidence_rel, **expected}:
            raise RuntimeError('fixture is not pinned by the accepted observation inventory: ' + evidence_rel)
        fixture_rows[rel] = {'path': str(path), **expected}
    return {
        'harness_execution': {'path': str(HARNESS_EXECUTION), 'sha256': HARNESS_EXECUTION_SHA256},
        'observation_inventory': {'path': str(OBSERVATION_INVENTORY), 'sha256': OBSERVATION_INVENTORY_SHA256},
        'fixtures': fixture_rows,
    }


def input_pins(guard, includes: set[str], fixture_manifest: dict) -> dict:
    return {
        'product_sources': guard.source_pins(PRODUCT_PATHS),
        'test_sources': guard.source_pins(TEST_PATHS),
        'src_facade_and_test_literal_include_closure': guard.source_pins(includes),
        'fixture_manifests': {
            'harness_execution': fixture_manifest['harness_execution']['sha256'],
            'observation_inventory': fixture_manifest['observation_inventory']['sha256'],
        },
        'actual_frozen_fixtures': {name: row['sha256'] for name, row in fixture_manifest['fixtures'].items()},
    }


def summarize_test(index: int, stdout: bytes, core_test_name: str) -> dict:
    text = stdout.decode('utf-8', errors='replace')
    actual = [tuple(map(int, match)) for match in SUMMARY_RE.findall(text)]
    nonempty_success = bool(actual) and all(failed == 0 for _, failed, _ in actual) and any(passed + ignored > 0 for passed, _, ignored in actual)
    exact_core_line = re.search(r'(?m)^test ' + re.escape(core_test_name) + r' \.\.\. ok$', text) is not None if index == CORE_TEST_INDEX else None
    ok = nonempty_success and (index != CORE_TEST_INDEX or exact_core_line is True)
    reason = None
    if not actual:
        reason = 'successful Cargo test emitted no test-result summary'
    elif any(failed for _, failed, _ in actual):
        reason = 'Cargo test summary reports failed tests'
    elif not any(passed + ignored > 0 for passed, _, ignored in actual):
        reason = 'Cargo test summary is empty'
    elif index == CORE_TEST_INDEX and not exact_core_line:
        reason = 'the caller-selected exact core test fullname did not print as passed'
    return {
        'expected_aggregate': None,
        'actual_summaries': [list(row) for row in actual],
        'required': index in TEST_COMMANDS,
        'exact_core_test_name': core_test_name if index == CORE_TEST_INDEX else None,
        'exact_core_test_passed_line': exact_core_line,
        'ok': ok,
        'failure_reason': reason,
    }


def write_execution(out: Path, payload: dict) -> None:
    temp = out / '.execution.json.tmp'
    temp.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    os.replace(temp, out / 'execution.json')


def prepare_guard(guard, out: Path, frozen_cli: Path, source_base: str, env_overrides: dict[str, str]):
    guard.ROOT = ROOT
    guard.HERE = out.parent
    guard.RESULTS = out.parent
    guard.OUT = out
    guard.CLI_PATH = frozen_cli
    guard.METADATA_PATH = out / 'frozen-cli.json'
    guard.SOURCE_COMMIT_BASE = source_base
    guard.ENV_VALUES = {**guard.ENV_VALUES, **env_overrides}
    # The inherited guard's size callback is from the pinned race-safe one-stat adapter.
    # Its default command_stream insists on workspace-relative paths; evidence lives in private tmp.
    guard.command_stream = lambda path: {
        'path': str(path.resolve()), 'bytes': path.stat().st_size, 'sha256': guard.sha_file(path),
    }
    guard.EXPECTED_SUMMARIES = {}
    guard.TEST_COMMANDS = TEST_COMMANDS
    guard.expected_test_summaries = lambda index, stdout: summarize_test(index, stdout, CORE_TEST_NAME)


def test_binary_path(guard, out: Path, index: int) -> dict:
    raw_paths = []
    for stream_name in ('stdout', 'stderr'):
        raw = (out / f'{index}.{stream_name}.raw').read_text(encoding='utf-8', errors='replace')
        raw = guard.ANSI_RE.sub('', raw)
        raw_paths.extend(match.group(1).strip() for line in raw.splitlines()
                         if (match := guard.RUNNING_BINARY_RE.match(line)))
    candidates = [Path(value) for value in raw_paths if 'target/debug/deps' in Path(value).as_posix()]
    if len(candidates) != 1:
        raise RuntimeError(f'successful test command must report exactly one completed target/debug/deps executable; got {raw_paths!r}')
    path = candidates[0]
    if not path.is_absolute():
        path = ROOT / path
    binary, info = guard.checked_deps_file(path, guard.target_deps(), executable=True)
    row = {'path': str(binary), 'size_bytes': info.st_size, 'sha256': guard.sha_file(binary), 'action': 'delete-this-completed-test-executable'}
    binary.unlink()
    return row


def build_commands(core_test_name: str, conditional) -> list[list[str]]:
    lints = conditional.CI_CLIPPY_ALLOW_LINTS
    clippy = ['cargo', 'clippy', '--workspace', '--all-targets', '--all-features', '--locked', '--']
    for lint in lints:
        clippy += ['-A', f'clippy::{lint}']
    clippy += ['-D', 'warnings']
    return [
        ['git', 'rev-parse', 'HEAD'],
        ['cargo', 'fmt', '--all', '--', '--check'],
        clippy,
        ['cargo', 'test', '-p', 'jarde', '--lib', '--locked'],
        ['cargo', 'test', '-p', 'jarde', '--lib', '--locked', '--', core_test_name, '--exact', '--nocapture'],
        ['cargo', 'test', '-p', 'jarde', '--test', 'member_family_identity', '--locked', '--', '--nocapture'],
        ['cargo', 'test', '-p', 'jarde', '--test', 'class_source', '--locked', '--', '--nocapture'],
        ['cargo', 'test', '-p', 'jarde', '--test', 'p3_nested_annotation_source', '--locked', '--', '--nocapture'],
        ['cargo', 'test', '-p', 'jarde', '--test', 'inner_class_static_mixed_folding', '--locked', '--', '--nocapture'],
        ['cargo', 'test', '-p', 'jarde', '--test', 'member_class_static_folding', '--locked', '--', '--nocapture'],
        ['cargo', 'build', '-p', 'jarde-cli', '--locked'],
        ['cargo', 'clean'],
    ]


def main() -> int:
    global CORE_TEST_NAME
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source-base', required=True, help='exact settled 40-character source-base commit SHA; root supplies after prior CI/cleanup closes')
    parser.add_argument('--out', required=True, help='fresh absolute private evidence directory')
    parser.add_argument('--frozen-cli', required=True, help='fresh absolute /private/tmp CLI destination, frozen mode 0555')
    parser.add_argument('--core-test-name', required=True, help='exact Rust test fullname, e.g. facade::integer_constant_name_tests::<new_test>')
    args = parser.parse_args()
    source_base = args.source_base.lower()
    out = Path(args.out).resolve()
    frozen_cli = Path(args.frozen_cli).resolve()
    CORE_TEST_NAME = args.core_test_name
    if not re.fullmatch(r'[0-9a-f]{40}', source_base):
        raise SystemExit('--source-base must be the exact 40-character settled base SHA')
    if not CORE_TEST_RE.fullmatch(CORE_TEST_NAME):
        raise SystemExit('--core-test-name must name exactly one integer_constant_name_tests unit test')
    if not str(out).startswith('/private/tmp/') or not str(frozen_cli).startswith('/private/tmp/'):
        raise SystemExit('--out and --frozen-cli must be absolute paths below /private/tmp')
    if out.exists() or frozen_cli.exists():
        raise SystemExit('refusing to overwrite prior evidence or frozen CLI')
    if not out.parent.is_dir() or not os.access(out.parent, os.W_OK):
        raise SystemExit('private output parent must already exist and be writable')
    if frozen_cli.parent != out.parent:
        raise SystemExit('--out and --frozen-cli must share one fresh private parent for atomic cleanup')

    conditional, guard = load_validation_modules()
    if sha_file(GUARD_TEMPLATE) != GUARD_TEMPLATE_SHA256 or len(conditional.CI_CLIPPY_ALLOW_LINTS) != 29:
        raise SystemExit('pinned guard or exact 29-entry CI Clippy source drifted')
    fixture_manifest = verify_fixture_manifest()
    includes = canonical_include_paths(PRODUCT_PATHS | TEST_PATHS)
    if len(TEST_PATHS) != 5:
        raise SystemExit('focused reader/family regression inventory changed; review this runner')
    commands = build_commands(CORE_TEST_NAME, conditional)
    if len(commands) != 12:
        raise SystemExit('expected command inventory changed; review before execution')
    prepare_guard(guard, out, frozen_cli, source_base, {'CARGO_PROFILE_TEST_STRIP': 'symbols'})
    if shutil.disk_usage(ROOT).free < guard.FREE_LIMIT or guard.target_bytes() > guard.TARGET_LIMIT:
        raise SystemExit('initial 5 GiB free / 1 GiB target guard failed')
    out.mkdir()
    pins_before = input_pins(guard, includes, fixture_manifest)
    payload = {
        'schema': 'project-integer-constant-names-with-refused-member-family-validation-root-v1',
        'status': 'running',
        'runner': {'path': str(Path(__file__).resolve()), 'sha256': sha_file(Path(__file__).resolve())},
        'imported_conditional_runner': {'path': str(CONDITIONAL_RUNNER), 'sha256': CONDITIONAL_RUNNER_SHA256,
                                        'load_guard_reused': True, 'cli_lint_allowlist_entries': len(conditional.CI_CLIPPY_ALLOW_LINTS)},
        'guard': {'path': str(GUARD_TEMPLATE), 'sha256': GUARD_TEMPLATE_SHA256,
                  'size_scanner': {'path': str(SIZE_SCAN), 'sha256': SIZE_SCAN_SHA256, 'mode': 'single-stat race-tolerant'},
                  'minimum_free_bytes': guard.FREE_LIMIT, 'maximum_target_bytes': guard.TARGET_LIMIT,
                  'poll_seconds': 1, 'process_group_termination': True},
        'source_base_expected': source_base,
        'output_directory': str(out), 'frozen_cli_destination': str(frozen_cli),
        'environment_overrides': {**guard.ENV_VALUES},
        'stripped_environment_keys': list(guard.STRIPPED_ENV_KEYS),
        'fixture_manifest': fixture_manifest,
        'source_pins_before': pins_before, 'source_pins_after': None,
        'commands': [], 'freeze': None, 'failure': None,
    }
    write_execution(out, payload)
    cli_created = False
    temp_cli = None
    try:
        clean_env = os.environ.copy()
        for key in guard.STRIPPED_ENV_KEYS:
            clean_env.pop(key, None)
        clean_env.update(guard.ENV_VALUES)
        for index, argv in enumerate(commands):
            row = guard.run_command(index, [str(arg) for arg in argv], clean_env.copy())
            payload['commands'].append(row)
            write_execution(out, payload)
            print(json.dumps(row, ensure_ascii=False), flush=True)
            if row['exit_code'] != 0 or row['guard_stop'] is not None:
                raise RuntimeError(f'command {index} failed or tripped a disk guard')
            if index == 0:
                actual = (out / '0.stdout.raw').read_text(encoding='ascii', errors='replace').strip()
                if actual != source_base:
                    raise RuntimeError('read-only git rev-parse HEAD differs from supplied --source-base')
            if index in TEST_COMMANDS:
                check = row.get('test_summary_check')
                if not check or not check.get('ok'):
                    raise RuntimeError(f'test command {index} has no valid successful summary: {check!r}')
                row['cleanup'] = {'completed_test_binary': test_binary_path(guard, out, index)}
                write_execution(out, payload)
            if index == 10:
                built = ROOT / 'target/debug/jarde-cli'
                if not built.is_file() or built.is_symlink():
                    raise RuntimeError('cargo build succeeded without a regular target/debug/jarde-cli')
                built_record = {'path': str(built), 'size_bytes': built.stat().st_size, 'sha256': sha_file(built)}
                temp_cli = frozen_cli.with_name('.' + frozen_cli.name + f'.tmp-{os.getpid()}')
                if temp_cli.exists() or frozen_cli.exists():
                    raise FileExistsError('refusing to overwrite frozen CLI or temporary destination')
                shutil.copyfile(built, temp_cli)
                os.chmod(temp_cli, 0o555)
                os.replace(temp_cli, frozen_cli)
                cli_created = True
                frozen_record = {'path': str(frozen_cli), 'size_bytes': frozen_cli.stat().st_size,
                                 'sha256': sha_file(frozen_cli), 'mode': oct(stat.S_IMODE(frozen_cli.stat().st_mode))}
                if frozen_record['mode'] != '0o555' or frozen_record['sha256'] != built_record['sha256']:
                    raise RuntimeError('frozen candidate CLI identity/mode differs from the built binary')
                payload['freeze'] = {'built_binary': built_record, 'frozen_cli': frozen_record,
                                     'source_base': source_base, 'product_source_pins': pins_before['product_sources']}
                write_execution(out, payload)
        pins_after = input_pins(guard, includes, fixture_manifest)
        payload['source_pins_after'] = pins_after
        if pins_after != pins_before:
            raise RuntimeError('product/test/include/fixture pins changed during the run')
        if not frozen_cli.is_file() or sha_file(frozen_cli) != payload['freeze']['frozen_cli']['sha256']:
            raise RuntimeError('frozen CLI changed during cargo clean')
        payload['status'] = 'validation-passed-cli-frozen-target-cleaned'
        payload['completed_at_utc'] = datetime.datetime.now(datetime.timezone.utc).isoformat()
        write_execution(out, payload)
        (out / 'frozen-cli.json').write_text(json.dumps(payload['freeze'], ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
        return 0
    except Exception as exc:
        payload['failure'] = f'{type(exc).__name__}: {exc}'
        payload['status'] = 'failed'
        try:
            payload['source_pins_after'] = input_pins(guard, includes, fixture_manifest)
        except Exception as pin_exc:
            payload['source_pins_after_error'] = f'{type(pin_exc).__name__}: {pin_exc}'
        # A failed run leaves no apparently successful candidate CLI. Only remove the fresh
        # private file created by this invocation; no workspace artifact is touched here.
        if temp_cli is not None and temp_cli.exists():
            temp_cli.unlink()
            payload['temporary_cli_removed_after_failure'] = True
        if cli_created and frozen_cli.exists():
            frozen_cli.unlink()
            payload['freeze_removed_after_failure'] = True
        write_execution(out, payload)
        print(f'validation failed: {payload["failure"]}', file=sys.stderr)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
