#!/usr/bin/env python3
"""Run frozen hoisted-iterator controls through the pinned full-class collector and checker."""
from __future__ import annotations

import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
PREP = Path('/private/tmp/jarde-hoisted-iterator-controls-baseline-prep-luna-v2')
INPUTS = PREP / 'inputs'
INPUT_MANIFEST = INPUTS / 'manifest.json'
INPUT_PINS = PREP / 'frozen-inputs-pins.json'
INPUT_PINS_SHA256 = 'd47c6bb37722458709e524bdb3b03c332fea5e4b7297546790fb66bf22287334'
SOURCE_PREP = Path('/private/tmp/jarde-hoisted-iterator-controls-source-prep-luna-v2')
SOURCE_PREP_PINS = SOURCE_PREP / 'pins.json'
BASELINE_CONFIG_RECEIPT = Path('/private/tmp/jarde-cf11-testnestedloops2-baseline-root-v1/execution.json')
BASELINE_CONFIG_SHA256 = 'a7791a753850c636aa0c2d5fee5cce5b0a79434ef539eb486f5f4c8499f56951'
COLLECTOR = Path('/private/tmp/jarde-for-condition-collector-root-v3/collect_controls.py')
COLLECTOR_SHA256 = 'f944a892f4920feaff9631822bddf4dd9e49a0e8349ec4e09e4a16046257a7ac'
CHECKER = Path('/private/tmp/jarde-for-condition-checker-root-v7/check_controls.py')
CHECKER_SHA256 = 'a05acce687bf0e22bfab9eb8ae00520d53a006b740732f4b0041182ce72a464a'
CLI = Path('/private/tmp/jarde-hoisted-iterator-candidate-cli-root-v2')
CLI_SHA256 = '8bdb868ab58e4c9196970438c71de529996c66ba826c84fe216794d328fdfc9a'
FOCUSED_EXECUTION = Path('/private/tmp/jarde-hoisted-iterator-focused-root-v2/execution.json')
FOCUSED_EXECUTION_SHA256 = '69206e83ccb235ce5318b161c8398fc70e3da6ddb40b377217415af0a9e90092'
BUILD_RECEIPT = Path('/private/tmp/jarde-hoisted-iterator-controls-candidate-root-v4-build-subreceipt.json')
BUILD_RECEIPT_SHA256 = None
SOURCE_CLOSURE_HELPER = Path('/private/tmp/jarde-constructor-conditional-own-ci-verifier-prep-luna-v2/verify-own-ci.py')
SOURCE_CLOSURE_HELPER_SHA256 = 'ac511e1560c5da57fca5fcf4d37e837b726b4e20f2327014b9f29394870f0d1e'
SOURCE_COUNT = 4729
EXPECTED_COMMAND_COUNT = 43
OUT = Path('/private/tmp/jarde-hoisted-iterator-controls-candidate-root-v4')
CHECKED = OUT / 'checker-root-v7.json'
LAUNCH_RECEIPT = Path('/private/tmp/jarde-hoisted-iterator-controls-candidate-root-v4-launcher-receipt.json')
EXPECTED_TARGETS = ['cf11.HoistedIteratorPositive', 'cf11.HoistedIteratorRefusal']
EXPECTED_SOURCES = ['HoistedIteratorPositive.java', 'HoistedIteratorRefusal.java', 'Runner.java']


def sha(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def sha_file(path: Path) -> str:
    return sha(path.read_bytes())


def require_pin(path: Path, expected: str, label: str) -> None:
    if not path.is_file() or sha_file(path) != expected:
        raise SystemExit(f'{label} missing or SHA-256 changed: {path}')


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding='utf-8'))


def tracked_source_pins() -> dict[str, str]:
    require_pin(SOURCE_CLOSURE_HELPER, SOURCE_CLOSURE_HELPER_SHA256, 'current source-closure helper')
    spec = importlib.util.spec_from_file_location('constructor_conditional_own_ci_v2', SOURCE_CLOSURE_HELPER)
    if spec is None or spec.loader is None:
        raise SystemExit('cannot import pinned current source-closure helper')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    paths = module.source_paths()
    pins = module.pin_sources(paths)
    if len(pins) != SOURCE_COUNT:
        raise SystemExit(f'expected {SOURCE_COUNT} dynamically discovered source files, found {len(pins)}')
    return pins


