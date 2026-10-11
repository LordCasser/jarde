#!/usr/bin/env python3
"""Replay complete CF07 original/JADX/Jarde sources on two JDKs; execution is root-owned."""
from __future__ import annotations
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import stat
import sys

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
OUT_DEFAULT = Path('/private/tmp/jarde-computed-for-cf07-final-jdk-replay-root-v1')
CLI_DEFAULT = Path('/private/tmp/jarde-computed-for-candidate-cli-root-v1')
RECEIPT_DEFAULT = Path('/private/tmp/jarde-computed-for-cli-build-root-v1/execution.json')
ORIGINAL_SOURCE = ROOT / 'openspec/changes/preserve-proved-for-latch-origins/results/cf07-candidate-root-v1/original-sources/LoopCases.java'
RUNNER_SOURCE = ROOT / 'openspec/changes/preserve-proved-for-latch-origins/results/cf07-candidate-root-v1/original-sources/Runner.java'
ORIGINAL_CLASSES = {
    'javac8': ROOT / 'openspec/changes/preserve-proved-for-latch-origins/results/cf07-candidate-root-v1/cases/javac8-original/classes',
    'javac23': ROOT / 'openspec/changes/preserve-proved-for-latch-origins/results/cf07-candidate-root-v1/cases/javac23-original/classes',
}
JADX_EXECUTION = ROOT / 'openspec/changes/recover-proved-computed-for-initializers/results/complete-controls-root-v1/fresh-cf07-jadx/execution.json'
JADX_SOURCE = ROOT / 'openspec/changes/recover-proved-computed-for-initializers/results/complete-controls-root-v1/fresh-cf07-jadx/out/sources/cf07/LoopCases.java'
GUARD_PATH = ROOT / 'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
SCANNER_PATH = ROOT / 'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
EXPECTED = {
    'original_source': '3184f43aa78ba2f26752d15e0b706fe7b0cc9cf25bca34dded34bf603ecb3c38',
    'runner_source': '28b0a08ceb89b6ff31d12056831a2d7cc62bfda0a61ebc39835d2f62bc89bdf5',
    'original_javac8_class': 'eeda5de66a2f436f077ef3fcf852fe054c01e1a86b71b5638777ce5a0b0a9708',
    'original_javac23_class': '004e99baeb062861f95a4dac13e3a78399a66efd31f2589b0028cfe50432ae33',
    'jadx_source': '4d5023782e6d10dae9bd8b786ad18ec62d8029e08cea9eebec38f8036e815fc2',
    'guard': '51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33',
    'scanner': 'd954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7',
}
JDKS = {
    'javac8': {
        'home': Path('/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home'),
        'javac': '598601455e9f81ef05225c014743aaafa993d572afc7f83f5e7934ff10f9dddd',
        'java': '3167d94b1b56e1330e25e48a92bba9cba423ca578525c4ae66e8b5a06fb5dbb5',
    },
    'javac23': {
        'home': Path('/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home'),
        'javac': 'a3e79462d70cb70c34b85ab94ae615328d1cbe7ac6a2c69635288ae6cbb902a8',
        'java': 'b79b8bac2b2a2c2b4d0dcb9c3981d477bd58bc95c5c4404dfb40a37e581497a1',
    },
}
ENV_STRIP = ('JAVA_TOOL_OPTIONS', '_JAVA_OPTIONS', 'JDK_JAVA_OPTIONS', 'CLASSPATH')


def sha_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha_file(path: Path) -> str:
    return sha_bytes(path.read_bytes())


def pin(path: Path) -> dict:
    data = path.read_bytes()
    return {'path': str(path), 'bytes': len(data), 'sha256': sha_bytes(data)}


