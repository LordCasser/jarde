#!/usr/bin/env python3
"""Serial baseline/final evidence collector for exception-only finally completion.

This script is a root-run draft. Invoke with `python -B`; it records every external
command's raw stdout/stderr through the pinned root guard and never edits source inputs.
"""
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
import subprocess
import sys

BASELINE_CLI = Path('/private/tmp/jarde-nested-local-gates-root-v3-20261017/candidate-cli')
BASELINE_CLI_SHA256 = '4811eea2d7f52d4c1b818c86acc1d961c91e3d7fe9d5adaa2c53131f84140f8e'
DEFAULT_JDK_MANIFEST_REL = 'openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json'
DEFAULT_JDK_MANIFEST_SHA256 = 'ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec'
GUARD_REL = 'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
SCANNER_REL = 'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
GUARD_SHA256 = '51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
SCANNER_SHA256 = 'd954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
SEALED_CLASS_SHA256 = {
    'javac8': '49e649478c3d45ba6486029778a8415239caafdd93234a1d5956f5ccf3031393',
    'javac23': '2b8902d998065d2d1747abaeba386e006cdf5ff7f1311abb28506e8d8dc0cc94',
}
ENV_STRIP = ('JAVA_TOOL_OPTIONS', '_JAVA_OPTIONS', 'JDK_JAVA_OPTIONS', 'CLASSPATH', 'CARGO_TARGET_DIR')
JDK_LEGS = ('javac8', 'javac23')
EVIDENCE = ('default', 'all')
METHODS_EXCEPTION_ONLY = {
    ('<init>', '()V'), ('mark', '(I)I'), ('tryThrowsFinallyRuns', '()I'), ('<clinit>', '()V'),
}
METHODS_SEALED = {
    ('<init>', '()V'), ('mark', '(I)I'), ('normalReturn', '()I'),
    ('finallyReturns', '()I'), ('finallyThrows', '()I'), ('tryThrowsFinallyRuns', '()I'),
    ('<clinit>', '()V'),
}


def sha(path: Path) -> str:
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for part in iter(lambda: stream.read(1024 * 1024), b''):
            h.update(part)
    return h.hexdigest()


def pin(path: Path) -> dict:
    path = path.resolve(strict=True)
    return {'path': str(path), 'bytes': path.stat().st_size, 'sha256': sha(path)}


def write_json(path: Path, value) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_suffix(path.suffix + '.tmp')
    tmp.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    tmp.replace(path)


def source_closure(root: Path) -> dict[str, dict]:
    result = {}
    candidates = [root / 'Cargo.toml', root / 'Cargo.lock']
    for name in ('src', 'crates', 'tests', 'openspec/changes/recover-exception-only-finally-completion'):
        directory = root / name
        if directory.exists():
            for current, dirs, files in os.walk(directory):
                dirs[:] = sorted(d for d in dirs if d not in {'.git', 'target', '__pycache__'})
                for filename in sorted(files):
                    path = Path(current) / filename
                    if path.is_file() and not path.is_symlink():
                        candidates.append(path)
    for path in sorted(set(candidates)):
        if path.is_file() and not path.is_symlink():
            result[path.relative_to(root).as_posix()] = pin(path)
    return result


def load_module(path: Path, name: str):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f'cannot load helper: {path}')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def cli_receipt_digest(receipt: dict, cli: Path) -> str:
    found = []
    records = []
    if isinstance(receipt.get('cli'), dict):
        records.append(receipt['cli'])
    if 'cli_path' in receipt or 'cli_sha256' in receipt:
        records.append({'path': receipt.get('cli_path'), 'sha256': receipt.get('cli_sha256')})
    for row in records:
        if isinstance(row.get('path'), str) and isinstance(row.get('sha256'), str):
            try:
                if Path(row['path']).resolve() == cli:
                    found.append(row['sha256'].lower())
            except OSError:
                pass
    if len(set(found)) != 1:
        raise RuntimeError('final CLI receipt must bind the exact CLI path and digest')
    return found[0]


def configured_jdks(manifest_path: Path, expected_sha: str) -> tuple[dict, dict]:
    if sha(manifest_path) != expected_sha:
        raise RuntimeError(f'JDK manifest pin mismatch: {manifest_path}')
    manifest = json.loads(manifest_path.read_text(encoding='utf-8'))
    result = {}
    records = {}
    for leg in JDK_LEGS:
        item = next((row for row in manifest.get('legs', []) if row.get('leg') == leg), None)
        if item is None:
            raise RuntimeError(f'manifest has no {leg}')
        tools = {}
        for name in ('java', 'javac', 'javap'):
            record = item.get('jdk_tools', {}).get(name, {})
            path = Path(record.get('path', '')).resolve(strict=True)
            if not record.get('sha256') or sha(path) != record['sha256']:
                raise RuntimeError(f'{leg} {name} differs from manifest pin: {path}')
            tools[name] = path
        home = tools['java'].parent.parent
        if any(path.parent != home / 'bin' for path in tools.values()):
            raise RuntimeError(f'{leg} tools do not share JAVA_HOME/bin')
        result[leg] = {'home': home, 'tools': tools}
        records[leg] = {name: pin(path) for name, path in tools.items()}
    return result, {'path': str(manifest_path.resolve()), 'sha256': expected_sha, 'tools': records}


