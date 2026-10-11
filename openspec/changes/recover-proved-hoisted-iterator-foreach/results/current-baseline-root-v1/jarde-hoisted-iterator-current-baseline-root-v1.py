#!/usr/bin/env python3
"""Run the frozen TestNestedLoops baseline collector, then its pinned checker."""
from __future__ import annotations

import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
PREP = Path('/private/tmp/jarde-hoisted-iterator-current-baseline-prep-luna-v1')
INPUTS = Path('/private/tmp/jarde-cf11-testnestedloops-inputs-root-v1')
OUT = Path('/private/tmp/jarde-hoisted-iterator-current-baseline-root-v1')
CHECKED = OUT / 'checker-root-v7.json'
LAUNCH_RECEIPT = Path('/private/tmp/jarde-hoisted-iterator-current-baseline-root-v1-launcher-receipt.json')
INPUT_PINS = INPUTS / 'frozen-pins.json'
INPUT_PINS_SHA256 = '850f390681d35a98cbaed1fb8e5c714dd5e0e698dd7f18f8976950c9c52a4099'
INPUT_MANIFEST = INPUTS / 'manifest.json'
SOURCE_PREP_PINS = Path('/private/tmp/jarde-cf11-testnestedloops-input-luna-v3/pins.json')
BASELINE_CONFIG_RECEIPT = Path('/private/tmp/jarde-cf11-testnestedloops2-baseline-root-v1/execution.json')
BASELINE_CONFIG_SHA256 = 'a7791a753850c636aa0c2d5fee5cce5b0a79434ef539eb486f5f4c8499f56951'
COLLECTOR = Path('/private/tmp/jarde-for-condition-collector-root-v3/collect_controls.py')
COLLECTOR_SHA256 = 'f944a892f4920feaff9631822bddf4dd9e49a0e8349ec4e09e4a16046257a7ac'
CHECKER = Path('/private/tmp/jarde-for-condition-checker-root-v7/check_controls.py')
CHECKER_SHA256 = 'a05acce687bf0e22bfab9eb8ae00520d53a006b740732f4b0041182ce72a464a'
CLI = Path('/private/tmp/jarde-constructor-conditional-final-cli-root-v1')
CLI_SHA256 = 'a71c31e80da7c6550aa572e5368f0ecad12ac862066c5d40535f5818c2263241'
BUILD_RECEIPT = Path('/private/tmp/jarde-constructor-conditional-final-lint-build-root-v1/execution.json')
BUILD_RECEIPT_SHA256 = '90ecba1b52177d7dfae590679603e4624c4e56a42ae2b58d0616d7078c1f46e2'
SOURCE_CLOSURE_HELPER = Path('/private/tmp/jarde-constructor-conditional-own-ci-verifier-prep-luna-v2/verify-own-ci.py')
SOURCE_CLOSURE_HELPER_SHA256 = 'ac511e1560c5da57fca5fcf4d37e837b726b4e20f2327014b9f29394870f0d1e'
EXPECTED_SOURCE_COUNT = 4727


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def require_pin(path: Path, expected_sha: str, label: str) -> None:
    if not path.is_file() or sha256(path) != expected_sha:
        raise SystemExit(f'{label} missing or SHA-256 changed: {path}')


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding='utf-8'))


def tracked_source_pins() -> dict[str, str]:
    # Reuse the dynamically reconstructed source closure from the pinned current own-CI verifier v2.
    # It includes source/test Rust files, fixtures, frozen evidence, and literal include dependencies.
    require_pin(SOURCE_CLOSURE_HELPER, SOURCE_CLOSURE_HELPER_SHA256, 'source-closure helper')
    spec = importlib.util.spec_from_file_location('constructor_conditional_own_ci_v2', SOURCE_CLOSURE_HELPER)
    if spec is None or spec.loader is None:
        raise SystemExit('cannot load pinned source-closure helper')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    paths = module.source_paths()
    pins = module.pin_sources(paths)
    if len(pins) != EXPECTED_SOURCE_COUNT:
        raise SystemExit(f'expected {EXPECTED_SOURCE_COUNT} source closure files, found {len(pins)}')
    return pins


