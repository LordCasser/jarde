#!/usr/bin/env python3
"""Root-only, serial full-source and runtime comparator for finThrow / full EX."""
from __future__ import annotations

import argparse
import datetime
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import sys
import subprocess

ROOT = Path('/Users/lordcasser/workspace/projects/jarde').resolve()
DEFAULT_JDK_MANIFEST = ROOT / 'openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json'
DEFAULT_JDK_MANIFEST_SHA256 = 'ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec'
GUARD_PATH = ROOT / 'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
SCANNER_PATH = ROOT / 'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
GUARD_SHA256 = '51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
SCANNER_SHA256 = 'd954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
EXPECTED_EX_SHA256 = 'e6f0eb1e029a2c883d3ee5df5e089eda17fea27833a9244297cede139cd61287'
STRIPPED_ENV = ('JAVA_TOOL_OPTIONS', '_JAVA_OPTIONS', 'JDK_JAVA_OPTIONS', 'CLASSPATH', 'CARGO_TARGET_DIR')
EXPECTED_SHAPE = {'code_length_bytes': 38, 'start_bci': 0, 'end_bci': 17, 'handler_bci': 27, 'catch_type': 'any'}


def sha_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            digest.update(chunk)
    return digest.hexdigest()


def pin(path: Path) -> dict:
    path = path.resolve(strict=True)
    return {'path': str(path), 'bytes': path.stat().st_size, 'sha256': sha_file(path)}


def write_json(path: Path, value) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')


def load_module(path: Path, name: str):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f'cannot import guard module {path}')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def cli_receipt_binding(document, cli: Path) -> str:
    records = []
    if isinstance(document, dict):
        if isinstance(document.get('cli'), dict):
            records.append(document['cli'])
        if 'cli_path' in document or 'cli_sha256' in document:
            records.append({'path': document.get('cli_path'), 'sha256': document.get('cli_sha256')})
    matches = []
    for record in records:
        path = record.get('path')
        digest = record.get('sha256')
        if not isinstance(path, str) or not isinstance(digest, str):
            continue
        try:
            if Path(path).resolve() == cli:
                matches.append(digest)
        except OSError:
            continue
    if len(set(matches)) != 1:
        raise RuntimeError('CLI receipt must bind exact cli.path and cli.sha256 to --cli')
    return matches[0]


def configured_jdks(manifest_path: Path, expected_manifest_sha: str) -> tuple[dict, dict]:
    if sha_file(manifest_path) != expected_manifest_sha:
        raise RuntimeError(f'JDK manifest SHA-256 mismatch: {manifest_path}')
    manifest = json.loads(manifest_path.read_text(encoding='utf-8'))
    result = {}
    for leg_name, directory in (('javac8', 'javac8'), ('javac23', 'javac23')):
        leg = next((row for row in manifest.get('legs', []) if row.get('leg') == leg_name), None)
        if leg is None:
            raise RuntimeError(f'JDK manifest has no {leg_name} leg')
        tools = {}
        for tool in ('java', 'javac', 'javap'):
            record = leg.get('jdk_tools', {}).get(tool, {})
            path = Path(record.get('path', '')).resolve(strict=True)
            expected = record.get('sha256')
            if not expected or sha_file(path) != expected:
                raise RuntimeError(f'{leg_name} {tool} does not match its manifest pin: {path}')
            tools[tool] = path
        home = tools['java'].parent.parent
        if any(path.parent != home / 'bin' for path in tools.values()):
            raise RuntimeError(f'{leg_name} tools do not share one JAVA_HOME/bin')
        result[directory] = {'name': leg_name, 'home': home, 'tools': tools}
    return result, {'path': str(manifest_path.resolve()), 'sha256': expected_manifest_sha,
                    'legs': {key: {name: pin(path) for name, path in value['tools'].items()}
                             for key, value in result.items()}}


def environment(base: dict[str, str], jdk: dict | None = None) -> dict[str, str]:
    env = {key: value for key, value in base.items() if key not in STRIPPED_ENV}
    if jdk is not None:
        env['JAVA_HOME'] = str(jdk['home'])
        env['PATH'] = str(jdk['home'] / 'bin') + os.pathsep + env.get('PATH', '')
    return env