def clean_env(base: dict[str, str], jdk: dict | None = None) -> dict[str, str]:
    env = {key: value for key, value in base.items() if key not in ENV_STRIP}
    if jdk:
        env['JAVA_HOME'] = str(jdk['home'])
        env['PATH'] = str(jdk['home'] / 'bin') + os.pathsep + env.get('PATH', '')
    return env


def method_key(method: dict) -> tuple[str, str]:
    identity = method['item']['identity']
    return (bytes(identity['name']).decode('utf-8'), bytes(identity['descriptor']).decode('utf-8'))


def parse_class(path: Path) -> dict:
    data = path.read_bytes(); pos = 0
    def take(n: int) -> bytes:
        nonlocal pos
        b = data[pos:pos+n]
        if len(b) != n: raise ValueError(f'truncated class file {path}')
        pos += n; return b
    def u1() -> int: return take(1)[0]
    def u2() -> int: return int.from_bytes(take(2), 'big')
    def u4() -> int: return int.from_bytes(take(4), 'big')
    if u4() != 0xCAFEBABE: raise ValueError(f'not a class file: {path}')
    minor, major = u2(), u2(); count = u2(); cp = [None] * count; i = 1
    while i < count:
        tag = u1()
        if tag == 1:
            n = u2(); cp[i] = ('utf8', take(n).decode('utf-8', errors='replace'))
        elif tag in (3, 4): take(4)
        elif tag in (5, 6): take(8); i += 1
        elif tag in (7, 8, 16, 19, 20): cp[i] = (tag, u2())
        elif tag in (9, 10, 11, 12, 17, 18): cp[i] = (tag, u2(), u2())
        elif tag == 15: take(3)
        else: raise ValueError(f'unknown CP tag {tag} in {path}')
        i += 1
    def utf(idx: int) -> str:
        entry = cp[idx]
        if not entry or entry[0] != 'utf8': raise ValueError(f'bad UTF8 CP index {idx}')
        return entry[1]
    def attrs() -> dict:
        out = {}
        for _ in range(u2()):
            name = utf(u2()); size = u4(); out[name] = take(size)
        return out
    access = u2(); this_idx = u2(); super_idx = u2()
    for _ in range(u2()): u2()
    fields = []
    for _ in range(u2()):
        flags = u2(); name = utf(u2()); desc = utf(u2()); attrs(); fields.append((name, desc, flags))
    methods = []
    for ordinal in range(u2()):
        flags = u2(); name = utf(u2()); desc = utf(u2()); at = attrs(); code = None
        if 'Code' in at:
            blob = at['Code']; q = 0
            def r2() -> int:
                nonlocal q
                v = int.from_bytes(blob[q:q+2], 'big'); q += 2; return v
            def r4() -> int:
                nonlocal q
                v = int.from_bytes(blob[q:q+4], 'big'); q += 4; return v
            max_stack = r2(); max_locals = r2(); ncode = r4(); bcode = blob[q:q+ncode]; q += ncode
            rows = []
            for _ in range(r2()): rows.append([r2(), r2(), r2(), r2()])
            code = {'max_stack': max_stack, 'max_locals': max_locals, 'code_length': ncode,
                    'code_hex': bcode.hex(), 'rows': rows, 'bcis': instruction_bcis(bcode)}
        methods.append({'index': ordinal, 'name': name, 'descriptor': desc, 'access': flags,
                        'code': code})
    class_attrs = attrs()
    return {'path': str(path.resolve()), 'bytes': len(data), 'sha256': sha(path),
            'minor': minor, 'major': major, 'class_name': cp[cp[this_idx][1]][1],
            'fields': fields, 'methods': methods, 'attributes': sorted(class_attrs)}