def validate_frozen_inputs() -> dict:
    require_pin(SOURCE_PREP_PINS, 'd09a4acb1e4a5a0789e06643028f53b1aa3103a2c5ea15287710e81d65c2273b', 'source prep pins')
    require_pin(INPUT_PINS, INPUT_PINS_SHA256, 'frozen input pins')
    source_pins = load_json(SOURCE_PREP_PINS)
    frozen = load_json(INPUT_PINS)
    require_pin(INPUT_MANIFEST, frozen['manifest_file']['sha256'], 'frozen manifest')
    manifest = load_json(INPUT_MANIFEST)
    if frozen.get('schema') != 'cf11-testnestedloops-root-baseline-frozen-inputs-v1':
        raise SystemExit('unexpected frozen-input pins schema')
    for entry in frozen['input_files'] + frozen['provenance_files'] + [frozen['manifest_file']]:
        path = Path(entry['path'])
        if not path.is_file() or path.stat().st_size != entry['bytes'] or sha256(path) != entry['sha256']:
            raise SystemExit(f'frozen input pin mismatch: {path}')
        if (path.stat().st_mode & 0o777) != 0o444:
            raise SystemExit(f'frozen input mode is not 0444: {path}')
    all_frozen_paths = {Path(x['path']).name: Path(x['path']) for x in frozen['input_files'] + frozen['provenance_files'] + [frozen['manifest_file']]}
    provenance = {Path(x['path']).name: Path(x['path']) for x in frozen['provenance_files']}
    upstream = provenance['TestNestedLoops.java']
    live_upstream = Path(source_pins['source_path'])
    expected_provenance = {
        'UpstreamNestedLoops.java': source_pins['wrapper'],
        'Runner.java': source_pins['runner'],
        'manifest.json': source_pins['manifest'],
        'README.md': source_pins['readme'],
        'test-method.slice': source_pins['method_slice'],
    }
    for name, row in expected_provenance.items():
        copy = all_frozen_paths.get(name)
        if copy is None or copy.stat().st_size != row['bytes'] or sha256(copy) != row['sha256']:
            raise SystemExit(f'private prep pin differs for {name}')
    if sha256(upstream) != source_pins['source_sha256'] or live_upstream.read_bytes() != upstream.read_bytes():
        raise SystemExit('live upstream source differs from frozen pinned source')
    slice_pin = source_pins['method_slice']
    slice_path = provenance['test-method.slice']
    raw = upstream.read_bytes()
    method_slice = raw[slice_pin['start_byte_in_source']:slice_pin['end_byte_exclusive_in_source']]
    if (len(method_slice) != slice_pin['bytes'] or hashlib.sha256(method_slice).hexdigest() != slice_pin['sha256']
            or method_slice != slice_path.read_bytes()
            or method_slice not in Path(next(x['path'] for x in frozen['input_files'] if x['manifest_entry']['class'] == 'cf11.UpstreamNestedLoops')).read_bytes()):
        raise SystemExit('upstream method slice pin/exact-wrapper check failed')
    expected = [('cf11.UpstreamNestedLoops', 'UpstreamNestedLoops.java'), ('cf11.Runner', 'Runner.java')]
    actual = [(x['class'], x['source']) for x in manifest['target_classes']] + [(manifest['runner']['class'], manifest['runner']['source'])]
    if actual != expected or manifest.get('package') != 'cf11' or manifest.get('source_level') != 'Java 8':
        raise SystemExit('frozen manifest differs from the intended top-level Java 8 inputs')
    return frozen