def load_module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f'cannot import existing helper: {path}')
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cli', type=Path, required=True)
    parser.add_argument('--cli-sha256', required=True)
    parser.add_argument('--build-receipt', type=Path, default=RECEIPT_DEFAULT)
    parser.add_argument('--output', type=Path, default=OUT_DEFAULT)
    args = parser.parse_args()
    out = args.output.resolve()
    cli = args.cli.resolve()
    receipt_path = args.build_receipt.resolve()
    if cli != CLI_DEFAULT:
        raise ValueError(f'expected the planned frozen CLI path {CLI_DEFAULT}, got {cli}')
    if out.exists():
        raise FileExistsError(f'refusing to overwrite replay output: {out}')
    out.mkdir(parents=True)
    record = {
        'schema': 'cf07-final-dual-jdk-full-source-replay-luna-v2',
        'status': 'preflight',
        'claim': 'fresh compilation/runtime replay; frozen JADX 1.5.6 source reused byte-for-byte; no new JADX execution',
        'cli': None,
        'inputs': {},
        'tools': {},
        'commands': [],
        'source_variants': [],
        'runtime_comparisons': [],
    }
    execution_path = out / 'execution.json'
    record['runner'] = pin(Path(__file__).resolve())

    def save(status: str | None = None) -> None:
        if status is not None:
            record['status'] = status
        execution_path.write_text(json.dumps(record, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')

    save()
    try:
        if not receipt_path.is_file():
            raise FileNotFoundError(f'final CLI build receipt is not present: {receipt_path}')
        receipt = json.loads(receipt_path.read_text(encoding='utf-8'))
        if receipt.get('schema') != 'jarde-final-cli-build-clean-root-v1':
            raise ValueError(f'unexpected final CLI build schema: {receipt.get("schema")!r}')
        freeze = receipt.get('freeze')
        if not isinstance(freeze, dict):
            raise ValueError('build receipt has no freeze object')
        status = receipt.get('status')
        if status != 'validation-passed-cli-frozen-target-cleaned':
            raise ValueError(f'build receipt is not an accepted frozen-and-clean build: {status!r}')
        frozen = freeze.get('frozen_cli')
        if not isinstance(frozen, dict):
            raise ValueError('build receipt freeze has no frozen_cli object')
        frozen_path = Path(frozen.get('path', '')).resolve()
        if frozen_path != cli:
            raise ValueError(f'build receipt freezes a different CLI: {frozen_path}')
        if receipt.get('target_absent_after_clean') is not True:
            raise ValueError('build receipt does not confirm target_absent_after_clean')
        before_paths = receipt.get('source_paths_before')
        after_paths = receipt.get('source_paths_after')
        before_pins = receipt.get('source_pins_before')
        after_pins = receipt.get('source_pins_after')
        if not isinstance(before_paths, list) or before_paths != after_paths:
            raise ValueError('build receipt source path closure is missing or changed')
        if not isinstance(before_pins, dict) or before_pins != after_pins:
            raise ValueError('build receipt source pin closure is missing or changed')
        full_closure = before_pins.get('full_product_source_and_include_closure')
        if not isinstance(full_closure, dict) or not full_closure:
            raise ValueError('build receipt has no full product source/include pin closure')
        if freeze.get('product_source_pins') != full_closure:
            raise ValueError('frozen build closure differs from the before/after full source pins')
        actual_cli_sha = sha_file(cli)
        actual_cli_size = cli.stat().st_size
        actual_mode = stat.S_IMODE(cli.stat().st_mode)
        try:
            recorded_mode = int(frozen.get('mode', ''), 8)
        except (TypeError, ValueError):
            raise ValueError(f'build receipt has invalid frozen CLI mode: {frozen.get("mode")!r}')
        if (actual_cli_sha != args.cli_sha256 or frozen.get('sha256') != args.cli_sha256
                or frozen.get('size_bytes') != actual_cli_size):
            raise ValueError('CLI path/size/SHA-256 disagrees with argv and build freeze')
        if actual_mode != 0o555 or recorded_mode != actual_mode:
            raise ValueError(f'CLI is not frozen read/execute-only: mode={oct(actual_mode)}, receipt={frozen.get("mode")!r}')
        record['cli'] = {
            'path': str(cli), 'bytes': actual_cli_size, 'sha256': actual_cli_sha,
            'mode': oct(actual_mode), 'expected_sha256_argv': args.cli_sha256,
            'build_receipt': pin(receipt_path), 'build_schema': receipt.get('schema'),
            'build_status': status, 'freeze': freeze,
            'source_paths_before_after_equal': True,
            'full_source_include_closure_pin_count': len(full_closure),
            'full_source_include_closure_sha256': sha_bytes(json.dumps(full_closure, sort_keys=True, separators=(',', ':')).encode()),
        }

        static_inputs = [
            ('original_source', ORIGINAL_SOURCE, EXPECTED['original_source']),
            ('original_runner', RUNNER_SOURCE, EXPECTED['runner_source']),
            ('jadx_source', JADX_SOURCE, EXPECTED['jadx_source']),
            ('jadx_execution_record', JADX_EXECUTION, None),
            ('guard_template', GUARD_PATH, EXPECTED['guard']),
            ('target_size_scanner', SCANNER_PATH, EXPECTED['scanner']),
        ]
        for leg in JDKS:
            classfile = ORIGINAL_CLASSES[leg] / 'cf07/LoopCases.class'
            static_inputs.append((f'original_{leg}_class', classfile, EXPECTED[f'original_{leg}_class']))
        for name, path, expected_sha in static_inputs:
            actual = pin(path)
            if expected_sha is not None and actual['sha256'] != expected_sha:
                raise ValueError(f'{name} pin mismatch: {actual}')
            record['inputs'][name] = actual
        jadx_record = json.loads(JADX_EXECUTION.read_text(encoding='utf-8'))
        if jadx_record.get('status') != 'passed_fresh_JADX_full_LoopCases_both_JDKs':
            raise ValueError('frozen JADX source record is not accepted')
        if jadx_record.get('generated_source_sha256') != EXPECTED['jadx_source']:
            raise ValueError('frozen JADX source hash disagrees with its accepted execution record')
        for leg, facts in JDKS.items():
            for tool in ('javac', 'java'):
                path = facts['home'] / 'bin' / tool
                actual = pin(path)
                if actual['sha256'] != facts[tool]:
                    raise ValueError(f'{leg} {tool} binary pin mismatch: {actual}')
                record['tools'][leg + '.' + tool] = actual

        guard = load_module('jarde_guard_v9', GUARD_PATH)
        scanner = load_module('jarde_target_scan_v1', SCANNER_PATH)
        if sha_file(GUARD_PATH) != EXPECTED['guard'] or sha_file(SCANNER_PATH) != EXPECTED['scanner']:
            raise ValueError('guard/scanner source changed after pin validation')
        guard.ROOT = ROOT
        guard.OUT = out
        guard.target_bytes = lambda: scanner.target_bytes(ROOT)
        guard.expected_test_summaries = lambda _index, _stdout: None
        guard.command_stream = lambda path: {
            'path': str(path), 'bytes': path.stat().st_size, 'sha256': sha_file(path)
        }
        command_index = 100

        def command(label: str, argv: list[str], jdk: str) -> tuple[dict, bytes, bytes]:
            nonlocal command_index
            env = os.environ.copy()
            for key in ENV_STRIP:
                env.pop(key, None)
            env.update(guard.ENV_VALUES)
            home = JDKS[jdk]['home']
            env['JAVA_HOME'] = str(home)
            env['PATH'] = str(home / 'bin') + os.pathsep + env.get('PATH', '')
            row = guard.run_command(command_index, [str(v) for v in argv], env)
            row['label'] = label
            stdout_path = out / f'{command_index}.stdout.raw'
            stderr_path = out / f'{command_index}.stderr.raw'
            stdout, stderr = stdout_path.read_bytes(), stderr_path.read_bytes()
            record['commands'].append(row)
            command_index += 1
            save('running')
            if row['exit_code'] != 0 or row['guard_stop'] is not None:
                raise RuntimeError(f'{label} failed or hit a disk guard: exit={row["exit_code"]}, guard={row["guard_stop"]}')
            return row, stdout, stderr

        def javac_argv(jdk: str, source_files: list[Path], classes: Path, classpath: Path, sourcepath: Path) -> list[str]:
            javac = JDKS[jdk]['home'] / 'bin/javac'
            flags = ['-source', '8', '-target', '8'] if jdk == 'javac8' else ['--release', '8']
            argv = [javac, *flags, '-g:none', '-proc:none']
            argv += ['-classpath', classpath, '-sourcepath', sourcepath]
            return [*argv, '-d', classes, *source_files]

        def java_argv(jdk: str, classpath: list[Path]) -> list[str]:
            return [JDKS[jdk]['home'] / 'bin/java', '-Xverify:all', '-cp', os.pathsep.join(map(str, classpath)), 'cf07.Runner']

        # Four full JSON renders: two original class inputs times default/all evidence.
        jarde_sources: dict[tuple[str, str], Path] = {}
        jarde_docs: dict[tuple[str, str], dict] = {}
        for jdk in ('javac8', 'javac23'):
            classfile = ORIGINAL_CLASSES[jdk] / 'cf07/LoopCases.class'
            for evidence in ('default', 'all'):
                argv = [cli, 'class-source', '--input', classfile, '--class', 'cf07.LoopCases',
                        '--policy', 'single-class', '--release', '8', '--format', 'json']
                if evidence == 'all':
                    argv += ['--evidence', 'all']
                render_row, stdout, _ = command(f'jarde-render-{jdk}-{evidence}', argv, jdk)
                doc = json.loads(stdout.decode('utf-8'))
                source = doc.get('text')
                if not isinstance(source, str) or not source:
                    raise ValueError(f'{jdk}/{evidence} report has no complete source text')
                source_path = out / 'jarde-source' / jdk / evidence / 'LoopCases.java'
                source_path.parent.mkdir(parents=True, exist_ok=True)
                source_path.write_bytes(source.encode('utf-8'))
                jarde_sources[(jdk, evidence)] = source_path
                jarde_docs[(jdk, evidence)] = doc
                record.setdefault('renders', []).append({
                    'jdk': jdk, 'evidence': evidence, 'command_index': render_row['index'],
                    'class_input': pin(classfile),
                    'raw_json_stdout_sha256': sha_bytes(stdout), 'source': pin(source_path),
                    'report_outcome': doc.get('outcome'), 'member_count': len(doc.get('methods', [])),
                })
                save('running')
        for jdk in ('javac8', 'javac23'):
            if jarde_docs[(jdk, 'default')].get('text') != jarde_docs[(jdk, 'all')].get('text'):
                raise ValueError(f'{jdk}: default/all full class source differs')

        for jdk in ('javac8', 'javac23'):
            class_input_dir = ORIGINAL_CLASSES[jdk]
            original_class = class_input_dir / 'cf07/LoopCases.class'
            oracle_runner_dir = out / 'build' / jdk / 'original-class-oracle' / 'runner-classes'
            oracle_runner_dir.mkdir(parents=True)
            oracle_empty_path = oracle_runner_dir / 'empty-sourcepath'
            oracle_empty_path.mkdir()
            _, _, _ = command(
                f'compile-original-runner-{jdk}',
                javac_argv(jdk, [RUNNER_SOURCE], oracle_runner_dir, class_input_dir, oracle_empty_path), jdk,
            )
            _, oracle_out, oracle_err = command(
                f'run-original-input-class-{jdk}',
                java_argv(jdk, [oracle_runner_dir, class_input_dir]), jdk,
            )
            oracle = {'stdout': oracle_out, 'stderr': oracle_err}
            oracle_paths = {}
            for stream, data in [('stdout', oracle_out), ('stderr', oracle_err)]:
                path = out / 'oracle' / jdk / f'{stream}.raw'
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(data)
                oracle_paths[stream] = pin(path)
            record.setdefault('oracle_runs', []).append({
                'jdk': jdk, 'class_input': pin(original_class),
                'runner_source': pin(RUNNER_SOURCE), 'runner_classes_dir': str(oracle_runner_dir),
                'generated_runner_class': pin(oracle_runner_dir / 'cf07/Runner.class'),
                'streams': oracle_paths,
            })
            save('running')

            variants = [
                ('original-source', ORIGINAL_SOURCE),
                ('jadx-frozen', JADX_SOURCE),
                ('jarde-default', jarde_sources[(jdk, 'default')]),
                ('jarde-all', jarde_sources[(jdk, 'all')]),
            ]
            for variant, source in variants:
                build_dir = out / 'build' / jdk / variant
                classes = build_dir / 'classes'
                classes.mkdir(parents=True)
                copied_source = build_dir / 'LoopCases.java'
                copied_runner = build_dir / 'Runner.java'
                empty_path = build_dir / 'empty-classpath-sourcepath'
                empty_path.mkdir(parents=True)
                shutil.copyfile(source, copied_source)
                shutil.copyfile(RUNNER_SOURCE, copied_runner)
                _, compile_out, compile_err = command(
                    f'compile-{variant}-{jdk}',
                    javac_argv(jdk, [copied_source, copied_runner], classes, empty_path, empty_path), jdk,
                )
                actual_loop_class = classes / 'cf07/LoopCases.class'
                if not actual_loop_class.is_file():
                    raise FileNotFoundError(f'{variant}/{jdk} compiler produced no cf07/LoopCases.class')
                generated = {'LoopCases.class': pin(actual_loop_class), 'Runner.class': pin(classes / 'cf07/Runner.class')}
                source_class_is_oracle_bytes = (
                    variant == 'original-source' and sha_file(actual_loop_class) == sha_file(original_class)
                )
                _, stdout, stderr = command(
                    f'run-{variant}-{jdk}', java_argv(jdk, [classes]), jdk,
                )
                equal = stdout == oracle_out and stderr == oracle_err
                result = {
                    'jdk': jdk, 'variant': variant, 'source': pin(source),
                    'copied_source': pin(copied_source), 'runner': pin(copied_runner),
                    'compile_stdout_sha256': sha_bytes(compile_out), 'compile_stderr_sha256': sha_bytes(compile_err),
                    'generated_classes_post_compile_observed': generated,
                    'original_source_class_exactly_matches_input_class': source_class_is_oracle_bytes if variant == 'original-source' else None,
                    'runtime_stdout_sha256': sha_bytes(stdout), 'runtime_stderr_sha256': sha_bytes(stderr),
                    'runtime_stdout_bytes': len(stdout), 'runtime_stderr_bytes': len(stderr),
                    'matches_original_class_oracle': equal,
                }
                record['source_variants'].append(result)
                record['runtime_comparisons'].append({
                    'jdk': jdk, 'variant': variant, 'stdout_equal': stdout == oracle_out,
                    'stderr_equal': stderr == oracle_err, 'equal': equal,
                })
                save('running')
                if not equal:
                    raise AssertionError(f'{variant}/{jdk} runtime streams differ from the original class oracle')
        # Recheck all preflight inputs, tools, CLI, receipt, and this runner after execution.
        for name, original_pin in record['inputs'].items():
            current = pin(Path(original_pin['path']))
            if current != original_pin:
                raise ValueError(f'input changed during replay: {name}: {current}')
        for name, original_pin in record['tools'].items():
            current = pin(Path(original_pin['path']))
            if current != original_pin:
                raise ValueError(f'tool changed during replay: {name}: {current}')
        current_cli = pin(cli)
        if (current_cli['sha256'] != record['cli']['sha256'] or current_cli['bytes'] != record['cli']['bytes']
                or stat.S_IMODE(cli.stat().st_mode) != 0o555):
            raise ValueError('frozen CLI changed during replay')
        current_receipt = pin(receipt_path)
        if current_receipt != record['cli']['build_receipt']:
            raise ValueError('build receipt changed during replay')
        current_runner = pin(Path(__file__).resolve())
        if current_runner != record['runner']:
            raise ValueError('replay runner changed during execution')
        record['end_pins_rechecked'] = {
            'inputs': len(record['inputs']), 'tools': len(record['tools']),
            'cli': current_cli, 'build_receipt': current_receipt, 'runner': current_runner,
        }
        record['scope_note'] = 'JADX source is the exact previously accepted JADX 1.5.6 source; this run reuses its frozen source and does not invoke JADX or claim fresh decompilation.'
        save('passed_all_sources_both_jdks_against_original_class_oracles')
        print(json.dumps({'status': record['status'], 'commands': len(record['commands']), 'variants': len(record['source_variants'])}, indent=2))
        return 0
    except Exception as exc:
        record['failure'] = f'{type(exc).__name__}: {exc}'
        save('failed')
        raise

if __name__ == '__main__':
    raise SystemExit(main())