def instruction_bcis(code: bytes) -> list[int]:
    fixed = {0x10:2,0x11:3,0x12:2,0x13:3,0x14:3,0x15:2,0x16:2,0x17:2,0x18:2,0x19:2,
             0x36:2,0x37:2,0x38:2,0x39:2,0x3a:2,0x84:3,0xa9:2,0xbc:2,0xc5:4,
             0xb2:3,0xb3:3,0xb4:3,0xb5:3,0xb6:3,0xb7:3,0xb8:3,0xbb:3,0xbd:3,0xc0:3,0xc1:3,
             0xb9:5,0xba:5,0xc8:5,0xc9:5}
    fixed.update({op:3 for op in range(0x99,0xa9)}); fixed.update({0xc6:3,0xc7:3})
    bcis=[]; pc=0
    while pc < len(code):
        bcis.append(pc); op=code[pc]
        if op == 0xaa:
            aligned=(pc+4)&~3; low=int.from_bytes(code[aligned+4:aligned+8],'big',signed=True); high=int.from_bytes(code[aligned+8:aligned+12],'big',signed=True)
            if high < low: raise ValueError(f'invalid tableswitch at {pc}')
            size=(aligned-pc)+12+4*(high-low+1)
        elif op == 0xab:
            aligned=(pc+4)&~3; pairs=int.from_bytes(code[aligned+4:aligned+8],'big',signed=True)
            if pairs < 0: raise ValueError(f'invalid lookupswitch at {pc}')
            size=(aligned-pc)+8+8*pairs
        elif op == 0xc4:
            if pc+1 >= len(code): raise ValueError('truncated wide')
            size=6 if code[pc+1] == 0x84 else 4
        else: size=fixed.get(op, 1)
        if pc+size > len(code): raise ValueError(f'instruction overruns Code at {pc}')
        pc += size
    return bcis


def class_methods(facts: dict) -> list[tuple[str, str]]:
    return [(row['name'], row['descriptor']) for row in facts['methods']]


def command_artifact(row: dict, channel: str) -> bytes:
    rec = row['streams'][channel]; path = Path(rec['path'])
    if not path.is_absolute(): raise ValueError(f'raw stream path must be absolute: {path}')
    raw = path.read_bytes()
    if len(raw) != rec['bytes'] or sha(path) != rec['sha256']:
        raise ValueError(f'raw stream digest changed: command {row["index"]} {channel}')
    return raw


def valid_nine_stdout(data: bytes, label: str) -> None:
    lines = data.decode('utf-8', errors='strict').splitlines()
    kinds = ('none', 'runtime', 'error')
    expected = []
    for try_kind in kinds:
        for cleanup_kind in kinds:
            if cleanup_kind == 'runtime': typ, msg = 'java.lang.IllegalStateException', 'cleanup-runtime'
            elif cleanup_kind == 'error': typ, msg = 'java.lang.AssertionError', 'cleanup-error'
            elif try_kind == 'runtime': typ, msg = 'java.lang.IllegalStateException', 'try-runtime'
            elif try_kind == 'error': typ, msg = 'java.lang.AssertionError', 'try-error'
            else: typ, msg = 'java.lang.IllegalArgumentException', 'try'
            expected.append(f'{try_kind}/{cleanup_kind}:events=78:identity=true:type={typ}:message={msg}')
    if lines != expected:
        raise ValueError(f'{label}: output is not the exact 9-case oracle\nexpected={expected!r}\nactual={lines!r}')


def missing_return_count(stderr: bytes, source: Path) -> tuple[int, list[dict]]:
    text = stderr.decode('utf-8', errors='replace')
    declarations = []
    for number, line in enumerate(source.read_text(encoding='utf-8').splitlines(), 1):
        found = re.search(r'\b(?:int|void)\s+([\w$]+)\s*\([^;]*\)\s*\{', line)
        if found:
            declarations.append((number, found.group(1)))
    diagnostics = []
    for raw in text.splitlines():
        if 'missing return statement' not in raw and '缺少返回语句' not in raw:
            continue
        match = re.search(r':(\d+):', raw)
        source_line = int(match.group(1)) if match else None
        preceding = [row for row in declarations if source_line is not None and row[0] <= source_line]
        diagnostics.append({'text': raw.strip(), 'source_line': source_line,
                            'method': preceding[-1][1] if preceding else None})
    return len(diagnostics), diagnostics


