#!/usr/bin/env python3
"""Read-only validator for the arithmetic-stack CLI v1 build evidence."""
import hashlib
import importlib.util
import json
import re
import stat
from pathlib import Path

ROOT = Path('/Users/lordcasser/workspace/projects/jarde').resolve()
BUILD_PATH = Path('/private/tmp/jarde-arithmetic-stack-focused-and-cli-root-v2/execution.json').resolve()
CLI_PATH = Path('/private/tmp/jarde-arithmetic-stack-candidate-cli-root-v2').resolve()
RUNNER_PATH = Path('/private/tmp/jarde-arithmetic-stack-focused-and-cli-root-v2.py').resolve()
RUNNER_SHA256 = 'dc0c74fa9e5944a2cc19a7968bed6c66e700ba7f1c798cab6d32167f04f24844'
BUILD_SCHEMA = 'arithmetic-stack-focused-and-cli-root-v2'
ANSI_RE = re.compile(r'\x1b\[[0-9;]*m')
SUMMARY_RE = re.compile(
    r'^test result: (?:ok|FAILED)\. (?P<passed>\d+) passed; (?P<failed>\d+) failed; '
    r'(?P<ignored>\d+) ignored; (?P<measured>\d+) measured; (?P<filtered>\d+) filtered out;',
    re.MULTILINE,
)
EXPECTED_COMMANDS = [
    ['cargo', 'fmt', '--all', '--', '--check'],
    ['cargo', 'test', '--test', 'p3_immediate_functional_receivers', '--locked', '--', '--nocapture'],
    ['cargo', 'test', '-p', 'jarde-java', '--test', 'p3_conditional_switch_fallthrough',
     '--locked', '--', '--nocapture'],
    ['cargo', 'build', '-p', 'jarde-cli', '--locked'],
    ['cargo', 'clean'],
]
EXPECTED_PHASES = ('format-check', 'lambda-helper-stack-regressions', 'public-integration-tests',
                   'cli-build', 'cargo-clean')


def sha(path):
    h = hashlib.sha256()
    with Path(path).open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def require(condition, message):
    if not condition:
        raise ValueError(message)


def resolve_record_path(value):
    path = Path(value)
    return path if path.is_absolute() else ROOT / path


def pin_matches(pin):
    path = resolve_record_path(pin['path'])
    return path.is_file() and path.stat().st_size == pin.get('bytes') and sha(path) == pin.get('sha256')


def raw_streams(command):
    result = {}
    streams = command.get('streams')
    require(isinstance(streams, dict), f"command {command.get('index')} missing raw stream pins")
    for name in ('stdout', 'stderr'):
        entry = streams.get(name)
        require(isinstance(entry, dict) and pin_matches(entry), f"command {command.get('index')} {name} pin mismatch")
        result[name] = resolve_record_path(entry['path']).read_bytes()
    return result


def expected_product_paths():
    require(RUNNER_PATH.is_file() and sha(RUNNER_PATH) == RUNNER_SHA256,
            'arithmetic-stack build runner path/digest mismatch')
    spec = importlib.util.spec_from_file_location('arithmetic_stack_runner_for_verification', RUNNER_PATH)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    guard = module.load_module('arithmetic_stack_guard_v9_for_verification', module.GUARD)
    return module, module.compound_paths(guard)