def target_shape(text: str, descriptor: str = '(I)Ljava/lang/String;') -> dict:
    lines = text.splitlines()
    descriptor_line = next((i for i, line in enumerate(lines)
                            if line.strip() == f'descriptor: {descriptor}'), None)
    if descriptor_line is None:
        return {'descriptor_found': False, 'instructions': [], 'rows': []}
    end = next((i for i in range(descriptor_line + 1, len(lines))
                if lines[i].strip().startswith('descriptor: ')), len(lines))
    block = lines[descriptor_line:end]
    instructions = []
    in_code = False
    rows = []
    in_exceptions = False
    for line in block:
        if line.strip() == 'Code:':
            in_code = True
        elif in_code and line.strip().startswith('Exception table:'):
            in_exceptions = True
        elif in_exceptions and line.strip().startswith('StackMapTable:'):
            in_exceptions = False
        match = re.match(r'^\s*(\d+):\s+([a-zA-Z0-9_]+)(?:\s+(.*))?$', line)
        if in_code and match:
            instructions.append({'bci': int(match.group(1)), 'opcode': match.group(2),
                                 'operand': (match.group(3) or '').strip()})
        if in_exceptions:
            row = re.match(r'^\s*(\d+)\s+(\d+)\s+(\d+)\s+(any|Class\s+.*)$', line)
            if row:
                rows.append({'start_bci': int(row.group(1)), 'end_bci': int(row.group(2)),
                             'handler_bci': int(row.group(3)),
                             'catch_type': 'any' if row.group(4) == 'any' else row.group(4)})
    last = instructions[-1] if instructions else None
    length = last['bci'] + 1 if last and last['opcode'] in {'athrow', 'areturn', 'return', 'ireturn', 'lreturn', 'freturn', 'dreturn', 'nop', 'aconst_null', 'iconst_m1', 'iconst_0', 'iconst_1', 'iconst_2', 'iconst_3', 'iconst_4', 'iconst_5', 'iload_0', 'iload_1', 'iload_2', 'iload_3', 'aload_0', 'aload_1', 'aload_2', 'aload_3', 'pop', 'dup', 'swap', 'arraylength', 'monitorenter', 'monitorexit'} else None
    return {'descriptor_found': True, 'instructions': instructions, 'rows': rows,
            'last_instruction': last, 'code_length_if_final_opcode_is_one_byte': length,
            'matches_parent_38b_single_row': length == 38 and rows == [
                {'start_bci': 0, 'end_bci': 17, 'handler_bci': 27, 'catch_type': 'any'}]}