def validate_inputs() -> dict:
    require_pin(SOURCE_PREP_PINS, 'db7f68454d199b9c6fb802732e6cb181129aed6c5129f9099ac0d5beddfbb376',
                'prepared controls source pins')
    prep_pins = load_json(SOURCE_PREP_PINS)
    if (SOURCE_PREP_PINS.stat().st_mode & 0o777) != 0o444:
        raise SystemExit('source-prep pins are not immutable (0444)')
    for name, pin in prep_pins['files'].items():
        path = SOURCE_PREP / name
        if (not path.is_file() or path.stat().st_size != pin['bytes'] or sha_file(path) != pin['sha256']
                or (path.stat().st_mode & 0o777) != 0o444):
            raise SystemExit(f'source-prep file differs from immutable pins: {path}')
    require_pin(INPUT_PINS, INPUT_PINS_SHA256, 'frozen control-input pins')
    if (INPUT_PINS.stat().st_mode & 0o777) != 0o444:
        raise SystemExit('frozen input pins are not immutable (0444)')
    frozen = load_json(INPUT_PINS)
    require_pin(INPUT_MANIFEST, frozen['files']['manifest.json']['sha256'], 'frozen input manifest')
    manifest = load_json(INPUT_MANIFEST)
    if manifest.get('package') != 'cf11' or manifest.get('source_level') != 'Java 8':
        raise SystemExit('control manifest package/source level mismatch')
    targets = [entry.get('class') for entry in manifest.get('target_classes', [])]
    if targets != EXPECTED_TARGETS or manifest.get('runner', {}).get('class') != 'cf11.Runner':
        raise SystemExit('control manifest target/runner identities differ')
    source_entries = manifest['target_classes'] + [manifest['runner']]
    if [entry['source'] for entry in source_entries] != EXPECTED_SOURCES:
        raise SystemExit('control manifest source list differs')
    for entry in source_entries:
        path = INPUTS / entry['source']
        if (not path.is_file() or path.stat().st_size != entry['bytes'] or sha_file(path) != entry['sha256']
                or (path.stat().st_mode & 0o777) != 0o444):
            raise SystemExit(f'frozen source differs from manifest or is not read-only: {path}')
    for name, pin in frozen['files'].items():
        path = INPUTS / name
        if not path.is_file() or path.stat().st_size != pin['bytes'] or sha_file(path) != pin['sha256']:
            raise SystemExit(f'frozen input pin mismatch: {path}')
    source_files = prep_pins['files']
    for entry in source_entries:
        source = SOURCE_PREP / entry['source']
        frozen_source = INPUTS / entry['source']
        if (sha_file(source) != source_files[entry['source']]['sha256']
                or source.read_bytes() != frozen_source.read_bytes()):
            raise SystemExit(f'frozen source differs from private prepared source: {source}')
    if prep_pins['files']['manifest.json']['sha256'] != sha_file(INPUT_MANIFEST):
        raise SystemExit('frozen manifest differs from source-preparation manifest')
    if len(manifest['target_classes']) != 2:
        raise SystemExit('expected exactly two target classes')
    return frozen


def raw_stream_pins(execution: dict) -> list[dict]:
    commands = execution.get('commands', [])
    if not commands:
        raise ValueError('collector receipt contains no commands')
    rows = []
    for command in commands:
        streams = command.get('streams', {})
        if set(streams) != {'stdout', 'stderr'}:
            raise ValueError(f"command {command.get('label')} lacks stdout/stderr pins")
        for stream, record in streams.items():
            path = Path(record['path'])
            if not path.is_file() or path.is_symlink():
                raise ValueError(f'missing or symlink raw stream: {path}')
            raw = path.read_bytes()
            if len(raw) != record['bytes'] or sha(raw) != record['sha256']:
                raise ValueError(f'raw stream bytes/SHA mismatch: {path}')
            rows.append({'command_index': command['index'], 'label': command['label'], 'stream': stream,
                         'path': str(path), 'bytes': len(raw), 'sha256': sha(raw)})
    if len(rows) != 2 * len(execution.get('commands', [])):
        raise ValueError('raw stream count differs from actual command count')
    return rows