def verify_build(build_path, replay_cli, expected_cli_sha=None):
    build_path = Path(build_path).resolve(strict=True)
    require(build_path == BUILD_PATH, 'candidate build path is not the final CLI v2 build record')
    build = json.loads(build_path.read_text(encoding='utf-8'))
    require(build.get('schema') == BUILD_SCHEMA, 'candidate build schema mismatch')
    require(build.get('status') == 'candidate-cli-built', 'candidate CLI build did not complete')
    require(build.get('cli_is_unaccepted_candidate') is True, 'build does not identify its CLI as a candidate')
    cli = build.get('cli')
    require(isinstance(cli, dict) and set(cli) == {'path', 'bytes', 'sha256', 'mode'}, 'CLI pin malformed')
    require(Path(cli['path']).resolve() == CLI_PATH, 'build CLI path is not the fixed root-v2 candidate')
    require(pin_matches(cli), 'build CLI pin differs from live CLI')
    require(cli.get('mode') == 0o555 and stat.S_IMODE(Path(cli['path']).stat().st_mode) == 0o555,
            'candidate CLI mode is not 0555')
    require(isinstance(replay_cli, dict) and
            {k: replay_cli.get(k) for k in ('path', 'bytes', 'sha256')} ==
            {k: cli.get(k) for k in ('path', 'bytes', 'sha256')},
            'replay CLI does not match candidate build CLI')
    if expected_cli_sha is not None:
        require(cli.get('sha256') == expected_cli_sha, 'CLI SHA differs from argv pin')

    runner, expected_paths = expected_product_paths()
    require(build.get('output_path') == str(BUILD_PATH.parent),
            'build record output path differs from the fixed arithmetic-stack evidence directory')
    require(build.get('runner_path') == str(RUNNER_PATH) and
            build.get('runner_sha256') == RUNNER_SHA256 and sha(RUNNER_PATH) == RUNNER_SHA256,
            'build record runner identity differs from pinned arithmetic-stack runner')
    for field, expected_path, expected_sha in (
            ('guard', runner.GUARD, runner.GUARD_SHA256),
            ('target_size_scanner', runner.SCAN, runner.SCAN_SHA256)):
        item = build.get(field, {})
        require(item.get('path') == str(expected_path) and item.get('sha256') == expected_sha
                and Path(expected_path).is_file() and sha(expected_path) == expected_sha,
                f'build {field} identity/pin mismatch')
    before, after = build.get('product_pins_before'), build.get('product_pins_after')
    paths_before, paths_after = build.get('product_paths_before'), build.get('product_paths_after')
    require(isinstance(before, dict) and isinstance(after, dict), 'product source pin maps missing')
    require(set(before) == expected_paths and set(after) == expected_paths,
            'product source pin keys differ from current arithmetic-stack runner dynamic selection')
    require(paths_before == sorted(expected_paths) and paths_after == sorted(expected_paths),
            'product source paths differ from v2 dynamic selection')
    require(before == after, 'product source pins changed during build')
    for rel, expected in before.items():
        source = (ROOT / rel).resolve(strict=True)
        require(source.is_relative_to(ROOT) and sha(source) == expected,
                f'live source pin mismatch: {rel}')

    commands = build.get('commands')
    require(isinstance(commands, list) and len(commands) == len(EXPECTED_COMMANDS),
            'expected four build commands plus final cargo clean')
    parsed_test_summaries = {}
    for index, (command, argv, phase) in enumerate(zip(commands, EXPECTED_COMMANDS, EXPECTED_PHASES)):
        require(command.get('index') == index and command.get('argv') == argv,
                f'build command {index} argv/index mismatch')
        require(Path(command.get('cwd', '')).resolve() == ROOT, f'build command {index} cwd mismatch')
        require(command.get('phase') == phase, f'build command {index} phase mismatch')
        require(command.get('exit_code') == 0 and command.get('guard_stop') is None,
                f'build command {index} failed or hit resource guard')
        raw = raw_streams(command)
        if index in (1, 2):
            text = ANSI_RE.sub('', raw['stdout'].decode('utf-8', errors='strict'))
            found = [{k: int(match.group(k)) for k in ('passed', 'failed', 'ignored', 'measured', 'filtered')}
                     for match in SUMMARY_RE.finditer(text)]
            require(found and all(row['failed'] == 0 for row in found) and
                    sum(row['passed'] for row in found) > 0,
                    f'build command {index} has no successful nonempty test summary')
            require(command.get('actual_test_summaries') == found,
                    f'build command {index} recorded summary differs from raw stdout')
            require(command.get('test_summary_valid') is True,
                    f'build command {index} summary-valid marker missing')
            parsed_test_summaries[phase] = found
    require(build.get('target_absent_after_clean') is True, 'build does not record post-clean target absence')
    require(commands[-1].get('argv') == ['cargo', 'clean'], 'final cleanup command is not cargo clean')
    return {
        'path': str(build_path), 'sha256': sha(build_path), 'cli': cli,
        'product_source_pin_count': len(expected_paths),
        'dynamic_product_paths': sorted(expected_paths),
        'passed_named_test_counts': {phase: row[0]['passed'] for phase, row in parsed_test_summaries.items()},
        'successful_command_count': len(commands),
        'test_summaries': parsed_test_summaries,
        'target_absent_after_clean': True,
    }