def source_inputs(root: Path, guard) -> dict[str, str]:
    raw = subprocess.check_output(['git', 'ls-files', '--cached', '--others', '--exclude-standard', '-z', '--', 'src', 'crates', 'tests', 'Cargo.toml', 'Cargo.lock'], cwd=root)
    paths = sorted(set(raw.split(b'\0')) - {b''})
    return {os.fsdecode(p): sha_file(root / os.fsdecode(p)) for p in paths if (root / os.fsdecode(p)).is_file()}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', required=True, type=Path)
    parser.add_argument('--cli', required=True, type=Path)
    parser.add_argument('--cli-receipt', required=True, type=Path)
    parser.add_argument('--jadx', required=True, type=Path)
    parser.add_argument('--jadx-jar', required=True, type=Path)
    parser.add_argument('--jadx-sha256', required=True)
    parser.add_argument('--jadx-jar-sha256', required=True)
    parser.add_argument('--jdk-manifest', type=Path, default=DEFAULT_JDK_MANIFEST)
    parser.add_argument('--jdk-manifest-sha256', default=DEFAULT_JDK_MANIFEST_SHA256)
    parser.add_argument('--branch-source', type=Path, default=ROOT / 'tests/fixtures/recover-branch-throw-return-finally/BranchThrowFinally.java')
    parser.add_argument('--driver-source', type=Path, default=ROOT / 'tests/fixtures/recover-branch-throw-return-finally/BranchThrowFinallyDriver.java')
    parser.add_argument('--ex-source', type=Path, default=ROOT / 'openspec/evidence/java-syntax-2026-10-05/exception-shape-family-patrol/fixture/EX.java')
    parser.add_argument('--javac8-class', type=Path, default=ROOT / 'tests/fixtures/recover-branch-throw-return-finally/v8-javac8/BranchThrowFinally.class')
    parser.add_argument('--javac23-class', type=Path, default=ROOT / 'tests/fixtures/recover-branch-throw-return-finally/v8/BranchThrowFinally.class')
    parser.add_argument('--ex-class', type=Path, default=ROOT / 'openspec/evidence/java-syntax-2026-10-05/exception-shape-family-patrol/fixture/EX.class')
    parser.add_argument('--expect', choices=('recovered', 'observed'), required=True,
                        help='recovered requires branch generated sources to compile and match; observed preserves baseline failures without requiring success')
    args = parser.parse_args()

    out = args.out.resolve()
    if out.exists():
        raise SystemExit(f'refusing to overwrite comparator evidence: {out}')
    if not ROOT.is_dir():
        raise SystemExit(f'repository root missing: {ROOT}')
    if not args.ex_class.is_file() or sha_file(args.ex_class) != EXPECTED_EX_SHA256:
        raise SystemExit('sealed EX.class path/SHA-256 does not match the pinned evidence class')
    for path in (args.cli, args.cli_receipt, args.jadx, args.jadx_jar, args.jdk_manifest,
                 args.branch_source, args.driver_source, args.ex_source,
                 args.javac8_class, args.javac23_class, args.ex_class):
        if not path.is_file():
            raise SystemExit(f'required comparator input is missing: {path}')
    args.cli = args.cli.resolve(strict=True)
    args.cli_receipt = args.cli_receipt.resolve(strict=True)
    args.jadx = args.jadx.resolve(strict=True)
    args.jdk_manifest = args.jdk_manifest.resolve(strict=True)
    receipt = json.loads(args.cli_receipt.read_text(encoding='utf-8'))
    if receipt.get('status') != 'commands_passed_pending_independent_acceptance':
        raise SystemExit('CLI receipt is not the pinned successful build receipt')
    receipt_cli_sha = cli_receipt_binding(receipt, args.cli)
    if sha_file(args.cli) != receipt_cli_sha:
        raise SystemExit('CLI SHA-256 differs from the CLI receipt')
    if sha_file(args.jadx) != args.jadx_sha256.lower():
        raise SystemExit('JADX executable SHA-256 differs from --jadx-sha256')
    if sha_file(args.jadx_jar) != args.jadx_jar_sha256.lower():
        raise SystemExit('JADX JAR SHA-256 differs from --jadx-jar-sha256')

    if (ROOT / 'target').exists() or (ROOT / 'target').is_symlink():
        raise SystemExit('refusing comparator run while repository target exists; root requires exact-clean no-Rust state')
    out.mkdir(parents=True)
    raw = out / 'raw'
    raw.mkdir()
    if sha_file(GUARD_PATH) != GUARD_SHA256 or sha_file(SCANNER_PATH) != SCANNER_SHA256:
        raise SystemExit('durable root guard/scanner SHA-256 mismatch')
    guard = load_module(GUARD_PATH, 'durable_root_guard')
    scanner = load_module(SCANNER_PATH, 'durable_target_scanner')
    guard.ROOT = ROOT
    guard.OUT = raw
    guard.target_bytes = lambda: scanner.target_bytes(ROOT)
    guard.command_stream = lambda path: pin(path)
    guard.expected_test_summaries = lambda index, stdout_bytes: None
    guard_path = GUARD_PATH
    jdks, jdk_record = configured_jdks(args.jdk_manifest, args.jdk_manifest_sha256)
    if shutil.disk_usage(ROOT).free < 5 * 1024**3:
        raise SystemExit('5 GiB free-space preflight failed')
    if guard.target_bytes() > 1024**3:
        raise SystemExit('1 GiB target-size preflight failed')

    repo_pins_before = source_inputs(ROOT, guard)
    pinned_paths = (
        Path(__file__).resolve(), args.branch_source, args.driver_source, args.ex_source, args.javac8_class,
        args.javac23_class, args.ex_class, args.cli, args.cli_receipt, args.jadx, args.jadx_jar,
        args.jdk_manifest, guard_path, SCANNER_PATH,
    )
    jdk_tool_paths = tuple(path for leg in jdks.values() for path in leg['tools'].values())
    static_inputs = {str(p.resolve()): pin(p) for p in (*pinned_paths, *jdk_tool_paths)}
    jdk_tools_before = {str(path): pin(path) for leg in jdks.values() for path in leg['tools'].values()}
    env_base = environment(dict(os.environ))
    rows = []
    results = {'schema': 'recover-branch-throw-return-finally-comparison-v1',
               'status': 'running', 'expectation': args.expect, 'root': str(ROOT),
               'guard_runner': pin(guard_path), 'cli_receipt': pin(args.cli_receipt),
               'cli': pin(args.cli), 'jdk_manifest': jdk_record,
               'jadx': pin(args.jadx), 'jadx_profile': 'rename-flags none; config none; no-res; one thread', 'inputs_before': static_inputs,
               'source_pins_before': repo_pins_before, 'commands': rows,
               'matrix': [], 'ex_controls': [], 'notes': [
                   'Commands run serially; each is polled once per second with 5 GiB free and 1 GiB target guards.',
                   'Generated source is compiled byte-for-byte as emitted; no source repairs are applied.',
                   'EX generated-source compilation failures are retained because the complete class has other red methods.',
               ]}
    write_json(out / 'execution.json', results)
    process_env = {key: value for key, value in env_base.items()}

    def command(label: str, argv: list[str], jdk: dict | None = None, phase: str = '') -> dict:
        env = environment(process_env, jdk)
        row = guard.run_command(len(rows) + 1, argv, env)
        row['label'] = label
        row['phase'] = phase
        rows.append(row)
        write_json(out / 'execution.json', results)
        if row.get('guard_stop') is not None:
            raise RuntimeError(f'guard_stop on command {row["index"]}: {row["guard_stop"]}')
        return row

    def stream(row: dict, channel: str) -> bytes:
        path = Path(row['streams'][channel]['path'])
        return path.read_bytes()

    def fresh_dir(path: Path) -> Path:
        if path.exists():
            raise RuntimeError(f'refusing non-clean output directory: {path}')
        path.mkdir(parents=True)
        return path

    def compile_java(source: Path, class_name: str, compiler: str, output: Path,
                     classpath: list[Path], label: str, phase: str) -> dict:
        fresh_dir(output)
        empty = output.parent / (output.name + '-empty-cp')
        fresh_dir(empty)
        cp = os.pathsep.join(str(p) for p in classpath) if classpath else str(empty)
        javac = jdks[compiler]['tools']['javac']
        if compiler == 'javac8':
            flags = ['-source', '8', '-target', '8', '-g:none', '-Xlint:-options', '-proc:none']
        else:
            flags = ['--release', '8', '-g:none', '-proc:none']
        return command(label, [str(javac), *flags, '-classpath', cp, '-sourcepath', str(empty),
                               '-d', str(output), str(source)], jdks[compiler], phase)

    def javap(class_file: Path, compiler: str, label: str, phase: str) -> tuple[dict, dict]:
        row = command(label, [str(jdks[compiler]['tools']['javap']), '-p', '-c', '-s', '-v', str(class_file)], jdks[compiler], phase)
        return row, target_shape(stream(row, 'stdout').decode('utf-8', errors='replace'))

    def source_cell(input_name: str, class_file: Path, class_name: str, evidence: str) -> tuple[dict, Path | None]:
        argv = [str(args.cli), 'class-source', '--input', str(class_file), '--class', class_name,
                '--policy', 'single-class', '--release', '8', '--format', 'json']
        if evidence == 'all':
            argv.extend(['--evidence', 'all'])
        row = command(f'jarde-{input_name}-{evidence}', argv, phase='class-source')
        target = out / 'sources' / input_name / f'{evidence}.json'
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(stream(row, 'stdout'))
        if row['exit_code'] != 0 or row['guard_stop'] is not None:
            return row, None
        try:
            document = json.loads(target.read_text(encoding='utf-8'))
            text = document['text']
            if not isinstance(text, str):
                raise TypeError('JSON class-source text is not a string')
        except Exception as error:
            row['json_error'] = f'{type(error).__name__}: {error}'
            write_json(out / 'execution.json', results)
            return row, None
        ext_dir = out / 'sources' / input_name / evidence
        ext_dir.mkdir(parents=True, exist_ok=True)
        ext = ext_dir / f'{class_name}.java'
        ext.write_bytes(text.encode('utf-8'))
        row['json_artifact'] = pin(target)
        row['source_artifact'] = pin(ext)
        row['source_text_sha256'] = sha_file(ext)
        row['source_text_has_bytecode_fallback'] = '@bytecode' in text
        write_json(out / 'execution.json', results)
        return row, ext

    def run_java(label: str, classpath: list[Path], main: str, runtime: str, phase: str) -> dict:
        cp = os.pathsep.join(str(p) for p in classpath)
        return command(label, [str(jdks[runtime]['tools']['java']), '-Xverify:all', '-cp', cp, main],
                       jdks[runtime], phase)

    try:
        # Pin and validate JADX 1.5.6 once, then generate source once for each committed input class.
        version = command('jadx-version', [str(args.jadx), '--version'], jdks['javac23'], 'tool-version')
        version_text = (stream(version, 'stdout') + stream(version, 'stderr')).decode('utf-8', errors='replace')
        results['jadx_version_text'] = version_text
        results['jadx_version_verified'] = '1.5.6' in version_text
        if not results['jadx_version_verified']:
            raise RuntimeError(f'JADX version output does not identify 1.5.6: {version_text!r}')

        inputs = [
            ('javac8', args.javac8_class.resolve(), 'BranchThrowFinally', args.branch_source.resolve()),
            ('javac23', args.javac23_class.resolve(), 'BranchThrowFinally', args.branch_source.resolve()),
            ('sealed-ex', args.ex_class.resolve(), 'EX', args.ex_source.resolve()),
        ]
        jadx_records = {}
        for name, class_file, class_name, _ in inputs:
            jadx_out = out / 'jadx' / name
            jadx_out.parent.mkdir(parents=True, exist_ok=True)
            if jadx_out.exists():
                raise RuntimeError(f'refusing non-clean JADX output directory: {jadx_out}')
            argv = [str(args.jadx), '--config', 'none', '--threads-count', '1', '--no-res', '--rename-flags', 'none',
                    '-d', str(jadx_out), str(class_file)]
            row = command(f'jadx-source-{name}', argv, jdks['javac23'], 'jadx-source')
            jadx_record = {'input': name, 'command': row['index'],
                           'exit_code': row['exit_code'], 'output_dir': str(jadx_out),
                           'compiles': [], 'runtime_comparisons': []}
            if row['exit_code'] == 0 and jadx_out.is_dir():
                candidates = list(jadx_out.rglob(f'{class_name}.java'))
                matching = []
                class_pattern = re.compile(r'\bclass\s+' + re.escape(class_name) + r'\b')
                for candidate in candidates:
                    source_text = candidate.read_text(encoding='utf-8', errors='replace')
                    if class_pattern.search(source_text):
                        matching.append((candidate, source_text))
                if len(matching) == 1:
                    emitted, emitted_text = matching[0]
                    package_match = re.search(r'^\s*package\s+([A-Za-z_$][\w$]*(?:\.[A-Za-z_$][\w$]*)*)\s*;',
                                              emitted_text, re.MULTILINE)
                    package_name = package_match.group(1) if package_match else ''
                    fqcn = f'{package_name}.{class_name}' if package_name else class_name
                    jadx_record.update({'source': pin(emitted), 'declared_class': fqcn,
                                        'package': package_name})
                else:
                    jadx_record['source_error'] = f'expected exactly one {class_name}.java declaration; found {len(matching)}'
            jadx_records[name] = jadx_record
            results.setdefault('jadx_runs', []).append(jadx_record)
            write_json(out / 'execution.json', results)

        # Source-compiler origins: compile the unchanged fixture with its own pinned javac and
        # compare its method Code/handler facts to the two committed target class inputs.
        origin = {}
        for input_leg, class_file, _, source in inputs[:2]:
            compiled = out / 'origin' / input_leg / 'classes'
            row = compile_java(source, 'BranchThrowFinally', input_leg, compiled, [],
                               f'compile-original-branch-{input_leg}', 'original-source-compile')
            compiled_class = compiled / 'BranchThrowFinally.class'
            original_shape = None
            if row['exit_code'] == 0 and compiled_class.is_file():
                jrow, original_shape = javap(compiled_class, input_leg,
                                             f'javap-compiled-original-{input_leg}', 'original-shape')
            input_jrow, input_shape = javap(class_file, input_leg,
                                             f'javap-input-{input_leg}', 'input-shape')
            origin[input_leg] = {'input_class': pin(class_file), 'input_javap_command': input_jrow['index'],
                                 'input_shape': input_shape, 'source_compile_command': row['index'],
                                 'compiled_class': pin(compiled_class) if compiled_class.is_file() else None,
                                 'compiled_shape': original_shape,
                                 'matches_expected_parent_shape': input_shape.get('matches_parent_38b_single_row')}
            if row['exit_code'] != 0 or not compiled_class.is_file():
                raise RuntimeError(f'unchanged BranchThrowFinally source failed its {input_leg} origin compile')
            if not input_shape.get('matches_parent_38b_single_row') or not (original_shape or {}).get('matches_parent_38b_single_row'):
                raise RuntimeError(f'{input_leg} input/original source did not retain Code 38B and catch-all [0,17)->27')
        for leg, record in origin.items():
            if not record['compiled_class'] or record['compiled_class']['sha256'] != record['input_class']['sha256']:
                raise RuntimeError('original fixture did not reproduce exact input bytes: '+leg)
        results['origins'] = origin
        write_json(out / 'execution.json', results)

        # The driver is compiled into its own directory against each original class. The original
        # class is first on the baseline class path; rebuilt directories replace it at runtime.
        baselines = {}
        driver_dirs = {}
        for input_leg, class_file, _, _ in inputs[:2]:
            original_dir = out / 'original' / input_leg / 'classes'
            fresh_dir(original_dir)
            shutil.copyfile(class_file, original_dir / 'BranchThrowFinally.class')
            driver_dir = out / 'original' / input_leg / 'driver'
            driver_compile = compile_java(args.driver_source.resolve(), 'BranchThrowFinallyDriver', input_leg,
                                          driver_dir, [original_dir], f'compile-driver-{input_leg}', 'driver-compile')
            driver_dirs[input_leg] = driver_dir
            baselines[input_leg] = {}
            for runtime in ('javac8', 'javac23'):
                baseline = run_java(f'run-original-{input_leg}-{runtime}', [original_dir, driver_dir],
                                    'BranchThrowFinallyDriver', runtime, 'original-runtime')
                if baseline['exit_code'] != 0:
                    raise RuntimeError(f'original {input_leg} driver baseline failed on {runtime}')
                baselines[input_leg][runtime] = {'exit_code': baseline['exit_code'],
                                                  'stdout_sha256': baseline['streams']['stdout']['sha256'],
                                                  'stderr_sha256': baseline['streams']['stderr']['sha256'],
                                                  'command': baseline['index']}
            if driver_compile['exit_code'] != 0 or not (driver_dir / 'BranchThrowFinallyDriver.class').is_file():
                raise RuntimeError(f'driver failed to compile against original {input_leg} class')
        results['original_runtime_oracles'] = baselines
        write_json(out / 'execution.json', results)

        # Compile each untouched JADX emission with both compilers. If the source retains the
        # default-package class, run it under both runtimes with the same unchanged driver.
        for name, class_file, class_name, _ in inputs:
            record = jadx_records[name]
            if 'source' not in record:
                continue
            emitted = Path(record['source']['path'])
            package_name = record['package']
            fqcn = record['declared_class']
            for build_leg in ('javac8', 'javac23'):
                output = out / 'jadx-compiled' / name / build_leg / 'classes'
                compile_row = compile_java(emitted, class_name, build_leg, output, [],
                                           f'compile-jadx-{name}-with-{build_leg}', 'jadx-compile')
                expected_class = output.joinpath(*fqcn.split('.')).with_suffix('.class')
                compile_record = {'compiler': build_leg, 'command': compile_row['index'],
                                  'exit_code': compile_row['exit_code'],
                                  'class': pin(expected_class) if expected_class.is_file() else None,
                                  'runtime_comparable': False, 'runtime_comparisons': []}
                if (name in ('javac8', 'javac23') and compile_row['exit_code'] == 0
                        and expected_class.is_file()):
                    if not package_name and class_name == 'BranchThrowFinally':
                        for runtime in ('javac8', 'javac23'):
                            run = run_java(f'run-jadx-{name}-{build_leg}-{runtime}',
                                           [output, driver_dirs[name]],
                                           'BranchThrowFinallyDriver', runtime, 'jadx-runtime')
                            baseline = baselines[name][runtime]
                            compared = {'runtime': runtime, 'command': run['index'],
                                        'exit_matches_original': run['exit_code'] == baseline['exit_code'],
                                        'stdout_matches_original': run['streams']['stdout']['sha256'] == baseline['stdout_sha256'],
                                        'stderr_matches_original': run['streams']['stderr']['sha256'] == baseline['stderr_sha256'],
                                        'all_match_original': run['exit_code'] == baseline['exit_code']
                                            and run['streams']['stdout']['sha256'] == baseline['stdout_sha256']
                                            and run['streams']['stderr']['sha256'] == baseline['stderr_sha256']}
                            compile_record['runtime_comparisons'].append(compared)
                            record['runtime_comparisons'].append(compared)
                        compile_record['runtime_comparable'] = True
                    else:
                        compile_record['runtime_note'] = 'no unchanged default-package driver can link this emitted class/package'
                record['compiles'].append(compile_record)
                write_json(out / 'execution.json', results)

        # Jarde full source default/all for both BranchThrowFinally compiler inputs. Compile every
        # emitted source with both pinned javacs into a newly created, empty target directory.
        for input_leg, class_file, class_name, _ in inputs[:2]:
            evidence_texts = {}
            for evidence in ('default', 'all'):
                src_row, generated = source_cell(input_leg, class_file, class_name, evidence)
                cell = {'input_compiler': input_leg, 'evidence': evidence,
                        'class_source_command': src_row['index'], 'source': pin(generated) if generated else None,
                        'source_text_sha256': src_row.get('source_text_sha256'),
                        'compiles': [], 'runtime_comparisons': []}
                if generated is None:
                    results['matrix'].append(cell)
                    write_json(out / 'execution.json', results)
                    if args.expect == 'recovered':
                        raise RuntimeError(f'{input_leg}/{evidence} class-source did not produce JSON source')
                    continue
                evidence_texts[evidence] = generated.read_bytes()
                if args.expect == 'recovered' and src_row.get('source_text_has_bytecode_fallback'):
                    raise RuntimeError(f'{input_leg}/{evidence} source still contains @bytecode fallback')
                for build_leg in ('javac8', 'javac23'):
                    rebuilt = out / 'rebuilt' / input_leg / evidence / build_leg / 'classes'
                    compile_row = compile_java(generated, class_name, build_leg, rebuilt, [],
                                               f'compile-{input_leg}-{evidence}-with-{build_leg}', 'rebuild-compile')
                    rebuilt_class = rebuilt / f'{class_name}.class'
                    rebuilt_record = {'compiler': build_leg, 'command': compile_row['index'],
                                      'exit_code': compile_row['exit_code'],
                                      'class': pin(rebuilt_class) if rebuilt_class.is_file() else None,
                                      'shape': None}
                    if compile_row['exit_code'] == 0 and rebuilt_class.is_file():
                        _, rebuilt_record['shape'] = javap(rebuilt_class, build_leg,
                                                           f'javap-rebuilt-{input_leg}-{evidence}-{build_leg}', 'rebuilt-shape')
                        for runtime in ('javac8', 'javac23'):
                            run = run_java(f'run-rebuilt-{input_leg}-{evidence}-{build_leg}-{runtime}',
                                           [rebuilt, driver_dirs[input_leg]], 'BranchThrowFinallyDriver',
                                           runtime, 'rebuilt-runtime')
                            baseline = baselines[input_leg][runtime]
                            compared = {
                                'runtime': runtime, 'command': run['index'],
                                'exit_matches_original': run['exit_code'] == baseline['exit_code'],
                                'stdout_matches_original': run['streams']['stdout']['sha256'] == baseline['stdout_sha256'],
                                'stderr_matches_original': run['streams']['stderr']['sha256'] == baseline['stderr_sha256'],
                                'all_match_original': run['exit_code'] == baseline['exit_code']
                                    and run['streams']['stdout']['sha256'] == baseline['stdout_sha256']
                                    and run['streams']['stderr']['sha256'] == baseline['stderr_sha256'],
                            }
                            rebuilt_record.setdefault('runtime_comparisons', []).append(compared)
                            cell['runtime_comparisons'].append(compared)
                    cell['compiles'].append(rebuilt_record)
                    write_json(out / 'execution.json', results)
                    if args.expect == 'recovered':
                        if compile_row['exit_code'] != 0 or not rebuilt_class.is_file():
                            raise RuntimeError(f'{input_leg}/{evidence}/{build_leg} recovered source did not compile')
                        if any(not row['all_match_original'] for row in rebuilt_record.get('runtime_comparisons', [])):
                            raise RuntimeError(f'{input_leg}/{evidence}/{build_leg} runtime diverged from same-runtime original')
                if args.expect == 'recovered' and len(cell['runtime_comparisons']) != 4:
                    raise RuntimeError(f'{input_leg}/{evidence} requires exactly four runtime comparisons')
                results['matrix'].append(cell)
                write_json(out / 'execution.json', results)
            same_text = evidence_texts.get('default') == evidence_texts.get('all') if len(evidence_texts) == 2 else False
            results['matrix'][-2]['default_all_text_matches'] = same_text
            results['matrix'][-1]['default_all_text_matches'] = same_text
            write_json(out / 'execution.json', results)
            if args.expect == 'recovered' and not same_text:
                raise RuntimeError(f'{input_leg} default/all evidence changed recovered Java text')

        # Full sealed EX is a separate red control. Preserve default/all JSON, compile its unchanged
        # original source, and attempt compilation of both untouched generated full-class texts.
        original_ex = out / 'ex-control' / 'original-source'
        for build_leg in ('javac8', 'javac23'):
            row = compile_java(args.ex_source.resolve(), 'EX', build_leg, original_ex / build_leg,
                               [], f'compile-original-sealed-ex-{build_leg}', 'original-ex-compile')
            if row['exit_code'] != 0 or not (original_ex / build_leg / 'EX.class').is_file():
                raise RuntimeError(f'unchanged sealed EX.java failed its {build_leg} control compile')
            compiled_ex = original_ex / build_leg / 'EX.class'
            ex_source_javap, compiled_ex_shape = javap(
                compiled_ex, build_leg, f'javap-compiled-original-ex-{build_leg}', 'ex-original-shape')
            if not compiled_ex_shape.get('matches_parent_38b_single_row'):
                raise RuntimeError(f'unchanged EX.java target method shape differed under {build_leg}')
            results['ex_controls'].append({'kind': 'original-source-compile', 'compiler': build_leg,
                                           'command': row['index'], 'exit_code': row['exit_code'],
                                           'javap_command': ex_source_javap['index'],
                                           'shape': compiled_ex_shape})
        ex_javap, ex_shape = javap(args.ex_class.resolve(), 'javac23', 'javap-sealed-ex-target', 'ex-input-shape')
        results['ex_controls'].append({'kind': 'sealed-class-shape', 'command': ex_javap['index'],
                                       'shape': ex_shape,
                                       'matches_expected_parent_shape': ex_shape.get('matches_parent_38b_single_row')})
        if not ex_shape.get('matches_parent_38b_single_row'):
            raise RuntimeError('sealed EX finThrow shape did not match Code 38B and catch-all [0,17)->27')
        for evidence in ('default', 'all'):
            src_row, generated = source_cell('sealed-ex', args.ex_class.resolve(), 'EX', evidence)
            control = {'kind': 'generated-full-class', 'evidence': evidence,
                       'class_source_command': src_row['index'],
                       'source': pin(generated) if generated else None, 'compiles': []}
            if generated is not None:
                for build_leg in ('javac8', 'javac23'):
                    output = out / 'ex-control' / evidence / build_leg / 'classes'
                    row = compile_java(generated, 'EX', build_leg, output, [],
                                       f'compile-generated-ex-{evidence}-{build_leg}', 'red-ex-compile')
                    control['compiles'].append({'compiler': build_leg, 'command': row['index'],
                                                'exit_code': row['exit_code'],
                                                'compiled_class': pin(output / 'EX.class') if (output / 'EX.class').is_file() else None})
            results['ex_controls'].append(control)
            write_json(out / 'execution.json', results)

        results['status'] = 'complete_pending_review'
    except BaseException as error:
        results['status'] = 'failed'
        results['error'] = f'{type(error).__name__}: {error}'
    finally:
        results['jdk_tools_before'] = jdk_tools_before
        results['jdk_tools_after'] = {str(path): pin(path) for leg in jdks.values() for path in leg['tools'].values()}
        results['jdk_tools_unchanged'] = results['jdk_tools_before'] == results['jdk_tools_after']
        results['source_pins_after'] = source_inputs(ROOT, guard)
        results['source_pins_unchanged'] = results['source_pins_before'] == results['source_pins_after']
        after_inputs = {str(p.resolve()): pin(p) for p in (*pinned_paths, *jdk_tool_paths)}
        results['inputs_after'] = after_inputs
        results['inputs_and_tools_unchanged'] = static_inputs == after_inputs
        results['repository_target_absent_after'] = not (ROOT / 'target').exists() and not (ROOT / 'target').is_symlink()
        if (not results['jdk_tools_unchanged'] or not results['source_pins_unchanged'] or not results['inputs_and_tools_unchanged']
                or not results['repository_target_absent_after']):
            results['status'] = 'failed'
            results['pin_error'] = 'source, tool, CLI receipt, or JDK manifest changed during comparison'
        results['finished_utc'] = datetime.datetime.now(datetime.timezone.utc).isoformat()
        write_json(out / 'execution.json', results)
        summary = {
            'status': results['status'], 'expectation': args.expect,
            'commands': len(rows), 'source_pins_unchanged': results.get('source_pins_unchanged'),
            'inputs_and_tools_unchanged': results.get('inputs_and_tools_unchanged'),
            'repository_target_absent_after': results.get('repository_target_absent_after'),
            'matrix_cells': len(results['matrix']), 'ex_controls': len(results['ex_controls']),
            'error': results.get('error'),
        }
        write_json(out / 'report.json', summary)
        (out / 'README.md').write_text(
            '# Branch throw / return / finally comparison\n\n'
            'See `execution.json` for every guarded command, raw stream pin, compiler/runtime matrix, '
            'JADX source capture, full EX red control, and before/after source/tool pins. '
            f'Expectation mode: `{args.expect}`. Overall status: `{results["status"]}`.\n',
            encoding='utf-8')
        print(json.dumps(summary, ensure_ascii=False))
    return 0 if results['status'] == 'complete_pending_review' else 1


if __name__ == '__main__':
    raise SystemExit(main())