def expected_methods(path: Path, expected: set[tuple[str, str]]) -> dict:
    facts = parse_class(path)
    actual = set(class_methods(facts))
    if actual != expected:
        raise ValueError(f'{path}: method identity set mismatch: expected {sorted(expected)}, got {sorted(actual)}')
    for item in facts['methods']:
        if item['code'] is None:
            raise ValueError(f'{path}: method lacks Code: {item["name"]}{item["descriptor"]}')
    return facts


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--root', type=Path, required=True)
    ap.add_argument('--out', type=Path, required=True)
    ap.add_argument('--mode', choices=('baseline', 'final'), required=True)
    ap.add_argument('--cli', type=Path, required=True)
    ap.add_argument('--cli-receipt', type=Path)
    ap.add_argument('--jdk-manifest', type=Path)
    ap.add_argument('--jdk-manifest-sha256', default=DEFAULT_JDK_MANIFEST_SHA256)
    ap.add_argument('--guard', type=Path)
    ap.add_argument('--scanner', type=Path)
    ap.add_argument('--fixture-source', type=Path, required=True)
    ap.add_argument('--fixture-driver', type=Path, required=True)
    ap.add_argument('--input-javac8-class', type=Path, required=True)
    ap.add_argument('--input-javac23-class', type=Path, required=True)
    ap.add_argument('--sealed-javac8-class', type=Path, required=True)
    ap.add_argument('--sealed-javac23-class', type=Path, required=True)
    ap.add_argument('--sealed-source', type=Path, required=True)
    ap.add_argument('--sealed-runner', type=Path, required=True)
    ap.add_argument('--jadx', type=Path, required=True)
    ap.add_argument('--jadx-sha256', required=True)
    ap.add_argument('--jadx-jar', type=Path, required=True)
    ap.add_argument('--jadx-jar-sha256', required=True)
    args = ap.parse_args()

    root = args.root.resolve(strict=True); out = args.out.resolve()
    if out.exists(): raise SystemExit(f'refusing to overwrite output: {out}')
    if (root / 'target').exists() or (root / 'target').is_symlink():
        raise SystemExit('refusing to run while repository target exists')
    for p in (args.cli, args.fixture_source, args.fixture_driver, args.input_javac8_class,
              args.input_javac23_class, args.sealed_javac8_class, args.sealed_javac23_class,
              args.sealed_source, args.sealed_runner, args.jadx, args.jadx_jar):
        if not p.is_file(): raise SystemExit(f'missing input: {p}')
    cli = args.cli.resolve(strict=True)
    if args.mode == 'baseline':
        if cli != BASELINE_CLI or sha(cli) != BASELINE_CLI_SHA256:
            raise SystemExit('baseline CLI path/digest does not match the frozen 4811eea2 pin')
        cli_receipt = None
    else:
        if args.cli_receipt is None or not args.cli_receipt.is_file():
            raise SystemExit('final mode requires the built CLI receipt')
        cli_receipt = args.cli_receipt.resolve(strict=True)
        receipt = json.loads(cli_receipt.read_text(encoding='utf-8'))
        if receipt.get('status') != 'commands_passed_pending_independent_acceptance':
            raise SystemExit('final CLI receipt is not a successful pending-acceptance receipt')
        digest = cli_receipt_digest(receipt, cli)
        if sha(cli) != digest: raise SystemExit('final CLI differs from receipt-bound digest')
    if sha(args.jadx) != args.jadx_sha256.lower() or sha(args.jadx_jar) != args.jadx_jar_sha256.lower():
        raise SystemExit('JADX executable/JAR digest differs from supplied nested-matrix pins')
    if sha(args.sealed_javac8_class) != SEALED_CLASS_SHA256['javac8'] or sha(args.sealed_javac23_class) != SEALED_CLASS_SHA256['javac23']:
        raise SystemExit('sealed FinallyCompletion class bytes differ from handoff pins')

    manifest = args.jdk_manifest or root / DEFAULT_JDK_MANIFEST_REL
    manifest = manifest.resolve(strict=True)
    jdks, jdk_record = configured_jdks(manifest, args.jdk_manifest_sha256)
    guard_path = args.guard or root / GUARD_REL; scanner_path = args.scanner or root / SCANNER_REL
    guard_path = guard_path.resolve(strict=True); scanner_path = scanner_path.resolve(strict=True)
    if sha(guard_path) != GUARD_SHA256 or sha(scanner_path) != SCANNER_SHA256:
        raise SystemExit('root guard/scanner digest differs from nested matrix pins')
    out.mkdir(parents=True); raw_dir = out / 'raw'; raw_dir.mkdir()
    guard = load_module(guard_path, 'exception_finally_guard')
    scanner = load_module(scanner_path, 'exception_finally_scanner')
    guard.ROOT = root; guard.OUT = raw_dir
    guard.target_bytes = lambda: scanner.target_bytes(root)
    guard.command_stream = lambda path: pin(path)
    guard.expected_test_summaries = lambda index, stdout_bytes: None
    if shutil.disk_usage(root).free < 5 * 1024**3:
        raise SystemExit('5 GiB free-space guard failed before run')
    source_before = source_closure(root)
    inputs = {
        'source': pin(args.fixture_source), 'driver': pin(args.fixture_driver),
        'javac8_input': pin(args.input_javac8_class), 'javac23_input': pin(args.input_javac23_class),
        'sealed_javac8': pin(args.sealed_javac8_class), 'sealed_javac23': pin(args.sealed_javac23_class),
        'sealed_source': pin(args.sealed_source), 'sealed_runner': pin(args.sealed_runner),
        'cli': pin(cli), 'jadx': pin(args.jadx), 'jadx_jar': pin(args.jadx_jar),
        'jdk_manifest': pin(manifest), 'guard': pin(guard_path), 'scanner': pin(scanner_path),
    }
    jdks_paths = [path for leg in jdks.values() for path in leg['tools'].values()]
    inputs['jdk_tools'] = {str(p): pin(p) for p in jdks_paths}
    inputs['runner'] = pin(Path(__file__))
    rows = []
    result = {'schema': 'exception-only-finally-comparison-v1', 'status': 'running',
              'mode': args.mode, 'expectation': 'recovered' if args.mode == 'final' else 'observed',
              'root': str(root), 'output': str(out), 'inputs_before': inputs,
              'source_pins_before': source_before, 'cli_receipt': pin(cli_receipt) if cli_receipt else None,
              'jdk_manifest_record': jdk_record,
              'jadx_profile': 'JADX 1.5.6; rename-flags none; config none; no-res; one thread',
              'commands': rows, 'jarde_cells': [], 'original_oracles': [],
              'rebuilt_runs': [], 'jadx_runs': [], 'sealed_controls': [],
              'notes': ['Commands run serially with root run_command raw stream and disk/target guards.',
                        'Generated Java is compiled exactly as emitted; no source repair or method removal.',
                        'Observed baseline compile failures retain stderr and have no runtime row.']}
    write_json(out / 'execution.json', result)
    env_base = clean_env(dict(os.environ))

    def cmd(label: str, argv: list[str], compiler: str | None = None, phase: str = '') -> dict:
        env = clean_env(env_base, jdks[compiler] if compiler else None)
        row = guard.run_command(len(rows) + 1, argv, env)
        row.update({'label': label, 'phase': phase})
        rows.append(row); write_json(out / 'execution.json', result)
        if row.get('guard_stop') is not None:
            raise RuntimeError(f'guard stop at {label}: {row["guard_stop"]}')
        return row

    def stdout(row: dict) -> bytes: return command_artifact(row, 'stdout')
    def fresh_dir(path: Path) -> None:
        if path.exists(): raise RuntimeError(f'refusing existing output path: {path}')
        path.mkdir(parents=True)

    def compile_java(source: Path, leg: str, output: Path, cp: list[Path], label: str) -> dict:
        output.parent.mkdir(parents=True, exist_ok=True)
        fresh_dir(output)
        empty = output.parent / (output.name + '-empty-sourcepath'); fresh_dir(empty)
        cp_value = os.pathsep.join(str(x) for x in cp) if cp else str(empty)
        flags = (['-source', '8', '-target', '8', '-g:none', '-Xlint:-options', '-proc:none']
                 if leg == 'javac8' else ['--release', '8', '-g:none', '-proc:none'])
        return cmd(label, [str(jdks[leg]['tools']['javac']), *flags, '-classpath', cp_value,
                           '-sourcepath', str(empty), '-d', str(output), str(source)], leg, 'javac')

    def run_java(label: str, target_root: Path, driver_dir: Path, main: str, runtime: str, phase: str) -> dict:
        cp = os.pathsep.join((str(target_root), str(driver_dir)))
        return cmd(label, [str(jdks[runtime]['tools']['java']), '-Xverify:all', '-cp', cp, main], runtime, phase)

    def run_cell(class_path: Path, driver_source: Path, class_name: str, main: str,
                 leg: str, label: str, phase: str, oracle: bytes | None = None,
                 require_recovered: bool = False, target_root: Path | None = None,
                 package: str = '') -> dict:
        target_dir = target_root or class_path.parent
        driver_input = driver_source
        main_fqn = f'{package}.{main}' if package else main
        if package:
            if re.search(r'^\s*package\s+', driver_source.read_text(encoding='utf-8'), re.M):
                raise RuntimeError(f'{label}: fixture driver already declares a package')
            driver_input = out / 'adapted-drivers' / label / driver_source.name
            driver_input.parent.mkdir(parents=True, exist_ok=True)
            driver_input.write_text(f'package {package};\n\n' + driver_source.read_text(encoding='utf-8'), encoding='utf-8')
        driver_dir = out / 'driver-classes' / label
        driver_row = compile_java(driver_input, leg, driver_dir, [target_dir], f'{label}-driver-compile')
        driver_class = driver_dir / ((package.replace('.', '/') + '/' if package else '') + f'{main}.class')
        item = {'label': label, 'driver_compile_command': driver_row['index'],
                'driver_class': pin(driver_class) if driver_class.is_file() else None,
                'driver_source': pin(driver_input), 'main_fqn': main_fqn,
                'target_class_root': str(target_dir), 'target_package': package,
                'runs': [], 'compile_success': driver_row['exit_code'] == 0 and driver_class.is_file()}
        if not item['compile_success']:
            if require_recovered: raise RuntimeError(f'{label}: driver compilation failed')
            return item
        for runtime in JDK_LEGS:
            run = run_java(f'{label}-{runtime}', target_dir, driver_dir, main_fqn, runtime, phase)
            match = False
            if run['exit_code'] == 0:
                try:
                    valid_nine_stdout(stdout(run), f'{label}/{runtime}')
                    match = oracle is None or stdout(run) == oracle
                except Exception as e:
                    run['oracle_validation_error'] = f'{type(e).__name__}: {e}'
            rec = {'runtime': runtime, 'command': run['index'], 'exit_code': run['exit_code'],
                   'stdout_sha256': run['streams']['stdout']['sha256'],
                   'stderr_sha256': run['streams']['stderr']['sha256'], 'nine_cases_valid': match,
                   'matches_original_oracle': bool(oracle is not None and stdout(run) == oracle)}
            item['runs'].append(rec)
            if require_recovered and (run['exit_code'] != 0 or not match):
                raise RuntimeError(f'{label}/{runtime}: expected recovered runtime to match the original oracle')
        return item

    try:
        # Seal the input class files and prove they are fresh javac outputs of the supplied source.
        source_facts = {}
        for leg, inp in (('javac8', args.input_javac8_class.resolve()), ('javac23', args.input_javac23_class.resolve())):
            facts = expected_methods(inp, METHODS_EXCEPTION_ONLY)
            source_facts[leg] = facts
            compiled_dir = out / 'source-origin' / leg
            original = compile_java(args.fixture_source.resolve(), leg, compiled_dir, [], f'origin-source-{leg}')
            produced = compiled_dir / 'ExceptionOnlyFinally.class'
            if original['exit_code'] != 0 or not produced.is_file() or sha(produced) != sha(inp):
                raise RuntimeError(f'{leg}: supplied source does not reproduce its pinned class input')
            result.setdefault('origin_classes', []).append({'leg': leg, 'compile_command': original['index'],
                'compiled_class': pin(produced), 'input_class': pin(inp), 'exact_bytes_match': True,
                'classfile_facts': facts})
            write_json(out / 'execution.json', result)

        # Original compiler inputs: four original JVM runs, each containing nine throwable cases.
        original_bytes = {}
        for leg, inp in (('javac8', args.input_javac8_class.resolve()), ('javac23', args.input_javac23_class.resolve())):
            target_dir = out / 'original' / leg / 'target'; target_dir.parent.mkdir(parents=True, exist_ok=True); fresh_dir(target_dir)
            shutil.copyfile(inp, target_dir / 'ExceptionOnlyFinally.class')
            driver_dir = out / 'original' / leg / 'driver';
            driver_compile = compile_java(args.fixture_driver.resolve(), leg, driver_dir, [target_dir], f'original-{leg}-driver')
            if driver_compile['exit_code'] != 0 or not (driver_dir / 'ExceptionOnlyFinallyDriver.class').is_file():
                raise RuntimeError(f'original {leg}: driver compilation failed')
            original_bytes[leg] = {}
            for runtime in JDK_LEGS:
                run = run_java(f'original-{leg}-{runtime}', target_dir, driver_dir,
                               'ExceptionOnlyFinallyDriver', runtime, 'original-runtime')
                if run['exit_code'] != 0: raise RuntimeError(f'original oracle failed: {leg}/{runtime}')
                data = stdout(run); valid_nine_stdout(data, f'original {leg}/{runtime}')
                original_bytes[leg][runtime] = data
                result['original_oracles'].append({'input_compiler': leg, 'runtime': runtime,
                    'command': run['index'], 'stdout_sha256': run['streams']['stdout']['sha256'],
                    'stderr_sha256': run['streams']['stderr']['sha256'], 'nine_cases_valid': True})
                write_json(out / 'execution.json', result)

        # Jarde full-class JSON for both compiler inputs and both evidence profiles.
        class_inputs = [('exception-only-javac8', 'ExceptionOnlyFinally', args.input_javac8_class.resolve()),
                        ('exception-only-javac23', 'ExceptionOnlyFinally', args.input_javac23_class.resolve()),
                        ('sealed-javac8', 'FinallyCompletion', args.sealed_javac8_class.resolve()),
                        ('sealed-javac23', 'FinallyCompletion', args.sealed_javac23_class.resolve())]
        for label, class_name, class_path in class_inputs:
            for evidence in EVIDENCE:
                argv = [str(cli), 'class-source', '--input', str(class_path), '--class', class_name,
                        '--policy', 'single-class', '--release', '8', '--format', 'json']
                if evidence == 'all': argv.extend(['--evidence', 'all'])
                row = cmd(f'jarde-{label}-{evidence}', argv, phase='jarde-class-source')
                cell = {'label': label, 'class_name': class_name, 'input_class': pin(class_path),
                        'evidence': evidence, 'command': row['index'], 'exit_code': row['exit_code'],
                        'json_artifact': None, 'source_artifact': None, 'compiles': []}
                if row['exit_code'] == 0:
                    raw_json = stdout(row)
                    json_path = out / 'jarde-json' / label / f'{evidence}.json'
                    json_path.parent.mkdir(parents=True, exist_ok=True); json_path.write_bytes(raw_json)
                    doc = json.loads(raw_json.decode('utf-8'))
                    text = doc.get('text')
                    if not isinstance(text, str): raise RuntimeError(f'{label}/{evidence}: missing class source text')
                    source_path = out / 'jarde-source' / label / evidence / f'{class_name}.java'
                    source_path.parent.mkdir(parents=True, exist_ok=True); source_path.write_bytes(text.encode('utf-8'))
                    cell['json_artifact'] = pin(json_path); cell['source_artifact'] = pin(source_path)
                    cell['source_text_sha256'] = sha(source_path)
                    if label.startswith('exception-only-'):
                        cell['input_class_facts'] = expected_methods(class_path, METHODS_EXCEPTION_ONLY)
                        for build_leg in JDK_LEGS:
                            compiled_dir = out / 'jarde-compiled' / label / evidence / build_leg / 'classes'
                            compile_row = compile_java(source_path, build_leg, compiled_dir, [],
                                f'jarde-{label}-{evidence}-{build_leg}-compile')
                            target_class = compiled_dir / 'ExceptionOnlyFinally.class'
                            comp = {'compiler': build_leg, 'command': compile_row['index'],
                                    'exit_code': compile_row['exit_code'],
                                    'class': pin(target_class) if target_class.is_file() else None,
                                    'class_facts': expected_methods(target_class, METHODS_EXCEPTION_ONLY)
                                        if target_class.is_file() else None,
                                    'driver': None, 'runs': []}
                            if compile_row['exit_code'] == 0 and target_class.is_file():
                                pair = run_cell(target_class, args.fixture_driver.resolve(), class_name,
                                    'ExceptionOnlyFinallyDriver', build_leg,
                                    f'jarde-{label}-{evidence}-{build_leg}', 'jarde-rebuilt-runtime',
                                    oracle=original_bytes[label.removeprefix('exception-only-')][build_leg],
                                    require_recovered=(args.mode == 'final'))
                                comp['driver'] = pair['driver_class']; comp['runs'] = pair['runs']
                            elif args.mode == 'final':
                                raise RuntimeError(f'{label}/{evidence}/{build_leg}: expected whole source to compile')
                            cell['compiles'].append(comp)
                    else:
                        for build_leg in JDK_LEGS:
                            compiled_dir = out / 'sealed-compiled' / label / evidence / build_leg / 'classes'
                            compile_row = compile_java(source_path, build_leg, compiled_dir, [],
                                f'jarde-{label}-{evidence}-{build_leg}-compile')
                            count, lines = missing_return_count(command_artifact(compile_row, 'stderr'), source_path)
                            expected_count = 3 if args.mode == 'baseline' else 2
                            comp = {'compiler': build_leg, 'command': compile_row['index'],
                                    'exit_code': compile_row['exit_code'], 'missing_return_count': count,
                                    'missing_return_lines': lines,
                                    'compiled_class': pin(compiled_dir / 'FinallyCompletion.class')
                                        if (compiled_dir / 'FinallyCompletion.class').is_file() else None}
                            if count != expected_count:
                                raise RuntimeError(f'{label}/{evidence}/{build_leg}: expected {expected_count} missing-return diagnostics, saw {count}')
                            expected_failed_methods = ({'finallyReturns', 'finallyThrows', 'tryThrowsFinallyRuns'}
                                                       if args.mode == 'baseline' else {'finallyReturns', 'finallyThrows'})
                            if {row['method'] for row in lines} != expected_failed_methods:
                                raise RuntimeError(f'{label}/{evidence}/{build_leg}: missing-return diagnostics map to '
                                                   f'{sorted({row["method"] for row in lines})}, expected {sorted(expected_failed_methods)}')
                            cell['compiles'].append(comp)
                elif args.mode == 'final':
                    raise RuntimeError(f'{label}/{evidence}: final Jarde class-source failed')
                result['jarde_cells'].append(cell); write_json(out / 'execution.json', result)

        # JADX 1.5.6: preserve source, compiler failures, and runtime outcomes as observations.
        version = cmd('jadx-version', [str(args.jadx), '--version'], 'javac23', 'jadx-version')
        version_text = (stdout(version) + command_artifact(version, 'stderr')).decode('utf-8', errors='replace')
        if '1.5.6' not in version_text: raise RuntimeError(f'JADX version is not 1.5.6: {version_text!r}')
        for leg, inp in (('javac8', args.input_javac8_class.resolve()), ('javac23', args.input_javac23_class.resolve())):
            jadx_dir = out / 'jadx' / leg; jadx_dir.parent.mkdir(parents=True, exist_ok=True)
            if jadx_dir.exists(): raise RuntimeError(f'refusing existing JADX output {jadx_dir}')
            jrow = cmd(f'jadx-source-{leg}', [str(args.jadx), '--config', 'none', '--threads-count', '1',
                '--no-res', '--rename-flags', 'none', '-d', str(jadx_dir), str(inp)], 'javac23', 'jadx-source')
            record = {'input_compiler': leg, 'command': jrow['index'], 'exit_code': jrow['exit_code'],
                      'source': None, 'compiles': []}
            if jrow['exit_code'] == 0:
                hits = [p for p in jadx_dir.rglob('ExceptionOnlyFinally.java') if p.is_file()]
                if len(hits) != 1: record['source_error'] = f'expected one source, found {len(hits)}'
                else:
                    source_path = hits[0]; record['source'] = pin(source_path)
                    package_match = re.search(r'^\s*package\s+([\w.]+)\s*;',
                                              source_path.read_text(encoding='utf-8'), re.M)
                    package = package_match.group(1) if package_match else ''
                    record['package'] = package
                    for build_leg in JDK_LEGS:
                        compiled_dir = out / 'jadx-compiled' / leg / build_leg / 'classes'
                        compile_row = compile_java(source_path, build_leg, compiled_dir, [],
                            f'jadx-{leg}-{build_leg}-compile')
                        target_class = compiled_dir / ((package.replace('.', '/') + '/') if package else '') / 'ExceptionOnlyFinally.class'
                        comp = {'compiler': build_leg, 'command': compile_row['index'],
                                'exit_code': compile_row['exit_code'],
                                'class': pin(target_class) if target_class.is_file() else None,
                                'runs': []}
                        if compile_row['exit_code'] == 0 and target_class.is_file():
                            pair = run_cell(target_class, args.fixture_driver.resolve(), 'ExceptionOnlyFinally',
                                'ExceptionOnlyFinallyDriver', build_leg, f'jadx-{leg}-{build_leg}',
                                'jadx-rebuilt-runtime', oracle=original_bytes[leg][build_leg], require_recovered=False,
                                target_root=compiled_dir, package=package)
                            comp['driver'] = pair['driver_class']; comp['runs'] = pair['runs']
                        record['compiles'].append(comp)
            result['jadx_runs'].append(record); write_json(out / 'execution.json', result)

        # Sealed seven-method FinallyCompletion controls: never remove methods or repair the source.
        sealed_source = args.sealed_source.resolve(); sealed_runner = args.sealed_runner.resolve()
        for leg, inp in (('javac8', args.sealed_javac8_class.resolve()), ('javac23', args.sealed_javac23_class.resolve())):
            facts = expected_methods(inp, METHODS_SEALED)
            for evidence in EVIDENCE:
                cell = next(c for c in result['jarde_cells'] if c['label'] == f'sealed-{leg}' and c['evidence'] == evidence)
                for build_leg in JDK_LEGS:
                    comp = next(c for c in cell['compiles'] if c['compiler'] == build_leg)
                    stderr_row = rows[comp['command']-1]
                    original_count, original_lines = missing_return_count(
                        command_artifact(stderr_row, 'stderr'), Path(cell['source_artifact']['path']))
                    expected_count = 3 if args.mode == 'baseline' else 2
                    if original_count != expected_count:
                        raise RuntimeError(f'sealed {leg}/{evidence}/{build_leg}: expected {expected_count} missing-return diagnostics')
                    expected_failed_methods = ({'finallyReturns', 'finallyThrows', 'tryThrowsFinallyRuns'}
                                               if args.mode == 'baseline' else {'finallyReturns', 'finallyThrows'})
                    if {row['method'] for row in original_lines} != expected_failed_methods:
                        raise RuntimeError(f'sealed {leg}/{evidence}/{build_leg}: diagnostic method mapping mismatch')
                    comp['missing_return_count'] = original_count
                    comp['missing_return_lines'] = original_lines
                    result['sealed_controls'].append({'input_compiler': leg, 'evidence': evidence,
                        'build_compiler': build_leg, 'input_class': pin(inp), 'method_order': class_methods(facts),
                        'source': cell['source_artifact'], 'json': cell['json_artifact'],
                        'compile_command': comp['command'], 'missing_return_count': original_count,
                        'diagnostic_lines': original_lines})
        result['source_pins_after'] = source_closure(root)
        result['source_pins_unchanged'] = result['source_pins_before'] == result['source_pins_after']
        result['inputs_and_tools_unchanged'] = all(
            Path(record['path']).is_file() and pin(Path(record['path'])) == record
            for record in inputs.values() if isinstance(record, dict) and 'path' in record)
        result['inputs_and_tools_unchanged'] = result['inputs_and_tools_unchanged'] and all(
            Path(record['path']).is_file() and pin(Path(record['path'])) == record
            for record in inputs['jdk_tools'].values())
        result['repository_target_absent_after'] = not (root / 'target').exists()
        if not result['source_pins_unchanged'] or not result['repository_target_absent_after']:
            raise RuntimeError('source closure changed or repository target appeared during comparison')
        result['status'] = 'complete_pending_review'; result['finished_utc'] = datetime.datetime.now(datetime.timezone.utc).isoformat()
        write_json(out / 'execution.json', result)
        return 0
    except Exception as exc:
        result['status'] = 'interrupted'; result['failure'] = f'{type(exc).__name__}: {exc}'
        result['source_pins_after'] = source_closure(root)
        result['source_pins_unchanged'] = result['source_pins_before'] == result['source_pins_after']
        result['finished_utc'] = datetime.datetime.now(datetime.timezone.utc).isoformat()
        write_json(out / 'execution.json', result)
        raise


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except Exception as exc:
        raise SystemExit(f'comparison failed: {type(exc).__name__}: {exc}')
