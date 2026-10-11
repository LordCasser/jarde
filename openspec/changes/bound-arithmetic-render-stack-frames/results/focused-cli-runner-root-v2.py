#!/usr/bin/env python3
"""Run guarded arithmetic stack checks and freeze the candidate CLI v1."""

import hashlib
import importlib.util
import json
import os
import re
import shutil
import stat
import sys
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path('/Users/lordcasser/workspace/projects/jarde').resolve()
EXPECTED_OUT = Path('/private/tmp/jarde-arithmetic-stack-focused-and-cli-root-v2').resolve()
CLI_PATH = Path('/private/tmp/jarde-arithmetic-stack-candidate-cli-root-v2').resolve()
GUARD = ROOT / 'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
SCAN = ROOT / 'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
GUARD_SHA256 = '51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
SCAN_SHA256 = 'd954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
COMPOUND_TEST = 'crates/jarde-java/src/region.rs'
PUBLIC_TEST = 'crates/jarde-java/tests/p3_conditional_switch_fallthrough.rs'
FROZEN_COMPOUND_CLASS = (
    'openspec/changes/recover-proved-compound-statement-conditions/results/'
    'complete-runtime-baseline-root-v1/observed/original-bytecode/CompoundStatementConditions.class'
)
INCLUDE_RE = re.compile(r'(?:include_bytes|include_str)!\s*\(\s*"([^"]+)"\s*,?\s*\)')
FIXTURE_MACRO_RE = re.compile(r'crate::test_fixtures::fixture!\s*\(\s*"([^"]+)"\s*\)')
SUMMARY_RE = re.compile(
    r'^test result: (?:ok|FAILED)\. (?P<passed>\d+) passed; (?P<failed>\d+) failed; '
    r'(?P<ignored>\d+) ignored; (?P<measured>\d+) measured; (?P<filtered>\d+) filtered out;',
    flags=re.MULTILINE,
)


def sha256(path):
    digest = hashlib.sha256()
    with Path(path).open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            digest.update(block)
    return digest.hexdigest()


def load_module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def compound_paths(guard):
    paths = set(guard.PRODUCT_PATHS)
    paths.add('tests/p3_immediate_functional_receivers.rs')
    paths.update({PUBLIC_TEST, FROZEN_COMPOUND_CLASS, 'crates/jarde-jvm/src/canonical.rs',
                  'crates/jarde-jvm/src/test_fixtures.rs'})
    fixture_root = (ROOT / 'crates/jarde-jvm/Cargo.toml').parent / '../../tests/fixtures'
    selected_sources = sorted(path for path in paths if path.endswith('.rs'))
    for source in selected_sources:
        # This file defines concat!-based fixture expansion; fixture literals are resolved below.
        if source == 'crates/jarde-jvm/src/test_fixtures.rs':
            continue
        text = (ROOT / source).read_text(encoding='utf-8')
        for included in INCLUDE_RE.findall(text):
            resolved = (ROOT / source).parent.joinpath(included).resolve(strict=True)
            paths.add(resolved.relative_to(ROOT).as_posix())
    canonical = ROOT / 'crates/jarde-jvm/src/canonical.rs'
    for included in FIXTURE_MACRO_RE.findall(canonical.read_text(encoding='utf-8')):
        resolved = fixture_root.joinpath(included).resolve(strict=True)
        paths.add(resolved.relative_to(ROOT).as_posix())
    return paths


def summarize_test_output(raw):
    text = re.sub(r'\x1b\[[0-9;]*m', '', raw.decode('utf-8', errors='replace'))
    rows = [
        {key: int(match.group(key)) for key in ('passed', 'failed', 'ignored', 'measured', 'filtered')}
        for match in SUMMARY_RE.finditer(text)
    ]
    return rows, text