def main() -> int:
    if OUT.exists() or CHECKED.exists() or LAUNCH_RECEIPT.exists():
        raise SystemExit('refuse to overwrite existing collector/checker/launcher output')
    if (ROOT / 'target').exists():
        raise SystemExit('workspace target already exists; no collector command started')

    frozen = validate_frozen_inputs()
    require_pin(COLLECTOR, COLLECTOR_SHA256, 'collector')
    require_pin(CHECKER, CHECKER_SHA256, 'checker')
    require_pin(CLI, CLI_SHA256, 'final CLI')
    require_pin(BUILD_RECEIPT, BUILD_RECEIPT_SHA256, 'final build receipt')
    if (CLI.stat().st_mode & 0o777) != 0o555:
        raise SystemExit('final CLI mode is not 0555')
    build = load_json(BUILD_RECEIPT)
    if build.get('schema') != 'constructor-conditional-final-lint-build-root-v1' or build.get('status') != 'passed_candidate_build_only':
        raise SystemExit('build receipt schema/status is not the pinned current final candidate build')
    binding = build.get('candidate_cli')
    if binding != {'path': str(CLI), 'bytes': CLI.stat().st_size, 'sha256': CLI_SHA256, 'mode': '0o555'}:
        raise SystemExit('build receipt does not bind the requested frozen CLI')

    require_pin(BASELINE_CONFIG_RECEIPT, BASELINE_CONFIG_SHA256, 'baseline collector config receipt')
    old = load_json(BASELINE_CONFIG_RECEIPT)
    old_cfg = old['collector_config']
    # Preserve the established TestNestedLoops2 environment/tool pins; replace only the input,
    # output, CLI, and build-receipt binding for this TestNestedLoops capture.
    cfg = dict(old_cfg)
    cfg.update({
        'inputs_dir': str(INPUTS),
        'inputs_manifest': str(INPUT_MANIFEST),
        'out': str(OUT),
        'cli': str(CLI),
        'cli_sha256': CLI_SHA256,
        'build_receipt': str(BUILD_RECEIPT),
        'build_receipt_sha256': BUILD_RECEIPT_SHA256,
        'receipt_cli_object': 'candidate_cli',
        'receipt_status_key': 'status',
        'receipt_accepted_value': 'passed_candidate_build_only',
        'receipt_mode_key': 'mode',
    })
    option_names = (
        'inputs_dir', 'inputs_manifest', 'out', 'cli', 'cli_sha256', 'build_receipt',
        'build_receipt_sha256', 'receipt_cli_object', 'receipt_status_key', 'receipt_accepted_value',
        'receipt_mode_key', 'jdk_manifest', 'jdk_manifest_sha256', 'jadx', 'jadx_sha256',
        'jadx_jar', 'jadx_jar_sha256', 'guard', 'guard_sha256', 'scanner', 'scanner_sha256',
        'classfile_parser', 'classfile_parser_sha256',
    )
    argv = [sys.executable, str(COLLECTOR)]
    for name in option_names:
        if name not in cfg:
            raise SystemExit(f'baseline collector config misses {name}')
        argv.extend(['--' + name.replace('_', '-'), str(cfg[name])])

    before = tracked_source_pins()
    if before != build['source_pins_before'] or before != build['source_pins_after']:
        raise SystemExit('current complete closure differs from final build receipt; no collector run')
    if (ROOT / 'target').exists():
        raise SystemExit('workspace target appeared before collector start')
    collector = subprocess.run(argv, cwd=ROOT, check=False)

    checker_argv = None
    checker = None
    checker_skip_reason = None
    execution = OUT / 'execution.json'
    if execution.is_file():
        checker_argv = [sys.executable, str(CHECKER), '--execution', str(execution), '--out', str(CHECKED)]
        checker = subprocess.run(checker_argv, cwd=ROOT, check=False)
    else:
        checker_skip_reason = 'collector did not leave execution.json; checker was not invoked'

    after = tracked_source_pins()
    target_absent = not (ROOT / 'target').exists()
    receipt = {
        'schema': 'cf11-testnestedloops-hoisted-iterator-current-baseline-launch-receipt-v1',
        'collector_argv': argv,
        'collector_exit_code': collector.returncode,
        'checker_argv': checker_argv,
        'checker_exit_code': checker.returncode if checker is not None else None,
        'checker_skip_reason': checker_skip_reason,
        'collector_execution_exists': execution.is_file(),
        'checker_output': str(CHECKED) if CHECKED.is_file() else None,
        'source_closure_helper': {'path': str(SOURCE_CLOSURE_HELPER), 'sha256': SOURCE_CLOSURE_HELPER_SHA256},
        'source_pins_before': before,
        'source_pins_after': after,
        'source_pin_count_before': len(before),
        'source_pin_count_after': len(after),
        'source_pins_unchanged': before == after,
        'target_absent_after': target_absent,
        'frozen_inputs_pin': {'path': str(INPUT_PINS), 'sha256': sha256(INPUT_PINS)},
        'status': 'complete' if collector.returncode == 0 and checker is not None and checker.returncode == 0 and before == after and target_absent else 'observed_failure_or_guard_mismatch',
    }
    LAUNCH_RECEIPT.write_text(json.dumps(receipt, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({key: receipt[key] for key in (
        'status', 'collector_exit_code', 'checker_exit_code', 'checker_skip_reason',
        'source_pin_count_before', 'source_pin_count_after', 'source_pins_unchanged', 'target_absent_after',
    )}, ensure_ascii=False))
    if collector.returncode != 0:
        return collector.returncode
    if checker is None or checker.returncode != 0:
        return checker.returncode if checker is not None else 1
    return 0 if before == after and target_absent else 1


if __name__ == '__main__':
    raise SystemExit(main())