def sha_file(path: Path) -> str:
    return sha(path.read_bytes())


def derive_candidate_build_receipt() -> dict:
    """Bind the cargo-build subcommand from the frozen focused receipt for collector v3."""
    require_pin(FOCUSED_EXECUTION, FOCUSED_EXECUTION_SHA256, 'focused build execution')
    focused = load_json(FOCUSED_EXECUTION)
    if focused.get('schema') != 'hoisted-iterator-focused-root-v2' or focused.get('status') != 'passed_focused_candidate_only':
        raise SystemExit('focused execution schema/status mismatch')
    if focused.get('target_absent') is not True or focused.get('runner_sha256_before') != focused.get('runner_sha256_after'):
        raise SystemExit('focused execution target/source restoration check failed')
    before, after = focused.get('source_pins_before'), focused.get('source_pins_after')
    if not isinstance(before, dict) or len(before) != SOURCE_COUNT or before != after:
        raise SystemExit('focused source closure is incomplete or changed')
    cli = focused.get('candidate_cli')
    if cli != {'path': str(CLI), 'bytes': 61314560, 'sha256': CLI_SHA256, 'mode': '0o555'}:
        raise SystemExit('focused receipt does not bind the requested candidate CLI')
    if not CLI.is_file() or CLI.stat().st_size != cli['bytes'] or sha_file(CLI) != CLI_SHA256 or (CLI.stat().st_mode & 0o777) != 0o555:
        raise SystemExit('candidate CLI bytes or mode differ from focused receipt')
    expected_argv = ['cargo', 'build', '-p', 'jarde-cli', '--bin', 'jarde-cli', '--locked']
    rows = [row for row in focused.get('commands', []) if row.get('argv') == expected_argv]
    if len(rows) != 1 or rows[0].get('index') != 6 or rows[0].get('exit_code') != 0 or rows[0].get('guard_stop') is not None:
        raise SystemExit('focused receipt lacks one successful unguarded cargo build command')
    command = rows[0]
    streams = command.get('streams')
    if not isinstance(streams, dict) or set(streams) != {'stdout', 'stderr'}:
        raise SystemExit('focused cargo build command has incomplete raw streams')
    stream_pins = {}
    for stream, record in streams.items():
        path = Path(record['path'])
        if not path.is_file() or path.is_symlink():
            raise SystemExit(f'focused cargo build raw stream missing: {path}')
        raw = path.read_bytes()
        if len(raw) != record.get('bytes') or hashlib.sha256(raw).hexdigest() != record.get('sha256'):
            raise SystemExit(f'focused cargo build raw stream pin mismatch: {path}')
        stream_pins[stream] = dict(record)
    if BUILD_RECEIPT.exists():
        raise SystemExit(f'refuse to overwrite derived build subreceipt: {BUILD_RECEIPT}')
    receipt = {
        'schema': 'hoisted-iterator-candidate-cli-build-subreceipt-v1',
        'status': 'passed_candidate_build_only',
        'parent_focused_receipt': {'path': str(FOCUSED_EXECUTION), 'bytes': FOCUSED_EXECUTION.stat().st_size,
                                   'sha256': FOCUSED_EXECUTION_SHA256,
                                   'schema': focused['schema'], 'status': focused['status']},
        'build_command': {'index': command.get('index'), 'argv': command['argv'],
                          'exit_code': command['exit_code'], 'guard_stop': command.get('guard_stop'),
                          'streams': stream_pins},
        'candidate_cli': dict(cli),
        'source_pins_before': before,
        'source_pins_after': after,
        'source_pin_count': len(before),
        'target_absent': focused['target_absent'],
        'runner_sha256_before': focused['runner_sha256_before'],
        'runner_sha256_after': focused['runner_sha256_after'],
    }
    BUILD_RECEIPT.write_text(json.dumps(receipt, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    return receipt

def main() -> int:
    if OUT.exists() or CHECKED.exists() or LAUNCH_RECEIPT.exists():
        raise SystemExit('refuse to overwrite existing collector/checker/launcher output')
    if (ROOT / 'target').exists():
        raise SystemExit('workspace target already exists; no collector command started')

    frozen = validate_inputs()
    for path, digest, label in ((COLLECTOR, COLLECTOR_SHA256, 'collector'),
                                (CHECKER, CHECKER_SHA256, 'checker'),
                                (CLI, CLI_SHA256, 'candidate CLI'),
                                (BASELINE_CONFIG_RECEIPT, BASELINE_CONFIG_SHA256, 'baseline config receipt')):
        require_pin(path, digest, label)
    if (CLI.stat().st_mode & 0o777) != 0o555:
        raise SystemExit('candidate CLI mode is not 0555')
    build = derive_candidate_build_receipt()
    BUILD_RECEIPT_SHA256 = sha_file(BUILD_RECEIPT)

    old_cfg = load_json(BASELINE_CONFIG_RECEIPT)['collector_config']
    cfg = dict(old_cfg)
    cfg.update({
        'inputs_dir': str(INPUTS), 'inputs_manifest': str(INPUT_MANIFEST),
        'out': str(OUT), 'cli': str(CLI), 'cli_sha256': CLI_SHA256,
        'build_receipt': str(BUILD_RECEIPT), 'build_receipt_sha256': BUILD_RECEIPT_SHA256,
        'receipt_cli_object': 'candidate_cli', 'receipt_status_key': 'status',
        'receipt_accepted_value': 'passed_candidate_build_only', 'receipt_mode_key': 'mode',
    })
    option_names = (
        'inputs_dir', 'inputs_manifest', 'out', 'cli', 'cli_sha256', 'build_receipt',
        'build_receipt_sha256', 'receipt_cli_object', 'receipt_status_key', 'receipt_accepted_value',
        'receipt_mode_key', 'jdk_manifest', 'jdk_manifest_sha256', 'jadx', 'jadx_sha256',
        'jadx_jar', 'jadx_jar_sha256', 'guard', 'guard_sha256', 'scanner', 'scanner_sha256',
        'classfile_parser', 'classfile_parser_sha256',
    )
    argv = [sys.executable, str(COLLECTOR)]
    for key in option_names:
        if key not in cfg:
            raise SystemExit(f'baseline collector config misses {key}')
        argv.extend(['--' + key.replace('_', '-'), str(cfg[key])])

    before = tracked_source_pins()
    if before != build.get('source_pins_before') or before != build.get('source_pins_after'):
        raise SystemExit('dynamic current source closure differs from final build before/after pins; no collector run')
    if (ROOT / 'target').exists():
        raise SystemExit('workspace target appeared before collector start')
    collector = subprocess.run(argv, cwd=ROOT, check=False)

    execution_path = OUT / 'execution.json'
    execution = load_json(execution_path) if execution_path.is_file() else None
    checker = None
    checker_argv = None
    checker_skip_reason = None
    if execution is not None:
        checker_argv = [sys.executable, str(CHECKER), '--execution', str(execution_path), '--out', str(CHECKED)]
        checker = subprocess.run(checker_argv, cwd=ROOT, check=False)
    else:
        checker_skip_reason = 'collector did not leave execution.json; checker was not invoked'

    after = tracked_source_pins()
    target_absent = not (ROOT / 'target').exists()
    root_pins_unchanged = before == after == build.get('source_pins_before') == build.get('source_pins_after')
    frozen_inputs_unchanged = False
    frozen_input_error = None
    try:
        frozen_inputs_unchanged = validate_inputs() == frozen
    except (KeyError, OSError, TypeError, ValueError, SystemExit) as exc:
        frozen_input_error = {'type': type(exc).__name__, 'message': str(exc)}
    raw_pins = []
    raw_error = None
    if execution is not None:
        try:
            raw_pins = raw_stream_pins(execution)
        except (KeyError, OSError, TypeError, ValueError) as exc:
            raw_error = {'type': type(exc).__name__, 'message': str(exc)}

    compile_rows = []
    if execution is not None:
        compile_rows = (execution.get('original_compile_rows', []) + execution.get('jadx_compile_rows', [])
                        + execution.get('jarde_compile_rows', []))
    compile_failures = [{'label': row.get('label'), 'compile_exit': row.get('compile_exit'),
                         'class_paths': row.get('class_paths'), 'class_set_exact': row.get('class_set_exact')}
                        for row in compile_rows if row.get('compile_exit') not in (None, 0)]
    checker_result = load_json(CHECKED) if CHECKED.is_file() else None
    checker_status = checker_result.get('status') if checker_result else None

    command_count_ok = bool(execution) and len(execution.get('commands', [])) == EXPECTED_COMMAND_COUNT
    integrity_ok = root_pins_unchanged and frozen_inputs_unchanged and target_absent and raw_error is None and command_count_ok and len(raw_pins) == 2 * EXPECTED_COMMAND_COUNT
    checks_ok = collector.returncode == 0 and checker is not None and checker.returncode == 0
    if not integrity_ok:
        status = 'postflight_integrity_failure'
    elif not checks_ok:
        status = 'collector_or_checker_failure'
    elif compile_failures:
        status = 'capture_checked_with_compile_failures'
    elif checker_status == 'verified_observation_capture':
        status = 'capture_checked'
    else:
        status = 'capture_checked_with_quality_gaps'

    receipt = {
        'schema': 'hoisted-iterator-controls-baseline-launch-receipt-v3',
        'collector_argv': argv, 'collector_exit_code': collector.returncode,
        'checker_argv': checker_argv, 'checker_exit_code': checker.returncode if checker else None,
        'checker_skip_reason': checker_skip_reason, 'checker_status': checker_status,
        'collector_execution': str(execution_path) if execution is not None else None,
        'actual_command_count': len(execution.get('commands', [])) if execution else None,
        'expected_command_count': EXPECTED_COMMAND_COUNT, 'command_count_ok': command_count_ok,
        'raw_stream_count': len(raw_pins), 'raw_stream_pins': raw_pins, 'raw_stream_error': raw_error,
        'compile_failures': compile_failures,
        'source_closure_helper': {'path': str(SOURCE_CLOSURE_HELPER), 'sha256': SOURCE_CLOSURE_HELPER_SHA256},
        'focused_receipt': {'path': str(FOCUSED_EXECUTION), 'sha256': FOCUSED_EXECUTION_SHA256},
        'derived_build_subreceipt': {'path': str(BUILD_RECEIPT), 'sha256': BUILD_RECEIPT_SHA256},
        'source_closure_count': len(before), 'source_pins_before': before, 'source_pins_after': after,
        'source_pins_unchanged': root_pins_unchanged, 'frozen_inputs_unchanged': frozen_inputs_unchanged,
        'frozen_input_error': frozen_input_error, 'target_absent_after': target_absent,
        'frozen_input_pin': {'path': str(INPUT_PINS), 'sha256': sha_file(INPUT_PINS)},
        'status': status,
    }
    LAUNCH_RECEIPT.write_text(json.dumps(receipt, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({key: receipt[key] for key in (
        'status', 'collector_exit_code', 'checker_exit_code', 'checker_status',
        'actual_command_count', 'expected_command_count', 'command_count_ok', 'raw_stream_count', 'compile_failures',
        'source_closure_count', 'source_pins_unchanged', 'frozen_inputs_unchanged', 'target_absent_after',
    )}, ensure_ascii=False))
    return 0 if integrity_ok and checks_ok else (collector.returncode or (checker.returncode if checker else 1) or 1)


if __name__ == '__main__':
    raise SystemExit(main())