def write_record(out, record):
    temporary = out / '.execution.json.tmp'
    temporary.write_text(json.dumps(record, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    temporary.replace(out / 'execution.json')


def main():
    if len(sys.argv) != 2 or Path(sys.argv[1]).resolve() != EXPECTED_OUT:
        raise SystemExit(f'usage: {Path(sys.argv[0]).name} {EXPECTED_OUT}')
    if EXPECTED_OUT.exists():
        raise SystemExit(f'refusing to overwrite build evidence: {EXPECTED_OUT}')
    if CLI_PATH.exists():
        raise SystemExit(f'refusing to overwrite candidate CLI: {CLI_PATH}')
    if sha256(GUARD) != GUARD_SHA256 or sha256(SCAN) != SCAN_SHA256:
        raise SystemExit('frozen build guard or target-size scanner digest changed')

    guard = load_module('compound_condition_guard_v2', GUARD)
    scanner = load_module('compound_condition_target_size_scanner_v2', SCAN)
    guard.OUT = EXPECTED_OUT
    guard.target_bytes = lambda: scanner.target_bytes(ROOT)
    guard.expected_test_summaries = lambda index, stdout: None
    guard.command_stream = lambda path: {
        'path': str(path), 'bytes': Path(path).stat().st_size, 'sha256': sha256(path)
    }

    initial_paths = compound_paths(guard)
    initial_pins = guard.source_pins(initial_paths)
    EXPECTED_OUT.mkdir(parents=False, exist_ok=False)

    environment = os.environ.copy()
    environment.update(guard.ENV_VALUES)
    stripped = []
    for key in guard.STRIPPED_ENV_KEYS:
        if key in environment:
            environment.pop(key)
            stripped.append(key)

    record = {
        'schema': 'arithmetic-stack-focused-and-cli-root-v2',
        'status': 'running',
        'runner_path': str(Path(__file__).resolve()),
        'runner_sha256': sha256(__file__),
        'guard': {'path': str(GUARD), 'sha256': GUARD_SHA256},
        'target_size_scanner': {'path': str(SCAN), 'sha256': SCAN_SHA256},
        'output_path': str(EXPECTED_OUT),
        'cli_is_unaccepted_candidate': True,
        'environment_overrides': guard.ENV_VALUES,
        'stripped_environment_keys': stripped,
        'guards': {'minimum_free_bytes': guard.FREE_LIMIT, 'maximum_target_bytes': guard.TARGET_LIMIT},
        'product_paths_before': sorted(initial_paths),
        'product_pins_before': initial_pins,
        'commands': [],
    }
    write_record(EXPECTED_OUT, record)

    commands = [
        ['cargo', 'fmt', '--all', '--', '--check'],
        ['cargo', 'test', '--test', 'p3_immediate_functional_receivers', '--locked', '--', '--nocapture'],
        ['cargo', 'test', '-p', 'jarde-java', '--test', 'p3_conditional_switch_fallthrough',
         '--locked', '--', '--nocapture'],
        ['cargo', 'build', '-p', 'jarde-cli', '--locked'],
    ]
    failure = None
    try:
        for index, argv in enumerate(commands):
            row = guard.run_command(index, argv, environment)
            row['phase'] = ('format-check', 'lambda-helper-stack-regressions', 'public-integration-tests', 'cli-build')[index]
            if index in (1, 2):
                raw = (EXPECTED_OUT / f'{index}.stdout.raw').read_bytes()
                summaries, text = summarize_test_output(raw)
                row['actual_test_summaries'] = summaries
                row['successful_test_names'] = re.findall(r'^test ([^\r\n]+) \.\.\. ok$', text, flags=re.MULTILINE)
                row['test_summary_valid'] = (
                    bool(summaries)
                    and all(summary['failed'] == 0 for summary in summaries)
                    and sum(summary['passed'] for summary in summaries) > 0
                )
            record['commands'].append(row)
            write_record(EXPECTED_OUT, record)
            print(json.dumps(row, ensure_ascii=False), flush=True)

            if row.get('exit_code') != 0 or row.get('guard_stop') is not None:
                raise RuntimeError(f"{row['phase']} failed or was stopped by the resource guard")
            if index in (1, 2) and row.get('test_summary_valid') is not True:
                raise RuntimeError(f"{row['phase']} had no successful nonempty test summary")

            current_paths = compound_paths(guard)
            current_pins = guard.source_pins(current_paths)
            if current_paths != initial_paths or current_pins != initial_pins:
                raise RuntimeError(f"pinned production/public/include inputs changed during {row['phase']}")

        if len(record['commands']) != 4:
            raise RuntimeError('expected exactly four pre-clean validation commands')
        built_binary = ROOT / 'target/debug/jarde-cli'
        if not built_binary.is_file():
            raise FileNotFoundError(f'successful CLI build did not produce {built_binary}')
        if CLI_PATH.exists():
            raise FileExistsError(f'refusing to overwrite candidate CLI: {CLI_PATH}')
        temporary_cli = CLI_PATH.with_name(f'.{CLI_PATH.name}.tmp-{os.getpid()}')
        if temporary_cli.exists():
            raise FileExistsError(f'refusing to overwrite temporary CLI: {temporary_cli}')
        shutil.copyfile(built_binary, temporary_cli)
        temporary_cli.chmod(0o555)
        temporary_cli.replace(CLI_PATH)
        mode = stat.S_IMODE(CLI_PATH.stat().st_mode)
        if mode != 0o555:
            raise RuntimeError(f'candidate CLI mode must be 0555, got {oct(mode)}')
        record['cli'] = {
            'path': str(CLI_PATH), 'bytes': CLI_PATH.stat().st_size,
            'sha256': sha256(CLI_PATH), 'mode': mode,
        }
        record['status'] = 'candidate-cli-built'
    except Exception as error:
        failure = repr(error)
        record['status'] = 'failed'
        record['error'] = failure
    finally:
        try:
            clean = guard.run_command(len(record['commands']), ['cargo', 'clean'], environment)
            clean['phase'] = 'cargo-clean'
            record['commands'].append(clean)
            record['target_absent_after_clean'] = not (ROOT / 'target').exists()
            record['product_paths_after'] = sorted(compound_paths(guard))
            record['product_pins_after'] = guard.source_pins(set(record['product_paths_after']))
            if clean.get('exit_code') != 0 or clean.get('guard_stop') is not None:
                record['status'] = 'failed'
                record['cleanup_error'] = 'cargo clean failed or hit a resource guard'
            if record['target_absent_after_clean'] is not True:
                record['status'] = 'failed'
                record['cleanup_error'] = 'target still exists after cargo clean'
            if record['product_paths_after'] != record['product_paths_before']:
                record['status'] = 'failed'
                record['cleanup_error'] = 'included source/class path set changed during build'
            if record['product_pins_after'] != record['product_pins_before']:
                record['status'] = 'failed'
                record['cleanup_error'] = 'production/public/include source pins changed during build'
        except Exception as error:
            record['status'] = 'failed'
            record['cleanup_error'] = repr(error)
        record['failure'] = failure
        write_record(EXPECTED_OUT, record)

    print(json.dumps({key: record.get(key) for key in (
        'status', 'error', 'cleanup_error', 'target_absent_after_clean', 'cli'
    )}, ensure_ascii=False))
    return 0 if record['status'] == 'candidate-cli-built' else 1


if __name__ == '__main__':
    raise SystemExit(main())
